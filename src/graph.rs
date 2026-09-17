use super::*;

#[derive(Debug)]
pub(super) struct Graph<'a> {
  components: HashMap<&'a str, usize>,
  edges: HashMap<&'a str, Vec<&'a str>>,
}

impl<'a> Graph<'a> {
  fn components(
    edges: &HashMap<&'a str, Vec<&'a str>>,
    reverse: &HashMap<&'a str, Vec<&'a str>>,
  ) -> HashMap<&'a str, usize> {
    let mut order = Vec::new();
    let mut stack = Vec::new();
    let mut visited = HashSet::new();

    for &node in edges.keys() {
      stack.push((node, false));

      while let Some((node, finished)) = stack.pop() {
        if finished {
          order.push(node);
        } else if visited.insert(node) {
          stack.push((node, true));
          stack.extend(edges[node].iter().map(|&node| (node, false)));
        }
      }
    }

    let mut components = HashMap::new();
    let mut stack = Vec::new();

    for (component, node) in order.into_iter().rev().enumerate() {
      stack.push(node);

      while let Some(node) = stack.pop() {
        if let Entry::Vacant(entry) = components.entry(node) {
          entry.insert(component);
          stack.extend(reverse[node].iter().copied());
        }
      }
    }

    components
  }

  pub(super) fn cycle(&self, start: &str) -> Option<Vec<&'a str>> {
    let (&start, dependencies) = self.edges.get_key_value(start)?;
    let component = self.components[start];

    let dependency = dependencies
      .iter()
      .copied()
      .find(|dependency| self.components[dependency] == component)?;

    let mut parents = HashMap::from([(dependency, None)]);
    let mut queue = VecDeque::from([dependency]);

    while let Some(node) = queue.pop_front() {
      if node == start {
        let mut cycle =
          successors(Some(start), |node| parents[node]).collect::<Vec<_>>();

        cycle.push(start);
        cycle.reverse();

        return Some(cycle);
      }

      for &dependency in &self.edges[node] {
        if self.components[dependency] == component
          && let Entry::Vacant(entry) = parents.entry(dependency)
        {
          entry.insert(Some(node));
          queue.push_back(dependency);
        }
      }
    }

    None
  }
}

impl<'a> FromIterator<(&'a str, &'a str)> for Graph<'a> {
  fn from_iter<T: IntoIterator<Item = (&'a str, &'a str)>>(iter: T) -> Self {
    let mut edges = HashMap::<_, Vec<_>>::new();
    let mut reverse = HashMap::<_, Vec<_>>::new();

    for (source, target) in iter {
      edges.entry(source).or_default().push(target);
      edges.entry(target).or_default();
      reverse.entry(target).or_default().push(source);
      reverse.entry(source).or_default();
    }

    Self {
      components: Self::components(&edges, &reverse),
      edges,
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn components() {
    let graph = [
      ("foo", "bar"),
      ("foo", "baz"),
      ("bar", "baz"),
      ("baz", "qux"),
      ("qux", "baz"),
    ]
    .into_iter()
    .collect::<Graph>();

    let mut components = HashMap::<_, BTreeSet<_>>::new();

    for (name, component) in graph.components {
      components.entry(component).or_default().insert(name);
    }

    assert_eq!(
      components.into_values().collect::<BTreeSet<_>>(),
      BTreeSet::from([
        BTreeSet::from(["foo"]),
        BTreeSet::from(["bar"]),
        BTreeSet::from(["baz", "qux"]),
      ])
    );
  }
}
