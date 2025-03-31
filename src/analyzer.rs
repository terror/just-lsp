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
      .chain(self.analyze_values())
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
      for identifier_node in attribute_node.find_all("identifier") {
        let attribute_name = self.document.get_node_text(&identifier_node);

        let matching_attributes: Vec<_> = builtins::BUILTINS
          .iter()
          .filter(|f| matches!(f, Builtin::Attribute { name, .. } if *name == attribute_name))
          .collect();

        if matching_attributes.is_empty() {
          diagnostics.push(lsp::Diagnostic {
            range: identifier_node.get_range(),
            severity: Some(lsp::DiagnosticSeverity::ERROR),
            source: Some("just-lsp".to_string()),
            message: format!("Unknown attribute '{}'", attribute_name),
            ..Default::default()
          });

          continue;
        }

        let argument_count = identifier_node
          .find_siblings_until("string", "identifier")
          .len();

        let has_arguments = argument_count > 0;

        let parameter_mismatch = matching_attributes.iter().all(|attr| {
          if let Builtin::Attribute { parameters, .. } = attr {
            (parameters.is_some() && !has_arguments)
              || (parameters.is_none() && has_arguments)
              || (parameters.map_or(0, |_| 1) < argument_count)
          } else {
            false
          }
        });

        if parameter_mismatch {
          let param_error_msg = if matching_attributes.iter().any(|attr| {
            matches!(attr, Builtin::Attribute { parameters, .. } if parameters.is_none())
          }) {
            format!("Attribute '{}' doesn't accept parameters", attribute_name)
          } else if matching_attributes.iter().any(|attr| {
            matches!(attr, Builtin::Attribute { parameters, .. } if parameters.map_or(0, |_| 1) < argument_count)
          }) {
            format!(
              "Attribute '{}' got {} arguments but takes {} argument",
              attribute_name,
              argument_count,
              matching_attributes.iter().find_map(|attr| {
                if let Builtin::Attribute { parameters, .. } = attr {
                  parameters.map(|_| 1)
                } else {
                  None
                }
              }).unwrap_or(0),
            )
          } else {
            format!("Attribute '{}' requires parameters", attribute_name)
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

          let is_valid_target = matching_attributes
            .iter()
            .filter_map(|attr| {
              if let Builtin::Attribute { targets, .. } = attr {
                Some(targets)
              } else {
                None
              }
            })
            .any(|targets| {
              targets
                .iter()
                .any(|target| target.is_valid_for(target_type))
            });

          if !is_valid_target {
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
          let arguments = function_call.find_all("expression > value");

          let arg_count = arguments.len();

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

    for recipe in &recipes {
      let mut seen = HashSet::new();

      let (mut passed_default, mut passed_variadic) = (false, false);

      for (index, param) in recipe.parameters.iter().enumerate() {
        if !seen.insert(param.name.clone()) {
          diagnostics.push(lsp::Diagnostic {
            range: param.range,
            severity: Some(lsp::DiagnosticSeverity::ERROR),
            source: Some("just-lsp".to_string()),
            message: format!("Duplicate parameter '{}'", param.name),
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

    let mut recipe_groups: HashMap<String, Vec<(Recipe, HashSet<OsGroup>)>> =
      HashMap::new();

    for recipe in &recipes {
      let mut os_groups = HashSet::new();

      for attribute in &recipe.attributes {
        let attr_name = attribute.name.value.as_str();
        if let Some(group) = OsGroup::from_attribute(attr_name) {
          os_groups.insert(group);
        }
      }

      if os_groups.is_empty() {
        os_groups.insert(OsGroup::None);
      }

      recipe_groups
        .entry(recipe.name.clone())
        .or_default()
        .push((recipe.clone(), os_groups));
    }

    for (recipe_name, group) in &recipe_groups {
      if group.len() <= 1 {
        continue;
      }

      for i in 0..group.len() {
        let (recipe1, os_groups1) = &group[i];

        for j in 0..i {
          let (_, os_groups2) = &group[j];

          let has_conflict = os_groups1.iter().any(|group1| {
            os_groups2
              .iter()
              .any(|group2| group1.conflicts_with(group2))
          });

          if has_conflict {
            diagnostics.push(lsp::Diagnostic {
              range: recipe1.range,
              severity: Some(lsp::DiagnosticSeverity::ERROR),
              source: Some("just-lsp".to_string()),
              message: format!("Duplicate recipe name '{}'", recipe_name),
              ..Default::default()
            });

            break;
          }
        }
      }
    }

    let recipe_names: HashSet<_> =
      recipes.iter().map(|r| r.name.clone()).collect();

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

  fn analyze_values(&self) -> Vec<lsp::Diagnostic> {
    let mut diagnostics = Vec::new();

    let root = match self.document.tree {
      Some(ref tree) => tree.root_node(),
      None => return diagnostics,
    };

    let identifiers = root.find_all("expression > value > identifier");

    let mut variables = self
      .document
      .get_variables()
      .iter()
      .map(|variable| variable.name.value.clone())
      .collect::<HashSet<_>>();

    variables.extend(builtins::BUILTINS.into_iter().filter_map(|builtin| {
      match builtin {
        Builtin::Constant { name, .. } => Some(name.to_owned()),
        _ => None,
      }
    }));

    let mut recipe_identifier_map = self.document.get_recipes().iter().fold(
      HashMap::new(),
      |mut acc, recipe| {
        acc.insert(recipe.name.clone(), HashSet::new());
        acc
      },
    );

    let mut variable_usage_map = self.document.get_variables().iter().fold(
      HashMap::new(),
      |mut acc, variable| {
        acc.insert(variable.name.value.clone(), (false, variable.export));
        acc
      },
    );

    for identifier in identifiers {
      let recipe_name = identifier
        .get_parent("recipe")
        .as_ref()
        .and_then(|recipe_node| recipe_node.find("recipe_header > identifier"))
        .map_or_else(String::new, |identifier_node| {
          self.document.get_node_text(&identifier_node)
        });

      let recipe = self.document.find_recipe(&recipe_name);

      let identifier_name = self.document.get_node_text(&identifier);

      let create_diagnostic = |message: String| lsp::Diagnostic {
        range: identifier.get_range(),
        severity: Some(lsp::DiagnosticSeverity::ERROR),
        source: Some("just-lsp".to_string()),
        message,
        ..Default::default()
      };

      match recipe {
        Some(recipe) => {
          recipe_identifier_map
            .entry(recipe.name.clone())
            .or_insert_with(HashSet::new)
            .insert(identifier_name.clone());

          let recipe_parameters = recipe
            .parameters
            .iter()
            .map(|p| p.name.clone())
            .collect::<HashSet<_>>();

          if !recipe_parameters.contains(&identifier_name) {
            if variable_usage_map.contains_key(&identifier_name) {
              if let Some((_, export)) =
                variable_usage_map.get(&identifier_name)
              {
                variable_usage_map
                  .insert(identifier_name.clone(), (true, *export));
              }
            }

            if !variables.contains(&identifier_name) {
              diagnostics.push(create_diagnostic(format!(
                "Variable '{}' not found",
                identifier_name
              )));
            }
          }
        }
        None => {
          if variable_usage_map.contains_key(&identifier_name) {
            if let Some((_, export)) = variable_usage_map.get(&identifier_name)
            {
              variable_usage_map
                .insert(identifier_name.clone(), (true, *export));
            }
          }

          if !variables.contains(&identifier_name) {
            diagnostics.push(create_diagnostic(format!(
              "Variable '{}' not found",
              identifier_name
            )));
          }
        }
      }
    }

    for (variable_name, (is_used, is_exported)) in variable_usage_map {
      if !is_used && !is_exported {
        if let Some(variable) = self
          .document
          .get_variables()
          .iter()
          .find(|v| v.name.value == variable_name)
        {
          diagnostics.push(lsp::Diagnostic {
            range: variable.name.range,
            severity: Some(lsp::DiagnosticSeverity::WARNING),
            source: Some("just-lsp".to_string()),
            message: format!("Variable '{}' appears unused", variable_name),
            ..Default::default()
          });
        }
      }
    }

    for (recipe_name, identifiers) in recipe_identifier_map {
      if let Some(recipe) = self.document.find_recipe(&recipe_name) {
        recipe.parameters.iter().for_each(|parameter| {
          if !identifiers.contains(&parameter.name) {
            diagnostics.push(lsp::Diagnostic {
              range: parameter.range,
              severity: Some(lsp::DiagnosticSeverity::WARNING),
              source: Some("just-lsp".to_string()),
              message: format!("Parameter '{}' appears unused", parameter.name),
              ..Default::default()
            });
          }
        });
      }
    }

    diagnostics
  }
}

#[cfg(test)]
mod tests {
  use {super::*, indoc::indoc, pretty_assertions::assert_eq};

  #[derive(Debug)]
  struct Test {
    document: Document,
    messages: Vec<(String, Option<lsp::DiagnosticSeverity>)>,
  }

  impl Test {
    fn new(content: &str) -> Self {
      Self {
        document: Document::try_from(lsp::DidOpenTextDocumentParams {
          text_document: lsp::TextDocumentItem {
            uri: lsp::Url::parse("file:///test.just").unwrap(),
            language_id: "just".to_string(),
            version: 1,
            text: content.to_string(),
          },
        })
        .unwrap(),
        messages: Vec::new(),
      }
    }

    fn error(self, message: &str) -> Self {
      Self {
        messages: self
          .messages
          .into_iter()
          .chain([(message.to_owned(), Some(lsp::DiagnosticSeverity::ERROR))])
          .collect(),
        ..self
      }
    }

    fn warning(self, message: &str) -> Self {
      Self {
        messages: self
          .messages
          .into_iter()
          .chain([(message.to_owned(), Some(lsp::DiagnosticSeverity::WARNING))])
          .collect(),
        ..self
      }
    }

    fn run(self) {
      let analyzer = Analyzer::new(&self.document);

      let messages = analyzer
        .analyze()
        .into_iter()
        .map(|d| (d.message, d.severity))
        .collect::<Vec<(String, Option<lsp::DiagnosticSeverity>)>>();

      assert_eq!(messages, self.messages);
    }
  }

  #[test]
  fn aliases_basic() {
    Test::new(indoc! {
      "
      foo:
        echo \"foo\"

      alias bar := foo
      "
    })
    .run()
  }

  #[test]
  fn aliases_duplicate() {
    Test::new(indoc! {
      "
      foo:
        echo \"foo\"

      alias bar := foo
      alias bar := foo
      alias bar := foo
      "
    })
    .error("Duplicate alias 'bar'")
    .error("Duplicate alias 'bar'")
    .run()
  }

  #[test]
  fn aliases_missing_recipe() {
    Test::new(indoc! {
      "
      foo:
        echo \"foo\"

      alias bar := baz
      "
    })
    .error("Recipe 'baz' not found")
    .run()
  }

  #[test]
  fn analyze_complete() {
    Test::new(indoc! {
      "
      foo:
        echo \"foo\"

      bar: missing
        echo \"bar\"

      alias baz := nonexistent
      "
    })
    .error("Recipe 'nonexistent' not found")
    .error("Recipe 'missing' not found")
    .run()
  }

  #[test]
  fn attributes_correct() {
    Test::new(indoc! {
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
    })
    .run()
  }

  #[test]
  fn attributes_extra_parameters() {
    Test::new(indoc! {
      "
      [linux('invalid')]
      foo:
        echo \"foo\"
      "
    })
    .error("Attribute 'linux' doesn't accept parameters")
    .run()
  }

  #[test]
  fn attributes_missing_parameters() {
    Test::new(indoc! {
      "
      [doc]
      foo:
        echo \"foo\"
      "
    })
    .error("Attribute 'doc' requires parameters")
    .run()
  }

  #[test]
  fn attributes_no_parameters_needed() {
    Test::new(indoc! {
      "
      [script]
      foo:
        echo \"foo\"
      "
    })
    .run();

    Test::new(indoc! {
      "
      [confirm]
      foo:
        echo \"foo\"
      "
    })
    .run()
  }

  #[test]
  fn attributes_unknown() {
    Test::new(indoc! {
      "
      [unknown_attribute]
      foo:
        echo \"foo\"
      "
    })
    .error("Unknown attribute 'unknown_attribute'")
    .run()
  }

  #[test]
  fn attributes_wrong_target() {
    Test::new(indoc! {
      "
      [group: 'foo']
      alias f := foo

      foo:
        echo \"foo\"
      "
    })
    .error("Attribute 'group' cannot be applied to alias target")
    .run()
  }

  #[test]
  fn attributes_invalid_inline() {
    Test::new(indoc! {
      "
      [group: 'foo', foo]
      foo:
        echo \"foo\"
      "
    })
    .error("Unknown attribute 'foo'")
    .run()
  }

  #[test]
  fn attributes_inline_parameters_focused() {
    Test::new(indoc! {
      "
      [group: 'foo', no-cd]
      foo:
        echo \"foo\"
      "
    })
    .run()
  }

  #[test]
  fn attributes_more_arguments_than_required() {
    Test::new(indoc! {
      "
      [group('foo', 'bar')]
      foo:
        echo \"foo\"
      "
    })
    .error("Attribute 'group' got 2 arguments but takes 1 argument")
    .run()
  }

  #[test]
  fn function_calls_correct() {
    Test::new(indoc! {
      "
      foo:
        echo {{ arch() }}
        echo {{ join(\"a\", \"b\", \"c\") }}
      "
    })
    .run()
  }

  #[test]
  fn function_calls_too_few_args() {
    Test::new(indoc! {
      "
      foo:
        echo {{ replace() }}
      "
    })
    .error("Function 'replace' requires at least 3 argument(s), but 0 provided")
    .run()
  }

  #[test]
  fn function_calls_too_many_args() {
    Test::new(indoc! {
      "
      foo:
        echo {{ uppercase(\"hello\", \"extra\") }}
      "
    })
    .error("Function 'uppercase' accepts 1 argument(s), but 2 provided")
    .run()
  }

  #[test]
  fn function_calls_unknown() {
    Test::new(indoc! {
      "
      foo:
        echo {{ unknown_function() }}
      "
    })
    .error("Unknown function 'unknown_function'")
    .run()
  }

  #[test]
  fn parser_errors_invalid() {
    Test::new(indoc! {
      "
      foo
        echo \"foo\"
      "
    })
    .error("Syntax error")
    .run()
  }

  #[test]
  fn parser_errors_valid() {
    Test::new(indoc! {
      "
      foo:
        echo \"foo\"
      "
    })
    .run()
  }

  #[test]
  fn recipe_dependencies_correct() {
    Test::new(indoc! {
      "
      foo:
        echo \"foo\"

      bar: foo
        echo \"bar\"
      "
    })
    .run()
  }

  #[test]
  fn recipe_dependencies_missing() {
    Test::new(indoc! {
      "
      foo:
        echo \"foo\"

      bar: baz
        echo \"bar\"
      "
    })
    .error("Recipe 'baz' not found")
    .run()
  }

  #[test]
  fn recipe_dependencies_multiple_missing() {
    Test::new(indoc! {
      "
      foo:
        echo \"foo\"

      bar: missing1 missing2
        echo \"bar\"
      "
    })
    .error("Recipe 'missing1' not found")
    .error("Recipe 'missing2' not found")
    .run()
  }

  #[test]
  fn recipe_invocation_argument_count_correct() {
    Test::new(indoc! {
      "
      foo arg1 arg2=\"default\":
        echo \"{{arg1}} {{arg2}}\"

      bar: (foo 'value1')
        echo \"bar\"
      "
    })
    .run()
  }

  #[test]
  fn recipe_invocation_missing_args() {
    Test::new(indoc! {
      "
      foo arg1 arg2:
        echo \"{{arg1}} {{arg2}}\"

      bar: (foo)
        echo \"bar\"
      "
    })
    .error("Dependency 'foo' requires 2 argument(s), but 0 provided")
    .run()
  }

  #[test]
  fn recipe_invocation_too_few_args() {
    Test::new(indoc! {
      "
      foo arg1 arg2:
        echo \"{{arg1}} {{arg2}}\"

      bar: (foo 'value1')
        echo \"bar\"
      "
    })
    .error("Dependency 'foo' requires 2 argument(s), but 1 provided")
    .run()
  }

  #[test]
  fn recipe_invocation_too_many_args() {
    Test::new(indoc! {
      "
      foo arg1:
        echo \"{{arg1}}\"

      bar: (foo 'value1' 'value2' 'value3')
        echo \"bar\"
      "
    })
    .error("Dependency 'foo' accepts 1 argument(s), but 3 provided")
    .run()
  }

  #[test]
  fn recipe_invocation_unknown_variable() {
    Test::new(indoc! {
      "
      foo arg1:
        echo {{ arg1 }}

      bar: (foo wow)
        echo \"bar\"
      "
    })
    .error("Variable 'wow' not found")
    .run()
  }

  #[test]
  fn recipe_invocation_valid_variable() {
    Test::new(indoc! {
      "
      wow := 'foo'

      foo arg1:
        echo \"{{arg1}}\"

      bar: (foo wow)
        echo \"bar\"
      "
    })
    .run()
  }

  #[test]
  fn recipe_invocation_variadic_params() {
    Test::new(indoc! {
      "
      foo arg1 +args:
        echo \"{{arg1}} {{args}}\"

      bar: (foo 'value1' 'value2' 'value3')
        echo \"bar\"
      "
    })
    .run()
  }

  #[test]
  fn recipe_parameters_defaults_all() {
    Test::new(indoc! {
      "
      recipe_with_defaults arg1=\"first\" arg2=\"second\":
        echo \"{{arg1}} {{arg2}}\"
      "
    })
    .run()
  }

  #[test]
  fn recipe_parameters_duplicate() {
    Test::new(indoc! {
      "
      recipe_with_duplicate_param arg1 arg1:
        echo \"{{arg1}}\"
      "
    })
    .error("Duplicate parameter 'arg1'")
    .run()
  }

  #[test]
  fn recipe_parameters_order() {
    Test::new(indoc! {
      "
      recipe_with_param_order arg1=\"default\" arg2:
        echo \"{{arg1}} {{arg2}}\"
      "
    })
    .error("Required parameter 'arg2' follows a parameter with a default value")
    .run()
  }

  #[test]
  fn recipe_parameters_valid() {
    Test::new(indoc! {
      "
      valid_recipe arg1 arg2=\"default\":
        echo \"{{arg1}} {{arg2}}\"
      "
    })
    .run()
  }

  #[test]
  fn recipe_parameters_variadic() {
    Test::new(indoc! {
      "
      recipe_with_variadic arg1=\"default\" +args:
        echo \"{{arg1}} {{args}}\"
      "
    })
    .run()
  }

  #[test]
  fn settings_boolean_shorthand() {
    Test::new(indoc! {
      "
      set export

      foo:
        echo \"foo\"
      "
    })
    .run()
  }

  #[test]
  fn settings_boolean_type_correct() {
    Test::new(indoc! {
      "
      set export := true
      set dotenv-load := false

      foo:
        echo \"foo\"
      "
    })
    .run()
  }

  #[test]
  fn settings_boolean_type_error() {
    Test::new(indoc! {
      "
      set export := 'foo'

      foo:
        echo \"foo\"
      "
    })
    .error("Setting 'export' expects a boolean value")
    .run()
  }

  #[test]
  fn settings_duplicate() {
    Test::new(indoc! {
      "
      set export := true
      set shell := [\"bash\", \"-c\"]
      set export := false

      foo:
        echo \"foo\"
      "
    })
    .error("Duplicate setting 'export'")
    .run()
  }

  #[test]
  fn settings_multiple_errors() {
    Test::new(indoc! {
      "
      set unknown-setting := true
      set export := false
      set shell := ['bash']
      set export := false

      foo:
        echo \"foo\"
      "
    })
    .error("Unknown setting 'unknown-setting'")
    .error("Duplicate setting 'export'")
    .run()
  }

  #[test]
  fn settings_string_type_correct() {
    Test::new(indoc! {
      "
      set dotenv-path := \".env.development\"

      foo:
        echo \"foo\"
      "
    })
    .run()
  }

  #[test]
  fn settings_string_type_error() {
    Test::new(indoc! {
      "
      set dotenv-path := true

      foo:
        echo \"foo\"
      "
    })
    .error("Setting 'dotenv-path' expects a string value")
    .run()
  }

  #[test]
  fn settings_unknown() {
    Test::new(indoc! {
      "
      set unknown-setting := true

      foo:
        echo \"foo\"
      "
    })
    .error("Unknown setting 'unknown-setting'")
    .run()
  }

  #[test]
  fn should_recognize_recipe_parameters_in_dependency_arguments() {
    Test::new(indoc! {
      "
      other-recipe var=\"else\":
        echo {{ var }}

      test var=\"something\": (other-recipe var)
      "
    })
    .run()
  }

  #[test]
  fn unreferenced_variable_in_expression() {
    Test::new(indoc! {
      "
      foo:
        echo {{ var }}
      "
    })
    .error("Variable 'var' not found")
    .run()
  }

  #[test]
  fn warn_for_unused_recipe_parameters() {
    Test::new(indoc! {
      "
      foo bar:
        echo foo
      "
    })
    .warning("Parameter 'bar' appears unused")
    .run()
  }

  #[test]
  fn duplicate_recipe_names() {
    Test::new(indoc! {
      "
      foo:
        echo foo

      foo:
        echo foo

      foo:
        echo foo
      "
    })
    .error("Duplicate recipe name 'foo'")
    .error("Duplicate recipe name 'foo'")
    .run()
  }

  #[test]
  fn warn_for_unused_variables() {
    Test::new(indoc! {
      "
      foo := \"unused value\"
      bar := \"used value\"

      recipe:
        echo {{ bar }}
      "
    })
    .warning("Variable 'foo' appears unused")
    .run()
  }

  #[test]
  fn used_variables_no_warnings() {
    Test::new(indoc! {
      "
      foo := \"used in recipe\"
      bar := \"used as dependency arg\"

      another arg:
        echo {{ arg }}

      recipe: (another bar)
        echo {{ foo }}
      "
    })
    .run()
  }

  #[test]
  fn variables_used_in_recipe_parameters() {
    Test::new(indoc! {
      "
      param_value := \"value\"
      unused := \"unused\"

      recipe arg=\"default\": (another param_value)
        echo {{ arg }}

      another arg:
        echo {{ arg }}
      "
    })
    .warning("Variable 'unused' appears unused")
    .run()
  }

  #[test]
  fn variables_used_in_dependency_args() {
    Test::new(indoc! {
      "
      used_arg := \"value\"
      unused_var := \"not used\"

      recipe: (another used_arg)
        echo \"something\"

      another arg:
        echo {{ arg }}
      "
    })
    .warning("Variable 'unused_var' appears unused")
    .run()
  }

  #[test]
  fn variables_and_parameters_same_name() {
    Test::new(indoc! {
      "
      param := \"variable value\"
      other := \"other value\"

      recipe param:
        # This should reference the parameter, not the variable
        echo {{ param }}
        echo {{ other }}
      "
    })
    .warning("Variable 'param' appears unused")
    .run()
  }

  #[test]
  fn variables_used_in_multiple_recipes() {
    Test::new(indoc! {
      "
      shared := \"shared value\"
      only_in_first := \"first value\"
      only_in_second := \"second value\"
      never_used := \"unused\"

      first:
        echo {{ shared }}
        echo {{ only_in_first }}

      second:
        echo {{ shared }}
        echo {{ only_in_second }}
      "
    })
    .warning("Variable 'never_used' appears unused")
    .run()
  }

  #[test]
  fn exported_variables_not_warned() {
    Test::new(indoc! {
      "
      foo := \"unused value\"
      export bar := \"exported but unused\"
      baz := \"used value\"

      recipe:
        echo {{ baz }}
      "
    })
    .warning("Variable 'foo' appears unused")
    .run()
  }

  #[test]
  fn os_specific_duplicate_recipes() {
    Test::new(indoc! {
      "
      [linux]
      build:
        echo \"Building on Linux\"

      [windows]
      build:
        echo \"Building on Windows\"

      [unix]
      build:
        echo \"Building on Unix\"
      "
    })
    .run()
  }

  #[test]
  fn duplicate_recipes_with_same_os_attribute() {
    Test::new(indoc! {
      "
      [linux]
      build:
        echo \"Building on Linux version 1\"

      [linux]
      build:
        echo \"Building on Linux version 2\"
      "
    })
    .error("Duplicate recipe name 'build'")
    .run()
  }

  #[test]
  fn mixed_os_specific_and_regular_recipe() {
    Test::new(indoc! {
      "
      [linux]
      build:
        echo \"Building on Linux\"

      build:
        echo \"Building on any OS\"
      "
    })
    .error("Duplicate recipe name 'build'")
    .run()
  }

  #[test]
  fn unix_macos_conflicts() {
    Test::new(indoc! {
      "
      [unix]
      build:
        echo \"Building on Unix systems\"

      [macos]
      build:
        echo \"Building on macOS specifically\"
      "
    })
    .error("Duplicate recipe name 'build'")
    .run()
  }

  #[test]
  fn linux_openbsd_conflicts() {
    Test::new(indoc! {
      "
      [linux]
      build:
        echo \"Building on Linux\"

      [openbsd]
      build:
        echo \"Building on OpenBSD\"
      "
    })
    .error("Duplicate recipe name 'build'")
    .run()
  }

  #[test]
  fn linux_unix_no_conflict() {
    Test::new(indoc! {
      "
      [linux]
      build:
        echo \"Building on Linux\"

      [unix]
      build:
        echo \"Building on Unix systems\"
      "
    })
    .run()
  }

  #[test]
  fn openbsd_macos_no_conflict() {
    Test::new(indoc! {
      "
      [openbsd]
      build:
        echo \"Building on OpenBSD\"

      [macos]
      build:
        echo \"Building on macOS\"
      "
    })
    .run()
  }

  #[test]
  fn all_four_os_groups_no_conflict() {
    Test::new(indoc! {
      "
      [linux]
      build:
        echo \"Building on Linux\"

      [macos]
      build:
        echo \"Building on macOS\"

      [windows]
      build:
        echo \"Building on Windows\"
      "
    })
    .run()
  }

  #[test]
  fn recipe_with_multiple_os_attributes() {
    Test::new(indoc! {
      "
      [windows]
      [linux]
      build:
        echo \"Building on Linux or Windows\"

      [linux]
      build:
        echo \"Building on macOS\"

      [macos]
      build:
        echo \"Building on macOS\"
      "
    })
    .error("Duplicate recipe name 'build'")
    .run()
  }

  #[test]
  fn recipe_with_conflicting_multiple_os_attributes() {
    Test::new(indoc! {
      "
      [linux]
      [openbsd]
      build:
        echo \"Building on Linux and OpenBSD\"

      [linux]
      build:
        echo \"Building on Linux again\"
      "
    })
    .error("Duplicate recipe name 'build'")
    .run()
  }

  #[test]
  fn recipe_with_all_os_attributes() {
    Test::new(indoc! {
      "
      [linux]
      [windows]
      [unix]
      [macos]
      [openbsd]
      build:
        echo \"Building everywhere\"

      test:
        echo \"Testing\"
      "
    })
    .run()
  }
}
