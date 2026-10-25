<div align="center">

<img src="../static/icon.png" width="96" alt="nrzPaper">

# nrzPaper

**A Rust-powered document management tool.**

[English](../README.md) · [Deutsch](README.de.md) · [Français](README.fr.md) · [Italiano](README.it.md) · [Español](README.es.md) · [Português](README.pt.md) · [Nederlands](README.nl.md) · [中文](README.zh.md) · [日本語](README.ja.md) · [Polski](README.pl.md) · [Русский](README.ru.md) · [العربية](README.ar.md) · [한국어](README.ko.md) · [Türkçe](README.tr.md) · [Bahasa Indonesia](README.id.md) · [हिन्दी](README.hi.md) · [Tiếng Việt](README.vi.md) · [ไทย](README.th.md) · [Українська](README.uk.md) · [Čeština](README.cs.md) · [Română](README.ro.md) · [Magyar](README.hu.md) · [Ελληνικά](README.el.md) · [Svenska](README.sv.md) · [Norsk](README.nb.md) · [Dansk](README.da.md) · [Suomi](README.fi.md) · [עברית](README.he.md) · [বাংলা](README.bn.md) · [فارسی](README.fa.md)

![Dokument med tidslinje](screenshots/documents.png)

</div>

nrzPaper är ett enda program utan extern databas. Allt ligger i en enda SQLCipher-krypterad SQLite-fil, även filerna själva. Det gör OCR med Tesseract, skannar direkt från nätverksskannrar, hämtar e-postbilagor och låter en AI du väljer föreslå titlar, taggar och metadata. Den inbyggda assistenten **Dieter** svarar på frågor om dina dokument och länkar källorna. Webbgränssnittet är en Rust/WebAssembly-app (Dioxus) i ett lugnt mörkt tema och fungerar även på telefoner.

## Funktioner

| | |
|---|---|
| **Infångning** | Dra och släpp, uppladdning, **nätverksskannrar** (eSCL/AirScan: Canon, HP, Epson, Brother …), **e-postinkorgar** (IMAP, flera konton), **bevakade mappar** (t.ex. skannermappen på NAS:en), REST API |
| **Inkorg** | Skanningar, mejl, mappimporter och uppladdningar väntar på granskning: AI:n föreslår namn, taggar och fält, och du godkänner ett i taget eller **alla på en gång**. Filtrera efter källa (skanner, varje brevlåda, varje mapp, uppladdning). |
| **Igenkänning** | Tesseract och Poppler i bakgrunden, även för skannade PDF:er och foton; fulltextsökning med prefix, fraser och filter (`tag:`, `-tag:`, `type:`, `created:>2026-06-01`, …) |
| **Organisering** | Färgade taggar, **mappar byggda av taggar** (kapslade), **taggingsregler** (”innehåller Northwind → Faktura”), anpassade **egenskaper** (text, tal, datum, ja/nej, val, flerval, användare, dokumentlänk) |
| **Tidslinje** | Vertikal tidslinje bredvid dokumenten: idag överst, under varje dag med mejl och de tysta luckorna mellan dem (”12 dagar utan dokument”). Filtrera efter tagg, typ och text. |
| **Påminnelser** | ”Påminn mig om 10 dagar” på valfritt dokument, med anteckning; förfallna påminnelser visas i sidofältet och högst upp på tidslinjen. |
| **Utskrift** | Skriv ut dokument på vilken **IPP-nätverksskrivare** (AirPrint, IPP Everywhere, Mopria, CUPS) servern hittar. PDF:er går direkt, eller som sidbilder till skrivare som bara tar JPEG. |
| **AI, valfri leverantör** | OpenRouter (kostnadsfria modeller inbyggda), OpenAI, Anthropic, Google Gemini, Mistral, Groq, xAI, DeepSeek, Together, Fireworks, Perplexity, Cohere, NVIDIA, Hugging Face, Qwen, Azure OpenAI, **Ollama, LM Studio, vLLM/LocalAI** eller valfri OpenAI-kompatibel URL. Textbaserade modeller får OCR-texten i stället för bilder. |
| **Dieter** | Chattackassistent (längst ner till höger): *”Vilket är mitt försäkringsnummer?”*, *”Alla lönespecifikationer från Fabrikam 2026”*. Han söker, läser de matchande avsnitten och svarar med länkar till dokumenten. |
| **Team** | Flera organisationer, **flera konton per organisation** (inbjudan, roller: ägare, administratör, medlem; överför ägarskap), API-nycklar med behörigheter, webhooks (Standard Webhooks-signaturer), offentliga delningslänkar |
| **Språk** | English, Deutsch, Français, Italiano, Español, Português, Nederlands (fler språk kommer snart) |
| **Säkert som standard** | En krypterad fil, Argon2-lösenord, CSP utan inline-skript, CSRF-skydd, filer endast via API |

## Skärmbilder

