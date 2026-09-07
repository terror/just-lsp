use {super::*, analyze::Analyze};

mod analyze;

#[derive(Parser)]
pub(crate) enum Subcommand {
  Analyze(Analyze),
}

impl Subcommand {
  fn find_justfile() -> Result<PathBuf> {
    let mut current_dir = env::current_dir()?;

    loop {
      let mut candidates = BTreeSet::new();

      for entry in fs::read_dir(&current_dir)? {
        let entry = entry?;

        let name = entry.file_name();

        let Some(name) = name.to_str() else {
          continue;
        };

        if ["justfile", ".justfile"]
          .iter()
          .any(|candidate| name.eq_ignore_ascii_case(candidate))
        {
          candidates.insert(entry.path());
        }
      }

      match candidates.len() {
        0 => {}
        1 => return Ok(candidates.pop_first().unwrap()),
        _ => bail!(
          "multiple candidate justfiles found in `{}`",
          current_dir.display()
        ),
      }

      if !current_dir.pop() {
        bail!(
          "could not find `justfile` in current directory or any parent directory"
        );
      }
    }
  }

  pub(crate) fn run(self) -> Result {
    match self {
      Self::Analyze(analyze) => analyze.run(),
    }
  }
}
