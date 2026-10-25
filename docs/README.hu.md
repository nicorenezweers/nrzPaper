<div align="center">

<img src="../static/icon.png" width="96" alt="nrzPaper">

# nrzPaper

**Saját üzemeltetésű dokumentumkezelés Rustban: szkennelj, ismerj fel, címkézz és találj meg mindent — egyetlen titkosított fájlban.**

[English](../README.md) · [Deutsch](README.de.md) · [Français](README.fr.md) · [Italiano](README.it.md) · [Español](README.es.md) · [Português](README.pt.md) · [Nederlands](README.nl.md) · [中文](README.zh.md) · [日本語](README.ja.md) · [Polski](README.pl.md) · [Русский](README.ru.md) · [العربية](README.ar.md) · [한국어](README.ko.md) · [Türkçe](README.tr.md) · [Bahasa Indonesia](README.id.md) · [हिन्दी](README.hi.md) · [Tiếng Việt](README.vi.md) · [ไทย](README.th.md) · [Українська](README.uk.md) · [Čeština](README.cs.md) · [Română](README.ro.md) · [Magyar](README.hu.md) · [Ελληνικά](README.el.md) · [Svenska](README.sv.md) · [Norsk](README.nb.md) · [Dansk](README.da.md) · [Suomi](README.fi.md) · [עברית](README.he.md) · [বাংলা](README.bn.md) · [فارسی](README.fa.md)

![Dokumentumok idővonallal](screenshots/documents.png)

</div>

Az nrzPaper egyetlen program külső adatbázis nélkül. Minden egyetlen SQLCipherrel titkosított SQLite-fájlban van, a fájlokkal együtt. OCR-t végez a Tesseracttal, közvetlenül szkennel hálózati szkennerekről, letölti az e-mail-mellékleteket, és a választott AI javasolhat címeket, címkéket és metaadatokat. A beépített asszisztens, **Dieter** válaszol a dokumentumaiddal kapcsolatos kérdésekre, és hivatkozik a forrásokra. A webes felület egy nyugodt sötét témájú Rust/WebAssembly-alkalmazás (Dioxus), és telefonon is működik.

## Funkciók

| | |
|---|---|
| **Rögzítés** | Fogd és ejtsd, feltöltés, **hálózati szkennerek** (eSCL/AirScan: Canon, HP, Epson, Brother …), **e-mail-postafiókok** (IMAP, több fiók), **figyelt mappák** (pl. a NAS-on lévő szkennmappa), REST API |
| **Beérkező** | A szkennelések, e-mailek, mappaimportok és feltöltések átvizsgálásra várnak: az AI nevet, címkéket és mezőket javasol, te pedig egyenként vagy **egyszerre mindet** jóváhagyod. Szűrés forrás szerint (szkenner, minden postafiók, minden mappa, feltöltés). |
| **Felismerés** | Tesseract és Poppler a háttérben, beszkennelt PDF-ekhez és fényképekhez is; teljes szöveges keresés előtagokkal, kifejezésekkel és szűrőkkel (`tag:`, `-tag:`, `type:`, `created:>2026-06-01`, …) |
| **Rendszerezés** | Színes címkék, **címkékből épülő mappák** (beágyazott), **címkézési szabályok** („tartalmazza: Northwind → Számla”), egyéni **tulajdonságok** (szöveg, szám, dátum, igen/nem, választás, többválasztás, felhasználó, dokumentumhivatkozás) |
| **Idővonal** | Függőleges idővonal a dokumentumok mellett: ma felül, alatta minden nap e-maillel és a csendes hézagok közöttük („12 nap dokumentum nélkül”). Szűrés címke, típus és szöveg szerint. |
| **Emlékeztetők** | „Emlékeztess 10 nap múlva” bármely dokumentumon, megjegyzéssel; az esedékes emlékeztetők az oldalsávban és az idővonal tetején jelennek meg. |
| **Nyomtatás** | Nyomtass dokumentumokat bármely, a szerver által megtalált **IPP hálózati nyomtatón** (AirPrint, IPP Everywhere, Mopria, CUPS). A PDF-ek közvetlenül mennek, vagy oldalképként azokra a nyomtatókra, amelyek csak JPEG-et fogadnak. |
| **AI, bármely szolgáltató** | OpenRouter (beépített ingyenes modellek), OpenAI, Anthropic, Google Gemini, Mistral, Groq, xAI, DeepSeek, Together, Fireworks, Perplexity, Cohere, NVIDIA, Hugging Face, Qwen, Azure OpenAI, **Ollama, LM Studio, vLLM/LocalAI** vagy bármely OpenAI-kompatibilis URL. A csak szöveges modellek a képek helyett az OCR-szöveget kapják. |
| **Dieter** | Chatasszisztens (jobb alul): *„Mi a biztosítási számom?”*, *„Minden bérjegyzék a Fabrikamtól 2026-ban”*. Keres, elolvassa a megfelelő részeket, és hivatkozásokkal válaszol a dokumentumokra. |
| **Csapatok** | Több szervezet, **több fiók szervezetenként** (meghívás, szerepkörök: tulajdonos, admin, tag; tulajdonjog átadása), jogosultságokkal rendelkező API-kulcsok, webhookok (Standard Webhooks aláírások), nyilvános megosztási linkek |
| **Alapból biztonságos** | Egy titkosított fájl, Argon2-jelszavak, CSP inline szkriptek nélkül, CSRF-védelem, fájlok csak az API-n keresztül |

