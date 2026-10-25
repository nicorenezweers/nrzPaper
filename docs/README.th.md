<div align="center">

<img src="../static/icon.png" width="96" alt="nrzPaper">

# nrzPaper

**การจัดการเอกสารแบบโฮสต์เองด้วย Rust: สแกน รู้จำ ติดแท็ก และค้นหาเอกสารของคุณ — เข้ารหัสในไฟล์เดียว**

[English](../README.md) · [Deutsch](README.de.md) · [Français](README.fr.md) · [Italiano](README.it.md) · [Español](README.es.md) · [Português](README.pt.md) · [Nederlands](README.nl.md) · [中文](README.zh.md) · [日本語](README.ja.md) · [Polski](README.pl.md) · [Русский](README.ru.md) · [العربية](README.ar.md) · [한국어](README.ko.md) · [Türkçe](README.tr.md) · [Bahasa Indonesia](README.id.md) · [हिन्दी](README.hi.md) · [Tiếng Việt](README.vi.md) · [ไทย](README.th.md) · [Українська](README.uk.md) · [Čeština](README.cs.md) · [Română](README.ro.md) · [Magyar](README.hu.md) · [Ελληνικά](README.el.md) · [Svenska](README.sv.md) · [Norsk](README.nb.md) · [Dansk](README.da.md) · [Suomi](README.fi.md) · [עברית](README.he.md) · [বাংলা](README.bn.md) · [فارسی](README.fa.md)

![เอกสารพร้อมไทม์ไลน์](screenshots/documents.png)

</div>

nrzPaper เป็นโปรแกรมเดียวที่ไม่มีฐานข้อมูลภายนอก ทุกอย่างอยู่ในไฟล์ SQLite เดียวที่เข้ารหัสด้วย SQLCipher รวมถึงตัวไฟล์เองด้วย ทำงาน OCR ด้วย Tesseract สแกน直接从เครื่องสแกนเครือข่าย ดึงไฟล์แนบอีเมล และให้ AI ที่คุณเลือกเสนอชื่อ แท็ก และเมทาดาทา ผู้ช่วยในตัว **Dieter** ตอบคำถามเกี่ยวกับเอกสารของคุณและเชื่อมโยงแหล่งที่มา อินเทอร์เฟซเว็บเป็นแอป Rust/WebAssembly (Dioxus) ในธีมมืดที่สงบ และใช้งานได้บนโทรศัพท์

## คุณสมบัติ

