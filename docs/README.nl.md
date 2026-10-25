<div align="center">

<img src="../static/icon.png" width="96" alt="nrzPaper">

# nrzPaper

**A Rust-powered document management tool.**

[English](../README.md) · [Deutsch](README.de.md) · [Français](README.fr.md) · [Italiano](README.it.md) · [Español](README.es.md) · [Português](README.pt.md)

![Documenten met tijdlijn](screenshots/documents.png)

</div>

nrzPaper is één programma zonder externe database. Alles staat in één met SQLCipher versleuteld SQLite-bestand, ook de bestanden zelf. Het herkent tekst met Tesseract, scant rechtstreeks vanaf netwerkscanners, haalt e-mailbijlagen op en laat een AI naar keuze titels, tags en metadata voorstellen. **Dieter**, de ingebouwde assistent, beantwoordt vragen over je documenten en linkt naar de bronnen. De interface is een Rust/WebAssembly-app (Dioxus) met een donker thema en werkt ook op de telefoon.

## Functies

| | |
|---|---|
| **Vastleggen** | Slepen en neerzetten, uploaden, **netwerkscanners** (eSCL/AirScan: Canon, HP, Epson, Brother …), **mailboxen** (IMAP, meerdere accounts), **bewaakte mappen** (bijv. de scanmap op je NAS), REST-API |
| **Inbox** | Scans, e-mails, mapimports en uploads wachten op controle: de AI stelt naam, tags en velden voor, jij keurt één item of **alles tegelijk** goed. Filter op bron. |
| **Herkennen** | Tesseract en Poppler op de achtergrond, ook voor gescande PDF’s en foto’s; volledige-tekstzoeken met voorvoegsels, zinnen en filters (`tag:`, `-tag:`, `type:`, `created:>2026-06-01`, …) |
| **Ordenen** | Gekleurde tags, **mappen op basis van tags**, **tagregels**, eigen **eigenschappen** (tekst, getal, datum, ja/nee, keuze, meervoudige keuze, gebruiker, link naar document) |
| **Tijdlijn** | Verticale tijdlijn naast de documenten: vandaag bovenaan, elke dag met post eronder en de lege periodes daartussen. Filter op tag, type en tekst. |
| **Herinneringen** | „Herinner me over 10 dagen” bij elk document, met notitie; verlopen herinneringen staan in de zijbalk en bovenaan de tijdlijn. |
| **Afdrukken** | Op elke **IPP-netwerkprinter** (AirPrint, IPP Everywhere, Mopria, CUPS) die de server vindt. PDF’s gaan direct, of als JPEG-afbeeldingen naar printers die alleen dat formaat accepteren. |
| **AI, elke aanbieder** | OpenRouter (gratis modellen ingebouwd), OpenAI, Anthropic, Google Gemini, Mistral, Groq, xAI, DeepSeek, Together, Fireworks, Perplexity, Cohere, NVIDIA, Hugging Face, Qwen, Azure OpenAI, **Ollama, LM Studio, vLLM/LocalAI** of elke OpenAI-compatibele URL. Modellen die alleen tekst lezen krijgen de herkende tekst. |
| **Dieter** | Chatassistent rechtsonder: *„Wat is mijn verzekeringsnummer?”*, *„Alle loonstroken van Fabrikam uit 2026”*. Hij zoekt, leest de relevante passages en antwoordt met links naar de documenten. |
| **Teams** | Meerdere organisaties, **meerdere accounts per organisatie** (uitnodigingen, rollen eigenaar/beheerder/lid, eigendom overdragen), API-sleutels met rechten, webhooks (Standard Webhooks-handtekeningen), openbare deellinks |
| **Talen** | English, Deutsch, Français, Italiano, Español, Português, Nederlands (meer talen komen binnenkort) |
| **Veilig** | Eén versleuteld bestand, Argon2-wachtwoorden, CSP zonder inline scripts, CSRF-bescherming, bestanden alleen via de API |

## Schermafbeeldingen

