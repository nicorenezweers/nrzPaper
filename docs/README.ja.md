<div align="center">

<img src="../static/icon.png" width="96" alt="nrzPaper">

# nrzPaper

**Rust 製のセルフホスト型文書管理：書類をスキャン、認識、タグ付け、検索。すべてを 1 つの暗号化ファイルに。**

[English](../README.md) · [Deutsch](README.de.md) · [Français](README.fr.md) · [Italiano](README.it.md) · [Español](README.es.md) · [Português](README.pt.md) · [Nederlands](README.nl.md) · [中文](README.zh.md) · [日本語](README.ja.md) · [Polski](README.pl.md) · [Русский](README.ru.md) · [العربية](README.ar.md) · [한국어](README.ko.md) · [Türkçe](README.tr.md) · [Bahasa Indonesia](README.id.md) · [हिन्दी](README.hi.md) · [Tiếng Việt](README.vi.md) · [ไทย](README.th.md) · [Українська](README.uk.md) · [Čeština](README.cs.md) · [Română](README.ro.md) · [Magyar](README.hu.md) · [Ελληνικά](README.el.md) · [Svenska](README.sv.md) · [Norsk](README.nb.md) · [Dansk](README.da.md) · [Suomi](README.fi.md) · [עברית](README.he.md) · [বাংলা](README.bn.md) · [فارسی](README.fa.md)

![タイムライン付きの文書](screenshots/documents.png)

</div>

nrzPaper は外部データベースを使わない単一のプログラムです。ファイル自体を含め、すべてが SQLCipher で暗号化された 1 つの SQLite ファイルに保存されます。Tesseract による OCR、ネットワークスキャナーからの直接スキャン、メール添付ファイルの取り込みを行い、選んだ AI にタイトル、タグ、メタデータを提案させることができます。内蔵アシスタント **Dieter** は文書に関する質問に答え、出典をリンクします。Web インターフェースは落ち着いたダークテーマの Rust/WebAssembly アプリ（Dioxus）で、スマートフォンでも動作します。

## 機能

| | |
|---|---|
| **取り込み** | ドラッグ＆ドロップ、アップロード、**ネットワークスキャナー**（eSCL/AirScan：Canon、HP、Epson、Brother …）、**メールボックス**（IMAP、複数アカウント）、**監視フォルダー**（例：NAS 上のスキャンフォルダー）、REST API |
| **受信箱** | スキャン、メール、フォルダーの取り込み、アップロードが確認待ちになります。AI が名前、タグ、フィールドを提案し、1 件ずつまたは**まとめて**承認できます。ソース別（スキャナー、各メールボックス、各フォルダー、アップロード）に絞り込めます。 |
| **認識** | バックグラウンドで Tesseract と Poppler を実行。スキャンした PDF や写真にも対応。プレフィックス、フレーズ、フィルター（`tag:`、`-tag:`、`type:`、`created:>2026-06-01` など）を使った全文検索。 |
| **整理** | 色付きタグ、**タグから作るフォルダー**（ネスト可）、**タグ付けルール**（「Northwind を含む → 請求書」）、カスタム**プロパティ**（テキスト、数値、日付、はい/いいえ、選択、複数選択、ユーザー、文書リンク） |
| **タイムライン** | 文書の横にある縦のタイムライン。今日が一番上、その下にメール付きの各日、そして間の静かな空白（「文書のない 12 日間」）。タグ、種類、テキストで絞り込めます。 |
| **リマインダー** | どの文書にも「10 日後に通知」をメモ付きで設定可能。期限が来たリマインダーはサイドバーとタイムラインの上部に表示されます。 |
| **印刷** | サーバーが見つけた任意の **IPP ネットワークプリンター**（AirPrint、IPP Everywhere、Mopria、CUPS）で印刷。PDF は直接送信、JPEG しか受け付けないプリンターにはページ画像として送信。 |
| **AI、あらゆるプロバイダー** | OpenRouter（無料モデル内蔵）、OpenAI、Anthropic、Google Gemini、Mistral、Groq、xAI、DeepSeek、Together、Fireworks、Perplexity、Cohere、NVIDIA、Hugging Face、Qwen、Azure OpenAI、**Ollama、LM Studio、vLLM/LocalAI**、または任意の OpenAI 互換 URL。テキスト専用モデルには画像ではなく OCR テキストが渡されます。 |
| **Dieter** | チャットアシスタント（右下）：*「私の保険番号は？」*、*「2026 年の Fabrikam の給与明細すべて」*。検索し、該当箇所を読み、文書へのリンク付きで答えます。 |
| **チーム** | 複数の組織、**組織ごとに複数アカウント**（招待、役割：オーナー、管理者、メンバー、所有権の移譲）、権限付き API キー、Webhook（Standard Webhooks 署名）、公開共有リンク |
| **言語** | English、Deutsch、Français、Italiano、Español、Português、Nederlands、中文、日本語、Polski、Русский、العربية、한국어、Türkçe、Bahasa Indonesia、हिन्दी、Tiếng Việt、ไทย、Українська、Čeština、Română、Magyar、Ελληνικά、Svenska、Norsk、Dansk、Suomi、עברית、বাংলা、فارسی（その他は `locales/<code>.json`） |
| **デフォルトで安全** | 1 つの暗号化ファイル、Argon2 パスワード、インラインスクリプトなしの CSP、CSRF 対策、ファイルは API 経由のみ |

