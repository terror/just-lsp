Rules run when just-lsp analyzes a document. Each diagnostic includes a rule
code, a message, a source range, and a severity. Use the code shown with each
rule to configure it in your editor’s LSP `initializationOptions`:

```json
{
  "rules": {
    "unused-variables": "off",
    "unused-parameters": { "level": "warning" }
  }
}
```

Accepted levels are `error`, `warning`, `information` (or `info`), `hint`, and
`off`. Omitted rules keep the default severity listed below. Changing a
severity changes editor diagnostics; it does not change what just accepts.

Some rules offer an editor quick fix, as noted in their entries. Name
resolution and usage checks include the document’s imports. Examples are
independent justfiles; each corrected example resolves the problem shown
beside it.
