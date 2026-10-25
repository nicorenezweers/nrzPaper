<div align="center">

<img src="../static/icon.png" width="96" alt="nrzPaper">

# nrzPaper

**A Rust-powered document management tool.**

[English](../README.md) · [Deutsch](README.de.md) · [Français](README.fr.md) · [Italiano](README.it.md) · [Español](README.es.md) · [Português](README.pt.md) · [Nederlands](README.nl.md) · [中文](README.zh.md) · [日本語](README.ja.md) · [Polski](README.pl.md) · [Русский](README.ru.md) · [العربية](README.ar.md) · [한국어](README.ko.md) · [Türkçe](README.tr.md) · [Bahasa Indonesia](README.id.md) · [हिन्दी](README.hi.md) · [Tiếng Việt](README.vi.md) · [ไทย](README.th.md) · [Українська](README.uk.md) · [Čeština](README.cs.md) · [Română](README.ro.md) · [Magyar](README.hu.md) · [Ελληνικά](README.el.md) · [Svenska](README.sv.md) · [Norsk](README.nb.md) · [Dansk](README.da.md) · [Suomi](README.fi.md) · [עברית](README.he.md) · [বাংলা](README.bn.md) · [فارسی](README.fa.md)

![সময়রেখা সহ নথি](screenshots/documents.png)

</div>

nrzPaper বাহ্যিক ডেটাবেস ছাড়া একটি একক প্রোগ্রাম। ফাইলসহ সবকিছু একটি SQLCipher-এনক্রিপ্টেড SQLite ফাইলে থাকে। এটি Tesseract দিয়ে OCR করে, নেটওয়ার্ক স্ক্যানার থেকে সরাসরি স্ক্যান করে, ইমেলের সংযুক্তি নেয় এবং আপনার পছন্দের AI শিরোনাম, ট্যাগ ও মেটাডেটা প্রস্তাব করতে দেয়। অন্তর্নির্মিত সহায়ক **Dieter** আপনার নথি সম্পর্কে প্রশ্নের উত্তর দেয় এবং উৎস লিঙ্ক করে। ওয়েব ইন্টারফেসটি শান্ত গাঢ় থিমের একটি Rust/WebAssembly অ্যাপ (Dioxus) এবং ফোনেও কাজ করে।

## বৈশিষ্ট্য

