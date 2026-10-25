<div align="center">

<img src="../static/icon.png" width="96" alt="nrzPaper">

# nrzPaper

**Самостоятельное управление документами на Rust: сканируйте, распознавайте, помечайте тегами и находите свои бумаги — всё зашифровано в одном файле.**

[English](../README.md) · [Deutsch](README.de.md) · [Français](README.fr.md) · [Italiano](README.it.md) · [Español](README.es.md) · [Português](README.pt.md) · [Nederlands](README.nl.md) · [中文](README.zh.md) · [日本語](README.ja.md) · [Polski](README.pl.md) · [Русский](README.ru.md) · [العربية](README.ar.md) · [한국어](README.ko.md) · [Türkçe](README.tr.md) · [Bahasa Indonesia](README.id.md) · [हिन्दी](README.hi.md) · [Tiếng Việt](README.vi.md) · [ไทย](README.th.md) · [Українська](README.uk.md) · [Čeština](README.cs.md) · [Română](README.ro.md) · [Magyar](README.hu.md) · [Ελληνικά](README.el.md) · [Svenska](README.sv.md) · [Norsk](README.nb.md) · [Dansk](README.da.md) · [Suomi](README.fi.md) · [עברית](README.he.md) · [বাংলা](README.bn.md) · [فارسی](README.fa.md)

![Документы с временной шкалой](screenshots/documents.png)

</div>

nrzPaper — это одна программа без внешней базы данных. Всё хранится в одном зашифрованном SQLCipher файле SQLite, включая сами файлы. Она выполняет OCR с помощью Tesseract, сканирует напрямую с сетевых сканеров, забирает вложения из электронной почты и позволяет выбранному вами ИИ предлагать названия, теги и метаданные. Встроенный помощник **Dieter** отвечает на вопросы о ваших документах и ссылается на источники. Веб-интерфейс — это приложение Rust/WebAssembly (Dioxus) в спокойной тёмной теме, которое работает и на телефонах.

## Возможности

| | |
|---|---|
| **Захват** | Перетаскивание, загрузка, **сетевые сканеры** (eSCL/AirScan: Canon, HP, Epson, Brother …), **почтовые ящики** (IMAP, несколько аккаунтов), **наблюдаемые папки** (например, папка сканирования на NAS), REST API |
| **Входящие** | Сканы, письма, импорт из папок и загрузки ждут проверки: ИИ предлагает название, теги и поля, а вы одобряете по одному или **все сразу**. Фильтр по источнику (сканер, каждый почтовый ящик, каждая папка, загрузка). |
| **Распознавание** | Tesseract и Poppler в фоне, в том числе для сканированных PDF и фотографий; полнотекстовый поиск с префиксами, фразами и фильтрами (`tag:`, `-tag:`, `type:`, `created:>2026-06-01`, …) |
| **Организация** | Цветные теги, **папки из тегов** (вложенные), **правила тегирования** («содержит Northwind → Счёт»), пользовательские **свойства** (текст, число, дата, да/нет, выбор, множественный выбор, пользователь, ссылка на документ) |
| **Временная шкала** | Вертикальная временная шкала рядом с документами: сегодня сверху, ниже каждый день с почтой и тихие промежутки между ними («12 дней без документов»). Фильтр по тегу, типу и тексту. |
| **Напоминания** | «Напомнить через 10 дней» для любого документа с заметкой; наступившие напоминания видны в боковой панели и вверху временной шкалы. |
| **Печать** | Печать на любом **сетевом IPP-принтере** (AirPrint, IPP Everywhere, Mopria, CUPS), который найдёт сервер. PDF отправляются напрямую или как изображения страниц на принтеры, принимающие только JPEG. |
| **ИИ, любой провайдер** | OpenRouter (бесплатные модели встроены), OpenAI, Anthropic, Google Gemini, Mistral, Groq, xAI, DeepSeek, Together, Fireworks, Perplexity, Cohere, NVIDIA, Hugging Face, Qwen, Azure OpenAI, **Ollama, LM Studio, vLLM/LocalAI** или любой URL, совместимый с OpenAI. Текстовые модели получают распознанный текст вместо изображений. |
| **Dieter** | Чат-помощник (внизу справа): *«Какой у меня номер страховки?»*, *«Все расчётные листки от Fabrikam за 2026 год»*. Он ищет, читает подходящие фрагменты и отвечает со ссылками на документы. |
| **Команды** | Несколько организаций, **несколько аккаунтов на организацию** (приглашение, роли: владелец, администратор, участник; передача владения), API-ключи с правами, вебхуки (подписи Standard Webhooks), публичные ссылки для общего доступа |
| **Языки** | English, Deutsch, Français, Italiano, Español, Português, Nederlands, 中文, 日本語, Polski, Русский, العربية, 한국어, Türkçe, Bahasa Indonesia, हिन्दी, Tiếng Việt, ไทย, Українська, Čeština, Română, Magyar, Ελληνικά, Svenska, Norsk, Dansk, Suomi, עברית, বাংলা, فارسی (больше через `locales/<code>.json`) |
| **Безопасность по умолчанию** | Один зашифрованный файл, пароли Argon2, CSP без встроенных скриптов, защита от CSRF, доступ к файлам только через API |

