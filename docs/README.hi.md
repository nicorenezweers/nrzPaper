<div align="center">

<img src="../static/icon.png" width="96" alt="nrzPaper">

# nrzPaper

**Rust में स्वयं-होस्टेड दस्तावेज़ प्रबंधन: अपने कागज़ात स्कैन करें, पहचानें, टैग करें और खोजें — एक एन्क्रिप्टेड फ़ाइल में।**

[English](../README.md) · [Deutsch](README.de.md) · [Français](README.fr.md) · [Italiano](README.it.md) · [Español](README.es.md) · [Português](README.pt.md) · [Nederlands](README.nl.md) · [中文](README.zh.md) · [日本語](README.ja.md) · [Polski](README.pl.md) · [Русский](README.ru.md) · [العربية](README.ar.md) · [한국어](README.ko.md) · [Türkçe](README.tr.md) · [Bahasa Indonesia](README.id.md) · [हिन्दी](README.hi.md) · [Tiếng Việt](README.vi.md) · [ไทย](README.th.md) · [Українська](README.uk.md) · [Čeština](README.cs.md) · [Română](README.ro.md) · [Magyar](README.hu.md) · [Ελληνικά](README.el.md) · [Svenska](README.sv.md) · [Norsk](README.nb.md) · [Dansk](README.da.md) · [Suomi](README.fi.md) · [עברית](README.he.md) · [বাংলা](README.bn.md) · [فارسی](README.fa.md)

![समयरेखा के साथ दस्तावेज़](screenshots/documents.png)

</div>

nrzPaper बिना किसी बाहरी डेटाबेस के एक ही प्रोग्राम है। फ़ाइलों सहित सब कुछ एक SQLCipher-एन्क्रिप्टेड SQLite फ़ाइल में रहता है। यह Tesseract से OCR करता है, नेटवर्क स्कैनर से सीधे स्कैन करता है, ईमेल अनुलग्नक लेता है, और आपकी पसंद का AI शीर्षक, टैग और मेटाडेटा सुझाता है। अंतर्निहित सहायक **Dieter** आपके दस्तावेज़ों के बारे में सवालों के जवाब देता है और स्रोतों को लिंक करता है। वेब इंटरफ़ेस शांत गहरे थीम वाला Rust/WebAssembly ऐप (Dioxus) है और फ़ोन पर भी काम करता है।

## विशेषताएँ

