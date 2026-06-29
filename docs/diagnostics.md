## Diagnostics

`just-lsp` runs the following diagnostics on every open document (see
`src/analyzer.rs`). Each rule emits a Language Server Protocol diagnostic with
the rule’s identifier in the `code` field so editors can group or filter them.

| Code                              | Name                            | Description                                                                                                                   |
| --------------------------------- | ------------------------------- | ----------------------------------------------------------------------------------------------------------------------------- |
| `syntax-errors`                   | Syntax Errors                   | Parse tree contains errors or missing nodes.                                                                                  |
| `missing-recipe-for-alias`        | Missing Recipe for Alias        | Alias points to a recipe that doesn't exist.                                                                                  |
| `duplicate-alias`                 | Duplicate Alias                 | Alias name is defined more than once.                                                                                         |
| `alias-recipe-conflict`           | Alias/Recipe Conflict           | Alias and recipe share a name and would shadow each other.                                                                    |
| `unknown-attribute`               | Unknown Attribute               | Attribute name is not part of the builtin catalog.                                                                            |
| `attribute-arguments`             | Attribute Arguments             | Attribute invocation uses the wrong number of arguments.                                                                      |
| `attribute-argument-expressions`  | Attribute Argument Expressions  | Attribute invocation uses an expression where just requires a string literal.                                                 |
| `arg-attribute`                   | Arg Attribute                   | `[arg(NAME, ...)]` references an unknown parameter, duplicates a parameter config, or contains invalid keywords or keyword values. |
| `attribute-invalid-target`        | Attribute Invalid Target        | Attribute is attached to a syntax element that cannot take attributes.                                                        |
| `attribute-target-support`        | Attribute Target Support        | Attribute is used on an unsupported target kind.                                                                              |
| `duplicate-attribute`             | Duplicate Attribute             | Attributes that must be unique appear more than once on a target, or `[default]` appears more than once in a module.          |
| `script-shebang-conflict`         | Script Shebang Conflict         | Recipe combines a shebang line with the `[script]` attribute.                                                                 |
| `exit-message-conflict`           | Exit Message Conflict           | Recipe combines mutually exclusive `[exit-message]` and `[no-exit-message]` attributes.                                       |
| `extension-without-script`        | Extension Without Script        | Recipe uses `[extension]` without `[script]` or a shebang, so the attribute has no effect.                                    |
| `duplicate-recipes`               | Duplicate Recipes               | Recipe name collides with another recipe for overlapping targets unless duplicates are allowed.                               |
| `recipe-parameters`               | Recipe Parameters               | Recipe parameter list has duplicates, required-after-default parameters, or misplaced variadic parameters.                    |
| `recipe-dependency-cycles`        | Recipe Dependency Cycles        | Recipe participates in a circular dependency chain.                                                                           |
| `missing-dependencies`            | Missing Dependencies            | Recipe depends on another recipe that is missing.                                                                             |
| `duplicate-dependencies`          | Duplicate Dependencies          | Warning: recipe lists the same dependency with identical arguments more than once; just only runs it once, so it's redundant. |
| `dependency-arguments`            | Dependency Arguments            | Dependency invocation provides the wrong number of arguments.                                                                 |
| `mapped-dependencies`             | Mapped Dependencies             | Mapped dependencies require `set lists`, at least one starred argument, and at most one starred argument.                      |
| `parallel-dependencies`           | Parallel Dependencies           | Warning: `[parallel]` is applied to a recipe with fewer than two dependencies, so it has no effect.                           |
| `working-directory-conflict`      | Working Directory Conflict      | Recipe combines mutually exclusive `[no-cd]` and `[working-directory]` attributes.                                            |
| `mixed-recipe-indentation`        | Mixed Recipe Indentation        | Recipe body mixes tabs and spaces for indentation.                                                                            |
| `inconsistent-recipe-indentation` | Inconsistent Recipe Indentation | Recipe indentation width changes after the first indented line.                                                               |
| `unknown-function`                | Unknown Function                | Call targets a function that isn't a builtin or user-defined function.                                                        |
| `function-arguments`              | Function Arguments              | Builtin or user-defined function call uses the wrong number of arguments.                                                     |
| `deprecated-function`             | Deprecated Function             | Warning: function call uses a deprecated builtin function with a replacement.                                                 |
| `duplicate-function`              | Duplicate Function              | User-defined function name is defined more than once.                                                                         |
| `function-parameters`             | Function Parameters             | User-defined function parameter list has duplicates.                                                                          |
| `unstable-feature-gate`           | Unstable Feature Gate           | Warning: justfile-visible unstable features such as `set lists` and user-defined functions are used without `set unstable`.   |
| `list-features`                   | List Features                   | Syntax and builtins that require `set lists` are used without enabling it.                                                    |
| `unknown-setting`                 | Unknown Setting                 | `set` statement references an unknown setting.                                                                                |
| `invalid-setting-kind`            | Invalid Setting Kind            | Setting is assigned a value of the wrong type.                                                                                |
| `deprecated-setting`              | Deprecated Setting              | Warning: setting uses a deprecated builtin setting with a replacement.                                                        |
| `duplicate-setting`               | Duplicate Setting               | Setting key is configured more than once.                                                                                     |
| `duplicate-variable`              | Duplicate Variable              | Variable assignment reuses a name without allowing duplicates.                                                                |
| `duplicate-unexport`              | Duplicate Unexport              | Environment variable name is unexported more than once.                                                                       |
| `export-unexport-conflict`        | Export/Unexport Conflict        | Variable is both assigned and unexported.                                                                                     |
| `undefined-identifiers`           | Undefined Identifiers           | Expression identifier cannot be resolved to a parameter, variable, builtin, or user-defined function.                         |
| `unused-variables`                | Unused Variables                | Warning: non-exported global variable is never referenced.                                                                    |
| `unused-parameters`               | Unused Parameters               | Warning: recipe parameter is never read unless it is exported or available through positional arguments.                       |
| `dotenv-path-filename-conflict`   | Dotenv Path/Filename Conflict   | Warning: `dotenv-path` overrides `dotenv-filename`; setting both is redundant.                                                |
| `invalid-import-path`             | Invalid Import Path             | Literal non-optional import path points to a path that does not exist on disk.                                                |
