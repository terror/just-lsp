---
severity: error
order: 10
---

Reports a positional argument or keyword value where an attribute requires a
string literal or a constant expression.

- Most positional arguments require a string literal, including the parameter
  name in `[arg]`. A variable, concatenation, function call, or format string
  does not satisfy that requirement.
- Values supplied for the `[arg]` keywords `long` and `short` must also be
  string literals.
- `[doc]` arguments and the `[arg]` keywords `help` and `pattern` accept
  constant expressions, including variables and concatenations, but reject
  function calls and backtick commands.
- `[confirm]`, `[env]`, `[timestamp]`, and `[working-directory]` accept
  arbitrary expressions. So do `[cache]` keyword values and values for the
  `[arg]` keywords `flag`, `max`, `min`, `multiple`, and `value`.

## How to fix it

Use a literal string where required. For `[doc]` arguments and `[arg]` values
for `help` or `pattern`, remove function calls and backtick commands from the
expression.

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

Keyword values follow the same expression restrictions.

```just reported
[arg('foo', long='bar' + 'baz')]
bar foo:
  echo {{foo}}
```

```just corrected
[arg('foo', long='barbaz')]
bar foo:
  echo {{foo}}
```
