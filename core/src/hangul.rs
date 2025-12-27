use std::sync::OnceLock;

use crate::hangul_letter::HangulLetter;

struct CharUnit {
  original: char,
  hangul: Option<HangulLetter>,
}

pub struct Hangul {
  char_units: Vec<CharUnit>,
  original: String,
  disassembled_cache: OnceLock<String>,
  choseong_cache: OnceLock<String>,
}

// TODO: NFD 지원
impl Hangul {
  pub fn new(string: &str) -> Self {
    let mut char_units = Vec::with_capacity(string.chars().count());

    for ch in string.chars() {
      char_units.push(CharUnit {
        original: ch,
        hangul: HangulLetter::parse_from_char(ch),
      });
    }

    Self {
      char_units,
      original: string.to_string(),
      disassembled_cache: OnceLock::new(),
      choseong_cache: OnceLock::new(),
    }
  }

  pub fn original(&self) -> &str {
    &self.original
  }

  pub fn len(&self) -> usize {
    self.char_units.len()
  }

  pub fn is_empty(&self) -> bool {
    self.char_units.is_empty()
  }

  pub fn disassemble(&self) -> String {
    self
      .disassembled_cache
      .get_or_init(|| self.disassemble_uncached())
      .clone()
  }

  pub fn get_choseong(&self) -> String {
    self
      .choseong_cache
      .get_or_init(|| self.choseong_uncached())
      .clone()
  }

  fn disassemble_uncached(&self) -> String {
    if self.is_empty() {
      return String::new();
    }

    let mut result = String::with_capacity(self.char_units.len() * 3);

    for unit in &self.char_units {
      match &unit.hangul {
        Some(hangul) => hangul.append_disassembled(&mut result),
        None => result.push(unit.original),
      }
    }

    result
  }

