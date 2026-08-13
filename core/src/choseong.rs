// 조합형 초성 범위
const CHOSEONG_BASE: u32 = 0x1100;
const CHOSEONG_LAST: u32 = 0x1112;

// 호환형 초성 범위
const COMPAT_CHOSEONG_BASE: u32 = 0x3131;
const COMPAT_CHOSEONG_LAST: u32 = 0x314E;

// 호환형 초성 매핑 테이블
const COMPATIBILITY_CHOSEONG_MAPPING: [u32; 19] = [
  0x3131, 0x3132, 0x3134, 0x3137, 0x3138, 0x3139, 0x3141, 0x3142, 0x3143, 0x3145, 0x3146, 0x3147,
  0x3148, 0x3149, 0x314A, 0x314B, 0x314C, 0x314D, 0x314E,
];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Choseong {
  pub conjoining_value: char,
  pub conjoining_unicode: u32,
  pub compatibility_value: char,
  pub compatibility_unicode: u32,
}

impl Choseong {
  pub fn new(unicode: u32) -> Self {
    if CHOSEONG_BASE <= unicode && unicode <= CHOSEONG_LAST {
      let offset = unicode - CHOSEONG_BASE;
      let compatibility_jamo = COMPATIBILITY_CHOSEONG_MAPPING[offset as usize];

      return Self {
        conjoining_value: unsafe { std::char::from_u32_unchecked(unicode) },
        conjoining_unicode: unicode,
        compatibility_value: unsafe { std::char::from_u32_unchecked(compatibility_jamo) },
        compatibility_unicode: compatibility_jamo,
      };
    }

    if COMPAT_CHOSEONG_BASE <= unicode && unicode <= COMPAT_CHOSEONG_LAST {
      if let Some(position) = COMPATIBILITY_CHOSEONG_MAPPING
        .iter()
        .position(|&x| x == unicode)
      {
        let conjoining_jamo = CHOSEONG_BASE + position as u32;

        return Self {
          conjoining_value: unsafe { std::char::from_u32_unchecked(conjoining_jamo) },
          conjoining_unicode: conjoining_jamo,
          compatibility_value: unsafe { std::char::from_u32_unchecked(unicode) },
          compatibility_unicode: unicode,
        };
      }
    }

    panic!("유효한 초성 유니코드가 아닙니다: {}", unicode);
  }

  // 조합형 초성 확인(Conjoining Choseong)
  pub fn is_conjoining_choseong(choseong_code: u32) -> bool {
    CHOSEONG_BASE <= choseong_code && choseong_code <= CHOSEONG_LAST
  }

  // 호환형 초성 확인(Compatibility Choseong)
  pub fn is_compatibility_choseong(unicode: u32) -> bool {
    COMPAT_CHOSEONG_BASE <= unicode && unicode <= COMPAT_CHOSEONG_LAST
  }

  pub(crate) fn compatibility_index(ch: char) -> Option<usize> {
    COMPATIBILITY_CHOSEONG_MAPPING
      .iter()
      .position(|&unicode| unicode == ch as u32)
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_is_conjoining_choseong() {
    assert!(Choseong::is_conjoining_choseong(0x1100));
    assert!(Choseong::is_conjoining_choseong(0x1112));

    assert!(Choseong::is_conjoining_choseong(CHOSEONG_BASE));
    assert!(Choseong::is_conjoining_choseong(CHOSEONG_LAST));

    assert!(!Choseong::is_conjoining_choseong(CHOSEONG_BASE - 1));
    assert!(!Choseong::is_conjoining_choseong(CHOSEONG_LAST + 1));
    assert!(!Choseong::is_conjoining_choseong(0x1113));
  }

  #[test]
  fn test_is_compatibility_choseong() {
    assert!(Choseong::is_compatibility_choseong(0x3131));
    assert!(Choseong::is_compatibility_choseong(0x314E));

    assert!(Choseong::is_compatibility_choseong(COMPAT_CHOSEONG_BASE));
    assert!(Choseong::is_compatibility_choseong(COMPAT_CHOSEONG_LAST));

    assert!(!Choseong::is_compatibility_choseong(
      COMPAT_CHOSEONG_BASE - 1
    ));
    assert!(!Choseong::is_compatibility_choseong(
      COMPAT_CHOSEONG_LAST + 1
    ));
    assert!(!Choseong::is_compatibility_choseong(0x314F));
    assert!(!Choseong::is_compatibility_choseong(0xAC00));
  }

  #[test]
  fn test_choseong_construction_from_conjoining() {
    let choseong = Choseong::new(0x1100);
    assert_eq!(choseong.conjoining_unicode, 0x1100);
    assert_eq!(choseong.conjoining_value, 'ᄀ');
    assert_eq!(choseong.compatibility_unicode, 0x3131);
    assert_eq!(choseong.compatibility_value, 'ㄱ');

    let choseong = Choseong::new(0x1112);
    assert_eq!(choseong.conjoining_unicode, 0x1112);
    assert_eq!(choseong.conjoining_value, 'ᄒ');
    assert_eq!(choseong.compatibility_unicode, 0x314E);
    assert_eq!(choseong.compatibility_value, 'ㅎ');
  }

  #[test]
  fn test_choseong_construction_from_compatibility() {
    let choseong = Choseong::new(0x3131);
    assert_eq!(choseong.conjoining_unicode, 0x1100);
    assert_eq!(choseong.conjoining_value, 'ᄀ');
    assert_eq!(choseong.compatibility_unicode, 0x3131);
    assert_eq!(choseong.compatibility_value, 'ㄱ');

    let choseong = Choseong::new(0x314E);
    assert_eq!(choseong.conjoining_unicode, 0x1112);
    assert_eq!(choseong.conjoining_value, 'ᄒ');
    assert_eq!(choseong.compatibility_unicode, 0x314E);
    assert_eq!(choseong.compatibility_value, 'ㅎ');
  }

  #[test]
  #[should_panic(expected = "유효한 초성 유니코드가 아닙니다")]
  fn test_invalid_unicode_should_panic() {
    Choseong::new(0xAC00);
  }
}
