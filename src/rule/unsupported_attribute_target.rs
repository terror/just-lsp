use super::*;

define_rule! {
  /// Validates that each attribute is attached to a supported target kind
  /// (recipe, module, alias, etc.) according to the builtin metadata.
  UnsupportedAttributeTargetRule {
    id: "unsupported-attribute-target",
    message: "unsupported attribute target",
  }
}

impl UnsupportedAttributeTargetRule {
  fn run(context: &RuleContext<'_>) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();

    for attribute in context.attributes() {
      let attribute_name = &attribute.name.value;

      let Some(Builtin::Attribute { targets, .. }) =
        Builtin::attribute(attribute_name)
      else {
        continue;
      };

      let Some(target_type) = attribute.target else {
        continue;
      };

      if !targets.contains(&target_type) {
        diagnostics.push(Diagnostic::error(
          format!(
            "Attribute `{attribute_name}` cannot be applied to {target_type} target",
          ),
          attribute.range,
        ));
      }
    }

    diagnostics
  }
}

#[cfg(test)]
mod tests {
  use {super::*, indoc::indoc};

  #[test]
  fn attributes_on_assignments() {
    AnalyzerTest::new(indoc! {
      "
      [private]
      secret := \"secret value\"

      [private]
      _db_url := \"postgres://user:pass@host:port/db\"

      public_var := \"public value\"

      test:
        echo {{ secret }}
        echo {{ _db_url }}
        echo {{ public_var }}
      "
    })
    .run();
  }

  #[test]
  fn attributes_on_exported_assignments() {
    AnalyzerTest::new(indoc! {
      "
      [private]
      export PATH := '/usr/local/bin'
      "
    })
    .run();
  }

  #[test]
  fn attributes_wrong_target() {
    AnalyzerTest::new(indoc! {
      "
      [group: 'foo']
      alias f := foo

      foo:
        echo \"foo\"
      "
    })
    .error(
      "Attribute `group` cannot be applied to alias target",
      lsp::Range::at(0, 0, 1, 0),
    )
    .run();
  }

  #[test]
  fn conditional_attributes_on_all_top_level_items() {
    AnalyzerTest::new(indoc! {
      "
      set unstable

      [windows]
      foo() := 'bar'

      [unix]
      import? 'foo.just'

      [windows]
      unexport FOO

      [windows]
      export BAR := foo()
      "
    })
    .run();
  }

  #[test]
  fn continue_attribute_rejects_unsupported_target() {
    AnalyzerTest::new(indoc! {
      "
      [continue]
      alias bar := foo

      foo:
        echo foo
      "
    })
    .error(
      "Attribute `continue` cannot be applied to alias target",
      lsp::Range::at(0, 0, 1, 0),
    )
    .run();
  }

  #[test]
  fn env_attribute_wrong_target() {
    AnalyzerTest::new(indoc! {
      "
      [env('FOO', 'bar')]
      alias baz := foo

      foo:
        echo \"foo\"
      "
    })
    .error(
      "Attribute `env` cannot be applied to alias target",
      lsp::Range::at(0, 0, 1, 0),
    )
    .run();
  }

  #[test]
  fn module_attributes_group() {
    AnalyzerTest::new(indoc! {
      "
      [group: 'tools']
      mod foo
      "
    })
    .file("foo.just", "")
    .run();
  }

  #[test]
  fn unstable_feature_gate_invalid_cache_target() {
    AnalyzerTest::new(indoc! {
      "
      [cache]
      foo := 'bar'
      baz:
        echo {{ foo }}
      "
    })
    .error(
      "Attribute `cache` cannot be applied to assignment target",
      lsp::Range::at(0, 0, 1, 0),
    )
    .run();
  }
}
