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
    cargo build {{ target }}

# Run the test with cargo test
test: (build "") clone_for_test
    cargo llvm-cov --lcov --output-path target/coverage.lcov

# Run cargo fmt for formatting the source codes.
formats:
    cargo fmt -- --emit=files

# Run clippy for checking the source codes.
clippy:
    cargo clippy --all-targets                       -- -D warnings
    cargo clippy --all-targets --no-default-features -- -D warnings

clone_default_ignores:
    test -d testdata/boilerplates/default || git clone https://github.com/github/gitignore.git testdata/boilerplates/default

clone_tamada_ignores:
    test -d testdata/boilerplates/tamada || git clone https://github.com/tamada/gitignore.git testdata/boilerplates/tamada

clone_for_test: clone_default_ignores clone_tamada_ignores

# Generate the completion files
gen_complete:
    cargo run -- --generate-completion-files

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
        --build-arg APT_OPTIONAL="{{apt_optional}}" \
        --build-arg FEATURES="{{features}}" \
        -t ghcr.io/tamada/gixor:{{VERSION}}{{docker_tag_suffix}} .

# Build the docker image for multiple platforms and push them into ghcr.io
docker_buildx: _docker_buildx_default_feature _docker_buildx_systemgit_feature

_docker_buildx_default_feature:   (_docker_buildx "" "" "")
_docker_buildx_systemgit_feature: (_docker_buildx "--no-default-features" "git" "-systemgit")

_docker_buildx features apt_optional docker_tag_suffix:
    docker buildx build --platform linux/arm64/v8,linux/amd64 \
        --output=type=image,push=true \
        --build-arg APT_OPTIONAL="{{apt_optional}}" \
        --build-arg FEATURES="{{features}}" \
        -t ghcr.io/tamada/gixor:{{VERSION}}{{docker_tag_suffix}} .
