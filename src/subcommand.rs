use {super::*, analyze::Analyze};

mod analyze;

#[derive(Clap)]
pub(crate) enum Subcommand {
  Analyze(Analyze),
}

impl Subcommand {
  fn find_justfile() -> Result<PathBuf> {
    let mut current_dir = env::current_dir()?;

    loop {
      let candidate = current_dir.join("justfile");

      if candidate.exists() {
        return Ok(candidate);
      }

      if !current_dir.pop() {
        bail!(
          "could not find `justfile` in current directory or any parent directory"
        );
      }
    }
  }

  pub(crate) fn run(self) -> Result {
    match self {
      Self::Analyze(analyze) => analyze.run(),
    }
  }
}
