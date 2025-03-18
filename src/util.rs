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

pub(crate) fn builtin_functions() -> Vec<(String, String, String)> {
  vec![
    ("arch".to_string(), "arch() -> string".to_string(), "Instruction set architecture".to_string()),
    ("num_cpus".to_string(), "num_cpus() -> number".to_string(), "Number of logical CPUs".to_string()),
    ("os".to_string(), "os() -> string".to_string(), "Operating system".to_string()),
    ("os_family".to_string(), "os_family() -> string".to_string(), "Operating system family".to_string()),
    ("shell".to_string(), "shell(command: string, args: string...) -> string".to_string(), "Execute shell command".to_string()),
    ("env".to_string(), "env(key: string) -> string or env(key: string, default: string) -> string".to_string(), "Retrieve environment variable".to_string()),
    ("require".to_string(), "require(name: string) -> string".to_string(), "Find executable in PATH".to_string()),
    ("which".to_string(), "which(name: string) -> string".to_string(), "Find executable in PATH or return empty string".to_string()),
    ("is_dependency".to_string(), "is_dependency() -> string".to_string(), "Check if recipe is being run as dependency".to_string()),
    ("invocation_directory".to_string(), "invocation_directory() -> string".to_string(), "Current directory when just was invoked".to_string()),
    ("invocation_directory_native".to_string(), "invocation_directory_native() -> string".to_string(), "Current directory when just was invoked (native format)".to_string()),
    ("justfile".to_string(), "justfile() -> string".to_string(), "Path of current justfile".to_string()),
    ("justfile_directory".to_string(), "justfile_directory() -> string".to_string(), "Directory of current justfile".to_string()),
    ("source_file".to_string(), "source_file() -> string".to_string(), "Path of current source file".to_string()),
    ("source_directory".to_string(), "source_directory() -> string".to_string(), "Directory of current source file".to_string()),
    ("just_executable".to_string(), "just_executable() -> string".to_string(), "Path to just executable".to_string()),
    ("just_pid".to_string(), "just_pid() -> string".to_string(), "Process ID of just executable".to_string()),
    ("append".to_string(), "append(suffix: string, s: string) -> string".to_string(), "Append suffix to strings".to_string()),
    ("prepend".to_string(), "prepend(prefix: string, s: string) -> string".to_string(), "Prepend prefix to strings".to_string()),
    ("encode_uri_component".to_string(), "encode_uri_component(s: string) -> string".to_string(), "Percent-encode special characters".to_string()),
    ("quote".to_string(), "quote(s: string) -> string".to_string(), "Quote string for shell".to_string()),
    ("replace".to_string(), "replace(s: string, from: string, to: string) -> string".to_string(), "Replace substring".to_string()),
    ("replace_regex".to_string(), "replace_regex(s: string, regex: string, replacement: string) -> string".to_string(), "Replace with regex".to_string()),
    ("trim".to_string(), "trim(s: string) -> string".to_string(), "Remove leading and trailing whitespace".to_string()),
    ("trim_end".to_string(), "trim_end(s: string) -> string".to_string(), "Remove trailing whitespace".to_string()),
    ("trim_end_match".to_string(), "trim_end_match(s: string, substring: string) -> string".to_string(), "Remove suffix".to_string()),
    ("trim_end_matches".to_string(), "trim_end_matches(s: string, substring: string) -> string".to_string(), "Repeatedly remove suffix".to_string()),
    ("trim_start".to_string(), "trim_start(s: string) -> string".to_string(), "Remove leading whitespace".to_string()),
    ("trim_start_match".to_string(), "trim_start_match(s: string, substring: string) -> string".to_string(), "Remove prefix".to_string()),
    ("trim_start_matches".to_string(), "trim_start_matches(s: string, substring: string) -> string".to_string(), "Repeatedly remove prefix".to_string()),
    ("capitalize".to_string(), "capitalize(s: string) -> string".to_string(), "Convert first character to uppercase".to_string()),
    ("kebabcase".to_string(), "kebabcase(s: string) -> string".to_string(), "Convert to kebab-case".to_string()),
    ("lowercamelcase".to_string(), "lowercamelcase(s: string) -> string".to_string(), "Convert to lowerCamelCase".to_string()),
    ("lowercase".to_string(), "lowercase(s: string) -> string".to_string(), "Convert to lowercase".to_string()),
    ("shoutykebabcase".to_string(), "shoutykebabcase(s: string) -> string".to_string(), "Convert to SHOUTY-KEBAB-CASE".to_string()),
    ("shoutysnakecase".to_string(), "shoutysnakecase(s: string) -> string".to_string(), "Convert to SHOUTY_SNAKE_CASE".to_string()),
    ("snakecase".to_string(), "snakecase(s: string) -> string".to_string(), "Convert to snake_case".to_string()),
    ("titlecase".to_string(), "titlecase(s: string) -> string".to_string(), "Convert to Title Case".to_string()),
    ("uppercamelcase".to_string(), "uppercamelcase(s: string) -> string".to_string(), "Convert to UpperCamelCase".to_string()),
    ("uppercase".to_string(), "uppercase(s: string) -> string".to_string(), "Convert to uppercase".to_string()),
    ("absolute_path".to_string(), "absolute_path(path: string) -> string".to_string(), "Get absolute path".to_string()),
    ("canonicalize".to_string(), "canonicalize(path: string) -> string".to_string(), "Canonicalize path".to_string()),
    ("extension".to_string(), "extension(path: string) -> string".to_string(), "Get file extension".to_string()),
    ("file_name".to_string(), "file_name(path: string) -> string".to_string(), "Get file name".to_string()),
    ("file_stem".to_string(), "file_stem(path: string) -> string".to_string(), "Get file name without extension".to_string()),
    ("parent_directory".to_string(), "parent_directory(path: string) -> string".to_string(), "Get parent directory".to_string()),
    ("without_extension".to_string(), "without_extension(path: string) -> string".to_string(), "Get path without extension".to_string()),
    ("clean".to_string(), "clean(path: string) -> string".to_string(), "Simplify path".to_string()),
    ("join".to_string(), "join(a: string, b: string...) -> string".to_string(), "Join paths".to_string()),
    ("path_exists".to_string(), "path_exists(path: string) -> boolean".to_string(), "Check if path exists".to_string()),
    ("read".to_string(), "read(path: string) -> string".to_string(), "Read file content".to_string()),
    ("error".to_string(), "error(message: string) -> !".to_string(), "Abort with error message".to_string()),
    ("blake3".to_string(), "blake3(string: string) -> string".to_string(), "Calculate BLAKE3 hash of string".to_string()),
    ("blake3_file".to_string(), "blake3_file(path: string) -> string".to_string(), "Calculate BLAKE3 hash of file".to_string()),
    ("sha256".to_string(), "sha256(string: string) -> string".to_string(), "Calculate SHA-256 hash of string".to_string()),
    ("sha256_file".to_string(), "sha256_file(path: string) -> string".to_string(), "Calculate SHA-256 hash of file".to_string()),
    ("uuid".to_string(), "uuid() -> string".to_string(), "Generate random UUID".to_string()),
    ("choose".to_string(), "choose(n: string, alphabet: string) -> string".to_string(), "Generate random string from alphabet".to_string()),
    ("datetime".to_string(), "datetime(format: string) -> string".to_string(), "Get formatted local time".to_string()),
    ("datetime_utc".to_string(), "datetime_utc(format: string) -> string".to_string(), "Get formatted UTC time".to_string()),
    ("semver_matches".to_string(), "semver_matches(version: string, requirement: string) -> string".to_string(), "Check if version matches requirement".to_string()),
    ("style".to_string(), "style(name: string) -> string".to_string(), "Get terminal style escape sequence".to_string()),
    ("cache_directory".to_string(), "cache_directory() -> string".to_string(), "User cache directory".to_string()),
    ("config_directory".to_string(), "config_directory() -> string".to_string(), "User config directory".to_string()),
    ("config_local_directory".to_string(), "config_local_directory() -> string".to_string(), "User local config directory".to_string()),
    ("data_directory".to_string(), "data_directory() -> string".to_string(), "User data directory".to_string()),
    ("data_local_directory".to_string(), "data_local_directory() -> string".to_string(), "User local data directory".to_string()),
    ("executable_directory".to_string(), "executable_directory() -> string".to_string(), "User executable directory".to_string()),
    ("home_directory".to_string(), "home_directory() -> string".to_string(), "User home directory".to_string()),
  ]
}

