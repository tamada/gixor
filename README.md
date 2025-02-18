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

Gixor means "GItignore indeX ORganizer," or "Git Ignorizer."
Pronounce it as "jigsaw" or "gigsor".

### Related Tools and Services

- [gibo](https://github.com/simonwhitaker/gibo)
- [gitignore.io](https://www.gitignore.io/)
- [bliss](https://github.com/ajmwagar/bliss)
- [gitignore-it](https://github.com/christopherkade/gitignore-it)
- [gitnr](https://github.com/reemus-dev/gitnr)
- [gig](https://github.com/shihanng/gig)