## スクリーンショット

| | |
|---|---|
| ![Dieter、アシスタント](screenshots/dieter.png) | ![AI 提案付きの受信箱](screenshots/inbox.png) |
| **Dieter** が出典付きで回答 | **受信箱**：確認と承認、ソース別フィルター |
| ![文書ビュー](screenshots/document.png) | ![AI プロバイダー](screenshots/settings-ai.png) |
| **文書**：PDF ビューアー、タグ、リマインダー、印刷 | **AI とモデル**：あらゆるプロバイダー、ローカルまたはクラウド |
| ![リマインダー](screenshots/reminders.png) | ![メンバー](screenshots/members.png) |
| **リマインダー** | **メンバー**と招待 |

## クイックスタート

Docker が必要です。イメージにはプログラム、Web インターフェース、Tesseract（ドイツ語、英語）、Poppler が含まれます。

```bash
git clone https://github.com/nicorenezweers/nrzPaper.git
cd nrzPaper
docker build -t nrzpaper .
docker run -d --name nrzpaper -p 8320:8320 -v nrzpaper-data:/data nrzpaper
```

<http://localhost:8320> を開きます。最初に作成したアカウントが管理者になります。その後、*設定 → 登録とユーザー*で登録を開くまで登録は閉じられます。そこでは他のユーザーのアカウントも作成できます。

### Docker Compose

```yaml
services:
  nrzpaper:
    build: .                      # または: image: ghcr.io/nicorenezweers/nrzpaper:1.0.0
    container_name: nrzpaper
    restart: unless-stopped
    ports:
      - "8320:8320"
    volumes:
      - nrzpaper-data:/data       # 暗号化データベース + キーファイル
      - /srv/scans:/scans         # 任意：フォルダー受信箱用のフォルダー
    environment:
      TZ: Europe/Zurich
      # NRZPAPER_DB_KEY: "a-long-secret"     # 未設定なら /data/nrzpaper.key が作成されます
      # NRZPAPER_COOKIE_SECURE: "true"       # HTTPS 経由で提供する場合
      # NRZPAPER_AI_PROVIDER: "openai"       # または Web インターフェースで設定
      # NRZPAPER_AI_KEY: "sk-…"

volumes:
  nrzpaper-data:
```

```bash
docker compose up -d
```

> **`/data` をバックアップしてください。** ここには `nrzpaper.db`（すべて、暗号化済み）と `nrzpaper.key` があります。キーがないとデータベースを開けません。`NRZPAPER_DB_KEY` を設定した場合は、代わりにその値を安全に保管してください。

