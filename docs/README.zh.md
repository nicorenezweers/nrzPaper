<div align="center">

<img src="../static/icon.png" width="96" alt="nrzPaper">

# nrzPaper

**基于 Rust 的自托管文档管理：扫描、识别、标记并找到你的文件，全部加密在一个文件中。**

[English](../README.md) · [Deutsch](README.de.md) · [Français](README.fr.md) · [Italiano](README.it.md) · [Español](README.es.md) · [Português](README.pt.md) · [Nederlands](README.nl.md) · [中文](README.zh.md) · [日本語](README.ja.md) · [Polski](README.pl.md) · [Русский](README.ru.md) · [العربية](README.ar.md) · [한국어](README.ko.md) · [Türkçe](README.tr.md) · [Bahasa Indonesia](README.id.md) · [हिन्दी](README.hi.md) · [Tiếng Việt](README.vi.md) · [ไทย](README.th.md) · [Українська](README.uk.md) · [Čeština](README.cs.md) · [Română](README.ro.md) · [Magyar](README.hu.md) · [Ελληνικά](README.el.md) · [Svenska](README.sv.md) · [Norsk](README.nb.md) · [Dansk](README.da.md) · [Suomi](README.fi.md) · [עברית](README.he.md) · [বাংলা](README.bn.md) · [فارسی](README.fa.md)

![带时间线的文档](screenshots/documents.png)

</div>

nrzPaper 是一个无需外部数据库的单一程序。所有内容都存放在一个由 SQLCipher 加密的 SQLite 文件中，文件本身也包含在内。它使用 Tesseract 进行 OCR，直接从网络扫描仪扫描，获取电子邮件附件，并让你选择的 AI 建议标题、标签和元数据。内置助手 **Dieter** 会回答有关你文档的问题并链接来源。网页界面是一个采用平静深色主题的 Rust/WebAssembly 应用（Dioxus），在手机上也能使用。

## 功能

| | |
|---|---|
| **采集** | 拖放、上传、**网络扫描仪**（eSCL/AirScan：Canon、HP、Epson、Brother …）、**电子邮箱**（IMAP，多个账户）、**监视文件夹**（例如 NAS 上的扫描文件夹）、REST API |
| **收件箱** | 扫描件、邮件、文件夹导入和上传等待审核：AI 建议名称、标签和字段，你可以逐项或**一次性全部**批准。按来源筛选（扫描仪、每个邮箱、每个文件夹、上传）。 |
| **识别** | 后台运行 Tesseract 和 Poppler，也支持扫描版 PDF 和照片；全文搜索支持前缀、短语和筛选器（`tag:`、`-tag:`、`type:`、`created:>2026-06-01` 等） |
| **整理** | 彩色标签、**由标签生成的文件夹**（可嵌套）、**标记规则**（“包含 Northwind → 发票”）、自定义**属性**（文本、数字、日期、是/否、单选、多选、用户、文档链接） |
| **时间线** | 文档旁的垂直时间线：今天在最上方，下面是每一天的邮件，以及之间的安静空白（“12 天没有文档”）。按标签、类型和文本筛选。 |
| **提醒** | 可为任意文档设置“10 天后提醒我”并附备注；到期的提醒显示在侧边栏和时间线顶部。 |
| **打印** | 可在服务器发现的任意 **IPP 网络打印机**（AirPrint、IPP Everywhere、Mopria、CUPS）上打印。PDF 直接发送，或对仅接受 JPEG 的打印机以页面图像发送。 |
| **AI，任何提供商** | OpenRouter（内置免费模型）、OpenAI、Anthropic、Google Gemini、Mistral、Groq、xAI、DeepSeek、Together、Fireworks、Perplexity、Cohere、NVIDIA、Hugging Face、Qwen、Azure OpenAI、**Ollama、LM Studio、vLLM/LocalAI** 或任何兼容 OpenAI 的 URL。纯文本模型接收 OCR 文本而非图像。 |
| **Dieter** | 聊天助手（右下角）：*“我的保险号是多少？”*、*“2026 年 Fabrikam 的所有工资单”*。他会搜索、阅读匹配段落并用文档链接回答。 |
| **团队** | 多个组织、**每个组织多个账户**（邀请、角色：所有者、管理员、成员；转移所有权）、带权限的 API 密钥、Webhook（Standard Webhooks 签名）、公开分享链接 |
| **语言** | English、Deutsch、Français、Italiano、Español、Português、Nederlands、中文、日本語、Polski、Русский、العربية、한국어、Türkçe、Bahasa Indonesia、हिन्दी、Tiếng Việt、ไทย、Українська、Čeština、Română、Magyar、Ελληνικά、Svenska、Norsk、Dansk、Suomi、עברית、বাংলা、فارسی（更多见 `locales/<code>.json`） |
| **默认安全** | 一个加密文件、Argon2 密码、无内联脚本的 CSP、CSRF 防护、文件仅通过 API 访问 |

## 截图

