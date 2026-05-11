use super::*;

#[derive(Debug)]
pub enum SettingKind {
  Array,
  Boolean(bool),
  String,
}

impl Display for SettingKind {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    match self {
      SettingKind::Array => write!(f, "array"),
      SettingKind::Boolean(_) => write!(f, "boolean"),
      SettingKind::String => write!(f, "string"),
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
    )
  }
}
