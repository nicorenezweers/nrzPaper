<div align="center">

<img src="../static/icon.png" width="96" alt="nrzPaper">

# nrzPaper

**A Rust-powered document management tool.**

[English](../README.md) · [Deutsch](README.de.md) · [Français](README.fr.md) · [Español](README.es.md) · [Português](README.pt.md) · [Nederlands](README.nl.md)

![Documenti con cronologia](screenshots/documents.png)

</div>

nrzPaper è un unico programma senza database esterno. Tutto si trova in un file SQLite cifrato con SQLCipher, compresi i file stessi. Riconosce il testo con Tesseract, scansiona direttamente dagli scanner di rete, raccoglie gli allegati e-mail e lascia che un’IA a tua scelta proponga titoli, tag e metadati. **Dieter**, l’assistente integrato, risponde alle domande sui tuoi documenti e cita le fonti. L’interfaccia è un’app Rust/WebAssembly (Dioxus) con tema scuro e funziona anche su smartphone.

## Funzionalità

| | |
|---|---|
| **Acquisizione** | Trascina e rilascia, caricamento, **scanner di rete** (eSCL/AirScan: Canon, HP, Epson, Brother …), **caselle e-mail** (IMAP, più account), **cartelle monitorate** (ad es. la cartella di scansione del NAS), API REST |
| **Posta in arrivo** | Scansioni, e-mail, importazioni da cartelle e caricamenti attendono la revisione: l’IA propone nome, tag e campi, tu approvi uno alla volta o **tutti insieme**. Filtro per origine. |
| **Riconoscimento** | Tesseract e Poppler in background, anche per PDF scansionati e foto; ricerca full-text con prefissi, frasi e filtri (`tag:`, `-tag:`, `type:`, `created:>2026-06-01`, …) |
| **Organizzazione** | Tag colorati, **cartelle basate sui tag**, **regole di tagging**, **proprietà** personalizzate (testo, numero, data, sì/no, selezione, selezione multipla, utente, collegamento a documento) |
| **Cronologia** | Cronologia verticale accanto ai documenti: oggi in alto, ogni giorno con posta più in basso e i periodi vuoti nel mezzo. Filtro per tag, tipo e testo. |
| **Promemoria** | «Ricordamelo tra 10 giorni» su ogni documento, con una nota; i promemoria scaduti compaiono nella barra laterale e in cima alla cronologia. |
| **Stampa** | Su qualsiasi **stampante di rete IPP** (AirPrint, IPP Everywhere, Mopria, CUPS) trovata dal server. I PDF vengono inviati direttamente, oppure come immagini JPEG alle stampanti che accettano solo quel formato. |
| **IA, qualsiasi fornitore** | OpenRouter (modelli gratuiti integrati), OpenAI, Anthropic, Google Gemini, Mistral, Groq, xAI, DeepSeek, Together, Fireworks, Perplexity, Cohere, NVIDIA, Hugging Face, Qwen, Azure OpenAI, **Ollama, LM Studio, vLLM/LocalAI** o qualsiasi URL compatibile con OpenAI. I modelli solo testo ricevono il testo riconosciuto. |
| **Dieter** | Assistente chat in basso a destra: *«Qual è il mio numero di assicurazione?»*, *«Tutte le buste paga di Fabrikam del 2026»*. Cerca, legge i passaggi pertinenti e risponde con link ai documenti. |
| **Team** | Più organizzazioni, **più account per organizzazione** (inviti, ruoli proprietario/admin/membro, trasferimento della proprietà), chiavi API con permessi, webhook (firme Standard Webhooks), link di condivisione pubblici |
| **Lingue** | English, Deutsch, Français, Italiano, Español, Português, Nederlands (altre lingue in arrivo) |
| **Sicuro** | Un file cifrato, password Argon2, CSP senza script inline, protezione CSRF, file solo tramite API |

## Screenshot

