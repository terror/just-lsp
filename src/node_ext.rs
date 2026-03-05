use super::*;

pub(crate) trait NodeExt {
  fn find(&self, selector: &str) -> Option<Node<'_>>;
  fn find_all(&self, selector: &str) -> Vec<Node<'_>>;
  fn find_siblings_until(&self, kind: &str, until: &str) -> Vec<Node<'_>>;
  fn get_parent(&self, kind: &str) -> Option<Node<'_>>;
  fn get_range(&self, document: &Document) -> lsp::Range;
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

fn collect_descendants_by_kind<'a>(
  node: Node<'a>,
  kind: &str,
) -> Vec<Node<'a>> {
  (0..node.child_count())
    .filter_map(|i| child_at(&node, i))
    .flat_map(|child| {
      let self_match = if child.kind() == kind {
        vec![child]
      } else {
        Vec::new()
      };

      let descendants = collect_descendants_by_kind(child, kind);

      [self_match, descendants].concat()
    })
    .collect()
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

    if let Some(position_str) = selector.strip_prefix('@') {
      return position_str
        .parse::<usize>()
        .ok()
        .and_then(|position| child_at(self, position))
        .map_or_else(Vec::new, |child| vec![child]);
    }

    if let Some(rest) = selector.strip_prefix('^') {
      if rest.contains('[') && rest.ends_with(']') {
        let parts: Vec<&str> = rest.split('[').collect();

        if parts.len() == 2 {
          let (kind, index_str) = (parts[0], &parts[1][..parts[1].len() - 1]);

          if let Ok(index) = index_str.parse::<usize>() {
            let direct_children = (0..self.child_count())
              .filter_map(|i| child_at(self, i))
              .filter(|child| child.kind() == kind)
              .collect::<Vec<_>>();

            return direct_children
              .get(index)
              .copied()
              .map_or_else(Vec::new, |node| vec![node]);
          }
        }
      }

      return (0..self.child_count())
        .filter_map(|i| child_at(self, i))
        .filter(|child| child.kind() == rest)
        .collect();
    }

    if selector.contains('[') && selector.ends_with(']') {
      let parts: Vec<&str> = selector.split('[').collect();

      if parts.len() == 2 {
        let (kind, index_str) = (parts[0], &parts[1][..parts[1].len() - 1]);

        if let Ok(index) = index_str.parse::<usize>() {
          let all_of_kind = self.find_all(kind);
          return all_of_kind
            .get(index)
            .copied()
            .map_or_else(Vec::new, |node| vec![node]);
        }
      }
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

    if selector.contains(' ') {
      let parts: Vec<&str> = selector.split_whitespace().collect();

      return parts.iter().skip(1).fold(
        self.find_all(parts[0]),
        |ancestors, &descendant_kind| {
          ancestors
            .iter()
            .flat_map(|&ancestor| {
              collect_descendants_by_kind(ancestor, descendant_kind)
            })
            .collect()
        },
      );
    }

    collect_nodes_by_kind(*self, selector)
  }

  fn find_siblings_until(&self, kind: &str, until: &str) -> Vec<Node<'_>> {
    let mut siblings = Vec::new();

    let mut current = self.next_sibling();

    while let Some(sibling) = current {
      if sibling.kind() == until {
        break;
      }

      if sibling.kind() == kind {
        siblings.push(sibling);
      }

      current = sibling.next_sibling();
    }

    siblings
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
  fn find_indexed_nodes() {
    let document = Document::from(indoc! {
      "
      foo:
        echo \"foo\"

      bar:
        echo \"bar\"

      baz:
        echo \"baz\"
      "
    });

    let root = document.tree.as_ref().unwrap().root_node();

    let selectors = ["recipe[0]", "recipe[1]", "recipe[2]"];

    let recipe_texts = selectors
      .iter()
      .map(|selector| {
        document
          .get_node_text(&root.find(selector).unwrap())
          .trim()
          .to_string()
      })
      .collect::<Vec<_>>();

    assert_eq!(
      recipe_texts,
      vec![
        "foo:\n  echo \"foo\"".to_string(),
        "bar:\n  echo \"bar\"".to_string(),
        "baz:\n  echo \"baz\"".to_string()
      ]
    );

    assert!(root.find("recipe[10]").is_none());
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

    let second_recipe = root.find("recipe[1]").unwrap();

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

    let recipe_identifier_texts = root
      .find_all("recipe identifier")
      .iter()
      .map(|node| document.get_node_text(node))
      .collect::<Vec<_>>();

    assert_eq!(recipe_identifier_texts, identifier_texts);

    let function_call_texts = root
      .find_all("recipe function_call")
      .iter()
      .map(|node| document.get_node_text(node).trim().to_string())
      .collect::<Vec<_>>();

    assert_eq!(function_call_texts, vec!["arch()".to_string()]);

    let function_identifier_texts = root
      .find_all("function_call identifier")
      .iter()
      .map(|node| document.get_node_text(node))
      .collect::<Vec<_>>();

    assert_eq!(function_identifier_texts, vec!["arch".to_string()]);
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

    let second_recipe = root.find("recipe[1]").unwrap();

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
  fn find_nonexistent() {
    let document = Document::from(indoc! {
      "
      foo:
        echo \"foo\"
      "
    });

    let tree = document.tree.as_ref().unwrap();
    let root = tree.root_node();

    let nonexistent = root.find("nonexistent_kind");
    assert!(nonexistent.is_none());

    let empty_results = root.find_all("nonexistent_kind");
    assert_eq!(empty_results.len(), 0);

    let no_function_calls = root.find_all("function_call");
    assert_eq!(no_function_calls.len(), 0);
  }

  #[test]
  fn find_nth_occurrence() {
    let document = Document::from(indoc! {
      "
      alias foo := bar
      "
    });

    let root = document.tree.as_ref().unwrap().root_node();

    let alias = root.find("alias").unwrap();

    let first_identifier = alias.find("identifier[0]").unwrap();
    let second_identifier = alias.find("identifier[1]").unwrap();

    assert_eq!(document.get_node_text(&first_identifier), "foo");
    assert_eq!(document.get_node_text(&second_identifier), "bar");
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
}
