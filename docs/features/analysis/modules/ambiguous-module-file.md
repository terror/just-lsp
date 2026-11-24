---
severity: error
order: 10
---

Reports a module declaration that matches more than one source file.

- Without an explicit path, `mod foo` can match `foo.just`, `foo/mod.just`, `foo/justfile`, or `foo/.justfile`. A module path that names a directory is searched for `mod.just`, `justfile`, or `.justfile`.
- The diagnostic lists every matching file. `just` rejects ambiguous modules instead of picking one.
- Optional modules are checked too. Modules disabled for the current platform are skipped.

## How to fix it

Remove or rename all but one of the candidate files, or give the module an explicit path to the file you want.

Assume both foo.just and foo/mod.just exist.

```just reported
mod foo
```

```just corrected
mod foo 'foo.just'
```