| | |
|---|---|
| **সংগ্রহ** | ড্র্যাগ ও ড্রপ, আপলোড, **নেটওয়ার্ক স্ক্যানার** (eSCL/AirScan: Canon, HP, Epson, Brother …), **ইমেল ইনবক্স** (IMAP, একাধিক অ্যাকাউন্ট), **নজরদারি করা ফোল্ডার** (যেমন NAS-এ স্ক্যান ফোল্ডার), REST API |
| **ইনবক্স** | স্ক্যান, মেইল, ফোল্ডার আমদানি এবং আপলোড পর্যালোচনার জন্য অপেক্ষা করে: AI নাম, ট্যাগ ও ক্ষেত্র প্রস্তাব করে, এবং আপনি একটি একটি বা **একসাথে সব** অনুমোদন করেন। উৎস অনুযায়ী ফিল্টার করুন (স্ক্যানার, প্রতিটি মেইলবক্স, প্রতিটি ফোল্ডার, আপলোড)। |
| **স্বীকৃতি** | ব্যাকগ্রাউন্ডে Tesseract এবং Poppler, স্ক্যান করা PDF ও ছবির জন্যও; উপসর্গ, বাক্যাংশ ও ফিল্টারসহ পূর্ণ-পাঠ অনুসন্ধান (`tag:`, `-tag:`, `type:`, `created:>2026-06-01`, …) |
| **সংগঠন** | রঙিন ট্যাগ, **ট্যাগ থেকে তৈরি ফোল্ডার** (নেস্টেড), **ট্যাগিং নিয়ম** ("Northwind আছে → চালান"), কাস্টম **বৈশিষ্ট্য** (টেক্সট, সংখ্যা, তারিখ, হ্যাঁ/না, নির্বাচন, বহু-নির্বাচন, ব্যবহারকারী, নথি লিঙ্ক) |
| **সময়রেখা** | নথির পাশে উল্লম্ব সময়রেখা: আজ উপরে, নিচে মেইলসহ প্রতিটি দিন এবং মাঝের নীরব বিরতি ("১২ দিন নথি ছাড়া")। ট্যাগ, ধরন ও টেক্সট অনুযায়ী ফিল্টার করুন। |
| **রিমাইন্ডার** | যেকোনো নথিতে নোটসহ "১০ দিনে মনে করিয়ে দাও"; নির্ধারিত রিমাইন্ডার সাইডবারে এবং সময়রেখার উপরে দেখায়। |
| **মুদ্রণ** | সার্ভার যে কোনো **IPP নেটওয়ার্ক প্রিন্টার** (AirPrint, IPP Everywhere, Mopria, CUPS) খুঁজে পায় সেখানে নথি মুদ্রণ করুন। PDF সরাসরি যায়, বা শুধু JPEG গ্রহণকারী প্রিন্টারে পৃষ্ঠার ছবি হিসেবে। |
| **AI, যেকোনো প্রদানকারী** | OpenRouter (অন্তর্নির্মিত বিনামূল্যের মডেল), OpenAI, Anthropic, Google Gemini, Mistral, Groq, xAI, DeepSeek, Together, Fireworks, Perplexity, Cohere, NVIDIA, Hugging Face, Qwen, Azure OpenAI, **Ollama, LM Studio, vLLM/LocalAI** বা যেকোনো OpenAI-সামঞ্জস্যপূর্ণ URL। শুধু-টেক্সট মডেল ছবির বদলে OCR টেক্সট পায়। |
| **Dieter** | চ্যাট সহায়ক (নিচে ডানে): *"আমার বীমা নম্বর কী?"*, *"২০২৬ সালে Fabrikam-এর সব বেতন স্লিপ"*। তিনি খোঁজেন, মিলে যাওয়া অংশ পড়েন এবং নথির লিঙ্কসহ উত্তর দেন। |
| **দল** | একাধিক প্রতিষ্ঠান, **প্রতি প্রতিষ্ঠানে একাধিক অ্যাকাউন্ট** (আমন্ত্রণ, ভূমিকা: মালিক, প্রশাসক, সদস্য; মালিকানা হস্তান্তর), অনুমতিসহ API কী, ওয়েবহুক (Standard Webhooks স্বাক্ষর), পাবলিক শেয়ার লিঙ্ক |
| **ভাষা** | English, Deutsch, Français, Italiano, Español, Português, Nederlands (আরও ভাষা শীঘ্রই আসছে) |
| **ডিফল্টভাবে নিরাপদ** | একটি এনক্রিপ্টেড ফাইল, Argon2 পাসওয়ার্ড, ইনলাইন স্ক্রিপ্ট ছাড়া CSP, CSRF সুরক্ষা, ফাইল শুধু API-এর মাধ্যমে |

## স্ক্রিনশট

| | |
|---|---|
| ![Dieter, সহায়ক](screenshots/dieter.png) | ![AI প্রস্তাবসহ ইনবক্স](screenshots/inbox.png) |
| **Dieter** উৎসসহ উত্তর দেন | **ইনবক্স**: পর্যালোচনা ও অনুমোদন, উৎস অনুযায়ী ফিল্টার |
| ![নথি দৃশ্য](screenshots/document.png) | ![AI প্রদানকারী](screenshots/settings-ai.png) |
| **নথি**: PDF ভিউয়ার, ট্যাগ, রিমাইন্ডার, মুদ্রণ | **AI ও মডেল**: প্রতিটি প্রদানকারী, স্থানীয় বা ক্লাউড |
| ![রিমাইন্ডার](screenshots/reminders.png) | ![সদস্য](screenshots/members.png) |
| **রিমাইন্ডার** | **সদস্য** ও আমন্ত্রণ |

## দ্রুত শুরু

আপনার Docker দরকার। ইমেজে প্রোগ্রাম, ওয়েব ইন্টারফেস, Tesseract (জার্মান, ইংরেজি) এবং Poppler আছে।

```bash
git clone https://github.com/nicorenezweers/nrzPaper.git
cd nrzPaper
docker build -t nrzpaper .
docker run -d --name nrzpaper -p 8320:8320 -v nrzpaper-data:/data nrzpaper
```

