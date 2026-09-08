---
title: Duplicate alias
category: Aliases
severity: error
order: 70
---

Reports multiple aliases with the same name when their platform constraints overlap.

- The duplicate is reported even when both aliases point to the same recipe.
- Separate aliases with disjoint platform attributes, such as `[linux]` and `[windows]`, are allowed. An unrestricted alias overlaps every platform.

## How to fix it

Keep one alias for each name, give the aliases distinct names, or use disjoint platform attributes when the targets are platform-specific.

## Examples

```just reported
foo:
  echo foo

alias bar := foo
alias bar := foo
```

```just corrected
foo:
  echo foo

alias bar := foo
```
