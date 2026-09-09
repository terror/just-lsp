---
severity: error
order: 10
---

Reports a recipe that uses `[cache]` without script mode.

- Use `[script]`, a shebang, or enabled `set default-script` to run a recipe as a script.
- `[shell]` overrides implicit script mode from a shebang or `set default-script`. Combining explicit `[script]` and `[shell]` is a separate `script-shell-conflict`.
- `set unstable` enables the unstable cache feature but does not enable script mode.

## How to fix it

Enable script mode for the cached recipe, or remove `[cache]` if the recipe should run as ordinary shell lines.

## Examples

```just reported
set unstable

[cache]
foo:
  echo foo
```

```just corrected
set unstable

[script]
[cache]
foo:
  echo foo
```
