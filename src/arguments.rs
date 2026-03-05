use super::*;

#[derive(Clap)]
#[clap(author, version, about, long_about = None)]
pub(crate) struct Arguments {
  #[clap(subcommand)]
  subcommand: Option<Subcommand>,
}

impl Arguments {
  pub(crate) async fn run(self) -> Result {
    match self.subcommand {
      Some(subcommand) => subcommand.run(),
      None => Server::run().await,
    }
  }
}
