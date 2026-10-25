<div align="center">

<img src="../static/icon.png" width="96" alt="nrzPaper">

# nrzPaper

**Gestión documental autoalojada en Rust: escanea, reconoce, etiqueta y encuentra tus papeles, cifrados en un solo archivo.**

[English](../README.md) · [Deutsch](README.de.md) · [Français](README.fr.md) · [Italiano](README.it.md) · [Português](README.pt.md) · [Nederlands](README.nl.md)

![Documentos con línea de tiempo](screenshots/documents.png)

</div>

nrzPaper es un único programa sin base de datos externa. Todo se guarda en un archivo SQLite cifrado con SQLCipher, incluidos los propios archivos. Reconoce texto con Tesseract, escanea directamente desde escáneres de red, recoge adjuntos de correo y deja que una IA de tu elección sugiera títulos, etiquetas y metadatos. **Dieter**, el asistente integrado, responde preguntas sobre tus documentos y enlaza las fuentes. La interfaz es una app Rust/WebAssembly (Dioxus) con tema oscuro y también funciona en el móvil.

## Funciones

| | |
|---|---|
| **Captura** | Arrastrar y soltar, subida, **escáneres de red** (eSCL/AirScan: Canon, HP, Epson, Brother …), **buzones de correo** (IMAP, varias cuentas), **carpetas vigiladas** (p. ej. la carpeta de escaneo del NAS), API REST |
| **Bandeja de entrada** | Escaneos, correos, importaciones de carpetas y subidas esperan revisión: la IA sugiere nombre, etiquetas y campos, y apruebas uno a uno o **todos a la vez**. Filtro por origen. |
| **Reconocimiento** | Tesseract y Poppler en segundo plano, también para PDF escaneados y fotos; búsqueda de texto completo con prefijos, frases y filtros (`tag:`, `-tag:`, `type:`, `created:>2026-06-01`, …) |
| **Organización** | Etiquetas de colores, **carpetas a partir de etiquetas**, **reglas de etiquetado**, **propiedades** propias (texto, número, fecha, sí/no, selección, selección múltiple, usuario, enlace a documento) |
| **Línea de tiempo** | Línea de tiempo vertical junto a los documentos: hoy arriba, cada día con correo debajo y los huecos vacíos entre medias. Filtro por etiqueta, tipo y texto. |
| **Recordatorios** | «Recuérdamelo en 10 días» en cualquier documento, con nota; los recordatorios vencidos aparecen en la barra lateral y arriba en la línea de tiempo. |
| **Impresión** | En cualquier **impresora de red IPP** (AirPrint, IPP Everywhere, Mopria, CUPS) que encuentre el servidor. Los PDF se envían directamente, o como imágenes JPEG a impresoras que solo aceptan ese formato. |
| **IA, cualquier proveedor** | OpenRouter (modelos gratuitos integrados), OpenAI, Anthropic, Google Gemini, Mistral, Groq, xAI, DeepSeek, Together, Fireworks, Perplexity, Cohere, NVIDIA, Hugging Face, Qwen, Azure OpenAI, **Ollama, LM Studio, vLLM/LocalAI** o cualquier URL compatible con OpenAI. Los modelos solo de texto reciben el texto reconocido. |
| **Dieter** | Asistente de chat abajo a la derecha: *«¿Cuál es mi número de seguro?»*, *«Todas las nóminas de Fabrikam de 2026»*. Busca, lee los pasajes relevantes y responde con enlaces a los documentos. |
| **Equipos** | Varias organizaciones, **varias cuentas por organización** (invitaciones, roles propietario/admin/miembro, transferir la propiedad), claves API con permisos, webhooks (firmas Standard Webhooks), enlaces públicos para compartir |
| **Idiomas** | English, Deutsch, Français, Italiano, Español, Português, Nederlands (más mediante `locales/<código>.json`) |
| **Seguro** | Un archivo cifrado, contraseñas Argon2, CSP sin scripts en línea, protección CSRF, archivos solo a través de la API |

## Capturas

