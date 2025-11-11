use {
  cc::Build,
  std::path::{Path, PathBuf},
};

struct Parser<'a> {
  extra: Vec<&'a str>,
  name: &'a str,
  src: &'a str,
}

impl Parser<'_> {
  fn build(&self) {
    let path = PathBuf::from(self.src);

    let mut files = vec!["parser.c"];
    files.extend(self.extra.clone());

    let c = files
      .iter()
      .filter(|file| {
        Path::new(file)
          .extension()
          .is_some_and(|extension| extension.eq_ignore_ascii_case("c"))
      })
      .copied()
      .collect::<Vec<&str>>();

    let mut build = Build::new();
    build.include(&path).warnings(false);

    for file in &c {
      build.file(path.join(file));
    }

    build.compile(self.name);

    let cpp = files
      .iter()
      .filter(|file| {
        !Path::new(file)
          .extension()
          .is_some_and(|extension| extension.eq_ignore_ascii_case("c"))
      })
      .copied()
      .collect::<Vec<&str>>();

    if !cpp.is_empty() {
      let mut build = cc::Build::new();

      build
        .include(&path)
        .warnings(false)
        .cpp(true)
        .flag_if_supported("-Wno-implicit-fallthrough")
        .flag_if_supported("-Wno-unused-parameter")
        .flag_if_supported("-Wno-ignored-qualifiers")
        .flag_if_supported("-Wno-return-type");

      build.flag(if cfg!(windows) {
        "/std:c++14"
      } else {
        "--std=c++14"
      });

      for file in &cpp {
        build.file(path.join(file));
      }

      build.compile(&format!("{}-cpp", self.name));
    }
  }
}

fn main() {
  let parsers = vec![Parser {
    name: "tree-sitter-just",
    src: "../../vendor/tree-sitter-just-src",
    extra: vec!["scanner.c"],
  }];

  for parser in &parsers {
    println!("cargo:rerun-if-changed={}", parser.src);
  }

  parsers.iter().for_each(Parser::build);
}
