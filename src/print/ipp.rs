//! IPP (RFC 8010/8011), so weit nrzPaper es braucht: Druckerangaben abfragen
//! (`Get-Printer-Attributes`) und einen Auftrag schicken (`Print-Job`). Das spricht praktisch
//! jeder Netzwerkdrucker (AirPrint, IPP Everywhere, Mopria, CUPS).

use std::collections::HashMap;

pub const GET_PRINTER_ATTRIBUTES: u16 = 0x000B;
pub const PRINT_JOB: u16 = 0x0002;

// Gruppen
const OPERATION: u8 = 0x01;
const JOB: u8 = 0x02;
const END: u8 = 0x03;
// Werttypen
const INTEGER: u8 = 0x21;
const ENUM: u8 = 0x23;
const NAME: u8 = 0x42;
const KEYWORD: u8 = 0x44;
const URI: u8 = 0x45;
const CHARSET: u8 = 0x47;
const LANGUAGE: u8 = 0x48;
const MIME: u8 = 0x49;

/// Ein Attribut für die Anfrage.
pub enum Value {
    Text(u8, String),
    Int(u8, i32),
}

pub struct Request {
    op: u16,
    id: u32,
    operation: Vec<(String, Vec<Value>)>,
    job: Vec<(String, Vec<Value>)>,
}

impl Request {
    pub fn new(op: u16, printer_uri: &str) -> Self {
        let mut r = Request { op, id: 1, operation: Vec::new(), job: Vec::new() };
        r.op_attr("attributes-charset", Value::Text(CHARSET, "utf-8".into()));
        r.op_attr("attributes-natural-language", Value::Text(LANGUAGE, "en".into()));
        r.op_attr("printer-uri", Value::Text(URI, printer_uri.into()));
        r
    }

    pub fn op_attr(&mut self, name: &str, v: Value) -> &mut Self {
        self.operation.push((name.into(), vec![v]));
        self
    }

    pub fn op_keywords(&mut self, name: &str, words: &[&str]) -> &mut Self {
        self.operation
            .push((name.into(), words.iter().map(|w| Value::Text(KEYWORD, w.to_string())).collect()));
        self
    }

    pub fn job_attr(&mut self, name: &str, v: Value) -> &mut Self {
        self.job.push((name.into(), vec![v]));
        self
    }

    pub fn user(&mut self, user: &str) -> &mut Self {
        self.op_attr("requesting-user-name", Value::Text(NAME, user.chars().take(60).collect()))
    }

    pub fn job_name(&mut self, name: &str) -> &mut Self {
        self.op_attr("job-name", Value::Text(NAME, name.chars().take(120).collect()))
    }

    pub fn format(&mut self, mime: &str) -> &mut Self {
        self.op_attr("document-format", Value::Text(MIME, mime.into()))
    }

    pub fn copies(&mut self, n: i32) -> &mut Self {
        self.job_attr("copies", Value::Int(INTEGER, n))
    }

    pub fn keyword(&mut self, name: &str, value: &str) -> &mut Self {
        self.job_attr(name, Value::Text(KEYWORD, value.into()))
    }

    pub fn encode(&self) -> Vec<u8> {
        let mut out = vec![2, 0];
        out.extend_from_slice(&self.op.to_be_bytes());
        out.extend_from_slice(&self.id.to_be_bytes());
        for (tag, group) in [(OPERATION, &self.operation), (JOB, &self.job)] {
            if group.is_empty() {
                continue;
            }
            out.push(tag);
            for (name, values) in group {
                for (i, v) in values.iter().enumerate() {
                    // weitere Werte desselben Attributs: leerer Name
                    let n = if i == 0 { name.as_bytes() } else { &[][..] };
                    let (vtag, bytes) = match v {
                        Value::Text(t, s) => (*t, s.as_bytes().to_vec()),
                        Value::Int(t, x) => (*t, x.to_be_bytes().to_vec()),
                    };
                    out.push(vtag);
                    out.extend_from_slice(&(n.len() as u16).to_be_bytes());
                    out.extend_from_slice(n);
                    out.extend_from_slice(&(bytes.len() as u16).to_be_bytes());
                    out.extend_from_slice(&bytes);
                }
            }
        }
        out.push(END);
        out
    }
}

/// Antwort: Statuscode und alle Attribute als Text (Zahlen als Dezimalzahl).
#[derive(Debug, Default)]
pub struct Response {
    pub status: u16,
    pub attrs: HashMap<String, Vec<String>>,
}

impl Response {
    pub fn ok(&self) -> bool {
        self.status < 0x0100
    }

    pub fn first(&self, name: &str) -> Option<&str> {
        self.attrs.get(name).and_then(|v| v.first()).map(String::as_str)
    }

    pub fn all(&self, name: &str) -> Vec<String> {
        self.attrs.get(name).cloned().unwrap_or_default()
    }

