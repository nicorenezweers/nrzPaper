<div align="center">

<img src="../static/icon.png" width="96" alt="nrzPaper">

# nrzPaper

**Selvhostet dokumenthåndtering i Rust: skann, gjenkjenn, merk og finn papirene dine — kryptert i én fil.**

[English](../README.md) · [Deutsch](README.de.md) · [Français](README.fr.md) · [Italiano](README.it.md) · [Español](README.es.md) · [Português](README.pt.md) · [Nederlands](README.nl.md) · [中文](README.zh.md) · [日本語](README.ja.md) · [Polski](README.pl.md) · [Русский](README.ru.md) · [العربية](README.ar.md) · [한국어](README.ko.md) · [Türkçe](README.tr.md) · [Bahasa Indonesia](README.id.md) · [हिन्दी](README.hi.md) · [Tiếng Việt](README.vi.md) · [ไทย](README.th.md) · [Українська](README.uk.md) · [Čeština](README.cs.md) · [Română](README.ro.md) · [Magyar](README.hu.md) · [Ελληνικά](README.el.md) · [Svenska](README.sv.md) · [Norsk](README.nb.md) · [Dansk](README.da.md) · [Suomi](README.fi.md) · [עברית](README.he.md) · [বাংলা](README.bn.md) · [فارسی](README.fa.md)

![Dokumenter med tidslinje](screenshots/documents.png)

</div>

nrzPaper er ett enkelt program uten ekstern database. Alt ligger i én SQLCipher-kryptert SQLite-fil, også filene selv. Det gjør OCR med Tesseract, skanner direkte fra nettverksskannere, henter e-postvedlegg og lar en AI du velger foreslå titler, tagger og metadata. Den innebygde assistenten **Dieter** svarer på spørsmål om dokumentene dine og lenker kildene. Nettgrensesnittet er en Rust/WebAssembly-app (Dioxus) i et rolig mørkt tema og fungerer også på telefoner.

## Funksjoner

| | |
|---|---|
| **Innsamling** | Dra og slipp, opplasting, **nettverksskannere** (eSCL/AirScan: Canon, HP, Epson, Brother …), **e-postinnbokser** (IMAP, flere kontoer), **overvåkede mapper** (f.eks. skannemappen på NAS-en), REST API |
| **Innboks** | Skann, e-post, mappeimport og opplastinger venter på gjennomgang: AI-en foreslår navn, tagger og felt, og du godkjenner ett om gangen eller **alle på én gang**. Filtrer etter kilde (skanner, hver postkasse, hver mappe, opplasting). |
| **Gjenkjenning** | Tesseract og Poppler i bakgrunnen, også for skannede PDF-er og bilder; fulltekstsøk med prefikser, fraser og filtre (`tag:`, `-tag:`, `type:`, `created:>2026-06-01`, …) |
| **Organisering** | Fargede tagger, **mapper laget av tagger** (nestede), **taggeregler** («inneholder Northwind → Faktura»), egendefinerte **egenskaper** (tekst, tall, dato, ja/nei, valg, flervalg, bruker, dokumentlenke) |
| **Tidslinje** | Loddrett tidslinje ved siden av dokumentene: i dag øverst, under hver dag med e-post og de stille hullene mellom dem («12 dager uten dokumenter»). Filtrer etter tagg, type og tekst. |
| **Påminnelser** | «Påminn meg om 10 dager» på ethvert dokument, med notat; forfalte påminnelser vises i sidepanelet og øverst på tidslinjen. |
| **Utskrift** | Skriv ut dokumenter på hvilken som helst **IPP-nettverksskriver** (AirPrint, IPP Everywhere, Mopria, CUPS) serveren finner. PDF-er går direkte, eller som sidebilder til skrivere som bare tar JPEG. |
| **AI, enhver leverandør** | OpenRouter (gratis modeller innebygd), OpenAI, Anthropic, Google Gemini, Mistral, Groq, xAI, DeepSeek, Together, Fireworks, Perplexity, Cohere, NVIDIA, Hugging Face, Qwen, Azure OpenAI, **Ollama, LM Studio, vLLM/LocalAI** eller en hvilken som helst OpenAI-kompatibel URL. Tekstbaserte modeller får OCR-teksten i stedet for bilder. |
| **Dieter** | Chatassistent (nederst til høyre): *«Hva er forsikringsnummeret mitt?»*, *«Alle lønnsslipper fra Fabrikam i 2026»*. Han søker, leser de samsvarende avsnittene og svarer med lenker til dokumentene. |
| **Team** | Flere organisasjoner, **flere kontoer per organisasjon** (invitasjon, roller: eier, administrator, medlem; overfør eierskap), API-nøkler med tillatelser, webhooks (Standard Webhooks-signaturer), offentlige delingslenker |
| **Språk** | English, Deutsch, Français, Italiano, Español, Português, Nederlands (flere språk kommer snart) |
| **Sikkert som standard** | Én kryptert fil, Argon2-passord, CSP uten innebygde skript, CSRF-beskyttelse, filer bare via API |

