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
  requests: Vec<Value>,
  responses: Vec<Value>,
  service: Spawn<LspService>,
}

impl Test {
  fn new() -> Result<Self> {
    let (service, messages) = LspService::new(Server::new);

    Ok(Self {
      _messages: messages,
      requests: Vec::new(),
      responses: Vec::new(),
      service: Spawn::new(service),
    })
  }

  fn request(mut self, request: Value) -> Self {
    self.requests.push(request);
    self
  }

  fn response(mut self, response: Value) -> Self {
    self.responses.push(response);
    self
  }

  async fn run(mut self) -> Result {
    for (request, expected_response) in
      self.requests.iter().zip(self.responses.iter())
    {
      assert_eq!(
        *expected_response,
        self
          .service
          .call(serde_json::from_value(request.clone())?)
          .await?
          .map(|v| serde_json::to_value(v).unwrap())
          .unwrap()
      );
    }

    Ok(())
  }
}

#[tokio::test]
async fn initialize() -> Result {
  Test::new()?
    .request(json!({
      "jsonrpc": "2.0",
      "id": 1,
      "method": "initialize",
      "params": {
        "capabilities": {}
      },
    }))
    .response(json!({
      "jsonrpc": "2.0",
      "id": 1,
      "result": {
        "serverInfo": {
          "name": env!("CARGO_PKG_NAME"),
          "version": env!("CARGO_PKG_VERSION")
        },
        "capabilities": Server::capabilities()
      },
    }))
    .run()
    .await
}

#[tokio::test]
async fn initialize_once() -> Result {
  Test::new()?
    .request(json!({
      "jsonrpc": "2.0",
      "id": 1,
      "method": "initialize",
      "params": {
        "capabilities": {}
      },
    }))
    .response(json!({
      "jsonrpc": "2.0",
      "id": 1,
      "result": {
        "serverInfo": {
          "name": env!("CARGO_PKG_NAME"),
          "version": env!("CARGO_PKG_VERSION")
        },
        "capabilities": Server::capabilities()
      }
    }))
    .request(json!({
      "jsonrpc": "2.0",
      "id": 1,
      "method": "initialize",
      "params": {
        "capabilities": {}
      }
    }))
    .response(json!({
      "jsonrpc": "2.0",
      "id": 1,
      "error": {
        "code": -32600,
        "message": "Invalid request"
      }
    }))
    .run()
    .await
}
