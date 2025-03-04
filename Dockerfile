FROM rust:1-bullseye AS builder

WORKDIR /app
COPY . .
RUN    cargo build --release \
    && mkdir -p /opt/gixor/boilerplates \
    && git clone https://github.com/github/gitignore.git /opt/gixor/boilerplates/default \
    && echo '{ \n\
    "base-path": "boilerplates",\n\
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

FROM debian:bullseye-slim

ARG VERSION=0.2.8

LABEL   org.opencontainers.image.source=https://github.com/tamada/gixor \
        org.opencontainers.image.version=${VERSION} \
        org.opencontainers.image.title=gixor \
        org.opencontainers.image.description="Git Ignore Managenemnt System for Multiple Repositories."

RUN    adduser --disabled-password --disabled-login --home /opt/gixor nonroot \
    && mkdir -p /app /opt/gixor/boilerplates
COPY --from=builder /app/target/release/gixor-cli /opt/gixor/gixor
COPY --from=builder /opt/gixor /opt/gixor

USER nonroot

WORKDIR /app

ENTRYPOINT [ "/opt/gixor/gixor", "--config", "/opt/gixor/config.json" ]
