use super::*;

pub trait NodeExt {
  fn find(&self, selector: &str) -> Option<Node<'_>>;
  fn find_all(&self, selector: &str) -> Vec<Node<'_>>;
  fn find_siblings_until(&self, kind: &str, until: &str) -> Vec<Node<'_>>;
  fn get_parent(&self, kind: &str) -> Option<Node<'_>>;
  fn get_range(&self) -> lsp::Range;
}

fn collect_nodes_by_kind<'a>(node: Node<'a>, kind: &str) -> Vec<Node<'a>> {
  let self_match = if node.kind() == kind {
    vec![node]
  } else {
    Vec::new()
  };

  let children_matches = (0..node.child_count())
    .filter_map(|i| node.child(i))
    .flat_map(|child| collect_nodes_by_kind(child, kind))
    .collect::<Vec<_>>();

  [self_match, children_matches].concat()
}

fn collect_descendants_by_kind<'a>(
  node: Node<'a>,
  kind: &str,
) -> Vec<Node<'a>> {
  (0..node.child_count())
    .filter_map(|i| node.child(i))
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
        .and_then(|position| self.child(position))
        .map_or_else(Vec::new, |child| vec![child]);
    }

    if let Some(rest) = selector.strip_prefix('^') {
      if rest.contains('[') && rest.ends_with(']') {
        let parts: Vec<&str> = rest.split('[').collect();

        if parts.len() == 2 {
          let (kind, index_str) = (parts[0], &parts[1][..parts[1].len() - 1]);

          if let Ok(index) = index_str.parse::<usize>() {
            let direct_children = (0..self.child_count())
              .filter_map(|i| self.child(i))
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
        .filter_map(|i| self.child(i))
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
                .filter_map(|i| parent.child(i))
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
  fn find_basic_kind() {
    let doc = document(indoc! {
      "
      foo:
        echo \"foo\"

      bar:
        echo \"bar\"
      "
    });

    let root = doc.tree.as_ref().unwrap().root_node();

    let recipes = root.find_all("recipe");

    assert_eq!(recipes.len(), 2);

    assert_eq!(
      doc.get_node_text(&recipes[0]).trim(),
      "foo:\n  echo \"foo\""
    );

    assert_eq!(
      doc.get_node_text(&recipes[1]).trim(),
      "bar:\n  echo \"bar\""
    );

    let first_recipe = root.find("recipe");

    assert!(first_recipe.is_some());

    assert_eq!(
      doc.get_node_text(&first_recipe.unwrap()).trim(),
      "foo:\n  echo \"foo\""
    );
  }

  #[test]
  fn find_indexed_nodes() {
    let doc = document(indoc! {
      "
      foo:
        echo \"foo\"

      bar:
        echo \"bar\"

      baz:
        echo \"baz\"
      "
    });

    let root = doc.tree.as_ref().unwrap().root_node();

    let first_recipe = root.find("recipe[0]");

    assert!(first_recipe.is_some());

    assert_eq!(
      doc.get_node_text(&first_recipe.unwrap()).trim(),
      "foo:\n  echo \"foo\""
    );

    let second_recipe = root.find("recipe[1]");

    assert!(second_recipe.is_some());

    assert_eq!(
      doc.get_node_text(&second_recipe.unwrap()).trim(),
      "bar:\n  echo \"bar\""
    );

    let third_recipe = root.find("recipe[2]");

    assert!(third_recipe.is_some());

    assert_eq!(
      doc.get_node_text(&third_recipe.unwrap()).trim(),
      "baz:\n  echo \"baz\""
    );

    let non_existent = root.find("recipe[10]");

    assert!(non_existent.is_none());
  }

  #[test]
  fn find_direct_child() {
    let doc = document(indoc! {
      "
      foo:
        echo \"foo\"

      bar arg1 arg2:
        echo \"bar\"
      "
    });

    let root = doc.tree.as_ref().unwrap().root_node();

    let recipe_header_identifiers = root.find_all("recipe_header > identifier");

    assert_eq!(recipe_header_identifiers.len(), 2);

    let second_recipe = root.find("recipe[1]").unwrap();

    let recipe_header = second_recipe.find("recipe_header");

    assert!(recipe_header.is_some());

    let recipe_header_node = recipe_header.unwrap();

    let parameters = recipe_header_node.find_all("parameters > parameter");

    assert_eq!(parameters.len(), 2);
  }

  #[test]
  fn find_descendant() {
    let doc = document(indoc! {
      "
      foo:
        echo \"foo\"

      bar arg1 arg2:
        echo \"{{ arch() }}\"
      "
    });

    let root = doc.tree.as_ref().unwrap().root_node();

    let all_identifiers = root.find_all("identifier");
    assert!(all_identifiers.len() >= 4);

    let recipe_identifiers = root.find_all("recipe identifier");
    assert!(recipe_identifiers.len() >= 4);

    let function_calls = root.find_all("recipe function_call");
    assert_eq!(function_calls.len(), 1);

    let function_identifiers = root.find_all("function_call identifier");
    assert_eq!(function_identifiers.len(), 1);
  }

  #[test]
  fn find_union() {
    let doc = document(indoc! {
      "
      foo := \"value\"

      foo:
        echo \"foo\"

      bar:
        echo \"bar\"
      "
    });

    let root = doc.tree.as_ref().unwrap().root_node();

    let recipes_and_assignments = root.find_all("recipe, assignment");

    assert_eq!(recipes_and_assignments.len(), 3);

    let kinds = recipes_and_assignments
      .iter()
      .map(Node::kind)
      .collect::<Vec<_>>();

    assert_eq!(kinds, ["recipe", "recipe", "assignment"]);

    let identifiers =
      root.find_all("recipe_header > identifier, function_call > identifier");

    assert_eq!(identifiers.len(), 2);
  }

  #[test]
  fn find_direct_child_marker() {
    let doc = document(indoc! {
      "
      foo:
        echo \"foo\"

      bar arg1 arg2:
        echo \"{{ arch() }}\"
      "
    });

    let root = doc.tree.as_ref().unwrap().root_node();

    let second_recipe = root.find("recipe[1]").unwrap();

    let recipe_header = second_recipe.find("recipe_header").unwrap();
    let parameters_node = recipe_header.find("parameters").unwrap();
    let direct_parameters = parameters_node.find_all("^parameter");
    assert_eq!(direct_parameters.len(), 2);

    assert_eq!(doc.get_node_text(&direct_parameters[0]), "arg1");
    assert_eq!(doc.get_node_text(&direct_parameters[1]), "arg2");
  }

  #[test]
  fn find_nonexistent() {
    let doc = document(indoc! {
      "
      foo:
        echo \"foo\"
      "
    });

    let tree = doc.tree.as_ref().unwrap();
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
    let doc = document(indoc! {
      "
      alias foo := bar
      "
    });

    let root = doc.tree.as_ref().unwrap().root_node();

    let alias = root.find("alias");
    assert!(alias.is_some());

    let alias = alias.unwrap();

    let first_identifier = alias.find("identifier[0]");
    assert!(first_identifier.is_some());

    let second_identifier = alias.find("identifier[1]");
    assert!(second_identifier.is_some());
  }

  #[test]
  fn find_nested_child() {
    let doc = document(indoc! {
      "
      foo: (bar baz):
        echo foo
      "
    });

    let root = doc.tree.as_ref().unwrap().root_node();

    let identifier =
      root.find("dependency_expression > expression > value > identifier");

    assert!(identifier.is_some());

    assert_eq!(doc.get_node_text(&identifier.unwrap()), "baz");
  }
}