| | |
|---|---|
| **संग्रह** | ड्रैग और ड्रॉप, अपलोड, **नेटवर्क स्कैनर** (eSCL/AirScan: Canon, HP, Epson, Brother …), **ईमेल इनबॉक्स** (IMAP, कई खाते), **निगरानी किए गए फ़ोल्डर** (जैसे NAS पर स्कैन फ़ोल्डर), REST API |
| **इनबॉक्स** | स्कैन, मेल, फ़ोल्डर आयात और अपलोड समीक्षा की प्रतीक्षा करते हैं: AI नाम, टैग और फ़ील्ड सुझाता है, और आप एक-एक या **सभी को एक साथ** स्वीकृत करते हैं। स्रोत के अनुसार फ़िल्टर करें (स्कैनर, प्रत्येक मेलबॉक्स, प्रत्येक फ़ोल्डर, अपलोड)। |
| **पहचान** | पृष्ठभूमि में Tesseract और Poppler, स्कैन किए गए PDF और फ़ोटो के लिए भी; उपसर्गों, वाक्यांशों और फ़िल्टर के साथ पूर्ण-पाठ खोज (`tag:`, `-tag:`, `type:`, `created:>2026-06-01`, …) |
| **व्यवस्थित करें** | रंगीन टैग, **टैग से बने फ़ोल्डर** (नेस्टेड), **टैगिंग नियम** ("Northwind शामिल है → चालान"), कस्टम **गुण** (पाठ, संख्या, तिथि, हाँ/नहीं, चयन, बहु-चयन, उपयोगकर्ता, दस्तावेज़ लिंक) |
| **समयरेखा** | दस्तावेज़ों के बगल में ऊर्ध्वाधर समयरेखा: आज सबसे ऊपर, नीचे मेल के साथ प्रत्येक दिन और बीच के शांत अंतराल ("12 दिन बिना दस्तावेज़")। टैग, प्रकार और पाठ से फ़िल्टर करें। |
| **अनुस्मारक** | किसी भी दस्तावेज़ पर नोट के साथ "10 दिन में याद दिलाएँ"; देय अनुस्मारक साइडबार और समयरेखा के शीर्ष पर दिखते हैं। |
| **प्रिंट** | सर्वर द्वारा खोजे गए किसी भी **IPP नेटवर्क प्रिंटर** (AirPrint, IPP Everywhere, Mopria, CUPS) पर दस्तावेज़ प्रिंट करें। PDF सीधे जाते हैं, या केवल JPEG स्वीकार करने वाले प्रिंटर पर पृष्ठ छवियों के रूप में। |
| **AI, कोई भी प्रदाता** | OpenRouter (अंतर्निहित मुफ़्त मॉडल), OpenAI, Anthropic, Google Gemini, Mistral, Groq, xAI, DeepSeek, Together, Fireworks, Perplexity, Cohere, NVIDIA, Hugging Face, Qwen, Azure OpenAI, **Ollama, LM Studio, vLLM/LocalAI** या कोई भी OpenAI-संगत URL। केवल-पाठ मॉडल को छवियों के बजाय OCR पाठ मिलता है। |
| **Dieter** | चैट सहायक (नीचे दाएँ): *"मेरा बीमा नंबर क्या है?"*, *"2026 में Fabrikam के सभी वेतन पर्चियाँ"*। वह खोजता है, मेल खाते अंश पढ़ता है और दस्तावेज़ों के लिंक के साथ उत्तर देता है। |
| **टीमें** | कई संगठन, **प्रति संगठन कई खाते** (आमंत्रण, भूमिकाएँ: स्वामी, व्यवस्थापक, सदस्य; स्वामित्व हस्तांतरण), अनुमतियों वाली API कुंजियाँ, वेबहुक (Standard Webhooks हस्ताक्षर), सार्वजनिक साझा लिंक |
| **भाषाएँ** | English, Deutsch, Français, Italiano, Español, Português, Nederlands (अन्य भाषाएँ जल्द आ रही हैं) |
| **डिफ़ॉल्ट रूप से सुरक्षित** | एक एन्क्रिप्टेड फ़ाइल, Argon2 पासवर्ड, इनलाइन स्क्रिप्ट रहित CSP, CSRF सुरक्षा, फ़ाइलें केवल API के माध्यम से |

## स्क्रीनशॉट

| | |
|---|---|
| ![Dieter, सहायक](screenshots/dieter.png) | ![AI सुझावों के साथ इनबॉक्स](screenshots/inbox.png) |
| **Dieter** स्रोतों के साथ उत्तर देता है | **इनबॉक्स**: समीक्षा और स्वीकृति, स्रोत के अनुसार फ़िल्टर |
| ![दस्तावेज़ दृश्य](screenshots/document.png) | ![AI प्रदाता](screenshots/settings-ai.png) |
| **दस्तावेज़**: PDF व्यूअर, टैग, अनुस्मारक, प्रिंट | **AI और मॉडल**: हर प्रदाता, स्थानीय या क्लाउड |
| ![अनुस्मारक](screenshots/reminders.png) | ![सदस्य](screenshots/members.png) |
| **अनुस्मारक** | **सदस्य** और आमंत्रण |

## त्वरित शुरुआत

आपको Docker चाहिए। इमेज में प्रोग्राम, वेब इंटरफ़ेस, Tesseract (जर्मन, अंग्रेज़ी) और Poppler है।

```bash
git clone https://github.com/nicorenezweers/nrzPaper.git
cd nrzPaper
docker build -t nrzpaper .
docker run -d --name nrzpaper -p 8320:8320 -v nrzpaper-data:/data nrzpaper
```

