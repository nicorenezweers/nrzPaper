<div align="center">

<img src="../static/icon.png" width="96" alt="nrzPaper">

# nrzPaper

**Самогосподарське керування документами на Rust: скануйте, розпізнавайте, позначайте тегами й знаходьте свої папери — усе зашифровано в одному файлі.**

[English](../README.md) · [Deutsch](README.de.md) · [Français](README.fr.md) · [Italiano](README.it.md) · [Español](README.es.md) · [Português](README.pt.md) · [Nederlands](README.nl.md) · [中文](README.zh.md) · [日本語](README.ja.md) · [Polski](README.pl.md) · [Русский](README.ru.md) · [العربية](README.ar.md) · [한국어](README.ko.md) · [Türkçe](README.tr.md) · [Bahasa Indonesia](README.id.md) · [हिन्दी](README.hi.md) · [Tiếng Việt](README.vi.md) · [ไทย](README.th.md) · [Українська](README.uk.md) · [Čeština](README.cs.md) · [Română](README.ro.md) · [Magyar](README.hu.md) · [Ελληνικά](README.el.md) · [Svenska](README.sv.md) · [Norsk](README.nb.md) · [Dansk](README.da.md) · [Suomi](README.fi.md) · [עברית](README.he.md) · [বাংলা](README.bn.md) · [فارسی](README.fa.md)

![Документи з часовою шкалою](screenshots/documents.png)

</div>

nrzPaper — це одна програма без зовнішньої бази даних. Усе зберігається в одному файлі SQLite, зашифрованому SQLCipher, включно із самими файлами. Вона виконує OCR за допомогою Tesseract, сканує безпосередньо з мережевих сканерів, забирає вкладення електронної пошти й дозволяє обраному вами ШІ пропонувати назви, теги та метадані. Вбудований помічник **Dieter** відповідає на запитання про ваші документи й посилається на джерела. Веб-інтерфейс — це застосунок Rust/WebAssembly (Dioxus) у спокійній темній темі, який працює й на телефонах.

## Можливості

| | |
|---|---|
| **Захоплення** | Перетягування, завантаження, **мережеві сканери** (eSCL/AirScan: Canon, HP, Epson, Brother …), **поштові скриньки** (IMAP, кілька облікових записів), **наглядові теки** (наприклад, тека сканування на NAS), REST API |
| **Вхідні** | Скани, листи, імпорт із тек і завантаження чекають на перевірку: ШІ пропонує назву, теги й поля, а ви схвалюєте по одному або **всі одразу**. Фільтр за джерелом (сканер, кожна поштова скринька, кожна тека, завантаження). |
| **Розпізнавання** | Tesseract і Poppler у фоні, зокрема для сканованих PDF і фотографій; повнотекстовий пошук із префіксами, фразами й фільтрами (`tag:`, `-tag:`, `type:`, `created:>2026-06-01`, …) |
| **Організація** | Кольорові теги, **теки з тегів** (вкладені), **правила тегування** («містить Northwind → Рахунок»), власні **властивості** (текст, число, дата, так/ні, вибір, множинний вибір, користувач, посилання на документ) |
| **Часова шкала** | Вертикальна часова шкала поруч із документами: сьогодні вгорі, нижче кожен день із поштою й тихі проміжки між ними («12 днів без документів»). Фільтр за тегом, типом і текстом. |
| **Нагадування** | «Нагадати через 10 днів» для будь-якого документа з нотаткою; насталі нагадування видно в бічній панелі й угорі часової шкали. |
| **Друк** | Друк на будь-якому **мережевому IPP-принтері** (AirPrint, IPP Everywhere, Mopria, CUPS), який знайде сервер. PDF надсилаються напряму або як зображення сторінок на принтери, що приймають лише JPEG. |
| **ШІ, будь-який постачальник** | OpenRouter (вбудовані безкоштовні моделі), OpenAI, Anthropic, Google Gemini, Mistral, Groq, xAI, DeepSeek, Together, Fireworks, Perplexity, Cohere, NVIDIA, Hugging Face, Qwen, Azure OpenAI, **Ollama, LM Studio, vLLM/LocalAI** або будь-яка сумісна з OpenAI URL-адреса. Текстові моделі отримують розпізнаний текст замість зображень. |
| **Dieter** | Чат-помічник (унизу праворуч): *«Який у мене номер страхування?»*, *«Усі розрахункові листки від Fabrikam за 2026 рік»*. Він шукає, читає відповідні фрагменти й відповідає з посиланнями на документи. |
| **Команди** | Кілька організацій, **кілька облікових записів на організацію** (запрошення, ролі: власник, адміністратор, учасник; передавання володіння), API-ключі з правами, вебхуки (підписи Standard Webhooks), публічні посилання для спільного доступу |
| **Мови** | English, Deutsch, Français, Italiano, Español, Português, Nederlands (інші мови незабаром) |
| **Безпечно за замовчуванням** | Один зашифрований файл, паролі Argon2, CSP без вбудованих скриптів, захист від CSRF, файли лише через API |

