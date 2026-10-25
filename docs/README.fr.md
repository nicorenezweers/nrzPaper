<div align="center">

<img src="../static/icon.png" width="96" alt="nrzPaper">

# nrzPaper

**Gestion documentaire auto-hébergée en Rust : numériser, reconnaître, étiqueter et retrouver vos papiers, chiffrés dans un seul fichier.**

[English](../README.md) · [Deutsch](README.de.md) · [Italiano](README.it.md) · [Español](README.es.md) · [Português](README.pt.md) · [Nederlands](README.nl.md)

![Documents avec chronologie](screenshots/documents.png)

</div>

nrzPaper est un programme unique, sans base de données externe. Tout est stocké dans un fichier SQLite chiffré avec SQLCipher, y compris les fichiers eux-mêmes. Il reconnaît le texte avec Tesseract, numérise directement depuis les scanners réseau, récupère les pièces jointes des e-mails et laisse une IA de votre choix proposer titres, étiquettes et métadonnées. **Dieter**, l’assistant intégré, répond aux questions sur vos documents et cite ses sources. L’interface est une application Rust/WebAssembly (Dioxus) au thème sombre et fonctionne aussi sur mobile.

## Fonctionnalités

| | |
|---|---|
| **Capture** | Glisser-déposer, envoi, **scanners réseau** (eSCL/AirScan : Canon, HP, Epson, Brother …), **boîtes e-mail** (IMAP, plusieurs comptes), **dossiers surveillés** (p. ex. le dossier de numérisation du NAS), API REST |
| **Boîte de réception** | Numérisations, e-mails, imports de dossiers et envois attendent la validation : l’IA propose nom, étiquettes et champs, vous validez un élément ou **tout d’un coup**. Filtre par source. |
| **Reconnaissance** | Tesseract et Poppler en arrière-plan, aussi pour les PDF numérisés et les photos ; recherche plein texte avec préfixes, phrases et filtres (`tag:`, `-tag:`, `type:`, `created:>2026-06-01`, …) |
| **Organisation** | Étiquettes colorées, **dossiers construits à partir d’étiquettes**, **règles d’étiquetage**, **propriétés** personnalisées (texte, nombre, date, oui/non, choix, choix multiple, utilisateur, lien vers un document) |
| **Chronologie** | Chronologie verticale à côté des documents : aujourd’hui en haut, chaque jour avec du courrier en dessous, et les périodes vides entre les deux. Filtre par étiquette, type et texte. |
| **Rappels** | « Me le rappeler dans 10 jours » sur chaque document, avec une note ; les rappels échus apparaissent dans la barre latérale et en haut de la chronologie. |
| **Impression** | Sur toute **imprimante réseau IPP** (AirPrint, IPP Everywhere, Mopria, CUPS) trouvée par le serveur. Les PDF partent directement, ou en images JPEG pour les imprimantes qui n’acceptent que ce format. |
| **IA, tout fournisseur** | OpenRouter (modèles gratuits intégrés), OpenAI, Anthropic, Google Gemini, Mistral, Groq, xAI, DeepSeek, Together, Fireworks, Perplexity, Cohere, NVIDIA, Hugging Face, Qwen, Azure OpenAI, **Ollama, LM Studio, vLLM/LocalAI** ou toute URL compatible OpenAI. Les modèles texte seul reçoivent le texte reconnu. |
| **Dieter** | Assistant de discussion en bas à droite : *« Quel est mon numéro d’assurance ? »*, *« Toutes les fiches de paie de Fabrikam en 2026 »*. Il cherche, lit les passages pertinents et répond avec des liens vers les documents. |
| **Équipes** | Plusieurs organisations, **plusieurs comptes par organisation** (invitations, rôles propriétaire/admin/membre, transfert de propriété), clés API avec droits, webhooks (signatures Standard Webhooks), liens de partage publics |
| **Langues** | English, Deutsch, Français, Italiano, Español, Português, Nederlands (d’autres via `locales/<code>.json`) |
| **Sûr** | Un fichier chiffré, mots de passe Argon2, CSP sans scripts inline, protection CSRF, fichiers uniquement via l’API |

## Captures d’écran

