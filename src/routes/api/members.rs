//! Mitglieder und Einladungen einer Organisation, eigene Einladungen, und für Administratoren
//! der Instanz: Konten anlegen und löschen (damit jemand auch bei geschlossener Registrierung
//! in eine Organisation kommt). Regeln siehe services::members.

use crate::auth::guards::{AdminUser, CurrentUser};
use crate::db::models::{now, Role};
use crate::error::{AppError, AppResult};
use crate::services::{members, organizations, users};
use rocket::http::Status;
use rocket::serde::json::{json, Json, Value};
use rocket::{delete, get, post, put, routes, Route, State};
use serde::Deserialize;
use sqlx::SqlitePool;

pub fn routes() -> Vec<Route> {
    routes![
        list,
        invite,
        revoke,
        set_role,
        remove,
        my_invitations,
        accept,
        decline,
        create_user,
        delete_user
    ]
}

async fn role(pool: &SqlitePool, user: &CurrentUser, org: &str) -> AppResult<Role> {
    Ok(organizations::membership(pool, &user.0.id, org).await?.1)
}

/// Mitglieder (Besitzer zuerst); für Besitzer und Admins auch die offenen Einladungen.
#[get("/organizations/<org>/members")]
async fn list(user: CurrentUser, org: &str, pool: &State<SqlitePool>) -> AppResult<Value> {
    let me = role(pool, &user, org).await?;
    let list: Vec<Value> = organizations::members(pool, org)
        .await?
        .into_iter()
        .map(|(id, username, role, since)| json!({ "id": id, "username": username, "role": role, "since": since }))
        .collect();
    let invitations = if me.can_manage() {
        json!(members::pending(pool, org).await?)
    } else {
        json!([])
    };
    Ok(json!({ "members": list, "invitations": invitations, "role": me }))
}

#[derive(Deserialize)]
struct InviteInput {
    username: String,
    role: Role,
}

#[post("/organizations/<org>/invitations", data = "<body>")]
async fn invite(
    user: CurrentUser,
    org: &str,
    body: Json<InviteInput>,
    pool: &State<SqlitePool>,
) -> AppResult<(Status, Value)> {
    let me = role(pool, &user, org).await?;
    let inv = members::invite(pool, org, (&user.0.id, me), &body.username, body.role).await?;
    let exists = users::by_username(pool, &inv.invitee).await?.is_some();
    Ok((Status::Created, json!({ "invitation": inv, "userExists": exists })))
}

#[delete("/organizations/<org>/invitations/<id>")]
async fn revoke(user: CurrentUser, org: &str, id: &str, pool: &State<SqlitePool>) -> AppResult<Status> {
    let me = role(pool, &user, org).await?;
    members::revoke(pool, org, me, id).await?;
    Ok(Status::NoContent)
}

#[derive(Deserialize)]
struct RoleInput {
    role: Role,
}

#[put("/organizations/<org>/members/<id>", data = "<body>")]
async fn set_role(
    user: CurrentUser,
    org: &str,
    id: &str,
    body: Json<RoleInput>,
    pool: &State<SqlitePool>,
) -> AppResult<Status> {
    let me = role(pool, &user, org).await?;
    members::set_role(pool, org, (&user.0.id, me), id, body.role).await?;
    Ok(Status::NoContent)
}

/// Mitglied entfernen; mit der eigenen ID: austreten.
#[delete("/organizations/<org>/members/<id>")]
async fn remove(user: CurrentUser, org: &str, id: &str, pool: &State<SqlitePool>) -> AppResult<Status> {
    let me = role(pool, &user, org).await?;
    members::remove(pool, org, (&user.0.id, me), id).await?;
    Ok(Status::NoContent)
}

#[get("/me/invitations")]
async fn my_invitations(user: CurrentUser, pool: &State<SqlitePool>) -> AppResult<Value> {
    Ok(json!({ "invitations": members::for_user(pool, &user.0.username).await? }))
}

#[post("/me/invitations/<id>/accept")]
async fn accept(user: CurrentUser, id: &str, pool: &State<SqlitePool>) -> AppResult<Value> {
    let org = members::accept(pool, &user.0.id, &user.0.username, id).await?;
    Ok(json!({ "organizationId": org }))
}

#[delete("/me/invitations/<id>")]
async fn decline(user: CurrentUser, id: &str, pool: &State<SqlitePool>) -> AppResult<Status> {
    members::decline(pool, &user.0.username, id).await?;
    Ok(Status::NoContent)
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct NewUser {
    username: String,
    password: String,
    #[serde(default)]
    is_admin: bool,
}

/// Konto anlegen (Administratoren der Instanz) – auch wenn die Registrierung geschlossen ist.
#[post("/settings/users", data = "<body>")]
async fn create_user(
    _admin: AdminUser,
    body: Json<NewUser>,
    pool: &State<SqlitePool>,
) -> AppResult<(Status, Value)> {
    let u = users::create(pool, &body.username, &body.password, "").await?;
    if body.is_admin && !u.is_admin {
        sqlx::query("UPDATE users SET is_admin = 1, updated_at = ? WHERE id = ?")
            .bind(now())
            .bind(&u.id)
            .execute(pool.inner())
            .await?;
    }
    Ok((
        Status::Created,
        json!({ "user": { "id": u.id, "username": u.username, "isAdmin": body.is_admin || u.is_admin, "createdAt": u.created_at } }),
    ))
}

/// Konto löschen. Nicht das eigene; Organisationen, in denen die Person Besitzer ist und die
/// noch andere Mitglieder haben, müssen erst übergeben werden. Eigene Einzel-Organisationen
/// verschwinden mit (Papierkorb der Organisation).
#[delete("/settings/users/<id>")]
async fn delete_user(admin: AdminUser, id: &str, pool: &State<SqlitePool>) -> AppResult<Status> {
    if admin.0.id == id {
        return Err(AppError::bad("Das eigene Konto lässt sich hier nicht löschen."));
    }
    users::by_id(pool, id).await?.ok_or(AppError::NotFound)?;
    let owned: Vec<(String, String, i64)> = sqlx::query_as(
        "SELECT o.id, o.name, (SELECT count(*) FROM organization_members x WHERE x.organization_id = o.id)
         FROM organizations o JOIN organization_members m ON m.organization_id = o.id
         WHERE m.user_id = ? AND m.role = 'owner' AND o.deleted_at IS NULL",
    )
    .bind(id)
    .fetch_all(pool.inner())
    .await?;
    if let Some((_, name, _)) = owned.iter().find(|(_, _, n)| *n > 1) {
        return Err(AppError::bad(format!(
            "Die Person besitzt „{name}“ mit weiteren Mitgliedern – dort zuerst den Besitz übergeben."
        )));
    }
    for (org, _, _) in &owned {
        organizations::delete(pool, org).await?;
    }
    sqlx::query("DELETE FROM users WHERE id = ?")
        .bind(id)
        .execute(pool.inner())
        .await?;
    Ok(Status::NoContent)
}
