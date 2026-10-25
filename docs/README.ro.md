<div align="center">

<img src="../static/icon.png" width="96" alt="nrzPaper">

# nrzPaper

**Gestionare de documente auto-găzduită în Rust: scanează, recunoaște, etichetează și găsește-ți actele — criptate într-un singur fișier.**

[English](../README.md) · [Deutsch](README.de.md) · [Français](README.fr.md) · [Italiano](README.it.md) · [Español](README.es.md) · [Português](README.pt.md) · [Nederlands](README.nl.md) · [中文](README.zh.md) · [日本語](README.ja.md) · [Polski](README.pl.md) · [Русский](README.ru.md) · [العربية](README.ar.md) · [한국어](README.ko.md) · [Türkçe](README.tr.md) · [Bahasa Indonesia](README.id.md) · [हिन्दी](README.hi.md) · [Tiếng Việt](README.vi.md) · [ไทย](README.th.md) · [Українська](README.uk.md) · [Čeština](README.cs.md) · [Română](README.ro.md) · [Magyar](README.hu.md) · [Ελληνικά](README.el.md) · [Svenska](README.sv.md) · [Norsk](README.nb.md) · [Dansk](README.da.md) · [Suomi](README.fi.md) · [עברית](README.he.md) · [বাংলা](README.bn.md) · [فارسی](README.fa.md)

![Documente cu cronologie](screenshots/documents.png)

</div>

nrzPaper este un singur program fără bază de date externă. Totul se află într-un singur fișier SQLite criptat cu SQLCipher, inclusiv fișierele în sine. Face OCR cu Tesseract, scanează direct de la scanere de rețea, preia atașamentele e-mail și permite unui AI ales de tine să sugereze titluri, etichete și metadate. Asistentul integrat **Dieter** răspunde la întrebări despre documentele tale și leagă sursele. Interfața web este o aplicație Rust/WebAssembly (Dioxus) cu o temă întunecată calmă și funcționează și pe telefoane.

## Funcționalități

| | |
|---|---|
| **Captare** | Tragere și plasare, încărcare, **scanere de rețea** (eSCL/AirScan: Canon, HP, Epson, Brother …), **căsuțe de e-mail** (IMAP, mai multe conturi), **foldere monitorizate** (de ex. folderul de scanare de pe NAS), REST API |
| **Căsuța de intrare** | Scanările, mailurile, importurile din foldere și încărcările așteaptă revizuirea: AI propune nume, etichete și câmpuri, iar tu aprobi câte un element sau **pe toate deodată**. Filtrare după sursă (scanner, fiecare căsuță poștală, fiecare folder, încărcare). |
| **Recunoaștere** | Tesseract și Poppler în fundal, inclusiv pentru PDF-uri scanate și fotografii; căutare full-text cu prefixe, fraze și filtre (`tag:`, `-tag:`, `type:`, `created:>2026-06-01`, …) |
| **Organizare** | Etichete colorate, **foldere create din etichete** (imbricate), **reguli de etichetare** („conține Northwind → Factură”), **proprietăți** personalizate (text, număr, dată, da/nu, selecție, selecție multiplă, utilizator, link document) |
| **Cronologie** | Cronologie verticală lângă documente: azi sus, dedesubt fiecare zi cu mail și golurile liniștite dintre ele („12 zile fără documente”). Filtrare după etichetă, tip și text. |
| **Memento-uri** | „Amintește-mi peste 10 zile” pe orice document, cu notă; memento-urile scadente apar în bara laterală și în partea de sus a cronologiei. |
| **Imprimare** | Imprimă documente pe orice **imprimantă de rețea IPP** (AirPrint, IPP Everywhere, Mopria, CUPS) găsită de server. PDF-urile merg direct, sau ca imagini de pagină la imprimantele care acceptă doar JPEG. |
| **AI, orice furnizor** | OpenRouter (modele gratuite integrate), OpenAI, Anthropic, Google Gemini, Mistral, Groq, xAI, DeepSeek, Together, Fireworks, Perplexity, Cohere, NVIDIA, Hugging Face, Qwen, Azure OpenAI, **Ollama, LM Studio, vLLM/LocalAI** sau orice URL compatibil OpenAI. Modelele doar-text primesc textul OCR în loc de imagini. |
| **Dieter** | Asistent de chat (jos dreapta): *„Care este numărul meu de asigurare?”*, *„Toate fluturașii de salariu de la Fabrikam în 2026”*. Caută, citește pasajele potrivite și răspunde cu linkuri către documente. |
| **Echipe** | Mai multe organizații, **mai multe conturi per organizație** (invitație, roluri: proprietar, admin, membru; transfer de proprietate), chei API cu permisiuni, webhook-uri (semnături Standard Webhooks), linkuri publice de partajare |
| **Sigur implicit** | Un fișier criptat, parole Argon2, CSP fără scripturi inline, protecție CSRF, fișiere doar prin API |

