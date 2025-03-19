#[derive(Debug)]
pub(crate) enum SettingKind {
  Array,
  Boolean,
  String,
}

#[derive(Debug)]
pub(crate) struct Setting<'a> {
  pub(crate) name: &'a str,
  pub(crate) kind: SettingKind,
}
