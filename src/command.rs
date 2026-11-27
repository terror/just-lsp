use super::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Command {
  RunRecipe,
}

impl Command {
  #[must_use]
  pub fn all() -> Vec<String> {
    vec![Command::RunRecipe.to_string()]
  }
}

impl Display for Command {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    write!(
      f,
      "{}",
      match self {
        Command::RunRecipe => "just-lsp.run_recipe",
      }
    )
  }
}

impl TryFrom<&str> for Command {
  type Error = Error;

  fn try_from(value: &str) -> Result<Self> {
    match value {
      "just-lsp.run_recipe" => Ok(Command::RunRecipe),
      _ => Err(Error::UnknownCommand(value.into())),
    }
  }
}
