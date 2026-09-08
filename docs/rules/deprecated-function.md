---
title: Deprecated function
category: Functions
severity: warning
order: 330
---

Reports a call to a builtin function with a documented replacement.

- The catalog marks `env_var` and `env_var_or_default` as deprecated in favor of `env`.
- The replacement keeps the environment-variable name and optional default argument.
- A user-defined function that shadows a deprecated builtin is not reported by this rule.

## How to fix it

Use the replacement function named in the diagnostic, preserving the arguments.

**Editor quick fix:** Renames the deprecated function to its replacement.

## Examples

```just reported
foo:
  echo {{env_var_or_default('FOO', 'bar')}}
```

```just corrected
foo:
  echo {{env('FOO', 'bar')}}
```