<http://localhost:8320> खोलें। आपके द्वारा बनाया गया पहला खाता व्यवस्थापक बनता है। उसके बाद पंजीकरण बंद रहता है जब तक आप इसे *सेटिंग्स → पंजीकरण और उपयोगकर्ता* में नहीं खोलते, जहाँ आप दूसरों के लिए खाते भी बना सकते हैं।

### Docker Compose

```yaml
services:
  nrzpaper:
    build: .                      # या: image: ghcr.io/nicorenezweers/nrzpaper:1.0.0
    container_name: nrzpaper
    restart: unless-stopped
    ports:
      - "8320:8320"
    volumes:
      - nrzpaper-data:/data       # एन्क्रिप्टेड डेटाबेस + कुंजी फ़ाइल
      - /srv/scans:/scans         # वैकल्पिक: फ़ोल्डर इनबॉक्स के लिए फ़ोल्डर
    environment:
      TZ: Europe/Zurich
      # NRZPAPER_DB_KEY: "a-long-secret"     # अन्यथा /data/nrzpaper.key बनाया जाता है
      # NRZPAPER_COOKIE_SECURE: "true"       # HTTPS के माध्यम से परोसते समय
      # NRZPAPER_AI_PROVIDER: "openai"       # या वेब इंटरफ़ेस में सेट करें
      # NRZPAPER_AI_KEY: "sk-…"

volumes:
  nrzpaper-data:
```

```bash
docker compose up -d
```

> **`/data` का बैकअप लें।** इसमें `nrzpaper.db` (सब कुछ, एन्क्रिप्टेड) और `nrzpaper.key` है। कुंजी के बिना डेटाबेस नहीं खुल सकता। यदि आपने `NRZPAPER_DB_KEY` सेट किया है, तो उस मान को सुरक्षित रखें।

### दोनों आर्किटेक्चर

```bash
docker buildx build --platform linux/amd64,linux/arm64 -t <registry>/nrzpaper:1.0.0 --push .
```

Rust को एम्युलेट नहीं बल्कि क्रॉस-कंपाइल किया जाता है, इसलिए arm64 बिल्ड x86 मशीन पर तेज़ है और इसके विपरीत भी।

## पहले कदम

1. कुछ PDF या फ़ोटो **अपलोड करें**: पाठ पृष्ठभूमि में पहचाना जाता है।
2. **टैग**: *चालान* या *बीमा* जैसे टैग बनाएँ और विवरण जोड़ें। AI इसका उपयोग टैग असाइन करने के लिए करता है।
3. **AI** (*सेटिंग्स → AI और मॉडल*): प्रदाता चुनें, कुंजी चिपकाएँ (या अंतर्निहित मुफ़्त OpenRouter मॉडल, या स्थानीय Ollama उपयोग करें), फिर *मॉडल लोड करें* और *कनेक्शन परीक्षण करें*।
4. **स्कैनर** (*प्रोफ़ाइल → स्कैनर*): *नेटवर्क खोजें* eSCL/AirScan स्कैनर ढूँढता है; फिर साइडबार में *स्कैन करें*।
5. **ईमेल** (*प्रोफ़ाइल → ईमेल इनबॉक्स*): एक या अधिक IMAP मेलबॉक्स जोड़ें; अनुलग्नकों से PDF और छवियाँ इनबॉक्स में आती हैं।
6. **फ़ोल्डर इनबॉक्स** (*सेटिंग्स → फ़ोल्डर इनबॉक्स*, व्यवस्थापक): `/scans` जैसे सर्वर फ़ोल्डर की निगरानी करें (पहले इसे कंटेनर में माउंट करें)।
7. **प्रिंटर** (*सेटिंग्स → प्रिंटर*, व्यवस्थापक): *नेटवर्क खोजें* या `ipp://printer/ipp/print` दर्ज करें; फिर हर सदस्य दस्तावेज़ पृष्ठ से प्रिंट कर सकता है।

## कॉन्फ़िगरेशन

