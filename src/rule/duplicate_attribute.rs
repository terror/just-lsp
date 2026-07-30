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

      let (mut diagnostics, mut default_groups) = (Vec::new(), GroupSet::default());

      for recipe in context.document().recipes() {
        for attribute in recipe
          .attributes
          .iter()
          .filter(|attribute| attribute.name.value == "default")
        {
          let current = GroupSet::from_attributes(&recipe.attributes);

          let duplicate = default_groups.conflicts_with(&current);

          default_groups.union_with(current);

          if duplicate {
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
      let mut target_groups: HashMap<
        (usize, usize),
        HashSet<(bool, String)>,
      > = HashMap::new();

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
              .and_then(|argument| Self::group(argument, document));

            let Some((decoded, group)) = group else {
              continue;
            };

            let seen = target_groups.entry(target_key).or_default();

            if !seen.insert((decoded, group.clone())) {
              diagnostics.push(Diagnostic::error(
                format!(
                  "{} attribute `group` with value `{group}` is duplicated",
                  target.target_name()
                ),
                attribute_node.get_range(document),
              ));
            }

            continue;
          }

          if REPEATABLE_ATTRIBUTES.contains(&attribute_name.as_str()) {
            continue;
          }

          if context.builtin_attributes(&attribute_name).is_empty() {
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

impl DuplicateAttributeRule {
  fn group(argument: Node, document: &Document) -> Option<(bool, String)> {
    let value = argument.find("^value")?;

    let mut cursor = value.walk();

    let children = value.named_children(&mut cursor).collect::<Vec<_>>();

    let [string] = children.as_slice() else {
      return None;
    };

    if string.kind() != "string" || string.find("format_string").is_some() {
      return None;
    }

    let group = document.get_node_text(&argument);

    Some(match group.literal() {
      Some(group) => (true, group),
      None => (false, group),
    })
  }
}
