use super::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Command {
  RunRecipe,
}

impl Command {
  pub(crate) fn all() -> Vec<Command> {
    vec![Command::RunRecipe]
  }
}

impl Display for Command {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    write!(f, "{}", self)
  }
}

impl Into<&str> for Command {
  fn into(self) -> &'static str {
    match self {
      Command::RunRecipe => "just-lsp.run_recipe",
    }
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
