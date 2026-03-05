use super::*;

#[derive(Debug, Clap)]
pub(crate) struct Analyze {
  #[arg(
    value_name = "PATH",
    help = "Path to the justfile to analyze",
    value_hint = clap::ValueHint::FilePath
  )]
  path: Option<PathBuf>,
}

impl Analyze {
  pub(crate) fn run(self) -> Result<()> {
    let path = match self.path {
      Some(path) => path,
      None => Subcommand::find_justfile()?,
    };

    let content = fs::read_to_string(&path)?;

    let absolute_path = if path.is_absolute() {
      path.clone()
    } else {
      env::current_dir()?.join(&path)
    };

    let uri = lsp::Url::from_file_path(&absolute_path).map_err(|()| {
      anyhow!("failed to convert `{}` to file url", path.display())
    })?;

    let document = Document::try_from(lsp::DidOpenTextDocumentParams {
      text_document: lsp::TextDocumentItem {
        language_id: "just".to_string(),
        text: content.clone(),
        uri,
        version: 1,
      },
    })?;

    let analyzer = Analyzer::new(&document);

    let diagnostics = analyzer.analyze();

    if diagnostics.is_empty() {
      return Ok(());
    }

    let any_error = diagnostics.iter().any(|diagnostic| {
      matches!(diagnostic.severity, lsp::DiagnosticSeverity::ERROR)
    });

    let source_id = path.to_string_lossy().to_string();

    let mut cache = sources(vec![(source_id.clone(), content.as_str())]);

    let source_len = document.content.len_chars();

    for diagnostic in diagnostics {
      let (severity_label, color) =
        Self::severity_to_style(diagnostic.severity)?;

      let kind_label = format!("{severity_label}[{}]", diagnostic.id.trim());

      let start = document
        .content
        .lsp_position_to_position(diagnostic.range.start)
        .char
        .min(source_len);

      let end = document
        .content
        .lsp_position_to_position(diagnostic.range.end)
        .char
        .min(source_len);

      let (start, end) = (start.min(end), start.max(end));

      let span = (source_id.clone(), start..end);

      let report = Report::build(
        ReportKind::Custom(kind_label.as_str(), color),
        span.clone(),
      )
      .with_message(&diagnostic.display)
      .with_label(
        Label::new(span.clone())
          .with_message(diagnostic.message.trim().to_string())
          .with_color(color),
      );

      let report = report.finish();

      report
        .print(&mut cache)
        .map_err(|error| anyhow!("failed to render diagnostic: {error}"))?;
    }

    if any_error {
      process::exit(1);
    }

    Ok(())
  }

  fn severity_to_style(
    severity: lsp::DiagnosticSeverity,
  ) -> Result<(&'static str, Color)> {
    match severity {
      lsp::DiagnosticSeverity::ERROR => Ok(("error", Color::Red)),
      lsp::DiagnosticSeverity::WARNING => Ok(("warning", Color::Yellow)),
      lsp::DiagnosticSeverity::INFORMATION => Ok(("info", Color::Blue)),
      lsp::DiagnosticSeverity::HINT => Ok(("hint", Color::Cyan)),
      _ => bail!("failed to map unknown severity {severity:?}"),
    }
  }
}