### 両方のアーキテクチャ

```bash
docker buildx build --platform linux/amd64,linux/arm64 -t <registry>/nrzpaper:1.0.0 --push .
```

Rust はエミュレーションではなくクロスコンパイルされるため、arm64 ビルドは x86 マシンでも高速で、その逆も同様です。

## 最初のステップ

1. **アップロード**：いくつかの PDF や写真。テキストはバックグラウンドで認識されます。
2. **タグ**：*請求書*や*保険*などのタグを作成し、説明を付けます。AI はそれを使ってタグを割り当てます。
3. **AI**（*設定 → AI とモデル*）：プロバイダーを選び、キーを貼り付け（または内蔵の無料 OpenRouter モデル、ローカルの Ollama を使用）、*モデルを読み込む*、*接続をテスト*。
4. **スキャナー**（*プロフィール → スキャナー*）：*ネットワークを検索*で eSCL/AirScan スキャナーを見つけ、サイドバーの*スキャン*で実行。
5. **メール**（*プロフィール → メール受信箱*）：1 つ以上の IMAP メールボックスを追加。添付ファイルの PDF と画像が受信箱に入ります。
6. **フォルダー受信箱**（*設定 → フォルダー受信箱*、管理者）：`/scans` などのサーバーフォルダーを監視（先にコンテナへマウント）。
7. **プリンター**（*設定 → プリンター*、管理者）：*ネットワークを検索*または `ipp://printer/ipp/print` を入力。その後、各メンバーが文書ページから印刷できます。

## 設定

すべての設定は `NRZPAPER_` プレフィックス付きの環境変数（または `Rocket.toml`）で指定できます。

| 変数 | デフォルト | 意味 |
|---|---|---|
| `NRZPAPER_DATA_DIR` | `/data`（イメージ） | データベース、キーファイル、取り込みフォルダー |
| `NRZPAPER_DB_KEY` | – | データベースキー。未設定ならデータフォルダーに `nrzpaper.key` が生成されます |
| `NRZPAPER_PORT` / `NRZPAPER_ADDRESS` | `8320` / `0.0.0.0` | サーバーの待ち受け先 |
| `NRZPAPER_COOKIE_SECURE` | `false` | HTTPS の背後では `true` に |
| `NRZPAPER_REGISTRATION_OPEN` | `false` | 自己登録を許可（UI でも切り替え可能） |
| `NRZPAPER_OCR_LANGUAGES` | `deu+eng` | Tesseract の言語（イメージには `deu` と `eng` を同梱） |
| `NRZPAPER_OCR_WORKERS` | `2` | 並列 OCR ジョブ数 |
| `NRZPAPER_TRASH_RETENTION_DAYS` | `30` | 完全削除までのゴミ箱の日数 |
| `NRZPAPER_INBOX_KEEP_DAYS` | `7` | 未承認の受信箱項目を保持する日数 |
| `NRZPAPER_SCANNER_URL` | – | 新規ユーザー用の既定スキャナー、例 `http://192.168.1.50` |
| `NRZPAPER_SCAN_SUBNETS` | `192.168.0.0/24,192.168.1.0/24` | スキャナーとプリンターを検索するネットワーク |
| `NRZPAPER_AI_PROVIDER` | `openrouter` | `openai`、`anthropic`、`gemini`、`mistral`、`groq`、`ollama`、`custom` など |
| `NRZPAPER_AI_KEY` / `NRZPAPER_AI_URL` / `NRZPAPER_AI_MODEL` | – | そのプロバイダーのキー、ベース URL、モデル |
| `NRZPAPER_OPENROUTER_KEY` | – | OpenRouter キー（上記の代替） |
| `NRZPAPER_LOG` | `info` | ログフィルター、例 `debug` |

Web インターフェースで行った設定（AI プロバイダー、キー、モデル、登録）は暗号化データベースに保存され、環境変数より優先されます。

### AI プロバイダー

