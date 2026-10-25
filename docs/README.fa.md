<div align="center" dir="rtl">

<img src="../static/icon.png" width="96" alt="nrzPaper">

# nrzPaper

**A Rust-powered document management tool.**

[English](../README.md) · [Deutsch](README.de.md) · [Français](README.fr.md) · [Italiano](README.it.md) · [Español](README.es.md) · [Português](README.pt.md) · [Nederlands](README.nl.md) · [中文](README.zh.md) · [日本語](README.ja.md) · [Polski](README.pl.md) · [Русский](README.ru.md) · [العربية](README.ar.md) · [한국어](README.ko.md) · [Türkçe](README.tr.md) · [Bahasa Indonesia](README.id.md) · [हिन्दी](README.hi.md) · [Tiếng Việt](README.vi.md) · [ไทย](README.th.md) · [Українська](README.uk.md) · [Čeština](README.cs.md) · [Română](README.ro.md) · [Magyar](README.hu.md) · [Ελληνικά](README.el.md) · [Svenska](README.sv.md) · [Norsk](README.nb.md) · [Dansk](README.da.md) · [Suomi](README.fi.md) · [עברית](README.he.md) · [বাংলা](README.bn.md) · [فارسی](README.fa.md)

![اسناد با خط زمان](screenshots/documents.png)

</div>

nrzPaper یک برنامه واحد بدون پایگاه داده خارجی است. همه‌چیز در یک فایل SQLite رمزنگاری‌شده با SQLCipher قرار دارد، از جمله خود فایل‌ها. با Tesseract OCR انجام می‌دهد، مستقیماً از اسکنرهای شبکه اسکن می‌کند، پیوست‌های ایمیل را برمی‌دارد و اجازه می‌دهد هوش مصنوعی انتخابی شما عنوان، برچسب و فراداده پیشنهاد دهد. دستیار داخلی **Dieter** به پرسش‌ها درباره اسناد شما پاسخ می‌دهد و منابع را پیوند می‌دهد. رابط وب یک برنامه Rust/WebAssembly (Dioxus) با پوسته تیره آرام است و روی گوشی هم کار می‌کند.

## ویژگی‌ها

| | |
|---|---|
| **جمع‌آوری** | کشیدن و رها کردن، بارگذاری، **اسکنرهای شبکه** (eSCL/AirScan: Canon، HP، Epson، Brother …)، **صندوق‌های ایمیل** (IMAP، چند حساب)، **پوشه‌های تحت نظارت** (مثلاً پوشه اسکن روی NAS)، REST API |
| **صندوق ورودی** | اسکن‌ها، ایمیل‌ها، ورود از پوشه‌ها و بارگذاری‌ها منتظر بازبینی هستند: هوش مصنوعی نام، برچسب‌ها و فیلدها را پیشنهاد می‌دهد و شما یک مورد یا **همه را یکجا** تأیید می‌کنید. فیلتر بر اساس منبع (اسکنر، هر صندوق ایمیل، هر پوشه، بارگذاری). |
| **شناسایی** | Tesseract و Poppler در پس‌زمینه، همچنین برای PDFهای اسکن‌شده و عکس‌ها؛ جستجوی متن کامل با پیشوندها، عبارات و فیلترها (`tag:`، `-tag:`، `type:`، `created:>2026-06-01` و …) |
| **سازمان‌دهی** | برچسب‌های رنگی، **پوشه‌های ساخته‌شده از برچسب‌ها** (تودرتو)، **قواعد برچسب‌گذاری** («شامل Northwind → فاکتور»)، **ویژگی‌های** سفارشی (متن، عدد، تاریخ، بله/خیر، انتخاب، انتخاب چندگانه، کاربر، پیوند سند) |
| **خط زمان** | خط زمان عمودی کنار اسناد: امروز در بالا، زیر آن هر روز با ایمیل و شکاف‌های آرام میانشان («۱۲ روز بدون سند»). فیلتر بر اساس برچسب، نوع و متن. |
| **یادآورها** | «۱۰ روز دیگر یادآوری کن» روی هر سند، با یادداشت؛ یادآورهای سررسیده در نوار کناری و بالای خط زمان نمایش داده می‌شوند. |
| **چاپ** | چاپ اسناد روی هر **چاپگر شبکه IPP** (AirPrint، IPP Everywhere، Mopria، CUPS) که سرور پیدا کند. PDFها مستقیم یا به‌صورت تصویر صفحه برای چاپگرهایی که فقط JPEG می‌پذیرند ارسال می‌شوند. |
| **هوش مصنوعی، هر ارائه‌دهنده** | OpenRouter (مدل‌های رایگان داخلی)، OpenAI، Anthropic، Google Gemini، Mistral، Groq، xAI، DeepSeek، Together، Fireworks، Perplexity، Cohere، NVIDIA، Hugging Face، Qwen، Azure OpenAI، **Ollama، LM Studio، vLLM/LocalAI** یا هر نشانی سازگار با OpenAI. مدل‌های متنی به‌جای تصویر، متن شناسایی‌شده را دریافت می‌کنند. |
| **Dieter** | دستیار چت (پایین راست): *«شماره بیمه من چیست؟»*، *«همه فیش‌های حقوقی Fabrikam در ۲۰۲۶»*. جستجو می‌کند، بخش‌های منطبق را می‌خواند و با پیوند به اسناد پاسخ می‌دهد. |
| **تیم‌ها** | چند سازمان، **چند حساب برای هر سازمان** (دعوت، نقش‌ها: مالک، مدیر، عضو؛ انتقال مالکیت)، کلیدهای API با مجوزها، وب‌هوک‌ها (امضای Standard Webhooks)، پیوندهای اشتراک عمومی |
| **زبان‌ها** | English, Deutsch, Français, Italiano, Español, Português, Nederlands (زبان‌های بیشتر به‌زودی) |
| **امن به‌طور پیش‌فرض** | یک فایل رمزنگاری‌شده، گذرواژه‌های Argon2، CSP بدون اسکریپت درون‌خطی، محافظت CSRF، فایل‌ها فقط از طریق API |