| | |
|---|---|
| **การเก็บ** | ลากและวาง อัปโหลด **เครื่องสแกนเครือข่าย** (eSCL/AirScan: Canon, HP, Epson, Brother …) **กล่องอีเมล** (IMAP หลายบัญชี) **โฟลเดอร์ที่เฝ้าดู** (เช่น โฟลเดอร์สแกนบน NAS) REST API |
| **กล่องขาเข้า** | การสแกน อีเมล การนำเข้าโฟลเดอร์ และการอัปโหลดรอการตรวจสอบ: AI เสนอชื่อ แท็ก และฟิลด์ และคุณอนุมัติทีละรายการหรือ**ทั้งหมดพร้อมกัน** กรองตามแหล่งที่มา (เครื่องสแกน แต่ละกล่องจดหมาย แต่ละโฟลเดอร์ การอัปโหลด) |
| **การรู้จำ** | Tesseract และ Poppler ทำงานเบื้องหลัง รวมถึง PDF ที่สแกนและรูปถ่าย; ค้นหาข้อความเต็มด้วยคำนำ วลี และตัวกรอง (`tag:`, `-tag:`, `type:`, `created:>2026-06-01`, …) |
| **การจัดระเบียบ** | แท็กสี **โฟลเดอร์ที่สร้างจากแท็ก** (ซ้อนกัน) **กฎการติดแท็ก** ("มี Northwind → ใบแจ้งหนี้") **คุณสมบัติ** ที่กำหนดเอง (ข้อความ ตัวเลข วันที่ ใช่/ไม่ใช่ เลือก เลือกหลายค่า ผู้ใช้ ลิงก์เอกสาร) |
| **ไทม์ไลน์** | ไทม์ไลน์แนวตั้งข้างเอกสาร: วันนี้อยู่บนสุด ด้านล่างเป็นแต่ละวันพร้อมอีเมล และช่องว่างที่เงียบสงบระหว่างนั้น ("12 วันไม่มีเอกสาร") กรองตามแท็ก ประเภท และข้อความ |
| **การเตือน** | "เตือนฉันใน 10 วัน" บนเอกสารใดก็ได้ พร้อมบันทึก; การเตือนที่ครบกำหนดแสดงในแถบด้านข้างและด้านบนของไทม์ไลน์ |
| **การพิมพ์** | พิมพ์เอกสารบน**เครื่องพิมพ์เครือข่าย IPP** ใดก็ได้ (AirPrint, IPP Everywhere, Mopria, CUPS) ที่เซิร์ฟเวอร์ค้นพบ PDF ส่งโดยตรง หรือเป็นภาพหน้าไปยังเครื่องพิมพ์ที่รับเฉพาะ JPEG |
| **AI ผู้ให้บริการใดก็ได้** | OpenRouter (มีโมเดลฟรีในตัว), OpenAI, Anthropic, Google Gemini, Mistral, Groq, xAI, DeepSeek, Together, Fireworks, Perplexity, Cohere, NVIDIA, Hugging Face, Qwen, Azure OpenAI, **Ollama, LM Studio, vLLM/LocalAI** หรือ URL ใดก็ตามที่เข้ากันได้กับ OpenAI โมเดลข้อความล้วนจะได้รับข้อความ OCR แทนรูปภาพ |
| **Dieter** | ผู้ช่วยแชท (มุมขวาล่าง): *"หมายเลขประกันของฉันคืออะไร?"*, *"สลิปเงินเดือนทั้งหมดจาก Fabrikam ในปี 2026"* เขาค้นหา อ่านข้อความที่ตรงกัน และตอบพร้อมลิงก์ไปยังเอกสาร |
| **ทีม** | หลายองค์กร **หลายบัญชีต่อองค์กร** (เชิญ บทบาท: เจ้าของ ผู้ดูแล สมาชิก; โอนความเป็นเจ้าของ) คีย์ API พร้อมสิทธิ์ เว็บฮุก (ลายเซ็น Standard Webhooks) ลิงก์แชร์สาธารณะ |
| **ภาษา** | English, Deutsch, Français, Italiano, Español, Português, Nederlands (ภาษาอื่น ๆ เร็ว ๆ นี้) |
| **ปลอดภัยโดยค่าเริ่มต้น** | ไฟล์เข้ารหัสเดียว รหัสผ่าน Argon2 CSP โดยไม่มีสคริปต์ในบรรทัด การป้องกัน CSRF ไฟล์เข้าถึงได้เฉพาะผ่าน API |

## ภาพหน้าจอ

| | |
|---|---|
| ![Dieter ผู้ช่วย](screenshots/dieter.png) | ![กล่องขาเข้าพร้อมข้อเสนอแนะ AI](screenshots/inbox.png) |
| **Dieter** ตอบพร้อมแหล่งที่มา | **กล่องขาเข้า**: ตรวจสอบและอนุมัติ กรองตามแหล่งที่มา |
| ![มุมมองเอกสาร](screenshots/document.png) | ![ผู้ให้บริการ AI](screenshots/settings-ai.png) |
| **เอกสาร**: โปรแกรมดู PDF แท็ก การเตือน การพิมพ์ | **AI และโมเดล**: ทุกผู้ให้บริการ ทั้งในเครื่องหรือคลาวด์ |
| ![การเตือน](screenshots/reminders.png) | ![สมาชิก](screenshots/members.png) |
| **การเตือน** | **สมาชิก** และคำเชิญ |

## เริ่มต้นอย่างรวดเร็ว

คุณต้องมี Docker อิมเมจมีโปรแกรม อินเทอร์เฟซเว็บ Tesseract (เยอรมัน อังกฤษ) และ Poppler

```bash
git clone https://github.com/nicorenezweers/nrzPaper.git
cd nrzPaper
docker build -t nrzpaper .
docker run -d --name nrzpaper -p 8320:8320 -v nrzpaper-data:/data nrzpaper
```

