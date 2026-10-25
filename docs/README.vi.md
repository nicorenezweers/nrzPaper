<div align="center">

<img src="../static/icon.png" width="96" alt="nrzPaper">

# nrzPaper

**A Rust-powered document management tool.**

[English](../README.md) · [Deutsch](README.de.md) · [Français](README.fr.md) · [Italiano](README.it.md) · [Español](README.es.md) · [Português](README.pt.md) · [Nederlands](README.nl.md) · [中文](README.zh.md) · [日本語](README.ja.md) · [Polski](README.pl.md) · [Русский](README.ru.md) · [العربية](README.ar.md) · [한국어](README.ko.md) · [Türkçe](README.tr.md) · [Bahasa Indonesia](README.id.md) · [हिन्दी](README.hi.md) · [Tiếng Việt](README.vi.md) · [ไทย](README.th.md) · [Українська](README.uk.md) · [Čeština](README.cs.md) · [Română](README.ro.md) · [Magyar](README.hu.md) · [Ελληνικά](README.el.md) · [Svenska](README.sv.md) · [Norsk](README.nb.md) · [Dansk](README.da.md) · [Suomi](README.fi.md) · [עברית](README.he.md) · [বাংলা](README.bn.md) · [فارسی](README.fa.md)

![Tài liệu với dòng thời gian](screenshots/documents.png)

</div>

nrzPaper là một chương trình duy nhất không có cơ sở dữ liệu bên ngoài. Mọi thứ nằm trong một tệp SQLite được mã hóa bằng SQLCipher, kể cả bản thân các tệp. Nó thực hiện OCR bằng Tesseract, quét trực tiếp từ máy quét mạng, lấy tệp đính kèm email và cho phép AI bạn chọn đề xuất tiêu đề, thẻ và siêu dữ liệu. Trợ lý tích hợp **Dieter** trả lời câu hỏi về tài liệu của bạn và liên kết nguồn. Giao diện web là ứng dụng Rust/WebAssembly (Dioxus) với chủ đề tối dịu mắt và cũng hoạt động trên điện thoại.

## Tính năng

| | |
|---|---|
| **Thu thập** | Kéo & thả, tải lên, **máy quét mạng** (eSCL/AirScan: Canon, HP, Epson, Brother …), **hộp thư email** (IMAP, nhiều tài khoản), **thư mục được theo dõi** (ví dụ thư mục quét trên NAS), REST API |
| **Hộp thư đến** | Bản quét, email, nhập từ thư mục và tải lên chờ xem xét: AI đề xuất tên, thẻ và trường, bạn phê duyệt từng mục hoặc **tất cả cùng lúc**. Lọc theo nguồn (máy quét, từng hộp thư, từng thư mục, tải lên). |
| **Nhận dạng** | Tesseract và Poppler chạy nền, cả cho PDF quét và ảnh; tìm kiếm toàn văn với tiền tố, cụm từ và bộ lọc (`tag:`, `-tag:`, `type:`, `created:>2026-06-01`, …) |
| **Tổ chức** | Thẻ màu, **thư mục tạo từ thẻ** (lồng nhau), **quy tắc gắn thẻ** ("chứa Northwind → Hóa đơn"), **thuộc tính** tùy chỉnh (văn bản, số, ngày, có/không, chọn, chọn nhiều, người dùng, liên kết tài liệu) |
| **Dòng thời gian** | Dòng thời gian dọc bên cạnh tài liệu: hôm nay ở trên cùng, bên dưới mỗi ngày có email và những khoảng lặng ở giữa ("12 ngày không có tài liệu"). Lọc theo thẻ, loại và văn bản. |
| **Nhắc nhở** | "Nhắc tôi sau 10 ngày" trên bất kỳ tài liệu nào, kèm ghi chú; nhắc nhở đến hạn hiển thị ở thanh bên và đầu dòng thời gian. |
| **In** | In tài liệu trên bất kỳ **máy in mạng IPP** nào (AirPrint, IPP Everywhere, Mopria, CUPS) mà máy chủ tìm thấy. PDF được gửi trực tiếp, hoặc dưới dạng ảnh trang cho máy in chỉ nhận JPEG. |
| **AI, mọi nhà cung cấp** | OpenRouter (mô hình miễn phí tích hợp), OpenAI, Anthropic, Google Gemini, Mistral, Groq, xAI, DeepSeek, Together, Fireworks, Perplexity, Cohere, NVIDIA, Hugging Face, Qwen, Azure OpenAI, **Ollama, LM Studio, vLLM/LocalAI** hoặc bất kỳ URL tương thích OpenAI. Mô hình chỉ văn bản nhận văn bản OCR thay vì hình ảnh. |
| **Dieter** | Trợ lý trò chuyện (góc dưới bên phải): *"Số bảo hiểm của tôi là gì?"*, *"Tất cả phiếu lương từ Fabrikam năm 2026"*. Anh ấy tìm kiếm, đọc các đoạn phù hợp và trả lời kèm liên kết đến tài liệu. |
| **Nhóm** | Nhiều tổ chức, **nhiều tài khoản mỗi tổ chức** (mời, vai trò: chủ sở hữu, quản trị viên, thành viên; chuyển quyền sở hữu), khóa API có quyền, webhook (chữ ký Standard Webhooks), liên kết chia sẻ công khai |
| **Ngôn ngữ** | English, Deutsch, Français, Italiano, Español, Português, Nederlands (các ngôn ngữ khác sắp ra mắt) |
| **An toàn mặc định** | Một tệp mã hóa, mật khẩu Argon2, CSP không có script nội tuyến, bảo vệ CSRF, tệp chỉ qua API |

