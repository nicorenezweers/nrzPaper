# syntax=docker/dockerfile:1.7
# nrzPaper – Multi-Stage-Build für amd64 und arm64.
#   docker build -t nrzpaper .
#   docker run -p 8320:8320 -v nrzpaper-data:/data nrzpaper
# Beide Architekturen (Rust wird quer-kompiliert, nicht emuliert):
#   docker buildx build --platform linux/amd64,linux/arm64 -t <registry>/nrzpaper:<version> --push .
# Für den Umbrel-Store nur Programm und OCR-Paket (landen in dist/linux_<arch>/):
#   docker buildx build --platform linux/amd64,linux/arm64 --target umbrel --output type=local,dest=dist .

ARG DEBIAN=debian:bookworm-slim@sha256:7ff6ba31527b9e336162fa061746c366c4b2032f4d2cff2078bea43f78efe3f8

# ---------------------------------------------------------------- Programm
FROM --platform=$BUILDPLATFORM rust:1.98-bookworm@sha256:93ce27a88655056a51dbdd8f5f2d7ddc071c7b0070fb288a37b5a285fc83971e AS build
ARG TARGETARCH
ARG BUILDARCH
# Querübersetzer, wenn Ziel und Build-Rechner verschieden sind (SQLCipher und OpenSSL
# werden mit dem C-Compiler des Ziels gebaut).
RUN set -eu; \
    case "$TARGETARCH" in \
      amd64) t=x86_64-unknown-linux-gnu; p=x86_64-linux-gnu; pkg="gcc-x86-64-linux-gnu libc6-dev-amd64-cross" ;; \
      arm64) t=aarch64-unknown-linux-gnu; p=aarch64-linux-gnu; pkg="gcc-aarch64-linux-gnu libc6-dev-arm64-cross" ;; \
      *) echo "Architektur $TARGETARCH nicht unterstützt" >&2; exit 1 ;; \
    esac; \
    echo "$t" > /rust-target; \
    if [ "$TARGETARCH" != "$BUILDARCH" ]; then \
      apt-get update && apt-get install -y --no-install-recommends $pkg && rm -rf /var/lib/apt/lists/*; \
      rustup target add "$t"; \
      printf '[target.%s]\nlinker = "%s-gcc"\n' "$t" "$p" >> /usr/local/cargo/config.toml; \
      u=$(echo "$t" | tr - _); \
      printf 'CC_%s=%s-gcc\nAR_%s=%s-ar\n' "$u" "$p" "$u" "$p" > /cross-env; \
    else \
      touch /cross-env; \
    fi
WORKDIR /src
COPY Cargo.toml Cargo.lock build.rs tailwind.config.js ./
COPY migrations migrations
COPY src src
COPY templates templates
COPY static static
COPY assets assets
COPY scripts scripts
# CSS frisch aus den Templates (eigenständige Tailwind-CLI, Prüfsumme in scripts/tailwind.sh)
RUN scripts/tailwind.sh
# Der eingebaute OpenRouter-Schlüssel (KI beim Scannen) kommt als Build-Secret – nie in den
# Quelltext oder die Image-Schichten. Ohne Secret: KI nur mit NRZPAPER_OPENROUTER_KEY.
#   --secret id=openrouter,env=NRZ_OPENROUTER_KEY
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/src/target,id=nrzpaper-target-$TARGETARCH \
    --mount=type=secret,id=openrouter,env=NRZ_OPENROUTER_KEY \
    env $(cat /cross-env) cargo build --release --locked --target "$(cat /rust-target)" \
 && cp "target/$(cat /rust-target)/release/nrzpaper" /nrzpaper \
 && mkdir /data-empty

# ---------------------------------------------------------------- Texterkennung
# Tesseract (Deutsch, Englisch) und Poppler. Für Umbrel zusätzlich als eigenständiges Paket:
# Programme, Sprachdaten und genau die Bibliotheken, die im schlanken Debian fehlen.
FROM ${DEBIAN} AS ocr
RUN find /usr/lib -name '*.so*' -exec readlink -f {} \; | sort -u > /base-libs \
 && apt-get update \
 && apt-get install -y --no-install-recommends tesseract-ocr tesseract-ocr-deu tesseract-ocr-eng poppler-utils ca-certificates \
 && rm -rf /var/lib/apt/lists/* \
 && mkdir -p /ocr/bin /ocr/lib \
 && for b in tesseract pdftotext pdfinfo pdftocairo; do cp /usr/bin/$b /ocr/bin/; done \
 && for b in /ocr/bin/*; do ldd "$b" | awk '/=> \// {print $3}'; done | sort -u | while read -r lib; do \
      grep -qxF "$(readlink -f "$lib")" /base-libs || cp -L "$lib" /ocr/lib/; \
    done \
 && cp -r /usr/share/tesseract-ocr/5/tessdata /ocr/tessdata \
 && rm /ocr/tessdata/osd.traineddata \
 && ! ls /ocr/lib | grep -E '^(libc|libm|libstdc\+\+|libgcc_s|ld-linux)[.-]' \
 && LD_LIBRARY_PATH=/ocr/lib TESSDATA_PREFIX=/ocr/tessdata /ocr/bin/tesseract --list-langs

FROM scratch AS umbrel
COPY --from=build /nrzpaper /nrzpaper
COPY --from=ocr /ocr /ocr
COPY Rocket.toml /Rocket.toml

# ---------------------------------------------------------------- Laufzeit
FROM ocr AS runtime
RUN rm -rf /ocr /base-libs \
 && useradd --system --uid 1000 --home-dir /data --shell /usr/sbin/nologin nrzpaper
COPY --from=build --chown=1000:1000 /data-empty /data
COPY --from=build /nrzpaper /usr/local/bin/nrzpaper
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
