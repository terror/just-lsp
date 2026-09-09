---
severity: error
order: 10
---

Reports syntax and builtin calls that require enabled `set lists`.

- Checked syntax includes list literals, `++` list concatenation, `&&` and `||` logical operators, unary `!`, and comparisons used as values.
- An `if` without `else`, an `if` or `assert` condition other than a comparison, and the `[arg]` keyword `flag` also require list mode.
- The builtins `bool`, `join_list`, `num_jobs`, `show`, `split`, and `which` require list mode. A user-defined function shadowing one of those names is exempt from the builtin check.
- Interpreter arrays for `shell`, `windows-shell`, and `script-interpreter` are allowed without list mode. Ordinary comparison conditions are also allowed without it.
- Enabling `set lists` additionally requires `set unstable`, checked by `unstable-feature-gate`.

## How to fix it

Enable both list and unstable features, or replace the feature with syntax and builtins that work without list mode.

## Examples

```just reported
foo:
  echo {{join_list(['foo', 'bar'])}}
```

```just corrected
set unstable
set lists

foo:
  echo {{join_list(['foo', 'bar'])}}
```
