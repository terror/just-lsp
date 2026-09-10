---
severity: error
order: 10
---

Reports a known attribute that is not attached to a declaration that can take attributes.

- A dangling attribute at the end of a file is one example. It may also produce a `syntax-errors` diagnostic.
- If the attribute is attached to a recognized declaration of the wrong kind, `unsupported-attribute-target` reports that problem instead.

## How to fix it

Place the attribute immediately before the declaration it should modify, or remove the dangling attribute.

## Examples

```just reported
[private]

```

```just corrected
[private]
foo:
  echo foo
```
