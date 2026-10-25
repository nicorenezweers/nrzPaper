<div align="center">

<img src="../static/icon.png" width="96" alt="nrzPaper">

# nrzPaper

**Rust 기반 자체 호스팅 문서 관리: 서류를 스캔하고, 인식하고, 태그를 달고, 찾아보세요. 모든 것이 하나의 암호화된 파일에.**

[English](../README.md) · [Deutsch](README.de.md) · [Français](README.fr.md) · [Italiano](README.it.md) · [Español](README.es.md) · [Português](README.pt.md) · [Nederlands](README.nl.md) · [中文](README.zh.md) · [日本語](README.ja.md) · [Polski](README.pl.md) · [Русский](README.ru.md) · [العربية](README.ar.md) · [한국어](README.ko.md) · [Türkçe](README.tr.md) · [Bahasa Indonesia](README.id.md) · [हिन्दी](README.hi.md) · [Tiếng Việt](README.vi.md) · [ไทย](README.th.md) · [Українська](README.uk.md) · [Čeština](README.cs.md) · [Română](README.ro.md) · [Magyar](README.hu.md) · [Ελληνικά](README.el.md) · [Svenska](README.sv.md) · [Norsk](README.nb.md) · [Dansk](README.da.md) · [Suomi](README.fi.md) · [עברית](README.he.md) · [বাংলা](README.bn.md) · [فارسی](README.fa.md)

![타임라인이 있는 문서](screenshots/documents.png)

</div>

nrzPaper는 외부 데이터베이스가 없는 단일 프로그램입니다. 파일 자체를 포함한 모든 것이 SQLCipher로 암호화된 하나의 SQLite 파일에 저장됩니다. Tesseract로 OCR을 수행하고, 네트워크 스캐너에서 직접 스캔하며, 이메일 첨부 파일을 가져오고, 선택한 AI가 제목, 태그, 메타데이터를 제안하도록 합니다. 내장 도우미 **Dieter**는 문서에 대한 질문에 답하고 출처를 연결합니다. 웹 인터페이스는 차분한 다크 테마의 Rust/WebAssembly 앱(Dioxus)이며 휴대폰에서도 작동합니다.

## 기능

| | |
|---|---|
| **수집** | 드래그 앤 드롭, 업로드, **네트워크 스캐너**(eSCL/AirScan: Canon, HP, Epson, Brother …), **이메일 받은편지함**(IMAP, 여러 계정), **감시 폴더**(예: NAS의 스캔 폴더), REST API |
| **받은편지함** | 스캔, 메일, 폴더 가져오기, 업로드가 검토를 기다립니다. AI가 이름, 태그, 필드를 제안하며 하나씩 또는 **한 번에 모두** 승인할 수 있습니다. 출처별(스캐너, 각 사서함, 각 폴더, 업로드)로 필터링합니다. |
| **인식** | 백그라운드에서 Tesseract와 Poppler 실행, 스캔한 PDF와 사진도 지원. 접두사, 구문, 필터(`tag:`, `-tag:`, `type:`, `created:>2026-06-01` 등)를 사용한 전체 텍스트 검색 |
| **정리** | 색상 태그, **태그로 만든 폴더**(중첩 가능), **태그 규칙**("Northwind 포함 → 청구서"), 사용자 지정 **속성**(텍스트, 숫자, 날짜, 예/아니오, 선택, 다중 선택, 사용자, 문서 링크) |
| **타임라인** | 문서 옆의 세로 타임라인: 오늘이 맨 위, 그 아래 각 날짜의 메일, 그 사이의 조용한 공백("문서 없는 12일"). 태그, 유형, 텍스트로 필터링합니다. |
| **알림** | 모든 문서에 메모와 함께 "10일 후 알림" 설정. 기한이 된 알림은 사이드바와 타임라인 상단에 표시됩니다. |
| **인쇄** | 서버가 찾은 모든 **IPP 네트워크 프린터**(AirPrint, IPP Everywhere, Mopria, CUPS)에서 인쇄. PDF는 직접 전송하거나 JPEG만 받는 프린터에는 페이지 이미지로 전송합니다. |
| **AI, 모든 제공업체** | OpenRouter(무료 모델 내장), OpenAI, Anthropic, Google Gemini, Mistral, Groq, xAI, DeepSeek, Together, Fireworks, Perplexity, Cohere, NVIDIA, Hugging Face, Qwen, Azure OpenAI, **Ollama, LM Studio, vLLM/LocalAI** 또는 모든 OpenAI 호환 URL. 텍스트 전용 모델에는 이미지 대신 OCR 텍스트가 전달됩니다. |
| **Dieter** | 채팅 도우미(오른쪽 아래): *"내 보험 번호는?"*, *"2026년 Fabrikam의 모든 급여 명세서"*. 검색하고 일치하는 구절을 읽고 문서 링크와 함께 답합니다. |
| **팀** | 여러 조직, **조직당 여러 계정**(초대, 역할: 소유자, 관리자, 구성원, 소유권 이전), 권한이 있는 API 키, 웹훅(Standard Webhooks 서명), 공개 공유 링크 |
| **언어** | English, Deutsch, Français, Italiano, Español, Português, Nederlands, 中文, 日本語, Polski, Русский, العربية, 한국어, Türkçe, Bahasa Indonesia, हिन्दी, Tiếng Việt, ไทย, Українська, Čeština, Română, Magyar, Ελληνικά, Svenska, Norsk, Dansk, Suomi, עברית, বাংলা, فارسی (더 많은 언어는 `locales/<code>.json`) |
| **기본 보안** | 하나의 암호화된 파일, Argon2 비밀번호, 인라인 스크립트 없는 CSP, CSRF 보호, 파일은 API를 통해서만 접근 |

