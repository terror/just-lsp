use super::*;

#[derive(Debug, Clone, PartialEq)]
pub struct Recipe {
  pub attributes: Vec<Attribute>,
  pub body: Vec<TextNode>,
  pub content: String,
  pub dependencies: Vec<Dependency>,
  pub name: TextNode,
  pub parameters: Vec<Parameter>,
  pub range: lsp::Range,
  pub shebang: Option<TextNode>,
}

impl Recipe {
  pub(crate) fn body_lines(&self) -> impl Iterator<Item = RecipeLine<'_>> {
    self.body.iter().filter_map(RecipeLine::parse)
  }

  pub(crate) fn completion_item(&self) -> lsp::CompletionItem {
    lsp::CompletionItem {
      label: self.name.value.clone(),
      kind: Some(lsp::CompletionItemKind::FUNCTION),
      documentation: Some(lsp::Documentation::MarkupContent(
        lsp::MarkupContent {
          kind: lsp::MarkupKind::PlainText,
          value: self.content.clone(),
        },
      )),
      insert_text: Some(self.name.value.clone()),
      insert_text_format: Some(lsp::InsertTextFormat::PLAIN_TEXT),
      ..Default::default()
    }
  }

  #[must_use]
  pub fn find_attribute(&self, name: &str) -> Option<&Attribute> {
    self
      .attributes
      .iter()
      .find(|attribute| attribute.name.value == name)
  }

  #[cfg(test)]
  #[must_use]
  pub(crate) fn groups(&self) -> GroupSet {
    GroupSet::from_attributes(&self.attributes)
  }

  #[must_use]
  pub fn has_attribute(&self, name: &str) -> bool {
    self
      .attributes
      .iter()
      .any(|attribute| attribute.name.value == name)
  }

  fn has_required_parameters(&self) -> bool {
    self
      .parameters
      .iter()
      .any(|parameter| self.is_required(parameter))
  }

  pub(crate) fn is_option(&self, parameter: &str) -> bool {
    self
      .attributes
      .iter()
      .filter(|attribute| {
        attribute.arg_parameter().as_deref() == Some(parameter)
      })
      .any(|attribute| {
        attribute.keyword_argument("long").is_some()
          || attribute.keyword_argument("short").is_some()
      })
  }

  pub(crate) fn is_required(&self, parameter: &Parameter) -> bool {
    let attribute = self.attributes.iter().find(|attribute| {
      attribute.arg_parameter().as_deref() == Some(parameter.name.as_str())
    });

    let keyword =
      |name| attribute.and_then(|attribute| attribute.keyword_argument(name));

    let min = keyword("min")
      .and_then(AttributeArgument::literal)
      .and_then(|min| min.parse::<usize>().ok())
      .unwrap_or_default();

    let optional_variadic =
      parameter.kind == ParameterKind::Variadic(VariadicType::ZeroOrMore);

    parameter.default_value.is_none()
      && keyword("flag").is_none()
      && (!optional_variadic || min > 0)
  }

  #[must_use]
  pub fn run_arguments(
    &self,
    justfile: &lsp::Url,
    modules: &[String],
  ) -> Vec<Value> {
    let path = modules
      .iter()
      .map(String::as_str)
      .chain([self.name.value.as_str()])
      .collect::<Vec<_>>()
      .join("::");

    vec![
      Value::String(path),
      Value::String(justfile.to_string()),
      Value::Bool(self.has_required_parameters()),
    ]
  }

  #[must_use]
  pub fn runs_as_script(&self, default_script: bool) -> bool {
    if self.has_attribute("shell") {
      return false;
    }

    self.has_attribute("script") || self.shebang.is_some() || default_script
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn body_lines() {
    let recipe = Recipe {
      attributes: Vec::new(),
      body: vec![
        TextNode {
          value: "  foo \\".into(),
          range: lsp::Range::at(1, 0, 1, 7),
        },
        TextNode {
          value: " \t".into(),
          range: lsp::Range::at(2, 0, 2, 2),
        },
        TextNode {
          value: "\tbar".into(),
          range: lsp::Range::at(3, 0, 3, 4),
        },
      ],
      content: String::new(),
      dependencies: Vec::new(),
      name: TextNode::default(),
      parameters: Vec::new(),
      range: lsp::Range::default(),
      shebang: None,
    };

    assert_eq!(
      recipe
        .body_lines()
        .map(|line| (line.number, line.indent, line.kind, line.continues))
        .collect::<Vec<_>>(),
      [
        (1, "  ", IndentKind::Spaces, true),
        (3, "\t", IndentKind::Tabs, false),
      ],
    );
  }

  #[test]
  fn find_attribute() {
    let recipe = Recipe {
      attributes: ["foo", "bar"]
        .into_iter()
        .map(|name| Attribute {
          name: TextNode {
            value: name.into(),
            ..Default::default()
          },
          ..Default::default()
        })
        .collect(),
      body: Vec::new(),
      content: String::new(),
      dependencies: Vec::new(),
      name: TextNode::default(),
      parameters: Vec::new(),
      range: lsp::Range::default(),
      shebang: None,
    };

    assert_eq!(recipe.find_attribute("foo"), Some(&recipe.attributes[0]));
    assert_eq!(recipe.find_attribute("bar"), Some(&recipe.attributes[1]));
    assert_eq!(recipe.find_attribute("baz"), None);
  }

  #[test]
  fn groups() {
    #[track_caller]
    fn case(attributes: &[&str], expected: &GroupSet) {
      let recipe = Recipe {
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
        body: Vec::new(),
        content: String::new(),
        dependencies: Vec::new(),
        name: TextNode::default(),
        parameters: Vec::new(),
        range: lsp::Range::default(),
        shebang: None,
      };

      assert_eq!(&recipe.groups(), expected);
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
    let recipe = Recipe {
      attributes: ["foo", "bar"]
        .into_iter()
        .map(|name| Attribute {
          name: TextNode {
            value: name.into(),
            ..Default::default()
          },
          ..Default::default()
        })
        .collect(),
      body: Vec::new(),
      content: String::new(),
      dependencies: Vec::new(),
      name: TextNode::default(),
      parameters: Vec::new(),
      range: lsp::Range::default(),
      shebang: None,
    };

    assert!(recipe.has_attribute("foo"));
    assert!(recipe.has_attribute("bar"));
    assert!(!recipe.has_attribute("baz"));
  }

  #[test]
  fn has_required_parameters() {
    #[track_caller]
    fn case(source: &str, expected: bool) {
      assert_eq!(
        Document::from(source).recipes()[0].has_required_parameters(),
        expected,
      );
    }

    case("foo bar:", true);
    case("foo bar='baz':", false);
    case("foo +bar:", true);
    case("foo *bar:", false);
    case("[arg('bar', long, flag)]\nfoo bar:", false);
    case("[arg('bar', long, min='1')]\nfoo *bar:", true);
  }

  #[test]
  fn runs_as_script() {
    #[track_caller]
    fn case(attributes: &[&str], shebang: bool, expected: [bool; 2]) {
      let recipe = Recipe {
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
        body: Vec::new(),
        content: String::new(),
        dependencies: Vec::new(),
        name: TextNode::default(),
        parameters: Vec::new(),
        range: lsp::Range::default(),
        shebang: shebang.then(TextNode::default),
      };

      assert_eq!(
        [recipe.runs_as_script(false), recipe.runs_as_script(true)],
        expected,
      );
    }

    case(&[], false, [false, true]);
    case(&["script"], false, [true, true]);
    case(&[], true, [true, true]);
    case(&["shell"], false, [false, false]);
    case(&["shell", "script"], false, [false, false]);
    case(&["shell"], true, [false, false]);
  }
}
