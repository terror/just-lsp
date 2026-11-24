use super::*;

define_rule! {
  /// Flags aliases that point to recipes which aren't defined.
  UnresolvedAliasTargetRule {
    id: "unresolved-alias-target",
    message: "alias target not found",
    run(context) {
      context
        .document()
        .aliases()
        .iter()
        .filter(|alias| !context.declares_module(&alias.value))
        .filter_map(|alias| context.unresolved_recipe(&alias.value, alias.value.range))
        .collect()
    }
  }
}
