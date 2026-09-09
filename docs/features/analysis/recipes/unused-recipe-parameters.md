---
severity: warning
order: 10
---

Reports recipe parameters that are not referenced and are not available through an applicable export or positional-argument exemption.

- References in the recipe’s expressions count as usage. Parameters prefixed with `$`, and parameters covered by `set export`, are exempt.
- With `set positional-arguments` or `[positional-arguments]`, a reference such as `$1` or `${1}` marks the corresponding parameter as used.
- With positional arguments enabled, `$@`, `$*`, `${@}`, or `${*}` mark all parameters as used. Script recipes in that mode also treat all parameters as available.
- An underscore prefix alone does not exempt a recipe parameter; that convention is supported by `unused-function-parameter` and `unused-variables` instead.

## How to fix it

Use or remove the parameter. If the shell consumes it, export it with `$` or enable positional arguments and refer to the appropriate position.

## Examples

```just reported
foo bar:
  echo foo
```

```just corrected
foo bar:
  echo {{bar}}
```

Shell positional arguments count only when positional-argument mode is enabled.

```just reported
foo bar:
  echo "$1"
```

```just corrected
[positional-arguments]
foo bar:
  echo "$1"
```
