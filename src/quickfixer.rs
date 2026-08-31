use super::*;

pub struct Quickfixer<'a> {
  pub diagnostics: &'a [Diagnostic],
  pub parameters: &'a lsp::CodeActionParams,
}

impl Quickfixer<'_> {
  fn action(
    &self,
    source: &Diagnostic,
    quickfix: &Quickfix,
  ) -> lsp::CodeActionOrCommand {
    let diagnostics = self
      .parameters
      .context
      .diagnostics
      .iter()
      .filter(|diagnostic| {
        diagnostic.range == source.range
          && matches!(
            &diagnostic.code,
            Some(lsp::NumberOrString::String(value)) if value == &source.id
          )
      })
      .cloned()
      .collect::<Vec<_>>();

    lsp::CodeActionOrCommand::CodeAction(lsp::CodeAction {
      title: quickfix.title.clone(),
      kind: Some(lsp::CodeActionKind::QUICKFIX),
      diagnostics: (!diagnostics.is_empty()).then_some(diagnostics),
      edit: Some(lsp::WorkspaceEdit {
        changes: Some(HashMap::from([(
          self.parameters.text_document.uri.clone(),
          quickfix.edits.clone(),
        )])),
        ..Default::default()
      }),
      ..Default::default()
    })
  }

  #[must_use]
  pub fn collect(&self) -> Vec<lsp::CodeActionOrCommand> {
    self
      .diagnostics
      .iter()
      .filter(|diagnostic| diagnostic.range.overlaps(self.parameters.range))
      .filter_map(|diagnostic| {
        diagnostic
          .quickfix
          .as_ref()
          .map(|quickfix| self.action(diagnostic, quickfix))
      })
      .collect()
  }
}

#[cfg(test)]
mod tests {
  use {super::*, indoc::indoc, pretty_assertions::assert_eq};

  #[derive(Debug)]
  struct Test {
    config: Config,
    diagnostics: Option<Vec<Diagnostic>>,
    document: Document,
    imported_documents: Vec<Document>,
    quickfixes: Vec<Quickfix>,
    range: lsp::Range,
  }

  impl Test {
    fn config(self, config: Config) -> Self {
      Self { config, ..self }
    }

    fn diagnostics(self, diagnostics: Vec<Diagnostic>) -> Self {
      Self {
        diagnostics: Some(diagnostics),
        ..self
      }
    }

    fn imported_document(self, content: &str) -> Self {
      Self {
        imported_documents: self
          .imported_documents
          .into_iter()
          .chain([Document::from(content)])
          .collect(),
        ..self
      }
    }

