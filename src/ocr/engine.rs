//! Poppler (`pdfinfo`, `pdftotext`, `pdftocairo`) und Tesseract als Systemprogramme.
//!
//! PDF: erst die Textebene (`pdftotext`). Hat sie zu wenig Text (gescanntes PDF), wird
//! jede Seite mit `pdftocairo` als PNG nach stdout gerendert und direkt in `tesseract`
//! gegeben. Bilder gehen direkt an Tesseract.

use std::process::Stdio;
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::process::Command;

/// Höchstens so viele Seiten per Bilderkennung (die Textebene wird immer ganz gelesen).
pub const MAX_OCR_PAGES: u32 = 150;
/// Durchschnittlich so viele Zeichen je Seite – darunter gilt die Textebene als leer.
const MIN_CHARS_PER_PAGE: usize = 25;
const TIMEOUT: Duration = Duration::from_secs(300);

#[derive(Debug, Clone)]
pub struct Engine {
    /// Sprachen für Tesseract, nur solche, die installiert sind (z. B. "deu+eng").
    pub languages: Option<String>,
    pub poppler: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Extracted {
    pub text: String,
    pub pages: Option<i64>,
}

/// Bild-Typen, die Tesseract lesen kann.
pub fn is_ocr_image(mime: &str) -> bool {
    matches!(
        mime,
        "image/png" | "image/jpeg" | "image/tiff" | "image/webp" | "image/bmp" | "image/gif"
    )
}

impl Engine {
    /// Prüft, welche Programme und Sprachen vorhanden sind.
    pub async fn detect(wanted_languages: &str) -> Engine {
        let poppler = run("pdftotext", &["-v"], &[]).await.is_ok()
            && run("pdftocairo", &["-v"], &[]).await.is_ok();
        let languages = match run("tesseract", &["--list-langs"], &[]).await {
            Ok(out) => {
                let installed: Vec<String> = String::from_utf8_lossy(&out)
                    .lines()
                    .skip(1)
                    .map(|l| l.trim().to_string())
                    .collect();
                let usable: Vec<&str> = wanted_languages
                    .split('+')
                    .map(str::trim)
                    .filter(|l| installed.iter().any(|i| i == l))
                    .collect();
                let missing: Vec<&str> = wanted_languages
                    .split('+')
                    .map(str::trim)
                    .filter(|l| !usable.contains(l))
                    .collect();
                if !missing.is_empty() {
                    tracing::warn!(
                        "Tesseract-Sprachen fehlen: {} (installiert: {})",
                        missing.join(", "),
                        installed.join(", ")
                    );
                }
                if usable.is_empty() {
                    installed.iter().find(|l| *l == "eng").cloned()
                } else {
                    Some(usable.join("+"))
                }
            }
            Err(_) => None,
        };
        match (&languages, poppler) {
            (Some(l), true) => tracing::info!("Texterkennung bereit: Tesseract ({l}) und Poppler"),
            (Some(l), false) => {
                tracing::warn!("Texterkennung nur für Bilder: Tesseract ({l}), Poppler fehlt")
            }
            (None, true) => tracing::warn!("Nur PDF-Textebene: Tesseract fehlt"),
            (None, false) => tracing::warn!("Keine Texterkennung: Tesseract und Poppler fehlen"),
        }
        Engine { languages, poppler }
    }

    /// Text aus einer Datei. `Ok(None)`: Dateityp ohne Texterkennung.
    pub async fn extract(&self, mime: &str, data: &[u8]) -> Result<Option<Extracted>, String> {
        if mime == "application/pdf" {
            return self.pdf(data).await.map(Some);
        }
        if is_ocr_image(mime) {
            let text = self.tesseract(data).await?;
            return Ok(Some(Extracted {
                text,
                pages: Some(1),
            }));
        }
        Ok(None)
    }

    async fn tesseract(&self, image: &[u8]) -> Result<String, String> {
        let langs = self
            .languages
            .as_deref()
            .ok_or("Tesseract ist nicht installiert.")?;
        let out = run(
            "tesseract",
            &["stdin", "stdout", "-l", langs, "--psm", "3"],
            image,
        )
        .await?;
        Ok(String::from_utf8_lossy(&out).into_owned())
    }

