# 📋 Kanban Board – nrzPaper

**Project:** nrzPaper – Self-hosted document management in Rust  
**Board:** https://github.com/nicorenezweers/nrzPaper/projects  
**Columns:** `Backlog` → `To Do` → `In Progress` → `Review` → `Done`

> Tip: In GitHub, go to *Projects → New project*, create a Board, and add the columns **Backlog**, **To Do**, **In Progress**, **Review**, **Done**. Then create an Issue for every `[ ]` task above and assign it to column **To Do**. Move finished tasks to **Done**.

---

## 1️⃣ Languages / i18n (Goal: 25 languages)

**Currently (7):** en, de, fr, it, es, pt, nl  
**Missing for 25:** 18 more languages

### Backlog
- [ ] Document localization workflow (`scripts/i18n_extract.py`)
- [ ] Secure placeholder checks (`{0}`, `{1}`) in all translations
- [ ] Check RTL support (Arabic, Hebrew) in UI CSS

### To Do (not started yet)
- [ ] cs – Czech
- [ ] da – Danish
- [ ] el – Greek
- [ ] fi – Finnish
- [ ] he – Hebrew
- [ ] hu – Hungarian
- [ ] ja – Japanese
- [ ] ko – Korean
- [ ] nb – Norwegian (Bokmål)
- [ ] pl – Polish
- [ ] ro – Romanian
- [ ] ru – Russian
- [ ] sk – Slovak
- [ ] sv – Swedish
- [ ] tr – Turkish
- [ ] uk – Ukrainian
- [ ] zh – Chinese (simplified)
- [ ] ar – Arabic

### In Progress
- (none)

### Review
- (none)

### Done
- [x] en – English
- [x] de – German
- [x] fr – French
- [x] it – Italian
- [x] es – Spanish
- [x] pt – Portuguese
- [x] nl – Dutch

---

## 2️⃣ Native Apps (iOS, Android, Windows, macOS, Linux)

**Currently:** Web UI (Dioxus/WASM), works in mobile browser  
**Goal:** Native apps for all 5 platforms

### Backlog
- [ ] Decide on technology for native apps (e.g. Tauri, Flutter, Compose Multiplatform, .NET MAUI)
- [ ] Secure API authentication for native apps (API keys / OAuth2)
- [ ] Design offline mode and local storage
- [ ] Sync concept (Web UI ↔ native apps via the same nrzPaper instance)

### To Do
- [ ] 🐧 Linux app (native)
- [ ] 🪟 Windows app (native)
- [ ] 🍎 macOS app (native)
- [ ] 📱 iOS app (native)
- [ ] 🤖 Android app (native)
- [ ] App icon & branding for all platforms
- [ ] Push notifications (reminders, new documents)

### In Progress
- (none)

### Review
- (none)

### Done
- [x] Web UI (Dioxus, responsive, works on phones)

---

## 3️⃣ Backend / Core

### Backlog
- [ ] Extend full-text search filters
- [ ] Make OCR languages configurable via UI (currently `deu+eng`)
- [ ] Document versions / history
- [ ] Batch processing for inbox entries

### To Do
- [ ] Email: improve parallel IMAP account retrieval
- [ ] Scanner: stabilize eSCL/AirScan detection
- [ ] Printing: cache IPP printer list in UI
- [ ] Webhooks: additional events (e.g. `reminder:due`)

### In Progress
- (none)

### Review
- (none)

### Done
- [x] One encrypted SQLite file (everything incl. documents)
- [x] OCR with Tesseract + Poppler
- [x] Inbox with AI suggestions (name, tags, fields)
- [x] Tags with colors, folders from tags, tagging rules
- [x] Timeline, reminders
- [x] IPP printing (AirPrint, IPP Everywhere, Mopria, CUPS)
- [x] AI: OpenRouter, OpenAI, Anthropic, Gemini, Mistral, Groq, xAI, DeepSeek, Ollama, LM Studio, vLLM, Azure, etc.
- [x] Dieter assistant (chat with sources)
- [x] Teams/organizations, roles, API keys, webhooks, public share links
- [x] REST API under `/api/v1`
- [x] Docker (linux/amd64 + linux/arm64)

---

## 4️⃣ UI / Web Interface

### Backlog
- [ ] Dark-/Light-theme toggle
- [ ] PDF viewer: annotations / comments
- [ ] Drag & drop documents directly onto tags

### To Do
- [ ] Mobile: optimize touch operation (scan, upload, signature)
- [ ] Search suggestions (autocomplete while typing)
- [ ] Exports: PDF/CSV/JSON of document lists

### In Progress
- (none)

### Review
- (none)

### Done
- [x] Dioxus Web UI, dark theme
- [x] Login, sidebar, profile menu
- [x] Document page, tags, trash
- [x] Settings: AI, members, scanner, email, folder, printers

---

## 5️⃣ DevOps / Release / Infrastructure

### Backlog
- [ ] GitHub Actions: CI (cargo build + test) for all PRs
- [ ] GitHub Releases: automatic binaries for Windows/macOS/Linux
- [ ] Docker image: multi-arch build (amd64/arm64) publish to GHCR
- [ ] Helm chart / Kubernetes manifests
- [ ] Backup/restore documentation (file + key)

### To Do
- [ ] Maintain release notes per version
- [ ] Update check in the app (show new version)
- [ ] Monitor health endpoint `/api/v1/health` (Prometheus metrics)

### In Progress
- (none)

### Review
- (none)

### Done
- [x] Dockerfile + docker-compose.yml
- [x] `cargo run` / `cargo test` work
- [x] `scripts/ui.sh` builds the web UI

---

## 🚀 Suggested Milestones

| Milestone | Content |
|---|---|
| **v1.1** | 15 languages (7 current + 8 new), i18n workflow secured |
| **v1.2** | First native app (Linux or Windows) |
| **v1.3** | iOS + Android apps |
| **v1.4** | 25 languages complete, offline mode |
| **v2.0** | All 5 platforms native + app stores |

---

### How to adopt this board on GitHub

1. Go to https://github.com/nicorenezweers/nrzPaper/projects
2. Click *New project* → *Board*
3. Create the columns: **Backlog**, **To Do**, **In Progress**, **Review**, **Done**
4. Create an Issue for every `[ ]` task above and assign it to column **To Do**
5. Move completed tasks to **Done**

Do you want me to create GitHub Issues directly from one of these categories, or export this to another format (Trello CSV, Notion, etc.)?