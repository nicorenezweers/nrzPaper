<div align="center" dir="rtl">

<img src="../static/icon.png" width="96" alt="nrzPaper">

# nrzPaper

**A Rust-powered document management tool.**

[English](../README.md) · [Deutsch](README.de.md) · [Français](README.fr.md) · [Italiano](README.it.md) · [Español](README.es.md) · [Português](README.pt.md) · [Nederlands](README.nl.md) · [中文](README.zh.md) · [日本語](README.ja.md) · [Polski](README.pl.md) · [Русский](README.ru.md) · [العربية](README.ar.md) · [한국어](README.ko.md) · [Türkçe](README.tr.md) · [Bahasa Indonesia](README.id.md) · [हिन्दी](README.hi.md) · [Tiếng Việt](README.vi.md) · [ไทย](README.th.md) · [Українська](README.uk.md) · [Čeština](README.cs.md) · [Română](README.ro.md) · [Magyar](README.hu.md) · [Ελληνικά](README.el.md) · [Svenska](README.sv.md) · [Norsk](README.nb.md) · [Dansk](README.da.md) · [Suomi](README.fi.md) · [עברית](README.he.md) · [বাংলা](README.bn.md) · [فارسی](README.fa.md)

![مستندات مع خط زمني](screenshots/documents.png)

</div>

nrzPaper برنامج واحد بدون قاعدة بيانات خارجية. كل شيء موجود في ملف SQLite واحد مشفّر بـ SQLCipher، بما في ذلك الملفات نفسها. يقوم بتعرّف ضوئي (OCR) باستخدام Tesseract، والمسح مباشرة من ماسحات الشبكة، ويجلب مرفقات البريد الإلكتروني، ويسمح لأي ذكاء اصطناعي تختاره باقتراح العناوين والوسوم والبيانات الوصفية. المساعد المدمج **Dieter** يجيب عن الأسئلة المتعلقة بمستنداتك ويربط المصادر. واجهة الويب تطبيق Rust/WebAssembly (Dioxus) بسمة داكنة هادئة وتعمل أيضًا على الهواتف.

## الميزات

| | |
|---|---|
| **الالتقاط** | السحب والإفلات، الرفع، **ماسحات الشبكة** (eSCL/AirScan: Canon وHP وEpson وBrother …)، **صناديق البريد الإلكتروني** (IMAP، عدة حسابات)، **المجلدات المراقبة** (مثل مجلد المسح على NAS)، واجهة REST API |
| **الوارد** | عمليات المسح والرسائل واستيراد المجلدات والملفات المرفوعة تنتظر المراجعة: يقترح الذكاء الاصطناعي الاسم والوسوم والحقول، وتوافق على عنصر واحد أو **الكل دفعة واحدة**. تصفية حسب المصدر (الماسح، كل صندوق بريد، كل مجلد، الرفع). |
| **التعرّف** | Tesseract وPoppler في الخلفية، أيضًا لملفات PDF الممسوحة والصور؛ بحث نصي كامل بالبادئات والعبارات والمرشحات (`tag:`، `-tag:`، `type:`، `created:>2026-06-01` …) |
| **التنظيم** | وسوم ملوّنة، **مجلدات من الوسوم** (متداخلة)، **قواعد الوسم** («يحتوي على Northwind → فاتورة»)، **خصائص** مخصصة (نص، رقم، تاريخ، نعم/لا، اختيار، اختيار متعدد، مستخدم، رابط مستند) |
| **الخط الزمني** | خط زمني عمودي بجانب المستندات: اليوم في الأعلى، تحته كل يوم مع البريد، والفراغات الهادئة بينها («12 يومًا بدون مستندات»). تصفية حسب الوسم والنوع والنص. |
| **التذكيرات** | «ذكّرني بعد 10 أيام» على أي مستند، مع ملاحظة؛ تظهر التذكيرات المستحقة في الشريط الجانبي وأعلى الخط الزمني. |
| **الطباعة** | اطبع المستندات على أي **طابعة شبكة IPP** (AirPrint وIPP Everywhere وMopria وCUPS) يجدها الخادم. تُرسل ملفات PDF مباشرة، أو كصور صفحات للطابعات التي تقبل JPEG فقط. |
| **الذكاء الاصطناعي، أي مزوّد** | OpenRouter (نماذج مجانية مدمجة)، OpenAI، Anthropic، Google Gemini، Mistral، Groq، xAI، DeepSeek، Together، Fireworks، Perplexity، Cohere، NVIDIA، Hugging Face، Qwen، Azure OpenAI، **Ollama وLM Studio وvLLM/LocalAI** أو أي عنوان متوافق مع OpenAI. تحصل النماذج النصية على النص المتعرَّف عليه بدلًا من الصور. |
| **Dieter** | مساعد محادثة (أسفل اليمين): *«ما رقم تأميني؟»*، *«كل كشوف الرواتب من Fabrikam في 2026»*. يبحث ويقرأ المقاطع المطابقة ويجيب مع روابط للمستندات. |
| **الفرق** | عدة مؤسسات، **عدة حسابات لكل مؤسسة** (دعوة، أدوار: مالك، مسؤول، عضو؛ نقل الملكية)، مفاتيح API بصلاحيات، ويب هوكس (توقيعات Standard Webhooks)، روابط مشاركة عامة |
| **اللغات** | English, Deutsch, Français, Italiano, Español, Português, Nederlands (المزيد من اللغات قريبًا) |
| **آمن افتراضيًا** | ملف مشفّر واحد، كلمات مرور Argon2، CSP بدون سكربتات مضمّنة، حماية CSRF، الملفات عبر API فقط |

