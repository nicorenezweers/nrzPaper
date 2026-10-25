<div align="center">

<img src="../static/icon.png" width="96" alt="nrzPaper">

# nrzPaper

**A Rust-powered document management tool.**

[English](../README.md) · [Deutsch](README.de.md) · [Français](README.fr.md) · [Italiano](README.it.md) · [Español](README.es.md) · [Português](README.pt.md) · [Nederlands](README.nl.md) · [中文](README.zh.md) · [日本語](README.ja.md) · [Polski](README.pl.md) · [Русский](README.ru.md) · [العربية](README.ar.md) · [한국어](README.ko.md) · [Türkçe](README.tr.md) · [Bahasa Indonesia](README.id.md) · [हिन्दी](README.hi.md) · [Tiếng Việt](README.vi.md) · [ไทย](README.th.md) · [Українська](README.uk.md) · [Čeština](README.cs.md) · [Română](README.ro.md) · [Magyar](README.hu.md) · [Ελληνικά](README.el.md) · [Svenska](README.sv.md) · [Norsk](README.nb.md) · [Dansk](README.da.md) · [Suomi](README.fi.md) · [עברית](README.he.md) · [বাংলা](README.bn.md) · [فارسی](README.fa.md)

![Dokumenty s časovou osou](screenshots/documents.png)

</div>

nrzPaper je jediný program bez externí databáze. Vše je v jednom souboru SQLite šifrovaném SQLCipher, včetně samotných souborů. Provádí OCR pomocí Tesseractu, skenuje přímo ze síťových skenerů, stahuje e-mailové přílohy a nechává vámi zvolenou AI navrhovat názvy, štítky a metadata. Vestavěný asistent **Dieter** odpovídá na otázky o vašich dokumentech a odkazuje na zdroje. Webové rozhraní je aplikace Rust/WebAssembly (Dioxus) v klidném tmavém motivu a funguje i na telefonech.

## Funkce

| | |
|---|---|
| **Zachytávání** | Přetažení, nahrání, **síťové skenery** (eSCL/AirScan: Canon, HP, Epson, Brother …), **e-mailové schránky** (IMAP, několik účtů), **sledované složky** (např. složka skenů na NAS), REST API |
| **Doručená pošta** | Skeny, maily, importy ze složek a nahrávky čekají na kontrolu: AI navrhne název, štítky a pole a vy schválíte jednotlivě nebo **vše najednou**. Filtr podle zdroje (skener, každá schránka, každá složka, nahrání). |
| **Rozpoznávání** | Tesseract a Poppler na pozadí, i pro skenované PDF a fotografie; fulltextové hledání s prefixy, frázemi a filtry (`tag:`, `-tag:`, `type:`, `created:>2026-06-01`, …) |
| **Organizace** | Barevné štítky, **složky ze štítků** (vnořené), **pravidla štítkování** („obsahuje Northwind → Faktura“), vlastní **vlastnosti** (text, číslo, datum, ano/ne, výběr, vícenásobný výběr, uživatel, odkaz na dokument) |
| **Časová osa** | Svislá časová osa vedle dokumentů: dnes nahoře, pod tím každý den s poštou a tichými mezerami mezi nimi („12 dní bez dokumentů“). Filtr podle štítku, typu a textu. |
| **Připomínky** | „Připomeň mi za 10 dní“ u libovolného dokumentu s poznámkou; splatné připomínky se zobrazují v postranním panelu a nahoře na časové ose. |
| **Tisk** | Tisk dokumentů na libovolné **síťové tiskárně IPP** (AirPrint, IPP Everywhere, Mopria, CUPS), kterou server najde. PDF jdou přímo, nebo jako obrázky stránek do tiskáren, které přijímají jen JPEG. |
| **AI, jakýkoli poskytovatel** | OpenRouter (vestavěné bezplatné modely), OpenAI, Anthropic, Google Gemini, Mistral, Groq, xAI, DeepSeek, Together, Fireworks, Perplexity, Cohere, NVIDIA, Hugging Face, Qwen, Azure OpenAI, **Ollama, LM Studio, vLLM/LocalAI** nebo jakákoli adresa kompatibilní s OpenAI. Textové modely dostanou rozpoznaný text místo obrázků. |
| **Dieter** | Chatový asistent (vpravo dole): *„Jaké je moje číslo pojištění?“*, *„Všechny výplatní pásky od Fabrikam v roce 2026“*. Vyhledá, přečte odpovídající pasáže a odpoví s odkazy na dokumenty. |
| **Týmy** | Několik organizací, **několik účtů na organizaci** (pozvánka, role: vlastník, správce, člen; převod vlastnictví), API klíče s oprávněními, webhooky (podpisy Standard Webhooks), veřejné odkazy ke sdílení |
| **Jazyky** | English, Deutsch, Français, Italiano, Español, Português, Nederlands (další jazyky brzy) |
| **Ve výchozím nastavení bezpečné** | Jeden šifrovaný soubor, hesla Argon2, CSP bez inline skriptů, ochrana CSRF, soubory pouze přes API |

