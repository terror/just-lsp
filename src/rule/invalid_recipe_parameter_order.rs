use super::*;

define_rule! {
  InvalidRecipeParameterOrderRule {
    id: "invalid-recipe-parameter-order",
    message: "invalid recipe parameter order",
    run(context) {
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
            diagnostics.push(Diagnostic::error(
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
}
