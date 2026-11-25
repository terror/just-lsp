use super::*;

#[derive(Debug)]
pub struct Analyzer<'a> {
  pub config: Option<&'a Config>,
  pub view: ModuleView<'a>,
}

impl Analyzer<'_> {
  /// Returns the diagnostics produced by running every rule against the
  /// view's document.
  ///
  /// Each rule's settings are looked up by its ID in `config`, or in the
  /// default configuration when `config` is `None`. Rules whose level is
  /// `off` are skipped. Otherwise, each diagnostic's severity is replaced by
  /// the configured level, or kept as the rule's default if no level is
  /// configured, and its `id` and `display` are set to the rule's ID and
  /// message.
  ///
  /// Rules can see declarations from imported files and other modules through
  /// the view, but only report problems in the view's document. Diagnostics
  /// are sorted by start position, then by message, and are not deduplicated.
  ///
  /// # Example
  ///
  /// ```
  /// use just_lsp::{Analyzer, Config, Document};
  ///
  /// let document = Document::from("foo: bar\n");
  ///
  /// let diagnostics = Analyzer {
  ///   config: None,
  ///   view: (&document).into(),
  /// }
  /// .analyze();
  ///
  /// assert_eq!(diagnostics[0].id, "unresolved-dependency");
  /// assert_eq!(diagnostics[0].message, "Recipe `bar` not found");
  ///
  /// let config = serde_json::from_value::<Config>(serde_json::json!({
  ///   "rules": {
  ///     "unresolved-dependency": "off"
  ///   }
  /// }))
  /// .unwrap();
  ///
  /// let diagnostics = Analyzer {
  ///   config: Some(&config),
  ///   view: (&document).into(),
  /// }
  /// .analyze();
  ///
  /// assert!(diagnostics.is_empty());
  /// ```
  #[must_use]
  pub fn analyze(&self) -> Vec<Diagnostic> {
    let context = RuleContext::new(&self.view);

    let default = Config::default();

    let config = self.config.unwrap_or(&default);

    let mut diagnostics = inventory::iter::<&dyn Rule>
      .into_iter()
      .flat_map(|rule| {
        let rule_config = config.rule_config(rule.id());

        if !rule.enabled(&rule_config) {
          return Vec::new();
        }

        rule
          .run(&context)
          .into_iter()
          .filter_map(move |diagnostic| {
            Some(Diagnostic {
              id: rule.id(),
              display: rule.title(),
              severity: rule_config.severity(rule.severity())?,
              ..diagnostic
            })
          })
          .collect()
      })
      .collect::<Vec<_>>();

    diagnostics.sort_by(|a, b| {
      a.range
        .start
        .line
        .cmp(&b.range.start.line)
        .then_with(|| a.range.start.character.cmp(&b.range.start.character))
        .then_with(|| a.message.cmp(&b.message))
    });

    diagnostics
  }
}

#[cfg(test)]
mod tests {
  use {
    super::*,
    indoc::{formatdoc, indoc},
  };

  #[test]
  fn analyze_complete() {
    AnalyzerTest::new(indoc! {
      "
      foo:
        echo \"foo\"

      bar: missing
        echo \"bar\"

      alias baz := nonexistent
      "
    })
    .error("Recipe `missing` not found", lsp::Range::at(3, 5, 3, 12))
    .error(
      "Recipe `nonexistent` not found",
      lsp::Range::at(6, 13, 6, 24),
    )
    .run();
  }

  #[test]
  fn analyzer_ignores_imported_recipe_diagnostics() {
    AnalyzerTest::new("foo: (bar 'baz')\nimport 'foo.just'\n")
      .file("foo.just", "bar baz: bar\n")
      .run();
  }

  #[test]
  fn arg_attribute_keywords_without_parameter_name() {
    #[track_caller]
    fn case(arguments: &str, message: &'static str, start: u32, end: u32) {
      AnalyzerTest::new(&formatdoc! {
        "
        [arg({arguments})]
        bar foo:
          echo {{{{foo}}}}
        "
      })
      .error(
        "Attribute `arg` got 0 arguments but takes 1 argument",
        lsp::Range::at(0, 0, 1, 0),
      )
      .error(message, lsp::Range::at(0, start, 0, end))
      .run();
    }

    case("flag", "`flag` arguments require `set lists`", 5, 9);

    case(
      "foo='bar'",
      "Unknown `[arg]` keyword `foo`, expected one of flag, help, long, max, min, multiple, pattern, short, value",
      5,
      14,
    );

    case(
      "help=arch()",
      "Attribute `arg` arguments must be const expressions",
      10,
      16,
    );

    case(
      "pattern=`foo`",
      "Attribute `arg` arguments must be const expressions",
      13,
      18,
    );

    case(
      "long='foo' + 'bar'",
      "Attribute `arg` arguments must be string literals",
      10,
      23,
    );
  }

