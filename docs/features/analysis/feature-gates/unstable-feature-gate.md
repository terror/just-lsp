---
severity: warning
order: 10
---

Reports supported unstable features used without enabled `set unstable`.

- The rule checks enabled `set lists`, user-defined function declarations, and `[cache]` on recipes.
- `set unstable` may be provided by the document or its imports. Merely declaring it as false does not enable the features.
- This is separate from the feature’s other requirements: list syntax needs `set lists`, and caching needs script mode.

## How to fix it

Add `set unstable` to opt into the feature, or rewrite the justfile without that feature.

## Examples

```just reported
set lists

foo:
  echo foo
```

```just corrected
set unstable
set lists

foo:
  echo foo
```
