<div align="center">

<img src="../static/icon.png" width="96" alt="nrzPaper">

# nrzPaper

**A Rust-powered document management tool.**

[English](../README.md) · [Deutsch](README.de.md) · [Français](README.fr.md) · [Italiano](README.it.md) · [Español](README.es.md) · [Português](README.pt.md) · [Nederlands](README.nl.md) · [中文](README.zh.md) · [日本語](README.ja.md) · [Polski](README.pl.md) · [Русский](README.ru.md) · [العربية](README.ar.md) · [한국어](README.ko.md) · [Türkçe](README.tr.md) · [Bahasa Indonesia](README.id.md) · [हिन्दी](README.hi.md) · [Tiếng Việt](README.vi.md) · [ไทย](README.th.md) · [Українська](README.uk.md) · [Čeština](README.cs.md) · [Română](README.ro.md) · [Magyar](README.hu.md) · [Ελληνικά](README.el.md) · [Svenska](README.sv.md) · [Norsk](README.nb.md) · [Dansk](README.da.md) · [Suomi](README.fi.md) · [עברית](README.he.md) · [বাংলা](README.bn.md) · [فارسی](README.fa.md)

![Zaman çizelgeli belgeler](screenshots/documents.png)

</div>

nrzPaper, harici veritabanı olmayan tek bir programdır. Dosyaların kendisi dahil her şey SQLCipher ile şifrelenmiş tek bir SQLite dosyasında yaşar. Tesseract ile OCR yapar, doğrudan ağ tarayıcılarından tarar, e-posta eklerini alır ve seçtiğiniz bir yapay zekânın başlık, etiket ve meta veri önermesini sağlar. Yerleşik asistan **Dieter** belgelerinizle ilgili soruları yanıtlar ve kaynakları bağlar. Web arayüzü sakin koyu temalı bir Rust/WebAssembly uygulamasıdır (Dioxus) ve telefonlarda da çalışır.

## Özellikler

