---
title: Script and shell conflict
category: Attributes
severity: error
order: 200
---

Reports a recipe that combines `[script]` with `[shell]`.

- The attributes select mutually exclusive execution modes, whether they appear together or in separate attribute lists.
- `[shell]` on a recipe with global `set default-script` is a valid override. The conflict requires both explicit attributes on the recipe.

## How to fix it

Keep `[script]` to execute the body as one script, or keep `[shell]` for ordinary shell recipe execution.

## Examples

```just reported
[script]
[shell]
foo:
  echo foo
```

```just corrected
[script]
foo:
  echo foo
```
