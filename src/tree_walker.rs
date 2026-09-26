use super::*;

pub(super) struct TreeWalker<'a> {
  node: Node<'a>,
}

impl<'a> TreeWalker<'a> {
  pub(super) fn find(&self, selector: &str) -> Option<Node<'a>> {
    self
      .visit_matches(selector, ControlFlow::Break)
      .break_value()
  }

  pub(super) fn find_all(&self, selector: &str) -> Vec<Node<'a>> {
    let mut nodes = Vec::new();

    let _ = self.visit_matches(selector, |node| {
      nodes.push(node);
      ControlFlow::<()>::Continue(())
    });

    nodes
  }

  pub(super) fn new(node: Node<'a>) -> Self {
    Self { node }
  }

  fn visit_matches<B>(
    &self,
    selector: &str,
    mut visit: impl FnMut(Node<'a>) -> ControlFlow<B>,
  ) -> ControlFlow<B> {
    let union = selector.contains(',');

    for selector in selector.split(',') {
      let selector = if union { selector.trim() } else { selector };

      if let Some(kind) = selector.strip_prefix('^') {
        for child in self.node.children(&mut self.node.walk()) {
          if child.kind() == kind {
            visit(child)?;
          }
        }

        continue;
      }

      let mut kinds = selector.split(" > ");

      let kind = kinds.next().unwrap();
      let has_children = kinds.next().is_some();

      self.walk(|node, _| {
        if node.kind() == kind {
          if has_children {
            Self::new(node).walk(|node, depth| {
              let mut kinds = selector.split(" > ");

              if kinds.nth(depth as usize) != Some(node.kind()) {
                return ControlFlow::Continue(false);
              }

              if kinds.next().is_none() {
                visit(node)?;
                return ControlFlow::Continue(false);
              }

              ControlFlow::Continue(true)
            })?;
          } else {
            visit(node)?;
          }
        }

        ControlFlow::Continue(true)
      })?;
    }

    ControlFlow::Continue(())
  }

  fn walk<B>(
    &self,
    mut visit: impl FnMut(Node<'a>, u32) -> ControlFlow<B, bool>,
  ) -> ControlFlow<B> {
    let mut cursor = self.node.walk();

    loop {
      if visit(cursor.node(), cursor.depth())? && cursor.goto_first_child() {
        continue;
      }

      while !cursor.goto_next_sibling() {
        if !cursor.goto_parent() {
          return ControlFlow::Continue(());
        }
      }
    }
  }
}

#[cfg(test)]
mod tests {
  use {super::*, pretty_assertions::assert_eq};

  #[derive(Debug)]
  struct Test {
    document: Document,
    path: Vec<u32>,
  }

  impl Test {
    #[track_caller]
    fn find(self, selector: &str, expected: Option<&str>) {
      assert_eq!(
        TreeWalker::new(self.node())
          .find(selector)
          .map(|node| self.document.get_node_text(&node))
          .as_deref(),
        expected,
      );
    }

    #[track_caller]
    fn find_all(self, selector: &str, expected: &[&str]) {
      assert_eq!(
        TreeWalker::new(self.node())
          .find_all(selector)
          .iter()
          .map(|node| self.document.get_node_text(node))
          .collect::<Vec<_>>(),
        expected,
      );
    }

    #[track_caller]
    fn new(content: &str) -> Self {
      let document = Document::from(content);

      assert!(!document.tree.root_node().has_error());

      Self {
        document,
        path: Vec::new(),
      }
    }

    #[track_caller]
    fn node(&self) -> Node<'_> {
      let mut node = self.document.tree.root_node();

      for &index in &self.path {
        node = node.named_child(index).unwrap();
      }

      node
    }

    fn path(self, path: &[u32]) -> Self {
      Self {
        path: path.to_vec(),
        ..self
      }
    }
  }

  #[test]
  fn find_all_anonymous() {
    Test::new("foo:\n").find_all(":", &[":"]);
  }

  #[test]
  fn find_all_deep() {
    Test::new(&format!(
      "foo := {}bar{}\n",
      "(".repeat(4096),
      ")".repeat(4096),
    ))
    .find_all("identifier", &["foo", "bar"]);
  }

  #[test]
  fn find_all_descendant() {
    Test::new(indoc! {
      "
      foo:
      bar baz qux:
        echo {{ foo() }}
      "
    })
    .find_all("identifier", &["foo", "bar", "baz", "qux", "foo"]);
  }

  #[test]
  fn find_all_direct_child_anonymous() {
    Test::new("foo:\n").path(&[0, 0]).find_all("^:", &[":"]);
  }

  #[test]
  fn find_all_direct_child_excludes_descendants() {
    Test::new("foo bar baz:\n")
      .path(&[0, 0, 1])
      .find_all("^identifier", &[]);
  }

  #[test]
  fn find_all_direct_child_marker() {
    Test::new("foo bar baz:\n")
      .path(&[0, 0, 1])
      .find_all("^parameter", &["bar", "baz"]);
  }

  #[test]
  fn find_all_direct_child_parameters() {
    Test::new("foo bar baz:\n")
      .find_all("parameters > parameter", &["bar", "baz"]);
  }

  #[test]
  fn find_all_includes_subtree_root() {
    let test = Test::new(indoc! {
      "
      foo:
      bar:
      "
    })
    .path(&[0]);

    let node = test.node();

    assert_eq!(TreeWalker::new(node).find_all("recipe"), [node]);
  }

  #[test]
  fn find_all_leaf() {
    let test = Test::new("foo:\n").path(&[0, 0, 0]);

    let node = test.node();

    assert_eq!(TreeWalker::new(node).find_all("identifier"), [node]);
  }

  #[test]
  fn find_all_leaf_no_children() {
    Test::new("foo:\n")
      .path(&[0, 0, 0])
      .find_all("^identifier", &[]);
  }

  #[test]
  fn find_all_nested_child_order() {
    Test::new("foo := [[bar], baz]\n").find_all(
      "list_elements > expression > value > identifier",
      &["baz", "bar"],
    );
  }

  #[test]
  fn find_all_subtree_ancestors() {
    Test::new(indoc! {
      "
      foo:
      bar baz:
      "
    })
    .path(&[0])
    .find_all("source_file", &[]);
  }

  #[test]
  fn find_all_subtree_siblings() {
    Test::new(indoc! {
      "
      foo:
      bar baz:
      "
    })
    .path(&[0])
    .find_all("parameters", &[]);
  }

  #[test]
  fn find_all_union() {
    let test = Test::new(indoc! {
      "
      foo := bar
      baz:
      qux:
      "
    });

    let root = test.node();

    let (assignment, first, second) = (
      root.named_child(0).unwrap(),
      root.named_child(1).unwrap(),
      root.named_child(2).unwrap(),
    );

    assert_eq!(
      TreeWalker::new(root).find_all(" , recipe, foo, assignment, recipe, "),
      [first, second, assignment, first, second],
    );
  }

  #[test]
  fn find_all_union_paths() {
    Test::new(indoc! {
      "
      foo := bar
      baz:
      "
    })
    .find_all(
      "recipe_header > identifier, assignment > identifier",
      &["baz", "foo"],
    );
  }

  #[test]
  fn find_basic_kind() {
    Test::new(indoc! {
      "
      foo:
      bar:
      "
    })
    .find("recipe", Some("foo:\n"));
  }

  #[test]
  fn find_direct_child_marker() {
    Test::new("foo bar baz:\n")
      .path(&[0, 0, 1])
      .find("^parameter", Some("bar"));
  }

  #[test]
  fn find_empty_child_selector() {
    Test::new("foo:\n").find("recipe > ", None);
  }

  #[test]
  fn find_empty_selector() {
    Test::new("foo:\n").find("", None);
  }

  #[test]
  fn find_leading_whitespace() {
    Test::new("foo:\n").find(" recipe", None);
  }

  #[test]
  fn find_trailing_whitespace() {
    Test::new("foo:\n").find("recipe ", None);
  }

  #[test]
  fn find_union() {
    Test::new("foo := [[bar], baz]\n").find(
      "qux, list_elements > expression > value > identifier, identifier",
      Some("baz"),
    );
  }

  #[test]
  fn find_unsupported_marker_path() {
    Test::new("foo:\n").find("^recipe > recipe_header", None);
  }
}