## Skjermbilder

| | |
|---|---|
| ![Dieter, assistenten](screenshots/dieter.png) | ![Innboks med AI-forslag](screenshots/inbox.png) |
| **Dieter** svarer med kilder | **Innboks**: gjennomgå og godkjenn, filtrer etter kilde |
| ![Dokumentvisning](screenshots/document.png) | ![AI-leverandører](screenshots/settings-ai.png) |
| **Dokument**: PDF-viser, tagger, påminnelser, utskrift | **AI og modeller**: enhver leverandør, lokal eller sky |
| ![Påminnelser](screenshots/reminders.png) | ![Medlemmer](screenshots/members.png) |
| **Påminnelser** | **Medlemmer** og invitasjoner |

## Hurtigstart

Du trenger Docker. Bildet inneholder programmet, nettgrensesnittet, Tesseract (tysk, engelsk) og Poppler.

```bash
git clone https://github.com/nicorenezweers/nrzPaper.git
cd nrzPaper
docker build -t nrzpaper .
docker run -d --name nrzpaper -p 8320:8320 -v nrzpaper-data:/data nrzpaper
```

Åpne <http://localhost:8320>. Den første kontoen du oppretter blir administrator. Etter det er registreringen stengt til du åpner den under *Innstillinger → Registrering og brukere*, der du også kan opprette kontoer for andre.

### Docker Compose

```yaml
services:
  nrzpaper:
    build: .                      # eller: image: ghcr.io/nicorenezweers/nrzpaper:1.0.0
    container_name: nrzpaper
    restart: unless-stopped
    ports:
      - "8320:8320"
    volumes:
      - nrzpaper-data:/data       # kryptert database + nøkkelfil
      - /srv/scans:/scans         # valgfritt: en mappe for mappeinnboksen
    environment:
      TZ: Europe/Zurich
      # NRZPAPER_DB_KEY: "a-long-secret"     # ellers opprettes /data/nrzpaper.key
      # NRZPAPER_COOKIE_SECURE: "true"       # når det serveres via HTTPS
      # NRZPAPER_AI_PROVIDER: "openai"       # eller angi i nettgrensesnittet
      # NRZPAPER_AI_KEY: "sk-…"

volumes:
  nrzpaper-data:
```

```bash
docker compose up -d
```

> **Ta sikkerhetskopi av `/data`.** Der ligger `nrzpaper.db` (alt, kryptert) og `nrzpaper.key`. Uten nøkkelen kan databasen ikke åpnes. Hvis du angir `NRZPAPER_DB_KEY`, oppbevar den verdien trygt i stedet.

### Begge arkitekturene

```bash
docker buildx build --platform linux/amd64,linux/arm64 -t <registry>/nrzpaper:1.0.0 --push .
```

Rust krysskompileres, ikke emuleres, så arm64-bygget er raskt på en x86-maskin og omvendt.

