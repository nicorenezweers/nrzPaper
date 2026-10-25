<div align="center">

<img src="static/icon.png" width="96" alt="nrzPaper">

# nrzPaper

**Self-hosted document management in Rust: scan, recognize, tag and find your paperwork, encrypted in one file.**

[Deutsch](docs/README.de.md) · [Français](docs/README.fr.md) · [Italiano](docs/README.it.md) · [Español](docs/README.es.md) · [Português](docs/README.pt.md) · [Nederlands](docs/README.nl.md)

![Documents with timeline](docs/screenshots/documents.png)

</div>

nrzPaper is a single program with no external database. Everything lives in one SQLCipher-encrypted SQLite file, including the files themselves. It does OCR with Tesseract, scans straight from network scanners, picks up e-mail attachments, and lets an AI of your choice suggest titles, tags and metadata. **Dieter**, the built-in assistant, answers questions about your documents and links the sources. The web interface is a Rust/WebAssembly app (Dioxus) in a calm dark theme and also works on phones.

## Features

| | |
|---|---|
| **Capture** | Drag & drop, upload, **network scanners** (eSCL/AirScan: Canon, HP, Epson, Brother …), **e-mail inboxes** (IMAP, several accounts), **watched folders** (e.g. the scan folder on your NAS), REST API |
| **Inbox** | Scans, mails, folder imports and uploads wait for review: the AI suggests name, tags and fields, and you approve one item or **all at once**. Filter by source (scanner, each mailbox, each folder, upload). |
| **Recognize** | Tesseract and Poppler in the background, also for scanned PDFs and photos; full-text search with prefixes, phrases and filters (`tag:`, `-tag:`, `type:`, `created:>2026-06-01`, …) |
| **Organize** | Tags with colors, **folders built from tags** (nested), **tagging rules** (“contains Northwind → Invoice”), custom **properties** (text, number, date, yes/no, select, multi-select, user, document link) |
| **Timeline** | A vertical timeline next to your documents: today at the top, every day with mail below, and the quiet gaps in between (“12 days without documents”). Filter by tag, type and text. |
| **Reminders** | “Remind me in 10 days” on any document, with a note; due reminders show in the sidebar and at the top of the timeline. |
| **Print** | Print documents on any **IPP network printer** (AirPrint, IPP Everywhere, Mopria, CUPS) the server finds. PDFs go directly, or as page images to printers that only accept JPEG. |
| **AI, any provider** | OpenRouter (free models built in), OpenAI, Anthropic, Google Gemini, Mistral, Groq, xAI, DeepSeek, Together, Fireworks, Perplexity, Cohere, NVIDIA, Hugging Face, Qwen, Azure OpenAI, **Ollama, LM Studio, vLLM/LocalAI** or any OpenAI-compatible URL. Text-only models get the OCR text instead of images. |
| **Dieter** | Chat assistant (bottom right): *“What is my insurance number?”*, *“All payslips from Fabrikam in 2026”*. He searches, reads the matching passages and answers with links to the documents. |
| **Teams** | Several organizations, **several accounts per organization** (invite, roles: owner, admin, member; transfer ownership), API keys with permissions, webhooks (Standard Webhooks signatures), public share links |
| **Languages** | English, Deutsch, Français, Italiano, Español, Português, Nederlands (more via `locales/<code>.json`) |
| **Safe by default** | One encrypted file, Argon2 passwords, CSP without inline scripts, CSRF protection, files only through the API |

## Screenshots

| | |
|---|---|
| ![Dieter, the assistant](docs/screenshots/dieter.png) | ![Inbox with AI suggestions](docs/screenshots/inbox.png) |
| **Dieter** answers with sources | **Inbox**: review and approve, filter by source |
| ![Document view](docs/screenshots/document.png) | ![AI providers](docs/screenshots/settings-ai.png) |
| **Document**: PDF viewer, tags, reminders, print | **AI & models**: every provider, local or cloud |
| ![Reminders](docs/screenshots/reminders.png) | ![Members](docs/screenshots/members.png) |
| **Reminders** | **Members** and invitations |

## Quick start

You need Docker. The image contains the program, the web interface, Tesseract (German, English) and Poppler.

