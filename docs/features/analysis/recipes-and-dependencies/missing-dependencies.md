---
title: Missing dependencies
severity: error
order: 250
---

Reports a recipe dependency whose name cannot be resolved to a recipe.

- The lookup includes recipes from the document and its imports.
- Dependencies before and after `&&` are checked. Alias names do not substitute for recipe names in this lookup.
- A close recipe name is suggested when available.

## How to fix it

Correct the dependency name, define the recipe, or import the file that contains it.

**Editor quick fix:** Replaces the dependency name with the suggested recipe name when a close match is found.

## Examples

```just reported
foo: bar
  echo foo
```

```just corrected
foo: bar
  echo foo

bar:
  echo bar
```
