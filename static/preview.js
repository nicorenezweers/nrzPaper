// nrzPaper – grosses Vorschau-Fenster mit Zoom (PDF.js), als ES-Modul für die Dioxus-Oberfläche.
const $ = (sel, root = document) => root.querySelector(sel);
const $$ = (sel, root = document) => [...root.querySelectorAll(sel)];

// Vorschau-Fenster (<button data-preview="…/file" data-preview-kind="pdf|image" data-preview-name="…">):
// grosse Ansicht mit Zoom, ohne die Liste zu verlassen. Esc schliesst, +/− und Strg/⌘+Mausrad zoomen.
let pdfjsLib = null;
export async function openPreview({ url, kind, name = "", docUrl = "" }) {
  const ui = document.createElement("div");
  ui.className = "fixed inset-0 z-50 flex flex-col bg-zinc-950/95 backdrop-blur-sm";
  ui.innerHTML = `
    <div class="flex items-center gap-2 border-b border-zinc-800 bg-zinc-900/90 px-4 py-2.5 text-sm text-zinc-300">
      <span class="min-w-0 flex-1 truncate font-medium text-zinc-100"></span>
      <button data-z="-1" title="Verkleinern (−)" class="rounded-md px-2.5 py-1 hover:bg-zinc-800">−</button>
      <button data-z="fit" title="Breite anpassen" class="min-w-16 rounded-md px-2 py-1 tabular-nums hover:bg-zinc-800" data-zl>100 %</button>
      <button data-z="1" title="Vergrössern (+)" class="rounded-md px-2.5 py-1 hover:bg-zinc-800">+</button>
      <span class="mx-1 h-5 w-px bg-zinc-800"></span>
      ${docUrl ? `<a href="${docUrl}" data-open class="rounded-md px-2.5 py-1 hover:bg-zinc-800">Öffnen</a>` : ""}
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
