//! Handy-Web-App: Kamera-Scan und Teilen, unter `/static/mobile.js` ausgeliefert.

pub const SRC: &str = r#"// nrzPaper – Handy-Helfer für die Oberfläche (per import() aus der Dioxus-App geladen).
//
// registerServiceWorker(): Service Worker für das Teilen-Ziel (Android) anmelden.
// takeShared(org):         geteilte Dateien aus dem Cache in die Inbox laden.
// uploadPhotos(id, url):   Fotos aus <input id=…> drehen (EXIF), verkleinern, als JPEG senden.

export function registerServiceWorker() {
  if ('serviceWorker' in navigator && window.isSecureContext) {
    navigator.serviceWorker.register('/sw.js', { scope: '/' }).catch(() => {});
  }
}

/// Liefert { count, error, items } – count 0, wenn nichts geteilt wurde.
export async function takeShared(org) {
  const failed = new URLSearchParams(location.search).get('shared') === 'failed';
  if (failed) history.replaceState(null, '', location.pathname);
  if (!('caches' in window)) return { count: 0, failed };
  const cache = await caches.open('nrz-share');
  const keys = await cache.keys();
  if (!keys.length) return { count: 0, failed };
  const form = new FormData();
  for (const req of keys) {
    const res = await cache.match(req);
    if (!res) continue;
    const name = decodeURIComponent(res.headers.get('X-File-Name') || 'Dokument');
    form.append('file', await res.blob(), name);
  }
  // Erst löschen: lieber einmal verlieren als dieselben Dateien bei jedem Öffnen erneut senden
  for (const req of keys) await cache.delete(req);
  try {
    const res = await fetch(`/api/v1/organizations/${org}/review/import`, {
      method: 'POST', body: form, headers: { Accept: 'application/json' }, credentials: 'same-origin',
    });
    const body = await res.json().catch(() => ({}));
    if (!res.ok) return { count: 0, error: (body.error && body.error.message) || String(res.status) };
    return { count: (body.items || []).length, items: (body.items || []).map((i) => i.id) };
  } catch (e) {
    return { count: 0, error: String(e) };
  }
}

const MAX_SIDE = 2400;

/// Ein Bild → JPEG-Blob, richtig gedreht und höchstens MAX_SIDE Pixel lang.
async function toJpeg(file) {
  let bitmap;
  try {
    bitmap = await createImageBitmap(file, { imageOrientation: 'from-image' });
  } catch (e) {
    // Ältere Safaris: über <img>, das dreht nach EXIF von selbst
    bitmap = await new Promise((resolve, reject) => {
      const img = new Image();
      img.onload = () => resolve(img);
      img.onerror = reject;
      img.src = URL.createObjectURL(file);
    });
  }
  const w = bitmap.width, h = bitmap.height;
  const scale = Math.min(1, MAX_SIDE / Math.max(w, h));
  const canvas = document.createElement('canvas');
  canvas.width = Math.round(w * scale);
  canvas.height = Math.round(h * scale);
  canvas.getContext('2d').drawImage(bitmap, 0, 0, canvas.width, canvas.height);
  if (bitmap.close) bitmap.close();
  return await new Promise((resolve) => canvas.toBlob(resolve, 'image/jpeg', 0.85));
}

/// Fotos aus dem Eingabefeld hochladen. Gibt die Antwort der API zurück oder { error }.
export async function uploadPhotos(inputId, url) {
  const input = document.getElementById(inputId);
  const files = input ? Array.from(input.files || []) : [];
  if (input) input.value = '';
  if (!files.length) return { empty: true };
  const form = new FormData();
  let i = 0;
  for (const f of files) {
    i++;
    let blob = f;
    try { blob = (await toJpeg(f)) || f; } catch (e) { blob = f; }
    form.append('file', blob, `seite-${i}.jpg`);
  }
  try {
    const res = await fetch(url, { method: 'POST', body: form, headers: { Accept: 'application/json' }, credentials: 'same-origin' });
    const body = await res.json().catch(() => ({}));
    if (!res.ok) return { error: (body.error && body.error.message) || String(res.status) };
    return body;
  } catch (e) {
    return { error: String(e) };
  }
}

// Teilen einer Datei aus der App (Mail, WhatsApp, Dateien …) über das Teilen-Menü des Handys.
// navigator.share() verlangt einen frischen Tipp – darum die Datei vorher laden.
let prepared = null;

/// Datei vorbereiten; true, wenn das Gerät Dateien teilen kann (Handy, Tablet, Safari am Mac).
export async function prepareShare(url, name, type) {
  prepared = null;
  if (!navigator.canShare) return false;
  try {
    const probe = new File([new Blob([''], { type })], name, { type });
    if (!navigator.canShare({ files: [probe] })) return false;
  } catch (e) {
    return false;
  }
  try {
    const res = await fetch(url, { credentials: 'same-origin' });
    if (!res.ok) return false;
    const blob = await res.blob();
    prepared = new File([blob], name, { type: type || blob.type });
    return true;
  } catch (e) {
    return false;
  }
}

/// Teilen-Menü öffnen. Gibt "ok", "abgebrochen" oder eine Fehlermeldung zurück.
export async function shareFile(title) {
  if (!prepared) return 'nicht bereit';
  try {
    await navigator.share({ files: [prepared], title });
    return 'ok';
  } catch (e) {
    return e && e.name === 'AbortError' ? 'abgebrochen' : String(e && e.message || e);
  }
}
"#;
