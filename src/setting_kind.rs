use super::*;

#[derive(Debug)]
pub enum SettingKind {
  Array,
  Boolean(bool),
  String,
  StringOrArray,
}

impl Display for SettingKind {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    match self {
      SettingKind::Array => write!(f, "array"),
      SettingKind::Boolean(_) => write!(f, "boolean"),
      SettingKind::String => write!(f, "string"),
      SettingKind::StringOrArray => write!(f, "string or array"),
    }
  }
}

impl PartialEq for SettingKind {
  fn eq(&self, other: &Self) -> bool {
    matches!(
      (self, other),
      (SettingKind::Array, SettingKind::Array)
        | (SettingKind::Boolean(_), SettingKind::Boolean(_))
        | (SettingKind::String, SettingKind::String)
        | (SettingKind::StringOrArray, SettingKind::StringOrArray)
    )
  }
}

impl SettingKind {
  #[must_use]
  pub fn accepts(&self, other: &Self) -> bool {
    matches!(
      (self, other),
      (SettingKind::Array, SettingKind::Array)
        | (SettingKind::Boolean(_), SettingKind::Boolean(_))
        | (SettingKind::String, SettingKind::String)
        | (
          SettingKind::StringOrArray,
          SettingKind::Array | SettingKind::String
        )
    )
  }
}
