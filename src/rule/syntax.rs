use super::*;

/// Surfaces tree-sitter syntax errors and missing nodes so users get feedback
/// on malformed `justfile` syntax before other rules run.
pub(crate) struct SyntaxRule;

impl Rule for SyntaxRule {
  fn display_name(&self) -> &'static str {
    "Syntax Errors"
  }

  fn id(&self) -> &'static str {
    "syntax-errors"
  }

  fn run(&self, context: &RuleContext<'_>) -> Vec<lsp::Diagnostic> {
    let mut diagnostics = Vec::new();

    if let Some(tree) = context.tree() {
      let document = context.document();
      let mut cursor = tree.root_node().walk();
      self.collect(document, &mut cursor, &mut diagnostics);
    }

    diagnostics
  }
}

impl SyntaxRule {
  fn collect(
    &self,
    document: &Document,
    cursor: &mut TreeCursor<'_>,
    diagnostics: &mut Vec<lsp::Diagnostic>,
  ) {
    let node = cursor.node();

    if node.is_error() {
      diagnostics.push(self.diagnostic(lsp::Diagnostic {
        range: node.get_range(document),
        severity: Some(lsp::DiagnosticSeverity::ERROR),
        message: "Syntax error".to_string(),
        ..Default::default()
      }));
    }

    if node.is_missing() {
      diagnostics.push(self.diagnostic(lsp::Diagnostic {
        range: node.get_range(document),
        severity: Some(lsp::DiagnosticSeverity::ERROR),
        message: "Missing syntax element".to_string(),
        ..Default::default()
      }));
    }

    if cursor.goto_first_child() {
      loop {
        self.collect(document, cursor, diagnostics);

        if !cursor.goto_next_sibling() {
          break;
        }
      }

      cursor.goto_parent();
    }
  }
}
