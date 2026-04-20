use super::*;

pub trait RangeExt {
  fn overlaps(&self, other: lsp::Range) -> bool;
}

impl RangeExt for lsp::Range {
  fn overlaps(&self, other: lsp::Range) -> bool {
    self.start <= other.end && other.start <= self.end
  }
}

#[cfg(test)]
mod tests {
  use {super::*, pretty_assertions::assert_eq};

  fn range(
    start_line: u32,
    start_character: u32,
    end_line: u32,
    end_character: u32,
  ) -> lsp::Range {
    lsp::Range {
      start: lsp::Position {
        line: start_line,
        character: start_character,
      },
      end: lsp::Position {
        line: end_line,
        character: end_character,
      },
    }
  }

  #[test]
  fn overlaps() {
    #[track_caller]
    fn case(a: lsp::Range, b: lsp::Range, expected: bool) {
      assert_eq!(a.overlaps(b), expected);
      assert_eq!(b.overlaps(a), expected);
    }

    case(range(0, 0, 0, 5), range(0, 3, 0, 8), true);
    case(range(0, 0, 0, 5), range(0, 5, 0, 8), true);
    case(range(0, 0, 0, 5), range(0, 6, 0, 8), false);
    case(range(0, 0, 0, 5), range(0, 2, 0, 2), true);
    case(range(0, 0, 1, 0), range(0, 2, 0, 10), true);
    case(range(0, 0, 0, 5), range(1, 0, 1, 5), false);
  }
}
