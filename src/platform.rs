use super::*;

pub(crate) fn attributes_enabled(attributes: &[Attribute]) -> bool {
  let mut platform_attribute = false;

  for attribute in attributes {
    let enabled = match attribute.name.value.as_str() {
      "android" => cfg!(target_os = "android"),
      "dragonfly" => cfg!(target_os = "dragonfly"),
      "freebsd" => cfg!(target_os = "freebsd"),
      "linux" => cfg!(target_os = "linux"),
      "macos" => cfg!(target_os = "macos"),
      "netbsd" => cfg!(target_os = "netbsd"),
      "openbsd" => cfg!(target_os = "openbsd"),
      "unix" => cfg!(unix),
      "windows" => cfg!(windows),
      _ => continue,
    };

    platform_attribute = true;

    if enabled {
      return true;
    }
  }

  !platform_attribute
}

#[cfg(test)]
mod tests {
  use super::*;

  fn attribute(name: &str) -> Attribute {
    Attribute {
      name: TextNode {
        value: name.into(),
        ..Default::default()
      },
      ..Default::default()
    }
  }

  fn case(name: &str, expected: bool) {
    assert_eq!(attributes_enabled(&[attribute(name)]), expected);
  }

  #[test]
  fn platform_attributes_match_host() {
    case("android", cfg!(target_os = "android"));
    case("dragonfly", cfg!(target_os = "dragonfly"));
    case("freebsd", cfg!(target_os = "freebsd"));
    case("linux", cfg!(target_os = "linux"));
    case("macos", cfg!(target_os = "macos"));
    case("netbsd", cfg!(target_os = "netbsd"));
    case("openbsd", cfg!(target_os = "openbsd"));
    case("unix", cfg!(unix));
    case("windows", cfg!(windows));
  }

  #[test]
  fn platform_attributes_use_or_semantics() {
    assert_eq!(
      attributes_enabled(&[attribute("unix"), attribute("windows")]),
      cfg!(unix) || cfg!(windows),
    );
  }

  #[test]
  fn unrecognized_attributes_do_not_disable_items() {
    assert!(attributes_enabled(&[attribute("private")]));
  }
}
