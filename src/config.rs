use super::*;

#[derive(Clone, Debug, Default, Deserialize)]
pub struct Config {
  #[serde(default)]
  pub rules: HashMap<String, RuleConfig>,
}

impl Config {
  #[must_use]
  pub fn rule_config(&self, id: &str) -> RuleConfig {
    self.rules.get(id).cloned().unwrap_or_default()
  }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub enum RuleLevel {
  Error,
  Hint,
  #[serde(alias = "info")]
  Information,
  Off,
  Warning,
}

impl From<RuleLevel> for lsp::DiagnosticSeverity {
  fn from(value: RuleLevel) -> Self {
    match value {
      RuleLevel::Error => Self::ERROR,
      RuleLevel::Hint | RuleLevel::Off => Self::HINT,
      RuleLevel::Information => Self::INFORMATION,
      RuleLevel::Warning => Self::WARNING,
    }
  }
}

#[derive(Clone, Debug, Deserialize)]
#[serde(untagged)]
pub enum RuleConfig {
  Level(RuleLevel),
  Settings {
    #[serde(default)]
    level: Option<RuleLevel>,
  },
}

impl Default for RuleConfig {
  fn default() -> Self {
    Self::Settings { level: None }
  }
}

impl RuleConfig {
  #[must_use]
  pub fn level(&self) -> Option<RuleLevel> {
    match self {
      RuleConfig::Level(level) => Some(*level),
      RuleConfig::Settings { level } => *level,
    }
  }

  #[must_use]
  pub fn severity(
    &self,
    default: lsp::DiagnosticSeverity,
  ) -> Option<lsp::DiagnosticSeverity> {
    match self.level() {
      None => Some(default),
      Some(RuleLevel::Off) => None,
      Some(level) => Some(level.into()),
    }
  }
}

#[cfg(test)]
mod tests {
  use {super::*, serde_json::json};

  #[test]
  fn parses_rule_config_from_string() {
    let config: Config = serde_json::from_value(json!({
      "rules": {
        "foo": "warning"
      }
    }))
    .unwrap();

    assert_eq!(config.rule_config("foo").level(), Some(RuleLevel::Warning));
  }

  #[test]
  fn parses_rule_config_from_table() {
    let config: Config = serde_json::from_value(json!({
      "rules": {
        "foo": { "level": "hint" }
      }
    }))
    .unwrap();

    assert_eq!(config.rule_config("foo").level(), Some(RuleLevel::Hint));
  }

  #[test]
  fn missing_rule_config_returns_default() {
    let config = Config::default();

    assert_eq!(config.rule_config("foo").level(), None);
  }

  #[test]
  fn off_level_produces_none_severity() {
    let config: Config = serde_json::from_value(json!({
      "rules": {
        "foo": "off"
      }
    }))
    .unwrap();

    assert_eq!(
      config
        .rule_config("foo")
        .severity(lsp::DiagnosticSeverity::ERROR),
      None
    );
  }

  #[test]
  fn rule_config_overrides_default_severity() {
    let config: Config = serde_json::from_value(json!({
      "rules": {
        "foo": "warning"
      }
    }))
    .unwrap();

    assert_eq!(
      config
        .rule_config("foo")
        .severity(lsp::DiagnosticSeverity::ERROR),
      Some(lsp::DiagnosticSeverity::WARNING)
    );
  }

  #[test]
  fn missing_rule_config_uses_default_severity() {
    let config = Config::default();

    assert_eq!(
      config
        .rule_config("foo")
        .severity(lsp::DiagnosticSeverity::ERROR),
      Some(lsp::DiagnosticSeverity::ERROR)
    );
  }

  #[test]
  fn info_alias_parses() {
    let config: Config = serde_json::from_value(json!({
      "rules": {
        "foo": "info"
      }
    }))
    .unwrap();

    assert_eq!(
      config.rule_config("foo").level(),
      Some(RuleLevel::Information)
    );
  }
}