| | |
|---|---|
| ![Dieter](screenshots/dieter.png) | ![Bandeja de entrada](screenshots/inbox.png) |
| **Dieter** responde con fuentes | **Bandeja de entrada**: revisar, aprobar, filtrar por origen |
| ![Documento](screenshots/document.png) | ![Proveedores de IA](screenshots/settings-ai.png) |
| **Documento**: visor PDF, etiquetas, recordatorios, imprimir | **IA y modelos**: cualquier proveedor, local o en la nube |
| ![Recordatorios](screenshots/reminders.png) | ![Miembros](screenshots/members.png) |
| **Recordatorios** | **Miembros** e invitaciones |

## Inicio rápido

Necesitas Docker. La imagen incluye el programa, la interfaz, Tesseract (alemán, inglés) y Poppler.

```bash
git clone https://github.com/nicorenezweers/nrzPaper.git
cd nrzPaper
docker build -t nrzpaper .
docker run -d --name nrzpaper -p 8320:8320 -v nrzpaper-data:/data nrzpaper
```

Abre <http://localhost:8320>. La primera cuenta creada será la de administrador. Después, el registro queda cerrado hasta que lo abras en *Ajustes → Registro y usuarios*, donde también puedes crear cuentas para otras personas.

### Docker Compose

```yaml
services:
  nrzpaper:
    build: .                      # o: image: ghcr.io/nicorenezweers/nrzpaper:1.0.0
    container_name: nrzpaper
    restart: unless-stopped
    ports:
      - "8320:8320"
    volumes:
      - nrzpaper-data:/data       # base de datos cifrada + archivo de clave
      - /srv/scans:/scans         # opcional: carpeta vigilada
    environment:
      TZ: Europe/Madrid
      # NRZPAPER_DB_KEY: "un-secreto-largo"   # si no, se crea /data/nrzpaper.key
      # NRZPAPER_COOKIE_SECURE: "true"        # detrás de HTTPS
      # NRZPAPER_AI_PROVIDER: "openai"        # o en la interfaz
      # NRZPAPER_AI_KEY: "sk-…"

volumes:
  nrzpaper-data:
```

```bash
docker compose up -d
```

> **Haz copia de seguridad de `/data`.** Contiene `nrzpaper.db` (todo, cifrado) y `nrzpaper.key`. Sin la clave no se puede abrir la base de datos. Si defines `NRZPAPER_DB_KEY`, guarda ese valor en un lugar seguro.

### Ambas arquitecturas

```bash
docker buildx build --platform linux/amd64,linux/arm64 -t <registry>/nrzpaper:1.0.0 --push .
```

## Primeros pasos

1. **Sube** algunos PDF o fotos: el texto se reconoce en segundo plano.
2. **Etiquetas**: créalas (p. ej. *Factura*, *Seguro*) con una descripción; la IA la usa para asignarlas.
3. **IA** (*Ajustes → IA y modelos*): elige un proveedor, pega una clave (o usa los modelos gratuitos de OpenRouter integrados o un Ollama local), luego *Cargar modelos* y *Probar conexión*.
4. **Escáner** (*Perfil → Escáner*): *Buscar en la red* encuentra escáneres eSCL/AirScan; luego *Escanear* en la barra lateral.
5. **Correo** (*Perfil → Correo entrante*): uno o varios buzones IMAP; los PDF e imágenes adjuntos llegan a la bandeja de entrada.
6. **Carpeta vigilada** (*Ajustes → Carpetas*, admins): vigila una carpeta del servidor como `/scans` (móntala antes en el contenedor).
7. **Impresoras** (*Ajustes → Impresoras*, admins): *Buscar en la red* o introduce `ipp://impresora/ipp/print`; cualquier miembro puede imprimir desde la página del documento.

## Configuración

Cada ajuste puede darse como variable de entorno con el prefijo `NRZPAPER_` (o en `Rocket.toml`).

