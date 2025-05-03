# Changelog

## [0.2.1](https://github.com/terror/just-lsp/releases/tag/0.2.1) - 2025-05-02

### Misc

- Fix typo in Cargo.toml keywords ([#61](https://github.com/terror/just-lsp/pull/61) by [39bytes](https://github.com/39bytes))
- Add `glibc` targets for linux in release workflow ([#63](https://github.com/terror/just-lsp/pull/63) by [39bytes](https://github.com/39bytes))
- Truncate code sections in development instructions ([#62](https://github.com/terror/just-lsp/pull/62) by [terror](https://github.com/terror))
- Add mason to readme installation section ([#57](https://github.com/terror/just-lsp/pull/57) by [terror](https://github.com/terror))
- Add release badge to readme ([#56](https://github.com/terror/just-lsp/pull/56) by [terror](https://github.com/terror))
- Add changelog document ([#54](https://github.com/terror/just-lsp/pull/54) by [terror](https://github.com/terror))

## [0.2.0](https://github.com/terror/just-lsp/releases/tag/0.2.0) - 2025-04-07

### Added

- Add formatting support ([#51](https://github.com/terror/just-lsp/pull/51) by [terror](https://github.com/terror))
- Add check for circular recipe dependencies ([#50](https://github.com/terror/just-lsp/pull/50) by [terror](https://github.com/terror))

### Misc

- Add usage section to readme ([#48](https://github.com/terror/just-lsp/pull/48) by [terror](https://github.com/terror))
- Simplify variable export check ([#47](https://github.com/terror/just-lsp/pull/47) by [terror](https://github.com/terror))
- Update `env_logger` to latest version ([#46](https://github.com/terror/just-lsp/pull/46) by [terror](https://github.com/terror))
- Add `ci` recipe to justfile ([#45](https://github.com/terror/just-lsp/pull/45) by [terror](https://github.com/terror))
- Put hover resolution logic into `Resolver` ([#44](https://github.com/terror/just-lsp/pull/44) by [terror](https://github.com/terror))
- Add additional packaging information to readme ([#42](https://github.com/terror/just-lsp/pull/42) by [terror](https://github.com/terror))

## [0.1.3](https://github.com/terror/just-lsp/releases/tag/0.1.3) - 2025-03-31

### Added

- Add warnings for unused variables ([#33](https://github.com/terror/just-lsp/pull/33) by [terror](https://github.com/terror))
- Add warnings for unused recipe parameters ([#28](https://github.com/terror/just-lsp/pull/28) by [terror](https://github.com/terror))
- Add code actions for recipes ([#34](https://github.com/terror/just-lsp/pull/34) by [terror](https://github.com/terror))

### Fixed

- Properly resolve identifier definitions ([#31](https://github.com/terror/just-lsp/pull/31) by [terror](https://github.com/terror))
- Don't warn for unused exported variables ([#37](https://github.com/terror/just-lsp/pull/37) by [terror](https://github.com/terror))

### Misc

- Lift attributes onto `Recipe` ([#39](https://github.com/terror/just-lsp/pull/39) by [terror](https://github.com/terror))
- Add a recipe for testing the release workflow ([#36](https://github.com/terror/just-lsp/pull/36) by [terror](https://github.com/terror))
- Handle hover for local symbols ([#32](https://github.com/terror/just-lsp/pull/32) by [terror](https://github.com/terror))
- Make analyzer tests easier to write ([#29](https://github.com/terror/just-lsp/pull/29) by [terror](https://github.com/terror))
