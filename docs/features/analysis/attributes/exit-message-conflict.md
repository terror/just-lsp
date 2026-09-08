---
title: Exit message conflict
severity: error
order: 210
---

Reports a recipe that combines `[exit-message]` with `[no-exit-message]`.

- The two attributes request opposite behavior for reporting a recipe’s exit failure.
- The conflict is checked per recipe and is independent of the order of the attributes.

## How to fix it

Keep the attribute that expresses whether this recipe should print an exit message.

## Examples

```just reported
[exit-message]
[no-exit-message]
foo:
  echo foo
```

```just corrected
[no-exit-message]
foo:
  echo foo
```