## Képernyőképek

| | |
|---|---|
| ![Dieter, az asszisztens](screenshots/dieter.png) | ![Beérkező AI-javaslatokkal](screenshots/inbox.png) |
| **Dieter** forrásokkal válaszol | **Beérkező**: átvizsgálás és jóváhagyás, szűrés forrás szerint |
| ![Dokumentumnézet](screenshots/document.png) | ![AI-szolgáltatók](screenshots/settings-ai.png) |
| **Dokumentum**: PDF-megjelenítő, címkék, emlékeztetők, nyomtatás | **AI és modellek**: minden szolgáltató, helyi vagy felhő |
| ![Emlékeztetők](screenshots/reminders.png) | ![Tagok](screenshots/members.png) |
| **Emlékeztetők** | **Tagok** és meghívók |

## Gyors kezdés

Dockerre van szükség. A lemezkép tartalmazza a programot, a webes felületet, a Tesseractot (német, angol) és a Popplert.

```bash
git clone https://github.com/nicorenezweers/nrzPaper.git
cd nrzPaper
docker build -t nrzpaper .
docker run -d --name nrzpaper -p 8320:8320 -v nrzpaper-data:/data nrzpaper
```

Nyisd meg a <http://localhost:8320> címet. Az első létrehozott fiók lesz az adminisztrátor. Utána a regisztráció zárva van, amíg meg nem nyitod a *Beállítások → Regisztráció és felhasználók* alatt, ahol másoknak is létrehozhatsz fiókokat.

### Docker Compose

```yaml
services:
  nrzpaper:
    build: .                      # vagy: image: ghcr.io/nicorenezweers/nrzpaper:1.0.0
    container_name: nrzpaper
    restart: unless-stopped
    ports:
      - "8320:8320"
    volumes:
      - nrzpaper-data:/data       # titkosított adatbázis + kulcsfájl
      - /srv/scans:/scans         # opcionális: mappa a mappás beérkezőhöz
    environment:
      TZ: Europe/Zurich
      # NRZPAPER_DB_KEY: "a-long-secret"     # különben létrejön a /data/nrzpaper.key
      # NRZPAPER_COOKIE_SECURE: "true"       # HTTPS-en keresztüli kiszolgáláskor
      # NRZPAPER_AI_PROVIDER: "openai"       # vagy állítsd be a webes felületen
      # NRZPAPER_AI_KEY: "sk-…"

volumes:
  nrzpaper-data:
```

```bash
docker compose up -d
```

> **Készíts biztonsági másolatot a `/data` mappáról.** Itt van a `nrzpaper.db` (minden, titkosítva) és a `nrzpaper.key`. Kulcs nélkül az adatbázis nem nyitható meg. Ha beállítod a `NRZPAPER_DB_KEY` értéket, azt tartsd biztonságban.

### Mindkét architektúra

```bash
docker buildx build --platform linux/amd64,linux/arm64 -t <registry>/nrzpaper:1.0.0 --push .
```

