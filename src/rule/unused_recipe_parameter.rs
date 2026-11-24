use super::*;

define_rule! {
  /// Highlights recipe parameters that never get read anywhere in the recipe
  /// body (unless `set export` is on).
  UnusedRecipeParameterRule {
    id: "unused-recipe-parameter",
    message: "unused parameter",
    run(context) {
      let default_script = context.setting_enabled("default-script");

      let exported = context.setting_enabled("export");

      let positional_arguments_enabled = context.setting_enabled("positional-arguments");

      let used_parameters = &context.references().used_parameters;

      context
        .local_declarations(context.recipes())
        .flat_map(|recipe| {
          let recipe_enables_positional_arguments =
            positional_arguments_enabled || recipe.has_attribute("positional-arguments");

          let (positional_usage, uses_all) = if recipe_enables_positional_arguments {
            let (indices, uses_all) = UnusedRecipeParameterRule::positional_argument_usage(recipe);

            (indices, recipe.runs_as_script(default_script) || uses_all)
          } else {
            (HashSet::new(), false)
          };

          recipe.parameters.iter().enumerate().filter_map(move |(index, parameter)| {
            let used_via_position = uses_all || positional_usage.contains(&(index + 1));

            let is_unused = !used_parameters.contains(&Binding::Parameter {
              name: parameter.name.clone(),
              recipe: recipe.range,
            })
              && !parameter.export
              && !exported
              && !used_via_position;

            is_unused.then(|| {
              Diagnostic::warning(
                format!("Parameter `{}` appears unused", parameter.name),
                parameter.range,
              )
            })
          })
        })
        .collect()
    }
  }
}

enum Slice {
  All,
  Malformed,
  Positions(Vec<usize>),
}

impl UnusedRecipeParameterRule {
  fn parse_positional(bytes: &[u8], braced: bool) -> Option<usize> {
    let inner = if braced {
      bytes.strip_prefix(b"{").and_then(|b| b.strip_suffix(b"}"))
    } else {
      Some(bytes)
    }?;

    if inner.is_empty() || !inner.iter().all(u8::is_ascii_digit) {
      return None;
    }

    str::from_utf8(inner).ok()?.parse().ok().filter(|&n| n > 0)
  }

  fn parse_slice(rest: &[u8], parameters: usize) -> Slice {
    let Some(end) = rest.iter().position(|&b| b == b'}') else {
      return Slice::Malformed;
    };

    let Some(inner) = rest[..end].strip_prefix(b":") else {
      return Slice::Malformed;
    };

    let Ok(inner) = str::from_utf8(inner) else {
      return Slice::Malformed;
    };

    let (offset, length) = match inner.split_once(':') {
      Some((offset, length)) => (offset, Some(length)),
      None => (inner, None),
    };

    let Some(offset) = Self::slice_number(offset) else {
      return Slice::All;
    };

    let upper = match length.and_then(Self::slice_number) {
      Some(length) => offset.saturating_add(length).saturating_sub(1),
      None if offset == 0 => return Slice::All,
      None => parameters,
    };

    Slice::Positions((offset.max(1)..=upper.min(parameters)).collect())
  }

  fn positional_argument_usage(recipe: &Recipe) -> (HashSet<usize>, bool) {
    recipe
      .body
      .iter()
      .flat_map(|line| {
        let bytes = line.value.as_bytes();

        bytes
          .iter()
          .enumerate()
          .filter(move |&(i, _)| {
            bytes[i] == b'$' && (i == 0 || bytes[i - 1] != b'\\')
          })
          .map(move |(i, _)| &bytes[i + 1..])
      })
      .fold((HashSet::new(), false), |(mut indices, uses_all), rest| {
        if matches!(rest.first(), Some(b'@' | b'*'))
          || matches!(rest, [b'{', b'@' | b'*', b'}', ..])
        {
          return (indices, true);
        }

        if let [b'{', b'@' | b'*', tail @ ..] = rest {
          match Self::parse_slice(tail, recipe.parameters.len()) {
            Slice::Positions(positions) => indices.extend(positions),
            Slice::All => return (indices, true),
            Slice::Malformed => {}
          }

          return (indices, uses_all);
        }

        let unbraced_end =
          rest.iter().take_while(|b| b.is_ascii_digit()).count();

        let index = if unbraced_end > 0 {
          Self::parse_positional(&rest[..unbraced_end], false)
        } else {
          rest
            .iter()
            .position(|&b| b == b'}')
            .and_then(|end| Self::parse_positional(&rest[..=end], true))
        };

        if let Some(index) = index {
          indices.insert(index);
        }

        (indices, uses_all)
      })
  }

  fn slice_number(text: &str) -> Option<usize> {
    let text = text.trim();

    if text.bytes().all(|b| b.is_ascii_digit()) {
      text.parse().ok()
    } else {
      None
    }
  }
}

