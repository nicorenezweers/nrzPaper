<div align="center">

<img src="../static/icon.png" width="96" alt="nrzPaper">

# nrzPaper

**Samodzielnie hostowane zarządzanie dokumentami w Rust: skanuj, rozpoznawaj, oznaczaj i znajduj swoje papiery — zaszyfrowane w jednym pliku.**

[English](../README.md) · [Deutsch](README.de.md) · [Français](README.fr.md) · [Italiano](README.it.md) · [Español](README.es.md) · [Português](README.pt.md) · [Nederlands](README.nl.md) · [中文](README.zh.md) · [日本語](README.ja.md) · [Polski](README.pl.md) · [Русский](README.ru.md) · [العربية](README.ar.md) · [한국어](README.ko.md) · [Türkçe](README.tr.md) · [Bahasa Indonesia](README.id.md) · [हिन्दी](README.hi.md) · [Tiếng Việt](README.vi.md) · [ไทย](README.th.md) · [Українська](README.uk.md) · [Čeština](README.cs.md) · [Română](README.ro.md) · [Magyar](README.hu.md) · [Ελληνικά](README.el.md) · [Svenska](README.sv.md) · [Norsk](README.nb.md) · [Dansk](README.da.md) · [Suomi](README.fi.md) · [עברית](README.he.md) · [বাংলা](README.bn.md) · [فارسی](README.fa.md)

![Dokumenty z osią czasu](screenshots/documents.png)

</div>

nrzPaper to jeden program bez zewnętrznej bazy danych. Wszystko znajduje się w jednym pliku SQLite zaszyfrowanym przez SQLCipher, łącznie z samymi plikami. Wykonuje OCR za pomocą Tesseract, skanuje bezpośrednio ze skanerów sieciowych, pobiera załączniki e-mail i pozwala wybranej przez Ciebie AI proponować tytuły, tagi i metadane. Wbudowany asystent **Dieter** odpowiada na pytania dotyczące Twoich dokumentów i linkuje źródła. Interfejs webowy to aplikacja Rust/WebAssembly (Dioxus) w spokojnym ciemnym motywie, działa też na telefonach.

## Funkcje

| | |
|---|---|
| **Przechwytywanie** | Przeciągnij i upuść, przesyłanie, **skanery sieciowe** (eSCL/AirScan: Canon, HP, Epson, Brother …), **skrzynki e-mail** (IMAP, kilka kont), **obserwowane foldery** (np. folder skanów na NAS), REST API |
| **Skrzynka odbiorcza** | Skany, maile, importy z folderów i przesłane pliki czekają na przegląd: AI proponuje nazwę, tagi i pola, a Ty zatwierdzasz pojedynczo lub **wszystko naraz**. Filtruj według źródła (skaner, każda skrzynka, każdy folder, przesłanie). |
| **Rozpoznawanie** | Tesseract i Poppler w tle, także dla zeskanowanych PDF-ów i zdjęć; wyszukiwanie pełnotekstowe z prefiksami, frazami i filtrami (`tag:`, `-tag:`, `type:`, `created:>2026-06-01`, …) |
| **Porządkowanie** | Kolorowe tagi, **foldery tworzone z tagów** (zagnieżdżone), **reguły tagowania** („zawiera Northwind → Faktura”), własne **właściwości** (tekst, liczba, data, tak/nie, wybór, wielokrotny wybór, użytkownik, link do dokumentu) |
| **Oś czasu** | Pionowa oś czasu obok dokumentów: dziś na górze, poniżej każdy dzień z pocztą i ciche przerwy między nimi („12 dni bez dokumentów”). Filtruj według tagu, typu i tekstu. |
| **Przypomnienia** | „Przypomnij mi za 10 dni” przy dowolnym dokumencie, z notatką; zaległe przypomnienia pojawiają się na pasku bocznym i na górze osi czasu. |
| **Druk** | Drukuj dokumenty na dowolnej **sieciowej drukarce IPP** (AirPrint, IPP Everywhere, Mopria, CUPS) znalezionej przez serwer. PDF-y idą bezpośrednio lub jako obrazy stron do drukarek przyjmujących tylko JPEG. |
| **AI, dowolny dostawca** | OpenRouter (wbudowane darmowe modele), OpenAI, Anthropic, Google Gemini, Mistral, Groq, xAI, DeepSeek, Together, Fireworks, Perplexity, Cohere, NVIDIA, Hugging Face, Qwen, Azure OpenAI, **Ollama, LM Studio, vLLM/LocalAI** lub dowolny URL zgodny z OpenAI. Modele tekstowe otrzymują rozpoznany tekst zamiast obrazów. |
| **Dieter** | Asystent czatu (w prawym dolnym rogu): *„Jaki jest mój numer ubezpieczenia?”*, *„Wszystkie odcinki wypłat od Fabrikam w 2026”*. Wyszukuje, czyta pasujące fragmenty i odpowiada z linkami do dokumentów. |
| **Zespoły** | Wiele organizacji, **wiele kont na organizację** (zaproszenie, role: właściciel, administrator, członek; przekazanie własności), klucze API z uprawnieniami, webhooki (podpisy Standard Webhooks), publiczne linki do udostępniania |
| **Języki** | English, Deutsch, Français, Italiano, Español, Português, Nederlands, 中文, 日本語, Polski, Русский, العربية, 한국어, Türkçe, Bahasa Indonesia, हिन्दी, Tiếng Việt, ไทย, Українська, Čeština, Română, Magyar, Ελληνικά, Svenska, Norsk, Dansk, Suomi, עברית, বাংলা, فارسی (więcej przez `locales/<code>.json`) |
| **Domyślnie bezpieczne** | Jeden zaszyfrowany plik, hasła Argon2, CSP bez skryptów inline, ochrona CSRF, pliki tylko przez API |

