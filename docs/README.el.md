<div align="center">

<img src="../static/icon.png" width="96" alt="nrzPaper">

# nrzPaper

**Αυτο-φιλοξενούμενη διαχείριση εγγράφων σε Rust: σαρώστε, αναγνωρίστε, προσθέστε ετικέτες και βρείτε τα χαρτιά σας — κρυπτογραφημένα σε ένα αρχείο.**

[English](../README.md) · [Deutsch](README.de.md) · [Français](README.fr.md) · [Italiano](README.it.md) · [Español](README.es.md) · [Português](README.pt.md) · [Nederlands](README.nl.md) · [中文](README.zh.md) · [日本語](README.ja.md) · [Polski](README.pl.md) · [Русский](README.ru.md) · [العربية](README.ar.md) · [한국어](README.ko.md) · [Türkçe](README.tr.md) · [Bahasa Indonesia](README.id.md) · [हिन्दी](README.hi.md) · [Tiếng Việt](README.vi.md) · [ไทย](README.th.md) · [Українська](README.uk.md) · [Čeština](README.cs.md) · [Română](README.ro.md) · [Magyar](README.hu.md) · [Ελληνικά](README.el.md) · [Svenska](README.sv.md) · [Norsk](README.nb.md) · [Dansk](README.da.md) · [Suomi](README.fi.md) · [עברית](README.he.md) · [বাংলা](README.bn.md) · [فارسی](README.fa.md)

![Έγγραφα με χρονολόγιο](screenshots/documents.png)

</div>

Το nrzPaper είναι ένα μόνο πρόγραμμα χωρίς εξωτερική βάση δεδομένων. Όλα βρίσκονται σε ένα αρχείο SQLite κρυπτογραφημένο με SQLCipher, συμπεριλαμβανομένων των ίδιων των αρχείων. Κάνει OCR με το Tesseract, σαρώνει απευθείας από σαρωτές δικτύου, λαμβάνει συνημμένα email και επιτρέπει σε μια AI της επιλογής σας να προτείνει τίτλους, ετικέτες και μεταδεδομένα. Ο ενσωματωμένος βοηθός **Dieter** απαντά σε ερωτήσεις για τα έγγραφά σας και συνδέει τις πηγές. Η διεπαφή ιστού είναι μια εφαρμογή Rust/WebAssembly (Dioxus) με ήρεμο σκούρο θέμα και λειτουργεί και σε κινητά.

## Χαρακτηριστικά

