---
severity: error
order: 10
---

Reports an import or module declaration that leads back to a file already being loaded.

- Cycles are detected along the load chain from the root justfile, including a file that imports itself.
- Module declarations whose path points back into the chain are reported the same way as imports.

## How to fix it

Remove the import that closes the cycle. If two files need the same declarations, move them into a third file that both import.

Assume this file is named justfile.

```just reported
import 'justfile'

foo:
  echo foo
```

```just corrected
foo:
  echo foo
```
