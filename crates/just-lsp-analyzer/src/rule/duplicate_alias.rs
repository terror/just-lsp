use super::*;

/// Flags alias declarations that reuse the same name multiple times.
pub struct DuplicateAliasRule;

impl Rule for DuplicateAliasRule {
  fn display_name(&self) -> &'static str {
    "Duplicate Alias"
  }

  fn id(&self) -> &'static str {
    "duplicate-alias"
  }

  fn run(&self, context: &RuleContext<'_>) -> Vec<lsp::Diagnostic> {
    let mut diagnostics = Vec::new();

    let mut seen = HashSet::new();

    for alias in context.aliases() {
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
