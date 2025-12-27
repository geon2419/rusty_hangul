use crate::choseong::Choseong;
use crate::jongseong::Jongseong;
use crate::jungseong::Jungseong;
use crate::nfc::NFC;
use crate::nfd::NFD;

pub struct HangulLetter {
  value_chars: [char; 3],
  unicode_codes: [u32; 3],
  len: u8,
  nfc: Option<char>,
  pub choseong: Choseong,
  pub jungseong: Jungseong,
  pub jongseong: Option<Jongseong>,
}

impl HangulLetter {
  pub fn parse(string: &str) -> Option<Self> {
    if NFC::is_nfc_hangul(string) {
      let ch = string.chars().next().unwrap();
      let unicode = ch as u32;
      let NFD(cho, jung, jong) = NFD::normalize(unicode).unwrap();
      let mut value_chars = ['\0'; 3];
      let mut unicode_codes = [0u32; 3];
      value_chars[0] = ch;
      unicode_codes[0] = unicode;

      return Some(Self {
        value_chars,
        unicode_codes,
        len: 1,
        nfc: Some(ch),
        choseong: Choseong::new(cho),
        jungseong: Jungseong::new(jung),
        jongseong: jong.map(Jongseong::new),
      });
    }

    if NFD::is_nfd_hangul(string) {
      let mut value_chars = ['\0'; 3];
      let mut unicode_codes = [0u32; 3];
      let mut len: u8 = 0;

      for ch in string.chars() {
        let index = len as usize;
        value_chars[index] = ch;
        unicode_codes[index] = ch as u32;
        len += 1;
      }

      let choseong = Choseong::new(value_chars[0] as u32);
      let jungseong = Jungseong::new(value_chars[1] as u32);
      let jongseong = if len == 3 {
        Some(Jongseong::new(value_chars[2] as u32))
      } else {
        None
      };

      return Some(Self {
        value_chars,
        unicode_codes,
        len,
        nfc: None,
        choseong,
        jungseong,
        jongseong,
      });
    }

    return None;
  }

  pub fn parse_from_char(nfc_char: char) -> Option<Self> {
    if NFC::is_nfc_hangul_char(nfc_char) {
      let unicode = nfc_char as u32;
      let NFD(cho, jung, jong) = NFD::normalize(unicode).unwrap();
      let mut value_chars = ['\0'; 3];
      let mut unicode_codes = [0u32; 3];
      value_chars[0] = nfc_char;
      unicode_codes[0] = unicode;

      return Some(Self {
        value_chars,
        unicode_codes,
        len: 1,
        nfc: Some(nfc_char),
        choseong: Choseong::new(cho),
        jungseong: Jungseong::new(jung),
        jongseong: jong.map(Jongseong::new),
      });
    }
    None
  }

  pub fn value_chars(&self) -> &[char] {
    &self.value_chars[..self.len as usize]
  }

  pub fn unicode_codes(&self) -> &[u32] {
    &self.unicode_codes[..self.len as usize]
  }

  pub fn value_nfc(&self) -> Option<char> {
    self.nfc
  }

  pub fn value_string(&self) -> String {
    self.value_chars().iter().collect()
  }

  pub fn append_disassembled(&self, output: &mut String) {
    output.push(self.choseong.compatibility_value);
    output.push(self.jungseong.compatibility_value);

    if let Some(ref jong) = self.jongseong {
      jong.append_disassembled(output);
    }
  }

  pub fn disassemble(&self) -> String {
    let mut result = String::with_capacity(4);

    self.append_disassembled(&mut result);

    result
  }

