use super::*;

const REPEATABLE_ATTRIBUTES: &[&str] = &["arg", "env", "group", "metadata"];

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

      for recipe in context.document().recipes() {
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

      let mut target_seen: HashMap<(usize, usize), HashSet<String>> =
        HashMap::new();

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

          if REPEATABLE_ATTRIBUTES.contains(&attribute_name.as_str()) {
            continue;
          }

          if context.builtin_attribute(&attribute_name).is_none() {
            continue;
          }

          if attribute_name == "default" && target == AttributeTarget::Recipe {
            continue;
          }

          let seen = target_seen.entry(target_key).or_default();

          if !seen.insert(attribute_name.clone()) {
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