เปิด <http://localhost:8320> บัญชีแรกที่คุณสร้างจะกลายเป็นผู้ดูแลระบบ หลังจากนั้นการลงทะเบียนจะปิดจนกว่าคุณจะเปิดใน *การตั้งค่า → การลงทะเบียนและผู้ใช้* ซึ่งคุณสามารถสร้างบัญชีให้ผู้อื่นได้ด้วย

### Docker Compose

```yaml
services:
  nrzpaper:
    build: .                      # หรือ: image: ghcr.io/nicorenezweers/nrzpaper:1.0.0
    container_name: nrzpaper
    restart: unless-stopped
    ports:
      - "8320:8320"
    volumes:
      - nrzpaper-data:/data       # ฐานข้อมูลที่เข้ารหัส + ไฟล์คีย์
      - /srv/scans:/scans         # ไม่บังคับ: โฟลเดอร์สำหรับกล่องขาเข้าจากโฟลเดอร์
    environment:
      TZ: Europe/Zurich
      # NRZPAPER_DB_KEY: "a-long-secret"     # มิฉะนั้นจะสร้าง /data/nrzpaper.key
      # NRZPAPER_COOKIE_SECURE: "true"       # เมื่อให้บริการผ่าน HTTPS
      # NRZPAPER_AI_PROVIDER: "openai"       # หรือตั้งค่าในอินเทอร์เฟซเว็บ
      # NRZPAPER_AI_KEY: "sk-…"

volumes:
  nrzpaper-data:
```

```bash
docker compose up -d
```

> **สำรองข้อมูล `/data`** มี `nrzpaper.db` (ทุกอย่าง เข้ารหัส) และ `nrzpaper.key` หากไม่มีคีย์จะเปิดฐานข้อมูลไม่ได้ หากคุณตั้ง `NRZPAPER_DB_KEY` ให้เก็บค่านั้นอย่างปลอดภัยแทน

### ทั้งสองสถาปัตยกรรม

```bash
docker buildx build --platform linux/amd64,linux/arm64 -t <registry>/nrzpaper:1.0.0 --push .
```

Rust ถูกคอมไพล์ข้ามแพลตฟอร์ม ไม่ได้จำลอง ดังนั้นการสร้าง arm64 จึงเร็วบนเครื่อง x86 และในทางกลับกัน

## ขั้นตอนแรก

1. **อัปโหลด** PDF หรือรูปถ่ายสองสามไฟล์: ข้อความจะถูกรู้จำเบื้องหลัง
2. **แท็ก**: สร้างแท็กเช่น *ใบแจ้งหนี้* หรือ *ประกัน* และเพิ่มคำอธิบาย AI ใช้คำอธิบายเพื่อกำหนดแท็ก
3. **AI** (*การตั้งค่า → AI และโมเดล*): เลือกผู้ให้บริการ วางคีย์ (หรือใช้โมเดล OpenRouter ฟรีในตัว หรือ Ollama ในเครื่อง) จากนั้น *โหลดโมเดล* และ *ทดสอบการเชื่อมต่อ*
4. **เครื่องสแกน** (*โปรไฟล์ → เครื่องสแกน*): *ค้นหาในเครือข่าย* จะพบเครื่องสแกน eSCL/AirScan; จากนั้น *สแกน* ในแถบด้านข้าง
5. **อีเมล** (*โปรไฟล์ → กล่องขาเข้าอีเมล*): เพิ่มกล่องจดหมาย IMAP หนึ่งกล่องขึ้นไป; PDF และรูปภาพจากไฟล์แนบจะเข้าสู่กล่องขาเข้า
6. **กล่องขาเข้าจากโฟลเดอร์** (*การตั้งค่า → กล่องขาเข้าจากโฟลเดอร์*, ผู้ดูแล): เฝ้าดูโฟลเดอร์เซิร์ฟเวอร์เช่น `/scans` (เมานต์ลงในคอนเทนเนอร์ก่อน)
7. **เครื่องพิมพ์** (*การตั้งค่า → เครื่องพิมพ์*, ผู้ดูแล): *ค้นหาในเครือข่าย* หรือป้อน `ipp://printer/ipp/print`; จากนั้นสมาชิกทุกคนสามารถพิมพ์จากหน้าเอกสารได้

