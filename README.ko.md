# rusty_hangul

Rust로 만든 한글 문자열 처리 라이브러리입니다. 음절 분해, 초성 추출, 받침·조사 선택, 자모 조립을 제공하고 Node.js·WebAssembly 바인딩을 함께 둡니다.

[English](./README.md)

## 기능

- 한글 음절을 자모로 분해 (NFC / NFD, 그룹 출력 포함)
- 음절 단위 순회 및 초·중·종성 확인
- 초성 추출 (한글이 아닌 문자는 그대로 유지)
- 초성·음절 접두로 검색
- 받침 확인 및 조사 선택 (조사만 반환하는 API 포함)
- 자모를 음절로 조립

## 구성

이 저장소는 Cargo와 bun 워크스페이스입니다. 태그된 릴리스는 [Releases](https://github.com/geon2419/rusty_hangul/releases)에 있습니다.

| 경로 | 패키지 이름 | 역할 |
| --- | --- | --- |
| `core/` | `rusty_hangul` (코드에서는 `hangul`) | Rust 크레이트 |
| `node/` | `node` | [napi-rs](https://napi.rs/) Node.js 네이티브 애드온 |
| `wasm/` | `wasm` (`hangul-wasm`) | wasm-bindgen WebAssembly 바인딩 |

브라우저 데모는 [`examples/browser-basic`](./examples/browser-basic)에 있습니다.

## 설치

[Rust 툴체인](https://rustup.rs/)이 필요합니다. Node.js·WebAssembly 바인딩은 [bun](https://bun.sh/) (`>= 1.3.3`)도 필요합니다. Node.js 바인딩은 Node `>= 20`이 필요하고, WebAssembly 빌드에는 [`wasm-pack`](https://rustwasm.github.io/wasm-pack/)이 필요합니다.

### Rust

crates.io 패키지 이름은 `rusty_hangul`입니다. Rust 코드에서는 계속 `hangul`이므로 예시는 `use hangul::...`입니다. crates.io에 `hangul`이 이미 있습니다.

crates.io에서:

```toml
[dependencies]
rusty_hangul = "0.1.1"
```

git 태그에서:

```toml
[dependencies]
rusty_hangul = { git = "https://github.com/geon2419/rusty_hangul", tag = "v0.1.1" }
```

로컬 클론에서는 `core/`를 가리킵니다.

```toml
[dependencies]
rusty_hangul = { path = "path/to/rusty_hangul/core" }
```

### Node.js와 WebAssembly

릴리스 태그를 클론한 뒤 필요한 바인딩을 빌드합니다.

```sh
git clone --branch v0.1.1 --depth 1 https://github.com/geon2419/rusty_hangul
cd rusty_hangul
bun install
bun run node:build   # 네이티브 애드온
# 또는
bun run wasm:build
```

```ts
import { Hangul, assemble } from "node";
```

이 import는 이 bun 워크스페이스 안의 다른 패키지에서 동작합니다. 워크스페이스 밖에서는 로컬 빌드 후 `./node`를 가리키면 됩니다.

```ts
import init, { disassemble, getChoseong } from "./wasm/pkg/hangul";

await init();
```

WASM import 경로는 앱 구조에 맞게 바꾸면 됩니다. Vite 설정은 [`examples/browser-basic`](./examples/browser-basic)을 참고하세요.

## Rust 예시

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

## Node.js 예시

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

## WebAssembly 예시

WASM은 `Hangul` 클래스가 아니라 함수를 노출합니다.

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

## 벤치마크

Rust, Node.js, WebAssembly, es-hangul 네 가지 구현이 [`benchmark/fixtures`](./benchmark/fixtures)의 동일한 입력을 사용하므로 같은 작업을 측정합니다.

| 명령 | 측정 대상 |
| --- | --- |
| `cargo bench -p rusty_hangul_benchmark` | [criterion](https://github.com/bheisler/criterion.rs) 기반 Rust 코어 |
| `bun run bench` | `vitest bench` 기반 es-hangul, Node.js 바인딩, WebAssembly 바인딩 |
| `bun run bench:report` | 위 두 하네스를 한 번에 실행해 하나의 표로 출력 |

`bun run bench:report`는 `wasm/pkg`와 Node.js 애드온이 없으면 먼저 빌드하므로, 새로 클론한 저장소에는 Rust, bun, [`wasm-pack`](https://rustwasm.github.io/wasm-pack/)만 있으면 됩니다.

벤치마크 이름은 `작업 · 입력` 형태입니다. Apple silicon 노트북(macOS arm64)에서 측정한 초당 평균 처리량이며, 정밀한 점수가 아니라 대략적인 규모로 읽어 주세요.

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
<summary>벤치마크 → API</summary>

| 벤치마크 | API |
| --- | --- |
| 글자 분해 + 초성 추출 | `disassemble` + `getChoseong` |
| 초성·중성·종성 분해 | `disassembleToGroups` |
| 받침 확인 | `hasBatchim` |
| 조사 선택 (을/를) | `josa("을/를")` |
| 자모 조합 | `assemble` |
| 초성 포함 확인 | `containsChoseong("ㄷㅎ")` |
| 초성 위치 찾기 | `findChoseong("ㄴㄹ")` |

</details>

- es-hangul은 FFI 경계를 넘지 않으므로 아주 짧은 입력에서 앞서고, 입력이 길어질수록 순위가 뒤집힙니다.
- WebAssembly 레인은 선형 메모리로 문자열을 복사하므로 같은 Rust 코어를 감싸고도 긴 입력에서 Node.js 레인보다 느립니다.
- es-hangul에는 초성 검색이 없어 `초성 포함 확인`·`초성 위치 찾기`는 Rust 레인끼리만 비교합니다.
- 각 행은 바인딩이 입력을 감싸는 객체를 만드는 비용까지 포함해 호출 전체를 측정합니다.

`benchmark` GitHub Actions 워크플로는 수동 실행으로만 리포트를 만들고 `benchmark-report.json`을 아티팩트로 업로드합니다.

## AI 코딩 에이전트

API 요약은 [`llms.txt`](./llms.txt)에 있습니다.

## 라이선스

[MIT](./LICENSE)
