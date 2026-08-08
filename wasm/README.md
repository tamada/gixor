# gixor-wasm

Browser bindings for [gixor](https://github.com/tamada/gixor), the `.gitignore` manager.

The boilerplates are compiled into the module, so the page needs no network, no server and no
Git. It hands over the `.gitignore` it has and gets the new one back.

## Build

```shell
wasm-pack build wasm --target web
```

## Use

```js
import init, { list_boilerplates, generate } from "./pkg/gixor_wasm.js";

await init();

list_boilerplates();                       // ["Actionscript", "Ada", ..., "Zig"]
generate(["Rust", "macOS"], currentText);  // the new .gitignore
```

`generate` keeps whatever precedes the first boilerplate of `currentText` — the rules the reader
wrote themselves — and writes the rest from the boilerplates named. Pass `""` to start from
nothing. A name no build carries raises, rather than quietly leaving the rules out.

## Freshness

The boilerplates are a snapshot taken when the module was built, not the live contents of
[github/gitignore](https://github.com/github/gitignore). `snapshot_commits()` reports the commit
they came from.

## License

MIT. See [LICENSE](https://github.com/tamada/gixor/blob/main/LICENSE).
