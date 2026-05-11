use super::*;

#[derive(Parser)]
#[clap(
  author,
  version,
  about,
  long_about = None,
  styles = styling::Styles::styled()
    .header(styling::AnsiColor::Yellow.on_default().bold())
    .usage(styling::AnsiColor::Yellow.on_default().bold())
    .literal(styling::AnsiColor::Green.on_default().bold())
    .placeholder(styling::AnsiColor::Cyan.on_default()),
)]
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
