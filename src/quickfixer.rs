use super::*;

pub struct Quickfixer<'a> {
  config: Option<&'a Config>,
  document: &'a Document,
  parameters: &'a lsp::CodeActionParams,
}

impl<'a> Quickfixer<'a> {
  fn action(&self, code: &str, quickfix: Quickfix) -> lsp::CodeActionOrCommand {
    let diagnostics = self.matching_diagnostics(quickfix.range, code);

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

    let default = Config::default();

    let config = self.config.unwrap_or(&default);

    inventory::iter::<&dyn Rule>
      .into_iter()
      .filter(|rule| {
        config.rule_config(rule.id()).level() != Some(RuleLevel::Off)
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

  fn matching_diagnostics(
    &self,
    range: lsp::Range,
    code: &str,
  ) -> Vec<lsp::Diagnostic> {
    self
      .parameters
      .context
      .diagnostics
      .iter()
      .filter(|diagnostic| {
        diagnostic.range == range
          && matches!(
            &diagnostic.code,
            Some(lsp::NumberOrString::String(c)) if c == code
          )
      })
      .cloned()
      .collect()
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

  #[test]
  fn collect_filters_multiple_calls_by_range() {
    let document = Document::from(
      "foo := env_var(\"A\")\nbar := env_var_or_default(\"B\", \"C\")\n",
    );

    let parameters = lsp::CodeActionParams {
      text_document: lsp::TextDocumentIdentifier {
        uri: lsp::Url::parse("file:///test.just").unwrap(),
      },
      range: lsp::Range::at(0, 10, 0, 10),
      context: lsp::CodeActionContext {
        diagnostics: vec![],
        ..Default::default()
      },
      work_done_progress_params: lsp::WorkDoneProgressParams::default(),
      partial_result_params: lsp::PartialResultParams::default(),
    };

    let actions = Quickfixer::new(&document, &parameters).collect();

    assert_eq!(actions.len(), 1);

    let lsp::CodeActionOrCommand::CodeAction(action) = &actions[0] else {
      unreachable!("expected CodeAction");
    };

    assert_eq!(action.title, "Replace `env_var` with `env`");
  }

  #[test]
  fn collect_ignores_setting_outside_range() {
    let document =
      Document::from("set windows-powershell := true\nset export := true\n");

    let parameters = lsp::CodeActionParams {
      text_document: lsp::TextDocumentIdentifier {
        uri: lsp::Url::parse("file:///test.just").unwrap(),
      },
      range: lsp::Range::at(1, 4, 1, 4),
      context: lsp::CodeActionContext {
        diagnostics: vec![],
        ..Default::default()
      },
      work_done_progress_params: lsp::WorkDoneProgressParams::default(),
      partial_result_params: lsp::PartialResultParams::default(),
    };

    assert_eq!(Quickfixer::new(&document, &parameters).collect(), vec![]);
  }

  #[test]
  fn collect_replaces_deprecated_setting() {
    let document = Document::from("set windows-powershell := true\n");

    let parameters = lsp::CodeActionParams {
      text_document: lsp::TextDocumentIdentifier {
        uri: lsp::Url::parse("file:///test.just").unwrap(),
      },
      range: lsp::Range::at(0, 4, 0, 4),
      context: lsp::CodeActionContext {
        diagnostics: vec![],
        ..Default::default()
      },
      work_done_progress_params: lsp::WorkDoneProgressParams::default(),
      partial_result_params: lsp::PartialResultParams::default(),
    };

    let actions = Quickfixer::new(&document, &parameters).collect();

    assert_eq!(actions.len(), 1);

    let lsp::CodeActionOrCommand::CodeAction(action) = &actions[0] else {
      unreachable!("expected CodeAction");
    };

    assert_eq!(
      action.title,
      "Replace `windows-powershell` with `windows-shell`"
    );

    assert_eq!(
      action.edit,
      Some(lsp::WorkspaceEdit {
        changes: Some(HashMap::from([(
          lsp::Url::parse("file:///test.just").unwrap(),
          vec![lsp::TextEdit {
            range: lsp::Range::at(0, 4, 0, 22),
            new_text: "windows-shell".to_string(),
          }],
        )])),
        ..Default::default()
      }),
    );
  }

  #[test]
  fn collect_skips_disabled_rules() {
    let config = serde_json::from_value::<Config>(serde_json::json!({
      "rules": {
        "deprecated-function": "off"
      }
    }))
    .unwrap();

    let document = Document::from("foo := env_var(\"A\")\n");

    let parameters = lsp::CodeActionParams {
      text_document: lsp::TextDocumentIdentifier {
        uri: lsp::Url::parse("file:///test.just").unwrap(),
      },
      range: lsp::Range::at(0, 10, 0, 10),
      context: lsp::CodeActionContext {
        diagnostics: vec![],
        ..Default::default()
      },
      work_done_progress_params: lsp::WorkDoneProgressParams::default(),
      partial_result_params: lsp::PartialResultParams::default(),
    };

    let actions = Quickfixer::new(&document, &parameters)
      .config(&config)
      .collect();

    assert_eq!(actions, vec![]);
  }

  #[test]
  fn matching_diagnostics_filters_by_code_and_range() {
    let document = Document::from("foo := env_var(\"A\")\n");

    let target = lsp::Range::at(0, 7, 0, 14);

    let diagnostic = lsp::Diagnostic {
      range: target,
      code: Some(lsp::NumberOrString::String(
        "deprecated-function".to_string(),
      )),
      ..Default::default()
    };

    let parameters = lsp::CodeActionParams {
      text_document: lsp::TextDocumentIdentifier {
        uri: lsp::Url::parse("file:///test.just").unwrap(),
      },
      range: target,
      context: lsp::CodeActionContext {
        diagnostics: vec![
          diagnostic.clone(),
          lsp::Diagnostic {
            range: target,
            code: Some(lsp::NumberOrString::String("other-rule".to_string())),
            ..Default::default()
          },
          lsp::Diagnostic {
            range: lsp::Range::at(1, 0, 1, 5),
            code: Some(lsp::NumberOrString::String(
              "deprecated-function".to_string(),
            )),
            ..Default::default()
          },
        ],
        ..Default::default()
      },
      work_done_progress_params: lsp::WorkDoneProgressParams::default(),
      partial_result_params: lsp::PartialResultParams::default(),
    };

    let diagnostics = Quickfixer::new(&document, &parameters)
      .matching_diagnostics(target, "deprecated-function");

    assert_eq!(diagnostics, vec![diagnostic]);
  }
}
