use super::*;

pub trait NodeExt {
  fn find(&self, selector: &str) -> Option<Node<'_>>;
  fn find_all(&self, selector: &str) -> Vec<Node<'_>>;
  fn get_function(&self, document: &Document) -> Option<Function>;
  fn get_parent(&self, kind: &str) -> Option<Node<'_>>;
  fn get_range(&self, document: &Document) -> lsp::Range;
  fn get_recipe(&self, document: &Document) -> Option<Recipe>;
  fn has_any_parent(&self, kinds: &[&str]) -> bool;
  fn siblings(&self) -> impl Iterator<Item = Node<'_>>;
}

fn collect_nodes_by_kind<'a>(node: Node<'a>, kind: &str) -> Vec<Node<'a>> {
  let self_match = if node.kind() == kind {
    vec![node]
  } else {
    Vec::new()
  };

  let children_matches = (0..node.child_count())
    .filter_map(|i| child_at(&node, i))
    .flat_map(|child| collect_nodes_by_kind(child, kind))
    .collect::<Vec<_>>();

  [self_match, children_matches].concat()
}

fn child_at<'a>(node: &Node<'a>, index: usize) -> Option<Node<'a>> {
  index.try_into().ok().and_then(|index| node.child(index))
}

impl NodeExt for Node<'_> {
  fn find(&self, selector: &str) -> Option<Node<'_>> {
    self.find_all(selector).into_iter().next()
  }

  fn find_all(&self, selector: &str) -> Vec<Node<'_>> {
    if selector.contains(',') {
      return selector
        .split(',')
        .map(str::trim)
        .flat_map(|sub_selector| self.find_all(sub_selector))
        .collect();
    }

    if let Some(rest) = selector.strip_prefix('^') {
      return (0..self.child_count())
        .filter_map(|i| child_at(self, i))
        .filter(|child| child.kind() == rest)
        .collect();
    }

    if selector.contains(" > ") {
      let parts: Vec<&str> = selector.split(" > ").collect();

      return parts.iter().skip(1).fold(
        self.find_all(parts[0]),
        |parents, &child_kind| {
          parents
            .iter()
            .flat_map(|parent| {
              (0..parent.child_count())
                .filter_map(|i| child_at(parent, i))
                .filter(|child| child.kind() == child_kind)
                .collect::<Vec<_>>()
            })
            .collect()
        },
      );
    }

    collect_nodes_by_kind(*self, selector)
  }

  fn get_function(&self, document: &Document) -> Option<Function> {
    let range = self.get_parent("function_definition")?.get_range(document);

    document
      .functions()
      .into_iter()
      .find(|function| function.range == range)
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

  fn get_range(&self, document: &Document) -> lsp::Range {
    lsp::Range {
      start: self.start_position().position(document),
      end: self.end_position().position(document),
    }
  }

  fn get_recipe(&self, document: &Document) -> Option<Recipe> {
    let range = self.get_parent("recipe")?.get_range(document);

    document
      .recipes()
      .into_iter()
      .find(|recipe| recipe.range == range)
  }

  fn has_any_parent(&self, kinds: &[&str]) -> bool {
    kinds.iter().any(|kind| self.get_parent(kind).is_some())
  }

  fn siblings(&self) -> impl Iterator<Item = Node<'_>> {
    successors(self.next_sibling(), Node::next_sibling)
  }
}

#[cfg(test)]
mod tests {
  use {super::*, indoc::indoc, pretty_assertions::assert_eq};

  #[test]
  fn find_basic_kind() {
    let document = Document::from(indoc! {
      "
      foo:
        echo \"foo\"

      bar:
        echo \"bar\"
      "
    });

    let root = document.tree.as_ref().unwrap().root_node();

    let recipes = root.find_all("recipe");

    assert_eq!(recipes.len(), 2);

    let recipe_texts = recipes
      .iter()
      .map(|recipe| document.get_node_text(recipe).trim().to_string())
      .collect::<Vec<_>>();

    assert_eq!(
      recipe_texts,
      vec![
        "foo:\n  echo \"foo\"".to_string(),
        "bar:\n  echo \"bar\"".to_string()
      ]
    );

    let first_recipe = root.find("recipe").unwrap();

    assert_eq!(
      document.get_node_text(&first_recipe).trim(),
      "foo:\n  echo \"foo\""
    );
  }

