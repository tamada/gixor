# gixor-cli

Gixor means "GitIgnore indeX ORganizer," and is pronounced as "jigsaw." It manages the
`.gitignore` files of your repositories from the boilerplates published by
[github/gitignore](https://github.com/github/gitignore) and others.

The crate is named `gixor-cli` so that the library keeps the `gixor` name on crates.io, but the
command it installs is `gixor`.

```shell
cargo install gixor-cli
```

or via [Homebrew](https://brew.sh/):

```shell
brew install tamada/tap/gixor
```

## Usage

```shell
gixor dump Rust          # add Rust to the .gitignore, keeping what is already there
gixor dump -Rust         # drop Rust from it
gixor dump --dry-run Go  # see the result without writing
gixor entries            # list what the .gitignore currently carries
gixor list               # list the available boilerplates
```

Building the boilerplates into the binary is done by [`gixor`](https://crates.io/crates/gixor),
the library this command is built on.

## License

MIT. See [LICENSE](https://github.com/tamada/gixor/blob/main/LICENSE).
