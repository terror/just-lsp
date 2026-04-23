use super::*;

pub struct Quickfixer<'a> {
  document: &'a Document,
  parameters: &'a lsp::CodeActionParams,
}

impl<'a> Quickfixer<'a> {
  #[must_use]
  pub fn collect(&self) -> Vec<lsp::CodeActionOrCommand> {
    let mut actions = Vec::new();
    actions.extend(self.deprecated_functions());
    actions
  }

  fn deprecated_function_replacement(name: &str) -> Option<&'static str> {
    BUILTINS.iter().find_map(|builtin| match builtin {
      Builtin::Function {
        name: builtin_name,
        deprecated: Some(replacement),
        ..
      } if *builtin_name == name => Some(*replacement),
      _ => None,
    })
  }

  fn deprecated_functions(&self) -> Vec<lsp::CodeActionOrCommand> {
    let mut actions = Vec::new();

    for function_call in self.document.function_calls() {
      if !function_call.name.range.overlaps(self.parameters.range) {
        continue;
      }

      let Some(replacement) =
        Self::deprecated_function_replacement(&function_call.name.value)
      else {
        continue;
      };

      let diagnostics = self
        .matching_diagnostics(function_call.name.range, "deprecated-function");

      actions.push(lsp::CodeActionOrCommand::CodeAction(lsp::CodeAction {
        title: format!(
          "Replace `{}` with `{}`",
          function_call.name.value, replacement
        ),
        kind: Some(lsp::CodeActionKind::QUICKFIX),
        diagnostics: (!diagnostics.is_empty()).then_some(diagnostics),
        edit: Some(lsp::WorkspaceEdit {
          changes: Some(HashMap::from([(
            self.parameters.text_document.uri.clone(),
            vec![lsp::TextEdit {
              range: function_call.name.range,
              new_text: replacement.to_string(),
            }],
          )])),
          ..Default::default()
        }),
        ..Default::default()
      }));
    }

    actions
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
      panic!("expected CodeAction");
    };

    assert_eq!(action.title, "Replace `env_var` with `env`");
  }

  #[test]
  fn deprecated_function_replacement() {
    #[track_caller]
    fn case(name: &str, expected: Option<&'static str>) {
      assert_eq!(Quickfixer::deprecated_function_replacement(name), expected);
    }

    case("env_var", Some("env"));
    case("env_var_or_default", Some("env"));
    case("env", None);
    case("nonexistent", None);
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