## Zrzuty ekranu

| | |
|---|---|
| ![Dieter, asystent](screenshots/dieter.png) | ![Skrzynka odbiorcza z propozycjami AI](screenshots/inbox.png) |
| **Dieter** odpowiada ze źródłami | **Skrzynka odbiorcza**: przegląd i zatwierdzanie, filtr według źródła |
| ![Widok dokumentu](screenshots/document.png) | ![Dostawcy AI](screenshots/settings-ai.png) |
| **Dokument**: przeglądarka PDF, tagi, przypomnienia, druk | **AI i modele**: każdy dostawca, lokalny lub w chmurze |
| ![Przypomnienia](screenshots/reminders.png) | ![Członkowie](screenshots/members.png) |
| **Przypomnienia** | **Członkowie** i zaproszenia |

## Szybki start

Potrzebujesz Dockera. Obraz zawiera program, interfejs webowy, Tesseract (niemiecki, angielski) i Poppler.

```bash
git clone https://github.com/nicorenezweers/nrzPaper.git
cd nrzPaper
docker build -t nrzpaper .
docker run -d --name nrzpaper -p 8320:8320 -v nrzpaper-data:/data nrzpaper
```

Otwórz <http://localhost:8320>. Pierwsze utworzone konto zostaje administratorem. Potem rejestracja jest zamknięta, dopóki nie otworzysz jej w *Ustawienia → Rejestracja i użytkownicy*, gdzie możesz też tworzyć konta dla innych.

### Docker Compose

```yaml
services:
  nrzpaper:
    build: .                      # lub: image: ghcr.io/nicorenezweers/nrzpaper:1.0.0
    container_name: nrzpaper
    restart: unless-stopped
    ports:
      - "8320:8320"
    volumes:
      - nrzpaper-data:/data       # zaszyfrowana baza danych + plik klucza
      - /srv/scans:/scans         # opcjonalnie: folder dla skrzynki folderowej
    environment:
      TZ: Europe/Zurich
      # NRZPAPER_DB_KEY: "a-long-secret"     # inaczej tworzony jest /data/nrzpaper.key
      # NRZPAPER_COOKIE_SECURE: "true"       # gdy udostępniane przez HTTPS
      # NRZPAPER_AI_PROVIDER: "openai"       # lub ustaw w interfejsie webowym
      # NRZPAPER_AI_KEY: "sk-…"

volumes:
  nrzpaper-data:
```

```bash
docker compose up -d
```

> **Twórz kopie zapasowe `/data`.** Znajdują się tam `nrzpaper.db` (wszystko, zaszyfrowane) i `nrzpaper.key`. Bez klucza nie można otworzyć bazy danych. Jeśli ustawisz `NRZPAPER_DB_KEY`, przechowuj bezpiecznie tę wartość.