## Скриншоти

| | |
|---|---|
| ![Dieter, помічник](screenshots/dieter.png) | ![Вхідні з пропозиціями ШІ](screenshots/inbox.png) |
| **Dieter** відповідає з джерелами | **Вхідні**: перевірка й схвалення, фільтр за джерелом |
| ![Перегляд документа](screenshots/document.png) | ![Постачальники ШІ](screenshots/settings-ai.png) |
| **Документ**: переглядач PDF, теги, нагадування, друк | **ШІ та моделі**: будь-який постачальник, локальний чи хмарний |
| ![Нагадування](screenshots/reminders.png) | ![Учасники](screenshots/members.png) |
| **Нагадування** | **Учасники** й запрошення |

## Швидкий старт

Потрібен Docker. Образ містить програму, веб-інтерфейс, Tesseract (німецька, англійська) і Poppler.

```bash
git clone https://github.com/nicorenezweers/nrzPaper.git
cd nrzPaper
docker build -t nrzpaper .
docker run -d --name nrzpaper -p 8320:8320 -v nrzpaper-data:/data nrzpaper
```

Відкрийте <http://localhost:8320>. Перший створений обліковий запис стає адміністратором. Після цього реєстрація закрита, доки ви не відкриєте її в *Налаштування → Реєстрація та користувачі*, де також можна створювати облікові записи для інших.

### Docker Compose

```yaml
services:
  nrzpaper:
    build: .                      # або: image: ghcr.io/nicorenezweers/nrzpaper:1.0.0
    container_name: nrzpaper
    restart: unless-stopped
    ports:
      - "8320:8320"
    volumes:
      - nrzpaper-data:/data       # зашифрована база даних + файл ключа
      - /srv/scans:/scans         # необов'язково: тека для текових вхідних
    environment:
      TZ: Europe/Zurich
      # NRZPAPER_DB_KEY: "a-long-secret"     # інакше створюється /data/nrzpaper.key
      # NRZPAPER_COOKIE_SECURE: "true"       # під час роботи через HTTPS
      # NRZPAPER_AI_PROVIDER: "openai"       # або задайте у веб-інтерфейсі
      # NRZPAPER_AI_KEY: "sk-…"

volumes:
  nrzpaper-data:
```

```bash
docker compose up -d
```

> **Створюйте резервні копії `/data`.** Там лежать `nrzpaper.db` (усе, зашифровано) і `nrzpaper.key`. Без ключа базу даних не відкрити. Якщо ви задали `NRZPAPER_DB_KEY`, надійно зберігайте це значення.

### Обидві архітектури

```bash
docker buildx build --platform linux/amd64,linux/arm64 -t <registry>/nrzpaper:1.0.0 --push .
```

Rust компілюється крос-компілятором, а не емулюється, тому збірка arm64 швидка на машині x86 і навпаки.