  pub fn has_batchim(&self) -> bool {
    self.jongseong.is_some()
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_parse_nfc_hangul() {
    let hangul = HangulLetter::parse("가").unwrap();
    assert_eq!(hangul.value_string(), "가");
    assert_eq!(hangul.unicode_codes(), &[0xAC00]);
    assert_eq!(hangul.choseong.compatibility_value, 'ㄱ');
    assert_eq!(hangul.jungseong.compatibility_value, 'ㅏ');
    assert!(hangul.jongseong.is_none());

    let hangul = HangulLetter::parse("한").unwrap();
    assert_eq!(hangul.value_string(), "한");
    assert_eq!(hangul.unicode_codes(), &[0xD55C]);
    assert_eq!(hangul.choseong.compatibility_value, 'ㅎ');
    assert_eq!(hangul.jungseong.compatibility_value, 'ㅏ');
    assert_eq!(hangul.jongseong.unwrap().compatibility_value, 'ㄴ');

    let hangul = HangulLetter::parse("쌍").unwrap();
    assert_eq!(hangul.value_string(), "쌍");
    assert_eq!(hangul.choseong.compatibility_value, 'ㅆ');
    assert_eq!(hangul.jungseong.compatibility_value, 'ㅏ');
    assert_eq!(hangul.jongseong.unwrap().compatibility_value, 'ㅇ');

    let hangul = HangulLetter::parse("귀").unwrap();
    assert_eq!(hangul.value_string(), "귀");
    assert_eq!(hangul.choseong.compatibility_value, 'ㄱ');
    assert_eq!(hangul.jungseong.compatibility_value, 'ㅟ');
    assert!(hangul.jongseong.is_none());

    let hangul = HangulLetter::parse("값").unwrap();
    assert_eq!(hangul.value_string(), "값");
    assert_eq!(hangul.choseong.compatibility_value, 'ㄱ');
    assert_eq!(hangul.jungseong.compatibility_value, 'ㅏ');
    assert_eq!(hangul.jongseong.unwrap().compatibility_value, 'ㅄ');
  }

  #[test]
  fn test_parse_nfd_hangul() {
    let nfd_ga = "\u{1100}\u{1161}";
    let hangul = HangulLetter::parse(nfd_ga).unwrap();
    assert_eq!(hangul.value_string(), nfd_ga);
    assert_eq!(hangul.unicode_codes(), &[0x1100, 0x1161]);
    assert_eq!(hangul.choseong.compatibility_value, 'ㄱ');
    assert_eq!(hangul.jungseong.compatibility_value, 'ㅏ');
    assert!(hangul.jongseong.is_none());

    let nfd_han = "\u{1112}\u{1161}\u{11AB}";
    let hangul = HangulLetter::parse(nfd_han).unwrap();
    assert_eq!(hangul.value_string(), nfd_han);
    assert_eq!(hangul.unicode_codes(), &[0x1112, 0x1161, 0x11AB]);
    assert_eq!(hangul.choseong.compatibility_value, 'ㅎ');
    assert_eq!(hangul.jungseong.compatibility_value, 'ㅏ');
    assert_eq!(hangul.jongseong.unwrap().compatibility_value, 'ㄴ');
  }

  #[test]
  fn test_parse_from_char() {
    let hangul = HangulLetter::parse_from_char('한').unwrap();
    assert_eq!(hangul.value_string(), "한");
    assert_eq!(hangul.unicode_codes(), &[0xD55C]);

    assert!(HangulLetter::parse_from_char('a').is_none());
    assert!(HangulLetter::parse_from_char('ㄱ').is_none());
    assert!(HangulLetter::parse_from_char('!').is_none());
  }

  #[test]
  fn test_disassemble() {
    let hangul = HangulLetter::parse("가").unwrap();
    assert_eq!(hangul.disassemble(), "ㄱㅏ");

    let hangul = HangulLetter::parse("한").unwrap();
    assert_eq!(hangul.disassemble(), "ㅎㅏㄴ");

    let hangul = HangulLetter::parse("의").unwrap();
    assert_eq!(hangul.disassemble(), "ㅇㅢ");

    let hangul = HangulLetter::parse("값").unwrap();
    assert_eq!(hangul.disassemble(), "ㄱㅏㅂㅅ");
  }

  #[test]
  fn test_has_batchim() {
    assert!(HangulLetter::parse("한").unwrap().has_batchim());
    assert!(HangulLetter::parse("값").unwrap().has_batchim());

    assert!(!HangulLetter::parse("가").unwrap().has_batchim());
    assert!(!HangulLetter::parse("뉘").unwrap().has_batchim());
  }

  #[test]
  fn test_invalid_input() {
    assert!(HangulLetter::parse("a").is_none());
    assert!(HangulLetter::parse("ㄱ").is_none());
    assert!(HangulLetter::parse("ㅏ").is_none());
    assert!(HangulLetter::parse("ᄀᄀ").is_none());
    assert!(HangulLetter::parse("ᅡᅡ").is_none());
    assert!(HangulLetter::parse("ᄀᆫ").is_none());
    assert!(HangulLetter::parse("").is_none());
    assert!(HangulLetter::parse("!").is_none());
    assert!(HangulLetter::parse("가나").is_none());
  }
}
