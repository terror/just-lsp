---
severity: warning
order: 10
---

Reports a setting with a documented replacement in the builtin catalog.

- `windows-shell` is deprecated in favor of a `[windows]` attribute on `set shell`.
- The older `windows-powershell` setting is also marked deprecated, with `windows-shell` as its catalog replacement.
- For a replacement that needs an attribute, a quick fix is withheld if the replacement setting already has that attribute.

## How to fix it

Migrate to the replacement named in the diagnostic. For `windows-shell`, use a platform-specific `shell` setting with the same interpreter array.

**Editor quick fix:** Renames a setting when it has a direct replacement, or rewrites `windows-shell` as `[windows]` followed by `set shell` when that replacement is not already configured.

## Examples

```just reported
set windows-shell := ['pwsh', '-c']
```

```just corrected
[windows]
set shell := ['pwsh', '-c']
```
