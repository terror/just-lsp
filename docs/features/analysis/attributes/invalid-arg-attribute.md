---
severity: error
order: 10
---

Validates parameter references and option dependencies in `[arg(NAME, ...)]`.

- The first argument must name an existing recipe parameter. Configuring the
  same parameter more than once in one recipe is an error.
- The `value` keyword must be paired with `long` or `short`.

Argument counts are checked by
[invalid-attribute-argument-count](#invalid-attribute-argument-count). Keyword
names and required values are checked by
[invalid-attribute-keyword](#invalid-attribute-keyword), and value expressions
by
[invalid-attribute-argument-expression](#invalid-attribute-argument-expression).

## How to fix it

Use the exact parameter name, configure it once, and add `long` or `short` when
using `value`.

## Examples

```just reported
[arg('bar', long='bar')]
foo baz:
  echo {{baz}}
```

```just corrected
[arg('baz', long='baz')]
foo baz:
  echo {{baz}}
```

An option value needs a long or short option name.

```just reported
[arg('bar', value='baz')]
foo bar:
  echo {{bar}}
```

```just corrected
[arg('bar', long='bar', value='baz')]
foo bar:
  echo {{bar}}
```