## Første steg

1. **Last opp** noen PDF-er eller bilder: teksten gjenkjennes i bakgrunnen.
2. **Tagger**: lag tagger som *Faktura* eller *Forsikring* og gi dem en beskrivelse. AI-en bruker den til å tildele tagger.
3. **AI** (*Innstillinger → AI og modeller*): velg en leverandør, lim inn en nøkkel (eller bruk de innebygde gratis OpenRouter-modellene eller en lokal Ollama), deretter *Last modeller* og *Test tilkobling*.
4. **Skanner** (*Profil → Skanner*): *Søk i nettverket* finner eSCL/AirScan-skannere; deretter *Skann* i sidepanelet.
5. **E-post** (*Profil → E-postinnboks*): legg til én eller flere IMAP-postkasser; PDF-er og bilder fra vedlegg havner i innboksen.
6. **Mappeinnboks** (*Innstillinger → Mappeinnboks*, administratorer): overvåk en servermappe som `/scans` (monter den i containeren først).
7. **Skrivere** (*Innstillinger → Skrivere*, administratorer): *Søk i nettverket* eller skriv inn `ipp://printer/ipp/print`; deretter kan hvert medlem skrive ut fra en dokumentside.

## Konfigurasjon

Hver innstilling kan angis som en miljøvariabel med prefikset `NRZPAPER_` (eller i `Rocket.toml`).

| Variabel | Standard | Betydning |
|---|---|---|
| `NRZPAPER_DATA_DIR` | `/data` (bilde) | Database, nøkkelfil og inntaksmapper |
| `NRZPAPER_DB_KEY` | – | Databasenøkkel; hvis ikke angitt, opprettes `nrzpaper.key` i datamappen |
| `NRZPAPER_PORT` / `NRZPAPER_ADDRESS` | `8320` / `0.0.0.0` | Hvor serveren lytter |
| `NRZPAPER_COOKIE_SECURE` | `false` | Sett til `true` bak HTTPS |
| `NRZPAPER_REGISTRATION_OPEN` | `false` | Tillat selvregistrering (kan også slås på i UI-et) |
| `NRZPAPER_OCR_LANGUAGES` | `deu+eng` | Tesseract-språk (bildet inneholder `deu` og `eng`) |
| `NRZPAPER_OCR_WORKERS` | `2` | Parallelle OCR-jobber |
| `NRZPAPER_TRASH_RETENTION_DAYS` | `30` | Dager i papirkurven før endelig sletting |
| `NRZPAPER_INBOX_KEEP_DAYS` | `7` | Dager elementer i innboksen som ikke er godkjent, beholdes |
| `NRZPAPER_SCANNER_URL` | – | Forhåndsinnstilt skanner for nye brukere, f.eks. `http://192.168.1.50` |
| `NRZPAPER_SCAN_SUBNETS` | `192.168.0.0/24,192.168.1.0/24` | Nettverk som søkes gjennom for skannere og skrivere |
| `NRZPAPER_AI_PROVIDER` | `openrouter` | `openai`, `anthropic`, `gemini`, `mistral`, `groq`, `ollama`, `custom`, … |
| `NRZPAPER_AI_KEY` / `NRZPAPER_AI_URL` / `NRZPAPER_AI_MODEL` | – | Nøkkel, basis-URL og modell for den leverandøren |
| `NRZPAPER_OPENROUTER_KEY` | – | OpenRouter-nøkkel (alternativ til ovenstående) |
| `NRZPAPER_LOG` | `info` | Loggfilter, f.eks. `debug` |

Innstillinger i nettgrensesnittet (AI-leverandør, nøkler, modeller, registrering) lagres i den krypterte databasen og har forrang over miljøet.

### AI-leverandører