## Snímky obrazovky

| | |
|---|---|
| ![Dieter, asistent](screenshots/dieter.png) | ![Doručená pošta s návrhy AI](screenshots/inbox.png) |
| **Dieter** odpovídá se zdroji | **Doručená pošta**: kontrola a schválení, filtr podle zdroje |
| ![Zobrazení dokumentu](screenshots/document.png) | ![Poskytovatelé AI](screenshots/settings-ai.png) |
| **Dokument**: prohlížeč PDF, štítky, připomínky, tisk | **AI a modely**: každý poskytovatel, lokální i cloudový |
| ![Připomínky](screenshots/reminders.png) | ![Členové](screenshots/members.png) |
| **Připomínky** | **Členové** a pozvánky |

## Rychlý start

Potřebujete Docker. Obraz obsahuje program, webové rozhraní, Tesseract (němčina, angličtina) a Poppler.

```bash
git clone https://github.com/nicorenezweers/nrzPaper.git
cd nrzPaper
docker build -t nrzpaper .
docker run -d --name nrzpaper -p 8320:8320 -v nrzpaper-data:/data nrzpaper
```

Otevřete <http://localhost:8320>. První vytvořený účet se stane správcem. Poté je registrace uzavřena, dokud ji neotevřete v *Nastavení → Registrace a uživatelé*, kde můžete vytvářet účty i pro ostatní.

### Docker Compose

```yaml
services:
  nrzpaper:
    build: .                      # nebo: image: ghcr.io/nicorenezweers/nrzpaper:1.0.0
    container_name: nrzpaper
    restart: unless-stopped
    ports:
      - "8320:8320"
    volumes:
      - nrzpaper-data:/data       # šifrovaná databáze + soubor klíče
      - /srv/scans:/scans         # volitelné: složka pro složkovou doručenou poštu
    environment:
      TZ: Europe/Zurich
      # NRZPAPER_DB_KEY: "a-long-secret"     # jinak se vytvoří /data/nrzpaper.key
      # NRZPAPER_COOKIE_SECURE: "true"       # při provozu přes HTTPS
      # NRZPAPER_AI_PROVIDER: "openai"       # nebo nastavte ve webovém rozhraní
      # NRZPAPER_AI_KEY: "sk-…"

volumes:
  nrzpaper-data:
```

```bash
docker compose up -d
```

> **Zálohujte `/data`.** Obsahuje `nrzpaper.db` (vše, šifrované) a `nrzpaper.key`. Bez klíče databázi nelze otevřít. Pokud nastavíte `NRZPAPER_DB_KEY`, uchovávejte tuto hodnotu v bezpečí.

### Obě architektury

```bash
docker buildx build --platform linux/amd64,linux/arm64 -t <registry>/nrzpaper:1.0.0 --push .
```

Rust se křížově kompiluje, nikoli emuluje, takže build arm64 je rychlý na stroji x86 a naopak.