  fn choseong_uncached(&self) -> String {
    if self.is_empty() {
      return String::new();
    }

    let mut result = String::with_capacity(self.char_units.len());

    for unit in &self.char_units {
      match &unit.hangul {
        Some(hangul) => result.push(hangul.choseong.compatibility_value),
        None => result.push(unit.original),
      }
    }

    result
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_create_hangul() {
    let sentence = Hangul::new("안녕하세요");
    assert_eq!(sentence.len(), 5);
    assert_eq!(sentence.original(), "안녕하세요");

    let mixed = Hangul::new("Hello 안녕!");
    assert_eq!(mixed.len(), 9);
    assert_eq!(mixed.original(), "Hello 안녕!");

    let empty = Hangul::new("");
    assert!(empty.is_empty());
    assert_eq!(empty.len(), 0);
    assert_eq!(empty.original(), "");
  }

  #[test]
  fn test_original() {
    let sentence = Hangul::new("안녕하세요");
    assert_eq!(sentence.original(), "안녕하세요");

    let special = Hangul::new("특수문자!@#");
    assert_eq!(special.original(), "특수문자!@#");
  }

  #[test]
  fn test_len_and_is_empty() {
    let sentence = Hangul::new("안녕하세요");
    assert_eq!(sentence.len(), 5);
    assert!(!sentence.is_empty());

    let empty = Hangul::new("");
    assert_eq!(empty.len(), 0);
    assert!(empty.is_empty());

    let mixed = Hangul::new("A한글1");
    assert_eq!(mixed.len(), 4);
    assert!(!mixed.is_empty());
  }

  #[test]
  fn test_disassemble() {
    let sentence = Hangul::new("안녕");
    assert_eq!(sentence.disassemble(), "ㅇㅏㄴㄴㅕㅇ");

    let mixed = Hangul::new("안녕 Hello");
    assert_eq!(mixed.disassemble(), "ㅇㅏㄴㄴㅕㅇ Hello");

    let special = Hangul::new("안녕!");
    assert_eq!(special.disassemble(), "ㅇㅏㄴㄴㅕㅇ!");
  }

  #[test]
  fn test_get_choseong() {
    let sentence = Hangul::new("안녕하세요");
    assert_eq!(sentence.get_choseong(), "ㅇㄴㅎㅅㅇ");

    let mixed = Hangul::new("Hello 안녕!");
    assert_eq!(mixed.get_choseong(), "Hello ㅇㄴ!");

    let empty = Hangul::new("");
    assert_eq!(empty.get_choseong(), "");
  }

  #[test]
  fn test_empty_repeated_calls() {
    let empty = Hangul::new("");
    assert_eq!(empty.disassemble(), "");
    assert_eq!(empty.get_choseong(), "");
    assert_eq!(empty.disassemble(), "");
    assert_eq!(empty.get_choseong(), "");
  }

  #[test]
  fn test_non_hangul_only() {
    let text = "ABC123!@";
    let sentence = Hangul::new(text);
    assert_eq!(sentence.disassemble(), text);
    assert_eq!(sentence.get_choseong(), text);
  }

  #[test]
  fn test_mixed_boundaries() {
    let middle = Hangul::new("가A나!");
    assert_eq!(middle.disassemble(), "ㄱㅏAㄴㅏ!");
    assert_eq!(middle.get_choseong(), "ㄱAㄴ!");

    let prefix = Hangul::new("A가");
    assert_eq!(prefix.disassemble(), "Aㄱㅏ");
    assert_eq!(prefix.get_choseong(), "Aㄱ");

    let suffix = Hangul::new("가A");
    assert_eq!(suffix.disassemble(), "ㄱㅏA");
    assert_eq!(suffix.get_choseong(), "ㄱA");
  }

  #[test]
  fn test_whitespace_preserved() {
    let sentence = Hangul::new("안녕\n하세요\t");
    assert_eq!(sentence.disassemble(), "ㅇㅏㄴㄴㅕㅇ\nㅎㅏㅅㅔㅇㅛ\t");
    assert_eq!(sentence.get_choseong(), "ㅇㄴ\nㅎㅅㅇ\t");
  }

  #[test]
  fn test_nfd_input_passthrough() {
    let nfd = "\u{1100}\u{1161}\u{11AB}";
    let sentence = Hangul::new(nfd);
    assert_eq!(sentence.disassemble(), nfd);
    assert_eq!(sentence.get_choseong(), nfd);
  }

  #[test]
  fn test_single_char_inputs() {
    let hangul = Hangul::new("가");
    assert_eq!(hangul.disassemble(), "ㄱㅏ");
    assert_eq!(hangul.get_choseong(), "ㄱ");

    let jamo = Hangul::new("ㄱ");
    assert_eq!(jamo.disassemble(), "ㄱ");
    assert_eq!(jamo.get_choseong(), "ㄱ");

    let vowel = Hangul::new("ㅏ");
    assert_eq!(vowel.disassemble(), "ㅏ");
    assert_eq!(vowel.get_choseong(), "ㅏ");
  }

  #[test]
  fn test_emoji_mixed() {
    let sentence = Hangul::new("가🙂나");
    assert_eq!(sentence.disassemble(), "ㄱㅏ🙂ㄴㅏ");
    assert_eq!(sentence.get_choseong(), "ㄱ🙂ㄴ");
  }

  #[test]
  fn test_long_string_smoke() {
    let text = "가나다라마바사아자차카타파하";
    let long = text.repeat(1000);
    let sentence = Hangul::new(&long);
    assert_eq!(sentence.len(), long.chars().count());
    assert_eq!(sentence.disassemble().chars().count(), long.chars().count() * 2);
  }

  #[test]
  fn test_cache_reuse_same_instance() {
    let sentence = Hangul::new("안녕 Hello");
    let first = sentence.disassemble();
    let second = sentence.disassemble();
    assert_eq!(first, second);

    let first = sentence.get_choseong();
    let second = sentence.get_choseong();
    assert_eq!(first, second);
  }
}
