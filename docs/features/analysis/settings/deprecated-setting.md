---
severity: warning
order: 10
---

Reports a setting with a documented replacement in the builtin catalog.

- `windows-shell` is deprecated in favor of a `[windows]` attribute on `set shell`.
- `windows-powershell` is deprecated in favor of `[windows]` on `set shell := ['powershell.exe', '-NoLogo', '-Command']`.
- Quick fixes that add `[windows] set shell` are withheld when an existing `shell` setting already applies on Windows, including an unconditional `set shell`.

## How to fix it

Migrate to the replacement named in the diagnostic. For `windows-shell`, use a platform-specific `shell` setting with the same interpreter array. For `windows-powershell`, use a platform-specific `shell` setting that runs PowerShell.

**Editor quick fix:** Renames a setting when it has a direct replacement, or rewrites `windows-shell` or `windows-powershell` as `[windows]` followed by `set shell` when that replacement is not already configured.

```just reported
set windows-shell := ['pwsh', '-c']
```

```just corrected
[windows]
set shell := ['pwsh', '-c']
```
