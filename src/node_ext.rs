use super::*;

pub(crate) trait NodeExt {
  fn find_child_by_kind(&self, kind: &str) -> Option<Node>;
  fn get_range(&self) -> lsp::Range;
  fn search(&self, path: &str) -> Option<Node>;
}

impl NodeExt for Node<'_> {
  fn find_child_by_kind(&self, kind: &str) -> Option<Node> {
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

  fn search(&self, path: &str) -> Option<Node> {
    let parts: Vec<&str> = path.split('>').map(str::trim).collect();

    if parts.is_empty() {
      return None;
    }

    fn collect<'a>(node: Node<'a>, kind: &str, results: &mut Vec<Node<'a>>) {
      if node.kind() == kind {
        results.push(node);
      }

      for i in 0..node.child_count() {
        if let Some(child) = node.child(i) {
          collect(child, kind, results);
        }
      }
    }

    let first_kind = parts[0];

    let mut matches = Vec::new();

    if self.kind() == first_kind {
      matches.push(*self);
    } else {
      collect(*self, first_kind, &mut matches);
    }

    if matches.is_empty() {
      return None;
    }

    for &target_kind in parts.iter().skip(1) {
      let mut next_matches = Vec::with_capacity(matches.len());

      for node in matches {
        for j in 0..node.named_child_count() {
          if let Some(child) = node.named_child(j) {
            if child.kind() == target_kind {
              next_matches.push(child);
            }
          }
        }
      }

      if next_matches.is_empty() {
        return None;
      }

      matches = next_matches;
    }

    matches.first().copied()
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
