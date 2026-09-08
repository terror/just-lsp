use super::*;

const REPEATABLE_ATTRIBUTES: &[&str] = &["arg", "env", "metadata"];

define_rule! {
  DuplicateAttributeRule {
    id: "duplicate-attribute",
    message: "duplicate attribute",
    run(context) {
      let Some(tree) = context.tree() else {
        return Vec::new();
      };

      let document = context.document();

      let (mut diagnostics, mut conflicts) = (Vec::new(), ConflictTracker::default());

      for recipe in document.recipes() {
        for attribute in recipe
          .attributes
          .iter()
          .filter(|attribute| attribute.name.value == "default")
        {
          if conflicts.record(&attribute.name, &recipe.attributes) {
            diagnostics.push(Diagnostic::error(
              format!(
                "Recipe `{}` has duplicate `[default]` attribute, which may only appear once per module",
                recipe.name.value
              ),
              attribute.range,
            ));
          }
        }
      }

      let mut target_seen = HashSet::new();
      let mut target_groups = HashSet::new();

      for attribute_node in tree.root_node().find_all("attribute") {
        let Some(parent) = attribute_node.parent() else {
          continue;
        };

        let Some(target) = AttributeTarget::try_from_kind(parent.kind()) else {
          continue;
        };

        let target_key = (parent.start_byte(), parent.end_byte());

        for identifier in attribute_node.find_all("^identifier") {
          let attribute_name = document.get_node_text(&identifier);

          if attribute_name == "group" {
            let group = identifier
              .siblings()
              .take_while(|node| node.kind() != "identifier")
              .find(|node| node.kind() == "expression")
              .and_then(|argument| {
                let value = argument.find("^value")?;

                let string = value.find("^string")?;

                if string.byte_range() != argument.byte_range() {
                  return None;
                }

                StringLiteral::parse(&document.get_node_text(&string)).ok()?
              });

            let Some(group) = group else {
              continue;
            };

            if !target_groups.insert((target_key, group.clone())) {
              diagnostics.push(Diagnostic::error(
                format!(
                  "{} attribute `group` with value `{}` is duplicated",
                  target.target_name(),
                  group.cooked,
                ),
                attribute_node.get_range(document),
              ));
            }

            continue;
          }

          if REPEATABLE_ATTRIBUTES.contains(&attribute_name.as_str()) {
            continue;
          }

          if context.builtin_attribute(&attribute_name).is_none() {
            continue;
          }

          if attribute_name == "default" && target == AttributeTarget::Recipe {
            continue;
          }

          if !target_seen.insert((target_key, attribute_name.clone())) {
            diagnostics.push(Diagnostic::error(
              format!(
                "{} attribute `{attribute_name}` is duplicated",
                target.target_name()
              ),
              attribute_node.get_range(document),
            ));
          }
        }
      }

      diagnostics
    }
  }
}
