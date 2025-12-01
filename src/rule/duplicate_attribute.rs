use super::*;

#[derive(Clone, Copy, Debug)]
enum DuplicateScope {
  /// Attribute must be unique within the entire document.
  Module,
  /// Attribute must be unique per recipe.
  Recipe,
}

#[derive(Clone, Copy, Debug)]
enum DuplicateKey {
  Argument,
  Name,
}

#[derive(Debug)]
struct DuplicateConstraint {
  key: DuplicateKey,
  name: &'static str,
  scope: DuplicateScope,
}

const DUPLICATE_CONSTRAINTS: &[DuplicateConstraint] = &[
  DuplicateConstraint {
    name: "default",
    scope: DuplicateScope::Module,
    key: DuplicateKey::Name,
  },
  DuplicateConstraint {
    name: "confirm",
    scope: DuplicateScope::Recipe,
    key: DuplicateKey::Name,
  },
  DuplicateConstraint {
    name: "doc",
    scope: DuplicateScope::Recipe,
    key: DuplicateKey::Name,
  },
  DuplicateConstraint {
    name: "extension",
    scope: DuplicateScope::Recipe,
    key: DuplicateKey::Name,
  },
  DuplicateConstraint {
    name: "metadata",
    scope: DuplicateScope::Recipe,
    key: DuplicateKey::Name,
  },
  DuplicateConstraint {
    name: "group",
    scope: DuplicateScope::Recipe,
    key: DuplicateKey::Argument,
  },
  DuplicateConstraint {
    name: "linux",
    scope: DuplicateScope::Recipe,
    key: DuplicateKey::Name,
  },
  DuplicateConstraint {
    name: "macos",
    scope: DuplicateScope::Recipe,
    key: DuplicateKey::Name,
  },
  DuplicateConstraint {
    name: "no-cd",
    scope: DuplicateScope::Recipe,
    key: DuplicateKey::Name,
  },
  DuplicateConstraint {
    name: "no-exit-message",
    scope: DuplicateScope::Recipe,
    key: DuplicateKey::Name,
  },
  DuplicateConstraint {
    name: "no-quiet",
    scope: DuplicateScope::Recipe,
    key: DuplicateKey::Name,
  },
  DuplicateConstraint {
    name: "openbsd",
    scope: DuplicateScope::Recipe,
    key: DuplicateKey::Name,
  },
  DuplicateConstraint {
    name: "parallel",
    scope: DuplicateScope::Recipe,
    key: DuplicateKey::Name,
  },
  DuplicateConstraint {
    name: "positional-arguments",
    scope: DuplicateScope::Recipe,
    key: DuplicateKey::Name,
  },
  DuplicateConstraint {
    name: "private",
    scope: DuplicateScope::Recipe,
    key: DuplicateKey::Name,
  },
  DuplicateConstraint {
    name: "script",
    scope: DuplicateScope::Recipe,
    key: DuplicateKey::Name,
  },
  DuplicateConstraint {
    name: "unix",
    scope: DuplicateScope::Recipe,
    key: DuplicateKey::Name,
  },
  DuplicateConstraint {
    name: "windows",
    scope: DuplicateScope::Recipe,
    key: DuplicateKey::Name,
  },
  DuplicateConstraint {
    name: "working-directory",
    scope: DuplicateScope::Recipe,
    key: DuplicateKey::Name,
  },
];

/// Reports duplicate usages of attributes that must be unique.
pub(crate) struct DuplicateAttributeRule;

impl Rule for DuplicateAttributeRule {
  fn id(&self) -> &'static str {
    "duplicate-attribute"
  }

  fn message(&self) -> &'static str {
    "duplicate attribute"
  }

  fn run(&self, context: &RuleContext<'_>) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();

    let mut module_seen: HashMap<&'static str, HashSet<String>> =
      HashMap::new();

    for recipe in context.recipes() {
      let mut recipe_seen: HashMap<&'static str, HashSet<String>> =
        HashMap::new();

      for attribute in &recipe.attributes {
        let attribute_name = attribute.name.value.as_str();

        let Some(constraint) = Self::constraint(attribute_name) else {
          continue;
        };

        let Some(key) = Self::key(constraint, attribute) else {
          continue;
        };

        let seen = match constraint.scope {
          DuplicateScope::Module => {
            module_seen.entry(constraint.name).or_default()
          }
          DuplicateScope::Recipe => {
            recipe_seen.entry(constraint.name).or_default()
          }
        };

        if !seen.insert(key.clone()) {
          diagnostics.push(Diagnostic::error(
            Self::message(constraint, recipe),
            attribute.range,
          ));
        }
      }
    }

    diagnostics
  }
}

impl DuplicateAttributeRule {
  fn constraint(name: &str) -> Option<&'static DuplicateConstraint> {
    DUPLICATE_CONSTRAINTS
      .iter()
      .find(|constraint| constraint.name == name)
  }

  fn key(
    constraint: &DuplicateConstraint,
    attribute: &Attribute,
  ) -> Option<String> {
    match constraint.key {
      DuplicateKey::Name => Some(attribute.name.value.clone()),
      DuplicateKey::Argument => attribute
        .arguments
        .first()
        .map(|argument| argument.value.clone()),
    }
  }

  fn message(constraint: &DuplicateConstraint, recipe: &Recipe) -> String {
    match constraint.scope {
      DuplicateScope::Module => format!(
        "Recipe `{}` has duplicate `[{}]` attribute, which may only appear once per module",
        recipe.name, constraint.name
      ),
      DuplicateScope::Recipe => {
        format!("Recipe attribute `{}` is duplicated", constraint.name)
      }
    }
  }
}