pub(crate) fn create_function_snippet(function_name: &str) -> String {
  match function_name {
    "arch"
    | "num_cpus"
    | "os"
    | "os_family"
    | "is_dependency"
    | "invocation_directory"
    | "invocation_directory_native"
    | "justfile"
    | "justfile_directory"
    | "source_file"
    | "source_directory"
    | "just_executable"
    | "just_pid"
    | "uuid"
    | "cache_directory"
    | "config_directory"
    | "config_local_directory"
    | "data_directory"
    | "data_local_directory"
    | "executable_directory"
    | "home_directory" => format!("{}()", function_name),
    "require" => format!("{}(${{1:name:string}})", function_name),
    "which" => format!("{}(${{1:name:string}})", function_name),
    "error" => format!("{}(${{1:message:string}})", function_name),
    "encode_uri_component" => format!("{}(${{1:s:string}})", function_name),
    "quote" => format!("{}(${{1:s:string}})", function_name),
    "trim" => format!("{}(${{1:s:string}})", function_name),
    "trim_end" => format!("{}(${{1:s:string}})", function_name),
    "trim_start" => format!("{}(${{1:s:string}})", function_name),
    "capitalize" => format!("{}(${{1:s:string}})", function_name),
    "kebabcase" => format!("{}(${{1:s:string}})", function_name),
    "lowercamelcase" => format!("{}(${{1:s:string}})", function_name),
    "lowercase" => format!("{}(${{1:s:string}})", function_name),
    "shoutykebabcase" => format!("{}(${{1:s:string}})", function_name),
    "shoutysnakecase" => format!("{}(${{1:s:string}})", function_name),
    "snakecase" => format!("{}(${{1:s:string}})", function_name),
    "titlecase" => format!("{}(${{1:s:string}})", function_name),
    "uppercamelcase" => format!("{}(${{1:s:string}})", function_name),
    "uppercase" => format!("{}(${{1:s:string}})", function_name),
    "absolute_path" => format!("{}(${{1:path:string}})", function_name),
    "canonicalize" => format!("{}(${{1:path:string}})", function_name),
    "extension" => format!("{}(${{1:path:string}})", function_name),
    "file_name" => format!("{}(${{1:path:string}})", function_name),
    "file_stem" => format!("{}(${{1:path:string}})", function_name),
    "parent_directory" => format!("{}(${{1:path:string}})", function_name),
    "without_extension" => format!("{}(${{1:path:string}})", function_name),
    "clean" => format!("{}(${{1:path:string}})", function_name),
    "path_exists" => format!("{}(${{1:path:string}})", function_name),
    "read" => format!("{}(${{1:path:string}})", function_name),
    "blake3" => format!("{}(${{1:string:string}})", function_name),
    "blake3_file" => format!("{}(${{1:path:string}})", function_name),
    "sha256" => format!("{}(${{1:string:string}})", function_name),
    "sha256_file" => format!("{}(${{1:path:string}})", function_name),
    "datetime" => format!("{}(${{1:format:string}})", function_name),
    "datetime_utc" => format!("{}(${{1:format:string}})", function_name),
    "style" => format!("{}(${{1:name:string}})", function_name),
    "append" => {
      format!("{}(${{1:suffix:string}}, ${{2:s:string}})", function_name)
    }
    "prepend" => {
      format!("{}(${{1:prefix:string}}, ${{2:s:string}})", function_name)
    }
    "trim_end_match" => format!(
      "{}(${{1:s:string}}, ${{2:substring:string}})",
      function_name
    ),
    "trim_end_matches" => format!(
      "{}(${{1:s:string}}, ${{2:substring:string}})",
      function_name
    ),
    "trim_start_match" => format!(
      "{}(${{1:s:string}}, ${{2:substring:string}})",
      function_name
    ),
    "trim_start_matches" => format!(
      "{}(${{1:s:string}}, ${{2:substring:string}})",
      function_name
    ),
    "choose" => {
      format!("{}(${{1:n:string}}, ${{2:alphabet:string}})", function_name)
    }
    "semver_matches" => format!(
      "{}(${{1:version:string}}, ${{2:requirement:string}})",
      function_name
    ),
    "replace" => format!(
      "{}(${{1:s:string}}, ${{2:from:string}}, ${{3:to:string}})",
      function_name
    ),
    "replace_regex" => format!(
      "{}(${{1:s:string}}, ${{2:regex:string}}, ${{3:replacement:string}})",
      function_name
    ),
    "env" => format!(
      "{}(${{1:key:string}}${{2:, default:string}})",
      function_name
    ),
    "shell" => format!(
      "{}(${{1:command:string}}${{2:, args:string...}})",
      function_name
    ),
    "join" => format!(
      "{}(${{1:a:string}}, ${{2:b:string}}${{3:, more:string...}})",
      function_name
    ),
    _ => format!("{}(${{1:}})", function_name),
  }
}