| Variable | Predeterminado | Significado |
|---|---|---|
| `NRZPAPER_DATA_DIR` | `/data` (imagen) | Base de datos, archivo de clave, carpetas de ingesta |
| `NRZPAPER_DB_KEY` | – | Clave de la base de datos; si falta, se genera `nrzpaper.key` |
| `NRZPAPER_PORT` / `NRZPAPER_ADDRESS` | `8320` / `0.0.0.0` | Dónde escucha el servidor |
| `NRZPAPER_COOKIE_SECURE` | `false` | `true` detrás de HTTPS |
| `NRZPAPER_REGISTRATION_OPEN` | `false` | Permitir el registro libre |
| `NRZPAPER_OCR_LANGUAGES` | `deu+eng` | Idiomas de Tesseract |
| `NRZPAPER_OCR_WORKERS` | `2` | Reconocimientos en paralelo |
| `NRZPAPER_TRASH_RETENTION_DAYS` | `30` | Días en la papelera |
| `NRZPAPER_INBOX_KEEP_DAYS` | `7` | Días que se conservan los elementos no aprobados |
| `NRZPAPER_SCANNER_URL` | – | Escáner predefinido, p. ej. `http://192.168.1.50` |
| `NRZPAPER_SCAN_SUBNETS` | `192.168.0.0/24,192.168.1.0/24` | Redes donde buscar escáneres e impresoras |
| `NRZPAPER_AI_PROVIDER` | `openrouter` | `openai`, `anthropic`, `gemini`, `mistral`, `groq`, `ollama`, `custom`, … |
| `NRZPAPER_AI_KEY` / `_URL` / `_MODEL` | – | Clave, dirección y modelo del proveedor |
| `NRZPAPER_OPENROUTER_KEY` | – | Clave de OpenRouter (alternativa) |
| `NRZPAPER_LOG` | `info` | Nivel de registro, p. ej. `debug` |

Los ajustes hechos en la interfaz se guardan cifrados en la base de datos y tienen prioridad sobre el entorno.

### Proveedores de IA

| Proveedor | Dirección | Nota |
|---|---|---|
| OpenRouter | integrada | Modelos de visión gratuitos; clave propia opcional |
| OpenAI, Anthropic, Google Gemini, Mistral, Groq, xAI, DeepSeek, Together, Fireworks, Perplexity, Cohere, NVIDIA, Hugging Face, Qwen | integrada | Clave API del proveedor |
| Azure OpenAI | `https://NOMBRE.openai.azure.com/openai/v1` | Modelo = nombre de la implementación |
| Ollama / LM Studio / vLLM / LocalAI | `http://host:11434/v1` etc. | Sin clave |
| Cualquier servicio compatible con OpenAI | tu URL | Clave opcional |

## API REST

Crea una clave API en *Perfil → Claves API*. Todo está bajo `/api/v1` con `Authorization: Bearer np_…`.

```bash
TOKEN=np_tu_clave
BASE=http://localhost:8320/api/v1

curl -H "Authorization: Bearer $TOKEN" $BASE/organizations
curl -H "Authorization: Bearer $TOKEN" -F file=@factura.pdf $BASE/organizations/$ORG/documents
curl -H "Authorization: Bearer $TOKEN" -G $BASE/organizations/$ORG/documents \
     --data-urlencode 'searchQuery=northwind tag:factura created:2026'
curl -H "Authorization: Bearer $TOKEN" -o out.pdf \
     "$BASE/organizations/$ORG/documents/$DOC/file?download=true"
curl $BASE/health
```

Los webhooks (*Ajustes → Webhooks*) envían JSON firmado para `document:created`, `document:updated`, `document:deleted`, `document:restored`, `document:tag:added` y `document:tag:removed`.

## Desarrollo

```bash
cargo run && cargo test
rustup target add wasm32-unknown-unknown
cargo install dioxus-cli --version 0.7.10 --locked
cd ui && dx serve                       # http://localhost:8080
scripts/ui.sh && cargo build --release  # integrar la interfaz en el programa
```

Las traducciones están en `locales/<idioma>.json`. Arquitectura: [ARCHITECTURE.md](../ARCHITECTURE.md).

## Licencia

AGPL-3.0-or-later. Componentes de interfaz basados en [rust-ui](https://github.com/rust-ui/ui) (MIT), visor [PDF.js](https://mozilla.github.io/pdf.js/) (Apache-2.0), fuente [Inter](https://rsms.me/inter/) (OFL).
