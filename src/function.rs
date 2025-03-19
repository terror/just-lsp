#[derive(Debug)]
pub(crate) struct Function<'a> {
  pub(crate) name: &'a str,
  pub(crate) signature: &'a str,
  pub(crate) description: &'a str,
  pub(crate) required_args: usize,
  pub(crate) accepts_variadic: bool,
}

impl Function<'_> {
  pub(crate) fn documentation(&self) -> String {
    let example = match self.name {
      "absolute_path" => "absolute_path(\"./foo\") => \"/path/to/foo\"",
      "append" => "append(\"/src\", \"foo bar\") => \"foo/src bar/src\"",
      "arch" => "arch() => \"x86_64\"",
      "blake3" => "blake3(\"hello\") => \"a1744eeb6b921a9193df*...\"",
      "canonicalize" => "canonicalize(\"../foo/.\") => \"/path/to/foo\"",
      "capitalize" => "capitalize(\"hello\") => \"Hello\"",
      "choose" => "choose(\"5\", \"abcdef\") => \"bcafe\"",
      "clean" => "clean(\"foo//bar/../baz\") => \"foo/baz\"",
      "datetime" => "datetime(\"%Y-%m-%d\") => \"2023-07-14\"",
      "encode_uri_component" => "encode_uri_component(\"a+b\") => \"a%2Bb\"",
      "env" => "env(\"HOME\") => \"/home/user\"\nenv(\"MISSING\", \"default\") => \"default\"",
      "error" => "error(\"Something went wrong\") => *aborts execution*",
      "extension" => "extension(\"foo.txt\") => \"txt\"",
      "file_name" => "file_name(\"/path/to/foo.txt\") => \"foo.txt\"",
      "file_stem" => "file_stem(\"/path/to/foo.txt\") => \"foo\"",
      "home_directory" => "home_directory() => \"/home/user\"",
      "invocation_directory" => "invocation_directory() => \"/path/to/current/dir\"",
      "is_dependency" => "is_dependency() => \"false\"",
      "join" => "join(\"foo\", \"bar\", \"baz\") => \"foo/bar/baz\"",
      "just_executable" => "just_executable() => \"/usr/bin/just\"",
      "just_pid" => "just_pid() => \"12345\"",
      "justfile" => "justfile() => \"/path/to/justfile\"",
      "justfile_directory" => "justfile_directory() => \"/path/to\"",
      "kebabcase" => "kebabcase(\"HelloWorld\") => \"hello-world\"",
      "lowercase" => "lowercase(\"Hello\") => \"hello\"",
      "lowercamelcase" => "lowercamelcase(\"hello_world\") => \"helloWorld\"",
      "num_cpus" => "num_cpus() => 8",
      "os" => "os() => \"linux\"",
      "os_family" => "os_family() => \"unix\"",
      "parent_directory" => "parent_directory(\"/path/to/foo.txt\") => \"/path/to\"",
      "path_exists" => "path_exists(\"/etc/passwd\") => \"true\"",
      "prepend" => "prepend(\"src/\", \"foo bar\") => \"src/foo src/bar\"",
      "quote" => "quote(\"hello 'world'\") => \"'hello \\'world\\''\"",
      "read" => "read(\"foo.txt\") => \"contents of foo.txt\"",
      "replace" => "replace(\"hello\", \"l\", \"x\") => \"hexxo\"",
      "replace_regex" => "replace_regex(\"hello\", \"[aeiou]\", \"X\") => \"hXllX\"",
      "require" => "require(\"bash\") => \"/bin/bash\"",
      "semver_matches" => "semver_matches(\"1.2.3\", \">1.0.0\") => \"true\"",
      "sha256" => "sha256(\"hello\") => \"2cf24dba5fb0a30e*...\"",
      "shell" => "shell(\"echo $1\", \"hello\") => \"hello\"",
      "style" => "style(\"error\") => \"\\e[31m\"",
      "trim" => "trim(\"  hello  \") => \"hello\"",
      "trim_end" => "trim_end(\"hello  \") => \"hello\"",
      "trim_start" => "trim_start(\"  hello\") => \"hello\"",
      "uppercamelcase" => "uppercamelcase(\"hello_world\") => \"HelloWorld\"",
      "uppercase" => "uppercase(\"hello\") => \"HELLO\"",
      "uuid" => "uuid() => \"f81d4fae-7dec-11d0-a765-00a0c91e6bf6\"",
      "which" => "which(\"bash\") => \"/bin/bash\"\nwhich(\"nonexistent\") => \"\"",
      "without_extension" => "without_extension(\"/path/to/foo.txt\") => \"/path/to/foo\"",
      _ => "",
    };

    let mut doc = String::new();

    doc.push_str(self.description);

    if !example.is_empty() {
      doc.push_str("\n\n**Examples:**\n```\n\n");
      doc.push_str(example);
      doc.push_str("\n```");
    }

    doc
  }

  pub(crate) fn snippet(&self) -> String {
    match self.name {
      "absolute_path" => format!("{}(${{1:path:string}})", self.name),
      "append" => {
        format!("{}(${{1:suffix:string}}, ${{2:s:string}})", self.name)
      }
      "arch"
      | "num_cpus"
      | "os"
      | "os_family"
      | "is_dependency"
      | "invocation_directory"
      | "invocation_directory_native"
      | "justfile"
      | "justfile_directory"
      | "source_file"
      | "source_directory"
      | "just_executable"
      | "just_pid"
      | "uuid"
      | "cache_directory"
      | "config_directory"
      | "config_local_directory"
      | "data_directory"
      | "data_local_directory"
      | "executable_directory"
      | "home_directory" => format!("{}()", self.name),
      "blake3" => format!("{}(${{1:string:string}})", self.name),
      "blake3_file" => format!("{}(${{1:path:string}})", self.name),
      "canonicalize" => format!("{}(${{1:path:string}})", self.name),
      "capitalize" => format!("{}(${{1:s:string}})", self.name),
      "choose" => {
        format!("{}(${{1:n:string}}, ${{2:alphabet:string}})", self.name)
      }
      "clean" => format!("{}(${{1:path:string}})", self.name),
      "datetime" => format!("{}(${{1:format:string}})", self.name),
      "datetime_utc" => format!("{}(${{1:format:string}})", self.name),
      "encode_uri_component" => format!("{}(${{1:s:string}})", self.name),
      "env" => {
        format!("{}(${{1:key:string}}${{2:, default:string}})", self.name)
      }
      "error" => format!("{}(${{1:message:string}})", self.name),
      "extension" => format!("{}(${{1:path:string}})", self.name),
      "file_name" => format!("{}(${{1:path:string}})", self.name),
      "file_stem" => format!("{}(${{1:path:string}})", self.name),
      "join" => format!(
        "{}(${{1:a:string}}, ${{2:b:string}}${{3:, more:string...}})",
        self.name
      ),
      "kebabcase" => format!("{}(${{1:s:string}})", self.name),
      "lowercase" => format!("{}(${{1:s:string}})", self.name),
      "lowercamelcase" => format!("{}(${{1:s:string}})", self.name),
      "parent_directory" => format!("{}(${{1:path:string}})", self.name),
      "path_exists" => format!("{}(${{1:path:string}})", self.name),
      "prepend" => {
        format!("{}(${{1:prefix:string}}, ${{2:s:string}})", self.name)
      }
      "quote" => format!("{}(${{1:s:string}})", self.name),
      "read" => format!("{}(${{1:path:string}})", self.name),
      "replace" => format!(
        "{}(${{1:s:string}}, ${{2:from:string}}, ${{3:to:string}})",
        self.name
      ),
      "replace_regex" => format!(
        "{}(${{1:s:string}}, ${{2:regex:string}}, ${{3:replacement:string}})",
        self.name
      ),
      "require" => format!("{}(${{1:name:string}})", self.name),
      "semver_matches" => format!(
        "{}(${{1:version:string}}, ${{2:requirement:string}})",
        self.name
      ),
      "sha256" => format!("{}(${{1:string:string}})", self.name),
      "sha256_file" => format!("{}(${{1:path:string}})", self.name),
      "shell" => format!(
        "{}(${{1:command:string}}${{2:, args:string...}})",
        self.name
      ),
      "shoutykebabcase" => format!("{}(${{1:s:string}})", self.name),
      "shoutysnakecase" => format!("{}(${{1:s:string}})", self.name),
      "snakecase" => format!("{}(${{1:s:string}})", self.name),
      "style" => format!("{}(${{1:name:string}})", self.name),
      "titlecase" => format!("{}(${{1:s:string}})", self.name),
      "trim" => format!("{}(${{1:s:string}})", self.name),
      "trim_end" => format!("{}(${{1:s:string}})", self.name),
      "trim_end_match" => {
        format!("{}(${{1:s:string}}, ${{2:substring:string}})", self.name)
      }
      "trim_end_matches" => {
        format!("{}(${{1:s:string}}, ${{2:substring:string}})", self.name)
      }
      "trim_start" => format!("{}(${{1:s:string}})", self.name),
      "trim_start_match" => {
        format!("{}(${{1:s:string}}, ${{2:substring:string}})", self.name)
      }
      "trim_start_matches" => {
        format!("{}(${{1:s:string}}, ${{2:substring:string}})", self.name)
      }
      "uppercamelcase" => format!("{}(${{1:s:string}})", self.name),
      "uppercase" => format!("{}(${{1:s:string}})", self.name),
      "which" => format!("{}(${{1:name:string}})", self.name),
      "without_extension" => format!("{}(${{1:path:string}})", self.name),
      _ => format!("{}(${{1:}})", self.name),
    }
  }
}
