use super::*;

define_rule! {
  InvalidUnicodeEscapeRule {
    id: "invalid-unicode-escape",
    message: "invalid unicode escape",
    run(context) {
      let document = context.document();

      context
        .tree()
        .root_node()
        .find_all("escape_sequence")
        .into_iter()
        .filter_map(|escape| {
          let text = document.get_node_text(&escape);

          let hex = text.strip_prefix("\\u{")?.strip_suffix('}')?;

          let codepoint = u32::from_str_radix(hex, 16).ok()?;

          char::from_u32(codepoint).is_none().then(|| {
            Diagnostic::error(
              format!(
                "unicode escape sequence value `{hex}` greater than maximum valid code point `10FFFF`"
              ),
              document.get_range(&escape),
            )
          })
        })
        .collect()
    }
  }
}
