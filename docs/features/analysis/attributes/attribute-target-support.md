---
severity: error
order: 10
---

Reports a known attribute attached to a declaration kind that the attribute does not support.

- Recipes, aliases, assignments, functions, imports, modules, settings, and unexports are recognized target kinds. Individual attributes support only a subset.
- For example, `[parallel]` applies to a recipe, while `[group]` applies to recipes and modules. A valid attribute name alone does not make the placement valid.

## How to fix it

Move the attribute to a supported declaration or choose an attribute that supports the current target.

## Examples

```just reported
[group: 'bar']
foo := 'foo'

baz:
  echo {{foo}}
```

```just corrected
foo := 'foo'

[group: 'bar']
baz:
  echo {{foo}}
```
