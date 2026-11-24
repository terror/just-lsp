use super::*;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Variable {
  pub attributes: Vec<Attribute>,
  pub content: String,
  pub eager: bool,
  pub export: bool,
  pub name: TextNode,
  pub range: lsp::Range,
}

impl Variable {
  pub(crate) fn completion_item(&self) -> lsp::CompletionItem {
    lsp::CompletionItem {
      label: self.name.value.clone(),
      kind: Some(lsp::CompletionItemKind::VARIABLE),
      documentation: Some(lsp::Documentation::MarkupContent(
        lsp::MarkupContent {
          kind: lsp::MarkupKind::PlainText,
          value: self.content.clone(),
        },
      )),
      insert_text: Some(self.name.value.clone()),
      insert_text_format: Some(lsp::InsertTextFormat::PLAIN_TEXT),
      ..Default::default()
    }
  }
}
