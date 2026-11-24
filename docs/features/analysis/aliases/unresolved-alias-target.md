---
severity: error
order: 10
---

Reports an alias whose target cannot be resolved to a recipe or module.

- Recipes and modules in the document and its imports are considered. Another alias does not satisfy this lookup.
- A module path such as `foo::bar` is resolved through the declared modules. An unknown module in the path is reported, while paths into optional modules whose file is missing are not.
- When a similar recipe name exists, the diagnostic suggests that name.

## How to fix it

Correct the target name, define the missing recipe, or import the file that provides it.

**Editor quick fix:** Replaces the target with the suggested recipe name when a close match is found.

```just reported
alias foo := bar
```

```just corrected
alias foo := bar

bar:
  echo bar
```