हर सेटिंग `NRZPAPER_` उपसर्ग वाले पर्यावरण चर (या `Rocket.toml`) के रूप में दी जा सकती है।

| चर | डिफ़ॉल्ट | अर्थ |
|---|---|---|
| `NRZPAPER_DATA_DIR` | `/data` (इमेज) | डेटाबेस, कुंजी फ़ाइल और अंतर्ग्रहण फ़ोल्डर |
| `NRZPAPER_DB_KEY` | – | डेटाबेस कुंजी; यदि सेट नहीं है, तो डेटा फ़ोल्डर में `nrzpaper.key` बनाया जाता है |
| `NRZPAPER_PORT` / `NRZPAPER_ADDRESS` | `8320` / `0.0.0.0` | सर्वर कहाँ सुनता है |
| `NRZPAPER_COOKIE_SECURE` | `false` | HTTPS के पीछे `true` सेट करें |
| `NRZPAPER_REGISTRATION_OPEN` | `false` | स्व-पंजीकरण की अनुमति दें (UI में भी बदला जा सकता है) |
| `NRZPAPER_OCR_LANGUAGES` | `deu+eng` | Tesseract भाषाएँ (इमेज में `deu` और `eng` हैं) |
| `NRZPAPER_OCR_WORKERS` | `2` | समानांतर OCR कार्य |
| `NRZPAPER_TRASH_RETENTION_DAYS` | `30` | अंतिम विलोपन से पहले कचरा पेटी में दिन |
| `NRZPAPER_INBOX_KEEP_DAYS` | `7` | अस्वीकृत इनबॉक्स आइटम कितने दिन रखे जाते हैं |
| `NRZPAPER_SCANNER_URL` | – | नए उपयोगकर्ताओं के लिए प्रीसेट स्कैनर, जैसे `http://192.168.1.50` |
| `NRZPAPER_SCAN_SUBNETS` | `192.168.0.0/24,192.168.1.0/24` | स्कैनर और प्रिंटर खोजने के लिए नेटवर्क |
| `NRZPAPER_AI_PROVIDER` | `openrouter` | `openai`, `anthropic`, `gemini`, `mistral`, `groq`, `ollama`, `custom`, … |
| `NRZPAPER_AI_KEY` / `NRZPAPER_AI_URL` / `NRZPAPER_AI_MODEL` | – | उस प्रदाता के लिए कुंजी, आधार URL और मॉडल |
| `NRZPAPER_OPENROUTER_KEY` | – | OpenRouter कुंजी (ऊपर का विकल्प) |
| `NRZPAPER_LOG` | `info` | लॉग फ़िल्टर, जैसे `debug` |

वेब इंटरफ़ेस में की गई सेटिंग्स (AI प्रदाता, कुंजियाँ, मॉडल, पंजीकरण) एन्क्रिप्टेड डेटाबेस में संग्रहीत होती हैं और पर्यावरण से प्राथमिकता लेती हैं।

### AI प्रदाता

| प्रदाता | पता | नोट्स |
|---|---|---|
| OpenRouter | अंतर्निहित | मुफ़्त विज़न मॉडल; अपनी कुंजी वैकल्पिक |
| OpenAI, Anthropic, Google Gemini, Mistral, Groq, xAI, DeepSeek, Together, Fireworks, Perplexity, Cohere, NVIDIA, Hugging Face, Qwen | अंतर्निहित | प्रदाता से API कुंजी |
| Azure OpenAI | `https://NAME.openai.azure.com/openai/v1` | मॉडल = परिनियोजन नाम |
| Ollama / LM Studio / vLLM / LocalAI | `http://host:11434/v1` आदि | कुंजी की आवश्यकता नहीं |
| कोई भी OpenAI-संगत | आपका URL | कुंजी वैकल्पिक |

ऐसे मॉडल जो छवियाँ नहीं पढ़ सकते: *"मॉडल छवियाँ नहीं पढ़ सकता – पहचाना गया पाठ भेजें"* पर टिक करें। जो प्रदाता PDF स्वीकार नहीं करते उन्हें स्वचालित रूप से पहले पृष्ठ छवियों के रूप में मिलते हैं। सुझाए गए शीर्षक और सारांश की भाषा *सुझाव की भाषा* में सेट की जाती है।