    fn new(content: &str) -> Self {
      Self {
        config: Config::default(),
        diagnostics: None,
        document: Document::from(content),
        imported_documents: Vec::new(),
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
        diagnostics,
        document,
        imported_documents,
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

      let analyzer = Analyzer {
        config: Some(&config),
        document: &document,
        imported_documents: imported_documents.iter().collect(),
      };

      let actual_diagnostics = analyzer.quickfixes();

      if let Some(diagnostics) = diagnostics {
        assert_eq!(actual_diagnostics, diagnostics);
      }

      let quickfixer = Quickfixer {
        diagnostics: &actual_diagnostics,
        parameters: &parameters,
      };

      let actions = quickfixer.collect();

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
    Test::new(indoc! {
      "
      foo := env_var(\"A\")
      bar := env_var_or_default(\"B\", \"C\")
      "
    })
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
  fn ignores_imported_recipes() {
    Test::new("import 'dep.just'\n")
      .imported_document(indoc! {
        "
        [parallel]
        foo:
        "
      })
      .run();
  }

  #[test]
  fn ignores_setting_outside_range() {
    Test::new(indoc! {
      "
      set windows-powershell := true
      set export := true
      "
    })
    .range(lsp::Range::at(1, 4, 1, 4))
    .run();
  }

  #[test]
  fn only_runs_providers() {
    Test::new(indoc! {
      "
      foo := unknown
      bar := env_var(\"BAR\")
      "
    })
    .diagnostics(vec![Diagnostic {
      display: "deprecated function".into(),
      id: "deprecated-function".into(),
      message: "`env_var` is deprecated, use `env` instead".into(),
      quickfix: Some(Quickfix {
        edits: vec![lsp::TextEdit {
          range: lsp::Range::at(1, 7, 1, 14),
          new_text: "env".into(),
        }],
        range: lsp::Range::at(1, 7, 1, 14),
        title: "Replace `env_var` with `env`".into(),
      }),
      range: lsp::Range::at(1, 7, 1, 14),
      severity: lsp::DiagnosticSeverity::WARNING,
    }])
    .run();
  }

  #[test]
  fn removes_parallel_attribute() {
    Test::new(indoc! {
      "
      [parallel]
      foo: bar
      bar:
      "
    })
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
  fn replaces_misspelled_alias_target() {
    Test::new(indoc! {
      "
      build:

      alias b := biuld
      "
    })
    .range(lsp::Range::at(2, 11, 2, 11))
    .quickfix(Quickfix {
      edits: vec![lsp::TextEdit {
        range: lsp::Range::at(2, 11, 2, 16),
        new_text: "build".into(),
      }],
      range: lsp::Range::at(2, 11, 2, 16),
      title: "Replace `biuld` with `build`".into(),
    })
    .run();
  }

  #[test]
  fn replaces_misspelled_attribute() {
    Test::new(indoc! {
      "
      [prvate]
      build:
      "
    })
    .range(lsp::Range::at(0, 1, 0, 1))
    .quickfix(Quickfix {
      edits: vec![lsp::TextEdit {
        range: lsp::Range::at(0, 1, 0, 7),
        new_text: "private".into(),
      }],
      range: lsp::Range::at(0, 1, 0, 7),
      title: "Replace `prvate` with `private`".into(),
    })
    .run();
  }

  #[test]
  fn replaces_misspelled_dependency_name_only() {
    Test::new(indoc! {
      "
      import 'dep.just'

      test target: (biuld target)
      "
    })
    .imported_document("build target:\n")
    .range(lsp::Range::at(2, 14, 2, 14))
    .quickfix(Quickfix {
      edits: vec![lsp::TextEdit {
        range: lsp::Range::at(2, 14, 2, 19),
        new_text: "build".into(),
      }],
      range: lsp::Range::at(2, 14, 2, 19),
      title: "Replace `biuld` with `build`".into(),
    })
    .run();
  }

  #[test]
  fn replaces_misspelled_function() {
    Test::new("jobs := num_jobz()\n")
      .range(lsp::Range::at(0, 8, 0, 8))
      .quickfix(Quickfix {
        edits: vec![lsp::TextEdit {
          range: lsp::Range::at(0, 8, 0, 16),
          new_text: "num_jobs".into(),
        }],
        range: lsp::Range::at(0, 8, 0, 16),
        title: "Replace `num_jobz` with `num_jobs`".into(),
      })
      .run();
  }

  #[test]
  fn replaces_misspelled_identifier_with_local_parameter() {
    Test::new(indoc! {
      "
      build target:
        echo {{targte}}
      "
    })
    .range(lsp::Range::at(1, 9, 1, 9))
    .quickfix(Quickfix {
      edits: vec![lsp::TextEdit {
        range: lsp::Range::at(1, 9, 1, 15),
        new_text: "target".into(),
      }],
      range: lsp::Range::at(1, 9, 1, 15),
      title: "Replace `targte` with `target`".into(),
    })
    .run();
  }

  #[test]
  fn replaces_misspelled_setting() {
    Test::new("set shel := ['bash']\n")
      .range(lsp::Range::at(0, 4, 0, 4))
      .quickfix(Quickfix {
        edits: vec![lsp::TextEdit {
          range: lsp::Range::at(0, 4, 0, 8),
          new_text: "shell".into(),
        }],
        range: lsp::Range::at(0, 4, 0, 8),
        title: "Replace `shel` with `shell`".into(),
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
  fn replaces_windows_shell_setting() {
    Test::new(
      "set windows-shell := [\"powershell.exe\", \"-NoLogo\", \"-Command\"]\n",
    )
    .range(lsp::Range::at(0, 4, 0, 4))
    .quickfix(Quickfix {
      edits: vec![lsp::TextEdit {
        range: lsp::Range::at(0, 0, 1, 0),
        new_text: indoc! {
          "
          [windows]
          set shell := [\"powershell.exe\", \"-NoLogo\", \"-Command\"]
          "
        }
        .to_string(),
      }],
      range: lsp::Range::at(0, 4, 0, 17),
      title: "Replace `windows-shell` with `[windows] set shell`".to_string(),
    })
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

  #[test]
  fn skips_windows_shell_setting_when_replacement_exists() {
    Test::new(indoc! {
      "
      [windows]
      set shell := [\"foo\"]
      set windows-shell := [\"bar\"]
      "
    })
    .range(lsp::Range::at(2, 4, 2, 4))
    .run();
  }
}
