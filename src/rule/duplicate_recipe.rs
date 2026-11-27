use super::*;

define_rule! {
  /// Detects recipes that have the same name and overlapping OS constraints,
  /// which would shadow each other at runtime unless overrides are enabled.
  DuplicateRecipeRule {
    id: "duplicate-recipe",
    title: "duplicate recipes",
    severity: Error,
  }
}

impl DuplicateRecipeRule {
  fn run(context: &RuleContext<'_>) -> Vec<Diagnostic> {
    let allow_duplicates = context.setting_enabled("allow-duplicate-recipes");

    if allow_duplicates {
      return Vec::new();
    }

    let mut diagnostics = Vec::new();

    for recipe in context.duplicate_declarations(context.recipes(Scope::Module))
    {
      diagnostics.push(Diagnostic::new(
        format!("Duplicate recipe name `{}`", recipe.name.value),
        recipe.range,
      ));
    }

    diagnostics
  }
}

#[cfg(test)]
mod tests {
  use {super::*, indoc::indoc};

  #[test]
  fn all_four_os_groups_no_conflict() {
    AnalyzerTest::new(indoc! {
      "
      [linux]
      build:
        echo \"Building on Linux\"

      [macos]
      build:
        echo \"Building on macOS\"

      [windows]
      build:
        echo \"Building on Windows\"

      [dragonfly]
      build:
        echo \"Building on DragonFly BSD\"

      [freebsd]
      build:
        echo \"Building on FreeBSD\"

      [netbsd]
      build:
        echo \"Building on NetBSD\"

      [openbsd]
      build:
        echo \"Building on OpenBSD\"
      "
    })
    .run();
  }

  #[test]
  fn analyzer_ignores_imported_duplicate_recipes() {
    AnalyzerTest::new("import 'foo.just'\n")
      .file("foo.just", "foo:\nfoo:\n")
      .run();
  }

  #[test]
  fn analyzer_imported_duplicate_recipe_reports_local_declaration() {
    AnalyzerTest::new("foo:\nimport 'foo.just'\n")
      .file("foo.just", "\n\nfoo:\n")
      .error("Duplicate recipe name `foo`", lsp::Range::at(0, 0, 1, 0))
      .run();
  }

  #[test]
  fn bsd_os_specific_no_conflict() {
    AnalyzerTest::new(indoc! {
      "
      [dragonfly]
      build:
        echo \"Building on DragonFly BSD\"

      [freebsd]
      build:
        echo \"Building on FreeBSD\"

      [netbsd]
      build:
        echo \"Building on NetBSD\"

      [openbsd]
      build:
        echo \"Building on OpenBSD\"
      "
    })
    .run();
  }

  #[test]
  fn comma_separated_os_attributes_no_conflict() {
    AnalyzerTest::new(indoc! {
      "
      [private, unix]
      hello:
        @echo 'hello'

      [private, windows]
      hello:
        @echo hello
      "
    })
    .run();
  }

  #[test]
  fn comma_separated_os_attributes_with_conflict() {
    AnalyzerTest::new(indoc! {
      "
      [private, linux]
      hello:
        @echo 'hello on linux'

      [private, linux]
      hello:
        @echo 'hello on linux again'
      "
    })
    .error("Duplicate recipe name `hello`", lsp::Range::at(4, 0, 7, 0))
    .run();
  }

  #[test]
  fn comma_separated_unix_windows_no_conflict() {
    AnalyzerTest::new(indoc! {
      "
      [unix]
      [private]
      build:
        @echo 'building on unix'

      [private, windows]
      build:
        @echo 'building on windows'
      "
    })
    .run();
  }

  #[test]
  fn duplicate_recipe_names() {
    AnalyzerTest::new(indoc! {
      "
      foo:
        echo foo

      foo:
        echo foo

      foo:
        echo foo
      "
    })
    .error("Duplicate recipe name `foo`", lsp::Range::at(3, 0, 6, 0))
    .error("Duplicate recipe name `foo`", lsp::Range::at(6, 0, 8, 0))
    .run();
  }

  #[test]
  fn duplicate_recipe_names_allowed_via_setting() {
    AnalyzerTest::new(indoc! {
      "
      set allow-duplicate-recipes := true

      foo:
        echo foo

      [linux]
      foo:
        echo foo on linux
      "
    })
    .run();
  }

  #[test]
  fn duplicate_recipes_with_same_os_attribute() {
    AnalyzerTest::new(indoc! {
      "
      [linux]
      build:
        echo \"Building on Linux version 1\"

      [linux]
      build:
        echo \"Building on Linux version 2\"
      "
    })
    .error("Duplicate recipe name `build`", lsp::Range::at(4, 0, 7, 0))
    .run();
  }

