<div align="center" dir="rtl">

<img src="../static/icon.png" width="96" alt="nrzPaper">

# nrzPaper

**ניהול מסמכים באחסון עצמי ב-Rust: סרוק, זהה, תייג ומצא את המסמכים שלך — מוצפן בקובץ אחד.**

[English](../README.md) · [Deutsch](README.de.md) · [Français](README.fr.md) · [Italiano](README.it.md) · [Español](README.es.md) · [Português](README.pt.md) · [Nederlands](README.nl.md) · [中文](README.zh.md) · [日本語](README.ja.md) · [Polski](README.pl.md) · [Русский](README.ru.md) · [العربية](README.ar.md) · [한국어](README.ko.md) · [Türkçe](README.tr.md) · [Bahasa Indonesia](README.id.md) · [हिन्दी](README.hi.md) · [Tiếng Việt](README.vi.md) · [ไทย](README.th.md) · [Українська](README.uk.md) · [Čeština](README.cs.md) · [Română](README.ro.md) · [Magyar](README.hu.md) · [Ελληνικά](README.el.md) · [Svenska](README.sv.md) · [Norsk](README.nb.md) · [Dansk](README.da.md) · [Suomi](README.fi.md) · [עברית](README.he.md) · [বাংলা](README.bn.md) · [فارسی](README.fa.md)

![מסמכים עם ציר זמן](screenshots/documents.png)

</div>

nrzPaper הוא תוכנית אחת בלי מסד נתונים חיצוני. הכול נמצא בקובץ SQLite אחד מוצפן ב-SQLCipher, כולל הקבצים עצמם. הוא מבצע OCR עם Tesseract, סורק ישירות מסורקי רשת, אוסף קבצים מצורפים מדואר אלקטרוני, ומאפשר ל-AI שתבחר להציע כותרות, תגיות ומטא-נתונים. העוזר המובנה **Dieter** עונה על שאלות לגבי המסמכים שלך ומקשר למקורות. ממשק האינטרנט הוא אפליקציית Rust/WebAssembly (Dioxus) בעיצוב כהה רגוע ועובד גם בטלפונים.

## תכונות

| | |
|---|---|
| **לכידה** | גרירה ושחרור, העלאה, **סורקי רשת** (eSCL/AirScan: Canon, HP, Epson, Brother …), **תיבות דואר** (IMAP, כמה חשבונות), **תיקיות במעקב** (למשל תיקיית הסריקה ב-NAS), REST API |
| **דואר נכנס** | סריקות, הודעות, ייבוא מתיקיות והעלאות ממתינים לבדיקה: ה-AI מציע שם, תגיות ושדות, ואתה מאשר פריט אחד או **הכול בבת אחת**. סינון לפי מקור (סורק, כל תיבת דואר, כל תיקייה, העלאה). |
| **זיהוי** | Tesseract ו-Poppler ברקע, גם עבור PDF סרוקים ותמונות; חיפוש טקסט מלא עם קידומות, ביטויים ומסננים (`tag:`, `-tag:`, `type:`, `created:>2026-06-01`, …) |
| **ארגון** | תגיות צבעוניות, **תיקיות מתגיות** (מקוננות), **כללי תיוג** ("מכיל Northwind → חשבונית"), **מאפיינים** מותאמים (טקסט, מספר, תאריך, כן/לא, בחירה, בחירה מרובה, משתמש, קישור למסמך) |
| **ציר זמן** | ציר זמן אנכי לצד המסמכים: היום למעלה, מתחתיו כל יום עם דואר והפערים השקטים שביניהם ("12 ימים בלי מסמכים"). סינון לפי תגית, סוג וטקסט. |
| **תזכורות** | "הזכר לי בעוד 10 ימים" בכל מסמך, עם הערה; תזכורות שהגיע זמנן מופיעות בסרגל הצד ובראש ציר הזמן. |
| **הדפסה** | הדפס מסמכים בכל **מדפסת רשת IPP** (AirPrint, IPP Everywhere, Mopria, CUPS) שהשרת מוצא. PDF נשלחים ישירות, או כתמונות עמודים למדפסות שמקבלות רק JPEG. |
| **AI, כל ספק** | OpenRouter (מודלים חינמיים מובנים), OpenAI, Anthropic, Google Gemini, Mistral, Groq, xAI, DeepSeek, Together, Fireworks, Perplexity, Cohere, NVIDIA, Hugging Face, Qwen, Azure OpenAI, **Ollama, LM Studio, vLLM/LocalAI** או כל כתובת תואמת OpenAI. מודלים טקסטואליים מקבלים את הטקסט שזוהה במקום תמונות. |
| **Dieter** | עוזר צ'אט (למטה מימין): *"מה מספר הביטוח שלי?"*, *"כל תלושי השכר מ-Fabrikam ב-2026"*. הוא מחפש, קורא את הקטעים המתאימים ועונה עם קישורים למסמכים. |
| **צוותים** | כמה ארגונים, **כמה חשבונות לכל ארגון** (הזמנה, תפקידים: בעלים, מנהל, חבר; העברת בעלות), מפתחות API עם הרשאות, webhooks (חתימות Standard Webhooks), קישורי שיתוף ציבוריים |
| **שפות** | English, Deutsch, Français, Italiano, Español, Português, Nederlands (שפות נוספות בקרוב) |
| **מאובטח כברירת מחדל** | קובץ מוצפן אחד, סיסמאות Argon2, CSP בלי סקריפטים מוטמעים, הגנת CSRF, קבצים רק דרך ה-API |

