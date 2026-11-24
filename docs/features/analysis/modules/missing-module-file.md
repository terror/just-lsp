---
severity: error
order: 10
---

Reports a required module declaration whose source file cannot be found.

- Without an explicit path, `mod foo` looks for `foo.just`, `foo/mod.just`, `foo/justfile`, and `foo/.justfile`. An explicit path must name a file, or a directory containing `mod.just`, `justfile`, or `.justfile`.
- Optional modules declared with `mod?` are not reported when their file is missing.
- Dependencies and aliases that refer to recipes in a missing module are not reported separately.

## How to fix it

Create the module file, correct the module path, or declare the module with `mod?` if it is intentionally optional.

Assume foo.just does not exist and is intentionally optional.

```just reported
mod foo
```

```just corrected
mod? foo
```
