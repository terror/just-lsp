---
title: Extension without script
category: Attributes
severity: error
order: 190
---

Reports `[extension]` on a recipe that does not run in script mode.

- The extension applies to a generated script file, so it has no effect on an ordinary shell recipe.
- Script mode can come from `[script]`, a shebang, or `set default-script`. An explicit `[shell]` disables script mode for this check.

## How to fix it

Run the recipe as a script if it needs a script-file extension, or remove `[extension]`.

## Examples

```just reported
[extension: 'sh']
foo:
  echo foo
```

```just corrected
[script]
[extension: 'sh']
foo:
  echo foo
```
