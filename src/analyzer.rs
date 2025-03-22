use super::*;

#[derive(Debug)]
pub struct Analyzer<'a> {
  document: &'a Document,
}

impl<'a> Analyzer<'a> {
  pub(crate) fn new(document: &'a Document) -> Self {
    Self { document }
  }

  pub(crate) fn analyze(&self) -> Vec<lsp::Diagnostic> {
    self
      .aggregate_parser_errors()
      .into_iter()
      .chain(self.analyze_aliases())
      .chain(self.analyze_attributes())
      .chain(self.analyze_function_calls())
      .chain(self.analyze_recipes())
      .chain(self.analyze_settings())
      .collect()
  }

  fn aggregate_parser_errors(&self) -> Vec<lsp::Diagnostic> {
    let mut diagnostics = Vec::new();

    if let Some(tree) = &self.document.tree {
      let mut cursor = tree.root_node().walk();
      Self::aggregate_parser_errors_rec(&mut cursor, &mut diagnostics);
    }

    diagnostics
  }

  fn aggregate_parser_errors_rec(
    cursor: &mut TreeCursor<'_>,
    diagnostics: &mut Vec<lsp::Diagnostic>,
  ) {
    let node = cursor.node();

    if node.is_error() {
      diagnostics.push(lsp::Diagnostic {
        range: node.get_range(),
        severity: Some(lsp::DiagnosticSeverity::ERROR),
        source: Some("just-lsp".to_string()),
        message: "Syntax error".to_string(),
        ..Default::default()
      });
    }

    if node.is_missing() {
      diagnostics.push(lsp::Diagnostic {
        range: node.get_range(),
        severity: Some(lsp::DiagnosticSeverity::ERROR),
        source: Some("just-lsp".to_string()),
        message: "Missing syntax element".to_string(),
        ..Default::default()
      });
    }

    if cursor.goto_first_child() {
      loop {
        Self::aggregate_parser_errors_rec(cursor, diagnostics);

        if !cursor.goto_next_sibling() {
          break;
        }
      }

      cursor.goto_parent();
    }
  }

  fn analyze_aliases(&self) -> Vec<lsp::Diagnostic> {
    let recipe_names = self
      .document
      .get_recipes()
      .iter()
      .map(|recipe| recipe.name.clone())
      .collect::<HashSet<_>>();

    let aliases = self.document.get_aliases();

    let mut diagnostics = Vec::new();

    for alias in &aliases {
      if !recipe_names.contains(&alias.value.value) {
        diagnostics.push(lsp::Diagnostic {
          range: alias.value.range,
          severity: Some(lsp::DiagnosticSeverity::ERROR),
          source: Some("just-lsp".to_string()),
          message: format!("Recipe '{}' not found", alias.value.value),
          ..Default::default()
        });
      }
    }

    let mut seen = HashSet::new();

    for alias in &aliases {
      if !seen.insert(&alias.name.value) {
        diagnostics.push(lsp::Diagnostic {
          range: alias.range,
          severity: Some(lsp::DiagnosticSeverity::ERROR),
          source: Some("just-lsp".to_string()),
          message: format!("Duplicate alias '{}'", alias.name.value),
          ..Default::default()
        });
      }
    }

    diagnostics
  }

