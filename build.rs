use {
  cc::Build,
  std::{env, path::Path},
};

fn main() {
  let src = Path::new("vendor/tree-sitter-just-src");

  println!("cargo:rerun-if-changed={}", src.display());

  let mut build = Build::new();

  if let Ok(headers) = env::var("DEP_TREE_SITTER_LANGUAGE_WASM_HEADERS") {
    build.include(headers);
  }

  build
    .include(src)
    .warnings(false)
    .file(src.join("parser.c"))
    .file(src.join("scanner.c"))
    .compile("tree-sitter-just");
}
