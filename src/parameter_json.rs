use super::*;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub(crate) struct ParameterJson {
  pub(crate) default_value: Option<String>,
  pub(crate) name: String,
}

impl From<Parameter> for ParameterJson {
  fn from(parameter: Parameter) -> Self {
    ParameterJson {
      name: parameter.name,
      default_value: parameter.default_value,
    }
  }
}
