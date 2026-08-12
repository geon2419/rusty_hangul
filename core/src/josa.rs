use std::error::Error;
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum JosaError {
  InvalidPair(String),
}

impl fmt::Display for JosaError {
  fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Self::InvalidPair(pair) => {
        write!(formatter, "unsupported josa pair: {pair}")
      }
    }
  }
}

impl Error for JosaError {}

#[derive(Clone, Copy)]
pub(crate) enum JosaPair {
  EulReul,
  Iga,
  EunNeun,
  WaGwa,
  EuroRo,
  IeyoYeyo,
}

impl JosaPair {
  pub(crate) fn parse(pair: &str) -> Option<Self> {
    match pair {
      "을/를" => Some(Self::EulReul),
      "이/가" => Some(Self::Iga),
      "은/는" => Some(Self::EunNeun),
      "와/과" => Some(Self::WaGwa),
      "으로/로" => Some(Self::EuroRo),
      "이에요/예요" => Some(Self::IeyoYeyo),
      _ => None,
    }
  }

  pub(crate) fn select(self, has_batchim: bool, has_rieul_batchim: bool) -> &'static str {
    match self {
      Self::EulReul => {
        if has_batchim {
          "을"
        } else {
          "를"
        }
      }
      Self::Iga => {
        if has_batchim {
          "이"
        } else {
          "가"
        }
      }
      Self::EunNeun => {
        if has_batchim {
          "은"
        } else {
          "는"
        }
      }
      Self::WaGwa => {
        if has_batchim {
          "과"
        } else {
          "와"
        }
      }
      Self::EuroRo => {
        if has_batchim && !has_rieul_batchim {
          "으로"
        } else {
          "로"
        }
      }
      Self::IeyoYeyo => {
        if has_batchim {
          "이에요"
        } else {
          "예요"
        }
      }
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_parse_supported_pairs() {
    assert!(JosaPair::parse("을/를").is_some());
    assert!(JosaPair::parse("이/가").is_some());
    assert!(JosaPair::parse("은/는").is_some());
    assert!(JosaPair::parse("와/과").is_some());
    assert!(JosaPair::parse("으로/로").is_some());
    assert!(JosaPair::parse("이에요/예요").is_some());
    assert!(JosaPair::parse("을").is_none());
  }

  #[test]
  fn test_select_pairs() {
    assert_eq!(JosaPair::EulReul.select(true, false), "을");
    assert_eq!(JosaPair::EulReul.select(false, false), "를");
    assert_eq!(JosaPair::Iga.select(true, false), "이");
    assert_eq!(JosaPair::Iga.select(false, false), "가");
    assert_eq!(JosaPair::EunNeun.select(true, false), "은");
    assert_eq!(JosaPair::EunNeun.select(false, false), "는");
    assert_eq!(JosaPair::WaGwa.select(true, false), "과");
    assert_eq!(JosaPair::WaGwa.select(false, false), "와");
    assert_eq!(JosaPair::EuroRo.select(true, false), "으로");
    assert_eq!(JosaPair::EuroRo.select(true, true), "로");
    assert_eq!(JosaPair::EuroRo.select(false, false), "로");
    assert_eq!(JosaPair::IeyoYeyo.select(true, false), "이에요");
    assert_eq!(JosaPair::IeyoYeyo.select(false, false), "예요");
  }
}