## Ảnh chụp màn hình

| | |
|---|---|
| ![Dieter, trợ lý](screenshots/dieter.png) | ![Hộp thư đến với gợi ý AI](screenshots/inbox.png) |
| **Dieter** trả lời kèm nguồn | **Hộp thư đến**: xem xét và phê duyệt, lọc theo nguồn |
| ![Xem tài liệu](screenshots/document.png) | ![Nhà cung cấp AI](screenshots/settings-ai.png) |
| **Tài liệu**: trình xem PDF, thẻ, nhắc nhở, in | **AI & mô hình**: mọi nhà cung cấp, cục bộ hoặc đám mây |
| ![Nhắc nhở](screenshots/reminders.png) | ![Thành viên](screenshots/members.png) |
| **Nhắc nhở** | **Thành viên** và lời mời |

## Bắt đầu nhanh

Bạn cần Docker. Image chứa chương trình, giao diện web, Tesseract (Đức, Anh) và Poppler.

```bash
git clone https://github.com/nicorenezweers/nrzPaper.git
cd nrzPaper
docker build -t nrzpaper .
docker run -d --name nrzpaper -p 8320:8320 -v nrzpaper-data:/data nrzpaper
```

Mở <http://localhost:8320>. Tài khoản đầu tiên bạn tạo sẽ trở thành quản trị viên. Sau đó đăng ký bị đóng cho đến khi bạn mở nó trong *Cài đặt → Đăng ký & người dùng*, nơi bạn cũng có thể tạo tài khoản cho người khác.

### Docker Compose

```yaml
services:
  nrzpaper:
    build: .                      # hoặc: image: ghcr.io/nicorenezweers/nrzpaper:1.0.0
    container_name: nrzpaper
    restart: unless-stopped
    ports:
      - "8320:8320"
    volumes:
      - nrzpaper-data:/data       # cơ sở dữ liệu mã hóa + tệp khóa
      - /srv/scans:/scans         # tùy chọn: thư mục cho hộp thư đến từ thư mục
    environment:
      TZ: Europe/Zurich
      # NRZPAPER_DB_KEY: "a-long-secret"     # nếu không, /data/nrzpaper.key được tạo
      # NRZPAPER_COOKIE_SECURE: "true"       # khi phục vụ qua HTTPS
      # NRZPAPER_AI_PROVIDER: "openai"       # hoặc đặt trong giao diện web
      # NRZPAPER_AI_KEY: "sk-…"

volumes:
  nrzpaper-data:
```

```bash
docker compose up -d
```

> **Sao lưu `/data`.** Nơi đây chứa `nrzpaper.db` (mọi thứ, đã mã hóa) và `nrzpaper.key`. Không có khóa thì không thể mở cơ sở dữ liệu. Nếu bạn đặt `NRZPAPER_DB_KEY`, hãy giữ giá trị đó an toàn.

### Cả hai kiến trúc

```bash
docker buildx build --platform linux/amd64,linux/arm64 -t <registry>/nrzpaper:1.0.0 --push .
```

