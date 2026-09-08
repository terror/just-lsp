---
title: Duplicate dependencies
severity: warning
order: 260
---

Reports a recipe that repeats the same dependency with identical arguments in the same dependency phase.

- Just only runs that invocation once, so repeating it does not repeat the work.
- The rule compares the recipe name and written arguments, including starred arguments and whether the dependency is mapped. It does not evaluate expressions to compare their results.
- Dependencies before and after `&&` belong to different phases and are not duplicates of each other. Different arguments also make invocations distinct.

## How to fix it

Remove the repeated dependency. If you intended separate invocations, give them the arguments or dependency phases that express that intent.

## Examples

```just reported
foo: bar bar
  echo foo

bar:
  echo bar
```

```just corrected
foo: bar
  echo foo

bar:
  echo bar
```
