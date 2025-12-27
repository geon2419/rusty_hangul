// 조합형 종성 범위
const JONGSEONG_BASE: u32 = 0x11A8;
const JONGSEONG_LAST: u32 = 0x11C2;

// 호환형 종성 범위
const COMPAT_JONGSEONG_BASE: u32 = 0x3131;
const COMPAT_JONGSEONG_LAST: u32 = 0x314E;

// 호환형 종성 매핑 테이블
const COMPATIBILITY_JONGSEONG_MAPPING: [u32; 27] = [
  0x3131, 0x3132, 0x3133, 0x3134, 0x3135, 0x3136, 0x3137, 0x3139, 0x313A, 0x313B, 0x313C, 0x313D,
  0x313E, 0x313F, 0x3140, 0x3141, 0x3142, 0x3144, 0x3145, 0x3146, 0x3147, 0x3148, 0x314A, 0x314B,
  0x314C, 0x314D, 0x314E,
];

// 복합 종성 매핑 테이블
const COMPLEX_JONGSEONG_MAPPING: [u32; 11] = [
  0x3133, 0x3135, 0x3136, 0x313A, 0x313B, 0x313C, 0x313D, 0x313E, 0x313F, 0x3140, 0x3144,
];

#[derive(Debug)]
pub struct Jongseong {
  pub conjoining_value: char,
  pub conjoining_unicode: u32,
  pub compatibility_value: char,
  pub compatibility_unicode: u32,
}

impl Jongseong {
  #[inline]
  pub fn new(unicode: u32) -> Self {
    // 조합형 종성 범위 확인
    if JONGSEONG_BASE <= unicode && unicode <= JONGSEONG_LAST {
      let offset = unicode - JONGSEONG_BASE;
      let compatibility_jamo = COMPATIBILITY_JONGSEONG_MAPPING[offset as usize];

      return Self {
        conjoining_value: unsafe { std::char::from_u32_unchecked(unicode) },
        conjoining_unicode: unicode,
        compatibility_value: unsafe { std::char::from_u32_unchecked(compatibility_jamo) },
        compatibility_unicode: compatibility_jamo,
      };
    }

    // 호환형 종성 범위 확인
    if COMPAT_JONGSEONG_BASE <= unicode && unicode <= COMPAT_JONGSEONG_LAST {
      if let Some(position) = COMPATIBILITY_JONGSEONG_MAPPING
        .iter()
        .position(|&x| x == unicode)
      {
        let conjoining_jamo = JONGSEONG_BASE + position as u32;

        return Self {
          conjoining_value: unsafe { std::char::from_u32_unchecked(conjoining_jamo) },
          conjoining_unicode: conjoining_jamo,
          compatibility_value: unsafe { std::char::from_u32_unchecked(unicode) },
          compatibility_unicode: unicode,
        };
      }
    }

    panic!("유효한 종성 유니코드가 아닙니다: {}", unicode)
  }

  #[inline]
  pub fn is_conjoining_jongseong(jongseong_code: u32) -> bool {
    JONGSEONG_BASE <= jongseong_code && jongseong_code <= JONGSEONG_LAST
  }

  #[inline]
  pub fn is_compatibility_jongseong(unicode: u32) -> bool {
    COMPAT_JONGSEONG_BASE <= unicode && unicode <= COMPAT_JONGSEONG_LAST
  }

  #[inline]
  pub fn is_complex_jongseong(&self) -> bool {
    COMPLEX_JONGSEONG_MAPPING.contains(&self.compatibility_unicode)
  }