## Capturi de ecran

| | |
|---|---|
| ![Dieter, asistentul](screenshots/dieter.png) | ![Căsuța de intrare cu sugestii AI](screenshots/inbox.png) |
| **Dieter** răspunde cu surse | **Căsuța de intrare**: revizuire și aprobare, filtrare după sursă |
| ![Vizualizare document](screenshots/document.png) | ![Furnizori AI](screenshots/settings-ai.png) |
| **Document**: vizualizator PDF, etichete, memento-uri, imprimare | **AI și modele**: fiecare furnizor, local sau cloud |
| ![Memento-uri](screenshots/reminders.png) | ![Membri](screenshots/members.png) |
| **Memento-uri** | **Membri** și invitații |

## Pornire rapidă

Ai nevoie de Docker. Imaginea conține programul, interfața web, Tesseract (germană, engleză) și Poppler.

```bash
git clone https://github.com/nicorenezweers/nrzPaper.git
cd nrzPaper
docker build -t nrzpaper .
docker run -d --name nrzpaper -p 8320:8320 -v nrzpaper-data:/data nrzpaper
```

Deschide <http://localhost:8320>. Primul cont pe care îl creezi devine administrator. După aceea, înregistrarea este închisă până o deschizi în *Setări → Înregistrare și utilizatori*, unde poți crea conturi și pentru alții.

### Docker Compose

```yaml
services:
  nrzpaper:
    build: .                      # sau: image: ghcr.io/nicorenezweers/nrzpaper:1.0.0
    container_name: nrzpaper
    restart: unless-stopped
    ports:
      - "8320:8320"
    volumes:
      - nrzpaper-data:/data       # baza de date criptată + fișierul cheie
      - /srv/scans:/scans         # opțional: un folder pentru căsuța de intrare din foldere
    environment:
      TZ: Europe/Zurich
      # NRZPAPER_DB_KEY: "a-long-secret"     # altfel se creează /data/nrzpaper.key
      # NRZPAPER_COOKIE_SECURE: "true"       # când este servit prin HTTPS
      # NRZPAPER_AI_PROVIDER: "openai"       # sau setează în interfața web
      # NRZPAPER_AI_KEY: "sk-…"

volumes:
  nrzpaper-data:
```

```bash
docker compose up -d
```

> **Fă backup la `/data`.** Conține `nrzpaper.db` (totul, criptat) și `nrzpaper.key`. Fără cheie, baza de date nu poate fi deschisă. Dacă setezi `NRZPAPER_DB_KEY`, păstrează acea valoare în siguranță.

### Ambele arhitecturi

```bash
docker buildx build --platform linux/amd64,linux/arm64 -t <registry>/nrzpaper:1.0.0 --push .
```

Rust este compilat încrucișat, nu emulat, așa că build-ul arm64 este rapid pe o mașină x86 și invers.

## Primii pași

