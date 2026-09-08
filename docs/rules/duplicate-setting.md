---
title: Duplicate setting
category: Settings and features
severity: error
order: 420
---

Reports a setting key declared more than once with overlapping platform constraints.

- Repeating the same value is still a duplicate. The check compares the setting names rather than their values.
- The same key can be configured for disjoint platforms, such as separate `[unix]` and `[windows]` shell settings.

## How to fix it

Keep a single declaration for each setting, or use disjoint platform attributes when values need to vary by platform.

## Examples

```just reported
set dotenv-load
set dotenv-load := true
```

```just corrected
set dotenv-load
```
