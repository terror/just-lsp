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
  ) -> Self {
    Self {
      groups: GroupSet::from_attributes(attributes),
      kind,
      name,
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
  fn definitions<'a>(
    context: &RuleContext<'a>,
    scope: Scope,
  ) -> Vec<Definition<'a>> {
    let filesystem = context.filesystem();

    context
      .modules(scope)
      .filter(|module| module.resolve(&module.uri, filesystem).is_some())
      .map(|module| {
        Definition::new(
          DefinitionKind::Module,
          &module.name,
          &module.attributes,
        )
      })
      .chain(context.aliases(scope).map(|alias| {
        Definition::new(DefinitionKind::Alias, &alias.name, &alias.attributes)
      }))
      .chain(context.recipes(scope).map(|recipe| {
        Definition::new(
          DefinitionKind::Recipe,
          &recipe.name,
          &recipe.attributes,
        )
      }))
      .collect()
  }

  fn run(context: &RuleContext<'_>) -> Vec<Diagnostic> {
    if context.modules(Scope::Module).next().is_none() {
      return Vec::new();
    }

    let imported = Self::definitions(context, Scope::Imported);

    let mut local = Self::definitions(context, Scope::Local);

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
