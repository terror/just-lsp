use super::*;

#[derive(Copy, Clone)]
enum DuplicateScope {
  /// Attribute must be unique within the entire document.
  Module,
  /// Attribute must be unique per recipe.
  Recipe,
}

struct DuplicateConstraint {
  name: &'static str,
  scope: DuplicateScope,
}

const DUPLICATE_CONSTRAINTS: &[DuplicateConstraint] = &[
  DuplicateConstraint {
    name: "default",
    scope: DuplicateScope::Module,
  },
  DuplicateConstraint {
    name: "confirm",
    scope: DuplicateScope::Recipe,
  },
  DuplicateConstraint {
    name: "doc",
    scope: DuplicateScope::Recipe,
  },
  DuplicateConstraint {
    name: "extension",
    scope: DuplicateScope::Recipe,
  },
  DuplicateConstraint {
    name: "metadata",
    scope: DuplicateScope::Recipe,
  },
  DuplicateConstraint {
    name: "linux",
    scope: DuplicateScope::Recipe,
  },
  DuplicateConstraint {
    name: "macos",
    scope: DuplicateScope::Recipe,
  },
  DuplicateConstraint {
    name: "no-cd",
    scope: DuplicateScope::Recipe,
  },
  DuplicateConstraint {
    name: "no-exit-message",
    scope: DuplicateScope::Recipe,
  },
  DuplicateConstraint {
    name: "no-quiet",
    scope: DuplicateScope::Recipe,
  },
  DuplicateConstraint {
    name: "openbsd",
    scope: DuplicateScope::Recipe,
  },
  DuplicateConstraint {
    name: "parallel",
    scope: DuplicateScope::Recipe,
  },
  DuplicateConstraint {
    name: "positional-arguments",
    scope: DuplicateScope::Recipe,
  },
  DuplicateConstraint {
    name: "private",
    scope: DuplicateScope::Recipe,
  },
  DuplicateConstraint {
    name: "script",
    scope: DuplicateScope::Recipe,
  },
  DuplicateConstraint {
    name: "unix",
    scope: DuplicateScope::Recipe,
  },
  DuplicateConstraint {
    name: "windows",
    scope: DuplicateScope::Recipe,
  },
  DuplicateConstraint {
    name: "working-directory",
    scope: DuplicateScope::Recipe,
  },
];

/// Reports duplicate usages of attributes that must be unique.
pub(crate) struct DuplicateAttributeRule;

impl DuplicateAttributeRule {
  fn constraint(name: &str) -> Option<&'static DuplicateConstraint> {
    DUPLICATE_CONSTRAINTS
      .iter()
      .find(|constraint| constraint.name == name)
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

impl Rule for DuplicateAttributeRule {
  fn display_name(&self) -> &'static str {
    "Duplicate Attribute"
  }

  fn id(&self) -> &'static str {
    "duplicate-attribute"
  }

  fn run(&self, context: &RuleContext<'_>) -> Vec<lsp::Diagnostic> {
    let mut diagnostics = Vec::new();
    let mut module_seen: HashSet<&'static str> = HashSet::new();

    for recipe in context.recipes() {
      let mut recipe_seen: HashSet<&'static str> = HashSet::new();

      for attribute in &recipe.attributes {
        let attribute_name = attribute.name.value.as_str();

        let Some(constraint) = Self::constraint(attribute_name) else {
          continue;
        };

        let already_seen = match constraint.scope {
          DuplicateScope::Module => !module_seen.insert(constraint.name),
          DuplicateScope::Recipe => !recipe_seen.insert(constraint.name),
        };

        if already_seen {
          diagnostics.push(self.diagnostic(lsp::Diagnostic {
            range: attribute.range,
            severity: Some(lsp::DiagnosticSeverity::ERROR),
            message: Self::message(constraint, recipe),
            ..Default::default()
          }));
        }
      }
    }

    diagnostics
  }
}
