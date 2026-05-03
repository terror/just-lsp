use super::*;

const VALID_KWARGS: &[&str] = &["help", "long", "short", "value", "pattern"];

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

      tree
        .root_node()
        .find_all("attribute")
        .into_iter()
        .flat_map(|attribute| {
          attribute
            .find_all("^identifier")
            .into_iter()
            .filter(|node| context.document().get_node_text(node) == "arg")
            .flat_map(move |identifier| Self::validate(context, attribute, identifier))
            .collect::<Vec<_>>()
        })
        .collect()
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
      .chain(value_without_option)
      .collect()
  }
}
