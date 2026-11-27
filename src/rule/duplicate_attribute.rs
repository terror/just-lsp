use super::*;

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
enum AttributeKey {
  Group(StringLiteral),
  Name(String),
}

impl Display for AttributeKey {
  fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
    match self {
      Self::Group(group) => {
        write!(f, "`group` with value `{}`", group.cooked)
      }
      Self::Name(name) => write!(f, "`{name}`"),
    }
  }
}

define_rule! {
  DuplicateAttributeRule {
    id: "duplicate-attribute",
    title: "duplicate attribute",
    severity: Error,
  }
}

impl DuplicateAttributeRule {
  fn run(context: &RuleContext<'_>) -> Vec<Diagnostic> {
    let document = context.document();

    let (mut diagnostics, mut defaults) = (Vec::new(), GroupSet::default());

    for recipe in context.recipes(Scope::Imported) {
      if recipe.has_attribute("default") {
        defaults.union_with(recipe.groups());
      }
    }

    for recipe in context.recipes(Scope::Local) {
      let groups = recipe.groups();

      for attribute in recipe
        .attributes
        .iter()
        .filter(|attribute| attribute.name.value == "default")
      {
        let conflict = defaults.conflicts_with(&groups);

        defaults.union_with(groups.clone());

        if conflict {
          diagnostics.push(Diagnostic::new(
            format!(
              "Recipe `{}` has duplicate `[default]` attribute, which may only appear once per module",
              recipe.name.value
            ),
            attribute.range,
          ));
        }
      }
    }

    let mut seen = HashSet::new();

    for attribute_node in context.tree().root_node().find_all("attribute") {
      let Some(parent) = attribute_node.parent() else {
        continue;
      };

      let Some(target) = AttributeTarget::try_from_kind(parent.kind()) else {
        continue;
      };

      let target_key = (parent.start_byte(), parent.end_byte());

      for identifier in attribute_node.find_all("^identifier") {
        let attribute_name = document.get_node_text(&identifier);

        let key = match attribute_name.as_str() {
          "arg" | "env" | "metadata" => continue,
          "default" if target == AttributeTarget::Recipe => continue,
          "group" => {
            let group = identifier
              .siblings()
              .take_while(|node| node.kind() != "identifier")
              .find(|node| node.kind() == "expression")
              .and_then(|argument| {
                let value = argument.find("^value")?;

                let string = value.find("^string")?;

                if string.byte_range() != argument.byte_range() {
                  return None;
                }

                StringLiteral::parse(&document.get_node_text(&string)).ok()?
              });

            let Some(group) = group else {
              continue;
            };

            AttributeKey::Group(group)
          }
          _ if Builtin::attribute(&attribute_name).is_some() => {
            AttributeKey::Name(attribute_name)
          }
          _ => continue,
        };

        if !seen.insert((target_key, key.clone())) {
          diagnostics.push(Diagnostic::new(
            format!("{} attribute {key} is duplicated", target.target_name()),
            document.get_range(&attribute_node),
          ));
        }
      }
    }

    diagnostics
  }
}

#[cfg(test)]
mod tests {
  use {super::*, indoc::indoc};

  #[test]
  fn analyzer_imported_duplicate_default_reports_local_attribute() {
    AnalyzerTest::new("[default]\nfoo:\nimport 'foo.just'\n")
      .file("foo.just", "\n\n[default]\nbar:\n")
      .error(
        "Recipe `foo` has duplicate `[default]` attribute, which may only appear once per module",
        lsp::Range::at(0, 0, 1, 0),
      )
      .run();
  }

  #[test]
  fn attributes_duplicate_default_between_recipes() {
    AnalyzerTest::new(indoc! {
      "
      [default]
      check:
        echo \"check\"

      [default]
      ci:
        echo \"ci\"
      "
    })
    .error(
      "Recipe `ci` has duplicate `[default]` attribute, which may only appear once per module", lsp::Range::at(4, 0, 5, 0))
    .run();
  }

  #[test]
  fn attributes_duplicate_default_on_same_recipe() {
    AnalyzerTest::new(indoc! {
      "
      [default]
      [default]
      build:
        echo \"build\"
      "
    })
    .error(
      "Recipe `build` has duplicate `[default]` attribute, which may only appear once per module", lsp::Range::at(1, 0, 2, 0))
    .run();
  }

