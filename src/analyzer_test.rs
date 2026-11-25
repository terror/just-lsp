use {super::*, pretty_assertions::assert_eq};

#[derive(Debug)]
pub(crate) struct AnalyzerTest {
  config: Config,
  files: Vec<(&'static str, String)>,
  messages: Vec<(String, lsp::Range, Option<lsp::DiagnosticSeverity>)>,
}

impl AnalyzerTest {
  pub(crate) fn config(self, config: Config) -> Self {
    Self { config, ..self }
  }

  pub(crate) fn error(mut self, message: &str, range: lsp::Range) -> Self {
    self.messages.push((
      message.to_owned(),
      range,
      Some(lsp::DiagnosticSeverity::ERROR),
    ));
    self
  }

  pub(crate) fn file(mut self, path: &'static str, content: &str) -> Self {
    self.files.push((path, content.to_owned()));
    self
  }

  pub(crate) fn new(content: &str) -> Self {
    Self {
      config: Config::default(),
      files: vec![("justfile", content.to_owned())],
      messages: Vec::new(),
    }
  }

  #[track_caller]
  pub(crate) fn run(self) {
    let module = TestModule::new(
      &self
        .files
        .iter()
        .map(|(path, content)| (*path, content.as_str()))
        .collect::<Vec<_>>(),
    );

    let analyzer = Analyzer {
      config: Some(&self.config),
      view: module.view(),
    };

    let diagnostics = analyzer
      .analyze()
      .into_iter()
      .map(lsp::Diagnostic::from)
      .collect::<Vec<_>>();

    assert_eq!(
      diagnostics
        .iter()
        .map(|diagnostic| (
          diagnostic.message.clone(),
          diagnostic.range,
          diagnostic.severity,
        ))
        .collect::<Vec<_>>(),
      self.messages
    );
  }

  pub(crate) fn warning(mut self, message: &str, range: lsp::Range) -> Self {
    self.messages.push((
      message.to_owned(),
      range,
      Some(lsp::DiagnosticSeverity::WARNING),
    ));
    self
  }
}
