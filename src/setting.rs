use super::*;

#[derive(Debug)]
pub(crate) enum SettingKind {
  Array,
  Boolean(bool),
  String,
}

impl Display for SettingKind {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    match self {
      SettingKind::Array => write!(f, "array"),
      SettingKind::Boolean(_) => write!(f, "boolean"),
      SettingKind::String => write!(f, "string"),
    }
  }
}

impl PartialEq for SettingKind {
  fn eq(&self, other: &Self) -> bool {
    matches!(
      (self, other),
      (SettingKind::Array, SettingKind::Array)
        | (SettingKind::Boolean(_), SettingKind::Boolean(_))
        | (SettingKind::String, SettingKind::String)
    )
  }
}

#[derive(Debug, PartialEq)]
pub(crate) struct Setting {
  pub(crate) kind: SettingKind,
  pub(crate) name: String,
  pub(crate) range: lsp::Range,
}

impl Setting {
  #[must_use]
  pub(crate) fn parse(text: &str, range: lsp::Range) -> Option<Self> {
    if !text.starts_with("set ") {
      return None;
    }

    let text = &text[4..];

    if !text.contains(":=") {
      let name = text.trim();

      if name.is_empty() {
        return None;
      }

      return Some(Setting {
        name: name.to_string(),
        kind: SettingKind::Boolean(true),
        range,
      });
    }

    let parts = text.split(":=").collect::<Vec<&str>>();

    if parts.len() != 2 {
      return None;
    }

    let name_part = parts[0].trim();

    if name_part.is_empty() {
      return None;
    }

    let value_part = parts[1].trim();

    if value_part.is_empty() {
      return None;
    }

    let kind = if value_part == "true" || value_part == "false" {
      SettingKind::Boolean(value_part == "true")
    } else if value_part.starts_with('[') && value_part.ends_with(']') {
      SettingKind::Array
    } else {
      SettingKind::String
    };

    Some(Setting {
      name: name_part.to_string(),
      kind,
      range,
    })
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  fn create_range(
    start_line: u32,
    start_char: u32,
    end_line: u32,
    end_char: u32,
  ) -> lsp::Range {
    lsp::Range {
      start: lsp::Position {
        line: start_line,
        character: start_char,
      },
      end: lsp::Position {
        line: end_line,
        character: end_char,
      },
    }
  }

  #[test]
  fn parse_boolean_with_value() {
    let input = "set foo := true";

    let range = create_range(0, 0, 0, 14);

    let setting = Setting::parse(input, range).unwrap();

    assert_eq!(
      setting,
      Setting {
        name: "foo".to_string(),
        kind: SettingKind::Boolean(true),
        range,
      }
    );
  }

  #[test]
  fn parse_boolean_without_value() {
    let input = "set export";

    let range = create_range(0, 0, 0, 10);

    let setting = Setting::parse(input, range).unwrap();

    assert_eq!(
      setting,
      Setting {
        name: "export".to_string(),
        kind: SettingKind::Boolean(true),
        range,
      }
    );
  }

  #[test]
  fn parse_array() {
    let input = "set shell := [\"zsh\", \"-cu\"]";

    let range = create_range(0, 0, 0, 27);

    let setting = Setting::parse(input, range).unwrap();

    assert_eq!(
      setting,
      Setting {
        name: "shell".to_string(),
        kind: SettingKind::Array,
        range,
      }
    );
  }

  #[test]
  fn parse_string() {
    let input = "set name := \"value\"";

    let range = create_range(0, 0, 0, 18);

    let setting = Setting::parse(input, range).unwrap();

    assert_eq!(
      setting,
      Setting {
        name: "name".to_string(),
        kind: SettingKind::String,
        range,
      }
    );
  }

  #[test]
  fn parse_number_as_string() {
    let input = "set bar := 42";

    let range = create_range(0, 0, 0, 13);

    let setting = Setting::parse(input, range).unwrap();

    assert_eq!(
      setting,
      Setting {
        name: "bar".to_string(),
        kind: SettingKind::String,
        range,
      }
    );
  }

  #[test]
  fn invalid_input() {
    let range = create_range(0, 0, 0, 7);

    assert_eq!(Setting::parse("invalid", range), None);
    assert_eq!(Setting::parse("set :=", range), None);
    assert_eq!(Setting::parse("set name :=", range), None);
    assert_eq!(Setting::parse("set name := := value", range), None);
  }
}
