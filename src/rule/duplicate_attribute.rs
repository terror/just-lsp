use super::*;

const REPEATABLE_ATTRIBUTES: &[&str] = &["arg", "env", "metadata"];

#[derive(Debug, Eq, Hash, PartialEq)]
enum GroupValue {
  Cooked(String),
  Raw(String),
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
      let mut target_groups: HashMap<(usize, usize), HashSet<GroupValue>> =
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

          if attribute_name == "group" {
            let group = identifier
              .siblings()
              .take_while(|node| node.kind() != "identifier")
              .find(|node| node.kind() == "expression")
              .and_then(|argument| GroupValue::new(argument, document));

            let Some(group) = group else {
              continue;
            };

            let seen = target_groups.entry(target_key).or_default();

            if seen.contains(&group) {
              diagnostics.push(Diagnostic::error(
                format!(
                  "{} attribute `group` with value `{}` is duplicated",
                  target.target_name(),
                  group.value(),
                ),
                attribute_node.get_range(document),
              ));
            } else {
              seen.insert(group);
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

impl GroupValue {
  fn new(argument: Node, document: &Document) -> Option<Self> {
    let value = argument.find("^value")?;
    let string = value.find("^string")?;

    if string.find("format_string").is_some() {
      return None;
    }

    let group = document.get_node_text(&argument);

    Some(match group.literal() {
      Some(group) => Self::Cooked(group),
      None => Self::Raw(group),
    })
  }

  fn value(&self) -> &str {
    match self {
      Self::Cooked(value) | Self::Raw(value) => value,
    }
  }
}
