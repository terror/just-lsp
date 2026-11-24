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
    message: "module redefinition",
    run(context) {
      let filesystem = context.view().filesystem();

      let modules = context
        .view()
        .modules()
        .iter()
        .filter(|module| {
          module.resolve(&module.uri, filesystem).is_some()
        })
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

          Some(Diagnostic::error(
            first.redefinition_message(definition),
            definition.name.range,
          ))
        })
        .collect()
    }
  }
}

#[cfg(test)]
mod tests {
  use {super::*, pretty_assertions::assert_eq};

  #[test]
  fn module_redefinition() {
    #[track_caller]
    fn case(files: &[(&str, &str)], expected: &[(&str, lsp::Range)]) {
      assert_eq!(
        Analyzer {
          config: None,
          view: TestModule::new(files).view(),
        }
        .analyze()
        .iter()
        .map(|diagnostic| (diagnostic.message.as_str(), diagnostic.range))
        .collect::<Vec<_>>(),
        expected,
      );
    }

    case(
      &[("justfile", "mod foo\nmod foo\n"), ("foo.just", "")],
      &[("Duplicate module `foo`", lsp::Range::at(1, 4, 1, 7))],
    );

    case(
      &[("justfile", "foo:\nmod foo\n"), ("foo.just", "")],
      &[(
        "Recipe `foo` is redefined as a module",
        lsp::Range::at(1, 4, 1, 7),
      )],
    );

    case(
      &[
        ("justfile", "mod foo\nalias foo := bar\nbar:\n"),
        ("foo.just", ""),
      ],
      &[(
        "Module `foo` is redefined as an alias",
        lsp::Range::at(1, 6, 1, 9),
      )],
    );

    case(
      &[
        ("justfile", "import 'bar.just'\nfoo:\n"),
        ("bar.just", "mod foo\n"),
        ("foo.just", ""),
      ],
      &[(
        "Module `foo` is redefined as a recipe",
        lsp::Range::at(1, 0, 1, 3),
      )],
    );

    case(&[("justfile", "mod? foo\nfoo:\n")], &[]);

    case(
      &[
        ("justfile", "[linux]\nmod foo\n[windows]\nfoo:\n"),
        ("foo.just", ""),
      ],
      &[],
    );
  }
}