| | |
|---|---|
| ![Dieter](screenshots/dieter.png) | ![Posta in arrivo](screenshots/inbox.png) |
| **Dieter** risponde con le fonti | **Posta in arrivo**: verifica, approva, filtra per origine |
| ![Documento](screenshots/document.png) | ![Fornitori IA](screenshots/settings-ai.png) |
| **Documento**: visualizzatore PDF, tag, promemoria, stampa | **IA e modelli**: qualsiasi fornitore, locale o cloud |
| ![Promemoria](screenshots/reminders.png) | ![Membri](screenshots/members.png) |
| **Promemoria** | **Membri** e inviti |

## Avvio rapido

Serve Docker. L’immagine contiene il programma, l’interfaccia, Tesseract (tedesco, inglese) e Poppler.

```bash
git clone https://github.com/nicorenezweers/nrzPaper.git
cd nrzPaper
docker build -t nrzpaper .
docker run -d --name nrzpaper -p 8320:8320 -v nrzpaper-data:/data nrzpaper
```

Apri <http://localhost:8320>. Il primo account creato diventa amministratore. Dopo di che la registrazione resta chiusa finché non la apri in *Impostazioni → Registrazione e utenti*, dove puoi anche creare account per altri.

### Docker Compose

```yaml
services:
  nrzpaper:
    build: .                      # oppure: image: ghcr.io/nicorenezweers/nrz-paper:1.0.0
    container_name: nrzpaper
    restart: unless-stopped
    ports:
      - "8320:8320"
    volumes:
      - nrzpaper-data:/data       # database cifrato + file della chiave
      - /srv/scans:/scans         # facoltativo: cartella monitorata
    environment:
      TZ: Europe/Rome
      # NRZPAPER_DB_KEY: "un-segreto-lungo"   # altrimenti viene creato /data/nrzpaper.key
      # NRZPAPER_COOKIE_SECURE: "true"        # dietro HTTPS
      # NRZPAPER_AI_PROVIDER: "openai"        # oppure nell’interfaccia
      # NRZPAPER_AI_KEY: "sk-…"

volumes:
  nrzpaper-data:
```

```bash
docker compose up -d
```

> **Fai il backup di `/data`.** Contiene `nrzpaper.db` (tutto, cifrato) e `nrzpaper.key`. Senza la chiave il database non si apre. Se imposti `NRZPAPER_DB_KEY`, conserva quel valore al sicuro.

### Entrambe le architetture

```bash
docker buildx build --platform linux/amd64,linux/arm64 -t <registry>/nrzpaper:1.0.0 --push .
```

## Primi passi

1. **Carica** qualche PDF o foto: il testo viene riconosciuto in background.
2. **Tag**: creali (ad es. *Fattura*, *Assicurazione*) con una descrizione; l’IA la usa per assegnarli.
3. **IA** (*Impostazioni → IA e modelli*): scegli un fornitore, incolla una chiave (oppure usa i modelli OpenRouter gratuiti integrati o un Ollama locale), poi *Carica modelli* e *Prova connessione*.
4. **Scanner** (*Profilo → Scanner*): *Cerca in rete* trova gli scanner eSCL/AirScan, poi *Scansiona* nella barra laterale.
5. **E-mail** (*Profilo → Posta in entrata*): una o più caselle IMAP; PDF e immagini allegati finiscono nella posta in arrivo.
6. **Cartella monitorata** (*Impostazioni → Cartelle*, admin): monitora una cartella del server come `/scans` (montala prima nel container).
7. **Stampanti** (*Impostazioni → Stampanti*, admin): *Cerca in rete* o inserisci `ipp://stampante/ipp/print`; ogni membro può poi stampare dalla pagina del documento.

## Configurazione

Ogni impostazione può essere data come variabile d’ambiente con prefisso `NRZPAPER_` (o in `Rocket.toml`).

