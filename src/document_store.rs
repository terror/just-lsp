use super::*;

#[derive(Debug, Default)]
pub struct DocumentStore {
  documents: HashMap<lsp::Url, DocumentEntry>,
}

impl DocumentStore {
  /// # Errors
  ///
  /// Returns an [`Error`] if the changed document cannot be parsed.
  pub fn change(&mut self, params: lsp::DidChangeTextDocumentParams) -> Result {
    let Some(entry) = self.documents.get_mut(&params.text_document.uri) else {
      return Ok(());
    };

    if !entry.open {
      return Ok(());
    }

    entry.document.apply_change(params)?;

    Ok(())
  }

  pub fn close(&mut self, params: &lsp::DidCloseTextDocumentParams) -> bool {
    let Some(entry) = self.documents.get_mut(&params.text_document.uri) else {
      return false;
    };

    let open = entry.open;

    entry.open = false;

    open
  }

  #[must_use]
  pub fn get(&self, uri: &lsp::Url) -> Option<&Document> {
    self.documents.get(uri).map(|entry| &entry.document)
  }

  #[must_use]
  pub fn get_open(&self, uri: &lsp::Url) -> Option<&Document> {
    self
      .documents
      .get(uri)
      .filter(|entry| entry.open)
      .map(|entry| &entry.document)
  }

  #[must_use]
  pub fn is_open(&self, uri: &lsp::Url) -> bool {
    self.documents.get(uri).is_some_and(|entry| entry.open)
  }

  /// # Errors
  ///
  /// Returns an [`Error`] if the URI is not a file URI, the file cannot be
  /// read, or the document cannot be parsed.
  pub fn load(&mut self, uri: &lsp::Url) -> Result<&Document> {
    match self.documents.entry(uri.clone()) {
      Entry::Occupied(entry) => Ok(&entry.into_mut().document),
      Entry::Vacant(entry) => {
        let path = uri
          .to_file_path()
          .map_err(|()| Error::InvalidDocumentUri(uri.clone()))?;

        Ok(
          &entry
            .insert(DocumentEntry {
              document: Document::new(&fs::read_to_string(path)?, uri.clone())?,
              open: false,
            })
            .document,
        )
      }
    }
  }

  /// # Errors
  ///
  /// Returns an [`Error`] if the opened document cannot be parsed.
  pub fn open(&mut self, params: lsp::DidOpenTextDocumentParams) -> Result {
    let document = Document::try_from(params)?;

    self.documents.insert(
      document.uri.clone(),
      DocumentEntry {
        document,
        open: true,
      },
    );

    Ok(())
  }
}

#[cfg(test)]
mod tests {
  use {super::*, pretty_assertions::assert_eq};

  fn change(
    uri: lsp::Url,
    version: i32,
    text: &str,
  ) -> lsp::DidChangeTextDocumentParams {
    lsp::DidChangeTextDocumentParams {
      text_document: lsp::VersionedTextDocumentIdentifier { uri, version },
      content_changes: vec![lsp::TextDocumentContentChangeEvent {
        range: None,
        range_length: None,
        text: text.into(),
      }],
    }
  }

  fn close(uri: lsp::Url) -> lsp::DidCloseTextDocumentParams {
    lsp::DidCloseTextDocumentParams {
      text_document: lsp::TextDocumentIdentifier { uri },
    }
  }

  fn open(
    uri: lsp::Url,
    version: i32,
    text: &str,
  ) -> lsp::DidOpenTextDocumentParams {
    lsp::DidOpenTextDocumentParams {
      text_document: lsp::TextDocumentItem {
        uri,
        language_id: "just".into(),
        version,
        text: text.into(),
      },
    }
  }

  fn uri(path: &Path) -> lsp::Url {
    lsp::Url::from_file_path(path).unwrap()
  }

  #[test]
  fn change_updates_open_document() {
    let uri = lsp::Url::parse("file:///foo.just").unwrap();

    let mut store = DocumentStore::default();

    store.open(open(uri.clone(), 1, "foo:")).unwrap();
    store.change(change(uri.clone(), 2, "bar:")).unwrap();

    let document = store.get_open(&uri).unwrap();

    assert_eq!(document.content.to_string(), "bar:");
    assert_eq!(document.version, 2);
  }

  #[test]
  fn close_keeps_document_cached() {
    let uri = lsp::Url::parse("file:///foo.just").unwrap();

    let mut store = DocumentStore::default();

    store.open(open(uri.clone(), 1, "foo:")).unwrap();

    assert!(store.close(&close(uri.clone())));
    assert!(!store.is_open(&uri));
    assert!(store.get(&uri).is_some());
    assert!(store.get_open(&uri).is_none());
  }

  #[test]
  fn load_caches_disk_document() {
    let tempdir = tempfile::tempdir().unwrap();

    let path = tempdir.path().join("foo.just");
    let uri = uri(&path);

    fs::write(&path, "foo:").unwrap();

    let mut store = DocumentStore::default();

    assert_eq!(store.load(&uri).unwrap().content.to_string(), "foo:");

    fs::write(path, "bar:").unwrap();

    assert_eq!(store.load(&uri).unwrap().content.to_string(), "foo:");

    assert!(!store.is_open(&uri));
  }

  #[test]
  fn load_prefers_open_document() {
    let tempdir = tempfile::tempdir().unwrap();

    let path = tempdir.path().join("foo.just");
    let uri = uri(&path);

    fs::write(path, "foo:").unwrap();

    let mut store = DocumentStore::default();

    store.load(&uri).unwrap();
    store.open(open(uri.clone(), 1, "bar:")).unwrap();

    assert_eq!(store.load(&uri).unwrap().content.to_string(), "bar:");

    assert!(store.is_open(&uri));
  }
}