| | |
|---|---|
| ![Dieter, assistenten](screenshots/dieter.png) | ![Inkorg med AI-förslag](screenshots/inbox.png) |
| **Dieter** svarar med källor | **Inkorg**: granska och godkänn, filtrera efter källa |
| ![Dokumentvy](screenshots/document.png) | ![AI-leverantörer](screenshots/settings-ai.png) |
| **Dokument**: PDF-visare, taggar, påminnelser, utskrift | **AI och modeller**: alla leverantörer, lokalt eller i molnet |
| ![Påminnelser](screenshots/reminders.png) | ![Medlemmar](screenshots/members.png) |
| **Påminnelser** | **Medlemmar** och inbjudningar |

## Snabbstart

Du behöver Docker. Avbilden innehåller programmet, webbgränssnittet, Tesseract (tyska, engelska) och Poppler.

```bash
git clone https://github.com/nicorenezweers/nrzPaper.git
cd nrzPaper
docker build -t nrzpaper .
docker run -d --name nrzpaper -p 8320:8320 -v nrzpaper-data:/data nrzpaper
```

Öppna <http://localhost:8320>. Det första kontot du skapar blir administratör. Därefter är registreringen stängd tills du öppnar den under *Inställningar → Registrering och användare*, där du också kan skapa konton för andra.

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
      - nrzpaper-data:/data       # krypterad databas + nyckelfil
      - /srv/scans:/scans         # valfritt: en mapp för mappinkorgen
    environment:
      TZ: Europe/Zurich
      # NRZPAPER_DB_KEY: "a-long-secret"     # annars skapas /data/nrzpaper.key
      # NRZPAPER_COOKIE_SECURE: "true"       # när det serveras via HTTPS
      # NRZPAPER_AI_PROVIDER: "openai"       # eller ange i webbgränssnittet
      # NRZPAPER_AI_KEY: "sk-…"

volumes:
  nrzpaper-data:
```

```bash
docker compose up -d
```

> **Säkerhetskopiera `/data`.** Där finns `nrzpaper.db` (allt, krypterat) och `nrzpaper.key`. Utan nyckeln kan databasen inte öppnas. Om du anger `NRZPAPER_DB_KEY`, förvara det värdet säkert i stället.

### Båda arkitekturerna

```bash
docker buildx build --platform linux/amd64,linux/arm64 -t <registry>/nrzpaper:1.0.0 --push .
```

Rust korskompileras, emuleras inte, så arm64-bygget är snabbt på en x86-maskin och tvärtom.

## Första stegen

1. **Ladda upp** några PDF:er eller foton: texten känns igen i bakgrunden.
2. **Taggar**: skapa taggar som *Faktura* eller *Försäkring* och ge dem en beskrivning. AI:n använder den för att tilldela taggar.
3. **AI** (*Inställningar → AI och modeller*): välj en leverantör, klistra in en nyckel (eller använd de inbyggda kostnadsfria OpenRouter-modellerna eller en lokal Ollama), sedan *Ladda modeller* och *Testa anslutning*.
4. **Skanner** (*Profil → Skanner*): *Sök i nätverket* hittar eSCL/AirScan-skannrar; sedan *Skanna* i sidofältet.
5. **E-post** (*Profil → E-postinkorg*): lägg till en eller flera IMAP-brevlådor; PDF:er och bilder från bilagor hamnar i inkorgen.
6. **Mappinkorg** (*Inställningar → Mappinkorg*, administratörer): bevaka en servermapp som `/scans` (montera den i containern först).
7. **Skrivare** (*Inställningar → Skrivare*, administratörer): *Sök i nätverket* eller ange `ipp://printer/ipp/print`; sedan kan varje medlem skriva ut från en dokumentsida.

## Konfiguration

Varje inställning kan anges som en miljövariabel med prefixet `NRZPAPER_` (eller i `Rocket.toml`).

| Variabel | Standard | Betydelse |
|---|---|---|
| `NRZPAPER_DATA_DIR` | `/data` (avbild) | Databas, nyckelfil och intagsmappar |
| `NRZPAPER_DB_KEY` | – | Databasnyckel; om den inte anges skapas `nrzpaper.key` i datamappen |
| `NRZPAPER_PORT` / `NRZPAPER_ADDRESS` | `8320` / `0.0.0.0` | Var servern lyssnar |
| `NRZPAPER_COOKIE_SECURE` | `false` | Sätt till `true` bakom HTTPS |
| `NRZPAPER_REGISTRATION_OPEN` | `false` | Tillåt självregistrering (kan också växlas i UI:t) |
| `NRZPAPER_OCR_LANGUAGES` | `deu+eng` | Tesseract-språk (avbilden innehåller `deu` och `eng`) |
| `NRZPAPER_OCR_WORKERS` | `2` | Parallella OCR-jobb |
| `NRZPAPER_TRASH_RETENTION_DAYS` | `30` | Dagar i papperskorgen före slutgiltig radering |
| `NRZPAPER_INBOX_KEEP_DAYS` | `7` | Dagar som icke godkända inkorgsobjekt sparas |
| `NRZPAPER_SCANNER_URL` | – | Förinställd skanner för nya användare, t.ex. `http://192.168.1.50` |
| `NRZPAPER_SCAN_SUBNETS` | `192.168.0.0/24,192.168.1.0/24` | Nätverk som genomsöks efter skannrar och skrivare |
| `NRZPAPER_AI_PROVIDER` | `openrouter` | `openai`, `anthropic`, `gemini`, `mistral`, `groq`, `ollama`, `custom`, … |
| `NRZPAPER_AI_KEY` / `NRZPAPER_AI_URL` / `NRZPAPER_AI_MODEL` | – | Nyckel, bas-URL och modell för den leverantören |
| `NRZPAPER_OPENROUTER_KEY` | – | OpenRouter-nyckel (alternativ till ovanstående) |
| `NRZPAPER_LOG` | `info` | Loggfilter, t.ex. `debug` |

