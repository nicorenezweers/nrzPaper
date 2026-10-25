<div align="center">

<img src="../static/icon.png" width="96" alt="nrzPaper">

# nrzPaper

**A Rust-powered document management tool.**

[English](../README.md) · [Deutsch](README.de.md) · [Français](README.fr.md) · [Italiano](README.it.md) · [Español](README.es.md) · [Português](README.pt.md) · [Nederlands](README.nl.md) · [中文](README.zh.md) · [日本語](README.ja.md) · [Polski](README.pl.md) · [Русский](README.ru.md) · [العربية](README.ar.md) · [한국어](README.ko.md) · [Türkçe](README.tr.md) · [Bahasa Indonesia](README.id.md) · [हिन्दी](README.hi.md) · [Tiếng Việt](README.vi.md) · [ไทย](README.th.md) · [Українська](README.uk.md) · [Čeština](README.cs.md) · [Română](README.ro.md) · [Magyar](README.hu.md) · [Ελληνικά](README.el.md) · [Svenska](README.sv.md) · [Norsk](README.nb.md) · [Dansk](README.da.md) · [Suomi](README.fi.md) · [עברית](README.he.md) · [বাংলা](README.bn.md) · [فارسی](README.fa.md)

![Dokumenter med tidslinje](screenshots/documents.png)

</div>

nrzPaper er ét program uden ekstern database. Alt ligger i én SQLCipher-krypteret SQLite-fil, også selve filerne. Det laver OCR med Tesseract, scanner direkte fra netværksscannere, henter e-mailvedhæftninger og lader en AI efter eget valg foreslå titler, tags og metadata. Den indbyggede assistent **Dieter** svarer på spørgsmål om dine dokumenter og linker kilderne. Webgrænsefladen er en Rust/WebAssembly-app (Dioxus) i et roligt mørkt tema og virker også på telefoner.

## Funktioner