| | |
|---|---|
| **Σύλληψη** | Μεταφορά και απόθεση, μεταφόρτωση, **σαρωτές δικτύου** (eSCL/AirScan: Canon, HP, Epson, Brother …), **γραμματοκιβώτια email** (IMAP, πολλοί λογαριασμοί), **παρακολουθούμενοι φάκελοι** (π.χ. ο φάκελος σάρωσης στο NAS), REST API |
| **Εισερχόμενα** | Σαρώσεις, email, εισαγωγές φακέλων και μεταφορτώσεις περιμένουν έλεγχο: η AI προτείνει όνομα, ετικέτες και πεδία και εσείς εγκρίνετε ένα-ένα ή **όλα μαζί**. Φιλτράρισμα κατά πηγή (σαρωτής, κάθε γραμματοκιβώτιο, κάθε φάκελος, μεταφόρτωση). |
| **Αναγνώριση** | Tesseract και Poppler στο παρασκήνιο, και για σαρωμένα PDF και φωτογραφίες· αναζήτηση πλήρους κειμένου με προθέματα, φράσεις και φίλτρα (`tag:`, `-tag:`, `type:`, `created:>2026-06-01`, …) |
| **Οργάνωση** | Χρωματιστές ετικέτες, **φάκελοι από ετικέτες** (φωλιασμένοι), **κανόνες ετικετών** («περιέχει Northwind → Τιμολόγιο»), προσαρμοσμένες **ιδιότητες** (κείμενο, αριθμός, ημερομηνία, ναι/όχι, επιλογή, πολλαπλή επιλογή, χρήστης, σύνδεσμος εγγράφου) |
| **Χρονολόγιο** | Κάθετο χρονολόγιο δίπλα στα έγγραφα: σήμερα στην κορυφή, από κάτω κάθε μέρα με email και τα ήσυχα κενά ανάμεσά τους («12 ημέρες χωρίς έγγραφα»). Φιλτράρισμα κατά ετικέτα, τύπο και κείμενο. |
| **Υπενθυμίσεις** | «Υπενθύμισέ μου σε 10 ημέρες» σε οποιοδήποτε έγγραφο, με σημείωση· οι ληξιπρόθεσμες υπενθυμίσεις εμφανίζονται στην πλαϊνή γραμμή και στην κορυφή του χρονολογίου. |
| **Εκτύπωση** | Εκτυπώστε έγγραφα σε οποιονδήποτε **εκτυπωτή δικτύου IPP** (AirPrint, IPP Everywhere, Mopria, CUPS) βρει ο διακομιστής. Τα PDF πηγαίνουν απευθείας ή ως εικόνες σελίδων σε εκτυπωτές που δέχονται μόνο JPEG. |
| **AI, οποιοσδήποτε πάροχος** | OpenRouter (ενσωματωμένα δωρεάν μοντέλα), OpenAI, Anthropic, Google Gemini, Mistral, Groq, xAI, DeepSeek, Together, Fireworks, Perplexity, Cohere, NVIDIA, Hugging Face, Qwen, Azure OpenAI, **Ollama, LM Studio, vLLM/LocalAI** ή οποιοδήποτε URL συμβατό με OpenAI. Τα μοντέλα μόνο κειμένου λαμβάνουν το κείμενο OCR αντί για εικόνες. |
| **Dieter** | Βοηθός συνομιλίας (κάτω δεξιά): *«Ποιος είναι ο αριθμός ασφάλειάς μου;»*, *«Όλα τα εκκαθαριστικά από τη Fabrikam το 2026»*. Αναζητά, διαβάζει τα σχετικά αποσπάσματα και απαντά με συνδέσμους στα έγγραφα. |
| **Ομάδες** | Πολλοί οργανισμοί, **πολλοί λογαριασμοί ανά οργανισμό** (πρόσκληση, ρόλοι: ιδιοκτήτης, διαχειριστής, μέλος· μεταβίβαση ιδιοκτησίας), κλειδιά API με δικαιώματα, webhooks (υπογραφές Standard Webhooks), δημόσιοι σύνδεσμοι κοινής χρήσης |
| **Γλώσσες** | English, Deutsch, Français, Italiano, Español, Português, Nederlands (περισσότερες γλώσσες σύντομα) |
| **Ασφαλές εξ ορισμού** | Ένα κρυπτογραφημένο αρχείο, κωδικοί Argon2, CSP χωρίς inline scripts, προστασία CSRF, αρχεία μόνο μέσω API |

## Στιγμιότυπα

| | |
|---|---|
| ![Dieter, ο βοηθός](screenshots/dieter.png) | ![Εισερχόμενα με προτάσεις AI](screenshots/inbox.png) |
| **Dieter** απαντά με πηγές | **Εισερχόμενα**: έλεγχος και έγκριση, φιλτράρισμα κατά πηγή |
| ![Προβολή εγγράφου](screenshots/document.png) | ![Πάροχοι AI](screenshots/settings-ai.png) |
| **Έγγραφο**: προβολή PDF, ετικέτες, υπενθυμίσεις, εκτύπωση | **AI και μοντέλα**: κάθε πάροχος, τοπικός ή cloud |
| ![Υπενθυμίσεις](screenshots/reminders.png) | ![Μέλη](screenshots/members.png) |
| **Υπενθυμίσεις** | **Μέλη** και προσκλήσεις |

## Γρήγορη εκκίνηση

Χρειάζεστε Docker. Η εικόνα περιέχει το πρόγραμμα, τη διεπαφή ιστού, το Tesseract (γερμανικά, αγγλικά) και το Poppler.

```bash
git clone https://github.com/nicorenezweers/nrzPaper.git
cd nrzPaper
docker build -t nrzpaper .
docker run -d --name nrzpaper -p 8320:8320 -v nrzpaper-data:/data nrzpaper
```

