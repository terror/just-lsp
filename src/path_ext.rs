use super::*;

pub(crate) trait PathExt {
  fn clean(&self) -> PathBuf;
  fn is_justfile(&self) -> bool;
}

impl PathExt for Path {
  fn clean(&self) -> PathBuf {
    let mut components = Vec::new();

    for component in self.components() {
      match component {
        Component::CurDir => {}
        Component::ParentDir => match components.last() {
          Some(Component::Normal(_)) => {
            components.pop();
          }
          Some(Component::RootDir) => {}
          _ => components.push(component),
        },
        _ => components.push(component),
      }
    }

    if components.is_empty() {
      components.push(Component::CurDir);
    }

    components.into_iter().collect()
  }

  fn is_justfile(&self) -> bool {
    let Some(name) = self.file_name().and_then(OsStr::to_str) else {
      return false;
    };

    ["justfile", ".justfile"]
      .iter()
      .any(|candidate| name.eq_ignore_ascii_case(candidate))
  }
}

#[cfg(test)]
mod tests {
  use {super::*, pretty_assertions::assert_eq, std::path::MAIN_SEPARATOR_STR};

  #[test]
  fn clean() {
    #[track_caller]
    fn case(path: &str, expected: &str) {
      let expected = expected.replace('/', MAIN_SEPARATOR_STR);

      assert_eq!(
        Path::new(path).clean().as_os_str(),
        Path::new(&expected).as_os_str(),
      );
    }

    case("", ".");
    case("././.", ".");
    case("foo", "foo");
    case("./foo/./bar//", "foo/bar");
    case("foo/..", ".");
    case("./..", "..");
    case("../../foo/..", "../..");
    case("foo/../../bar", "../bar");
    case("/", "/");
    case("/../..", "/");
    case("/foo/../../bar", "/bar");

    #[cfg(unix)]
    {
      case("//foo//bar//", "/foo/bar");
      case(r"foo\bar/..", ".");
    }

    #[cfg(windows)]
    {
      case("C:..", "C:..");
      case(r"C:foo\..", "C:");
      case(r"C:foo\..\..\bar", r"C:..\bar");
      case(r"C:\foo\..\..\bar", r"C:\bar");
      case(r"\\foo\bar\..", r"\\foo\bar\");
      case(r"\\?\C:\foo\.\bar\..\..\..", r"\\?\C:\");
      case(r"\\?\UNC\foo\bar\baz\..\..", r"\\?\UNC\foo\bar\");
      case(r"\\?\foo\bar\..\..", r"\\?\foo\");
      case(r"\\.\foo\bar\..\..", r"\\.\foo\");
    }
  }
}
