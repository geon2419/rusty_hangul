# rusty_hangul

Hangul string processing in Rust — syllable disassembly, choseong extraction, batchim/josa selection, and jamo assembly — with Node.js and WebAssembly bindings.

[한국어](./README.ko.md)

## Features

- Disassemble Hangul syllables into jamo (NFC and NFD), including grouped output
- Iterate syllables and inspect choseong / jungseong / jongseong
- Extract choseong (initial consonants), leaving non-Hangul characters as-is
- Detect batchim (final consonants) and select a matching josa (or the particle alone)
- Assemble jamo back into syllables

## Packages

This repository is a Cargo and bun workspace. **Nothing is published to crates.io or npm yet.**

| Path | Package name | Role |
| --- | --- | --- |
| `core/` | `hangul` | Rust crate |
| `node/` | `node` | Node.js native addon via [napi-rs](https://napi.rs/) |
| `wasm/` | `wasm` (`hangul-wasm`) | WebAssembly bindings via wasm-bindgen |

A browser demo lives in [`examples/browser-basic`](./examples/browser-basic).

## Installation

You need a [Rust toolchain](https://rustup.rs/) and [bun](https://bun.sh/) (`>= 1.3.3`). Node.js bindings require Node `>= 20`. WebAssembly builds also need [`wasm-pack`](https://rustwasm.github.io/wasm-pack/).

Clone the repo, then:

```sh
bun install
```

### Rust

The crate name is `hangul`, not `rusty_hangul`.

From git:

```toml
[dependencies]
hangul = { git = "https://github.com/geon2419/rusty_hangul" }
```

From a local clone:

```toml
[dependencies]
hangul = { path = "path/to/rusty_hangul/core" }
```

### Node.js

Build the native addon, then import the workspace package named `node`:

```sh
bun run node:build
```

```ts
import { Hangul, assemble } from "node";
```

That import works for other packages in this bun workspace. Outside the workspace, point at `./node` after a local build.

### WebAssembly

```sh
bun run wasm:build
```

```ts
import init, { disassemble, getChoseong } from "./wasm/pkg/hangul";

await init();
```

Adjust the import path to match your app. See [`examples/browser-basic`](./examples/browser-basic) for a Vite setup.

## Rust example

```rust
use hangul::{assemble, assemble_with_policy, AssemblePolicy, Hangul};

let text = Hangul::new("안녕하세요");
assert_eq!(text.disassemble(), "ㅇㅏㄴㄴㅕㅇㅎㅏㅅㅔㅇㅛ");
assert_eq!(
  text.disassemble_to_groups(),
  vec![
    vec!['ㅇ', 'ㅏ', 'ㄴ'],
    vec!['ㄴ', 'ㅕ', 'ㅇ'],
    vec!['ㅎ', 'ㅏ'],
    vec!['ㅅ', 'ㅔ'],
    vec!['ㅇ', 'ㅛ'],
  ]
);
assert_eq!(text.get_choseong(), "ㅇㄴㅎㅅㅇ");
assert_eq!(text.len(), 5);
assert!(text.get(0).unwrap().is_hangul());

let mixed = Hangul::new("Hello 안녕!");
assert_eq!(mixed.get_choseong(), "Hello ㅇㄴ!");

let nfd = Hangul::new("\u{1100}\u{1161}\u{11AB}"); // 간
assert_eq!(nfd.disassemble(), "ㄱㅏㄴ");

assert!(!Hangul::new("사과").has_batchim());
assert_eq!(Hangul::new("사과").josa("을/를").unwrap(), "사과를");
assert_eq!(Hangul::new("수박").josa("을/를").unwrap(), "수박을");
assert_eq!(Hangul::new("수박").josa("아/야").unwrap(), "수박아");
assert_eq!(Hangul::new("사과").josa_particle("을/를").unwrap(), "를");

assert_eq!(assemble("ㄱㅏㅂㅅ"), "값");
assert_eq!(
  assemble_with_policy("ㄱㅏㄱㅅㅏ", AssemblePolicy::PreferCompoundFinal),
  "갃ㅏ"
);
```

## Node.js example

```typescript
import { Hangul, assemble } from "node";

const text = new Hangul("안녕하세요");
console.log(text.disassemble()); // "ㅇㅏㄴㄴㅕㅇㅎㅏㅅㅔㅇㅛ"
console.log(text.disassembleToGroups());
// [["ㅇ", "ㅏ", "ㄴ"], ["ㄴ", "ㅕ", "ㅇ"], ["ㅎ", "ㅏ"], ["ㅅ", "ㅔ"], ["ㅇ", "ㅛ"]]
console.log(text.getChoseong()); // "ㅇㄴㅎㅅㅇ"
console.log(text.length); // 5
console.log(text.get(0)); // { original: "안", isHangul: true, choseong: "ㅇ", jungseong: "ㅏ", jongseong: "ㄴ" }

const mixed = new Hangul("Hello 안녕!");
console.log(mixed.getChoseong()); // "Hello ㅇㄴ!"

console.log(new Hangul("한").hasBatchim()); // true
console.log(new Hangul("사과").josa("을/를")); // "사과를"
console.log(new Hangul("수박").josa("아/야")); // "수박아"
console.log(new Hangul("사과").josaParticle("을/를")); // "를"

console.log(assemble("ㄱㅏㅂㅅ")); // "값"
console.log(assemble("ㄱㅏㄱㅅㅏ", "compound-final")); // "갃ㅏ"
```

## WebAssembly example

WASM exposes free functions, not a `Hangul` class.

```typescript
import init, {
  assemble,
  disassemble,
  disassembleToGroups,
  getChoseong,
  hasBatchim,
  josa,
  josaParticle,
  unitAt,
} from "./wasm/pkg/hangul";

await init();

console.log(disassemble("안녕하세요")); // "ㅇㅏㄴㄴㅕㅇㅎㅏㅅㅔㅇㅛ"
console.log(disassembleToGroups("값")); // [["ㄱ", "ㅏ", "ㅂ", "ㅅ"]]
console.log(getChoseong("Hello 안녕!")); // "Hello ㅇㄴ!"
console.log(hasBatchim("한")); // true
console.log(josa("사과", "을/를")); // "사과를"
console.log(josaParticle("사과", "을/를")); // "를"
console.log(unitAt("가A", 1)); // { original: "A", isHangul: false, ... }
console.log(assemble("ㄱㅏㅂㅅ")); // "값"
```

## License

[MIT](./LICENSE)
