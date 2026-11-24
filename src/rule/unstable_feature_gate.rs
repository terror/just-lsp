use super::*;

define_rule! {
  UnstableFeatureGateRule {
    id: "unstable-feature-gate",
    message: "unstable feature used without set unstable",
    run(context) {
      let mut diagnostics = Vec::new();

      if context.setting_enabled("unstable") {
        return diagnostics;
      }

      for setting in context.local_declarations(context.settings()) {
        if setting.name.value == "lists" {
          diagnostics.push(Diagnostic::warning(
            "`set lists` is unstable without `set unstable`",
            setting.name.range,
          ));
        }
      }

      for function in context.local_declarations(context.functions()) {
        diagnostics.push(Diagnostic::warning(
          format!(
            "User-defined function `{}` is unstable without `set unstable`",
            function.name.value
          ),
          function.name.range,
        ));
      }

      for attribute in context.attributes() {
        if attribute.name.value == "cache"
          && attribute.target == Some(AttributeTarget::Recipe)
        {
          diagnostics.push(Diagnostic::warning(
            "`[cache]` is unstable without `set unstable`",
            attribute.name.range,
          ));
        }
      }

      diagnostics
    }
  }
}

#[cfg(test)]
mod tests {
  use {
    super::*,
    indoc::{formatdoc, indoc},
  };

  #[test]
  fn unstable_feature_gate_cache_requires_unstable() {
    AnalyzerTest::new("[script]\n[cache]\nfoo:\n")
      .warning(
        "`[cache]` is unstable without `set unstable`",
        lsp::Range::at(1, 1, 1, 6),
      )
      .run();
  }

  #[test]
  fn unstable_feature_gate_cache_with_arguments_requires_unstable() {
    AnalyzerTest::new("[script]\n[cache(extra='bar')]\nfoo:\n")
      .warning(
        "`[cache]` is unstable without `set unstable`",
        lsp::Range::at(1, 1, 1, 6),
      )
      .run();
  }

  #[test]
  fn unstable_feature_gate_disabled_lists() {
    AnalyzerTest::new("set lists := false\n")
      .warning(
        "`set lists` is unstable without `set unstable`",
        lsp::Range::at(0, 4, 0, 9),
      )
      .run();
  }

  #[test]
  fn unstable_feature_gate_disabled_unstable() {
    AnalyzerTest::new(indoc! {
      "
      set unstable := false
      set lists
      foo(bar) := bar
      [script]
      [cache]
      baz:
        echo {{foo('bar')}}
      "
    })
    .warning(
      "`set lists` is unstable without `set unstable`",
      lsp::Range::at(1, 4, 1, 9),
    )
    .warning(
      "User-defined function `foo` is unstable without `set unstable`",
      lsp::Range::at(2, 0, 2, 3),
    )
    .warning(
      "`[cache]` is unstable without `set unstable`",
      lsp::Range::at(4, 1, 4, 6),
    )
    .run();
  }

  #[test]
  fn unstable_feature_gate_enabled_unstable() {
    #[track_caller]
    fn case(setting: &str) {
      AnalyzerTest::new(&formatdoc! {
        "
        {setting}
        set lists
        foo(bar) := bar
        [script]
        [cache]
        baz:
          echo {{{{foo('bar')}}}}
        "
      })
      .run();
    }

    case("set unstable");
    case("set unstable := true");
  }

  #[test]
  fn unstable_feature_gate_imported_features() {
    AnalyzerTest::new("foo:\nimport 'foo.just'\n")
      .file(
        "foo.just",
        indoc! {
          "
          set lists
          bar(baz) := baz
          [script]
          [cache]
          baz:
          "
        },
      )
      .run();
  }

  #[test]
  fn unstable_feature_gate_imported_unstable() {
    AnalyzerTest::new(indoc! {
      "
      set lists
      foo(bar) := bar
      [script]
      [cache]
      baz:
        echo {{foo('bar')}}
      import 'foo.just'
      "
    })
    .file("foo.just", "set unstable\n")
    .run();
  }

  #[test]
  fn unstable_feature_gate_lists_requires_unstable() {
    AnalyzerTest::new("set lists\n")
      .warning(
        "`set lists` is unstable without `set unstable`",
        lsp::Range::at(0, 4, 0, 9),
      )
      .run();
  }

  #[test]
  fn unstable_feature_gate_lists_true_requires_unstable() {
    AnalyzerTest::new("set lists := true\n")
      .warning(
        "`set lists` is unstable without `set unstable`",
        lsp::Range::at(0, 4, 0, 9),
      )
      .run();
  }

  #[test]
  fn unstable_feature_gate_user_defined_function_requires_unstable() {
    AnalyzerTest::new("foo(bar) := bar\nbaz:\n  echo {{foo('bar')}}\n")
      .warning(
        "User-defined function `foo` is unstable without `set unstable`",
        lsp::Range::at(0, 0, 0, 3),
      )
      .run();
  }
}