### Obie architektury

```bash
docker buildx build --platform linux/amd64,linux/arm64 -t <registry>/nrzpaper:1.0.0 --push .
```

Rust jest kompilowany krzyżowo, a nie emulowany, więc kompilacja arm64 jest szybka na maszynie x86 i odwrotnie.

## Pierwsze kroki

1. **Prześlij** kilka PDF-ów lub zdjęć: tekst jest rozpoznawany w tle.
2. **Tagi**: utwórz tagi, np. *Faktura* lub *Ubezpieczenie*, i dodaj opis. AI używa go do przypisywania tagów.
3. **AI** (*Ustawienia → AI i modele*): wybierz dostawcę, wklej klucz (lub użyj wbudowanych darmowych modeli OpenRouter albo lokalnego Ollama), następnie *Wczytaj modele* i *Testuj połączenie*.
4. **Skaner** (*Profil → Skaner*): *Szukaj w sieci* znajduje skanery eSCL/AirScan; następnie *Skanuj* na pasku bocznym.
5. **E-mail** (*Profil → Skrzynka e-mail*): dodaj jedną lub więcej skrzynek IMAP; PDF-y i obrazy z załączników trafiają do skrzynki odbiorczej.
6. **Skrzynka folderowa** (*Ustawienia → Skrzynka folderowa*, administratorzy): obserwuj folder serwera, np. `/scans` (najpierw zamontuj go w kontenerze).
7. **Drukarki** (*Ustawienia → Drukarki*, administratorzy): *Szukaj w sieci* lub wpisz `ipp://printer/ipp/print`; potem każdy członek może drukować ze strony dokumentu.

## Konfiguracja

Każde ustawienie można podać jako zmienną środowiskową z prefiksem `NRZPAPER_` (lub w `Rocket.toml`).

| Zmienna | Domyślnie | Znaczenie |
|---|---|---|
| `NRZPAPER_DATA_DIR` | `/data` (obraz) | Baza danych, plik klucza i foldery przyjmowania |
| `NRZPAPER_DB_KEY` | – | Klucz bazy danych; jeśli nie ustawiono, w folderze danych generowany jest `nrzpaper.key` |
| `NRZPAPER_PORT` / `NRZPAPER_ADDRESS` | `8320` / `0.0.0.0` | Gdzie nasłuchuje serwer |
| `NRZPAPER_COOKIE_SECURE` | `false` | Ustaw `true` za HTTPS |
| `NRZPAPER_REGISTRATION_OPEN` | `false` | Zezwól na samodzielną rejestrację (można też przełączyć w UI) |
| `NRZPAPER_OCR_LANGUAGES` | `deu+eng` | Języki Tesseract (obraz zawiera `deu` i `eng`) |
| `NRZPAPER_OCR_WORKERS` | `2` | Równoległe zadania OCR |
| `NRZPAPER_TRASH_RETENTION_DAYS` | `30` | Dni w koszu przed ostatecznym usunięciem |
| `NRZPAPER_INBOX_KEEP_DAYS` | `7` | Dni przechowywania niezatwierdzonych elementów skrzynki |
| `NRZPAPER_SCANNER_URL` | – | Wstępnie ustawiony skaner dla nowych użytkowników, np. `http://192.168.1.50` |
| `NRZPAPER_SCAN_SUBNETS` | `192.168.0.0/24,192.168.1.0/24` | Sieci przeszukiwane w poszukiwaniu skanerów i drukarek |
| `NRZPAPER_AI_PROVIDER` | `openrouter` | `openai`, `anthropic`, `gemini`, `mistral`, `groq`, `ollama`, `custom`, … |
| `NRZPAPER_AI_KEY` / `NRZPAPER_AI_URL` / `NRZPAPER_AI_MODEL` | – | Klucz, bazowy URL i model dla tego dostawcy |
| `NRZPAPER_OPENROUTER_KEY` | – | Klucz OpenRouter (alternatywa dla powyższego) |
| `NRZPAPER_LOG` | `info` | Filtr logów, np. `debug` |

Ustawienia z interfejsu webowego (dostawca AI, klucze, modele, rejestracja) są przechowywane w zaszyfrowanej bazie danych i mają pierwszeństwo przed środowiskiem.

### Dostawcy AI

