use crate::choseong::Choseong;
use crate::jongseong::Jongseong;
use crate::jungseong::Jungseong;

const HANGUL_BASE: u32 = 0xAC00;
const JUNGSEONG_COUNT: u32 = 21;
const JONGSEONG_COUNT: u32 = 28;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AssemblePolicy {
  #[default]
  PreferNextSyllable,
  PreferCompoundFinal,
}

impl AssemblePolicy {
  pub fn parse(value: &str) -> Option<Self> {
    match value {
      "next-syllable" => Some(Self::PreferNextSyllable),
      "compound-final" => Some(Self::PreferCompoundFinal),
      _ => None,
    }
  }
}

pub fn assemble(string: &str) -> String {
  assemble_with_policy(string, AssemblePolicy::default())
}

pub fn assemble_with_policy(string: &str, policy: AssemblePolicy) -> String {
  let chars: Vec<char> = string.chars().collect();
  let mut result = String::with_capacity(string.len());
  let mut index = 0;

  while index < chars.len() {
    let Some(choseong_index) = Choseong::compatibility_index(chars[index]) else {
      result.push(chars[index]);
      index += 1;
      continue;
    };

    let Some(jungseong) = chars.get(index + 1) else {
      result.push(chars[index]);
      index += 1;
      continue;
    };
    let Some(jungseong_index) = Jungseong::compatibility_index(*jungseong) else {
      result.push(chars[index]);
      index += 1;
      continue;
    };

    let mut jongseong_index = 0;
    let mut consumed = 2;

    if let Some(&candidate) = chars.get(index + 2) {
      if let Some(candidate_index) = Jongseong::compatibility_index(candidate) {
        let second = chars.get(index + 3).copied();
        let next_is_jungseong = second
          .map(|ch| Jungseong::compatibility_index(ch).is_some())
          .unwrap_or(false);

        if !next_is_jungseong {
          if let Some(second) = second {
            if let Some(complex_index) = complex_jongseong_index(candidate, second) {
              let second_starts_syllable = Choseong::compatibility_index(second).is_some()
                && chars
                  .get(index + 4)
                  .and_then(|ch| Jungseong::compatibility_index(*ch))
                  .is_some();

              if second_starts_syllable && policy == AssemblePolicy::PreferNextSyllable {
                jongseong_index = candidate_index;
                consumed = 3;
              } else {
                jongseong_index = complex_index;
                consumed = 4;
              }
            } else {
              jongseong_index = candidate_index;
              consumed = 3;
            }
          } else {
            jongseong_index = candidate_index;
            consumed = 3;
          }
        }
      }
    }

    result.push(compose_syllable(
      choseong_index,
      jungseong_index,
      jongseong_index,
    ));
    index += consumed;
  }

  result
}

fn complex_jongseong_index(first: char, second: char) -> Option<usize> {
  let compound = match (first, second) {
    ('ㄱ', 'ㅅ') => 'ㄳ',
    ('ㄴ', 'ㅈ') => 'ㄵ',
    ('ㄴ', 'ㅎ') => 'ㄶ',
    ('ㄹ', 'ㄱ') => 'ㄺ',
    ('ㄹ', 'ㅁ') => 'ㄻ',
    ('ㄹ', 'ㅂ') => 'ㄼ',
    ('ㄹ', 'ㅅ') => 'ㄽ',
    ('ㄹ', 'ㅌ') => 'ㄾ',
    ('ㄹ', 'ㅍ') => 'ㄿ',
    ('ㄹ', 'ㅎ') => 'ㅀ',
    ('ㅂ', 'ㅅ') => 'ㅄ',
    _ => return None,
  };

  Jongseong::compatibility_index(compound)
}

