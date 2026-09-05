use {cc::Build, std::path::Path};

fn main() {
  let src = Path::new("vendor/tree-sitter-just-src");

  println!("cargo:rerun-if-changed={}", src.display());

  Build::new()
    .include(src)
    .warnings(false)
    .file(src.join("parser.c"))
    .file(src.join("scanner.c"))
    .compile("tree-sitter-just");
}
