use crate::common::*;

#[derive(Debug)]
pub(crate) struct Document {
  content: Rope,
  language: String,
  version: i32,
}

impl Document {
  /// Construct a new `Document` from a `textDocument/didOpen` notification's
  /// parameters.
  pub(crate) fn from_params(params: lsp::DidOpenTextDocumentParams) -> Self {
    let document = params.text_document;
    Self {
      content: Rope::from_str(&document.text),
      language: document.language_id,
      version: document.version,
    }
  }

  /// Apply a `textDocument/didChange` notification sent from the client.
  pub(crate) fn apply_change(
    &mut self,
    params: lsp::DidChangeTextDocumentParams,
  ) {
    params
      .content_changes
      .iter()
      .map(|change| self.content.build_edit(change))
      .collect::<Result<Vec<_>, _>>()
      .unwrap()
      .iter()
      .for_each(|edit| self.content.apply_edit(edit));
  }
}
