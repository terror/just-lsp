use super::*;

#[derive(Debug)]
pub enum Builtin<'a> {
  Attribute {
    name: &'a str,
    description: &'a str,
    targets: &'a [AttributeTarget],
    min_args: usize,
    max_args: Option<usize>,
  },
  Constant {
    name: &'a str,
    description: &'a str,
  },
  Function {
    name: &'a str,
    aliases: &'a [&'a str],
    kind: FunctionKind,
    description: &'a str,
    deprecated: Option<&'a str>,
  },
  Setting {
    name: &'a str,
    kind: SettingKind,
    description: &'a str,
    deprecated: Option<&'a str>,
  },
}

impl Builtin<'_> {
  #[must_use]
  pub fn completion_items(&self) -> Vec<lsp::CompletionItem> {
    match self {
      Self::Attribute { name, .. } => vec![lsp::CompletionItem {
        label: name.to_string(),
        kind: Some(lsp::CompletionItemKind::KEYWORD),
        documentation: Some(lsp::Documentation::MarkupContent(
          self.documentation(),
        )),
        insert_text: Some(name.to_string()),
        insert_text_format: Some(lsp::InsertTextFormat::PLAIN_TEXT),
        sort_text: Some(format!("z{name}")),
        ..Default::default()
      }],
      Self::Constant { name, .. } => vec![lsp::CompletionItem {
        label: name.to_string(),
        kind: Some(lsp::CompletionItemKind::CONSTANT),
        documentation: Some(lsp::Documentation::MarkupContent(
          self.documentation(),
        )),
        insert_text: Some(name.to_string()),
        insert_text_format: Some(lsp::InsertTextFormat::PLAIN_TEXT),
        sort_text: Some(format!("z{name}")),
        ..Default::default()
      }],
      Self::Function { name, aliases, .. } => once(*name)
        .chain(aliases.iter().copied())
        .map(|name| self.function_completion_item(name))
        .collect(),
      Self::Setting { name, .. } => vec![lsp::CompletionItem {
        label: name.to_string(),
        kind: Some(lsp::CompletionItemKind::PROPERTY),
        documentation: Some(lsp::Documentation::MarkupContent(
          self.documentation(),
        )),
        insert_text: Some(name.to_string()),
        insert_text_format: Some(lsp::InsertTextFormat::PLAIN_TEXT),
        sort_text: Some(format!("z{name}")),
        ..Default::default()
      }],
    }
  }

  #[must_use]
  pub fn documentation(&self) -> lsp::MarkupContent {
    lsp::MarkupContent {
      kind: lsp::MarkupKind::Markdown,
      value: (match self {
        Self::Attribute { description, .. }
        | Self::Constant { description, .. }
        | Self::Function { description, .. }
        | Self::Setting { description, .. } => description,
      })
      .to_string(),
    }
  }

  fn function_completion_item(&self, name: &str) -> lsp::CompletionItem {
    let snippet = match name {
      "absolute_path" | "blake3_file" | "canonicalize" | "clean"
      | "extension" | "file_name" | "file_stem" | "parent_dir"
      | "parent_directory" | "path_exists" | "read" | "sha256_file"
      | "without_extension" => {
        format!("{name}(${{1:path:string}})")
      }
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
      | "invocation_dir"
      | "invocation_dir_native"
      | "justfile"
      | "justfile_directory"
      | "justfile_dir"
      | "source_file"
      | "source_directory"
      | "source_dir"
      | "just_executable"
      | "just_pid"
      | "uuid"
      | "runtime_directory"
      | "runtime_dir"
      | "cache_directory"
      | "cache_dir"
      | "config_directory"
      | "config_dir"
      | "config_local_directory"
      | "config_local_dir"
      | "data_directory"
      | "data_dir"
      | "data_local_directory"
      | "data_local_dir"
      | "executable_directory"
      | "executable_dir"
      | "home_directory"
      | "home_dir" => format!("{name}()"),
      "blake3" | "sha256" => format!("{name}(${{1:string:string}})"),
      "capitalize"
      | "encode_uri_component"
      | "kebabcase"
      | "lowercase"
      | "lowercamelcase"
      | "quote"
      | "shoutykebabcase"
      | "shoutysnakecase"
      | "snakecase"
      | "titlecase"
      | "trim"
      | "trim_end"
      | "trim_start"
      | "uppercamelcase"
      | "uppercase" => format!("{name}(${{1:s:string}})"),
      "choose" => {
        format!("{name}(${{1:n:string}}, ${{2:alphabet:string}})")
      }
      "datetime" | "datetime_utc" => {
        format!("{name}(${{1:format:string}})")
      }
      "env" => {
        format!("{name}(${{1:key:string}}${{2:, default:string}})")
      }
      "env_var" => format!("{name}(${{1:key:string}})"),
      "env_var_or_default" => {
        format!("{name}(${{1:key:string}}, ${{2:default:string}})")
      }
      "error" => format!("{name}(${{1:message:string}})"),
      "join" => format!(
        "{name}(${{1:a:string}}, ${{2:b:string}}${{3:, more:string...}})",
      ),
      "prepend" => {
        format!("{name}(${{1:prefix:string}}, ${{2:s:string}})")
      }
      "replace" => {
        format!("{name}(${{1:s:string}}, ${{2:from:string}}, ${{3:to:string}})")
      }
      "replace_regex" => {
        format!(
          "{name}(${{1:s:string}}, ${{2:regex:string}}, ${{3:replacement:string}})"
        )
      }
      "require" | "style" | "which" => {
        format!("{name}(${{1:name:string}})")
      }
      "semver_matches" => {
        format!("{name}(${{1:version:string}}, ${{2:requirement:string}})")
      }
      "shell" => {
        format!("{name}(${{1:command:string}}${{2:, args:string...}})")
      }
      "trim_end_match" | "trim_end_matches" | "trim_start_match"
      | "trim_start_matches" => {
        format!("{name}(${{1:s:string}}, ${{2:substring:string}})")
      }
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
}