## REST API

*प्रोफ़ाइल → API कुंजियाँ* में API कुंजी बनाएँ (अनुमतियाँ चुनें)। सभी एंडपॉइंट `/api/v1` के अंतर्गत हैं और `Authorization: Bearer np_…` स्वीकार करते हैं।

```bash
TOKEN=np_your_key
BASE=http://localhost:8320/api/v1

# संगठन
curl -H "Authorization: Bearer $TOKEN" $BASE/organizations

# अपलोड (कई -F के साथ एक साथ कई फ़ाइलें)
curl -H "Authorization: Bearer $TOKEN" -F file=@invoice.pdf \
     $BASE/organizations/$ORG/documents

# खोज: UI जैसा ही सिंटैक्स
curl -H "Authorization: Bearer $TOKEN" -G $BASE/organizations/$ORG/documents \
     --data-urlencode 'searchQuery=northwind tag:invoice created:2026'

# फ़ाइल डाउनलोड करें
curl -H "Authorization: Bearer $TOKEN" -o out.pdf \
     "$BASE/organizations/$ORG/documents/$DOC/file?download=true"

# टैग जोड़ें
curl -H "Authorization: Bearer $TOKEN" -H "Content-Type: application/json" \
     -d '{"tagId":"'$TAG'"}' $BASE/organizations/$ORG/documents/$DOC/tags

# 7 दिनों के लिए वैध सार्वजनिक साझा लिंक
curl -H "Authorization: Bearer $TOKEN" -H "Content-Type: application/json" \
     -d '{"expiresInDays":7}' $BASE/organizations/$ORG/documents/$DOC/share-links

# स्वास्थ्य जाँच (कुंजी की आवश्यकता नहीं)
curl $BASE/health
```

वेबहुक (*सेटिंग्स → वेबहुक*) `document:created`, `document:updated`, `document:deleted`, `document:restored`, `document:tag:added` और `document:tag:removed` के लिए हस्ताक्षरित JSON (`webhook-id`, `webhook-timestamp`, `webhook-signature`) भेजते हैं।

अंतर्ग्रहण फ़ोल्डर `data/ingestion/<organization-id>/` फ़ाइलों को सीधे दस्तावेज़ों के रूप में आयात करता है। निगरानी किए गए फ़ोल्डर (*फ़ोल्डर इनबॉक्स*) पहले उन्हें समीक्षा के लिए इनबॉक्स में भेजते हैं।

## विकास

```bash
# बैकएंड (Rust 1.85+, OCR के लिए Tesseract और Poppler)
cargo run                         # http://localhost:8320
cargo test

# हॉट रीलोड वाला वेब इंटरफ़ेस (dx = dioxus-cli 0.7)
rustup target add wasm32-unknown-unknown
cargo install dioxus-cli --version 0.7.10 --locked
cd ui && dx serve                 # http://localhost:8080, /api को :8320 पर प्रॉक्सी करता है

# इंटरफ़ेस को प्रोग्राम में एम्बेड करें
scripts/ui.sh && cargo build --release
```

अनुवाद `locales/<language>.json` में हैं (जर्मन स्रोत पाठ → अनुवाद)। `scripts/i18n_extract.py` `locales/strings.json` को ताज़ा करता है और प्रति भाषा अनुपस्थित प्रविष्टियों की रिपोर्ट करता है।

आर्किटेक्चर विवरण: [ARCHITECTURE.md](../ARCHITECTURE.md)।

## लाइसेंस

AGPL-3.0-or-later. UI घटक [rust-ui](https://github.com/rust-ui/ui) (MIT) पर आधारित हैं, PDF व्यूअर [PDF.js](https://mozilla.github.io/pdf.js/) (Apache-2.0) पर, फ़ॉन्ट [Inter](https://rsms.me/inter/) (OFL) है।
