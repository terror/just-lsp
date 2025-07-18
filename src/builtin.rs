use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum AttributeTarget {
  Alias,
  Module,
  Recipe,
}

impl Display for AttributeTarget {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    write!(
      f,
      "{}",
      match self {
        AttributeTarget::Alias => "alias",
        AttributeTarget::Module => "module",
        AttributeTarget::Recipe => "recipe",
      }
    )
  }
}

impl AttributeTarget {
  pub(crate) fn is_valid_for(&self, target: AttributeTarget) -> bool {
    *self == target
  }
}

#[derive(Debug)]
pub(crate) enum Builtin<'a> {
  Attribute {
    name: &'a str,
    description: &'a str,
    version: &'a str,
    targets: &'a [AttributeTarget],
    parameters: Option<&'a str>,
  },
  Constant {
    name: &'a str,
    description: &'a str,
    value: &'a str,
  },
  Function {
    name: &'a str,
    signature: &'a str,
    description: &'a str,
    required_args: usize,
    accepts_variadic: bool,
  },
  Setting {
    name: &'a str,
    kind: SettingKind,
    description: &'a str,
    default: &'a str,
  },
}

impl Builtin<'_> {
  pub(crate) fn completion_item(&self) -> lsp::CompletionItem {
    match self {
      Self::Attribute { name, .. } => lsp::CompletionItem {
        label: name.to_string(),
        kind: Some(lsp::CompletionItemKind::CONSTANT),
        documentation: Some(lsp::Documentation::MarkupContent(
          self.documentation(),
        )),
        insert_text: Some(format!("[{name}]")),
        insert_text_format: Some(lsp::InsertTextFormat::PLAIN_TEXT),
        sort_text: Some(format!("z{name}")),
        ..Default::default()
      },
      Self::Constant { name, .. } => lsp::CompletionItem {
        label: name.to_string(),
        kind: Some(lsp::CompletionItemKind::CONSTANT),
        documentation: Some(lsp::Documentation::MarkupContent(
          self.documentation(),
        )),
        insert_text: Some(name.to_string()),
        insert_text_format: Some(lsp::InsertTextFormat::PLAIN_TEXT),
        sort_text: Some(format!("z{name}")),
        ..Default::default()
      },
      Self::Function { name, .. } => {
        let snippet = match *name {
          "absolute_path" => format!("{name}(${{1:path:string}})"),
          "append" => {
            format!("{name}(${{1:suffix:string}}, ${{2:s:string}})")
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
          | "home_directory" => format!("{name}()"),
          "blake3" => format!("{name}(${{1:string:string}})"),
          "blake3_file" => format!("{name}(${{1:path:string}})"),
          "canonicalize" => format!("{name}(${{1:path:string}})"),
          "capitalize" => format!("{name}(${{1:s:string}})"),
          "choose" => {
            format!("{name}(${{1:n:string}}, ${{2:alphabet:string}})")
          }
          "clean" => format!("{name}(${{1:path:string}})"),
          "datetime" => format!("{name}(${{1:format:string}})"),
          "datetime_utc" => format!("{name}(${{1:format:string}})"),
          "encode_uri_component" => format!("{name}(${{1:s:string}})"),
          "env" => {
            format!("{name}(${{1:key:string}}${{2:, default:string}})")
          }
          "error" => format!("{name}(${{1:message:string}})"),
          "extension" => format!("{name}(${{1:path:string}})"),
          "file_name" => format!("{name}(${{1:path:string}})"),
          "file_stem" => format!("{name}(${{1:path:string}})"),
          "join" => format!(
            "{name}(${{1:a:string}}, ${{2:b:string}}${{3:, more:string...}})",
          ),
          "kebabcase" => format!("{name}(${{1:s:string}})"),
          "lowercase" => format!("{name}(${{1:s:string}})"),
          "lowercamelcase" => format!("{name}(${{1:s:string}})"),
          "parent_directory" => format!("{name}(${{1:path:string}})"),
          "path_exists" => format!("{name}(${{1:path:string}})"),
          "prepend" => {
            format!("{name}(${{1:prefix:string}}, ${{2:s:string}})")
          }
          "quote" => format!("{name}(${{1:s:string}})"),
          "read" => format!("{name}(${{1:path:string}})"),
          "replace" => format!(
            "{name}(${{1:s:string}}, ${{2:from:string}}, ${{3:to:string}})",
          ),
          "replace_regex" => format!(
          "{name}(${{1:s:string}}, ${{2:regex:string}}, ${{3:replacement:string}})",
        ),
          "require" => format!("{name}(${{1:name:string}})"),
          "semver_matches" => {
            format!(
              "{name}(${{1:version:string}}, ${{2:requirement:string}})",
            )
          }
          "sha256" => format!("{name}(${{1:string:string}})"),
          "sha256_file" => format!("{name}(${{1:path:string}})"),
          "shell" => {
            format!("{name}(${{1:command:string}}${{2:, args:string...}})")
          }
          "shoutykebabcase" => format!("{name}(${{1:s:string}})"),
          "shoutysnakecase" => format!("{name}(${{1:s:string}})"),
          "snakecase" => format!("{name}(${{1:s:string}})"),
          "style" => format!("{name}(${{1:name:string}})"),
          "titlecase" => format!("{name}(${{1:s:string}})"),
          "trim" => format!("{name}(${{1:s:string}})"),
          "trim_end" => format!("{name}(${{1:s:string}})"),
          "trim_end_match" => {
            format!("{name}(${{1:s:string}}, ${{2:substring:string}})")
          }
          "trim_end_matches" => {
            format!("{name}(${{1:s:string}}, ${{2:substring:string}})")
          }
          "trim_start" => format!("{name}(${{1:s:string}})"),
          "trim_start_match" => {
            format!("{name}(${{1:s:string}}, ${{2:substring:string}})")
          }
          "trim_start_matches" => {
            format!("{name}(${{1:s:string}}, ${{2:substring:string}})")
          }
          "uppercamelcase" => format!("{name}(${{1:s:string}})"),
          "uppercase" => format!("{name}(${{1:s:string}})"),
          "which" => format!("{name}(${{1:name:string}})"),
          "without_extension" => format!("{name}(${{1:path:string}})"),
          _ => format!("{name}(${{1:}})"),
        };

        lsp::CompletionItem {
          label: name.to_string(),
          kind: Some(lsp::CompletionItemKind::FUNCTION),
          documentation: Some(lsp::Documentation::MarkupContent(
            self.documentation(),
          )),
          insert_text: Some(snippet),
          insert_text_format: Some(lsp::InsertTextFormat::SNIPPET),
          sort_text: Some(format!("z{name}")),
          ..Default::default()
        }
      }
      Self::Setting { name, .. } => lsp::CompletionItem {
        label: name.to_string(),
        kind: Some(lsp::CompletionItemKind::PROPERTY),
        documentation: Some(lsp::Documentation::MarkupContent(
          self.documentation(),
        )),
        insert_text: Some(format!("set {name} := ")),
        insert_text_format: Some(lsp::InsertTextFormat::PLAIN_TEXT),
        sort_text: Some(format!("z{name}")),
        ..Default::default()
      },
    }
  }

  pub(crate) fn documentation(&self) -> lsp::MarkupContent {
    match self {
      Self::Attribute {
        name,
        description,
        version,
        targets,
        parameters,
      } => {
        let mut documentation =
          format!("**Attribute**: [{name}]\n{description}");

        if let Some(params) = parameters {
          documentation.push_str(&format!("\n**Syntax**: [{name}({params})]"));
        }

        documentation.push_str(&format!("\n**Introduced in**: {version}"));

        let targets = targets
          .iter()
          .map(|target| target.to_string())
          .collect::<Vec<String>>();

        documentation
          .push_str(&format!("\n**Target(s)**: {}", targets.join(", ")));

        lsp::MarkupContent {
          kind: lsp::MarkupKind::Markdown,
          value: documentation,
        }
      }
      Self::Constant {
        description, value, ..
      } => lsp::MarkupContent {
        kind: lsp::MarkupKind::Markdown,
        value: format!("{description}\n{value}"),
      },
      Self::Function {
        name,
        signature,
        description,
        ..
      } => {
        let example = match *name {
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
          "hell" => "shell(\"echo $1\", \"hello\") => \"hello\"",
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

        let mut documentation = String::new();

        documentation.push_str(description);

        documentation.push_str(format!("\n```\n{signature}\n```").as_str());

        if !example.is_empty() {
          documentation.push_str("\n**Examples:**\n```\n");
          documentation.push_str(example);
          documentation.push_str("\n```");
        }

        lsp::MarkupContent {
          kind: lsp::MarkupKind::Markdown,
          value: documentation,
        }
      }
      Self::Setting {
        name,
        kind,
        description,
        default,
        ..
      } => {
        let mut documentation = format!("**Setting**: {name}\n{description}");

        documentation.push_str(&format!("\n**Type**: {kind}"));
        documentation.push_str(&format!("\n**Default**: {default}"));

        lsp::MarkupContent {
          kind: lsp::MarkupKind::Markdown,
          value: documentation,
        }
      }
    }
  }
}
