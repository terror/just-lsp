---
severity: error
order: 10
---

Reports unsupported keyword arguments, unknown keyword names, and keywords
missing required values.

- `[arg]` accepts `flag`, `help`, `long`, `max`, `min`, `multiple`, `pattern`,
  `short`, and `value`. Only `flag`, `long`, and `multiple` may be used without
  values.
- `[cache]` accepts `environment`, `extra`, `inputs`, and `outputs`. Each
  supplied keyword requires a value. The bare `[cache]` form is allowed.
- Other builtin attributes do not accept keyword arguments.
- Value expressions are checked by
  [invalid-attribute-argument-expression](#invalid-attribute-argument-expression),
  and positional argument counts by
  [invalid-attribute-argument-count](#invalid-attribute-argument-count).

The `[arg]` keyword `flag` requires [list mode](#list-feature-gate). Caching
requires [unstable features](#unstable-feature-gate) and
[script mode](#cache-without-script).

## How to fix it

Use a supported keyword name and supply a value when required. Remove keyword
arguments from attributes that do not accept them.

## Examples

Unknown keywords are rejected.

```just reported
[arg('foo', bar='baz')]
bar foo:
  echo {{foo}}
```

```just corrected
[arg('foo', help='baz')]
bar foo:
  echo {{foo}}
```

Cache keywords require values.

```just reported
set unstable

[script]
[cache(inputs)]
foo:
  echo foo
```

```just corrected
set unstable

[script]
[cache(inputs='foo')]
foo:
  echo foo
```

Attributes without keyword support reject keyword arguments.

```just reported
[private(foo='bar')]
foo:
  echo foo
```

```just corrected
[private]
foo:
  echo foo
```
