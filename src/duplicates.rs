use super::*;

#[derive(Debug)]
pub(crate) struct Duplicates {
  pub(crate) aliases: Vec<usize>,
  pub(crate) functions: Vec<usize>,
  pub(crate) recipes: Vec<usize>,
  pub(crate) settings: Vec<usize>,
  pub(crate) unexports: Vec<usize>,
  pub(crate) variables: Vec<usize>,
}

impl Duplicates {
  fn find<T>(
    declarations: &[Located<T>],
    name: impl Fn(&T) -> &TextNode,
    attributes: impl Fn(&T) -> &[Attribute],
  ) -> Vec<usize> {
    let mut names = HashMap::<&str, Vec<usize>>::new();

    for (index, declaration) in declarations.iter().enumerate() {
      names
        .entry(name(declaration).value.as_str())
        .or_default()
        .push(index);
    }

    let mut duplicates = Vec::new();

    for indices in names.values().filter(|indices| indices.len() > 1) {
      let candidates = indices
        .iter()
        .map(|index| {
          let groups =
            GroupSet::from_attributes(attributes(&declarations[*index]));

          (*index, groups)
        })
        .collect::<Vec<_>>();

      for (current, current_groups) in &candidates {
        let conflicts = candidates.iter().any(|(previous, previous_groups)| {
          Self::precedes(declarations, *previous, *current)
            && previous_groups.conflicts_with(current_groups)
        });

        if conflicts {
          duplicates.push(*current);
        }
      }
    }

    duplicates.sort_unstable();

    duplicates
  }

  pub(crate) fn new(symbols: &SymbolTable) -> Self {
    Self {
      aliases: Self::find(
        &symbols.aliases,
        |alias| &alias.name,
        |alias| &alias.attributes,
      ),
      functions: Self::find(
        &symbols.functions,
        |function| &function.name,
        |function| &function.attributes,
      ),
      recipes: Self::find(
        &symbols.recipes,
        |recipe| &recipe.name,
        |recipe| &recipe.attributes,
      ),
      settings: Self::find(
        &symbols.settings,
        |setting| &setting.name,
        |setting| &setting.attributes,
      ),
      unexports: Self::find(
        &symbols.unexports,
        |unexport| &unexport.name,
        |unexport| &unexport.attributes,
      ),
      variables: Self::find(
        &symbols.variables,
        |variable| &variable.name,
        |variable| &variable.attributes,
      ),
    }
  }

  fn precedes<T>(
    declarations: &[Located<T>],
    previous: usize,
    current: usize,
  ) -> bool {
    previous != current
      && (previous < current
        || declarations[previous].uri != declarations[current].uri)
  }
}
