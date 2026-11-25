---
severity: error
order: 10
---

Reports a `\u{...}` escape sequence that does not encode a valid Unicode scalar value.

- Values above `10FFFF` are reported, as are surrogate code points from `D800` through `DFFF`.
- Escape sequences are checked in every string that supports them, including import and module paths.

## How to fix it

Use a code point within the valid Unicode range, or write the intended character directly.

```just reported
foo := "\u{110000}"

bar:
  echo {{foo}}
```

```just corrected
foo := "\u{10FFFF}"

bar:
  echo {{foo}}
```