## 스크린샷

| | |
|---|---|
| ![도우미 Dieter](screenshots/dieter.png) | ![AI 제안이 있는 받은편지함](screenshots/inbox.png) |
| **Dieter**가 출처와 함께 답변 | **받은편지함**: 검토 및 승인, 출처별 필터 |
| ![문서 보기](screenshots/document.png) | ![AI 제공업체](screenshots/settings-ai.png) |
| **문서**: PDF 뷰어, 태그, 알림, 인쇄 | **AI 및 모델**: 모든 제공업체, 로컬 또는 클라우드 |
| ![알림](screenshots/reminders.png) | ![구성원](screenshots/members.png) |
| **알림** | **구성원** 및 초대 |

## 빠른 시작

Docker가 필요합니다. 이미지에는 프로그램, 웹 인터페이스, Tesseract(독일어, 영어), Poppler가 포함되어 있습니다.

```bash
git clone https://github.com/nicorenezweers/nrzPaper.git
cd nrzPaper
docker build -t nrzpaper .
docker run -d --name nrzpaper -p 8320:8320 -v nrzpaper-data:/data nrzpaper
```

<http://localhost:8320>을 엽니다. 처음 만든 계정이 관리자가 됩니다. 이후 *설정 → 등록 및 사용자*에서 등록을 열 때까지 등록이 닫힙니다. 그곳에서 다른 사용자의 계정도 만들 수 있습니다.

### Docker Compose

```yaml
services:
  nrzpaper:
    build: .                      # 또는: image: ghcr.io/nicorenezweers/nrzpaper:1.0.0
    container_name: nrzpaper
    restart: unless-stopped
    ports:
      - "8320:8320"
    volumes:
      - nrzpaper-data:/data       # 암호화된 데이터베이스 + 키 파일
      - /srv/scans:/scans         # 선택 사항: 폴더 받은편지함용 폴더
    environment:
      TZ: Europe/Zurich
      # NRZPAPER_DB_KEY: "a-long-secret"     # 그렇지 않으면 /data/nrzpaper.key가 생성됩니다
      # NRZPAPER_COOKIE_SECURE: "true"       # HTTPS를 통해 제공할 때
      # NRZPAPER_AI_PROVIDER: "openai"       # 또는 웹 인터페이스에서 설정
      # NRZPAPER_AI_KEY: "sk-…"

volumes:
  nrzpaper-data:
```

```bash
docker compose up -d
```

> **`/data`를 백업하세요.** 여기에는 `nrzpaper.db`(모든 것, 암호화됨)와 `nrzpaper.key`가 있습니다. 키가 없으면 데이터베이스를 열 수 없습니다. `NRZPAPER_DB_KEY`를 설정한 경우 대신 그 값을 안전하게 보관하세요.

### 두 아키텍처

