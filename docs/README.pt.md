<div align="center">

<img src="../static/icon.png" width="96" alt="nrzPaper">

# nrzPaper

**Gestão documental auto-hospedada em Rust: digitalize, reconheça, etiquete e encontre a sua papelada, cifrada num único ficheiro.**

[English](../README.md) · [Deutsch](README.de.md) · [Français](README.fr.md) · [Italiano](README.it.md) · [Español](README.es.md) · [Nederlands](README.nl.md)

![Documentos com linha do tempo](screenshots/documents.png)

</div>

O nrzPaper é um único programa sem base de dados externa. Tudo fica num ficheiro SQLite cifrado com SQLCipher, incluindo os próprios ficheiros. Reconhece texto com o Tesseract, digitaliza diretamente de scanners de rede, recolhe anexos de e-mail e deixa uma IA à sua escolha sugerir títulos, etiquetas e metadados. O **Dieter**, o assistente integrado, responde a perguntas sobre os seus documentos e indica as fontes. A interface é uma app Rust/WebAssembly (Dioxus) com tema escuro e também funciona no telemóvel.

## Funcionalidades

| | |
|---|---|
| **Captura** | Arrastar e largar, carregamento, **scanners de rede** (eSCL/AirScan: Canon, HP, Epson, Brother …), **caixas de e-mail** (IMAP, várias contas), **pastas vigiadas** (p. ex. a pasta de digitalização do NAS), API REST |
| **Caixa de entrada** | Digitalizações, e-mails, importações de pastas e carregamentos aguardam revisão: a IA sugere nome, etiquetas e campos, e aprova um a um ou **todos de uma vez**. Filtro por origem. |
| **Reconhecimento** | Tesseract e Poppler em segundo plano, também para PDFs digitalizados e fotos; pesquisa de texto integral com prefixos, frases e filtros (`tag:`, `-tag:`, `type:`, `created:>2026-06-01`, …) |
| **Organização** | Etiquetas coloridas, **pastas a partir de etiquetas**, **regras de etiquetagem**, **propriedades** próprias (texto, número, data, sim/não, seleção, seleção múltipla, utilizador, ligação a documento) |
| **Linha do tempo** | Linha do tempo vertical ao lado dos documentos: hoje no topo, cada dia com correio abaixo e os intervalos vazios pelo meio. Filtro por etiqueta, tipo e texto. |
| **Lembretes** | «Lembrar-me daqui a 10 dias» em qualquer documento, com nota; os lembretes vencidos aparecem na barra lateral e no topo da linha do tempo. |
| **Impressão** | Em qualquer **impressora de rede IPP** (AirPrint, IPP Everywhere, Mopria, CUPS) que o servidor encontre. Os PDFs seguem diretamente, ou como imagens JPEG para impressoras que só aceitam esse formato. |
| **IA, qualquer fornecedor** | OpenRouter (modelos gratuitos integrados), OpenAI, Anthropic, Google Gemini, Mistral, Groq, xAI, DeepSeek, Together, Fireworks, Perplexity, Cohere, NVIDIA, Hugging Face, Qwen, Azure OpenAI, **Ollama, LM Studio, vLLM/LocalAI** ou qualquer URL compatível com OpenAI. Os modelos só de texto recebem o texto reconhecido. |
| **Dieter** | Assistente de chat no canto inferior direito: *«Qual é o meu número de seguro?»*, *«Todos os recibos de vencimento da Fabrikam de 2026»*. Pesquisa, lê as passagens relevantes e responde com ligações para os documentos. |
| **Equipas** | Várias organizações, **várias contas por organização** (convites, funções proprietário/admin/membro, transferir a propriedade), chaves API com permissões, webhooks (assinaturas Standard Webhooks), ligações públicas de partilha |
| **Idiomas** | English, Deutsch, Français, Italiano, Español, Português, Nederlands (mais através de `locales/<código>.json`) |
| **Seguro** | Um ficheiro cifrado, palavras-passe Argon2, CSP sem scripts inline, proteção CSRF, ficheiros só através da API |

## Capturas de ecrã

