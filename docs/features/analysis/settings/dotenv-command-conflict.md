---
severity: error
order: 10
---

Reports `dotenv-command` combined with dotenv file-loading settings for overlapping platforms.

- `dotenv-command` conflicts with `dotenv-path`, `dotenv-filename`, enabled `dotenv-load`, and enabled `dotenv-required`.
- Explicitly disabled `dotenv-load` and `dotenv-required` do not conflict. `dotenv-override` can be used with a command.
- Disjoint platform configurations are allowed. The later conflicting setting is reported in the document being analyzed.

## How to fix it

Choose command output or file loading as the environment source. Remove or disable the conflicting file-loading settings when using `dotenv-command`.

## Examples

```just reported
set dotenv-command := 'echo FOO=bar'
set dotenv-load
```

```just corrected
set dotenv-command := 'echo FOO=bar'
```
