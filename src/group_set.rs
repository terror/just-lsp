use super::*;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct GroupSet(HashSet<Group>);

impl GroupSet {
  #[must_use]
  pub fn conflicts_with(&self, other: &Self) -> bool {
    self
      .0
      .iter()
      .any(|a| other.0.iter().any(|b| a.conflicts_with(*b)))
  }

  fn from_attribute(attribute: &str) -> Option<Self> {
    match attribute {
      "android" => Some(Self::from([Group::Android])),
      "dragonfly" => Some(Self::from([Group::Dragonfly])),
      "freebsd" => Some(Self::from([Group::Freebsd])),
      "linux" => Some(Self::from([Group::Linux])),
      "macos" => Some(Self::from([Group::Macos])),
      "netbsd" => Some(Self::from([Group::Netbsd])),
      "openbsd" => Some(Self::from([Group::Openbsd])),
      "unix" => Some(Self::from([
        Group::Android,
        Group::Dragonfly,
        Group::Freebsd,
        Group::Linux,
        Group::Macos,
        Group::Netbsd,
        Group::Openbsd,
      ])),
      "windows" => Some(Self::from([Group::Windows])),
      _ => None,
    }
  }

  #[must_use]
  pub fn from_attributes(attributes: &[Attribute]) -> Self {
    let mut groups = Self::default();

    for attribute in attributes {
      if let Some(targets) = Self::from_attribute(&attribute.name.value) {
        groups.union_with(targets);
      }
    }

    if groups.is_empty() {
      Self::from([Group::Any])
    } else {
      groups
    }
  }

  #[must_use]
  pub fn insert(&mut self, group: Group) -> bool {
    self.0.insert(group)
  }

  #[must_use]
  pub fn is_empty(&self) -> bool {
    self.0.is_empty()
  }

  pub fn union_with(&mut self, other: Self) {
    self.0.extend(other.0);
  }
}

impl<const N: usize> From<[Group; N]> for GroupSet {
  fn from(groups: [Group; N]) -> Self {
    Self(HashSet::from(groups))
  }
}

impl FromIterator<Group> for GroupSet {
  fn from_iter<T: IntoIterator<Item = Group>>(iter: T) -> Self {
    Self(iter.into_iter().collect())
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn conflicts_with() {
    assert!(
      !GroupSet::from([Group::Linux])
        .conflicts_with(&GroupSet::from([Group::Windows]))
    );

    assert!(
      GroupSet::from([Group::Linux])
        .conflicts_with(&GroupSet::from([Group::Linux]))
    );

    assert!(
      GroupSet::from([Group::Any])
        .conflicts_with(&GroupSet::from([Group::Windows]))
    );
  }

  #[test]
  fn from_attribute() {
    for (attribute, expected) in [
      ("android", Some(GroupSet::from([Group::Android]))),
      ("dragonfly", Some(GroupSet::from([Group::Dragonfly]))),
      ("freebsd", Some(GroupSet::from([Group::Freebsd]))),
      ("linux", Some(GroupSet::from([Group::Linux]))),
      ("macos", Some(GroupSet::from([Group::Macos]))),
      ("netbsd", Some(GroupSet::from([Group::Netbsd]))),
      ("openbsd", Some(GroupSet::from([Group::Openbsd]))),
      (
        "unix",
        Some(GroupSet::from([
          Group::Android,
          Group::Dragonfly,
          Group::Freebsd,
          Group::Linux,
          Group::Macos,
          Group::Netbsd,
          Group::Openbsd,
        ])),
      ),
      ("windows", Some(GroupSet::from([Group::Windows]))),
      ("foo", None),
    ] {
      assert_eq!(GroupSet::from_attribute(attribute), expected);
    }
  }

  #[test]
  fn from_attributes() {
    assert_eq!(GroupSet::from_attributes(&[]), GroupSet::from([Group::Any]));

    assert_eq!(
      GroupSet::from_attributes(&[
        Attribute {
          name: TextNode {
            value: "linux".into(),
            ..Default::default()
          },
          ..Default::default()
        },
        Attribute {
          name: TextNode {
            value: "private".into(),
            ..Default::default()
          },
          ..Default::default()
        },
        Attribute {
          name: TextNode {
            value: "windows".into(),
            ..Default::default()
          },
          ..Default::default()
        },
      ]),
      GroupSet::from([Group::Linux, Group::Windows])
    );
  }

  #[test]
  fn from_iterator() {
    assert_eq!(
      [Group::Linux, Group::Linux, Group::Windows]
        .into_iter()
        .collect::<GroupSet>(),
      GroupSet::from([Group::Linux, Group::Windows])
    );
  }

  #[test]
  fn insert() {
    let mut groups = GroupSet::default();

    assert!(groups.insert(Group::Linux));
    assert!(!groups.insert(Group::Linux));

    assert_eq!(groups, GroupSet::from([Group::Linux]));
  }

  #[test]
  fn is_empty() {
    assert!(GroupSet::default().is_empty());
    assert!(!GroupSet::from([Group::Linux]).is_empty());
  }

  #[test]
  fn union_with() {
    let mut groups = GroupSet::from([Group::Linux]);

    groups.union_with(GroupSet::from([Group::Linux, Group::Windows]));

    assert_eq!(groups, GroupSet::from([Group::Linux, Group::Windows]));
  }
}
