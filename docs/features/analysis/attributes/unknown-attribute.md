---
title: Unknown attribute
severity: error
order: 90
---

Reports an attribute name that is absent from the builtin attribute catalog.

- Names are checked even inside a comma-separated attribute list.
- A similar known attribute is suggested when available. Argument and target validation for an unknown attribute is left to the syntax and name checks.

## How to fix it

Correct the spelling or remove the unsupported attribute. Use a recognized attribute for the behavior you need.

**Editor quick fix:** Replaces the unknown name with the suggested builtin attribute.

## Examples

```just reported
[privat]
foo:
  echo foo
```

```just corrected
[private]
foo:
  echo foo
```
