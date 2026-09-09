---
severity: error
order: 10
---

Reports recipes that share a name and have overlapping platform constraints.

- Two unrestricted recipes conflict, as do a platform-specific recipe and an unrestricted recipe with the same name.
- Disjoint platforms can use the same recipe name. `[unix]` overlaps Unix platform attributes such as `[linux]` and `[macos]`.
- Enabled `set allow-duplicate-recipes` disables this rule, including when the setting is provided by an import.

## How to fix it

Rename or remove the duplicate recipe. Use disjoint platform attributes for platform-specific implementations, or enable duplicate recipes when overriding is intentional.

## Examples

```just reported
foo:
  echo foo

foo:
  echo bar
```

```just corrected
foo:
  echo foo

bar:
  echo bar
```
