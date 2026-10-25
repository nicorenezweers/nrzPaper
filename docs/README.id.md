<div align="center">

<img src="../static/icon.png" width="96" alt="nrzPaper">

# nrzPaper

**A Rust-powered document management tool.**

[English](../README.md) · [Deutsch](README.de.md) · [Français](README.fr.md) · [Italiano](README.it.md) · [Español](README.es.md) · [Português](README.pt.md) · [Nederlands](README.nl.md) · [中文](README.zh.md) · [日本語](README.ja.md) · [Polski](README.pl.md) · [Русский](README.ru.md) · [العربية](README.ar.md) · [한국어](README.ko.md) · [Türkçe](README.tr.md) · [Bahasa Indonesia](README.id.md) · [हिन्दी](README.hi.md) · [Tiếng Việt](README.vi.md) · [ไทย](README.th.md) · [Українська](README.uk.md) · [Čeština](README.cs.md) · [Română](README.ro.md) · [Magyar](README.hu.md) · [Ελληνικά](README.el.md) · [Svenska](README.sv.md) · [Norsk](README.nb.md) · [Dansk](README.da.md) · [Suomi](README.fi.md) · [עברית](README.he.md) · [বাংলা](README.bn.md) · [فارسی](README.fa.md)

![Dokumen dengan garis waktu](screenshots/documents.png)

</div>

nrzPaper adalah satu program tanpa basis data eksternal. Semua tersimpan dalam satu berkas SQLite yang dienkripsi SQLCipher, termasuk berkasnya sendiri. Ia melakukan OCR dengan Tesseract, memindai langsung dari pemindai jaringan, mengambil lampiran email, dan memungkinkan AI pilihan Anda menyarankan judul, tag, dan metadata. Asisten bawaan **Dieter** menjawab pertanyaan tentang dokumen Anda dan menautkan sumbernya. Antarmuka web adalah aplikasi Rust/WebAssembly (Dioxus) dengan tema gelap yang tenang dan juga berfungsi di ponsel.

## Fitur

| | |
|---|---|
| **Penangkapan** | Seret & lepas, unggah, **pemindai jaringan** (eSCL/AirScan: Canon, HP, Epson, Brother …), **kotak masuk email** (IMAP, beberapa akun), **folder yang dipantau** (mis. folder pindai di NAS), REST API |
| **Kotak masuk** | Pindai, email, impor folder, dan unggahan menunggu peninjauan: AI menyarankan nama, tag, dan bidang, dan Anda menyetujui satu per satu atau **semua sekaligus**. Saring berdasarkan sumber (pemindai, setiap kotak surat, setiap folder, unggahan). |
| **Pengenalan** | Tesseract dan Poppler di latar belakang, juga untuk PDF hasil pindai dan foto; pencarian teks lengkap dengan awalan, frasa, dan filter (`tag:`, `-tag:`, `type:`, `created:>2026-06-01`, …) |
| **Pengorganisasian** | Tag berwarna, **folder dari tag** (bersarang), **aturan penandaan** ("berisi Northwind → Faktur"), **properti** khusus (teks, angka, tanggal, ya/tidak, pilihan, pilihan ganda, pengguna, tautan dokumen) |
| **Garis waktu** | Garis waktu vertikal di samping dokumen: hari ini di atas, di bawahnya setiap hari dengan email, dan jeda tenang di antaranya ("12 hari tanpa dokumen"). Saring berdasarkan tag, jenis, dan teks. |
| **Pengingat** | "Ingatkan saya dalam 10 hari" pada dokumen mana pun, dengan catatan; pengingat yang jatuh tempo muncul di bilah samping dan di atas garis waktu. |
| **Cetak** | Cetak dokumen di **printer jaringan IPP** mana pun (AirPrint, IPP Everywhere, Mopria, CUPS) yang ditemukan server. PDF dikirim langsung, atau sebagai gambar halaman ke printer yang hanya menerima JPEG. |
| **AI, penyedia apa pun** | OpenRouter (model gratis bawaan), OpenAI, Anthropic, Google Gemini, Mistral, Groq, xAI, DeepSeek, Together, Fireworks, Perplexity, Cohere, NVIDIA, Hugging Face, Qwen, Azure OpenAI, **Ollama, LM Studio, vLLM/LocalAI**, atau URL apa pun yang kompatibel dengan OpenAI. Model khusus teks menerima teks OCR alih-alih gambar. |
| **Dieter** | Asisten obrolan (kanan bawah): *"Berapa nomor asuransi saya?"*, *"Semua slip gaji dari Fabrikam tahun 2026"*. Ia mencari, membaca bagian yang cocok, dan menjawab dengan tautan ke dokumen. |
| **Tim** | Beberapa organisasi, **beberapa akun per organisasi** (undangan, peran: pemilik, admin, anggota; alihkan kepemilikan), kunci API dengan izin, webhook (tanda tangan Standard Webhooks), tautan berbagi publik |
| **Bahasa** | English, Deutsch, Français, Italiano, Español, Português, Nederlands (bahasa lain segera hadir) |
| **Aman secara bawaan** | Satu berkas terenkripsi, kata sandi Argon2, CSP tanpa skrip inline, perlindungan CSRF, berkas hanya melalui API |

