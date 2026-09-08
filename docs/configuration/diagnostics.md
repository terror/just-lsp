---
title: Diagnostics
order: 20
---

Configure individual diagnostic rules under the `rules` key, using the rule
codes listed in the [analysis reference](#analysis). Each rule accepts a level
string or an object with a `level` field:

```json
{
  "rules": {
    "unused-variables": "off",
    "unused-parameters": { "level": "error" }
  }
}
```

Supported levels are:

- `error`: report the rule as an error.
- `warning`: report the rule as a warning.
- `information` (or `info`): report the rule as informational.
- `hint`: report the rule as a hint.
- `off`: suppress diagnostics from the rule.

Omitted rules, or rule objects without a `level`, keep their default severity.
Changing a severity changes editor diagnostics; it does not change what `just`
accepts.
