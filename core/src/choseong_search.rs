use crate::choseong::Choseong;
use crate::hangul::{Hangul, HangulUnit};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ChoseongMatch {
  pub start: usize,
  pub end: usize,
  pub byte_start: usize,
  pub byte_end: usize,
}

pub(crate) fn find_choseong(target: &Hangul, query: &str) -> Option<ChoseongMatch> {
  let query = Hangul::new(query);
  if query.is_empty() || query.len() > target.len() {
    return None;
  }

  let query_len = query.len();
  let last_start = target.len() - query_len;

  for start in 0..=last_start {
    let matched = (0..query_len).all(|offset| {
      unit_matches(
        query.get(offset).expect("query index in range"),
        target.get(start + offset).expect("target index in range"),
      )
    });

    if matched {
      let first = target.get(start).expect("match start in range");
      let last = target
        .get(start + query_len - 1)
        .expect("match end in range");
      return Some(ChoseongMatch {
        start,
        end: start + query_len,
        byte_start: first.byte_start(),
        byte_end: last.byte_end(),
      });
    }
  }

  None
}

fn unit_matches(query: HangulUnit<'_>, target: HangulUnit<'_>) -> bool {
  if let (Some(query_letter), Some(target_letter)) = (query.letter(), target.letter()) {
    return is_prefix(
      &query_letter.disassembled_chars(),
      &target_letter.disassembled_chars(),
    );
  }

  if let Some(target_letter) = target.letter() {
    return query_choseong(query.original()) == Some(target_letter.choseong.compatibility_value);
  }

  query.original() == target.original()
}

fn query_choseong(ch: char) -> Option<char> {
  if Choseong::compatibility_index(ch).is_some() {
    return Some(ch);
  }

  if Choseong::is_conjoining_choseong(ch as u32) {
    return Some(Choseong::new(ch as u32).compatibility_value);
  }

  None
}

fn is_prefix(query: &[char], target: &[char]) -> bool {
  query.len() <= target.len() && target.starts_with(query)
}

#[cfg(test)]
mod tests {
  use super::*;

  fn match_at(start: usize, end: usize, byte_start: usize, byte_end: usize) -> ChoseongMatch {
    ChoseongMatch {
      start,
      end,
      byte_start,
      byte_end,
    }
  }

  #[test]
  fn test_choseong_query_finds_hangul() {
    let text = Hangul::new("한글날");
    assert_eq!(
      text.find_choseong("ㅎ"),
      Some(match_at(0, 1, 0, "한".len()))
    );
    assert_eq!(
      text.find_choseong("ㅎㄱ"),
      Some(match_at(0, 2, 0, "한글".len()))
    );
    assert_eq!(text.find_choseong("ㅎㄴ"), None);
    assert!(text.contains_choseong("ㅎㄱ"));
    assert!(!text.contains_choseong("ㅎㄴ"));
  }

  #[test]
  fn test_progressive_syllable_query() {
    let text = Hangul::new("한글");
    assert_eq!(
      text.find_choseong("한ㄱ"),
      Some(match_at(0, 2, 0, "한글".len()))
    );
    assert_eq!(
      text.find_choseong("한글"),
      Some(match_at(0, 2, 0, "한글".len()))
    );
    assert_eq!(
      text.find_choseong("하ㄱ"),
      Some(match_at(0, 2, 0, "한글".len()))
    );
    assert_eq!(text.find_choseong("한ㄴ"), None);
  }

  #[test]
  fn test_first_match_and_overlap() {
    let text = Hangul::new("가가");
    assert_eq!(
      text.find_choseong("ㄱ"),
      Some(match_at(0, 1, 0, "가".len()))
    );
    assert_eq!(
      text.find_choseong("ㄱㄱ"),
      Some(match_at(0, 2, 0, "가가".len()))
    );
  }

  #[test]
  fn test_mixed_text_and_whitespace() {
    let text = Hangul::new("A한글");
    assert_eq!(
      text.find_choseong("Aㅎㄱ"),
      Some(match_at(0, 3, 0, "A한글".len()))
    );
    assert_eq!(
      text.find_choseong("ㅎㄱ"),
      Some(match_at(1, 3, "A".len(), "A한글".len()))
    );
    assert_eq!(Hangul::new("한 글").find_choseong("ㅎㄱ"), None);
    assert!(Hangul::new("한 글").contains_choseong("ㅎ ㄱ"));
  }

  #[test]
  fn test_nfd_and_empty() {
    let nfd = Hangul::new("\u{1100}\u{1161}\u{11AB}");
    assert_eq!(
      nfd.find_choseong("ㄱ"),
      Some(match_at(0, 1, 0, nfd.original().len()))
    );
    assert_eq!(
      nfd.find_choseong("간"),
      Some(match_at(0, 1, 0, nfd.original().len()))
    );
    assert_eq!(Hangul::new("한글").find_choseong(""), None);
    assert_eq!(Hangul::new("").find_choseong("ㄱ"), None);
    assert_eq!(Hangul::new("가").find_choseong("ㄱㄱ"), None);
  }

  #[test]
  fn test_compound_jongseong_prefix() {
    let text = Hangul::new("값");
    assert!(text.contains_choseong("ㄱ"));
    assert!(text.contains_choseong("가"));
    assert!(text.contains_choseong("갑"));
    assert!(text.contains_choseong("값"));
    assert!(!text.contains_choseong("간"));
  }
}