Ανοίξτε το <http://localhost:8320>. Ο πρώτος λογαριασμός που δημιουργείτε γίνεται διαχειριστής. Μετά από αυτό, η εγγραφή είναι κλειστή μέχρι να την ανοίξετε στο *Ρυθμίσεις → Εγγραφή και χρήστες*, όπου μπορείτε επίσης να δημιουργήσετε λογαριασμούς για άλλους.

### Docker Compose

```yaml
services:
  nrzpaper:
    build: .                      # ή: image: ghcr.io/nicorenezweers/nrzpaper:1.0.0
    container_name: nrzpaper
    restart: unless-stopped
    ports:
      - "8320:8320"
    volumes:
      - nrzpaper-data:/data       # κρυπτογραφημένη βάση δεδομένων + αρχείο κλειδιού
      - /srv/scans:/scans         # προαιρετικό: φάκελος για τα εισερχόμενα φακέλων
    environment:
      TZ: Europe/Zurich
      # NRZPAPER_DB_KEY: "a-long-secret"     # αλλιώς δημιουργείται το /data/nrzpaper.key
      # NRZPAPER_COOKIE_SECURE: "true"       # όταν σερβίρεται μέσω HTTPS
      # NRZPAPER_AI_PROVIDER: "openai"       # ή ορίστε το στη διεπαφή ιστού
      # NRZPAPER_AI_KEY: "sk-…"

volumes:
  nrzpaper-data:
```

```bash
docker compose up -d
```

> **Δημιουργήστε αντίγραφα ασφαλείας του `/data`.** Περιέχει το `nrzpaper.db` (όλα, κρυπτογραφημένα) και το `nrzpaper.key`. Χωρίς το κλειδί η βάση δεδομένων δεν ανοίγει. Αν ορίσετε `NRZPAPER_DB_KEY`, φυλάξτε αυτή την τιμή με ασφάλεια.

### Και οι δύο αρχιτεκτονικές

```bash
docker buildx build --platform linux/amd64,linux/arm64 -t <registry>/nrzpaper:1.0.0 --push .
```

Η Rust μεταγλωττίζεται διασταυρωτά, όχι εξομοιώνεται, οπότε η κατασκευή arm64 είναι γρήγορη σε μηχάνημα x86 και αντίστροφα.

## Πρώτα βήματα

1. **Μεταφορτώστε** μερικά PDF ή φωτογραφίες: το κείμενο αναγνωρίζεται στο παρασκήνιο.
2. **Ετικέτες**: δημιουργήστε ετικέτες όπως *Τιμολόγιο* ή *Ασφάλιση* και προσθέστε περιγραφή. Η AI τη χρησιμοποιεί για να αναθέσει ετικέτες.
3. **AI** (*Ρυθμίσεις → AI και μοντέλα*): επιλέξτε πάροχο, επικολλήστε κλειδί (ή χρησιμοποιήστε τα ενσωματωμένα δωρεάν μοντέλα OpenRouter ή ένα τοπικό Ollama), μετά *Φόρτωση μοντέλων* και *Δοκιμή σύνδεσης*.
4. **Σαρωτής** (*Προφίλ → Σαρωτής*): *Αναζήτηση δικτύου* βρίσκει σαρωτές eSCL/AirScan· μετά *Σάρωση* στην πλαϊνή γραμμή.
5. **Email** (*Προφίλ → Εισερχόμενα email*): προσθέστε ένα ή περισσότερα γραμματοκιβώτια IMAP· PDF και εικόνες από συνημμένα καταλήγουν στα εισερχόμενα.
6. **Εισερχόμενα φακέλων** (*Ρυθμίσεις → Εισερχόμενα φακέλων*, διαχειριστές): παρακολουθήστε έναν φάκελο διακομιστή όπως `/scans` (προσαρτήστε τον πρώτα στο container).
7. **Εκτυπωτές** (*Ρυθμίσεις → Εκτυπωτές*, διαχειριστές): *Αναζήτηση δικτύου* ή εισαγάγετε `ipp://printer/ipp/print`· μετά κάθε μέλος μπορεί να εκτυπώσει από τη σελίδα εγγράφου.

