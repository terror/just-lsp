use super::*;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Deprecation<'a> {
  BooleanSetting {
    attribute: &'a str,
    setting: &'a str,
    value: &'a str,
  },
  Replacement(&'a str),
  SettingAttribute {
    attribute: &'a str,
    setting: &'a str,
  },
}

impl Display for Deprecation<'_> {
  fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
    match self {
      Self::Replacement(replacement) => write!(f, "`{replacement}`"),
      Self::BooleanSetting {
        attribute, setting, ..
      }
      | Self::SettingAttribute { attribute, setting } => {
        write!(f, "`[{attribute}]` attribute on `set {setting}`")
      }
    }
  }
}
