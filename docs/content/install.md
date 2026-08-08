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

## :muscle: Compile yourself

```shell
git clone https://github.com/tamada/gixor.git
cd gixor
cargo build --release -p gixor-cli
cp target/release/gixor /install/path/of/gixor
```
