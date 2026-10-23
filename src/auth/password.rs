//! Passwörter als Argon2id-Hash (PHC-String). Hashen und Prüfen laufen auf einem
//! Blocking-Thread, damit sie die Server-Threads nicht aufhalten.

use crate::error::{AppError, AppResult};
use argon2::password_hash::{rand_core::OsRng, PasswordHash, SaltString};
use argon2::{Argon2, PasswordHasher, PasswordVerifier};
use std::sync::LazyLock;

pub const MIN_LENGTH: usize = 8;

/// Hash eines zufälligen Passworts – wird bei unbekannten Benutzernamen geprüft,
/// damit die Anmeldung immer gleich lange dauert.
pub static DUMMY_HASH: LazyLock<String> = LazyLock::new(|| {
    let salt = SaltString::generate(&mut OsRng);
    Argon2::default().hash_password(&rand::random::<[u8; 16]>(), &salt).expect("Argon2").to_string()
});

pub fn check_strength(password: &str) -> AppResult<()> {
    if password.chars().count() < MIN_LENGTH {
        return Err(AppError::bad(format!("Passwort: mindestens {MIN_LENGTH} Zeichen.")));
    }
    Ok(())
}

pub async fn hash(password: String) -> AppResult<String> {
    tokio::task::spawn_blocking(move || {
        let salt = SaltString::generate(&mut OsRng);
        Argon2::default().hash_password(password.as_bytes(), &salt).map(|h| h.to_string())
    })
    .await
    .map_err(|e| AppError::Config(format!("Hash-Thread: {e}")))?
    .map_err(|e| AppError::Config(format!("Argon2: {e}")))
}

pub async fn verify(password: String, hash: String) -> bool {
    tokio::task::spawn_blocking(move || {
        PasswordHash::new(&hash).is_ok_and(|h| Argon2::default().verify_password(password.as_bytes(), &h).is_ok())
    })
    .await
    .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    #[tokio::test]
    async fn hash_and_verify() {
        let h = super::hash("korrekt-pferd".into()).await.unwrap();
        assert!(h.starts_with("$argon2id$"));
        assert!(super::verify("korrekt-pferd".into(), h.clone()).await);
        assert!(!super::verify("falsch".into(), h).await);
        assert!(!super::verify("x".into(), "kein hash".into()).await);
    }
}
