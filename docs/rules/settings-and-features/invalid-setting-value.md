---
title: Invalid setting value
severity: error
order: 400
---

Reports a value that violates the constraints of `indentation` or `minimum-version`.

- `indentation` must be a nonempty string literal consisting entirely of whitespace.
- `minimum-version` must be a literal `MAJOR.MINOR.PATCH` version with exactly three numeric components.
- A version component is either `0` or a number without a leading zero and at most nine digits. Prerelease suffixes, signs, and extra components are rejected.

## How to fix it

Use whitespace for `indentation` and a three-component numeric version for `minimum-version`.

## Examples

```just reported
set indentation := 'foo'
```

```just corrected
set indentation := '  '
```

```just reported
set minimum-version := '1.2'
```

```just corrected
set minimum-version := '1.2.3'
```