## การกำหนดค่า

ทุกการตั้งค่าสามารถกำหนดเป็นตัวแปรสภาพแวดล้อมที่มีคำนำหน้า `NRZPAPER_` (หรือใน `Rocket.toml`)

| ตัวแปร | ค่าเริ่มต้น | ความหมาย |
|---|---|---|
| `NRZPAPER_DATA_DIR` | `/data` (อิมเมจ) | ฐานข้อมูล ไฟล์คีย์ และโฟลเดอร์นำเข้า |
| `NRZPAPER_DB_KEY` | – | คีย์ฐานข้อมูล; หากไม่ได้ตั้ง จะสร้าง `nrzpaper.key` ในโฟลเดอร์ข้อมูล |
| `NRZPAPER_PORT` / `NRZPAPER_ADDRESS` | `8320` / `0.0.0.0` | ที่เซิร์ฟเวอร์รับฟัง |
| `NRZPAPER_COOKIE_SECURE` | `false` | ตั้งเป็น `true` หลัง HTTPS |
| `NRZPAPER_REGISTRATION_OPEN` | `false` | อนุญาตการลงทะเบียนเอง (สลับได้ใน UI) |
| `NRZPAPER_OCR_LANGUAGES` | `deu+eng` | ภาษา Tesseract (อิมเมจมี `deu` และ `eng`) |
| `NRZPAPER_OCR_WORKERS` | `2` | งาน OCR แบบขนาน |
| `NRZPAPER_TRASH_RETENTION_DAYS` | `30` | จำนวนวันในถังขยะก่อนลบถาวร |
| `NRZPAPER_INBOX_KEEP_DAYS` | `7` | จำนวนวันที่เก็บรายการกล่องขาเข้าที่ยังไม่อนุมัติ |
| `NRZPAPER_SCANNER_URL` | – | เครื่องสแกนที่ตั้งไว้ล่วงหน้าสำหรับผู้ใช้ใหม่ เช่น `http://192.168.1.50` |
| `NRZPAPER_SCAN_SUBNETS` | `192.168.0.0/24,192.168.1.0/24` | เครือข่ายที่ค้นหาเครื่องสแกนและเครื่องพิมพ์ |
| `NRZPAPER_AI_PROVIDER` | `openrouter` | `openai`, `anthropic`, `gemini`, `mistral`, `groq`, `ollama`, `custom`, … |
| `NRZPAPER_AI_KEY` / `NRZPAPER_AI_URL` / `NRZPAPER_AI_MODEL` | – | คีย์ URL ฐาน และโมเดลสำหรับผู้ให้บริการนั้น |
| `NRZPAPER_OPENROUTER_KEY` | – | คีย์ OpenRouter (ทางเลือกแทนข้างต้น) |
| `NRZPAPER_LOG` | `info` | ตัวกรองบันทึก เช่น `debug` |

การตั้งค่าในอินเทอร์เฟซเว็บ (ผู้ให้บริการ AI คีย์ โมเดล การลงทะเบียน) ถูกเก็บในฐานข้อมูลที่เข้ารหัสและมีความสำคัญเหนือสภาพแวดล้อม

### ผู้ให้บริการ AI

| ผู้ให้บริการ | ที่อยู่ | หมายเหตุ |
|---|---|---|
| OpenRouter | ในตัว | โมเดลวิชันฟรี; คีย์ของคุณเองไม่บังคับ |
| OpenAI, Anthropic, Google Gemini, Mistral, Groq, xAI, DeepSeek, Together, Fireworks, Perplexity, Cohere, NVIDIA, Hugging Face, Qwen | ในตัว | คีย์ API จากผู้ให้บริการ |
| Azure OpenAI | `https://NAME.openai.azure.com/openai/v1` | โมเดล = ชื่อการปรับใช้ |
| Ollama / LM Studio / vLLM / LocalAI | `http://host:11434/v1` เป็นต้น | ไม่ต้องใช้คีย์ |
| สิ่งใดก็ตามที่เข้ากันได้กับ OpenAI | URL ของคุณ | คีย์ไม่บังคับ |

