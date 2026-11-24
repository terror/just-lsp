use super::*;

#[derive(Debug, Default)]
pub(crate) struct Bindings {
  identifiers: Vec<BoundIdentifier>,
  index: HashMap<(usize, usize), usize>,
  pub(crate) recipe_identifiers: HashMap<String, HashSet<String>>,
}

impl Bindings {
  fn bind(
    &mut self,
    document: &Document,
    identifier: &Node,
    binding: Binding,
    expression: bool,
  ) {
    let key = (identifier.start_byte(), identifier.end_byte());

    if self.index.contains_key(&key) {
      return;
    }

    self.index.insert(key, self.identifiers.len());

    self.identifiers.push(BoundIdentifier {
      binding,
      declaration: Self::is_declaration(identifier),
      expression,
      identifier: TextNode {
        range: document.get_range(identifier),
        value: document.get_node_text(identifier),
      },
    });
  }

  fn builtin(matches: impl Fn(&Builtin) -> bool) -> Option<Binding> {
    BUILTINS.iter().position(matches).map(Binding::Builtin)
  }

  fn comment_lines<'a>(
    view: &ModuleView,
    document: &Document,
    recipe_node: &'a Node,
  ) -> Vec<Node<'a>> {
    if !view.setting_enabled("ignore-comments") {
      return Vec::new();
    }

    let range = document.get_range(recipe_node);

    let script = view
      .recipes()
      .iter()
      .find(|recipe| recipe.uri == document.uri && recipe.range == range)
      .is_none_or(|recipe| {
        recipe.runs_as_script(view.setting_enabled("default-script"))
      });

    if script {
      return Vec::new();
    }

    recipe_node
      .find_all("recipe_body > recipe_line")
      .into_iter()
      .scan(false, |continued, line| {
        let comment = !*continued
          && line.child(0).is_some_and(|child| {
            child.kind() == "text"
              && document.get_node_text(&child).starts_with('#')
          });

        *continued = !comment && document.get_node_text(&line).ends_with('\\');

        Some(comment.then_some(line))
      })
      .flatten()
      .collect()
  }

  fn expression(
    &mut self,
    view: &ModuleView,
    document: &Document,
    globals: &[String],
    identifier: Node,
    recipe: Option<&str>,
    locals: &[(String, Binding)],
  ) {
    if identifier.is_missing() {
      return;
    }

    let name = document.get_node_text(&identifier);

    if let Some(recipe) = recipe {
      self
        .recipe_identifiers
        .entry(recipe.to_owned())
        .or_default()
        .insert(name.clone());
    }

    let binding = locals
      .iter()
      .find(|(local, _)| *local == name)
      .map(|(_, binding)| binding.clone())
      .or_else(|| {
        view
          .find_variable(&name)
          .map(|_| Binding::Variable(name.clone()))
      })
      .or_else(|| {
        Self::builtin(|builtin| {
          matches!(builtin, Builtin::Constant { name: constant, .. } if *constant == name)
        })
      })
      .unwrap_or_else(|| Binding::Unresolved {
        suggestion: name.find_suggestion(
          locals
            .iter()
            .map(|(local, _)| local.as_str())
            .chain(globals.iter().map(String::as_str)),
        ),
      });

    self.bind(document, &identifier, binding, true);
  }

  fn function(
    &mut self,
    view: &ModuleView,
    document: &Document,
    globals: &[String],
    function_node: Node,
  ) {
    let function = document.get_range(&function_node);

    let mut locals = Vec::new();

    for parameter_node in function_node
      .child_by_field_name("parameters")
      .iter()
      .flat_map(|parameters| parameters.find_all("^identifier"))
    {
      let name = document.get_node_text(&parameter_node);

      let binding = Binding::FunctionParameter {
        function,
        name: name.clone(),
      };

      self.bind(document, &parameter_node, binding.clone(), false);

      locals.push((name, binding));
    }

    if let Some(body_node) = function_node.child_by_field_name("body") {
      for identifier in document.get_expression_identifiers(&body_node) {
        self.expression(view, document, globals, identifier, None, &locals);
      }
    }
  }

  pub(crate) fn get(&self, identifier: &Node) -> Option<&BoundIdentifier> {
    self
      .index
      .get(&(identifier.start_byte(), identifier.end_byte()))
      .map(|index| &self.identifiers[*index])
  }

  pub(crate) fn identifiers(&self) -> &[BoundIdentifier] {
    &self.identifiers
  }

  fn is_declaration(identifier: &Node) -> bool {
    identifier.parent().is_some_and(|parent| {
      matches!(
        parent.kind(),
        "assignment"
          | "function_definition"
          | "function_parameters"
          | "parameter"
          | "recipe_header"
      )
    })
  }

  pub(crate) fn new(view: &ModuleView, document: &Document) -> Self {
    let globals = view
      .variables()
      .iter()
      .map(|variable| variable.name.value.clone())
      .chain(BUILTINS.iter().filter_map(|builtin| match builtin {
        Builtin::Constant { name, .. } => Some((*name).to_owned()),
        _ => None,
      }))
      .collect::<Vec<_>>();

    let mut bindings = Self::default();

    let root = document.tree.root_node();

    for recipe_node in root.find_all("recipe") {
      bindings.recipe(view, document, &globals, recipe_node);
    }

    for function_node in root.find_all("function_definition") {
      bindings.function(view, document, &globals, function_node);
    }

    for identifier in document.get_expression_identifiers(&root) {
      if identifier.has_any_parent(&["function_definition", "recipe"]) {
        continue;
      }

      bindings.expression(view, document, &globals, identifier, None, &[]);
    }

    for identifier in root.find_all("identifier") {
      bindings.reference(view, document, identifier);
    }

    bindings
  }

  fn recipe(
    &mut self,
    view: &ModuleView,
    document: &Document,
    globals: &[String],
    recipe_node: Node,
  ) {
    let Some(name_node) = recipe_node.find("recipe_header > identifier") else {
      return;
    };

    let name = document.get_node_text(&name_node);

    self.recipe_identifiers.entry(name.clone()).or_default();

    self.bind(
      document,
      &name_node,
      Binding::Recipe(vec![name.clone()]),
      false,
    );

    let recipe = document.get_range(&recipe_node);

    let mut locals = Vec::new();

    if let Some(parameters_node) =
      recipe_node.find("recipe_header > parameters")
    {
      for parameter_node in
        parameters_node.find_all("^parameter, ^variadic_parameter")
      {
        let parameter_node = if parameter_node.kind() == "variadic_parameter" {
          parameter_node.find("parameter")
        } else {
          Some(parameter_node)
        };

        let Some(parameter_node) = parameter_node else {
          continue;
        };

        if let Some(default_node) =
          parameter_node.child_by_field_name("default")
        {
          for identifier in document.get_expression_identifiers(&default_node) {
            self.expression(
              view,
              document,
              globals,
              identifier,
              Some(&name),
              &locals,
            );
          }
        }

        if let Some(parameter_name_node) =
          parameter_node.child_by_field_name("name")
        {
          let parameter = document.get_node_text(&parameter_name_node);

          let binding = Binding::Parameter {
            name: parameter.clone(),
            recipe,
          };

          self.bind(document, &parameter_name_node, binding.clone(), false);

          locals.push((parameter, binding));
        }
      }
    }

    let comments = Self::comment_lines(view, document, &recipe_node);

    for identifier in document.get_expression_identifiers(&recipe_node) {
      if identifier.has_any_parent(&["parameter", "variadic_parameter"])
        || comments.iter().any(|comment| {
          comment.byte_range().contains(&identifier.start_byte())
        })
      {
        continue;
      }

      self.expression(
        view,
        document,
        globals,
        identifier,
        Some(&name),
        &locals,
      );
    }
  }

  fn reference(
    &mut self,
    view: &ModuleView,
    document: &Document,
    identifier: Node,
  ) {
    if identifier.is_missing()
      || self.get(&identifier).is_some()
      || document.is_attribute_keyword(&identifier)
    {
      return;
    }

    let Some(parent) = identifier.parent() else {
      return;
    };

    let name = document.get_node_text(&identifier);

    let binding = match parent.kind() {
      "alias" | "dependency" | "dependency_expression" | "recipe_header" => {
        Some(Binding::Recipe(vec![name]))
      }
      "assignment" => Some(Binding::Variable(name)),
      "attribute" => Self::builtin(
        |builtin| matches!(builtin, Builtin::Attribute { name: attribute, .. } if *attribute == name),
      ),
      "function_call" | "assert_expression" => {
        if view.find_function(&name).is_some() {
          Some(Binding::Function(name))
        } else {
          Self::builtin(|builtin| {
            matches!(
              builtin,
              Builtin::Function { name: function, aliases, .. }
                if *function == name || aliases.contains(&name.as_str())
            )
          })
        }
      }
      "function_definition" => Some(Binding::Function(name)),
      "module_path" => {
        let segments = parent.find_all("^identifier");

        (segments.last() == Some(&identifier)).then(|| {
          Binding::Recipe(
            segments
              .iter()
              .map(|segment| document.get_node_text(segment))
              .collect(),
          )
        })
      }
      "setting" => Self::builtin(
        |builtin| matches!(builtin, Builtin::Setting { name: setting, .. } if *setting == name),
      ),
      _ => None,
    };

    if let Some(binding) = binding {
      self.bind(document, &identifier, binding, false);
    }
  }
}