  #[test]
  fn attributes_correct() {
    AnalyzerTest::new(indoc! {
      "
      [no-cd]
      [linux]
      [macos]
      foo:
        echo \"foo\"

      [doc('Recipe documentation')]
      bar:
        echo \"bar\"

      [default]
      baz:
        echo \"baz\"
      "
    })
    .run();
  }

  #[test]
  fn deeply_nested_expression() {
    AnalyzerTest::new(&format!(
      "_foo := {}'bar'{}",
      "(".repeat(1000),
      ")".repeat(1000)
    ))
    .run();
  }

  #[test]
  fn keywords_are_valid_names() {
    #[track_caller]
    fn case(name: &str) {
      AnalyzerTest::new(&formatdoc! {
        "
        set unstable

        {name} := 'foo'

        alias bar := {name}

        {name}() := 'foo'

        {name} baz:
          echo {{{{baz}}}} {{{{{name}}}}} {{{{{name}()}}}}

        bob: ({name} 'foo')
        "
      })
      .run();
    }

    case("alias");
    case("eager");
    case("export");
    case("import");
    case("mod");
    case("set");
    case("unexport");
  }

  #[test]
  fn recipe_named_import() {
    AnalyzerTest::new(indoc! {
      r"
      run: import

      import:
        body
      "
    })
    .run();
  }

  #[test]
  fn rule_config_off_suppresses_diagnostic() {
    let config = serde_json::from_value::<Config>(serde_json::json!({
      "rules": {
        "unresolved-dependency": "off"
      }
    }))
    .unwrap();

    AnalyzerTest::new("foo: bar\n").config(config).run();
  }

  #[test]
  fn rule_config_off_suppresses_invalid_attribute_placement() {
    let config = serde_json::from_value::<Config>(serde_json::json!({
      "rules": {
        "invalid-attribute-placement": "off"
      }
    }))
    .unwrap();

    AnalyzerTest::new("[private]\n")
      .config(config)
      .error("Syntax error near `[private]`", lsp::Range::at(0, 0, 1, 0))
      .run();
  }

  #[test]
  fn rule_config_overrides_severity_to_error() {
    let config = serde_json::from_value::<Config>(serde_json::json!({
      "rules": {
        "unused-variable": "error"
      }
    }))
    .unwrap();

    AnalyzerTest::new(indoc! {
      "
      foo := \"unused value\"

      recipe:
        echo foo
      "
    })
    .config(config)
    .error("Variable `foo` appears unused", lsp::Range::at(0, 0, 0, 3))
    .run();
  }

  #[test]
  fn rule_config_overrides_severity_to_warning() {
    let config = serde_json::from_value::<Config>(serde_json::json!({
      "rules": {
        "unresolved-dependency": "warning"
      }
    }))
    .unwrap();

    AnalyzerTest::new(indoc! {
      "
      foo:
        echo \"foo\"

      bar: baz
        echo \"bar\"
      "
    })
    .config(config)
    .warning(
      "Recipe `baz` not found. Did you mean `bar`?",
      lsp::Range::at(3, 5, 3, 8),
    )
    .run();
  }

  #[test]
  fn script_attribute_with_shebang_is_allowed() {
    AnalyzerTest::new(indoc! {
      "
      [script]
      publish:
        #!/usr/bin/env bash
        echo \"publish\"
      "
    })
    .run();
  }

  #[test]
  fn script_attribute_without_shebang_is_allowed() {
    AnalyzerTest::new(indoc! {
      "
      [script]
      publish:
        echo \"publish\"
      "
    })
    .run();
  }

  #[test]
  fn settings_multiple_errors() {
    AnalyzerTest::new(indoc! {
      "
      set unknown-setting := true
      set export := false
      set shell := ['bash']
      set export := false

      foo:
        echo \"foo\"
      "
    })
    .error(
      "Unknown setting `unknown-setting`",
      lsp::Range::at(0, 0, 1, 0),
    )
    .error("Duplicate setting `export`", lsp::Range::at(3, 0, 4, 0))
    .run();
  }

  #[test]
  fn unexport_followed_by_recipe() {
    AnalyzerTest::new(indoc! {
      "
      unexport foo
      bar:
      "
    })
    .run();
  }
}
