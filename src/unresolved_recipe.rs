use super::*;

pub(crate) enum UnresolvedRecipe {
  Module(TextNode),
  Recipe { suggestion: Option<Namepath> },
}
