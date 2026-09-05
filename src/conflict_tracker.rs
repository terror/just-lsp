use super::*;

#[derive(Default)]
pub(super) struct ConflictTracker {
  groups: HashMap<String, GroupSet>,
}

impl ConflictTracker {
  pub(super) fn conflicts_with(
    &self,
    name: &TextNode,
    attributes: &[Attribute],
  ) -> bool {
    let current = GroupSet::from_attributes(attributes);

    self
      .groups
      .get(&name.value)
      .is_some_and(|previous| previous.conflicts_with(&current))
  }

  pub(super) fn record(
    &mut self,
    name: &TextNode,
    attributes: &[Attribute],
  ) -> bool {
    let current = GroupSet::from_attributes(attributes);

    let previous = self.groups.entry(name.value.clone()).or_default();

    let conflict = previous.conflicts_with(&current);

    previous.union_with(current);

    conflict
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn record() {
    #[track_caller]
    fn case(declarations: &[(&str, &[&str], bool)]) {
      let mut conflicts = ConflictTracker::default();

      for (name, attributes, expected) in declarations {
        let name = TextNode {
          value: (*name).into(),
          ..Default::default()
        };

        let attributes = attributes
          .iter()
          .map(|name| Attribute {
            name: TextNode {
              value: (*name).into(),
              ..Default::default()
            },
            ..Default::default()
          })
          .collect::<Vec<_>>();

        assert_eq!(conflicts.conflicts_with(&name, &attributes), *expected);
        assert_eq!(conflicts.record(&name, &attributes), *expected);
        assert!(conflicts.conflicts_with(&name, &attributes));
      }
    }

    case(&[("foo", &[], false), ("bar", &[], false), ("foo", &[], true)]);

    case(&[
      ("foo", &["linux"], false),
      ("foo", &["windows"], false),
      ("foo", &["linux"], true),
      ("foo", &["windows"], true),
    ]);

    case(&[
      ("foo", &["linux"], false),
      ("foo", &["linux", "windows"], true),
      ("foo", &["windows"], true),
    ]);
  }
}
