---
title: Command Line
order: 5
---

Running `just-lsp` with no arguments starts the language server over
stdin/stdout.

## `analyze`

The `analyze` subcommand runs the diagnostic engine on a justfile and prints any
warnings or errors, without starting the language server:

```bash
just-lsp analyze [PATH]
```

When `PATH` is omitted it searches the current directory and its ancestors for a
file named `justfile`. The exit code is non-zero if any error-severity
diagnostic is found.
