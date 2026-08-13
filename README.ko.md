# rusty_hangul

Rust로 만든 한글 문자열 처리 라이브러리입니다. 음절 분해, 초성 추출, 받침·조사 선택, 자모 조립을 제공하고 Node.js·WebAssembly 바인딩을 함께 둡니다.

[English](./README.md)

## 기능

- 한글 음절을 자모로 분해 (NFC / NFD, 그룹 출력 포함)
- 음절 단위 순회 및 초·중·종성 확인
- 초성 추출 (한글이 아닌 문자는 그대로 유지)
- 받침 확인 및 조사 선택 (조사만 반환하는 API 포함)
- 자모를 음절로 조립

## 구성

이 저장소는 Cargo와 bun 워크스페이스입니다. **crates.io와 npm에는 아직 배포되어 있지 않습니다.**

| 경로 | 패키지 이름 | 역할 |
| --- | --- | --- |
| `core/` | `hangul` | Rust 크레이트 |
| `node/` | `node` | [napi-rs](https://napi.rs/) Node.js 네이티브 애드온 |
| `wasm/` | `wasm` (`hangul-wasm`) | wasm-bindgen WebAssembly 바인딩 |

브라우저 데모는 [`examples/browser-basic`](./examples/browser-basic)에 있습니다.

## 설치

[Rust 툴체인](https://rustup.rs/)과 [bun](https://bun.sh/) (`>= 1.3.3`)이 필요합니다. Node.js 바인딩은 Node `>= 20`이 필요하고, WebAssembly 빌드에는 [`wasm-pack`](https://rustwasm.github.io/wasm-pack/)이 필요합니다.

저장소를 클론한 뒤:

```sh
bun install
```

### Rust

크레이트 이름은 `rusty_hangul`이 아니라 `hangul`입니다.

git에서:

```toml
[dependencies]
hangul = { git = "https://github.com/geon2419/rusty_hangul" }
```

로컬 클론에서:

```toml
[dependencies]
hangul = { path = "path/to/rusty_hangul/core" }
```

### Node.js

네이티브 애드온을 빌드한 뒤, 워크스페이스 패키지 이름 `node`로 import합니다.

```sh
bun run node:build
```

```ts
import { Hangul, assemble } from "node";
```

이 import는 이 bun 워크스페이스 안의 다른 패키지에서 동작합니다. 워크스페이스 밖에서는 로컬 빌드 후 `./node`를 가리키면 됩니다.

### WebAssembly

```sh
bun run wasm:build
```

```ts
import init, { disassemble, getChoseong } from "./wasm/pkg/hangul";

await init();
```

앱 구조에 맞게 import 경로를 바꾸면 됩니다. Vite 설정은 [`examples/browser-basic`](./examples/browser-basic)을 참고하세요.

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

## Node.js 예시

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

## 라이선스

[MIT](./LICENSE)
