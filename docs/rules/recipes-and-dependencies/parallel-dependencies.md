---
title: Parallel dependencies
severity: warning
order: 290
---

Reports `[parallel]` on a recipe with fewer than two dependencies.

- With zero or one dependency, there is no pair of dependencies to run in parallel.
- The check counts dependency entries on the recipe. Repeated entries are checked separately by `duplicate-dependencies`.

## How to fix it

Remove `[parallel]` when it has no effect. Keep it for recipes with multiple dependencies that can run concurrently.

**Editor quick fix:** Removes `[parallel]`, preserving other attributes in the same attribute list.

## Examples

```just reported
bar:
  echo bar

[parallel]
foo: bar
  echo foo
```

```just corrected
bar:
  echo bar

foo: bar
  echo foo
```
