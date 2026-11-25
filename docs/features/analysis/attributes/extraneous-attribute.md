---
severity: error
order: 10
---

Reports attributes separated from the following declaration by a blank line or comment.

- Attributes must appear on the lines immediately before the declaration they modify.
- Consecutive attribute lines are reported together, and the message counts every attribute name in them.
- An attribute at the end of a file with no following declaration is reported by `invalid-attribute-placement` instead.

## How to fix it

Remove the blank line between the attributes and the declaration, and move comments above the attributes.

```just reported
[private]

foo:
  echo foo
```

```just corrected
[private]
foo:
  echo foo
```

Comments between attributes and their declaration are reported as well.

```just reported
[private]
# foo
foo:
  echo foo
```

```just corrected
# foo
[private]
foo:
  echo foo
```
