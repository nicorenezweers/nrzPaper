//! Mehrere Konten in einer Organisation: Einladungen per Benutzername (die eingeladene Person
//! nimmt sie in der App an), Rollen ändern, Mitglieder entfernen, selbst austreten und den
//! Besitz übergeben. Regeln:
//! - Besitzer und Admins laden ein, ändern Rollen und entfernen Mitglieder.
//! - Den Besitzer kann niemand entfernen oder herabstufen; nur er selbst übergibt den Besitz
//!   (danach ist er Admin). Admins ernennen oder entfernen keine anderen Admins.
//! - Jedes Mitglied außer dem Besitzer kann austreten.

use crate::db::models::{new_id, now, Invitation, Role};
use crate::error::{AppError, AppResult};
use crate::services::users;
use sqlx::SqlitePool;

/// Einladungen gelten zwei Wochen.
const VALID_SECS: i64 = 14 * 86_400;

fn invite_role(role: Role) -> AppResult<Role> {
    match role {
        Role::Admin | Role::Member => Ok(role),
        Role::Owner => Err(AppError::bad(
            "Besitzer wird man nur durch Übergabe – einladen lässt sich als Mitglied oder Admin.",
        )),
    }
}

async fn role_of(pool: &SqlitePool, org: &str, user: &str) -> AppResult<Option<Role>> {
    Ok(sqlx::query_scalar(
        "SELECT role FROM organization_members WHERE organization_id = ? AND user_id = ?",
    )
    .bind(org)
    .bind(user)
    .fetch_optional(pool)
    .await?)
}

/// Person einladen (Benutzername). Gibt es schon eine offene Einladung, wird sie erneuert.
pub async fn invite(
    pool: &SqlitePool,
    org: &str,
    actor: (&str, Role),
    username: &str,
    role: Role,
) -> AppResult<Invitation> {
    if !actor.1.can_manage() {
        return Err(AppError::forbidden(
            "Nur Besitzer und Admins können einladen.",
        ));
    }
    let role = invite_role(role)?;
    if role == Role::Admin && actor.1 != Role::Owner {
        return Err(AppError::forbidden(
            "Nur der Besitzer kann Admins einladen.",
        ));
    }
    let username = username.trim();
    if username.is_empty() || username.len() > 64 {
        return Err(AppError::bad("Bitte einen Benutzernamen angeben."));
    }
    if let Some(user) = users::by_username(pool, username).await? {
        if role_of(pool, org, &user.id).await?.is_some() {
            return Err(AppError::Conflict(
                "Diese Person ist schon Mitglied der Organisation.".into(),
            ));
        }
    }
    let ts = now();
    sqlx::query(
        "INSERT INTO organization_invitations (id, organization_id, invitee, role, invited_by, created_at, expires_at)
         VALUES (?, ?, ?, ?, ?, ?, ?)
         ON CONFLICT (organization_id, invitee) DO UPDATE SET
           role = excluded.role, invited_by = excluded.invited_by, created_at = excluded.created_at,
           expires_at = excluded.expires_at, accepted_at = NULL",
    )
    .bind(new_id())
    .bind(org)
    .bind(username)
    .bind(role)
    .bind(actor.0)
    .bind(ts)
    .bind(ts + VALID_SECS)
    .execute(pool)
    .await?;
    Ok(sqlx::query_as(
        "SELECT * FROM organization_invitations WHERE organization_id = ? AND invitee = ?",
    )
    .bind(org)
    .bind(username)
    .fetch_one(pool)
    .await?)
}

/// Offene Einladungen einer Organisation.
pub async fn pending(pool: &SqlitePool, org: &str) -> AppResult<Vec<Invitation>> {
    Ok(sqlx::query_as(
        "SELECT * FROM organization_invitations
         WHERE organization_id = ? AND accepted_at IS NULL AND expires_at > ?
         ORDER BY created_at DESC",
    )
    .bind(org)
    .bind(now())
    .fetch_all(pool)
    .await?)
}

pub async fn revoke(pool: &SqlitePool, org: &str, actor: Role, id: &str) -> AppResult<()> {
    if !actor.can_manage() {
        return Err(AppError::forbidden(
            "Nur Besitzer und Admins können Einladungen zurückziehen.",
        ));
    }
    let n = sqlx::query("DELETE FROM organization_invitations WHERE id = ? AND organization_id = ?")
        .bind(id)
        .bind(org)
        .execute(pool)
        .await?
        .rows_affected();
    if n == 0 {
        return Err(AppError::NotFound);
    }
    Ok(())
}

/// Einladung an mich, mit Name der Organisation und wer eingeladen hat.
#[derive(Debug, serde::Serialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct MyInvitation {
    pub id: String,
    pub organization_id: String,
    pub organization_name: String,
    pub role: Role,
    pub invited_by: Option<String>,
    pub expires_at: i64,
}