## تصاویر

| | |
|---|---|
| ![Dieter، دستیار](screenshots/dieter.png) | ![صندوق ورودی با پیشنهادهای هوش مصنوعی](screenshots/inbox.png) |
| **Dieter** با منابع پاسخ می‌دهد | **صندوق ورودی**: بازبینی و تأیید، فیلتر بر اساس منبع |
| ![نمای سند](screenshots/document.png) | ![ارائه‌دهندگان هوش مصنوعی](screenshots/settings-ai.png) |
| **سند**: نمایشگر PDF، برچسب‌ها، یادآورها، چاپ | **هوش مصنوعی و مدل‌ها**: هر ارائه‌دهنده، محلی یا ابری |
| ![یادآورها](screenshots/reminders.png) | ![اعضا](screenshots/members.png) |
| **یادآورها** | **اعضا** و دعوت‌ها |

## شروع سریع

به Docker نیاز دارید. تصویر شامل برنامه، رابط وب، Tesseract (آلمانی، انگلیسی) و Poppler است.

```bash
git clone https://github.com/nicorenezweers/nrzPaper.git
cd nrzPaper
docker build -t nrzpaper .
docker run -d --name nrzpaper -p 8320:8320 -v nrzpaper-data:/data nrzpaper
```

<http://localhost:8320> را باز کنید. اولین حسابی که می‌سازید مدیر می‌شود. پس از آن ثبت‌نام بسته است تا آن را در *تنظیمات → ثبت‌نام و کاربران* باز کنید، جایی که می‌توانید برای دیگران هم حساب بسازید.

### Docker Compose

```yaml
services:
  nrzpaper:
    build: .                      # یا: image: ghcr.io/nicorenezweers/nrzpaper:1.0.0
    container_name: nrzpaper
    restart: unless-stopped
    ports:
      - "8320:8320"
    volumes:
      - nrzpaper-data:/data       # پایگاه داده رمزنگاری‌شده + فایل کلید
      - /srv/scans:/scans         # اختیاری: پوشه‌ای برای صندوق ورودی پوشه‌ای
    environment:
      TZ: Europe/Zurich
      # NRZPAPER_DB_KEY: "a-long-secret"     # در غیر این صورت /data/nrzpaper.key ساخته می‌شود
      # NRZPAPER_COOKIE_SECURE: "true"       # هنگام سرو از طریق HTTPS
      # NRZPAPER_AI_PROVIDER: "openai"       # یا در رابط وب تنظیم کنید
      # NRZPAPER_AI_KEY: "sk-…"

volumes:
  nrzpaper-data:
```

```bash
docker compose up -d
```

> **از `/data` پشتیبان بگیرید.** شامل `nrzpaper.db` (همه‌چیز، رمزنگاری‌شده) و `nrzpaper.key` است. بدون کلید پایگاه داده باز نمی‌شود. اگر `NRZPAPER_DB_KEY` را تنظیم کردید، آن مقدار را امن نگه دارید.

### هر دو معماری

```bash
docker buildx build --platform linux/amd64,linux/arm64 -t <registry>/nrzpaper:1.0.0 --push .
```

