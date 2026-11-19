//! Web-App-Manifest, unter `/manifest.webmanifest` ausgeliefert.

pub const SRC: &str = r##"{
  "id": "/",
  "name": "nrzPaper",
  "short_name": "nrzPaper",
  "description": "Dokumente scannen, ablegen und finden",
  "start_url": "/",
  "scope": "/",
  "display": "standalone",
  "orientation": "any",
  "background_color": "#121212",
  "theme_color": "#121212",
  "icons": [
    { "src": "/static/icon-192.png", "sizes": "192x192", "type": "image/png", "purpose": "any" },
    { "src": "/static/icon-512.png", "sizes": "512x512", "type": "image/png", "purpose": "any" }
  ],
  "share_target": {
    "action": "/share-target",
    "method": "POST",
    "enctype": "multipart/form-data",
    "params": {
      "title": "title",
      "text": "text",
      "files": [
        { "name": "file", "accept": ["application/pdf", ".pdf", "image/*", ".jpg", ".jpeg", ".png", ".heic"] }
      ]
    }
  }
}
"##;
