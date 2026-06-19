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

      let mut diagnostics = Vec::new();
      let mut default_recipe = None;

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

          if context.builtin_attributes(&attribute_name).is_empty()
            || REPEATABLE_ATTRIBUTES.contains(&attribute_name.as_str())
          {
            continue;
          }

          if attribute_name == "default" && target == AttributeTarget::Recipe {
            let Some(recipe_name) = parent
              .find("recipe_header > identifier")
              .map(|node| document.get_node_text(&node))
            else {
              continue;
            };

            if default_recipe.replace(recipe_name.clone()).is_some() {
              diagnostics.push(Diagnostic::error(
                format!(
                  "Recipe `{recipe_name}` has duplicate `[default]` attribute, which may only appear once per module"
                ),
                attribute_node.get_range(document),
              ));
            }

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