Rust به‌صورت cross-compile ساخته می‌شود نه شبیه‌سازی، بنابراین ساخت arm64 روی ماشین x86 سریع است و برعکس.

## گام‌های نخست

1. **بارگذاری** چند PDF یا عکس: متن در پس‌زمینه شناسایی می‌شود.
2. **برچسب‌ها**: برچسب‌هایی مانند *فاکتور* یا *بیمه* بسازید و توضیح بدهید. هوش مصنوعی از آن برای تخصیص برچسب استفاده می‌کند.
3. **هوش مصنوعی** (*تنظیمات → هوش مصنوعی و مدل‌ها*): ارائه‌دهنده را انتخاب کنید، کلید را جای‌گذاری کنید (یا از مدل‌های رایگان داخلی OpenRouter یا Ollama محلی استفاده کنید)، سپس *بارگذاری مدل‌ها* و *آزمایش اتصال*.
4. **اسکنر** (*پروفایل → اسکنر*): *جستجوی شبکه* اسکنرهای eSCL/AirScan را پیدا می‌کند؛ سپس *اسکن* در نوار کناری.
5. **ایمیل** (*پروفایل → صندوق ایمیل*): یک یا چند صندوق IMAP اضافه کنید؛ PDFها و تصاویر پیوست‌ها به صندوق ورودی می‌رسند.
6. **صندوق ورودی پوشه‌ای** (*تنظیمات → صندوق ورودی پوشه‌ای*، مدیران): پوشه‌ای روی سرور مانند `/scans` را نظارت کنید (ابتدا آن را در کانتینر سوار کنید).
7. **چاپگرها** (*تنظیمات → چاپگرها*، مدیران): *جستجوی شبکه* یا `ipp://printer/ipp/print` را وارد کنید؛ سپس هر عضو می‌تواند از صفحه سند چاپ کند.

## پیکربندی

هر تنظیم را می‌توان به‌صورت متغیر محیطی با پیشوند `NRZPAPER_` (یا در `Rocket.toml`) داد.

| متغیر | پیش‌فرض | معنا |
|---|---|---|
| `NRZPAPER_DATA_DIR` | `/data` (تصویر) | پایگاه داده، فایل کلید و پوشه‌های دریافت |
| `NRZPAPER_DB_KEY` | – | کلید پایگاه داده؛ اگر تنظیم نشود، `nrzpaper.key` در پوشه داده ساخته می‌شود |
| `NRZPAPER_PORT` / `NRZPAPER_ADDRESS` | `8320` / `0.0.0.0` | جایی که سرور گوش می‌دهد |
| `NRZPAPER_COOKIE_SECURE` | `false` | پشت HTTPS روی `true` بگذارید |
| `NRZPAPER_REGISTRATION_OPEN` | `false` | اجازه ثبت‌نام خودکار (در رابط هم قابل تغییر است) |
| `NRZPAPER_OCR_LANGUAGES` | `deu+eng` | زبان‌های Tesseract (تصویر شامل `deu` و `eng` است) |
| `NRZPAPER_OCR_WORKERS` | `2` | کارهای موازی OCR |
| `NRZPAPER_TRASH_RETENTION_DAYS` | `30` | روزهای ماندن در سطل زباله تا حذف نهایی |
| `NRZPAPER_INBOX_KEEP_DAYS` | `7` | روزهای نگه‌داشتن موارد تأییدنشده صندوق ورودی |
| `NRZPAPER_SCANNER_URL` | – | اسکنر پیش‌تنظیم برای کاربران جدید، مثلاً `http://192.168.1.50` |
| `NRZPAPER_SCAN_SUBNETS` | `192.168.0.0/24,192.168.1.0/24` | شبکه‌هایی که برای اسکنر و چاپگر جستجو می‌شوند |
| `NRZPAPER_AI_PROVIDER` | `openrouter` | `openai`، `anthropic`، `gemini`، `mistral`، `groq`، `ollama`، `custom` و … |
| `NRZPAPER_AI_KEY` / `NRZPAPER_AI_URL` / `NRZPAPER_AI_MODEL` | – | کلید، نشانی پایه و مدل آن ارائه‌دهنده |
| `NRZPAPER_OPENROUTER_KEY` | – | کلید OpenRouter (جایگزین موارد بالا) |
| `NRZPAPER_LOG` | `info` | فیلتر گزارش، مثلاً `debug` |

تنظیمات انجام‌شده در رابط وب (ارائه‌دهنده هوش مصنوعی، کلیدها، مدل‌ها، ثبت‌نام) در پایگاه داده رمزنگاری‌شده ذخیره می‌شوند و بر محیط اولویت دارند.

### ارائه‌دهندگان هوش مصنوعی

