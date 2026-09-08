---
title: Cache attribute
severity: error
order: 170
---

Reports invalid arguments to `[cache]`.

- Only the keywords `environment`, `extra`, `inputs`, and `outputs` are accepted, and each supplied keyword needs a value.
- Positional arguments are rejected. The bare `[cache]` form is allowed.
- Using caching also needs unstable features and script mode, checked separately by `unstable-feature-gate` and `cache-without-script`.

## How to fix it

Replace positional arguments with the appropriate keyword and value, correct unknown keyword names, or use `[cache]` without arguments.

## Examples

```just reported
set unstable

[script]
[cache('foo')]
foo:
  echo foo
```

```just corrected
set unstable

[script]
[cache(extra='foo')]
foo:
  echo foo
```
