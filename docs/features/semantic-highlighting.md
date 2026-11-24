---
title: Semantic Highlighting
order: 100
---

`just-lsp` supplies semantic tokens so your editor can color justfile syntax
according to its role. Enable semantic highlighting in your editor or LSP client
to use this feature; the active theme determines the colors.

Tokens distinguish comments, keywords, strings, operators, variables,
parameters, functions, namespaces, attributes, and booleans. Recipe and
user-defined function declarations use function tokens with a declaration
modifier, while calls use function tokens without that modifier.

## Example

```just
foo := 'bar'

[private]
baz qux='quux':
  echo {{qux}} {{foo}}
```

The assignment, recipe declaration, parameter, strings, and attribute provide
different token roles for the editor to style. Attributes use the LSP
`decorator` token type, and module names use `namespace`.

In a module path such as `foo::bar` in a dependency or alias target, the module
segments use `namespace` and the final recipe name uses a function token. Alias
targets and `assert` are also highlighted as function calls.

Highlighting is computed from the current document using the bundled just
grammar and highlight queries. It does not run `just` or execute recipes.
Embedded shell or script highlighting is left to the editor's other syntax
support.

The server provides full-document token responses; range requests and token
delta updates are not supported.

LSP request: `textDocument/semanticTokens/full`.
