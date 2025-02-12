# Gibbs

Git Ignore Boilerplates Base System.
This is alternative tool for [gibo](https://github.com/simonwhitaker/gibo).

## :speaking_head: Overview

`gibo` は便利であるが，独自の `gitignore` も使いたい場合もある．
`gibo` は `github.com/github/gitignore` に依存しており，チーム独自の `gitignore`リポジトリを使いたい場合は別途設定が必要となる．
そこで，複数のリポジトリを対象にした `gitignore` ファイルの管理を行うツールを作成する．

## :runner: Usage

```shell
git ignore [OPTIONS] [ARGS...]
    or 
gibbs --log <LOG> <COMMAND>

Commands:
  dump        Dump the boilerplates
  show        Show the entries in the .gitignore file
  list        List available boilerplates
  root        Show the root directory of the boilerplate
  serach      Search the boilerplates from the query
  update      Update the gitignore boilerplate repositories
  repository  Manage the gitignore boilerplate repositories
  help        Print this message or the help of the given subcommand(s)

Options:
  -l, --log <LOG>  Specify the log level [possible values: trace, debug, info, warn, error]
  -h, --help       Print help
  -V, --version    Print version
```

