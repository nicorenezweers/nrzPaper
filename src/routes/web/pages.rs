//! Seiten einer Organisation: Dokumentliste mit Suche, Dokument-Detail mit Vorschau,
//! Tags und Papierkorb – samt den Formular-Aktionen (Post/Redirect/Get).

use super::Page;
use crate::auth::guards::CurrentUser;
use crate::config::Config;
use crate::db::models::User;
use crate::error::{AppError, AppResult};
use crate::ocr::OcrQueue;
use crate::search;
use crate::services::{documents, organizations, share_links, tags, users};
use crate::templates::{
    is_busy, render, ActivityRow, DetailPage, DocRow, DocumentsPage, ResultsPartial, Shell,
    TagsPage, TrashPage,
};
use rocket::form::Form;
use rocket::response::Redirect;
use rocket::{get, post, routes, FromForm, Route, State};
use sqlx::SqlitePool;

const PAGE_SIZE: u32 = 25;

pub fn routes() -> Vec<Route> {
    routes![
        list,
        detail,
        rename,
        set_content,
        add_tag,
        remove_tag,
        trash,
        restore,
        delete,
        reprocess,
        create_share,
        revoke_share,
        set_property,
        tags_page,
        create_tag,
        update_tag,
        delete_tag,
        trash_page,
        empty_trash
    ]
}

pub(super) async fn shell(
    pool: &SqlitePool,
    user: User,
    org: &str,
    active: &'static str,
    query: &str,
) -> AppResult<Shell> {
    let (org, role) = organizations::membership(pool, &user.id, org).await?;
    let orgs = organizations::list_for_user(pool, &user.id).await?;
    let review_count = crate::services::inbox::open_count(pool, &org.id).await?;
    Ok(Shell {
        user,
        org,
        role,
        orgs,
        active,
        query: query.to_string(),
        live_search: active.is_empty(),
        review_count,
    })
}

// ------------------------------------------------------------------ Liste & Suche

async fn results(
    pool: &SqlitePool,
    org: &str,
    query: &str,
    page: u32,
) -> AppResult<ResultsPartial> {
    let parsed = search::parse(query);
    let (docs, total) = search::search(pool, org, &parsed, page, PAGE_SIZE).await?;
    let ids: Vec<String> = docs.iter().map(|d| d.id.clone()).collect();
    let mut tag_map = tags::for_documents(pool, &ids).await?;
    let rows = docs
        .into_iter()
        .map(|doc| DocRow {
            tags: tag_map.remove(&doc.id).unwrap_or_default(),
            doc,
        })
        .collect();
    let any_documents =
        total > 0 || !parsed.is_empty() && documents::list(pool, org, false, 0, 1).await?.1 > 0;
    Ok(ResultsPartial {
        org_id: org.to_string(),
        rows,
        total,
        page,
        pages: ((total as u32).div_ceil(PAGE_SIZE)).max(1),
        query: query.trim().to_string(),
        any_documents,
    })
}

/// Dokumentliste. `?partial=1` liefert nur die Ergebnisse (Live-Suche).
#[get("/organizations/<org>?<q>&<page>&<partial>")]
async fn list(
    user: CurrentUser,
    org: &str,
    q: Option<&str>,
    page: Option<u32>,
    partial: Option<bool>,
    pool: &State<SqlitePool>,
) -> AppResult<Page> {
    let query = q.unwrap_or("").trim();
    let shell = shell(pool, user.0, org, "", query).await?;
    let results = results(pool, org, query, page.unwrap_or(0)).await?;
    if partial.unwrap_or(false) {
        return Ok(Page::Html(render(&results)?));
    }
    let poll = query.is_empty() && results.rows.iter().any(|r| is_busy(&r.doc.ocr_status));
    let tags = tags::list(pool, org).await?;
    let results_html = askama::Template::render(&results)?;
    Ok(Page::Html(render(&DocumentsPage {
        shell,
        results_html,
        tags,
        poll,
    })?))
}

// ------------------------------------------------------------------ Detail

