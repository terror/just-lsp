---
title: Mixed recipe indentation
category: Syntax and imports
severity: error
order: 40
---

Reports a recipe body that mixes tabs and spaces in its leading whitespace.

- Mixing both characters on one line, or indenting one line with spaces and another with tabs, triggers this rule.
- Blank lines are ignored. Only the first indentation problem in each recipe is reported.
- Script recipes are exempt, including shebang recipes and recipes using `[script]` or `set default-script`. An explicit `[shell]` keeps the ordinary recipe checks.

## How to fix it

Indent all ordinary lines in a recipe with either spaces or tabs. Use your editor’s whitespace display to locate invisible differences.

## Examples

The first command begins with a tab; the second begins with two spaces.

```just reported
foo:
	echo foo
  echo bar
```

```just corrected
foo:
  echo foo
  echo bar
```
