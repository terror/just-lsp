use super::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Command {
  RunRecipe,
}

impl Command {
  pub(crate) fn all() -> Vec<String> {
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

  fn try_from(value: &str) -> Result<Self, Self::Error> {
    match value {
      "just-lsp.run_recipe" => Ok(Command::RunRecipe),
      _ => Err(anyhow!("Unknown command: {}", value)),
    }
  }
}