  #[test]
  fn attributes_duplicate_group_attribute() {
    #[track_caller]
    fn case(first: &str, second: &str) {
      AnalyzerTest::new(&format!(
        "[group({first})]\n[group({second})]\nfoo:\n"
      ))
      .error(
        "Recipe attribute `group` with value `foo` is duplicated",
        lsp::Range::at(1, 0, 2, 0),
      )
      .run();
    }

    case("'foo'", "\"foo\"");
    case("'foo'", r#""\u{66}oo""#);
    case("'foo'", "'''  foo'''");
    case("x'foo'", "x\"foo\"");
  }

  #[test]
  fn attributes_duplicate_group_inline() {
    AnalyzerTest::new(indoc! {
      "
      [group: 'foo', no-cd, group('foo')]
      bar:
      "
    })
    .error(
      "Recipe attribute `group` with value `foo` is duplicated",
      lsp::Range::at(0, 0, 1, 0),
    )
    .run();
  }

  #[test]
  fn attributes_duplicate_group_module() {
    AnalyzerTest::new(indoc! {
      "
      [group('foo')]
      [group: 'foo']
      mod bar
      "
    })
    .file("bar.just", "")
    .error(
      "Module attribute `group` with value `foo` is duplicated",
      lsp::Range::at(1, 0, 2, 0),
    )
    .run();
  }

  #[test]
  fn attributes_duplicate_group_without_argument() {
    AnalyzerTest::new(indoc! {
      "
      [group('foo')]
      [group, group('foo')]
      bar:
      "
    })
    .error(
      "Attribute `group` got 0 arguments but takes 1 argument",
      lsp::Range::at(1, 0, 2, 0),
    )
    .error(
      "Recipe attribute `group` with value `foo` is duplicated",
      lsp::Range::at(1, 0, 2, 0),
    )
    .run();
  }

  #[test]
  fn attributes_duplicate_non_repeatable_outside_recipes() {
    #[track_caller]
    fn case(content: &'static str, message: &'static str) {
      AnalyzerTest::new(content)
        .file("foo.just", "")
        .error(message, lsp::Range::at(1, 0, 2, 0))
        .run();
    }

    case(
      indoc! {
        "
        [private]
        [private]
        alias f := foo

        foo:
        "
      },
      "Alias attribute `private` is duplicated",
    );

    case(
      indoc! {
        "
        [private]
        [private]
        foo := 'bar'

        bar:
          echo {{foo}}
        "
      },
      "Assignment attribute `private` is duplicated",
    );

    case(
      indoc! {
        "
        [doc('foo')]
        [doc('bar')]
        mod foo
        "
      },
      "Module attribute `doc` is duplicated",
    );
  }

  #[test]
  fn attributes_duplicate_recipe_attribute() {
    AnalyzerTest::new(indoc! {
      "
      [script]
      [script]
      build:
        echo \"build\"
      "
    })
    .error(
      "Recipe attribute `script` is duplicated",
      lsp::Range::at(1, 0, 2, 0),
    )
    .run();
  }

  #[test]
  fn attributes_duplicate_shell_attribute() {
    AnalyzerTest::new(indoc! {
      "
      [shell]
      [shell]
      build:
        echo \"build\"
      "
    })
    .error(
      "Recipe attribute `shell` is duplicated",
      lsp::Range::at(1, 0, 2, 0),
    )
    .run();
  }

  #[test]
  fn attributes_duplicate_working_directory_attribute() {
    AnalyzerTest::new(indoc! {
      "
      [working-directory: 'foo']
      [working-directory: 'bar']
      build:
        echo \"build\"
      "
    })
    .error(
      "Recipe attribute `working-directory` is duplicated",
      lsp::Range::at(1, 0, 2, 0),
    )
    .run();
  }

  #[test]
  fn attributes_multiple_group_attributes_allowed() {
    #[track_caller]
    fn case(first: &str, second: &str) {
      AnalyzerTest::new(&format!(
        "[group({first})]\n[group({second})]\nfoo:\n"
      ))
      .run();
    }

    case("'foo'", "'bar'");
    case("'$FOO'", "x'$FOO'");
    case(r"'\t'", r#""\t""#);
  }

  #[test]
  fn attributes_multiple_metadata_allowed() {
    AnalyzerTest::new(indoc! {
      "
      [metadata('foo', 'bar')]
      [metadata('baz', 'qux')]
      foo:
        echo \"foo\"
      "
    })
    .run();
  }

  #[test]
  fn attributes_platform_specific_defaults() {
    AnalyzerTest::new(indoc! {
      "
      [unix]
      [default]
      foo:
        echo \"foo\"

      [windows]
      [default]
      bar:
        echo \"bar\"
      "
    })
    .run();
  }

  #[test]
  fn attributes_same_group_on_multiple_targets_allowed() {
    AnalyzerTest::new(indoc! {
      "
      [group('foo')]
      foo:

      [group('foo')]
      bar:

      [group('foo')]
      mod baz
      "
    })
    .file("baz.just", "")
    .run();
  }

  #[test]
  fn duplicate_dragonfly_attribute() {
    AnalyzerTest::new(indoc! {
      "
      [dragonfly]
      [dragonfly]
      build:
        echo \"foo\"
      "
    })
    .error(
      "Recipe attribute `dragonfly` is duplicated",
      lsp::Range::at(1, 0, 2, 0),
    )
    .run();
  }

  #[test]
  fn duplicate_freebsd_attribute() {
    AnalyzerTest::new(indoc! {
      "
      [freebsd]
      [freebsd]
      build:
        echo \"foo\"
      "
    })
    .error(
      "Recipe attribute `freebsd` is duplicated",
      lsp::Range::at(1, 0, 2, 0),
    )
    .run();
  }

  #[test]
  fn duplicate_netbsd_attribute() {
    AnalyzerTest::new(indoc! {
      "
      [netbsd]
      [netbsd]
      build:
        echo \"foo\"
      "
    })
    .error(
      "Recipe attribute `netbsd` is duplicated",
      lsp::Range::at(1, 0, 2, 0),
    )
    .run();
  }

  #[test]
  fn env_attribute_duplicate_var_name_allowed() {
    AnalyzerTest::new(indoc! {
      "
      [env('FOO', 'bar')]
      [env('FOO', 'baz')]
      foo:
        echo \"$FOO\"
      "
    })
    .run();
  }
}