โมเดลที่ไม่สามารถอ่านรูปภาพ: ติ๊ก *"โมเดลอ่านรูปภาพไม่ได้ – ส่งข้อความที่รู้จำแล้ว"* ผู้ให้บริการที่ไม่รับ PDF จะได้รับหน้าแรกเป็นรูปภาพโดยอัตโนมัติ ภาษาของชื่อและบทสรุปที่เสนอตั้งค่าใน *ภาษาของข้อเสนอแนะ*

## REST API

สร้างคีย์ API ใน *โปรไฟล์ → คีย์ API* (เลือกสิทธิ์) ปลายทางทั้งหมดอยู่ใต้ `/api/v1` และรับ `Authorization: Bearer np_…`

```bash
TOKEN=np_your_key
BASE=http://localhost:8320/api/v1

# องค์กร
curl -H "Authorization: Bearer $TOKEN" $BASE/organizations

# อัปโหลด (หลายไฟล์พร้อมกันด้วยหลาย -F)
curl -H "Authorization: Bearer $TOKEN" -F file=@invoice.pdf \
     $BASE/organizations/$ORG/documents

# ค้นหา: ไวยากรณ์เดียวกับใน UI
curl -H "Authorization: Bearer $TOKEN" -G $BASE/organizations/$ORG/documents \
     --data-urlencode 'searchQuery=northwind tag:invoice created:2026'

# ดาวน์โหลดไฟล์
curl -H "Authorization: Bearer $TOKEN" -o out.pdf \
     "$BASE/organizations/$ORG/documents/$DOC/file?download=true"

# เพิ่มแท็ก
curl -H "Authorization: Bearer $TOKEN" -H "Content-Type: application/json" \
     -d '{"tagId":"'$TAG'"}' $BASE/organizations/$ORG/documents/$DOC/tags

# ลิงก์แชร์สาธารณะใช้ได้ 7 วัน
curl -H "Authorization: Bearer $TOKEN" -H "Content-Type: application/json" \
     -d '{"expiresInDays":7}' $BASE/organizations/$ORG/documents/$DOC/share-links

# ตรวจสอบสถานะ (ไม่ต้องใช้คีย์)
curl $BASE/health
```

เว็บฮุก (*การตั้งค่า → เว็บฮุก*) ส่ง JSON ที่ลงลายเซ็น (`webhook-id`, `webhook-timestamp`, `webhook-signature`) สำหรับ `document:created`, `document:updated`, `document:deleted`, `document:restored`, `document:tag:added` และ `document:tag:removed`

โฟลเดอร์นำเข้า `data/ingestion/<organization-id>/` นำเข้าไฟล์เป็นเอกสารโดยตรง โฟลเดอร์ที่เฝ้าดู (*กล่องขาเข้าจากโฟลเดอร์*) ส่งไปยังกล่องขาเข้าเพื่อตรวจสอบก่อน

## การพัฒนา

```bash
# แบ็กเอนด์ (Rust 1.85+, Tesseract และ Poppler สำหรับ OCR)
cargo run                         # http://localhost:8320
cargo test

# อินเทอร์เฟซเว็บพร้อมโหลดใหม่ทันที (dx = dioxus-cli 0.7)
rustup target add wasm32-unknown-unknown
cargo install dioxus-cli --version 0.7.10 --locked
cd ui && dx serve                 # http://localhost:8080, พร็อกซี /api ไปยัง :8320

# ฝังอินเทอร์เฟซลงในโปรแกรม
scripts/ui.sh && cargo build --release
```

คำแปลอยู่ใน `locales/<language>.json` (ข้อความต้นฉบับภาษาเยอรมัน → คำแปล) `scripts/i18n_extract.py` รีเฟรช `locales/strings.json` และรายงานรายการที่ขาดหายต่อภาษา

รายละเอียดสถาปัตยกรรม: [ARCHITECTURE.md](../ARCHITECTURE.md)

## สัญญาอนุญาต

AGPL-3.0-or-later คอมโพเนนต์ UI อ้างอิงจาก [rust-ui](https://github.com/rust-ui/ui) (MIT) โปรแกรมดู PDF จาก [PDF.js](https://mozilla.github.io/pdf.js/) (Apache-2.0) ฟอนต์คือ [Inter](https://rsms.me/inter/) (OFL)
