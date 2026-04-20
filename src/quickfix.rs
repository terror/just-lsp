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
