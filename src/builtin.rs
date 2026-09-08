use super::*;

#[derive(Debug)]
pub enum Builtin<'a> {
  Attribute {
    name: &'a str,
    kind: AttributeKind,
    description: &'a str,
    targets: &'a [AttributeTarget],
  },
  Constant {
    name: &'a str,
    description: &'a str,
  },
  Function {
    name: &'a str,
    aliases: &'a [&'a str],
    signature: FunctionSignature<'a>,
    description: &'a str,
    deprecated: Option<Deprecation<'a>>,
  },
  Setting {
    name: &'a str,
    kind: SettingKind,
    description: &'a str,
    deprecated: Option<Deprecation<'a>>,
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
          self.description(),
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
          self.description(),
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
      Self::Setting {
        name, deprecated, ..
      } => {
        let deprecated = deprecated.is_some();

        vec![lsp::CompletionItem {
          label: name.to_string(),
          kind: Some(lsp::CompletionItemKind::PROPERTY),
          documentation: Some(lsp::Documentation::MarkupContent(
            self.description(),
          )),
          deprecated: deprecated.then_some(true),
          insert_text: Some(name.to_string()),
          insert_text_format: Some(lsp::InsertTextFormat::PLAIN_TEXT),
          sort_text: Some(format!("z{name}")),
          tags: deprecated.then(|| vec![lsp::CompletionItemTag::DEPRECATED]),
          ..Default::default()
        }]
      }
    }
  }

  #[must_use]
  pub fn description(&self) -> lsp::MarkupContent {
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
    let Self::Function {
      signature,
      deprecated,
      ..
    } = self
    else {
      unreachable!();
    };

    let deprecated = deprecated.is_some();

    lsp::CompletionItem {
      label: name.to_string(),
      kind: Some(lsp::CompletionItemKind::FUNCTION),
      documentation: Some(lsp::Documentation::MarkupContent(
        self.description(),
      )),
      deprecated: deprecated.then_some(true),
      insert_text: Some(signature.snippet(name)),
      insert_text_format: Some(lsp::InsertTextFormat::SNIPPET),
      sort_text: Some(format!("z{name}")),
      tags: deprecated.then(|| vec![lsp::CompletionItemTag::DEPRECATED]),
      ..Default::default()
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn deprecated_function_completion_item_is_marked_deprecated() {
    let item = Builtin::Function {
      name: "foo",
      aliases: &[],
      signature: FunctionSignature(&[]),
      description: "",
      deprecated: Some(Deprecation::Replacement("bar")),
    }
    .completion_items()
    .into_iter()
    .next()
    .unwrap();

    assert_eq!(item.deprecated, Some(true));
    assert_eq!(item.tags, Some(vec![lsp::CompletionItemTag::DEPRECATED]));
  }

  #[test]
  fn function_alias_uses_alias_snippet() {
    #[track_caller]
    fn case(name: &str, parameters: &[FunctionParameter<'_>], arguments: &str) {
      let items = Builtin::Function {
        name,
        aliases: &["foo"],
        signature: FunctionSignature(parameters),
        description: "bar",
        deprecated: None,
      }
      .completion_items();

      assert_eq!(
        items,
        [name, "foo"].map(|name| lsp::CompletionItem {
          label: name.into(),
          kind: Some(lsp::CompletionItemKind::FUNCTION),
          documentation: Some(lsp::Documentation::MarkupContent(
            lsp::MarkupContent {
              kind: lsp::MarkupKind::Markdown,
              value: "bar".into(),
            },
          )),
          insert_text: Some(format!("{name}({arguments})")),
          insert_text_format: Some(lsp::InsertTextFormat::SNIPPET),
          sort_text: Some(format!("z{name}")),
          ..Default::default()
        }),
      );
    }

    case("bar", &[], "");

    case(
      "parent_directory",
      &[FunctionParameter::Required("path", Some("string"))],
      "${1:path:string}",
    );
  }

  #[test]
  fn function_completion_snippet_uses_signature() {
    let items = Builtin::Function {
      name: "foo",
      aliases: &[],
      signature: FunctionSignature(&[
        FunctionParameter::Required("value", None),
        FunctionParameter::Required("separator", Some("string")),
        FunctionParameter::Optional("default", Some("string")),
        FunctionParameter::Variadic("rest", Some("string")),
      ]),
      description: "",
      deprecated: None,
    }
    .completion_items();

    assert_eq!(
      items,
      vec![lsp::CompletionItem {
        label: "foo".into(),
        kind: Some(lsp::CompletionItemKind::FUNCTION),
        documentation: Some(lsp::Documentation::MarkupContent(
          lsp::MarkupContent {
            kind: lsp::MarkupKind::Markdown,
            value: String::new()
          },
        )),
        insert_text: Some(
          "foo(${1:value}, ${2:separator:string}${3:, default:string}${4:, rest:string...})"
            .into(),
        ),
        insert_text_format: Some(lsp::InsertTextFormat::SNIPPET),
        sort_text: Some("zfoo".into()),
        ..Default::default()
      }],
    );
  }
}
