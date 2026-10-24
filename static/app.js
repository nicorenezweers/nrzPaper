// nrzPaper – kleine Helfer für die server-seitig gerenderten Seiten.

const $ = (sel, root = document) => root.querySelector(sel);

// Zeitpunkte in der Zeitzone des Browsers anzeigen (<time data-ts="Unix-Sekunden">)
function localizeTimes(root = document) {
  const fmt = new Intl.DateTimeFormat("de-CH", { dateStyle: "medium", timeStyle: "short" });
  const fmtDate = new Intl.DateTimeFormat("de-CH", { dateStyle: "medium" });
  root.querySelectorAll("time[data-ts]").forEach((t) => {
    const d = new Date(Number(t.dataset.ts) * 1000);
    t.dateTime = d.toISOString();
    t.textContent = (t.dataset.dateOnly !== undefined ? fmtDate : fmt).format(d);
    t.title = d.toLocaleString("de-CH");
  });
}
localizeTimes();

// Kurze Meldungen unten rechts
function toast(text, kind = "") {
  let box = $("#toasts");
  if (!box) {
    box = Object.assign(document.createElement("div"), { id: "toasts", className: "fixed bottom-4 right-4 z-50 flex max-w-sm flex-col gap-2" });
    document.body.append(box);
  }
  const el = document.createElement("div");
  el.className = "rounded-lg border px-4 py-3 text-sm shadow-2xl shadow-black/50 " +
    (kind === "err" ? "border-zinc-600 bg-zinc-800 text-zinc-100" : "border-zinc-700 bg-zinc-900 text-zinc-200");
  el.textContent = text;
  box.append(el);
  setTimeout(() => el.remove(), kind === "err" ? 7000 : 3500);
}
window.nrzToast = toast;

// Relative Links als volle Adresse zeigen (<input data-absolute value="/share/…">)
document.querySelectorAll("input[data-absolute]").forEach((i) => { i.value = new URL(i.value, location.origin).href; });

// Nach einem Formular, das ein Geheimnis zeigt: Adresse auf die Seite selbst zurücksetzen,
// damit Neuladen nichts erneut absendet (<div data-clean-url="/…">)
const clean = $("[data-clean-url]");
if (clean) history.replaceState(null, "", clean.dataset.cleanUrl);

// In die Zwischenablage (<button data-copy="#feld">)
document.addEventListener("click", async (e) => {
  const btn = e.target.closest("[data-copy]");
  if (!btn) return;
  const field = $(btn.dataset.copy);
  try {
    await navigator.clipboard.writeText(field.value);
    toast("Kopiert.");
  } catch {
    field.select();
    toast("Bitte mit ⌘C kopieren.", "err");
  }
});

// Rückfrage vor gefährlichen Formularen (<form data-confirm="…">)
document.addEventListener("submit", (e) => {
  const msg = e.target.dataset?.confirm;
  if (msg && !confirm(msg)) e.preventDefault();
});

// Live-Suche: Ergebnisse ohne Neuladen tauschen (<input data-live-search data-target="#…">)
const live = $("[data-live-search]");
if (live) {
  const target = $(live.dataset.target);
  let timer, controller;
  const run = async () => {
    const url = new URL(location.href);
    url.searchParams.set("q", live.value);
    url.searchParams.delete("page");
    history.replaceState(null, "", url);
    url.searchParams.set("partial", "true");
    controller?.abort();
    controller = new AbortController();
    try {
      const res = await fetch(url, { signal: controller.signal, headers: { Accept: "text/html" } });
      target.innerHTML = await res.text();
      localizeTimes(target);
    } catch (e) { if (e.name !== "AbortError") toast("Suche fehlgeschlagen.", "err"); }
  };
  live.addEventListener("input", () => { clearTimeout(timer); timer = setTimeout(run, 220); });
  live.addEventListener("keydown", (e) => { if (e.key === "Enter") { e.preventDefault(); clearTimeout(timer); run(); } });
}

// Upload per Drag & Drop oder Knopf (<div data-upload="/organizations/…/documents">)
const upload = $("[data-upload]");
if (upload) {
  const endpoint = upload.dataset.upload;
  const overlay = $("[data-drop-overlay]");
  const input = $("[data-file-input]");
  const bar = $("[data-progress]");
  let depth = 0;

  const send = (files) => {
    if (!files.length) return;
    const form = new FormData();
    [...files].forEach((f) => form.append("file", f, f.name));
    const xhr = new XMLHttpRequest();
    xhr.open("POST", endpoint);
    xhr.setRequestHeader("Accept", "application/json");
    bar.hidden = false;
    xhr.upload.onprogress = (e) => { if (e.lengthComputable) bar.firstElementChild.style.width = (e.loaded / e.total) * 100 + "%"; };
    xhr.onload = () => {
      bar.hidden = true;
      let data = {};
      try { data = JSON.parse(xhr.responseText); } catch {}
      (data.documents || []).forEach((d) => toast(d.restored ? `„${d.name}“ aus dem Papierkorb geholt` : `„${d.name}“ hochgeladen – Texterkennung läuft`));
      (data.errors || []).forEach((e) => toast(`${e.file}: ${e.error}`, "err"));
      if (data.error) toast(data.error.message, "err");
      if ((data.documents || []).length) setTimeout(() => location.reload(), 600);
    };
    xhr.onerror = () => { bar.hidden = true; toast("Upload fehlgeschlagen.", "err"); };
    xhr.send(form);
  };

  input?.addEventListener("change", () => send(input.files));
  window.addEventListener("dragenter", (e) => { if (e.dataTransfer?.types.includes("Files")) { depth++; overlay.hidden = false; } });
  window.addEventListener("dragleave", () => { if (--depth <= 0) { depth = 0; overlay.hidden = true; } });
  window.addEventListener("dragover", (e) => e.preventDefault());
  window.addEventListener("drop", (e) => { e.preventDefault(); depth = 0; overlay.hidden = true; send(e.dataTransfer.files); });
}

// Seite neu laden, solange noch Texterkennung läuft (<body data-poll>)
if (document.body.dataset.poll) setTimeout(() => location.reload(), Number(document.body.dataset.poll));