## Tangkapan layar

| | |
|---|---|
| ![Dieter, asisten](screenshots/dieter.png) | ![Kotak masuk dengan saran AI](screenshots/inbox.png) |
| **Dieter** menjawab dengan sumber | **Kotak masuk**: tinjau dan setujui, saring berdasarkan sumber |
| ![Tampilan dokumen](screenshots/document.png) | ![Penyedia AI](screenshots/settings-ai.png) |
| **Dokumen**: penampil PDF, tag, pengingat, cetak | **AI & model**: setiap penyedia, lokal atau cloud |
| ![Pengingat](screenshots/reminders.png) | ![Anggota](screenshots/members.png) |
| **Pengingat** | **Anggota** dan undangan |

## Mulai cepat

Anda memerlukan Docker. Gambar berisi program, antarmuka web, Tesseract (Jerman, Inggris), dan Poppler.

```bash
git clone https://github.com/nicorenezweers/nrzPaper.git
cd nrzPaper
docker build -t nrzpaper .
docker run -d --name nrzpaper -p 8320:8320 -v nrzpaper-data:/data nrzpaper
```

Buka <http://localhost:8320>. Akun pertama yang Anda buat menjadi administrator. Setelah itu pendaftaran ditutup hingga Anda membukanya di *Pengaturan → Pendaftaran & pengguna*, tempat Anda juga dapat membuat akun untuk orang lain.

### Docker Compose

```yaml
services:
  nrzpaper:
    build: .                      # atau: image: ghcr.io/nicorenezweers/nrz-paper:1.0.0
    container_name: nrzpaper
    restart: unless-stopped
    ports:
      - "8320:8320"
    volumes:
      - nrzpaper-data:/data       # basis data terenkripsi + berkas kunci
      - /srv/scans:/scans         # opsional: folder untuk kotak masuk folder
    environment:
      TZ: Europe/Zurich
      # NRZPAPER_DB_KEY: "a-long-secret"     # jika tidak, /data/nrzpaper.key dibuat
      # NRZPAPER_COOKIE_SECURE: "true"       # saat disajikan melalui HTTPS
      # NRZPAPER_AI_PROVIDER: "openai"       # atau atur di antarmuka web
      # NRZPAPER_AI_KEY: "sk-…"

volumes:
  nrzpaper-data:
```

```bash
docker compose up -d
```

> **Cadangkan `/data`.** Di sana ada `nrzpaper.db` (semuanya, terenkripsi) dan `nrzpaper.key`. Tanpa kunci, basis data tidak dapat dibuka. Jika Anda menyetel `NRZPAPER_DB_KEY`, simpan nilai itu dengan aman.

### Kedua arsitektur

```bash
docker buildx build --platform linux/amd64,linux/arm64 -t <registry>/nrzpaper:1.0.0 --push .
```

