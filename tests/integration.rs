use {
  just_lsp::server::Server,
  lspower::{LspService, MessageStream},
  pretty_assertions::assert_eq,
  serde_json::{json, Value},
  std::env,
  tower_test::mock::Spawn,
};

type Result<T = (), E = Box<dyn std::error::Error>> = std::result::Result<T, E>;

#[derive(Debug)]
struct Test {
  _messages: MessageStream,
  request: Value,
  response: Value,
  service: Spawn<LspService>,
}

impl Test {
  fn new() -> Result<Self> {
    let (service, messages) = LspService::new(Server::new);

    Ok(Self {
      _messages: messages,
      request: Value::default(),
      response: Value::default(),
      service: Spawn::new(service),
    })
  }

  fn request(self, request: Value) -> Self {
    Self { request, ..self }
  }

  fn response(self, response: Value) -> Self {
    Self { response, ..self }
  }

  async fn run(mut self) -> Result {
    let request = serde_json::from_value(self.request)?;

    let response = self
      .service
      .call(request)
      .await?
      .map(|v| serde_json::to_value(v).unwrap());

    assert_eq!(self.response, response.unwrap());

    Ok(())
  }
}

#[tokio::test]
async fn initialize() -> Result {
  Test::new()?
    .request(json!({
      "jsonrpc": "2.0",
      "method": "initialize",
      "params": {
        "capabilities": {}
      },
      "id": 1
    }))
    .response(json!({
      "jsonrpc": "2.0",
      "result": {
        "serverInfo": {
          "name": env!("CARGO_PKG_NAME"),
          "version": env!("CARGO_PKG_VERSION")
        },
        "capabilities": Server::capabilities()
      },
      "id": 1
    }))
    .run()
    .await
}
