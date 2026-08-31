pub(crate) fn suggest<'a>(
  input: &str,
  candidates: impl IntoIterator<Item = &'a str>,
) -> Option<String> {
  candidates
    .into_iter()
    .map(|candidate| (strsim::levenshtein(input, candidate), candidate))
    .filter(|(distance, _)| *distance < 3)
    .min_by(|(left_distance, left), (right_distance, right)| {
      left_distance
        .cmp(right_distance)
        .then_with(|| left.cmp(right))
    })
    .map(|(_, candidate)| candidate.to_owned())
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn finds_close_candidate() {
    assert_eq!(suggest("shel", ["shell", "export"]), Some("shell".into()));
  }

  #[test]
  fn ignores_distant_candidates() {
    assert_eq!(suggest("unknown", ["shell", "export"]), None);
  }

  #[test]
  fn resolves_ties_lexicographically() {
    assert_eq!(suggest("cat", ["bat", "car"]), Some("bat".into()));
  }
}
