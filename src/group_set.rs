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
