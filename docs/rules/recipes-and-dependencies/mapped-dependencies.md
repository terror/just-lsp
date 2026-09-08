---
title: Mapped dependencies
severity: error
order: 280
---

Reports an invalid mapped dependency or a starred argument used outside a mapped dependency.

- A mapped invocation uses `*(recipe *argument)` and requires enabled `set lists`.
- Exactly one argument must be starred. No starred argument, or more than one, is an error.
- A starred argument in an ordinary dependency invocation is also an error. List mode separately requires `set unstable`.

## How to fix it

Enable list and unstable features, prefix the invocation with `*`, and star exactly one argument to map over.

## Examples

A mapped dependency needs one starred argument.

```just reported
set unstable
set lists

bar baz:
  echo {{baz}}

foo: *(bar 'baz')
```

```just corrected
set unstable
set lists

bar baz:
  echo {{baz}}

foo: *(bar *['baz'])
```

A starred argument also needs a mapped invocation.

```just reported
set unstable
set lists

bar baz:
  echo {{baz}}

foo: (bar *['baz'])
```

```just corrected
set unstable
set lists

bar baz:
  echo {{baz}}

foo: *(bar *['baz'])
```
