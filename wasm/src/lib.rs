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

  fn assert_assemble_policies() {
    assert_eq!(assemble("ㄱㅏㄱㅅㅏ", None).unwrap(), "각사");
    assert_eq!(
      assemble("ㄱㅏㄱㅅㅏ", Some("compound-final".to_string())).unwrap(),
      "갃ㅏ"
    );

    let compounds = [
      ('ㄳ', '갃'),
      ('ㄵ', '갅'),
      ('ㄶ', '갆'),
      ('ㄺ', '갉'),
      ('ㄻ', '갊'),
      ('ㄼ', '갋'),
      ('ㄽ', '갌'),
      ('ㄾ', '갍'),
      ('ㄿ', '갎'),
      ('ㅀ', '갏'),
      ('ㅄ', '값'),
    ];

    for (compound, syllable) in compounds {
      let input = format!("ㄱㅏ{compound}ㅏ");
      let expected = format!("{syllable}ㅏ");

      assert_eq!(assemble(&input, None).unwrap(), expected);
      assert_eq!(
        assemble(&input, Some("next-syllable".to_string())).unwrap(),
        expected
      );
      assert_eq!(
        assemble(&input, Some("compound-final".to_string())).unwrap(),
        expected
      );
    }
  }

  #[test]
  fn test_assemble_policies() {
    assert_assemble_policies();
  }

  #[cfg(target_arch = "wasm32")]
  #[wasm_bindgen_test::wasm_bindgen_test]
  fn test_assemble_policies_in_wasm_runtime() {
    assert_assemble_policies();
  }

  #[cfg(target_arch = "wasm32")]
  #[wasm_bindgen_test::wasm_bindgen_test]
  fn test_assemble_rejects_unknown_policy() {
    assert!(assemble("ㄱㅏ", Some("unknown".to_string())).is_err());
  }

  fn assert_new_surface() {
    assert_eq!(hangul_len("가A값"), 3);
    assert!(unit_at("", 0).is_none());

    let first = unit_at("가A값", 0).unwrap();
    assert!(first.is_hangul);
    assert_eq!(first.choseong.as_deref(), Some("ㄱ"));
    assert!(unit_at("가A값", 3).is_none());

    let other = unit_at("ㄱ", 0).unwrap();
    assert!(!other.is_hangul);
    assert_eq!(other.original, "ㄱ");

    assert_eq!(josa_particle("사과", "을/를").unwrap(), "를");
    assert_eq!(josa_particle("사과", "를/을").unwrap(), "를");
    assert_eq!(josa("수박", "아/야").unwrap(), "수박아");
    assert_eq!(josa("수박", "야/아").unwrap(), "수박아");
    assert_eq!(josa("값!", "이라고/라고").unwrap(), "값이라고!");

    assert!(contains_choseong("한글", "ㅎㄱ"));
    let found = find_choseong("한글", "한ㄱ").unwrap();
    assert_eq!(found.start, 0);
    assert_eq!(found.end, 2);
    assert!(find_choseong("한글", "ㅎㄴ").is_none());
    assert!(contains_choseong("꿈", "ㄲ"));
    assert!(!contains_choseong("꿈", "ㄱ"));
    assert!(!contains_choseong("과", "고"));

    for sample in ["", "값", "가A!", "과", "Hello 안녕!"] {
      let groups = hangul::Hangul::new(sample).disassemble_to_groups();
      let flat: String = groups.iter().flatten().collect();
      assert_eq!(flat, hangul::Hangul::new(sample).disassemble());
    }
  }

  #[cfg(target_arch = "wasm32")]
  fn assert_js_groups_match_core(text: &str) {
    use wasm_bindgen::JsCast;

    let expected = hangul::Hangul::new(text).disassemble_to_groups();
    let js_groups = disassemble_to_groups(text);
    assert_eq!(js_groups.length() as usize, expected.len());

    for (index, group) in expected.iter().enumerate() {
      let inner = js_groups
        .get(index as u32)
        .dyn_into::<Array>()
        .expect("group should be an array");
      assert_eq!(inner.length() as usize, group.len());

      for (jamo_index, ch) in group.iter().enumerate() {
        let expected = ch.to_string();
        assert_eq!(
          inner.get(jamo_index as u32).as_string().as_deref(),
          Some(expected.as_str())
        );
      }
    }
  }

  #[test]
  fn test_units_groups_and_josa_particle() {
    assert_new_surface();
  }

  #[cfg(target_arch = "wasm32")]
  #[wasm_bindgen_test::wasm_bindgen_test]
  fn test_units_groups_and_josa_particle_in_wasm_runtime() {
    assert_new_surface();
    assert_js_groups_match_core("값");
    assert_js_groups_match_core("가A!");
    assert_js_groups_match_core("과");
    assert_js_groups_match_core("");
    assert_js_unit_exposes_is_hangul();
  }

  #[cfg(target_arch = "wasm32")]
  fn assert_js_unit_exposes_is_hangul() {
    let unit = JsValue::from(unit_at("가", 0).unwrap());
    let camel = js_sys::Reflect::get(&unit, &JsValue::from_str("isHangul")).unwrap();
    let snake = js_sys::Reflect::get(&unit, &JsValue::from_str("is_hangul")).unwrap();
    assert_eq!(camel.as_bool(), Some(true));
    assert!(snake.is_undefined());
  }
}
