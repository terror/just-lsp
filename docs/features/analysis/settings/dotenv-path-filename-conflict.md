---
severity: warning
order: 10
---

Reports `dotenv-filename` when `dotenv-path` is also configured.

- `dotenv-path` takes precedence, so the filename setting does not select a different file.
- Either setting can be used alone. This is a warning about redundant configuration rather than a missing-file check.

## How to fix it

Keep `dotenv-path` to select a path explicitly, or keep `dotenv-filename` to select a filename for dotenv discovery.

## Examples

```just reported
set dotenv-path := 'foo/.env'
set dotenv-filename := '.env.bar'
```

```just corrected
set dotenv-path := 'foo/.env'
```