## Перші кроки

1. **Завантажте** кілька PDF або фотографій: текст розпізнається у фоні.
2. **Теги**: створіть теги, наприклад *Рахунок* або *Страхування*, і додайте опис. ШІ використовує його для призначення тегів.
3. **ШІ** (*Налаштування → ШІ та моделі*): виберіть постачальника, вставте ключ (або використайте вбудовані безкоштовні моделі OpenRouter чи локальний Ollama), потім *Завантажити моделі* й *Перевірити з'єднання*.
4. **Сканер** (*Профіль → Сканер*): *Пошук у мережі* знаходить сканери eSCL/AirScan; потім *Сканувати* в бічній панелі.
5. **Електронна пошта** (*Профіль → Поштові вхідні*): додайте одну або кілька скриньок IMAP; PDF та зображення з вкладень потрапляють до вхідних.
6. **Текові вхідні** (*Налаштування → Текові вхідні*, адміністратори): стежте за текою сервера, наприклад `/scans` (спершу змонтуйте її в контейнер).
7. **Принтери** (*Налаштування → Принтери*, адміністратори): *Пошук у мережі* або введіть `ipp://printer/ipp/print`; після цього кожен учасник може друкувати зі сторінки документа.

## Конфігурація

Кожне налаштування можна задати змінною середовища з префіксом `NRZPAPER_` (або в `Rocket.toml`).

| Змінна | За замовчуванням | Значення |
|---|---|---|
| `NRZPAPER_DATA_DIR` | `/data` (образ) | База даних, файл ключа й теки приймання |
| `NRZPAPER_DB_KEY` | – | Ключ бази даних; якщо не задано, у теці даних створюється `nrzpaper.key` |
| `NRZPAPER_PORT` / `NRZPAPER_ADDRESS` | `8320` / `0.0.0.0` | Де слухає сервер |
| `NRZPAPER_COOKIE_SECURE` | `false` | Встановіть `true` за HTTPS |
| `NRZPAPER_REGISTRATION_OPEN` | `false` | Дозволити самостійну реєстрацію (можна перемкнути в UI) |
| `NRZPAPER_OCR_LANGUAGES` | `deu+eng` | Мови Tesseract (образ містить `deu` та `eng`) |
| `NRZPAPER_OCR_WORKERS` | `2` | Паралельні завдання OCR |
| `NRZPAPER_TRASH_RETENTION_DAYS` | `30` | Днів у кошику до остаточного видалення |
| `NRZPAPER_INBOX_KEEP_DAYS` | `7` | Скільки днів зберігаються несхвалені елементи вхідних |
| `NRZPAPER_SCANNER_URL` | – | Попередньо заданий сканер для нових користувачів, наприклад `http://192.168.1.50` |
| `NRZPAPER_SCAN_SUBNETS` | `192.168.0.0/24,192.168.1.0/24` | Мережі для пошуку сканерів і принтерів |
| `NRZPAPER_AI_PROVIDER` | `openrouter` | `openai`, `anthropic`, `gemini`, `mistral`, `groq`, `ollama`, `custom`, … |
| `NRZPAPER_AI_KEY` / `NRZPAPER_AI_URL` / `NRZPAPER_AI_MODEL` | – | Ключ, базова URL-адреса й модель для цього постачальника |
| `NRZPAPER_OPENROUTER_KEY` | – | Ключ OpenRouter (альтернатива наведеному вище) |
| `NRZPAPER_LOG` | `info` | Фільтр журналу, наприклад `debug` |

Налаштування з веб-інтерфейсу (постачальник ШІ, ключі, моделі, реєстрація) зберігаються в зашифрованій базі даних і мають пріоритет над середовищем.

### Постачальники ШІ

