#![deny(clippy::all)]

#[macro_use]
extern crate napi_derive;

#[napi]
pub struct HangulCharUnit {
  original: String,
  is_hangul: bool,
  choseong: Option<String>,
  jungseong: Option<String>,
  jongseong: Option<String>,
}

#[napi]
impl HangulCharUnit {
  #[napi(getter)]
  pub fn original(&self) -> String {
    self.original.clone()
  }

  #[napi(getter)]
  pub fn is_hangul(&self) -> bool {
    self.is_hangul
  }

  #[napi(getter)]
  pub fn choseong(&self) -> Option<String> {
    self.choseong.clone()
  }

  #[napi(getter)]
  pub fn jungseong(&self) -> Option<String> {
    self.jungseong.clone()
  }

  #[napi(getter)]
  pub fn jongseong(&self) -> Option<String> {
    self.jongseong.clone()
  }
}

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

  #[napi(getter)]
  pub fn length(&self) -> u32 {
    self.hangul.len() as u32
  }

  #[napi]
  pub fn get(&self, index: u32) -> Option<HangulCharUnit> {
    self.hangul.get(index as usize).map(unit_to_js)
  }

  #[napi]
  pub fn disassemble(&self) -> String {
    self.hangul.disassemble()
  }

  #[napi]
  pub fn disassemble_to_groups(&self) -> Vec<Vec<String>> {
    groups_to_js(self.hangul.disassemble_to_groups())
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

  #[napi]
  pub fn josa_particle(&self, pair: String) -> napi::Result<String> {
    self
      .hangul
      .josa_particle(&pair)
      .map(str::to_string)
      .map_err(|error| napi::Error::from_reason(error.to_string()))
  }
}

#[napi]
pub fn assemble(text: String, policy: Option<String>) -> napi::Result<String> {
  let policy = match policy.as_deref() {
    Some(value) => hangul::AssemblePolicy::parse(value).ok_or_else(|| {
      napi::Error::from_reason(
        "invalid assemble policy; expected \"next-syllable\" or \"compound-final\"",
      )
    })?,
    None => hangul::AssemblePolicy::default(),
  };

  Ok(hangul::assemble_with_policy(&text, policy))
}

fn unit_to_js(unit: hangul::HangulUnit<'_>) -> HangulCharUnit {
  match unit.letter() {
    Some(letter) => HangulCharUnit {
      original: unit.original().to_string(),
      is_hangul: true,
      choseong: Some(letter.choseong.compatibility_value.to_string()),
      jungseong: Some(letter.jungseong.compatibility_value.to_string()),
      jongseong: letter
        .jongseong
        .as_ref()
        .map(|jongseong| jongseong.compatibility_value.to_string()),
    },
    None => HangulCharUnit {
      original: unit.original().to_string(),
      is_hangul: false,
      choseong: None,
      jungseong: None,
      jongseong: None,
    },
  }
}

fn groups_to_js(groups: Vec<Vec<char>>) -> Vec<Vec<String>> {
  groups
    .into_iter()
    .map(|group| group.into_iter().map(String::from).collect())
    .collect()
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
    assert_eq!(assemble("ㄱㅏ".to_string(), None).unwrap(), "가");
    assert_eq!(assemble("ㄱㅏㅂㅅ".to_string(), None).unwrap(), "값");
    assert_eq!(assemble("ㄱㅏㄱㅏ".to_string(), None).unwrap(), "가가");
    assert_eq!(
      assemble("Hello ㄱㅏ!".to_string(), None).unwrap(),
      "Hello 가!"
    );
  }

  #[test]
  fn test_assemble_policy() {
    let input = "ㄱㅏㄱㅅㅏ".to_string();

    assert_eq!(
      assemble(input.clone(), Some("next-syllable".to_string())).unwrap(),
      "각사"
    );
    assert_eq!(
      assemble(input, Some("compound-final".to_string())).unwrap(),
      "갃ㅏ"
    );
    assert!(assemble("ㄱㅏ".to_string(), Some("unknown".to_string())).is_err());
  }

  #[test]
  fn test_length_get_and_groups() {
    let hangul = Hangul::new("가A값".to_string());
    assert_eq!(hangul.length(), 3);

    let first = hangul.get(0).unwrap();
    assert!(first.is_hangul());
    assert_eq!(first.original(), "가");
    assert_eq!(first.choseong().as_deref(), Some("ㄱ"));
    assert_eq!(first.jungseong().as_deref(), Some("ㅏ"));
    assert_eq!(first.jongseong(), None);

    let middle = hangul.get(1).unwrap();
    assert!(!middle.is_hangul());
    assert_eq!(middle.original(), "A");

    let last = hangul.get(2).unwrap();
    assert_eq!(last.jongseong().as_deref(), Some("ㅄ"));
    assert!(hangul.get(3).is_none());

    assert_eq!(
      hangul.disassemble_to_groups(),
      vec![
        vec!["ㄱ".to_string(), "ㅏ".to_string()],
        vec!["A".to_string()],
        vec![
          "ㄱ".to_string(),
          "ㅏ".to_string(),
          "ㅂ".to_string(),
          "ㅅ".to_string()
        ]
      ]
    );
  }

  #[test]
  fn test_josa_particle_and_new_pairs() {
    assert_eq!(
      Hangul::new("사과".to_string())
        .josa_particle("을/를".to_string())
        .unwrap(),
      "를"
    );
    assert_eq!(
      Hangul::new("수박".to_string())
        .josa("아/야".to_string())
        .unwrap(),
      "수박아"
    );
    assert_eq!(
      Hangul::new("사과".to_string())
        .josa("라고/이라고".to_string())
        .unwrap(),
      "사과라고"
    );
    assert_eq!(
      Hangul::new("값!".to_string())
        .josa("아/야".to_string())
        .unwrap(),
      "값아!"
    );
    assert!(Hangul::new("사과".to_string())
      .josa_particle("을".to_string())
      .is_err());
  }
}
