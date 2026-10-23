//! Anmeldung: Argon2id-Passwörter, Sitzungen in der Datenbank, API-Keys und die
//! Request Guards, mit denen Routen Anmeldung und Rechte verlangen.

pub mod api_key;
pub mod guards;
pub mod password;
pub mod session;
