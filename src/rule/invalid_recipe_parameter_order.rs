use super::*;

define_rule! {
  InvalidRecipeParameterOrderRule {
    id: "invalid-recipe-parameter-order",
    title: "invalid recipe parameter order",
    severity: Error,
  }
}

impl InvalidRecipeParameterOrderRule {
  fn run(context: &RuleContext<'_>) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();

    for recipe in context.local_declarations(context.recipes()) {
      let mut passed_default = false;

      for param in recipe
        .parameters
        .iter()
        .filter(|param| !recipe.is_option(&param.name))
      {
        let has_default = param.default_value.is_some();

        if passed_default
          && !has_default
          && param.kind != ParameterKind::Variadic(VariadicType::ZeroOrMore)
        {
          diagnostics.push(Diagnostic::new(
            format!(
              "Required parameter `{}` follows a parameter with a default value",
              param.name
            ),
            param.range,
          ));
        }

        if has_default {
          passed_default = true;
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
  fn recipe_parameters_defaults_all() {
    AnalyzerTest::new(indoc! {
      "
      recipe_with_defaults arg1=\"first\" arg2=\"second\":
        echo \"{{arg1}} {{arg2}}\"
      "
    })
    .run();
  }

  #[test]
  fn recipe_parameters_order() {
    AnalyzerTest::new(indoc! {
      "
      recipe_with_param_order arg1=\"default\" arg2:
        echo \"{{arg1}} {{arg2}}\"
      "
    })
    .error(
      "Required parameter `arg2` follows a parameter with a default value",
      lsp::Range::at(0, 39, 0, 43),
    )
    .run();
  }

  #[test]
  fn recipe_parameters_order_options() {
    #[track_caller]
    fn case(attribute: &str) {
      AnalyzerTest::new(&format!(
        "{attribute}\nfoo bar='bar' baz:\n  echo {{{{bar}}}} {{{{baz}}}}\n"
      ))
      .run();
    }

    case("[arg('bar', long)]");
    case("[arg('baz', short='b')]");
  }

  #[test]
  fn recipe_parameters_valid() {
    AnalyzerTest::new(indoc! {
      "
      valid_recipe arg1 arg2=\"default\":
        echo \"{{arg1}} {{arg2}}\"
      "
    })
    .run();
  }

  #[test]
  fn recipe_parameters_variadic() {
    AnalyzerTest::new(indoc! {
      "
      foo bar='bar' +baz:
        echo {{bar}} {{baz}}
      "
    })
    .error(
      "Required parameter `baz` follows a parameter with a default value",
      lsp::Range::at(0, 14, 0, 18),
    )
    .run();

    AnalyzerTest::new(indoc! {
      "
      foo bar='bar' *baz:
        echo {{bar}} {{baz}}
      "
    })
    .run();
  }
}
