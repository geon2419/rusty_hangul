//! Hangul string processing: syllable disassembly, choseong search, batchim/josa, and jamo assembly.
//!
//! On crates.io the package is [`rusty_hangul`](https://crates.io/crates/rusty_hangul).
//! In Rust the crate is still imported as `hangul`.
//!
//! ```
//! use hangul::{assemble, Hangul};
//!
//! let text = Hangul::new("안녕하세요");
//! assert_eq!(text.disassemble(), "ㅇㅏㄴㄴㅕㅇㅎㅏㅅㅔㅇㅛ");
//! assert_eq!(text.get_choseong(), "ㅇㄴㅎㅅㅇ");
//! assert_eq!(text.josa("을/를").unwrap(), "안녕하세요를");
//! assert_eq!(assemble("ㄱㅏㅂㅅ"), "값");
//! ```

mod assemble;
mod choseong;
mod choseong_search;
mod hangul;
mod hangul_letter;
mod jongseong;
mod josa;
mod jungseong;
mod nfc;
mod nfd;

pub use crate::assemble::{assemble, assemble_with_policy, AssemblePolicy};
pub use crate::choseong_search::ChoseongMatch;
pub use crate::hangul::{Hangul, HangulUnit};
pub use crate::hangul_letter::HangulLetter;
pub use crate::josa::JosaError;