| | |
|---|---|
| ![Dieter](screenshots/dieter.png) | ![Caixa de entrada](screenshots/inbox.png) |
| **Dieter** responde com fontes | **Caixa de entrada**: rever, aprovar, filtrar por origem |
| ![Documento](screenshots/document.png) | ![Fornecedores de IA](screenshots/settings-ai.png) |
| **Documento**: visualizador PDF, etiquetas, lembretes, imprimir | **IA e modelos**: qualquer fornecedor, local ou na nuvem |
| ![Lembretes](screenshots/reminders.png) | ![Membros](screenshots/members.png) |
| **Lembretes** | **Membros** e convites |

## Início rápido

Precisa do Docker. A imagem contém o programa, a interface, o Tesseract (alemão, inglês) e o Poppler.

```bash
git clone https://github.com/nicorenezweers/nrzPaper.git
cd nrzPaper
docker build -t nrzpaper .
docker run -d --name nrzpaper -p 8320:8320 -v nrzpaper-data:/data nrzpaper
```

Abra <http://localhost:8320>. A primeira conta criada torna-se administradora. Depois disso, o registo fica fechado até o abrir em *Definições → Registo e utilizadores*, onde também pode criar contas para outras pessoas.

### Docker Compose

```yaml
services:
  nrzpaper:
    build: .                      # ou: image: ghcr.io/nicorenezweers/nrzpaper:1.0.0
    container_name: nrzpaper
    restart: unless-stopped
    ports:
      - "8320:8320"
    volumes:
      - nrzpaper-data:/data       # base de dados cifrada + ficheiro da chave
      - /srv/scans:/scans         # opcional: pasta vigiada
    environment:
      TZ: Europe/Lisbon
      # NRZPAPER_DB_KEY: "um-segredo-longo"   # caso contrário é criado /data/nrzpaper.key
      # NRZPAPER_COOKIE_SECURE: "true"        # atrás de HTTPS
      # NRZPAPER_AI_PROVIDER: "openai"        # ou na interface
      # NRZPAPER_AI_KEY: "sk-…"

volumes:
  nrzpaper-data:
```

```bash
docker compose up -d
```

> **Faça cópia de segurança de `/data`.** Contém `nrzpaper.db` (tudo, cifrado) e `nrzpaper.key`. Sem a chave a base de dados não abre. Se definir `NRZPAPER_DB_KEY`, guarde esse valor em segurança.

### Ambas as arquiteturas

```bash
docker buildx build --platform linux/amd64,linux/arm64 -t <registry>/nrzpaper:1.0.0 --push .
```

## Primeiros passos

1. **Carregue** alguns PDFs ou fotos: o texto é reconhecido em segundo plano.
2. **Etiquetas**: crie-as (p. ex. *Fatura*, *Seguro*) com uma descrição; a IA usa-a para as atribuir.
3. **IA** (*Definições → IA e modelos*): escolha um fornecedor, cole uma chave (ou use os modelos gratuitos do OpenRouter integrados ou um Ollama local), depois *Carregar modelos* e *Testar ligação*.
4. **Scanner** (*Perfil → Scanner*): *Procurar na rede* encontra scanners eSCL/AirScan; depois *Digitalizar* na barra lateral.
5. **E-mail** (*Perfil → E-mail de entrada*): uma ou várias caixas IMAP; PDFs e imagens anexados chegam à caixa de entrada.
6. **Pasta vigiada** (*Definições → Pastas*, admins): vigie uma pasta do servidor como `/scans` (monte-a primeiro no contentor).
7. **Impressoras** (*Definições → Impressoras*, admins): *Procurar na rede* ou introduza `ipp://impressora/ipp/print`; qualquer membro pode depois imprimir a partir da página do documento.

## Configuração

Cada definição pode ser dada como variável de ambiente com o prefixo `NRZPAPER_` (ou em `Rocket.toml`).

