---
title: Document Links
order: 80
---

Follow import and module links in your editor to open another justfile. The
server supplies a target URI and a tooltip showing its file path; your editor
chooses how to display and activate the link.

## Imports

```just
import 'foo.just'
```

The import path links to the file resolved by the project's import loader.
Unresolved imports do not receive links, including missing optional imports.

## Modules

```just
mod foo 'bar.just'
mod baz
```

An explicit module path such as `'bar.just'` becomes the link. Without an
explicit path, the module name becomes the link when a module file can be found.

For `mod baz`, resolution tries `baz.just`, then `baz/mod.just`, `baz/justfile`,
and `baz/.justfile`, in that order. The `justfile` and `.justfile` filenames are
matched without regard to case. An explicit module path can also point to a
directory, which is searched for `mod.just`, `justfile`, or `.justfile`.

Relative module paths are resolved from the directory containing the current
justfile, and paths beginning with `~/` expand to the home directory. Explicit
file paths may produce links even when the target file does not exist yet.

LSP request: `textDocument/documentLink`.