pub(crate) fn get_function_documentation(
  name: &str,
  description: &str,
) -> String {
  let example = match name {
    "arch" => "arch() => \"x86_64\"",
    "num_cpus" => "num_cpus() => 8",
    "os" => "os() => \"linux\"",
    "os_family" => "os_family() => \"unix\"",
    "shell" => "shell(\"echo $1\", \"hello\") => \"hello\"",
    "env" => "env(\"HOME\") => \"/home/user\"\nenv(\"MISSING\", \"default\") => \"default\"",
    "require" => "require(\"bash\") => \"/bin/bash\"",
    "which" => "which(\"bash\") => \"/bin/bash\"\nwhich(\"nonexistent\") => \"\"",
    "is_dependency" => "is_dependency() => \"false\"",
    "invocation_directory" => "invocation_directory() => \"/path/to/current/dir\"",
    "justfile" => "justfile() => \"/path/to/justfile\"",
    "justfile_directory" => "justfile_directory() => \"/path/to\"",
    "just_executable" => "just_executable() => \"/usr/bin/just\"",
    "just_pid" => "just_pid() => \"12345\"",
    "append" => "append(\"/src\", \"foo bar\") => \"foo/src bar/src\"",
    "prepend" => "prepend(\"src/\", \"foo bar\") => \"src/foo src/bar\"",
    "encode_uri_component" => "encode_uri_component(\"a+b\") => \"a%2Bb\"",
    "quote" => "quote(\"hello 'world'\") => \"'hello \\'world\\''\"",
    "replace" => "replace(\"hello\", \"l\", \"x\") => \"hexxo\"",
    "replace_regex" => "replace_regex(\"hello\", \"[aeiou]\", \"X\") => \"hXllX\"",
    "trim" => "trim(\"  hello  \") => \"hello\"",
    "trim_end" => "trim_end(\"hello  \") => \"hello\"",
    "trim_start" => "trim_start(\"  hello\") => \"hello\"",
    "capitalize" => "capitalize(\"hello\") => \"Hello\"",
    "kebabcase" => "kebabcase(\"HelloWorld\") => \"hello-world\"",
    "lowercamelcase" => "lowercamelcase(\"hello_world\") => \"helloWorld\"",
    "lowercase" => "lowercase(\"Hello\") => \"hello\"",
    "uppercamelcase" => "uppercamelcase(\"hello_world\") => \"HelloWorld\"",
    "uppercase" => "uppercase(\"hello\") => \"HELLO\"",
    "absolute_path" => "absolute_path(\"./foo\") => \"/path/to/foo\"",
    "canonicalize" => "canonicalize(\"../foo/.\") => \"/path/to/foo\"",
    "extension" => "extension(\"foo.txt\") => \"txt\"",
    "file_name" => "file_name(\"/path/to/foo.txt\") => \"foo.txt\"",
    "file_stem" => "file_stem(\"/path/to/foo.txt\") => \"foo\"",
    "parent_directory" => "parent_directory(\"/path/to/foo.txt\") => \"/path/to\"",
    "without_extension" => "without_extension(\"/path/to/foo.txt\") => \"/path/to/foo\"",
    "clean" => "clean(\"foo//bar/../baz\") => \"foo/baz\"",
    "join" => "join(\"foo\", \"bar\", \"baz\") => \"foo/bar/baz\"",
    "path_exists" => "path_exists(\"/etc/passwd\") => \"true\"",
    "read" => "read(\"foo.txt\") => \"contents of foo.txt\"",
    "error" => "error(\"Something went wrong\") => *aborts execution*",
    "blake3" => "blake3(\"hello\") => \"a1744eeb6b921a9193df*...\"",
    "sha256" => "sha256(\"hello\") => \"2cf24dba5fb0a30e*...\"",
    "uuid" => "uuid() => \"f81d4fae-7dec-11d0-a765-00a0c91e6bf6\"",
    "choose" => "choose(\"5\", \"abcdef\") => \"bcafe\"",
    "datetime" => "datetime(\"%Y-%m-%d\") => \"2023-07-14\"",
    "semver_matches" => "semver_matches(\"1.2.3\", \">1.0.0\") => \"true\"",
    "style" => "style(\"error\") => \"\\e[31m\"",
    "home_directory" => "home_directory() => \"/home/user\"",
    _ => "",
  };

  let mut doc = String::new();

  doc.push_str(description);

  if !example.is_empty() {
    doc.push_str("\n\n**Examples:**\n```\n\n");
    doc.push_str(example);
    doc.push_str("\n```");
  }

  doc
}