fn activity_label(event: &str, data: &serde_json::Value) -> String {
    let tag = data["tag"].as_str().unwrap_or("");
    match event {
        "created" => "Hochgeladen".into(),
        "renamed" => format!("Umbenannt in „{}“", data["to"].as_str().unwrap_or("")),
        "content_updated" => "Text bearbeitet".into(),
        "tag_added" => format!("Tag „{tag}“ hinzugefügt"),
        "tag_removed" => format!("Tag „{tag}“ entfernt"),
        "deleted" => "In den Papierkorb gelegt".into(),
        "restored" => "Wiederhergestellt".into(),
        "shared" => "Freigabelink erstellt".into(),
        "reviewed" => match data["model"].as_str().filter(|m| !m.is_empty()) {
            Some(m) => format!("Gescannt und geprüft (KI: {m})"),
            None => "Gescannt und geprüft".into(),
        },
        "share_revoked" => "Freigabelink widerrufen".into(),
        other => other.to_string(),
    }
}

#[get("/organizations/<org>/documents/<id>?<find>")]
async fn detail(
    user: CurrentUser,
    org: &str,
    id: &str,
    find: Option<&str>,
    pool: &State<SqlitePool>,
) -> AppResult<Page> {
    let page = detail_page(pool, user.0, org, id, find.unwrap_or(""), None).await?;
    Ok(Page::Html(render(&page)?))
}

async fn detail_page(
    pool: &SqlitePool,
    user: User,
    org: &str,
    id: &str,
    find: &str,
    new_share: Option<String>,
) -> AppResult<DetailPage> {
    let shell = shell(pool, user, org, "", "").await?;
    let doc = documents::get(pool, org, id).await?;
    let tags = tags::for_documents(pool, std::slice::from_ref(&doc.id))
        .await?
        .remove(&doc.id)
        .unwrap_or_default();
    let all_tags = tags::list(pool, org)
        .await?
        .into_iter()
        .map(|(t, _)| t)
        .collect();
    let rows: Vec<(String, Option<String>, String, i64)> = sqlx::query_as(
        "SELECT a.event, u.username, a.data, a.created_at FROM document_activity a LEFT JOIN users u ON u.id = a.user_id
         WHERE a.document_id = ? ORDER BY a.created_at DESC, a.id DESC LIMIT 50",
    )
    .bind(&doc.id)
    .fetch_all(pool)
    .await?;
    let activity = rows
        .into_iter()
        .map(|(event, user, data, at)| ActivityRow {
            label: activity_label(&event, &serde_json::from_str(&data).unwrap_or_default()),
            user,
            at,
        })
        .collect();
    let created_by = match &doc.created_by {
        Some(uid) => users::by_id(pool, uid).await?.map(|u| u.username),
        None => None,
    };
    let shares = share_links::list(pool, org, &doc.id).await?;
    let properties = crate::services::properties::for_document(pool, &doc.id).await?;
    Ok(DetailPage {
        shell,
        doc,
        shares,
        new_share,
        tags,
        all_tags,
        activity,
        created_by,
        find: find.to_string(),
        properties,
    })
}

fn back(org: &str, id: &str) -> Redirect {
    Redirect::to(format!("/organizations/{org}/documents/{id}"))
}

#[derive(FromForm)]
struct NameForm<'r> {
    name: &'r str,
}

#[post("/organizations/<org>/documents/<id>/rename", data = "<form>")]
async fn rename(
    user: CurrentUser,
    org: &str,
    id: &str,
    form: Form<NameForm<'_>>,
    pool: &State<SqlitePool>,
) -> AppResult<Redirect> {
    organizations::membership(pool, &user.0.id, org).await?;
    documents::rename(pool, org, id, form.name, Some(&user.0.id)).await?;
    Ok(back(org, id))
}

#[derive(FromForm)]
struct ContentForm<'r> {
    content: &'r str,
}

#[post("/organizations/<org>/documents/<id>/content", data = "<form>")]
async fn set_content(
    user: CurrentUser,
    org: &str,
    id: &str,
    form: Form<ContentForm<'_>>,
    pool: &State<SqlitePool>,
) -> AppResult<Redirect> {
    organizations::membership(pool, &user.0.id, org).await?;
    documents::set_content(pool, org, id, form.content, Some(&user.0.id)).await?;
    Ok(back(org, id))
}

/// Tag per Name hinzufügen; gibt es ihn nicht, wird er angelegt.
#[post("/organizations/<org>/documents/<id>/tags", data = "<form>")]
async fn add_tag(
    user: CurrentUser,
    org: &str,
    id: &str,
    form: Form<NameForm<'_>>,
    pool: &State<SqlitePool>,
) -> AppResult<Redirect> {
    organizations::membership(pool, &user.0.id, org).await?;
    let tag = match tags::by_name(pool, org, form.name).await? {
        Some(t) => t,
        None => tags::create(pool, org, form.name, "", "").await?,
    };
    tags::add_to_document(pool, org, id, &tag.id, Some(&user.0.id)).await?;
    Ok(back(org, id))
}

