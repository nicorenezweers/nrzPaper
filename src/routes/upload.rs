//! Datei-Upload aus Formularen – **nur im Speicher**. Rockets `TempFile` würde die Datei
//! zuerst unverschlüsselt in ein temporäres Verzeichnis schreiben; das wollen wir nicht.

use rocket::data::ToByteUnit;
use rocket::form::{self, error::ErrorKind, DataField, FromFormField};

pub struct UploadedFile {
    pub name: String,
    pub data: Vec<u8>,
}

#[rocket::async_trait]
impl<'v> FromFormField<'v> for UploadedFile {
    async fn from_data(field: DataField<'v, '_>) -> form::Result<'v, Self> {
        let limit = field
            .request
            .limits()
            .get("file")
            .unwrap_or(200.mebibytes());
        let bytes = field.data.open(limit).into_bytes().await?;
        if !bytes.is_complete() {
            Err(ErrorKind::InvalidLength {
                min: None,
                max: Some(limit.as_u64()),
            })?;
        }
        // Rohen Namen nehmen (inkl. Umlauten) – bereinigt wird beim Aufnehmen.
        let name = field
            .file_name
            .map(|f| f.dangerous_unsafe_unsanitized_raw().as_str().to_string())
            .unwrap_or_else(|| "Dokument".into());
        Ok(UploadedFile {
            name,
            data: bytes.into_inner(),
        })
    }
}

#[derive(rocket::FromForm)]
pub struct UploadForm {
    /// Ein oder mehrere Dateien im Feld `file`.
    pub file: Vec<UploadedFile>,
}
