---
title: Analysis
order: 10
---

`just-lsp` analyzes justfiles as you edit, checking for syntax errors,
unresolved names, invalid settings and attributes, and unused variables,
parameters, and functions. Name resolution and usage checks include imported
definitions, using unsaved editor contents for files that are open.

Files used as modules are analyzed in the context of the justfile that declares
them, so settings, imports, and recipe paths such as `foo::bar` resolve the way
`just` sees them. When you open a file that is imported or used as a module,
the server searches its parent directories for a `justfile` or `.justfile` that
loads it and analyzes the file as part of that project. A file loaded in
several contexts, such as an import shared by a justfile and one of its
modules, reports a variable, function, or parameter as unused only when it is
unused in every context.

Changes to closed files on disk are picked up as well. The server reloads
projects when a file is saved, and, when the editor supports dynamic file
watcher registration, watches justfiles, `*.just` files, and `just-lsp.toml`
for changes, creations, and deletions.

Analysis supports the Language Server Protocol's push diagnostics through
[`textDocument/publishDiagnostics`][publish-diagnostics]. The server publishes
results when you open or change a document and refreshes diagnostics in open
documents that depend on it. Published analysis results include the document
version and replace the previous diagnostics for that file. Closing a
document clears its diagnostics.

Each diagnostic includes a source range, a message explaining the problem,
a rule code, a severity, and `just-lsp` as its source. Rules report errors or
warnings by default. You can [configure their severity or disable individual
rules](#diagnostics) to control which problems your editor highlights.

Some diagnostics also have automatic fixes. Through the LSP
[`textDocument/codeAction`][code-action] request, `just-lsp` returns `quickfix`
code actions containing edits your editor can apply to the document.

The rules below describe the checks that drive this analysis, grouped by
language feature. Each entry lists the rule's code and default severity,
explains what it checks, and notes any available quick fixes. Examples are
independent justfiles, pairing a reported problem with its correction.

[publish-diagnostics]: https://microsoft.github.io/language-server-protocol/specifications/lsp/3.17/specification/#textDocument_publishDiagnostics
[code-action]: https://microsoft.github.io/language-server-protocol/specifications/lsp/3.17/specification/#textDocument_codeAction
