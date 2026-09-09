---
severity: error
order: 10
---

Reports an attribute invocation with too few or too many positional arguments.

- Each builtin attribute has a permitted argument range. For example, `[group]` needs one argument, `[env]` needs two, and `[private]` takes none.
- Some attributes accept optional or variadic arguments. The diagnostic states the actual count and the expected count or range.
- Keyword-specific restrictions for `[arg]` and `[cache]` are checked by their dedicated rules.

## How to fix it

Add missing arguments or remove extra ones according to the attribute’s expected range.

## Examples

```just reported
[group]
foo:
  echo foo
```

```just corrected
[group: 'bar']
foo:
  echo foo
```
