---
title: Attribute argument expressions
category: Attributes
severity: error
order: 110
---

Reports an expression where an attribute requires a string literal or a constant expression.

- Most attributes require a plain string literal. A variable, concatenation, function call, or format string does not satisfy that requirement.
- `[cache]`, `[confirm]`, `[env]`, `[timestamp]`, and `[working-directory]` accept expressions.
- `[doc]` accepts constant expressions but rejects function calls and backtick commands. Keyword values for `[arg]` have their own checks.

## How to fix it

Use a literal string for attributes that require one. For `[doc]`, remove function calls and backtick commands from the expression.

## Examples

A concatenation is an expression even when both operands are literals.

```just reported
[group: 'foo' + 'bar']
foo:
  echo foo
```

```just corrected
[group: 'foobar']
foo:
  echo foo
```

Function calls are not accepted in constant attribute expressions.

```just reported
[doc: uppercase('foo')]
foo:
  echo foo
```

```just corrected
[doc: 'FOO']
foo:
  echo foo
```
