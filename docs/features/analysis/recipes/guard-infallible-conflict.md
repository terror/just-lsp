---
severity: error
order: 10
---

Reports a recipe line that combines the guard `?` and infallible `-` sigils.

- A guard line stops the recipe without an error when its command fails, while an infallible line ignores failure and continues. A line cannot do both.
- The check only applies when `set guards` is enabled. Without it, `?` is not a sigil.
- Lines in script recipes and continuation lines after a trailing `\` are not checked, since sigils are only recognized at the start of a command.

## How to fix it

Keep the sigil for the behavior you want: `?` to stop the recipe when the command fails, or `-` to ignore the failure.

```just reported
set guards

foo:
  ?-test -f foo
  echo foo
```

```just corrected
set guards

foo:
  ?test -f foo
  echo foo
```
