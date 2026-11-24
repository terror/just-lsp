---
severity: error
order: 10
---

Reports a value that violates the constraints of `dry-run`, `indentation`, or `minimum-version`.

- `dry-run` must be `all` or `recipes`.
- `indentation` must be a nonempty string literal containing only spaces or only tabs.
- `minimum-version` must be a literal `MAJOR.MINOR.PATCH` version with exactly three numeric components.
- A version component is either `0` or a number without a leading zero and at most nine digits. Prerelease suffixes, signs, and extra components are rejected.

## How to fix it

Use `all` or `recipes` for `dry-run`, spaces or tabs for `indentation`, and a three-component numeric version for `minimum-version`.

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

```just reported
set dry-run := 'foo'
```

```just corrected
set dry-run := 'recipes'
```