#[post("/organizations/<org>/documents/<id>/tags/<tag>/remove")]
async fn remove_tag(
    user: CurrentUser,
    org: &str,
    id: &str,
    tag: &str,
    pool: &State<SqlitePool>,
) -> AppResult<Redirect> {
    organizations::membership(pool, &user.0.id, org).await?;
    tags::remove_from_document(pool, org, id, tag, Some(&user.0.id)).await?;
    Ok(back(org, id))
}

#[post("/organizations/<org>/documents/<id>/trash")]
async fn trash(
    user: CurrentUser,
    org: &str,
    id: &str,
    pool: &State<SqlitePool>,
) -> AppResult<Redirect> {
    organizations::membership(pool, &user.0.id, org).await?;
    documents::trash(pool, org, id, Some(&user.0.id)).await?;
    Ok(Redirect::to(format!("/organizations/{org}")))
}

#[post("/organizations/<org>/documents/<id>/restore")]
async fn restore(
    user: CurrentUser,
    org: &str,
    id: &str,
    pool: &State<SqlitePool>,
) -> AppResult<Redirect> {
    organizations::membership(pool, &user.0.id, org).await?;
    documents::restore(pool, org, id, Some(&user.0.id)).await?;
    Ok(back(org, id))
}

#[post("/organizations/<org>/documents/<id>/delete")]
async fn delete(
    user: CurrentUser,
    org: &str,
    id: &str,
    pool: &State<SqlitePool>,
) -> AppResult<Redirect> {
    organizations::membership(pool, &user.0.id, org).await?;
    documents::delete_permanently(pool, org, id).await?;
    Ok(Redirect::to(format!("/organizations/{org}/trash")))
}

#[post("/organizations/<org>/documents/<id>/reprocess")]
async fn reprocess(
    user: CurrentUser,
    org: &str,
    id: &str,
    pool: &State<SqlitePool>,
    queue: &State<OcrQueue>,
) -> AppResult<Redirect> {
    organizations::membership(pool, &user.0.id, org).await?;
    documents::reprocess(pool, queue, org, id).await?;
    Ok(back(org, id))
}

// ------------------------------------------------------------------ Eigenschaften

#[derive(FromForm)]
struct ValueForm<'r> {
    #[field(default = "")]
    value: &'r str,
}

/// Eigenschaft ändern (leer = entfernen).
#[post(
    "/organizations/<org>/documents/<id>/properties/<def>",
    data = "<form>"
)]
async fn set_property(
    user: CurrentUser,
    org: &str,
    id: &str,
    def: &str,
    form: Form<ValueForm<'_>>,
    pool: &State<SqlitePool>,
) -> AppResult<Redirect> {
    organizations::membership(pool, &user.0.id, org).await?;
    documents::get(pool, org, id).await?;
    let kind = crate::services::properties::definition(pool, org, def).await?;
    crate::services::properties::set(pool, id, def, kind, form.value).await?;
    Ok(back(org, id))
}

// ------------------------------------------------------------------ Freigabe

#[derive(FromForm)]
struct ShareForm {
    expires_days: Option<u32>,
}

/// Link anlegen und die Detailseite direkt mit dem Link zeigen – der Token steht so weder
/// in einer URL noch in einem Cookie.
#[post("/organizations/<org>/documents/<id>/share", data = "<form>")]
async fn create_share(
    user: CurrentUser,
    org: &str,
    id: &str,
    form: Form<ShareForm>,
    pool: &State<SqlitePool>,
) -> AppResult<Page> {
    organizations::membership(pool, &user.0.id, org).await?;
    let (_, token) =
        share_links::create(pool, org, id, Some(&user.0.id), form.expires_days).await?;
    let page = detail_page(pool, user.0, org, id, "", Some(format!("/share/{token}"))).await?;
    Ok(Page::Html(render(&page)?))
}

#[post("/organizations/<org>/documents/<id>/share/<link>/revoke")]
async fn revoke_share(
    user: CurrentUser,
    org: &str,
    id: &str,
    link: &str,
    pool: &State<SqlitePool>,
) -> AppResult<Redirect> {
    organizations::membership(pool, &user.0.id, org).await?;
    share_links::revoke(pool, org, id, link, Some(&user.0.id)).await?;
    Ok(back(org, id))
}

