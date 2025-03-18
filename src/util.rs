pub(crate) fn builtin_constants() -> Vec<(String, String)> {
  vec![
    ("HEX".to_string(), "0123456789abcdef".to_string()),
    ("HEXLOWER".to_string(), "0123456789abcdef".to_string()),
    ("HEXUPPER".to_string(), "0123456789ABCDEF".to_string()),
    ("CLEAR".to_string(), "Clear screen".to_string()),
    ("NORMAL".to_string(), "Reset terminal style".to_string()),
    ("BOLD".to_string(), "Bold text".to_string()),
    ("ITALIC".to_string(), "Italic text".to_string()),
    ("UNDERLINE".to_string(), "Underlined text".to_string()),
    ("INVERT".to_string(), "Inverted colors".to_string()),
    ("HIDE".to_string(), "Hidden text".to_string()),
    (
      "STRIKETHROUGH".to_string(),
      "Strikethrough text".to_string(),
    ),
    ("BLACK".to_string(), "Black text".to_string()),
    ("RED".to_string(), "Red text".to_string()),
    ("GREEN".to_string(), "Green text".to_string()),
    ("YELLOW".to_string(), "Yellow text".to_string()),
    ("BLUE".to_string(), "Blue text".to_string()),
    ("MAGENTA".to_string(), "Magenta text".to_string()),
    ("CYAN".to_string(), "Cyan text".to_string()),
    ("WHITE".to_string(), "White text".to_string()),
    ("BG_BLACK".to_string(), "Black background".to_string()),
    ("BG_RED".to_string(), "Red background".to_string()),
    ("BG_GREEN".to_string(), "Green background".to_string()),
    ("BG_YELLOW".to_string(), "Yellow background".to_string()),
    ("BG_BLUE".to_string(), "Blue background".to_string()),
    ("BG_MAGENTA".to_string(), "Magenta background".to_string()),
    ("BG_CYAN".to_string(), "Cyan background".to_string()),
    ("BG_WHITE".to_string(), "White background".to_string()),
  ]
}

