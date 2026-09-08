---
title: Duplicate unexport
category: Variables and parameters
severity: error
order: 480
---

Reports an environment variable name unexported more than once for overlapping platforms.

- The later `unexport` name is reported. Repeating the directive does not add behavior.
- The same name may be unexported in disjoint platform-specific declarations.

## How to fix it

Remove the repeated directive or use nonoverlapping platform attributes when separate declarations are needed.

## Examples

```just reported
unexport FOO
unexport FOO
```

```just corrected
unexport FOO
```
