---
severity: error
order: 10
---

Reports a setting assigned a value of the wrong kind.

- The builtin catalog specifies whether a setting accepts a boolean, string, array, or either a string or array.
- A bare boolean setting such as `set export` means true. Boolean values must not be quoted.
- Interpreter settings such as `shell` require an array. `dotenv-command` accepts a string or an array.

## How to fix it

Use the value kind expected by the setting. Keep boolean values unquoted, quote string values, and use array syntax for interpreter commands.

## Examples

```just reported
set export := 'true'
```

```just corrected
set export := true
```

Interpreter commands use an array containing the executable and its arguments.

```just reported
set shell := 'sh'
```

```just corrected
set shell := ['sh', '-cu']
```
