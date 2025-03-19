use super::*;

pub(crate) const CONSTANTS: [Constant<'_>; 27] = [
  Constant {
    name: "HEX",
    description: "0123456789abcdef",
    value: "\"0123456789abcdef\"",
  },
  Constant {
    name: "HEXLOWER",
    description: "0123456789abcdef",
    value: "\"0123456789abcdef\"",
  },
  Constant {
    name: "HEXUPPER",
    description: "0123456789ABCDEF",
    value: "\"0123456789ABCDEF\"",
  },
  Constant {
    name: "CLEAR",
    description: "Clear screen",
    value: "\"\\ec\"",
  },
  Constant {
    name: "NORMAL",
    description: "Reset terminal style",
    value: "\"\\e[0m\"",
  },
  Constant {
    name: "BOLD",
    description: "Bold text",
    value: "\"\\e[1m\"",
  },
  Constant {
    name: "ITALIC",
    description: "Italic text",
    value: "\"\\e[3m\"",
  },
  Constant {
    name: "UNDERLINE",
    description: "Underlined text",
    value: "\"\\e[4m\"",
  },
  Constant {
    name: "INVERT",
    description: "Inverted colors",
    value: "\"\\e[7m\"",
  },
  Constant {
    name: "HIDE",
    description: "Hidden text",
    value: "\"\\e[8m\"",
  },
  Constant {
    name: "STRIKETHROUGH",
    description: "Strikethrough text",
    value: "\"\\e[9m\"",
  },
  Constant {
    name: "BLACK",
    description: "Black text",
    value: "\"\\e[30m\"",
  },
  Constant {
    name: "RED",
    description: "Red text",
    value: "\"\\e[31m\"",
  },
  Constant {
    name: "GREEN",
    description: "Green text",
    value: "\"\\e[32m\"",
  },
  Constant {
    name: "YELLOW",
    description: "Yellow text",
    value: "\"\\e[33m\"",
  },
  Constant {
    name: "BLUE",
    description: "Blue text",
    value: "\"\\e[34m\"",
  },
  Constant {
    name: "MAGENTA",
    description: "Magenta text",
    value: "\"\\e[35m\"",
  },
  Constant {
    name: "CYAN",
    description: "Cyan text",
    value: "\"\\e[36m\"",
  },
  Constant {
    name: "WHITE",
    description: "White text",
    value: "\"\\e[37m\"",
  },
  Constant {
    name: "BG_BLACK",
    description: "Black background",
    value: "\"\\e[40m\"",
  },
  Constant {
    name: "BG_RED",
    description: "Red background",
    value: "\"\\e[41m\"",
  },
  Constant {
    name: "BG_GREEN",
    description: "Green background",
    value: "\"\\e[42m\"",
  },
  Constant {
    name: "BG_YELLOW",
    description: "Yellow background",
    value: "\"\\e[43m\"",
  },
  Constant {
    name: "BG_BLUE",
    description: "Blue background",
    value: "\"\\e[44m\"",
  },
  Constant {
    name: "BG_MAGENTA",
    description: "Magenta background",
    value: "\"\\e[45m\"",
  },
  Constant {
    name: "BG_CYAN",
    description: "Cyan background",
    value: "\"\\e[46m\"",
  },
  Constant {
    name: "BG_WHITE",
    description: "White background",
    value: "\"\\e[47m\"",
  },
];