| | |
|---|---|
| **Yakalama** | Sürükle-bırak, yükleme, **ağ tarayıcıları** (eSCL/AirScan: Canon, HP, Epson, Brother …), **e-posta kutuları** (IMAP, birden çok hesap), **izlenen klasörler** (ör. NAS'taki tarama klasörü), REST API |
| **Gelen kutusu** | Taramalar, e-postalar, klasör içe aktarımları ve yüklemeler incelemeyi bekler: YZ ad, etiket ve alanları önerir; tek tek veya **hepsini birden** onaylarsınız. Kaynağa göre filtreleyin (tarayıcı, her posta kutusu, her klasör, yükleme). |
| **Tanıma** | Arka planda Tesseract ve Poppler, taranmış PDF'ler ve fotoğraflar için de; önekler, ifadeler ve filtrelerle tam metin arama (`tag:`, `-tag:`, `type:`, `created:>2026-06-01`, …) |
| **Düzenleme** | Renkli etiketler, **etiketlerden oluşan klasörler** (iç içe), **etiketleme kuralları** ("Northwind içeriyorsa → Fatura"), özel **özellikler** (metin, sayı, tarih, evet/hayır, seçim, çoklu seçim, kullanıcı, belge bağlantısı) |
| **Zaman çizelgesi** | Belgelerin yanında dikey zaman çizelgesi: en üstte bugün, altında e-postalı her gün ve aralarındaki sessiz boşluklar ("12 gün belge yok"). Etiket, tür ve metne göre filtreleyin. |
| **Hatırlatıcılar** | Herhangi bir belgeye notla "10 gün sonra hatırlat"; zamanı gelen hatırlatıcılar kenar çubuğunda ve zaman çizelgesinin üstünde görünür. |
| **Yazdırma** | Sunucunun bulduğu herhangi bir **IPP ağ yazıcısında** (AirPrint, IPP Everywhere, Mopria, CUPS) belge yazdırın. PDF'ler doğrudan veya yalnızca JPEG kabul eden yazıcılara sayfa görüntüsü olarak gider. |
| **YZ, her sağlayıcı** | OpenRouter (ücretsiz modeller yerleşik), OpenAI, Anthropic, Google Gemini, Mistral, Groq, xAI, DeepSeek, Together, Fireworks, Perplexity, Cohere, NVIDIA, Hugging Face, Qwen, Azure OpenAI, **Ollama, LM Studio, vLLM/LocalAI** veya OpenAI uyumlu herhangi bir URL. Yalnızca metin modelleri görüntü yerine OCR metnini alır. |
| **Dieter** | Sohbet asistanı (sağ altta): *"Sigorta numaram nedir?"*, *"2026'daki tüm Fabrikam bordroları"*. Arar, eşleşen bölümleri okur ve belgelere bağlantılarla yanıtlar. |
| **Ekipler** | Birden çok kuruluş, **kuruluş başına birden çok hesap** (davet, roller: sahip, yönetici, üye; sahiplik devri), izinli API anahtarları, web kancaları (Standard Webhooks imzaları), genel paylaşım bağlantıları |
| **Diller** | English, Deutsch, Français, Italiano, Español, Português, Nederlands (diğer diller yakında) |
| **Varsayılan olarak güvenli** | Tek şifreli dosya, Argon2 parolalar, satır içi betiksiz CSP, CSRF koruması, dosyalar yalnızca API üzerinden |

## Ekran görüntüleri

| | |
|---|---|
| ![Dieter, asistan](screenshots/dieter.png) | ![YZ önerileriyle gelen kutusu](screenshots/inbox.png) |
| **Dieter** kaynaklarla yanıtlar | **Gelen kutusu**: incele ve onayla, kaynağa göre filtrele |
| ![Belge görünümü](screenshots/document.png) | ![YZ sağlayıcıları](screenshots/settings-ai.png) |
| **Belge**: PDF görüntüleyici, etiketler, hatırlatıcılar, yazdırma | **YZ ve modeller**: her sağlayıcı, yerel veya bulut |
| ![Hatırlatıcılar](screenshots/reminders.png) | ![Üyeler](screenshots/members.png) |
| **Hatırlatıcılar** | **Üyeler** ve davetler |

## Hızlı başlangıç

Docker gerekir. İmaj programı, web arayüzünü, Tesseract'ı (Almanca, İngilizce) ve Poppler'ı içerir.

```bash
git clone https://github.com/nicorenezweers/nrzPaper.git
cd nrzPaper
docker build -t nrzpaper .
docker run -d --name nrzpaper -p 8320:8320 -v nrzpaper-data:/data nrzpaper
```

<http://localhost:8320> adresini açın. Oluşturduğunuz ilk hesap yönetici olur. Ardından, *Ayarlar → Kayıt ve kullanıcılar* altında açana kadar kayıt kapalıdır; orada başkaları için de hesap oluşturabilirsiniz.

### Docker Compose

```yaml
services:
  nrzpaper:
    build: .                      # veya: image: ghcr.io/nicorenezweers/nrzpaper:1.0.0
    container_name: nrzpaper
    restart: unless-stopped
    ports:
      - "8320:8320"
    volumes:
      - nrzpaper-data:/data       # şifreli veritabanı + anahtar dosyası
      - /srv/scans:/scans         # isteğe bağlı: klasör gelen kutusu için klasör
    environment:
      TZ: Europe/Zurich
      # NRZPAPER_DB_KEY: "a-long-secret"     # aksi halde /data/nrzpaper.key oluşturulur
      # NRZPAPER_COOKIE_SECURE: "true"       # HTTPS üzerinden sunulduğunda
      # NRZPAPER_AI_PROVIDER: "openai"       # veya web arayüzünde ayarlayın
      # NRZPAPER_AI_KEY: "sk-…"

volumes:
  nrzpaper-data:
```

```bash
docker compose up -d
```

> **`/data` dizinini yedekleyin.** İçinde `nrzpaper.db` (her şey, şifreli) ve `nrzpaper.key` bulunur. Anahtar olmadan veritabanı açılamaz. `NRZPAPER_DB_KEY` ayarladıysanız bunun yerine o değeri güvende tutun.

### Her iki mimari

```bash
docker buildx build --platform linux/amd64,linux/arm64 -t <registry>/nrzpaper:1.0.0 --push .
```

Rust, emüle edilmeden çapraz derlenir; bu nedenle arm64 derlemesi x86 makinede hızlıdır ve tersi de geçerlidir.

## İlk adımlar

1. Birkaç PDF veya fotoğraf **yükleyin**: metin arka planda tanınır.
2. **Etiketler**: *Fatura* veya *Sigorta* gibi etiketler oluşturun ve açıklama ekleyin. YZ bunu etiketleri atamak için kullanır.
3. **YZ** (*Ayarlar → YZ ve modeller*): bir sağlayıcı seçin, anahtar yapıştırın (veya yerleşik ücretsiz OpenRouter modellerini veya yerel Ollama'yı kullanın), ardından *Modelleri yükle* ve *Bağlantıyı test et*.
4. **Tarayıcı** (*Profil → Tarayıcı*): *Ağı ara* eSCL/AirScan tarayıcılarını bulur; ardından kenar çubuğunda *Tara*.
5. **E-posta** (*Profil → E-posta gelen kutusu*): bir veya daha fazla IMAP posta kutusu ekleyin; eklerdeki PDF'ler ve görüntüler gelen kutusuna düşer.
6. **Klasör gelen kutusu** (*Ayarlar → Klasör gelen kutusu*, yöneticiler): `/scans` gibi bir sunucu klasörünü izleyin (önce kapsayıcıya bağlayın).
7. **Yazıcılar** (*Ayarlar → Yazıcılar*, yöneticiler): *Ağı ara* veya `ipp://printer/ipp/print` girin; ardından her üye belge sayfasından yazdırabilir.

## Yapılandırma

Her ayar `NRZPAPER_` önekli bir ortam değişkeni (veya `Rocket.toml`) olarak verilebilir.

| Değişken | Varsayılan | Anlamı |
|---|---|---|
| `NRZPAPER_DATA_DIR` | `/data` (imaj) | Veritabanı, anahtar dosyası ve alım klasörleri |
| `NRZPAPER_DB_KEY` | – | Veritabanı anahtarı; ayarlanmazsa veri klasöründe `nrzpaper.key` oluşturulur |
| `NRZPAPER_PORT` / `NRZPAPER_ADDRESS` | `8320` / `0.0.0.0` | Sunucunun dinlediği yer |
| `NRZPAPER_COOKIE_SECURE` | `false` | HTTPS arkasında `true` yapın |
| `NRZPAPER_REGISTRATION_OPEN` | `false` | Kendi kendine kayda izin ver (arayüzde de değiştirilebilir) |
| `NRZPAPER_OCR_LANGUAGES` | `deu+eng` | Tesseract dilleri (imaj `deu` ve `eng` içerir) |
| `NRZPAPER_OCR_WORKERS` | `2` | Paralel OCR işleri |
| `NRZPAPER_TRASH_RETENTION_DAYS` | `30` | Kalıcı silmeden önce çöp kutusunda gün |
| `NRZPAPER_INBOX_KEEP_DAYS` | `7` | Onaylanmamış gelen kutusu öğelerinin saklandığı gün |
| `NRZPAPER_SCANNER_URL` | – | Yeni kullanıcılar için önceden ayarlanmış tarayıcı, ör. `http://192.168.1.50` |
| `NRZPAPER_SCAN_SUBNETS` | `192.168.0.0/24,192.168.1.0/24` | Tarayıcı ve yazıcı aranan ağlar |
| `NRZPAPER_AI_PROVIDER` | `openrouter` | `openai`, `anthropic`, `gemini`, `mistral`, `groq`, `ollama`, `custom`, … |
| `NRZPAPER_AI_KEY` / `NRZPAPER_AI_URL` / `NRZPAPER_AI_MODEL` | – | O sağlayıcı için anahtar, temel URL ve model |
| `NRZPAPER_OPENROUTER_KEY` | – | OpenRouter anahtarı (yukarıdakine alternatif) |
| `NRZPAPER_LOG` | `info` | Günlük filtresi, ör. `debug` |

Web arayüzünde yapılan ayarlar (YZ sağlayıcısı, anahtarlar, modeller, kayıt) şifreli veritabanında saklanır ve ortam değişkenlerinden önce gelir.

### YZ sağlayıcıları

| Sağlayıcı | Adres | Notlar |
|---|---|---|
| OpenRouter | yerleşik | Ücretsiz görüntü modelleri; kendi anahtarınız isteğe bağlı |
| OpenAI, Anthropic, Google Gemini, Mistral, Groq, xAI, DeepSeek, Together, Fireworks, Perplexity, Cohere, NVIDIA, Hugging Face, Qwen | yerleşik | Sağlayıcıdan API anahtarı |
| Azure OpenAI | `https://NAME.openai.azure.com/openai/v1` | Model = dağıtım adı |
| Ollama / LM Studio / vLLM / LocalAI | `http://host:11434/v1` vb. | Anahtar gerekmez |
| OpenAI uyumlu her şey | kendi URL'niz | Anahtar isteğe bağlı |

Görüntü okuyamayan modeller: *"Model görüntüleri okuyamıyor – tanınan metni gönder"* seçeneğini işaretleyin. PDF kabul etmeyen sağlayıcılara otomatik olarak ilk sayfalar görüntü olarak verilir. Önerilen başlık ve özetlerin dili *Öneri dili* altında ayarlanır.

## REST API

*Profil → API anahtarları* altında bir API anahtarı oluşturun (izinleri seçin). Tüm uç noktalar `/api/v1` altındadır ve `Authorization: Bearer np_…` kabul eder.

```bash
TOKEN=np_your_key
BASE=http://localhost:8320/api/v1

# kuruluşlar
curl -H "Authorization: Bearer $TOKEN" $BASE/organizations

# yükleme (birkaç -F ile aynı anda birkaç dosya)
curl -H "Authorization: Bearer $TOKEN" -F file=@invoice.pdf \
     $BASE/organizations/$ORG/documents

# arama: arayüzdekiyle aynı sözdizimi
curl -H "Authorization: Bearer $TOKEN" -G $BASE/organizations/$ORG/documents \
     --data-urlencode 'searchQuery=northwind tag:invoice created:2026'

# dosyayı indir
curl -H "Authorization: Bearer $TOKEN" -o out.pdf \
     "$BASE/organizations/$ORG/documents/$DOC/file?download=true"

# etiket ekle
curl -H "Authorization: Bearer $TOKEN" -H "Content-Type: application/json" \
     -d '{"tagId":"'$TAG'"}' $BASE/organizations/$ORG/documents/$DOC/tags

# 7 gün geçerli genel paylaşım bağlantısı
curl -H "Authorization: Bearer $TOKEN" -H "Content-Type: application/json" \
     -d '{"expiresInDays":7}' $BASE/organizations/$ORG/documents/$DOC/share-links

# durum kontrolü (anahtar gerekmez)
curl $BASE/health
```

Web kancaları (*Ayarlar → Web kancaları*) `document:created`, `document:updated`, `document:deleted`, `document:restored`, `document:tag:added` ve `document:tag:removed` için imzalı JSON (`webhook-id`, `webhook-timestamp`, `webhook-signature`) gönderir.

Alım klasörü `data/ingestion/<organization-id>/` dosyaları doğrudan belge olarak içe aktarır. İzlenen klasörler (*Klasör gelen kutusu*) önce inceleme için gelen kutusuna gönderir.

## Geliştirme

```bash
# arka uç (Rust 1.85+, OCR için Tesseract ve Poppler)
cargo run                         # http://localhost:8320
cargo test

# sıcak yeniden yüklemeli web arayüzü (dx = dioxus-cli 0.7)
rustup target add wasm32-unknown-unknown
cargo install dioxus-cli --version 0.7.10 --locked
cd ui && dx serve                 # http://localhost:8080, /api'yi :8320'ye yönlendirir

# arayüzü programa göm
scripts/ui.sh && cargo build --release
```

Çeviriler `locales/<language>.json` içinde (Almanca kaynak metin → çeviri). `scripts/i18n_extract.py`, `locales/strings.json` dosyasını yeniler ve dil başına eksik girdileri bildirir.

Mimari ayrıntıları: [ARCHITECTURE.md](../ARCHITECTURE.md).

## Lisans

AGPL-3.0-or-later. Arayüz bileşenleri [rust-ui](https://github.com/rust-ui/ui) (MIT), PDF görüntüleyici [PDF.js](https://mozilla.github.io/pdf.js/) (Apache-2.0), yazı tipi [Inter](https://rsms.me/inter/) (OFL) temellidir.
