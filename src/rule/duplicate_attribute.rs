use super::*;

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
enum AttributeKey {
  Group(StringLiteral),
  Name(String),
}

impl Display for AttributeKey {
  fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
    match self {
      Self::Group(group) => {
        write!(f, "`group` with value `{}`", group.cooked)
      }
      Self::Name(name) => write!(f, "`{name}`"),
    }
  }
}

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

      let mut seen = HashSet::new();

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

          let key = match attribute_name.as_str() {
            "arg" | "env" | "metadata" => continue,
            "default" if target == AttributeTarget::Recipe => continue,
            "group" => {
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

              AttributeKey::Group(group)
            }
            _ if context.builtin_attribute(&attribute_name).is_some() => {
              AttributeKey::Name(attribute_name)
            }
            _ => continue,
          };

          if !seen.insert((target_key, key.clone())) {
            diagnostics.push(Diagnostic::error(
              format!(
                "{} attribute {key} is duplicated",
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
