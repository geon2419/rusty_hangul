use hangul::Hangul;
use wasm_bindgen::prelude::*;

#[wasm_bindgen(js_name = disassemble)]
pub fn disassemble(text: &str) -> String {
  Hangul::new(text).disassemble()
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
}
