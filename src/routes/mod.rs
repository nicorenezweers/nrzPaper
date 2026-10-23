//! Routen: `web` rendert die Oberfläche (Askama), `api` ist die REST-API unter `/api/v1`,
//! `catchers` beantwortet Fehler passend als HTML oder JSON.

pub mod api;
pub mod catchers;
pub mod upload;
pub mod web;
