use super::*;

pub(crate) static SETTINGS: [Setting<'_>; 18] = [
  Setting {
    name: "allow-duplicate-recipes",
    kind: SettingKind::Boolean,
  },
  Setting {
    name: "allow-duplicate-variables",
    kind: SettingKind::Boolean,
  },
  Setting {
    name: "dotenv-filename",
    kind: SettingKind::String,
  },
  Setting {
    name: "dotenv-load",
    kind: SettingKind::Boolean,
  },
  Setting {
    name: "dotenv-path",
    kind: SettingKind::String,
  },
  Setting {
    name: "dotenv-required",
    kind: SettingKind::Boolean,
  },
  Setting {
    name: "export",
    kind: SettingKind::Boolean,
  },
  Setting {
    name: "fallback",
    kind: SettingKind::Boolean,
  },
  Setting {
    name: "ignore-comments",
    kind: SettingKind::Boolean,
  },
  Setting {
    name: "positional-arguments",
    kind: SettingKind::Boolean,
  },
  Setting {
    name: "quiet",
    kind: SettingKind::Boolean,
  },
  Setting {
    name: "script-interpreter",
    kind: SettingKind::Array,
  },
  Setting {
    name: "shell",
    kind: SettingKind::Array,
  },
  Setting {
    name: "tempdir",
    kind: SettingKind::String,
  },
  Setting {
    name: "unstable",
    kind: SettingKind::Boolean,
  },
  Setting {
    name: "windows-powershell",
    kind: SettingKind::Boolean,
  },
  Setting {
    name: "windows-shell",
    kind: SettingKind::Array,
  },
  Setting {
    name: "working-directory",
    kind: SettingKind::String,
  },
];