    async fn pdf(&self, data: &[u8]) -> Result<Extracted, String> {
        if !self.poppler {
            return Err("Poppler (pdftotext, pdftocairo) ist nicht installiert.".into());
        }
        let info = String::from_utf8_lossy(&run("pdfinfo", &["fd://0"], data).await?).into_owned();
        let pages: u32 = info
            .lines()
            .find_map(|l| l.strip_prefix("Pages:"))
            .and_then(|v| v.trim().parse().ok())
            .unwrap_or(1);

        let layer = String::from_utf8_lossy(
            &run(
                "pdftotext",
                &["-layout", "-enc", "UTF-8", "fd://0", "-"],
                data,
            )
            .await?,
        )
        .into_owned();
        let chars = layer.chars().filter(|c| !c.is_whitespace()).count();
        if chars >= MIN_CHARS_PER_PAGE * pages.max(1) as usize || self.languages.is_none() {
            return Ok(Extracted {
                text: clean(&layer),
                pages: Some(pages.into()),
            });
        }

        // Gescanntes PDF: Seite für Seite rendern und lesen.
        let mut texts = Vec::new();
        for page in 1..=pages.min(MAX_OCR_PAGES) {
            let p = page.to_string();
            let png = run(
                "pdftocairo",
                &[
                    "-png",
                    "-singlefile",
                    "-r",
                    "300",
                    "-gray",
                    "-f",
                    &p,
                    "-l",
                    &p,
                    "fd://0",
                    "-",
                ],
                data,
            )
            .await?;
            texts.push(self.tesseract(&png).await?);
        }
        let ocr = clean(&texts.join("\n\n"));
        // Falls die Textebene doch etwas hatte und mehr ist als die Erkennung: die nehmen.
        let text = if chars > ocr.chars().filter(|c| !c.is_whitespace()).count() {
            clean(&layer)
        } else {
            ocr
        };
        Ok(Extracted {
            text,
            pages: Some(pages.into()),
        })
    }
}

/// Seitenumbrüche und überflüssige Leerzeilen entfernen.
fn clean(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut blank = 0;
    for line in text.replace('\u{c}', "\n").lines() {
        let line = line.trim_end();
        if line.trim().is_empty() {
            blank += 1;
            if blank > 1 {
                continue;
            }
        } else {
            blank = 0;
        }
        out.push_str(line);
        out.push('\n');
    }
    out.trim().to_string()
}

/// Programm starten, `input` über stdin geben, stdout zurückgeben – mit Zeitlimit.
pub(crate) async fn run(program: &str, args: &[&str], input: &[u8]) -> Result<Vec<u8>, String> {
    let mut child = Command::new(program)
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .kill_on_drop(true)
        .spawn()
        .map_err(|e| format!("{program}: {e}"))?;

    // Schreiben und Lesen gleichzeitig, sonst blockieren sich volle Pipes gegenseitig.
    let mut stdin = child.stdin.take().expect("stdin");
    let input = input.to_vec();
    let writer = tokio::spawn(async move {
        let _ = stdin.write_all(&input).await;
    });
    let mut stdout = child.stdout.take().expect("stdout");
    let mut stderr = child.stderr.take().expect("stderr");
    let work = async {
        let mut out = Vec::new();
        let mut err = Vec::new();
        let (r1, r2) = tokio::join!(stdout.read_to_end(&mut out), stderr.read_to_end(&mut err));
        r1.and(r2).map_err(|e| e.to_string())?;
        let status = child.wait().await.map_err(|e| e.to_string())?;
        Ok::<_, String>((status, out, err))
    };
    let (status, out, err) = tokio::time::timeout(TIMEOUT, work)
        .await
        .map_err(|_| format!("{program}: Zeitlimit überschritten"))??;
    let _ = writer.await;
    if !status.success() {
        let msg = String::from_utf8_lossy(&err);
        return Err(format!(
            "{program}: {}",
            msg.lines().last().unwrap_or("Fehler").trim()
        ));
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clean_collapses_blank_lines() {
        assert_eq!(clean("a\n\n\n\nb\u{c}c  \n"), "a\n\nb\nc");
    }

    #[tokio::test]
    async fn missing_program_is_an_error() {
        assert!(run("gibt-es-nicht-nrzpaper", &[], b"").await.is_err());
    }
}