| Variável | Predefinição | Significado |
|---|---|---|
| `NRZPAPER_DATA_DIR` | `/data` (imagem) | Base de dados, ficheiro da chave, pastas de importação |
| `NRZPAPER_DB_KEY` | – | Chave da base de dados; se faltar, é gerado `nrzpaper.key` |
| `NRZPAPER_PORT` / `NRZPAPER_ADDRESS` | `8320` / `0.0.0.0` | Onde o servidor escuta |
| `NRZPAPER_COOKIE_SECURE` | `false` | `true` atrás de HTTPS |
| `NRZPAPER_REGISTRATION_OPEN` | `false` | Permitir registo livre |
| `NRZPAPER_OCR_LANGUAGES` | `deu+eng` | Idiomas do Tesseract |
| `NRZPAPER_OCR_WORKERS` | `2` | Reconhecimentos em paralelo |
| `NRZPAPER_TRASH_RETENTION_DAYS` | `30` | Dias no lixo |
| `NRZPAPER_INBOX_KEEP_DAYS` | `7` | Dias de conservação dos itens não aprovados |
| `NRZPAPER_SCANNER_URL` | – | Scanner predefinido, p. ex. `http://192.168.1.50` |
| `NRZPAPER_SCAN_SUBNETS` | `192.168.0.0/24,192.168.1.0/24` | Redes onde procurar scanners e impressoras |
| `NRZPAPER_AI_PROVIDER` | `openrouter` | `openai`, `anthropic`, `gemini`, `mistral`, `groq`, `ollama`, `custom`, … |
| `NRZPAPER_AI_KEY` / `_URL` / `_MODEL` | – | Chave, endereço e modelo do fornecedor |
| `NRZPAPER_OPENROUTER_KEY` | – | Chave do OpenRouter (alternativa) |
| `NRZPAPER_LOG` | `info` | Nível de registo, p. ex. `debug` |

As definições feitas na interface são guardadas cifradas na base de dados e têm prioridade sobre o ambiente.

### Fornecedores de IA

| Fornecedor | Endereço | Nota |
|---|---|---|
| OpenRouter | integrado | Modelos de visão gratuitos; chave própria opcional |
| OpenAI, Anthropic, Google Gemini, Mistral, Groq, xAI, DeepSeek, Together, Fireworks, Perplexity, Cohere, NVIDIA, Hugging Face, Qwen | integrado | Chave API do fornecedor |
| Azure OpenAI | `https://NOME.openai.azure.com/openai/v1` | Modelo = nome da implementação |
| Ollama / LM Studio / vLLM / LocalAI | `http://host:11434/v1` etc. | Sem chave |
| Qualquer serviço compatível com OpenAI | o seu URL | Chave opcional |

## API REST

Crie uma chave API em *Perfil → Chaves API*. Tudo está em `/api/v1` com `Authorization: Bearer np_…`.

```bash
TOKEN=np_sua_chave
BASE=http://localhost:8320/api/v1

curl -H "Authorization: Bearer $TOKEN" $BASE/organizations
curl -H "Authorization: Bearer $TOKEN" -F file=@fatura.pdf $BASE/organizations/$ORG/documents
curl -H "Authorization: Bearer $TOKEN" -G $BASE/organizations/$ORG/documents \
     --data-urlencode 'searchQuery=northwind tag:fatura created:2026'
curl -H "Authorization: Bearer $TOKEN" -o out.pdf \
     "$BASE/organizations/$ORG/documents/$DOC/file?download=true"
curl $BASE/health
```

Os webhooks (*Definições → Webhooks*) enviam JSON assinado para `document:created`, `document:updated`, `document:deleted`, `document:restored`, `document:tag:added` e `document:tag:removed`.

## Desenvolvimento

```bash
cargo run && cargo test
rustup target add wasm32-unknown-unknown
cargo install dioxus-cli --version 0.7.10 --locked
cd ui && dx serve                       # http://localhost:8080
scripts/ui.sh && cargo build --release  # incorporar a interface no programa
```

As traduções estão em `locales/<idioma>.json`. Arquitetura: [ARCHITECTURE.md](../ARCHITECTURE.md).

## Licença

AGPL-3.0-or-later. Componentes de interface baseados em [rust-ui](https://github.com/rust-ui/ui) (MIT), visualizador [PDF.js](https://mozilla.github.io/pdf.js/) (Apache-2.0), tipo de letra [Inter](https://rsms.me/inter/) (OFL).
