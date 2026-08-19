use std::fmt;
use std::sync::OnceLock;

use crate::choseong::Choseong;
use crate::hangul_letter::HangulLetter;
use crate::jongseong::Jongseong;
use crate::josa::{JosaError, JosaPair};
use crate::jungseong::Jungseong;

#[derive(Clone)]
struct CharUnit {
  hangul: Option<HangulLetter>,
  start_byte: usize,
  end_byte: usize,
}

/// One parsed slot in a [`Hangul`] value: a Hangul syllable or a single other character.
///
/// NFD jamo that form one syllable (`ᄀ` + `ᅡ` + `ᆫ`) occupy a single unit.
/// [`HangulUnit::original`] is the source slice for that whole slot.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct HangulUnit<'a> {
  original: &'a str,
  letter: Option<&'a HangulLetter>,
  start_byte: usize,
  end_byte: usize,
}

impl<'a> HangulUnit<'a> {
  /// Source text occupied by this unit.
  ///
  /// For an NFD syllable this is every jamo in the cluster, not just the first.
  pub fn original(&self) -> &'a str {
    self.original
  }

  pub fn letter(&self) -> Option<&'a HangulLetter> {
    self.letter
  }

  pub fn is_hangul(&self) -> bool {
    self.letter.is_some()
  }

  pub fn disassembled_chars(&self) -> Vec<char> {
    match self.letter {
      Some(letter) => letter.disassembled_chars(),
      None => self.original.chars().collect(),
    }
  }

  pub fn byte_start(&self) -> usize {
    self.start_byte
  }

  pub fn byte_end(&self) -> usize {
    self.end_byte
  }
}

impl CharUnit {
  fn source<'a>(&self, text: &'a str) -> &'a str {
    &text[self.start_byte..self.end_byte]
  }

  fn as_unit<'a>(&'a self, text: &'a str) -> HangulUnit<'a> {
    HangulUnit {
      original: self.source(text),
      letter: self.hangul.as_ref(),
      start_byte: self.start_byte,
      end_byte: self.end_byte,
    }
  }
}

pub struct Hangul {
  char_units: Vec<CharUnit>,
  original: String,
  disassembled_cache: OnceLock<String>,
  choseong_cache: OnceLock<String>,
  groups_cache: OnceLock<Vec<Vec<char>>>,
}

impl Hangul {
  pub fn new(string: &str) -> Self {
    Self {
      char_units: Self::parse_char_units(string),
      original: string.to_string(),
      disassembled_cache: OnceLock::new(),
      choseong_cache: OnceLock::new(),
      groups_cache: OnceLock::new(),
    }
  }

