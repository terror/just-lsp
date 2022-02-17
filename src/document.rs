use crate::common::*;

#[derive(Debug)]
pub struct Document {
  content: Rope,
  _language: String,
  _version: i32,
}

impl Document {
  /// Construct a new `Document` from a `textDocument/didOpen` notification's
  /// parameters.
  pub(crate) fn from_params(params: lsp::DidOpenTextDocumentParams) -> Self {
    let document = params.text_document;
    Self {
      content: Rope::from_str(&document.text),
      _language: document.language_id,
      _version: document.version,
    }
  }

  /// Apply a `textDocument/didChange` notification sent from the client.
  pub(crate) fn apply_change(
    &mut self,
    params: lsp::DidChangeTextDocumentParams,
  ) -> Result {
    params
      .content_changes
      .iter()
      .map(|change| self.content.build_edit(change))
      .collect::<Result<Vec<_>, _>>()?
      .iter()
      .for_each(|edit| self.content.apply_edit(edit));
    Ok(())
  }
}