#[cfg(test)]
mod tests {
  use {super::*, indoc::indoc, pretty_assertions::assert_eq};

  #[test]
  fn analyzer_imported_recipe_usage_does_not_hide_unused_parameters() {
    AnalyzerTest::new(
      "set allow-duplicate-recipes\nfoo bar:\nimport 'foo.just'\n",
    )
    .file("foo.just", "foo bar:\n  echo {{bar}}\n")
    .warning("Parameter `bar` appears unused", lsp::Range::at(1, 4, 1, 7))
    .run();
  }

  #[test]
  fn analyzer_uses_imported_recipe_parameters() {
    AnalyzerTest::new(indoc! {
      "
      import? 'foo.just'

      bar: (use 'arg')
      "
    })
    .file(
      "foo.just",
      indoc! {
        "
        use arg:
          echo {{arg}}
        "
      },
    )
    .run();
  }

  #[test]
  fn duplicate_recipe_parameters_are_checked_at_their_declarations() {
    AnalyzerTest::new(indoc! {
      "
      set allow-duplicate-recipes
      [arg('bar')]
      foo bar:
      [arg('baz')]
      foo baz:
      "
    })
    .warning("Parameter `bar` appears unused", lsp::Range::at(2, 4, 2, 7))
    .warning("Parameter `baz` appears unused", lsp::Range::at(4, 4, 4, 7))
    .run();
  }

  #[test]
  fn exported_variadic_recipe_parameters_are_not_unused() {
    AnalyzerTest::new(indoc! {
      "
      foo +$args:
        echo foo

      bar *$args:
        echo bar
      "
    })
    .run();
  }

  #[test]
  fn global_attribute_expressions_do_not_use_parameters() {
    AnalyzerTest::new(indoc! {
      "
      [env('FOO', bar)]
      foo bar:
        echo
      "
    })
    .error("Variable `bar` not found", lsp::Range::at(0, 12, 0, 15))
    .warning("Parameter `bar` appears unused", lsp::Range::at(1, 4, 1, 7))
    .run();
  }

  #[test]
  fn parse_positional_rejects_incomplete_braced() {
    assert_eq!(
      UnusedRecipeParameterRule::parse_positional(b"{56", true),
      None
    );
  }

  #[test]
  fn parse_positional_rejects_missing_digits() {
    assert_eq!(
      UnusedRecipeParameterRule::parse_positional(b"rest", false),
      None
    );
  }

  #[test]
  fn parse_positional_rejects_zero() {
    assert_eq!(
      UnusedRecipeParameterRule::parse_positional(b"0", false),
      None
    );
  }

  #[test]
  fn parse_positional_with_braces_extracts_number() {
    assert_eq!(
      UnusedRecipeParameterRule::parse_positional(b"{34}", true),
      Some(34)
    );
  }

  #[test]
  fn parse_positional_without_braces_extracts_number() {
    assert_eq!(
      UnusedRecipeParameterRule::parse_positional(b"12", false),
      Some(12)
    );
  }

  #[test]
  fn positional_argument_usage() {
    #[track_caller]
    fn case(body: &str, indices: &[usize], uses_all: bool) {
      let document = Document::from(format!("foo a b c:\n{body}").as_str());

      assert!(!document.tree.root_node().has_error());

      assert_eq!(
        UnusedRecipeParameterRule::positional_argument_usage(
          &document.recipes()[0]
        ),
        (indices.iter().copied().collect::<HashSet<_>>(), uses_all),
      );
    }

    case("", &[], false);
    case("  bar ${3} ${4}", &[3, 4], false);
    case("  bar $1 $2", &[1, 2], false);
    case("  bar $0 $foo ${bar} ${5} \\$6 ${7", &[5], false);
    case("  bar \"${@}\"", &[], true);
    case("  bar ${*}", &[], true);
    case("  bar \"$@\"", &[], true);
    case("  bar $*", &[], true);
    case("  bar \\$@", &[], false);
    case("  bar $1\n  baz $@\n  qux ${3}", &[1, 3], true);
    case("  bar \"${@:1}\"", &[1, 2, 3], false);
    case("  bar \"${@:2}\"", &[2, 3], false);
    case("  bar \"${*:3}\"", &[3], false);
    case("  bar \"${@:4}\"", &[], false);
    case("  bar \"${@:0}\"", &[], true);
    case("  bar \"${@:2:1}\"", &[2], false);
    case("  bar \"${@:2:2}\"", &[2, 3], false);
    case("  bar \"${*:2:2}\"", &[2, 3], false);
    case("  bar \"${@:2:0}\"", &[], false);
    case("  bar \"${@:0:2}\"", &[1], false);
    case("  bar \"${@:2:$n}\"", &[2, 3], false);
    case("  bar \\${@:2}", &[], false);
    case("  bar \"${@:$n}\"", &[], true);
    case("  bar \"${@: -1}\"", &[], true);
    case("  bar \"${@: 2}\"", &[2, 3], false);
    case("  bar \"${@:2", &[], false);
  }