| ارائه‌دهنده | نشانی | نکته |
|---|---|---|
| OpenRouter | داخلی | مدل‌های بینایی رایگان؛ کلید شخصی اختیاری |
| OpenAI، Anthropic، Google Gemini، Mistral، Groq، xAI، DeepSeek، Together، Fireworks، Perplexity، Cohere، NVIDIA، Hugging Face، Qwen | داخلی | کلید API ارائه‌دهنده |
| Azure OpenAI | `https://NAME.openai.azure.com/openai/v1` | مدل = نام استقرار |
| Ollama / LM Studio / vLLM / LocalAI | `http://host:11434/v1` و غیره | نیازی به کلید نیست |
| هر چیز سازگار با OpenAI | نشانی شما | کلید اختیاری |

مدل‌هایی که تصویر نمی‌خوانند: *«مدل تصویر نمی‌خواند — متن شناسایی‌شده ارسال شود»* را تیک بزنید. ارائه‌دهندگانی که PDF نمی‌پذیرند به‌طور خودکار صفحات اول را به‌صورت تصویر دریافت می‌کنند. زبان عنوان‌ها و خلاصه‌های پیشنهادی در *زبان پیشنهادها* تنظیم می‌شود.

## REST API

کلید API را در *پروفایل → کلیدهای API* بسازید (مجوزها را انتخاب کنید). همه نقاط پایانی زیر `/api/v1` هستند و `Authorization: Bearer np_…` را می‌پذیرند.

```bash
TOKEN=np_your_key
BASE=http://localhost:8320/api/v1

# سازمان‌ها
curl -H "Authorization: Bearer $TOKEN" $BASE/organizations

# بارگذاری (چند فایل یکجا با چند -F)
curl -H "Authorization: Bearer $TOKEN" -F file=@invoice.pdf \
     $BASE/organizations/$ORG/documents

# جستجو: همان نحو رابط
curl -H "Authorization: Bearer $TOKEN" -G $BASE/organizations/$ORG/documents \
     --data-urlencode 'searchQuery=northwind tag:invoice created:2026'

# دانلود فایل
curl -H "Authorization: Bearer $TOKEN" -o out.pdf \
     "$BASE/organizations/$ORG/documents/$DOC/file?download=true"

# افزودن برچسب
curl -H "Authorization: Bearer $TOKEN" -H "Content-Type: application/json" \
     -d '{"tagId":"'$TAG'"}' $BASE/organizations/$ORG/documents/$DOC/tags

# پیوند اشتراک عمومی معتبر برای ۷ روز
curl -H "Authorization: Bearer $TOKEN" -H "Content-Type: application/json" \
     -d '{"expiresInDays":7}' $BASE/organizations/$ORG/documents/$DOC/share-links

# بررسی سلامت (بدون کلید)
curl $BASE/health
```

وب‌هوک‌ها (*تنظیمات → وب‌هوک‌ها*) برای `document:created`، `document:updated`، `document:deleted`، `document:restored`، `document:tag:added` و `document:tag:removed` JSON امضاشده (`webhook-id`، `webhook-timestamp`، `webhook-signature`) می‌فرستند.

پوشه دریافت `data/ingestion/<organization-id>/` فایل‌ها را مستقیماً به‌عنوان سند وارد می‌کند. پوشه‌های تحت نظارت (*صندوق ورودی پوشه‌ای*) ابتدا آن‌ها را برای بازبینی به صندوق ورودی می‌فرستند.

## توسعه

```bash
# بک‌اند (Rust 1.85+، Tesseract و Poppler برای OCR)
cargo run                         # http://localhost:8320
cargo test

# رابط وب با بارگذاری داغ (dx = dioxus-cli 0.7)
rustup target add wasm32-unknown-unknown
cargo install dioxus-cli --version 0.7.10 --locked
cd ui && dx serve                 # http://localhost:8080، /api را به :8320 پروکسی می‌کند

# جاسازی رابط در برنامه
scripts/ui.sh && cargo build --release
```

ترجمه‌ها در `locales/<language>.json` هستند (متن مبدأ آلمانی → ترجمه). `scripts/i18n_extract.py` فایل `locales/strings.json` را به‌روز می‌کند و موارد گمشده هر زبان را گزارش می‌دهد.

جزئیات معماری: [ARCHITECTURE.md](../ARCHITECTURE.md).

## مجوز

AGPL-3.0-or-later. اجزای رابط بر پایه [rust-ui](https://github.com/rust-ui/ui) (MIT)، نمایشگر PDF بر پایه [PDF.js](https://mozilla.github.io/pdf.js/) (Apache-2.0) و قلم [Inter](https://rsms.me/inter/) (OFL) است.
