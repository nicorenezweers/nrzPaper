// nrzPaper – Scannen und Prüfen (übernommen aus nrzDocsScanner).
// Scan-Seite: Seiten vom Scanner oder als Fotos, Lupe, Reihenfolge per Drag & Drop, Tags vorgeben.
// Prüf-Seite: Vorschläge der KI ansehen, ändern, freigeben → Dokument in nrzPaper.

const $ = (sel, root = document) => root.querySelector(sel);
const $$ = (sel, root = document) => [...root.querySelectorAll(sel)];
const esc = (s) => String(s ?? "").replace(/[&<>"']/g, (c) => ({ "&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;", "'": "&#39;" })[c]);
const toast = (text, kind) => window.nrzToast?.(text, kind);
const fmtDate = (ts) => new Date(ts * 1000).toLocaleString("de-CH", { dateStyle: "short", timeStyle: "short" });

async function api(method, path, body) {
  const opts = { method, headers: { Accept: "application/json" } };
  if (body instanceof FormData) opts.body = body;
  else if (body !== undefined) { opts.body = JSON.stringify(body); opts.headers["Content-Type"] = "application/json"; }
  const res = await fetch(path, opts);
  const data = await res.json().catch(() => ({}));
  if (!res.ok) throw new Error(data.error?.message || `Fehler ${res.status}`);
  return data;
}

/** Knopf sperren, solange die Aktion läuft; Fehler als Meldung. */
async function run(btn, fn) {
  if (btn) btn.disabled = true;
  try { await fn(); } catch (e) { toast(e.message, "err"); } finally { if (btn?.isConnected) btn.disabled = false; }
}

/** Lupe: beim Überfahren einer Seite vergrössert anzeigen. */
function attachLoupe(img) {
  const stage = img.closest("[data-stage]");
  if (!stage) return;
  const lens = document.createElement("div");
  lens.className = "loupe";
  stage.append(lens);
  const zoom = 2.6, half = 120;
  img.addEventListener("mousemove", (e) => {
    const r = img.getBoundingClientRect(), s = stage.getBoundingClientRect();
    const x = e.clientX - r.left, y = e.clientY - r.top;
    lens.style.display = "block";
    lens.style.left = `${e.clientX - s.left - half + stage.scrollLeft}px`;
    lens.style.top = `${e.clientY - s.top - half + stage.scrollTop}px`;
    lens.style.backgroundImage = `url("${img.src}")`;
    lens.style.backgroundSize = `${r.width * zoom}px ${r.height * zoom}px`;
    lens.style.backgroundPosition = `${half - x * zoom}px ${half - y * zoom}px`;
  });
  img.addEventListener("mouseleave", () => { lens.style.display = "none"; });
}

function chip(tag, on, extra = "") {
  return `<button type="button" data-tag="${esc(tag.name)}" title="${esc(tag.description || "")}"
    class="inline-flex items-center gap-1.5 rounded-full border px-2.5 py-1 text-xs ${on ? "border-zinc-300 bg-zinc-200 text-zinc-900" : "border-zinc-700 bg-zinc-800/60 text-zinc-300 hover:border-zinc-500"}">
    <span class="h-1.5 w-1.5 rounded-full" style="background:${esc(tag.color || "#71717a")}"></span>${esc(tag.name)}${extra}</button>`;
}

async function scannerStatus() {
  const box = $("[data-scanner-status]");
  if (!box) return;
  try {
    const s = await api("GET", "/scanner/status");
    $("[data-dot]", box).className = `h-2 w-2 rounded-full ${s.ok ? "bg-zinc-200" : "bg-zinc-700"}`;
    $("[data-label]", box).textContent = s.ok ? `Scanner bereit (${s.state})` : s.state;
  } catch {}
}

// ================================================================ Scannen

function scanPage(root) {
  const org = root.dataset.org;
  const base = `/organizations/${org}/scan`;
  const st = {
    session: { name: "", hintTags: [], pages: [] },
    tags: [],
    selected: null,
    scanning: false,
    color: (localStorage.getItem("nrz-scan-color") ?? root.dataset.color) === "true",
    dpi: Number(localStorage.getItem("nrz-scan-dpi") || root.dataset.dpi || 300),
  };
  const apply = (data) => { st.session = data.session; st.tags = data.tags; };

  const draw = () => {
    const s = st.session, n = s.pages.length;
    if (!s.pages.includes(st.selected)) st.selected = s.pages[n - 1] || null;
    const idx = s.pages.indexOf(st.selected);
    const box = $("[data-viewer-box]", root);
    box.innerHTML = n ? `
      <div data-stage class="relative grid flex-1 place-items-center overflow-auto bg-zinc-950 p-5">
        <img data-big src="${base}/pages/${st.selected}" alt="Seite ${idx + 1}" class="max-h-[62vh] rounded shadow-2xl shadow-black/60">
      </div>
      <div class="flex flex-wrap items-center justify-between gap-2 border-t border-zinc-800 px-4 py-2 text-sm">
        <span><span class="font-medium text-zinc-100">Seite ${idx + 1} von ${n}</span> <span class="text-zinc-500">· Lupe: Maus über die Seite · Reihenfolge: unten ziehen</span></span>
        <span class="flex gap-1">
          <button data-move="-1" ${idx === 0 ? "disabled" : ""} title="Nach vorne" class="rounded-md px-2.5 py-1 hover:bg-zinc-800 disabled:opacity-30">←</button>
          <button data-move="1" ${idx === n - 1 ? "disabled" : ""} title="Nach hinten" class="rounded-md px-2.5 py-1 hover:bg-zinc-800 disabled:opacity-30">→</button>
          <button data-remove-current class="rounded-md border border-zinc-700 px-2.5 py-1 text-zinc-400 hover:bg-zinc-800 hover:text-zinc-100">Seite entfernen</button>
        </span>
      </div>
      <div data-strip class="flex gap-3 overflow-x-auto border-t border-zinc-800 bg-zinc-900 p-3">
        ${s.pages.map((p, i) => `
          <div data-thumb="${p}" draggable="true" class="thumb group relative w-24 shrink-0 cursor-grab rounded-lg border-2 p-1 ${p === st.selected ? "border-zinc-300" : "border-transparent hover:border-zinc-700"}">
            <img src="${base}/pages/${p}" alt="" class="pointer-events-none aspect-[3/4] w-full rounded object-cover">
            <button data-remove="${p}" title="Seite ${i + 1} entfernen" class="absolute right-1.5 top-1.5 hidden h-5 w-5 place-items-center rounded-full bg-zinc-900/90 text-xs text-zinc-200 group-hover:grid">×</button>
            <span class="mt-1 block text-center text-[11px] text-zinc-500">Seite ${i + 1}</span>
          </div>`).join("")}
      </div>` : `
      <div class="grid flex-1 place-items-center p-10 text-center">
        <div>
          <p class="text-lg font-medium text-zinc-200">Noch keine Seiten</p>
          <p class="mt-2 text-sm text-zinc-500">Blatt aufs Glas legen und auf „Scannen“ drücken.<br>Mehrere Seiten einfach nacheinander scannen – alles wird ein PDF.</p>
        </div>
      </div>`;

    const btn = $("[data-scan]", root);
    btn.disabled = st.scanning;
    btn.innerHTML = st.scanning ? '<span class="h-5 w-5 animate-spin rounded-full border-2 border-zinc-500 border-t-zinc-900"></span>Scanne …' : n ? `Seite ${n + 1} scannen` : "Scannen";

    $("[data-actions]", root).innerHTML = n ? `
      <button data-finish class="w-full rounded-lg bg-zinc-200 px-4 py-2.5 font-semibold text-zinc-900 hover:bg-white">${n === 1 ? "1 Seite" : `${n} Seiten`} → zur Prüfung</button>
      <button data-discard class="w-full rounded-lg border border-zinc-700 px-4 py-2 text-sm text-zinc-400 hover:bg-zinc-800 hover:text-zinc-100">Alle Seiten verwerfen</button>` : "";

    const nameInput = $("[data-name]", root);
    if (document.activeElement !== nameInput) nameInput.value = s.name;

    $$("[data-color-seg] button", root).forEach((b) => b.className = `flex-1 px-3 py-1.5 ${String(st.color) === b.dataset.v ? "bg-zinc-200 text-zinc-900" : "text-zinc-400 hover:bg-zinc-800"}`);
    $("select[data-dpi]", root).value = String(st.dpi);
    drawHints();

    if (n) {
      attachLoupe($("[data-big]", root));
      $$("[data-move]", root).forEach((b) => b.onclick = () => move(st.selected, idx + Number(b.dataset.move)));
      $("[data-remove-current]", root).onclick = () => removePage(st.selected);
      setupStrip();
    }
    if ($("[data-finish]", root)) $("[data-finish]", root).onclick = (e) => run(e.target, finish);
    if ($("[data-discard]", root)) $("[data-discard]", root).onclick = (e) => {
      if (!confirm(`Alle ${n} Seiten verwerfen?`)) return;
      run(e.target, async () => { apply(await api("DELETE", `${base}/session`)); draw(); });
    };
  };

  const drawHints = () => {
    const box = $("[data-hints]", root);
    if (!st.tags.length) { box.innerHTML = `<span class="text-sm text-zinc-500">Noch keine Tags – unter <a class="underline" href="/organizations/${org}/tags">Tags</a> anlegen.</span>`; return; }
    box.innerHTML = st.tags.map((t) => chip(t, st.session.hintTags.includes(t.name))).join("");
    $$("[data-tag]", box).forEach((c) => c.onclick = () => {
      const name = c.dataset.tag, hints = st.session.hintTags;
      update({ hintTags: hints.includes(name) ? hints.filter((x) => x !== name) : [...hints, name] });
    });
  };

  const update = async (patch) => {
    try { apply(await api("PUT", `${base}/session`, patch)); draw(); } catch (e) { toast(e.message, "err"); }
  };

  const scanNow = async () => {
    if (st.scanning) return;
    st.scanning = true; draw();
    try {
      apply(await api("POST", `${base}/page`, { color: st.color, dpi: st.dpi }));
      st.selected = st.session.pages[st.session.pages.length - 1];
      toast(`Seite ${st.session.pages.length} gescannt.`);
    } catch (e) { toast(e.message, "err"); }
    st.scanning = false; draw(); scannerStatus();
  };

  const removePage = (id) => run(null, async () => { apply(await api("DELETE", `${base}/pages/${id}`)); draw(); });

  const move = async (id, to) => {
    const pages = st.session.pages.filter((p) => p !== id);
    pages.splice(Math.max(0, Math.min(to, pages.length)), 0, id);
    st.selected = id;
    await update({ pages });
  };

  /** Seiten-Leiste: anklicken, löschen und per Drag & Drop umsortieren. */
  const setupStrip = () => {
    const strip = $("[data-strip]", root);
    let dragId = null;
    $$("[data-thumb]", strip).forEach((el) => {
      el.onclick = (e) => {
        if (e.target.dataset.remove) { removePage(e.target.dataset.remove); return; }
        st.selected = el.dataset.thumb; draw();
      };
      el.ondragstart = (e) => { dragId = el.dataset.thumb; el.classList.add("opacity-40"); e.dataTransfer.effectAllowed = "move"; e.dataTransfer.setData("text/plain", dragId); };
      el.ondragend = () => { el.classList.remove("opacity-40"); $$("[data-thumb]", strip).forEach((t) => t.classList.remove("drop-before")); };
      el.ondragover = (e) => { e.preventDefault(); $$("[data-thumb]", strip).forEach((t) => t.classList.toggle("drop-before", t === el && t.dataset.thumb !== dragId)); };
      el.ondrop = (e) => {
        e.preventDefault(); e.stopPropagation();
        const id = dragId || e.dataTransfer.getData("text/plain");
        if (!id || id === el.dataset.thumb) return;
        move(id, st.session.pages.filter((p) => p !== id).indexOf(el.dataset.thumb));
      };
    });
    strip.ondragover = (e) => e.preventDefault();
    strip.ondrop = (e) => {
      if (e.target !== strip) return;
      const id = dragId || e.dataTransfer.getData("text/plain");
      if (id) move(id, st.session.pages.length);
    };
  };

  const finish = async () => {
    await api("PUT", `${base}/session`, { name: $("[data-name]", root).value.trim() });
    const { item } = await api("POST", `${base}/finish`);
    toast("Gespeichert – die KI liest das Dokument …");
    location.href = `/organizations/${org}/review?item=${item.id}`;
  };

  // Ohne Drag & Drop auf der ganzen Seite (app.js lädt sonst als normales Dokument hoch)
  $("[data-scan]", root).onclick = scanNow;
  let nameTimer;
  $("[data-name]", root).oninput = (e) => { clearTimeout(nameTimer); nameTimer = setTimeout(() => update({ name: e.target.value }), 400); };
  $$("[data-color-seg] button", root).forEach((b) => b.onclick = () => { st.color = b.dataset.v === "true"; localStorage.setItem("nrz-scan-color", st.color); draw(); });
  $("select[data-dpi]", root).onchange = (e) => { st.dpi = Number(e.target.value); localStorage.setItem("nrz-scan-dpi", st.dpi); };
  $("[data-photos]", root).onchange = (e) => {
    const form = new FormData();
    [...e.target.files].forEach((f) => form.append("file", f, f.name));
    e.target.value = "";
    run(null, async () => { apply(await api("POST", `${base}/photos`, form)); draw(); });
  };
  $("[data-pdfs]", root).onchange = (e) => {
    const form = new FormData();
    [...e.target.files].forEach((f) => form.append("file", f, f.name));
    e.target.value = "";
    run(null, async () => {
      const { items } = await api("POST", `/organizations/${org}/review/import`, form);
      toast(`${items.length} PDF zur Prüfung – die KI liest …`);
      location.href = `/organizations/${org}/review?item=${items[0].id}`;
    });
  };

  run(null, async () => { apply(await api("GET", `${base}/session`)); draw(); });
  scannerStatus();
  setInterval(scannerStatus, 30000);
}

// ================================================================ Prüfen

const STATUS = {
  analyse: ["KI liest …", "border-zinc-600 text-zinc-300"],
  pruefen: ["Prüfen", "border-zinc-300 bg-zinc-200 text-zinc-900"],
  import: ["Wird abgelegt …", "border-zinc-600 text-zinc-300"],
  fertig: ["Abgelegt", "border-zinc-800 text-zinc-500"],
  fehler: ["Fehler", "border-zinc-500 text-zinc-100"],
};
const badge = (status) => {
  const [label, cls] = STATUS[status] || [status, ""];
  return `<span class="shrink-0 rounded-full border px-2 py-0.5 text-[11px] ${cls}">${label}</span>`;
};
const spinner = '<span class="inline-block h-3.5 w-3.5 animate-spin rounded-full border-2 border-zinc-600 border-t-zinc-200"></span>';

function reviewPage(root) {
  const org = root.dataset.org;
  const base = `/organizations/${org}/review`;
  const st = { items: [], fields: [], tags: [], orgs: [], current: new URLSearchParams(location.search).get("item"), page: 1 };
  let pdfjs = null;

  const load = async () => {
    const d = await api("GET", `${base}/items`);
    st.items = d.items; st.fields = d.metaFields; st.tags = d.tags; st.orgs = d.organizations;
    const open = st.items.filter((i) => ["analyse", "pruefen", "fehler"].includes(i.status)).length;
    $$("[data-review-count]").forEach((b) => { b.textContent = open; b.hidden = !open; });
  };

  const drawQueue = () => {
    const open = st.items.filter((i) => i.status !== "fertig");
    const done = st.items.filter((i) => i.status === "fertig");
    if (!st.items.some((i) => i.id === st.current)) st.current = (open[0] || done[0])?.id || null;
    const row = (i) => `
      <button data-item="${i.id}" class="flex w-full items-start gap-2 rounded-lg px-2.5 py-2 text-left ${i.id === st.current ? "bg-zinc-800" : "hover:bg-zinc-800/50"}">
        <span class="mt-0.5 w-4 shrink-0">${["analyse", "import"].includes(i.status) ? spinner : ""}</span>
        <span class="min-w-0 flex-1"><span class="block truncate text-zinc-100">${esc(i.name)}</span><span class="block truncate text-xs text-zinc-500">${i.source === "mail" ? "✉ " : ""}${fmtDate(i.createdAt)}${i.pageCount ? ` · ${i.pageCount} S.` : ""}</span></span>
        ${badge(i.status)}
      </button>`;
    $("[data-queue]", root).innerHTML = `
      <p class="px-2.5 pb-1 pt-2 text-xs font-semibold uppercase tracking-wider text-zinc-500">Offen (${open.length})</p>
      ${open.map(row).join("") || '<p class="px-2.5 py-2 text-zinc-500">Nichts zu prüfen.</p>'}
      ${done.length ? `<p class="px-2.5 pb-1 pt-4 text-xs font-semibold uppercase tracking-wider text-zinc-500">Zuletzt abgelegt</p>${done.map(row).join("")}` : ""}
      <a href="/organizations/${org}/scan" class="mt-2 block rounded-lg px-2.5 py-2 text-zinc-400 hover:bg-zinc-800/50 hover:text-zinc-200">+ Scannen</a>`;
    $$("[data-item]", root).forEach((b) => b.onclick = () => { st.current = b.dataset.item; st.page = 1; history.replaceState(null, "", `?item=${st.current}`); drawQueue(); drawDetail(); });
  };

  /** Importierte PDFs ohne Seitenbilder: Seiten mit PDF.js zu Bildern machen (Lupe inklusive). */
  const renderPdf = async (box, item) => {
    try {
      pdfjs ||= await import("/static/vendor/pdfjs/pdf.min.mjs");
      pdfjs.GlobalWorkerOptions.workerSrc = "/static/vendor/pdfjs/pdf.worker.min.mjs";
      const pdf = await pdfjs.getDocument({ url: `${base}/${item.id}/pdf` }).promise;
      box.innerHTML = "";
      for (let n = 1; n <= Math.min(pdf.numPages, 12); n++) {
        const page = await pdf.getPage(n);
        const vp = page.getViewport({ scale: 1.6 });
        const canvas = Object.assign(document.createElement("canvas"), { width: vp.width, height: vp.height });
        await page.render({ canvasContext: canvas.getContext("2d"), viewport: vp }).promise;
        const img = Object.assign(document.createElement("img"), { src: canvas.toDataURL("image/jpeg", 0.85), alt: `Seite ${n}`, className: "mx-auto mb-4 max-w-full rounded shadow-2xl shadow-black/60" });
        box.append(img);
        attachLoupe(img);
      }
      if (pdf.numPages > 12) box.insertAdjacentHTML("beforeend", `<p class="text-center text-sm text-zinc-500">… und ${pdf.numPages - 12} weitere Seiten</p>`);
    } catch (e) { box.innerHTML = `<p class="p-6 text-sm text-zinc-400">Vorschau nicht möglich: ${esc(e.message)}</p>`; }
  };

  const drawDetail = () => {
    const box = $("[data-detail]", root);
    const item = st.items.find((i) => i.id === st.current);
    if (!item) {
      box.innerHTML = `<div class="grid min-h-[24rem] place-items-center rounded-xl border border-dashed border-zinc-800 p-10 text-center text-zinc-500">Alles erledigt. Neue Scans, Importe und E-Mail-Anhänge landen hier.</div>`;
      return;
    }
    const editable = item.status === "pruefen" || item.status === "fehler";
    const busy = item.status === "analyse" || item.status === "import";
    const page = Math.min(st.page, Math.max(item.pageCount, 1));
    const field = "w-full rounded-lg border border-zinc-800 bg-zinc-950 px-3 py-2 text-sm text-zinc-100 outline-none focus:border-zinc-600 disabled:text-zinc-400";

    let preview;
    if (item.status === "fertig") {
      preview = `<div class="grid h-full min-h-[20rem] place-items-center p-10 text-center">
        <div><p class="font-medium text-zinc-200">Abgelegt</p>
        ${item.documentId ? `<a href="/organizations/${item.organizationId}/documents/${item.documentId}" class="mt-3 inline-block rounded-lg bg-zinc-200 px-4 py-2 text-sm font-semibold text-zinc-900 hover:bg-white">Dokument öffnen</a>` : ""}</div></div>`;
    } else if (item.pageCount) {
      preview = `<div data-stage class="relative grid flex-1 place-items-center overflow-auto bg-zinc-950 p-5"><img data-big src="${base}/${item.id}/pages/${page}" alt="Seite ${page}" class="max-h-[70vh] rounded shadow-2xl shadow-black/60"></div>
        ${item.pageCount > 1 ? `<div class="flex gap-3 overflow-x-auto border-t border-zinc-800 p-3">${Array.from({ length: item.pageCount }, (_, i) => `
          <button data-page="${i + 1}" class="w-20 shrink-0 rounded-lg border-2 p-1 ${i + 1 === page ? "border-zinc-300" : "border-transparent hover:border-zinc-700"}"><img src="${base}/${item.id}/pages/${i + 1}" alt="" class="aspect-[3/4] w-full rounded object-cover"><span class="mt-1 block text-[11px] text-zinc-500">Seite ${i + 1}</span></button>`).join("")}</div>` : ""}`;
    } else {
      preview = `<div data-stage data-pdf-pages class="relative max-h-[78vh] flex-1 overflow-auto bg-zinc-950 p-5"><p class="text-center text-sm text-zinc-500">PDF wird geladen …</p></div>`;
    }

    box.innerHTML = `
      <div class="grid gap-5 xl:grid-cols-[minmax(0,1fr)_22rem]">
        <section class="flex min-h-[28rem] flex-col self-start overflow-hidden rounded-xl border border-zinc-800 bg-zinc-900 xl:sticky xl:top-4">${preview}</section>
        <div class="space-y-4 rounded-xl border border-zinc-800 bg-zinc-900/60 p-4">
          <div class="flex items-start justify-between gap-3"><h2 class="min-w-0 break-words font-semibold text-zinc-100">${esc(item.name)}</h2>${badge(item.status)}</div>
          ${item.sourceInfo ? `<p class="text-xs text-zinc-500">${item.source === "mail" ? "✉ " : ""}${esc(item.sourceInfo)}</p>` : ""}
          ${busy ? `<p class="flex items-center gap-2 rounded-lg border border-zinc-800 px-3 py-2 text-sm text-zinc-300">${spinner}${esc(item.step || "Bitte warten …")}</p>` : ""}
          ${item.error ? `<p class="rounded-lg border border-zinc-600 bg-zinc-800 px-3 py-2 text-sm text-zinc-100">${esc(item.error)}</p>` : ""}
          ${item.warning ? `<p class="rounded-lg border border-zinc-700 bg-zinc-800/60 px-3 py-2 text-sm text-zinc-300">${esc(item.warning)}</p>` : ""}
          <label class="block text-sm"><span class="mb-1 block text-zinc-500">Name</span><input data-f="name" value="${esc(item.name)}" ${editable ? "" : "disabled"} class="${field}"></label>
          <label class="block text-sm"><span class="mb-1 block text-zinc-500">Organisation</span>
            <select data-f="organizationId" ${editable ? "" : "disabled"} class="${field}">${st.orgs.map((o) => `<option value="${o.id}" ${o.id === item.organizationId ? "selected" : ""}>${esc(o.name)}</option>`).join("")}</select></label>
          <div><span class="mb-1.5 block text-sm text-zinc-500">Tags</span><div data-tags class="flex flex-wrap gap-1.5"></div>
            ${editable ? `<input data-newtag placeholder="Neuer Tag + Enter (wird beim Freigeben angelegt)" class="mt-2 ${field}">` : ""}</div>
          <div><span class="mb-1.5 block text-sm text-zinc-500">Metadaten aus dem Dokument</span>
            <div class="grid gap-2 sm:grid-cols-2">${st.fields.map((f) => `<label class="block text-xs"><span class="mb-0.5 block text-zinc-500">${esc(f.label)}</span><input data-meta="${f.key}" value="${esc(item.metadata[f.key] || "")}" ${editable ? "" : "disabled"} class="${field}"></label>`).join("")}</div></div>
          <label class="block text-sm"><span class="mb-1 block text-zinc-500">Zusammenfassung</span><textarea data-f="summary" rows="3" ${editable ? "" : "disabled"} class="${field}">${esc(item.summary)}</textarea></label>
          ${item.model ? `<p class="text-xs text-zinc-500">✦ gelesen von ${esc(item.model)}</p>` : ""}
          ${editable ? `<div class="flex flex-wrap gap-2">
            <button data-approve class="flex-1 rounded-lg bg-zinc-100 px-4 py-2.5 font-semibold text-zinc-900 hover:bg-white">Freigeben → ablegen</button>
            <button data-analyze class="rounded-lg bg-zinc-800 px-3 py-2 text-sm text-zinc-200 hover:bg-zinc-700">Neu analysieren</button>
            <button data-discard class="rounded-lg border border-zinc-700 px-3 py-2 text-sm text-zinc-400 hover:bg-zinc-800 hover:text-zinc-100">Verwerfen</button></div>` : ""}
          ${item.status === "fertig" ? `<button data-discard class="rounded-lg border border-zinc-700 px-3 py-2 text-sm text-zinc-400 hover:bg-zinc-800">Aus der Liste entfernen</button>` : ""}
        </div>
      </div>`;

    if ($("[data-big]", box)) attachLoupe($("[data-big]", box));
    if ($("[data-pdf-pages]", box)) renderPdf($("[data-pdf-pages]", box), item);
    $$("[data-page]", box).forEach((b) => b.onclick = () => { st.page = Number(b.dataset.page); drawDetail(); });

    const collect = () => ({
      name: $('[data-f="name"]', box).value,
      organizationId: $('[data-f="organizationId"]', box).value,
      tags: item.tags,
      metadata: Object.fromEntries($$("[data-meta]", box).map((i) => [i.dataset.meta, i.value.trim()])),
      summary: $('[data-f="summary"]', box).value,
    });
    let saveTimer;
    const save = () => {
      clearTimeout(saveTimer);
      saveTimer = setTimeout(() => api("PUT", `${base}/${item.id}`, collect()).then((r) => Object.assign(item, r.item)).catch((e) => toast(e.message, "err")), 500);
    };

    const drawTags = () => {
      const sameOrg = $('[data-f="organizationId"]', box).value === org;
      const all = sameOrg ? st.tags : [];
      const names = [...new Set([...all.map((t) => t.name), ...item.tags])];
      const tbox = $("[data-tags]", box);
      tbox.innerHTML = names.map((n) => {
        const t = all.find((x) => x.name.toLowerCase() === n.toLowerCase());
        const on = item.tags.some((x) => x.toLowerCase() === n.toLowerCase());
        return chip(t || { name: n, description: "neu – wird beim Freigeben angelegt" }, on, t ? "" : " ✱");
      }).join("") || '<span class="text-sm text-zinc-500">Keine Tags in dieser Organisation.</span>';
      if (editable) $$("[data-tag]", tbox).forEach((c) => c.onclick = () => {
        const n = c.dataset.tag;
        item.tags = item.tags.some((x) => x.toLowerCase() === n.toLowerCase()) ? item.tags.filter((x) => x.toLowerCase() !== n.toLowerCase()) : [...item.tags, n];
        save(); drawTags();
      });
      else $$("[data-tag]", tbox).forEach((c) => c.disabled = true);
    };
    drawTags();

    $$("[data-discard]", box).forEach((b) => b.onclick = () => {
      if (item.status !== "fertig" && !confirm(`„${item.name}“ verwerfen? Es wird nichts abgelegt.`)) return;
      run(b, async () => { await api("DELETE", `${base}/${item.id}`); st.current = null; await refresh(); });
    });
    if (!editable) return;

    $$("input, textarea", box).forEach((el) => { if (!el.matches("[data-newtag]")) el.oninput = save; });
    $('[data-f="organizationId"]', box).onchange = () => { item.tags = []; save(); drawTags(); };
    $("[data-newtag]", box).onkeydown = (e) => {
      if (e.key !== "Enter" || !e.target.value.trim()) return;
      e.preventDefault();
      const n = e.target.value.trim();
      if (!item.tags.some((x) => x.toLowerCase() === n.toLowerCase())) item.tags.push(n);
      e.target.value = "";
      save(); drawTags();
    };
    $("[data-approve]", box).onclick = (e) => run(e.target, async () => {
      clearTimeout(saveTimer);
      const body = collect();
      const r = await api("POST", `${base}/${item.id}/approve`, body);
      toast(`„${body.name}“ abgelegt – Texterkennung läuft.`);
      await load();
      const next = st.items.find((i) => i.status === "pruefen" && i.id !== item.id);
      if (next) { st.current = next.id; st.page = 1; drawQueue(); drawDetail(); }
      else location.href = r.url;
    });
    $("[data-analyze]", box).onclick = (e) => run(e.target, async () => {
      clearTimeout(saveTimer);
      await api("PUT", `${base}/${item.id}`, collect());
      await api("POST", `${base}/${item.id}/analyze`);
      await refresh();
    });
  };

  const refresh = async () => { await load(); drawQueue(); drawDetail(); };

  run(null, refresh);
  // Solange die KI liest oder abgelegt wird, regelmässig nachsehen.
  setInterval(async () => {
    if (!st.items.some((i) => i.status === "analyse" || i.status === "import")) return;
    const before = JSON.stringify(st.items.map((i) => [i.id, i.status]));
    try { await load(); } catch { return; }
    if (JSON.stringify(st.items.map((i) => [i.id, i.status])) !== before) {
      drawQueue();
      const editing = document.activeElement?.closest("[data-detail]");
      if (!editing) drawDetail();
    }
  }, 2500);
}

// ================================================================ Konto: Scanner suchen, KI testen

function accountTools() {
  const find = $("[data-discover]");
  if (find) find.onclick = () => run(find, async () => {
    const out = $("[data-discover-result]");
    out.innerHTML = `<span class="flex items-center gap-2 text-zinc-400">${spinner} Suche im Netz …</span>`;
    const r = await api("POST", "/scanner/discover", { subnets: $("[data-subnets]")?.value || "" });
    out.innerHTML = r.scanners.length
      ? r.scanners.map((s) => `<button type="button" data-pick="${esc(s.url)}" class="flex w-full items-center justify-between rounded-lg border border-zinc-800 px-3 py-2 text-left hover:border-zinc-600"><span class="text-zinc-100">${esc(s.model)}</span><span class="font-mono text-xs text-zinc-500">${esc(s.url)}</span></button>`).join("")
      : `<span class="text-zinc-400">Kein Scanner gefunden in ${esc(r.subnets.join(", "))}.</span>`;
    $$("[data-pick]", out).forEach((b) => b.onclick = () => { $("[data-scanner-url]").value = b.dataset.pick; toast("Übernommen – jetzt speichern."); });
  });
  const test = $("[data-ai-test]");
  if (test) test.onclick = () => run(test, async () => {
    const out = $("[data-ai-result]");
    out.innerHTML = `<span class="flex items-center gap-2 text-zinc-400">${spinner} Frage die KI …</span>`;
    try {
      const r = await api("POST", "/ai/test");
      out.innerHTML = `<span class="text-zinc-200">Funktioniert – ${esc(r.model)} las „${esc(r.title)}“.</span>`;
    } catch (e) { out.innerHTML = `<span class="text-zinc-300">${esc(e.message)}</span>`; }
  });
  scannerStatus();
}

// ================================================================ Konto: E-Mail-Eingang

function mailTools() {
  $$("[data-preset]").forEach((b) => b.onclick = () => {
    const form = b.closest("form");
    $("[data-mail-host]", form).value = b.dataset.preset;
    $("[data-mail-port]", form).value = 993;
    $("[data-mail-security]", form).value = "tls";
  });
  const act = (attr, path, ok) => $$(`[${attr}]`).forEach((b) => b.onclick = () => run(b, async () => {
    const id = b.getAttribute(attr), out = $(`[data-mail-result="${id}"]`);
    out.innerHTML = `<span class="flex items-center gap-2">${spinner} Verbinde …</span>`;
    try { out.textContent = ok(await api("POST", `/mail/${id}/${path}`)); }
    catch (e) { out.textContent = e.message; }
  }));
  act("data-mail-test", "test", () => "Verbindung und Anmeldung in Ordnung.");
  act("data-mail-fetch", "fetch", (r) => r.report.documents
    ? `${r.report.documents} Dokument(e) aus ${r.report.messages} Mail(s) in die Inbox gelegt.`
    : r.report.messages ? `${r.report.messages} neue Mail(s), aber keine PDFs oder Bilder.` : "Keine neuen Mails.");
}

const scanRoot = $("[data-scan-page]");
if (scanRoot) scanPage(scanRoot);
const reviewRoot = $("[data-review-page]");
if (reviewRoot) reviewPage(reviewRoot);
if ($("[data-account-scan]")) accountTools();
if ($("[data-account-mail]")) mailTools();
