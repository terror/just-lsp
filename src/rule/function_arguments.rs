use super::*;

/// Verifies builtin function calls use a valid argument count and respect
/// variadic constraints.
pub struct FunctionArgumentsRule;

impl Rule for FunctionArgumentsRule {
  fn id(&self) -> &'static str {
    "function-arguments"
  }

  fn display_name(&self) -> &'static str {
    "Function Arguments"
  }

  fn run(&self, context: &RuleContext<'_>) -> Vec<lsp::Diagnostic> {
    let mut diagnostics = Vec::new();

    let root = match context.tree() {
      Some(tree) => tree.root_node(),
      None => return diagnostics,
    };

    let document = context.document();

    for function_call in root.find_all("function_call") {
      if let Some(identifier_node) = function_call.find("identifier") {
        let function_name = document.get_node_text(&identifier_node);

        let builtin = builtins::BUILTINS.iter().find(|f| {
          matches!(f, Builtin::Function { name, .. } if *name == function_name)
        });

        if let Some(Builtin::Function {
          required_args,
          accepts_variadic,
          ..
        }) = builtin
        {
          let arg_count = Self::argument_count(&function_call);

          if arg_count < *required_args {
            diagnostics.push(self.diagnostic(lsp::Diagnostic {
              range: function_call.get_range(),
              severity: Some(lsp::DiagnosticSeverity::ERROR),
              message: format!(
                "Function `{function_name}` requires at least {required_args} {}, but {arg_count} provided",
                Count("argument", *required_args)
              ),
              ..Default::default()
            }));
          } else if !accepts_variadic && arg_count > *required_args {
            diagnostics.push(self.diagnostic(lsp::Diagnostic {
              range: function_call.get_range(),
              severity: Some(lsp::DiagnosticSeverity::ERROR),
              message: format!(
                "Function `{function_name}` accepts {required_args} {}, but {arg_count} provided",
                Count("argument", *required_args)
              ),
              ..Default::default()
            }));
          }
        }
      }
    }

    diagnostics
  }
}

impl FunctionArgumentsRule {
  fn argument_count(function_call: &Node) -> usize {
    function_call
      .find("sequence")
      .map(|sequence| {
        (0..sequence.child_count())
          .filter_map(|i| sequence.child(i))
          .filter(|child| child.kind() == "expression")
          .count()
      })
      .unwrap_or(0)
  }
}
