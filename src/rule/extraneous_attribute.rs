use super::*;

define_rule! {
  ExtraneousAttributeRule {
    id: "extraneous-attribute",
    message: "extraneous attribute",
    run(context) {
      let document = context.document();

      let mut diagnostics = Vec::new();

      for attribute in context.tree().root_node().find_all("attribute") {
        let Some(next) = attribute.next_sibling() else {
          continue;
        };

        if next.kind() != "comment"
          && next.start_position().row <= attribute.end_position().row
        {
          continue;
        }

        let attributes = successors(Some(attribute), |attribute| {
          attribute.prev_sibling().filter(|previous| {
            previous.kind() == "attribute"
              && previous.end_position().row == attribute.start_position().row
          })
        })
        .collect::<Vec<_>>();

        let count = attributes
          .iter()
          .map(|attribute| attribute.find_all("^identifier").len())
          .sum::<usize>();

        let first = attributes.last().unwrap_or(&attribute);

        diagnostics.push(Diagnostic::error(
          format!("extraneous {}", Count("attribute", count)),
          lsp::Range {
            start: document.get_range(first).start,
            end: document.get_range(&attribute).end,
          },
        ));
      }

      diagnostics
    }
  }
}
