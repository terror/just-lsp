---
severity: error
order: 10
---

Reports a builtin or user-defined function call with an invalid argument count.

- Builtin signatures specify required, optional, and variadic arguments. The diagnostic gives the required minimum or accepted maximum.
- User-defined functions require exactly one argument per declared parameter.
- A resolved user-defined function takes precedence over a builtin with the same name. Unknown functions are checked by `unknown-function`.

## How to fix it

Add required arguments or remove extra arguments to match the function’s signature.

## Examples

```just reported
foo:
  echo {{uppercase()}}
```

```just corrected
foo:
  echo {{uppercase('foo')}}
```
