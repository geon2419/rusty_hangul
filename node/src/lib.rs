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

#[napi(js_name = "ChoseongMatch")]
pub struct ChoseongMatchJs {
  start: u32,
  end: u32,
  byte_start: u32,
  byte_end: u32,
}

#[napi]
impl ChoseongMatchJs {
  #[napi(getter)]
  pub fn start(&self) -> u32 {
    self.start
  }

  #[napi(getter)]
  pub fn end(&self) -> u32 {
    self.end
  }

  #[napi(getter)]
  pub fn byte_start(&self) -> u32 {
    self.byte_start
  }

  #[napi(getter)]
  pub fn byte_end(&self) -> u32 {
    self.byte_end
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

  #[napi]
  pub fn contains_choseong(&self, query: String) -> bool {
    self.hangul.contains_choseong(&query)
  }

  #[napi]
  pub fn find_choseong(&self, query: String) -> Option<ChoseongMatchJs> {
    self.hangul.find_choseong(&query).map(match_to_js)
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

fn match_to_js(found: hangul::ChoseongMatch) -> ChoseongMatchJs {
  ChoseongMatchJs {
    start: found.start as u32,
    end: found.end as u32,
    byte_start: found.byte_start as u32,
    byte_end: found.byte_end as u32,
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn napi_surface_smoke() {
    let hangul = Hangul::new("가A값!".to_string());
    assert_eq!(hangul.length(), 4);
    assert_eq!(hangul.disassemble(), "ㄱㅏAㄱㅏㅂㅅ!");
    assert_eq!(hangul.get_choseong(), "ㄱAㄱ!");
    assert!(hangul.has_batchim());
    assert_eq!(hangul.josa("을/를".to_string()).unwrap(), "가A값을!");
    assert_eq!(hangul.josa_particle("을/를".to_string()).unwrap(), "을");
    assert!(hangul.contains_choseong("ㄱ".to_string()));
    assert_eq!(hangul.find_choseong("ㄱA".to_string()).unwrap().end(), 2);

    let first = hangul.get(0).unwrap();
    assert!(first.is_hangul());
    assert_eq!(first.choseong().as_deref(), Some("ㄱ"));
    assert!(hangul.get(4).is_none());
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
        ],
        vec!["!".to_string()]
      ]
    );

    assert_eq!(assemble("ㄱㅏㅂㅅ".to_string(), None).unwrap(), "값");
    assert_eq!(
      assemble("ㄱㅏㄱㅅㅏ".to_string(), Some("compound-final".to_string())).unwrap(),
      "갃ㅏ"
    );
    assert!(assemble("ㄱㅏ".to_string(), Some("unknown".to_string())).is_err());
    assert!(Hangul::new("사과".to_string())
      .josa("을".to_string())
      .is_err());
  }
}
