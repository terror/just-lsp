---
severity: error
order: 10
---

Reports repeated attributes that must be unique, repeated group values, or more than one default recipe for overlapping platforms.

- Most builtin attributes may appear once per target, including when written in separate attribute lists.
- `[group]` may be repeated with different literal values; equivalent decoded values on the same target are duplicates. `[env]` and `[metadata]` may repeat.
- `[default]` is limited to one recipe per module for overlapping platform constraints. Disjoint platform-specific defaults are allowed.
- `[arg]` may configure multiple parameters, but duplicate configuration for one parameter is checked by `arg-attribute`.

## How to fix it

Remove repeated attributes or group values. Keep a single default recipe for each platform in the module.

## Examples

```just reported
[private]
[private]
foo:
  echo foo
```

```just corrected
[private]
foo:
  echo foo
```

Default recipe uniqueness is checked across recipes, not only on one declaration.

```just reported
[default]
foo:
  echo foo

[default]
bar:
  echo bar
```

```just corrected
[default]
foo:
  echo foo

bar:
  echo bar
```