fn compose_syllable(choseong_index: usize, jungseong_index: usize, jongseong_index: usize) -> char {
  let code = HANGUL_BASE
    + ((choseong_index as u32 * JUNGSEONG_COUNT + jungseong_index as u32) * JONGSEONG_COUNT)
    + jongseong_index as u32;

  char::from_u32(code).expect("modern Hangul syllable index must be valid")
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::Hangul;

  #[test]
  fn test_assemble_basic_syllables() {
    assert_eq!(assemble("ㄱㅏ"), "가");
    assert_eq!(assemble("ㅎㅏㄴ"), "한");
    assert_eq!(assemble("ㅇㅢ"), "의");
  }

  #[test]
  fn test_assemble_compound_jungseong_and_jongseong() {
    assert_eq!(assemble("ㄱㅘ"), "과");
    assert_eq!(assemble("ㄱㅏㄱㅅ"), "갃");
    assert_eq!(assemble("ㄷㅏㄹㄱ"), "닭");
    assert_eq!(assemble("ㄱㅏㅄ"), "값");
  }

  #[test]
  fn test_assemble_syllable_boundaries() {
    assert_eq!(assemble("ㄱㅏㄱㅏ"), "가가");
    assert_eq!(assemble("ㅇㅏㄴㄴㅕㅇ"), "안녕");
    assert_eq!(assemble("ㄱㅏㅂㅅㄷㅏ"), "값다");
    assert_eq!(assemble("ㄱㅏㄱㅅㅏ"), "각사");
    assert_eq!(assemble("ㄱㅏㄹㄱㅏ"), "갈가");
  }

  #[test]
  fn test_assemble_policies() {
    let input = "ㄱㅏㄱㅅㅏ";

    assert_eq!(assemble(input), "각사");
    assert_eq!(
      assemble_with_policy(input, AssemblePolicy::PreferNextSyllable),
      "각사"
    );
    assert_eq!(
      assemble_with_policy(input, AssemblePolicy::PreferCompoundFinal),
      "갃ㅏ"
    );

    assert_eq!(
      AssemblePolicy::parse("next-syllable"),
      Some(AssemblePolicy::PreferNextSyllable)
    );
    assert_eq!(
      AssemblePolicy::parse("compound-final"),
      Some(AssemblePolicy::PreferCompoundFinal)
    );
    assert_eq!(AssemblePolicy::parse("unknown"), None);
  }

  #[test]
  fn test_all_compound_jongseong_policy_boundaries() {
    let compounds = [
      ('ㄱ', 'ㅅ'),
      ('ㄴ', 'ㅈ'),
      ('ㄴ', 'ㅎ'),
      ('ㄹ', 'ㄱ'),
      ('ㄹ', 'ㅁ'),
      ('ㄹ', 'ㅂ'),
      ('ㄹ', 'ㅅ'),
      ('ㄹ', 'ㅌ'),
      ('ㄹ', 'ㅍ'),
      ('ㄹ', 'ㅎ'),
      ('ㅂ', 'ㅅ'),
    ];

    for (first, second) in compounds {
      let input = format!("ㄱㅏ{first}{second}ㅏ");
      let simple_final = compose_syllable(0, 0, Jongseong::compatibility_index(first).unwrap());
      let next_syllable = assemble(&format!("{second}ㅏ"));
      let compound_final = compose_syllable(0, 0, complex_jongseong_index(first, second).unwrap());

      assert_eq!(
        assemble(&input),
        format!("{simple_final}{next_syllable}"),
        "next-syllable policy failed for {input}"
      );
      assert_eq!(
        assemble_with_policy(&input, AssemblePolicy::PreferCompoundFinal),
        format!("{compound_final}ㅏ"),
        "compound-final policy failed for {input}"
      );
    }
  }

  #[test]
  fn test_policies_agree_for_unambiguous_inputs() {
    let inputs = [
      "",
      "ㄱ",
      "ㄱㅏ",
      "ㄱㅏㄱ",
      "ㄱㅏㄱㅏ",
      "ㄱㅏㅂㅅㄷㅏ",
      "Hello ㄱㅏ!",
      "안녕",
    ];

    for input in inputs {
      assert_eq!(
        assemble_with_policy(input, AssemblePolicy::PreferNextSyllable),
        assemble_with_policy(input, AssemblePolicy::PreferCompoundFinal),
        "policies unexpectedly differ for {input}"
      );
    }
  }

  #[test]
  fn test_assemble_is_idempotent_for_each_policy() {
    let inputs = ["ㄱㅏㄱㅅㅏ", "ㄱㅏㅂㅅㄷㅏ", "안녕", "Hello ㄱㅏ!"];
    let policies = [
      AssemblePolicy::PreferNextSyllable,
      AssemblePolicy::PreferCompoundFinal,
    ];

    for input in inputs {
      for policy in policies {
        let once = assemble_with_policy(input, policy);
        assert_eq!(
          assemble_with_policy(&once, policy),
          once,
          "assembly was not idempotent for {input}"
        );
      }
    }
  }

  #[test]
  fn test_assemble_preserves_non_jamo_text() {
    assert_eq!(assemble("Hello ㄱㅏ!"), "Hello 가!");
    assert_eq!(assemble(""), "");
    assert_eq!(assemble("ㄱ"), "ㄱ");
    assert_eq!(assemble("ㅏ"), "ㅏ");
    assert_eq!(assemble("ㄱㄴ"), "ㄱㄴ");
    assert_eq!(assemble("ㄸㅃㅉ"), "ㄸㅃㅉ");
    assert_eq!(assemble("ㅥㆎ🙂"), "ㅥㆎ🙂");
    assert_eq!(assemble("\u{1100}\u{1161}"), "\u{1100}\u{1161}");
    assert_eq!(assemble("ㄱㅏㄸ"), "가ㄸ");
    assert_eq!(assemble("안녕"), "안녕");

    let nfd = Hangul::new("\u{1100}\u{1161}\u{11AB}");
    assert_eq!(assemble(&nfd.disassemble()), "간");
  }

  #[test]
  fn test_assemble_round_trip_for_common_words() {
    for word in ["각사", "갈가", "값다", "닭고기", "안녕하세요"] {
      let disassembled = Hangul::new(word).disassemble();
      assert_eq!(assemble(&disassembled), word, "failed for {word}");
    }
  }

  #[test]
  fn test_assemble_is_the_inverse_of_disassemble_for_all_syllables() {
    for code in 0xAC00..=0xD7A3 {
      let syllable = char::from_u32(code).unwrap();
      let disassembled = Hangul::new(&syllable.to_string()).disassemble();

      assert_eq!(
        assemble(&disassembled),
        syllable.to_string(),
        "failed to assemble U+{code:04X} ({syllable}) from {disassembled}"
      );
    }
  }
}
