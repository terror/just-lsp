use super::*;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
enum Definition<'a> {
  Function(&'a str),
  Variable(&'a str),
}

impl<'a> Definition<'a> {
  fn edges(
    view: &'a ModuleView,
    document: &Document,
    bindings: &'a Bindings,
  ) -> Vec<(Self, Self)> {
    let declarations = view
      .resolved_variables()
      .filter(|variable| variable.uri == document.uri)
      .map(|variable| (Self::Variable(&variable.name.value), variable.range))
      .chain(
        view
          .resolved_functions()
          .filter(|function| function.uri == document.uri)
          .map(|function| {
            (Self::Function(&function.name.value), function.range)
          }),
      )
      .collect::<Vec<_>>();

    bindings
      .identifiers()
      .iter()
      .filter_map(|bound| {
        let reference = Self::reference(bound)?;

        let (declaration, _) = declarations
          .iter()
          .find(|(_, range)| range.overlaps(bound.identifier.range))?;

        Some((*declaration, reference))
      })
      .collect()
  }

  fn reference(bound: &'a BoundIdentifier) -> Option<Self> {
    match &bound.binding {
      Binding::Function(name) if !bound.declaration => {
        Some(Self::Function(name))
      }
      Binding::Variable(name) if bound.expression => Some(Self::Variable(name)),
      _ => None,
    }
  }

  fn variable(self) -> Option<&'a str> {
    match self {
      Self::Function(_) => None,
      Self::Variable(name) => Some(name),
    }
  }
}

define_rule! {
  VariableDependencyCycleRule {
    id: "variable-dependency-cycle",
    message: "circular variable dependency",
    run(context) {
      let view = context.view();

      let bindings = view
        .documents()
        .map(|document| (document, view.bindings(document)))
        .collect::<Vec<_>>();

      let graph = bindings
        .iter()
        .flat_map(|(document, bindings)| Definition::edges(view, document, bindings))
        .collect::<Graph<_>>();

      view
        .resolved_variables()
        .filter(|variable| variable.uri == context.document().uri)
        .filter_map(|variable| {
          let name = &variable.name.value;

          let cycle = graph
            .cycle(Definition::Variable(name))?
            .into_iter()
            .filter_map(Definition::variable)
            .collect::<Vec<_>>();

          let message = if cycle.len() == 2 {
            format!("Variable `{name}` is defined in terms of itself")
          } else {
            format!(
              "Variable `{name}` depends on its own value: `{}`",
              cycle.join(" -> ")
            )
          };

          Some(Diagnostic::error(message, variable.name.range))
        })
        .collect()
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn variable_dependency_cycle() {
    #[track_caller]
    fn case(content: &str, errors: &[(&'static str, lsp::Range)]) {
      errors
        .iter()
        .fold(AnalyzerTest::new(content), |test, (message, range)| {
          test.error(message, *range)
        })
        .run();
    }

    case(
      "a := b\nb := a\n",
      &[
        (
          "Variable `a` depends on its own value: `a -> b -> a`",
          lsp::Range::at(0, 0, 0, 1),
        ),
        (
          "Variable `b` depends on its own value: `b -> a -> b`",
          lsp::Range::at(1, 0, 1, 1),
        ),
      ],
    );

    case(
      "set unstable\na := f()\nf() := a\n",
      &[(
        "Variable `a` is defined in terms of itself",
        lsp::Range::at(1, 0, 1, 1),
      )],
    );
  }
}
