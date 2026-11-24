---
severity: error
order: 10
---

Reports a required recipe parameter that follows a parameter with a default value.

- A `+` variadic parameter without a default is required, so it cannot follow a defaulted parameter. A `*` variadic parameter may.
- Option parameters, declared with `[arg(..., long)]` or `[arg(..., short)]`, are passed by name and are ignored by this check.

## How to fix it

Move required parameters before the first parameter with a default, or give them defaults.

```just reported
foo bar='bar' baz:
  echo {{bar}} {{baz}}
```

```just corrected
foo baz bar='bar':
  echo {{bar}} {{baz}}
```
