use super::*;

#[derive(Debug)]
pub struct ProjectView<'a> {
  document: &'a Document,
  imported_documents: Vec<&'a Document>,
}

impl<'a> ProjectView<'a> {
  #[must_use]
  pub fn document(&self) -> &'a Document {
    self.document
  }

  fn documents(&self) -> impl Iterator<Item = &'a Document> + '_ {
    once(self.document).chain(self.imported_documents.iter().copied())
  }

  #[must_use]
  pub fn find_function(&self, name: &str) -> Option<Located<Function>> {
    self.documents().find_map(|document| {
      document.find_function(name).map(|value| Located {
        uri: document.uri.clone(),
        value,
      })
    })
  }

  #[must_use]
  pub fn find_recipe(&self, name: &str) -> Option<Located<Recipe>> {
    self.documents().find_map(|document| {
      document.find_recipe(name).map(|value| Located {
        uri: document.uri.clone(),
        value,
      })
    })
  }

  #[must_use]
  pub fn find_variable(&self, name: &str) -> Option<Located<Variable>> {
    self.documents().find_map(|document| {
      document.find_variable(name).map(|value| Located {
        uri: document.uri.clone(),
        value,
      })
    })
  }

  #[must_use]
  pub fn new(
    document: &'a Document,
    imported_documents: impl IntoIterator<Item = &'a Document>,
  ) -> Self {
    Self {
      document,
      imported_documents: imported_documents.into_iter().collect(),
    }
  }
}

impl<'a> From<&'a Document> for ProjectView<'a> {
  fn from(document: &'a Document) -> Self {
    Self::new(document, [])
  }
}

#[cfg(test)]
mod tests {
  use {super::*, indoc::indoc, pretty_assertions::assert_eq};

  #[test]
  fn finds_imported_declarations() {
    let root = Document::new(
      "import 'bar.just'",
      lsp::Url::parse("file:///foo.just").unwrap(),
    )
    .unwrap();

    let imported = Document::new(
      indoc! {
        "
        foo:

        bar := 'baz'

        qux() := 'quux'
        "
      },
      lsp::Url::parse("file:///bar.just").unwrap(),
    )
    .unwrap();

    let view = ProjectView::new(&root, [&imported]);

    assert_eq!(view.find_recipe("foo").unwrap().uri, imported.uri);
    assert_eq!(view.find_variable("bar").unwrap().uri, imported.uri);
    assert_eq!(view.find_function("qux").unwrap().uri, imported.uri);
  }

  #[test]
  fn prefers_current_document() {
    let root =
      Document::new("foo:", lsp::Url::parse("file:///foo.just").unwrap())
        .unwrap();

    let imported =
      Document::new("foo:", lsp::Url::parse("file:///bar.just").unwrap())
        .unwrap();

    assert_eq!(
      ProjectView::new(&root, [&imported])
        .find_recipe("foo")
        .unwrap()
        .uri,
      root.uri,
    );
  }
}
