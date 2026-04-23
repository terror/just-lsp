use super::*;

pub struct Quickfixer<'a> {
  document: &'a Document,
  parameters: &'a lsp::CodeActionParams,
}

impl<'a> Quickfixer<'a> {
  #[must_use]
  pub fn collect(&self) -> Vec<lsp::CodeActionOrCommand> {
    self
      .deprecated_replacements()
      .into_iter()
      .filter(|(name, _, _)| name.range.overlaps(self.parameters.range))
      .map(|(name, code, replacement)| {
        self.replacement_action(&name, code, replacement)
      })
      .collect()
  }

  fn deprecated_replacements(
    &self,
  ) -> Vec<(TextNode, &'static str, &'static str)> {
    let functions =
      self
        .document
        .function_calls()
        .into_iter()
        .filter_map(|call| {
          let replacement =
            BUILTINS.iter().find_map(|builtin| match builtin {
              Builtin::Function {
                name,
                deprecated: Some(replacement),
                ..
              } if *name == call.name.value => Some(*replacement),
              _ => None,
            })?;

          Some((call.name, "deprecated-function", replacement))
        });

    let settings = self.document.settings().into_iter().filter_map(|setting| {
      let replacement = BUILTINS.iter().find_map(|builtin| match builtin {
        Builtin::Setting {
          name,
          deprecated: Some(replacement),
          ..
        } if *name == setting.name.value => Some(*replacement),
        _ => None,
      })?;

      Some((setting.name, "deprecated-setting", replacement))
    });

    functions.chain(settings).collect()
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
      document,
      parameters,
    }
  }

  fn replacement_action(
    &self,
    name: &TextNode,
    code: &str,
    replacement: &str,
  ) -> lsp::CodeActionOrCommand {
    let diagnostics = self.matching_diagnostics(name.range, code);

    lsp::CodeActionOrCommand::CodeAction(lsp::CodeAction {
      title: format!("Replace `{}` with `{}`", name.value, replacement),
      kind: Some(lsp::CodeActionKind::QUICKFIX),
      diagnostics: (!diagnostics.is_empty()).then_some(diagnostics),
      edit: Some(lsp::WorkspaceEdit {
        changes: Some(HashMap::from([(
          self.parameters.text_document.uri.clone(),
          vec![lsp::TextEdit {
            range: name.range,
            new_text: replacement.to_string(),
          }],
        )])),
        ..Default::default()
      }),
      ..Default::default()
    })
  }
}

#[cfg(test)]
mod tests {
  use {super::*, pretty_assertions::assert_eq};

  fn diagnostic(range: lsp::Range, code: &str) -> lsp::Diagnostic {
    lsp::Diagnostic {
      range,
      code: Some(lsp::NumberOrString::String(code.to_string())),
      ..Default::default()
    }
  }

  fn parameters(
    range: lsp::Range,
    diagnostics: Vec<lsp::Diagnostic>,
  ) -> lsp::CodeActionParams {
    lsp::CodeActionParams {
      text_document: lsp::TextDocumentIdentifier {
        uri: lsp::Url::parse("file:///test.just").unwrap(),
      },
      range,
      context: lsp::CodeActionContext {
        diagnostics,
        ..Default::default()
      },
      work_done_progress_params: lsp::WorkDoneProgressParams::default(),
      partial_result_params: lsp::PartialResultParams::default(),
    }
  }

  #[test]
  fn collect_filters_multiple_calls_by_range() {
    let document = Document::from(
      "foo := env_var(\"A\")\nbar := env_var_or_default(\"B\", \"C\")\n",
    );

    let actions = Quickfixer::new(
      &document,
      &parameters(lsp::Range::at(0, 10, 0, 10), vec![]),
    )
    .collect();

    assert_eq!(actions.len(), 1);

    let lsp::CodeActionOrCommand::CodeAction(action) = &actions[0] else {
      unreachable!("expected CodeAction");
    };

    assert_eq!(action.title, "Replace `env_var` with `env`");
  }

  #[test]
  fn collect_replaces_deprecated_setting() {
    let document = Document::from("set windows-powershell := true\n");

    let actions = Quickfixer::new(
      &document,
      &parameters(lsp::Range::at(0, 4, 0, 4), vec![]),
    )
    .collect();

    assert_eq!(actions.len(), 1);

    let lsp::CodeActionOrCommand::CodeAction(action) = &actions[0] else {
      unreachable!("expected CodeAction");
    };

    assert_eq!(
      action.title,
      "Replace `windows-powershell` with `windows-shell`"
    );

    let edits = action
      .edit
      .as_ref()
      .unwrap()
      .changes
      .as_ref()
      .unwrap()
      .values()
      .next()
      .unwrap();

    assert_eq!(
      edits,
      &vec![lsp::TextEdit {
        range: lsp::Range::at(0, 4, 0, 22),
        new_text: "windows-shell".to_string(),
      }]
    );
  }

  #[test]
  fn collect_ignores_setting_outside_range() {
    let document =
      Document::from("set windows-powershell := true\nset export := true\n");

    let actions = Quickfixer::new(
      &document,
      &parameters(lsp::Range::at(1, 4, 1, 4), vec![]),
    )
    .collect();

    assert_eq!(actions, vec![]);
  }

  #[test]
  fn matching_diagnostics_filters_by_code_and_range() {
    let document = Document::from("foo := env_var(\"A\")\n");

    let target = lsp::Range::at(0, 7, 0, 14);

    let diagnostics = vec![
      diagnostic(target, "deprecated-function"),
      diagnostic(target, "other-rule"),
      diagnostic(lsp::Range::at(1, 0, 1, 5), "deprecated-function"),
    ];

    assert_eq!(
      Quickfixer::new(&document, &parameters(target, diagnostics))
        .matching_diagnostics(target, "deprecated-function"),
      vec![diagnostic(target, "deprecated-function")]
    );
  }
}