| Постачальник | Адреса | Примітки |
|---|---|---|
| OpenRouter | вбудований | Безкоштовні візуальні моделі; власний ключ необов'язковий |
| OpenAI, Anthropic, Google Gemini, Mistral, Groq, xAI, DeepSeek, Together, Fireworks, Perplexity, Cohere, NVIDIA, Hugging Face, Qwen | вбудований | API-ключ постачальника |
| Azure OpenAI | `https://NAME.openai.azure.com/openai/v1` | Модель = назва розгортання |
| Ollama / LM Studio / vLLM / LocalAI | `http://host:11434/v1` тощо | Ключ не потрібен |
| Будь-що сумісне з OpenAI | ваша URL-адреса | Ключ необов'язковий |

Моделі, які не читають зображення: позначте *«Модель не читає зображення — надсилати розпізнаний текст»*. Постачальники, що не приймають PDF, автоматично отримують перші сторінки як зображення. Мова пропонованих назв і підсумків задається в *Мова пропозицій*.

## REST API

Створіть API-ключ у *Профіль → API-ключі* (виберіть права). Усі ендпоінти розташовані під `/api/v1` і приймають `Authorization: Bearer np_…`.

```bash
TOKEN=np_your_key
BASE=http://localhost:8320/api/v1

# організації
curl -H "Authorization: Bearer $TOKEN" $BASE/organizations

# завантаження (кілька файлів одразу кількома -F)
curl -H "Authorization: Bearer $TOKEN" -F file=@invoice.pdf \
     $BASE/organizations/$ORG/documents

# пошук: той самий синтаксис, що й в UI
curl -H "Authorization: Bearer $TOKEN" -G $BASE/organizations/$ORG/documents \
     --data-urlencode 'searchQuery=northwind tag:invoice created:2026'

# завантажити файл
curl -H "Authorization: Bearer $TOKEN" -o out.pdf \
     "$BASE/organizations/$ORG/documents/$DOC/file?download=true"

# додати тег
curl -H "Authorization: Bearer $TOKEN" -H "Content-Type: application/json" \
     -d '{"tagId":"'$TAG'"}' $BASE/organizations/$ORG/documents/$DOC/tags

# публічне посилання, чинне 7 днів
curl -H "Authorization: Bearer $TOKEN" -H "Content-Type: application/json" \
     -d '{"expiresInDays":7}' $BASE/organizations/$ORG/documents/$DOC/share-links

# перевірка стану (ключ не потрібен)
curl $BASE/health
```

Вебхуки (*Налаштування → Вебхуки*) надсилають підписаний JSON (`webhook-id`, `webhook-timestamp`, `webhook-signature`) для `document:created`, `document:updated`, `document:deleted`, `document:restored`, `document:tag:added` і `document:tag:removed`.

Тека приймання `data/ingestion/<organization-id>/` імпортує файли безпосередньо як документи. Наглядові теки (*Текові вхідні*) спершу надсилають їх до вхідних на перевірку.

## Розробка

```bash
# бекенд (Rust 1.85+, Tesseract і Poppler для OCR)
cargo run                         # http://localhost:8320
cargo test

# веб-інтерфейс із гарячим перезавантаженням (dx = dioxus-cli 0.7)
rustup target add wasm32-unknown-unknown
cargo install dioxus-cli --version 0.7.10 --locked
cd ui && dx serve                 # http://localhost:8080, проксіює /api на :8320

# вбудувати інтерфейс у програму
scripts/ui.sh && cargo build --release
```

Переклади містяться в `locales/<language>.json` (німецький вихідний текст → переклад). `scripts/i18n_extract.py` оновлює `locales/strings.json` і повідомляє про відсутні записи для кожної мови.

Деталі архітектури: [ARCHITECTURE.md](../ARCHITECTURE.md).

## Ліцензія

AGPL-3.0-or-later. Компоненти UI базуються на [rust-ui](https://github.com/rust-ui/ui) (MIT), переглядач PDF — на [PDF.js](https://mozilla.github.io/pdf.js/) (Apache-2.0), шрифт — [Inter](https://rsms.me/inter/) (OFL).