## První kroky

1. **Nahrajte** několik PDF nebo fotografií: text se rozpozná na pozadí.
2. **Štítky**: vytvořte štítky jako *Faktura* nebo *Pojištění* a přidejte popis. AI je podle něj přiřazuje.
3. **AI** (*Nastavení → AI a modely*): vyberte poskytovatele, vložte klíč (nebo použijte vestavěné bezplatné modely OpenRouter či lokální Ollama), poté *Načíst modely* a *Otestovat připojení*.
4. **Skener** (*Profil → Skener*): *Hledat v síti* najde skenery eSCL/AirScan; poté *Skenovat* v postranním panelu.
5. **E-mail** (*Profil → E-mailová doručená pošta*): přidejte jednu nebo více schránek IMAP; PDF a obrázky z příloh přijdou do doručené pošty.
6. **Složková doručená pošta** (*Nastavení → Složková doručená pošta*, správci): sledujte složku serveru jako `/scans` (nejprve ji připojte do kontejneru).
7. **Tiskárny** (*Nastavení → Tiskárny*, správci): *Hledat v síti* nebo zadejte `ipp://printer/ipp/print`; poté může každý člen tisknout ze stránky dokumentu.

## Konfigurace

Každé nastavení lze zadat jako proměnnou prostředí s předponou `NRZPAPER_` (nebo v `Rocket.toml`).

| Proměnná | Výchozí | Význam |
|---|---|---|
| `NRZPAPER_DATA_DIR` | `/data` (obraz) | Databáze, soubor klíče a složky pro příjem |
| `NRZPAPER_DB_KEY` | – | Klíč databáze; pokud není nastaven, ve složce dat se vytvoří `nrzpaper.key` |
| `NRZPAPER_PORT` / `NRZPAPER_ADDRESS` | `8320` / `0.0.0.0` | Kde server naslouchá |
| `NRZPAPER_COOKIE_SECURE` | `false` | Za HTTPS nastavte `true` |
| `NRZPAPER_REGISTRATION_OPEN` | `false` | Povolit samoregistraci (lze přepnout i v UI) |
| `NRZPAPER_OCR_LANGUAGES` | `deu+eng` | Jazyky Tesseractu (obraz obsahuje `deu` a `eng`) |
| `NRZPAPER_OCR_WORKERS` | `2` | Paralelní úlohy OCR |
| `NRZPAPER_TRASH_RETENTION_DAYS` | `30` | Dny v koši před trvalým smazáním |
| `NRZPAPER_INBOX_KEEP_DAYS` | `7` | Kolik dní se uchovávají neschválené položky doručené pošty |
| `NRZPAPER_SCANNER_URL` | – | Přednastavený skener pro nové uživatele, např. `http://192.168.1.50` |
| `NRZPAPER_SCAN_SUBNETS` | `192.168.0.0/24,192.168.1.0/24` | Sítě pro hledání skenerů a tiskáren |
| `NRZPAPER_AI_PROVIDER` | `openrouter` | `openai`, `anthropic`, `gemini`, `mistral`, `groq`, `ollama`, `custom`, … |
| `NRZPAPER_AI_KEY` / `NRZPAPER_AI_URL` / `NRZPAPER_AI_MODEL` | – | Klíč, základní URL a model pro daného poskytovatele |
| `NRZPAPER_OPENROUTER_KEY` | – | Klíč OpenRouter (alternativa k výše uvedenému) |
| `NRZPAPER_LOG` | `info` | Filtr logů, např. `debug` |

Nastavení provedená ve webovém rozhraní (poskytovatel AI, klíče, modely, registrace) se ukládají do šifrované databáze a mají přednost před prostředím.

### Poskytovatelé AI

