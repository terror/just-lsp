---
title: Unused function
category: Functions
severity: warning
order: 360
---

Reports a user-defined function that is never called in the document or its imports.

- Only the resolved definition in the document being analyzed is reported. A declaration superseded by another definition is not treated as a separate unused function.
- Function names beginning with `_` are exempt.
- Any function call counts as usage, including a call from another function. This is a reference check, not an execution-reachability analysis.

## How to fix it

Remove an obsolete function, call it where intended, or prefix its name with `_` when keeping an intentionally unused helper.

## Examples

```just reported
set unstable

foo() := 'foo'
```

```just corrected
set unstable

_foo() := 'foo'
```