  fn parse_char_units(string: &str) -> Vec<CharUnit> {
    let mut char_units = Vec::with_capacity(string.chars().count());
    let mut chars = string.char_indices().peekable();

    while let Some((start_byte, ch)) = chars.next() {
      if let Some(letter) = HangulLetter::parse_from_char(ch) {
        char_units.push(CharUnit {
          hangul: Some(letter),
          start_byte,
          end_byte: start_byte + ch.len_utf8(),
        });
        continue;
      }

      if Choseong::is_conjoining_choseong(ch as u32) {
        if let Some(&(_, jung)) = chars.peek() {
          if Jungseong::is_conjoining_jungseong(jung as u32) {
            let (jung_start_byte, jung) = chars.next().unwrap();

            let mut syllable = String::with_capacity(3);
            syllable.push(ch);
            syllable.push(jung);
            let mut end_byte = jung_start_byte + jung.len_utf8();

            if let Some(&(jong_start_byte, jong)) = chars.peek() {
              if Jongseong::is_conjoining_jongseong(jong as u32) {
                chars.next();
                syllable.push(jong);
                end_byte = jong_start_byte + jong.len_utf8();
              }
            }

            let letter = HangulLetter::parse(&syllable)
              .expect("choseong+jungseong(+jongseong) must form a valid NFD syllable");

            char_units.push(CharUnit {
              hangul: Some(letter),
              start_byte,
              end_byte,
            });
            continue;
          }
        }
      }

      char_units.push(CharUnit {
        hangul: None,
        start_byte,
        end_byte: start_byte + ch.len_utf8(),
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

  pub fn get(&self, index: usize) -> Option<HangulUnit<'_>> {
    self
      .char_units
      .get(index)
      .map(|unit| unit.as_unit(&self.original))
  }

  pub fn units(&self) -> impl ExactSizeIterator<Item = HangulUnit<'_>> + '_ {
    self
      .char_units
      .iter()
      .map(|unit| unit.as_unit(&self.original))
  }

  pub fn letters(&self) -> impl Iterator<Item = &HangulLetter> + '_ {
    self
      .char_units
      .iter()
      .filter_map(|unit| unit.hangul.as_ref())
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

  pub fn disassemble_to_groups(&self) -> Vec<Vec<char>> {
    self
      .groups_cache
      .get_or_init(|| self.disassemble_to_groups_uncached())
      .clone()
  }

  pub fn has_batchim(&self) -> bool {
    self
      .last_hangul_letter()
      .map(HangulLetter::has_batchim)
      .unwrap_or(false)
  }

  pub fn josa_particle(&self, pair: &str) -> Result<&'static str, JosaError> {
    let pair = JosaPair::parse(pair).ok_or_else(|| JosaError::InvalidPair(pair.to_owned()))?;
    let last_letter = self.last_hangul_letter();
    let has_batchim = last_letter.map(HangulLetter::has_batchim).unwrap_or(false);
    let has_rieul_batchim = last_letter
      .and_then(|letter| letter.jongseong.as_ref())
      .map(|jongseong| jongseong.compatibility_value == 'ㄹ')
      .unwrap_or(false);
    Ok(pair.select(has_batchim, has_rieul_batchim))
  }

  pub fn contains_choseong(&self, query: &str) -> bool {
    self.find_choseong(query).is_some()
  }

  pub fn find_choseong(&self, query: &str) -> Option<crate::ChoseongMatch> {
    crate::choseong_search::find_choseong(self, query)
  }

  pub fn josa(&self, pair: &str) -> Result<String, JosaError> {
    let particle = self.josa_particle(pair)?;
    let insertion = self
      .last_hangul_unit()
      .map(|unit| unit.end_byte)
      .unwrap_or(self.original.len());

    let mut result = String::with_capacity(self.original.len() + particle.len());
    result.push_str(&self.original[..insertion]);
    result.push_str(particle);
    result.push_str(&self.original[insertion..]);
    Ok(result)
  }

  fn last_hangul_unit(&self) -> Option<&CharUnit> {
    self
      .char_units
      .iter()
      .rev()
      .find(|unit| unit.hangul.is_some())
  }

  fn last_hangul_letter(&self) -> Option<&HangulLetter> {
    self
      .last_hangul_unit()
      .and_then(|unit| unit.hangul.as_ref())
  }

  fn disassemble_uncached(&self) -> String {
    if self.is_empty() {
      return String::new();
    }

    let mut result = String::with_capacity(self.char_units.len() * 3);

    for unit in &self.char_units {
      match &unit.hangul {
        Some(hangul) => hangul.append_disassembled(&mut result),
        None => result.push_str(unit.source(&self.original)),
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
        None => result.push_str(unit.source(&self.original)),
      }
    }

    result
  }

  fn disassemble_to_groups_uncached(&self) -> Vec<Vec<char>> {
    self
      .char_units
      .iter()
      .map(|unit| unit.as_unit(&self.original).disassembled_chars())
      .collect()
  }
}

impl Clone for Hangul {
  fn clone(&self) -> Self {
    Self {
      char_units: self.char_units.clone(),
      original: self.original.clone(),
      disassembled_cache: OnceLock::new(),
      choseong_cache: OnceLock::new(),
      groups_cache: OnceLock::new(),
    }
  }
}

impl PartialEq for Hangul {
  fn eq(&self, other: &Self) -> bool {
    self.original == other.original
  }
}

impl Eq for Hangul {}

impl fmt::Display for Hangul {
  fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
    formatter.write_str(&self.original)
  }
}

impl fmt::Debug for Hangul {
  fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
    formatter
      .debug_struct("Hangul")
      .field("original", &self.original)
      .field("len", &self.len())
      .finish()
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
  fn test_has_batchim() {
    assert!(Hangul::new("한").has_batchim());
    assert!(Hangul::new("값!").has_batchim());
    assert!(Hangul::new("\u{1112}\u{1161}\u{11AB}").has_batchim());

    assert!(!Hangul::new("하").has_batchim());
    assert!(!Hangul::new("Hello").has_batchim());
    assert!(!Hangul::new("").has_batchim());
    assert!(!Hangul::new("한 가").has_batchim());
  }

