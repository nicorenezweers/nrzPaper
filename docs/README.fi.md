<div align="center">

<img src="../static/icon.png" width="96" alt="nrzPaper">

# nrzPaper

**Itsenäisesti ylläpidettävä asiakirjahallinta Rustilla: skannaa, tunnista, merkitse ja löydä paperisi — salattuna yhteen tiedostoon.**

[English](../README.md) · [Deutsch](README.de.md) · [Français](README.fr.md) · [Italiano](README.it.md) · [Español](README.es.md) · [Português](README.pt.md) · [Nederlands](README.nl.md) · [中文](README.zh.md) · [日本語](README.ja.md) · [Polski](README.pl.md) · [Русский](README.ru.md) · [العربية](README.ar.md) · [한국어](README.ko.md) · [Türkçe](README.tr.md) · [Bahasa Indonesia](README.id.md) · [हिन्दी](README.hi.md) · [Tiếng Việt](README.vi.md) · [ไทย](README.th.md) · [Українська](README.uk.md) · [Čeština](README.cs.md) · [Română](README.ro.md) · [Magyar](README.hu.md) · [Ελληνικά](README.el.md) · [Svenska](README.sv.md) · [Norsk](README.nb.md) · [Dansk](README.da.md) · [Suomi](README.fi.md) · [עברית](README.he.md) · [বাংলা](README.bn.md) · [فارسی](README.fa.md)

![Asiakirjat aikajanalla](screenshots/documents.png)

</div>

nrzPaper on yksi ohjelma ilman ulkoista tietokantaa. Kaikki on yhdessä SQLCipher-salattuun SQLite-tiedostoon, myös itse tiedostot. Se tekee OCR:n Tesseractilla, skannaa suoraan verkkoskannereista, hakee sähköpostiliitteet ja antaa valitsemasi tekoälyn ehdottaa otsikoita, tunnisteita ja metatietoja. Sisäänrakennettu avustaja **Dieter** vastaa asiakirjojasi koskeviin kysymyksiin ja linkittää lähteet. Verkkokäyttöliittymä on rauhallisen tumman teeman Rust/WebAssembly-sovellus (Dioxus), ja se toimii myös puhelimissa.

## Ominaisuudet

| | |
|---|---|
| **Kerääminen** | Vedä ja pudota, lataus, **verkkoskannerit** (eSCL/AirScan: Canon, HP, Epson, Brother …), **sähköpostilaatikot** (IMAP, useita tilejä), **seuratut kansiot** (esim. NAS:n skannauskansio), REST API |
| **Saapuneet** | Skannaukset, sähköpostit, kansiotuonnit ja lataukset odottavat tarkistusta: tekoäly ehdottaa nimen, tunnisteet ja kentät, ja hyväksyt yhden kerrallaan tai **kaikki kerralla**. Suodata lähteen mukaan (skanneri, jokainen postilaatikko, jokainen kansio, lataus). |
| **Tunnistus** | Tesseract ja Poppler taustalla, myös skannatuille PDF:ille ja valokuville; kokotekstihaku etuliitteillä, lauseilla ja suodattimilla (`tag:`, `-tag:`, `type:`, `created:>2026-06-01`, …) |
| **Järjestäminen** | Värikkäät tunnisteet, **tunnisteista rakennetut kansiot** (sisäkkäiset), **merkintäsäännöt** (”sisältää Northwind → Lasku”), mukautetut **ominaisuudet** (teksti, luku, päiväys, kyllä/ei, valinta, monivalinta, käyttäjä, asiakirjalinkki) |
| **Aikajana** | Pysty aikajana asiakirjojen vieressä: tänään ylimpänä, alla jokainen päivä sähköposteineen ja hiljaiset aukot niiden välissä (”12 päivää ilman asiakirjoja”). Suodata tunnisteen, tyypin ja tekstin mukaan. |
| **Muistutukset** | ”Muistuta 10 päivän kuluttua” missä tahansa asiakirjassa, muistiinpanolla; erääntyneet muistutukset näkyvät sivupalkissa ja aikajanan yläosassa. |
| **Tulostus** | Tulosta asiakirjoja missä tahansa **IPP-verkkotulostimessa** (AirPrint, IPP Everywhere, Mopria, CUPS), jonka palvelin löytää. PDF:t menevät suoraan tai sivukuvina tulostimille, jotka hyväksyvät vain JPEG:n. |
| **Tekoäly, mikä tahansa tarjoaja** | OpenRouter (ilmaiset mallit sisäänrakennettuina), OpenAI, Anthropic, Google Gemini, Mistral, Groq, xAI, DeepSeek, Together, Fireworks, Perplexity, Cohere, NVIDIA, Hugging Face, Qwen, Azure OpenAI, **Ollama, LM Studio, vLLM/LocalAI** tai mikä tahansa OpenAI-yhteensopiva URL. Vain tekstimallit saavat OCR-tekstin kuvien sijaan. |
| **Dieter** | Chat-avustaja (oikeassa alakulmassa): *”Mikä on vakuutusnumeroni?”*, *”Kaikki Fabrikamin palkkalaskelmat vuonna 2026”*. Hän etsii, lukee osuvat kohdat ja vastaa linkeillä asiakirjoihin. |
| **Tiimit** | Useita organisaatioita, **useita tilejä organisaatiota kohden** (kutsu, roolit: omistaja, ylläpitäjä, jäsen; omistajuuden siirto), käyttöoikeuksin varustetut API-avaimet, webhookit (Standard Webhooks -allekirjoitukset), julkiset jakolinkit |
| **Oletuksena turvallinen** | Yksi salattu tiedosto, Argon2-salasanat, CSP ilman inline-skriptejä, CSRF-suojaus, tiedostot vain API:n kautta |