  #[test]
  fn positional_arguments_attribute_marks_parameters_as_used() {
    AnalyzerTest::new(indoc! {
      "
      [positional-arguments]
      graph log:
        ./bin/graph $1
      "
    })
    .run();
  }

  #[test]
  fn positional_arguments_attribute_scope_is_limited() {
    AnalyzerTest::new(indoc! {
      "
      [positional-arguments]
      graph log:
        ./bin/graph $1

      other data:
        ./bin/graph $1
      "
    })
    .warning(
      "Parameter `data` appears unused",
      lsp::Range::at(4, 6, 4, 10),
    )
    .run();
  }

  #[test]
  fn positional_arguments_disabled_still_warns() {
    AnalyzerTest::new(indoc! {
      "
      graph log:
        ./bin/graph $1
      "
    })
    .warning("Parameter `log` appears unused", lsp::Range::at(0, 6, 0, 9))
    .run();
  }

  #[test]
  fn positional_arguments_dollar_at_marks_all_as_used() {
    AnalyzerTest::new(indoc! {
      r#"
      [positional-arguments]
      run *args:
        #!/usr/bin/env bash
        exec "$@"
      "#
    })
    .run();
  }

  #[test]
  fn positional_arguments_ignore_attributes() {
    AnalyzerTest::new(indoc! {
      "
      [positional-arguments]
      [doc: '$@']
      foo bar baz:
        echo $2
      "
    })
    .warning("Parameter `bar` appears unused", lsp::Range::at(2, 4, 2, 7))
    .run();
  }

  #[test]
  fn positional_arguments_ignore_parameter_defaults() {
    AnalyzerTest::new(indoc! {
      "
      set positional-arguments
      foo bar='$1' baz='qux':
        echo $2
      "
    })
    .warning(
      "Parameter `bar` appears unused",
      lsp::Range::at(1, 4, 1, 12),
    )
    .run();
  }

  #[test]
  fn positional_arguments_only_mark_used_indices() {
    AnalyzerTest::new(indoc! {
      "
      set positional-arguments := true

      graph first second:
        ./bin/graph $2
      "
    })
    .warning(
      "Parameter `first` appears unused",
      lsp::Range::at(2, 6, 2, 11),
    )
    .run();
  }

  #[test]
  fn positional_arguments_setting_handles_multiple_parameters() {
    AnalyzerTest::new(indoc! {
      "
      set positional-arguments := true

      graph first second third:
        ./bin/graph $1 ${2} $3
      "
    })
    .run();
  }

  #[test]
  fn positional_arguments_setting_handles_multiple_parameters_unused() {
    AnalyzerTest::new(indoc! {
      "
      set positional-arguments := true

      graph first second third fourth:
        ./bin/graph $1 ${2} $3
      "
    })
    .warning(
      "Parameter `fourth` appears unused",
      lsp::Range::at(2, 25, 2, 31),
    )
    .run();
  }

  #[test]
  fn positional_arguments_setting_marks_parameters_as_used() {
    AnalyzerTest::new(indoc! {
      "
      set positional-arguments := true

      graph log:
        ./bin/graph $1
      "
    })
    .run();
  }

  #[test]
  fn positional_arguments_shebang_marks_all_as_used() {
    AnalyzerTest::new(indoc! {
      r"
      [positional-arguments]
      run *args:
        #!/usr/bin/env -S deno run
        console.log(Deno.args)
      "
    })
    .run();
  }

  #[test]
  fn same_name_recipes_track_parameter_usage_separately() {
    AnalyzerTest::new(indoc! {
      "
      set allow-duplicate-recipes

      foo bar:
        echo {{bar}}

      foo bar:
        echo
      "
    })
    .warning("Parameter `bar` appears unused", lsp::Range::at(5, 4, 5, 7))
    .run();
  }

  #[test]
  fn warn_for_unused_non_exported_recipe_parameters() {
    AnalyzerTest::new(indoc! {
      "
      foo bar:
        echo foo
      "
    })
    .warning("Parameter `bar` appears unused", lsp::Range::at(0, 4, 0, 7))
    .run();

    AnalyzerTest::new(indoc! {
      "
      foo $bar:
        echo foo
      "
    })
    .run();

    AnalyzerTest::new(indoc! {
      "
      set export := false

      foo bar:
        echo foo
      "
    })
    .warning("Parameter `bar` appears unused", lsp::Range::at(2, 4, 2, 7))
    .run();

    AnalyzerTest::new(indoc! {
      "
      set export

      foo bar:
        echo foo
      "
    })
    .run();
  }
}
