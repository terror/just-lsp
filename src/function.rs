use super::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Function {
  pub attributes: Vec<Attribute>,
  pub body: String,
  pub content: String,
  pub name: TextNode,
  pub parameters: Vec<TextNode>,
  pub range: lsp::Range,
}

impl Function {
  pub(crate) fn completion_item(&self) -> lsp::CompletionItem {
    let parameters = self
      .parameters
      .iter()
      .map(|parameter| parameter.value.as_str())
      .collect::<Vec<_>>()
      .join(", ");

    lsp::CompletionItem {
      label: format!("{}({})", self.name.value, parameters),
      kind: Some(lsp::CompletionItemKind::FUNCTION),
      documentation: Some(lsp::Documentation::MarkupContent(
        lsp::MarkupContent {
          kind: lsp::MarkupKind::PlainText,
          value: self.content.clone(),
        },
      )),
      filter_text: Some(self.name.value.clone()),
      insert_text: Some(self.name.value.clone()),
      insert_text_format: Some(lsp::InsertTextFormat::PLAIN_TEXT),
      ..Default::default()
    }
  }
}