```bash
docker buildx build --platform linux/amd64,linux/arm64 -t <registry>/nrzpaper:1.0.0 --push .
```

Rust는 에뮬레이션이 아닌 교차 컴파일되므로 arm64 빌드는 x86 머신에서도 빠르고 그 반대도 마찬가지입니다.

## 첫 단계

1. **업로드**: PDF 몇 개나 사진. 텍스트는 백그라운드에서 인식됩니다.
2. **태그**: *청구서*나 *보험* 같은 태그를 만들고 설명을 추가하세요. AI가 이를 사용해 태그를 할당합니다.
3. **AI**(*설정 → AI 및 모델*): 제공업체를 선택하고 키를 붙여넣은 다음(또는 내장 무료 OpenRouter 모델, 로컬 Ollama 사용), *모델 불러오기*와 *연결 테스트*.
4. **스캐너**(*프로필 → 스캐너*): *네트워크 검색*으로 eSCL/AirScan 스캐너를 찾은 다음 사이드바에서 *스캔*.
5. **이메일**(*프로필 → 이메일 받은편지함*): 하나 이상의 IMAP 사서함 추가. 첨부 파일의 PDF와 이미지가 받은편지함에 들어갑니다.
6. **폴더 받은편지함**(*설정 → 폴더 받은편지함*, 관리자): `/scans` 같은 서버 폴더를 감시합니다(먼저 컨테이너에 마운트).
7. **프린터**(*설정 → 프린터*, 관리자): *네트워크 검색* 또는 `ipp://printer/ipp/print` 입력. 그런 다음 모든 구성원이 문서 페이지에서 인쇄할 수 있습니다.

## 구성

모든 설정은 `NRZPAPER_` 접두사가 붙은 환경 변수(또는 `Rocket.toml`)로 지정할 수 있습니다.

| 변수 | 기본값 | 의미 |
|---|---|---|
| `NRZPAPER_DATA_DIR` | `/data`(이미지) | 데이터베이스, 키 파일, 수집 폴더 |
| `NRZPAPER_DB_KEY` | – | 데이터베이스 키. 설정하지 않으면 데이터 폴더에 `nrzpaper.key`가 생성됩니다 |
| `NRZPAPER_PORT` / `NRZPAPER_ADDRESS` | `8320` / `0.0.0.0` | 서버가 수신하는 위치 |
| `NRZPAPER_COOKIE_SECURE` | `false` | HTTPS 뒤에서는 `true`로 |
| `NRZPAPER_REGISTRATION_OPEN` | `false` | 자체 등록 허용(UI에서도 전환 가능) |
| `NRZPAPER_OCR_LANGUAGES` | `deu+eng` | Tesseract 언어(이미지에는 `deu`와 `eng` 포함) |
| `NRZPAPER_OCR_WORKERS` | `2` | 병렬 OCR 작업 수 |
| `NRZPAPER_TRASH_RETENTION_DAYS` | `30` | 최종 삭제까지 휴지통 보관 일수 |
| `NRZPAPER_INBOX_KEEP_DAYS` | `7` | 승인되지 않은 받은편지함 항목 보관 일수 |
| `NRZPAPER_SCANNER_URL` | – | 새 사용자를 위한 기본 스캐너, 예: `http://192.168.1.50` |
| `NRZPAPER_SCAN_SUBNETS` | `192.168.0.0/24,192.168.1.0/24` | 스캐너와 프린터를 검색하는 네트워크 |
| `NRZPAPER_AI_PROVIDER` | `openrouter` | `openai`, `anthropic`, `gemini`, `mistral`, `groq`, `ollama`, `custom` 등 |
| `NRZPAPER_AI_KEY` / `NRZPAPER_AI_URL` / `NRZPAPER_AI_MODEL` | – | 해당 제공업체의 키, 기본 URL, 모델 |
| `NRZPAPER_OPENROUTER_KEY` | – | OpenRouter 키(위의 대안) |
| `NRZPAPER_LOG` | `info` | 로그 필터, 예: `debug` |

웹 인터페이스에서 설정한 항목(AI 제공업체, 키, 모델, 등록)은 암호화된 데이터베이스에 저장되며 환경 변수보다 우선합니다.

### AI 제공업체

