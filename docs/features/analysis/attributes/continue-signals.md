---
severity: error
order: 10
---

Reports an unsupported signal name passed to `[continue]`.

- The accepted names are exactly `SIGHUP`, `SIGINT`, and `SIGQUIT`.
- The attribute can take zero or multiple arguments. Each literal signal argument is checked independently; other argument expression restrictions are checked separately.

## How to fix it

Use a supported signal name with its `SIG` prefix, or remove the unsupported argument.

## Examples

```just reported
[continue: 'SIGTERM']
foo:
  echo foo
```

```just corrected
[continue: 'SIGINT']
foo:
  echo foo
```