A Rust keresztfordítással készül, nem emulációval, így az arm64 build gyors x86-os gépen és fordítva.

## Első lépések

1. **Tölts fel** néhány PDF-et vagy fényképet: a szöveg a háttérben felismerésre kerül.
2. **Címkék**: hozz létre címkéket, például *Számla* vagy *Biztosítás*, és adj hozzá leírást. Az AI ezt használja a címkék hozzárendeléséhez.
3. **AI** (*Beállítások → AI és modellek*): válassz szolgáltatót, illeszd be a kulcsot (vagy használd a beépített ingyenes OpenRouter-modelleket, vagy egy helyi Ollamát), majd *Modellek betöltése* és *Kapcsolat tesztelése*.
4. **Szkenner** (*Profil → Szkenner*): a *Hálózat keresése* megtalálja az eSCL/AirScan szkennereket; majd *Szkennelés* az oldalsávban.
5. **E-mail** (*Profil → E-mail beérkező*): adj hozzá egy vagy több IMAP-postafiókot; a mellékletek PDF-jei és képei a beérkezőbe kerülnek.
6. **Mappás beérkező** (*Beállítások → Mappás beérkező*, adminok): figyelj egy szervermappát, például `/scans` (előbb csatold a konténerbe).
7. **Nyomtatók** (*Beállítások → Nyomtatók*, adminok): *Hálózat keresése* vagy írd be: `ipp://printer/ipp/print`; utána minden tag nyomtathat a dokumentum oldaláról.

## Konfiguráció

Minden beállítás megadható `NRZPAPER_` előtagú környezeti változóként (vagy a `Rocket.toml`-ban).

| Változó | Alapértelmezett | Jelentés |
|---|---|---|
| `NRZPAPER_DATA_DIR` | `/data` (lemezkép) | Adatbázis, kulcsfájl és beviteli mappák |
| `NRZPAPER_DB_KEY` | – | Adatbáziskulcs; ha nincs beállítva, az adatmappában létrejön a `nrzpaper.key` |
| `NRZPAPER_PORT` / `NRZPAPER_ADDRESS` | `8320` / `0.0.0.0` | Ahol a szerver figyel |
| `NRZPAPER_COOKIE_SECURE` | `false` | HTTPS mögött állítsd `true`-ra |
| `NRZPAPER_REGISTRATION_OPEN` | `false` | Önregisztráció engedélyezése (a felületen is átkapcsolható) |
| `NRZPAPER_OCR_LANGUAGES` | `deu+eng` | Tesseract-nyelvek (a lemezkép tartalmazza a `deu` és `eng` nyelvet) |
| `NRZPAPER_OCR_WORKERS` | `2` | Párhuzamos OCR-feladatok |
| `NRZPAPER_TRASH_RETENTION_DAYS` | `30` | Napok a kukában a végleges törlésig |
| `NRZPAPER_INBOX_KEEP_DAYS` | `7` | Hány napig maradnak a jóvá nem hagyott beérkező elemek |
| `NRZPAPER_SCANNER_URL` | – | Előre beállított szkenner új felhasználóknak, pl. `http://192.168.1.50` |
| `NRZPAPER_SCAN_SUBNETS` | `192.168.0.0/24,192.168.1.0/24` | A szkennerek és nyomtatók kereséséhez használt hálózatok |
| `NRZPAPER_AI_PROVIDER` | `openrouter` | `openai`, `anthropic`, `gemini`, `mistral`, `groq`, `ollama`, `custom`, … |
| `NRZPAPER_AI_KEY` / `NRZPAPER_AI_URL` / `NRZPAPER_AI_MODEL` | – | Kulcs, alap URL és modell az adott szolgáltatóhoz |
| `NRZPAPER_OPENROUTER_KEY` | – | OpenRouter-kulcs (a fentiek alternatívája) |
| `NRZPAPER_LOG` | `info` | Naplószűrő, pl. `debug` |

A webes felületen beállított értékek (AI-szolgáltató, kulcsok, modellek, regisztráció) a titkosított adatbázisban tárolódnak, és elsőbbséget élveznek a környezettel szemben.

### AI-szolgáltatók