  #[test]
  fn find_descendant() {
    let document = Document::from(indoc! {
      "
      foo:
        echo \"foo\"

      bar arg1 arg2:
        echo \"{{ arch() }}\"
      "
    });

    let root = document.tree.as_ref().unwrap().root_node();

    let identifier_texts = root
      .find_all("identifier")
      .iter()
      .map(|node| document.get_node_text(node))
      .collect::<Vec<_>>();

    assert_eq!(
      identifier_texts,
      vec![
        "foo".to_string(),
        "bar".to_string(),
        "arg1".to_string(),
        "arg2".to_string(),
        "arch".to_string()
      ]
    );
  }

  #[test]
  fn find_direct_child() {
    let document = Document::from(indoc! {
      "
      foo:
        echo \"foo\"

      bar arg1 arg2:
        echo \"bar\"
      "
    });

    let root = document.tree.as_ref().unwrap().root_node();

    let identifiers = root.find_all("recipe_header > identifier");

    let identifier_texts = identifiers
      .iter()
      .map(|node| document.get_node_text(node))
      .collect::<Vec<_>>();

    assert_eq!(identifier_texts, vec!["foo".to_string(), "bar".to_string()]);

    let second_recipe = root.find_all("recipe")[1];

    let recipe_header = second_recipe.find("recipe_header").unwrap();

    let parameters = recipe_header.find_all("parameters > parameter");

    let parameter_texts = parameters
      .iter()
      .map(|node| document.get_node_text(node))
      .collect::<Vec<_>>();

    assert_eq!(
      parameter_texts,
      vec!["arg1".to_string(), "arg2".to_string()]
    );
  }

  #[test]
  fn find_direct_child_marker() {
    let document = Document::from(indoc! {
      "
      foo:
        echo \"foo\"

      bar arg1 arg2:
        echo \"{{ arch() }}\"
      "
    });

    let root = document.tree.as_ref().unwrap().root_node();

    let second_recipe = root.find_all("recipe")[1];

    let recipe_header = second_recipe.find("recipe_header").unwrap();
    let parameters_node = recipe_header.find("parameters").unwrap();

    let direct_parameters = parameters_node.find_all("^parameter");

    assert_eq!(direct_parameters.len(), 2);

    let parameter_texts = direct_parameters
      .iter()
      .map(|node| document.get_node_text(node))
      .collect::<Vec<_>>();

    assert_eq!(
      parameter_texts,
      vec!["arg1".to_string(), "arg2".to_string()]
    );
  }

  #[test]
  fn find_nested_child() {
    let document = Document::from(indoc! {
      "
      foo: (bar baz):
        echo foo
      "
    });

    let root = document.tree.as_ref().unwrap().root_node();

    let identifier =
      root.find("dependency_expression > expression > value > identifier");

    let identifier = identifier.unwrap();

    assert_eq!(document.get_node_text(&identifier), "baz");
  }

  #[test]
  fn find_nonexistent() {
    #[track_caller]
    fn case(selector: &str) {
      let document = Document::from("foo:\n");
      let root = document.tree.as_ref().unwrap().root_node();

      assert!(root.find(selector).is_none());
      assert!(root.find_all(selector).is_empty());
    }

    case("");
    case(" ");
    case("foo");
    case("function_call");
    case("@0");
    case("recipe[0]");
    case("^recipe[0]");
    case("recipe identifier");
  }

  #[test]
  fn find_union() {
    let document = Document::from(indoc! {
      "
      foo := \"value\"

      foo:
        echo \"foo\"

      bar:
        echo \"bar\"
      "
    });

    let root = document.tree.as_ref().unwrap().root_node();

    let recipes_and_assignments = root.find_all("recipe, assignment");

    let kinds = recipes_and_assignments
      .iter()
      .map(Node::kind)
      .collect::<Vec<_>>();

    assert_eq!(kinds, ["recipe", "recipe", "assignment"]);

    let node_texts = recipes_and_assignments
      .iter()
      .map(|node| document.get_node_text(node).trim().to_string())
      .collect::<Vec<_>>();

    assert_eq!(
      node_texts,
      vec![
        "foo:\n  echo \"foo\"".to_string(),
        "bar:\n  echo \"bar\"".to_string(),
        "foo := \"value\"".to_string()
      ]
    );

    let identifier_texts = root
      .find_all("recipe_header > identifier, function_call > identifier")
      .iter()
      .map(|node| document.get_node_text(node))
      .collect::<Vec<_>>();

    assert_eq!(identifier_texts, vec!["foo".to_string(), "bar".to_string()]);
  }
}
