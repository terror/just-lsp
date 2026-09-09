use super::*;

#[derive(Debug, Default)]
pub struct Workspace {
  pub documents: DocumentStore,
  pub projects: HashMap<lsp::Url, Project>,
}

impl Workspace {
  #[must_use]
  pub fn affected_roots(&self, uri: &lsp::Url) -> HashSet<lsp::Url> {
    self
      .projects
      .iter()
      .filter(|(_, project)| project.contains(uri))
      .map(|(root, _)| root.clone())
      .collect()
  }

  fn analyze_document(
    &self,
    document: &Document,
    config: Option<&Config>,
    projects: &[&Project],
  ) -> Vec<Diagnostic> {
    let analyses = projects
      .iter()
      .filter(|project| project.import_scope.contains(&document.uri))
      .map(|project| {
        Analyzer {
          config,
          view: ProjectView::new(
            document,
            &project.import_scope,
            &self.documents,
          ),
        }
        .analyze()
      })
      .collect::<Vec<_>>();

    let mut quickfixes = analyses
      .first()
      .into_iter()
      .flatten()
      .flat_map(|diagnostic| diagnostic.quickfixes.iter().cloned())
      .collect::<Vec<_>>();

    for diagnostics in analyses.iter().skip(1) {
      quickfixes.retain(|quickfix| {
        diagnostics
          .iter()
          .any(|diagnostic| diagnostic.quickfixes.contains(quickfix))
      });
    }

    let mut diagnostics = Vec::<Diagnostic>::new();

    for mut diagnostic in analyses.into_iter().flatten() {
      diagnostic
        .quickfixes
        .retain(|quickfix| quickfixes.contains(quickfix));

      if let Some(previous) = diagnostics.iter_mut().find(|previous| {
        previous.id == diagnostic.id
          && previous.message == diagnostic.message
          && previous.range == diagnostic.range
          && previous.severity == diagnostic.severity
      }) {
        for quickfix in diagnostic.quickfixes {
          if !previous.quickfixes.contains(&quickfix) {
            previous.quickfixes.push(quickfix);
          }
        }
      } else {
        diagnostics.push(diagnostic);
      }
    }

    diagnostics.sort_by(|left, right| {
      left
        .range
        .start
        .cmp(&right.range.start)
        .then_with(|| left.message.cmp(&right.message))
    });

    diagnostics
  }

  #[must_use]
  pub fn diagnostics(
    &self,
    config: Option<&Config>,
  ) -> BTreeMap<lsp::Url, Vec<Diagnostic>> {
    let projects = self.root_projects();

    projects
      .iter()
      .flat_map(|project| project.import_scope.documents())
      .map(|document| &document.uri)
      .collect::<BTreeSet<_>>()
      .into_iter()
      .filter_map(|uri| self.documents.get(uri))
      .map(|document| {
        (
          document.uri.clone(),
          self.analyze_document(document, config, &projects),
        )
      })
      .collect()
  }

  #[must_use]
  pub fn document_diagnostics(
    &self,
    uri: &lsp::Url,
    config: Option<&Config>,
  ) -> Vec<Diagnostic> {
    self.documents.get(uri).map_or_else(Vec::new, |document| {
      self.analyze_document(document, config, &self.root_projects())
    })
  }

  /// # Errors
  ///
  /// Returns an [`Error`] if the project root cannot be loaded.
  pub fn load_project(&mut self, root: lsp::Url) -> Result {
    let project = ProjectLoader::load(&mut self.documents, &root)?;

    self.projects.insert(root, project);

    Ok(())
  }

  /// # Errors
  ///
  /// Returns an [`Error`] if a project root cannot be loaded.
  pub fn load_projects(
    &mut self,
    roots: impl IntoIterator<Item = lsp::Url>,
  ) -> Result {
    for root in roots {
      self.load_project(root)?;
    }

    self.projects.retain(|_, project| {
      project
        .import_scope
        .documents()
        .iter()
        .any(|document| self.documents.is_open(&document.uri))
    });

    Ok(())
  }

  #[must_use]
  pub fn project_view(&self, uri: &lsp::Url) -> Option<ProjectView<'_>> {
    let document = self.documents.get_open(uri)?;

    let project = self
      .root_projects()
      .into_iter()
      .find(|project| project.import_scope.contains(uri));

    Some(project.map_or_else(
      || ProjectView::from(document),
      |project| {
        ProjectView::new(document, &project.import_scope, &self.documents)
      },
    ))
  }

  fn root_projects(&self) -> Vec<&Project> {
    let mut projects = self
      .projects
      .values()
      .filter(|project| {
        !self.projects.values().any(|other| {
          other.root != project.root
            && other.import_scope.contains(&project.root)
            && (!project.import_scope.contains(&other.root)
              || other.root < project.root)
        })
      })
      .collect::<Vec<_>>();

    projects.sort_by_key(|project| &project.root);

    projects
  }
}