## Kuvakaappaukset

| | |
|---|---|
| ![Dieter, avustaja](screenshots/dieter.png) | ![Saapuneet tekoälyehdotuksin](screenshots/inbox.png) |
| **Dieter** vastaa lähteineen | **Saapuneet**: tarkista ja hyväksy, suodata lähteen mukaan |
| ![Asiakirjanäkymä](screenshots/document.png) | ![Tekoälytarjoajat](screenshots/settings-ai.png) |
| **Asiakirja**: PDF-katselin, tunnisteet, muistutukset, tulostus | **Tekoäly ja mallit**: jokainen tarjoaja, paikallinen tai pilvi |
| ![Muistutukset](screenshots/reminders.png) | ![Jäsenet](screenshots/members.png) |
| **Muistutukset** | **Jäsenet** ja kutsut |

## Pika-aloitus

Tarvitset Dockerin. Levykuva sisältää ohjelman, verkkokäyttöliittymän, Tesseractin (saksa, englanti) ja Popplerin.

```bash
git clone https://github.com/nicorenezweers/nrzPaper.git
cd nrzPaper
docker build -t nrzpaper .
docker run -d --name nrzpaper -p 8320:8320 -v nrzpaper-data:/data nrzpaper
```

Avaa <http://localhost:8320>. Ensimmäisestä luomastasi tilistä tulee ylläpitäjä. Sen jälkeen rekisteröinti on suljettu, kunnes avaat sen kohdassa *Asetukset → Rekisteröinti ja käyttäjät*, jossa voit myös luoda tilejä muille.

### Docker Compose

```yaml
services:
  nrzpaper:
    build: .                      # tai: image: ghcr.io/nicorenezweers/nrzpaper:1.0.0
    container_name: nrzpaper
    restart: unless-stopped
    ports:
      - "8320:8320"
    volumes:
      - nrzpaper-data:/data       # salattu tietokanta + avaintiedosto
      - /srv/scans:/scans         # valinnainen: kansio kansiosaapuneille
    environment:
      TZ: Europe/Zurich
      # NRZPAPER_DB_KEY: "a-long-secret"     # muuten /data/nrzpaper.key luodaan
      # NRZPAPER_COOKIE_SECURE: "true"       # kun tarjoillaan HTTPS:n kautta
      # NRZPAPER_AI_PROVIDER: "openai"       # tai aseta verkkokäyttöliittymässä
      # NRZPAPER_AI_KEY: "sk-…"

volumes:
  nrzpaper-data:
```

```bash
docker compose up -d
```

> **Varmuuskopioi `/data`.** Siellä on `nrzpaper.db` (kaikki, salattuna) ja `nrzpaper.key`. Ilman avainta tietokantaa ei voi avata. Jos asetat `NRZPAPER_DB_KEY`:n, säilytä se arvo turvassa sen sijaan.

### Molemmat arkkitehtuurit

```bash
docker buildx build --platform linux/amd64,linux/arm64 -t <registry>/nrzpaper:1.0.0 --push .
```