1. **Încarcă** câteva PDF-uri sau fotografii: textul este recunoscut în fundal.
2. **Etichete**: creează etichete precum *Factură* sau *Asigurare* și adaugă o descriere. AI-ul o folosește pentru a atribui etichete.
3. **AI** (*Setări → AI și modele*): alege un furnizor, lipește o cheie (sau folosește modelele gratuite OpenRouter integrate sau un Ollama local), apoi *Încarcă modelele* și *Testează conexiunea*.
4. **Scanner** (*Profil → Scanner*): *Caută în rețea* găsește scanere eSCL/AirScan; apoi *Scanează* în bara laterală.
5. **E-mail** (*Profil → Căsuța de e-mail*): adaugă una sau mai multe căsuțe IMAP; PDF-urile și imaginile din atașamente ajung în căsuța de intrare.
6. **Căsuța de intrare din foldere** (*Setări → Căsuța de intrare din foldere*, administratori): monitorizează un folder de server precum `/scans` (montează-l mai întâi în container).
7. **Imprimante** (*Setări → Imprimante*, administratori): *Caută în rețea* sau introdu `ipp://printer/ipp/print`; apoi fiecare membru poate imprima din pagina documentului.

## Configurare

Fiecare setare poate fi dată ca variabilă de mediu cu prefixul `NRZPAPER_` (sau în `Rocket.toml`).

| Variabilă | Implicit | Semnificație |
|---|---|---|
| `NRZPAPER_DATA_DIR` | `/data` (imagine) | Baza de date, fișierul cheie și folderele de ingestie |
| `NRZPAPER_DB_KEY` | – | Cheia bazei de date; dacă nu este setată, se generează `nrzpaper.key` în folderul de date |
| `NRZPAPER_PORT` / `NRZPAPER_ADDRESS` | `8320` / `0.0.0.0` | Unde ascultă serverul |
| `NRZPAPER_COOKIE_SECURE` | `false` | Setează `true` în spatele HTTPS |
| `NRZPAPER_REGISTRATION_OPEN` | `false` | Permite auto-înregistrarea (poate fi comutat și în UI) |
| `NRZPAPER_OCR_LANGUAGES` | `deu+eng` | Limbi Tesseract (imaginea include `deu` și `eng`) |
| `NRZPAPER_OCR_WORKERS` | `2` | Lucrări OCR paralele |
| `NRZPAPER_TRASH_RETENTION_DAYS` | `30` | Zile în coș înainte de ștergerea definitivă |
| `NRZPAPER_INBOX_KEEP_DAYS` | `7` | Zilele în care sunt păstrate elementele neaprobate din căsuța de intrare |
| `NRZPAPER_SCANNER_URL` | – | Scanner presetat pentru utilizatorii noi, de ex. `http://192.168.1.50` |
| `NRZPAPER_SCAN_SUBNETS` | `192.168.0.0/24,192.168.1.0/24` | Rețelele căutate pentru scanere și imprimante |
| `NRZPAPER_AI_PROVIDER` | `openrouter` | `openai`, `anthropic`, `gemini`, `mistral`, `groq`, `ollama`, `custom`, … |
| `NRZPAPER_AI_KEY` / `NRZPAPER_AI_URL` / `NRZPAPER_AI_MODEL` | – | Cheia, URL-ul de bază și modelul pentru acel furnizor |
| `NRZPAPER_OPENROUTER_KEY` | – | Cheie OpenRouter (alternativă la cele de mai sus) |
| `NRZPAPER_LOG` | `info` | Filtru de log, de ex. `debug` |

Setările făcute în interfața web (furnizor AI, chei, modele, înregistrare) sunt stocate în baza de date criptată și au prioritate față de mediu.

### Furnizori AI