  fn analyze_attributes(&self) -> Vec<lsp::Diagnostic> {
    let mut diagnostics = Vec::new();

    let root = match &self.document.tree {
      Some(tree) => tree.root_node(),
      None => return diagnostics,
    };

    let attribute_nodes = root.find_all("attribute");

    for attribute_node in attribute_nodes {
      if let Some(name_node) = attribute_node.find("identifier") {
        let attribute_name = self.document.get_node_text(&name_node);

        let matching_attributes: Vec<_> = builtins::BUILTINS
          .iter()
          .filter(|f| matches!(f, Builtin::Attribute { name, .. } if *name == attribute_name))
          .collect();

        if matching_attributes.is_empty() {
          diagnostics.push(lsp::Diagnostic {
            range: name_node.get_range(),
            severity: Some(lsp::DiagnosticSeverity::ERROR),
            source: Some("just-lsp".to_string()),
            message: format!("Unknown attribute '{}'", attribute_name),
            ..Default::default()
          });

          continue;
        }

        let has_parameters = attribute_node.child_count() > 2
          && attribute_node.find("string").is_some();

        let parameter_mismatch = matching_attributes.iter().all(|attr| {
          if let Builtin::Attribute { parameters, .. } = attr {
            (parameters.is_some() && !has_parameters)
              || (parameters.is_none() && has_parameters)
          } else {
            false
          }
        });

        if parameter_mismatch {
          let param_error_msg = if matching_attributes.iter().any(|attr| {
            matches!(attr, Builtin::Attribute { parameters, .. } if parameters.is_some())
          }) {
            format!("Attribute '{}' requires parameters", attribute_name)
          } else {
            format!("Attribute '{}' doesn't accept parameters", attribute_name)
          };

          diagnostics.push(lsp::Diagnostic {
            range: attribute_node.get_range(),
            severity: Some(lsp::DiagnosticSeverity::ERROR),
            source: Some("just-lsp".to_string()),
            message: param_error_msg,
            ..Default::default()
          });

          continue;
        }

        if let Some(parent) = attribute_node.parent() {
          let target_type = match parent.kind() {
            "alias" => AttributeTarget::Alias,
            "module" => AttributeTarget::Module,
            "recipe" => AttributeTarget::Recipe,
            _ => {
              diagnostics.push(lsp::Diagnostic {
                range: attribute_node.get_range(),
                severity: Some(lsp::DiagnosticSeverity::ERROR),
                source: Some("just-lsp".to_string()),
                message: format!(
                  "Attribute '{}' applied to invalid target",
                  attribute_name
                ),
                ..Default::default()
              });

              continue;
            }
          };

          if !matching_attributes.iter().any(|attr| {
            if let Builtin::Attribute { targets, .. } = attr {
              targets
                .iter()
                .any(|target| target.is_valid_for(target_type))
            } else {
              false
            }
          }) {
            diagnostics.push(lsp::Diagnostic {
            range: attribute_node.get_range(),
            severity: Some(lsp::DiagnosticSeverity::ERROR),
            source: Some("just-lsp".to_string()),
            message: format!(
              "Attribute '{attribute_name}' cannot be applied to {target_type} target",
            ),
            ..Default::default()
          });
          }
        }
      }
    }

    diagnostics
  }

  fn analyze_function_calls(&self) -> Vec<lsp::Diagnostic> {
    let mut diagnostics = Vec::new();

    let root = match &self.document.tree {
      Some(tree) => tree.root_node(),
      None => return diagnostics,
    };

    let function_calls = root.find_all("function_call");

    for function_call in function_calls {
      if let Some(identifier_node) = function_call.find("identifier") {
        let function_name = self.document.get_node_text(&identifier_node);

        let builtin = builtins::BUILTINS
          .iter()
          .find(|f| matches!(f, Builtin::Function { name, .. } if *name == function_name));

        if let Some(Builtin::Function {
          required_args,
          accepts_variadic,
          ..
        }) = builtin
        {
          let arguments = function_call.find("sequence");

          let arg_count = arguments.map_or(0, |args| args.named_child_count());

          if arg_count < *required_args {
            diagnostics.push(
              lsp::Diagnostic {
                range: function_call.get_range(),
                severity: Some(lsp::DiagnosticSeverity::ERROR),
                source: Some("just-lsp".to_string()),
                message: format!(
                "Function '{}' requires at least {} argument(s), but {} provided",
                function_name, required_args, arg_count
              ),
                ..Default::default()
            });
          } else if !accepts_variadic && arg_count > *required_args {
            diagnostics.push(lsp::Diagnostic {
              range: function_call.get_range(),
              severity: Some(lsp::DiagnosticSeverity::ERROR),
              source: Some("just-lsp".to_string()),
              message: format!(
                "Function '{}' accepts {} argument(s), but {} provided",
                function_name, required_args, arg_count
              ),
              ..Default::default()
            });
          }
        } else {
          diagnostics.push(lsp::Diagnostic {
            range: identifier_node.get_range(),
            severity: Some(lsp::DiagnosticSeverity::ERROR),
            source: Some("just-lsp".to_string()),
            message: format!("Unknown function '{}'", function_name),
            ..Default::default()
          });
        }
      }
    }

    diagnostics
  }