<http://localhost:8320> খুলুন। আপনি যে প্রথম অ্যাকাউন্ট তৈরি করবেন তিনি প্রশাসক হবেন। এরপর নিবন্ধন বন্ধ থাকে যতক্ষণ না আপনি *সেটিংস → নিবন্ধন ও ব্যবহারকারী* তে খোলেন, যেখানে অন্যদের জন্য অ্যাকাউন্টও তৈরি করতে পারেন।

### Docker Compose

```yaml
services:
  nrzpaper:
    build: .                      # অথবা: image: ghcr.io/nicorenezweers/nrzpaper:1.0.0
    container_name: nrzpaper
    restart: unless-stopped
    ports:
      - "8320:8320"
    volumes:
      - nrzpaper-data:/data       # এনক্রিপ্টেড ডেটাবেস + কী ফাইল
      - /srv/scans:/scans         # ঐচ্ছিক: ফোল্ডার ইনবক্সের জন্য ফোল্ডার
    environment:
      TZ: Europe/Zurich
      # NRZPAPER_DB_KEY: "a-long-secret"     # না হলে /data/nrzpaper.key তৈরি হয়
      # NRZPAPER_COOKIE_SECURE: "true"       # HTTPS-এর মাধ্যমে পরিবেশন করলে
      # NRZPAPER_AI_PROVIDER: "openai"       # অথবা ওয়েব ইন্টারফেসে সেট করুন
      # NRZPAPER_AI_KEY: "sk-…"

volumes:
  nrzpaper-data:
```

```bash
docker compose up -d
```

> **`/data` ব্যাকআপ করুন।** এতে `nrzpaper.db` (সবকিছু, এনক্রিপ্টেড) এবং `nrzpaper.key` আছে। কী ছাড়া ডেটাবেস খোলা যায় না। যদি `NRZPAPER_DB_KEY` সেট করেন, তার বদলে সেই মানটি নিরাপদে রাখুন।

### উভয় আর্কিটেকচার

```bash
docker buildx build --platform linux/amd64,linux/arm64 -t <registry>/nrzpaper:1.0.0 --push .
```

Rust ক্রস-কম্পাইল হয়, এমুলেট নয়, তাই arm64 বিল্ড x86 মেশিনে দ্রুত এবং উল্টোটাও।

## প্রথম ধাপ

1. কয়েকটি PDF বা ছবি **আপলোড করুন**: টেক্সট ব্যাকগ্রাউন্ডে চেনা হয়।
2. **ট্যাগ**: *চালান* বা *বীমা* এর মতো ট্যাগ তৈরি করুন এবং বর্ণনা যোগ করুন। AI এটি ব্যবহার করে ট্যাগ নির্ধারণ করে।
3. **AI** (*সেটিংস → AI ও মডেল*): প্রদানকারী বেছে নিন, কী পেস্ট করুন (অথবা অন্তর্নির্মিত বিনামূল্যের OpenRouter মডেল বা স্থানীয় Ollama ব্যবহার করুন), তারপর *মডেল লোড করুন* এবং *সংযোগ পরীক্ষা করুন*।
4. **স্ক্যানার** (*প্রোফাইল → স্ক্যানার*): *নেটওয়ার্ক খুঁজুন* eSCL/AirScan স্ক্যানার খুঁজে পায়; তারপর সাইডবারে *স্ক্যান*।
5. **ইমেল** (*প্রোফাইল → ইমেল ইনবক্স*): এক বা একাধিক IMAP মেইলবক্স যোগ করুন; সংযুক্তির PDF ও ছবি ইনবক্সে আসে।
6. **ফোল্ডার ইনবক্স** (*সেটিংস → ফোল্ডার ইনবক্স*, প্রশাসক): `/scans` এর মতো সার্ভার ফোল্ডার নজরদারি করুন (প্রথমে এটি কন্টেইনারে মাউন্ট করুন)।
7. **প্রিন্টার** (*সেটিংস → প্রিন্টার*, প্রশাসক): *নেটওয়ার্ক খুঁজুন* বা `ipp://printer/ipp/print` লিখুন; তারপর প্রতিটি সদস্য নথি পৃষ্ঠা থেকে মুদ্রণ করতে পারেন।