| Furnizor | Adresă | Note |
|---|---|---|
| OpenRouter | integrat | Modele de viziune gratuite; cheie proprie opțională |
| OpenAI, Anthropic, Google Gemini, Mistral, Groq, xAI, DeepSeek, Together, Fireworks, Perplexity, Cohere, NVIDIA, Hugging Face, Qwen | integrat | Cheie API de la furnizor |
| Azure OpenAI | `https://NAME.openai.azure.com/openai/v1` | Model = numele implementării |
| Ollama / LM Studio / vLLM / LocalAI | `http://host:11434/v1` etc. | Nu e nevoie de cheie |
| Orice compatibil OpenAI | URL-ul tău | Cheie opțională |

Modelele care nu pot citi imagini: bifează *„Modelul nu poate citi imagini – trimite textul recunoscut”*. Furnizorii care nu acceptă PDF primesc automat primele pagini ca imagini. Limba titlurilor și rezumatelor sugerate se setează la *Limba sugestiilor*.

## REST API

Creează o cheie API în *Profil → Chei API* (alege permisiunile). Toate endpoint-urile sunt sub `/api/v1` și acceptă `Authorization: Bearer np_…`.

```bash
TOKEN=np_your_key
BASE=http://localhost:8320/api/v1

# organizații
curl -H "Authorization: Bearer $TOKEN" $BASE/organizations

# încărcare (mai multe fișiere deodată cu mai multe -F)
curl -H "Authorization: Bearer $TOKEN" -F file=@invoice.pdf \
     $BASE/organizations/$ORG/documents

# căutare: aceeași sintaxă ca în UI
curl -H "Authorization: Bearer $TOKEN" -G $BASE/organizations/$ORG/documents \
     --data-urlencode 'searchQuery=northwind tag:invoice created:2026'

# descarcă fișierul
curl -H "Authorization: Bearer $TOKEN" -o out.pdf \
     "$BASE/organizations/$ORG/documents/$DOC/file?download=true"

# adaugă o etichetă
curl -H "Authorization: Bearer $TOKEN" -H "Content-Type: application/json" \
     -d '{"tagId":"'$TAG'"}' $BASE/organizations/$ORG/documents/$DOC/tags

# link public de partajare valabil 7 zile
curl -H "Authorization: Bearer $TOKEN" -H "Content-Type: application/json" \
     -d '{"expiresInDays":7}' $BASE/organizations/$ORG/documents/$DOC/share-links

# verificare stare (fără cheie)
curl $BASE/health
```

Webhook-urile (*Setări → Webhook-uri*) trimit JSON semnat (`webhook-id`, `webhook-timestamp`, `webhook-signature`) pentru `document:created`, `document:updated`, `document:deleted`, `document:restored`, `document:tag:added` și `document:tag:removed`.

Folderul de ingestie `data/ingestion/<organization-id>/` importă fișierele direct ca documente. Folderele monitorizate (*Căsuța de intrare din foldere*) le trimit mai întâi în căsuța de intrare pentru revizuire.

## Dezvoltare

```bash
# backend (Rust 1.85+, Tesseract și Poppler pentru OCR)
cargo run                         # http://localhost:8320
cargo test

# interfață web cu reîncărcare rapidă (dx = dioxus-cli 0.7)
rustup target add wasm32-unknown-unknown
cargo install dioxus-cli --version 0.7.10 --locked
cd ui && dx serve                 # http://localhost:8080, proxy /api către :8320

# încorporează interfața în program
scripts/ui.sh && cargo build --release
```

Traducerile se află în `locales/<language>.json` (text sursă german → traducere). `scripts/i18n_extract.py` reîmprospătează `locales/strings.json` și raportează intrările lipsă pe limbă.

Detalii de arhitectură: [ARCHITECTURE.md](../ARCHITECTURE.md).

## Licență

AGPL-3.0-or-later. Componentele UI se bazează pe [rust-ui](https://github.com/rust-ui/ui) (MIT), vizualizatorul PDF pe [PDF.js](https://mozilla.github.io/pdf.js/) (Apache-2.0), fontul este [Inter](https://rsms.me/inter/) (OFL).
