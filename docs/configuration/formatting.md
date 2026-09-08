Formatting requires `just` on your `PATH` and runs
`just --fmt --unstable --quiet`. By default, `just-lsp` lets `just` choose its
normal indentation. Set `formatting.indentation` to pass a custom indentation
string through `--indentation`:

```json
{
  "formatting": {
    "indentation": "  "
  }
}
```

Use `"\t"` for tabs or `"  "` for two spaces.
