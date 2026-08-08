[private]
@default: help

VERSION := `grep '^version = ' Cargo.toml | sed 's/version = "\(.*\)"/\1/g'`

# show help message
@help:
    echo "Task runner for Gixor {{ VERSION }} with Just"
    echo "Usage: just <recipe>"
    echo ""
    just --list

# Build the project with cargo build
build target = "": formats clippy
    cargo build --workspace {{ target }}

# Run the test with cargo test
test: (build "") clone_for_test
    cargo llvm-cov --lcov --output-path target/coverage.lcov

# Run cargo fmt for formatting the source codes.
formats:
    cargo fmt -- --emit=files

# Run clippy for checking the source codes.
clippy:
    cargo clippy --workspace --all-targets -- -D warnings
    cargo clippy -p gixor --all-targets --no-default-features --features local    -- -D warnings
    cargo clippy -p gixor --all-targets --no-default-features --features embedded -- -D warnings

clone_default_ignores:
    test -d lib/testdata/boilerplates/default || git clone https://github.com/github/gitignore.git lib/testdata/boilerplates/default

clone_tamada_ignores:
    test -d lib/testdata/boilerplates/tamada || git clone https://github.com/tamada/gitignore.git lib/testdata/boilerplates/tamada

clone_for_test: clone_default_ignores clone_tamada_ignores

# Refresh the boilerplate snapshot that the embedded build, and therefore wasm, carries.
# The snapshot is committed rather than fetched at build time: cargo package does not carry
# submodules, so anything not committed would reach crates.io empty.
vendor_boilerplates:
    #!/usr/bin/env bash
    set -euo pipefail
    tmp=$(mktemp -d)
    trap 'rm -rf "$tmp"' EXIT
    git clone --depth 1 https://github.com/github/gitignore.git "$tmp/default"
    commit=$(git -C "$tmp/default" rev-parse HEAD)
    rm -rf lib/boilerplates/default
    mkdir -p lib/boilerplates/default
    rsync -a --prune-empty-dirs --include='*/' --include='*.gitignore' --exclude='*' \
        "$tmp/default/" lib/boilerplates/default/
    printf '# repository url owner repo-name commit\ndefault https://github.com/github/gitignore.git github gitignore %s\n' \
        "$commit" > lib/boilerplates/SOURCES
    echo "vendored $(find lib/boilerplates/default -name '*.gitignore' | wc -l | tr -d ' ') boilerplates at $commit"

# Build the wasm module for the browser. It stands outside the workspace; see the root Cargo.toml.
wasm:
    wasm-pack build wasm --target web

# Check that the library still builds for the browser, without needing wasm-pack.
# It installs the target itself, so that neither `clippy` nor a fresh checkout has to carry a
# target that only this crate needs.
wasm_check:
    rustup target add wasm32-unknown-unknown
    cargo clippy --manifest-path wasm/Cargo.toml --target wasm32-unknown-unknown -- -D warnings

# Serve wasm/demo.html for a manual check, at http://localhost:8765/demo.html.
# A server is needed rather than opening the file: modules and wasm are not fetched from file://.
wasm_demo: wasm
    cd wasm && python3 -m http.server 8765

# Generate the completion files
gen_complete:
    cargo run -p gixor-cli -- --generate-completion-files

prepare_site_build:
    test -d docs/public || git worktree add -f docs/public gh-pages

# Generate the document site with Hugo
site: prepare_site_build
    docker run --rm -it -v $PWD/docs:/src hugomods/hugo:exts-non-root hugo
    rm -f docs/public/favicon* docs/public/{android-chome-*,apple-touch-icon}.png

    rm -rf docs/public/coverage
    cargo llvm-cov --html
    cp -r target/llvm-cov/html docs/public/coverage

    # Generate API document with cargo doc
    cargo doc
    cp -r target/doc/gixor docs/public/api
    cp -r target/doc/static.files docs/public/

# Start hugo server with Docker
start:
    docker run --name gixorwww --rm -it -p 1313:1313 -v $PWD/docs:/src hugomods/hugo:exts-non-root hugo server

# Stop the running hugo server
stop:
    docker stop gixorwww

# Build the docker images of gixor with different backends.
# The default image carries gix, so it needs no git installed alongside.
docker: _docker_build_default_feature _docker_build_systemgit_feature

_docker_build_default_feature:   (_docker_build "" "" "")
_docker_build_systemgit_feature: (_docker_build "--no-default-features" "git" "-systemgit")

_docker_build features apt_optional docker_tag_suffix:
    docker build \
        -f Containerfile \
        --build-arg APT_OPTIONAL="{{apt_optional}}" \
        --build-arg FEATURES="{{features}}" \
        -t quay.io/tama5/gixor:{{VERSION}}{{docker_tag_suffix}} .

# Build the docker image for multiple platforms and push them into quay.io
docker_buildx: _docker_buildx_default_feature _docker_buildx_systemgit_feature

_docker_buildx_default_feature:   (_docker_buildx "" "" "")
_docker_buildx_systemgit_feature: (_docker_buildx "--no-default-features" "git" "-systemgit")

_docker_buildx features apt_optional docker_tag_suffix:
    docker buildx build \
        -f Containerfile \
        --platform linux/arm64/v8,linux/amd64 \
        --output=type=image,push=true \
        --build-arg APT_OPTIONAL="{{apt_optional}}" \
        --build-arg FEATURES="{{features}}" \
        -t quay.io/tama5/gixor:{{VERSION}}{{docker_tag_suffix}} .