| | |
|---|---|
| ![Dieter，助手](screenshots/dieter.png) | ![带 AI 建议的收件箱](screenshots/inbox.png) |
| **Dieter** 带来源回答 | **收件箱**：审核并批准，按来源筛选 |
| ![文档视图](screenshots/document.png) | ![AI 提供商](screenshots/settings-ai.png) |
| **文档**：PDF 查看器、标签、提醒、打印 | **AI 与模型**：任何提供商，本地或云端 |
| ![提醒](screenshots/reminders.png) | ![成员](screenshots/members.png) |
| **提醒** | **成员**和邀请 |

## 快速开始

你需要 Docker。镜像包含程序、网页界面、Tesseract（德语、英语）和 Poppler。

```bash
git clone https://github.com/nicorenezweers/nrzPaper.git
cd nrzPaper
docker build -t nrzpaper .
docker run -d --name nrzpaper -p 8320:8320 -v nrzpaper-data:/data nrzpaper
```

打开 <http://localhost:8320>。你创建的第一个账户将成为管理员。之后注册将关闭，直到你在*设置 → 注册与用户*中开启，在那里你也可以为他人创建账户。

### Docker Compose

```yaml
services:
  nrzpaper:
    build: .                      # 或: image: ghcr.io/nicorenezweers/nrzpaper:1.0.0
    container_name: nrzpaper
    restart: unless-stopped
    ports:
      - "8320:8320"
    volumes:
      - nrzpaper-data:/data       # 加密数据库 + 密钥文件
      - /srv/scans:/scans         # 可选：文件夹收件箱的文件夹
    environment:
      TZ: Europe/Zurich
      # NRZPAPER_DB_KEY: "a-long-secret"     # 否则会创建 /data/nrzpaper.key
      # NRZPAPER_COOKIE_SECURE: "true"       # 通过 HTTPS 提供服务时
      # NRZPAPER_AI_PROVIDER: "openai"       # 或在网页界面中设置
      # NRZPAPER_AI_KEY: "sk-…"

volumes:
  nrzpaper-data:
```

```bash
docker compose up -d
```

> **备份 `/data`。** 其中包含 `nrzpaper.db`（所有内容，已加密）和 `nrzpaper.key`。没有密钥就无法打开数据库。如果你设置了 `NRZPAPER_DB_KEY`，请妥善保管该值。

### 两种架构

```bash
docker buildx build --platform linux/amd64,linux/arm64 -t <registry>/nrzpaper:1.0.0 --push .
```

Rust 采用交叉编译而非模拟，因此 arm64 构建在 x86 机器上很快，反之亦然。

## 第一步

1. **上传**一些 PDF 或照片：文本将在后台识别。
2. **标签**：创建如*发票*或*保险*等标签并添加描述。AI 会据此分配标签。
3. **AI**（*设置 → AI 与模型*）：选择提供商，粘贴密钥（或使用内置的免费 OpenRouter 模型，或本地 Ollama），然后*加载模型*并*测试连接*。
4. **扫描仪**（*个人资料 → 扫描仪*）：*搜索网络*可找到 eSCL/AirScan 扫描仪；然后在侧边栏中*扫描*。
5. **电子邮件**（*个人资料 → 电子邮箱*）：添加一个或多个 IMAP 邮箱；附件中的 PDF 和图像会进入收件箱。
6. **文件夹收件箱**（*设置 → 文件夹收件箱*，管理员）：监视服务器文件夹如 `/scans`（先将它挂载到容器中）。
7. **打印机**（*设置 → 打印机*，管理员）：*搜索网络*或输入 `ipp://printer/ipp/print`；之后每位成员都可以从文档页面打印。

## 配置

每项设置都可以通过带前缀 `NRZPAPER_` 的环境变量（或在 `Rocket.toml` 中）指定。

| 变量 | 默认值 | 含义 |
|---|---|---|
| `NRZPAPER_DATA_DIR` | `/data`（镜像） | 数据库、密钥文件和导入文件夹 |
| `NRZPAPER_DB_KEY` | – | 数据库密钥；如果未设置，将在数据文件夹中生成 `nrzpaper.key` |
| `NRZPAPER_PORT` / `NRZPAPER_ADDRESS` | `8320` / `0.0.0.0` | 服务器监听地址 |
| `NRZPAPER_COOKIE_SECURE` | `false` | 在 HTTPS 后设为 `true` |
| `NRZPAPER_REGISTRATION_OPEN` | `false` | 允许自行注册（也可在界面中切换） |
| `NRZPAPER_OCR_LANGUAGES` | `deu+eng` | Tesseract 语言（镜像自带 `deu` 和 `eng`） |
| `NRZPAPER_OCR_WORKERS` | `2` | 并行 OCR 任务数 |
| `NRZPAPER_TRASH_RETENTION_DAYS` | `30` | 回收站中的保留天数 |
| `NRZPAPER_INBOX_KEEP_DAYS` | `7` | 未批准的收件箱项目保留天数 |
| `NRZPAPER_SCANNER_URL` | – | 为新用户预设的扫描仪，例如 `http://192.168.1.50` |
| `NRZPAPER_SCAN_SUBNETS` | `192.168.0.0/24,192.168.1.0/24` | 搜索扫描仪和打印机的网络 |
| `NRZPAPER_AI_PROVIDER` | `openrouter` | `openai`、`anthropic`、`gemini`、`mistral`、`groq`、`ollama`、`custom` 等 |
| `NRZPAPER_AI_KEY` / `NRZPAPER_AI_URL` / `NRZPAPER_AI_MODEL` | – | 该提供商的密钥、基础 URL 和模型 |
| `NRZPAPER_OPENROUTER_KEY` | – | OpenRouter 密钥（上述的替代） |
| `NRZPAPER_LOG` | `info` | 日志过滤器，例如 `debug` |