| | |
|---|---|
| ![Dieter](screenshots/dieter.png) | ![Boîte de réception](screenshots/inbox.png) |
| **Dieter** répond avec ses sources | **Boîte de réception** : vérifier, valider, filtrer par source |
| ![Document](screenshots/document.png) | ![Fournisseurs d’IA](screenshots/settings-ai.png) |
| **Document** : visionneuse PDF, étiquettes, rappels, impression | **IA et modèles** : tout fournisseur, local ou cloud |
| ![Rappels](screenshots/reminders.png) | ![Membres](screenshots/members.png) |
| **Rappels** | **Membres** et invitations |

## Démarrage rapide

Il vous faut Docker. L’image contient le programme, l’interface, Tesseract (allemand, anglais) et Poppler.

```bash
git clone https://github.com/nicorenezweers/nrzPaper.git
cd nrzPaper
docker build -t nrzpaper .
docker run -d --name nrzpaper -p 8320:8320 -v nrzpaper-data:/data nrzpaper
```

Ouvrez <http://localhost:8320>. Le premier compte créé devient administrateur. Ensuite, l’inscription est fermée jusqu’à ce que vous l’ouvriez dans *Paramètres → Inscription et utilisateurs*, où vous pouvez aussi créer des comptes.

### Docker Compose

```yaml
services:
  nrzpaper:
    build: .                      # ou : image: ghcr.io/nicorenezweers/nrzpaper:1.0.0
    container_name: nrzpaper
    restart: unless-stopped
    ports:
      - "8320:8320"
    volumes:
      - nrzpaper-data:/data       # base chiffrée + fichier de clé
      - /srv/scans:/scans         # facultatif : dossier surveillé
    environment:
      TZ: Europe/Paris
      # NRZPAPER_DB_KEY: "un-long-secret"     # sinon /data/nrzpaper.key est créé
      # NRZPAPER_COOKIE_SECURE: "true"        # derrière HTTPS
      # NRZPAPER_AI_PROVIDER: "openai"        # ou dans l’interface
      # NRZPAPER_AI_KEY: "sk-…"

volumes:
  nrzpaper-data:
```

```bash
docker compose up -d
```

> **Sauvegardez `/data`.** Il contient `nrzpaper.db` (tout, chiffré) et `nrzpaper.key`. Sans la clé, la base ne peut pas être ouverte. Si vous définissez `NRZPAPER_DB_KEY`, conservez cette valeur en lieu sûr.

### Les deux architectures

```bash
docker buildx build --platform linux/amd64,linux/arm64 -t <registry>/nrzpaper:1.0.0 --push .
```

## Premiers pas

1. **Envoyer** quelques PDF ou photos : le texte est reconnu en arrière-plan.
2. **Étiquettes** : créez-en (p. ex. *Facture*, *Assurance*) avec une description ; l’IA s’en sert pour les attribuer.
3. **IA** (*Paramètres → IA et modèles*) : choisissez un fournisseur, collez une clé (ou utilisez les modèles OpenRouter gratuits intégrés, ou un Ollama local), puis *Charger les modèles* et *Tester la connexion*.
4. **Scanner** (*Profil → Scanner*) : *Rechercher sur le réseau* trouve les scanners eSCL/AirScan, puis *Numériser* dans la barre latérale.
5. **E-mail** (*Profil → Réception e-mail*) : une ou plusieurs boîtes IMAP ; les PDF et images joints arrivent dans la boîte de réception.
6. **Dossier surveillé** (*Paramètres → Dossiers*, admins) : surveillez un dossier du serveur comme `/scans` (montez-le d’abord dans le conteneur).
7. **Imprimantes** (*Paramètres → Imprimantes*, admins) : *Rechercher sur le réseau* ou saisissez `ipp://imprimante/ipp/print` ; chaque membre peut ensuite imprimer depuis la page d’un document.

## Configuration

Chaque paramètre peut être donné comme variable d’environnement préfixée par `NRZPAPER_` (ou dans `Rocket.toml`).

