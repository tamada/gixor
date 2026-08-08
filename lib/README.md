# gixor

An API for managing `.gitignore` files, and the library behind the
[`gixor`](https://github.com/tamada/gixor) command.

It keeps clones of boilerplate repositories such as
[github/gitignore](https://github.com/github/gitignore), finds the boilerplates you name, and
builds a `.gitignore` from them while keeping what you had written yourself.

```rust
use gixor::{GixorFactory, Name};

let gixor = GixorFactory::load("testdata/config.json").unwrap();
gixor.prepare(true).unwrap(); // clone or update the repositories, if needed
let names = Name::parse_all(vec!["rust", "macos", "linux", "windows"]);
gixor.dump(names, std::io::stdout(), false).unwrap();
```

## Features

- `usegix` (default): drive Git through [`gix`](https://docs.rs/gix/), a pure Rust
  implementation. Nothing has to be installed alongside, and no C library is linked.
- `--no-default-features --features local`: run the `git` command instead, which requires `git`
  on the `PATH`.

## Install the command

The command lives in a separate crate so that using the library does not pull the
command line dependencies in.

```shell
cargo install gixor-cli
```

## License

MIT. See [LICENSE](https://github.com/tamada/gixor/blob/main/LICENSE).