| 제공업체 | 주소 | 비고 |
|---|---|---|
| OpenRouter | 내장 | 무료 비전 모델. 자체 키는 선택 사항 |
| OpenAI, Anthropic, Google Gemini, Mistral, Groq, xAI, DeepSeek, Together, Fireworks, Perplexity, Cohere, NVIDIA, Hugging Face, Qwen | 내장 | 제공업체의 API 키 |
| Azure OpenAI | `https://NAME.openai.azure.com/openai/v1` | 모델 = 배포 이름 |
| Ollama / LM Studio / vLLM / LocalAI | `http://host:11434/v1` 등 | 키 불필요 |
| 모든 OpenAI 호환 서비스 | 사용자의 URL | 키 선택 사항 |

이미지를 읽을 수 없는 모델: *"모델이 이미지를 읽을 수 없음 – 인식된 텍스트 전송"*을 선택하세요. PDF를 받지 않는 제공업체에는 자동으로 첫 페이지가 이미지로 전달됩니다. 제안 제목과 요약의 언어는 *제안 언어*에서 설정합니다.

## REST API

*프로필 → API 키*에서 API 키를 만듭니다(권한 선택). 모든 엔드포인트는 `/api/v1` 아래에 있으며 `Authorization: Bearer np_…`를 받습니다.

```bash
TOKEN=np_your_key
BASE=http://localhost:8320/api/v1

# 조직
curl -H "Authorization: Bearer $TOKEN" $BASE/organizations

# 업로드(여러 -F로 한 번에 여러 파일)
curl -H "Authorization: Bearer $TOKEN" -F file=@invoice.pdf \
     $BASE/organizations/$ORG/documents

# 검색: UI와 동일한 구문
curl -H "Authorization: Bearer $TOKEN" -G $BASE/organizations/$ORG/documents \
     --data-urlencode 'searchQuery=northwind tag:invoice created:2026'

# 파일 다운로드
curl -H "Authorization: Bearer $TOKEN" -o out.pdf \
     "$BASE/organizations/$ORG/documents/$DOC/file?download=true"

# 태그 추가
curl -H "Authorization: Bearer $TOKEN" -H "Content-Type: application/json" \
     -d '{"tagId":"'$TAG'"}' $BASE/organizations/$ORG/documents/$DOC/tags

# 7일 동안 유효한 공개 공유 링크
curl -H "Authorization: Bearer $TOKEN" -H "Content-Type: application/json" \
     -d '{"expiresInDays":7}' $BASE/organizations/$ORG/documents/$DOC/share-links

# 상태 확인(키 불필요)
curl $BASE/health
```

웹훅(*설정 → 웹훅*)은 `document:created`, `document:updated`, `document:deleted`, `document:restored`, `document:tag:added`, `document:tag:removed`에 대해 서명된 JSON(`webhook-id`, `webhook-timestamp`, `webhook-signature`)을 보냅니다.

수집 폴더 `data/ingestion/<organization-id>/`는 파일을 문서로 직접 가져옵니다. 감시 폴더(*폴더 받은편지함*)는 먼저 검토를 위해 받은편지함으로 보냅니다.

## 개발

```bash
# 백엔드(Rust 1.85+, OCR용 Tesseract와 Poppler)
cargo run                         # http://localhost:8320
cargo test

# 핫 리로드가 있는 웹 인터페이스(dx = dioxus-cli 0.7)
rustup target add wasm32-unknown-unknown
cargo install dioxus-cli --version 0.7.10 --locked
cd ui && dx serve                 # http://localhost:8080, /api를 :8320으로 프록시

# 인터페이스를 프로그램에 포함
scripts/ui.sh && cargo build --release
```

번역은 `locales/<language>.json`(독일어 원문 → 번역)에 있습니다. `scripts/i18n_extract.py`는 `locales/strings.json`을 갱신하고 언어별 누락 항목을 보고합니다.

아키텍처 세부 정보: [ARCHITECTURE.md](../ARCHITECTURE.md).

## 라이선스

AGPL-3.0-or-later. UI 구성 요소는 [rust-ui](https://github.com/rust-ui/ui)(MIT), PDF 뷰어는 [PDF.js](https://mozilla.github.io/pdf.js/)(Apache-2.0), 글꼴은 [Inter](https://rsms.me/inter/)(OFL)를 기반으로 합니다.
