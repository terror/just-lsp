use super::*;

pub(crate) fn server_capabilities() -> ServerCapabilities {
  ServerCapabilities {
    text_document_sync: Some(TextDocumentSyncCapability::Options(
      TextDocumentSyncOptions {
        open_close: Some(true),
        change: Some(TextDocumentSyncKind::INCREMENTAL),
        will_save: None,
        will_save_wait_until: None,
        save: Some(SaveOptions::default().into()),
      },
    )),
    hover_provider: Some(HoverProviderCapability::Simple(true)),
    completion_provider: None,
    signature_help_provider: None,
    declaration_provider: None,
    definition_provider: None,
    type_definition_provider: None,
    implementation_provider: None,
    references_provider: None,
    document_highlight_provider: None,
    document_symbol_provider: None,
    workspace_symbol_provider: None,
    code_action_provider: None,
    code_lens_provider: None,
    document_formatting_provider: None,
    document_range_formatting_provider: None,
    document_on_type_formatting_provider: None,
    selection_range_provider: None,
    folding_range_provider: None,
    rename_provider: None,
    document_link_provider: None,
    color_provider: None,
    execute_command_provider: None,
    call_hierarchy_provider: None,
    semantic_tokens_provider: None,
    workspace: None,
    experimental: None,
    linked_editing_range_provider: None,
    moniker_provider: None,
  }
}
