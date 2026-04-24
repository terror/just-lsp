use {super::*, indoc::indoc};

pub static BUILTINS: [Builtin<'_>; 150] = [
  Builtin::Attribute {
    name: "arg",
    description: indoc! {
      "
      Configure a recipe parameter.

      Accepts a parameter name followed by one or more keyword
      arguments that customize how the parameter is surfaced on the
      command line:

      - `help=\"HELP\"` sets the usage-message help string.
      - `long=\"LONG\"` requires the value to be passed with `--LONG`.
      - `short=\"S\"` requires the value to be passed with `-S`.
      - `value=\"VALUE\"` makes the option a flag that substitutes
        `VALUE` when present. Combine with `long` and/or `short`.
      - `pattern=\"PATTERN\"` requires the value to match a regular
        expression. Patterns are full-match; `just` rejects the
        invocation if the supplied value does not match.

      Multiple keys may be combined in a single `[arg(...)]`.

      ```just
      [arg(NAME, long=\"name\", short=\"n\", help=\"greeting target\")]
      greet NAME:
        @echo Hello, {{NAME}}
      ```
      "
    },
    targets: &[AttributeTarget::Recipe],
    min_args: 1,
    max_args: None,
  },
  Builtin::Attribute {
    name: "env",
    description: indoc! {
      "
      Set environment variable `ENV_VAR` to `VALUE` for the recipe.

      The variable is exported to the recipe's shell and to any commands
      run from backticks within it. Does not affect variables outside
      the recipe.

      ```just
      [env(\"RUST_LOG\", \"debug\")]
      test:
        cargo test
      ```
      "
    },
    targets: &[AttributeTarget::Recipe],
    min_args: 2,
    max_args: Some(2),
  },
  Builtin::Attribute {
    name: "confirm",
    description: indoc! {
      "
      Require confirmation in the terminal prior to executing the recipe.

      With no argument, uses the default prompt. Pass a single string
      to override with a custom prompt.

      Can be overridden by passing `--yes` to `just`, which
      auto-confirms any recipe marked with this attribute.

      Recipes that depend on a recipe requiring confirmation will not
      run if the confirmation is denied. Recipes listed on the command
      line after a confirmation-required recipe are also skipped if
      the confirmation is denied.

      ```just
      [confirm]
      delete-all:
        rm -rf *

      [confirm(\"Are you sure you want to delete everything?\")]
      delete-everything:
        rm -rf *
      ```
      "
    },
    targets: &[AttributeTarget::Recipe],
    min_args: 0,
    max_args: Some(1),
  },
  Builtin::Attribute {
    name: "default",
    description: indoc! {
      "
      Use this recipe as the module's default recipe.

      Running `just` with no recipe name in a module that contains a
      `[default]`-marked recipe will run that recipe, overriding the
      usual behavior of running the first recipe defined.

      ```just
      [default]
      build:
        cargo build
      ```
      "
    },
    targets: &[AttributeTarget::Recipe],
    min_args: 0,
    max_args: Some(0),
  },
  Builtin::Attribute {
    name: "doc",
    description: indoc! {
      "
      Set or suppress the recipe's or module's documentation comment.

      With no argument, any `#`-prefixed comment immediately above the
      recipe or `mod` statement is omitted from `just --list` and
      other doc surfaces. With a string argument, that string is used
      as the documentation instead of any comment above.

      ```just
      # This comment will not appear in --list output.
      [doc]
      internal:
        @echo internal

      [doc(\"Build the project in release mode\")]
      build:
        cargo build --release
      ```
      "
    },
    targets: &[AttributeTarget::Module, AttributeTarget::Recipe],
    min_args: 0,
    max_args: Some(1),
  },
  Builtin::Attribute {
    name: "extension",
    description: indoc! {
      "
      Set the file extension used when writing a shebang recipe's script
      to a temporary file.

      `EXT` should include the leading period if one is desired (for
      example `\".py\"`). This affects only how the temp file is named,
      which can matter for interpreters that dispatch on extension.

      ```just
      [extension(\".py\")]
      hello:
        #!/usr/bin/env python3
        print(\"hello\")
      ```
      "
    },
    targets: &[AttributeTarget::Recipe],
    min_args: 1,
    max_args: Some(1),
  },
  Builtin::Attribute {
    name: "group",
    description: indoc! {
      "
      Place the recipe or module in the named group `NAME`.

      Groups are used by `just --list` to visually cluster related
      recipes. `[group(...)]` may be repeated on the same recipe or
      module to place it in multiple groups.

      ```just
      [group(\"build\")]
      compile:
        cargo build

      [group(\"build\")]
      [group(\"release\")]
      package:
        cargo build --release
      ```
      "
    },
    targets: &[AttributeTarget::Module, AttributeTarget::Recipe],
    min_args: 1,
    max_args: Some(1),
  },
  Builtin::Attribute {
    name: "metadata",
    description: indoc! {
      "
      Attach arbitrary metadata `METADATA` to the recipe.

      The attribute accepts any number of arguments. `just` does not
      interpret them; they are surfaced via `just --dump --dump-format
      json` and intended for consumption by external tooling.

      ```just
      [metadata(\"key1=value1\", \"key2=value2\")]
      build:
        cargo build
      ```
      "
    },
    targets: &[AttributeTarget::Recipe],
    min_args: 1,
    max_args: None,
  },
  Builtin::Attribute {
    name: "dragonfly",
    description: indoc! {
      "
      Enable the recipe on DragonFly BSD.

      Part of the platform-gating family of attributes
      (`[linux]`, `[macos]`, `[unix]`, `[windows]`, `[freebsd]`,
      `[openbsd]`, `[netbsd]`, `[dragonfly]`). When any platform
      attribute is present, the recipe is only enabled when one of the
      active platforms matches.

      ```just
      [dragonfly]
      install:
        pkg install myapp
      ```
      "
    },
    targets: &[AttributeTarget::Recipe],
    min_args: 0,
    max_args: Some(0),
  },
  Builtin::Attribute {
    name: "freebsd",
    description: indoc! {
      "
      Enable the recipe on FreeBSD.

      Part of the platform-gating family of attributes. When any
      platform attribute is present, the recipe is only enabled when
      one of the active platforms matches.

      ```just
      [freebsd]
      install:
        pkg install myapp
      ```
      "
    },
    targets: &[AttributeTarget::Recipe],
    min_args: 0,
    max_args: Some(0),
  },
  Builtin::Attribute {
    name: "linux",
    description: indoc! {
      "
      Enable the recipe on Linux.

      Part of the platform-gating family of attributes
      (`[linux]`, `[macos]`, `[unix]`, `[windows]`, and the BSDs). When
      any platform attribute is present, the recipe is only enabled
      when one of the active platforms matches. Useful for writing
      cross-platform justfiles that dispatch on the host OS.

      ```just
      [linux]
      run:
        cc main.c && ./a.out

      [windows]
      run:
        cl main.c && main.exe
      ```
      "
    },
    targets: &[AttributeTarget::Recipe],
    min_args: 0,
    max_args: Some(0),
  },
  Builtin::Attribute {
    name: "macos",
    description: indoc! {
      "
      Enable the recipe on macOS.

      Part of the platform-gating family of attributes. When any
      platform attribute is present, the recipe is only enabled when
      one of the active platforms matches.

      Note that `[unix]` also enables the recipe on macOS.

      ```just
      [macos]
      open target:
        open {{target}}
      ```
      "
    },
    targets: &[AttributeTarget::Recipe],
    min_args: 0,
    max_args: Some(0),
  },
  Builtin::Attribute {
    name: "no-cd",
    description: indoc! {
      "
      Don't change directory before executing the recipe.

      Normally `just` runs recipes with the current directory set to
      the directory containing the `justfile`. With `[no-cd]`, the
      recipe runs with the current directory unchanged, so it can use
      paths relative to the invocation directory or operate on the
      user's current directory.

      ```just
      [no-cd]
      commit file:
        git add {{file}}
        git commit
      ```
      "
    },
    targets: &[AttributeTarget::Recipe],
    min_args: 0,
    max_args: Some(0),
  },
  Builtin::Attribute {
    name: "exit-message",
    description: indoc! {
      "
      Print an error message if the recipe fails.

      The inverse of `[no-exit-message]`: forces `just` to emit its
      standard failure message even if `set no-exit-message` is active
      globally.
      "
    },
    targets: &[AttributeTarget::Recipe],
    min_args: 0,
    max_args: Some(0),
  },
  Builtin::Attribute {
    name: "no-exit-message",
    description: indoc! {
      "
      Don't print an error message if the recipe fails.

      `just` normally prints a line like `error: Recipe \\\"foo\\\" failed
      with exit code 1` when a recipe exits non-zero. This attribute
      suppresses that message for the annotated recipe, which is useful
      when the recipe already prints its own, more specific error
      output.

      ```just
      [no-exit-message]
      test:
        ./run-tests.sh
      ```
      "
    },
    targets: &[AttributeTarget::Recipe],
    min_args: 0,
    max_args: Some(0),
  },
  Builtin::Attribute {
    name: "no-quiet",
    description: indoc! {
      "
      Override globally quiet recipes and always echo the recipe lines.

      Cancels the effect of `set quiet := true` or a leading `@` on the
      recipe itself. Useful when the global default is to suppress echo
      but you want this particular recipe to be visible.

      ```just
      set quiet := true

      [no-quiet]
      build:
        cargo build
      ```
      "
    },
    targets: &[AttributeTarget::Recipe],
    min_args: 0,
    max_args: Some(0),
  },
  Builtin::Attribute {
    name: "netbsd",
    description: indoc! {
      "
      Enable the recipe on NetBSD.

      Part of the platform-gating family of attributes. When any
      platform attribute is present, the recipe is only enabled when
      one of the active platforms matches.
      "
    },
    targets: &[AttributeTarget::Recipe],
    min_args: 0,
    max_args: Some(0),
  },
  Builtin::Attribute {
    name: "openbsd",
    description: indoc! {
      "
      Enable the recipe on OpenBSD.

      Part of the platform-gating family of attributes. When any
      platform attribute is present, the recipe is only enabled when
      one of the active platforms matches.
      "
    },
    targets: &[AttributeTarget::Recipe],
    min_args: 0,
    max_args: Some(0),
  },
  Builtin::Attribute {
    name: "parallel",
    description: indoc! {
      "
      Run this recipe's dependencies in parallel.

      By default, a recipe's dependencies are executed serially in the
      order they appear. `[parallel]` fans them out so they run
      concurrently. The recipe body itself runs only after all
      dependencies complete.

      ```just
      [parallel]
      check: check-formatting check-lints check-tests
      ```
      "
    },
    targets: &[AttributeTarget::Recipe],
    min_args: 0,
    max_args: Some(0),
  },
  Builtin::Attribute {
    name: "positional-arguments",
    description: indoc! {
      "
      Turn on positional arguments for this recipe.

      Recipe arguments are passed to the shell as positional parameters
      (`$0`, `$1`, `$@`, ...) instead of being interpolated as
      `{{ name }}`. The recipe name is available as `$0` for linewise
      recipes.

      Note that PowerShell does not handle positional arguments the
      same way as POSIX shells; enabling this with PowerShell as the
      configured shell will likely break the recipe.

      ```just
      [positional-arguments]
      @foo bar:
        echo $0
        echo $1
      ```
      "
    },
    targets: &[AttributeTarget::Recipe],
    min_args: 0,
    max_args: Some(0),
  },
  Builtin::Attribute {
    name: "private",
    description: indoc! {
      "
      Make a recipe, alias, variable, or module private.

      Private items are hidden from `just --list` and `just
      --summary`, but are still callable by name and usable as
      dependencies of other recipes. A leading underscore on the name
      has the same effect.

      ```just
      [private]
      _helper:
        @echo internal
      ```
      "
    },
    targets: &[
      AttributeTarget::Alias,
      AttributeTarget::Assignment,
      AttributeTarget::Module,
      AttributeTarget::Recipe,
    ],
    min_args: 0,
    max_args: Some(0),
  },
  Builtin::Attribute {
    name: "script",
    description: indoc! {
      "
      Execute the recipe as a script.

      With no argument, uses the interpreter configured by
      `set script-interpreter := [...]` (default: `['sh', '-eu']`).
      With an argument, uses the supplied `COMMAND` as the
      interpreter, bypassing the `script-interpreter` setting.

      Instead of running each line separately through the shell, the
      entire recipe body is written to a temporary file and passed to
      the interpreter as a single script. This makes multi-line
      control flow, here-docs, and shell-specific features work as
      written.

      ```just
      set script-interpreter := ['bash', '-eu']

      [script]
      hello:
        for i in 1 2 3; do
          echo $i
        done

      [script(\"python3\")]
      hello-py:
        print(\"hello from python\")
      ```
      "
    },
    targets: &[AttributeTarget::Recipe],
    min_args: 0,
    max_args: None,
  },
  Builtin::Attribute {
    name: "unix",
    description: indoc! {
      "
      Enable the recipe on Unix-like platforms, including macOS.

      Part of the platform-gating family of attributes. When any
      platform attribute is present, the recipe is only enabled when
      one of the active platforms matches. `[unix]` matches Linux,
      macOS, and the BSDs.

      ```just
      [unix]
      run:
        cc main.c && ./a.out
      ```
      "
    },
    targets: &[AttributeTarget::Recipe],
    min_args: 0,
    max_args: Some(0),
  },
  Builtin::Attribute {
    name: "windows",
    description: indoc! {
      "
      Enable the recipe on Windows.

      Part of the platform-gating family of attributes. When any
      platform attribute is present, the recipe is only enabled when
      one of the active platforms matches.

      ```just
      [windows]
      run:
        cl main.c && main.exe
      ```
      "
    },
    targets: &[AttributeTarget::Recipe],
    min_args: 0,
    max_args: Some(0),
  },
  Builtin::Attribute {
    name: "working-directory",
    description: indoc! {
      "
      Set the recipe's working directory to `PATH`.

      `PATH` may be relative or absolute. If relative, it is
      interpreted relative to the default working directory (the
      directory containing the `justfile`, unless overridden).

      ```just
      [working-directory(\"./subproject\")]
      build:
        cargo build
      ```
      "
    },
    targets: &[AttributeTarget::Recipe],
    min_args: 1,
    max_args: Some(1),
  },
  Builtin::Constant {
    name: "HEX",
    description: indoc! {
      "
      Lowercase hexadecimal digit string: `\"0123456789abcdef\"`.

      Useful as the alphabet argument to `choose()` for generating
      random hex strings.

      ```just
      token := choose('32', HEX)
      ```
      "
    },
  },
  Builtin::Constant {
    name: "HEXLOWER",
    description: indoc! {
      "
      Explicitly lowercase hexadecimal digit string:
      `\"0123456789abcdef\"`.

      Identical to `HEX`. Prefer this name when paired with `HEXUPPER`
      to make the case intent obvious at the call site.
      "
    },
  },
  Builtin::Constant {
    name: "HEXUPPER",
    description: indoc! {
      "
      Uppercase hexadecimal digit string: `\"0123456789ABCDEF\"`.

      Useful as the alphabet argument to `choose()` for generating
      uppercase-hex strings.
      "
    },
  },
  Builtin::Constant {
    name: "PATH_SEP",
    description: indoc! {
      "
      Native path separator: `/` on Unix, `\\` on Windows.

      Use when constructing paths that must use the host's conventional
      separator.
      "
    },
  },
  Builtin::Constant {
    name: "PATH_VAR_SEP",
    description: indoc! {
      "
      Native `PATH`-variable list separator: `:` on Unix, `;` on
      Windows.

      Use when building colon- or semicolon-delimited path lists for
      environment variables like `$PATH`.
      "
    },
  },
  Builtin::Constant {
    name: "CLEAR",
    description: indoc! {
      "
      ANSI escape sequence that clears the terminal screen, similar to
      the `clear` command: `\\ec`.
      "
    },
  },
  Builtin::Constant {
    name: "NORMAL",
    description: indoc! {
      "
      ANSI escape sequence that resets all terminal display attributes:
      `\\e[0m`.

      Use at the end of a styled segment to return the terminal to its
      default colors and weights.

      ```just
      @greet:
        echo '{{BOLD}}{{RED}}danger{{NORMAL}}'
      ```
      "
    },
  },
  Builtin::Constant {
    name: "BOLD",
    description: indoc! {
      "
      ANSI escape sequence for bold text: `\\e[1m`.

      Combine with color constants and terminate with `NORMAL`.
      "
    },
  },
  Builtin::Constant {
    name: "ITALIC",
    description: indoc! {
      "
      ANSI escape sequence for italic text: `\\e[3m`.

      Support for italic rendering varies by terminal.
      "
    },
  },
  Builtin::Constant {
    name: "UNDERLINE",
    description: indoc! {
      "
      ANSI escape sequence for underlined text: `\\e[4m`.
      "
    },
  },
  Builtin::Constant {
    name: "INVERT",
    description: indoc! {
      "
      ANSI escape sequence that swaps foreground and background colors:
      `\\e[7m`.
      "
    },
  },
  Builtin::Constant {
    name: "HIDE",
    description: indoc! {
      "
      ANSI escape sequence for hidden (concealed) text: `\\e[8m`.

      Useful for sensitive output like passwords, though not a
      substitute for proper secret handling.
      "
    },
  },
  Builtin::Constant {
    name: "STRIKETHROUGH",
    description: indoc! {
      "
      ANSI escape sequence for strikethrough text: `\\e[9m`.
      "
    },
  },
  Builtin::Constant {
    name: "BLACK",
    description: indoc! {
      "
      ANSI escape sequence for black foreground text: `\\e[30m`.

      Terminate styled output with `NORMAL` to reset.
      "
    },
  },
  Builtin::Constant {
    name: "RED",
    description: indoc! {
      "
      ANSI escape sequence for red foreground text: `\\e[31m`.

      Terminate styled output with `NORMAL` to reset.
      "
    },
  },
  Builtin::Constant {
    name: "GREEN",
    description: indoc! {
      "
      ANSI escape sequence for green foreground text: `\\e[32m`.

      Terminate styled output with `NORMAL` to reset.
      "
    },
  },
  Builtin::Constant {
    name: "YELLOW",
    description: indoc! {
      "
      ANSI escape sequence for yellow foreground text: `\\e[33m`.

      Terminate styled output with `NORMAL` to reset.
      "
    },
  },
  Builtin::Constant {
    name: "BLUE",
    description: indoc! {
      "
      ANSI escape sequence for blue foreground text: `\\e[34m`.

      Terminate styled output with `NORMAL` to reset.
      "
    },
  },
  Builtin::Constant {
    name: "MAGENTA",
    description: indoc! {
      "
      ANSI escape sequence for magenta foreground text: `\\e[35m`.

      Terminate styled output with `NORMAL` to reset.
      "
    },
  },
  Builtin::Constant {
    name: "CYAN",
    description: indoc! {
      "
      ANSI escape sequence for cyan foreground text: `\\e[36m`.

      Terminate styled output with `NORMAL` to reset.
      "
    },
  },
  Builtin::Constant {
    name: "WHITE",
    description: indoc! {
      "
      ANSI escape sequence for white foreground text: `\\e[37m`.

      Terminate styled output with `NORMAL` to reset.
      "
    },
  },
  Builtin::Constant {
    name: "BG_BLACK",
    description: indoc! {
      "
      ANSI escape sequence for black background: `\\e[40m`.
      "
    },
  },
  Builtin::Constant {
    name: "BG_RED",
    description: indoc! {
      "
      ANSI escape sequence for red background: `\\e[41m`.
      "
    },
  },
  Builtin::Constant {
    name: "BG_GREEN",
    description: indoc! {
      "
      ANSI escape sequence for green background: `\\e[42m`.
      "
    },
  },
  Builtin::Constant {
    name: "BG_YELLOW",
    description: indoc! {
      "
      ANSI escape sequence for yellow background: `\\e[43m`.
      "
    },
  },
  Builtin::Constant {
    name: "BG_BLUE",
    description: indoc! {
      "
      ANSI escape sequence for blue background: `\\e[44m`.
      "
    },
  },
  Builtin::Constant {
    name: "BG_MAGENTA",
    description: indoc! {
      "
      ANSI escape sequence for magenta background: `\\e[45m`.
      "
    },
  },
  Builtin::Constant {
    name: "BG_CYAN",
    description: indoc! {
      "
      ANSI escape sequence for cyan background: `\\e[46m`.
      "
    },
  },
  Builtin::Constant {
    name: "BG_WHITE",
    description: indoc! {
      "
      ANSI escape sequence for white background: `\\e[47m`.
      "
    },
  },
  Builtin::Function {
    name: "absolute_path",
    aliases: &[],
    description: indoc! {
      "
      Return the absolute form of `path`, resolved against the current
      working directory. Does not follow symlinks or canonicalize. For
      that, use `canonicalize()`.

      ```just
      absolute_path(\"./bar.txt\")  # in /foo -> \"/foo/bar.txt\"
      ```
      "
    },
    required_arguments: 1,
    accepts_variadic: false,
    deprecated: None,
  },
  Builtin::Function {
    name: "append",
    aliases: &[],
    description: indoc! {
      "
      Append `suffix` to each whitespace-separated token in `s`.

      ```just
      append(\"/src\", \"foo bar baz\")  # => \"foo/src bar/src baz/src\"
      ```
      "
    },
    required_arguments: 2,
    accepts_variadic: false,
    deprecated: None,
  },
  Builtin::Function {
    name: "arch",
    aliases: &[],
    description: indoc! {
      "
      Instruction set architecture of the host machine.

      Returns one of: `aarch64`, `arm`, `asmjs`, `hexagon`, `mips`,
      `msp430`, `powerpc`, `powerpc64`, `s390x`, `sparc`, `wasm32`,
      `x86`, `x86_64`, or `xcore`.

      ```just
      system-info:
        @echo This is an {{arch()}} machine.
      ```
      "
    },
    required_arguments: 0,
    accepts_variadic: false,
    deprecated: None,
  },
  Builtin::Function {
    name: "blake3",
    aliases: &[],
    description: indoc! {
      "
      Return the BLAKE3 hash of `string` as a lowercase hex string.

      ```just
      blake3(\"hello\")  # => \"ea8f163db38682925e4491c5e58d4bb...\"
      ```
      "
    },
    required_arguments: 1,
    accepts_variadic: false,
    deprecated: None,
  },
  Builtin::Function {
    name: "blake3_file",
    aliases: &[],
    description: indoc! {
      "
      Return the BLAKE3 hash of the file at `path` as a lowercase hex
      string. Aborts if the file cannot be read.
      "
    },
    required_arguments: 1,
    accepts_variadic: false,
    deprecated: None,
  },
  Builtin::Function {
    name: "cache_directory",
    aliases: &["cache_dir"],
    description: indoc! {
      "
      User-specific cache directory.

      On Unix, follows the XDG Base Directory Specification
      (`$XDG_CACHE_HOME` or `$HOME/.cache`). On macOS, returns
      `~/Library/Caches`. On Windows, returns
      `{FOLDERID_LocalAppData}`.
      "
    },
    required_arguments: 0,
    accepts_variadic: false,
    deprecated: None,
  },
  Builtin::Function {
    name: "canonicalize",
    aliases: &[],
    description: indoc! {
      "
      Canonicalize `path` by resolving symlinks and removing `.`,
      `..`, and redundant path separators where possible. Aborts if
      the path does not exist.

      ```just
      canonicalize(\"../foo/./bar\")  # => \"/absolute/path/to/bar\"
      ```
      "
    },
    required_arguments: 1,
    accepts_variadic: false,
    deprecated: None,
  },
  Builtin::Function {
    name: "capitalize",
    aliases: &[],
    description: indoc! {
      "
      Return `s` with the first character uppercased and the rest
      lowercased.

      ```just
      capitalize(\"hello WORLD\")  # => \"Hello world\"
      ```
      "
    },
    required_arguments: 1,
    accepts_variadic: false,
    deprecated: None,
  },
  Builtin::Function {
    name: "choose",
    aliases: &[],
    description: indoc! {
      "
      Return a string of `n` randomly selected characters from
      `alphabet`. `alphabet` may not contain repeated characters.

      ```just
      choose('64', HEX)  # 64-character random lowercase hex string
      ```
      "
    },
    required_arguments: 2,
    accepts_variadic: false,
    deprecated: None,
  },
  Builtin::Function {
    name: "clean",
    aliases: &[],
    description: indoc! {
      "
      Simplify `path` by removing extra path separators, intermediate
      `.` components, and `..` where possible. Purely lexical; does not
      touch the filesystem.

      ```just
      clean(\"foo//bar/../baz\")  # => \"foo/baz\"
      clean(\"foo/./bar\")       # => \"foo/bar\"
      ```
      "
    },
    required_arguments: 1,
    accepts_variadic: false,
    deprecated: None,
  },
  Builtin::Function {
    name: "config_directory",
    aliases: &["config_dir"],
    description: indoc! {
      "
      User-specific configuration directory.

      On Unix, follows the XDG Base Directory Specification
      (`$XDG_CONFIG_HOME` or `$HOME/.config`). On macOS, returns
      `~/Library/Application Support`. On Windows, returns
      `{FOLDERID_RoamingAppData}`.
      "
    },
    required_arguments: 0,
    accepts_variadic: false,
    deprecated: None,
  },
  Builtin::Function {
    name: "config_local_directory",
    aliases: &["config_local_dir"],
    description: indoc! {
      "
      Local user-specific configuration directory, for configuration
      that should not roam or sync between machines.

      On Unix, follows XDG conventions. On Windows, returns
      `{FOLDERID_LocalAppData}`.
      "
    },
    required_arguments: 0,
    accepts_variadic: false,
    deprecated: None,
  },
  Builtin::Function {
    name: "data_directory",
    aliases: &["data_dir"],
    description: indoc! {
      "
      User-specific data directory.

      On Unix, follows the XDG Base Directory Specification
      (`$XDG_DATA_HOME` or `$HOME/.local/share`). On macOS, returns
      `~/Library/Application Support`. On Windows, returns
      `{FOLDERID_RoamingAppData}`.
      "
    },
    required_arguments: 0,
    accepts_variadic: false,
    deprecated: None,
  },
  Builtin::Function {
    name: "data_local_directory",
    aliases: &["data_local_dir"],
    description: indoc! {
      "
      Local user-specific data directory, for data that should not
      roam or sync between machines.

      On Unix, follows XDG conventions. On Windows, returns
      `{FOLDERID_LocalAppData}`.
      "
    },
    required_arguments: 0,
    accepts_variadic: false,
    deprecated: None,
  },
  Builtin::Function {
    name: "datetime",
    aliases: &[],
    description: indoc! {
      "
      Return the current local time formatted with `format`.

      `format` is a `strftime`-style string. See the
      [`chrono` docs](https://docs.rs/chrono/latest/chrono/format/strftime/index.html)
      for the full set of specifiers.

      ```just
      datetime(\"%Y-%m-%d\")  # => \"2026-04-23\"
      ```
      "
    },
    required_arguments: 1,
    accepts_variadic: false,
    deprecated: None,
  },
  Builtin::Function {
    name: "datetime_utc",
    aliases: &[],
    description: indoc! {
      "
      Return the current UTC time formatted with `format`.

      `format` is a `strftime`-style string. See the
      [`chrono` docs](https://docs.rs/chrono/latest/chrono/format/strftime/index.html)
      for the full set of specifiers.
      "
    },
    required_arguments: 1,
    accepts_variadic: false,
    deprecated: None,
  },
  Builtin::Function {
    name: "encode_uri_component",
    aliases: &[],
    description: indoc! {
      "
      Percent-encode every character in `s` except
      `[A-Za-z0-9_.!~*'()-]`. Matches the behavior of JavaScript's
      `encodeURIComponent`.

      ```just
      encode_uri_component(\"hello world!\")  # => \"hello%20world!\"
      ```
      "
    },
    required_arguments: 1,
    accepts_variadic: false,
    deprecated: None,
  },
  Builtin::Function {
    name: "env",
    aliases: &[],
    description: indoc! {
      "
      Retrieve the environment variable named `key`.

      Called with one argument, aborts execution if the variable is
      unset. Called with two arguments, returns `default` when the
      variable is unset.

      A default can be substituted for an *empty* value (not just an
      unset one) with the `||` operator, currently unstable:

      ```just
      set unstable
      foo := env('FOO', '') || 'DEFAULT_VALUE'
      ```
      "
    },
    required_arguments: 1,
    accepts_variadic: true,
    deprecated: None,
  },
  Builtin::Function {
    name: "env_var",
    aliases: &[],
    description: indoc! {
      "
      **Deprecated**: use `env(key)` instead.

      Retrieve the environment variable named `key`. Aborts if the
      variable is unset.
      "
    },
    required_arguments: 1,
    accepts_variadic: false,
    deprecated: Some("env"),
  },
  Builtin::Function {
    name: "env_var_or_default",
    aliases: &[],
    description: indoc! {
      "
      **Deprecated**: use `env(key, default)` instead.

      Retrieve the environment variable named `key`, returning
      `default` when the variable is unset.
      "
    },
    required_arguments: 2,
    accepts_variadic: false,
    deprecated: Some("env"),
  },
  Builtin::Function {
    name: "error",
    aliases: &[],
    description: indoc! {
      "
      Abort execution and report `message` to the user. Diverges and
      never returns a value.

      ```just
      check-flag:
        @echo {{ if flag == \"\" { error(\"flag must be set\") } else { flag } }}
      ```
      "
    },
    required_arguments: 1,
    accepts_variadic: false,
    deprecated: None,
  },
  Builtin::Function {
    name: "executable_directory",
    aliases: &["executable_dir"],
    description: indoc! {
      "
      User-specific executable directory.

      On Unix, follows the XDG Base Directory Specification
      (`$XDG_BIN_HOME` or `$HOME/.local/bin`). On Windows, returns no
      well-defined value, so use with care on that platform.
      "
    },
    required_arguments: 0,
    accepts_variadic: false,
    deprecated: None,
  },
  Builtin::Function {
    name: "extension",
    aliases: &[],
    description: indoc! {
      "
      Return the file extension of `path`, not including the leading
      period. Aborts if `path` has no extension.

      ```just
      extension(\"/foo/bar.txt\")  # => \"txt\"
      ```
      "
    },
    required_arguments: 1,
    accepts_variadic: false,
    deprecated: None,
  },
  Builtin::Function {
    name: "file_name",
    aliases: &[],
    description: indoc! {
      "
      Return the file name of `path` with any leading directory
      components removed.

      ```just
      file_name(\"/foo/bar.txt\")  # => \"bar.txt\"
      ```
      "
    },
    required_arguments: 1,
    accepts_variadic: false,
    deprecated: None,
  },
  Builtin::Function {
    name: "file_stem",
    aliases: &[],
    description: indoc! {
      "
      Return the file name of `path` without its extension or any
      leading directory components.

      ```just
      file_stem(\"/foo/bar.txt\")  # => \"bar\"
      ```
      "
    },
    required_arguments: 1,
    accepts_variadic: false,
    deprecated: None,
  },
  Builtin::Function {
    name: "home_directory",
    aliases: &["home_dir"],
    description: indoc! {
      "
      The user's home directory.

      On Unix, returns `$HOME`. On macOS, returns `$HOME`. On Windows,
      returns `{FOLDERID_Profile}`.
      "
    },
    required_arguments: 0,
    accepts_variadic: false,
    deprecated: None,
  },
  Builtin::Function {
    name: "invocation_directory",
    aliases: &["invocation_dir"],
    description: indoc! {
      "
      The absolute path of the directory in which `just` was invoked,
      before it `chdir`'d to the justfile directory.

      On Windows, paths are converted to Cygwin-style forward-slash
      form via `cygpath`. Use `invocation_directory_native()` to keep
      the native path on all platforms.

      Useful when a recipe needs to operate on files relative to where
      the user ran `just` rather than where the justfile lives.

      ```just
      rustfmt:
        find {{invocation_directory()}} -name '*.rs' -exec rustfmt {} \\\\;
      ```
      "
    },
    required_arguments: 0,
    accepts_variadic: false,
    deprecated: None,
  },
  Builtin::Function {
    name: "invocation_directory_native",
    aliases: &["invocation_dir_native"],
    description: indoc! {
      "
      The absolute path of the directory in which `just` was invoked,
      in the host's native path format.

      Unlike `invocation_directory()`, this does not convert paths to
      Cygwin style on Windows.
      "
    },
    required_arguments: 0,
    accepts_variadic: false,
    deprecated: None,
  },
  Builtin::Function {
    name: "is_dependency",
    aliases: &[],
    description: indoc! {
      "
      Return the string `\"true\"` if the current recipe is being run
      as a dependency of another recipe, and `\"false\"` if it was
      invoked directly from the command line.
      "
    },
    required_arguments: 0,
    accepts_variadic: false,
    deprecated: None,
  },
  Builtin::Function {
    name: "join",
    aliases: &[],
    description: indoc! {
      "
      Join two or more path components.

      Uses `/` on Unix and `\\` on Windows, which can lead to surprises
      in cross-platform justfiles. Prefer the `/` operator (`a / b`),
      which always uses `/`, unless Windows backslashes are
      specifically desired.

      ```just
      join(\"foo/bar\", \"baz\")  # => \"foo/bar/baz\"
      ```
      "
    },
    required_arguments: 2,
    accepts_variadic: true,
    deprecated: None,
  },
  Builtin::Function {
    name: "just_executable",
    aliases: &[],
    description: indoc! {
      "
      Absolute path to the `just` executable that is currently running.

      ```just
      executable:
        @echo just is at {{just_executable()}}
      ```
      "
    },
    required_arguments: 0,
    accepts_variadic: false,
    deprecated: None,
  },
  Builtin::Function {
    name: "just_pid",
    aliases: &[],
    description: indoc! {
      "
      Process ID of the running `just` executable, as a decimal string.
      "
    },
    required_arguments: 0,
    accepts_variadic: false,
    deprecated: None,
  },
  Builtin::Function {
    name: "justfile",
    aliases: &[],
    description: indoc! {
      "
      Absolute path to the current `justfile`.

      In submodules and imports, still refers to the *root* justfile.
      Use `source_file()` to get the path of the file currently being
      evaluated.
      "
    },
    required_arguments: 0,
    accepts_variadic: false,
    deprecated: None,
  },
  Builtin::Function {
    name: "justfile_directory",
    aliases: &["justfile_dir"],
    description: indoc! {
      "
      Absolute path to the parent directory of the current `justfile`.

      ```just
      script:
        {{justfile_directory()}}/scripts/deploy
      ```
      "
    },
    required_arguments: 0,
    accepts_variadic: false,
    deprecated: None,
  },
  Builtin::Function {
    name: "kebabcase",
    aliases: &[],
    description: indoc! {
      "
      Convert `s` to `kebab-case`.

      ```just
      kebabcase(\"helloWorld\")  # => \"hello-world\"
      ```
      "
    },
    required_arguments: 1,
    accepts_variadic: false,
    deprecated: None,
  },
  Builtin::Function {
    name: "lowercase",
    aliases: &[],
    description: indoc! {
      "
      Convert `s` to lowercase.

      ```just
      lowercase(\"Hello\")  # => \"hello\"
      ```
      "
    },
    required_arguments: 1,
    accepts_variadic: false,
    deprecated: None,
  },
  Builtin::Function {
    name: "lowercamelcase",
    aliases: &[],
    description: indoc! {
      "
      Convert `s` to `lowerCamelCase`.

      ```just
      lowercamelcase(\"hello_world\")  # => \"helloWorld\"
      ```
      "
    },
    required_arguments: 1,
    accepts_variadic: false,
    deprecated: None,
  },
  Builtin::Function {
    name: "num_cpus",
    aliases: &[],
    description: indoc! {
      "
      Number of logical CPUs available on the host machine.

      ```just
      build:
        make -j{{num_cpus()}}
      ```
      "
    },
    required_arguments: 0,
    accepts_variadic: false,
    deprecated: None,
  },
  Builtin::Function {
    name: "os",
    aliases: &[],
    description: indoc! {
      "
      Host operating system.

      Returns one of: `android`, `bitrig`, `dragonfly`, `emscripten`,
      `freebsd`, `haiku`, `ios`, `linux`, `macos`, `netbsd`, `openbsd`,
      `solaris`, or `windows`.
      "
    },
    required_arguments: 0,
    accepts_variadic: false,
    deprecated: None,
  },
  Builtin::Function {
    name: "os_family",
    aliases: &[],
    description: indoc! {
      "
      Host operating system family. Returns `unix` or `windows`.

      Useful for gating logic in cross-platform justfiles.

      ```just
      run:
        {{if os_family() == \"windows\" { \"main.exe\" } else { \"./main\" }}}
      ```
      "
    },
    required_arguments: 0,
    accepts_variadic: false,
    deprecated: None,
  },
  Builtin::Function {
    name: "parent_directory",
    aliases: &["parent_dir"],
    description: indoc! {
      "
      Return the parent directory of `path`. Aborts if `path` has no
      parent (e.g. the filesystem root).

      ```just
      parent_directory(\"/foo/bar.txt\")  # => \"/foo\"
      ```
      "
    },
    required_arguments: 1,
    accepts_variadic: false,
    deprecated: None,
  },
  Builtin::Function {
    name: "path_exists",
    aliases: &[],
    description: indoc! {
      "
      Return `\"true\"` if `path` points at an existing filesystem
      entity, `\"false\"` otherwise.

      Symbolic links are traversed. Returns `\"false\"` for broken
      symlinks or when the path is inaccessible.
      "
    },
    required_arguments: 1,
    accepts_variadic: false,
    deprecated: None,
  },
  Builtin::Function {
    name: "prepend",
    aliases: &[],
    description: indoc! {
      "
      Prepend `prefix` to each whitespace-separated token in `s`.

      ```just
      prepend(\"src/\", \"foo bar baz\")  # => \"src/foo src/bar src/baz\"
      ```
      "
    },
    required_arguments: 2,
    accepts_variadic: false,
    deprecated: None,
  },
  Builtin::Function {
    name: "quote",
    aliases: &[],
    description: indoc! {
      "
      Quote `s` for safe use as a single argument in a POSIX shell.

      Replaces every single quote with `'\\''` and surrounds the
      result in single quotes. Sufficient for `sh` and most
      descendants (`bash`, `zsh`, `dash`, ...).

      ```just
      quote(\"hello 'world'\")  # => \"'hello '\\\\''world'\\\\'''\"
      ```
      "
    },
    required_arguments: 1,
    accepts_variadic: false,
    deprecated: None,
  },
  Builtin::Function {
    name: "read",
    aliases: &[],
    description: indoc! {
      "
      Return the contents of the file at `path` as a string. Aborts if
      the file cannot be read.
      "
    },
    required_arguments: 1,
    accepts_variadic: false,
    deprecated: None,
  },
  Builtin::Function {
    name: "replace",
    aliases: &[],
    description: indoc! {
      "
      Replace every occurrence of `from` in `s` with `to`.

      ```just
      replace(\"hello\", \"l\", \"x\")  # => \"hexxo\"
      ```
      "
    },
    required_arguments: 3,
    accepts_variadic: false,
    deprecated: None,
  },
  Builtin::Function {
    name: "replace_regex",
    aliases: &[],
    description: indoc! {
      "
      Replace every match of `regex` in `s` with `replacement`.

      Regex syntax and replacement syntax are provided by the Rust
      [`regex`](https://docs.rs/regex) crate. Capture groups are
      supported; see the crate's
      [replacement syntax](https://docs.rs/regex/latest/regex/struct.Regex.html#replacement-string-syntax)
      docs for details.

      ```just
      replace_regex(\"hello\", \"[aeiou]\", \"X\")  # => \"hXllX\"
      ```
      "
    },
    required_arguments: 3,
    accepts_variadic: false,
    deprecated: None,
  },
  Builtin::Function {
    name: "require",
    aliases: &[],
    description: indoc! {
      "
      Search the directories in `$PATH` for an executable called
      `name` and return its full path. Aborts with an error if no
      such executable is found.

      ```just
      bash := require(\"bash\")

      @test:
        echo using {{bash}}
      ```
      "
    },
    required_arguments: 1,
    accepts_variadic: false,
    deprecated: None,
  },
  Builtin::Function {
    name: "runtime_directory",
    aliases: &["runtime_dir"],
    description: indoc! {
      "
      The user-specific runtime directory.

      Follows the XDG Base Directory Specification on Unix; returns
      the platform-specified runtime directory on other systems. May
      be unset on some platforms, in which case the function aborts.
      "
    },
    required_arguments: 0,
    accepts_variadic: false,
    deprecated: None,
  },
  Builtin::Function {
    name: "semver_matches",
    aliases: &[],
    description: indoc! {
      "
      Check whether a semantic version `version` satisfies a
      `requirement`, returning `\"true\"` or `\"false\"`.

      ```just
      semver_matches(\"1.2.3\", \">=1.0.0\")  # => \"true\"
      ```
      "
    },
    required_arguments: 2,
    accepts_variadic: false,
    deprecated: None,
  },
  Builtin::Function {
    name: "sha256",
    aliases: &[],
    description: indoc! {
      "
      Return the SHA-256 hash of `string` as a lowercase hex string.
      "
    },
    required_arguments: 1,
    accepts_variadic: false,
    deprecated: None,
  },
  Builtin::Function {
    name: "sha256_file",
    aliases: &[],
    description: indoc! {
      "
      Return the SHA-256 hash of the file at `path` as a lowercase hex
      string. Aborts if the file cannot be read.
      "
    },
    required_arguments: 1,
    accepts_variadic: false,
    deprecated: None,
  },
  Builtin::Function {
    name: "shell",
    aliases: &[],
    description: indoc! {
      "
      Return the standard output of shell script `command`, with zero
      or more positional arguments `args`.

      The shell used to interpret `command` is the same shell used to
      evaluate recipe lines, configurable with `set shell := [...]`.

      `command` is also passed as the first positional argument, so
      that `$@` works as expected and `$1` refers to the first
      user-supplied argument. With the default `sh -cu` shell and args
      `'foo'` and `'bar'`, the full invocation is:

      ```
      'sh' '-cu' 'echo $@' 'echo $@' 'foo' 'bar'
      ```
      "
    },
    required_arguments: 1,
    accepts_variadic: true,
    deprecated: None,
  },
  Builtin::Function {
    name: "shoutykebabcase",
    aliases: &[],
    description: indoc! {
      "
      Convert `s` to `SHOUTY-KEBAB-CASE`.

      ```just
      shoutykebabcase(\"hello_world\")  # => \"HELLO-WORLD\"
      ```
      "
    },
    required_arguments: 1,
    accepts_variadic: false,
    deprecated: None,
  },
  Builtin::Function {
    name: "shoutysnakecase",
    aliases: &[],
    description: indoc! {
      "
      Convert `s` to `SHOUTY_SNAKE_CASE`.

      ```just
      shoutysnakecase(\"hello-world\")  # => \"HELLO_WORLD\"
      ```
      "
    },
    required_arguments: 1,
    accepts_variadic: false,
    deprecated: None,
  },
  Builtin::Function {
    name: "snakecase",
    aliases: &[],
    description: indoc! {
      "
      Convert `s` to `snake_case`.

      ```just
      snakecase(\"helloWorld\")  # => \"hello_world\"
      ```
      "
    },
    required_arguments: 1,
    accepts_variadic: false,
    deprecated: None,
  },
  Builtin::Function {
    name: "module_directory",
    aliases: &["module_dir"],
    description: indoc! {
      "
      Directory of the current module file. Behaves like
      `justfile_directory()` in the root justfile, but resolves to the
      directory of the current `mod` source file inside submodules.
      "
    },
    required_arguments: 0,
    accepts_variadic: false,
    deprecated: None,
  },
  Builtin::Function {
    name: "module_file",
    aliases: &[],
    description: indoc! {
      "
      Path of the current module file. Behaves like `justfile()` in
      the root justfile, but resolves to the current `mod` source
      file inside submodules.
      "
    },
    required_arguments: 0,
    accepts_variadic: false,
    deprecated: None,
  },
  Builtin::Function {
    name: "source_directory",
    aliases: &["source_dir"],
    description: indoc! {
      "
      Directory of the current source file. Behaves like
      `justfile_directory()` in the root justfile, but resolves to the
      directory of the current `import` or `mod` source file when
      called from within an imported or submodule file.
      "
    },
    required_arguments: 0,
    accepts_variadic: false,
    deprecated: None,
  },
  Builtin::Function {
    name: "source_file",
    aliases: &[],
    description: indoc! {
      "
      Path of the current source file. Behaves like `justfile()` in
      the root justfile, but resolves to the current `import` or `mod`
      source file when called from within an imported or submodule
      file.
      "
    },
    required_arguments: 0,
    accepts_variadic: false,
    deprecated: None,
  },
  Builtin::Function {
    name: "style",
    aliases: &[],
    description: indoc! {
      "
      Return the terminal display attribute escape sequence used by
      `just` itself for styled output.

      Unlike the plain color constants, `style(name)` produces the
      exact sequence `just` uses, so recipe output can match
      `just`'s own styling.

      Recognized values of `name`: `'command'` (echoed recipe lines),
      `'error'`, and `'warning'`.

      ```just
      scary:
        @echo '{{ style(\"error\") }}OH NO{{ NORMAL }}'
      ```
      "
    },
    required_arguments: 1,
    accepts_variadic: false,
    deprecated: None,
  },
  Builtin::Function {
    name: "titlecase",
    aliases: &[],
    description: indoc! {
      "
      Convert `s` to `Title Case`.

      ```just
      titlecase(\"hello world\")  # => \"Hello World\"
      ```
      "
    },
    required_arguments: 1,
    accepts_variadic: false,
    deprecated: None,
  },
  Builtin::Function {
    name: "trim",
    aliases: &[],
    description: indoc! {
      "
      Remove leading and trailing whitespace from `s`.

      ```just
      trim(\"  hello  \")  # => \"hello\"
      ```
      "
    },
    required_arguments: 1,
    accepts_variadic: false,
    deprecated: None,
  },
  Builtin::Function {
    name: "trim_end",
    aliases: &[],
    description: indoc! {
      "
      Remove trailing whitespace from `s`.

      ```just
      trim_end(\"hello  \")  # => \"hello\"
      ```
      "
    },
    required_arguments: 1,
    accepts_variadic: false,
    deprecated: None,
  },
  Builtin::Function {
    name: "trim_end_match",
    aliases: &[],
    description: indoc! {
      "
      Remove a single trailing occurrence of `substring` from `s` if
      present. Leaves `s` unchanged if `substring` does not match at
      the end.

      ```just
      trim_end_match(\"hello.txt\", \".txt\")  # => \"hello\"
      ```
      "
    },
    required_arguments: 2,
    accepts_variadic: false,
    deprecated: None,
  },
  Builtin::Function {
    name: "trim_end_matches",
    aliases: &[],
    description: indoc! {
      "
      Repeatedly remove trailing occurrences of `substring` from `s`
      until none remain.

      ```just
      trim_end_matches(\"aaaab\", \"a\")  # => \"aaaab\"
      trim_end_matches(\"baaaa\", \"a\")  # => \"b\"
      ```
      "
    },
    required_arguments: 2,
    accepts_variadic: false,
    deprecated: None,
  },
  Builtin::Function {
    name: "trim_start",
    aliases: &[],
    description: indoc! {
      "
      Remove leading whitespace from `s`.

      ```just
      trim_start(\"  hello\")  # => \"hello\"
      ```
      "
    },
    required_arguments: 1,
    accepts_variadic: false,
    deprecated: None,
  },
  Builtin::Function {
    name: "trim_start_match",
    aliases: &[],
    description: indoc! {
      "
      Remove a single leading occurrence of `substring` from `s` if
      present. Leaves `s` unchanged if `substring` does not match at
      the start.

      ```just
      trim_start_match(\"hello-world\", \"hello-\")  # => \"world\"
      ```
      "
    },
    required_arguments: 2,
    accepts_variadic: false,
    deprecated: None,
  },
  Builtin::Function {
    name: "trim_start_matches",
    aliases: &[],
    description: indoc! {
      "
      Repeatedly remove leading occurrences of `substring` from `s`
      until none remain.

      ```just
      trim_start_matches(\"aaaab\", \"a\")  # => \"b\"
      ```
      "
    },
    required_arguments: 2,
    accepts_variadic: false,
    deprecated: None,
  },
  Builtin::Function {
    name: "uppercamelcase",
    aliases: &[],
    description: indoc! {
      "
      Convert `s` to `UpperCamelCase` (also known as PascalCase).

      ```just
      uppercamelcase(\"hello_world\")  # => \"HelloWorld\"
      ```
      "
    },
    required_arguments: 1,
    accepts_variadic: false,
    deprecated: None,
  },
  Builtin::Function {
    name: "uppercase",
    aliases: &[],
    description: indoc! {
      "
      Convert `s` to uppercase.

      ```just
      uppercase(\"hello\")  # => \"HELLO\"
      ```
      "
    },
    required_arguments: 1,
    accepts_variadic: false,
    deprecated: None,
  },
  Builtin::Function {
    name: "uuid",
    aliases: &[],
    description: indoc! {
      "
      Generate a random version 4 UUID.

      ```just
      uuid()  # => \"f81d4fae-7dec-11d0-a765-00a0c91e6bf6\"
      ```
      "
    },
    required_arguments: 0,
    accepts_variadic: false,
    deprecated: None,
  },
  Builtin::Function {
    name: "which",
    aliases: &[],
    description: indoc! {
      "
      Search the directories in `$PATH` for an executable called
      `name` and return its full path, or the empty string if no such
      executable is found.

      Unlike `require()`, does not abort on missing executables, so
      this is useful for optional tooling. Currently unstable; requires
      `set unstable`.

      ```just
      set unstable

      bosh := which(\"bosh\")
      ```
      "
    },
    required_arguments: 1,
    accepts_variadic: false,
    deprecated: None,
  },
  Builtin::Function {
    name: "without_extension",
    aliases: &[],
    description: indoc! {
      "
      Return `path` with its extension removed. Aborts if `path` has
      no extension.

      ```just
      without_extension(\"/foo/bar.txt\")  # => \"/foo/bar\"
      ```
      "
    },
    required_arguments: 1,
    accepts_variadic: false,
    deprecated: None,
  },
  Builtin::Setting {
    name: "allow-duplicate-recipes",
    kind: SettingKind::Boolean(false),
    description: indoc! {
      "
      Allow recipes appearing later in a `justfile` to override
      earlier recipes with the same name, instead of producing a
      duplicate-definition error.

      ```just
      set allow-duplicate-recipes

      @foo:
        echo foo

      @foo:
        echo bar
      ```

      `just foo` in the above justfile prints `bar`.
      "
    },
    deprecated: None,
  },
  Builtin::Setting {
    name: "allow-duplicate-variables",
    kind: SettingKind::Boolean(false),
    description: indoc! {
      "
      Allow variables appearing later in a `justfile` to override
      earlier variables with the same name, instead of producing a
      duplicate-definition error.

      ```just
      set allow-duplicate-variables

      a := \"foo\"
      a := \"bar\"
      ```
      "
    },
    deprecated: None,
  },
  Builtin::Setting {
    name: "dotenv-filename",
    kind: SettingKind::String,
    description: indoc! {
      "
      Load a `.env` file with a custom name.

      When set, `just` looks for a file with the given name relative
      to the working directory and each of its ancestors. It is not
      an error if no file is found unless `dotenv-required` is also
      set.

      ```just
      set dotenv-filename := \".env.local\"
      ```

      Compare with `dotenv-path`, which is only checked relative to
      the working directory.
      "
    },
    deprecated: None,
  },
  Builtin::Setting {
    name: "dotenv-load",
    kind: SettingKind::Boolean(false),
    description: indoc! {
      "
      Load a `.env` file, if present.

      When enabled, `just` looks for a file named `.env` relative to
      the working directory and each of its ancestors. It is not an
      error if no file is found unless `dotenv-required` is also set.

      Loaded values become **environment variables**, not `just`
      variables, so they must be accessed as `$VARIABLE_NAME` in
      recipes and backticks.
      "
    },
    deprecated: None,
  },
  Builtin::Setting {
    name: "dotenv-override",
    kind: SettingKind::Boolean(false),
    description: indoc! {
      "
      Override existing environment variables with values from the
      `.env` file.

      By default, variables already set in the ambient environment
      win over those loaded from `.env`. With this setting enabled,
      the dotenv values take precedence.
      "
    },
    deprecated: None,
  },
  Builtin::Setting {
    name: "dotenv-path",
    kind: SettingKind::String,
    description: indoc! {
      "
      Load a `.env` file from a specific path. Errors if the file is
      not found. Overrides `dotenv-filename`.

      Unlike `dotenv-filename`, which is searched for relative to the
      working directory and each ancestor, `dotenv-path` is checked
      only relative to the working directory (or is absolute).

      Can also be overridden at runtime via the `--dotenv-path` /
      `-E` command-line option.
      "
    },
    deprecated: None,
  },
  Builtin::Setting {
    name: "dotenv-required",
    kind: SettingKind::Boolean(false),
    description: indoc! {
      "
      Error if a `.env` file isn't found.

      By default, dotenv-related settings silently skip loading when
      the target file does not exist. With this enabled, the absence
      of the file is a hard error.
      "
    },
    deprecated: None,
  },
  Builtin::Setting {
    name: "export",
    kind: SettingKind::Boolean(false),
    description: indoc! {
      "
      Export every top-level `just` variable as an environment
      variable.

      Equivalent to prefixing each assignment with `export`, so
      recipes and backticks see the variables as `$NAME` rather than
      needing `{{ name }}` interpolation.

      ```just
      set export

      a := \"hello\"

      @foo b:
        echo $a
        echo $b
      ```
      "
    },
    deprecated: None,
  },
  Builtin::Setting {
    name: "fallback",
    kind: SettingKind::Boolean(false),
    description: indoc! {
      "
      If the first recipe on the command line is not found in the
      current justfile, search for a justfile in the parent directory
      and try there.

      Useful in nested project layouts where a subdirectory justfile
      should transparently defer to the root one for unknown
      recipes.
      "
    },
    deprecated: None,
  },
  Builtin::Setting {
    name: "no-exit-message",
    kind: SettingKind::Boolean(false),
    description: indoc! {
      "
      Suppress the trailing `error: Recipe \\\"foo\\\" failed with exit
      code N` message for failed recipes, globally. Individual
      recipes can still opt back in with the `[exit-message]`
      attribute.
      "
    },
    deprecated: None,
  },
  Builtin::Setting {
    name: "guards",
    kind: SettingKind::Boolean(false),
    description: indoc! {
      "
      Enable `?` sigils on recipe lines to conditionally stop recipe
      execution without reporting an error.
      "
    },
    deprecated: None,
  },
  Builtin::Setting {
    name: "ignore-comments",
    kind: SettingKind::Boolean(false),
    description: indoc! {
      "
      Ignore recipe lines beginning with `#`.

      With this enabled, comment lines inside recipe bodies are
      stripped before being passed to the shell, rather than being
      echoed and executed as comments.
      "
    },
    deprecated: None,
  },
  Builtin::Setting {
    name: "lazy",
    kind: SettingKind::Boolean(false),
    description: indoc! {
      "
      Don't evaluate unused variables.

      Normally `just` evaluates every assignment when the justfile is
      loaded. Lazy mode defers evaluation until a variable is actually
      referenced by the recipe being run. Useful when some assignments
      involve expensive backticks or `shell()` calls that only a subset
      of recipes need.
      "
    },
    deprecated: None,
  },
  Builtin::Setting {
    name: "positional-arguments",
    kind: SettingKind::Boolean(false),
    description: indoc! {
      "
      Pass recipe arguments as shell positional parameters (`$0`,
      `$1`, `$@`, ...) for all recipes in the justfile.

      Equivalent to annotating every recipe with
      `[positional-arguments]`. PowerShell does not handle positional
      arguments the same way as POSIX shells, so enabling this with a
      PowerShell-configured `shell` will likely break.
      "
    },
    deprecated: None,
  },
  Builtin::Setting {
    name: "quiet",
    kind: SettingKind::Boolean(false),
    description: indoc! {
      "
      Disable echoing recipe lines before executing them.

      Equivalent to prefixing every recipe line with `@`. Individual
      recipes can opt out with the `[no-quiet]` attribute.
      "
    },
    deprecated: None,
  },
  Builtin::Setting {
    name: "script-interpreter",
    kind: SettingKind::Array,
    description: indoc! {
      "
      Set the command used to invoke recipes annotated with a bare
      `[script]` attribute (no interpreter argument). Defaults to
      `['sh', '-eu']`.

      ```just
      set script-interpreter := ['bash', '-eu']

      [script]
      hello:
        for i in 1 2 3; do echo $i; done
      ```
      "
    },
    deprecated: None,
  },
  Builtin::Setting {
    name: "shell",
    kind: SettingKind::Array,
    description: indoc! {
      "
      Set the command used to invoke recipe lines and evaluate
      backticks. Shebang recipes are unaffected. Defaults to `sh -cu`
      on Unix; on Windows, use `windows-shell` to override.

      `just` appends the line-to-run as an additional argument, so
      most shells need a `-c`-style flag to make them evaluate the
      first argument as a command.

      ```just
      set shell := [\"zsh\", \"-cu\"]

      foo:
        ls **/*.txt  # will run via zsh
      ```
      "
    },
    deprecated: None,
  },
  Builtin::Setting {
    name: "tempdir",
    kind: SettingKind::String,
    description: indoc! {
      "
      Create temporary directories used by shebang and script recipes
      in `tempdir` instead of the system default (`$TMPDIR` /
      `/tmp`).

      Useful when the system temp directory is mounted `noexec` and
      cannot run the temporary script files `just` generates.
      "
    },
    deprecated: None,
  },
  Builtin::Setting {
    name: "unstable",
    kind: SettingKind::Boolean(false),
    description: indoc! {
      "
      Enable unstable features, such as the `||` operator for
      fallback defaults and the `which()` function.

      Unstable features may change or be removed without notice.
      "
    },
    deprecated: None,
  },
  Builtin::Setting {
    name: "windows-powershell",
    kind: SettingKind::Boolean(false),
    description: indoc! {
      "
      **Deprecated**: use `windows-shell` instead.

      Use the legacy `powershell.exe` binary on Windows as the
      default shell for recipes and backticks. Prefer `windows-shell`
      for a more flexible, version-agnostic alternative.
      "
    },
    deprecated: Some("windows-shell"),
  },
  Builtin::Setting {
    name: "windows-shell",
    kind: SettingKind::Array,
    description: indoc! {
      "
      Set the command used to invoke recipes and evaluate backticks
      on Windows.

      By default, `just` uses `sh` on Windows. Set this to use
      PowerShell, `cmd.exe`, Nushell, or any other shell.

      ```just
      set windows-shell := [\"powershell.exe\", \"-NoLogo\", \"-Command\"]

      hello:
        Write-Host 'Hello, world!'
      ```
      "
    },
    deprecated: None,
  },
  Builtin::Setting {
    name: "working-directory",
    kind: SettingKind::String,
    description: indoc! {
      "
      Set the working directory for recipes and backticks, relative
      to the default working directory.

      If relative, interpreted relative to the directory containing
      the justfile. If absolute, used as-is.

      Individual recipes can override this with the
      `[working-directory(PATH)]` attribute or disable directory
      change entirely with `[no-cd]`.
      "
    },
    deprecated: None,
  },
];
