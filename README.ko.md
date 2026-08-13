# rusty_hangul

🚧 현재 개발 중인 라이브러리입니다. API가 변경될 수 있습니다.

[English](./README.md)

## 소개 (Introduction)

rusty_hangul은 한글 문자열 처리를 위해 Rust로 작성된 라이브러리입니다. 두 가지 주요 부분으로 구성됩니다:

1. **core**: 한글 처리를 위한 핵심 기능을 담당하는 Rust 크레이트
2. **node**: [napi-rs](https://napi.rs/)를 이용해 코어 기능을 바인딩한 Node.js 라이브러리

## Rust(Core) 사용 예시

```rust
// 한글 문자열 생성
let text = Hangul::new("안녕하세요");

// 한글 분해
assert_eq!(text.disassemble(), "ㅇㅏㄴㄴㅕㅇㅎㅏㅅㅔㅇㅛ");

// 초성 추출
assert_eq!(text.get_choseong(), "ㅇㄴㅎㅅㅇ");

// 한글이 아닌 문자도 처리 가능
let mixed = Hangul::new("Hello 안녕!");
assert_eq!(mixed.get_choseong(), "Hello ㅇㄴ!");

// NFC / NFD 모두 처리 가능
let nfd = Hangul::new("\u{1100}\u{1161}\u{11AB}"); // 간
assert_eq!(nfd.disassemble(), "ㄱㅏㄴ");

// 받침 확인 및 조사 선택
assert!(!Hangul::new("사과").has_batchim());
assert_eq!(Hangul::new("사과").josa("을/를").unwrap(), "사과를");
assert_eq!(Hangul::new("수박").josa("을/를").unwrap(), "수박을");

// 자모 조립
assert_eq!(hangul::assemble("ㄱㅏㅂㅅ"), "값");
assert_eq!(
  hangul::assemble_with_policy(
    "ㄱㅏㄱㅅㅏ",
    hangul::AssemblePolicy::PreferCompoundFinal
  ),
  "갃ㅏ"
);
```

## Node.js 사용 예시

```typescript
// 한글 문자열 생성
const text = new Hangul("안녕하세요");

// 한글 분해
console.log(text.disassemble()); // "ㅇㅏㄴㄴㅕㅇㅎㅏㅅㅔㅇㅛ"

// 초성 추출
console.log(text.getChoseong()); // "ㅇㄴㅎㅅㅇ"

// 한글이 아닌 문자도 처리 가능
const mixed = new Hangul("Hello 안녕!");
console.log(mixed.getChoseong()); // "Hello ㅇㄴ!"

// 받침 확인 및 조사 선택
console.log(new Hangul("한").hasBatchim()); // true
console.log(new Hangul("사과").josa("을/를")); // "사과를"

// 자모 조립
console.log(assemble("ㄱㅏㅂㅅ")); // "값"
console.log(assemble("ㄱㅏㄱㅅㅏ", "compound-final")); // "갃ㅏ"
```