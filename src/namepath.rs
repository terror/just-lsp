use super::*;

#[derive(Clone, Debug, PartialEq)]
pub struct Namepath {
  pub range: lsp::Range,
  pub segments: Vec<TextNode>,
}

impl Namepath {
  pub(crate) fn from_node(node: &Node, document: &Document) -> Self {
    let segments = if node.kind() == "module_path" {
      node.find_all("^identifier")
    } else {
      Vec::new()
    };

    let segments = if segments.is_empty() {
      vec![*node]
    } else {
      segments
    };

    Self {
      range: document.get_range(node),
      segments: segments
        .iter()
        .map(|segment| TextNode {
          range: document.get_range(segment),
          value: document.get_node_text(segment),
        })
        .collect(),
    }
  }

  pub(crate) fn modules(&self) -> &[TextNode] {
    &self.segments[..self.segments.len() - 1]
  }

  pub(crate) fn name(&self) -> &TextNode {
    &self.segments[self.segments.len() - 1]
  }
}

impl Display for Namepath {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    for (index, segment) in self.segments.iter().enumerate() {
      if index > 0 {
        write!(f, "::")?;
      }

      write!(f, "{}", segment.value)?;
    }

    Ok(())
  }
}

impl From<TextNode> for Namepath {
  fn from(segment: TextNode) -> Self {
    Self {
      range: segment.range,
      segments: vec![segment],
    }
  }
}
