//! Tagging-Regeln (wie Papra): Bedingungen auf Name oder erkanntem Text eines Dokuments
//! (enthält, enthält nicht, gleich, beginnt mit, endet mit, regulärer Ausdruck), verknüpft mit
//! „alle“ oder „mindestens eine“. Trifft eine aktive Regel zu, bekommt das Dokument ihre Tags.
//!
//! Angewendet wird, sobald der Text eines Dokuments feststeht (nach der Texterkennung bzw. sofort
//! bei Text-Dateien), und auf Wunsch rückwirkend auf alle vorhandenen Dokumente.

use crate::db::models::{
    new_id, now, ConditionField, ConditionOperator, MatchMode, TaggingRule, TaggingRuleCondition,
};
use crate::error::{AppError, AppResult};
use crate::services::tags;
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;

/// Regel samt Bedingungen und Tags (für API und Oberfläche).
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Rule {
    #[serde(flatten)]
    pub rule: TaggingRule,
    pub conditions: Vec<TaggingRuleCondition>,
    pub tag_ids: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConditionInput {
    pub field: ConditionField,
    pub operator: ConditionOperator,
    pub value: String,
    #[serde(default)]
    pub case_sensitive: bool,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RuleInput {
    pub name: String,
    #[serde(default)]
    pub description: String,
    #[serde(default = "yes")]
    pub enabled: bool,
    #[serde(default = "all")]
    pub match_mode: MatchMode,
    pub conditions: Vec<ConditionInput>,
    pub tag_ids: Vec<String>,
}

fn yes() -> bool {
    true
}

fn all() -> MatchMode {
    MatchMode::All
}

/// Regulärer Ausdruck mit Grössenbegrenzung (Schutz vor teuren Mustern).
fn compile(pattern: &str, case_sensitive: bool) -> Result<regex::Regex, String> {
    regex::RegexBuilder::new(pattern)
        .case_insensitive(!case_sensitive)
        .size_limit(1 << 20)
        .build()
        .map_err(|e| e.to_string())
}

/// Trifft eine Bedingung auf diesen Namen/Text zu?
pub fn condition_matches(c: &TaggingRuleCondition, name: &str, content: &str) -> bool {
    let haystack = match c.field {
        ConditionField::Name => name,
        ConditionField::Content => content,
    };
    if c.operator == ConditionOperator::Regex {
        return compile(&c.value, c.case_sensitive).is_ok_and(|re| re.is_match(haystack));
    }
    let (h, v) = if c.case_sensitive {
        (haystack.to_string(), c.value.clone())
    } else {
        (haystack.to_lowercase(), c.value.to_lowercase())
    };
    match c.operator {
        ConditionOperator::Contains => h.contains(&v),
        ConditionOperator::NotContains => !h.contains(&v),
        ConditionOperator::Equals => h.trim() == v.trim(),
        ConditionOperator::StartsWith => h.starts_with(&v),
        ConditionOperator::EndsWith => h.trim_end().ends_with(&v),
        ConditionOperator::Regex => unreachable!(),
    }
}

pub fn rule_matches(rule: &Rule, name: &str, content: &str) -> bool {
    if rule.conditions.is_empty() {
        return false;
    }
    let mut hits = rule
        .conditions
        .iter()
        .map(|c| condition_matches(c, name, content));
    match rule.rule.match_mode {
        MatchMode::All => hits.all(|h| h),
        MatchMode::Any => hits.any(|h| h),
    }
}

fn validate(input: &RuleInput) -> AppResult<()> {
    let name = input.name.trim();
    if name.is_empty() || name.chars().count() > 100 {
        return Err(AppError::bad("Name der Regel: 1–100 Zeichen."));
    }
    if input.conditions.is_empty() {
        return Err(AppError::bad("Mindestens eine Bedingung angeben."));
    }
    if input.conditions.len() > 20 {
        return Err(AppError::bad("Höchstens 20 Bedingungen je Regel."));
    }
    for c in &input.conditions {
        if c.value.trim().is_empty() {
            return Err(AppError::bad("Jede Bedingung braucht einen Wert."));
        }
        if c.value.chars().count() > 500 {
            return Err(AppError::bad("Werte höchstens 500 Zeichen."));
        }
        if c.operator == ConditionOperator::Regex {
            compile(&c.value, c.case_sensitive)
                .map_err(|e| AppError::bad(format!("Ungültiger regulärer Ausdruck: {e}")))?;
        }
    }
    if input.tag_ids.is_empty() {
        return Err(AppError::bad(
            "Mindestens einen Tag wählen, der vergeben wird.",
        ));
    }
    Ok(())
}

pub async fn list(pool: &SqlitePool, org: &str) -> AppResult<Vec<Rule>> {
    let rules: Vec<TaggingRule> =
        sqlx::query_as("SELECT * FROM tagging_rules WHERE organization_id = ? ORDER BY created_at")
            .bind(org)
            .fetch_all(pool)
            .await?;
    let mut out = Vec::with_capacity(rules.len());
    for rule in rules {
        let conditions = sqlx::query_as(
            "SELECT * FROM tagging_rule_conditions WHERE rule_id = ? ORDER BY rowid",
        )
        .bind(&rule.id)
        .fetch_all(pool)
        .await?;
        let tag_ids =
            sqlx::query_scalar("SELECT tag_id FROM tagging_rule_actions WHERE rule_id = ?")
                .bind(&rule.id)
                .fetch_all(pool)
                .await?;
        out.push(Rule {
            rule,
            conditions,
            tag_ids,
        });
    }
    Ok(out)
}

pub async fn get(pool: &SqlitePool, org: &str, id: &str) -> AppResult<Rule> {
    list(pool, org)
        .await?
        .into_iter()
        .find(|r| r.rule.id == id)
        .ok_or(AppError::NotFound)
}

async fn write_parts(
    tx: &mut sqlx::SqliteConnection,
    id: &str,
    input: &RuleInput,
) -> AppResult<()> {
    sqlx::query("DELETE FROM tagging_rule_conditions WHERE rule_id = ?")
        .bind(id)
        .execute(&mut *tx)
        .await?;
    sqlx::query("DELETE FROM tagging_rule_actions WHERE rule_id = ?")
        .bind(id)
        .execute(&mut *tx)
        .await?;
    for c in &input.conditions {
        sqlx::query("INSERT INTO tagging_rule_conditions (id, rule_id, field, operator, value, case_sensitive) VALUES (?, ?, ?, ?, ?, ?)")
            .bind(new_id())
            .bind(id)
            .bind(c.field)
            .bind(c.operator)
            .bind(c.value.trim())
            .bind(c.case_sensitive)
            .execute(&mut *tx)
            .await?;
    }
    for t in &input.tag_ids {
        sqlx::query("INSERT OR IGNORE INTO tagging_rule_actions (rule_id, tag_id) VALUES (?, ?)")
            .bind(id)
            .bind(t)
            .execute(&mut *tx)
            .await?;
    }
    Ok(())
}

pub async fn create(pool: &SqlitePool, org: &str, input: &RuleInput) -> AppResult<Rule> {
    validate(input)?;
    for t in &input.tag_ids {
        tags::get(pool, org, t).await?;
    }
    let id = new_id();
    let ts = now();
    let mut tx = pool.begin().await?;
    sqlx::query("INSERT INTO tagging_rules (id, organization_id, name, description, enabled, match_mode, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?)")
        .bind(&id)
        .bind(org)
        .bind(input.name.trim())
        .bind(input.description.trim())
        .bind(input.enabled)
        .bind(input.match_mode)
        .bind(ts)
        .bind(ts)
        .execute(&mut *tx)
        .await?;
    write_parts(&mut tx, &id, input).await?;
    tx.commit().await?;
    get(pool, org, &id).await
}

pub async fn update(pool: &SqlitePool, org: &str, id: &str, input: &RuleInput) -> AppResult<Rule> {
    get(pool, org, id).await?;
    validate(input)?;
    for t in &input.tag_ids {
        tags::get(pool, org, t).await?;
    }
    let mut tx = pool.begin().await?;
    sqlx::query("UPDATE tagging_rules SET name = ?, description = ?, enabled = ?, match_mode = ?, updated_at = ? WHERE id = ?")
        .bind(input.name.trim())
        .bind(input.description.trim())
        .bind(input.enabled)
        .bind(input.match_mode)
        .bind(now())
        .bind(id)
        .execute(&mut *tx)
        .await?;
    write_parts(&mut tx, id, input).await?;
    tx.commit().await?;
    get(pool, org, id).await
}

pub async fn delete(pool: &SqlitePool, org: &str, id: &str) -> AppResult<()> {
    get(pool, org, id).await?;
    sqlx::query("DELETE FROM tagging_rules WHERE id = ?")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(())
}

/// Aktive Regeln auf ein Dokument anwenden. Gibt die Zahl neu gesetzter Tags zurück.
pub async fn apply_to_document(
    pool: &SqlitePool,
    org: &str,
    document_id: &str,
) -> AppResult<usize> {
    let rules: Vec<Rule> = list(pool, org)
        .await?
        .into_iter()
        .filter(|r| r.rule.enabled)
        .collect();
    if rules.is_empty() {
        return Ok(0);
    }
    let (name, content): (String, String) =
        sqlx::query_as("SELECT name, content FROM documents WHERE id = ? AND organization_id = ?")
            .bind(document_id)
            .bind(org)
            .fetch_optional(pool)
            .await?
            .ok_or(AppError::NotFound)?;
    let mut added = 0;
    for rule in rules.iter().filter(|r| rule_matches(r, &name, &content)) {
        for tag in &rule.tag_ids {
            if tags::add_to_document(pool, org, document_id, tag, None).await? {
                added += 1;
            }
        }
    }
    Ok(added)
}

/// Eine Regel rückwirkend auf alle Dokumente (ohne Papierkorb) anwenden.
pub async fn apply_to_all(pool: &SqlitePool, org: &str, id: &str) -> AppResult<(usize, usize)> {
    let rule = get(pool, org, id).await?;
    let docs: Vec<(String, String, String)> = sqlx::query_as(
        "SELECT id, name, content FROM documents WHERE organization_id = ? AND deleted_at IS NULL",
    )
    .bind(org)
    .fetch_all(pool)
    .await?;
    let (mut matched, mut added) = (0, 0);
    for (doc, name, content) in docs {
        if rule_matches(&rule, &name, &content) {
            matched += 1;
            for tag in &rule.tag_ids {
                if tags::add_to_document(pool, org, &doc, tag, None).await? {
                    added += 1;
                }
            }
        }
    }
    Ok((matched, added))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cond(
        field: ConditionField,
        operator: ConditionOperator,
        value: &str,
        cs: bool,
    ) -> TaggingRuleCondition {
        TaggingRuleCondition {
            id: String::new(),
            rule_id: String::new(),
            field,
            operator,
            value: value.into(),
            case_sensitive: cs,
        }
    }

    fn rule(mode: MatchMode, conditions: Vec<TaggingRuleCondition>) -> Rule {
        Rule {
            rule: TaggingRule {
                id: String::new(),
                organization_id: String::new(),
                name: "r".into(),
                description: String::new(),
                enabled: true,
                match_mode: mode,
                created_at: 0,
                updated_at: 0,
            },
            conditions,
            tag_ids: vec![],
        }
    }

    #[test]
    fn operators() {
        use ConditionField::*;
        use ConditionOperator::*;
        let (n, c) = ("Rechnung Swisscom.pdf", "Betrag CHF 79.90\nIBAN CH93 0076");
        assert!(condition_matches(
            &cond(Content, Contains, "swisscom", false),
            "x",
            "SWISSCOM AG"
        ));
        assert!(!condition_matches(
            &cond(Content, Contains, "swisscom", true),
            "x",
            "SWISSCOM AG"
        ));
        assert!(condition_matches(
            &cond(Content, NotContains, "mahnung", false),
            n,
            c
        ));
        assert!(condition_matches(
            &cond(Name, StartsWith, "rechnung", false),
            n,
            c
        ));
        assert!(condition_matches(
            &cond(Name, EndsWith, ".PDF", false),
            n,
            c
        ));
        assert!(condition_matches(
            &cond(Name, Equals, "rechnung swisscom.pdf", false),
            n,
            c
        ));
        assert!(condition_matches(
            &cond(Content, Regex, r"CH\d{2}\s?\d{4}", true),
            n,
            c
        ));
        assert!(
            !condition_matches(&cond(Content, Regex, "(", false), n, c),
            "ungültiges Muster trifft nie"
        );
    }

    #[test]
    fn modes() {
        use ConditionField::*;
        use ConditionOperator::*;
        let a = cond(Name, Contains, "rechnung", false);
        let b = cond(Content, Contains, "mahnung", false);
        assert!(!rule_matches(
            &rule(MatchMode::All, vec![a.clone(), b.clone()]),
            "Rechnung",
            "Text"
        ));
        assert!(rule_matches(
            &rule(MatchMode::Any, vec![a, b]),
            "Rechnung",
            "Text"
        ));
        assert!(!rule_matches(
            &rule(MatchMode::Any, vec![]),
            "Rechnung",
            "Text"
        ));
    }
}
