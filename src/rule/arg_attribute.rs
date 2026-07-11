use super::*;

const VALID_KWARGS: &[&str] = &[
  "flag", "help", "long", "max", "min", "multiple", "pattern", "short", "value",
];

define_rule! {
  /// Validates `[arg(NAME, ...)]` attributes: that NAME refers to an existing
  /// recipe parameter, that only known keyword arguments are used, and that
  /// `value=` is paired with `long=` or `short=`.
  ArgAttributeRule {
    id: "arg-attribute",
    message: "invalid arg attribute",
    run(context) {
      let Some(tree) = context.tree() else {
        return Vec::new();
      };

      let document = context.document();

      let mut diagnostics = Vec::new();
      let mut seen = HashSet::new();

      for attribute in tree.root_node().find_all("attribute") {
        for identifier in attribute.find_all("^identifier") {
          if document.get_node_text(&identifier) != "arg" {
            continue;
          }

          diagnostics.extend(Self::validate(context, attribute, identifier));

          let Some(recipe_node) = attribute.get_parent("recipe") else {
            continue;
          };

          let Some(name_node) = identifier
            .siblings()
            .take_while(|node| node.kind() != "identifier")
            .find(|node| {
              node.kind() == "expression"
                && node.start_byte() != node.end_byte()
            })
          else {
            continue;
          };

          let parameter_name = document
            .get_node_text(&name_node)
            .trim_matches(|c| c == '\'' || c == '"')
            .to_string();

          if !seen.insert((
            recipe_node.start_byte(),
            recipe_node.end_byte(),
            parameter_name.clone(),
          )) {
            diagnostics.push(Diagnostic::error(
              format!(
                "`[arg]` attribute for parameter `{parameter_name}` is duplicated"
              ),
              attribute.get_range(document),
            ));
          }
        }
      }

      diagnostics
    }
  }
}

impl ArgAttributeRule {
  fn parameter_unknown(
    context: &RuleContext,
    attribute: Node,
    parameter_name: &str,
  ) -> bool {
    let Some(recipe_node) = attribute.get_parent("recipe") else {
      return false;
    };

    let Some(name_node) = recipe_node.find("recipe_header > identifier") else {
      return false;
    };

    let recipe_name = context.document().get_node_text(&name_node);

    let Some(recipe) = context.recipe(&recipe_name) else {
      return false;
    };

    !recipe
      .parameters
      .iter()
      .any(|parameter| parameter.name == parameter_name)
  }

  fn const_expression(node: Node) -> bool {
    node.find("function_call").is_none()
      && node.find("external_command").is_none()
  }

  fn string_literal_expression(node: Node) -> bool {
    let Some(value) = node.find("^value") else {
      return false;
    };

    let mut cursor = value.walk();

    let children = value.named_children(&mut cursor).collect::<Vec<_>>();

    match children.as_slice() {
      [child] => {
        child.kind() == "string" && child.find("format_string").is_none()
      }
      _ => false,
    }
  }

  fn validate(
    context: &RuleContext,
    attribute: Node,
    identifier: Node,
  ) -> Vec<Diagnostic> {
    let document = context.document();

    let positional = identifier
      .siblings()
      .take_while(|node| node.kind() != "identifier")
      .filter(|node| {
        node.kind() == "expression" && node.start_byte() != node.end_byte()
      })
      .collect::<Vec<_>>();

    let kwargs = identifier
      .siblings()
      .take_while(|node| node.kind() != "identifier")
      .filter(|node| node.kind() == "attribute_named_param")
      .collect::<Vec<_>>();

    let Some(name_node) = positional.first().copied() else {
      return Vec::new();
    };

    let parameter_name = document
      .get_node_text(&name_node)
      .trim_matches(|c| c == '\'' || c == '"')
      .to_string();

    let unknown_parameter =
      Self::parameter_unknown(context, attribute, &parameter_name).then(|| {
        Diagnostic::error(
          format!("`[arg]` references unknown parameter `{parameter_name}`"),
          name_node.get_range(document),
        )
      });

    let unknown_kwargs = kwargs.iter().filter_map(|node| {
      let name = node
        .child_by_field_name("name")
        .map(|node| document.get_node_text(&node))?;

      (!VALID_KWARGS.contains(&name.as_str())).then(|| {
        Diagnostic::error(
          format!(
            "Unknown `[arg]` keyword `{name}`, expected one of {}",
            VALID_KWARGS.join(", ")
          ),
          node.get_range(document),
        )
      })
    });

    let kwarg_name = |node: &Node| {
      node
        .child_by_field_name("name")
        .map(|node| document.get_node_text(&node))
    };

    let invalid_string_kwargs = kwargs.iter().filter_map(|node| {
      let name = kwarg_name(node)?;

      if !matches!(name.as_str(), "help" | "long" | "pattern" | "short") {
        return None;
      }

      let value = node.child_by_field_name("value")?;

      let valid = if matches!(name.as_str(), "help" | "pattern") {
        Self::const_expression(value)
      } else {
        Self::string_literal_expression(value)
      };

      (!valid).then(|| {
        Diagnostic::error(
          if matches!(name.as_str(), "help" | "pattern") {
            "Attribute `arg` arguments must be const expressions".to_string()
          } else {
            "Attribute `arg` arguments must be string literals".to_string()
          },
          value.get_range(document),
        )
      })
    });

    let value_without_option = kwargs
      .iter()
      .find(|node| kwarg_name(node).as_deref() == Some("value"))
      .filter(|_| {
        !kwargs.iter().any(|node| {
          matches!(kwarg_name(node).as_deref(), Some("long" | "short"))
        })
      })
      .map(|node| {
        Diagnostic::error(
          "`[arg]` `value=` requires `long=` or `short=`".to_string(),
          node.get_range(document),
        )
      });

    unknown_parameter
      .into_iter()
      .chain(unknown_kwargs)
      .chain(invalid_string_kwargs)
      .chain(value_without_option)
      .collect()
  }
}