| Variable | Défaut | Signification |
|---|---|---|
| `NRZPAPER_DATA_DIR` | `/data` (image) | Base, fichier de clé, dossiers d’ingestion |
| `NRZPAPER_DB_KEY` | – | Clé de la base ; sinon `nrzpaper.key` est généré |
| `NRZPAPER_PORT` / `NRZPAPER_ADDRESS` | `8320` / `0.0.0.0` | Écoute du serveur |
| `NRZPAPER_COOKIE_SECURE` | `false` | `true` derrière HTTPS |
| `NRZPAPER_REGISTRATION_OPEN` | `false` | Autoriser l’inscription libre |
| `NRZPAPER_OCR_LANGUAGES` | `deu+eng` | Langues Tesseract |
| `NRZPAPER_OCR_WORKERS` | `2` | Reconnaissances simultanées |
| `NRZPAPER_TRASH_RETENTION_DAYS` | `30` | Jours dans la corbeille |
| `NRZPAPER_INBOX_KEEP_DAYS` | `7` | Durée de conservation des éléments non validés |
| `NRZPAPER_SCANNER_URL` | – | Scanner prédéfini, p. ex. `http://192.168.1.50` |
| `NRZPAPER_SCAN_SUBNETS` | `192.168.0.0/24,192.168.1.0/24` | Réseaux parcourus pour scanners et imprimantes |
| `NRZPAPER_AI_PROVIDER` | `openrouter` | `openai`, `anthropic`, `gemini`, `mistral`, `groq`, `ollama`, `custom`, … |
| `NRZPAPER_AI_KEY` / `_URL` / `_MODEL` | – | Clé, adresse et modèle du fournisseur |
| `NRZPAPER_OPENROUTER_KEY` | – | Clé OpenRouter (alternative) |
| `NRZPAPER_LOG` | `info` | Journal, p. ex. `debug` |

Les réglages faits dans l’interface sont stockés chiffrés dans la base et priment sur l’environnement.

### Fournisseurs d’IA

| Fournisseur | Adresse | Remarque |
|---|---|---|
| OpenRouter | intégrée | Modèles vision gratuits ; clé personnelle facultative |
| OpenAI, Anthropic, Google Gemini, Mistral, Groq, xAI, DeepSeek, Together, Fireworks, Perplexity, Cohere, NVIDIA, Hugging Face, Qwen | intégrée | Clé API du fournisseur |
| Azure OpenAI | `https://NOM.openai.azure.com/openai/v1` | Modèle = nom du déploiement |
| Ollama / LM Studio / vLLM / LocalAI | `http://hote:11434/v1` etc. | Pas de clé |
| Tout service compatible OpenAI | votre URL | Clé facultative |

## API REST

Créez une clé API dans *Profil → Clés API*. Tout se trouve sous `/api/v1` avec `Authorization: Bearer np_…`.

```bash
TOKEN=np_votre_cle
BASE=http://localhost:8320/api/v1

curl -H "Authorization: Bearer $TOKEN" $BASE/organizations
curl -H "Authorization: Bearer $TOKEN" -F file=@facture.pdf $BASE/organizations/$ORG/documents
curl -H "Authorization: Bearer $TOKEN" -G $BASE/organizations/$ORG/documents \
     --data-urlencode 'searchQuery=northwind tag:facture created:2026'
curl -H "Authorization: Bearer $TOKEN" -o out.pdf \
     "$BASE/organizations/$ORG/documents/$DOC/file?download=true"
curl $BASE/health
```

Les webhooks (*Paramètres → Webhooks*) envoient du JSON signé pour `document:created`, `document:updated`, `document:deleted`, `document:restored`, `document:tag:added` et `document:tag:removed`.

## Développement

```bash
cargo run && cargo test
rustup target add wasm32-unknown-unknown
cargo install dioxus-cli --version 0.7.10 --locked
cd ui && dx serve                       # http://localhost:8080
scripts/ui.sh && cargo build --release  # intégrer l’interface au programme
```

Les traductions se trouvent dans `locales/<langue>.json`. Architecture : [ARCHITECTURE.md](../ARCHITECTURE.md).

## Licence

AGPL-3.0-or-later. Composants d’interface d’après [rust-ui](https://github.com/rust-ui/ui) (MIT), visionneuse [PDF.js](https://mozilla.github.io/pdf.js/) (Apache-2.0), police [Inter](https://rsms.me/inter/) (OFL).