    pub fn parse(data: &[u8]) -> Result<Response, String> {
        if data.len() < 9 {
            return Err("Antwort des Druckers ist zu kurz.".into());
        }
        let mut r = Response { status: u16::from_be_bytes([data[2], data[3]]), ..Default::default() };
        let mut i = 8;
        let mut last = String::new();
        while i < data.len() {
            let tag = data[i];
            i += 1;
            if tag == END {
                break;
            }
            if tag < 0x10 {
                continue; // Beginn einer Gruppe
            }
            let take = |i: &mut usize, n: usize| -> Result<&[u8], String> {
                let s = data.get(*i..*i + n).ok_or("Antwort des Druckers ist unvollständig.")?;
                *i += n;
                Ok(s)
            };
            let nlen = u16::from_be_bytes(take(&mut i, 2)?.try_into().unwrap()) as usize;
            let name = String::from_utf8_lossy(take(&mut i, nlen)?).into_owned();
            let vlen = u16::from_be_bytes(take(&mut i, 2)?.try_into().unwrap()) as usize;
            let raw = take(&mut i, vlen)?;
            let value = match tag {
                INTEGER | ENUM if raw.len() == 4 => i32::from_be_bytes(raw.try_into().unwrap()).to_string(),
                0x22 => (raw.first() == Some(&1)).to_string(),
                _ => String::from_utf8_lossy(raw).into_owned(),
            };
            if !name.is_empty() {
                last = name;
            }
            r.attrs.entry(last.clone()).or_default().push(value);
        }
        Ok(r)
    }
}

/// `ipp://host/ipp/print` → `http://host:631/ipp/print` (ipps → https, Port 443 bzw. 631).
pub fn http_url(printer: &str) -> Result<String, String> {
    let p = printer.trim();
    let (scheme, rest) = if let Some(r) = p.strip_prefix("ipps://") {
        ("https", r)
    } else if let Some(r) = p.strip_prefix("ipp://") {
        ("http", r)
    } else if let Some(r) = p.strip_prefix("https://") {
        ("https", r)
    } else if let Some(r) = p.strip_prefix("http://") {
        ("http", r)
    } else {
        ("http", p)
    };
    let (host, path) = match rest.find('/') {
        Some(i) => (&rest[..i], &rest[i..]),
        None => (rest, "/ipp/print"),
    };
    if host.is_empty() || host.contains(char::is_whitespace) {
        return Err("Ungültige Drucker-Adresse.".into());
    }
    let host = if host.contains(':') && !host.ends_with(']') { host.to_string() } else { format!("{host}:631") };
    Ok(format!("{scheme}://{host}{path}"))
}

/// `http(s)://host:631/pfad` → die URI, die der Drucker in `printer-uri` erwartet.
pub fn printer_uri(http: &str) -> String {
    http.replacen("https://", "ipps://", 1).replacen("http://", "ipp://", 1)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encodes_and_parses() {
        let mut r = Request::new(GET_PRINTER_ATTRIBUTES, "ipp://p:631/ipp/print");
        r.op_keywords("requested-attributes", &["printer-name", "document-format-supported"]);
        let b = r.encode();
        assert_eq!(&b[..4], &[2, 0, 0, 0x0B]);
        assert_eq!(*b.last().unwrap(), END);

        // Antwort von Hand: Status 0, ein Name, zwei Formate, ein Zustand
        let mut resp = vec![2, 0, 0, 0, 0, 0, 0, 1, 0x04];
        let mut attr = |tag: u8, name: &str, val: &[u8]| {
            resp.push(tag);
            resp.extend_from_slice(&(name.len() as u16).to_be_bytes());
            resp.extend_from_slice(name.as_bytes());
            resp.extend_from_slice(&(val.len() as u16).to_be_bytes());
            resp.extend_from_slice(val);
        };
        attr(NAME, "printer-name", b"Buero");
        attr(MIME, "document-format-supported", b"application/pdf");
        attr(MIME, "", b"image/jpeg");
        attr(ENUM, "printer-state", &3i32.to_be_bytes());
        resp.push(END);
        let p = Response::parse(&resp).unwrap();
        assert!(p.ok());
        assert_eq!(p.first("printer-name"), Some("Buero"));
        assert_eq!(p.all("document-format-supported"), ["application/pdf", "image/jpeg"]);
        assert_eq!(p.first("printer-state"), Some("3"));
    }

    #[test]
    fn urls() {
        assert_eq!(http_url("ipp://192.168.1.5/ipp/print").unwrap(), "http://192.168.1.5:631/ipp/print");
        assert_eq!(http_url("ipps://drucker.local").unwrap(), "https://drucker.local:631/ipp/print");
        assert_eq!(http_url("192.168.1.5:8631/ipp").unwrap(), "http://192.168.1.5:8631/ipp");
        assert_eq!(http_url("http://cups:631/printers/HP").unwrap(), "http://cups:631/printers/HP");
        assert_eq!(printer_uri("http://a:631/ipp/print"), "ipp://a:631/ipp/print");
        assert!(http_url(" ").is_err());
    }
}
