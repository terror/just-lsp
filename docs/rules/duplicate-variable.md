---
title: Duplicate variable
category: Variables and parameters
severity: error
order: 470
---

Reports variable assignments that reuse a name for overlapping platforms.

- The check covers ordinary and exported assignments. Assigning the same value again is still a duplicate.
- Disjoint platform attributes allow separate values under the same name.
- Enabled `set allow-duplicate-variables` disables this rule, including when enabled through an import.

## How to fix it

Keep one assignment, rename a distinct variable, or explicitly allow duplicate variables when overriding a value is intentional.

## Examples

```just reported
foo := 'foo'
foo := 'bar'

bar:
  echo {{foo}}
```

```just corrected
foo := 'bar'

bar:
  echo {{foo}}
```