  #[inline]
  pub fn decompose_complex_jongseong(&self) -> Vec<char> {
    match self.compatibility_unicode {
      0x3133 => vec!['ㄱ', 'ㅅ'],
      0x3135 => vec!['ㄴ', 'ㅈ'],
      0x3136 => vec!['ㄴ', 'ㅎ'],
      0x313A => vec!['ㄹ', 'ㄱ'],
      0x313B => vec!['ㄹ', 'ㅁ'],
      0x313C => vec!['ㄹ', 'ㅂ'],
      0x313D => vec!['ㄹ', 'ㅅ'],
      0x313E => vec!['ㄹ', 'ㅌ'],
      0x313F => vec!['ㄹ', 'ㅍ'],
      0x3140 => vec!['ㄹ', 'ㅎ'],
      0x3144 => vec!['ㅂ', 'ㅅ'],
      _ => vec![self.compatibility_value],
    }
  }

  #[inline]
  pub(crate) fn append_decomposed_compatibility(output: &mut String, compatibility_unicode: u32) {
    match compatibility_unicode {
      0x3133 => {
        output.push('ㄱ');
        output.push('ㅅ');
      }
      0x3135 => {
        output.push('ㄴ');
        output.push('ㅈ');
      }
      0x3136 => {
        output.push('ㄴ');
        output.push('ㅎ');
      }
      0x313A => {
        output.push('ㄹ');
        output.push('ㄱ');
      }
      0x313B => {
        output.push('ㄹ');
        output.push('ㅁ');
      }
      0x313C => {
        output.push('ㄹ');
        output.push('ㅂ');
      }
      0x313D => {
        output.push('ㄹ');
        output.push('ㅅ');
      }
      0x313E => {
        output.push('ㄹ');
        output.push('ㅌ');
      }
      0x313F => {
        output.push('ㄹ');
        output.push('ㅍ');
      }
      0x3140 => {
        output.push('ㄹ');
        output.push('ㅎ');
      }
      0x3144 => {
        output.push('ㅂ');
        output.push('ㅅ');
      }
      _ => output.push(unsafe { std::char::from_u32_unchecked(compatibility_unicode) }),
    }
  }

