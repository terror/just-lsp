---
title: Working directory conflict
category: Recipes and dependencies
severity: error
order: 300
---

Reports conflicting working-directory and no-cd configuration.

- Enabled `set no-cd` conflicts with `set working-directory` when their platform constraints overlap. Explicit `set no-cd := false` does not conflict.
- A recipe cannot combine `[no-cd]` with `[working-directory]`.
- A recipe attribute can override the opposite global setting: `[no-cd]` with global `working-directory`, or `[working-directory]` with global `no-cd`, is allowed.

## How to fix it

Choose one global working-directory behavior for each platform, and one explicit working-directory behavior per recipe.

## Examples

```just reported
set no-cd
set working-directory := 'foo'

bar:
  echo bar
```

```just corrected
set working-directory := 'foo'

bar:
  echo bar
```

The same conflict is checked within a recipe’s attributes.

```just reported
[no-cd]
[working-directory: 'foo']
bar:
  echo bar
```

```just corrected
[working-directory: 'foo']
bar:
  echo bar
```