pub(crate) const FUNCTIONS: [Function<'_>; 69] = [
    Function {
        name: "absolute_path",
        signature: "absolute_path(path: string) -> string",
        description: "Get absolute path",
        required_args: 1,
        accepts_variadic: false
    },
    Function {
        name: "append",
        signature: "append(suffix: string, s: string) -> string",
        description: "Append suffix to strings",
        required_args: 2,
        accepts_variadic: false
    },
    Function {
        name: "arch",
        signature: "arch() -> string",
        description: "Instruction set architecture",
        required_args: 0,
        accepts_variadic: false
    },
    Function {
        name: "blake3",
        signature: "blake3(string: string) -> string",
        description: "Calculate BLAKE3 hash of string",
        required_args: 1,
        accepts_variadic: false
    },
    Function {
        name: "blake3_file",
        signature: "blake3_file(path: string) -> string",
        description: "Calculate BLAKE3 hash of file",
        required_args: 1,
        accepts_variadic: false
    },
    Function {
        name: "cache_directory",
        signature: "cache_directory() -> string",
        description: "User cache directory",
        required_args: 0,
        accepts_variadic: false
    },
    Function {
        name: "canonicalize",
        signature: "canonicalize(path: string) -> string",
        description: "Canonicalize path",
        required_args: 1,
        accepts_variadic: false
    },
    Function {
        name: "capitalize",
        signature: "capitalize(s: string) -> string",
        description: "Convert first character to uppercase",
        required_args: 1,
        accepts_variadic: false
    },
    Function {
        name: "choose",
        signature: "choose(n: string, alphabet: string) -> string",
        description: "Generate random string from alphabet",
        required_args: 2,
        accepts_variadic: false
    },
    Function {
        name: "clean",
        signature: "clean(path: string) -> string",
        description: "Simplify path",
        required_args: 1,
        accepts_variadic: false
    },
    Function {
        name: "config_directory",
        signature: "config_directory() -> string",
        description: "User config directory",
        required_args: 0,
        accepts_variadic: false
    },
    Function {
        name: "config_local_directory",
        signature: "config_local_directory() -> string",
        description: "User local config directory",
        required_args: 0,
        accepts_variadic: false
    },
    Function {
        name: "data_directory",
        signature: "data_directory() -> string",
        description: "User data directory",
        required_args: 0,
        accepts_variadic: false
    },
    Function {
        name: "data_local_directory",
        signature: "data_local_directory() -> string",
        description: "User local data directory",
        required_args: 0,
        accepts_variadic: false
    },
    Function {
        name: "datetime",
        signature: "datetime(format: string) -> string",
        description: "Get formatted local time",
        required_args: 1,
        accepts_variadic: false
    },
    Function {
        name: "datetime_utc",
        signature: "datetime_utc(format: string) -> string",
        description: "Get formatted UTC time",
        required_args: 1,
        accepts_variadic: false
    },
    Function {
        name: "encode_uri_component",
        signature: "encode_uri_component(s: string) -> string",
        description: "Percent-encode special characters",
        required_args: 1,
        accepts_variadic: false
    },
    Function {
        name: "env",
        signature: "env(key: string) -> string or env(key: string, default: string) -> string",
        description: "Retrieve environment variable",
        required_args: 1,
        accepts_variadic: true
    },
    Function {
        name: "error",
        signature: "error(message: string) -> !",
        description: "Abort with error message",
        required_args: 1,
        accepts_variadic: false
    },
    Function {
        name: "executable_directory",
        signature: "executable_directory() -> string",
        description: "User executable directory",
        required_args: 0,
        accepts_variadic: false
    },
    Function {
        name: "extension",
        signature: "extension(path: string) -> string",
        description: "Get file extension",
        required_args: 1,
        accepts_variadic: false
    },
    Function {
        name: "file_name",
        signature: "file_name(path: string) -> string",
        description: "Get file name",
        required_args: 1,
        accepts_variadic: false
    },
    Function {
        name: "file_stem",
        signature: "file_stem(path: string) -> string",
        description: "Get file name without extension",
        required_args: 1,
        accepts_variadic: false
    },
    Function {
        name: "home_directory",
        signature: "home_directory() -> string",
        description: "User home directory",
        required_args: 0,
        accepts_variadic: false
    },
    Function {
        name: "invocation_directory",
        signature: "invocation_directory() -> string",
        description: "Current directory when just was invoked",
        required_args: 0,
        accepts_variadic: false
    },
    Function {
        name: "invocation_directory_native",
        signature: "invocation_directory_native() -> string",
        description: "Current directory when just was invoked (native format)",
        required_args: 0,
        accepts_variadic: false
    },
    Function {
        name: "is_dependency",
        signature: "is_dependency() -> string",
        description: "Check if recipe is being run as dependency",
        required_args: 0,
        accepts_variadic: false
    },
    Function {
        name: "join",
        signature: "join(a: string, b: string...) -> string",
        description: "Join paths",
        required_args: 2,
        accepts_variadic: true
    },
    Function {
        name: "just_executable",
        signature: "just_executable() -> string",
        description: "Path to just executable",
        required_args: 0,
        accepts_variadic: false
    },
    Function {
        name: "just_pid",
        signature: "just_pid() -> string",
        description: "Process ID of just executable",
        required_args: 0,
        accepts_variadic: false
    },
    Function {
        name: "justfile",
        signature: "justfile() -> string",
        description: "Path of current justfile",
        required_args: 0,
        accepts_variadic: false
    },
    Function {
        name: "justfile_directory",
        signature: "justfile_directory() -> string",
        description: "Directory of current justfile",
        required_args: 0,
        accepts_variadic: false
    },
    Function {
        name: "kebabcase",
        signature: "kebabcase(s: string) -> string",
        description: "Convert to kebab-case",
        required_args: 1,
        accepts_variadic: false
    },
    Function {
        name: "lowercase",
        signature: "lowercase(s: string) -> string",
        description: "Convert to lowercase",
        required_args: 1,
        accepts_variadic: false
    },
    Function {
        name: "lowercamelcase",
        signature: "lowercamelcase(s: string) -> string",
        description: "Convert to lowerCamelCase",
        required_args: 1,
        accepts_variadic: false
    },
    Function {
        name: "num_cpus",
        signature: "num_cpus() -> number",
        description: "Number of logical CPUs",
        required_args: 0,
        accepts_variadic: false
    },
    Function {
        name: "os",
        signature: "os() -> string",
        description: "Operating system",
        required_args: 0,
        accepts_variadic: false
    },
    Function {
        name: "os_family",
        signature: "os_family() -> string",
        description: "Operating system family",
        required_args: 0,
        accepts_variadic: false
    },
    Function {
        name: "parent_directory",
        signature: "parent_directory(path: string) -> string",
        description: "Get parent directory",
        required_args: 1,
        accepts_variadic: false
    },
    Function {
        name: "path_exists",
        signature: "path_exists(path: string) -> boolean",
        description: "Check if path exists",
        required_args: 1,
        accepts_variadic: false
    },
    Function {
        name: "prepend",
        signature: "prepend(prefix: string, s: string) -> string",
        description: "Prepend prefix to strings",
        required_args: 2,
        accepts_variadic: false
    },
    Function {
        name: "quote",
        signature: "quote(s: string) -> string",
        description: "Quote string for shell",
        required_args: 1,
        accepts_variadic: false
    },
    Function {
        name: "read",
        signature: "read(path: string) -> string",
        description: "Read file content",
        required_args: 1,
        accepts_variadic: false
    },
    Function {
        name: "replace",
        signature: "replace(s: string, from: string, to: string) -> string",
        description: "Replace substring",
        required_args: 3,
        accepts_variadic: false
    },
    Function {
        name: "replace_regex",
        signature: "replace_regex(s: string, regex: string, replacement: string) -> string",
        description: "Replace with regex",
        required_args: 3,
        accepts_variadic: false
    },
    Function {
        name: "require",
        signature: "require(name: string) -> string",
        description: "Find executable in PATH",
        required_args: 1,
        accepts_variadic: false
    },
    Function {
        name: "semver_matches",
        signature: "semver_matches(version: string, requirement: string) -> string",
        description: "Check if version matches requirement",
        required_args: 2,
        accepts_variadic: false
    },
    Function {
        name: "sha256",
        signature: "sha256(string: string) -> string",
        description: "Calculate SHA-256 hash of string",
        required_args: 1,
        accepts_variadic: false
    },
    Function {
        name: "sha256_file",
        signature: "sha256_file(path: string) -> string",
        description: "Calculate SHA-256 hash of file",
        required_args: 1,
        accepts_variadic: false
    },
    Function {
        name: "shell",
        signature: "shell(command: string, args: string...) -> string",
        description: "Execute shell command",
        required_args: 1,
        accepts_variadic: true
    },
    Function {
        name: "shoutykebabcase",
        signature: "shoutykebabcase(s: string) -> string",
        description: "Convert to SHOUTY-KEBAB-CASE",
        required_args: 1,
        accepts_variadic: false
    },
    Function {
        name: "shoutysnakecase",
        signature: "shoutysnakecase(s: string) -> string",
        description: "Convert to SHOUTY_SNAKE_CASE",
        required_args: 1,
        accepts_variadic: false
    },
    Function {
        name: "snakecase",
        signature: "snakecase(s: string) -> string",
        description: "Convert to snake_case",
        required_args: 1,
        accepts_variadic: false
    },
    Function {
        name: "source_directory",
        signature: "source_directory() -> string",
        description: "Directory of current source file",
        required_args: 0,
        accepts_variadic: false
    },
    Function {
        name: "source_file",
        signature: "source_file() -> string",
        description: "Path of current source file",
        required_args: 0,
        accepts_variadic: false
    },
    Function {
        name: "style",
        signature: "style(name: string) -> string",
        description: "Get terminal style escape sequence",
        required_args: 1,
        accepts_variadic: false
    },
    Function {
        name: "titlecase",
        signature: "titlecase(s: string) -> string",
        description: "Convert to Title Case",
        required_args: 1,
        accepts_variadic: false
    },
    Function {
        name: "trim",
        signature: "trim(s: string) -> string",
        description: "Remove leading and trailing whitespace",
        required_args: 1,
        accepts_variadic: false
    },
    Function {
        name: "trim_end",
        signature: "trim_end(s: string) -> string",
        description: "Remove trailing whitespace",
        required_args: 1,
        accepts_variadic: false
    },
    Function {
        name: "trim_end_match",
        signature: "trim_end_match(s: string, substring: string) -> string",
        description: "Remove suffix",
        required_args: 2,
        accepts_variadic: false
    },
    Function {
        name: "trim_end_matches",
        signature: "trim_end_matches(s: string, substring: string) -> string",
        description: "Repeatedly remove suffix",
        required_args: 2,
        accepts_variadic: false
    },
    Function {
        name: "trim_start",
        signature: "trim_start(s: string) -> string",
        description: "Remove leading whitespace",
        required_args: 1,
        accepts_variadic: false
    },
    Function {
        name: "trim_start_match",
        signature: "trim_start_match(s: string, substring: string) -> string",
        description: "Remove prefix",
        required_args: 2,
        accepts_variadic: false
    },
    Function {
        name: "trim_start_matches",
        signature: "trim_start_matches(s: string, substring: string) -> string",
        description: "Repeatedly remove prefix",
        required_args: 2,
        accepts_variadic: false
    },
    Function {
        name: "uppercamelcase",
        signature: "uppercamelcase(s: string) -> string",
        description: "Convert to UpperCamelCase",
        required_args: 1,
        accepts_variadic: false
    },
    Function {
        name: "uppercase",
        signature: "uppercase(s: string) -> string",
        description: "Convert to uppercase",
        required_args: 1,
        accepts_variadic: false
    },
    Function {
        name: "uuid",
        signature: "uuid() -> string",
        description: "Generate random UUID",
        required_args: 0,
        accepts_variadic: false
    },
    Function {
        name: "which",
        signature: "which(name: string) -> string",
        description: "Find executable in PATH or return empty string",
        required_args: 1,
        accepts_variadic: false
    },
    Function {
        name: "without_extension",
        signature: "without_extension(path: string) -> string",
        description: "Get path without extension",
        required_args: 1,
        accepts_variadic: false
    },
];

pub(crate) const SETTINGS: [Setting<'_>; 18] = [
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