  fn analyze_recipes(&self) -> Vec<lsp::Diagnostic> {
    let mut diagnostics = Vec::new();

    let recipes = self.document.get_recipes();

    let mut recipe_names = HashSet::new();

    for recipe in &recipes {
      if !recipe_names.insert(&recipe.name) {
        diagnostics.push(lsp::Diagnostic {
          range: recipe.range,
          severity: Some(lsp::DiagnosticSeverity::ERROR),
          source: Some("just-lsp".to_string()),
          message: format!("Duplicate recipe name '{}'", recipe.name),
          ..Default::default()
        });
      }
    }

    for recipe in &recipes {
      let mut seen = HashSet::new();

      let (mut passed_default, mut passed_variadic) = (false, false);

      for (index, param) in recipe.parameters.iter().enumerate() {
        if !seen.insert(param.name.clone()) {
          diagnostics.push(lsp::Diagnostic {
            range: param.range,
            severity: Some(lsp::DiagnosticSeverity::ERROR),
            source: Some("just-lsp".to_string()),
            message: format!(
              "Duplicate parameter '{}' in recipe '{}'",
              param.name, recipe.name
            ),
            ..Default::default()
          });
        }

        let has_default = param.default_value.is_some();

        if matches!(param.kind, ParameterKind::Variadic(_)) {
          if index < recipe.parameters.len() - 1 {
            diagnostics.push(lsp::Diagnostic {
              range: param.range,
              severity: Some(lsp::DiagnosticSeverity::ERROR),
              source: Some("just-lsp".to_string()),
              message: format!(
                "Variadic parameter '{}' must be the last parameter",
                param.name
              ),
              ..Default::default()
            });
          }

          passed_variadic = true;
        }

        if passed_default
          && !has_default
          && !matches!(param.kind, ParameterKind::Variadic(_))
        {
          diagnostics.push(lsp::Diagnostic {
            range: param.range,
            severity: Some(lsp::DiagnosticSeverity::ERROR),
            source: Some("just-lsp".to_string()),
            message: format!("Required parameter '{}' follows a parameter with a default value", param.name),
            ..Default::default()
          });
        }

        if passed_variadic && index < recipe.parameters.len() - 1 {
          diagnostics.push(lsp::Diagnostic {
            range: param.range,
            severity: Some(lsp::DiagnosticSeverity::ERROR),
            source: Some("just-lsp".to_string()),
            message: format!(
              "Parameter '{}' follows a variadic parameter",
              param.name
            ),
            ..Default::default()
          });
        }

        if has_default {
          passed_default = true;
        }
      }
    }

    let variables = self
      .document
      .get_variables()
      .iter()
      .map(|variable| variable.name.value.clone())
      .collect::<HashSet<_>>();

    let recipe_parameters = recipes
      .iter()
      .map(|recipe| (recipe.name.clone(), recipe.parameters.clone()))
      .collect::<HashMap<String, Vec<Parameter>>>();

    for recipe in &recipes {
      for dependency in &recipe.dependencies {
        if !recipe_names.contains(&dependency.name) {
          diagnostics.push(lsp::Diagnostic {
            range: dependency.range,
            severity: Some(lsp::DiagnosticSeverity::ERROR),
            source: Some("just-lsp".to_string()),
            message: format!("Recipe '{}' not found", dependency.name),
            ..Default::default()
          });
        }

        for argument in &dependency.arguments {
          let parameters = recipe
            .parameters
            .iter()
            .map(|p| p.name.clone())
            .collect::<HashSet<_>>();

          if !argument.is_quoted()
            && !variables.contains(&argument.value)
            && !parameters.contains(&argument.value)
          {
            diagnostics.push(lsp::Diagnostic {
              range: argument.range,
              severity: Some(lsp::DiagnosticSeverity::ERROR),
              source: Some("just-lsp".to_string()),
              message: format!("Variable '{}' not found", argument.value),
              ..Default::default()
            });
          }
        }

        if let Some(params) = recipe_parameters.get(&dependency.name) {
          let required_params = params
            .iter()
            .filter(|p| {
              p.default_value.is_none()
                && !matches!(p.kind, ParameterKind::Variadic(_))
            })
            .count();

          let has_variadic = params
            .iter()
            .any(|p| matches!(p.kind, ParameterKind::Variadic(_)));

          let total_params = params.len();

          let arg_count = dependency.arguments.len();

          if arg_count < required_params {
            diagnostics.push(lsp::Diagnostic {
              range: dependency.range,
              severity: Some(lsp::DiagnosticSeverity::ERROR),
              source: Some("just-lsp".to_string()),
              message: format!(
                "Dependency '{}' requires {} argument(s), but {} provided",
                dependency.name, required_params, arg_count
              ),
              ..Default::default()
            });
          } else if !has_variadic && arg_count > total_params {
            diagnostics.push(lsp::Diagnostic {
              range: dependency.range,
              severity: Some(lsp::DiagnosticSeverity::ERROR),
              source: Some("just-lsp".to_string()),
              message: format!(
                "Dependency '{}' accepts {} argument(s), but {} provided",
                dependency.name, total_params, arg_count
              ),
              ..Default::default()
            });
          }
        }
      }
    }

    diagnostics
  }

