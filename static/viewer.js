// nrzPaper – Dokument-Vorschau. PDF über PDF.js (lokal, Range-Anfragen auf die verschlüsselte
// Datenbank), Bilder mit Zoom/Drehen, Text als Text. Erwartet ein Element mit data-viewer.
import * as pdfjsLib from "/static/vendor/pdfjs/pdf.min.mjs";

pdfjsLib.GlobalWorkerOptions.workerSrc = "/static/vendor/pdfjs/pdf.worker.min.mjs";

const $ = (sel, root = document) => root.querySelector(sel);
const ZOOMS = [0.25, 0.5, 0.75, 1, 1.25, 1.5, 2, 3, 4];
const fold = (s) => s.normalize("NFD").replace(/[̀-ͯ]/g, "").toLowerCase();

// Nur den Container scrollen (scrollIntoView würde auch die Seite verschieben).
function scrollWithin(container, el, block = "start") {
  const c = container.getBoundingClientRect();
  const r = el.getBoundingClientRect();
  const top = r.top - c.top + container.scrollTop;
  if (block === "nearest") {
    if (r.top < c.top) container.scrollTop = top - 8;
    else if (r.bottom > c.bottom) container.scrollTop = top - c.height + r.height + 8;
  } else if (block === "center") {
    container.scrollTop = top - (c.height - r.height) / 2;
  } else {
    container.scrollTop = top - 16;
  }
}

for (const root of document.querySelectorAll("[data-viewer]")) {
  const kind = root.dataset.kind;
  if (kind === "pdf") pdfViewer(root);
  else if (kind === "image") imageViewer(root);
}

// ------------------------------------------------------------------ PDF

