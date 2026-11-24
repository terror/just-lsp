use super::*;

#[derive(Debug)]
struct DocumentDiagnostics {
  diagnostics: Vec<Diagnostic>,
  params: lsp::PublishDiagnosticsParams,
}

#[derive(Debug, Default)]
struct Published {
  documents: BTreeMap<lsp::Url, DocumentDiagnostics>,
  generation: u64,
}

#[derive(Clone, Debug)]
pub(crate) struct Publisher {
  client: Client,
  delay: Duration,
  generation: Arc<AtomicU64>,
  published: Arc<Mutex<Published>>,
  workspace: Arc<RwLock<Workspace>>,
}

impl Publisher {
  pub(crate) const DELAY: Duration = Duration::from_millis(100);

  async fn current(
    &self,
    generation: u64,
  ) -> Option<RwLockReadGuard<'_, Workspace>> {
    let workspace = self.workspace.read().await;

    (self.generation.load(Ordering::SeqCst) == generation).then_some(workspace)
  }

  async fn diagnostics(
    &self,
    generation: u64,
  ) -> Option<BTreeMap<lsp::Url, DocumentDiagnostics>> {
    let uris = self.current(generation).await?.diagnostic_documents();

    let mut documents = BTreeMap::new();

    for uri in uris {
      let workspace = self.current(generation).await?;

      let diagnostics = workspace.document_diagnostics(&uri);

      let params = lsp::PublishDiagnosticsParams {
        uri: uri.clone(),
        diagnostics: diagnostics
          .iter()
          .cloned()
          .map(lsp::Diagnostic::from)
          .collect(),
        version: workspace
          .open_document(&uri)
          .map(|document| document.version),
      };

      documents.insert(
        uri,
        DocumentDiagnostics {
          diagnostics,
          params,
        },
      );
    }

    Some(documents)
  }

  pub(crate) async fn document_diagnostics(
    &self,
    workspace: &Workspace,
    uri: &lsp::Url,
  ) -> Vec<Diagnostic> {
    let published = self.published.lock().await;

    if published.generation == self.generation.load(Ordering::SeqCst)
      && let Some(document) = published.documents.get(uri)
    {
      return document.diagnostics.clone();
    }

    workspace.document_diagnostics(uri)
  }

  pub(crate) fn invalidate(&self) -> u64 {
    self.generation.fetch_add(1, Ordering::SeqCst) + 1
  }

  pub(crate) fn new(
    client: Client,
    delay: Duration,
    workspace: Arc<RwLock<Workspace>>,
  ) -> Self {
    Self {
      client,
      delay,
      generation: Arc::new(AtomicU64::new(0)),
      published: Arc::new(Mutex::new(Published::default())),
      workspace,
    }
  }

  pub(crate) async fn publish(&self, generation: u64) {
    let Some(documents) = self.diagnostics(generation).await else {
      return;
    };

    let mut published = self.published.lock().await;

    if published.generation > generation {
      return;
    }

    for uri in published.documents.keys() {
      if !documents.contains_key(uri) {
        self
          .client
          .publish_diagnostics(uri.clone(), Vec::new(), None)
          .await;
      }
    }

    for (uri, document) in &documents {
      let previous = published
        .documents
        .get(uri)
        .map(|previous| &previous.params);

      if previous != Some(&document.params) {
        self
          .client
          .publish_diagnostics(
            uri.clone(),
            document.params.diagnostics.clone(),
            document.params.version,
          )
          .await;
      }
    }

    *published = Published {
      documents,
      generation,
    };
  }

  pub(crate) async fn schedule(&self, generation: u64) {
    if self.delay.is_zero() {
      self.publish(generation).await;
      return;
    }

    let publisher = self.clone();

    tokio::spawn(async move {
      sleep(publisher.delay).await;
      publisher.publish(generation).await;
    });
  }
}
