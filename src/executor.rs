use super::*;

pub(crate) struct Executor {
  client: Client,
}

impl Executor {
  pub(crate) async fn execute(&self, params: lsp::ExecuteCommandParams) {
    if let Err(error) = self.try_execute(params).await {
      self
        .client
        .show_message(lsp::MessageType::ERROR, error)
        .await;
    }
  }

  pub(crate) fn new(client: Client) -> Self {
    Self { client }
  }

  async fn run_recipe(
    &self,
    recipe_name: &str,
    recipe_arguments: Vec<String>,
    directory: PathBuf,
  ) {
    let document_uri = lsp::Url::parse(&format!(
      "just-recipe:/{}/{}",
      directory.display(),
      recipe_name
    ))
    .unwrap_or_else(|_| lsp::Url::parse("just-recipe:/output").unwrap());

    let mut command = tokio::process::Command::new("just");

    command.arg(recipe_name);

    for argument in recipe_arguments {
      command.arg(argument);
    }

    command
      .current_dir(directory.clone())
      .stdout(process::Stdio::piped())
      .stderr(process::Stdio::piped());

    let client = self.client.clone();

    client
      .show_document(lsp::ShowDocumentParams {
        uri: document_uri.clone(),
        external: Some(false),
        take_focus: Some(true),
        selection: None,
      })
      .await
      .ok();

    let changes = HashMap::from([(
      document_uri.clone(),
      vec![lsp::TextEdit {
        range: lsp::Range::at(0, 0, u32::MAX, 0),
        new_text: String::new(),
      }],
    )]);

    client
      .apply_edit(lsp::WorkspaceEdit {
        changes: Some(changes),
        ..Default::default()
      })
      .await
      .ok();

    let recipe_name = recipe_name.to_string();

    tokio::spawn(async move {
      match command.spawn() {
        Ok(mut child) => {
          let stdout_lines = LinesStream::new(
            tokio::io::BufReader::new(
              child.stdout.take().expect("Failed to capture stdout"),
            )
            .lines(),
          );

          let stderr_lines = LinesStream::new(
            tokio::io::BufReader::new(
              child.stderr.take().expect("Failed to capture stderr"),
            )
            .lines(),
          );

          let mut merged_stream = StreamExt::merge(stdout_lines, stderr_lines);

          let mut buffer = String::new();
          let mut current_line = 0;
          let mut last_update = Instant::now();

          while let Some(line_result) = merged_stream.next().await {
            match line_result {
              Ok(line) => {
                buffer.push_str(&line);

                buffer.push('\n');

                let now = Instant::now();

                if (now.duration_since(last_update).as_millis() > 50
                  || buffer.len() > 1024)
                  && !buffer.is_empty()
                {
                  let changes = HashMap::from([(
                    document_uri.clone(),
                    vec![lsp::TextEdit {
                      range: lsp::Range::at(current_line, 0, current_line, 0),
                      new_text: buffer.trim().into(),
                    }],
                  )]);

                  client
                    .apply_edit(lsp::WorkspaceEdit {
                      changes: Some(changes),
                      ..Default::default()
                    })
                    .await
                    .ok();

                  let newlines = u32::try_from(buffer.matches('\n').count())
                    .expect("line count exceeds u32::MAX");

                  current_line += newlines;
                  buffer.clear();
                  last_update = now;
                }
              }
              Err(error) => {
                buffer.push_str("Error reading output: ");
                buffer.push_str(&error.to_string());
                buffer.push('\n');
              }
            }
          }

          if !buffer.is_empty() {
            let changes = HashMap::from([(
              document_uri.clone(),
              vec![lsp::TextEdit {
                range: lsp::Range::at(current_line, 0, current_line, 0),
                new_text: buffer.trim().into(),
              }],
            )]);

            client
              .apply_edit(lsp::WorkspaceEdit {
                changes: Some(changes),
                ..Default::default()
              })
              .await
              .ok();
          }

          match child.wait().await {
            Ok(status) => {
              if !status.success() {
                client
                  .show_message(
                    lsp::MessageType::WARNING,
                    format!("Recipe '{recipe_name}' completed with non-zero exit code: {status}"),
                  )
                  .await;
              }
            }
            Err(error) => {
              client
                .show_message(
                  lsp::MessageType::ERROR,
                  format!("Error waiting for recipe '{recipe_name}': {error}"),
                )
                .await;
            }
          }
        }
        Err(error) => {
          client
            .show_message(
              lsp::MessageType::ERROR,
              format!("Failed to run recipe '{recipe_name}': {error}"),
            )
            .await;
        }
      }
    });
  }

  async fn try_execute(&self, params: lsp::ExecuteCommandParams) -> Result {
    match Command::try_from(params.command.as_str())? {
      Command::RunRecipe => {
        let (recipe_name, uri, parameters) =
          serde_json::from_value::<(String, lsp::Url, Vec<ParameterJson>)>(
            Value::Array(params.arguments),
          )?;

        let directory = uri
          .to_file_path()
          .ok()
          .and_then(|path| path.parent().map(std::path::Path::to_path_buf))
          .unwrap_or_default();

        if !parameters.is_empty() {
          self.client.show_message(
            lsp::MessageType::WARNING,
            "Running a recipe code action with parameters is not yet supported."
          )
          .await;

          return Ok(());
        }

        self.run_recipe(&recipe_name, Vec::new(), directory).await;
      }
    }

    Ok(())
  }
}