  #[test]
  fn test_josa() {
    assert_eq!(Hangul::new("사과").josa("을/를").unwrap(), "사과를");
    assert_eq!(Hangul::new("사과").josa("이/가").unwrap(), "사과가");
    assert_eq!(Hangul::new("사과").josa("은/는").unwrap(), "사과는");
    assert_eq!(Hangul::new("사과").josa("와/과").unwrap(), "사과와");
    assert_eq!(Hangul::new("사과").josa("으로/로").unwrap(), "사과로");
    assert_eq!(Hangul::new("사과").josa("이에요/예요").unwrap(), "사과예요");

    assert_eq!(Hangul::new("수박").josa("을/를").unwrap(), "수박을");
    assert_eq!(Hangul::new("수박").josa("이/가").unwrap(), "수박이");
    assert_eq!(Hangul::new("수박").josa("은/는").unwrap(), "수박은");
    assert_eq!(Hangul::new("수박").josa("와/과").unwrap(), "수박과");
    assert_eq!(Hangul::new("수박").josa("으로/로").unwrap(), "수박으로");
    assert_eq!(
      Hangul::new("수박").josa("이에요/예요").unwrap(),
      "수박이에요"
    );
    assert_eq!(Hangul::new("사과").josa("아/야").unwrap(), "사과야");
    assert_eq!(Hangul::new("수박").josa("아/야").unwrap(), "수박아");
    assert_eq!(Hangul::new("사과").josa("이나/나").unwrap(), "사과나");
    assert_eq!(Hangul::new("수박").josa("이나/나").unwrap(), "수박이나");
    assert_eq!(Hangul::new("사과").josa("이란/란").unwrap(), "사과란");
    assert_eq!(Hangul::new("수박").josa("이란/란").unwrap(), "수박이란");
    assert_eq!(Hangul::new("사과").josa("이랑/랑").unwrap(), "사과랑");
    assert_eq!(Hangul::new("수박").josa("이랑/랑").unwrap(), "수박이랑");
    assert_eq!(Hangul::new("사과").josa("이며/며").unwrap(), "사과며");
    assert_eq!(Hangul::new("수박").josa("이며/며").unwrap(), "수박이며");
    assert_eq!(Hangul::new("사과").josa("이야/야").unwrap(), "사과야");
    assert_eq!(Hangul::new("수박").josa("이야/야").unwrap(), "수박이야");
    assert_eq!(Hangul::new("사과").josa("이라고/라고").unwrap(), "사과라고");
    assert_eq!(
      Hangul::new("수박").josa("이라고/라고").unwrap(),
      "수박이라고"
    );
    assert_eq!(Hangul::new("사과").josa("이든/든").unwrap(), "사과든");
    assert_eq!(Hangul::new("수박").josa("이든/든").unwrap(), "수박이든");
  }

  #[test]
  fn test_josa_aliases_match_canonical_pairs() {
    let aliases = [
      ("을/를", "를/을"),
      ("이/가", "가/이"),
      ("은/는", "는/은"),
      ("와/과", "과/와"),
      ("으로/로", "로/으로"),
      ("이에요/예요", "예요/이에요"),
      ("아/야", "야/아"),
      ("이나/나", "나/이나"),
      ("이란/란", "란/이란"),
      ("이랑/랑", "랑/이랑"),
      ("이며/며", "며/이며"),
      ("이야/야", "야/이야"),
      ("이라고/라고", "라고/이라고"),
      ("이든/든", "든/이든"),
    ];

    for noun in ["사과", "수박", "서울", "값!"] {
      for (forward, reverse) in aliases {
        let hangul = Hangul::new(noun);
        assert_eq!(
          hangul.josa(forward).unwrap(),
          hangul.josa(reverse).unwrap(),
          "{noun} {forward} vs {reverse}"
        );
        assert_eq!(
          hangul.josa_particle(forward).unwrap(),
          hangul.josa_particle(reverse).unwrap(),
          "{noun} particle {forward} vs {reverse}"
        );
      }
    }
  }

