# rusty_hangul

Hangul string processing in Rust — syllable disassembly, choseong extraction, batchim/josa selection, and jamo assembly — with Node.js and WebAssembly bindings.

[한국어](./README.ko.md)

## Features

- Disassemble Hangul syllables into jamo (NFC and NFD), including grouped output
- Iterate syllables and inspect choseong / jungseong / jongseong
- Extract choseong (initial consonants), leaving non-Hangul characters as-is
- Search text by choseong or progressive syllable prefixes
- Detect batchim (final consonants) and select a matching josa (or the particle alone)
- Assemble jamo back into syllables

## Packages

This repository is a Cargo and bun workspace. Tagged releases are on the [Releases](https://github.com/geon2419/rusty_hangul/releases) page.

| Path | Package name | Role |
| --- | --- | --- |
| `core/` | `hangul` | Rust crate |
| `node/` | `node` | Node.js native addon via [napi-rs](https://napi.rs/) |
| `wasm/` | `wasm` (`hangul-wasm`) | WebAssembly bindings via wasm-bindgen |

A browser demo lives in [`examples/browser-basic`](./examples/browser-basic).

## Installation

You need a [Rust toolchain](https://rustup.rs/). Node.js and WebAssembly bindings also need [bun](https://bun.sh/) (`>= 1.3.3`). Node.js bindings require Node `>= 20`. WebAssembly builds also need [`wasm-pack`](https://rustwasm.github.io/wasm-pack/).

### Rust

The crate name is `hangul`, not `rusty_hangul`.

```toml
[dependencies]
hangul = { git = "https://github.com/geon2419/rusty_hangul", tag = "v0.1.0" }
```

From a local clone, point at `core/`:

```toml
[dependencies]
hangul = { path = "path/to/rusty_hangul/core" }
```

### Node.js and WebAssembly

Clone this repository at the release tag, then build the binding you need.

```sh
git clone --branch v0.1.0 --depth 1 https://github.com/geon2419/rusty_hangul
cd rusty_hangul
bun install
bun run node:build   # native addon
# or
bun run wasm:build
```

```ts
import { Hangul, assemble } from "node";
```

That import works for other packages in this bun workspace. Outside the workspace, point at `./node` after a local build.

```ts
import init, { disassemble, getChoseong } from "./wasm/pkg/hangul";

await init();
```

Adjust the WASM import path to match your app. See [`examples/browser-basic`](./examples/browser-basic) for a Vite setup.

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
assert!(text.contains_choseong("ㅇㄴㅎ"));
assert_eq!(text.find_choseong("ㅎㅅ").unwrap().start, 2);
assert_eq!(text.len(), 5);
assert!(text.get(0).unwrap().is_hangul());

let mixed = Hangul::new("Hello 안녕!");
assert_eq!(mixed.get_choseong(), "Hello ㅇㄴ!");

let nfd = Hangul::new("\u{1100}\u{1161}\u{11AB}"); // 간
assert_eq!(nfd.disassemble(), "ㄱㅏㄴ");
assert_eq!(nfd.get(0).unwrap().original(), "\u{1100}\u{1161}\u{11AB}");

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
console.log(text.containsChoseong("ㅇㄴㅎ")); // true
console.log(text.findChoseong("ㅎㅅ")); // { start: 2, end: 4, byteStart, byteEnd }
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
  containsChoseong,
  findChoseong,
} from "./wasm/pkg/hangul";

await init();

console.log(disassemble("안녕하세요")); // "ㅇㅏㄴㄴㅕㅇㅎㅏㅅㅔㅇㅛ"
console.log(disassembleToGroups("값")); // [["ㄱ", "ㅏ", "ㅂ", "ㅅ"]]
console.log(getChoseong("Hello 안녕!")); // "Hello ㅇㄴ!"
console.log(containsChoseong("한글", "ㅎㄱ")); // true
console.log(findChoseong("한글", "한ㄱ")); // { start: 0, end: 2, ... }
console.log(hasBatchim("한")); // true
console.log(josa("사과", "을/를")); // "사과를"
console.log(josaParticle("사과", "을/를")); // "를"
console.log(unitAt("가A", 1)); // { original: "A", isHangul: false, ... }
console.log(assemble("ㄱㅏㅂㅅ")); // "값"
```

## For AI coding agents

See [`llms.txt`](./llms.txt) for a short API overview.

## License

[MIT](./LICENSE)
