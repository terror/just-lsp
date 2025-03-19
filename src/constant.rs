#[derive(Debug)]
pub(crate) struct Constant<'a> {
  pub(crate) name: &'a str,
  pub(crate) description: &'a str,
}
