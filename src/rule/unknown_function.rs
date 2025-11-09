use super::*;

/// Ensures every function call references a builtin function recognized by
/// `just`.
pub struct UnknownFunctionRule;

impl Rule for UnknownFunctionRule {
  fn id(&self) -> &'static str {
    "unknown-function"
  }

  fn display_name(&self) -> &'static str {
    "Unknown Function"
  }

  fn run(&self, ctx: &RuleContext<'_>) -> Vec<lsp::Diagnostic> {
    let mut diagnostics = Vec::new();

    let root = match ctx.tree() {
      Some(tree) => tree.root_node(),
      None => return diagnostics,
    };

    let document = ctx.document();

    for function_call in root.find_all("function_call") {
      if let Some(identifier_node) = function_call.find("identifier") {
        let function_name = document.get_node_text(&identifier_node);

        let is_builtin = builtins::BUILTINS.iter().any(|f| {
          matches!(f, Builtin::Function { name, .. } if *name == function_name)
        });

        if !is_builtin {
          diagnostics.push(self.diagnostic(lsp::Diagnostic {
            range: identifier_node.get_range(),
            severity: Some(lsp::DiagnosticSeverity::ERROR),
            message: format!("Unknown function `{function_name}`"),
            ..Default::default()
          }));
        }
      }
    }

    diagnostics
  }
}
