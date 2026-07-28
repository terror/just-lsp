use super::*;

#[derive(Debug)]
pub struct ProjectView<'a> {
  document: &'a Document,
  documents: Vec<ProjectViewDocument<'a>>,
}

impl<'a> ProjectView<'a> {
  #[must_use]
  pub fn document(&self) -> &'a Document {
    self.document
  }

  fn find<T>(
    &self,
    name: &str,
    declarations: impl Fn(&Document) -> Vec<T>,
    declaration_name: impl Fn(&T) -> &str,
    declaration_position: impl Fn(&T) -> lsp::Position,
  ) -> Option<Located<T>> {
    let mut candidates = Vec::new();

    for document in &self.documents {
      for declaration in declarations(document.document) {
        if declaration_name(&declaration) == name {
          candidates.push((document, declaration));
        }
      }
    }

    candidates
      .into_iter()
      .max_by_key(|(document, declaration)| {
        (
          Reverse(document.load_depth),
          document.traversal_order,
          declaration_position(declaration),
        )
      })
      .map(|(document, value)| Located {
        uri: document.document.uri.clone(),
        value,
      })
  }

  #[must_use]
  pub fn find_function(&self, name: &str) -> Option<Located<Function>> {
    self.find(
      name,
      Document::functions,
      |function| &function.name.value,
      |function| function.range.start,
    )
  }

  #[must_use]
  pub fn find_recipe(&self, name: &str) -> Option<Located<Recipe>> {
    self.find(
      name,
      Document::recipes,
      |recipe| &recipe.name.value,
      |recipe| recipe.range.start,
    )
  }

  #[must_use]
  pub fn find_variable(&self, name: &str) -> Option<Located<Variable>> {
    self.find(
      name,
      Document::variables,
      |variable| &variable.name.value,
      |variable| variable.range.start,
    )
  }

  #[must_use]
  pub fn new(
    document: &'a Document,
    import_scope: &'a ImportScope,
    documents: &'a DocumentStore,
  ) -> Self {
    let documents = import_scope
      .documents()
      .iter()
      .filter_map(|scope_document| {
        let scoped_document = if scope_document.uri == document.uri {
          document
        } else {
          documents.get(&scope_document.uri)?
        };

        Some(ProjectViewDocument {
          document: scoped_document,
          load_depth: scope_document.load_depth,
          traversal_order: scope_document.traversal_order,
        })
      })
      .collect();

    Self {
      document,
      documents,
    }
  }
}

impl<'a> From<&'a Document> for ProjectView<'a> {
  fn from(document: &'a Document) -> Self {
    Self {
      document,
      documents: vec![ProjectViewDocument {
        document,
        load_depth: 0,
        traversal_order: 0,
      }],
    }
  }
}

#[cfg(test)]
mod tests {
  use {super::*, indoc::indoc, pretty_assertions::assert_eq};

  #[test]
  fn direct_import_overrides_nested_import() {
    let root =
      Document::new("", lsp::Url::parse("file:///justfile").unwrap()).unwrap();

    let direct = Document::new(
      indoc! {"
        foo := 'foo'
        foo() := 'foo'
        foo:
          echo foo
      "},
      lsp::Url::parse("file:///foo.just").unwrap(),
    )
    .unwrap();

    let nested = Document::new(
      indoc! {"
        foo := 'bar'
        foo() := 'bar'
        foo:
          echo bar
      "},
      lsp::Url::parse("file:///bar.just").unwrap(),
    )
    .unwrap();

    let view = ProjectView {
      document: &root,
      documents: vec![
        ProjectViewDocument {
          document: &root,
          load_depth: 0,
          traversal_order: 0,
        },
        ProjectViewDocument {
          document: &direct,
          load_depth: 1,
          traversal_order: 1,
        },
        ProjectViewDocument {
          document: &nested,
          load_depth: 2,
          traversal_order: 2,
        },
      ],
    };

    assert_eq!(view.find_recipe("foo").unwrap().uri, direct.uri);
    assert_eq!(view.find_variable("foo").unwrap().uri, direct.uri);
    assert_eq!(view.find_function("foo").unwrap().uri, direct.uri);
  }

  #[test]
  fn equal_depth_imports_use_lifo_precedence() {
    let root =
      Document::new("", lsp::Url::parse("file:///justfile").unwrap()).unwrap();

    let first = Document::new(
      indoc! {"
        foo := 'foo'
        foo() := 'foo'
        foo:
          echo foo
      "},
      lsp::Url::parse("file:///foo.just").unwrap(),
    )
    .unwrap();

    let second = Document::new(
      indoc! {"
        foo := 'bar'
        foo() := 'bar'
        foo:
          echo bar
      "},
      lsp::Url::parse("file:///bar.just").unwrap(),
    )
    .unwrap();

    let view = ProjectView {
      document: &root,
      documents: vec![
        ProjectViewDocument {
          document: &root,
          load_depth: 0,
          traversal_order: 0,
        },
        ProjectViewDocument {
          document: &second,
          load_depth: 1,
          traversal_order: 1,
        },
        ProjectViewDocument {
          document: &first,
          load_depth: 1,
          traversal_order: 2,
        },
      ],
    };

    assert_eq!(view.find_recipe("foo").unwrap().uri, first.uri);
    assert_eq!(view.find_variable("foo").unwrap().uri, first.uri);
    assert_eq!(view.find_function("foo").unwrap().uri, first.uri);
  }

  #[test]
  fn later_declaration_in_document_wins() {
    let root = Document::new(
      indoc! {"
        foo := 'foo'
        foo() := 'foo'
        foo:
          echo foo

        foo := 'bar'
        foo() := 'bar'
        foo:
          echo bar
      "},
      lsp::Url::parse("file:///justfile").unwrap(),
    )
    .unwrap();

    let view = ProjectView::from(&root);

    assert_eq!(
      view.find_recipe("foo").unwrap().value.content,
      "foo:\n  echo bar",
    );

    assert_eq!(
      view.find_variable("foo").unwrap().value.content,
      "foo := 'bar'",
    );

    assert_eq!(
      view.find_function("foo").unwrap().value.content,
      "foo() := 'bar'",
    );
  }

  #[test]
  fn root_overrides_imported_declarations() {
    let root = Document::new(
      indoc! {"
        foo := 'foo'
        foo() := 'foo'
        foo:
          echo foo
      "},
      lsp::Url::parse("file:///justfile").unwrap(),
    )
    .unwrap();

    let imported = Document::new(
      indoc! {"
        foo := 'bar'
        foo() := 'bar'
        foo:
          echo bar
      "},
      lsp::Url::parse("file:///foo.just").unwrap(),
    )
    .unwrap();

    let view = ProjectView {
      document: &root,
      documents: vec![
        ProjectViewDocument {
          document: &root,
          load_depth: 0,
          traversal_order: 0,
        },
        ProjectViewDocument {
          document: &imported,
          load_depth: 1,
          traversal_order: 1,
        },
      ],
    };

    assert_eq!(view.find_recipe("foo").unwrap().uri, root.uri);
    assert_eq!(view.find_variable("foo").unwrap().uri, root.uri);
    assert_eq!(view.find_function("foo").unwrap().uri, root.uri);
  }
}
