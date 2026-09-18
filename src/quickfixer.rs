use super::*;

pub(crate) struct Quickfixer<'a> {
  pub(crate) diagnostics: &'a [Diagnostic],
  pub(crate) parameters: &'a lsp::CodeActionParams,
}

impl Quickfixer<'_> {
  #[must_use]
  pub(crate) fn collect(&self) -> Vec<lsp::CodeActionOrCommand> {
    self
      .diagnostics
      .iter()
      .filter(|diagnostic| diagnostic.range.overlaps(self.parameters.range))
      .flat_map(|source| {
        source.quickfixes.iter().map(move |quickfix| {
          let diagnostics = self
            .parameters
            .context
            .diagnostics
            .iter()
            .filter(|diagnostic| source == *diagnostic)
            .cloned()
            .collect::<Vec<_>>();

          lsp::CodeActionOrCommand::CodeAction(lsp::CodeAction {
            title: quickfix.title().to_string(),
            kind: Some(lsp::CodeActionKind::QUICKFIX),
            diagnostics: (!diagnostics.is_empty()).then_some(diagnostics),
            edit: Some(lsp::WorkspaceEdit {
              changes: Some(HashMap::from([(
                self.parameters.text_document.uri.clone(),
                quickfix.edits().to_vec(),
              )])),
              ..Default::default()
            }),
            ..Default::default()
          })
        })
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
    imported_documents: Vec<String>,
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

    fn imported_document(mut self, content: &str) -> Self {
      self.imported_documents.push(content.into());
      self
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

    fn quickfix(mut self, quickfix: Quickfix) -> Self {
      self.quickfixes.push(quickfix);
      self
    }

    fn range(self, range: lsp::Range) -> Self {
      Self { range, ..self }
    }

    #[track_caller]
    fn run(self) {
      let Test {
        config,
        diagnostics,
        document,
        imported_documents,
        quickfixes,
        range,
      } = self;

      let mut documents = DocumentStore::default();

      let mut project = Project::new(document.uri.clone());

      for (index, text) in imported_documents.into_iter().enumerate() {
        let uri = lsp::Url::parse(&format!("file:///foo{index}.just")).unwrap();

        documents
          .open(lsp::DidOpenTextDocumentParams {
            text_document: lsp::TextDocumentItem {
              uri: uri.clone(),
              language_id: "just".into(),
              version: 1,
              text,
            },
          })
          .unwrap();

        project
          .dependencies
          .entry(document.uri.clone())
          .or_default()
          .push(ProjectDependency {
            kind: ProjectDependencyKind::Import {
              attributes: Vec::new(),
              optional: false,
            },
            location: lsp::Range::default(),
            target: ProjectDependencyTarget::Resolved(uri),
          });
      }

      let import_scope = ImportScope::from(&project);

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
        view: ProjectView::new(&document, &import_scope, &documents),
      };

      let actual_diagnostics = analyzer
        .analyze()
        .into_iter()
        .filter(|diagnostic| !diagnostic.quickfixes.is_empty())
        .collect::<Vec<_>>();

      if let Some(diagnostics) = diagnostics {
        assert_eq!(actual_diagnostics, diagnostics);
      }

      let quickfixer = Quickfixer {
        diagnostics: &actual_diagnostics,
        parameters: &parameters,
      };

      let actions = quickfixer.collect();

      let expected = quickfixes
        .into_iter()
        .map(|quickfix| {
          lsp::CodeActionOrCommand::CodeAction(lsp::CodeAction {
            title: quickfix.title().to_string(),
            kind: Some(lsp::CodeActionKind::QUICKFIX),
            diagnostics: None,
            edit: Some(lsp::WorkspaceEdit {
              changes: Some(HashMap::from([(
                document.uri.clone(),
                quickfix.edits().to_vec(),
              )])),
              ..Default::default()
            }),
            ..Default::default()
          })
        })
        .collect::<Vec<_>>();

      assert_eq!(actions, expected);
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
    .quickfix(Quickfix::edit(
      "Replace `env_var` with `env`",
      lsp::Range::at(0, 7, 0, 14),
      "env",
    ))
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
  fn only_returns_diagnostics_with_quickfixes() {
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
      quickfixes: vec![Quickfix::edit(
        "Replace `env_var` with `env`",
        lsp::Range::at(1, 7, 1, 14),
        "env",
      )],
      range: lsp::Range::at(1, 7, 1, 14),
      severity: lsp::DiagnosticSeverity::WARNING,
    }])
    .run();
  }

  #[test]
  fn removes_disabled_windows_powershell_setting() {
    #[track_caller]
    fn case(content: &str, range: lsp::Range) {
      Test::new(content)
        .range(range)
        .quickfix(Quickfix::removal(range, "Remove `set windows-powershell`"))
        .run();
    }

    case(
      indoc! {
        "
        set windows-powershell := false
        set shell := ['foo']
        "
      },
      lsp::Range::at(0, 0, 1, 0),
    );

    case(
      indoc! {
        "
        [windows]
        set windows-powershell := false
        set shell := ['foo']
        "
      },
      lsp::Range::at(0, 0, 2, 0),
    );
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
    .quickfix(Quickfix::removal(
      lsp::Range::at(0, 0, 1, 0),
      "Remove `[parallel]`",
    ))
    .run();
  }

  #[test]
  fn removes_parallel_attribute_item() {
    Test::new(indoc! {
      "
      [private, parallel]
      foo: bar
      bar:
      "
    })
    .range(lsp::Range::at(0, 0, 1, 0))
    .quickfix(Quickfix::removal(
      lsp::Range::at(0, 8, 0, 18),
      "Remove `[parallel]`",
    ))
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
    .quickfix(Quickfix::edit(
      "Replace `biuld` with `build`",
      lsp::Range::at(2, 11, 2, 16),
      "build",
    ))
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
    .quickfix(Quickfix::edit(
      "Replace `prvate` with `private`",
      lsp::Range::at(0, 1, 0, 7),
      "private",
    ))
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
    .quickfix(Quickfix::edit(
      "Replace `biuld` with `build`",
      lsp::Range::at(2, 14, 2, 19),
      "build",
    ))
    .run();
  }

  #[test]
  fn replaces_misspelled_function() {
    Test::new("jobs := num_jobz()\n")
      .range(lsp::Range::at(0, 8, 0, 8))
      .quickfix(Quickfix::edit(
        "Replace `num_jobz` with `num_jobs`",
        lsp::Range::at(0, 8, 0, 16),
        "num_jobs",
      ))
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
    .quickfix(Quickfix::edit(
      "Replace `targte` with `target`",
      lsp::Range::at(1, 9, 1, 15),
      "target",
    ))
    .run();
  }

  #[test]
  fn replaces_misspelled_setting() {
    Test::new("set shel := ['bash']\n")
      .range(lsp::Range::at(0, 4, 0, 4))
      .quickfix(Quickfix::edit(
        "Replace `shel` with `shell`",
        lsp::Range::at(0, 4, 0, 8),
        "shell",
      ))
      .run();
  }

  #[test]
  fn replaces_windows_powershell_setting() {
    #[track_caller]
    fn case(content: &str, range: lsp::Range) {
      Test::new(content)
        .range(range)
        .quickfix(Quickfix::edit(
          "Replace `windows-powershell` with `windows-shell`",
          range,
          r#"windows-shell := ["powershell.exe", "-NoLogo", "-Command"]"#,
        ))
        .run();
    }

    case(
      "set windows-powershell := true\n",
      lsp::Range::at(0, 4, 0, 30),
    );

    case("set windows-powershell\n", lsp::Range::at(0, 4, 0, 22));

    case(
      indoc! {
        "
        [windows]
        set windows-powershell := true
        "
      },
      lsp::Range::at(1, 4, 1, 30),
    );

    case(
      indoc! {
        "
        set shell := ['foo']
        set windows-powershell := true
        "
      },
      lsp::Range::at(1, 4, 1, 30),
    );

    case(
      indoc! {
        "
        [unix]
        set windows-shell := ['foo']
        [windows]
        set windows-powershell := true
        "
      },
      lsp::Range::at(3, 4, 3, 30),
    );
  }

  #[test]
  fn replaces_windows_shell_setting() {
    Test::new(
      "set windows-shell := [\"powershell.exe\", \"-NoLogo\", \"-Command\"]\n",
    )
    .range(lsp::Range::at(0, 4, 0, 4))
    .quickfix(Quickfix::new(
      "Replace `windows-shell` with `[windows] set shell`",
      [
        lsp::TextEdit {
          range: lsp::Range::at(0, 4, 0, 17),
          new_text: "shell".into(),
        },
        lsp::TextEdit {
          range: lsp::Range::at(0, 0, 0, 0),
          new_text: "[windows]\n".into(),
        },
      ],
    ))
    .run();
  }

  #[test]
  fn replaces_windows_shell_setting_with_multiline_value() {
    Test::new(indoc! {
      "
      set windows-shell := [
        'foo',
        'bar',
      ]
      "
    })
    .range(lsp::Range::at(0, 4, 0, 4))
    .quickfix(Quickfix::new(
      "Replace `windows-shell` with `[windows] set shell`",
      [
        lsp::TextEdit {
          range: lsp::Range::at(0, 4, 0, 17),
          new_text: "shell".into(),
        },
        lsp::TextEdit {
          range: lsp::Range::at(0, 0, 0, 0),
          new_text: "[windows]\n".into(),
        },
      ],
    ))
    .run();
  }

  #[test]
  fn replaces_windows_shell_setting_with_windows_attribute() {
    Test::new(indoc! {
      "
      [windows]
      set windows-shell := [
        'foo',
        'bar',
      ]
      "
    })
    .range(lsp::Range::at(1, 4, 1, 4))
    .quickfix(Quickfix::edit(
      "Replace `windows-shell` with `[windows] set shell`",
      lsp::Range::at(1, 4, 1, 17),
      "shell",
    ))
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
  fn skips_invalid_windows_powershell_setting() {
    #[track_caller]
    fn case(content: &str) {
      Test::new(content).range(lsp::Range::at(0, 4, 0, 4)).run();
    }

    case("set windows-powershell := 'foo'\n");
    case("set windows-powershell := ['foo']\n");
  }

  #[test]
  fn skips_windows_powershell_setting_when_imported_replacement_exists() {
    Test::new("set windows-powershell := true\n")
      .imported_document("set windows-shell := ['foo']\n")
      .range(lsp::Range::at(0, 4, 0, 4))
      .run();
  }

  #[test]
  fn skips_windows_powershell_setting_when_replacement_exists() {
    #[track_caller]
    fn case(content: &str) {
      Test::new(content).range(lsp::Range::at(0, 4, 0, 4)).run();
    }

    case(indoc! {
      "
      set windows-powershell := true
      set windows-shell := ['foo']
      "
    });

    case(indoc! {
      "
      set windows-powershell := true
      [windows]
      set windows-shell := ['foo']
      "
    });
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
