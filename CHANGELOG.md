# Changelog

## [0.2.5](https://github.com/terror/just-lsp/releases/tag/0.2.5) - 2025-08-02

### Fixed

- Detect variable usage in default recipe parameters ([#94](https://github.com/terror/just-lsp/pull/94) by [terror](https://github.com/terror))
- Fix nested function call argument parsing ([#89](https://github.com/terror/just-lsp/pull/89) by [terror](https://github.com/terror))

### Misc

- Update dependencies ([#95](https://github.com/terror/just-lsp/pull/95) by [terror](https://github.com/terror))
- Reduce headings in most recent changelog entry ([#87](https://github.com/terror/just-lsp/pull/87) by [terror](https://github.com/terror))

## [0.2.4](https://github.com/terror/just-lsp/releases/tag/0.2.4) - 2025-07-18

### Added

- Add `parallel` to builtin attributes ([#83](https://github.com/terror/just-lsp/pull/83) by [dpassen](https://github.com/dpassen))

### Fixed

- Revert back to using `just --fmt` on format ([#85](https://github.com/terror/just-lsp/pull/85) by [terror](https://github.com/terror))

### Misc

- Interpolate in format strings directly ([#84](https://github.com/terror/just-lsp/pull/84) by [terror](https://github.com/terror))

## [0.2.3](https://github.com/terror/just-lsp/releases/tag/0.2.3) - 2025-06-03

### Fixed

- Fix formatting by using `just --dump` instead of `just --fmt` ([#77](https://github.com/terror/just-lsp/pull/77) by [DoctorDalek1963](https://github.com/DoctorDalek1963))

### Misc

- Add backtraces to server error messages ([#79](https://github.com/terror/just-lsp/pull/79) by [terror](https://github.com/terror))
- Update dependencies ([#78](https://github.com/terror/just-lsp/pull/78) by [terror](https://github.com/terror))

## [0.2.2](https://github.com/terror/just-lsp/releases/tag/0.2.2) - 2025-05-09

### Fixed

- Remove warnings for default exported recipe parameters ([#70](https://github.com/terror/just-lsp/pull/70) by [terror](https://github.com/terror))
- Don't warn for exported recipe parameters ([#66](https://github.com/terror/just-lsp/pull/66) by [terror](https://github.com/terror))

### Misc

- Use backticks where applicable in analyzer error messages ([#74](https://github.com/terror/just-lsp/pull/74) by [terror](https://github.com/terror))
- Add zed extension to usage section ([#73](https://github.com/terror/just-lsp/pull/73) by [terror](https://github.com/terror))

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
