use super::*;

pub struct AliasesRule;

impl Rule for AliasesRule {
  fn id(&self) -> &'static str {
    "aliases"
  }

  fn display_name(&self) -> &'static str {
    "Aliases"
  }

  fn run(&self, ctx: &RuleContext<'_>) -> Vec<lsp::Diagnostic> {
    let mut diagnostics = Vec::new();

    let recipe_names = ctx.recipe_names();

    for alias in ctx.aliases() {
      if !recipe_names.contains(&alias.value.value) {
        diagnostics.push(self.diagnostic(lsp::Diagnostic {
          range: alias.value.range,
          severity: Some(lsp::DiagnosticSeverity::ERROR),
          message: format!("Recipe `{}` not found", alias.value.value),
          ..Default::default()
        }));
      }
    }

    let mut seen = HashSet::new();

    for alias in ctx.aliases() {
      if !seen.insert(alias.name.value.clone()) {
        diagnostics.push(self.diagnostic(lsp::Diagnostic {
          range: alias.range,
          severity: Some(lsp::DiagnosticSeverity::ERROR),
          message: format!("Duplicate alias `{}`", alias.name.value),
          ..Default::default()
        }));
      }
    }

    diagnostics
  }
}