| | |
|---|---|
| ![Dieter](screenshots/dieter.png) | ![Inbox](screenshots/inbox.png) |
| **Dieter** antwoordt met bronnen | **Inbox**: controleren, goedkeuren, filteren op bron |
| ![Document](screenshots/document.png) | ![AI-aanbieders](screenshots/settings-ai.png) |
| **Document**: PDF-viewer, tags, herinneringen, afdrukken | **AI en modellen**: elke aanbieder, lokaal of in de cloud |
| ![Herinneringen](screenshots/reminders.png) | ![Leden](screenshots/members.png) |
| **Herinneringen** | **Leden** en uitnodigingen |

## Snel starten

Je hebt Docker nodig. De image bevat het programma, de interface, Tesseract (Duits, Engels) en Poppler.

```bash
git clone https://github.com/nicorenezweers/nrzPaper.git
cd nrzPaper
docker build -t nrzpaper .
docker run -d --name nrzpaper -p 8320:8320 -v nrzpaper-data:/data nrzpaper
```

Open <http://localhost:8320>. Het eerste account wordt beheerder. Daarna is registreren gesloten totdat je het opent onder *Instellingen → Registratie en gebruikers*, waar je ook accounts voor anderen aanmaakt.

### Docker Compose

```yaml
services:
  nrzpaper:
    build: .                      # of: image: ghcr.io/nicorenezweers/nrzpaper:1.0.0
    container_name: nrzpaper
    restart: unless-stopped
    ports:
      - "8320:8320"
    volumes:
      - nrzpaper-data:/data       # versleutelde database + sleutelbestand
      - /srv/scans:/scans         # optioneel: bewaakte map
    environment:
      TZ: Europe/Amsterdam
      # NRZPAPER_DB_KEY: "een-lang-geheim"    # anders wordt /data/nrzpaper.key aangemaakt
      # NRZPAPER_COOKIE_SECURE: "true"        # achter HTTPS
      # NRZPAPER_AI_PROVIDER: "openai"        # of in de interface
      # NRZPAPER_AI_KEY: "sk-…"

volumes:
  nrzpaper-data:
```

```bash
docker compose up -d
```

> **Maak een back-up van `/data`.** Daarin staan `nrzpaper.db` (alles, versleuteld) en `nrzpaper.key`. Zonder de sleutel kan de database niet worden geopend. Als je `NRZPAPER_DB_KEY` instelt, bewaar die waarde dan goed.

### Beide architecturen

```bash
docker buildx build --platform linux/amd64,linux/arm64 -t <registry>/nrzpaper:1.0.0 --push .
```

## Eerste stappen

1. **Upload** een paar PDF’s of foto’s: de tekst wordt op de achtergrond herkend.
2. **Tags**: maak ze aan (bijv. *Factuur*, *Verzekering*) met een beschrijving; de AI gebruikt die om ze toe te wijzen.
3. **AI** (*Instellingen → AI en modellen*): kies een aanbieder, plak een sleutel (of gebruik de ingebouwde gratis OpenRouter-modellen of een lokale Ollama), daarna *Modellen laden* en *Verbinding testen*.
4. **Scanner** (*Profiel → Scanner*): *Netwerk doorzoeken* vindt eSCL/AirScan-scanners; daarna *Scannen* in de zijbalk.
5. **E-mail** (*Profiel → Inkomende e-mail*): één of meer IMAP-mailboxen; PDF’s en afbeeldingen uit bijlagen komen in de inbox.
6. **Bewaakte map** (*Instellingen → Mappen*, beheerders): bewaak een servermap zoals `/scans` (koppel die eerst in de container).
7. **Printers** (*Instellingen → Printers*, beheerders): *Netwerk doorzoeken* of voer `ipp://printer/ipp/print` in; elk lid kan daarna afdrukken vanaf de documentpagina.

## Configuratie

Elke instelling kan als omgevingsvariabele met het voorvoegsel `NRZPAPER_` worden opgegeven (of in `Rocket.toml`).