  #[test]
  fn test_josa_inserts_josa_particle() {
    let cases = [
      ("사과", "을/를"),
      ("수박", "아/야"),
      ("서울", "으로/로"),
      ("값!", "이라고/라고"),
      ("사과?!", "이야/야"),
      ("Hello", "이/가"),
    ];

    for (noun, pair) in cases {
      let hangul = Hangul::new(noun);
      let particle = hangul.josa_particle(pair).unwrap();
      let attached = hangul.josa(pair).unwrap();
      assert!(
        attached.contains(particle),
        "{noun} + {pair} should contain {particle}, got {attached}"
      );
      assert_eq!(
        attached.chars().count(),
        noun.chars().count() + particle.chars().count()
      );
    }
  }

  #[test]
  fn test_josa_rieul_exception_and_trailing_punctuation() {
    assert_eq!(Hangul::new("서울").josa("으로/로").unwrap(), "서울로");
    assert_eq!(Hangul::new("달").josa("으로/로").unwrap(), "달로");
    assert_eq!(Hangul::new("값!").josa("을/를").unwrap(), "값을!");
    assert_eq!(Hangul::new("사과?!").josa("을/를").unwrap(), "사과를?!");
  }

  #[test]
  fn test_josa_with_nfd_and_non_hangul() {
    let nfd = "\u{1112}\u{1161}\u{11AB}";
    assert_eq!(Hangul::new(nfd).josa("을/를").unwrap(), format!("{nfd}을"));
    assert_eq!(Hangul::new("Hello").josa("이/가").unwrap(), "Hello가");
  }

