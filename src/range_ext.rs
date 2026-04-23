use super::*;

pub trait RangeExt {
  fn at(
    start_line: u32,
    start_character: u32,
    end_line: u32,
    end_character: u32,
  ) -> Self;

  fn overlaps(&self, other: lsp::Range) -> bool;
}

impl RangeExt for lsp::Range {
  fn at(
    start_line: u32,
    start_character: u32,
    end_line: u32,
    end_character: u32,
  ) -> Self {
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

  fn overlaps(&self, other: lsp::Range) -> bool {
    self.start <= other.end && other.start <= self.end
  }
}

#[cfg(test)]
mod tests {
  use {super::*, lsp::Range, pretty_assertions::assert_eq};

  #[test]
  fn overlaps() {
    #[track_caller]
    fn case(a: Range, b: Range, expected: bool) {
      assert_eq!(a.overlaps(b), expected);
      assert_eq!(b.overlaps(a), expected);
    }

    case(Range::at(0, 0, 0, 5), Range::at(0, 3, 0, 8), true);
    case(Range::at(0, 0, 0, 5), Range::at(0, 5, 0, 8), true);
    case(Range::at(0, 0, 0, 5), Range::at(0, 6, 0, 8), false);
    case(Range::at(0, 0, 0, 5), Range::at(0, 2, 0, 2), true);
    case(Range::at(0, 0, 1, 0), Range::at(0, 2, 0, 10), true);
    case(Range::at(0, 0, 0, 5), Range::at(1, 0, 1, 5), false);
  }
}