| Poskytovatel | Adresa | Poznámky |
|---|---|---|
| OpenRouter | vestavěný | Bezplatné vizuální modely; vlastní klíč volitelný |
| OpenAI, Anthropic, Google Gemini, Mistral, Groq, xAI, DeepSeek, Together, Fireworks, Perplexity, Cohere, NVIDIA, Hugging Face, Qwen | vestavěný | API klíč od poskytovatele |
| Azure OpenAI | `https://NAME.openai.azure.com/openai/v1` | Model = název nasazení |
| Ollama / LM Studio / vLLM / LocalAI | `http://host:11434/v1` atd. | Klíč není potřeba |
| Cokoli kompatibilního s OpenAI | vaše URL | Klíč volitelný |

Modely, které nečtou obrázky: zaškrtněte *„Model nečte obrázky – posílat rozpoznaný text“*. Poskytovatelé, kteří nepřijímají PDF, automaticky dostanou první stránky jako obrázky. Jazyk navrhovaných názvů a shrnutí se nastavuje v *Jazyk návrhů*.

## REST API

Vytvořte API klíč v *Profil → API klíče* (vyberte oprávnění). Všechny koncové body jsou pod `/api/v1` a přijímají `Authorization: Bearer np_…`.

```bash
TOKEN=np_your_key
BASE=http://localhost:8320/api/v1

# organizace
curl -H "Authorization: Bearer $TOKEN" $BASE/organizations

# nahrání (několik souborů najednou pomocí několika -F)
curl -H "Authorization: Bearer $TOKEN" -F file=@invoice.pdf \
     $BASE/organizations/$ORG/documents

# hledání: stejná syntaxe jako v UI
curl -H "Authorization: Bearer $TOKEN" -G $BASE/organizations/$ORG/documents \
     --data-urlencode 'searchQuery=northwind tag:invoice created:2026'

# stáhnout soubor
curl -H "Authorization: Bearer $TOKEN" -o out.pdf \
     "$BASE/organizations/$ORG/documents/$DOC/file?download=true"

# přidat štítek
curl -H "Authorization: Bearer $TOKEN" -H "Content-Type: application/json" \
     -d '{"tagId":"'$TAG'"}' $BASE/organizations/$ORG/documents/$DOC/tags

# veřejný odkaz platný 7 dní
curl -H "Authorization: Bearer $TOKEN" -H "Content-Type: application/json" \
     -d '{"expiresInDays":7}' $BASE/organizations/$ORG/documents/$DOC/share-links

# kontrola stavu (klíč není potřeba)
curl $BASE/health
```

Webhooky (*Nastavení → Webhooky*) posílají podepsaný JSON (`webhook-id`, `webhook-timestamp`, `webhook-signature`) pro `document:created`, `document:updated`, `document:deleted`, `document:restored`, `document:tag:added` a `document:tag:removed`.

Složka pro příjem `data/ingestion/<organization-id>/` importuje soubory přímo jako dokumenty. Sledované složky (*Složková doručená pošta*) je nejprve pošlou do doručené pošty ke kontrole.

## Vývoj

```bash
# backend (Rust 1.85+, Tesseract a Poppler pro OCR)
cargo run                         # http://localhost:8320
cargo test

# webové rozhraní s hot reload (dx = dioxus-cli 0.7)
rustup target add wasm32-unknown-unknown
cargo install dioxus-cli --version 0.7.10 --locked
cd ui && dx serve                 # http://localhost:8080, proxy /api na :8320

# vložit rozhraní do programu
scripts/ui.sh && cargo build --release
```

Překlady jsou v `locales/<language>.json` (německý zdrojový text → překlad). `scripts/i18n_extract.py` obnovuje `locales/strings.json` a hlásí chybějící položky podle jazyka.

Podrobnosti architektury: [ARCHITECTURE.md](../ARCHITECTURE.md).

## Licence

AGPL-3.0-or-later. Komponenty UI vycházejí z [rust-ui](https://github.com/rust-ui/ui) (MIT), prohlížeč PDF z [PDF.js](https://mozilla.github.io/pdf.js/) (Apache-2.0), písmo je [Inter](https://rsms.me/inter/) (OFL).
