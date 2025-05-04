use super::*;

pub(crate) const BUILTINS: [Builtin<'_>; 132] = [
  Builtin::Attribute {
    name: "confirm",
    description: "Require confirmation prior to executing recipe.",
    version: "1.17.0",
    targets: &[AttributeTarget::Recipe],
    parameters: None,
  },
  Builtin::Attribute {
    name: "confirm",
    description: "Require confirmation prior to executing recipe with a custom prompt.",
    version: "1.23.0",
    targets: &[AttributeTarget::Recipe],
    parameters: Some("'PROMPT'"),
  },
  Builtin::Attribute {
    name: "doc",
    description: "Set recipe or module's documentation comment.",
    version: "1.27.0",
    targets: &[AttributeTarget::Module, AttributeTarget::Recipe],
    parameters: Some("'DOC'"),
  },
  Builtin::Attribute {
    name: "extension",
    description: "Set shebang recipe script's file extension. EXT should include a period if one is desired.",
    version: "1.32.0",
    targets: &[AttributeTarget::Recipe],
    parameters: Some("'EXT'"),
  },
  Builtin::Attribute {
    name: "group",
    description: "Put recipe or module in group NAME.",
    version: "1.27.0",
    targets: &[AttributeTarget::Module, AttributeTarget::Recipe],
    parameters: Some("'NAME'"),
  },
  Builtin::Attribute {
    name: "linux",
    description: "Enable recipe on Linux.",
    version: "1.8.0",
    targets: &[AttributeTarget::Recipe],
    parameters: None,
  },
  Builtin::Attribute {
    name: "macos",
    description: "Enable recipe on MacOS.",
    version: "1.8.0",
    targets: &[AttributeTarget::Recipe],
    parameters: None,
  },
  Builtin::Attribute {
    name: "no-cd",
    description: "Don't change directory before executing recipe.",
    version: "1.9.0",
    targets: &[AttributeTarget::Recipe],
    parameters: None,
  },
  Builtin::Attribute {
    name: "no-exit-message",
    description: "Don't print an error message if recipe fails.",
    version: "1.7.0",
    targets: &[AttributeTarget::Recipe],
    parameters: None,
  },
  Builtin::Attribute {
    name: "no-quiet",
    description: "Override globally quiet recipes and always echo out the recipe.",
    version: "1.23.0",
    targets: &[AttributeTarget::Recipe],
    parameters: None,
  },
  Builtin::Attribute {
    name: "openbsd",
    description: "Enable recipe on OpenBSD.",
    version: "1.38.0",
    targets: &[AttributeTarget::Recipe],
    parameters: None,
  },
  Builtin::Attribute {
    name: "positional-arguments",
    description: "Turn on positional arguments for this recipe.",
    version: "1.29.0",
    targets: &[AttributeTarget::Recipe],
    parameters: None,
  },
  Builtin::Attribute {
    name: "private",
    description: "Make recipe, alias, or variable private.",
    version: "1.10.0",
    targets: &[AttributeTarget::Alias, AttributeTarget::Recipe],
    parameters: None,
  },
  Builtin::Attribute {
    name: "script",
    description: "Execute recipe as script.",
    version: "1.33.0",
    targets: &[AttributeTarget::Recipe],
    parameters: None,
  },
  Builtin::Attribute {
    name: "script",
    description: "Execute recipe as a script interpreted by COMMAND.",
    version: "1.32.0",
    targets: &[AttributeTarget::Recipe],
    parameters: Some("COMMAND"),
  },
  Builtin::Attribute {
    name: "unix",
    description: "Enable recipe on Unixes. (Includes MacOS)",
    version: "1.8.0",
    targets: &[AttributeTarget::Recipe],
    parameters: None,
  },
  Builtin::Attribute {
    name: "windows",
    description: "Enable recipe on Windows.",
    version: "1.8.0",
    targets: &[AttributeTarget::Recipe],
    parameters: None,
  },
  Builtin::Attribute {
    name: "working-directory",
    description: "Set recipe working directory. PATH may be relative or absolute. If relative, it is interpreted relative to the default working directory.",
    version: "1.38.0",
    targets: &[AttributeTarget::Recipe],
    parameters: Some("PATH"),
  },
  Builtin::Constant {
    name: "HEX",
    description: "Lowercase hexadecimal digit string",
    value: "\"0123456789abcdef\"",
  },
  Builtin::Constant {
    name: "HEXLOWER",
    description: "Explicit lowercase hex digits",
    value: "\"0123456789abcdef\"",
  },
  Builtin::Constant {
    name: "HEXUPPER",
    description: "Uppercase hexadecimal digit string",
    value: "\"0123456789ABCDEF\"",
  },
  Builtin::Constant {
    name: "CLEAR",
    description: "Clear screen",
    value: "\"\\ec\"",
  },
  Builtin::Constant {
    name: "NORMAL",
    description: "Reset terminal style",
    value: "\"\\e[0m\"",
  },
  Builtin::Constant {
    name: "BOLD",
    description: "Bold text",
    value: "\"\\e[1m\"",
  },
  Builtin::Constant {
    name: "ITALIC",
    description: "Italic text",
    value: "\"\\e[3m\"",
  },
  Builtin::Constant {
    name: "UNDERLINE",
    description: "Underlined text",
    value: "\"\\e[4m\"",
  },
  Builtin::Constant {
    name: "INVERT",
    description: "Inverted colors",
    value: "\"\\e[7m\"",
  },
  Builtin::Constant {
    name: "HIDE",
    description: "Hidden text",
    value: "\"\\e[8m\"",
  },
  Builtin::Constant {
    name: "STRIKETHROUGH",
    description: "Strikethrough text",
    value: "\"\\e[9m\"",
  },
  Builtin::Constant {
    name: "BLACK",
    description: "Black text",
    value: "\"\\e[30m\"",
  },
  Builtin::Constant {
    name: "RED",
    description: "Red text",
    value: "\"\\e[31m\"",
  },
  Builtin::Constant {
    name: "GREEN",
    description: "Green text",
    value: "\"\\e[32m\"",
  },
  Builtin::Constant {
    name: "YELLOW",
    description: "Yellow text",
    value: "\"\\e[33m\"",
  },
  Builtin::Constant {
    name: "BLUE",
    description: "Blue text",
    value: "\"\\e[34m\"",
  },
  Builtin::Constant {
    name: "MAGENTA",
    description: "Magenta text",
    value: "\"\\e[35m\"",
  },
  Builtin::Constant {
    name: "CYAN",
    description: "Cyan text",
    value: "\"\\e[36m\"",
  },
  Builtin::Constant {
    name: "WHITE",
    description: "White text",
    value: "\"\\e[37m\"",
  },
  Builtin::Constant {
    name: "BG_BLACK",
    description: "Black background",
    value: "\"\\e[40m\"",
  },
  Builtin::Constant {
    name: "BG_RED",
    description: "Red background",
    value: "\"\\e[41m\"",
  },
  Builtin::Constant {
    name: "BG_GREEN",
    description: "Green background",
    value: "\"\\e[42m\"",
  },
  Builtin::Constant {
    name: "BG_YELLOW",
    description: "Yellow background",
    value: "\"\\e[43m\"",
  },
  Builtin::Constant {
    name: "BG_BLUE",
    description: "Blue background",
    value: "\"\\e[44m\"",
  },
  Builtin::Constant {
    name: "BG_MAGENTA",
    description: "Magenta background",
    value: "\"\\e[45m\"",
  },
  Builtin::Constant {
    name: "BG_CYAN",
    description: "Cyan background",
    value: "\"\\e[46m\"",
  },
  Builtin::Constant {
    name: "BG_WHITE",
    description: "White background",
    value: "\"\\e[47m\"",
  },
  Builtin::Function {
    name: "absolute_path",
    signature: "absolute_path(path: string) -> string",
    description: "Get the absolute path relative to `path` in the working directory.",
    required_args: 1,
    accepts_variadic: false
  },
  Builtin::Function {
    name: "append",
    signature: "append(suffix: string, s: string) -> string",
    description: "Append suffix to strings",
    required_args: 2,
    accepts_variadic: false
  },
  Builtin::Function {
    name: "arch",
    signature: "arch() -> string",
    description: "Instruction set architecture",
    required_args: 0,
    accepts_variadic: false
  },
  Builtin::Function {
    name: "blake3",
    signature: "blake3(string: string) -> string",
    description: "Calculate BLAKE3 hash of string",
    required_args: 1,
    accepts_variadic: false
  },
  Builtin::Function {
    name: "blake3_file",
    signature: "blake3_file(path: string) -> string",
    description: "Calculate BLAKE3 hash of file",
    required_args: 1,
    accepts_variadic: false
  },
  Builtin::Function {
    name: "cache_directory",
    signature: "cache_directory() -> string",
    description: "User cache directory",
    required_args: 0,
    accepts_variadic: false
  },
  Builtin::Function {
    name: "canonicalize",
    signature: "canonicalize(path: string) -> string",
    description: "Canonicalize `path` by resolving symlinks and removing `.`, `..`, and extra `/`s where possible.",
    required_args: 1,
    accepts_variadic: false
  },
  Builtin::Function {
    name: "capitalize",
    signature: "capitalize(s: string) -> string",
    description: "Convert first character to uppercase",
    required_args: 1,
    accepts_variadic: false
  },
  Builtin::Function {
    name: "choose",
    signature: "choose(n: string, alphabet: string) -> string",
    description: "Generate random string from alphabet",
    required_args: 2,
    accepts_variadic: false
  },
  Builtin::Function {
    name: "clean",
    signature: "clean(path: string) -> string",
    description: "Simplify path",
    required_args: 1,
    accepts_variadic: false
  },
  Builtin::Function {
    name: "config_directory",
    signature: "config_directory() -> string",
    description: "User config directory",
    required_args: 0,
    accepts_variadic: false
  },
  Builtin::Function {
    name: "config_local_directory",
    signature: "config_local_directory() -> string",
    description: "User local config directory",
    required_args: 0,
    accepts_variadic: false
  },
  Builtin::Function {
    name: "data_directory",
    signature: "data_directory() -> string",
    description: "User data directory",
    required_args: 0,
    accepts_variadic: false
  },
  Builtin::Function {
    name: "data_local_directory",
    signature: "data_local_directory() -> string",
    description: "User local data directory",
    required_args: 0,
    accepts_variadic: false
  },
  Builtin::Function {
    name: "datetime",
    signature: "datetime(format: string) -> string",
    description: "Get formatted local time",
    required_args: 1,
    accepts_variadic: false
  },
  Builtin::Function {
    name: "datetime_utc",
    signature: "datetime_utc(format: string) -> string",
    description: "Get formatted UTC time",
    required_args: 1,
    accepts_variadic: false
  },
  Builtin::Function {
    name: "encode_uri_component",
    signature: "encode_uri_component(s: string) -> string",
    description: "Percent-encode special characters",
    required_args: 1,
    accepts_variadic: false
  },
  Builtin::Function {
    name: "env",
    signature: "env(key: string) -> string or env(key: string, default: string) -> string",
    description: "Retrieve environment variable",
    required_args: 1,
    accepts_variadic: true
  },
  Builtin::Function {
    name: "error",
    signature: "error(message: string) -> !",
    description: "Abort with error message",
    required_args: 1,
    accepts_variadic: false
  },
  Builtin::Function {
    name: "executable_directory",
    signature: "executable_directory() -> string",
    description: "User executable directory",
    required_args: 0,
    accepts_variadic: false
  },
  Builtin::Function {
    name: "extension",
    signature: "extension(path: string) -> string",
    description: "Get file extension",
    required_args: 1,
    accepts_variadic: false
  },
  Builtin::Function {
    name: "file_name",
    signature: "file_name(path: string) -> string",
    description: "Get file name",
    required_args: 1,
    accepts_variadic: false
  },
  Builtin::Function {
    name: "file_stem",
    signature: "file_stem(path: string) -> string",
    description: "Get file name without extension",
    required_args: 1,
    accepts_variadic: false
  },
  Builtin::Function {
    name: "home_directory",
    signature: "home_directory() -> string",
    description: "User home directory",
    required_args: 0,
    accepts_variadic: false
  },
  Builtin::Function {
    name: "invocation_directory",
    signature: "invocation_directory() -> string",
    description: "Current directory when just was invoked",
    required_args: 0,
    accepts_variadic: false
  },
  Builtin::Function {
    name: "invocation_directory_native",
    signature: "invocation_directory_native() -> string",
    description: "Current directory when just was invoked (native format)",
    required_args: 0,
    accepts_variadic: false
  },
  Builtin::Function {
    name: "is_dependency",
    signature: "is_dependency() -> string",
    description: "Check if recipe is being run as dependency",
    required_args: 0,
    accepts_variadic: false
  },
  Builtin::Function {
    name: "join",
    signature: "join(a: string, b: string...) -> string",
    description: "Join paths",
    required_args: 2,
    accepts_variadic: true
  },
  Builtin::Function {
    name: "just_executable",
    signature: "just_executable() -> string",
    description: "Path to just executable",
    required_args: 0,
    accepts_variadic: false
  },
  Builtin::Function {
    name: "just_pid",
    signature: "just_pid() -> string",
    description: "Process ID of just executable",
    required_args: 0,
    accepts_variadic: false
  },
  Builtin::Function {
    name: "justfile",
    signature: "justfile() -> string",
    description: "Path of current justfile",
    required_args: 0,
    accepts_variadic: false
  },
  Builtin::Function {
    name: "justfile_directory",
    signature: "justfile_directory() -> string",
    description: "Directory of current justfile",
    required_args: 0,
    accepts_variadic: false
  },
  Builtin::Function {
    name: "kebabcase",
    signature: "kebabcase(s: string) -> string",
    description: "Convert to kebab-case",
    required_args: 1,
    accepts_variadic: false
  },
  Builtin::Function {
    name: "lowercase",
    signature: "lowercase(s: string) -> string",
    description: "Convert to lowercase",
    required_args: 1,
    accepts_variadic: false
  },
  Builtin::Function {
    name: "lowercamelcase",
    signature: "lowercamelcase(s: string) -> string",
    description: "Convert to lowerCamelCase",
    required_args: 1,
    accepts_variadic: false
  },
  Builtin::Function {
    name: "num_cpus",
    signature: "num_cpus() -> number",
    description: "Number of logical CPUs",
    required_args: 0,
    accepts_variadic: false
  },
  Builtin::Function {
    name: "os",
    signature: "os() -> string",
    description: "Operating system",
    required_args: 0,
    accepts_variadic: false
  },
  Builtin::Function {
    name: "os_family",
    signature: "os_family() -> string",
    description: "Operating system family",
    required_args: 0,
    accepts_variadic: false
  },
  Builtin::Function {
    name: "parent_directory",
    signature: "parent_directory(path: string) -> string",
    description: "Get parent directory",
    required_args: 1,
    accepts_variadic: false
  },
  Builtin::Function {
    name: "path_exists",
    signature: "path_exists(path: string) -> boolean",
    description: "Check if path exists",
    required_args: 1,
    accepts_variadic: false
  },
  Builtin::Function {
    name: "prepend",
    signature: "prepend(prefix: string, s: string) -> string",
    description: "Prepend prefix to strings",
    required_args: 2,
    accepts_variadic: false
  },
  Builtin::Function {
    name: "quote",
    signature: "quote(s: string) -> string",
    description: "Quote string for shell",
    required_args: 1,
    accepts_variadic: false
  },
  Builtin::Function {
    name: "read",
    signature: "read(path: string) -> string",
    description: "Read file content",
    required_args: 1,
    accepts_variadic: false
  },
  Builtin::Function {
    name: "replace",
    signature: "replace(s: string, from: string, to: string) -> string",
    description: "Replace substring",
    required_args: 3,
    accepts_variadic: false
  },
  Builtin::Function {
    name: "replace_regex",
    signature: "replace_regex(s: string, regex: string, replacement: string) -> string",
    description: "Replace with regex",
    required_args: 3,
    accepts_variadic: false
  },
  Builtin::Function {
    name: "require",
    signature: "require(name: string) -> string",
    description: "Find executable in PATH",
    required_args: 1,
    accepts_variadic: false
  },
  Builtin::Function {
    name: "semver_matches",
    signature: "semver_matches(version: string, requirement: string) -> string",
    description: "Check if version matches requirement",
    required_args: 2,
    accepts_variadic: false
  },
  Builtin::Function {
    name: "sha256",
    signature: "sha256(string: string) -> string",
    description: "Calculate SHA-256 hash of string",
    required_args: 1,
    accepts_variadic: false
  },
  Builtin::Function {
    name: "sha256_file",
    signature: "sha256_file(path: string) -> string",
    description: "Calculate SHA-256 hash of file",
    required_args: 1,
    accepts_variadic: false
  },
  Builtin::Function {
    name: "shell",
    signature: "shell(command: string, args: string...) -> string",
    description: "Execute shell command",
    required_args: 1,
    accepts_variadic: true
  },
  Builtin::Function {
    name: "shoutykebabcase",
    signature: "shoutykebabcase(s: string) -> string",
    description: "Convert to SHOUTY-KEBAB-CASE",
    required_args: 1,
    accepts_variadic: false
  },
  Builtin::Function {
    name: "shoutysnakecase",
    signature: "shoutysnakecase(s: string) -> string",
    description: "Convert to SHOUTY_SNAKE_CASE",
    required_args: 1,
    accepts_variadic: false
  },
  Builtin::Function {
    name: "snakecase",
    signature: "snakecase(s: string) -> string",
    description: "Convert to snake_case",
    required_args: 1,
    accepts_variadic: false
  },
  Builtin::Function {
    name: "source_directory",
    signature: "source_directory() -> string",
    description: "Directory of current source file",
    required_args: 0,
    accepts_variadic: false
  },
  Builtin::Function {
    name: "source_file",
    signature: "source_file() -> string",
    description: "Path of current source file",
    required_args: 0,
    accepts_variadic: false
  },
  Builtin::Function {
    name: "style",
    signature: "style(name: string) -> string",
    description: "Get terminal style escape sequence",
    required_args: 1,
    accepts_variadic: false
  },
  Builtin::Function {
    name: "titlecase",
    signature: "titlecase(s: string) -> string",
    description: "Convert to Title Case",
    required_args: 1,
    accepts_variadic: false
  },
  Builtin::Function {
    name: "trim",
    signature: "trim(s: string) -> string",
    description: "Remove leading and trailing whitespace",
    required_args: 1,
    accepts_variadic: false
  },
  Builtin::Function {
    name: "trim_end",
    signature: "trim_end(s: string) -> string",
    description: "Remove trailing whitespace",
    required_args: 1,
    accepts_variadic: false
  },
  Builtin::Function {
    name: "trim_end_match",
    signature: "trim_end_match(s: string, substring: string) -> string",
    description: "Remove suffix",
    required_args: 2,
    accepts_variadic: false
  },
  Builtin::Function {
    name: "trim_end_matches",
    signature: "trim_end_matches(s: string, substring: string) -> string",
    description: "Repeatedly remove suffix",
    required_args: 2,
    accepts_variadic: false
  },
  Builtin::Function {
    name: "trim_start",
    signature: "trim_start(s: string) -> string",
    description: "Remove leading whitespace",
    required_args: 1,
    accepts_variadic: false
  },
  Builtin::Function {
    name: "trim_start_match",
    signature: "trim_start_match(s: string, substring: string) -> string",
    description: "Remove prefix",
    required_args: 2,
    accepts_variadic: false
  },
  Builtin::Function {
    name: "trim_start_matches",
    signature: "trim_start_matches(s: string, substring: string) -> string",
    description: "Repeatedly remove prefix",
    required_args: 2,
    accepts_variadic: false
  },
  Builtin::Function {
    name: "uppercamelcase",
    signature: "uppercamelcase(s: string) -> string",
    description: "Convert to UpperCamelCase",
    required_args: 1,
    accepts_variadic: false
  },
  Builtin::Function {
    name: "uppercase",
    signature: "uppercase(s: string) -> string",
    description: "Convert to uppercase",
    required_args: 1,
    accepts_variadic: false
  },
  Builtin::Function {
    name: "uuid",
    signature: "uuid() -> string",
    description: "Generate random UUID",
    required_args: 0,
    accepts_variadic: false
  },
  Builtin::Function {
    name: "which",
    signature: "which(name: string) -> string",
    description: "Find executable in PATH or return empty string",
    required_args: 1,
    accepts_variadic: false
  },
  Builtin::Function {
    name: "without_extension",
    signature: "without_extension(path: string) -> string",
    description: "Get path without extension",
    required_args: 1,
    accepts_variadic: false
  },
  Builtin::Setting {
    name: "allow-duplicate-recipes",
    kind: SettingKind::Boolean(false),
    description: "Allow recipes appearing later in a `justfile` to override earlier recipes with the same name.",
    default: "false",
  },
  Builtin::Setting {
    name: "allow-duplicate-variables",
    kind: SettingKind::Boolean(false),
    description: "Allow variables appearing later in a `justfile` to override earlier variables with the same name.",
    default: "false",
  },
  Builtin::Setting {
    name: "dotenv-filename",
    kind: SettingKind::String,
    description: "Load a `.env` file with a custom name, if present.",
    default: "",
  },
  Builtin::Setting {
    name: "dotenv-load",
    kind: SettingKind::Boolean(false),
    description: "Load a `.env` file, if present.",
    default: "false",
  },
  Builtin::Setting {
    name: "dotenv-path",
    kind: SettingKind::String,
    description: "Load a `.env` file from a custom path and error if not present. Overrides `dotenv-filename`.",
    default: "",
  },
  Builtin::Setting {
    name: "dotenv-required",
    kind: SettingKind::Boolean(false),
    description: "Error if a `.env` file isn't found.",
    default: "false",
  },
  Builtin::Setting {
    name: "export",
    kind: SettingKind::Boolean(false),
    description: "Export all variables as environment variables.",
    default: "false",
  },
  Builtin::Setting {
    name: "fallback",
    kind: SettingKind::Boolean(false),
    description: "Search `justfile` in parent directory if the first recipe on the command line is not found.",
    default: "false",
  },
  Builtin::Setting {
    name: "ignore-comments",
    kind: SettingKind::Boolean(false),
    description: "Ignore recipe lines beginning with `#`.",
    default: "false",
  },
  Builtin::Setting {
    name: "positional-arguments",
    kind: SettingKind::Boolean(false),
    description: "Pass positional arguments.",
    default: "false",
  },
  Builtin::Setting {
    name: "quiet",
    kind: SettingKind::Boolean(false),
    description: "Disable echoing recipe lines before executing.",
    default: "false",
  },
  Builtin::Setting {
    name: "script-interpreter",
    kind: SettingKind::Array,
    description: "Set command used to invoke recipes with empty `[script]` attribute.",
    default: "['sh', '-eu']",
  },
  Builtin::Setting {
    name: "shell",
    kind: SettingKind::Array,
    description: "Set command used to invoke recipes and evaluate backticks.",
    default: "",
  },
  Builtin::Setting {
    name: "tempdir",
    kind: SettingKind::String,
    description: "Create temporary directories in `tempdir` instead of the system default temporary directory.",
    default: "",
  },
  Builtin::Setting {
    name: "unstable",
    kind: SettingKind::Boolean(false),
    description: "Enable unstable features.",
    default: "false",
  },
  Builtin::Setting {
    name: "windows-powershell",
    kind: SettingKind::Boolean(false),
    description: "Use PowerShell on Windows as default shell. (Deprecated. Use `windows-shell` instead.)",
    default: "false",
  },
  Builtin::Setting {
    name: "windows-shell",
    kind: SettingKind::Array,
    description: "Set the command used to invoke recipes and evaluate backticks.",
    default: "",
  },
  Builtin::Setting {
    name: "working-directory",
    kind: SettingKind::String,
    description: "Set the working directory for recipes and backticks, relative to the default working directory.",
    default: "",
  },
];
