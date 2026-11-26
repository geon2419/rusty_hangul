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
| `core/` | `rusty_hangul` (imported as `hangul`) | Rust crate |
| `node/` | `node` | Node.js native addon via [napi-rs](https://napi.rs/) |
| `wasm/` | `wasm` (`hangul-wasm`) | WebAssembly bindings via wasm-bindgen |

A browser demo lives in [`examples/browser-basic`](./examples/browser-basic).

## Installation

You need a [Rust toolchain](https://rustup.rs/). Node.js and WebAssembly bindings also need [bun](https://bun.sh/) (`>= 1.3.3`). Node.js bindings require Node `>= 20`. WebAssembly builds also need [`wasm-pack`](https://rustwasm.github.io/wasm-pack/).

### Rust

The crates.io package is `rusty_hangul`. In Rust the crate is still `hangul`, so examples use `use hangul::...`. The name `hangul` is already taken on crates.io.

From crates.io:

```toml
[dependencies]
rusty_hangul = "0.1.1"
```

From a git tag:

```toml
[dependencies]
rusty_hangul = { git = "https://github.com/geon2419/rusty_hangul", tag = "v0.1.1" }
```

From a local clone, point at `core/`:

```toml
[dependencies]
rusty_hangul = { path = "path/to/rusty_hangul/core" }
```

### Node.js and WebAssembly

Clone this repository at the release tag, then build the binding you need.

```sh
git clone --branch v0.1.1 --depth 1 https://github.com/geon2419/rusty_hangul
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

## Benchmarks

Both harnesses read the same inputs from [`benchmark/fixtures`](./benchmark/fixtures), so the Rust, Node.js, WebAssembly, and es-hangul numbers describe identical work.

| Command | What it measures |
| --- | --- |
| `cargo bench -p rusty_hangul_benchmark` | Rust core, via [criterion](https://github.com/bheisler/criterion.rs) |
| `bun run bench` | es-hangul, the Node.js binding, and the WebAssembly binding, via `vitest bench` |
| `bun run bench:report` | both harnesses at once, merged into a single table |

`bun run bench:report` builds `wasm/pkg` and the Node.js addon when they are missing, so a fresh checkout only needs Rust, bun, and [`wasm-pack`](https://rustwasm.github.io/wasm-pack/).

Benchmark names read `작업 · 입력` (what it does · what it runs on). Mean throughput in operations per second on an Apple silicon laptop (macOS arm64): read the numbers as orders of magnitude, not as precise scores.

| Benchmark | core | node | wasm | es-hangul |
| --- | ---: | ---: | ---: | ---: |
| 글자 분해 + 초성 추출 · 이름 (3자) | 3.50M | 983.1K | 1.03M | 1.31M |
| 글자 분해 + 초성 추출 · 문장 (42자) | 797.4K | 287.7K | 265.3K | 146.5K |
| 글자 분해 + 초성 추출 · 문단 (176자, 4줄) | 228.8K | 67.4K | 66.3K | 28.3K |
| 글자 분해 + 초성 추출 · 긴 문단 (407자, 9줄) | 98.1K | 37.6K | 30.1K | 13.3K |
| 글자 분해 + 초성 추출 · 한영·숫자 혼합 (41자) | 1.93M | 374.6K | 555.8K | 296.0K |
| 글자 분해 + 초성 추출 · 자모로 풀린 텍스트 (14자) | 1.59M | 683.3K | 624.6K | 689.6K |
| 초성·중성·종성 분해 · 이름 (3자) | 5.28M | 644.7K | 628.1K | 2.38M |
| 초성·중성·종성 분해 · 문장 (42자) | 648.6K | 79.7K | 70.6K | 590.8K |
| 초성·중성·종성 분해 · 문단 (176자, 4줄) | 160.3K | 19.8K | 17.5K | 144.5K |
| 받침 확인 · 이름 (3자) | 23.01M | 1.83M | 5.82M | 24.77M |
| 받침 확인 · 문장 (42자) | 6.59M | 990.2K | 2.08M | 27.23M |
| 조사 선택 (을/를) · 이름 (3자) | 15.39M | 1.75M | 2.31M | 9.53M |
| 자모 조합 · 자모 입력 (5자) | 8.62M | 4.51M | 3.52M | 251.2K |
| 초성 포함 확인 · 문장 (42자) | 4.38M | 1.10M | 1.60M | - |
| 초성 포함 확인 · 문단 (176자, 4줄) | 1.64M | 549.1K | 597.1K | - |
| 초성 위치 찾기 · 문장 (42자) | 4.39M | 855.8K | 1.33M | - |
| 초성 위치 찾기 · 문단 (176자, 4줄) | 1.66M | 492.8K | 540.5K | - |

<details>
<summary>Benchmark → API</summary>

| Benchmark | API |
| --- | --- |
| 글자 분해 + 초성 추출 | `disassemble` + `getChoseong` |
| 초성·중성·종성 분해 | `disassembleToGroups` |
| 받침 확인 | `hasBatchim` |
| 조사 선택 (을/를) | `josa("을/를")` |
| 자모 조합 | `assemble` |
| 초성 포함 확인 | `containsChoseong("ㄷㅎ")` |
| 초성 위치 찾기 | `findChoseong("ㄴㄹ")` |

</details>

- es-hangul wins on very short inputs because it never crosses the FFI boundary; the gap reverses as input grows.
- The WebAssembly lane copies strings in and out of linear memory, so it trails the Node.js lane on long inputs even though both wrap the same Rust core.
- es-hangul has no choseong search, so 초성 포함 확인 and 초성 위치 찾기 compare only the Rust lanes.
- Each row times the whole call, including the wrapper the bindings create around the input.

The `benchmark` GitHub Actions workflow runs the report on manual dispatch only and uploads `benchmark-report.json` as an artifact.

## For AI coding agents

See [`llms.txt`](./llms.txt) for a short API overview.

## License

[MIT](./LICENSE)
