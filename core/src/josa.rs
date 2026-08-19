use std::error::Error;
use std::fmt;

/// Failed josa lookup.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum JosaError {
  /// `pair` is not a supported form such as `"을/를"`.
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
  AYa,
  InaNa,
  IranRan,
  IrangRang,
  ImyeoMyeo,
  IyaYa,
  IragoRago,
  IdeunDeun,
}

impl JosaPair {
  pub(crate) fn parse(pair: &str) -> Option<Self> {
    match pair {
      "을/를" | "를/을" => Some(Self::EulReul),
      "이/가" | "가/이" => Some(Self::Iga),
      "은/는" | "는/은" => Some(Self::EunNeun),
      "와/과" | "과/와" => Some(Self::WaGwa),
      "으로/로" | "로/으로" => Some(Self::EuroRo),
      "이에요/예요" | "예요/이에요" => Some(Self::IeyoYeyo),
      "아/야" | "야/아" => Some(Self::AYa),
      "이나/나" | "나/이나" => Some(Self::InaNa),
      "이란/란" | "란/이란" => Some(Self::IranRan),
      "이랑/랑" | "랑/이랑" => Some(Self::IrangRang),
      "이며/며" | "며/이며" => Some(Self::ImyeoMyeo),
      "이야/야" | "야/이야" => Some(Self::IyaYa),
      "이라고/라고" | "라고/이라고" => Some(Self::IragoRago),
      "이든/든" | "든/이든" => Some(Self::IdeunDeun),
      _ => None,
    }
  }

  pub(crate) fn select(self, has_batchim: bool, has_rieul_batchim: bool) -> &'static str {
    match self {
      Self::EulReul => Self::pick(has_batchim, "을", "를"),
      Self::Iga => Self::pick(has_batchim, "이", "가"),
      Self::EunNeun => Self::pick(has_batchim, "은", "는"),
      Self::WaGwa => Self::pick(has_batchim, "과", "와"),
      Self::EuroRo => {
        if has_batchim && !has_rieul_batchim {
          "으로"
        } else {
          "로"
        }
      }
      Self::IeyoYeyo => Self::pick(has_batchim, "이에요", "예요"),
      Self::AYa => Self::pick(has_batchim, "아", "야"),
      Self::InaNa => Self::pick(has_batchim, "이나", "나"),
      Self::IranRan => Self::pick(has_batchim, "이란", "란"),
      Self::IrangRang => Self::pick(has_batchim, "이랑", "랑"),
      Self::ImyeoMyeo => Self::pick(has_batchim, "이며", "며"),
      Self::IyaYa => Self::pick(has_batchim, "이야", "야"),
      Self::IragoRago => Self::pick(has_batchim, "이라고", "라고"),
      Self::IdeunDeun => Self::pick(has_batchim, "이든", "든"),
    }
  }

  fn pick(
    has_batchim: bool,
    with_batchim: &'static str,
    without_batchim: &'static str,
  ) -> &'static str {
    if has_batchim {
      with_batchim
    } else {
      without_batchim
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_parse_supported_pairs() {
    assert!(JosaPair::parse("을/를").is_some());
    assert!(JosaPair::parse("를/을").is_some());
    assert!(JosaPair::parse("이/가").is_some());
    assert!(JosaPair::parse("은/는").is_some());
    assert!(JosaPair::parse("와/과").is_some());
    assert!(JosaPair::parse("으로/로").is_some());
    assert!(JosaPair::parse("이에요/예요").is_some());
    assert!(JosaPair::parse("아/야").is_some());
    assert!(JosaPair::parse("이나/나").is_some());
    assert!(JosaPair::parse("이란/란").is_some());
    assert!(JosaPair::parse("이랑/랑").is_some());
    assert!(JosaPair::parse("이며/며").is_some());
    assert!(JosaPair::parse("이야/야").is_some());
    assert!(JosaPair::parse("이라고/라고").is_some());
    assert!(JosaPair::parse("이든/든").is_some());
    assert!(JosaPair::parse("을").is_none());
  }

  #[test]
  fn test_parse_aliases_select_the_same_particle() {
    let aliases = [
      ("을/를", "를/을"),
      ("이/가", "가/이"),
      ("은/는", "는/은"),
      ("와/과", "과/와"),
      ("으로/로", "로/으로"),
      ("이에요/예요", "예요/이에요"),
      ("아/야", "야/아"),
      ("이나/나", "나/이나"),
      ("이란/란", "란/이란"),
      ("이랑/랑", "랑/이랑"),
      ("이며/며", "며/이며"),
      ("이야/야", "야/이야"),
      ("이라고/라고", "라고/이라고"),
      ("이든/든", "든/이든"),
    ];

    for (forward, reverse) in aliases {
      let parsed_forward = JosaPair::parse(forward).unwrap();
      let parsed_reverse = JosaPair::parse(reverse).unwrap();

      for has_batchim in [true, false] {
        for has_rieul in [true, false] {
          assert_eq!(
            parsed_forward.select(has_batchim, has_rieul),
            parsed_reverse.select(has_batchim, has_rieul),
            "{forward} vs {reverse} for batchim={has_batchim}, rieul={has_rieul}"
          );
        }
      }
    }
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
    assert_eq!(JosaPair::AYa.select(true, false), "아");
    assert_eq!(JosaPair::AYa.select(false, false), "야");
    assert_eq!(JosaPair::InaNa.select(true, false), "이나");
    assert_eq!(JosaPair::InaNa.select(false, false), "나");
    assert_eq!(JosaPair::IranRan.select(true, false), "이란");
    assert_eq!(JosaPair::IranRan.select(false, false), "란");
    assert_eq!(JosaPair::IrangRang.select(true, false), "이랑");
    assert_eq!(JosaPair::IrangRang.select(false, false), "랑");
    assert_eq!(JosaPair::ImyeoMyeo.select(true, false), "이며");
    assert_eq!(JosaPair::ImyeoMyeo.select(false, false), "며");
    assert_eq!(JosaPair::IyaYa.select(true, false), "이야");
    assert_eq!(JosaPair::IyaYa.select(false, false), "야");
    assert_eq!(JosaPair::IragoRago.select(true, false), "이라고");
    assert_eq!(JosaPair::IragoRago.select(false, false), "라고");
    assert_eq!(JosaPair::IdeunDeun.select(true, false), "이든");
    assert_eq!(JosaPair::IdeunDeun.select(false, false), "든");
  }
}