  #[test]
  fn test_josa_rejects_unsupported_pairs() {
    let error = Hangul::new("사과").josa("을").unwrap_err();
    assert_eq!(error, JosaError::InvalidPair("을".to_string()));
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
  fn test_consecutive_nfd_syllables() {
    // 안녕 in NFD: 안=안, 녕=녕
    let annyeong = "\u{110B}\u{1161}\u{11AB}\u{1102}\u{1167}\u{11BC}";
    let sentence = Hangul::new(annyeong);
    assert_eq!(sentence.len(), 2);
    assert_eq!(sentence.disassemble(), "ㅇㅏㄴㄴㅕㅇ");
    assert_eq!(sentence.get_choseong(), "ㅇㄴ");

    // 가가 in NFD (no batchim between syllables)
    let gaga = "\u{1100}\u{1161}\u{1100}\u{1161}";
    let sentence = Hangul::new(gaga);
    assert_eq!(sentence.len(), 2);
    assert_eq!(sentence.disassemble(), "ㄱㅏㄱㅏ");
    assert_eq!(sentence.get_choseong(), "ㄱㄱ");
  }

  #[test]
  fn test_nfd_matches_nfc_results() {
    let pairs = [
      ("간", "\u{1100}\u{1161}\u{11AB}"),
      ("가", "\u{1100}\u{1161}"),
      ("과", "\u{1100}\u{116A}"),
      ("값", "\u{1100}\u{1161}\u{11B9}"),
      ("안녕", "\u{110B}\u{1161}\u{11AB}\u{1102}\u{1167}\u{11BC}"),
    ];

    for (nfc, nfd) in pairs {
      let from_nfc = Hangul::new(nfc);
      let from_nfd = Hangul::new(nfd);

      assert_eq!(from_nfc.len(), from_nfd.len(), "len mismatch for {nfc}");
      assert_eq!(
        from_nfc.disassemble(),
        from_nfd.disassemble(),
        "disassemble mismatch for {nfc}"
      );
      assert_eq!(
        from_nfc.get_choseong(),
        from_nfd.get_choseong(),
        "choseong mismatch for {nfc}"
      );
    }
  }

  #[test]
  fn test_nfd_compound_jamo() {
    let gwa = Hangul::new("\u{1100}\u{116A}");
    assert_eq!(gwa.len(), 1);
    assert_eq!(gwa.disassemble(), "ㄱㅘ");
    assert_eq!(gwa.get_choseong(), "ㄱ");

    let gaps = Hangul::new("\u{1100}\u{1161}\u{11B9}");
    assert_eq!(gaps.len(), 1);
    assert_eq!(gaps.disassemble(), "ㄱㅏㅂㅅ");
    assert_eq!(gaps.get_choseong(), "ㄱ");
  }

  #[test]
  fn test_choseong_followed_by_nfc_syllable() {
    let mixed = Hangul::new("\u{1100}가");
    assert_eq!(mixed.len(), 2);
    assert_eq!(mixed.disassemble(), "\u{1100}ㄱㅏ");
    assert_eq!(mixed.get_choseong(), "\u{1100}ㄱ");
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

  #[test]
  fn test_units_and_get() {
    let sentence = Hangul::new("가A값");
    assert_eq!(sentence.len(), 3);
    assert_eq!(sentence.units().len(), 3);

    let first = sentence.get(0).unwrap();
    assert!(first.is_hangul());
    assert_eq!(first.original(), "가");
    assert_eq!(first.letter().unwrap().choseong.compatibility_value, 'ㄱ');
    assert_eq!(first.letter().unwrap().jungseong.compatibility_value, 'ㅏ');
    assert!(first.letter().unwrap().jongseong.is_none());
    assert_eq!(first.disassembled_chars(), vec!['ㄱ', 'ㅏ']);

    let middle = sentence.get(1).unwrap();
    assert!(!middle.is_hangul());
    assert_eq!(middle.original(), "A");
    assert!(middle.letter().is_none());
    assert_eq!(middle.disassembled_chars(), vec!['A']);

    let last = sentence.get(2).unwrap();
    assert_eq!(last.disassembled_chars(), vec!['ㄱ', 'ㅏ', 'ㅂ', 'ㅅ']);
    assert!(sentence.get(3).is_none());

    let units: Vec<_> = sentence.units().collect();
    assert_eq!(units.len(), 3);
    assert_eq!(sentence.letters().count(), 2);

    let mixed = Hangul::new("가A나");
    let letters: Vec<_> = mixed.letters().collect();
    assert_eq!(letters.len(), 2);
    assert_eq!(letters[0].choseong.compatibility_value, 'ㄱ');
    assert_eq!(letters[1].choseong.compatibility_value, 'ㄴ');
  }

  #[test]
  fn test_units_empty_lone_jamo_and_nfd_original() {
    let empty = Hangul::new("");
    assert!(empty.get(0).is_none());
    assert_eq!(empty.units().len(), 0);
    assert_eq!(empty.letters().count(), 0);

    let giyeok = Hangul::new("ㄱ");
    assert_eq!(giyeok.len(), 1);
    assert!(!giyeok.get(0).unwrap().is_hangul());
    assert_eq!(giyeok.get(0).unwrap().original(), "ㄱ");
    assert_eq!(giyeok.disassemble_to_groups(), vec![vec!['ㄱ']]);

    let vowel = Hangul::new("ㅏ");
    assert!(!vowel.get(0).unwrap().is_hangul());
    assert_eq!(vowel.disassemble_to_groups(), vec![vec!['ㅏ']]);

    let nfd = Hangul::new("\u{1100}\u{1161}\u{11AB}");
    assert_eq!(nfd.len(), 1);
    let unit = nfd.get(0).unwrap();
    assert!(unit.is_hangul());
    assert_eq!(unit.original(), "\u{1100}\u{1161}\u{11AB}");
    assert_eq!(unit.disassembled_chars(), vec!['ㄱ', 'ㅏ', 'ㄴ']);
  }

  #[test]
  fn test_unit_originals_rejoin_source() {
    let samples = [
      "",
      "가",
      "가A값",
      "Hello 안녕!",
      "\u{1100}\u{1161}\u{11AB}",
      "가\u{1100}\u{1161}!",
      "\u{110B}\u{1161}\u{11AB}\u{1102}\u{1167}\u{11BC}",
      "가🙂나",
    ];

    for sample in samples {
      let hangul = Hangul::new(sample);
      let rejoined: String = hangul.units().map(|unit| unit.original()).collect();
      assert_eq!(rejoined, sample, "units should rejoin {sample:?}");
    }
  }

  #[test]
  fn test_units_nfd_is_one_slot() {
    let nfd = Hangul::new("\u{1100}\u{1161}\u{11AB}");
    assert_eq!(nfd.len(), 1);
    let unit = nfd.get(0).unwrap();
    assert!(unit.is_hangul());
    assert_eq!(unit.disassembled_chars(), vec!['ㄱ', 'ㅏ', 'ㄴ']);
  }

  fn flatten_groups(groups: &[Vec<char>]) -> String {
    groups.iter().flatten().copied().collect()
  }

  #[test]
  fn test_disassemble_to_groups() {
    assert_eq!(
      Hangul::new("안녕").disassemble_to_groups(),
      vec![vec!['ㅇ', 'ㅏ', 'ㄴ'], vec!['ㄴ', 'ㅕ', 'ㅇ']]
    );
    assert_eq!(
      Hangul::new("과").disassemble_to_groups(),
      vec![vec!['ㄱ', 'ㅘ']]
    );
    assert_eq!(
      Hangul::new("값").disassemble_to_groups(),
      vec![vec!['ㄱ', 'ㅏ', 'ㅂ', 'ㅅ']]
    );
    assert_eq!(
      Hangul::new("가A!").disassemble_to_groups(),
      vec![vec!['ㄱ', 'ㅏ'], vec!['A'], vec!['!']]
    );
    assert_eq!(
      Hangul::new("").disassemble_to_groups(),
      Vec::<Vec<char>>::new()
    );
    assert_eq!(
      Hangul::new("가🙂나\n").disassemble_to_groups(),
      vec![vec!['ㄱ', 'ㅏ'], vec!['🙂'], vec!['ㄴ', 'ㅏ'], vec!['\n']]
    );
    assert_eq!(
      Hangul::new("안녕\t하세요").disassemble_to_groups(),
      vec![
        vec!['ㅇ', 'ㅏ', 'ㄴ'],
        vec!['ㄴ', 'ㅕ', 'ㅇ'],
        vec!['\t'],
        vec!['ㅎ', 'ㅏ'],
        vec!['ㅅ', 'ㅔ'],
        vec!['ㅇ', 'ㅛ']
      ]
    );

    let nfd = Hangul::new("\u{1100}\u{1161}\u{11AB}");
    assert_eq!(nfd.disassemble_to_groups(), vec![vec!['ㄱ', 'ㅏ', 'ㄴ']]);
  }

  #[test]
  fn test_disassemble_to_groups_flattens_to_disassemble() {
    let samples = [
      "",
      "안녕",
      "과",
      "값",
      "Hello 안녕!",
      "가🙂나\n",
      "ㄱㅏ",
      "\u{1100}\u{1161}\u{11AB}",
    ];

    for sample in samples {
      let hangul = Hangul::new(sample);
      assert_eq!(
        flatten_groups(&hangul.disassemble_to_groups()),
        hangul.disassemble(),
        "group flatten mismatch for {sample:?}"
      );
    }

    for code in S_BASE..=S_LAST {
      let syllable = char::from_u32(code).unwrap().to_string();
      let hangul = Hangul::new(&syllable);
      assert_eq!(
        flatten_groups(&hangul.disassemble_to_groups()),
        hangul.disassemble(),
        "group flatten mismatch for U+{code:04X} ({syllable})"
      );
    }
  }

  #[test]
  fn test_disassemble_to_groups_cache_reuse() {
    let sentence = Hangul::new("값A");
    let expected = vec![vec!['ㄱ', 'ㅏ', 'ㅂ', 'ㅅ'], vec!['A']];
    assert_eq!(sentence.disassemble_to_groups(), expected);
    assert_eq!(sentence.disassemble_to_groups(), expected);
  }

  #[test]
  fn test_josa_particle() {
    assert_eq!(Hangul::new("사과").josa_particle("을/를").unwrap(), "를");
    assert_eq!(Hangul::new("수박").josa_particle("을/를").unwrap(), "을");
    assert_eq!(Hangul::new("서울").josa_particle("으로/로").unwrap(), "로");
    assert_eq!(Hangul::new("값!").josa_particle("을/를").unwrap(), "을");
    assert_eq!(Hangul::new("Hello").josa_particle("이/가").unwrap(), "가");
    assert_eq!(Hangul::new("").josa_particle("이/가").unwrap(), "가");
    assert_eq!(Hangul::new("사과").josa_particle("아/야").unwrap(), "야");
    assert_eq!(
      Hangul::new("수박").josa_particle("이라고/라고").unwrap(),
      "이라고"
    );

    let error = Hangul::new("사과").josa_particle("을").unwrap_err();
    assert_eq!(error, JosaError::InvalidPair("을".to_string()));
  }

  #[test]
  fn test_clone_eq_display() {
    let original = Hangul::new("안녕!");
    let cloned = original.clone();
    assert_eq!(original, cloned);
    assert_eq!(cloned.disassemble(), "ㅇㅏㄴㄴㅕㅇ!");
    assert_eq!(cloned.to_string(), "안녕!");
    assert_ne!(Hangul::new("안녕"), Hangul::new("안녕!"));
  }
}