| Leverandør | Adresse | Merknader |
|---|---|---|
| OpenRouter | innebygd | Gratis visjonsmodeller; egen nøkkel valgfritt |
| OpenAI, Anthropic, Google Gemini, Mistral, Groq, xAI, DeepSeek, Together, Fireworks, Perplexity, Cohere, NVIDIA, Hugging Face, Qwen | innebygd | API-nøkkel fra leverandøren |
| Azure OpenAI | `https://NAME.openai.azure.com/openai/v1` | Modell = distribusjonsnavn |
| Ollama / LM Studio / vLLM / LocalAI | `http://host:11434/v1` osv. | Ingen nøkkel nødvendig |
| Alt OpenAI-kompatibelt | din URL | Nøkkel valgfritt |

Modeller som ikke kan lese bilder: merk av for *«Modellen kan ikke lese bilder – send gjenkjent tekst»*. Leverandører som ikke godtar PDF-er får automatisk de første sidene som bilder. Språket for foreslåtte titler og sammendrag angis under *Språk for forslagene*.

## REST API

Opprett en API-nøkkel under *Profil → API-nøkler* (velg tillatelser). Alle endepunkter ligger under `/api/v1` og godtar `Authorization: Bearer np_…`.

```bash
TOKEN=np_your_key
BASE=http://localhost:8320/api/v1

# organisasjoner
curl -H "Authorization: Bearer $TOKEN" $BASE/organizations

# opplasting (flere filer samtidig med flere -F)
curl -H "Authorization: Bearer $TOKEN" -F file=@invoice.pdf \
     $BASE/organizations/$ORG/documents

# søk: samme syntaks som i UI-et
curl -H "Authorization: Bearer $TOKEN" -G $BASE/organizations/$ORG/documents \
     --data-urlencode 'searchQuery=northwind tag:invoice created:2026'

# last ned filen
curl -H "Authorization: Bearer $TOKEN" -o out.pdf \
     "$BASE/organizations/$ORG/documents/$DOC/file?download=true"

# legg til en tagg
curl -H "Authorization: Bearer $TOKEN" -H "Content-Type: application/json" \
     -d '{"tagId":"'$TAG'"}' $BASE/organizations/$ORG/documents/$DOC/tags

# offentlig delingslenke gyldig i 7 dager
curl -H "Authorization: Bearer $TOKEN" -H "Content-Type: application/json" \
     -d '{"expiresInDays":7}' $BASE/organizations/$ORG/documents/$DOC/share-links

# helsesjekk (ingen nøkkel nødvendig)
curl $BASE/health
```

Webhooks (*Innstillinger → Webhooks*) sender signert JSON (`webhook-id`, `webhook-timestamp`, `webhook-signature`) for `document:created`, `document:updated`, `document:deleted`, `document:restored`, `document:tag:added` og `document:tag:removed`.

Inntaksmappen `data/ingestion/<organization-id>/` importerer filer direkte som dokumenter. Overvåkede mapper (*Mappeinnboks*) sender dem først til innboksen for gjennomgang.

## Utvikling

```bash
# backend (Rust 1.85+, Tesseract og Poppler for OCR)
cargo run                         # http://localhost:8320
cargo test

# nettgrensesnitt med hot reload (dx = dioxus-cli 0.7)
rustup target add wasm32-unknown-unknown
cargo install dioxus-cli --version 0.7.10 --locked
cd ui && dx serve                 # http://localhost:8080, proxyer /api til :8320

# bygg inn grensesnittet i programmet
scripts/ui.sh && cargo build --release
```

Oversettelser ligger i `locales/<language>.json` (tysk kildetekst → oversettelse). `scripts/i18n_extract.py` oppdaterer `locales/strings.json` og rapporterer manglende oppføringer per språk.

Arkitekturdetaljer: [ARCHITECTURE.md](../ARCHITECTURE.md).

## Lisens

AGPL-3.0-or-later. UI-komponentene bygger på [rust-ui](https://github.com/rust-ui/ui) (MIT), PDF-viseren på [PDF.js](https://mozilla.github.io/pdf.js/) (Apache-2.0), skrifttypen er [Inter](https://rsms.me/inter/) (OFL).
