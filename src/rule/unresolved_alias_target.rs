use super::*;

define_rule! {
  /// Flags aliases that point to recipes which aren't defined.
  UnresolvedAliasTargetRule {
    id: "unresolved-alias-target",
    message: "alias target not found",
    run(context) {
      context
        .local_declarations(context.aliases())
        .filter(|alias| !context.declares_module(&alias.value))
        .filter_map(|alias| context.unresolved_recipe(&alias.value, alias.value.range))
        .collect()
    }
  }
}

#[cfg(test)]
mod tests {
  use {super::*, indoc::indoc};

  #[test]
  fn aliases_basic() {
    AnalyzerTest::new(indoc! {
      "
      foo:
        echo \"foo\"

      alias bar := foo
      "
    })
    .run();
  }

  #[test]
  fn aliases_missing_recipe() {
    AnalyzerTest::new(indoc! {
      "
      foo:
        echo \"foo\"

      alias bar := baz
      "
    })
    .error("Recipe `baz` not found", lsp::Range::at(3, 13, 3, 16))
    .run();
  }

  #[test]
  fn aliases_missing_target() {
    AnalyzerTest::new(indoc! {
      "
      alias foo :=
      "
    })
    .error("Missing identifier in alias", lsp::Range::at(0, 12, 0, 12))
    .error("Recipe `` not found", lsp::Range::at(0, 12, 0, 12))
    .run();
  }
}
