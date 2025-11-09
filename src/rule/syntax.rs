use super::*;

pub struct SyntaxRule;

impl Rule for SyntaxRule {
  fn id(&self) -> &'static str {
    "syntax-errors"
  }

  fn display_name(&self) -> &'static str {
    "Syntax Errors"
  }

  fn run(&self, ctx: &RuleContext<'_>) -> Vec<lsp::Diagnostic> {
    let mut diagnostics = Vec::new();

    if let Some(tree) = ctx.tree() {
      let mut cursor = tree.root_node().walk();
      self.collect(&mut cursor, &mut diagnostics);
    }

    diagnostics
  }
}

impl SyntaxRule {
  fn collect(
    &self,
    cursor: &mut TreeCursor<'_>,
    diagnostics: &mut Vec<lsp::Diagnostic>,
  ) {
    let node = cursor.node();

    if node.is_error() {
      diagnostics.push(self.diagnostic(lsp::Diagnostic {
        range: node.get_range(),
        severity: Some(lsp::DiagnosticSeverity::ERROR),
        message: "Syntax error".to_string(),
        ..Default::default()
      }));
    }

    if node.is_missing() {
      diagnostics.push(self.diagnostic(lsp::Diagnostic {
        range: node.get_range(),
        severity: Some(lsp::DiagnosticSeverity::ERROR),
        message: "Missing syntax element".to_string(),
        ..Default::default()
      }));
    }

    if cursor.goto_first_child() {
      loop {
        self.collect(cursor, diagnostics);

        if !cursor.goto_next_sibling() {
          break;
        }
      }

      cursor.goto_parent();
    }
  }
}
