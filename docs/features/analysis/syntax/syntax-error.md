---
severity: error
order: 10
---

Reports malformed justfile syntax, including unexpected text and missing syntax elements.

- The diagnostic points to the part of the parse tree that contains an error or a missing element. Its message includes a short snippet or the expected syntax element.
- An incomplete declaration can also cause other rules to report problems. Correct the syntax first, then review the remaining diagnostics.

## How to fix it

Complete the declaration or expression at the reported location. Check recipe colons, assignment operators, matching quotes, brackets, and parentheses.

## Examples

A recipe header must end with a colon.

```just reported
foo
  echo foo
```

```just corrected
foo:
  echo foo
```

A variadic parameter must be last. A malformed parameter list can be rejected by the parser before recipe parameter validation.

```just reported
foo *bar baz:
  echo {{bar}} {{baz}}
```

```just corrected
foo baz *bar:
  echo {{bar}} {{baz}}
```
