use super::*;

pub struct FunctionCallsRule;

impl Rule for FunctionCallsRule {
  fn id(&self) -> &'static str {
    "function-calls"
  }

  fn display_name(&self) -> &'static str {
    "Function Calls"
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

        let builtin = builtins::BUILTINS
          .iter()
          .find(|f| matches!(f, Builtin::Function { name, .. } if *name == function_name));

        if let Some(Builtin::Function {
          required_args,
          accepts_variadic,
          ..
        }) = builtin
        {
          let arguments = function_call
            .find("sequence")
            .map(|sequence| {
              (0..sequence.child_count())
                .filter_map(|i| sequence.child(i))
                .filter(|child| child.kind() == "expression")
                .collect::<Vec<_>>()
            })
            .unwrap_or_default();

          let arg_count = arguments.len();

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
        } else {
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
