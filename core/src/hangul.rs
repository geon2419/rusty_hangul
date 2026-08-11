use std::sync::OnceLock;

use crate::choseong::Choseong;
use crate::hangul_letter::HangulLetter;
use crate::jongseong::Jongseong;
use crate::jungseong::Jungseong;

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

impl Hangul {
  pub fn new(string: &str) -> Self {
    Self {
      char_units: Self::parse_char_units(string),
      original: string.to_string(),
      disassembled_cache: OnceLock::new(),
      choseong_cache: OnceLock::new(),
    }
  }

  fn parse_char_units(string: &str) -> Vec<CharUnit> {
    let mut char_units = Vec::with_capacity(string.chars().count());
    let mut chars = string.chars().peekable();

    while let Some(ch) = chars.next() {
      if let Some(letter) = HangulLetter::parse_from_char(ch) {
        char_units.push(CharUnit {
          original: ch,
          hangul: Some(letter),
        });
        continue;
      }

      if Choseong::is_conjoining_choseong(ch as u32) {
        if let Some(&jung) = chars.peek() {
          if Jungseong::is_conjoining_jungseong(jung as u32) {
            chars.next();

            let mut syllable = String::with_capacity(3);
            syllable.push(ch);
            syllable.push(jung);

            if let Some(&jong) = chars.peek() {
              if Jongseong::is_conjoining_jongseong(jong as u32) {
                syllable.push(chars.next().unwrap());
              }
            }

            let letter = HangulLetter::parse(&syllable)
              .expect("choseong+jungseong(+jongseong) must form a valid NFD syllable");

            char_units.push(CharUnit {
              original: ch,
              hangul: Some(letter),
            });
            continue;
          }
        }
      }

      char_units.push(CharUnit {
        original: ch,
        hangul: None,
      });
    }

    char_units
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

  const S_BASE: u32 = 0xAC00;
  const S_LAST: u32 = 0xD7A3;
  const N_COUNT: u32 = 21 * 28;
  const T_COUNT: u32 = 28;

  const CHOSEONG: [char; 19] = [
    'ㄱ', 'ㄲ', 'ㄴ', 'ㄷ', 'ㄸ', 'ㄹ', 'ㅁ', 'ㅂ', 'ㅃ', 'ㅅ', 'ㅆ', 'ㅇ', 'ㅈ', 'ㅉ', 'ㅊ', 'ㅋ',
    'ㅌ', 'ㅍ', 'ㅎ',
  ];

  const JUNGSEONG: [char; 21] = [
    'ㅏ', 'ㅐ', 'ㅑ', 'ㅒ', 'ㅓ', 'ㅔ', 'ㅕ', 'ㅖ', 'ㅗ', 'ㅘ', 'ㅙ', 'ㅚ', 'ㅛ', 'ㅜ', 'ㅝ', 'ㅞ',
    'ㅟ', 'ㅠ', 'ㅡ', 'ㅢ', 'ㅣ',
  ];

  const JONGSEONG: [&str; 28] = [
    "", "ㄱ", "ㄲ", "ㄱㅅ", "ㄴ", "ㄴㅈ", "ㄴㅎ", "ㄷ", "ㄹ", "ㄹㄱ", "ㄹㅁ", "ㄹㅂ", "ㄹㅅ",
    "ㄹㅌ", "ㄹㅍ", "ㄹㅎ", "ㅁ", "ㅂ", "ㅂㅅ", "ㅅ", "ㅆ", "ㅇ", "ㅈ", "ㅊ", "ㅋ", "ㅌ", "ㅍ",
    "ㅎ",
  ];

  fn modern_hangul_indices(code: u32) -> (usize, usize, usize) {
    let index = code - S_BASE;

    (
      (index / N_COUNT) as usize,
      ((index % N_COUNT) / T_COUNT) as usize,
      (index % T_COUNT) as usize,
    )
  }

  fn modern_hangul_from_indices(
    choseong_index: usize,
    jungseong_index: usize,
    jongseong_index: usize,
  ) -> char {
    let code = S_BASE
      + choseong_index as u32 * N_COUNT
      + jungseong_index as u32 * T_COUNT
      + jongseong_index as u32;

    char::from_u32(code).unwrap()
  }

  fn expected_disassembly(
    choseong_index: usize,
    jungseong_index: usize,
    jongseong_index: usize,
  ) -> String {
    let mut expected = String::new();
    expected.push(CHOSEONG[choseong_index]);
    expected.push(JUNGSEONG[jungseong_index]);
    expected.push_str(JONGSEONG[jongseong_index]);
    expected
  }

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
  fn test_disassemble_all_modern_hangul_syllables() {
    for code in S_BASE..=S_LAST {
      let ch = char::from_u32(code).unwrap();
      let (choseong_index, jungseong_index, jongseong_index) = modern_hangul_indices(code);

      assert_eq!(
        Hangul::new(&ch.to_string()).disassemble(),
        expected_disassembly(choseong_index, jungseong_index, jongseong_index),
        "failed to disassemble U+{code:04X} ({ch})"
      );
    }
  }

  #[test]
  fn test_disassemble_edge_combinations_of_jamo_indices() {
    let choseong_edges = [0usize, 1, 17, 18];
    let jungseong_edges = [0usize, 1, 19, 20];
    let jongseong_edges = [0usize, 1, 26, 27];

    for choseong_index in choseong_edges {
      for jungseong_index in jungseong_edges {
        for jongseong_index in jongseong_edges {
          let ch = modern_hangul_from_indices(choseong_index, jungseong_index, jongseong_index);
          let code = ch as u32;

          assert_eq!(
            Hangul::new(&ch.to_string()).disassemble(),
            expected_disassembly(choseong_index, jungseong_index, jongseong_index),
            "failed at choseong_index={choseong_index}, jungseong_index={jungseong_index}, jongseong_index={jongseong_index}, U+{code:04X} ({ch})"
          );
        }
      }
    }
  }

  #[test]
  fn test_disassemble_all_compound_jungseong() {
    let cases = [
      (9usize, "ㄱㅘ"),
      (10, "ㄱㅙ"),
      (11, "ㄱㅚ"),
      (14, "ㄱㅝ"),
      (15, "ㄱㅞ"),
      (16, "ㄱㅟ"),
      (19, "ㄱㅢ"),
    ];

    for (jungseong_index, expected) in cases {
      let ch = modern_hangul_from_indices(0, jungseong_index, 0);

      assert_eq!(
        Hangul::new(&ch.to_string()).disassemble(),
        expected,
        "failed to disassemble compound jungseong index={jungseong_index}, char={ch}"
      );
    }
  }

  #[test]
  fn test_disassemble_all_compound_jongseong() {
    let cases = [
      (3usize, "ㄱㅏㄱㅅ"),
      (5, "ㄱㅏㄴㅈ"),
      (6, "ㄱㅏㄴㅎ"),
      (9, "ㄱㅏㄹㄱ"),
      (10, "ㄱㅏㄹㅁ"),
      (11, "ㄱㅏㄹㅂ"),
      (12, "ㄱㅏㄹㅅ"),
      (13, "ㄱㅏㄹㅌ"),
      (14, "ㄱㅏㄹㅍ"),
      (15, "ㄱㅏㄹㅎ"),
      (18, "ㄱㅏㅂㅅ"),
    ];

    for (jongseong_index, expected) in cases {
      let ch = modern_hangul_from_indices(0, 0, jongseong_index);

      assert_eq!(
        Hangul::new(&ch.to_string()).disassemble(),
        expected,
        "failed to disassemble compound jongseong index={jongseong_index}, char={ch}"
      );
    }
  }

  #[test]
  fn test_disassemble_compound_jungseong_with_compound_jongseong() {
    let ch = modern_hangul_from_indices(0, 9, 18);

    assert_eq!(Hangul::new(&ch.to_string()).disassemble(), "ㄱㅘㅂㅅ");
  }

  #[test]
  fn test_disassemble_modern_hangul_boundaries() {
    assert_eq!(Hangul::new("가").disassemble(), "ㄱㅏ");
    assert_eq!(Hangul::new("힣").disassemble(), "ㅎㅣㅎ");

    assert_eq!(Hangul::new("\u{ABFF}").disassemble(), "\u{ABFF}");
    assert_eq!(Hangul::new("\u{D7A4}").disassemble(), "\u{D7A4}");
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
  fn test_nfd_input() {
    let nfd_gan = "\u{1100}\u{1161}\u{11AB}";
    let sentence = Hangul::new(nfd_gan);
    assert_eq!(sentence.len(), 1);
    assert_eq!(sentence.original(), nfd_gan);
    assert_eq!(sentence.disassemble(), "ㄱㅏㄴ");
    assert_eq!(sentence.get_choseong(), "ㄱ");

    let nfd_ga = "\u{1100}\u{1161}";
    let sentence = Hangul::new(nfd_ga);
    assert_eq!(sentence.len(), 1);
    assert_eq!(sentence.disassemble(), "ㄱㅏ");
    assert_eq!(sentence.get_choseong(), "ㄱ");
  }

  #[test]
  fn test_nfd_mixed_with_nfc_and_non_hangul() {
    let mixed = Hangul::new("Hello \u{1112}\u{1161}\u{11AB}!");
    assert_eq!(mixed.len(), 8);
    assert_eq!(mixed.disassemble(), "Hello ㅎㅏㄴ!");
    assert_eq!(mixed.get_choseong(), "Hello ㅎ!");

    let mixed_nfc_nfd = Hangul::new("가\u{1100}\u{1161}");
    assert_eq!(mixed_nfc_nfd.len(), 2);
    assert_eq!(mixed_nfc_nfd.disassemble(), "ㄱㅏㄱㅏ");
    assert_eq!(mixed_nfc_nfd.get_choseong(), "ㄱㄱ");
  }

  #[test]
  fn test_lone_conjoining_jamo_passthrough() {
    let choseong_only = Hangul::new("\u{1100}");
    assert_eq!(choseong_only.len(), 1);
    assert_eq!(choseong_only.disassemble(), "\u{1100}");
    assert_eq!(choseong_only.get_choseong(), "\u{1100}");

    let jungseong_only = Hangul::new("\u{1161}");
    assert_eq!(jungseong_only.disassemble(), "\u{1161}");
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
    assert_eq!(
      sentence.disassemble().chars().count(),
      long.chars().count() * 2
    );
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