Inställningar i webbgränssnittet (AI-leverantör, nycklar, modeller, registrering) lagras i den krypterade databasen och har företräde framför miljön.

### AI-leverantörer

| Leverantör | Adress | Anteckningar |
|---|---|---|
| OpenRouter | inbyggd | Kostnadsfria bildmodeller; egen nyckel valfri |
| OpenAI, Anthropic, Google Gemini, Mistral, Groq, xAI, DeepSeek, Together, Fireworks, Perplexity, Cohere, NVIDIA, Hugging Face, Qwen | inbyggd | API-nyckel från leverantören |
| Azure OpenAI | `https://NAME.openai.azure.com/openai/v1` | Modell = distributionsnamn |
| Ollama / LM Studio / vLLM / LocalAI | `http://host:11434/v1` osv. | Ingen nyckel behövs |
| Allt OpenAI-kompatibelt | din URL | Nyckel valfri |

Modeller som inte kan läsa bilder: markera *”Modellen kan inte läsa bilder – skicka igenkänd text”*. Leverantörer som inte accepterar PDF:er får automatiskt de första sidorna som bilder. Språket för föreslagna titlar och sammanfattningar ställs in under *Språk för förslagen*.

## REST API

Skapa en API-nyckel under *Profil → API-nycklar* (välj behörigheter). Alla slutpunkter ligger under `/api/v1` och tar `Authorization: Bearer np_…`.

```bash
TOKEN=np_your_key
BASE=http://localhost:8320/api/v1

# organisationer
curl -H "Authorization: Bearer $TOKEN" $BASE/organizations

# uppladdning (flera filer samtidigt med flera -F)
curl -H "Authorization: Bearer $TOKEN" -F file=@invoice.pdf \
     $BASE/organizations/$ORG/documents

# sökning: samma syntax som i UI:t
curl -H "Authorization: Bearer $TOKEN" -G $BASE/organizations/$ORG/documents \
     --data-urlencode 'searchQuery=northwind tag:invoice created:2026'

# ladda ner filen
curl -H "Authorization: Bearer $TOKEN" -o out.pdf \
     "$BASE/organizations/$ORG/documents/$DOC/file?download=true"

# lägg till en tagg
curl -H "Authorization: Bearer $TOKEN" -H "Content-Type: application/json" \
     -d '{"tagId":"'$TAG'"}' $BASE/organizations/$ORG/documents/$DOC/tags

# offentlig delningslänk giltig i 7 dagar
curl -H "Authorization: Bearer $TOKEN" -H "Content-Type: application/json" \
     -d '{"expiresInDays":7}' $BASE/organizations/$ORG/documents/$DOC/share-links

# hälsokontroll (ingen nyckel behövs)
curl $BASE/health
```

Webhooks (*Inställningar → Webhooks*) skickar signerad JSON (`webhook-id`, `webhook-timestamp`, `webhook-signature`) för `document:created`, `document:updated`, `document:deleted`, `document:restored`, `document:tag:added` och `document:tag:removed`.

Intagsmappen `data/ingestion/<organization-id>/` importerar filer direkt som dokument. Bevakade mappar (*Mappinkorg*) skickar dem först till inkorgen för granskning.

## Utveckling

```bash
# backend (Rust 1.85+, Tesseract och Poppler för OCR)
cargo run                         # http://localhost:8320
cargo test

# webbgränssnitt med hot reload (dx = dioxus-cli 0.7)
rustup target add wasm32-unknown-unknown
cargo install dioxus-cli --version 0.7.10 --locked
cd ui && dx serve                 # http://localhost:8080, proxar /api till :8320

# bädda in gränssnittet i programmet
scripts/ui.sh && cargo build --release
```

Översättningar finns i `locales/<language>.json` (tysk källtext → översättning). `scripts/i18n_extract.py` uppdaterar `locales/strings.json` och rapporterar saknade poster per språk.

Arkitekturdetaljer: [ARCHITECTURE.md](../ARCHITECTURE.md).

## Licens

AGPL-3.0-or-later. UI-komponenterna bygger på [rust-ui](https://github.com/rust-ui/ui) (MIT), PDF-visaren på [PDF.js](https://mozilla.github.io/pdf.js/) (Apache-2.0), typsnittet är [Inter](https://rsms.me/inter/) (OFL).
