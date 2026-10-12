FROM rust:1-trixie AS builder

# Handed to the build script, which embeds it in `gixor --version`: there is no .git to ask here.
ARG GIT_REVISION=unknown

WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY cli ./cli
COPY lib ./lib

RUN     GIT_SHA=${GIT_REVISION} cargo build --release --locked -p gixor-cli \
    && mkdir -p /opt/gixor/boilerplates \
    && git clone https://github.com/github/gitignore.git /opt/gixor/boilerplates/default \
    && echo '{ \n\
    "base-path": "/opt/gixor/boilerplates",\n\
    "repositories": [\n\
        {\n\
            "url": "https://github.com/github/gitignore.git",\n\
            "repo-name": "gitignore",\n\
            "owner": "github",\n\
            "name": "default",\n\
            "path": "default"\n\
        }\n\
    ]\n\
}' > /opt/gixor/config.json

FROM debian:trixie-slim

ARG VERSION=0.5.2
ARG GIT_REVISION
ARG BUILD_DATE
ARG AUTHOR="Haruaki Tamada"
ARG TITLE=gixor
ARG URL=https://tamada.github.io/gixor
ARG SOURCE=https://github.com/tamada/gixor
ARG LICENSE=MIT

LABEL org.opencontainers.image.authors=${AUTHOR} \
      org.opencontainers.image.title=${TITLE} \
      org.opencontainers.image.description="Git Ignore Management System for Multiple Repositories." \
      org.opencontainers.image.url=${URL} \
      org.opencontainers.image.source=${SOURCE} \
      org.opencontainers.image.version=${VERSION} \
      org.opencontainers.image.revision=${GIT_REVISION} \
      org.opencontainers.image.created=${BUILD_DATE} \
      org.opencontainers.image.licenses=${LICENSE}

RUN    useradd --create-home --home-dir /app --shell /usr/sbin/nologin nonroot \
    && mkdir -p /app /opt/gixor/boilerplates \
    && apt-get update \
    && apt-get install --no-install-recommends -y ca-certificates \
    && apt-get clean \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/gixor     /opt/gixor/gixor
COPY --from=builder /opt/gixor                    /opt/gixor

RUN  chown -R nonroot:nonroot /opt/gixor

USER nonroot

WORKDIR /app

ENTRYPOINT [ "/opt/gixor/gixor", "--config", "/opt/gixor/config.json" ]