pub(crate) fn builtin_functions() -> Vec<(String, String)> {
  vec![
    (
      "arch".to_string(),
      "Instruction set architecture".to_string(),
    ),
    ("num_cpus".to_string(), "Number of logical CPUs".to_string()),
    ("os".to_string(), "Operating system".to_string()),
    (
      "os_family".to_string(),
      "Operating system family".to_string(),
    ),
    ("shell".to_string(), "Execute shell command".to_string()),
    (
      "env".to_string(),
      "Retrieve environment variable".to_string(),
    ),
    ("require".to_string(), "Find executable in PATH".to_string()),
    (
      "which".to_string(),
      "Find executable in PATH or return empty string".to_string(),
    ),
    (
      "is_dependency".to_string(),
      "Check if recipe is being run as dependency".to_string(),
    ),
    (
      "invocation_directory".to_string(),
      "Current directory when just was invoked".to_string(),
    ),
    (
      "invocation_directory_native".to_string(),
      "Current directory when just was invoked (native format)".to_string(),
    ),
    (
      "justfile".to_string(),
      "Path of current justfile".to_string(),
    ),
    (
      "justfile_directory".to_string(),
      "Directory of current justfile".to_string(),
    ),
    (
      "source_file".to_string(),
      "Path of current source file".to_string(),
    ),
    (
      "source_directory".to_string(),
      "Directory of current source file".to_string(),
    ),
    (
      "just_executable".to_string(),
      "Path to just executable".to_string(),
    ),
    (
      "just_pid".to_string(),
      "Process ID of just executable".to_string(),
    ),
    ("append".to_string(), "Append suffix to strings".to_string()),
    (
      "prepend".to_string(),
      "Prepend prefix to strings".to_string(),
    ),
    (
      "encode_uri_component".to_string(),
      "Percent-encode special characters".to_string(),
    ),
    ("quote".to_string(), "Quote string for shell".to_string()),
    ("replace".to_string(), "Replace substring".to_string()),
    (
      "replace_regex".to_string(),
      "Replace with regex".to_string(),
    ),
    (
      "trim".to_string(),
      "Remove leading and trailing whitespace".to_string(),
    ),
    (
      "trim_end".to_string(),
      "Remove trailing whitespace".to_string(),
    ),
    ("trim_end_match".to_string(), "Remove suffix".to_string()),
    (
      "trim_end_matches".to_string(),
      "Repeatedly remove suffix".to_string(),
    ),
    (
      "trim_start".to_string(),
      "Remove leading whitespace".to_string(),
    ),
    ("trim_start_match".to_string(), "Remove prefix".to_string()),
    (
      "trim_start_matches".to_string(),
      "Repeatedly remove prefix".to_string(),
    ),
    (
      "capitalize".to_string(),
      "Convert first character to uppercase".to_string(),
    ),
    ("kebabcase".to_string(), "Convert to kebab-case".to_string()),
    (
      "lowercamelcase".to_string(),
      "Convert to lowerCamelCase".to_string(),
    ),
    ("lowercase".to_string(), "Convert to lowercase".to_string()),
    (
      "shoutykebabcase".to_string(),
      "Convert to SHOUTY-KEBAB-CASE".to_string(),
    ),
    (
      "shoutysnakecase".to_string(),
      "Convert to SHOUTY_SNAKE_CASE".to_string(),
    ),
    ("snakecase".to_string(), "Convert to snake_case".to_string()),
    ("titlecase".to_string(), "Convert to Title Case".to_string()),
    (
      "uppercamelcase".to_string(),
      "Convert to UpperCamelCase".to_string(),
    ),
    ("uppercase".to_string(), "Convert to uppercase".to_string()),
    ("absolute_path".to_string(), "Get absolute path".to_string()),
    ("canonicalize".to_string(), "Canonicalize path".to_string()),
    ("extension".to_string(), "Get file extension".to_string()),
    ("file_name".to_string(), "Get file name".to_string()),
    (
      "file_stem".to_string(),
      "Get file name without extension".to_string(),
    ),
    (
      "parent_directory".to_string(),
      "Get parent directory".to_string(),
    ),
    (
      "without_extension".to_string(),
      "Get path without extension".to_string(),
    ),
    ("clean".to_string(), "Simplify path".to_string()),
    ("join".to_string(), "Join paths".to_string()),
    (
      "path_exists".to_string(),
      "Check if path exists".to_string(),
    ),
    ("read".to_string(), "Read file content".to_string()),
    ("error".to_string(), "Abort with error message".to_string()),
    (
      "blake3".to_string(),
      "Calculate BLAKE3 hash of string".to_string(),
    ),
    (
      "blake3_file".to_string(),
      "Calculate BLAKE3 hash of file".to_string(),
    ),
    (
      "sha256".to_string(),
      "Calculate SHA-256 hash of string".to_string(),
    ),
    (
      "sha256_file".to_string(),
      "Calculate SHA-256 hash of file".to_string(),
    ),
    ("uuid".to_string(), "Generate random UUID".to_string()),
    (
      "choose".to_string(),
      "Generate random string from alphabet".to_string(),
    ),
    (
      "datetime".to_string(),
      "Get formatted local time".to_string(),
    ),
    (
      "datetime_utc".to_string(),
      "Get formatted UTC time".to_string(),
    ),
    (
      "semver_matches".to_string(),
      "Check if version matches requirement".to_string(),
    ),
    (
      "style".to_string(),
      "Get terminal style escape sequence".to_string(),
    ),
    (
      "cache_directory".to_string(),
      "User cache directory".to_string(),
    ),
    (
      "config_directory".to_string(),
      "User config directory".to_string(),
    ),
    (
      "config_local_directory".to_string(),
      "User local config directory".to_string(),
    ),
    (
      "data_directory".to_string(),
      "User data directory".to_string(),
    ),
    (
      "data_local_directory".to_string(),
      "User local data directory".to_string(),
    ),
    (
      "executable_directory".to_string(),
      "User executable directory".to_string(),
    ),
    (
      "home_directory".to_string(),
      "User home directory".to_string(),
    ),
  ]
}
