# Changelog

## [0.3.3](https://github.com/terror/just-lsp/releases/tag/0.3.3) - 2026-02-03

### Added

- Add support for format strings ([#225](https://github.com/terror/just-lsp/pull/225) by [terror](https://github.com/terror))
- Add support for `arg` attribute ([#218](https://github.com/terror/just-lsp/pull/218) by [terror](https://github.com/terror))

### Misc

- Insert raw names for setting and attribute completions ([#226](https://github.com/terror/just-lsp/pull/226) by [terror](https://github.com/terror))
- Bump bytes from 1.10.1 to 1.11.1 ([#224](https://github.com/terror/just-lsp/pull/224) by [app/dependabot](https://github.com/app/dependabot))
- Bump cc from 1.2.54 to 1.2.55 ([#223](https://github.com/terror/just-lsp/pull/223) by [app/dependabot](https://github.com/app/dependabot))
- Bump tree-sitter from 0.26.3 to 0.26.5 ([#222](https://github.com/terror/just-lsp/pull/222) by [app/dependabot](https://github.com/app/dependabot))
- Bump clap from 4.5.54 to 4.5.57 ([#221](https://github.com/terror/just-lsp/pull/221) by [app/dependabot](https://github.com/app/dependabot))
- Bump regex from 1.12.2 to 1.12.3 ([#220](https://github.com/terror/just-lsp/pull/220) by [app/dependabot](https://github.com/app/dependabot))
- Bump cc from 1.2.52 to 1.2.54 ([#216](https://github.com/terror/just-lsp/pull/216) by [app/dependabot](https://github.com/app/dependabot))
- Remove redundant diagnostic sort in analyze subcommand ([#217](https://github.com/terror/just-lsp/pull/217) by [terror](https://github.com/terror))
- Bump cc from 1.2.51 to 1.2.52 ([#214](https://github.com/terror/just-lsp/pull/214) by [app/dependabot](https://github.com/app/dependabot))

## [0.3.2](https://github.com/terror/just-lsp/releases/tag/0.3.2) - 2026-01-08

### Added

- Add deprecation warning for `windows-powershell` setting ([#211](https://github.com/terror/just-lsp/pull/211) by [terror](https://github.com/terror))
- Add deprecation warnings for `env_var` and `env_var_or_default` functions ([#210](https://github.com/terror/just-lsp/pull/210) by [terror](https://github.com/terror))
- Add support for `unexport` keyword ([#205](https://github.com/terror/just-lsp/pull/205) by [terror](https://github.com/terror))

### Fixed

- Fix attribute parsing for comma-separated syntax in recipes ([#198](https://github.com/terror/just-lsp/pull/198) by [terror](https://github.com/terror))

### Misc

- Use `inventory` for rule registration ([#207](https://github.com/terror/just-lsp/pull/207) by [terror](https://github.com/terror))
- Remove `once_cell` dependency in favor of standard library equivalents ([#206](https://github.com/terror/just-lsp/pull/206) by [terror](https://github.com/terror))
- Bump tokio from 1.48.0 to 1.49.0 ([#203](https://github.com/terror/just-lsp/pull/203) by [app/dependabot](https://github.com/app/dependabot))
- Bump tokio-stream from 0.1.17 to 0.1.18 ([#202](https://github.com/terror/just-lsp/pull/202) by [app/dependabot](https://github.com/app/dependabot))
- Bump clap from 4.5.53 to 4.5.54 ([#201](https://github.com/terror/just-lsp/pull/201) by [app/dependabot](https://github.com/app/dependabot))
- Bump serde_json from 1.0.148 to 1.0.149 ([#200](https://github.com/terror/just-lsp/pull/200) by [app/dependabot](https://github.com/app/dependabot))

## [0.3.1](https://github.com/terror/just-lsp/releases/tag/0.3.1) - 2026-01-01

### Added

- Warn on duplicate recipe dependencies with same arguments ([#191](https://github.com/terror/just-lsp/pull/191) by [terror](https://github.com/terror))

### Fixed

- Extend parser to handle module path syntax ([#193](https://github.com/terror/just-lsp/pull/193) by [terror](https://github.com/terror))

### Misc

- Add dependabot workflow ([#190](https://github.com/terror/just-lsp/pull/190) by [terror](https://github.com/terror))
- Update dependencies ([#189](https://github.com/terror/just-lsp/pull/189) by [terror](https://github.com/terror))
- Remove unused `start` script in `/bin` ([#188](https://github.com/terror/just-lsp/pull/188) by [terror](https://github.com/terror))

## [0.3.0](https://github.com/terror/just-lsp/releases/tag/0.3.0) - 2025-12-26

### Added

- Add general duplicate attribute rule ([#164](https://github.com/terror/just-lsp/pull/164) by [terror](https://github.com/terror))
- Add parallel attribute rule for recipes without enough dependencies ([#170](https://github.com/terror/just-lsp/pull/170) by [terror](https://github.com/terror))
- Add rule for `[script]` recipes with a shebang ([#155](https://github.com/terror/just-lsp/pull/155) by [terror](https://github.com/terror))
- Add rule for alias/recipe conflicts ([#146](https://github.com/terror/just-lsp/pull/146) by [terror](https://github.com/terror))
- Add rule for detecting duplicate default attributes ([#145](https://github.com/terror/just-lsp/pull/145) by [terror](https://github.com/terror))
- Add rule for detecting working directory attribute conflicts ([#172](https://github.com/terror/just-lsp/pull/172) by [terror](https://github.com/terror))
- Add rule for reused variable names ([#148](https://github.com/terror/just-lsp/pull/148) by [terror](https://github.com/terror))
- Add rules for mixed indentation ([#149](https://github.com/terror/just-lsp/pull/149) by [terror](https://github.com/terror))
- Add support for `textDocument/semanticTokens` ([#154](https://github.com/terror/just-lsp/pull/154) by [terror](https://github.com/terror))
- Improve syntax-related error messages ([#175](https://github.com/terror/just-lsp/pull/175) by [terror](https://github.com/terror))

### Fixed

- Detect catch-all parameters in unused parameters rule ([#182](https://github.com/terror/just-lsp/pull/182) by [terror](https://github.com/terror))
- Honor positional arguments when warning about unused parameters ([#157](https://github.com/terror/just-lsp/pull/157) by [terror](https://github.com/terror))
- Properly convert positions to tree-sitter points for node selection ([#166](https://github.com/terror/just-lsp/pull/166) by [terror](https://github.com/terror))

### Misc

- Lift out a `HighlightConfig` for tokenizer ([#186](https://github.com/terror/just-lsp/pull/186) by [terror](https://github.com/terror))
- Avoid manual tree traversal in mixed indentation rule ([#185](https://github.com/terror/just-lsp/pull/185) by [terror](https://github.com/terror))
- Use `Recipe` in inconsistent indentation rule ([#184](https://github.com/terror/just-lsp/pull/184) by [terror](https://github.com/terror))
- Use `TextNode` for recipe name ([#183](https://github.com/terror/just-lsp/pull/183) by [terror](https://github.com/terror))
- Refactor rules to use `define_rule` macro ([#180](https://github.com/terror/just-lsp/pull/180) by [terror](https://github.com/terror))
- Add custom diagnostic type ([#179](https://github.com/terror/just-lsp/pull/179) by [terror](https://github.com/terror))
- Rename `display_name` to `message` for `Rule` ([#178](https://github.com/terror/just-lsp/pull/178) by [terror](https://github.com/terror))
- Remove duplicate test document helpers in favour of `Document::from` ([#177](https://github.com/terror/just-lsp/pull/177) by [terror](https://github.com/terror))
- Simplify parallel dependency diagnostics generation ([#176](https://github.com/terror/just-lsp/pull/176) by [terror](https://github.com/terror))
- Make `Rule::diagnostic` take an immutable reference ([#174](https://github.com/terror/just-lsp/pull/174) by [terror](https://github.com/terror))
- Assert entire collections in `Document` tests ([#173](https://github.com/terror/just-lsp/pull/173) by [terror](https://github.com/terror))
- Add server test for document change before open ([#169](https://github.com/terror/just-lsp/pull/169) by [terror](https://github.com/terror))
- Remove `idx` suffix from intermediate definitions ([#168](https://github.com/terror/just-lsp/pull/168) by [terror](https://github.com/terror))
- Encapsulate `Position` and `Point` conversions in extension traits ([#167](https://github.com/terror/just-lsp/pull/167) by [terror](https://github.com/terror))
- Expand feature documentation in readme ([#163](https://github.com/terror/just-lsp/pull/163) by [terror](https://github.com/terror))
- Replace recipe argument `if let` with `map` ([#162](https://github.com/terror/just-lsp/pull/162) by [terror](https://github.com/terror))
- Add shebang field onto `Recipe` ([#161](https://github.com/terror/just-lsp/pull/161) by [terror](https://github.com/terror))
- Add homebrew to package manager table ([#160](https://github.com/terror/just-lsp/pull/160) by [terror](https://github.com/terror))
- Simplify document test range formatting ([#159](https://github.com/terror/just-lsp/pull/159) by [terror](https://github.com/terror))
- Allow ranges to be asserted for analyzer tests ([#158](https://github.com/terror/just-lsp/pull/158) by [terror](https://github.com/terror))
- Handle line continuations and shebang recipes in inconsistent indentation rule ([#156](https://github.com/terror/just-lsp/pull/156) by [terror](https://github.com/terror))
- Update readme documentation for Neovim 0.11+ ([#153](https://github.com/terror/just-lsp/pull/153) by [terror](https://github.com/terror))
- Add newly added builtin constructs ([#152](https://github.com/terror/just-lsp/pull/152) by [terror](https://github.com/terror))
- Put builtin extractors onto `RuleContext` ([#151](https://github.com/terror/just-lsp/pull/151) by [terror](https://github.com/terror))
- Lift out enabled setting helper onto `RuleContext` ([#150](https://github.com/terror/just-lsp/pull/150) by [terror](https://github.com/terror))
- Remove stale entries from changelog ([#147](https://github.com/terror/just-lsp/pull/147) by [terror](https://github.com/terror))
- Add documentation to `Resolver` methods ([#144](https://github.com/terror/just-lsp/pull/144) by [terror](https://github.com/terror))
- Replace `pub` keyword with `pub(crate)` ([#143](https://github.com/terror/just-lsp/pull/143) by [terror](https://github.com/terror))

## [0.2.8](https://github.com/terror/just-lsp/releases/tag/0.2.8) - 2025-11-11

### Added

- Add web playground ([#117](https://github.com/terror/just-lsp/pull/117) by [terror](https://github.com/terror))
- Enable release LTO with single codegen unit ([#120](https://github.com/terror/just-lsp/pull/120) by [terror](https://github.com/terror))

### Fixed

- Fix escaped brace handling in recipe command text ([#126](https://github.com/terror/just-lsp/pull/126) by [terror](https://github.com/terror))
- Replace global mutex with scoped `RwLock` ([#123](https://github.com/terror/just-lsp/pull/123) by [terror](https://github.com/terror))

### Misc

- Enable lints for root workspace ([#141](https://github.com/terror/just-lsp/pull/141) by [terror](https://github.com/terror))
- Update edition to 2024 ([#139](https://github.com/terror/just-lsp/pull/139) by [terror](https://github.com/terror))
- Expand coverage workflow to include all workspace targets ([#138](https://github.com/terror/just-lsp/pull/138) by [terror](https://github.com/terror))
- Enforce stricter default clippy ruleset ([#134](https://github.com/terror/just-lsp/pull/134) by [terror](https://github.com/terror))
- Add attribute extraction to `Document` ([#133](https://github.com/terror/just-lsp/pull/133) by [terror](https://github.com/terror))
- Add function call extraction to `Document` ([#132](https://github.com/terror/just-lsp/pull/132) by [terror](https://github.com/terror))
- Rename `ctx` to `context` for explicitness ([#131](https://github.com/terror/just-lsp/pull/131) by [terror](https://github.com/terror))
- Rename `update-contributors` crate to `just-lsp-changelog` ([#129](https://github.com/terror/just-lsp/pull/129) by [terror](https://github.com/terror))
- Refactor analyzer into rule-based system ([#124](https://github.com/terror/just-lsp/pull/124) by [terror](https://github.com/terror))
- Track code coverage metrics ([#122](https://github.com/terror/just-lsp/pull/122) by [terror](https://github.com/terror))

## [0.2.7](https://github.com/terror/just-lsp/releases/tag/0.2.7) - 2025-09-25

### Added

- Add `analyze` subcommand ([#116](https://github.com/terror/just-lsp/pull/116) by [terror](https://github.com/terror))

### Fixed

- Allow attributes on exported assignments ([#115](https://github.com/terror/just-lsp/pull/115) by [terror](https://github.com/terror))
- Fix parser to handle variables after hash in recipe commands ([#114](https://github.com/terror/just-lsp/pull/114) by [terror](https://github.com/terror))
- Add parser extension workflow to readme ([#113](https://github.com/terror/just-lsp/pull/113) by [terror](https://github.com/terror))
- Extend parser to allow attributes on modules ([#112](https://github.com/terror/just-lsp/pull/112) by [terror](https://github.com/terror))

### Misc

- Add release version to `default` attribute ([#110](https://github.com/terror/just-lsp/pull/110) by [dpassen](https://github.com/dpassen))

## [0.2.6](https://github.com/terror/just-lsp/releases/tag/0.2.6) - 2025-09-25

### Added

- Add `default` built-in attribute ([#103](https://github.com/terror/just-lsp/pull/103) by [terror](https://github.com/terror))
- Add support for attributes on variables ([#100](https://github.com/terror/just-lsp/pull/100) by [terror](https://github.com/terror))

### Fixed

- Properly handle dependency arguments ([#105](https://github.com/terror/just-lsp/pull/105) by [terror](https://github.com/terror))

### Misc

- Remove unnecessary full import path qualifiers ([#102](https://github.com/terror/just-lsp/pull/102) by [terror](https://github.com/terror))
- Fix typo in readme ([#101](https://github.com/terror/just-lsp/pull/101) by [terror](https://github.com/terror))
- Be more explicit about `Node` lifetime elision ([#98](https://github.com/terror/just-lsp/pull/98) by [terror](https://github.com/terror))
- Add downloads badge to readme ([#97](https://github.com/terror/just-lsp/pull/97) by [terror](https://github.com/terror))

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
