use hangul::Hangul;
use js_sys::Array;
use wasm_bindgen::prelude::*;

#[wasm_bindgen(js_name = disassemble)]
pub fn disassemble(text: &str) -> String {
  Hangul::new(text).disassemble()
}

#[wasm_bindgen(js_name = disassembleToGroups)]
pub fn disassemble_to_groups(text: &str) -> Array {
  groups_to_js(Hangul::new(text).disassemble_to_groups())
}

#[wasm_bindgen(js_name = getChoseong)]
pub fn get_choseong(text: &str) -> String {
  Hangul::new(text).get_choseong()
}

#[wasm_bindgen(js_name = hasBatchim)]
pub fn has_batchim(text: &str) -> bool {
  Hangul::new(text).has_batchim()
}

#[wasm_bindgen(js_name = josa)]
pub fn josa(text: &str, pair: &str) -> Result<String, JsValue> {
  Hangul::new(text)
    .josa(pair)
    .map_err(|error| JsValue::from_str(&error.to_string()))
}

#[wasm_bindgen(js_name = josaParticle)]
pub fn josa_particle(text: &str, pair: &str) -> Result<String, JsValue> {
  Hangul::new(text)
    .josa_particle(pair)
    .map(str::to_string)
    .map_err(|error| JsValue::from_str(&error.to_string()))
}

#[wasm_bindgen(js_name = hangulLen)]
pub fn hangul_len(text: &str) -> usize {
  Hangul::new(text).len()
}

#[wasm_bindgen(js_name = unitAt)]
pub fn unit_at(text: &str, index: usize) -> Option<HangulCharUnit> {
  Hangul::new(text).get(index).map(unit_to_js)
}

#[wasm_bindgen(js_name = assemble)]
pub fn assemble(text: &str, policy: Option<String>) -> Result<String, JsValue> {
  let policy = match policy.as_deref() {
    Some(value) => hangul::AssemblePolicy::parse(value).ok_or_else(|| {
      JsValue::from_str("invalid assemble policy; expected \"next-syllable\" or \"compound-final\"")
    })?,
    None => hangul::AssemblePolicy::default(),
  };

  Ok(hangul::assemble_with_policy(text, policy))
}

#[wasm_bindgen(js_name = containsChoseong)]
pub fn contains_choseong(text: &str, query: &str) -> bool {
  Hangul::new(text).contains_choseong(query)
}

#[wasm_bindgen(js_name = findChoseong)]
pub fn find_choseong(text: &str, query: &str) -> Option<JsChoseongMatch> {
  Hangul::new(text).find_choseong(query).map(match_to_js)
}

#[wasm_bindgen(js_name = ChoseongMatch)]
pub struct JsChoseongMatch {
  pub start: usize,
  pub end: usize,
  #[wasm_bindgen(js_name = byteStart)]
  pub byte_start: usize,
  #[wasm_bindgen(js_name = byteEnd)]
  pub byte_end: usize,
}

fn match_to_js(found: hangul::ChoseongMatch) -> JsChoseongMatch {
  JsChoseongMatch {
    start: found.start,
    end: found.end,
    byte_start: found.byte_start,
    byte_end: found.byte_end,
  }
}

#[wasm_bindgen(getter_with_clone)]
pub struct HangulCharUnit {
  pub original: String,
  #[wasm_bindgen(js_name = isHangul)]
  pub is_hangul: bool,
  pub choseong: Option<String>,
  pub jungseong: Option<String>,
  pub jongseong: Option<String>,
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

fn groups_to_js(groups: Vec<Vec<char>>) -> Array {
  let result = Array::new();

  for group in groups {
    let inner = Array::new();
    for ch in group {
      inner.push(&JsValue::from_str(&ch.to_string()));
    }
    result.push(&inner);
  }

  result
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn wasm_surface_smoke() {
    assert_eq!(disassemble("가A값"), "ㄱㅏAㄱㅏㅂㅅ");
    assert_eq!(get_choseong("가A값"), "ㄱAㄱ");
    assert!(has_batchim("값!"));
    assert_eq!(josa("값!", "을/를").unwrap(), "값을!");
    assert_eq!(josa_particle("사과", "을/를").unwrap(), "를");
    assert_eq!(hangul_len("가A값"), 3);
    assert!(unit_at("가A값", 0).unwrap().is_hangul);
    assert_eq!(unit_at("가A값", 0).unwrap().original, "가");
    assert_eq!(
      unit_at("\u{1100}\u{1161}\u{11AB}", 0).unwrap().original,
      "\u{1100}\u{1161}\u{11AB}"
    );
    assert!(!unit_at("ㄱ", 0).unwrap().is_hangul);
    assert!(unit_at("가A값", 3).is_none());
    assert_eq!(assemble("ㄱㅏㄱㅅㅏ", None).unwrap(), "각사");
    assert!(contains_choseong("한글", "ㅎㄱ"));
    assert_eq!(find_choseong("한글", "한ㄱ").unwrap().end, 2);
  }

  #[cfg(target_arch = "wasm32")]
  #[wasm_bindgen_test::wasm_bindgen_test]
  fn wasm_runtime_js_shape() {
    use wasm_bindgen::JsCast;

    wasm_surface_smoke();
    assert_eq!(
      assemble("ㄱㅏㄱㅅㅏ", Some("compound-final".to_string())).unwrap(),
      "갃ㅏ"
    );
    assert!(assemble("ㄱㅏ", Some("unknown".to_string())).is_err());

    let expected = hangul::Hangul::new("가A!").disassemble_to_groups();
    let js_groups = disassemble_to_groups("가A!");
    assert_eq!(js_groups.length() as usize, expected.len());
    for (index, group) in expected.iter().enumerate() {
      let inner = js_groups
        .get(index as u32)
        .dyn_into::<Array>()
        .expect("group should be an array");
      assert_eq!(inner.length() as usize, group.len());
    }

    let unit = JsValue::from(unit_at("가", 0).unwrap());
    let camel = js_sys::Reflect::get(&unit, &JsValue::from_str("isHangul")).unwrap();
    let snake = js_sys::Reflect::get(&unit, &JsValue::from_str("is_hangul")).unwrap();
    assert_eq!(camel.as_bool(), Some(true));
    assert!(snake.is_undefined());
  }
}
