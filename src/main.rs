use crate::common::*;

mod common;
mod document;
mod server;

async fn run() -> Result {
  let (service, messages) = LspService::new(|client| Server::new(client));
  lspower::Server::new(tokio::io::stdin(), tokio::io::stdout())
    .interleave(messages)
    .serve(service)
    .await;
  Ok(())
}

#[tokio::main]
async fn main() {
  env_logger::init();
  if let Err(error) = run().await {
    println!("error: {error}");
    process::exit(1);
  }
}
