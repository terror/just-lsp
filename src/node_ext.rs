use super::*;

pub(crate) trait NodeExt {
  fn find_child_by_kind<'a>(&self, kind: &'a str) -> Option<Node<'_>>;
  fn get_range(&self) -> lsp::Range;
}

impl NodeExt for Node<'_> {
  fn find_child_by_kind<'a>(&self, kind: &'a str) -> Option<Node<'_>> {
    (0..self.child_count())
      .filter_map(|i| self.child(i))
      .find(|child| child.kind() == kind)
  }

  fn get_range(&self) -> lsp::Range {
    lsp::Range {
      start: self.start_position().position(),
      end: self.end_position().position(),
    }
  }
}

#[cfg(test)]
mod tests {
  use {super::*, indoc::indoc, pretty_assertions::assert_eq};

  fn document(content: &str) -> Document {
    Document::try_from(lsp::DidOpenTextDocumentParams {
      text_document: lsp::TextDocumentItem {
        uri: lsp::Url::parse("file:///test.just").unwrap(),
        language_id: "just".to_string(),
        version: 1,
        text: content.to_string(),
      },
    })
    .unwrap()
  }

  #[test]
  fn find_child_by_kind() {
    let doc = document(indoc! {"
      foo:
        echo \"foo\"
    "});

    let recipes = doc.get_nodes_by_kind("recipe");
    assert_eq!(recipes.len(), 1);

    let recipe = recipes.first().unwrap();

    let header = recipe.find_child_by_kind("recipe_header");
    assert!(header.is_some());

    let body = recipe.find_child_by_kind("recipe_body");
    assert!(body.is_some());

    let nonexistent = recipe.find_child_by_kind("nonexistent");
    assert!(nonexistent.is_none());
  }
}
