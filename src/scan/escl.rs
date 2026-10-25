//! eSCL (AirScan): Status, Scan einer Seite vom Glas, Suche im Netz.

use std::time::{Duration, Instant};

fn xml_text<'a>(xml: &'a str, tag: &str) -> Option<&'a str> {
    let open = format!("<{tag}>");
    let start = xml.find(&open)? + open.len();
    let end = xml[start..].find('<')? + start;
    Some(&xml[start..end])
}

fn base(url: &str) -> Result<String, String> {
    let url = url.trim().trim_end_matches('/');
    if url.is_empty() {
        return Err("Keine Scanner-Adresse in den Einstellungen.".into());
    }
    Ok(if url.starts_with("http") {
        url.to_string()
    } else {
        format!("http://{url}")
    })
}

pub async fn status(scanner: &str) -> Result<String, String> {
    let http = &*super::HTTP;
    let body = http
        .get(format!("{}/eSCL/ScannerStatus", base(scanner)?))
        .timeout(Duration::from_secs(5))
        .send()
        .await
        .map_err(|e| format!("Scanner nicht erreichbar: {e}"))?
        .text()
        .await
        .map_err(|e| e.to_string())?;
    Ok(xml_text(&body, "pwg:State").unwrap_or("?").to_string())
}

async fn job_reason(http: &reqwest::Client, scanner: &str, job_uuid: &str) -> Option<String> {
    let body = http
        .get(format!("{scanner}/eSCL/ScannerStatus"))
        .send()
        .await
        .ok()?
        .text()
        .await
        .ok()?;
    let job = &body[body.find(job_uuid)?..];
    let job = &job[..job.find("</scan:JobInfo>").unwrap_or(job.len())];
    Some(format!(
        "{} ({})",
        xml_text(job, "pwg:JobState").unwrap_or("?"),
        xml_text(job, "pwg:JobStateReason").unwrap_or("")
    ))
}

fn settings(color: bool, dpi: u32) -> String {
    // A4 in 1/300 Zoll
    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<scan:ScanSettings xmlns:scan="http://schemas.hp.com/imaging/escl/2011/05/03" xmlns:pwg="http://www.pwg.org/schemas/2010/12/sm">
  <pwg:Version>2.9</pwg:Version>
  <scan:Intent>Document</scan:Intent>
  <pwg:ScanRegions><pwg:ScanRegion>
    <pwg:Height>3508</pwg:Height><pwg:Width>2480</pwg:Width>
    <pwg:XOffset>0</pwg:XOffset><pwg:YOffset>0</pwg:YOffset>
    <pwg:ContentRegionUnits>escl:ThreeHundredthsOfInches</pwg:ContentRegionUnits>
  </pwg:ScanRegion></pwg:ScanRegions>
  <pwg:InputSource>Platen</pwg:InputSource>
  <scan:ColorMode>{}</scan:ColorMode>
  <scan:XResolution>{dpi}</scan:XResolution>
  <scan:YResolution>{dpi}</scan:YResolution>
  <pwg:DocumentFormat>image/jpeg</pwg:DocumentFormat>
  <scan:DocumentFormatExt>image/jpeg</scan:DocumentFormatExt>
</scan:ScanSettings>"#,
        if color { "RGB24" } else { "Grayscale8" }
    )
}