## Διαμόρφωση

Κάθε ρύθμιση μπορεί να δοθεί ως μεταβλητή περιβάλλοντος με πρόθεμα `NRZPAPER_` (ή στο `Rocket.toml`).

| Μεταβλητή | Προεπιλογή | Σημασία |
|---|---|---|
| `NRZPAPER_DATA_DIR` | `/data` (εικόνα) | Βάση δεδομένων, αρχείο κλειδιού και φάκελοι εισαγωγής |
| `NRZPAPER_DB_KEY` | – | Κλειδί βάσης δεδομένων· αν δεν οριστεί, δημιουργείται `nrzpaper.key` στον φάκελο δεδομένων |
| `NRZPAPER_PORT` / `NRZPAPER_ADDRESS` | `8320` / `0.0.0.0` | Πού ακούει ο διακομιστής |
| `NRZPAPER_COOKIE_SECURE` | `false` | Ορίστε `true` πίσω από HTTPS |
| `NRZPAPER_REGISTRATION_OPEN` | `false` | Να επιτρέπεται η αυτο-εγγραφή (εναλλάσσεται και στο UI) |
| `NRZPAPER_OCR_LANGUAGES` | `deu+eng` | Γλώσσες Tesseract (η εικόνα περιέχει `deu` και `eng`) |
| `NRZPAPER_OCR_WORKERS` | `2` | Παράλληλες εργασίες OCR |
| `NRZPAPER_TRASH_RETENTION_DAYS` | `30` | Ημέρες στον κάδο πριν από την οριστική διαγραφή |
| `NRZPAPER_INBOX_KEEP_DAYS` | `7` | Ημέρες διατήρησης μη εγκεκριμένων στοιχείων εισερχομένων |
| `NRZPAPER_SCANNER_URL` | – | Προκαθορισμένος σαρωτής για νέους χρήστες, π.χ. `http://192.168.1.50` |
| `NRZPAPER_SCAN_SUBNETS` | `192.168.0.0/24,192.168.1.0/24` | Δίκτυα αναζήτησης σαρωτών και εκτυπωτών |
| `NRZPAPER_AI_PROVIDER` | `openrouter` | `openai`, `anthropic`, `gemini`, `mistral`, `groq`, `ollama`, `custom`, … |
| `NRZPAPER_AI_KEY` / `NRZPAPER_AI_URL` / `NRZPAPER_AI_MODEL` | – | Κλειδί, βασικό URL και μοντέλο για τον πάροχο |
| `NRZPAPER_OPENROUTER_KEY` | – | Κλειδί OpenRouter (εναλλακτικό των παραπάνω) |
| `NRZPAPER_LOG` | `info` | Φίλτρο καταγραφής, π.χ. `debug` |

Οι ρυθμίσεις από τη διεπαφή ιστού (πάροχος AI, κλειδιά, μοντέλα, εγγραφή) αποθηκεύονται στην κρυπτογραφημένη βάση δεδομένων και υπερισχύουν του περιβάλλοντος.

### Πάροχοι AI

| Πάροχος | Διεύθυνση | Σημειώσεις |
|---|---|---|
| OpenRouter | ενσωματωμένο | Δωρεάν μοντέλα όρασης· προαιρετικό δικό σας κλειδί |
| OpenAI, Anthropic, Google Gemini, Mistral, Groq, xAI, DeepSeek, Together, Fireworks, Perplexity, Cohere, NVIDIA, Hugging Face, Qwen | ενσωματωμένο | Κλειδί API από τον πάροχο |
| Azure OpenAI | `https://NAME.openai.azure.com/openai/v1` | Μοντέλο = όνομα ανάπτυξης |
| Ollama / LM Studio / vLLM / LocalAI | `http://host:11434/v1` κ.λπ. | Δεν χρειάζεται κλειδί |
| Οτιδήποτε συμβατό με OpenAI | το URL σας | Προαιρετικό κλειδί |

