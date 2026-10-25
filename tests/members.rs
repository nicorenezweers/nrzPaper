//! Mehrere Konten in einer Organisation: Einladen, Annehmen, Rollen, Entfernen, Austreten,
//! Besitz übergeben, Konten durch den Administrator der Instanz.

mod common;

use common::{call, login, logout, register};
use rocket::http::Status;
use serde_json::json;

#[rocket::async_test]
async fn invite_accept_roles_and_leave() {
    let (_d, c, org) = common::client().await;
    let base = format!("/api/v1/organizations/{org}");

    // Der Admin der Instanz legt zwei Konten an (Registrierung ist zu)
    for (name, admin) in [("anna", false), ("ben", false)] {
        let (s, v) = call(
            &c,
            "POST",
            "/api/v1/settings/users",
            Some(json!({ "username": name, "password": "geheim-123", "isAdmin": admin })),
        )
        .await;
        assert_eq!(s, Status::Created, "{v}");
    }
    assert_eq!(
        call(&c, "POST", "/api/v1/settings/users", Some(json!({ "username": "anna", "password": "geheim-123" }))).await.0,
        Status::Conflict
    );

    // Einladen: Anna als Admin, Ben als Mitglied, Unbekannte geht auch (für später)
    let invite = |name: &str, role: &str| {
        call(&c, "POST", format!("{base}/invitations"), Some(json!({ "username": name, "role": role })))
    };
    let (s, v) = invite("anna", "admin").await;
    assert_eq!(s, Status::Created);
    assert_eq!(v["userExists"], true);
    assert_eq!(invite("ben", "member").await.0, Status::Created);
    let (s, v) = invite("niemand", "member").await;
    assert_eq!((s, v["userExists"].clone()), (Status::Created, json!(false)));
    assert_eq!(invite("alice", "member").await.0, Status::Conflict, "schon Mitglied");
    assert_eq!(invite("x", "owner").await.0, Status::BadRequest);
    let (_, v) = call(&c, "GET", format!("{base}/members"), None).await;
    assert_eq!(v["invitations"].as_array().unwrap().len(), 3);
    let niemand = v["invitations"].as_array().unwrap().iter().find(|i| i["invitee"] == "niemand").unwrap()["id"]
        .as_str()
        .unwrap()
        .to_string();
    assert_eq!(call(&c, "DELETE", format!("{base}/invitations/{niemand}"), None).await.0, Status::NoContent);

    // Anna nimmt an, Ben lehnt erst ab (wird dann neu eingeladen und nimmt an)
    let accept = |user: &'static str, by: &'static str| {
        let c = &c;
        async move {
            logout(c).await;
            login(c, user, "geheim-123").await;
            let (_, v) = call(c, "GET", "/api/v1/me/invitations", None).await;
            let inv = v["invitations"][0].clone();
            assert_eq!(inv["organizationName"], "Privat", "{v}");
            assert_eq!(inv["invitedBy"], by);
            inv["id"].as_str().unwrap().to_string()
        }
    };
    let id = accept("anna", "alice").await;
    // Vor dem Annehmen: kein Zugriff
    assert_eq!(call(&c, "GET", format!("{base}/overview"), None).await.0, Status::NotFound);
    let (s, v) = call(&c, "POST", format!("/api/v1/me/invitations/{id}/accept"), None).await;
    assert_eq!((s, v["organizationId"].clone()), (Status::Ok, json!(org)));
    let (_, v) = call(&c, "GET", format!("{base}/overview"), None).await;
    assert_eq!(v["role"], "admin");
    let (_, v) = call(&c, "GET", "/api/v1/me/invitations", None).await;
    assert_eq!(v["invitations"], json!([]), "angenommen = weg");

    let id = accept("ben", "alice").await;
    assert_eq!(call(&c, "DELETE", format!("/api/v1/me/invitations/{id}"), None).await.0, Status::NoContent);
    assert_eq!(call(&c, "POST", format!("/api/v1/me/invitations/{id}/accept"), None).await.0, Status::NotFound);
    // Mitglieder sehen keine Einladungen und laden nicht ein
    logout(&c).await;
    login(&c, "anna", "geheim-123").await;
    assert_eq!(invite("ben", "admin").await.0, Status::Forbidden, "Admins laden keine Admins ein");
    assert_eq!(invite("ben", "member").await.0, Status::Created);
    let id = accept("ben", "anna").await;
    call(&c, "POST", format!("/api/v1/me/invitations/{id}/accept"), None).await;
    let (_, v) = call(&c, "GET", format!("{base}/members"), None).await;
    assert_eq!(v["role"], "member");
    assert_eq!(v["invitations"], json!([]));
    let names: Vec<&str> = v["members"].as_array().unwrap().iter().map(|m| m["username"].as_str().unwrap()).collect();
    assert_eq!(names, ["alice", "anna", "ben"], "Besitzer, Admin, Mitglied");
    let ids: std::collections::HashMap<String, String> = v["members"]
        .as_array()
        .unwrap()
        .iter()
        .map(|m| (m["username"].as_str().unwrap().to_string(), m["id"].as_str().unwrap().to_string()))
        .collect();
    assert_eq!(invite("x", "member").await.0, Status::Forbidden);
    assert_eq!(
        call(&c, "DELETE", format!("{base}/members/{}", ids["anna"]), None).await.0,
        Status::Forbidden
    );
    // Gemeinsame Dokumente: Ben sieht, was in der Organisation liegt
    assert_eq!(call(&c, "GET", format!("{base}/documents"), None).await.0, Status::Ok);

    // Anna (Admin): Ben zum Admin machen geht nicht, entfernen schon; der Besitzer bleibt
    logout(&c).await;
    login(&c, "anna", "geheim-123").await;
    let role = |user: &str, role: &str| {
        call(&c, "PUT", format!("{base}/members/{}", ids[user]), Some(json!({ "role": role })))
    };
    assert_eq!(role("ben", "admin").await.0, Status::Forbidden);
    assert_eq!(role("alice", "member").await.0, Status::BadRequest);
    assert_eq!(
        call(&c, "DELETE", format!("{base}/members/{}", ids["alice"]), None).await.0,
        Status::BadRequest
    );
    assert_eq!(
        call(&c, "DELETE", format!("{base}/members/{}", ids["ben"]), None).await.0,
        Status::NoContent
    );

    // Besitzer übergibt an Anna und ist danach Admin; Anna kann nicht austreten, Alice schon
    logout(&c).await;
    login(&c, "alice", "geheim-123").await;
    assert_eq!(
        call(&c, "DELETE", format!("{base}/members/{}", ids["alice"]), None).await.0,
        Status::BadRequest,
        "Besitzer tritt nicht einfach aus"
    );
    assert_eq!(role("anna", "owner").await.0, Status::NoContent);
    let (_, v) = call(&c, "GET", format!("{base}/overview"), None).await;
    assert_eq!(v["role"], "admin");
    assert_eq!(
        call(&c, "DELETE", format!("{base}/members/{}", ids["alice"]), None).await.0,
        Status::NoContent,
        "austreten"
    );
    assert_eq!(call(&c, "GET", format!("{base}/overview"), None).await.0, Status::NotFound);

    // Konto löschen: Anna besitzt jetzt eine Organisation, aber allein → geht, die Organisation mit
    let (_, v) = call(&c, "GET", "/api/v1/settings/users", None).await;
    let anna = v["users"].as_array().unwrap().iter().find(|u| u["username"] == "anna").unwrap()["id"]
        .as_str()
        .unwrap()
        .to_string();
    assert_eq!(call(&c, "DELETE", format!("/api/v1/settings/users/{anna}"), None).await.0, Status::NoContent);
    assert_eq!(login(&c, "anna", "geheim-123").await, Status::Unauthorized);
    c.terminate().await;
}

#[rocket::async_test]
async fn only_instance_admins_manage_accounts() {
    let (_d, c, _org) = common::client().await;
    call(&c, "POST", "/api/v1/settings/users", Some(json!({ "username": "carla", "password": "geheim-123" }))).await;
    let (_, v) = call(&c, "GET", "/api/v1/me", None).await;
    let me = v["user"]["id"].as_str().unwrap().to_string();
    assert_eq!(call(&c, "DELETE", format!("/api/v1/settings/users/{me}"), None).await.0, Status::BadRequest);
    logout(&c).await;
    login(&c, "carla", "geheim-123").await;
    assert_eq!(
        call(&c, "POST", "/api/v1/settings/users", Some(json!({ "username": "dora", "password": "geheim-123" }))).await.0,
        Status::Forbidden
    );
    assert_eq!(register(&c, "dora", "geheim-123").await, Status::Forbidden, "Registrierung ist zu");
    c.terminate().await;
}