pub async fn for_user(pool: &SqlitePool, username: &str) -> AppResult<Vec<MyInvitation>> {
    Ok(sqlx::query_as(
        "SELECT i.id, i.organization_id, o.name AS organization_name, i.role,
                u.username AS invited_by, i.expires_at
         FROM organization_invitations i
         JOIN organizations o ON o.id = i.organization_id AND o.deleted_at IS NULL
         LEFT JOIN users u ON u.id = i.invited_by
         WHERE i.invitee = ? AND i.accepted_at IS NULL AND i.expires_at > ?
         ORDER BY i.created_at DESC",
    )
    .bind(username)
    .bind(now())
    .fetch_all(pool)
    .await?)
}

/// Einladung annehmen: Mitglied mit der eingeladenen Rolle. Gibt die Organisation zurück.
pub async fn accept(pool: &SqlitePool, user_id: &str, username: &str, id: &str) -> AppResult<String> {
    let inv = for_user(pool, username)
        .await?
        .into_iter()
        .find(|i| i.id == id)
        .ok_or(AppError::NotFound)?;
    let mut tx = pool.begin().await?;
    sqlx::query(
        "INSERT INTO organization_members (organization_id, user_id, role, created_at) VALUES (?, ?, ?, ?)
         ON CONFLICT DO NOTHING",
    )
    .bind(&inv.organization_id)
    .bind(user_id)
    .bind(inv.role)
    .bind(now())
    .execute(&mut *tx)
    .await?;
    sqlx::query("UPDATE organization_invitations SET accepted_at = ? WHERE id = ?")
        .bind(now())
        .bind(id)
        .execute(&mut *tx)
        .await?;
    tx.commit().await?;
    Ok(inv.organization_id)
}

pub async fn decline(pool: &SqlitePool, username: &str, id: &str) -> AppResult<()> {
    let n = sqlx::query(
        "DELETE FROM organization_invitations WHERE id = ? AND invitee = ? AND accepted_at IS NULL",
    )
    .bind(id)
    .bind(username)
    .execute(pool)
    .await?
    .rows_affected();
    if n == 0 {
        return Err(AppError::NotFound);
    }
    Ok(())
}

/// Rolle ändern. `Owner` heißt: Besitz übergeben (nur der Besitzer; er wird danach Admin).
pub async fn set_role(
    pool: &SqlitePool,
    org: &str,
    actor: (&str, Role),
    target: &str,
    role: Role,
) -> AppResult<()> {
    let current = role_of(pool, org, target).await?.ok_or(AppError::NotFound)?;
    if !actor.1.can_manage() {
        return Err(AppError::forbidden(
            "Nur Besitzer und Admins können Rollen ändern.",
        ));
    }
    if current == Role::Owner {
        return Err(AppError::bad(
            "Der Besitzer behält seine Rolle – er kann den Besitz übergeben.",
        ));
    }
    if actor.1 != Role::Owner && (role != Role::Member || current == Role::Admin) {
        return Err(AppError::forbidden(
            "Nur der Besitzer ernennt Admins oder stuft sie zurück.",
        ));
    }
    let mut tx = pool.begin().await?;
    if role == Role::Owner {
        sqlx::query("UPDATE organization_members SET role = 'admin' WHERE organization_id = ? AND user_id = ?")
            .bind(org)
            .bind(actor.0)
            .execute(&mut *tx)
            .await?;
    }
    sqlx::query("UPDATE organization_members SET role = ? WHERE organization_id = ? AND user_id = ?")
        .bind(role)
        .bind(org)
        .bind(target)
        .execute(&mut *tx)
        .await?;
    tx.commit().await?;
    Ok(())
}

/// Mitglied entfernen – oder selbst austreten (`target == actor`).
pub async fn remove(pool: &SqlitePool, org: &str, actor: (&str, Role), target: &str) -> AppResult<()> {
    let current = role_of(pool, org, target).await?.ok_or(AppError::NotFound)?;
    if current == Role::Owner {
        return Err(AppError::bad(if target == actor.0 {
            "Als Besitzer zuerst den Besitz übergeben (oder die Organisation löschen)."
        } else {
            "Der Besitzer lässt sich nicht entfernen."
        }));
    }
    if target != actor.0 {
        if !actor.1.can_manage() {
            return Err(AppError::forbidden(
                "Nur Besitzer und Admins können Mitglieder entfernen.",
            ));
        }
        if current == Role::Admin && actor.1 != Role::Owner {
            return Err(AppError::forbidden(
                "Nur der Besitzer kann Admins entfernen.",
            ));
        }
    }
    sqlx::query("DELETE FROM organization_members WHERE organization_id = ? AND user_id = ?")
        .bind(org)
        .bind(target)
        .execute(pool)
        .await?;
    Ok(())
}
