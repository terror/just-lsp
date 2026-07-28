use super::*;

define_rule! {
  /// Flags alias declarations that reuse the same name multiple times.
  DuplicateAliasRule {
    id: "duplicate-alias",
    message: "duplicate alias",
    phase: RulePhase::Project,
    run(context) {
      let mut diagnostics = Vec::new();

      let mut groups = HashMap::<String, GroupSet>::new();

      for alias in context.aliases() {
        let current = GroupSet::from_attributes(&alias.attributes);

        let previous = groups
          .entry(alias.name.value.clone())
          .or_default();

        let duplicate = previous.conflicts_with(&current);

        previous.union_with(current);

        if duplicate {
          diagnostics.push(Diagnostic::error(
            format!("Duplicate alias `{}`", alias.name.value),
            alias.range,
          ));
        }
      }

      diagnostics
    }
  }
}
