use super::*;

#[derive(Debug, Clone, PartialEq)]
pub struct Dependency {
  pub arguments: Vec<DependencyArgument>,
  pub mapped: Option<lsp::Range>,
  pub name: TextNode,
  pub phase: DependencyPhase,
  pub range: lsp::Range,
}

impl Dependency {
  pub(crate) fn from_node(
    node: &Node,
    document: &Document,
    phase: DependencyPhase,
  ) -> Option<Self> {
    let expression = node.find("dependency_expression");

    let name = node.child_by_field_name("name").or_else(|| {
      expression.and_then(|node| node.child_by_field_name("name"))
    })?;

    let arguments = expression.map_or_else(Vec::new, |node| {
      let mut cursor = node.walk();

      node
        .named_children(&mut cursor)
        .filter_map(|node| {
          let (node, starred) = match node.kind() {
            "expression" => (node, None),
            "starred_dependency_argument" => (
              node.child_by_field_name("argument")?,
              node
                .child_by_field_name("star")
                .map(|node| document.get_range(&node)),
            ),
            _ => return None,
          };

          Some(DependencyArgument {
            range: document.get_range(&node),
            starred,
            value: document.get_node_text(&node),
          })
        })
        .collect()
    });

    let mapped = expression
      .and_then(|node| node.child_by_field_name("map"))
      .map(|node| document.get_range(&node));

    Some(Self {
      arguments,
      mapped,
      name: TextNode {
        range: document.get_range(&name),
        value: document.get_node_text(&name),
      },
      phase,
      range: document.get_range(node),
    })
  }
}