async function pdfViewer(root) {
  const url = root.dataset.url;
  const pagesEl = $("[data-pages]", root);
  const scroller = $("[data-scroller]", root);
  const thumbsEl = $("[data-thumbs]", root);
  const pageInput = $("[data-page-input]", root);
  const pageCount = $("[data-page-count]", root);
  const zoomSelect = $("[data-zoom]", root);
  const searchInput = $("[data-search]", root);
  const searchInfo = $("[data-search-info]", root);
  const status = $("[data-status]", root);

  const state = { pdf: null, zoom: "width", rotation: 0, current: 1, pages: [], hits: [], hitIndex: -1, texts: null };

  let pdf;
  try {
    pdf = await pdfjsLib.getDocument({
      url,
      withCredentials: true,
      rangeChunkSize: 1024 * 1024,
      isEvalSupported: false, // Content-Security-Policy erlaubt kein eval
    }).promise;
  } catch (e) {
    status.textContent = "Die Vorschau konnte nicht geladen werden: " + e.message;
    status.hidden = false;
    return;
  }
  state.pdf = pdf;
  pageCount.textContent = pdf.numPages;
  pageInput.max = pdf.numPages;

  // Platzhalter für alle Seiten (Grösse der ersten Seite), gerendert wird erst beim Sichtbarwerden.
  const first = await pdf.getPage(1);
  const base = first.getViewport({ scale: 1 });
  for (let n = 1; n <= pdf.numPages; n++) {
    const el = document.createElement("div");
    el.className = "pdf-page";
    el.dataset.page = n;
    el.innerHTML = `<div class="placeholder">Seite ${n}</div>`;
    pagesEl.append(el);
    state.pages.push({ el, rendered: null, task: null });
  }

  const scaleFor = (viewport1) => {
    const avail = scroller.clientWidth - 48;
    const availH = scroller.clientHeight - 36;
    const rotated = state.rotation % 180 !== 0;
    const w = rotated ? viewport1.height : viewport1.width;
    const h = rotated ? viewport1.width : viewport1.height;
    if (state.zoom === "width") return Math.max(0.2, avail / w);
    if (state.zoom === "page") return Math.max(0.2, Math.min(avail / w, availH / h));
    return Number(state.zoom);
  };

  const layout = () => {
    const scale = scaleFor(base);
    const vp = base.clone({ scale, rotation: state.rotation });
    for (const p of state.pages) {
      p.el.style.width = vp.width + "px";
      p.el.style.height = vp.height + "px";
      if (p.rendered !== null) {
        p.task?.cancel();
        p.rendered = null;
        p.el.innerHTML = `<div class="placeholder">Seite ${p.el.dataset.page}</div>`;
      }
    }
    zoomSelect.value = String(state.zoom);
    if (!zoomSelect.value) zoomSelect.value = "custom";
    $("[data-zoom-custom]", root).textContent = Math.round(scale * 100) + " %";
    observeAll();
  };

  async function renderPage(n) {
    const p = state.pages[n - 1];
    if (p.rendered !== null) return;
    p.rendered = "busy";
    const page = await pdf.getPage(n);
    const scale = scaleFor(page.getViewport({ scale: 1 }));
    const viewport = page.getViewport({ scale, rotation: state.rotation });
    const ratio = window.devicePixelRatio || 1;
    const canvas = document.createElement("canvas");
    canvas.width = Math.floor(viewport.width * ratio);
    canvas.height = Math.floor(viewport.height * ratio);
    p.el.style.width = viewport.width + "px";
    p.el.style.height = viewport.height + "px";
    const text = document.createElement("div");
    text.className = "textLayer";
    p.el.replaceChildren(canvas, text);
    p.task = page.render({
      canvasContext: canvas.getContext("2d"),
      viewport,
      transform: ratio !== 1 ? [ratio, 0, 0, ratio, 0, 0] : null,
    });
    try {
      await p.task.promise;
      text.style.setProperty("--scale-factor", viewport.scale);
      await new pdfjsLib.TextLayer({ textContentSource: page.streamTextContent(), container: text, viewport }).render();
      p.rendered = "done";
      markHits(n);
    } catch (e) {
      if (e?.name !== "RenderingCancelledException") console.warn(e);
      p.rendered = null;
    }
  }

  // Sichtbare Seiten rendern, aktuelle Seite verfolgen.
  const renderObserver = new IntersectionObserver(
    (entries) => entries.forEach((e) => e.isIntersecting && renderPage(Number(e.target.dataset.page))),
    { root: scroller, rootMargin: "600px 0px" },
  );
  const currentObserver = new IntersectionObserver(
    (entries) => {
      const visible = entries.filter((e) => e.isIntersecting).sort((a, b) => b.intersectionRatio - a.intersectionRatio)[0];
      if (visible) setCurrent(Number(visible.target.dataset.page), false);
    },
    { root: scroller, threshold: [0.25, 0.5, 0.75] },
  );
  function observeAll() {
    for (const p of state.pages) {
      renderObserver.unobserve(p.el);
      renderObserver.observe(p.el);
      currentObserver.observe(p.el);
    }
  }

  function setCurrent(n, scroll) {
    n = Math.min(Math.max(1, n), pdf.numPages);
    state.current = n;
    pageInput.value = n;
    thumbsEl?.querySelectorAll(".thumb").forEach((t) => t.classList.toggle("active", Number(t.dataset.page) === n));
    const thumb = thumbsEl?.querySelector(`.thumb[data-page="${n}"]`);
    if (thumb && !thumbsEl.hidden) scrollWithin(thumbsEl, thumb, "nearest");
    if (scroll) scrollWithin(scroller, state.pages[n - 1].el, "start");
  }

  // Miniaturen (klein, nacheinander gerendert)
  async function renderThumbs() {
    if (!thumbsEl || thumbsEl.dataset.done) return;
    thumbsEl.dataset.done = "1";
    for (let n = 1; n <= pdf.numPages; n++) {
      const item = document.createElement("div");
      item.className = "thumb" + (n === state.current ? " active" : "");
      item.dataset.page = n;
      const canvas = document.createElement("canvas");
      item.append(canvas, Object.assign(document.createElement("span"), { textContent: n }));
      item.onclick = () => setCurrent(n, true);
      thumbsEl.append(item);
    }
    const io = new IntersectionObserver(async (entries) => {
      for (const e of entries) {
        if (!e.isIntersecting) continue;
        io.unobserve(e.target);
        const page = await pdf.getPage(Number(e.target.dataset.page));
        const vp = page.getViewport({ scale: 1, rotation: state.rotation });
        const viewport = page.getViewport({ scale: 220 / vp.width, rotation: state.rotation });
        const canvas = e.target.querySelector("canvas");
        canvas.width = viewport.width;
        canvas.height = viewport.height;
        await page.render({ canvasContext: canvas.getContext("2d"), viewport }).promise;
      }
    }, { root: thumbsEl, rootMargin: "300px 0px" });
    thumbsEl.querySelectorAll(".thumb").forEach((t) => io.observe(t));
  }

  // ---- Suche im PDF (Umlaute und Gross/Klein egal)
  async function allTexts() {
    if (state.texts) return state.texts;
    status.textContent = "Text wird gelesen …";
    status.hidden = false;
    state.texts = [];
    for (let n = 1; n <= pdf.numPages; n++) {
      const tc = await (await pdf.getPage(n)).getTextContent();
      state.texts.push(tc.items.map((i) => fold(i.str || "")));
    }
    status.hidden = true;
    return state.texts;
  }

  async function search(term) {
    state.hits = [];
    state.hitIndex = -1;
    const needle = fold(term.trim());
    if (needle.length >= 2) {
      const texts = await allTexts();
      texts.forEach((items, p) => items.forEach((s, i) => s.includes(needle) && state.hits.push({ page: p + 1, item: i })));
    }
    for (let n = 1; n <= pdf.numPages; n++) markHits(n);
    if (state.hits.length) gotoHit(0);
    else searchInfo.textContent = needle.length >= 2 ? "Keine Treffer" : "";
  }

  function markHits(n) {
    const layer = state.pages[n - 1].el.querySelector(".textLayer");
    if (!layer) return;
    const spans = layer.querySelectorAll(":scope > span:not(.markedContent), .markedContent > span");
    spans.forEach((s) => s.classList.remove("hit", "current"));
    state.hits.forEach((h, idx) => {
      if (h.page !== n) return;
      const span = spans[h.item];
      if (!span) return;
      span.classList.add("hit");
      if (idx === state.hitIndex) span.classList.add("current");
    });
  }

  async function gotoHit(idx) {
    if (!state.hits.length) return;
    state.hitIndex = (idx + state.hits.length) % state.hits.length;
    const hit = state.hits[state.hitIndex];
    searchInfo.textContent = `${state.hitIndex + 1} / ${state.hits.length}`;
    setCurrent(hit.page, true);
    await renderPage(hit.page);
    for (let n = 1; n <= pdf.numPages; n++) markHits(n);
    const current = state.pages[hit.page - 1].el.querySelector(".textLayer .current");
    if (current) scrollWithin(scroller, current, "center");
  }

  // ---- Bedienung
  const step = (dir) => {
    const current = typeof state.zoom === "number" ? state.zoom : scaleFor(base);
    const next = dir > 0 ? ZOOMS.find((z) => z > current + 0.01) : [...ZOOMS].reverse().find((z) => z < current - 0.01);
    if (next) setZoom(next);
  };
  function setZoom(z) {
    const keep = state.current;
    state.zoom = z;
    layout();
    setCurrent(keep, true);
  }
  $("[data-prev]", root).onclick = () => setCurrent(state.current - 1, true);
  $("[data-next]", root).onclick = () => setCurrent(state.current + 1, true);
  pageInput.onchange = () => setCurrent(Number(pageInput.value), true);
  $("[data-zoom-in]", root).onclick = () => step(1);
  $("[data-zoom-out]", root).onclick = () => step(-1);
  zoomSelect.onchange = () => zoomSelect.value !== "custom" && setZoom(isNaN(zoomSelect.value) ? zoomSelect.value : Number(zoomSelect.value));
  $("[data-rotate]", root).onclick = () => {
    state.rotation = (state.rotation + 90) % 360;
    if (thumbsEl) { thumbsEl.replaceChildren(); delete thumbsEl.dataset.done; if (!thumbsEl.hidden) renderThumbs(); }
    setZoom(state.zoom);
  };
  $("[data-toggle-thumbs]", root).onclick = () => {
    thumbsEl.hidden = !thumbsEl.hidden;
    if (!thumbsEl.hidden) renderThumbs();
    if (typeof state.zoom === "string") setZoom(state.zoom);
  };
  $("[data-fullscreen]", root).onclick = () => {
    root.classList.toggle("viewer-full");
    setZoom(state.zoom);
  };
  let searchTimer;
  searchInput.oninput = () => { clearTimeout(searchTimer); searchTimer = setTimeout(() => search(searchInput.value), 300); };
  searchInput.onkeydown = (e) => {
    if (e.key === "Enter") { e.preventDefault(); gotoHit(state.hitIndex + (e.shiftKey ? -1 : 1)); }
    if (e.key === "Escape") { searchInput.value = ""; search(""); scroller.focus(); }
  };
  $("[data-search-prev]", root).onclick = () => gotoHit(state.hitIndex - 1);
  $("[data-search-next]", root).onclick = () => gotoHit(state.hitIndex + 1);

  root.addEventListener("keydown", (e) => {
    if (e.target.matches("input, select, textarea")) return;
    if ((e.metaKey || e.ctrlKey) && e.key === "f") { e.preventDefault(); searchInput.focus(); searchInput.select(); }
    else if (e.key === "ArrowRight" || e.key === "PageDown") setCurrent(state.current + 1, true);
    else if (e.key === "ArrowLeft" || e.key === "PageUp") setCurrent(state.current - 1, true);
    else if (e.key === "+" || e.key === "=") step(1);
    else if (e.key === "-") step(-1);
    else if (e.key === "r") $("[data-rotate]", root).click();
    else if (e.key === "Escape" && root.classList.contains("viewer-full")) $("[data-fullscreen]", root).click();
  });

  let resizeTimer;
  new ResizeObserver(() => {
    clearTimeout(resizeTimer);
    resizeTimer = setTimeout(() => typeof state.zoom === "string" && setZoom(state.zoom), 150);
  }).observe(scroller);

  layout();
  setCurrent(1, false);
  if (thumbsEl && !thumbsEl.hidden) renderThumbs();
  // Suche aus der Dokumentliste übernehmen (?q=…)
  const q = new URLSearchParams(location.search).get("find");
  if (q) { searchInput.value = q; search(q); }
}

// ------------------------------------------------------------------ Bilder

function imageViewer(root) {
  const img = $("img", root);
  const state = { zoom: 1, rotation: 0 };
  const apply = () => {
    img.style.transform = `rotate(${state.rotation}deg) scale(${state.zoom})`;
    $("[data-zoom-custom]", root).textContent = Math.round(state.zoom * 100) + " %";
  };
  $("[data-zoom-in]", root).onclick = () => { state.zoom = Math.min(8, state.zoom * 1.25); apply(); };
  $("[data-zoom-out]", root).onclick = () => { state.zoom = Math.max(0.1, state.zoom / 1.25); apply(); };
  $("[data-zoom-reset]", root).onclick = () => { state.zoom = 1; apply(); };
  $("[data-rotate]", root).onclick = () => { state.rotation = (state.rotation + 90) % 360; apply(); };
  $("[data-fullscreen]", root).onclick = () => root.classList.toggle("viewer-full");
  apply();
}