/// Scannt eine Seite vom Glas und liefert das JPEG.
pub async fn scan_page(scanner: &str, color: bool, dpi: u32) -> Result<Vec<u8>, String> {
    let http = &*super::HTTP;
    let scanner = base(scanner)?;
    let resp = http
        .post(format!("{scanner}/eSCL/ScanJobs"))
        .header("Content-Type", "text/xml")
        .body(settings(color, dpi))
        .timeout(Duration::from_secs(20))
        .send()
        .await
        .map_err(|e| format!("Scanner nicht erreichbar: {e}"))?;
    if resp.status().as_u16() == 503 {
        return Err("Scanner ist beschäftigt – kurz warten und nochmal versuchen.".into());
    }
    if resp.status().as_u16() != 201 {
        return Err(format!(
            "Scan-Auftrag abgelehnt (HTTP {}).",
            resp.status().as_u16()
        ));
    }
    let location = resp
        .headers()
        .get("Location")
        .and_then(|v| v.to_str().ok())
        .ok_or("Scanner hat keinen Auftrag zurückgegeben.")?
        .trim_end_matches('/')
        .to_string();
    let job_url = if location.starts_with("http") {
        location
    } else {
        format!("{scanner}/{}", location.trim_start_matches('/'))
    };
    let job_uuid = job_url.rsplit('/').next().unwrap_or_default().to_string();

    let deadline = Instant::now() + Duration::from_secs(180);
    while Instant::now() < deadline {
        let doc = http
            .get(format!("{job_url}/NextDocument"))
            .timeout(Duration::from_secs(180))
            .send()
            .await
            .map_err(|e| format!("Verbindung zum Scanner verloren: {e}"))?;
        match doc.status().as_u16() {
            200 => {
                let bytes = doc.bytes().await.map_err(|e| e.to_string())?;
                if bytes.is_empty() {
                    return Err("Scanner hat ein leeres Bild geliefert.".into());
                }
                return Ok(bytes.to_vec());
            }
            503 => tokio::time::sleep(Duration::from_secs(1)).await,
            code => {
                let reason = job_reason(http, &scanner, &job_uuid)
                    .await
                    .unwrap_or_else(|| format!("HTTP {code}"));
                let hint = if reason.contains("CanceledAtDevice") {
                    " – Drucker einmal ganz vom Strom trennen und neu starten."
                } else {
                    ""
                };
                return Err(format!("Scan wurde vom Gerät abgebrochen: {reason}{hint}"));
            }
        }
    }
    Err("Zeitüberschreitung beim Scannen.".into())
}

// ---------------------------------------------------------------- Suche im Netzwerk

#[derive(Debug, serde::Serialize)]
pub struct Found {
    pub url: String,
    pub model: String,
}

/// Wandelt "192.168.0.0/24", "192.168.0" oder "192.168.1.50" in die drei ersten Oktette.
pub fn prefix(subnet: &str) -> Option<String> {
    let ip = subnet.trim().split('/').next()?.trim();
    let parts: Vec<&str> = ip.split('.').filter(|p| !p.is_empty()).collect();
    if parts.len() < 3 || !parts[..3].iter().all(|p| p.parse::<u8>().is_ok()) {
        return None;
    }
    Some(parts[..3].join("."))
}

/// Fragt in jedem /24-Netz alle Adressen nach eSCL (AirScan) ab – parallel, kurze Zeitgrenze.
pub async fn discover(subnets: &[String]) -> (Vec<Found>, Vec<String>) {
    let mut prefixes: Vec<String> = subnets.iter().filter_map(|s| prefix(s)).collect();
    prefixes.sort();
    prefixes.dedup();
    let limit = std::sync::Arc::new(tokio::sync::Semaphore::new(96));
    let mut jobs = tokio::task::JoinSet::new();
    for p in &prefixes {
        for host in 1..=254u8 {
            let (http, limit, url) = (
                super::HTTP.clone(),
                limit.clone(),
                format!("http://{p}.{host}"),
            );
            jobs.spawn(async move {
                let _permit = limit.acquire().await.ok()?;
                let resp = http
                    .get(format!("{url}/eSCL/ScannerCapabilities"))
                    .timeout(Duration::from_millis(1800))
                    .send()
                    .await
                    .ok()?;
                if !resp.status().is_success() {
                    return None;
                }
                let body = resp.text().await.ok()?;
                if !body.contains("ScannerCapabilities") {
                    return None;
                }
                let model = xml_text(&body, "pwg:MakeAndModel")
                    .unwrap_or("eSCL-Scanner")
                    .to_string();
                Some(Found { url, model })
            });
        }
    }
    let mut found = Vec::new();
    while let Some(result) = jobs.join_next().await {
        if let Ok(Some(f)) = result {
            found.push(f);
        }
    }
    found.sort_by(|a, b| a.url.cmp(&b.url));
    (
        found,
        prefixes.into_iter().map(|p| format!("{p}.0/24")).collect(),
    )
}

#[cfg(test)]
mod tests {
    #[test]
    fn subnet_prefix() {
        assert_eq!(
            super::prefix("192.168.0.0/24").as_deref(),
            Some("192.168.0")
        );
        assert_eq!(super::prefix(" 10.0.5.70 ").as_deref(), Some("10.0.5"));
        assert_eq!(super::prefix("192.168.0").as_deref(), Some("192.168.0"));
        assert_eq!(super::prefix("300.1.1.0/24"), None);
        assert_eq!(super::prefix("scanner.local"), None);
    }
}