  fn analyze_settings(&self) -> Vec<lsp::Diagnostic> {
    let mut diagnostics = Vec::new();

    let settings = self.document.get_settings();

    for setting in &settings {
      let builtin = builtins::BUILTINS.iter().find(
        |f| matches!(f, Builtin::Setting { name, .. } if *name == setting.name),
      );

      if let Some(Builtin::Setting { kind, .. }) = builtin {
        if setting.kind != *kind {
          diagnostics.push(lsp::Diagnostic {
            range: setting.range,
            severity: Some(lsp::DiagnosticSeverity::ERROR),
            source: Some("just-lsp".to_string()),
            message: format!(
              "Setting '{}' expects a {kind} value",
              setting.name,
            ),
            ..Default::default()
          });
        }
      } else {
        diagnostics.push(lsp::Diagnostic {
          range: setting.range,
          severity: Some(lsp::DiagnosticSeverity::ERROR),
          source: Some("just-lsp".to_string()),
          message: format!("Unknown setting '{}'", setting.name),
          ..Default::default()
        });
      }
    }

    let mut seen = HashSet::new();

    for setting in settings {
      if !seen.insert(setting.name.clone()) {
        diagnostics.push(lsp::Diagnostic {
          range: setting.range,
          severity: Some(lsp::DiagnosticSeverity::ERROR),
          source: Some("just-lsp".to_string()),
          message: format!("Duplicate setting '{}'", setting.name),
          ..Default::default()
        });
      }
    }

    diagnostics
  }
}

#[cfg(test)]
mod tests {
  use {super::*, indoc::indoc, pretty_assertions::assert_eq};

  fn document(content: &str) -> Document {
    Document::try_from(lsp::DidOpenTextDocumentParams {
      text_document: lsp::TextDocumentItem {
        uri: lsp::Url::parse("file:///test.just").unwrap(),
        language_id: "just".to_string(),
        version: 1,
        text: content.to_string(),
      },
    })
    .unwrap()
  }

