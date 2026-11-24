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
    files: Vec<(&'static str, String)>,
    quickfixes: Vec<Quickfix>,
    range: lsp::Range,
  }

  impl Test {
    fn config(self, config: Config) -> Self {
      Self { config, ..self }
    }

    fn file(mut self, path: &'static str, content: &str) -> Self {
      self.files.push((path, content.to_owned()));
      self
    }

    fn new(content: &str) -> Self {
      Self {
        config: Config::default(),
        files: vec![("justfile", content.to_owned())],
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

    #[track_caller]
    fn run(self) {
      let directory = if cfg!(windows) {
        PathBuf::from("C:\\")
      } else {
        PathBuf::from("/")
      };

      let uris = self
        .files
        .iter()
        .map(|(path, _)| {
          lsp::Url::from_file_path(directory.join(path)).unwrap()
        })
        .collect::<Vec<_>>();

      let mut workspace = Workspace::default();

      for (uri, (_, text)) in uris.iter().zip(&self.files).rev() {
        workspace
          .open(lsp::DidOpenTextDocumentParams {
            text_document: lsp::TextDocumentItem {
              language_id: "just".into(),
              text: text.clone(),
              uri: uri.clone(),
              version: 1,
            },
          })
          .unwrap();
      }

      let root = &uris[0];

      let parameters = lsp::CodeActionParams {
        text_document: lsp::TextDocumentIdentifier { uri: root.clone() },
        range: self.range,
        context: lsp::CodeActionContext::default(),
        work_done_progress_params: lsp::WorkDoneProgressParams::default(),
        partial_result_params: lsp::PartialResultParams::default(),
      };

      let diagnostics = Analyzer {
        config: Some(&self.config),
        view: workspace.views(root).remove(0),
      }
      .analyze();

      let actions = Quickfixer {
        diagnostics: &diagnostics,
        parameters: &parameters,
      }
      .collect();

      let expected = self
        .quickfixes
        .into_iter()
        .map(|quickfix| {
          lsp::CodeActionOrCommand::CodeAction(lsp::CodeAction {
            title: quickfix.title().to_string(),
            kind: Some(lsp::CodeActionKind::QUICKFIX),
            diagnostics: None,
            edit: Some(lsp::WorkspaceEdit {
              changes: Some(HashMap::from([(
                root.clone(),
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
      .file(
        "dep.just",
        indoc! {
          "
          [parallel]
          foo:
          "
        },
      )
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
    .range(lsp::Range::at(0, 0, 2, 0))
    .quickfix(Quickfix::edit(
      "Replace `env_var` with `env`",
      lsp::Range::at(1, 7, 1, 14),
      "env",
    ))
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
    .file("dep.just", "build target:\n")
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
    fn case<const N: usize>(content: &str, edits: [lsp::TextEdit; N]) {
      Test::new(content)
        .range(edits[0].range)
        .quickfix(Quickfix::new(
          "Replace `windows-powershell` with `[windows] set shell`",
          edits,
        ))
        .run();
    }

    let shell = |range| lsp::TextEdit {
      range,
      new_text: r#"shell := ["powershell.exe", "-NoLogo", "-Command"]"#.into(),
    };

    let windows = |line| lsp::TextEdit {
      range: lsp::Range::at(line, 0, line, 0),
      new_text: "[windows]\n".into(),
    };

    case(
      "set windows-powershell := true\n",
      [shell(lsp::Range::at(0, 4, 0, 30)), windows(0)],
    );

    case(
      "set windows-powershell\n",
      [shell(lsp::Range::at(0, 4, 0, 22)), windows(0)],
    );

    case(
      indoc! {
        "
        [windows]
        set windows-powershell := true
        "
      },
      [shell(lsp::Range::at(1, 4, 1, 30))],
    );

    case(
      indoc! {
        "
        [unix]
        set shell := ['foo']
        set windows-powershell := true
        "
      },
      [shell(lsp::Range::at(2, 4, 2, 30)), windows(2)],
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
    Test::new("set windows-powershell := true\nimport 'foo.just'\n")
      .file("foo.just", "set shell := ['foo']\n")
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
      set shell := ['foo']
      "
    });

    case(indoc! {
      "
      set windows-powershell := true
      [windows]
      set shell := ['foo']
      "
    });
  }

  #[test]
  fn skips_windows_powershell_setting_with_non_windows_attribute() {
    Test::new(indoc! {
      "
      [unix]
      set windows-powershell := true
      "
    })
    .range(lsp::Range::at(1, 4, 1, 4))
    .run();
  }

  #[test]
  fn skips_windows_shell_setting_when_replacement_exists() {
    #[track_caller]
    fn case(content: &str) {
      Test::new(content).range(lsp::Range::at(2, 4, 2, 4)).run();
    }

    case(indoc! {
      "
      [windows]
      set shell := [\"foo\"]
      set windows-shell := [\"bar\"]
      "
    });

    case(indoc! {
      "
      set shell := [\"foo\"]

      set windows-shell := [\"bar\"]
      "
    });
  }
}
