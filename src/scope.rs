use super::*;

pub(crate) struct Scope {
  current_recipe: Option<String>,
  frames: Vec<HashSet<String>>,
  pub(crate) recipe_identifier_usage: HashMap<String, HashSet<String>>,
  pub(crate) unresolved_identifiers: Vec<(String, lsp::Range)>,
  pub(crate) variable_usage: HashMap<String, bool>,
}

impl Scope {
  pub(crate) fn analyze(context: &RuleContext<'_>) -> Self {
    let mut scope = Self::new(context);

    let Some(tree) = context.tree() else {
      return scope;
    };

    let root = tree.root_node();

    for node in root.find_all("recipe") {
      scope.walk_recipe(context.document(), node);
    }

    for node in root.find_all("function_definition") {
      scope.walk_function(context.document(), node);
    }

    for identifier in root.find_all("expression > value > identifier") {
      if identifier.get_parent("recipe").is_none()
        && identifier.get_parent("function_definition").is_none()
      {
        scope.record(context.document(), identifier);
      }
    }

    scope
  }

  fn define(&mut self, name: &str) {
    if let Some(frame) = self.frames.last_mut() {
      frame.insert(name.to_owned());
    }
  }

  fn enter(&mut self) {
    self.frames.push(HashSet::new());
  }

  fn exit(&mut self) {
    self.frames.pop();
  }

  fn new(context: &RuleContext<'_>) -> Self {
    Self {
      current_recipe: None,
      frames: vec![
        context
          .variable_and_builtin_names()
          .iter()
          .cloned()
          .chain(context.user_function_names().iter().cloned())
          .collect(),
      ],
      recipe_identifier_usage: context
        .recipes()
        .iter()
        .map(|recipe| (recipe.name.value.clone(), HashSet::new()))
        .collect(),
      unresolved_identifiers: Vec::new(),
      variable_usage: context
        .variables()
        .iter()
        .map(|variable| (variable.name.value.clone(), false))
        .collect::<HashMap<_, _>>(),
    }
  }

  /// Resolve an identifier against the scope stack.
  ///
  /// Recipe identifier usage is recorded unconditionally before resolution,
  /// so parameter self-references like `foo foo` still count as usage for the
  /// `unused-parameters` rule.
  fn record(&mut self, document: &Document, identifier: Node<'_>) {
    let name = document.get_node_text(&identifier);

    if let Some(recipe_name) = &self.current_recipe {
      self
        .recipe_identifier_usage
        .entry(recipe_name.clone())
        .or_default()
        .insert(name.clone());
    }

    if self.frames[1..].iter().any(|frame| frame.contains(&name)) {
      return;
    }

    if let Some(used) = self.variable_usage.get_mut(&name) {
      *used = true;
      return;
    }

    if self.frames[0].contains(&name) {
      return;
    }

    self
      .unresolved_identifiers
      .push((name, identifier.get_range(document)));
  }

  /// Enter a function definition scope and record its body.
  ///
  /// Parameters are all defined before processing the body, since `just`
  /// function parameters have no default values and cannot reference each
  /// other.
  fn walk_function(&mut self, document: &Document, function_node: Node<'_>) {
    self.enter();

    if let Some(parameters_node) =
      function_node.child_by_field_name("parameters")
    {
      for parameter_node in parameters_node.find_all("^identifier") {
        self.define(&document.get_node_text(&parameter_node));
      }
    }

    if let Some(body_node) = function_node.child_by_field_name("body") {
      for identifier in body_node.find_all("value > identifier") {
        self.record(document, identifier);
      }
    }

    self.exit();
  }

  /// Enter a recipe scope and record its parameters and body.
  ///
  /// Parameters are defined one at a time: each default value is recorded
  /// before defining that parameter, so `b=a` resolves `a` against earlier
  /// parameters and globals but not `b` itself. Body identifiers inside
  /// parameter defaults are skipped in the final expression walk to avoid
  /// double-recording with the wrong scope.
  fn walk_recipe(&mut self, document: &Document, recipe_node: Node<'_>) {
    let Some(name_node) = recipe_node.find("recipe_header > identifier") else {
      return;
    };

    self.current_recipe = Some(document.get_node_text(&name_node));
    self.enter();

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
          for identifier in default_node.find_all("^identifier") {
            self.record(document, identifier);
          }

          for identifier in
            default_node.find_all("expression > value > identifier")
          {
            self.record(document, identifier);
          }
        }

        if let Some(name_node) = parameter_node.child_by_field_name("name") {
          self.define(&document.get_node_text(&name_node));
        }
      }
    }

    for identifier in recipe_node.find_all("expression > value > identifier") {
      if identifier.get_parent("parameter").is_some()
        || identifier.get_parent("variadic_parameter").is_some()
      {
        continue;
      }

      self.record(document, identifier);
    }

    self.exit();
    self.current_recipe = None;
  }
}