## لقطات الشاشة

| | |
|---|---|
| ![Dieter، المساعد](screenshots/dieter.png) | ![الوارد مع اقتراحات الذكاء الاصطناعي](screenshots/inbox.png) |
| **Dieter** يجيب مع المصادر | **الوارد**: مراجعة وموافقة، تصفية حسب المصدر |
| ![عرض المستند](screenshots/document.png) | ![مزوّدو الذكاء الاصطناعي](screenshots/settings-ai.png) |
| **المستند**: عارض PDF، وسوم، تذكيرات، طباعة | **الذكاء الاصطناعي والنماذج**: كل مزوّد، محلي أو سحابي |
| ![التذكيرات](screenshots/reminders.png) | ![الأعضاء](screenshots/members.png) |
| **التذكيرات** | **الأعضاء** والدعوات |

## البدء السريع

تحتاج إلى Docker. تحتوي الصورة على البرنامج وواجهة الويب وTesseract (الألمانية والإنجليزية) وPoppler.

```bash
git clone https://github.com/nicorenezweers/nrzPaper.git
cd nrzPaper
docker build -t nrzpaper .
docker run -d --name nrzpaper -p 8320:8320 -v nrzpaper-data:/data nrzpaper
```

افتح <http://localhost:8320>. يصبح أول حساب تنشئه هو المسؤول. بعد ذلك يُغلق التسجيل حتى تفتحه في *الإعدادات → التسجيل والمستخدمون*، حيث يمكنك أيضًا إنشاء حسابات للآخرين.

### Docker Compose

```yaml
services:
  nrzpaper:
    build: .                      # أو: image: ghcr.io/nicorenezweers/nrz-paper:1.0.0
    container_name: nrzpaper
    restart: unless-stopped
    ports:
      - "8320:8320"
    volumes:
      - nrzpaper-data:/data       # قاعدة البيانات المشفّرة + ملف المفتاح
      - /srv/scans:/scans         # اختياري: مجلد للوارد من المجلدات
    environment:
      TZ: Europe/Zurich
      # NRZPAPER_DB_KEY: "a-long-secret"     # وإلا يُنشأ /data/nrzpaper.key
      # NRZPAPER_COOKIE_SECURE: "true"       # عند التقديم عبر HTTPS
      # NRZPAPER_AI_PROVIDER: "openai"       # أو اضبطه في واجهة الويب
      # NRZPAPER_AI_KEY: "sk-…"

volumes:
  nrzpaper-data:
```

```bash
docker compose up -d
```

> **انسخ احتياطيًا `/data`.** يحتوي على `nrzpaper.db` (كل شيء، مشفّرًا) و`nrzpaper.key`. بدون المفتاح لا يمكن فتح قاعدة البيانات. إذا ضبطت `NRZPAPER_DB_KEY`، فاحفظ هذه القيمة بأمان بدلًا من ذلك.

### كلتا المعماريتين

```bash
docker buildx build --platform linux/amd64,linux/arm64 -t <registry>/nrzpaper:1.0.0 --push .
```

يُترجم Rust ترجمةً متقاطعة لا محاكاةً، لذا بناء arm64 سريع على جهاز x86 والعكس صحيح.

