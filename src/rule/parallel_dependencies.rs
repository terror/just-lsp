use super::*;

define_rule! {
  /// Warn when `[parallel]` is applied to a recipe that lacks enough
  /// dependencies for the attribute to have any effect.
  ParallelDependenciesRule {
    id: "parallel-dependencies",
    message: "unnecessary parallel attribute",
    provides_quickfixes: true,
    run(context) {
      context
        .document()
        .recipes()
        .into_iter()
        .filter_map(|recipe| {
          let attribute = recipe.find_attribute("parallel")?;

          let diagnostic = match recipe.dependencies.len() {
            0 => Diagnostic::warning(
              format!(
                "Recipe `{}` has no dependencies, so `[parallel]` has no effect",
                recipe.name.value
              ),
              attribute.range,
            ),
            1 => Diagnostic::warning(
              format!(
                "Recipe `{}` has only one dependency, so `[parallel]` has no effect",
                recipe.name.value
              ),
              attribute.range,
            ),
            _ => return None,
          };

          Some(diagnostic.quickfix(ParallelDependenciesRule::attribute_removal(
            attribute,
            context.document(),
          )))
        })
        .collect()
    }
  }
}

impl ParallelDependenciesRule {
  fn attribute_removal(
    attribute: &Attribute,
    document: &Document,
  ) -> Option<Quickfix> {
    let root = document.tree.as_ref()?.root_node();

    let attribute_node = root
      .find_all("attribute")
      .into_iter()
      .find(|node| node.get_range(document) == attribute.range)?;

    let mut cursor = attribute_node.walk();

    let children = attribute_node.children(&mut cursor).collect::<Vec<_>>();

    let identifiers = children
      .iter()
      .filter(|node| node.kind() == "identifier")
      .collect::<Vec<_>>();

    let index = identifiers
      .iter()
      .position(|node| node.get_range(document) == attribute.name.range)?;

    let range = if identifiers.len() == 1 {
      attribute.range
    } else if let Some(next) = identifiers.get(index + 1) {
      lsp::Range {
        start: attribute.name.range.start,
        end: next.get_range(document).start,
      }
    } else {
      let identifier = identifiers[index];

      let comma = children.iter().rev().find(|node| {
        node.kind() == "," && node.end_byte() <= identifier.start_byte()
      })?;

      let previous = comma.prev_sibling()?;

      let closing_bracket = children.iter().find(|node| {
        node.kind() == "]" && node.start_byte() >= identifier.end_byte()
      })?;

      lsp::Range {
        start: previous.get_range(document).end,
        end: closing_bracket.get_range(document).start,
      }
    };

    Some(Quickfix::removal(
      range,
      format!("Remove `[{}]`", attribute.name.value),
    ))
  }
}