Rust được biên dịch chéo chứ không mô phỏng, nên bản dựng arm64 nhanh trên máy x86 và ngược lại.

## Những bước đầu tiên

1. **Tải lên** một vài PDF hoặc ảnh: văn bản được nhận dạng ở chế độ nền.
2. **Thẻ**: tạo thẻ như *Hóa đơn* hoặc *Bảo hiểm* và thêm mô tả. AI dùng nó để gán thẻ.
3. **AI** (*Cài đặt → AI & mô hình*): chọn nhà cung cấp, dán khóa (hoặc dùng mô hình OpenRouter miễn phí tích hợp, hoặc Ollama cục bộ), rồi *Tải mô hình* và *Kiểm tra kết nối*.
4. **Máy quét** (*Hồ sơ → Máy quét*): *Tìm kiếm mạng* tìm máy quét eSCL/AirScan; rồi *Quét* ở thanh bên.
5. **Email** (*Hồ sơ → Hộp thư email*): thêm một hoặc nhiều hộp thư IMAP; PDF và ảnh từ tệp đính kèm vào hộp thư đến.
6. **Hộp thư đến từ thư mục** (*Cài đặt → Hộp thư đến từ thư mục*, quản trị viên): theo dõi thư mục máy chủ như `/scans` (gắn nó vào container trước).
7. **Máy in** (*Cài đặt → Máy in*, quản trị viên): *Tìm kiếm mạng* hoặc nhập `ipp://printer/ipp/print`; sau đó mỗi thành viên có thể in từ trang tài liệu.

## Cấu hình

Mọi cài đặt có thể được đưa vào dưới dạng biến môi trường với tiền tố `NRZPAPER_` (hoặc trong `Rocket.toml`).

| Biến | Mặc định | Ý nghĩa |
|---|---|---|
| `NRZPAPER_DATA_DIR` | `/data` (image) | Cơ sở dữ liệu, tệp khóa và thư mục nhập |
| `NRZPAPER_DB_KEY` | – | Khóa cơ sở dữ liệu; nếu không đặt, `nrzpaper.key` được tạo trong thư mục dữ liệu |
| `NRZPAPER_PORT` / `NRZPAPER_ADDRESS` | `8320` / `0.0.0.0` | Nơi máy chủ lắng nghe |
| `NRZPAPER_COOKIE_SECURE` | `false` | Đặt `true` sau HTTPS |
| `NRZPAPER_REGISTRATION_OPEN` | `false` | Cho phép tự đăng ký (cũng có thể chuyển trong UI) |
| `NRZPAPER_OCR_LANGUAGES` | `deu+eng` | Ngôn ngữ Tesseract (image có sẵn `deu` và `eng`) |
| `NRZPAPER_OCR_WORKERS` | `2` | Tác vụ OCR song song |
| `NRZPAPER_TRASH_RETENTION_DAYS` | `30` | Số ngày trong thùng rác trước khi xóa vĩnh viễn |
| `NRZPAPER_INBOX_KEEP_DAYS` | `7` | Số ngày giữ các mục hộp thư đến chưa phê duyệt |
| `NRZPAPER_SCANNER_URL` | – | Máy quét đặt trước cho người dùng mới, ví dụ `http://192.168.1.50` |
| `NRZPAPER_SCAN_SUBNETS` | `192.168.0.0/24,192.168.1.0/24` | Mạng được tìm kiếm máy quét và máy in |
| `NRZPAPER_AI_PROVIDER` | `openrouter` | `openai`, `anthropic`, `gemini`, `mistral`, `groq`, `ollama`, `custom`, … |
| `NRZPAPER_AI_KEY` / `NRZPAPER_AI_URL` / `NRZPAPER_AI_MODEL` | – | Khóa, URL gốc và mô hình cho nhà cung cấp đó |
| `NRZPAPER_OPENROUTER_KEY` | – | Khóa OpenRouter (thay thế cho mục trên) |
| `NRZPAPER_LOG` | `info` | Bộ lọc log, ví dụ `debug` |

Cài đặt trong giao diện web (nhà cung cấp AI, khóa, mô hình, đăng ký) được lưu trong cơ sở dữ liệu mã hóa và được ưu tiên hơn biến môi trường.

### Nhà cung cấp AI

