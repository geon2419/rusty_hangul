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

Apple silicon 노트북(macOS arm64)에서 측정한 초당 평균 처리량입니다. 정밀한 점수가 아니라 대략적인 규모로 읽어 주세요.

| 벤치마크 | core | es-hangul | node | wasm |
| --- | ---: | ---: | ---: | ---: |
| disassemble + getChoseong · 한글 이름 분석 | 3.42M | 1.39M | 999.0K | 1.03M |
| disassemble + getChoseong · 한글 문장 분석 | 798.6K | 149.7K | 282.5K | 260.5K |
| disassemble + getChoseong · 한글 문단 분석 | 234.2K | 33.4K | 87.9K | 75.6K |
| disassemble + getChoseong · 한글 긴 문단 분석 | 101.4K | 14.2K | 31.9K | 31.6K |
| disassemble + getChoseong · 한영 혼합 텍스트 | 1.95M | 291.3K | 589.3K | 561.0K |
| disassemble + getChoseong · 한글 NFD 텍스트 | 1.59M | 692.0K | 681.2K | 619.1K |
| disassembleToGroups · 한글 이름 분석 | 5.23M | 6.04M | 624.3K | 627.8K |
| disassembleToGroups · 한글 문장 분석 | 621.9K | 584.0K | 76.9K | 61.8K |
| disassembleToGroups · 한글 문단 분석 | 151.2K | 138.6K | 18.7K | 16.3K |
| hasBatchim · 한글 이름 분석 | 22.37M | 24.06M | 1.46M | 5.75M |
| hasBatchim · 한글 문장 분석 | 6.05M | 24.62M | 1.09M | 1.84M |
| josa · 한글 이름 분석 | 15.59M | 12.43M | 1.59M | 2.16M |
| assemble · 자모 조합 텍스트 | 8.28M | 259.7K | 4.61M | 3.53M |
| containsChoseong · 한글 문장 분석 | 3.77M | - | 1.16M | 1.56M |
| containsChoseong · 한글 문단 분석 | 1.38M | - | 530.6K | 604.9K |
| findChoseong · 한글 문장 분석 | 4.54M | - | 790.6K | 1.32M |
| findChoseong · 한글 문단 분석 | 1.51M | - | 439.5K | 544.3K |

- es-hangul은 FFI 경계를 넘지 않으므로 아주 짧은 입력에서 앞서고, 입력이 길어질수록 순위가 뒤집힙니다.
- WebAssembly 레인은 선형 메모리로 문자열을 복사하므로 같은 Rust 코어를 감싸고도 긴 입력에서 Node.js 레인보다 느립니다.
- es-hangul에는 초성 검색이 없어 `containsChoseong`, `findChoseong`은 Rust 레인 두 개만 비교합니다.
- 각 행은 바인딩에서 `Hangul`을 생성하는 비용까지 포함한 시나리오 전체를 측정합니다.

`benchmark` GitHub Actions 워크플로는 `main` 브랜치에 푸시될 때 리포트를 실행하고 `benchmark-report.json`을 아티팩트로 업로드합니다.

## AI 코딩 에이전트

API 요약은 [`llms.txt`](./llms.txt)에 있습니다.

## 라이선스

[MIT](./LICENSE)
