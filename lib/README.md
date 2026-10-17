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

- `local` (default): clone the boilerplate repositories with [`gix`](https://docs.rs/gix/), a pure
  Rust Git, and read them from the file system. No `git` command has to be installed. HTTPS goes
  through rustls, whose crypto provider `aws-lc-rs` builds a C library (`aws-lc-sys`, not
  OpenSSL), so a C compiler is needed to build it.
- `embedded`: compile a snapshot of the boilerplates into the binary instead, needing neither a
  clone nor a file system, as for WebAssembly. It cannot be enabled together with `local`:
  build with `--no-default-features --features embedded`.

## Install the command

The command lives in a separate crate so that using the library does not pull the
command line dependencies in.

```shell
cargo install gixor-cli
```

## License

MIT. See [LICENSE](https://github.com/tamada/gixor/blob/main/LICENSE).