Μοντέλα που δεν διαβάζουν εικόνες: τσεκάρετε *«Το μοντέλο δεν διαβάζει εικόνες – αποστολή αναγνωρισμένου κειμένου»*. Οι πάροχοι που δεν δέχονται PDF λαμβάνουν αυτόματα τις πρώτες σελίδες ως εικόνες. Η γλώσσα των προτεινόμενων τίτλων και περιλήψεων ορίζεται στο *Γλώσσα προτάσεων*.

## REST API

Δημιουργήστε κλειδί API στο *Προφίλ → Κλειδιά API* (επιλέξτε δικαιώματα). Όλα τα endpoints βρίσκονται κάτω από το `/api/v1` και δέχονται `Authorization: Bearer np_…`.

```bash
TOKEN=np_your_key
BASE=http://localhost:8320/api/v1

# οργανισμοί
curl -H "Authorization: Bearer $TOKEN" $BASE/organizations

# μεταφόρτωση (πολλά αρχεία μαζί με πολλά -F)
curl -H "Authorization: Bearer $TOKEN" -F file=@invoice.pdf \
     $BASE/organizations/$ORG/documents

# αναζήτηση: ίδια σύνταξη όπως στο UI
curl -H "Authorization: Bearer $TOKEN" -G $BASE/organizations/$ORG/documents \
     --data-urlencode 'searchQuery=northwind tag:invoice created:2026'

# λήψη αρχείου
curl -H "Authorization: Bearer $TOKEN" -o out.pdf \
     "$BASE/organizations/$ORG/documents/$DOC/file?download=true"

# προσθήκη ετικέτας
curl -H "Authorization: Bearer $TOKEN" -H "Content-Type: application/json" \
     -d '{"tagId":"'$TAG'"}' $BASE/organizations/$ORG/documents/$DOC/tags

# δημόσιος σύνδεσμος κοινής χρήσης για 7 ημέρες
curl -H "Authorization: Bearer $TOKEN" -H "Content-Type: application/json" \
     -d '{"expiresInDays":7}' $BASE/organizations/$ORG/documents/$DOC/share-links

# έλεγχος υγείας (χωρίς κλειδί)
curl $BASE/health
```

Τα webhooks (*Ρυθμίσεις → Webhooks*) στέλνουν υπογεγραμμένο JSON (`webhook-id`, `webhook-timestamp`, `webhook-signature`) για `document:created`, `document:updated`, `document:deleted`, `document:restored`, `document:tag:added` και `document:tag:removed`.

Ο φάκελος εισαγωγής `data/ingestion/<organization-id>/` εισάγει αρχεία απευθείας ως έγγραφα. Οι παρακολουθούμενοι φάκελοι (*Εισερχόμενα φακέλων*) τα στέλνουν πρώτα στα εισερχόμενα για έλεγχο.

## Ανάπτυξη

```bash
# backend (Rust 1.85+, Tesseract και Poppler για OCR)
cargo run                         # http://localhost:8320
cargo test

# διεπαφή ιστού με hot reload (dx = dioxus-cli 0.7)
rustup target add wasm32-unknown-unknown
cargo install dioxus-cli --version 0.7.10 --locked
cd ui && dx serve                 # http://localhost:8080, κάνει proxy το /api στο :8320

# ενσωμάτωση της διεπαφής στο πρόγραμμα
scripts/ui.sh && cargo build --release
```

Οι μεταφράσεις βρίσκονται στο `locales/<language>.json` (γερμανικό κείμενο προέλευσης → μετάφραση). Το `scripts/i18n_extract.py` ανανεώνει το `locales/strings.json` και αναφέρει ελλείπουσες καταχωρίσεις ανά γλώσσα.

Λεπτομέρειες αρχιτεκτονικής: [ARCHITECTURE.md](../ARCHITECTURE.md).

## Άδεια

AGPL-3.0-or-later. Τα στοιχεία UI βασίζονται στο [rust-ui](https://github.com/rust-ui/ui) (MIT), η προβολή PDF στο [PDF.js](https://mozilla.github.io/pdf.js/) (Apache-2.0), η γραμματοσειρά είναι η [Inter](https://rsms.me/inter/) (OFL).