| Nhà cung cấp | Địa chỉ | Ghi chú |
|---|---|---|
| OpenRouter | tích hợp | Mô hình thị giác miễn phí; khóa riêng tùy chọn |
| OpenAI, Anthropic, Google Gemini, Mistral, Groq, xAI, DeepSeek, Together, Fireworks, Perplexity, Cohere, NVIDIA, Hugging Face, Qwen | tích hợp | Khóa API từ nhà cung cấp |
| Azure OpenAI | `https://NAME.openai.azure.com/openai/v1` | Mô hình = tên triển khai |
| Ollama / LM Studio / vLLM / LocalAI | `http://host:11434/v1` v.v. | Không cần khóa |
| Bất kỳ thứ gì tương thích OpenAI | URL của bạn | Khóa tùy chọn |

Mô hình không đọc được hình ảnh: đánh dấu *"Mô hình không đọc được hình ảnh – gửi văn bản đã nhận dạng"*. Các nhà cung cấp không nhận PDF sẽ tự động nhận các trang đầu dưới dạng hình ảnh. Ngôn ngữ của tiêu đề và tóm tắt đề xuất được đặt trong *Ngôn ngữ gợi ý*.

## REST API

Tạo khóa API trong *Hồ sơ → Khóa API* (chọn quyền). Tất cả endpoint nằm dưới `/api/v1` và chấp nhận `Authorization: Bearer np_…`.

```bash
TOKEN=np_your_key
BASE=http://localhost:8320/api/v1

# tổ chức
curl -H "Authorization: Bearer $TOKEN" $BASE/organizations

# tải lên (nhiều tệp cùng lúc với nhiều -F)
curl -H "Authorization: Bearer $TOKEN" -F file=@invoice.pdf \
     $BASE/organizations/$ORG/documents

# tìm kiếm: cùng cú pháp như trong UI
curl -H "Authorization: Bearer $TOKEN" -G $BASE/organizations/$ORG/documents \
     --data-urlencode 'searchQuery=northwind tag:invoice created:2026'

# tải tệp xuống
curl -H "Authorization: Bearer $TOKEN" -o out.pdf \
     "$BASE/organizations/$ORG/documents/$DOC/file?download=true"

# thêm thẻ
curl -H "Authorization: Bearer $TOKEN" -H "Content-Type: application/json" \
     -d '{"tagId":"'$TAG'"}' $BASE/organizations/$ORG/documents/$DOC/tags

# liên kết chia sẻ công khai có hiệu lực 7 ngày
curl -H "Authorization: Bearer $TOKEN" -H "Content-Type: application/json" \
     -d '{"expiresInDays":7}' $BASE/organizations/$ORG/documents/$DOC/share-links

# kiểm tra tình trạng (không cần khóa)
curl $BASE/health
```

Webhook (*Cài đặt → Webhook*) gửi JSON có chữ ký (`webhook-id`, `webhook-timestamp`, `webhook-signature`) cho `document:created`, `document:updated`, `document:deleted`, `document:restored`, `document:tag:added` và `document:tag:removed`.

Thư mục nhập `data/ingestion/<organization-id>/` nhập tệp trực tiếp thành tài liệu. Thư mục được theo dõi (*Hộp thư đến từ thư mục*) gửi chúng vào hộp thư đến để xem xét trước.

## Phát triển

```bash
# backend (Rust 1.85+, Tesseract và Poppler cho OCR)
cargo run                         # http://localhost:8320
cargo test

# giao diện web với tải lại nóng (dx = dioxus-cli 0.7)
rustup target add wasm32-unknown-unknown
cargo install dioxus-cli --version 0.7.10 --locked
cd ui && dx serve                 # http://localhost:8080, chuyển tiếp /api tới :8320

# nhúng giao diện vào chương trình
scripts/ui.sh && cargo build --release
```

Bản dịch nằm trong `locales/<language>.json` (văn bản nguồn tiếng Đức → bản dịch). `scripts/i18n_extract.py` làm mới `locales/strings.json` và báo cáo các mục còn thiếu theo từng ngôn ngữ.

Chi tiết kiến trúc: [ARCHITECTURE.md](../ARCHITECTURE.md).

## Giấy phép

AGPL-3.0-or-later. Các thành phần UI dựa trên [rust-ui](https://github.com/rust-ui/ui) (MIT), trình xem PDF dựa trên [PDF.js](https://mozilla.github.io/pdf.js/) (Apache-2.0), phông chữ là [Inter](https://rsms.me/inter/) (OFL).
