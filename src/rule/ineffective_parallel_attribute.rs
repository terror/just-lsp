use super::*;

define_rule! {
  /// Warn when `[parallel]` is applied to a recipe that lacks enough
  /// dependencies for the attribute to have any effect.
  IneffectiveParallelAttributeRule {
    id: "ineffective-parallel-attribute",
    message: "unnecessary parallel attribute",
    run(context) {
      context
        .local_declarations(context.recipes())
        .filter_map(|recipe| {
          let attribute = recipe.find_attribute("parallel")?;

          if recipe
            .dependencies
            .iter()
            .any(|dependency| dependency.mapped.is_some())
          {
            return None;
          }

          let count = |phase| {
            recipe
              .dependencies
              .iter()
              .filter(|dependency| dependency.phase == phase)
              .count()
          };

          let diagnostic = match (
            count(DependencyPhase::Prior),
            count(DependencyPhase::Subsequent),
          ) {
            (0, 0) => Diagnostic::warning(
              format!(
                "Recipe `{}` has no dependencies, so `[parallel]` has no effect",
                recipe.name.value
              ),
              attribute.range,
            ),
            (1, 0) | (0, 1) => Diagnostic::warning(
              format!(
                "Recipe `{}` has only one dependency, so `[parallel]` has no effect",
                recipe.name.value
              ),
              attribute.range,
            ),
            (1, 1) => Diagnostic::warning(
              format!(
                "Recipe `{}` has only one dependency before and after `&&`, so `[parallel]` has no effect",
                recipe.name.value
              ),
              attribute.range,
            ),
            _ => return None,
          };

          Some(diagnostic.quickfix(Self::attribute_removal_quickfix(
            attribute,
            context.document(),
          )))
        })
        .collect()
    }
  }
}

impl IneffectiveParallelAttributeRule {
  fn attribute_removal_quickfix(
    attribute: &Attribute,
    document: &Document,
  ) -> Option<Quickfix> {
    let root = document.tree.root_node();

    let attribute_node = root
      .find_all("attribute")
      .into_iter()
      .find(|node| document.get_range(node) == attribute.range)?;

    let mut cursor = attribute_node.walk();

    let children = attribute_node.children(&mut cursor).collect::<Vec<_>>();

    let identifiers = children
      .iter()
      .filter(|node| node.kind() == "identifier")
      .collect::<Vec<_>>();

    let index = identifiers
      .iter()
      .position(|node| document.get_range(node) == attribute.name.range)?;

    let range = if identifiers.len() == 1 {
      attribute.range
    } else if let Some(next) = identifiers.get(index + 1) {
      lsp::Range {
        start: attribute.name.range.start,
        end: document.get_range(next).start,
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
        start: document.get_range(&previous).end,
        end: document.get_range(closing_bracket).start,
      }
    };

    Some(Quickfix::removal(
      range,
      format!("Remove `[{}]`", attribute.name.value),
    ))
  }
}

#[cfg(test)]
mod tests {
  use {super::*, indoc::indoc};

  #[test]
  fn parallel_with_mapped_dependency() {
    AnalyzerTest::new(indoc! {
      "
      set unstable
      set lists

      [parallel]
      foo *args: *(bar *args)
        echo {{args}}

      bar arg:
        echo {{arg}}
      "
    })
    .run();
  }

  #[test]
  fn parallel_with_single_dependency_in_each_group_warns() {
    AnalyzerTest::new(indoc! {
      "
      [parallel]
      foo: bar && baz

      bar:

      baz:
      "
    })
    .warning(
      "Recipe `foo` has only one dependency before and after `&&`, so `[parallel]` has no effect",
      lsp::Range::at(0, 0, 1, 0),
    )
    .run();
  }

  #[test]
  fn parallel_with_single_dependency_warns() {
    AnalyzerTest::new(indoc! {
      "
      [parallel]
      foo: bar
        echo \"foo\"

      bar:
        echo \"bar\"
      "
    })
    .warning(
      "Recipe `foo` has only one dependency, so `[parallel]` has no effect",
      lsp::Range::at(0, 0, 1, 0),
    )
    .run();
  }

  #[test]
  fn parallel_without_dependencies_warns() {
    AnalyzerTest::new(indoc! {
      "
      [parallel]
      foo:
        echo \"foo\"
      "
    })
    .warning(
      "Recipe `foo` has no dependencies, so `[parallel]` has no effect",
      lsp::Range::at(0, 0, 1, 0),
    )
    .run();
  }
}