| Szolgáltató | Cím | Megjegyzések |
|---|---|---|
| OpenRouter | beépített | Ingyenes látásmodellek; saját kulcs opcionális |
| OpenAI, Anthropic, Google Gemini, Mistral, Groq, xAI, DeepSeek, Together, Fireworks, Perplexity, Cohere, NVIDIA, Hugging Face, Qwen | beépített | API-kulcs a szolgáltatótól |
| Azure OpenAI | `https://NAME.openai.azure.com/openai/v1` | Modell = üzemelő példány neve |
| Ollama / LM Studio / vLLM / LocalAI | `http://host:11434/v1` stb. | Nem kell kulcs |
| Bármi OpenAI-kompatibilis | a saját URL-ed | Kulcs opcionális |

A képeket nem olvasó modellek: jelöld be *„A modell nem olvas képeket – a felismert szöveg küldése”*. A PDF-eket nem fogadó szolgáltatók automatikusan az első oldalakat kapják képként. A javasolt címek és összefoglalók nyelvét a *Javaslatok nyelve* alatt állítod be.

## REST API

Hozz létre API-kulcsot a *Profil → API-kulcsok* alatt (válaszd ki a jogosultságokat). Minden végpont a `/api/v1` alatt van, és `Authorization: Bearer np_…` fejlécet fogad.

```bash
TOKEN=np_your_key
BASE=http://localhost:8320/api/v1

# szervezetek
curl -H "Authorization: Bearer $TOKEN" $BASE/organizations

# feltöltés (több fájl egyszerre több -F kapcsolóval)
curl -H "Authorization: Bearer $TOKEN" -F file=@invoice.pdf \
     $BASE/organizations/$ORG/documents

# keresés: ugyanaz a szintaxis, mint a felületen
curl -H "Authorization: Bearer $TOKEN" -G $BASE/organizations/$ORG/documents \
     --data-urlencode 'searchQuery=northwind tag:invoice created:2026'

# fájl letöltése
curl -H "Authorization: Bearer $TOKEN" -o out.pdf \
     "$BASE/organizations/$ORG/documents/$DOC/file?download=true"

# címke hozzáadása
curl -H "Authorization: Bearer $TOKEN" -H "Content-Type: application/json" \
     -d '{"tagId":"'$TAG'"}' $BASE/organizations/$ORG/documents/$DOC/tags

# 7 napig érvényes nyilvános megosztási link
curl -H "Authorization: Bearer $TOKEN" -H "Content-Type: application/json" \
     -d '{"expiresInDays":7}' $BASE/organizations/$ORG/documents/$DOC/share-links

# állapotellenőrzés (kulcs nélkül)
curl $BASE/health
```

A webhookok (*Beállítások → Webhookok*) aláírt JSON-t (`webhook-id`, `webhook-timestamp`, `webhook-signature`) küldenek a `document:created`, `document:updated`, `document:deleted`, `document:restored`, `document:tag:added` és `document:tag:removed` eseményekhez.

A beviteli mappa, `data/ingestion/<organization-id>/`, közvetlenül dokumentumként importálja a fájlokat. A figyelt mappák (*Mappás beérkező*) először átvizsgálásra a beérkezőbe küldik őket.

## Fejlesztés

```bash
# háttérrendszer (Rust 1.85+, Tesseract és Poppler az OCR-hez)
cargo run                         # http://localhost:8320
cargo test

# webes felület hot reloaddal (dx = dioxus-cli 0.7)
rustup target add wasm32-unknown-unknown
cargo install dioxus-cli --version 0.7.10 --locked
cd ui && dx serve                 # http://localhost:8080, a /api-t a :8320-ra továbbítja

# a felület beágyazása a programba
scripts/ui.sh && cargo build --release
```

A fordítások a `locales/<language>.json` fájlokban vannak (német forrászöveg → fordítás). A `scripts/i18n_extract.py` frissíti a `locales/strings.json` fájlt, és jelenti a nyelvenként hiányzó bejegyzéseket.

Architektúra részletei: [ARCHITECTURE.md](../ARCHITECTURE.md).

## Licenc

AGPL-3.0-or-later. A felület komponensei a [rust-ui](https://github.com/rust-ui/ui) (MIT), a PDF-megjelenítő a [PDF.js](https://mozilla.github.io/pdf.js/) (Apache-2.0) alapján készültek, a betűtípus [Inter](https://rsms.me/inter/) (OFL).