## Скриншоты

| | |
|---|---|
| ![Dieter, помощник](screenshots/dieter.png) | ![Входящие с предложениями ИИ](screenshots/inbox.png) |
| **Dieter** отвечает с источниками | **Входящие**: проверка и одобрение, фильтр по источнику |
| ![Просмотр документа](screenshots/document.png) | ![Провайдеры ИИ](screenshots/settings-ai.png) |
| **Документ**: просмотр PDF, теги, напоминания, печать | **ИИ и модели**: любой провайдер, локальный или облачный |
| ![Напоминания](screenshots/reminders.png) | ![Участники](screenshots/members.png) |
| **Напоминания** | **Участники** и приглашения |

## Быстрый старт

Нужен Docker. Образ содержит программу, веб-интерфейс, Tesseract (немецкий, английский) и Poppler.

```bash
git clone https://github.com/nicorenezweers/nrzPaper.git
cd nrzPaper
docker build -t nrzpaper .
docker run -d --name nrzpaper -p 8320:8320 -v nrzpaper-data:/data nrzpaper
```

Откройте <http://localhost:8320>. Первый созданный аккаунт становится администратором. После этого регистрация закрыта, пока вы не откроете её в *Настройки → Регистрация и пользователи*, где также можно создавать аккаунты для других.

### Docker Compose

```yaml
services:
  nrzpaper:
    build: .                      # или: image: ghcr.io/nicorenezweers/nrzpaper:1.0.0
    container_name: nrzpaper
    restart: unless-stopped
    ports:
      - "8320:8320"
    volumes:
      - nrzpaper-data:/data       # зашифрованная база данных + файл ключа
      - /srv/scans:/scans         # необязательно: папка для папочных входящих
    environment:
      TZ: Europe/Zurich
      # NRZPAPER_DB_KEY: "a-long-secret"     # иначе создаётся /data/nrzpaper.key
      # NRZPAPER_COOKIE_SECURE: "true"       # при работе через HTTPS
      # NRZPAPER_AI_PROVIDER: "openai"       # или задайте в веб-интерфейсе
      # NRZPAPER_AI_KEY: "sk-…"

volumes:
  nrzpaper-data:
```

```bash
docker compose up -d
```

> **Сделайте резервную копию `/data`.** Там находятся `nrzpaper.db` (всё, зашифровано) и `nrzpaper.key`. Без ключа базу данных открыть нельзя. Если вы задали `NRZPAPER_DB_KEY`, храните это значение в надёжном месте.

### Обе архитектуры

```bash
docker buildx build --platform linux/amd64,linux/arm64 -t <registry>/nrzpaper:1.0.0 --push .
```

