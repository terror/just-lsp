---
severity: error
order: 10
---

Reports an alias whose target cannot be resolved to a recipe.

- Recipes in the document and its imports are considered. The target must be a recipe; another alias does not satisfy this lookup.
- When a similar recipe name exists, the diagnostic suggests that name.

## How to fix it

Correct the target name, define the missing recipe, or import the file that provides it.

**Editor quick fix:** Replaces the target with the suggested recipe name when a close match is found.

## Examples

```just reported
alias foo := bar
```

```just corrected
alias foo := bar

bar:
  echo bar
```