```bash
git clone https://github.com/nicorenezweers/nrzPaper.git
cd nrzPaper
docker build -t nrzpaper .
docker run -d --name nrzpaper -p 8320:8320 -v nrzpaper-data:/data nrzpaper
```

Open <http://localhost:8320>. The first account you create becomes the administrator. After that, registration is closed until you open it under *Settings → Registration & users*, where you can also create accounts for others.

### Docker Compose

```yaml
services:
  nrzpaper:
    build: .                      # or: image: ghcr.io/nicorenezweers/nrzpaper:1.0.0
    container_name: nrzpaper
    restart: unless-stopped
    ports:
      - "8320:8320"
    volumes:
      - nrzpaper-data:/data       # encrypted database + key file
      - /srv/scans:/scans         # optional: a folder for the folder inbox
    environment:
      TZ: Europe/Zurich
      # NRZPAPER_DB_KEY: "a-long-secret"     # otherwise /data/nrzpaper.key is created
      # NRZPAPER_COOKIE_SECURE: "true"       # when served via HTTPS
      # NRZPAPER_AI_PROVIDER: "openai"       # or set it in the web interface
      # NRZPAPER_AI_KEY: "sk-…"

volumes:
  nrzpaper-data:
```

```bash
docker compose up -d
```

> **Back up `/data`.** It holds `nrzpaper.db` (everything, encrypted) and `nrzpaper.key`. Without the key the database cannot be opened. If you set `NRZPAPER_DB_KEY`, keep that value safe instead.

### Both architectures

```bash
docker buildx build --platform linux/amd64,linux/arm64 -t <registry>/nrzpaper:1.0.0 --push .
```

Rust is cross-compiled, not emulated, so the arm64 build is fast on an x86 machine and vice versa.

## First steps

1. **Upload** a few PDFs or photos: the text is recognized in the background.
2. **Tags**: create tags such as *Invoice* or *Insurance* and give them a description. The AI uses it to assign them.
3. **AI** (*Settings → AI & models*): pick a provider, paste a key (or use the built-in free OpenRouter models, or a local Ollama), then *Load models* and *Test connection*.
4. **Scanner** (*Profile → Scanner*): *Search the network* finds eSCL/AirScan scanners; then *Scan* in the sidebar.
5. **E-mail** (*Profile → E-mail inbox*): add one or more IMAP mailboxes; PDFs and images from attachments land in the inbox.
6. **Folder inbox** (*Settings → Folder inbox*, admins): watch a server folder such as `/scans` (mount it into the container first).
7. **Printers** (*Settings → Printers*, admins): *Search the network* or enter `ipp://printer/ipp/print`; every member can then print from a document page.

## Configuration

Every setting can be given as an environment variable with the prefix `NRZPAPER_` (or in `Rocket.toml`).

| Variable | Default | Meaning |
|---|---|---|
| `NRZPAPER_DATA_DIR` | `/data` (image) | Database, key file and ingestion folders |
| `NRZPAPER_DB_KEY` | – | Database key; if unset, `nrzpaper.key` is generated in the data folder |
| `NRZPAPER_PORT` / `NRZPAPER_ADDRESS` | `8320` / `0.0.0.0` | Where the server listens |
| `NRZPAPER_COOKIE_SECURE` | `false` | Set to `true` behind HTTPS |
| `NRZPAPER_REGISTRATION_OPEN` | `false` | Allow self-registration (can also be switched in the UI) |
| `NRZPAPER_OCR_LANGUAGES` | `deu+eng` | Tesseract languages (the image ships `deu` and `eng`) |
| `NRZPAPER_OCR_WORKERS` | `2` | Parallel OCR jobs |
| `NRZPAPER_TRASH_RETENTION_DAYS` | `30` | Days in the trash before final deletion |
| `NRZPAPER_INBOX_KEEP_DAYS` | `7` | Days unapproved inbox items are kept |
| `NRZPAPER_SCANNER_URL` | – | Preset scanner for new users, e.g. `http://192.168.1.50` |
| `NRZPAPER_SCAN_SUBNETS` | `192.168.0.0/24,192.168.1.0/24` | Networks searched for scanners and printers |
| `NRZPAPER_AI_PROVIDER` | `openrouter` | `openai`, `anthropic`, `gemini`, `mistral`, `groq`, `ollama`, `custom`, … |
| `NRZPAPER_AI_KEY` / `NRZPAPER_AI_URL` / `NRZPAPER_AI_MODEL` | – | Key, base URL and model for that provider |
| `NRZPAPER_OPENROUTER_KEY` | – | OpenRouter key (alternative to the above) |
| `NRZPAPER_LOG` | `info` | Log filter, e.g. `debug` |