  #[test]
  fn aliases_basic() {
    let doc = document(indoc! {"
      foo:
        echo \"foo\"

      alias bar := foo
    "});

    let analyzer = Analyzer::new(&doc);

    let diagnostics = analyzer.analyze();

    assert_eq!(
      diagnostics.len(),
      0,
      "Valid alias should not produce diagnostics"
    );
  }

  #[test]
  fn aliases_duplicate() {
    let doc = document(indoc! {"
      foo:
        echo \"foo\"

      alias bar := foo
      alias bar := foo
      alias bar := foo
    "});

    let analyzer = Analyzer::new(&doc);

    let diagnostics = analyzer.analyze();

    assert_eq!(diagnostics.len(), 2);
    assert_eq!(diagnostics[0].message, "Duplicate alias 'bar'");
    assert_eq!(diagnostics[1].message, "Duplicate alias 'bar'");
  }

  #[test]
  fn aliases_missing_recipe() {
    let doc = document(indoc! {"
      foo:
        echo \"foo\"

      alias bar := baz
    "});

    let analyzer = Analyzer::new(&doc);

    let diagnostics = analyzer.analyze();

    assert_eq!(diagnostics.len(), 1);
    assert_eq!(diagnostics[0].message, "Recipe 'baz' not found");
  }

  #[test]
  fn analyze_complete() {
    let doc = document(indoc! {
      "
      foo:
        echo \"foo\"

      bar: missing
        echo \"bar\"

      alias baz := nonexistent
      "
    });

    let analyzer = Analyzer::new(&doc);

    let diagnostics = analyzer.analyze();

    assert_eq!(diagnostics.len(), 2);

    let messages: Vec<String> =
      diagnostics.iter().map(|d| d.message.clone()).collect();

    assert!(messages.contains(&"Recipe 'missing' not found".to_string()));
    assert!(messages.contains(&"Recipe 'nonexistent' not found".to_string()));
  }

  #[test]
  fn attributes_correct() {
    let doc = document(indoc! {
      "
      [no-cd]
      [linux]
      [macos]
      foo:
        echo \"foo\"

      [doc('Recipe documentation')]
      bar:
        echo \"bar\"
      "
    });

    let analyzer = Analyzer::new(&doc);

    let diagnostics = analyzer.analyze();

    assert_eq!(
      diagnostics.len(),
      0,
      "Valid attributes should not produce diagnostics"
    );
  }

  #[test]
  fn attributes_extra_parameters() {
    let doc = document(indoc! {
      "
      [linux('invalid')]
      foo:
        echo \"foo\"
      "
    });

    let analyzer = Analyzer::new(&doc);

    let diagnostics = analyzer.analyze();

    assert_eq!(diagnostics.len(), 1);

    assert_eq!(
      diagnostics[0].message,
      "Attribute 'linux' doesn't accept parameters"
    );
  }

  #[test]
  fn attributes_missing_parameters() {
    let doc = document(indoc! {
      "
      [doc]
      foo:
        echo \"foo\"
      "
    });

    let analyzer = Analyzer::new(&doc);

    let diagnostics = analyzer.analyze();

    assert_eq!(diagnostics.len(), 1);

    assert_eq!(
      diagnostics[0].message,
      "Attribute 'doc' requires parameters"
    );
  }

  #[test]
  fn attributes_no_parameters_needed() {
    let doc = document(indoc! {
      "
      [script]
      foo:
        echo \"foo\"
      "
    });

    let analyzer = Analyzer::new(&doc);

    let diagnostics = analyzer.analyze();

    assert_eq!(
      diagnostics.len(),
      0,
      "Should be valid since script can be used without parameters"
    );

    let doc = document(indoc! {
      "
      [confirm]
      foo:
        echo \"foo\"
      "
    });

    let analyzer = Analyzer::new(&doc);

    let diagnostics = analyzer.analyze();

    assert_eq!(
      diagnostics.len(),
      0,
      "Should be valid since confirm can be used without parameters"
    );
  }

  #[test]
  fn attributes_unknown() {
    let doc = document(indoc! {
      "
      [unknown_attribute]
      foo:
        echo \"foo\"
      "
    });

    let analyzer = Analyzer::new(&doc);

    let diagnostics = analyzer.analyze();

    assert_eq!(diagnostics.len(), 1);

    assert_eq!(
      diagnostics[0].message,
      "Unknown attribute 'unknown_attribute'"
    );
  }

  #[test]
  #[ignore]
  fn attributes_wrong_target() {
    let doc = document(indoc! {
      "
      [linux]
      set export := true
      "
    });

    let analyzer = Analyzer::new(&doc);

    let diagnostics = analyzer.analyze();

    assert_eq!(diagnostics.len(), 1);

    assert!(diagnostics[0]
      .message
      .contains("cannot be applied to variable target"));
  }

  #[test]
  fn function_calls_correct() {
    let doc = document(indoc! {
      "
      foo:
        echo {{ arch() }}
        echo {{ join(\"a\", \"b\", \"c\") }}
      "
    });

    let analyzer = Analyzer::new(&doc);

    let diagnostics = analyzer.analyze();

    assert_eq!(
      diagnostics.len(),
      0,
      "Valid function calls should not produce diagnostics"
    );
  }

  #[test]
  fn function_calls_too_few_args() {
    let doc = document(indoc! {
      "
      foo:
        echo {{ replace() }}
      "
    });

    let analyzer = Analyzer::new(&doc);

    let diagnostics = analyzer.analyze();

    assert!(
      !diagnostics.is_empty(),
      "Should have at least one diagnostic"
    );

    assert!(
      diagnostics
        .iter()
        .any(|d| d.message.contains("requires at least 3 argument(s)")),
      "Should have diagnostic about missing arguments"
    );
  }

  #[test]
  fn function_calls_too_many_args() {
    let doc = document(indoc! {
      "
      foo:
        echo {{ uppercase(\"hello\", \"extra\") }}
      "
    });

    let analyzer = Analyzer::new(&doc);

    let diagnostics = analyzer.analyze();

    assert!(
      !diagnostics.is_empty(),
      "Should have at least one diagnostic"
    );

    assert!(
      diagnostics
        .iter()
        .any(|d| d.message.contains("accepts 1 argument(s)")),
      "Should have diagnostic about too many arguments"
    );
  }

  #[test]
  fn function_calls_unknown() {
    let doc = document(indoc! {
      "
      foo:
        echo {{ unknown_function() }}
      "
    });

    let analyzer = Analyzer::new(&doc);

    let diagnostics = analyzer.analyze();

    assert!(
      !diagnostics.is_empty(),
      "Should have at least one diagnostic"
    );

    assert!(
      diagnostics
        .iter()
        .any(|d| d.message.contains("Unknown function 'unknown_function'")),
      "Should have diagnostic about unknown function"
    );
  }

  #[test]
  fn parser_errors_invalid() {
    let doc = document(indoc! {
      "
      foo
        echo \"foo\"
      "
    });

    let analyzer = Analyzer::new(&doc);

    let diagnostics = analyzer.analyze();

    assert!(!diagnostics.is_empty(), "Should detect syntax errors");

    let syntax_errors = diagnostics
      .iter()
      .filter(|d| {
        d.message.contains("Syntax error")
          || d.message.contains("Missing syntax")
      })
      .collect::<Vec<_>>();

    assert!(
      !syntax_errors.is_empty(),
      "Should have at least one syntax error diagnostic"
    );
  }

  #[test]
  fn parser_errors_valid() {
    let valid_doc = document(indoc! {
      "
      foo:
        echo \"foo\"
      "
    });

    let analyzer = Analyzer::new(&valid_doc);

    let valid_diagnostics = analyzer.analyze();

    assert!(
      valid_diagnostics.is_empty(),
      "Valid document should not have syntax errors"
    );
  }

  #[test]
  fn recipe_dependencies_correct() {
    let doc = document(indoc! {"
      foo:
        echo \"foo\"

      bar: foo
        echo \"bar\"
    "});

    let analyzer = Analyzer::new(&doc);

    let diagnostics = analyzer.analyze();

    assert_eq!(diagnostics.len(), 0);
  }

  #[test]
  fn recipe_dependencies_missing() {
    let doc = document(indoc! {"
      foo:
        echo \"foo\"

      bar: baz
        echo \"bar\"
    "});

    let analyzer = Analyzer::new(&doc);

    let diagnostics = analyzer.analyze();

    assert_eq!(diagnostics.len(), 1);
    assert_eq!(diagnostics[0].message, "Recipe 'baz' not found");
    assert_eq!(diagnostics[0].range.start.line, 3);
  }

  #[test]
  fn recipe_dependencies_multiple_missing() {
    let doc = document(indoc! {"
      foo:
        echo \"foo\"

      bar: missing1 missing2
        echo \"bar\"
    "});

    let analyzer = Analyzer::new(&doc);

    let diagnostics = analyzer.analyze();

    assert_eq!(diagnostics.len(), 2);
  }

  #[test]
  fn recipe_invocation_argument_count_correct() {
    let doc = document(indoc! {
      "
      foo arg1 arg2=\"default\":
        echo \"{{arg1}} {{arg2}}\"

      bar: (foo 'value1')
        echo \"bar\"
      "
    });

    let analyzer = Analyzer::new(&doc);

    let diagnostics = analyzer.analyze();

    assert_eq!(
      diagnostics.len(),
      0,
      "Should not have errors when default values are used"
    );
  }

  #[test]
  fn recipe_invocation_missing_args() {
    let doc = document(indoc! {
      "
      foo arg1 arg2:
        echo \"{{arg1}} {{arg2}}\"

      bar: (foo)
        echo \"bar\"
      "
    });

    let analyzer = Analyzer::new(&doc);

    let diagnostics = analyzer.analyze();

    assert_eq!(diagnostics.len(), 1);

    assert!(diagnostics[0]
      .message
      .contains("requires 2 argument(s), but 0 provided"));
  }

  #[test]
  fn recipe_invocation_too_few_args() {
    let doc = document(indoc! {
      "
      foo arg1 arg2:
        echo \"{{arg1}} {{arg2}}\"

      bar: (foo 'value1')
        echo \"bar\"
      "
    });

    let analyzer = Analyzer::new(&doc);

    let diagnostics = analyzer.analyze();

    assert_eq!(diagnostics.len(), 1);

    assert!(diagnostics[0]
      .message
      .contains("requires 2 argument(s), but 1 provided"));
  }

  #[test]
  fn recipe_invocation_too_many_args() {
    let doc = document(indoc! {
      "
      foo arg1:
        echo \"{{arg1}}\"

      bar: (foo 'value1' 'value2' 'value3')
        echo \"bar\"
      "
    });

    let analyzer = Analyzer::new(&doc);

    let diagnostics = analyzer.analyze();

    assert_eq!(diagnostics.len(), 1);

    assert!(diagnostics[0]
      .message
      .contains("accepts 1 argument(s), but 3 provided"));
  }

  #[test]
  fn recipe_invocation_unknown_variable() {
    let doc = document(indoc! {
      "
      foo arg1:
        echo \"{{arg1}} {{arg2}}\"

      bar: (foo wow)
        echo \"bar\"
      "
    });

    let analyzer = Analyzer::new(&doc);

    let diagnostics = analyzer.analyze();

    assert_eq!(diagnostics.len(), 1);

    assert!(diagnostics[0].message.contains("Variable 'wow' not found"));
  }

  #[test]
  fn recipe_invocation_valid_variable() {
    let doc = document(indoc! {
      "
      wow := 'foo'

      foo arg1:
        echo \"{{arg1}} {{arg2}}\"

      bar: (foo wow)
        echo \"bar\"
      "
    });

    let analyzer = Analyzer::new(&doc);

    let diagnostics = analyzer.analyze();

    assert_eq!(diagnostics.len(), 0);
  }

  #[test]
  fn recipe_invocation_variadic_params() {
    let doc = document(indoc! {
      "
      foo arg1 +args:
        echo \"{{arg1}} {{args}}\"

      bar: (foo 'value1' 'value2' 'value3')
        echo \"bar\"
      "
    });

    let analyzer = Analyzer::new(&doc);

    let diagnostics = analyzer.analyze();

    assert_eq!(
      diagnostics.len(),
      0,
      "Should not have errors when variadic parameters are used"
    );
  }

  #[test]
  fn recipe_parameters_defaults_all() {
    let doc = document(indoc! {
      "
      recipe_with_defaults arg1=\"first\" arg2=\"second\":
        echo \"{{arg1}} {{arg2}}\"
      "
    });

    let analyzer = Analyzer::new(&doc);

    let diagnostics = analyzer.analyze();

    assert_eq!(
      diagnostics.len(),
      0,
      "Parameters with all defaults should not produce diagnostics"
    );
  }

  #[test]
  fn recipe_parameters_duplicate() {
    let doc = document(indoc! {
      "
      recipe_with_duplicate_param arg1 arg1:
        echo \"{{arg1}}\"
      "
    });

    let analyzer = Analyzer::new(&doc);

    let diagnostics = analyzer.analyze();

    assert_eq!(diagnostics.len(), 1);
    assert!(diagnostics[0].message.contains("Duplicate parameter"));
  }

  #[test]
  fn recipe_parameters_order() {
    let doc = document(indoc! {
      "
      recipe_with_param_order arg1=\"default\" arg2:
        echo \"{{arg1}} {{arg2}}\"
      "
    });

    let analyzer = Analyzer::new(&doc);

    let diagnostics = analyzer.analyze();

    assert_eq!(diagnostics.len(), 1);

    assert!(diagnostics[0].message.contains(
      "Required parameter 'arg2' follows a parameter with a default value"
    ));
  }

  #[test]
  fn recipe_parameters_valid() {
    let doc = document(indoc! {
      "
      valid_recipe arg1 arg2=\"default\":
        echo \"{{arg1}} {{arg2}}\"
      "
    });

    let analyzer = Analyzer::new(&doc);

    let diagnostics = analyzer.analyze();

    assert_eq!(
      diagnostics.len(),
      0,
      "Valid parameter order should not produce diagnostics"
    );
  }

  #[test]
  fn recipe_parameters_variadic() {
    let doc = document(indoc! {
      "
      recipe_with_variadic arg1=\"default\" +args:
        echo \"{{arg1}} {{args}}\"
      "
    });

    let analyzer = Analyzer::new(&doc);

    let diagnostics = analyzer.analyze();

    assert_eq!(
      diagnostics.len(),
      0,
      "Variadic parameter after default should not produce diagnostics"
    );
  }

  #[test]
  fn settings_boolean_shorthand() {
    let doc = document(indoc! {
      "
      set export

      foo:
        echo \"foo\"
      "
    });

    let analyzer = Analyzer::new(&doc);

    let diagnostics = analyzer.analyze();

    assert_eq!(
      diagnostics.len(),
      0,
      "Shorthand boolean syntax should be valid"
    );
  }

  #[test]
  fn settings_boolean_type_correct() {
    let doc = document(indoc! {
      "
      set export := true
      set dotenv-load := false

      foo:
        echo \"foo\"
      "
    });

    let analyzer = Analyzer::new(&doc);

    let diagnostics = analyzer.analyze();

    assert_eq!(
      diagnostics.len(),
      0,
      "Valid boolean values should not produce diagnostics"
    );
  }

  #[test]
  fn settings_boolean_type_error() {
    let doc = document(indoc! {
      "
      set export := 'foo'

      foo:
        echo \"foo\"
      "
    });

    let analyzer = Analyzer::new(&doc);

    let diagnostics = analyzer.analyze();

    assert_eq!(diagnostics.len(), 1);

    assert_eq!(
      diagnostics[0].message,
      "Setting 'export' expects a boolean value"
    );
  }

  #[test]
  fn settings_duplicate() {
    let doc = document(indoc! {
      "
      set export := true
      set shell := [\"bash\", \"-c\"]
      set export := false

      foo:
        echo \"foo\"
      "
    });

    let analyzer = Analyzer::new(&doc);

    let diagnostics = analyzer.analyze();

    assert_eq!(diagnostics.len(), 1);
    assert_eq!(diagnostics[0].message, "Duplicate setting 'export'");
  }

  #[test]
  fn settings_multiple_errors() {
    let doc = document(indoc! {
      "
      set unknown-setting := true
      set export := false
      set shell := ['bash']
      set export := false

      foo:
        echo \"foo\"
      "
    });

    let analyzer = Analyzer::new(&doc);

    let diagnostics = analyzer.analyze();

    assert_eq!(diagnostics.len(), 2, "Should detect all errors in settings");

    let messages: Vec<String> =
      diagnostics.iter().map(|d| d.message.clone()).collect();

    assert!(messages.contains(&"Unknown setting 'unknown-setting'".to_string()));
    assert!(messages.contains(&"Duplicate setting 'export'".to_string()));
  }

  #[test]
  fn settings_string_type_correct() {
    let doc = document(indoc! {
      "
      set dotenv-path := \".env.development\"

      foo:
        echo \"foo\"
      "
    });

    let analyzer = Analyzer::new(&doc);

    let diagnostics = analyzer.analyze();

    assert_eq!(
      diagnostics.len(),
      0,
      "Valid string value should not produce diagnostics"
    );
  }

  #[test]
  fn settings_string_type_error() {
    let doc = document(indoc! {
      "
      set dotenv-path := true

      foo:
        echo \"foo\"
      "
    });

    let analyzer = Analyzer::new(&doc);

    let diagnostics = analyzer.analyze();

    assert_eq!(diagnostics.len(), 1);

    assert_eq!(
      diagnostics[0].message,
      "Setting 'dotenv-path' expects a string value"
    );
  }

  #[test]
  fn settings_unknown() {
    let doc = document(indoc! {
      "
      set unknown-setting := true

      foo:
        echo \"foo\"
      "
    });

    let analyzer = Analyzer::new(&doc);

    let diagnostics = analyzer.analyze();

    assert_eq!(diagnostics.len(), 1);
    assert_eq!(diagnostics[0].message, "Unknown setting 'unknown-setting'");
  }

  #[test]
  fn should_recognize_recipe_parameters_in_dependency_arguments() {
    let doc = document(indoc! {
      "
      other-recipe var=\"else\":
        echo {{ var }}

      test var=\"something\": (other-recipe var)
      "
    });

    let analyzer = Analyzer::new(&doc);

    let diagnostics = analyzer.analyze();

    assert_eq!(diagnostics.len(), 0);
  }
}
