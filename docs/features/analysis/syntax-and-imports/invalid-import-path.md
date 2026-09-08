---
title: Invalid import path
severity: error
order: 50
---

Reports an import path that cannot be decoded or expanded, or a required import whose path is empty or does not exist.

- Relative paths are resolved from the importing document’s directory. Shell-expanded strings are checked after environment and home-directory expansion.
- Missing or empty paths are allowed for optional imports and imports disabled for the current platform. Invalid escapes or failed expansion can still be reported.
- Dynamic format-string paths are not resolved by this rule.

## How to fix it

Correct the path, string escapes, or environment variable used in the import. If a file is intentionally optional, use `import?`; required imports should point to an existing file.

## Examples

Assume foo.just does not exist and is intentionally optional.

```just reported
import 'foo.just'
```

```just corrected
import? 'foo.just'
```
