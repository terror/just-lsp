use super::*;

/// Reports recipes that combine a shebang line with the `[script]` attribute.
pub(crate) struct ScriptShebangConflictRule;

impl Rule for ScriptShebangConflictRule {
  fn display_name(&self) -> &'static str {
    "Script Shebang Conflict"
  }

  fn id(&self) -> &'static str {
    "script-shebang-conflict"
  }

  fn run(&self, context: &RuleContext<'_>) -> Vec<lsp::Diagnostic> {
    let Some(tree) = context.tree() else {
      return Vec::new();
    };

    let root = tree.root_node();
    let recipe_nodes = root.find_all("recipe");

    let mut diagnostics = Vec::new();

    for recipe in context.recipes() {
      let script_attributes = recipe
        .attributes
        .iter()
        .filter(|attribute| attribute.name.value == "script")
        .collect::<Vec<_>>();

      if script_attributes.is_empty() {
        continue;
      }

      let has_shebang = recipe_nodes.iter().any(|node| {
        node.get_range() == recipe.range
          && node.find("recipe_body > shebang").is_some()
      });

      if !has_shebang {
        continue;
      }

      for attribute in script_attributes {
        diagnostics.push(self.diagnostic(lsp::Diagnostic {
          range: attribute.range,
          severity: Some(lsp::DiagnosticSeverity::ERROR),
          message: format!(
            "Recipe `{}` has both shebang line and `[script]` attribute",
            recipe.name
          ),
          ..Default::default()
        }));
      }
    }

    diagnostics
  }
}
