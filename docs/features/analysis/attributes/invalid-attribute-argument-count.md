---
severity: error
order: 10
---

Reports an attribute invocation with too few or too many positional arguments.

- Each builtin attribute has a permitted argument range. For example, `[group]`
  needs one argument, `[env]` needs two, and `[private]` takes none.
- Some attributes accept optional or variadic arguments. The diagnostic states
  the actual count and the expected count or range.
- Keyword arguments do not count toward this range. `[arg]` needs one positional
  argument, while `[cache]` accepts none.
- Keyword names and required values are checked by
  [invalid-attribute-keyword](#invalid-attribute-keyword).

## How to fix it

Add missing arguments or remove extra ones according to the attribute’s expected
range.

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

Use keyword arguments to configure caching.

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