Rust компилируется кросс-компилятором, а не эмулируется, поэтому сборка arm64 быстра на машине x86 и наоборот.

## Первые шаги

1. **Загрузите** несколько PDF или фотографий: текст распознаётся в фоне.
2. **Теги**: создайте теги, например *Счёт* или *Страховка*, и добавьте описание. ИИ использует его для назначения тегов.
3. **ИИ** (*Настройки → ИИ и модели*): выберите провайдера, вставьте ключ (или используйте встроенные бесплатные модели OpenRouter или локальный Ollama), затем *Загрузить модели* и *Проверить соединение*.
4. **Сканер** (*Профиль → Сканер*): *Поиск в сети* находит сканеры eSCL/AirScan; затем *Сканировать* в боковой панели.
5. **Электронная почта** (*Профиль → Почтовые входящие*): добавьте один или несколько почтовых ящиков IMAP; PDF и изображения из вложений попадают во входящие.
6. **Папочные входящие** (*Настройки → Папочные входящие*, администраторы): наблюдайте за папкой сервера, например `/scans` (сначала смонтируйте её в контейнер).
7. **Принтеры** (*Настройки → Принтеры*, администраторы): *Поиск в сети* или введите `ipp://printer/ipp/print`; после этого каждый участник сможет печатать со страницы документа.

## Конфигурация

Каждую настройку можно задать переменной окружения с префиксом `NRZPAPER_` (или в `Rocket.toml`).

| Переменная | По умолчанию | Значение |
|---|---|---|
| `NRZPAPER_DATA_DIR` | `/data` (образ) | База данных, файл ключа и папки приёма |
| `NRZPAPER_DB_KEY` | – | Ключ базы данных; если не задан, в папке данных создаётся `nrzpaper.key` |
| `NRZPAPER_PORT` / `NRZPAPER_ADDRESS` | `8320` / `0.0.0.0` | Где слушает сервер |
| `NRZPAPER_COOKIE_SECURE` | `false` | Установите `true` за HTTPS |
| `NRZPAPER_REGISTRATION_OPEN` | `false` | Разрешить самостоятельную регистрацию (можно переключить в интерфейсе) |
| `NRZPAPER_OCR_LANGUAGES` | `deu+eng` | Языки Tesseract (в образе есть `deu` и `eng`) |
| `NRZPAPER_OCR_WORKERS` | `2` | Параллельные задачи OCR |
| `NRZPAPER_TRASH_RETENTION_DAYS` | `30` | Дней в корзине до окончательного удаления |
| `NRZPAPER_INBOX_KEEP_DAYS` | `7` | Сколько дней хранятся неодобренные элементы входящих |
| `NRZPAPER_SCANNER_URL` | – | Предустановленный сканер для новых пользователей, например `http://192.168.1.50` |
| `NRZPAPER_SCAN_SUBNETS` | `192.168.0.0/24,192.168.1.0/24` | Сети для поиска сканеров и принтеров |
| `NRZPAPER_AI_PROVIDER` | `openrouter` | `openai`, `anthropic`, `gemini`, `mistral`, `groq`, `ollama`, `custom`, … |
| `NRZPAPER_AI_KEY` / `NRZPAPER_AI_URL` / `NRZPAPER_AI_MODEL` | – | Ключ, базовый URL и модель для этого провайдера |
| `NRZPAPER_OPENROUTER_KEY` | – | Ключ OpenRouter (альтернатива вышеуказанному) |
| `NRZPAPER_LOG` | `info` | Фильтр журнала, например `debug` |

Настройки, сделанные в веб-интерфейсе (провайдер ИИ, ключи, модели, регистрация), хранятся в зашифрованной базе данных и имеют приоритет над окружением.

### Провайдеры ИИ

