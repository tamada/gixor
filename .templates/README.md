# Gixor

[![build](https://github.com/tamada/gixor/actions/workflows/build.yaml/badge.svg)](https://github.com/tamada/gixor/actions/workflows/build.yaml)
[![Coverage Status](https://coveralls.io/repos/github/tamada/gixor/badge.svg?branch=main)](https://coveralls.io/github/tamada/gixor?branch=main)
[![Rust Report Card](https://rust-reportcard.xuri.me/badge/github.com/tamada/gixor)](https://rust-reportcard.xuri.me/report/github.com/tamada/gixor)

[![Version](https://img.shields.io/badge/Version-v$VERSION-green)](https://github.com/tamada/gixor/releases/tag/v$VERSION)
[![License](https://img.shields.io/badge/License-MIT-green)](https://github.com/tamada/gixor/blob/main/LICENSE)

[![Docker](https://img.shields.io/badge/Docker-ghcr.io/tamada/gixor:$VERSION-blue?logo=docker)](https://github.com/tamada/gixor/pkgs/container/gixor/)
[![Homebrew](https://img.shields.io/badge/Homebrew-tamada/tap/gixor-blue?logo=homebrew)](https://github.com/tamada/homebrew-tap)

Git Ignore Managenemnt System for Multiple Repositories.
This is alternative tool for [gibo](https://github.com/simonwhitaker/gibo).

## :speaking_head: Overview

`gibo` is the great tool to manage the `.gitignore` file.
However, `gibo` uses [`github.com/github/gitignore`](https://github.com/github/gitignore) as the default and only repository, and we cannot use own `gitignore` boilerplates.
Then, we needs further configuration aprt from `gibo` if the team want to use their own `gitignore` repository.
Therefore, I just create a tool named `gixor` to manage `gitignore` files for multiple repositories.

`gixor` is also uses [`github.com/github/gitignore`](https://github.com/github/gitignore) as the default repository (no explicit `git clone`).
Then, the team want to use their own `gitignore` repository, run `gixor repository add <GIT_URL>` to add the repository.

## :runner: Usage

```shell
git ignore [OPTIONS] [ARGS...]
    or 
gixor [OPTIONS] <COMMAND>

Commands:
  dump        Dump the boilerplates
  entries     List the the current entries in the .gitignore file
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

## About

### Product Name

Gixor means "GitIgnore indeX ORganizer," and pronounce it as "jigsaw."

### Related Tools and Services

- [gibo](https://github.com/simonwhitaker/gibo) (Go lang)
- [gitignore.io](https://www.gitignore.io/) (Swift, Less, JavaScript, ...)
- [bliss](https://github.com/ajmwagar/bliss) (Rust)
- [gitignore-it](https://github.com/christopherkade/gitignore-it) (JavaScript)
- [gitnr](https://github.com/reemus-dev/gitnr) (Rust)
- [gig](https://github.com/shihanng/gig) (Go lang)