| プロバイダー | アドレス | 備考 |
|---|---|---|
| OpenRouter | 内蔵 | 無料のvisionモデル。自分のキーは任意 |
| OpenAI、Anthropic、Google Gemini、Mistral、Groq、xAI、DeepSeek、Together、Fireworks、Perplexity、Cohere、NVIDIA、Hugging Face、Qwen | 内蔵 | プロバイダーの API キー |
| Azure OpenAI | `https://NAME.openai.azure.com/openai/v1` | モデル = デプロイ名 |
| Ollama / LM Studio / vLLM / LocalAI | `http://host:11434/v1` など | キー不要 |
| OpenAI 互換のものすべて | 自分の URL | キーは任意 |

画像を読めないモデル：*「モデルは画像を読めません – 認識したテキストを送信」*にチェック。PDF を受け付けないプロバイダーには自動的に最初のページが画像として送られます。提案のタイトルと要約の言語は*提案の言語*で設定します。

## REST API

*プロフィール → API キー*で API キーを作成（権限を選択）。すべてのエンドポイントは `/api/v1` にあり、`Authorization: Bearer np_…` を受け付けます。

```bash
TOKEN=np_your_key
BASE=http://localhost:8320/api/v1

# 組織
curl -H "Authorization: Bearer $TOKEN" $BASE/organizations

# アップロード（複数の -F で複数ファイルを一度に）
curl -H "Authorization: Bearer $TOKEN" -F file=@invoice.pdf \
     $BASE/organizations/$ORG/documents

# 検索：UI と同じ構文
curl -H "Authorization: Bearer $TOKEN" -G $BASE/organizations/$ORG/documents \
     --data-urlencode 'searchQuery=northwind tag:invoice created:2026'

# ファイルをダウンロード
curl -H "Authorization: Bearer $TOKEN" -o out.pdf \
     "$BASE/organizations/$ORG/documents/$DOC/file?download=true"

# タグを追加
curl -H "Authorization: Bearer $TOKEN" -H "Content-Type: application/json" \
     -d '{"tagId":"'$TAG'"}' $BASE/organizations/$ORG/documents/$DOC/tags

# 7 日間有効な公開共有リンク
curl -H "Authorization: Bearer $TOKEN" -H "Content-Type: application/json" \
     -d '{"expiresInDays":7}' $BASE/organizations/$ORG/documents/$DOC/share-links

# ヘルスチェック（キー不要）
curl $BASE/health
```

Webhook（*設定 → Webhook*）は `document:created`、`document:updated`、`document:deleted`、`document:restored`、`document:tag:added`、`document:tag:removed` に対して署名付き JSON（`webhook-id`、`webhook-timestamp`、`webhook-signature`）を送信します。

取り込みフォルダー `data/ingestion/<organization-id>/` はファイルを直接文書として取り込みます。監視フォルダー（*フォルダー受信箱*）はまず確認のため受信箱に送ります。

## 開発

```bash
# バックエンド（Rust 1.85+、OCR 用の Tesseract と Poppler）
cargo run                         # http://localhost:8320
cargo test

# ホットリロード付き Web インターフェース（dx = dioxus-cli 0.7）
rustup target add wasm32-unknown-unknown
cargo install dioxus-cli --version 0.7.10 --locked
cd ui && dx serve                 # http://localhost:8080、/api を :8320 にプロキシ

# インターフェースをプログラムに埋め込む
scripts/ui.sh && cargo build --release
```

翻訳は `locales/<language>.json`（ドイツ語の原文 → 翻訳）にあります。`scripts/i18n_extract.py` は `locales/strings.json` を更新し、言語ごとに不足している項目を報告します。

アーキテクチャの詳細：[ARCHITECTURE.md](../ARCHITECTURE.md)。

## ライセンス

AGPL-3.0-or-later。UI コンポーネントは [rust-ui](https://github.com/rust-ui/ui)（MIT）、PDF ビューアーは [PDF.js](https://mozilla.github.io/pdf.js/)（Apache-2.0）、フォントは [Inter](https://rsms.me/inter/)（OFL）に基づいています。
