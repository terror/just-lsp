---
severity: error
order: 10
---

Reports a module that shares its name with another module, a recipe, or an alias.

- Modules, recipes, and aliases share one namespace in a justfile. Declarations from imports are included.
- The later conflicting declaration is reported.
- Optional modules whose file cannot be found do not define a name. Declarations with disjoint platform attributes do not conflict.

## How to fix it

Rename the module, recipe, or alias, or remove the redundant declaration.

Assume foo.just exists.

```just reported
mod foo

foo:
  echo foo
```

```just corrected
mod foo

bar:
  echo bar
```