## কনফিগারেশন

প্রতিটি সেটিং `NRZPAPER_` উপসর্গসহ পরিবেশ চলক (বা `Rocket.toml`-এ) হিসেবে দেওয়া যায়।

| চলক | ডিফল্ট | অর্থ |
|---|---|---|
| `NRZPAPER_DATA_DIR` | `/data` (ইমেজ) | ডেটাবেস, কী ফাইল ও গ্রহণ ফোল্ডার |
| `NRZPAPER_DB_KEY` | – | ডেটাবেস কী; সেট না থাকলে ডেটা ফোল্ডারে `nrzpaper.key` তৈরি হয় |
| `NRZPAPER_PORT` / `NRZPAPER_ADDRESS` | `8320` / `0.0.0.0` | সার্ভার কোথায় শোনে |
| `NRZPAPER_COOKIE_SECURE` | `false` | HTTPS-এর পিছনে `true` সেট করুন |
| `NRZPAPER_REGISTRATION_OPEN` | `false` | স্ব-নিবন্ধন অনুমোদন (UI-তেও বদলানো যায়) |
| `NRZPAPER_OCR_LANGUAGES` | `deu+eng` | Tesseract ভাষা (ইমেজে `deu` ও `eng` আছে) |
| `NRZPAPER_OCR_WORKERS` | `2` | সমান্তরাল OCR কাজ |
| `NRZPAPER_TRASH_RETENTION_DAYS` | `30` | চূড়ান্ত মুছে ফেলার আগে ট্র্যাশে দিন |
| `NRZPAPER_INBOX_KEEP_DAYS` | `7` | অননুমোদিত ইনবক্স আইটেম কত দিন রাখা হয় |
| `NRZPAPER_SCANNER_URL` | – | নতুন ব্যবহারকারীর জন্য পূর্বনির্ধারিত স্ক্যানার, যেমন `http://192.168.1.50` |
| `NRZPAPER_SCAN_SUBNETS` | `192.168.0.0/24,192.168.1.0/24` | স্ক্যানার ও প্রিন্টার খোঁজার নেটওয়ার্ক |
| `NRZPAPER_AI_PROVIDER` | `openrouter` | `openai`, `anthropic`, `gemini`, `mistral`, `groq`, `ollama`, `custom`, … |
| `NRZPAPER_AI_KEY` / `NRZPAPER_AI_URL` / `NRZPAPER_AI_MODEL` | – | সেই প্রদানকারীর কী, বেস URL ও মডেল |
| `NRZPAPER_OPENROUTER_KEY` | – | OpenRouter কী (উপরের বিকল্প) |
| `NRZPAPER_LOG` | `info` | লগ ফিল্টার, যেমন `debug` |

ওয়েব ইন্টারফেসে করা সেটিংস (AI প্রদানকারী, কী, মডেল, নিবন্ধন) এনক্রিপ্টেড ডেটাবেসে সংরক্ষিত হয় এবং পরিবেশের ওপর অগ্রাধিকার পায়।

### AI প্রদানকারী

| প্রদানকারী | ঠিকানা | নোট |
|---|---|---|
| OpenRouter | অন্তর্নির্মিত | বিনামূল্যের ভিশন মডেল; নিজের কী ঐচ্ছিক |
| OpenAI, Anthropic, Google Gemini, Mistral, Groq, xAI, DeepSeek, Together, Fireworks, Perplexity, Cohere, NVIDIA, Hugging Face, Qwen | অন্তর্নির্মিত | প্রদানকারীর API কী |
| Azure OpenAI | `https://NAME.openai.azure.com/openai/v1` | মডেল = ডিপ্লয়মেন্ট নাম |
| Ollama / LM Studio / vLLM / LocalAI | `http://host:11434/v1` ইত্যাদি | কী দরকার নেই |
| OpenAI-সামঞ্জস্যপূর্ণ যেকোনো কিছু | আপনার URL | কী ঐচ্ছিক |