在网页界面中进行的设置（AI 提供商、密钥、模型、注册）存储在加密数据库中，并优先于环境变量。

### AI 提供商

| 提供商 | 地址 | 说明 |
|---|---|---|
| OpenRouter | 内置 | 免费视觉模型；可选自己的密钥 |
| OpenAI、Anthropic、Google Gemini、Mistral、Groq、xAI、DeepSeek、Together、Fireworks、Perplexity、Cohere、NVIDIA、Hugging Face、Qwen | 内置 | 提供商的 API 密钥 |
| Azure OpenAI | `https://NAME.openai.azure.com/openai/v1` | 模型 = 部署名称 |
| Ollama / LM Studio / vLLM / LocalAI | `http://host:11434/v1` 等 | 无需密钥 |
| 任何兼容 OpenAI 的服务 | 你的 URL | 密钥可选 |

无法读取图像的模型：勾选*“模型无法读取图像 – 发送已识别的文本”*。不接受 PDF 的提供商自动获得前几页作为图像。建议标题和摘要的语言在*建议语言*下设置。

## REST API

在*个人资料 → API 密钥*下创建 API 密钥（选择权限）。所有端点位于 `/api/v1` 下，并接受 `Authorization: Bearer np_…`。

```bash
TOKEN=np_your_key
BASE=http://localhost:8320/api/v1

# 组织
curl -H "Authorization: Bearer $TOKEN" $BASE/organizations

# 上传（使用多个 -F 一次上传多个文件）
curl -H "Authorization: Bearer $TOKEN" -F file=@invoice.pdf \
     $BASE/organizations/$ORG/documents

# 搜索：与界面中相同的语法
curl -H "Authorization: Bearer $TOKEN" -G $BASE/organizations/$ORG/documents \
     --data-urlencode 'searchQuery=northwind tag:invoice created:2026'

# 下载文件
curl -H "Authorization: Bearer $TOKEN" -o out.pdf \
     "$BASE/organizations/$ORG/documents/$DOC/file?download=true"

# 添加标签
curl -H "Authorization: Bearer $TOKEN" -H "Content-Type: application/json" \
     -d '{"tagId":"'$TAG'"}' $BASE/organizations/$ORG/documents/$DOC/tags

# 有效期 7 天的公开分享链接
curl -H "Authorization: Bearer $TOKEN" -H "Content-Type: application/json" \
     -d '{"expiresInDays":7}' $BASE/organizations/$ORG/documents/$DOC/share-links

# 健康检查（无需密钥）
curl $BASE/health
```

Webhook（*设置 → Webhook*）为 `document:created`、`document:updated`、`document:deleted`、`document:restored`、`document:tag:added` 和 `document:tag:removed` 发送签名的 JSON（`webhook-id`、`webhook-timestamp`、`webhook-signature`）。

导入文件夹 `data/ingestion/<organization-id>/` 直接将文件作为文档导入。监视文件夹（*文件夹收件箱*）先将它们发送到收件箱以供审核。

## 开发

```bash
# 后端（Rust 1.85+、用于 OCR 的 Tesseract 和 Poppler）
cargo run                         # http://localhost:8320
cargo test

# 带热重载的网页界面（dx = dioxus-cli 0.7）
rustup target add wasm32-unknown-unknown
cargo install dioxus-cli --version 0.7.10 --locked
cd ui && dx serve                 # http://localhost:8080，将 /api 代理到 :8320

# 将界面嵌入程序
scripts/ui.sh && cargo build --release
```

翻译位于 `locales/<language>.json`（德语源文本 → 翻译）。`scripts/i18n_extract.py` 刷新 `locales/strings.json` 并报告每种语言缺失的条目。

架构详情：[ARCHITECTURE.md](../ARCHITECTURE.md)。

## 许可证

AGPL-3.0-or-later。UI 组件基于 [rust-ui](https://github.com/rust-ui/ui)（MIT），PDF 查看器基于 [PDF.js](https://mozilla.github.io/pdf.js/)（Apache-2.0），字体为 [Inter](https://rsms.me/inter/)（OFL）。
