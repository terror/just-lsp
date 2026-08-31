use super::*;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Quickfix {
  pub edits: Vec<lsp::TextEdit>,
  pub range: lsp::Range,
  pub title: String,
}

impl Quickfix {
  #[must_use]
  pub fn attribute_removal(
    attribute: &Attribute,
    document: &Document,
  ) -> Option<Self> {
    let root = document.tree.as_ref()?.root_node();

    let attribute_node = root
      .find_all("attribute")
      .into_iter()
      .find(|node| node.get_range(document) == attribute.range)?;

    let mut cursor = attribute_node.walk();

    let children = attribute_node.children(&mut cursor).collect::<Vec<_>>();

    let identifiers = children
      .iter()
      .filter(|node| node.kind() == "identifier")
      .collect::<Vec<_>>();

    let index = identifiers
      .iter()
      .position(|node| node.get_range(document) == attribute.name.range)?;

    let range = if identifiers.len() == 1 {
      attribute.range
    } else if let Some(next) = identifiers.get(index + 1) {
      lsp::Range {
        start: attribute.name.range.start,
        end: next.get_range(document).start,
      }
    } else {
      let identifier = identifiers[index];

      let comma = children.iter().rev().find(|node| {
        node.kind() == "," && node.end_byte() <= identifier.start_byte()
      })?;

      let previous = comma.prev_sibling()?;

      let closing_bracket = children.iter().find(|node| {
        node.kind() == "]" && node.start_byte() >= identifier.end_byte()
      })?;

      lsp::Range {
        start: previous.get_range(document).end,
        end: closing_bracket.get_range(document).start,
      }
    };

    Some(Self::removal(
      range,
      format!("Remove `[{}]`", attribute.name.value),
    ))
  }

  #[must_use]
  pub fn removal(range: lsp::Range, title: impl Into<String>) -> Self {
    Self {
      edits: vec![lsp::TextEdit {
        range,
        new_text: String::new(),
      }],
      range,
      title: title.into(),
    }
  }

  #[must_use]
  pub fn replacement(name: &TextNode, replacement: impl Into<String>) -> Self {
    let replacement = replacement.into();

    Self {
      edits: vec![lsp::TextEdit {
        range: name.range,
        new_text: replacement.clone(),
      }],
      range: name.range,
      title: format!("Replace `{}` with `{replacement}`", name.value),
    }
  }

  #[must_use]
  pub fn setting_attribute(
    setting: &Setting,
    document: &Document,
    attribute: &str,
    replacement: &str,
  ) -> Self {
    let line = document
      .content
      .line(setting.range.start.line as usize)
      .to_string();

    let line = line.replacen(&setting.name.value, replacement, 1);

    Self {
      edits: vec![lsp::TextEdit {
        range: setting.range,
        new_text: format!("[{attribute}]\n{line}"),
      }],
      range: setting.name.range,
      title: format!(
        "Replace `{}` with `[{attribute}] set {replacement}`",
        setting.name.value
      ),
    }
  }
}

#[cfg(test)]
mod tests {
  use {super::*, pretty_assertions::assert_eq};

  #[test]
  fn attribute_removal_handles_item_positions() {
    fn case(input: &str, expected: &str) {
      let mut document = Document::from(input);

      let attribute = document
        .attributes()
        .into_iter()
        .find(|attribute| attribute.name.value == "parallel")
        .unwrap();

      let quickfix =
        Quickfix::attribute_removal(&attribute, &document).unwrap();

      let text_edit = quickfix.edits.into_iter().next().unwrap();

      let change = lsp::TextDocumentContentChangeEvent {
        range: Some(text_edit.range),
        range_length: None,
        text: text_edit.new_text,
      };

      let edit = document.content.build_edit(&change);

      document.content.apply_edit(&edit);

      assert_eq!(document.content.to_string(), expected);
    }

    case("[parallel]\nfoo:\n", "foo:\n");
    case(
      "[parallel, private, linux]\nfoo:\n",
      "[private, linux]\nfoo:\n",
    );
    case(
      "[private, parallel, linux]\nfoo:\n",
      "[private, linux]\nfoo:\n",
    );
    case("[private, parallel]\nfoo:\n", "[private]\nfoo:\n");
    case(
      "[parallel, description: 'a,b']\nfoo:\n",
      "[description: 'a,b']\nfoo:\n",
    );
    case(
      "[description: 'a,b' , parallel]\nfoo:\n",
      "[description: 'a,b']\nfoo:\n",
    );
  }
}