Rust ristikäännetään, ei emuloida, joten arm64-koonti on nopea x86-koneella ja päinvastoin.

## Ensimmäiset vaiheet

1. **Lataa** muutama PDF tai valokuva: teksti tunnistetaan taustalla.
2. **Tunnisteet**: luo tunnisteita, kuten *Lasku* tai *Vakuutus*, ja lisää kuvaus. Tekoäly käyttää sitä tunnisteiden määrittämiseen.
3. **Tekoäly** (*Asetukset → Tekoäly ja mallit*): valitse tarjoaja, liitä avain (tai käytä sisäänrakennettuja ilmaisia OpenRouter-malleja tai paikallista Ollamaa), sitten *Lataa mallit* ja *Testaa yhteys*.
4. **Skanneri** (*Profiili → Skanneri*): *Etsi verkosta* löytää eSCL/AirScan-skannerit; sitten *Skannaa* sivupalkista.
5. **Sähköposti** (*Profiili → Sähköpostisaapuneet*): lisää yksi tai useampi IMAP-postilaatikko; liitteiden PDF:t ja kuvat päätyvät saapuneisiin.
6. **Kansiosaapuneet** (*Asetukset → Kansiosaapuneet*, ylläpitäjät): seuraa palvelinkansiota, kuten `/scans` (liitä se ensin konttiin).
7. **Tulostimet** (*Asetukset → Tulostimet*, ylläpitäjät): *Etsi verkosta* tai syötä `ipp://printer/ipp/print`; sen jälkeen jokainen jäsen voi tulostaa asiakirjasivulta.

## Kokoonpano

Jokainen asetus voidaan antaa ympäristömuuttujana etuliitteellä `NRZPAPER_` (tai `Rocket.toml`-tiedostossa).

| Muuttuja | Oletus | Merkitys |
|---|---|---|
| `NRZPAPER_DATA_DIR` | `/data` (levykuva) | Tietokanta, avaintiedosto ja tuontikansiot |
| `NRZPAPER_DB_KEY` | – | Tietokannan avain; jos sitä ei aseteta, `nrzpaper.key` luodaan datakansioon |
| `NRZPAPER_PORT` / `NRZPAPER_ADDRESS` | `8320` / `0.0.0.0` | Missä palvelin kuuntelee |
| `NRZPAPER_COOKIE_SECURE` | `false` | Aseta `true`, kun HTTPS:n takana |
| `NRZPAPER_REGISTRATION_OPEN` | `false` | Salli itse rekisteröityminen (voi vaihtaa myös käyttöliittymässä) |
| `NRZPAPER_OCR_LANGUAGES` | `deu+eng` | Tesseract-kielet (levykuva sisältää `deu` ja `eng`) |
| `NRZPAPER_OCR_WORKERS` | `2` | Rinnakkaiset OCR-työt |
| `NRZPAPER_TRASH_RETENTION_DAYS` | `30` | Päivät roskakorissa ennen lopullista poistoa |
| `NRZPAPER_INBOX_KEEP_DAYS` | `7` | Kuinka monta päivää hyväksymättömät saapuneet säilytetään |
| `NRZPAPER_SCANNER_URL` | – | Uusille käyttäjille esiasetettu skanneri, esim. `http://192.168.1.50` |
| `NRZPAPER_SCAN_SUBNETS` | `192.168.0.0/24,192.168.1.0/24` | Verkot, joista skannereita ja tulostimia etsitään |
| `NRZPAPER_AI_PROVIDER` | `openrouter` | `openai`, `anthropic`, `gemini`, `mistral`, `groq`, `ollama`, `custom`, … |
| `NRZPAPER_AI_KEY` / `NRZPAPER_AI_URL` / `NRZPAPER_AI_MODEL` | – | Kyseisen tarjoajan avain, perus-URL ja malli |
| `NRZPAPER_OPENROUTER_KEY` | – | OpenRouter-avain (vaihtoehto yllä oleville) |
| `NRZPAPER_LOG` | `info` | Lokisuodatin, esim. `debug` |

Verkkokäyttöliittymässä tehdyt asetukset (tekoälytarjoaja, avaimet, mallit, rekisteröinti) tallennetaan salattuun tietokantaan, ja ne ovat etusijalla ympäristöön nähden.

### Tekoälytarjoajat

