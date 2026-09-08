---
title: Recipe dependency cycles
severity: error
order: 240
---

Reports recipes that participate in a circular dependency chain.

- A recipe depending on itself is a cycle. Longer cycles include chains such as `foo -> bar -> foo`.
- The check follows resolved recipes, including imported recipes and declaration precedence. It reports cycle members in the document being analyzed.
- A recipe that merely depends on a cycle is not itself reported as a cycle member.

## How to fix it

Remove or redirect a dependency to break the cycle. If two recipes share setup work, move that work into a third recipe that neither depends back on.

## Examples

```just reported
foo: bar
  echo foo

bar: foo
  echo bar
```

```just corrected
foo: bar
  echo foo

bar:
  echo bar
```