Rust dikompilasi silang, bukan diemulasi, sehingga build arm64 cepat di mesin x86 dan sebaliknya.

## Langkah pertama

1. **Unggah** beberapa PDF atau foto: teks dikenali di latar belakang.
2. **Tag**: buat tag seperti *Faktur* atau *Asuransi* dan beri deskripsi. AI menggunakannya untuk menetapkan tag.
3. **AI** (*Pengaturan → AI & model*): pilih penyedia, tempel kunci (atau gunakan model OpenRouter gratis bawaan, atau Ollama lokal), lalu *Muat model* dan *Uji koneksi*.
4. **Pemindai** (*Profil → Pemindai*): *Cari di jaringan* menemukan pemindai eSCL/AirScan; lalu *Pindai* di bilah samping.
5. **Email** (*Profil → Kotak masuk email*): tambahkan satu atau lebih kotak surat IMAP; PDF dan gambar dari lampiran masuk ke kotak masuk.
6. **Kotak masuk folder** (*Pengaturan → Kotak masuk folder*, admin): pantau folder server seperti `/scans` (pasang dulu ke kontainer).
7. **Printer** (*Pengaturan → Printer*, admin): *Cari di jaringan* atau masukkan `ipp://printer/ipp/print`; setelah itu setiap anggota dapat mencetak dari halaman dokumen.

## Konfigurasi

Setiap pengaturan dapat diberikan sebagai variabel lingkungan dengan awalan `NRZPAPER_` (atau di `Rocket.toml`).

| Variabel | Bawaan | Arti |
|---|---|---|
| `NRZPAPER_DATA_DIR` | `/data` (gambar) | Basis data, berkas kunci, dan folder penyerapan |
| `NRZPAPER_DB_KEY` | – | Kunci basis data; jika tidak disetel, `nrzpaper.key` dibuat di folder data |
| `NRZPAPER_PORT` / `NRZPAPER_ADDRESS` | `8320` / `0.0.0.0` | Tempat server mendengarkan |
| `NRZPAPER_COOKIE_SECURE` | `false` | Setel `true` di belakang HTTPS |
| `NRZPAPER_REGISTRATION_OPEN` | `false` | Izinkan pendaftaran mandiri (juga dapat diubah di UI) |
| `NRZPAPER_OCR_LANGUAGES` | `deu+eng` | Bahasa Tesseract (gambar menyertakan `deu` dan `eng`) |
| `NRZPAPER_OCR_WORKERS` | `2` | Pekerjaan OCR paralel |
| `NRZPAPER_TRASH_RETENTION_DAYS` | `30` | Hari di tempat sampah sebelum dihapus permanen |
| `NRZPAPER_INBOX_KEEP_DAYS` | `7` | Hari item kotak masuk yang belum disetujui disimpan |
| `NRZPAPER_SCANNER_URL` | – | Pemindai prasetel untuk pengguna baru, mis. `http://192.168.1.50` |
| `NRZPAPER_SCAN_SUBNETS` | `192.168.0.0/24,192.168.1.0/24` | Jaringan yang dicari untuk pemindai dan printer |
| `NRZPAPER_AI_PROVIDER` | `openrouter` | `openai`, `anthropic`, `gemini`, `mistral`, `groq`, `ollama`, `custom`, … |
| `NRZPAPER_AI_KEY` / `NRZPAPER_AI_URL` / `NRZPAPER_AI_MODEL` | – | Kunci, URL dasar, dan model untuk penyedia tersebut |
| `NRZPAPER_OPENROUTER_KEY` | – | Kunci OpenRouter (alternatif untuk di atas) |
| `NRZPAPER_LOG` | `info` | Filter log, mis. `debug` |

Pengaturan yang dibuat di antarmuka web (penyedia AI, kunci, model, pendaftaran) disimpan di basis data terenkripsi dan lebih diutamakan daripada lingkungan.

### Penyedia AI

