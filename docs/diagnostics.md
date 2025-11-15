## Diagnostics

`just-lsp` runs the following diagnostics on every open document (see
`src/analyzer.rs`). Each rule emits a Language Server Protocol diagnostic with
the rule’s identifier in the `code` field so editors can group or filter them.

| Code                              | Name                            | Description                                                                                         |
| --------------------------------- | ------------------------------- | --------------------------------------------------------------------------------------------------- |
| `syntax-errors`                   | Syntax Errors                   | Parse tree contains errors or missing nodes.                                                        |
| `missing-recipe-for-alias`        | Missing Recipe for Alias        | Alias points to a recipe that doesn’t exist.                                                        |
| `duplicate-alias`                 | Duplicate Alias                 | Alias name is defined more than once.                                                               |
| `alias-recipe-conflict`           | Alias/Recipe Conflicts          | Alias and recipe share a name and would shadow each other.                                          |
| `unknown-attribute`               | Unknown Attribute               | Attribute name is not part of the builtin catalog.                                                  |
| `attribute-arguments`             | Attribute Arguments             | Attribute invocation uses the wrong number of arguments.                                            |
| `attribute-invalid-target`        | Attribute Invalid Target        | Attribute is attached to a syntax element that cannot take attributes.                              |
| `attribute-target-support`        | Attribute Target Support        | Attribute is used on an unsupported target kind (recipe, alias, module, etc.).                      |
| `duplicate-attribute`             | Duplicate Attribute             | Attributes that must be unique (e.g., `[default]`, `[script]`) appear more than once.               |
| `script-shebang-conflict`         | Script Shebang Conflict         | Recipe combines a shebang line with the `[script]` attribute.                                       |
| `duplicate-recipes`               | Duplicate Recipes               | Recipe name collides with another recipe for overlapping targets (unless duplicates allowed).       |
| `recipe-parameters`               | Recipe Parameters               | Parameter list has duplicates, required-after-default parameters, or misplaced variadic parameters. |
| `recipe-dependency-cycles`        | Recipe Dependency Cycles        | Recipe participates in a circular dependency chain.                                                 |
| `missing-dependencies`            | Missing Dependencies            | Recipe depends on another recipe that is missing.                                                   |
| `dependency-arguments`            | Dependency Arguments            | Dependency invocation provides the wrong number of arguments.                                       |
| `parallel-dependencies`           | Parallel Dependencies           | Warning: `[parallel]` is applied to a recipe with fewer than two dependencies, so it has no effect. |
| `mixed-recipe-indentation`        | Mixed Recipe Indentation        | Recipe body mixes tabs and spaces for indentation.                                                  |
| `inconsistent-recipe-indentation` | Inconsistent Recipe Indentation | Recipe indentation width changes after the first indented line.                                     |
| `unknown-function`                | Unknown Function                | Call targets a function that isn’t part of the builtin set.                                         |
| `function-arguments`              | Function Arguments              | Builtin function call uses the wrong number of arguments.                                           |
| `unknown-setting`                 | Unknown Setting                 | `set` statement references an unknown setting.                                                      |
| `invalid-setting-kind`            | Invalid Setting Kind            | Setting is assigned a value of the wrong type (bool/string/array).                                  |
| `duplicate-setting`               | Duplicate Setting               | Setting key is configured more than once.                                                           |
| `duplicate-variable`              | Duplicate Variable              | Variable assignment reuses a name without allowing duplicates.                                      |
| `undefined-identifiers`           | Undefined Identifiers           | Identifier cannot be resolved to a parameter, variable, or builtin.                                 |
| `unused-variables`                | Unused Variables                | Warning: non-exported global variable is never referenced.                                          |
| `unused-parameters`               | Unused Parameters               | Warning: recipe parameter is never read (unless exported).                                          |
