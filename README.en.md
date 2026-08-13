# rusty_hangul

🚧 This library is currently under development. The API may change.

[한국어](./README.md)

## Introduction

rusty_hangul is a Rust library designed for processing Korean (Hangul) strings. It consists of two main components:

1. **core**: A Rust crate that handles core functionality for Hangul processing
2. **node**: A Node.js library that binds core functionality using [napi-rs](https://napi.rs/)

## Rust(Core) Usage Examples

```rust
// Create a Hangul string
let text = Hangul::new("안녕하세요");

// Disassemble Hangul
assert_eq!(text.disassemble(), "ㅇㅏㄴㄴㅕㅇㅎㅏㅅㅔㅇㅛ");

// Extract initial consonants
assert_eq!(text.get_choseong(), "ㅇㄴㅎㅅㅇ");

// Can handle non-Hangul characters too
let mixed = Hangul::new("Hello 안녕!");
assert_eq!(mixed.get_choseong(), "Hello ㅇㄴ!");

// Both NFC and NFD are supported
let nfd = Hangul::new("\u{1100}\u{1161}\u{11AB}"); // 간
assert_eq!(nfd.disassemble(), "ㄱㅏㄴ");

// Detect batchim and select a josa
assert!(!Hangul::new("사과").has_batchim());
assert_eq!(Hangul::new("사과").josa("을/를").unwrap(), "사과를");
assert_eq!(Hangul::new("수박").josa("을/를").unwrap(), "수박을");

// Assemble Jamo
assert_eq!(hangul::assemble("ㄱㅏㅂㅅ"), "값");
```

## Node.js Usage Examples

```typescript
// Create a Hangul string
const text = new Hangul("안녕하세요");

// Disassemble Hangul
console.log(text.disassemble()); // "ㅇㅏㄴㄴㅕㅇㅎㅏㅅㅔㅇㅛ"

// Extract initial consonants
console.log(text.getChoseong()); // "ㅇㄴㅎㅅㅇ"

// Can handle non-Hangul characters too
const mixed = new Hangul("Hello 안녕!");
console.log(mixed.getChoseong()); // "Hello ㅇㄴ!"

// Detect batchim and select a josa
console.log(new Hangul("한").hasBatchim()); // true
console.log(new Hangul("사과").josa("을/를")); // "사과를"

// Assemble Jamo
console.log(assemble("ㄱㅏㅂㅅ")); // "값"
```