## צילומי מסך

| | |
|---|---|
| ![Dieter, העוזר](screenshots/dieter.png) | ![דואר נכנס עם הצעות AI](screenshots/inbox.png) |
| **Dieter** עונה עם מקורות | **דואר נכנס**: בדיקה ואישור, סינון לפי מקור |
| ![תצוגת מסמך](screenshots/document.png) | ![ספקי AI](screenshots/settings-ai.png) |
| **מסמך**: מציג PDF, תגיות, תזכורות, הדפסה | **AI ומודלים**: כל ספק, מקומי או בענן |
| ![תזכורות](screenshots/reminders.png) | ![חברים](screenshots/members.png) |
| **תזכורות** | **חברים** והזמנות |

## התחלה מהירה

נדרש Docker. התמונה מכילה את התוכנית, ממשק האינטרנט, Tesseract (גרמנית, אנגלית) ו-Poppler.

```bash
git clone https://github.com/nicorenezweers/nrzPaper.git
cd nrzPaper
docker build -t nrzpaper .
docker run -d --name nrzpaper -p 8320:8320 -v nrzpaper-data:/data nrzpaper
```

פתח <http://localhost:8320>. החשבון הראשון שתיצור הופך למנהל. לאחר מכן ההרשמה סגורה עד שתפתח אותה ב-*הגדרות → הרשמה ומשתמשים*, שם גם תוכל ליצור חשבונות לאחרים.

### Docker Compose

```yaml
services:
  nrzpaper:
    build: .                      # או: image: ghcr.io/nicorenezweers/nrzpaper:1.0.0
    container_name: nrzpaper
    restart: unless-stopped
    ports:
      - "8320:8320"
    volumes:
      - nrzpaper-data:/data       # מסד נתונים מוצפן + קובץ מפתח
      - /srv/scans:/scans         # אופציונלי: תיקייה לדואר הנכנס מתיקיות
    environment:
      TZ: Europe/Zurich
      # NRZPAPER_DB_KEY: "a-long-secret"     # אחרת נוצר /data/nrzpaper.key
      # NRZPAPER_COOKIE_SECURE: "true"       # כשמוגש דרך HTTPS
      # NRZPAPER_AI_PROVIDER: "openai"       # או הגדר בממשק האינטרנט
      # NRZPAPER_AI_KEY: "sk-…"

volumes:
  nrzpaper-data:
```

```bash
docker compose up -d
```

> **גבה את `/data`.** הוא מכיל את `nrzpaper.db` (הכול, מוצפן) ואת `nrzpaper.key`. בלי המפתח לא ניתן לפתוח את מסד הנתונים. אם הגדרת `NRZPAPER_DB_KEY`, שמור את הערך הזה במקום בטוח.

### שתי הארכיטקטורות

```bash
docker buildx build --platform linux/amd64,linux/arm64 -t <registry>/nrzpaper:1.0.0 --push .
```