  #[inline]
  pub fn append_disassembled(&self, output: &mut String) {
    Self::append_decomposed_compatibility(output, self.compatibility_unicode);
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_is_conjoining_jongseong() {
    assert!(Jongseong::is_conjoining_jongseong(0x11A8));
    assert!(Jongseong::is_conjoining_jongseong(0x11B8));
    assert!(Jongseong::is_conjoining_jongseong(0x11C2));

    assert!(!Jongseong::is_conjoining_jongseong(0x11A7));
    assert!(!Jongseong::is_conjoining_jongseong(0x11C3));
    assert!(!Jongseong::is_conjoining_jongseong(0x3131));
  }

  #[test]
  fn test_is_compatibility_jongseong() {
    assert!(Jongseong::is_compatibility_jongseong(0x3131));
    assert!(Jongseong::is_compatibility_jongseong(0x3137));
    assert!(Jongseong::is_compatibility_jongseong(0x314E));

    assert!(!Jongseong::is_compatibility_jongseong(0x3130));
    assert!(!Jongseong::is_compatibility_jongseong(0x314F));
    assert!(!Jongseong::is_compatibility_jongseong(0x11A8));
  }

  #[test]
  fn test_new_from_conjoining_jamo() {
    let jongseong = Jongseong::new(0x11A8);
    assert_eq!(jongseong.conjoining_unicode, 0x11A8);
    assert_eq!(jongseong.conjoining_value, 'ᆨ');
    assert_eq!(jongseong.compatibility_unicode, 0x3131);
    assert_eq!(jongseong.compatibility_value, 'ㄱ');

    let jongseong = Jongseong::new(0x11AC);
    assert_eq!(jongseong.conjoining_unicode, 0x11AC);
    assert_eq!(jongseong.conjoining_value, 'ᆬ');
    assert_eq!(jongseong.compatibility_unicode, 0x3135);
    assert_eq!(jongseong.compatibility_value, 'ㄵ');
  }

  #[test]
  fn test_new_from_compatibility_jamo() {
    let jongseong = Jongseong::new(0x3131);
    assert_eq!(jongseong.conjoining_unicode, 0x11A8);
    assert_eq!(jongseong.conjoining_value, 'ᆨ');
    assert_eq!(jongseong.compatibility_unicode, 0x3131);
    assert_eq!(jongseong.compatibility_value, 'ㄱ');

    let jongseong = Jongseong::new(0x3135);
    assert_eq!(jongseong.conjoining_unicode, 0x11AC);
    assert_eq!(jongseong.conjoining_value, 'ᆬ');
    assert_eq!(jongseong.compatibility_unicode, 0x3135);
    assert_eq!(jongseong.compatibility_value, 'ㄵ');
  }

  #[test]
  fn test_is_complex_jongseong() {
    assert!(Jongseong::new(0x3133).is_complex_jongseong());
    assert!(Jongseong::new(0x3135).is_complex_jongseong());
    assert!(Jongseong::new(0x3136).is_complex_jongseong());
    assert!(Jongseong::new(0x313A).is_complex_jongseong());
    assert!(Jongseong::new(0x313B).is_complex_jongseong());
    assert!(Jongseong::new(0x313C).is_complex_jongseong());
    assert!(Jongseong::new(0x313D).is_complex_jongseong());
    assert!(Jongseong::new(0x313E).is_complex_jongseong());
    assert!(Jongseong::new(0x313F).is_complex_jongseong());
    assert!(Jongseong::new(0x3140).is_complex_jongseong());
    assert!(Jongseong::new(0x3144).is_complex_jongseong());

    assert!(!Jongseong::new(0x3131).is_complex_jongseong());
    assert!(!Jongseong::new(0x3134).is_complex_jongseong());
    assert!(!Jongseong::new(0x3139).is_complex_jongseong());
  }

  #[test]
  fn test_decompose_complex_jongseong() {
    assert_eq!(
      Jongseong::new(0x3133).decompose_complex_jongseong(),
      vec!['ㄱ', 'ㅅ']
    );
    assert_eq!(
      Jongseong::new(0x3135).decompose_complex_jongseong(),
      vec!['ㄴ', 'ㅈ']
    );
    assert_eq!(
      Jongseong::new(0x3136).decompose_complex_jongseong(),
      vec!['ㄴ', 'ㅎ']
    );
    assert_eq!(
      Jongseong::new(0x313A).decompose_complex_jongseong(),
      vec!['ㄹ', 'ㄱ']
    );
    assert_eq!(
      Jongseong::new(0x313B).decompose_complex_jongseong(),
      vec!['ㄹ', 'ㅁ']
    );
    assert_eq!(
      Jongseong::new(0x313C).decompose_complex_jongseong(),
      vec!['ㄹ', 'ㅂ']
    );
    assert_eq!(
      Jongseong::new(0x313D).decompose_complex_jongseong(),
      vec!['ㄹ', 'ㅅ']
    );
    assert_eq!(
      Jongseong::new(0x313E).decompose_complex_jongseong(),
      vec!['ㄹ', 'ㅌ']
    );
    assert_eq!(
      Jongseong::new(0x313F).decompose_complex_jongseong(),
      vec!['ㄹ', 'ㅍ']
    );
    assert_eq!(
      Jongseong::new(0x3140).decompose_complex_jongseong(),
      vec!['ㄹ', 'ㅎ']
    );
    assert_eq!(
      Jongseong::new(0x3144).decompose_complex_jongseong(),
      vec!['ㅂ', 'ㅅ']
    );

    assert_eq!(
      Jongseong::new(0x3131).decompose_complex_jongseong(),
      vec!['ㄱ']
    );
    assert_eq!(
      Jongseong::new(0x3134).decompose_complex_jongseong(),
      vec!['ㄴ']
    );
    assert_eq!(
      Jongseong::new(0x3139).decompose_complex_jongseong(),
      vec!['ㄹ']
    );
  }

  #[test]
  #[should_panic(expected = "유효한 종성 유니코드가 아닙니다")]
  fn test_invalid_unicode() {
    Jongseong::new(0x1100);
  }
}