## الخطوات الأولى

1. **ارفع** بعض ملفات PDF أو الصور: يُتعرّف على النص في الخلفية.
2. **الوسوم**: أنشئ وسومًا مثل *فاتورة* أو *تأمين* وأضف وصفًا. يستخدمه الذكاء الاصطناعي لتعيين الوسوم.
3. **الذكاء الاصطناعي** (*الإعدادات → الذكاء الاصطناعي والنماذج*): اختر مزوّدًا، والصق مفتاحًا (أو استخدم نماذج OpenRouter المجانية المدمجة أو Ollama محليًا)، ثم *تحميل النماذج* و*اختبار الاتصال*.
4. **الماسح** (*الملف الشخصي → الماسح*): *البحث في الشبكة* يجد ماسحات eSCL/AirScan؛ ثم *مسح* في الشريط الجانبي.
5. **البريد الإلكتروني** (*الملف الشخصي → وارد البريد*): أضف صندوق بريد IMAP واحدًا أو أكثر؛ تصل ملفات PDF والصور من المرفقات إلى الوارد.
6. **وارد المجلدات** (*الإعدادات → وارد المجلدات*، المسؤولون): راقب مجلد خادم مثل `/scans` (اربطه في الحاوية أولًا).
7. **الطابعات** (*الإعدادات → الطابعات*، المسؤولون): *البحث في الشبكة* أو أدخل `ipp://printer/ipp/print`؛ بعدها يمكن لكل عضو الطباعة من صفحة المستند.

## الإعدادات

يمكن إعطاء كل إعداد كمتغير بيئة بالبادئة `NRZPAPER_` (أو في `Rocket.toml`).

| المتغير | الافتراضي | المعنى |
|---|---|---|
| `NRZPAPER_DATA_DIR` | `/data` (الصورة) | قاعدة البيانات وملف المفتاح ومجلدات الاستقبال |
| `NRZPAPER_DB_KEY` | – | مفتاح قاعدة البيانات؛ إن لم يُضبط يُنشأ `nrzpaper.key` في مجلد البيانات |
| `NRZPAPER_PORT` / `NRZPAPER_ADDRESS` | `8320` / `0.0.0.0` | حيث يستمع الخادم |
| `NRZPAPER_COOKIE_SECURE` | `false` | اضبطه على `true` خلف HTTPS |
| `NRZPAPER_REGISTRATION_OPEN` | `false` | السماح بالتسجيل الذاتي (يمكن تبديله في الواجهة أيضًا) |
| `NRZPAPER_OCR_LANGUAGES` | `deu+eng` | لغات Tesseract (تحتوي الصورة على `deu` و`eng`) |
| `NRZPAPER_OCR_WORKERS` | `2` | مهام OCR المتوازية |
| `NRZPAPER_TRASH_RETENTION_DAYS` | `30` | أيام في سلة المهملات قبل الحذف النهائي |
| `NRZPAPER_INBOX_KEEP_DAYS` | `7` | أيام الاحتفاظ بعناصر الوارد غير الموافَق عليها |
| `NRZPAPER_SCANNER_URL` | – | ماسح مُعدّ مسبقًا للمستخدمين الجدد، مثل `http://192.168.1.50` |
| `NRZPAPER_SCAN_SUBNETS` | `192.168.0.0/24,192.168.1.0/24` | الشبكات التي يُبحث فيها عن الماسحات والطابعات |
| `NRZPAPER_AI_PROVIDER` | `openrouter` | `openai`، `anthropic`، `gemini`، `mistral`، `groq`، `ollama`، `custom` … |
| `NRZPAPER_AI_KEY` / `NRZPAPER_AI_URL` / `NRZPAPER_AI_MODEL` | – | المفتاح وعنوان URL الأساسي والنموذج لذلك المزوّد |
| `NRZPAPER_OPENROUTER_KEY` | – | مفتاح OpenRouter (بديل عمّا سبق) |
| `NRZPAPER_LOG` | `info` | مرشح السجل، مثل `debug` |

الإعدادات التي تُجرى في واجهة الويب (مزوّد الذكاء الاصطناعي، المفاتيح، النماذج، التسجيل) تُخزَّن في قاعدة البيانات المشفّرة ولها الأولوية على متغيرات البيئة.

### مزوّدو الذكاء الاصطناعي