| Penyedia | Alamat | Catatan |
|---|---|---|
| OpenRouter | bawaan | Model visi gratis; kunci sendiri opsional |
| OpenAI, Anthropic, Google Gemini, Mistral, Groq, xAI, DeepSeek, Together, Fireworks, Perplexity, Cohere, NVIDIA, Hugging Face, Qwen | bawaan | Kunci API dari penyedia |
| Azure OpenAI | `https://NAME.openai.azure.com/openai/v1` | Model = nama deployment |
| Ollama / LM Studio / vLLM / LocalAI | `http://host:11434/v1` dll. | Tidak perlu kunci |
| Apa pun yang kompatibel dengan OpenAI | URL Anda | Kunci opsional |

Model yang tidak dapat membaca gambar: centang *"Model tidak dapat membaca gambar – kirim teks yang dikenali"*. Penyedia yang tidak menerima PDF otomatis mendapatkan halaman pertama sebagai gambar. Bahasa judul dan ringkasan yang disarankan diatur di *Bahasa saran*.

## REST API

Buat kunci API di *Profil → Kunci API* (pilih izin). Semua endpoint berada di bawah `/api/v1` dan menerima `Authorization: Bearer np_…`.

```bash
TOKEN=np_your_key
BASE=http://localhost:8320/api/v1

# organisasi
curl -H "Authorization: Bearer $TOKEN" $BASE/organizations

# unggah (beberapa berkas sekaligus dengan beberapa -F)
curl -H "Authorization: Bearer $TOKEN" -F file=@invoice.pdf \
     $BASE/organizations/$ORG/documents

# cari: sintaks yang sama seperti di UI
curl -H "Authorization: Bearer $TOKEN" -G $BASE/organizations/$ORG/documents \
     --data-urlencode 'searchQuery=northwind tag:invoice created:2026'

# unduh berkas
curl -H "Authorization: Bearer $TOKEN" -o out.pdf \
     "$BASE/organizations/$ORG/documents/$DOC/file?download=true"

# tambahkan tag
curl -H "Authorization: Bearer $TOKEN" -H "Content-Type: application/json" \
     -d '{"tagId":"'$TAG'"}' $BASE/organizations/$ORG/documents/$DOC/tags

# tautan berbagi publik berlaku 7 hari
curl -H "Authorization: Bearer $TOKEN" -H "Content-Type: application/json" \
     -d '{"expiresInDays":7}' $BASE/organizations/$ORG/documents/$DOC/share-links

# pemeriksaan kesehatan (tanpa kunci)
curl $BASE/health
```

Webhook (*Pengaturan → Webhook*) mengirim JSON bertanda tangan (`webhook-id`, `webhook-timestamp`, `webhook-signature`) untuk `document:created`, `document:updated`, `document:deleted`, `document:restored`, `document:tag:added`, dan `document:tag:removed`.

Folder penyerapan `data/ingestion/<organization-id>/` mengimpor berkas langsung sebagai dokumen. Folder yang dipantau (*Kotak masuk folder*) mengirimnya dulu ke kotak masuk untuk ditinjau.

## Pengembangan

```bash
# backend (Rust 1.85+, Tesseract dan Poppler untuk OCR)
cargo run                         # http://localhost:8320
cargo test

# antarmuka web dengan hot reload (dx = dioxus-cli 0.7)
rustup target add wasm32-unknown-unknown
cargo install dioxus-cli --version 0.7.10 --locked
cd ui && dx serve                 # http://localhost:8080, memproksi /api ke :8320

# sematkan antarmuka ke dalam program
scripts/ui.sh && cargo build --release
```

Terjemahan berada di `locales/<language>.json` (teks sumber Jerman → terjemahan). `scripts/i18n_extract.py` menyegarkan `locales/strings.json` dan melaporkan entri yang hilang per bahasa.

Detail arsitektur: [ARCHITECTURE.md](../ARCHITECTURE.md).

## Lisensi

AGPL-3.0-or-later. Komponen UI berdasarkan [rust-ui](https://github.com/rust-ui/ui) (MIT), penampil PDF berdasarkan [PDF.js](https://mozilla.github.io/pdf.js/) (Apache-2.0), font [Inter](https://rsms.me/inter/) (OFL).