| Провайдер | Адрес | Примечания |
|---|---|---|
| OpenRouter | встроен | Бесплатные визуальные модели; свой ключ необязателен |
| OpenAI, Anthropic, Google Gemini, Mistral, Groq, xAI, DeepSeek, Together, Fireworks, Perplexity, Cohere, NVIDIA, Hugging Face, Qwen | встроен | API-ключ провайдера |
| Azure OpenAI | `https://NAME.openai.azure.com/openai/v1` | Модель = имя развёртывания |
| Ollama / LM Studio / vLLM / LocalAI | `http://host:11434/v1` и т. п. | Ключ не нужен |
| Всё, совместимое с OpenAI | ваш URL | Ключ необязателен |

Модели, не умеющие читать изображения: установите *«Модель не читает изображения — отправлять распознанный текст»*. Провайдеры, не принимающие PDF, автоматически получают первые страницы в виде изображений. Язык предлагаемых названий и сводок задаётся в *Язык предложений*.

## REST API

Создайте API-ключ в *Профиль → API-ключи* (выберите права). Все эндпоинты находятся под `/api/v1` и принимают `Authorization: Bearer np_…`.

```bash
TOKEN=np_your_key
BASE=http://localhost:8320/api/v1

# организации
curl -H "Authorization: Bearer $TOKEN" $BASE/organizations

# загрузка (несколько файлов сразу с несколькими -F)
curl -H "Authorization: Bearer $TOKEN" -F file=@invoice.pdf \
     $BASE/organizations/$ORG/documents

# поиск: тот же синтаксис, что и в интерфейсе
curl -H "Authorization: Bearer $TOKEN" -G $BASE/organizations/$ORG/documents \
     --data-urlencode 'searchQuery=northwind tag:invoice created:2026'

# скачать файл
curl -H "Authorization: Bearer $TOKEN" -o out.pdf \
     "$BASE/organizations/$ORG/documents/$DOC/file?download=true"

# добавить тег
curl -H "Authorization: Bearer $TOKEN" -H "Content-Type: application/json" \
     -d '{"tagId":"'$TAG'"}' $BASE/organizations/$ORG/documents/$DOC/tags

# публичная ссылка, действительная 7 дней
curl -H "Authorization: Bearer $TOKEN" -H "Content-Type: application/json" \
     -d '{"expiresInDays":7}' $BASE/organizations/$ORG/documents/$DOC/share-links

# проверка состояния (ключ не нужен)
curl $BASE/health
```

Вебхуки (*Настройки → Вебхуки*) отправляют подписанный JSON (`webhook-id`, `webhook-timestamp`, `webhook-signature`) для `document:created`, `document:updated`, `document:deleted`, `document:restored`, `document:tag:added` и `document:tag:removed`.

Папка приёма `data/ingestion/<organization-id>/` импортирует файлы напрямую как документы. Наблюдаемые папки (*Папочные входящие*) сначала отправляют их во входящие на проверку.

## Разработка

```bash
# бэкенд (Rust 1.85+, Tesseract и Poppler для OCR)
cargo run                         # http://localhost:8320
cargo test

# веб-интерфейс с горячей перезагрузкой (dx = dioxus-cli 0.7)
rustup target add wasm32-unknown-unknown
cargo install dioxus-cli --version 0.7.10 --locked
cd ui && dx serve                 # http://localhost:8080, проксирует /api на :8320

# встроить интерфейс в программу
scripts/ui.sh && cargo build --release
```

Переводы находятся в `locales/<language>.json` (немецкий исходный текст → перевод). `scripts/i18n_extract.py` обновляет `locales/strings.json` и сообщает об отсутствующих записях по языкам.

Подробности архитектуры: [ARCHITECTURE.md](../ARCHITECTURE.md).

## Лицензия

AGPL-3.0-or-later. Компоненты интерфейса основаны на [rust-ui](https://github.com/rust-ui/ui) (MIT), просмотрщик PDF — на [PDF.js](https://mozilla.github.io/pdf.js/) (Apache-2.0), шрифт — [Inter](https://rsms.me/inter/) (OFL).
