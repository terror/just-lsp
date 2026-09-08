---
title: Unknown setting
category: Settings and features
severity: error
order: 380
---

Reports a `set` statement whose setting name is absent from the builtin catalog.

- Names must match a known setting exactly. A close name is suggested when available.
- Unknown setting names are not checked for value type by `invalid-setting-kind`; correct the name before reviewing its value.

## How to fix it

Correct the setting name or remove the unsupported setting.

**Editor quick fix:** Replaces the setting name with the suggested builtin setting.

## Examples

```just reported
set dotenv-laod
```

```just corrected
set dotenv-load
```