| Variabele | Standaard | Betekenis |
|---|---|---|
| `NRZPAPER_DATA_DIR` | `/data` (image) | Database, sleutelbestand, importmappen |
| `NRZPAPER_DB_KEY` | – | Databasesleutel; ontbreekt die, dan wordt `nrzpaper.key` aangemaakt |
| `NRZPAPER_PORT` / `NRZPAPER_ADDRESS` | `8320` / `0.0.0.0` | Waar de server luistert |
| `NRZPAPER_COOKIE_SECURE` | `false` | `true` achter HTTPS |
| `NRZPAPER_REGISTRATION_OPEN` | `false` | Vrije registratie toestaan |
| `NRZPAPER_OCR_LANGUAGES` | `deu+eng` | Tesseract-talen |
| `NRZPAPER_OCR_WORKERS` | `2` | Gelijktijdige herkenningen |
| `NRZPAPER_TRASH_RETENTION_DAYS` | `30` | Dagen in de prullenbak |
| `NRZPAPER_INBOX_KEEP_DAYS` | `7` | Dagen dat niet-goedgekeurde items bewaard blijven |
| `NRZPAPER_SCANNER_URL` | – | Vooraf ingestelde scanner, bijv. `http://192.168.1.50` |
| `NRZPAPER_SCAN_SUBNETS` | `192.168.0.0/24,192.168.1.0/24` | Netwerken waarin naar scanners en printers wordt gezocht |
| `NRZPAPER_AI_PROVIDER` | `openrouter` | `openai`, `anthropic`, `gemini`, `mistral`, `groq`, `ollama`, `custom`, … |
| `NRZPAPER_AI_KEY` / `_URL` / `_MODEL` | – | Sleutel, adres en model van de aanbieder |
| `NRZPAPER_OPENROUTER_KEY` | – | OpenRouter-sleutel (alternatief) |
| `NRZPAPER_LOG` | `info` | Logniveau, bijv. `debug` |

Instellingen uit de interface worden versleuteld in de database opgeslagen en gaan voor op de omgeving.

### AI-aanbieders

| Aanbieder | Adres | Opmerking |
|---|---|---|
| OpenRouter | ingebouwd | Gratis vision-modellen; eigen sleutel optioneel |
| OpenAI, Anthropic, Google Gemini, Mistral, Groq, xAI, DeepSeek, Together, Fireworks, Perplexity, Cohere, NVIDIA, Hugging Face, Qwen | ingebouwd | API-sleutel van de aanbieder |
| Azure OpenAI | `https://NAAM.openai.azure.com/openai/v1` | Model = naam van de deployment |
| Ollama / LM Studio / vLLM / LocalAI | `http://host:11434/v1` enz. | Geen sleutel nodig |
| Alles wat OpenAI-compatibel is | je eigen URL | Sleutel optioneel |

## REST-API

Maak een API-sleutel aan onder *Profiel → API-sleutels*. Alles staat onder `/api/v1` met `Authorization: Bearer np_…`.

```bash
TOKEN=np_jouw_sleutel
BASE=http://localhost:8320/api/v1

curl -H "Authorization: Bearer $TOKEN" $BASE/organizations
curl -H "Authorization: Bearer $TOKEN" -F file=@factuur.pdf $BASE/organizations/$ORG/documents
curl -H "Authorization: Bearer $TOKEN" -G $BASE/organizations/$ORG/documents \
     --data-urlencode 'searchQuery=northwind tag:factuur created:2026'
curl -H "Authorization: Bearer $TOKEN" -o out.pdf \
     "$BASE/organizations/$ORG/documents/$DOC/file?download=true"
curl $BASE/health
```

Webhooks (*Instellingen → Webhooks*) sturen ondertekende JSON voor `document:created`, `document:updated`, `document:deleted`, `document:restored`, `document:tag:added` en `document:tag:removed`.

## Ontwikkeling

```bash
cargo run && cargo test
rustup target add wasm32-unknown-unknown
cargo install dioxus-cli --version 0.7.10 --locked
cd ui && dx serve                       # http://localhost:8080
scripts/ui.sh && cargo build --release  # de interface in het programma inbouwen
```

Vertalingen staan in `locales/<taal>.json`. Architectuur: [ARCHITECTURE.md](../ARCHITECTURE.md).

## Licentie

AGPL-3.0-or-later. Interfacecomponenten gebaseerd op [rust-ui](https://github.com/rust-ui/ui) (MIT), viewer [PDF.js](https://mozilla.github.io/pdf.js/) (Apache-2.0), lettertype [Inter](https://rsms.me/inter/) (OFL).
