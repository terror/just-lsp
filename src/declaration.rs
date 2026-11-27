use super::*;

pub(crate) trait Declaration {
  fn attributes(&self) -> &[Attribute];

  #[must_use]
  fn find_attribute(&self, name: &str) -> Option<&Attribute> {
    self
      .attributes()
      .iter()
      .find(|attribute| attribute.name.value == name)
  }

  #[must_use]
  fn groups(&self) -> GroupSet {
    GroupSet::from_attributes(self.attributes())
  }

  #[must_use]
  fn has_attribute(&self, name: &str) -> bool {
    self.find_attribute(name).is_some()
  }

  #[must_use]
  fn is_enabled(&self) -> bool {
    Attribute::is_enabled(self.attributes())
  }

  fn name(&self) -> &TextNode;

  fn range(&self) -> lsp::Range;
}

macro_rules! impl_declaration {
  ($($ty:ty),* $(,)?) => {
    $(
      impl Declaration for $ty {
        fn attributes(&self) -> &[Attribute] {
          &self.attributes
        }

        fn name(&self) -> &TextNode {
          &self.name
        }

        fn range(&self) -> lsp::Range {
          self.range
        }
      }
    )*
  };
}

impl_declaration!(
  Alias,
  Function,
  ModuleDeclaration,
  Recipe,
  Setting,
  Unexport,
  Variable,
);

impl<T: Declaration> Declaration for Located<T> {
  fn attributes(&self) -> &[Attribute] {
    (**self).attributes()
  }

  fn name(&self) -> &TextNode {
    (**self).name()
  }

  fn range(&self) -> lsp::Range {
    (**self).range()
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  fn unexport(attributes: &[&str]) -> Unexport {
    Unexport {
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
      name: TextNode::default(),
      range: lsp::Range::default(),
    }
  }

  #[test]
  fn find_attribute() {
    let unexport = unexport(&["foo", "bar"]);

    assert_eq!(
      unexport.find_attribute("foo"),
      Some(&unexport.attributes[0])
    );

    assert_eq!(
      unexport.find_attribute("bar"),
      Some(&unexport.attributes[1])
    );

    assert_eq!(unexport.find_attribute("baz"), None);
  }

  #[test]
  fn groups() {
    #[track_caller]
    fn case(attributes: &[&str], expected: &GroupSet) {
      assert_eq!(&unexport(attributes).groups(), expected);
    }

    case(&[], &GroupSet::from([Group::Any]));
    case(&["private"], &GroupSet::from([Group::Any]));
    case(&["linux"], &GroupSet::from([Group::Linux]));

    case(
      &["linux", "windows"],
      &GroupSet::from([Group::Linux, Group::Windows]),
    );

    case(
      &[
        "linux",
        "windows",
        "macos",
        "unix",
        "dragonfly",
        "freebsd",
        "netbsd",
        "openbsd",
      ],
      &GroupSet::from([
        Group::Android,
        Group::Dragonfly,
        Group::Freebsd,
        Group::Linux,
        Group::Macos,
        Group::Netbsd,
        Group::Openbsd,
        Group::Windows,
      ]),
    );
  }

  #[test]
  fn has_attribute() {
    let unexport = unexport(&["foo", "bar"]);

    assert!(unexport.has_attribute("foo"));
    assert!(unexport.has_attribute("bar"));
    assert!(!unexport.has_attribute("baz"));
  }
}