  #[test]
  fn linux_openbsd_no_conflict() {
    AnalyzerTest::new(indoc! {
      "
      [linux]
      build:
        echo \"Building on Linux\"

      [openbsd]
      build:
        echo \"Building on OpenBSD\"
      "
    })
    .run();
  }

  #[test]
  fn linux_unix_conflict() {
    AnalyzerTest::new(indoc! {
      "
      [linux]
      build:
        echo \"Building on Linux\"

      [unix]
      build:
        echo \"Building on Unix systems\"
      "
    })
    .error("Duplicate recipe name `build`", lsp::Range::at(4, 0, 7, 0))
    .run();
  }

  #[test]
  fn mixed_os_specific_and_regular_recipe() {
    AnalyzerTest::new(indoc! {
      "
      [linux]
      build:
        echo \"Building on Linux\"

      build:
        echo \"Building on any OS\"
      "
    })
    .error("Duplicate recipe name `build`", lsp::Range::at(4, 0, 6, 0))
    .run();
  }

  #[test]
  fn openbsd_macos_no_conflict() {
    AnalyzerTest::new(indoc! {
      "
      [openbsd]
      build:
        echo \"Building on OpenBSD\"

      [macos]
      build:
        echo \"Building on macOS\"
      "
    })
    .run();
  }

  #[test]
  fn os_specific_duplicate_recipes() {
    AnalyzerTest::new(indoc! {
      "
      [linux]
      build:
        echo \"Building on Linux\"

      [windows]
      build:
        echo \"Building on Windows\"

      [macos]
      build:
        echo \"Building on macOS\"
      "
    })
    .run();
  }

  #[test]
  fn recipe_with_all_os_attributes() {
    AnalyzerTest::new(indoc! {
      "
      [linux]
      [windows]
      [unix]
      [macos]
      [dragonfly]
      [freebsd]
      [netbsd]
      [openbsd]
      build:
        echo \"Building everywhere\"

      test:
        echo \"Testing\"
      "
    })
    .run();
  }

  #[test]
  fn recipe_with_conflicting_multiple_os_attributes() {
    AnalyzerTest::new(indoc! {
      "
      [linux]
      [openbsd]
      build:
        echo \"Building on Linux and OpenBSD\"

      [linux]
      build:
        echo \"Building on Linux again\"
      "
    })
    .error("Duplicate recipe name `build`", lsp::Range::at(5, 0, 8, 0))
    .run();
  }

  #[test]
  fn recipe_with_multiple_os_attributes() {
    AnalyzerTest::new(indoc! {
      "
      [windows]
      [linux]
      build:
        echo \"Building on Linux or Windows\"

      [linux]
      build:
        echo \"Building on macOS\"

      [macos]
      build:
        echo \"Building on macOS\"
      "
    })
    .error("Duplicate recipe name `build`", lsp::Range::at(5, 0, 9, 0))
    .run();
  }

  #[test]
  fn unix_dragonfly_conflict() {
    AnalyzerTest::new(indoc! {
      "
      [unix]
      build:
        echo \"Building on Unix systems\"

      [dragonfly]
      build:
        echo \"Building on DragonFly BSD\"
      "
    })
    .error("Duplicate recipe name `build`", lsp::Range::at(4, 0, 7, 0))
    .run();
  }

  #[test]
  fn unix_freebsd_conflict() {
    AnalyzerTest::new(indoc! {
      "
      [unix]
      build:
        echo \"Building on Unix systems\"

      [freebsd]
      build:
        echo \"Building on FreeBSD\"
      "
    })
    .error("Duplicate recipe name `build`", lsp::Range::at(4, 0, 7, 0))
    .run();
  }

  #[test]
  fn unix_macos_conflicts() {
    AnalyzerTest::new(indoc! {
      "
      [unix]
      build:
        echo \"Building on Unix systems\"

      [macos]
      build:
        echo \"Building on macOS specifically\"
      "
    })
    .error("Duplicate recipe name `build`", lsp::Range::at(4, 0, 7, 0))
    .run();
  }

  #[test]
  fn unix_netbsd_conflict() {
    AnalyzerTest::new(indoc! {
      "
      [unix]
      build:
        echo \"Building on Unix systems\"

      [netbsd]
      build:
        echo \"Building on NetBSD\"
      "
    })
    .error("Duplicate recipe name `build`", lsp::Range::at(4, 0, 7, 0))
    .run();
  }

  #[test]
  fn windows_recipe_conflicts_with_default() {
    AnalyzerTest::new(indoc! {
      "
      [windows]
      build:
        echo \"Building on Windows\"

      build:
        echo \"Building on every OS\"
      "
    })
    .error("Duplicate recipe name `build`", lsp::Range::at(4, 0, 6, 0))
    .run();
  }
}
