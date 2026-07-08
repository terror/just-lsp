use super::*;

pub struct Quickfixer<'a> {
  config: Option<&'a Config>,
  document: &'a Document,
  parameters: &'a lsp::CodeActionParams,
}

impl<'a> Quickfixer<'a> {
  fn action(&self, code: &str, quickfix: Quickfix) -> lsp::CodeActionOrCommand {
    let diagnostics = self
      .parameters
      .context
      .diagnostics
      .iter()
      .filter(|diagnostic| {
        diagnostic.range == quickfix.range
          && matches!(
            &diagnostic.code,
            Some(lsp::NumberOrString::String(c)) if c == code
          )
      })
      .cloned()
      .collect::<Vec<_>>();

    lsp::CodeActionOrCommand::CodeAction(lsp::CodeAction {
      title: quickfix.title,
      kind: Some(lsp::CodeActionKind::QUICKFIX),
      diagnostics: (!diagnostics.is_empty()).then_some(diagnostics),
      edit: Some(lsp::WorkspaceEdit {
        changes: Some(HashMap::from([(
          self.parameters.text_document.uri.clone(),
          quickfix.edits,
        )])),
        ..Default::default()
      }),
      ..Default::default()
    })
  }

  #[must_use]
  pub fn collect(&self) -> Vec<lsp::CodeActionOrCommand> {
    let context = RuleContext::new(self.document);

    inventory::iter::<&dyn Rule>
      .into_iter()
      .filter(|rule| {
        self
          .config
          .unwrap_or(&Config::default())
          .rule_config(rule.id())
          .level()
          != Some(RuleLevel::Off)
      })
      .flat_map(|rule| {
        rule
          .quickfixes(&context)
          .into_iter()
          .filter(|quickfix| quickfix.range.overlaps(self.parameters.range))
          .map(|quickfix| self.action(rule.id(), quickfix))
      })
      .collect()
  }

  #[must_use]
  pub fn config(self, config: &'a Config) -> Self {
    Self {
      config: Some(config),
      ..self
    }
  }

  #[must_use]
  pub fn new(
    document: &'a Document,
    parameters: &'a lsp::CodeActionParams,
  ) -> Self {
    Self {
      config: None,
      document,
      parameters,
    }
  }
}

#[cfg(test)]
mod tests {
  use {super::*, pretty_assertions::assert_eq};

  #[derive(Debug)]
  struct Test {
    config: Config,
    document: Document,
    quickfixes: Vec<Quickfix>,
    range: lsp::Range,
  }

  impl Test {
    fn config(self, config: Config) -> Self {
      Self { config, ..self }
    }

    fn new(content: &str) -> Self {
      Self {
        config: Config::default(),
        document: Document::from(content),
        quickfixes: Vec::new(),
        range: lsp::Range::at(0, 0, 0, 0),
      }
    }

    fn quickfix(self, quickfix: Quickfix) -> Self {
      Self {
        quickfixes: self.quickfixes.into_iter().chain([quickfix]).collect(),
        ..self
      }
    }

    fn range(self, range: lsp::Range) -> Self {
      Self { range, ..self }
    }

    fn run(self) {
      let Test {
        config,
        document,
        quickfixes,
        range,
      } = self;

      let parameters = lsp::CodeActionParams {
        text_document: lsp::TextDocumentIdentifier {
          uri: document.uri.clone(),
        },
        range,
        context: lsp::CodeActionContext {
          diagnostics: Vec::new(),
          ..Default::default()
        },
        work_done_progress_params: lsp::WorkDoneProgressParams::default(),
        partial_result_params: lsp::PartialResultParams::default(),
      };

      let actions = Quickfixer::new(&document, &parameters)
        .config(&config)
        .collect();

      assert_eq!(actions.len(), quickfixes.len());

      for (action, quickfix) in actions.into_iter().zip(quickfixes) {
        let lsp::CodeActionOrCommand::CodeAction(action) = action else {
          unreachable!("expected CodeAction");
        };

        assert_eq!(
          action,
          lsp::CodeAction {
            title: quickfix.title,
            kind: Some(lsp::CodeActionKind::QUICKFIX),
            diagnostics: None,
            edit: Some(lsp::WorkspaceEdit {
              changes: Some(HashMap::from([(
                document.uri.clone(),
                quickfix.edits,
              )])),
              ..Default::default()
            }),
            ..Default::default()
          }
        );
      }
    }
  }

  #[test]
  fn filters_multiple_calls_by_range() {
    Test::new(
      "foo := env_var(\"A\")\nbar := env_var_or_default(\"B\", \"C\")\n",
    )
    .range(lsp::Range::at(0, 10, 0, 10))
    .quickfix(Quickfix {
      edits: vec![lsp::TextEdit {
        range: lsp::Range::at(0, 7, 0, 14),
        new_text: "env".to_string(),
      }],
      range: lsp::Range::at(0, 7, 0, 14),
      title: "Replace `env_var` with `env`".to_string(),
    })
    .run();
  }

  #[test]
  fn ignores_setting_outside_range() {
    Test::new("set windows-powershell := true\nset export := true\n")
      .range(lsp::Range::at(1, 4, 1, 4))
      .run();
  }

  #[test]
  fn removes_parallel_attribute() {
    Test::new("[parallel]\nfoo: bar\nbar:\n")
      .range(lsp::Range::at(0, 0, 1, 0))
      .quickfix(Quickfix {
        edits: vec![lsp::TextEdit {
          range: lsp::Range::at(0, 0, 1, 0),
          new_text: String::new(),
        }],
        range: lsp::Range::at(0, 0, 1, 0),
        title: "Remove `[parallel]`".to_string(),
      })
      .run();
  }

  #[test]
  fn replaces_deprecated_setting() {
    Test::new("set windows-powershell := true\n")
      .range(lsp::Range::at(0, 4, 0, 4))
      .quickfix(Quickfix {
        edits: vec![lsp::TextEdit {
          range: lsp::Range::at(0, 4, 0, 22),
          new_text: "windows-shell".to_string(),
        }],
        range: lsp::Range::at(0, 4, 0, 22),
        title: "Replace `windows-powershell` with `windows-shell`".to_string(),
      })
      .run();
  }

  #[test]
  fn skips_non_rename_deprecated_setting() {
    Test::new(
      "set windows-shell := [\"powershell.exe\", \"-NoLogo\", \"-Command\"]\n",
    )
    .range(lsp::Range::at(0, 4, 0, 4))
    .run();
  }

  #[test]
  fn skips_disabled_rules() {
    let config = serde_json::from_value::<Config>(serde_json::json!({
      "rules": {
        "deprecated-function": "off"
      }
    }))
    .unwrap();

    Test::new("foo := env_var(\"A\")\n")
      .config(config)
      .range(lsp::Range::at(0, 10, 0, 10))
      .run();
  }
}