| | |
|---|---|
| **Indsamling** | Træk og slip, upload, **netværksscannere** (eSCL/AirScan: Canon, HP, Epson, Brother …), **e-mailindbakker** (IMAP, flere konti), **overvågede mapper** (f.eks. scannermappen på NAS'et), REST API |
| **Indbakke** | Scanninger, mails, mappeimporter og uploads venter på gennemgang: AI'en foreslår navn, tags og felter, og du godkender én ad gangen eller **alle på én gang**. Filtrer efter kilde (scanner, hver postkasse, hver mappe, upload). |
| **Genkendelse** | Tesseract og Poppler i baggrunden, også til scannede PDF'er og billeder; fuldtekstsøgning med præfikser, fraser og filtre (`tag:`, `-tag:`, `type:`, `created:>2026-06-01`, …) |
| **Organisering** | Farvede tags, **mapper bygget af tags** (indlejrede), **taggeregler** ("indeholder Northwind → Faktura"), brugerdefinerede **egenskaber** (tekst, tal, dato, ja/nej, valg, flervalg, bruger, dokumentlink) |
| **Tidslinje** | Lodret tidslinje ved siden af dokumenterne: i dag øverst, nedenunder hver dag med mail og de stille huller imellem ("12 dage uden dokumenter"). Filtrer efter tag, type og tekst. |
| **Påmindelser** | "Påmind mig om 10 dage" på et vilkårligt dokument, med note; forfaldne påmindelser vises i sidebjælken og øverst på tidslinjen. |
| **Udskrivning** | Udskriv dokumenter på enhver **IPP-netværksprinter** (AirPrint, IPP Everywhere, Mopria, CUPS), som serveren finder. PDF'er går direkte, eller som sidebilleder til printere, der kun tager JPEG. |
| **AI, enhver udbyder** | OpenRouter (gratis modeller indbygget), OpenAI, Anthropic, Google Gemini, Mistral, Groq, xAI, DeepSeek, Together, Fireworks, Perplexity, Cohere, NVIDIA, Hugging Face, Qwen, Azure OpenAI, **Ollama, LM Studio, vLLM/LocalAI** eller en hvilken som helst OpenAI-kompatibel URL. Tekstbaserede modeller får OCR-teksten i stedet for billeder. |
| **Dieter** | Chatassistent (nederst til højre): *"Hvad er mit forsikringsnummer?"*, *"Alle lønsedler fra Fabrikam i 2026"*. Han søger, læser de matchende afsnit og svarer med links til dokumenterne. |
| **Teams** | Flere organisationer, **flere konti pr. organisation** (invitation, roller: ejer, administrator, medlem; overfør ejerskab), API-nøgler med tilladelser, webhooks (Standard Webhooks-signaturer), offentlige delelinks |
| **Sprog** | English, Deutsch, Français, Italiano, Español, Português, Nederlands (flere sprog kommer snart) |
| **Sikkert som standard** | Én krypteret fil, Argon2-adgangskoder, CSP uden inline-scripts, CSRF-beskyttelse, filer kun via API |

## Skærmbilleder

| | |
|---|---|
| ![Dieter, assistenten](screenshots/dieter.png) | ![Indbakke med AI-forslag](screenshots/inbox.png) |
| **Dieter** svarer med kilder | **Indbakke**: gennemgå og godkend, filtrer efter kilde |
| ![Dokumentvisning](screenshots/document.png) | ![AI-udbydere](screenshots/settings-ai.png) |
| **Dokument**: PDF-fremviser, tags, påmindelser, udskrivning | **AI og modeller**: enhver udbyder, lokal eller cloud |
| ![Påmindelser](screenshots/reminders.png) | ![Medlemmer](screenshots/members.png) |
| **Påmindelser** | **Medlemmer** og invitationer |

## Hurtig start

Du skal bruge Docker. Billedet indeholder programmet, webgrænsefladen, Tesseract (tysk, engelsk) og Poppler.

```bash
git clone https://github.com/nicorenezweers/nrzPaper.git
cd nrzPaper
docker build -t nrzpaper .
docker run -d --name nrzpaper -p 8320:8320 -v nrzpaper-data:/data nrzpaper
```

Åbn <http://localhost:8320>. Den første konto, du opretter, bliver administrator. Derefter er registreringen lukket, indtil du åbner den under *Indstillinger → Registrering og brugere*, hvor du også kan oprette konti til andre.

### Docker Compose

```yaml
services:
  nrzpaper:
    build: .                      # eller: image: ghcr.io/nicorenezweers/nrz-paper:1.0.0
    container_name: nrzpaper
    restart: unless-stopped
    ports:
      - "8320:8320"
    volumes:
      - nrzpaper-data:/data       # krypteret database + nøglefil
      - /srv/scans:/scans         # valgfrit: en mappe til mappeindbakken
    environment:
      TZ: Europe/Zurich
      # NRZPAPER_DB_KEY: "a-long-secret"     # ellers oprettes /data/nrzpaper.key
      # NRZPAPER_COOKIE_SECURE: "true"       # når det serveres via HTTPS
      # NRZPAPER_AI_PROVIDER: "openai"       # eller angiv i webgrænsefladen
      # NRZPAPER_AI_KEY: "sk-…"

volumes:
  nrzpaper-data:
```

```bash
docker compose up -d
```

> **Tag backup af `/data`.** Der ligger `nrzpaper.db` (alt, krypteret) og `nrzpaper.key`. Uden nøglen kan databasen ikke åbnes. Hvis du angiver `NRZPAPER_DB_KEY`, skal du opbevare den værdi sikkert i stedet.

### Begge arkitekturer

```bash
docker buildx build --platform linux/amd64,linux/arm64 -t <registry>/nrzpaper:1.0.0 --push .
```

Rust krydscompileres, ikke emuleres, så arm64-buildet er hurtigt på en x86-maskine og omvendt.

## Første skridt

1. **Upload** nogle PDF'er eller billeder: teksten genkendes i baggrunden.
2. **Tags**: opret tags som *Faktura* eller *Forsikring*, og giv dem en beskrivelse. AI'en bruger den til at tildele tags.
3. **AI** (*Indstillinger → AI og modeller*): vælg en udbyder, indsæt en nøgle (eller brug de indbyggede gratis OpenRouter-modeller eller en lokal Ollama), derefter *Indlæs modeller* og *Test forbindelse*.
4. **Scanner** (*Profil → Scanner*): *Søg i netværket* finder eSCL/AirScan-scannere; derefter *Scan* i sidebjælken.
5. **E-mail** (*Profil → E-mailindbakke*): tilføj en eller flere IMAP-postkasser; PDF'er og billeder fra vedhæftninger havner i indbakken.
6. **Mappeindbakke** (*Indstillinger → Mappeindbakke*, administratorer): overvåg en servermappe som `/scans` (montér den i containeren først).
7. **Printere** (*Indstillinger → Printere*, administratorer): *Søg i netværket* eller indtast `ipp://printer/ipp/print`; derefter kan hvert medlem udskrive fra en dokumentside.

## Konfiguration

Hver indstilling kan angives som en miljøvariabel med præfikset `NRZPAPER_` (eller i `Rocket.toml`).

| Variabel | Standard | Betydning |
|---|---|---|
| `NRZPAPER_DATA_DIR` | `/data` (billede) | Database, nøglefil og indtagsmapper |
| `NRZPAPER_DB_KEY` | – | Databasenøgle; hvis den ikke er angivet, oprettes `nrzpaper.key` i datamappen |
| `NRZPAPER_PORT` / `NRZPAPER_ADDRESS` | `8320` / `0.0.0.0` | Hvor serveren lytter |
| `NRZPAPER_COOKIE_SECURE` | `false` | Sæt til `true` bag HTTPS |
| `NRZPAPER_REGISTRATION_OPEN` | `false` | Tillad selvregistrering (kan også slås til i UI'et) |
| `NRZPAPER_OCR_LANGUAGES` | `deu+eng` | Tesseract-sprog (billedet indeholder `deu` og `eng`) |
| `NRZPAPER_OCR_WORKERS` | `2` | Parallelle OCR-job |
| `NRZPAPER_TRASH_RETENTION_DAYS` | `30` | Dage i papirkurven før endelig sletning |
| `NRZPAPER_INBOX_KEEP_DAYS` | `7` | Dage indbakke-elementer, der ikke er godkendt, opbevares |
| `NRZPAPER_SCANNER_URL` | – | Forudindstillet scanner til nye brugere, f.eks. `http://192.168.1.50` |
| `NRZPAPER_SCAN_SUBNETS` | `192.168.0.0/24,192.168.1.0/24` | Netværk, der søges igennem for scannere og printere |
| `NRZPAPER_AI_PROVIDER` | `openrouter` | `openai`, `anthropic`, `gemini`, `mistral`, `groq`, `ollama`, `custom`, … |
| `NRZPAPER_AI_KEY` / `NRZPAPER_AI_URL` / `NRZPAPER_AI_MODEL` | – | Nøgle, basis-URL og model til den udbyder |
| `NRZPAPER_OPENROUTER_KEY` | – | OpenRouter-nøgle (alternativ til ovenstående) |
| `NRZPAPER_LOG` | `info` | Logfilter, f.eks. `debug` |

Indstillinger i webgrænsefladen (AI-udbyder, nøgler, modeller, registrering) gemmes i den krypterede database og har forrang over miljøet.

### AI-udbydere

| Udbyder | Adresse | Bemærkninger |
|---|---|---|
| OpenRouter | indbygget | Gratis synsmodeller; egen nøgle valgfri |
| OpenAI, Anthropic, Google Gemini, Mistral, Groq, xAI, DeepSeek, Together, Fireworks, Perplexity, Cohere, NVIDIA, Hugging Face, Qwen | indbygget | API-nøgle fra udbyderen |
| Azure OpenAI | `https://NAME.openai.azure.com/openai/v1` | Model = udrulningsnavn |
| Ollama / LM Studio / vLLM / LocalAI | `http://host:11434/v1` osv. | Ingen nøgle nødvendig |
| Alt OpenAI-kompatibelt | din URL | Nøgle valgfri |

Modeller, der ikke kan læse billeder: markér *"Modellen kan ikke læse billeder – send genkendt tekst"*. Udbydere, der ikke accepterer PDF'er, får automatisk de første sider som billeder. Sproget for foreslåede titler og resuméer angives under *Sprog for forslagene*.

## REST API

Opret en API-nøgle under *Profil → API-nøgler* (vælg tilladelser). Alle endpoints ligger under `/api/v1` og accepterer `Authorization: Bearer np_…`.

```bash
TOKEN=np_your_key
BASE=http://localhost:8320/api/v1

# organisationer
curl -H "Authorization: Bearer $TOKEN" $BASE/organizations

# upload (flere filer på én gang med flere -F)
curl -H "Authorization: Bearer $TOKEN" -F file=@invoice.pdf \
     $BASE/organizations/$ORG/documents

# søgning: samme syntaks som i UI'et
curl -H "Authorization: Bearer $TOKEN" -G $BASE/organizations/$ORG/documents \
     --data-urlencode 'searchQuery=northwind tag:invoice created:2026'

# download filen
curl -H "Authorization: Bearer $TOKEN" -o out.pdf \
     "$BASE/organizations/$ORG/documents/$DOC/file?download=true"

# tilføj et tag
curl -H "Authorization: Bearer $TOKEN" -H "Content-Type: application/json" \
     -d '{"tagId":"'$TAG'"}' $BASE/organizations/$ORG/documents/$DOC/tags

# offentligt delelink gyldigt i 7 dage
curl -H "Authorization: Bearer $TOKEN" -H "Content-Type: application/json" \
     -d '{"expiresInDays":7}' $BASE/organizations/$ORG/documents/$DOC/share-links

# sundhedstjek (ingen nøgle nødvendig)
curl $BASE/health
```

Webhooks (*Indstillinger → Webhooks*) sender signeret JSON (`webhook-id`, `webhook-timestamp`, `webhook-signature`) for `document:created`, `document:updated`, `document:deleted`, `document:restored`, `document:tag:added` og `document:tag:removed`.

Indtagsmappen `data/ingestion/<organization-id>/` importerer filer direkte som dokumenter. Overvågede mapper (*Mappeindbakke*) sender dem først til indbakken til gennemgang.

## Udvikling

```bash
# backend (Rust 1.85+, Tesseract og Poppler til OCR)
cargo run                         # http://localhost:8320
cargo test

# webgrænseflade med hot reload (dx = dioxus-cli 0.7)
rustup target add wasm32-unknown-unknown
cargo install dioxus-cli --version 0.7.10 --locked
cd ui && dx serve                 # http://localhost:8080, proxyer /api til :8320

# indbyg grænsefladen i programmet
scripts/ui.sh && cargo build --release
```

Oversættelser ligger i `locales/<language>.json` (tysk kildetekst → oversættelse). `scripts/i18n_extract.py` opdaterer `locales/strings.json` og rapporterer manglende poster pr. sprog.

Arkitekturdetaljer: [ARCHITECTURE.md](../ARCHITECTURE.md).

## Licens

AGPL-3.0-or-later. UI-komponenterne bygger på [rust-ui](https://github.com/rust-ui/ui) (MIT), PDF-fremviseren på [PDF.js](https://mozilla.github.io/pdf.js/) (Apache-2.0), skrifttypen er [Inter](https://rsms.me/inter/) (OFL).
