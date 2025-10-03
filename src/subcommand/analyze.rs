use super::*;

#[derive(Debug, Clap)]
pub(crate) struct Analyze {
  #[arg(value_name = "PATH")]
  path: PathBuf,
}

impl Analyze {
  pub(crate) fn run(self) -> Result<()> {
    let contents = fs::read_to_string(&self.path)?;

    let absolute_path = if self.path.is_absolute() {
      self.path.clone()
    } else {
      env::current_dir()?.join(&self.path)
    };

    let uri = lsp::Url::from_file_path(&absolute_path).map_err(|_| {
      anyhow!("failed to convert `{}` to file url", self.path.display())
    })?;

    let document = Document::try_from(lsp::DidOpenTextDocumentParams {
      text_document: lsp::TextDocumentItem {
        uri,
        language_id: "just".to_string(),
        version: 1,
        text: contents.clone(),
      },
    })?;

    let analyzer = Analyzer::new(&document);

    let mut diagnostics = analyzer.analyze();

    if diagnostics.is_empty() {
      return Ok(());
    }

    diagnostics.sort_by_key(|diagnostic| {
      (
        diagnostic.range.start.line,
        diagnostic.range.start.character,
        diagnostic.range.end.line,
        diagnostic.range.end.character,
      )
    });

    let any_error = diagnostics.iter().any(|diagnostic| {
      matches!(diagnostic.severity, Some(lsp::DiagnosticSeverity::ERROR))
    });

    let source_id = self.path.to_string_lossy().to_string();

    let mut cache = sources(vec![(source_id.clone(), contents.as_str())]);

    let source_len = document.content.len_chars();

    for diagnostic in diagnostics {
      let message = diagnostic.message.trim().to_string();

      let severity = diagnostic
        .severity
        .ok_or_else(|| anyhow!("diagnostic missing severity"))?;

      let (kind, color) = Self::severity_to_style(severity)?;

      let start = document
        .content
        .lsp_position_to_core(diagnostic.range.start)
        .char
        .min(source_len);

      let end = document
        .content
        .lsp_position_to_core(diagnostic.range.end)
        .char
        .min(source_len);

      let (start, end) = (start.min(end), start.max(end));

      let span = (source_id.clone(), start..end);

      let report = Report::build(kind, span.clone())
        .with_message(&message)
        .with_label(
          Label::new(span.clone())
            .with_message(&message)
            .with_color(color),
        );

      let report = match diagnostic.code.as_ref() {
        Some(lsp::NumberOrString::Number(n)) => report.with_code(n.to_string()),
        Some(lsp::NumberOrString::String(s)) => report.with_code(s.clone()),
        None => report,
      }
      .finish();

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
  ) -> Result<(ReportKind<'static>, Color)> {
    match severity {
      lsp::DiagnosticSeverity::ERROR => {
        Ok((ReportKind::Custom("error", Color::Red), Color::Red))
      }
      lsp::DiagnosticSeverity::WARNING => {
        Ok((ReportKind::Custom("warning", Color::Yellow), Color::Yellow))
      }
      lsp::DiagnosticSeverity::INFORMATION => {
        Ok((ReportKind::Custom("info", Color::Blue), Color::Blue))
      }
      lsp::DiagnosticSeverity::HINT => {
        Ok((ReportKind::Custom("hint", Color::Cyan), Color::Cyan))
      }
      _ => bail!("failed to map unknown severity {severity:?}"),
    }
  }
}
