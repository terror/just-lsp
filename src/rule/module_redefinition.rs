use super::*;

#[derive(Clone, Copy, PartialEq)]
enum DefinitionKind {
  Alias,
  Module,
  Recipe,
}

impl DefinitionKind {
  fn indefinite(self) -> &'static str {
    match self {
      Self::Alias => "an alias",
      Self::Module => "a module",
      Self::Recipe => "a recipe",
    }
  }

  fn title(self) -> &'static str {
    match self {
      Self::Alias => "Alias",
      Self::Module => "Module",
      Self::Recipe => "Recipe",
    }
  }
}

struct Definition<'a> {
  groups: GroupSet,
  kind: DefinitionKind,
  name: &'a TextNode,
  uri: &'a lsp::Url,
}

impl<'a> Definition<'a> {
  fn conflicts_with(&self, other: &Self) -> bool {
    self.name.value == other.name.value
      && (self.kind == DefinitionKind::Module
        || other.kind == DefinitionKind::Module)
      && self.groups.conflicts_with(&other.groups)
  }

  fn new(
    kind: DefinitionKind,
    name: &'a TextNode,
    attributes: &[Attribute],
    uri: &'a lsp::Url,
  ) -> Self {
    Self {
      groups: GroupSet::from_attributes(attributes),
      kind,
      name,
      uri,
    }
  }

  fn redefinition_message(&self, redefinition: &Self) -> String {
    if self.kind == redefinition.kind {
      format!("Duplicate module `{}`", self.name.value)
    } else {
      format!(
        "{} `{}` is redefined as {}",
        self.kind.title(),
        self.name.value,
        redefinition.kind.indefinite(),
      )
    }
  }
}

define_rule! {
  ModuleRedefinitionRule {
    id: "module-redefinition",
    title: "module redefinition",
    severity: Error,
  }
}

impl ModuleRedefinitionRule {
  fn run(context: &RuleContext<'_>) -> Vec<Diagnostic> {
    let filesystem = context.view().filesystem();

    let modules = context
      .view()
      .modules()
      .iter()
      .filter(|module| module.resolve(&module.uri, filesystem).is_some())
      .map(|module| {
        Definition::new(
          DefinitionKind::Module,
          &module.name,
          &module.attributes,
          &module.uri,
        )
      })
      .collect::<Vec<_>>();

    if modules.is_empty() {
      return Vec::new();
    }

    let aliases = context.aliases().iter().map(|alias| {
      Definition::new(
        DefinitionKind::Alias,
        &alias.name,
        &alias.attributes,
        &alias.uri,
      )
    });

    let recipes = context.recipes().iter().map(|recipe| {
      Definition::new(
        DefinitionKind::Recipe,
        &recipe.name,
        &recipe.attributes,
        &recipe.uri,
      )
    });

    let (mut local, imported) = modules
      .into_iter()
      .chain(aliases)
      .chain(recipes)
      .partition::<Vec<_>, _>(|definition| {
        definition.uri == &context.document().uri
      });

    local.sort_by_key(|definition| definition.name.range.start);

    local
      .iter()
      .enumerate()
      .filter_map(|(index, definition)| {
        let first = imported
          .iter()
          .chain(&local[..index])
          .find(|previous| previous.conflicts_with(definition))?;

        Some(Diagnostic::new(
          first.redefinition_message(definition),
          definition.name.range,
        ))
      })
      .collect()
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn module_redefinition() {
    AnalyzerTest::new("mod foo\nmod foo\n")
      .file("foo.just", "")
      .error("Duplicate module `foo`", lsp::Range::at(1, 4, 1, 7))
      .run();

    AnalyzerTest::new("foo:\nmod foo\n")
      .file("foo.just", "")
      .error(
        "Recipe `foo` is redefined as a module",
        lsp::Range::at(1, 4, 1, 7),
      )
      .run();

    AnalyzerTest::new("mod foo\nalias foo := bar\nbar:\n")
      .file("foo.just", "")
      .error(
        "Module `foo` is redefined as an alias",
        lsp::Range::at(1, 6, 1, 9),
      )
      .run();

    AnalyzerTest::new("import 'bar.just'\nfoo:\n")
      .file("bar.just", "mod foo\n")
      .file("foo.just", "")
      .error(
        "Module `foo` is redefined as a recipe",
        lsp::Range::at(1, 0, 1, 3),
      )
      .run();

    AnalyzerTest::new("mod? foo\nfoo:\n").run();

    AnalyzerTest::new("[linux]\nmod foo\n[windows]\nfoo:\n")
      .file("foo.just", "")
      .run();
  }
}
