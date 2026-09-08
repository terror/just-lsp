---
title: Arg attribute
category: Attributes
severity: error
order: 150
---

Validates the parameter name and keyword arguments in `[arg(NAME, ...)]`.

- The first argument must name an existing recipe parameter. Configuring the same parameter more than once in one recipe is an error.
- Recognized keywords are `flag`, `help`, `long`, `max`, `min`, `multiple`, `pattern`, `short`, and `value`.
- `long` and `short` require string literals. `help` and `pattern` allow constant expressions but reject function calls and backtick commands.
- `value=` must be paired with `long=` or `short=`. The `flag` keyword also requires list mode, checked separately by `list-features`.

## How to fix it

Use the exact parameter name, configure it once, and supply supported keywords with the required value forms. Add `long=` or `short=` when using `value=`.

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