যে মডেল ছবি পড়তে পারে না: *"মডেল ছবি পড়তে পারে না – চেনা টেক্সট পাঠান"* টিক দিন। যে প্রদানকারীরা PDF গ্রহণ করে না তারা স্বয়ংক্রিয়ভাবে প্রথম পৃষ্ঠাগুলো ছবি হিসেবে পায়। প্রস্তাবিত শিরোনাম ও সারাংশের ভাষা *প্রস্তাবের ভাষা*-এ সেট করা হয়।

## REST API

*প্রোফাইল → API কী* তে একটি API কী তৈরি করুন (অনুমতি বেছে নিন)। সব এন্ডপয়েন্ট `/api/v1`-এর অধীনে এবং `Authorization: Bearer np_…` গ্রহণ করে।

```bash
TOKEN=np_your_key
BASE=http://localhost:8320/api/v1

# প্রতিষ্ঠান
curl -H "Authorization: Bearer $TOKEN" $BASE/organizations

# আপলোড (একাধিক -F দিয়ে একসাথে একাধিক ফাইল)
curl -H "Authorization: Bearer $TOKEN" -F file=@invoice.pdf \
     $BASE/organizations/$ORG/documents

# অনুসন্ধান: UI-এর মতো একই সিনট্যাক্স
curl -H "Authorization: Bearer $TOKEN" -G $BASE/organizations/$ORG/documents \
     --data-urlencode 'searchQuery=northwind tag:invoice created:2026'

# ফাইল ডাউনলোড করুন
curl -H "Authorization: Bearer $TOKEN" -o out.pdf \
     "$BASE/organizations/$ORG/documents/$DOC/file?download=true"

# ট্যাগ যোগ করুন
curl -H "Authorization: Bearer $TOKEN" -H "Content-Type: application/json" \
     -d '{"tagId":"'$TAG'"}' $BASE/organizations/$ORG/documents/$DOC/tags

# ৭ দিনের জন্য বৈধ পাবলিক শেয়ার লিঙ্ক
curl -H "Authorization: Bearer $TOKEN" -H "Content-Type: application/json" \
     -d '{"expiresInDays":7}' $BASE/organizations/$ORG/documents/$DOC/share-links

# স্বাস্থ্য পরীক্ষা (কী দরকার নেই)
curl $BASE/health
```

ওয়েবহুক (*সেটিংস → ওয়েবহুক*) `document:created`, `document:updated`, `document:deleted`, `document:restored`, `document:tag:added` এবং `document:tag:removed`-এর জন্য স্বাক্ষরিত JSON (`webhook-id`, `webhook-timestamp`, `webhook-signature`) পাঠায়।

গ্রহণ ফোল্ডার `data/ingestion/<organization-id>/` ফাইলগুলো সরাসরি নথি হিসেবে আমদানি করে। নজরদারি করা ফোল্ডার (*ফোল্ডার ইনবক্স*) প্রথমে সেগুলো পর্যালোচনার জন্য ইনবক্সে পাঠায়।

## উন্নয়ন

```bash
# ব্যাকএন্ড (Rust 1.85+, OCR-এর জন্য Tesseract ও Poppler)
cargo run                         # http://localhost:8320
cargo test

# হট রিলোডসহ ওয়েব ইন্টারফেস (dx = dioxus-cli 0.7)
rustup target add wasm32-unknown-unknown
cargo install dioxus-cli --version 0.7.10 --locked
cd ui && dx serve                 # http://localhost:8080, /api কে :8320-এ প্রক্সি করে

# ইন্টারফেস প্রোগ্রামে এমবেড করুন
scripts/ui.sh && cargo build --release
```

অনুবাদ `locales/<language>.json`-এ (জার্মান উৎস টেক্সট → অনুবাদ)। `scripts/i18n_extract.py` `locales/strings.json` রিফ্রেশ করে এবং প্রতি ভাষায় অনুপস্থিত এন্ট্রি রিপোর্ট করে।

আর্কিটেকচার বিবরণ: [ARCHITECTURE.md](../ARCHITECTURE.md)।

## লাইসেন্স

AGPL-3.0-or-later। UI উপাদানগুলো [rust-ui](https://github.com/rust-ui/ui) (MIT), PDF ভিউয়ার [PDF.js](https://mozilla.github.io/pdf.js/) (Apache-2.0), ফন্ট [Inter](https://rsms.me/inter/) (OFL) ভিত্তিক।
