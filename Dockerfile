# syntax=docker/dockerfile:1.7
# nrzPaper – Multi-Stage-Build.
#   docker build -t nrzpaper .
#   docker run -p 8320:8320 -v nrzpaper-data:/data nrzpaper
#
# 1) builder: Tailwind-CSS frisch erzeugen, dann das Programm (SQLCipher und OpenSSL werden
#    mitgebaut, deshalb keine Laufzeit-Bibliotheken dafür nötig).
# 2) runtime: debian:bookworm-slim mit Tesseract (Deutsch, Englisch) und Poppler.

FROM rust:1-bookworm AS builder
WORKDIR /src

# Abhängigkeiten zuerst – ändern sich selten und bleiben im Cache
COPY Cargo.toml Cargo.lock build.rs ./
COPY migrations migrations
COPY src src
COPY templates templates
COPY static static
COPY assets assets
COPY scripts scripts
COPY tailwind.config.js ./

# CSS aus den Templates erzeugen (eigenständige Tailwind-CLI, Prüfsumme in scripts/tailwind.sh)
RUN scripts/tailwind.sh

RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/src/target \
    cargo build --release --locked \
 && cp target/release/nrzpaper /usr/local/bin/nrzpaper

FROM debian:bookworm-slim AS runtime
RUN apt-get update \
 && apt-get install -y --no-install-recommends \
      tesseract-ocr tesseract-ocr-deu tesseract-ocr-eng poppler-utils ca-certificates \
 && rm -rf /var/lib/apt/lists/* \
 && useradd --system --uid 1000 --home-dir /data --shell /usr/sbin/nologin nrzpaper \
 && mkdir -p /data && chown nrzpaper:nrzpaper /data

COPY --from=builder /usr/local/bin/nrzpaper /usr/local/bin/nrzpaper
COPY Rocket.toml /app/Rocket.toml

WORKDIR /app
ENV NRZPAPER_CONFIG=/app/Rocket.toml \
    NRZPAPER_DATA_DIR=/data \
    NRZPAPER_ADDRESS=0.0.0.0 \
    NRZPAPER_PORT=8320 \
    NRZPAPER_LOG=info,rocket::server=warn,rocket::launch::_=warn,rocket::shield=warn
USER nrzpaper
VOLUME /data
EXPOSE 8320
HEALTHCHECK --interval=30s --timeout=5s --start-period=20s --retries=3 \
  CMD ["nrzpaper", "healthcheck"]
STOPSIGNAL SIGTERM
CMD ["nrzpaper"]
