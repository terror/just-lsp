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
      .chain(self.analyze_dependencies())
      .chain(self.analyze_function_calls())
      .chain(self.analyze_recipe_parameters())
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

    let alias_nodes = self.document.find_nodes_by_kind("alias");

    alias_nodes
      .into_iter()
      .filter_map(|alias_node| {
        self
          .document
          .find_child_by_kind_at_position(&alias_node, "identifier", 3)
          .filter(|identifier| {
            !recipe_names.contains(&self.document.get_node_text(identifier))
          })
          .map(|identifier| lsp::Diagnostic {
            range: alias_node.get_range(),
            severity: Some(lsp::DiagnosticSeverity::ERROR),
            code: None,
            code_description: None,
            source: Some("just-lsp".to_string()),
            message: format!(
              "Recipe '{}' not found",
              self.document.get_node_text(&identifier)
            ),
            related_information: None,
            tags: None,
            data: None,
          })
      })
      .collect()
  }

  fn analyze_attributes(&self) -> Vec<lsp::Diagnostic> {
    let mut diagnostics = Vec::new();

    let attribute_nodes = self.document.find_nodes_by_kind("attribute");

    for attribute_node in attribute_nodes {
      if let Some(name_node) = self
        .document
        .find_child_by_kind(&attribute_node, "identifier")
      {
        let attribute_name = self.document.get_node_text(&name_node);

        let attribute_matches: Vec<_> = constants::ATTRIBUTES
          .iter()
          .filter(|attr| attr.name == attribute_name)
          .collect();

        if attribute_matches.is_empty() {
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
          && self
            .document
            .find_child_by_kind(&attribute_node, "string")
            .is_some();

        for attr in &attribute_matches {
          if attr.parameters.is_some() && !has_parameters {
            if attribute_matches.iter().any(|a| a.parameters.is_none()) {
              continue;
            }

            diagnostics.push(lsp::Diagnostic {
              range: attribute_node.get_range(),
              severity: Some(lsp::DiagnosticSeverity::ERROR),
              source: Some("just-lsp".to_string()),
              message: format!(
                "Attribute '{}' requires parameters",
                attribute_name
              ),
              ..Default::default()
            });
            break;
          }

          if attr.parameters.is_none() && has_parameters {
            if attribute_matches.iter().any(|a| a.parameters.is_some()) {
              continue;
            }

            diagnostics.push(lsp::Diagnostic {
              range: attribute_node.get_range(),
              severity: Some(lsp::DiagnosticSeverity::ERROR),
              source: Some("just-lsp".to_string()),
              message: format!(
                "Attribute '{}' doesn't accept parameters",
                attribute_name
              ),
              ..Default::default()
            });

            break;
          }
        }

        if let Some(parent) = attribute_node.parent() {
          let target_type = match parent.kind() {
            "recipe" => AttributeTarget::Recipe,
            "module" => AttributeTarget::Module,
            "alias" => AttributeTarget::Alias,
            "assignment" => AttributeTarget::Variable,
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

          if !attribute_matches
            .iter()
            .any(|attr| attr.target.is_valid_for(target_type))
          {
            diagnostics.push(lsp::Diagnostic {
              range: attribute_node.get_range(),
              severity: Some(lsp::DiagnosticSeverity::ERROR),
              source: Some("just-lsp".to_string()),
              message: format!(
                "Attribute '{}' cannot be applied to {} target",
                attribute_name,
                target_type.as_str()
              ),
              ..Default::default()
            });
          }
        }
      }
    }

    diagnostics
  }

  fn analyze_dependencies(&self) -> Vec<lsp::Diagnostic> {
    let recipe_names = self
      .document
      .get_recipes()
      .iter()
      .map(|recipe| recipe.name.clone())
      .collect::<HashSet<_>>();

    let recipe_nodes = self.document.find_nodes_by_kind("recipe");

    recipe_nodes
      .iter()
      .flat_map(|recipe_node| {
        let recipe_header = self
          .document
          .find_child_by_kind(recipe_node, "recipe_header");

        let dependencies = recipe_header.as_ref().and_then(|header| {
          self.document.find_child_by_kind(header, "dependencies")
        });

        dependencies.map_or(Vec::new(), |dependencies| {
          (0..dependencies.named_child_count())
            .filter_map(|i| dependencies.named_child(i))
            .filter(|dependency| dependency.kind() == "dependency")
            .filter_map(|dependency| {
              let identifier =
                self.document.find_child_by_kind(&dependency, "identifier");

              identifier.map(|identifier| {
                let text = self.document.get_node_text(&identifier);

                (!recipe_names.contains(&text)).then_some(lsp::Diagnostic {
                  range: identifier.get_range(),
                  severity: Some(lsp::DiagnosticSeverity::ERROR),
                  source: Some("just-lsp".to_string()),
                  message: format!("Recipe '{}' not found", text),
                  ..Default::default()
                })
              })
            })
            .flatten()
            .collect::<Vec<_>>()
        })
      })
      .collect()
  }

  fn analyze_function_calls(&self) -> Vec<lsp::Diagnostic> {
    let mut diagnostics = Vec::new();

    let function_calls = self.document.find_nodes_by_kind("function_call");

    for function_call in function_calls {
      if let Some(name_node) = self
        .document
        .find_child_by_kind(&function_call, "identifier")
      {
        let function_name = self.document.get_node_text(&name_node);

        if let Some(function) = constants::FUNCTIONS
          .iter()
          .find(|f| f.name == function_name)
        {
          let arguments =
            self.document.find_child_by_kind(&function_call, "sequence");

          let arg_count = arguments.map_or(0, |args| args.named_child_count());

          if arg_count < function.required_args {
            diagnostics.push(
              lsp::Diagnostic {
                range: function_call.get_range(),
                severity: Some(lsp::DiagnosticSeverity::ERROR),
                source: Some("just-lsp".to_string()),
                message: format!(
                "Function '{}' requires at least {} argument(s), but {} provided",
                function_name, function.required_args, arg_count
              ),
                ..Default::default()
            });
          } else if !function.accepts_variadic
            && arg_count > function.required_args
          {
            diagnostics.push(lsp::Diagnostic {
              range: function_call.get_range(),
              severity: Some(lsp::DiagnosticSeverity::ERROR),
              source: Some("just-lsp".to_string()),
              message: format!(
                "Function '{}' accepts {} argument(s), but {} provided",
                function_name, function.required_args, arg_count
              ),
              ..Default::default()
            });
          }
        } else {
          diagnostics.push(lsp::Diagnostic {
            range: name_node.get_range(),
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

  fn analyze_recipe_parameters(&self) -> Vec<lsp::Diagnostic> {
    let mut diagnostics = Vec::new();

    let recipe_nodes = self.document.find_nodes_by_kind("recipe");

    for recipe_node in recipe_nodes {
      let recipe_name = self
        .document
        .find_child_by_kind(&recipe_node, "recipe_header")
        .as_ref()
        .and_then(|header| {
          self.document.find_child_by_kind(header, "identifier")
        })
        .map(|id| self.document.get_node_text(&id))
        .unwrap_or_else(|| "unknown".to_string());

      if let Some(header) = self
        .document
        .find_child_by_kind(&recipe_node, "recipe_header")
      {
        if let Some(parameter_list) =
          self.document.find_child_by_kind(&header, "parameters")
        {
          let mut parameters = Vec::new();

          let mut seen = HashSet::new();

          let mut passed_default = false;

          for i in 0..parameter_list.named_child_count() {
            if let Some(param) = parameter_list.named_child(i) {
              if param.kind() == "parameter" {
                let param_name_node =
                  self.document.find_child_by_kind(&param, "identifier");

                if let Some(param_name) = param_name_node {
                  let name_text = self.document.get_node_text(&param_name);

                  if !seen.insert(name_text.clone()) {
                    diagnostics.push(
                      lsp::Diagnostic {
                        range: param_name.get_range(),
                        severity: Some(lsp::DiagnosticSeverity::ERROR),
                        source: Some("just-lsp".to_string()),
                        message: format!("Duplicate parameter '{name_text}' in recipe '{recipe_name}'"),
                        ..Default::default()
                    });
                  }

                  let has_default =
                    self.document.find_child_by_kind(&param, "=").is_some();

                  if passed_default
                    && !has_default
                    && !name_text.starts_with('+')
                  {
                    diagnostics.push(
                      lsp::Diagnostic {
                        range: param_name.get_range(),
                        severity: Some(lsp::DiagnosticSeverity::ERROR),
                        source: Some("just-lsp".to_string()),
                        message: format!("Required parameter '{}' follows a parameter with a default value", name_text),
                        ..Default::default()
                    });
                  }

                  if has_default {
                    passed_default = true;
                  }

                  parameters.push((name_text, has_default));
                }
              }
            }
          }
        }
      }
    }

    diagnostics
  }

  fn analyze_settings(&self) -> Vec<lsp::Diagnostic> {
    let mut diagnostics = Vec::new();

    let setting_nodes = self.document.find_nodes_by_kind("setting");

    for setting_node in setting_nodes.clone() {
      if let Some(name_node) = self
        .document
        .find_child_by_kind(&setting_node, "identifier")
      {
        let setting_name = self.document.get_node_text(&name_node);

        let valid_setting = constants::SETTINGS
          .iter()
          .find(|setting| setting.name == setting_name);

        if let Some(setting) = valid_setting {
          if let Some(setting_value_node) = setting_node.child(3) {
            let (setting_value_kind, setting_value) = (
              setting_value_node.kind(),
              self.document.get_node_text(&setting_value_node),
            );

            match setting.kind {
              SettingKind::Boolean => {
                if setting_value_kind != "boolean" {
                  diagnostics.push(lsp::Diagnostic {
                    range: setting_value_node.get_range(),
                    severity: Some(lsp::DiagnosticSeverity::ERROR),
                    source: Some("just-lsp".to_string()),
                    message: format!(
                      "Setting '{}' expects a boolean value",
                      setting_name
                    ),
                    ..Default::default()
                  });
                }
              }
              SettingKind::String => {
                if setting_value_kind != "string" {
                  diagnostics.push(lsp::Diagnostic {
                    range: setting_value_node.get_range(),
                    severity: Some(lsp::DiagnosticSeverity::ERROR),
                    source: Some("just-lsp".to_string()),
                    message: format!(
                      "Setting '{}' expects a string value",
                      setting_name
                    ),
                    ..Default::default()
                  });
                }
              }
              SettingKind::Array => {
                if !setting_value.starts_with('[')
                  || !setting_value.ends_with(']')
                {
                  diagnostics.push(lsp::Diagnostic {
                    range: setting_value_node.get_range(),
                    severity: Some(lsp::DiagnosticSeverity::ERROR),
                    source: Some("just-lsp".to_string()),
                    message: format!(
                      "Setting '{}' expects an array value",
                      setting_name
                    ),
                    ..Default::default()
                  });
                }
              }
            }
          }
        } else {
          diagnostics.push(lsp::Diagnostic {
            range: name_node.get_range(),
            severity: Some(lsp::DiagnosticSeverity::ERROR),
            source: Some("just-lsp".to_string()),
            message: format!("Unknown setting '{}'", setting_name),
            ..Default::default()
          });
        }
      }
    }

    let mut seen = std::collections::HashSet::new();

    for setting_node in setting_nodes {
      if let Some(name_node) = self
        .document
        .find_child_by_kind(&setting_node, "identifier")
      {
        let setting_name = self.document.get_node_text(&name_node);

        if !seen.insert(setting_name.clone()) {
          diagnostics.push(lsp::Diagnostic {
            range: name_node.get_range(),
            severity: Some(lsp::DiagnosticSeverity::ERROR),
            source: Some("just-lsp".to_string()),
            message: format!("Duplicate setting '{}'", setting_name),
            ..Default::default()
          });
        }
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
  fn analyze() {
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
  fn aggregate_parser_errors() {
    let doc = document(indoc! {
      "
      foo
        echo \"foo\"
      "
    });

    let analyzer = Analyzer::new(&doc);

    let diagnostics = analyzer.aggregate_parser_errors();

    assert!(!diagnostics.is_empty(), "Should detect syntax errors");

    let syntax_errors: Vec<_> = diagnostics
      .iter()
      .filter(|d| {
        d.message.contains("Syntax error")
          || d.message.contains("Missing syntax")
      })
      .collect();

    assert!(
      !syntax_errors.is_empty(),
      "Should have at least one syntax error diagnostic"
    );

    let valid_doc = document(indoc! {
      "
      foo:
        echo \"foo\"
      "
    });

    let analyzer = Analyzer::new(&valid_doc);

    let valid_diagnostics = analyzer.aggregate_parser_errors();

    assert!(
      valid_diagnostics.is_empty(),
      "Valid document should not have syntax errors"
    );
  }

  #[test]
  fn analyze_aliases() {
    let doc = document(indoc! {"
      foo:
        echo \"foo\"

      alias bar := baz
    "});

    let analyzer = Analyzer::new(&doc);

    let diagnostics = analyzer.analyze_aliases();
    assert_eq!(diagnostics.len(), 1);
    assert_eq!(diagnostics[0].message, "Recipe 'baz' not found");

    let doc = document(indoc! {"
      foo:
        echo \"foo\"

      alias bar := foo
    "});

    let analyzer = Analyzer::new(&doc);

    let diagnostics = analyzer.analyze_aliases();
    assert_eq!(diagnostics.len(), 0);
  }

  #[test]
  fn analyze_dependencies() {
    let doc = document(indoc! {"
      foo:
        echo \"foo\"

      bar: baz
        echo \"bar\"
    "});

    let analyzer = Analyzer::new(&doc);

    let diagnostics = analyzer.analyze_dependencies();
    assert_eq!(diagnostics.len(), 1);
    assert_eq!(diagnostics[0].message, "Recipe 'baz' not found");
    assert_eq!(diagnostics[0].range.start.line, 3);

    let doc = document(indoc! {"
      foo:
        echo \"foo\"

      bar: foo
        echo \"bar\"
    "});

    let analyzer = Analyzer::new(&doc);

    let diagnostics = analyzer.analyze_dependencies();
    assert_eq!(diagnostics.len(), 0);

    let doc = document(indoc! {"
      foo:
        echo \"foo\"

      bar: missing1 missing2
        echo \"bar\"
    "});

    let analyzer = Analyzer::new(&doc);

    let diagnostics = analyzer.analyze_dependencies();
    assert_eq!(diagnostics.len(), 2);
  }

  #[test]
  fn analyze_function_calls() {
    let doc = document(indoc! {
      "
      foo:
        echo {{ unknown_function() }}
      "
    });

    let analyzer = Analyzer::new(&doc);

    let diagnostics = analyzer.analyze_function_calls();

    assert!(diagnostics.len() > 0, "Should have at least one diagnostic");

    assert!(
      diagnostics
        .iter()
        .any(|d| d.message.contains("Unknown function 'unknown_function'")),
      "Should have diagnostic about unknown function"
    );

    let doc = document(indoc! {
      "
      foo:
        echo {{ replace() }}
      "
    });

    let analyzer = Analyzer::new(&doc);

    let diagnostics = analyzer.analyze_function_calls();

    assert!(diagnostics.len() > 0, "Should have at least one diagnostic");

    assert!(
      diagnostics
        .iter()
        .any(|d| d.message.contains("requires at least 3 argument(s)")),
      "Should have diagnostic about missing arguments"
    );

    let doc = document(indoc! {
      "
      foo:
        echo {{ uppercase(\"hello\", \"extra\") }}
      "
    });

    let analyzer = Analyzer::new(&doc);

    let diagnostics = analyzer.analyze_function_calls();

    assert!(diagnostics.len() > 0, "Should have at least one diagnostic");

    assert!(
      diagnostics
        .iter()
        .any(|d| d.message.contains("accepts 1 argument(s)")),
      "Should have diagnostic about too many arguments"
    );

    let doc = document(indoc! {
      "
      foo:
        echo {{ arch() }}
        echo {{ join(\"a\", \"b\", \"c\") }}
      "
    });

    let analyzer = Analyzer::new(&doc);

    let diagnostics = analyzer.analyze_function_calls();

    assert_eq!(
      diagnostics.len(),
      0,
      "Valid function calls should not produce diagnostics"
    );
  }

  #[test]
  fn analyze_recipe_parameters() {
    let doc = document(indoc! {
      "
      recipe_with_duplicate_param arg1 arg1:
        echo \"{{arg1}}\"
      "
    });

    let analyzer = Analyzer::new(&doc);

    let diagnostics = analyzer.analyze_recipe_parameters();

    assert_eq!(diagnostics.len(), 1);
    assert!(diagnostics[0].message.contains("Duplicate parameter"));

    let doc = document(indoc! {
      "
      recipe_with_param_order arg1=\"default\" arg2:
        echo \"{{arg1}} {{arg2}}\"
      "
    });

    let analyzer = Analyzer::new(&doc);

    let diagnostics = analyzer.analyze_recipe_parameters();

    assert_eq!(diagnostics.len(), 1);

    assert!(diagnostics[0].message.contains(
      "Required parameter 'arg2' follows a parameter with a default value"
    ));

    let doc = document(indoc! {
      "
      recipe_with_variadic arg1=\"default\" +args:
        echo \"{{arg1}} {{args}}\"
      "
    });

    let analyzer = Analyzer::new(&doc);

    let diagnostics = analyzer.analyze_recipe_parameters();

    assert_eq!(
      diagnostics.len(),
      0,
      "Variadic parameter after default should not produce diagnostics"
    );

    let doc = document(indoc! {
      "
      recipe_with_defaults arg1=\"first\" arg2=\"second\":
        echo \"{{arg1}} {{arg2}}\"
      "
    });

    let analyzer = Analyzer::new(&doc);

    let diagnostics = analyzer.analyze_recipe_parameters();

    assert_eq!(
      diagnostics.len(),
      0,
      "Parameters with all defaults should not produce diagnostics"
    );

    let doc = document(indoc! {
      "
      valid_recipe arg1 arg2=\"default\":
        echo \"{{arg1}} {{arg2}}\"
      "
    });

    let analyzer = Analyzer::new(&doc);

    let diagnostics = analyzer.analyze_recipe_parameters();

    assert_eq!(
      diagnostics.len(),
      0,
      "Valid parameter order should not produce diagnostics"
    );
  }

  #[test]
  fn analyze_settings_unknown_setting() {
    let doc = document(indoc! {
      "
    set unknown-setting := true

    foo:
      echo \"foo\"
    "
    });

    let analyzer = Analyzer::new(&doc);

    let diagnostics = analyzer.analyze_settings();

    assert_eq!(diagnostics.len(), 1);
    assert_eq!(diagnostics[0].message, "Unknown setting 'unknown-setting'");
  }

  #[test]
  fn analyze_settings_boolean_type() {
    let doc = document(indoc! {
      "
    set export := 'foo'

    foo:
      echo \"foo\"
    "
    });

    let analyzer = Analyzer::new(&doc);

    let diagnostics = analyzer.analyze_settings();

    assert_eq!(diagnostics.len(), 1);

    assert_eq!(
      diagnostics[0].message,
      "Setting 'export' expects a boolean value"
    );

    let doc = document(indoc! {
      "
      set export

      foo:
        echo \"foo\"
      "
    });

    let analyzer = Analyzer::new(&doc);

    let diagnostics = analyzer.analyze_settings();

    assert_eq!(
      diagnostics.len(),
      0,
      "Shorthand boolean syntax should be valid"
    );

    let doc = document(indoc! {
      "
      set export := true
      set dotenv-load := false

      foo:
        echo \"foo\"
      "
    });

    let analyzer = Analyzer::new(&doc);

    let diagnostics = analyzer.analyze_settings();

    assert_eq!(
      diagnostics.len(),
      0,
      "Valid boolean values should not produce diagnostics"
    );
  }

  #[test]
  fn analyze_settings_string_type() {
    let doc = document(indoc! {
      "
      set dotenv-path := true

      foo:
        echo \"foo\"
      "
    });

    let analyzer = Analyzer::new(&doc);

    let diagnostics = analyzer.analyze_settings();

    assert_eq!(diagnostics.len(), 1);
    assert_eq!(
      diagnostics[0].message,
      "Setting 'dotenv-path' expects a string value"
    );

    let doc = document(indoc! {
      "
      set dotenv-path := \".env.development\"

      foo:
        echo \"foo\"
      "
    });

    let analyzer = Analyzer::new(&doc);

    let diagnostics = analyzer.analyze_settings();

    assert_eq!(
      diagnostics.len(),
      0,
      "Valid string value should not produce diagnostics"
    );
  }

  #[test]
  fn analyze_settings_duplicate() {
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

    let diagnostics = analyzer.analyze_settings();

    assert_eq!(diagnostics.len(), 1);
    assert_eq!(diagnostics[0].message, "Duplicate setting 'export'");
  }

  #[test]
  fn analyze_settings_multiple_errors() {
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

    let diagnostics = analyzer.analyze_settings();

    assert_eq!(diagnostics.len(), 2, "Should detect all errors in settings");

    let messages: Vec<String> =
      diagnostics.iter().map(|d| d.message.clone()).collect();

    assert!(messages.contains(&"Unknown setting 'unknown-setting'".to_string()));

    assert!(messages.contains(&"Duplicate setting 'export'".to_string()));
  }

  #[test]
  fn analyze_attributes_unknown() {
    let doc = document(indoc! {
      "
      [unknown_attribute]
      foo:
        echo \"foo\"
      "
    });

    let analyzer = Analyzer::new(&doc);

    let diagnostics = analyzer.analyze_attributes();

    assert_eq!(diagnostics.len(), 1);

    assert_eq!(
      diagnostics[0].message,
      "Unknown attribute 'unknown_attribute'"
    );
  }

  #[test]
  #[ignore]
  fn analyze_attributes_wrong_target() {
    let doc = document(indoc! {
      "
      [linux]
      set export := true
      "
    });

    let analyzer = Analyzer::new(&doc);

    let diagnostics = analyzer.analyze_attributes();

    assert_eq!(diagnostics.len(), 1);

    assert!(diagnostics[0]
      .message
      .contains("cannot be applied to variable target"));
  }

  #[test]
  fn analyze_attributes_missing_parameters() {
    let doc = document(indoc! {
      "
      [script]
      foo:
        echo \"foo\"
      "
    });

    let analyzer = Analyzer::new(&doc);

    let diagnostics = analyzer.analyze_attributes();

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

    let diagnostics = analyzer.analyze_attributes();

    assert_eq!(
      diagnostics.len(),
      0,
      "Should be valid since confirm can be used without parameters"
    );

    let doc = document(indoc! {
      "
      [doc]
      foo:
        echo \"foo\"
      "
    });

    let analyzer = Analyzer::new(&doc);

    let diagnostics = analyzer.analyze_attributes();

    assert_eq!(diagnostics.len(), 1);

    assert_eq!(
      diagnostics[0].message,
      "Attribute 'doc' requires parameters"
    );
  }

  #[test]
  fn analyze_attributes_extra_parameters() {
    let doc = document(indoc! {
      "
      [linux('invalid')]
      foo:
        echo \"foo\"
      "
    });

    let analyzer = Analyzer::new(&doc);

    let diagnostics = analyzer.analyze_attributes();

    assert_eq!(diagnostics.len(), 1);

    assert_eq!(
      diagnostics[0].message,
      "Attribute 'linux' doesn't accept parameters"
    );
  }

  #[test]
  fn analyze_attributes_correct() {
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

    let diagnostics = analyzer.analyze_attributes();

    assert_eq!(
      diagnostics.len(),
      0,
      "Valid attributes should not produce diagnostics"
    );
  }
}
