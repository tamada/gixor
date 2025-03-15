---
title: "Gixor"
description: "Git Ignore Management System for Multiple Repositories."
date: 2025-02-25
---

[![build](https://github.com/tamada/gixor/actions/workflows/build.yaml/badge.svg)](https://github.com/tamada/gixor/actions/workflows/build.yaml)
[![Coverage Status](https://coveralls.io/repos/github/tamada/gixor/badge.svg?branch=main)](https://coveralls.io/github/tamada/gixor?branch=main)
[![Rust Report Card](https://rust-reportcard.xuri.me/badge/github.com/tamada/gixor)](https://rust-reportcard.xuri.me/report/github.com/tamada/gixor)

[![Version](https://img.shields.io/badge/Version-v0.2.8-green)](https://github.com/tamada/gixor/releases/tag/v0.2.8)
[![License](https://img.shields.io/badge/License-MIT-green)](https://github.com/tamada/gixor/blob/main/LICENSE)

[![Docker](https://img.shields.io/badge/Docker-ghcr.io/tamada/gixor:0.2.8-blue?logo=docker)](https://github.com/tamada/gixor/pkgs/container/gixor/)
[![Homebrew](https://img.shields.io/badge/Homebrew-tamada/tap/gixor-blue?logo=homebrew)](https://github.com/tamada/homebrew-tap)

Gixor is Git Ignore Managenemnt System for Multiple Repositories.
This is alternative tool for [gibo](https://github.com/simonwhitaker/gibo).

## 🗣️ Overview

`gibo` is the great tool to manage the `.gitignore` file.
However, you may want to use your own `gitignore` boilerplate. 
`gibo` relies on `github.com/github/gitignore` and needs further configuration separately if you want to use your team's own `gitignore` repository.
Therefore, I just create a tool to manage `gitignore` files for multiple repositories.

## 🏃 Usage

```shell
git ignore [OPTIONS] [ARGS...]
    or
gixor [OPTIONS] <COMMAND>

Commands:
  dump        Dump the boilerplates
  entries     List the current entries in the .gitignore file
  list        List available boilerplates
  root        Show the root directory of the boilerplate
  search      Search the boilerplates from the query
  update      Update the gitignore boilerplate repositories (alias of `repository update`)
  repository  Manage the gitignore boilerplate repositories
  help        Print this message or the help of the given subcommand(s)

Options:
  -l, --log <LOG>             Specify the log level [default: warn] [possible values: trace, debug, info, warn, error]
  -c, --config <CONFIG_JSON>  Specify the configuration file
  -h, --help                  Print help
  -V, --version               Print version
```

Gixor clones and updates the boilerplate repositories into the local root directory if necessary.
The local root directory is specified in the configuration file.

## ℹ️ About

### 👩‍💻 Authors 👨‍💻

* Haruaki Tamada [GitHub](https://github.com/tamada) [🌏](https://tamada.github.io/)

### 🎃 Product Name

Gixor means "GitIgnore indeX ORganizer," and
pronounce it as "jigsaw".

### 🔗 Related Tools and Services

* [gibo](https://github.com/simonwhitaker/gibo)
* [gitignore.io](https://www.gitignore.io/)
* [bliss](https://github.com/ajmwagar/bliss)
* [gitignore-it](https://github.com/christopherkade/gitignore-it)
* [gitnr](https://github.com/reemus-dev/gitnr)
* [gig](https://github.com/shihanng/gig)
