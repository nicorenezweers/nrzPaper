// nrzPaper – kleine Helfer für die server-seitig gerenderten Seiten.

const $ = (sel, root = document) => root.querySelector(sel);
const $$ = (sel, root = document) => [...root.querySelectorAll(sel)];

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

// Vorschau-Fenster (<button data-preview="…/file" data-preview-kind="pdf|image" data-preview-name="…">):
// grosse Ansicht mit Zoom, ohne die Liste zu verlassen. Esc schliesst, +/− und Strg/⌘+Mausrad zoomen.
let pdfjsLib = null;
async function openPreview(btn) {
  const url = btn.dataset.preview, kind = btn.dataset.previewKind, name = btn.dataset.previewName || "";
  const docUrl = url.replace(/\/file$/, "");
  const ui = document.createElement("div");
  ui.className = "fixed inset-0 z-50 flex flex-col bg-zinc-950/95 backdrop-blur-sm";
  ui.innerHTML = `
    <div class="flex items-center gap-2 border-b border-zinc-800 bg-zinc-900/90 px-4 py-2.5 text-sm text-zinc-300">
      <span class="min-w-0 flex-1 truncate font-medium text-zinc-100"></span>
      <button data-z="-1" title="Verkleinern (−)" class="rounded-md px-2.5 py-1 hover:bg-zinc-800">−</button>
      <button data-z="fit" title="Breite anpassen" class="min-w-16 rounded-md px-2 py-1 tabular-nums hover:bg-zinc-800" data-zl>100 %</button>
      <button data-z="1" title="Vergrössern (+)" class="rounded-md px-2.5 py-1 hover:bg-zinc-800">+</button>
      <span class="mx-1 h-5 w-px bg-zinc-800"></span>
      <a href="${docUrl}" class="rounded-md px-2.5 py-1 hover:bg-zinc-800">Öffnen</a>
      <a href="${url}?download=true" class="rounded-md px-2.5 py-1 hover:bg-zinc-800">Herunterladen</a>
      <button data-close title="Schliessen (Esc)" class="ml-1 grid h-8 w-8 place-items-center rounded-md text-lg hover:bg-zinc-800">×</button>
    </div>
    <div data-pv-scroll class="flex-1 overflow-auto px-6 py-6"><div data-pv-pages class="mx-auto flex w-max flex-col items-center gap-5"></div></div>`;
  $("span", ui).textContent = name;
  document.body.append(ui);
  document.body.style.overflow = "hidden";
  const scroller = $("[data-pv-scroll]", ui), pagesEl = $("[data-pv-pages]", ui), label = $("[data-zl]", ui);
  let scale = null, pdf = null, img = null, natural = 1, token = 0;

  const close = () => { ui.remove(); document.body.style.overflow = ""; document.removeEventListener("keydown", onKey); };
  const onKey = (e) => {
    if (e.key === "Escape") close();
    else if (e.key === "+" || e.key === "=") zoom(1.2);
    else if (e.key === "-") zoom(1 / 1.2);
  };
  document.addEventListener("keydown", onKey);
  $("[data-close]", ui).onclick = close;
  ui.addEventListener("click", (e) => { if (e.target === scroller) close(); });
  scroller.addEventListener("wheel", (e) => { if (e.ctrlKey || e.metaKey) { e.preventDefault(); zoom(e.deltaY < 0 ? 1.1 : 1 / 1.1); } }, { passive: false });
  $$("[data-z]", ui).forEach((b) => b.onclick = () => b.dataset.z === "fit" ? fit() : zoom(b.dataset.z === "1" ? 1.2 : 1 / 1.2));

  const draw = async () => {
    label.textContent = Math.round(scale * 100) + " %";
    if (img) { img.style.width = natural * scale + "px"; return; }
    if (!pdf) return;
    const mine = ++token;
    pagesEl.replaceChildren();
    const ratio = window.devicePixelRatio || 1;
    for (let n = 1; n <= Math.min(pdf.numPages, 50); n++) {
      if (mine !== token) return;
      const page = await pdf.getPage(n);
      const vp = page.getViewport({ scale });
      const canvas = Object.assign(document.createElement("canvas"), { width: Math.floor(vp.width * ratio), height: Math.floor(vp.height * ratio) });
      canvas.style.width = vp.width + "px";
      canvas.style.height = vp.height + "px";
      canvas.className = "rounded-sm bg-white shadow-2xl shadow-black/60";
      pagesEl.append(canvas);
      await page.render({ canvasContext: canvas.getContext("2d"), viewport: vp, transform: ratio !== 1 ? [ratio, 0, 0, ratio, 0, 0] : null }).promise;
    }
  };
  let timer;
  const zoom = (f) => { scale = Math.min(5, Math.max(0.2, scale * f)); clearTimeout(timer); timer = setTimeout(draw, img ? 0 : 120); label.textContent = Math.round(scale * 100) + " %"; };
  const fit = async () => {
    const avail = Math.min(scroller.clientWidth - 48, 1100);
    if (img) scale = Math.min(avail / natural, 1.5);
    else { const p = await pdf.getPage(1); scale = avail / p.getViewport({ scale: 1 }).width; }
    draw();
  };

  try {
    if (kind === "image") {
      img = Object.assign(document.createElement("img"), { src: url, alt: name, className: "rounded-sm shadow-2xl shadow-black/60" });
      pagesEl.append(img);
      await img.decode();
      natural = img.naturalWidth;
      await fit();
    } else if (kind === "pdf") {
      pdfjsLib ||= await import("/static/vendor/pdfjs/pdf.min.mjs");
      pdfjsLib.GlobalWorkerOptions.workerSrc = "/static/vendor/pdfjs/pdf.worker.min.mjs";
      pdf = await pdfjsLib.getDocument({ url }).promise;
      await fit();
    } else {
      pagesEl.innerHTML = `<p class="py-20 text-zinc-400">Keine Vorschau für diesen Dateityp – bitte öffnen oder herunterladen.</p>`;
    }
  } catch (e) {
    pagesEl.innerHTML = `<p class="py-20 text-zinc-400">Vorschau nicht möglich: ${e.message}</p>`;
  }
}
document.addEventListener("click", (e) => {
  const btn = e.target.closest("[data-preview]");
  if (btn) { e.preventDefault(); openPreview(btn); }
});

// Dokumentseite: Angaben ein-/ausblenden (gemerkt im Browser)
const sideToggle = $("[data-toggle-side]");
if (sideToggle) {
  const grid = $("[data-detail-grid]");
  const apply = (hidden) => {
    grid.classList.toggle("side-hidden", hidden);
    $("[data-side-label]", sideToggle).textContent = hidden ? "Angaben einblenden" : "Angaben ausblenden";
  };
  let hidden = false;
  try { hidden = localStorage.getItem("nrz-side-hidden") === "1"; } catch {}
  apply(hidden);
  sideToggle.onclick = () => {
    hidden = !hidden;
    try { localStorage.setItem("nrz-side-hidden", hidden ? "1" : "0"); } catch {}
    apply(hidden);
  };
}
