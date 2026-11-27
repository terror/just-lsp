use super::*;

pub(crate) struct List<T: Display, I: Iterator<Item = T> + Clone> {
  conjunction: &'static str,
  values: I,
}

impl<T: Display, I: Iterator<Item = T> + Clone> List<T, I> {
  pub(crate) fn and<II: IntoIterator<Item = T, IntoIter = I>>(
    values: II,
  ) -> Self {
    Self {
      conjunction: "and",
      values: values.into_iter(),
    }
  }

  pub(crate) fn and_ticked<II: IntoIterator<Item = T, IntoIter = I>>(
    values: II,
  ) -> List<Enclosure<T>, impl Iterator<Item = Enclosure<T>> + Clone> {
    List::and(values.into_iter().map(Enclosure::tick))
  }

  pub(crate) fn or<II: IntoIterator<Item = T, IntoIter = I>>(
    values: II,
  ) -> Self {
    Self {
      conjunction: "or",
      values: values.into_iter(),
    }
  }
}

impl<T: Display, I: Iterator<Item = T> + Clone> Display for List<T, I> {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    let mut values = self.values.clone().fuse();

    if let Some(first) = values.next() {
      write!(f, "{first}")?;
    } else {
      return Ok(());
    }

    let second = values.next();

    if second.is_none() {
      return Ok(());
    }

    let third = values.next();

    if let (Some(second), None) = (second.as_ref(), third.as_ref()) {
      write!(f, " {} {second}", self.conjunction)?;
      return Ok(());
    }

    let mut current = second;
    let mut next = third;

    loop {
      match (current, next) {
        (Some(c), Some(n)) => {
          write!(f, ", {c}")?;
          current = Some(n);
          next = values.next();
        }
        (Some(c), None) => {
          write!(f, ", {} {c}", self.conjunction)?;
          return Ok(());
        }
        _ => unreachable!("Iterator was fused, but returned Some after None"),
      }
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn and() {
    assert_eq!(List::and(&[1]).to_string(), "1");
    assert_eq!(List::and(&[1, 2]).to_string(), "1 and 2");
    assert_eq!(List::and(&[1, 2, 3]).to_string(), "1, 2, and 3");
    assert_eq!(List::and(&[1, 2, 3, 4]).to_string(), "1, 2, 3, and 4");
  }

  #[test]
  fn and_ticked() {
    assert_eq!(List::and_ticked(&[1]).to_string(), "`1`");
    assert_eq!(List::and_ticked(&[1, 2]).to_string(), "`1` and `2`");
    assert_eq!(
      List::and_ticked(&[1, 2, 3]).to_string(),
      "`1`, `2`, and `3`"
    );
  }
}