| المزوّد | العنوان | ملاحظات |
|---|---|---|
| OpenRouter | مدمج | نماذج رؤية مجانية؛ المفتاح الخاص اختياري |
| OpenAI وAnthropic وGoogle Gemini وMistral وGroq وxAI وDeepSeek وTogether وFireworks وPerplexity وCohere وNVIDIA وHugging Face وQwen | مدمج | مفتاح API من المزوّد |
| Azure OpenAI | `https://NAME.openai.azure.com/openai/v1` | النموذج = اسم النشر |
| Ollama / LM Studio / vLLM / LocalAI | `http://host:11434/v1` إلخ | لا حاجة لمفتاح |
| أي شيء متوافق مع OpenAI | عنوانك | مفتاح اختياري |

النماذج التي لا تقرأ الصور: ضع علامة على *«النموذج لا يقرأ الصور – أرسل النص المتعرَّف عليه»*. المزوّدون الذين لا يقبلون PDF يحصلون تلقائيًا على الصفحات الأولى كصور. لغة العناوين والملخصات المقترحة تُضبط في *لغة الاقتراحات*.

## واجهة REST API

أنشئ مفتاح API في *الملف الشخصي → مفاتيح API* (اختر الصلاحيات). كل نقاط النهاية تحت `/api/v1` وتقبل `Authorization: Bearer np_…`.

```bash
TOKEN=np_your_key
BASE=http://localhost:8320/api/v1

# المؤسسات
curl -H "Authorization: Bearer $TOKEN" $BASE/organizations

# الرفع (عدة ملفات دفعة واحدة بعدة -F)
curl -H "Authorization: Bearer $TOKEN" -F file=@invoice.pdf \
     $BASE/organizations/$ORG/documents

# البحث: نفس الصياغة كما في الواجهة
curl -H "Authorization: Bearer $TOKEN" -G $BASE/organizations/$ORG/documents \
     --data-urlencode 'searchQuery=northwind tag:invoice created:2026'

# تنزيل الملف
curl -H "Authorization: Bearer $TOKEN" -o out.pdf \
     "$BASE/organizations/$ORG/documents/$DOC/file?download=true"

# إضافة وسم
curl -H "Authorization: Bearer $TOKEN" -H "Content-Type: application/json" \
     -d '{"tagId":"'$TAG'"}' $BASE/organizations/$ORG/documents/$DOC/tags

# رابط مشاركة عام صالح لمدة 7 أيام
curl -H "Authorization: Bearer $TOKEN" -H "Content-Type: application/json" \
     -d '{"expiresInDays":7}' $BASE/organizations/$ORG/documents/$DOC/share-links

# فحص الحالة (لا حاجة لمفتاح)
curl $BASE/health
```

الويب هوكس (*الإعدادات → الويب هوكس*) ترسل JSON موقّعًا (`webhook-id`، `webhook-timestamp`، `webhook-signature`) لـ `document:created` و`document:updated` و`document:deleted` و`document:restored` و`document:tag:added` و`document:tag:removed`.

مجلد الاستقبال `data/ingestion/<organization-id>/` يستورد الملفات مباشرة كمستندات. المجلدات المراقبة (*وارد المجلدات*) ترسلها أولًا إلى الوارد للمراجعة.

## التطوير

```bash
# الخادم الخلفي (Rust 1.85+ وTesseract وPoppler لـ OCR)
cargo run                         # http://localhost:8320
cargo test

# واجهة الويب مع إعادة التحميل الفوري (dx = dioxus-cli 0.7)
rustup target add wasm32-unknown-unknown
cargo install dioxus-cli --version 0.7.10 --locked
cd ui && dx serve                 # http://localhost:8080، يمرر /api إلى :8320

# تضمين الواجهة في البرنامج
scripts/ui.sh && cargo build --release
```

توجد الترجمات في `locales/<language>.json` (النص الألماني المصدر → الترجمة). يحدّث `scripts/i18n_extract.py` ملف `locales/strings.json` ويبلّغ عن المدخلات الناقصة لكل لغة.

تفاصيل المعمارية: [ARCHITECTURE.md](../ARCHITECTURE.md).

## الترخيص

AGPL-3.0-or-later. مكوّنات الواجهة مبنية على [rust-ui](https://github.com/rust-ui/ui) (MIT)، وعارض PDF على [PDF.js](https://mozilla.github.io/pdf.js/) (Apache-2.0)، والخط هو [Inter](https://rsms.me/inter/) (OFL).