Settings made in the web interface (AI provider, keys, models, registration) are stored in the encrypted database and take precedence over the environment.

### AI providers

| Provider | Address | Notes |
|---|---|---|
| OpenRouter | built in | Free vision models; optional own key |
| OpenAI, Anthropic, Google Gemini, Mistral, Groq, xAI, DeepSeek, Together, Fireworks, Perplexity, Cohere, NVIDIA, Hugging Face, Qwen | built in | API key from the provider |
| Azure OpenAI | `https://NAME.openai.azure.com/openai/v1` | Model = deployment name |
| Ollama / LM Studio / vLLM / LocalAI | `http://host:11434/v1` etc. | No key needed |
| Anything OpenAI-compatible | your URL | Optional key |

Models that cannot read images: tick *“Model can’t read images – send recognized text”*. Providers that don’t accept PDFs automatically get the first pages as images. The language of suggested titles and summaries is set under *Language of the suggestions*.

## REST API

Create an API key under *Profile → API keys* (choose the permissions). All endpoints live under `/api/v1` and accept `Authorization: Bearer np_…`.

```bash
TOKEN=np_your_key
BASE=http://localhost:8320/api/v1

# organizations
curl -H "Authorization: Bearer $TOKEN" $BASE/organizations

# upload (several files at once with several -F)
curl -H "Authorization: Bearer $TOKEN" -F file=@invoice.pdf \
     $BASE/organizations/$ORG/documents

# search: same syntax as in the UI
curl -H "Authorization: Bearer $TOKEN" -G $BASE/organizations/$ORG/documents \
     --data-urlencode 'searchQuery=northwind tag:invoice created:2026'

# download the file
curl -H "Authorization: Bearer $TOKEN" -o out.pdf \
     "$BASE/organizations/$ORG/documents/$DOC/file?download=true"

# add a tag
curl -H "Authorization: Bearer $TOKEN" -H "Content-Type: application/json" \
     -d '{"tagId":"'$TAG'"}' $BASE/organizations/$ORG/documents/$DOC/tags

# public share link valid for 7 days
curl -H "Authorization: Bearer $TOKEN" -H "Content-Type: application/json" \
     -d '{"expiresInDays":7}' $BASE/organizations/$ORG/documents/$DOC/share-links

# health check (no key needed)
curl $BASE/health
```

Webhooks (*Settings → Webhooks*) send signed JSON (`webhook-id`, `webhook-timestamp`, `webhook-signature`) for `document:created`, `document:updated`, `document:deleted`, `document:restored`, `document:tag:added` and `document:tag:removed`.

The ingestion folder `data/ingestion/<organization-id>/` imports files directly as documents. Watched folders (*Folder inbox*) send them to the inbox for review first.

## Development

```bash
# backend (Rust 1.85+, Tesseract and Poppler for OCR)
cargo run                         # http://localhost:8320
cargo test

# web interface with hot reload (dx = dioxus-cli 0.7)
rustup target add wasm32-unknown-unknown
cargo install dioxus-cli --version 0.7.10 --locked
cd ui && dx serve                 # http://localhost:8080, proxies /api to :8320

# embed the interface into the program
scripts/ui.sh && cargo build --release
```

Translations live in `locales/<language>.json` (German source text → translation). `scripts/i18n_extract.py` refreshes `locales/strings.json` and reports missing entries per language.

Architecture details: [ARCHITECTURE.md](ARCHITECTURE.md).

## License

AGPL-3.0-or-later. The UI components are based on [rust-ui](https://github.com/rust-ui/ui) (MIT), the PDF viewer on [PDF.js](https://mozilla.github.io/pdf.js/) (Apache-2.0), the font is [Inter](https://rsms.me/inter/) (OFL).
