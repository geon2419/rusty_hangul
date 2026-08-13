#![deny(clippy::all)]

#[macro_use]
extern crate napi_derive;

#[napi]
pub struct Hangul {
  hangul: hangul::Hangul,
}

#[napi]
impl Hangul {
  #[napi(constructor)]
  pub fn new(text: String) -> Self {
    Self {
      hangul: hangul::Hangul::new(&text),
    }
  }

  #[napi]
  pub fn disassemble(&self) -> String {
    self.hangul.disassemble()
  }

  #[napi]
  pub fn get_choseong(&self) -> String {
    self.hangul.get_choseong()
  }

  #[napi]
  pub fn has_batchim(&self) -> bool {
    self.hangul.has_batchim()
  }

  #[napi]
  pub fn josa(&self, pair: String) -> napi::Result<String> {
    self
      .hangul
      .josa(&pair)
      .map_err(|error| napi::Error::from_reason(error.to_string()))
  }
}

#[napi]
pub fn assemble(text: String) -> String {
  hangul::assemble(&text)
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_disassemble_basic() {
    let hangul = Hangul::new("안녕".to_string());
    assert_eq!(hangul.disassemble(), "ㅇㅏㄴㄴㅕㅇ");

    let hangul = Hangul::new("가나다".to_string());
    assert_eq!(hangul.disassemble(), "ㄱㅏㄴㅏㄷㅏ");

    let hangul = Hangul::new("한글".to_string());
    assert_eq!(hangul.disassemble(), "ㅎㅏㄴㄱㅡㄹ");
  }

  #[test]
  fn test_disassemble_with_non_hangul() {
    let hangul = Hangul::new("Hello 안녕!".to_string());
    assert_eq!(hangul.disassemble(), "Hello ㅇㅏㄴㄴㅕㅇ!");

    let hangul = Hangul::new("123 한글 ABC".to_string());
    assert_eq!(hangul.disassemble(), "123 ㅎㅏㄴㄱㅡㄹ ABC");
  }

  #[test]
  fn test_disassemble_empty_string() {
    let hangul = Hangul::new("".to_string());
    assert_eq!(hangul.disassemble(), "");
  }

  #[test]
  fn test_disassemble_complex_syllables() {
    let hangul = Hangul::new("꿈".to_string());
    assert_eq!(hangul.disassemble(), "ㄲㅜㅁ");

    let hangul = Hangul::new("밝다".to_string());
    assert_eq!(hangul.disassemble(), "ㅂㅏㄹㄱㄷㅏ");

    let hangul = Hangul::new("닭고기".to_string());
    assert_eq!(hangul.disassemble(), "ㄷㅏㄹㄱㄱㅗㄱㅣ");
  }

  #[test]
  fn test_disassemble_with_spaces() {
    let hangul = Hangul::new("안녕 하세요".to_string());
    assert_eq!(hangul.disassemble(), "ㅇㅏㄴㄴㅕㅇ ㅎㅏㅅㅔㅇㅛ");
  }

  #[test]
  fn test_get_choseong_basic() {
    let hangul = Hangul::new("안녕".to_string());
    assert_eq!(hangul.get_choseong(), "ㅇㄴ");

    let hangul = Hangul::new("가나다".to_string());
    assert_eq!(hangul.get_choseong(), "ㄱㄴㄷ");

    let hangul = Hangul::new("한글".to_string());
    assert_eq!(hangul.get_choseong(), "ㅎㄱ");
  }

  #[test]
  fn test_get_choseong_with_non_hangul() {
    let hangul = Hangul::new("Hello 안녕!".to_string());
    assert_eq!(hangul.get_choseong(), "Hello ㅇㄴ!");

    let hangul = Hangul::new("123 한글 ABC".to_string());
    assert_eq!(hangul.get_choseong(), "123 ㅎㄱ ABC");
  }

  #[test]
  fn test_get_choseong_empty_string() {
    let hangul = Hangul::new("".to_string());
    assert_eq!(hangul.get_choseong(), "");
  }

  #[test]
  fn test_get_choseong_complex_syllables() {
    let hangul = Hangul::new("꿈".to_string());
    assert_eq!(hangul.get_choseong(), "ㄲ");

    let hangul = Hangul::new("밝다".to_string());
    assert_eq!(hangul.get_choseong(), "ㅂㄷ");

    let hangul = Hangul::new("닭고기".to_string());
    assert_eq!(hangul.get_choseong(), "ㄷㄱㄱ");
  }

  #[test]
  fn test_get_choseong_with_spaces() {
    let hangul = Hangul::new("안녕 하세요".to_string());
    assert_eq!(hangul.get_choseong(), "ㅇㄴ ㅎㅅㅇ");
  }

  #[test]
  fn test_has_batchim() {
    assert!(Hangul::new("한".to_string()).has_batchim());
    assert!(!Hangul::new("하".to_string()).has_batchim());
    assert!(Hangul::new("값!".to_string()).has_batchim());
    assert!(Hangul::new("\u{1112}\u{1161}\u{11AB}".to_string()).has_batchim());
    assert!(!Hangul::new("Hello".to_string()).has_batchim());
  }

  #[test]
  fn test_josa() {
    assert_eq!(
      Hangul::new("사과".to_string())
        .josa("을/를".to_string())
        .unwrap(),
      "사과를"
    );
    assert_eq!(
      Hangul::new("수박".to_string())
        .josa("을/를".to_string())
        .unwrap(),
      "수박을"
    );
    assert_eq!(
      Hangul::new("서울".to_string())
        .josa("으로/로".to_string())
        .unwrap(),
      "서울로"
    );
    assert_eq!(
      Hangul::new("값!".to_string())
        .josa("을/를".to_string())
        .unwrap(),
      "값을!"
    );
  }

  #[test]
  fn test_josa_rejects_unsupported_pair() {
    assert!(Hangul::new("사과".to_string())
      .josa("을".to_string())
      .is_err());
  }

  #[test]
  fn test_assemble() {
    assert_eq!(assemble("ㄱㅏ".to_string()), "가");
    assert_eq!(assemble("ㄱㅏㅂㅅ".to_string()), "값");
    assert_eq!(assemble("ㄱㅏㄱㅏ".to_string()), "가가");
    assert_eq!(assemble("Hello ㄱㅏ!".to_string()), "Hello 가!");
  }
}