// ------------------------------------------------------------------ Tags

#[get("/organizations/<org>/tags")]
async fn tags_page(user: CurrentUser, org: &str, pool: &State<SqlitePool>) -> AppResult<Page> {
    let shell = shell(pool, user.0, org, "tags", "").await?;
    let tags = tags::list(pool, org).await?;
    Ok(Page::Html(render(&TagsPage {
        shell,
        tags,
        palette: tags::PALETTE,
        error: None,
    })?))
}

#[derive(FromForm)]
struct TagForm<'r> {
    name: &'r str,
    #[field(default = "")]
    color: &'r str,
    #[field(default = "")]
    description: &'r str,
}

/// Fehler (z. B. Name schon vergeben) zeigt die Tag-Seite mit Meldung.
async fn tags_with_error(pool: &SqlitePool, user: User, org: &str, e: AppError) -> AppResult<Page> {
    let shell = shell(pool, user, org, "tags", "").await?;
    let tags = tags::list(pool, org).await?;
    let status = e.status();
    let page = render(&TagsPage {
        shell,
        tags,
        palette: tags::PALETTE,
        error: Some(e.public_message()),
    })?;
    Ok(Page::Status(rocket::response::status::Custom(status, page)))
}

#[post("/organizations/<org>/tags", data = "<form>")]
async fn create_tag(
    user: CurrentUser,
    org: &str,
    form: Form<TagForm<'_>>,
    pool: &State<SqlitePool>,
) -> AppResult<Page> {
    organizations::membership(pool, &user.0.id, org).await?;
    match tags::create(pool, org, form.name, form.color, form.description).await {
        Ok(_) => Ok(Page::Redirect(Redirect::to(format!(
            "/organizations/{org}/tags"
        )))),
        Err(e @ (AppError::Conflict(_) | AppError::BadRequest(_))) => {
            tags_with_error(pool, user.0, org, e).await
        }
        Err(e) => Err(e),
    }
}

#[post("/organizations/<org>/tags/<id>", data = "<form>")]
async fn update_tag(
    user: CurrentUser,
    org: &str,
    id: &str,
    form: Form<TagForm<'_>>,
    pool: &State<SqlitePool>,
) -> AppResult<Page> {
    organizations::membership(pool, &user.0.id, org).await?;
    match tags::update(pool, org, id, form.name, form.color, form.description).await {
        Ok(_) => Ok(Page::Redirect(Redirect::to(format!(
            "/organizations/{org}/tags"
        )))),
        Err(e @ (AppError::Conflict(_) | AppError::BadRequest(_))) => {
            tags_with_error(pool, user.0, org, e).await
        }
        Err(e) => Err(e),
    }
}

#[post("/organizations/<org>/tags/<id>/delete")]
async fn delete_tag(
    user: CurrentUser,
    org: &str,
    id: &str,
    pool: &State<SqlitePool>,
) -> AppResult<Redirect> {
    organizations::membership(pool, &user.0.id, org).await?;
    tags::delete(pool, org, id).await?;
    Ok(Redirect::to(format!("/organizations/{org}/tags")))
}

// ------------------------------------------------------------------ Papierkorb

#[get("/organizations/<org>/trash")]
async fn trash_page(
    user: CurrentUser,
    org: &str,
    pool: &State<SqlitePool>,
    config: &State<Config>,
) -> AppResult<Page> {
    let shell = shell(pool, user.0, org, "trash", "").await?;
    let (docs, _) = documents::list(pool, org, true, 0, 200).await?;
    Ok(Page::Html(render(&TrashPage {
        shell,
        docs,
        retention_days: config.trash_retention_days,
    })?))
}

#[post("/organizations/<org>/trash/empty")]
async fn empty_trash(
    user: CurrentUser,
    org: &str,
    pool: &State<SqlitePool>,
) -> AppResult<Redirect> {
    let (_, role) = organizations::membership(pool, &user.0.id, org).await?;
    if !role.can_manage() {
        return Err(AppError::forbidden(
            "Nur Besitzer und Admins können den Papierkorb leeren.",
        ));
    }
    sqlx::query("DELETE FROM documents WHERE organization_id = ? AND deleted_at IS NOT NULL")
        .bind(org)
        .execute(pool.inner())
        .await?;
    Ok(Redirect::to(format!("/organizations/{org}/trash")))
}
