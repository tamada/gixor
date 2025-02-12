# Gixor

Git Ignore Managenemnt System for Multiple Repositories.
This is alternative tool for [gibo](https://github.com/simonwhitaker/gibo).

## :speaking_head: Overview

`gibo` is the great tool to manage the `.gitignore` file.
However, you may want to use your own `gitignore` boilerplate. 
`gibo` relies on `github.com/github/gitignore` and needs further configuration separately if you want to use your team's own `gitignore` repository.
Therefore, I just create a tool to manage `gitignore` files for multiple repositories.

## :runner: Usage

```shell
git ignore [OPTIONS] [ARGS...]
    or 
gixor --log <LOG> <COMMAND>

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

## About

### Product Name

Gixor means "GItignore indeX ORganizer," or "Git Ignorizer."
Pronounce it as "jigsaw" or "gigsor".