| Variabile | Predefinito | Significato |
|---|---|---|
| `NRZPAPER_DATA_DIR` | `/data` (immagine) | Database, file della chiave, cartelle di importazione |
| `NRZPAPER_DB_KEY` | – | Chiave del database; altrimenti viene generato `nrzpaper.key` |
| `NRZPAPER_PORT` / `NRZPAPER_ADDRESS` | `8320` / `0.0.0.0` | Dove ascolta il server |
| `NRZPAPER_COOKIE_SECURE` | `false` | `true` dietro HTTPS |
| `NRZPAPER_REGISTRATION_OPEN` | `false` | Consenti la registrazione libera |
| `NRZPAPER_OCR_LANGUAGES` | `deu+eng` | Lingue di Tesseract |
| `NRZPAPER_OCR_WORKERS` | `2` | Riconoscimenti in parallelo |
| `NRZPAPER_TRASH_RETENTION_DAYS` | `30` | Giorni nel cestino |
| `NRZPAPER_INBOX_KEEP_DAYS` | `7` | Giorni di conservazione degli elementi non approvati |
| `NRZPAPER_SCANNER_URL` | – | Scanner predefinito, ad es. `http://192.168.1.50` |
| `NRZPAPER_SCAN_SUBNETS` | `192.168.0.0/24,192.168.1.0/24` | Reti in cui cercare scanner e stampanti |
| `NRZPAPER_AI_PROVIDER` | `openrouter` | `openai`, `anthropic`, `gemini`, `mistral`, `groq`, `ollama`, `custom`, … |
| `NRZPAPER_AI_KEY` / `_URL` / `_MODEL` | – | Chiave, indirizzo e modello del fornitore |
| `NRZPAPER_OPENROUTER_KEY` | – | Chiave OpenRouter (alternativa) |
| `NRZPAPER_LOG` | `info` | Livello di log, ad es. `debug` |

Le impostazioni fatte nell’interfaccia sono salvate cifrate nel database e hanno la precedenza sull’ambiente.

### Fornitori IA

| Fornitore | Indirizzo | Nota |
|---|---|---|
| OpenRouter | integrato | Modelli vision gratuiti; chiave propria facoltativa |
| OpenAI, Anthropic, Google Gemini, Mistral, Groq, xAI, DeepSeek, Together, Fireworks, Perplexity, Cohere, NVIDIA, Hugging Face, Qwen | integrato | Chiave API del fornitore |
| Azure OpenAI | `https://NOME.openai.azure.com/openai/v1` | Modello = nome del deployment |
| Ollama / LM Studio / vLLM / LocalAI | `http://host:11434/v1` ecc. | Nessuna chiave |
| Qualsiasi servizio compatibile OpenAI | il tuo URL | Chiave facoltativa |

## API REST

Crea una chiave API in *Profilo → Chiavi API*. Tutto si trova sotto `/api/v1` con `Authorization: Bearer np_…`.

```bash
TOKEN=np_tua_chiave
BASE=http://localhost:8320/api/v1

curl -H "Authorization: Bearer $TOKEN" $BASE/organizations
curl -H "Authorization: Bearer $TOKEN" -F file=@fattura.pdf $BASE/organizations/$ORG/documents
curl -H "Authorization: Bearer $TOKEN" -G $BASE/organizations/$ORG/documents \
     --data-urlencode 'searchQuery=northwind tag:fattura created:2026'
curl -H "Authorization: Bearer $TOKEN" -o out.pdf \
     "$BASE/organizations/$ORG/documents/$DOC/file?download=true"
curl $BASE/health
```

I webhook (*Impostazioni → Webhook*) inviano JSON firmato per `document:created`, `document:updated`, `document:deleted`, `document:restored`, `document:tag:added` e `document:tag:removed`.

## Sviluppo

```bash
cargo run && cargo test
rustup target add wasm32-unknown-unknown
cargo install dioxus-cli --version 0.7.10 --locked
cd ui && dx serve                       # http://localhost:8080
scripts/ui.sh && cargo build --release  # incorporare l’interfaccia nel programma
```

Le traduzioni si trovano in `locales/<lingua>.json`. Architettura: [ARCHITECTURE.md](../ARCHITECTURE.md).

## Licenza

AGPL-3.0-or-later. Componenti dell’interfaccia basati su [rust-ui](https://github.com/rust-ui/ui) (MIT), visualizzatore [PDF.js](https://mozilla.github.io/pdf.js/) (Apache-2.0), font [Inter](https://rsms.me/inter/) (OFL).
