use super::*;

pub trait NodeExt {
  fn find(&self, selector: &str) -> Option<Node<'_>>;
  fn find_all(&self, selector: &str) -> Vec<Node<'_>>;
  fn get_parent(&self, kind: &str) -> Option<Node<'_>>;
  fn has_any_parent(&self, kinds: &[&str]) -> bool;
  fn siblings(&self) -> impl Iterator<Item = Node<'_>>;
}

impl NodeExt for Node<'_> {
  fn find(&self, selector: &str) -> Option<Node<'_>> {
    TreeWalker::new(*self).find(selector)
  }

  fn find_all(&self, selector: &str) -> Vec<Node<'_>> {
    TreeWalker::new(*self).find_all(selector)
  }

  fn get_parent(&self, kind: &str) -> Option<Node<'_>> {
    let mut current = *self;

    while let Some(parent) = current.parent() {
      if parent.kind() == kind {
        return Some(parent);
      }

      current = parent;
    }

    None
  }

  fn has_any_parent(&self, kinds: &[&str]) -> bool {
    kinds.iter().any(|kind| self.get_parent(kind).is_some())
  }

  fn siblings(&self) -> impl Iterator<Item = Node<'_>> {
    successors(self.next_sibling(), Node::next_sibling)
  }
}
