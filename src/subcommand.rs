use {super::*, analyze::Analyze};

mod analyze;

#[derive(Clap)]
pub(crate) enum Subcommand {
  Analyze(Analyze),
}

impl Subcommand {
  pub(crate) fn run(self) -> Result {
    match self {
      Self::Analyze(analyze) => analyze.run(),
    }
  }
}
