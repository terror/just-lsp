use super::*;

#[derive(Default)]
pub(super) struct ConflictTracker {
  groups: HashMap<String, GroupSet>,
}

impl ConflictTracker {
  pub(super) fn conflicts_with(&self, declaration: &impl Declaration) -> bool {
    let current = declaration.groups();

    self
      .groups
      .get(&declaration.name().value)
      .is_some_and(|previous| previous.conflicts_with(&current))
  }

  pub(super) fn record(&mut self, declaration: &impl Declaration) -> bool {
    let current = declaration.groups();

    let previous = self
      .groups
      .entry(declaration.name().value.clone())
      .or_default();

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
        let unexport = Unexport {
          attributes: attributes
            .iter()
            .map(|name| Attribute {
              name: TextNode {
                value: (*name).into(),
                ..Default::default()
              },
              ..Default::default()
            })
            .collect(),
          name: TextNode {
            value: (*name).into(),
            ..Default::default()
          },
          range: lsp::Range::default(),
        };

        assert_eq!(conflicts.conflicts_with(&unexport), *expected);
        assert_eq!(conflicts.record(&unexport), *expected);

        assert!(conflicts.conflicts_with(&unexport));
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
