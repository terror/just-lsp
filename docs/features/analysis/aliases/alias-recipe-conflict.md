---
severity: error
order: 10
---

Reports an alias and a recipe that share the same name.

- An alias occupies a recipe name, so the two declarations cannot be addressed independently.
- The later conflicting declaration is reported, whether the alias or the recipe appears first. Allowing duplicate recipes does not disable this rule.

## How to fix it

Rename the alias or recipe and update its references, or remove the redundant declaration.

## Examples

```just reported
bar:
  echo bar

alias foo := bar

foo:
  echo foo
```

```just corrected
bar:
  echo bar

alias baz := bar

foo:
  echo foo
```