| Tarjoaja | Osoite | Huomautukset |
|---|---|---|
| OpenRouter | sisäänrakennettu | Ilmaiset näkömallit; oma avain valinnainen |
| OpenAI, Anthropic, Google Gemini, Mistral, Groq, xAI, DeepSeek, Together, Fireworks, Perplexity, Cohere, NVIDIA, Hugging Face, Qwen | sisäänrakennettu | API-avain tarjoajalta |
| Azure OpenAI | `https://NAME.openai.azure.com/openai/v1` | Malli = käyttöönoton nimi |
| Ollama / LM Studio / vLLM / LocalAI | `http://host:11434/v1` jne. | Avainta ei tarvita |
| Mikä tahansa OpenAI-yhteensopiva | oma URL | Avain valinnainen |

Mallit, jotka eivät lue kuvia: valitse *”Malli ei lue kuvia – lähetä tunnistettu teksti”*. Tarjoajat, jotka eivät hyväksy PDF:iä, saavat automaattisesti ensimmäiset sivut kuvina. Ehdotettujen otsikoiden ja tiivistelmien kieli asetetaan kohdassa *Ehdotusten kieli*.

## REST API

Luo API-avain kohdassa *Profiili → API-avaimet* (valitse käyttöoikeudet). Kaikki päätepisteet ovat polun `/api/v1` alla ja hyväksyvät `Authorization: Bearer np_…`.

```bash
TOKEN=np_your_key
BASE=http://localhost:8320/api/v1

# organisaatiot
curl -H "Authorization: Bearer $TOKEN" $BASE/organizations

# lataus (useita tiedostoja kerralla usealla -F:llä)
curl -H "Authorization: Bearer $TOKEN" -F file=@invoice.pdf \
     $BASE/organizations/$ORG/documents

# haku: sama syntaksi kuin käyttöliittymässä
curl -H "Authorization: Bearer $TOKEN" -G $BASE/organizations/$ORG/documents \
     --data-urlencode 'searchQuery=northwind tag:invoice created:2026'

# lataa tiedosto
curl -H "Authorization: Bearer $TOKEN" -o out.pdf \
     "$BASE/organizations/$ORG/documents/$DOC/file?download=true"

# lisää tunniste
curl -H "Authorization: Bearer $TOKEN" -H "Content-Type: application/json" \
     -d '{"tagId":"'$TAG'"}' $BASE/organizations/$ORG/documents/$DOC/tags

# julkinen jakolinkki voimassa 7 päivää
curl -H "Authorization: Bearer $TOKEN" -H "Content-Type: application/json" \
     -d '{"expiresInDays":7}' $BASE/organizations/$ORG/documents/$DOC/share-links

# kuntotarkistus (avainta ei tarvita)
curl $BASE/health
```

Webhookit (*Asetukset → Webhookit*) lähettävät allekirjoitettua JSONia (`webhook-id`, `webhook-timestamp`, `webhook-signature`) tapahtumille `document:created`, `document:updated`, `document:deleted`, `document:restored`, `document:tag:added` ja `document:tag:removed`.

Tuontikansio `data/ingestion/<organization-id>/` tuo tiedostot suoraan asiakirjoiksi. Seuratut kansiot (*Kansiosaapuneet*) lähettävät ne ensin saapuneisiin tarkistettaviksi.

## Kehitys

```bash
# taustajärjestelmä (Rust 1.85+, Tesseract ja Poppler OCR:ää varten)
cargo run                         # http://localhost:8320
cargo test

# verkkokäyttöliittymä hot reloadilla (dx = dioxus-cli 0.7)
rustup target add wasm32-unknown-unknown
cargo install dioxus-cli --version 0.7.10 --locked
cd ui && dx serve                 # http://localhost:8080, välittää /api:n :8320:lle

# upota käyttöliittymä ohjelmaan
scripts/ui.sh && cargo build --release
```

Käännökset ovat kansiossa `locales/<language>.json` (saksankielinen lähdeteksti → käännös). `scripts/i18n_extract.py` päivittää `locales/strings.json`-tiedoston ja raportoi puuttuvat merkinnät kielittäin.

Arkkitehtuurin yksityiskohdat: [ARCHITECTURE.md](../ARCHITECTURE.md).

## Lisenssi

AGPL-3.0-or-later. Käyttöliittymäkomponentit perustuvat [rust-ui](https://github.com/rust-ui/ui) (MIT), PDF-katselin [PDF.js](https://mozilla.github.io/pdf.js/) (Apache-2.0), fontti on [Inter](https://rsms.me/inter/) (OFL).
