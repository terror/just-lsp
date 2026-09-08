---
title: Backtick shebang
severity: error
order: 20
---

Reports a backtick command whose contents begin with the reserved `#!` sequence.

- Both single-backtick and triple-backtick commands are checked. For a multiline command, the check accounts for the initial blank line and common indentation.
- A shebang is supported at the start of a recipe body. It cannot select an interpreter for a backtick expression.

## How to fix it

Remove the shebang from the backtick command and use the configured shell, invoke an interpreter explicitly, or move the commands into a script recipe.

## Examples

```just reported
foo := `#!/bin/sh`

bar:
  echo {{foo}}
```

```just corrected
foo := `echo foo`

bar:
  echo {{foo}}
```