Rust מקומפלcross-compile ולא באמולציה, כך שבניית arm64 מהירה במכונת x86 ולהפך.

## צעדים ראשונים

1. **העלה** כמה PDF או תמונות: הטקסט מזוהה ברקע.
2. **תגיות**: צור תגיות כמו *חשבונית* או *ביטוח* והוסף תיאור. ה-AI משתמש בו כדי להקצות תגיות.
3. **AI** (*הגדרות → AI ומודלים*): בחר ספק, הדבק מפתח (או השתמש במודלים החינמיים המובנים של OpenRouter, או ב-Ollama מקומי), ואז *טען מודלים* ו-*בדוק חיבור*.
4. **סורק** (*פרופיל → סורק*): *חפש ברשת* מוצא סורקי eSCL/AirScan; ואז *סרוק* בסרגל הצד.
5. **דואר אלקטרוני** (*פרופיל → דואר נכנס*): הוסף תיבת IMAP אחת או יותר; PDF ותמונות מקבצים מצורפים מגיעים לדואר הנכנס.
6. **דואר נכנס מתיקיות** (*הגדרות → דואר נכנס מתיקיות*, מנהלים): עקוב אחר תיקיית שרת כמו `/scans` (תחילה חבר אותה לקונטיינר).
7. **מדפסות** (*הגדרות → מדפסות*, מנהלים): *חפש ברשת* או הזן `ipp://printer/ipp/print`; לאחר מכן כל חבר יכול להדפיס מדף המסמך.

## תצורה

כל הגדרה יכולה להינתן כמשתנה סביבה עם הקידומת `NRZPAPER_` (או ב-`Rocket.toml`).

| משתנה | ברירת מחדל | משמעות |
|---|---|---|
| `NRZPAPER_DATA_DIR` | `/data` (תמונה) | מסד נתונים, קובץ מפתח ותיקיות קליטה |
| `NRZPAPER_DB_KEY` | – | מפתח מסד הנתונים; אם לא הוגדר, נוצר `nrzpaper.key` בתיקיית הנתונים |
| `NRZPAPER_PORT` / `NRZPAPER_ADDRESS` | `8320` / `0.0.0.0` | היכן השרת מאזין |
| `NRZPAPER_COOKIE_SECURE` | `false` | הגדר `true` מאחורי HTTPS |
| `NRZPAPER_REGISTRATION_OPEN` | `false` | אפשר הרשמה עצמית (ניתן להחליף גם בממשק) |
| `NRZPAPER_OCR_LANGUAGES` | `deu+eng` | שפות Tesseract (התמונה כוללת `deu` ו-`eng`) |
| `NRZPAPER_OCR_WORKERS` | `2` | משימות OCR מקבילות |
| `NRZPAPER_TRASH_RETENTION_DAYS` | `30` | ימים באשפה לפני מחיקה סופית |
| `NRZPAPER_INBOX_KEEP_DAYS` | `7` | ימים שבהם נשמרים פריטי דואר נכנס שלא אושרו |
| `NRZPAPER_SCANNER_URL` | – | סורק מוגדר מראש למשתמשים חדשים, למשל `http://192.168.1.50` |
| `NRZPAPER_SCAN_SUBNETS` | `192.168.0.0/24,192.168.1.0/24` | רשתות לחיפוש סורקים ומדפסות |
| `NRZPAPER_AI_PROVIDER` | `openrouter` | `openai`, `anthropic`, `gemini`, `mistral`, `groq`, `ollama`, `custom`, … |
| `NRZPAPER_AI_KEY` / `NRZPAPER_AI_URL` / `NRZPAPER_AI_MODEL` | – | מפתח, כתובת בסיס ומודל לאותו ספק |
| `NRZPAPER_OPENROUTER_KEY` | – | מפתח OpenRouter (חלופה לאמור לעיל) |
| `NRZPAPER_LOG` | `info` | מסנן לוגים, למשל `debug` |

הגדרות שנעשות בממשק האינטרנט (ספק AI, מפתחות, מודלים, הרשמה) נשמרות במסד הנתונים המוצפן וגוברות על משתני הסביבה.

### ספקי AI

