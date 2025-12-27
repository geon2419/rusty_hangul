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
}