| Dostawca | Adres | Uwagi |
|---|---|---|
| OpenRouter | wbudowany | Darmowe modele wizualne; własny klucz opcjonalny |
| OpenAI, Anthropic, Google Gemini, Mistral, Groq, xAI, DeepSeek, Together, Fireworks, Perplexity, Cohere, NVIDIA, Hugging Face, Qwen | wbudowany | Klucz API dostawcy |
| Azure OpenAI | `https://NAME.openai.azure.com/openai/v1` | Model = nazwa wdrożenia |
| Ollama / LM Studio / vLLM / LocalAI | `http://host:11434/v1` itp. | Klucz niepotrzebny |
| Cokolwiek zgodne z OpenAI | Twój URL | Klucz opcjonalny |

Modele, które nie czytają obrazów: zaznacz *„Model nie czyta obrazów – wyślij rozpoznany tekst”*. Dostawcy, którzy nie przyjmują PDF-ów, automatycznie dostają pierwsze strony jako obrazy. Język proponowanych tytułów i podsumowań ustawiasz w *Język propozycji*.

## REST API

Utwórz klucz API w *Profil → Klucze API* (wybierz uprawnienia). Wszystkie endpointy są pod `/api/v1` i przyjmują `Authorization: Bearer np_…`.

```bash
TOKEN=np_your_key
BASE=http://localhost:8320/api/v1

# organizacje
curl -H "Authorization: Bearer $TOKEN" $BASE/organizations

# przesyłanie (kilka plików naraz kilkoma -F)
curl -H "Authorization: Bearer $TOKEN" -F file=@invoice.pdf \
     $BASE/organizations/$ORG/documents

# wyszukiwanie: ta sama składnia co w UI
curl -H "Authorization: Bearer $TOKEN" -G $BASE/organizations/$ORG/documents \
     --data-urlencode 'searchQuery=northwind tag:invoice created:2026'

# pobierz plik
curl -H "Authorization: Bearer $TOKEN" -o out.pdf \
     "$BASE/organizations/$ORG/documents/$DOC/file?download=true"

# dodaj tag
curl -H "Authorization: Bearer $TOKEN" -H "Content-Type: application/json" \
     -d '{"tagId":"'$TAG'"}' $BASE/organizations/$ORG/documents/$DOC/tags

# publiczny link ważny 7 dni
curl -H "Authorization: Bearer $TOKEN" -H "Content-Type: application/json" \
     -d '{"expiresInDays":7}' $BASE/organizations/$ORG/documents/$DOC/share-links

# sprawdzenie stanu (klucz niepotrzebny)
curl $BASE/health
```

Webhooki (*Ustawienia → Webhooki*) wysyłają podpisany JSON (`webhook-id`, `webhook-timestamp`, `webhook-signature`) dla `document:created`, `document:updated`, `document:deleted`, `document:restored`, `document:tag:added` i `document:tag:removed`.

Folder przyjmowania `data/ingestion/<organization-id>/` importuje pliki bezpośrednio jako dokumenty. Obserwowane foldery (*Skrzynka folderowa*) najpierw wysyłają je do skrzynki odbiorczej do przeglądu.

## Rozwój

```bash
# backend (Rust 1.85+, Tesseract i Poppler do OCR)
cargo run                         # http://localhost:8320
cargo test

# interfejs webowy z hot reload (dx = dioxus-cli 0.7)
rustup target add wasm32-unknown-unknown
cargo install dioxus-cli --version 0.7.10 --locked
cd ui && dx serve                 # http://localhost:8080, proxy /api do :8320

# osadź interfejs w programie
scripts/ui.sh && cargo build --release
```

Tłumaczenia znajdują się w `locales/<language>.json` (niemiecki tekst źródłowy → tłumaczenie). `scripts/i18n_extract.py` odświeża `locales/strings.json` i raportuje brakujące wpisy według języka.

Szczegóły architektury: [ARCHITECTURE.md](../ARCHITECTURE.md).

## Licencja

AGPL-3.0-or-later. Komponenty UI bazują na [rust-ui](https://github.com/rust-ui/ui) (MIT), przeglądarka PDF na [PDF.js](https://mozilla.github.io/pdf.js/) (Apache-2.0), czcionka to [Inter](https://rsms.me/inter/) (OFL).
