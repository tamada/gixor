---
title: ":anchor: Install"
date: "2025-09-27"
---

## :beer: Homebrew

[![Homebrew](https://img.shields.io/badge/Homebrew-tamada/tap/gixor-blue?logo=homebrew)](https://github.com/tamada/homebrew-tap)

You can install `gixor` via [:beer: Homebrew](https://brew.sh/).

```shell
brew install tamada/tap/gixor
```

## :crab: Cargo

The command lives in the `gixor-cli` crate, so that using `gixor` as a library does not pull the
command line dependencies in. The installed command is still called `gixor`.

```shell
cargo install gixor-cli
```

## :package: Prebuilt binaries

Each [release](https://github.com/tamada/gixor/releases) carries the command for Linux and macOS
(`tar.gz`, amd64 and arm64) and for Windows (`zip`, amd64). Extract the archive and put `gixor`
(`gixor.exe` on Windows) on the `PATH`.

On Windows, the configuration lives in `%APPDATA%\gixor\config.json`, and the boilerplate
repositories are cloned next to it, in `%APPDATA%\gixor\boilerplates`.

## :muscle: Compile yourself

```shell
git clone https://github.com/tamada/gixor.git
cd gixor
cargo build --release -p gixor-cli
cp target/release/gixor /install/path/of/gixor
```