| ספק | כתובת | הערות |
|---|---|---|
| OpenRouter | מובנה | מודלי ראייה חינמיים; מפתח עצמי אופציונלי |
| OpenAI, Anthropic, Google Gemini, Mistral, Groq, xAI, DeepSeek, Together, Fireworks, Perplexity, Cohere, NVIDIA, Hugging Face, Qwen | מובנה | מפתח API מהספק |
| Azure OpenAI | `https://NAME.openai.azure.com/openai/v1` | מודל = שם הפריסה |
| Ollama / LM Studio / vLLM / LocalAI | `http://host:11434/v1` וכו' | אין צורך במפתח |
| כל דבר תואם OpenAI | הכתובת שלך | מפתח אופציונלי |

מודלים שלא קוראים תמונות: סמן *"המודל לא קורא תמונות – שלח טקסט שזוהה"*. ספקים שלא מקבלים PDF מקבלים אוטומטית את העמודים הראשונים כתמונות. שפת הכותרות והתקצירים המוצעים נקבעת ב-*שפת ההצעות*.

## REST API

צור מפתח API ב-*פרופיל → מפתחות API* (בחר הרשאות). כל נקודות הקצה נמצאות תחת `/api/v1` ומקבלות `Authorization: Bearer np_…`.

```bash
TOKEN=np_your_key
BASE=http://localhost:8320/api/v1

# ארגונים
curl -H "Authorization: Bearer $TOKEN" $BASE/organizations

# העלאה (כמה קבצים בבת אחת עם כמה -F)
curl -H "Authorization: Bearer $TOKEN" -F file=@invoice.pdf \
     $BASE/organizations/$ORG/documents

# חיפוש: אותה תחביר כמו בממשק
curl -H "Authorization: Bearer $TOKEN" -G $BASE/organizations/$ORG/documents \
     --data-urlencode 'searchQuery=northwind tag:invoice created:2026'

# הורדת הקובץ
curl -H "Authorization: Bearer $TOKEN" -o out.pdf \
     "$BASE/organizations/$ORG/documents/$DOC/file?download=true"

# הוספת תגית
curl -H "Authorization: Bearer $TOKEN" -H "Content-Type: application/json" \
     -d '{"tagId":"'$TAG'"}' $BASE/organizations/$ORG/documents/$DOC/tags

# קישור שיתוף ציבורי בתוקף ל-7 ימים
curl -H "Authorization: Bearer $TOKEN" -H "Content-Type: application/json" \
     -d '{"expiresInDays":7}' $BASE/organizations/$ORG/documents/$DOC/share-links

# בדיקת תקינות (אין צורך במפתח)
curl $BASE/health
```

Webhooks (*הגדרות → Webhooks*) שולחים JSON חתום (`webhook-id`, `webhook-timestamp`, `webhook-signature`) עבור `document:created`, `document:updated`, `document:deleted`, `document:restored`, `document:tag:added` ו-`document:tag:removed`.

תיקיית הקליטה `data/ingestion/<organization-id>/` מייבאת קבצים ישירות כמסמכים. תיקיות במעקב (*דואר נכנס מתיקיות*) שולחות אותם תחילה לדואר הנכנס לבדיקה.

## פיתוח

```bash
# backend (Rust 1.85+, Tesseract ו-Poppler ל-OCR)
cargo run                         # http://localhost:8320
cargo test

# ממשק אינטרנט עם טעינה חמה (dx = dioxus-cli 0.7)
rustup target add wasm32-unknown-unknown
cargo install dioxus-cli --version 0.7.10 --locked
cd ui && dx serve                 # http://localhost:8080, מפנה /api ל-:8320

# הטמעת הממשק בתוכנית
scripts/ui.sh && cargo build --release
```

תרגומים נמצאים ב-`locales/<language>.json` (טקסט מקור גרמני → תרגום). `scripts/i18n_extract.py` מרענן את `locales/strings.json` ומדווח על ערכים חסרים לכל שפה.

פרטי ארכיטקטורה: [ARCHITECTURE.md](../ARCHITECTURE.md).

## רישיון

AGPL-3.0-or-later. רכיבי הממשק מבוססים על [rust-ui](https://github.com/rust-ui/ui) (MIT), מציג ה-PDF על [PDF.js](https://mozilla.github.io/pdf.js/) (Apache-2.0), והגופן הוא [Inter](https://rsms.me/inter/) (OFL).
