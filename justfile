set dotenv-load

export CARGO_MSG_LIMIT := '1'

[doc: 'List available recipes']
default:
	just --list

alias f := fmt
alias r := run
alias t := test

[doc: 'Run build, test, clippy, and fmt-check']
all: build test clippy fmt-check

[group: 'dev']
[doc: 'Build the project']
[arg('args', help="Additional cargo build arguments")]
build *args:
  cargo build {{ args }}

[group: 'dev']
[doc: 'Build tree-sitter-just WASM for web']
build-wasm:
  just -f vendor/tree-sitter-just/justfile build-wasm
  cp vendor/tree-sitter-just/tree-sitter-just.wasm www/public/tree-sitter-just.wasm

[group: 'check']
[doc: 'Run cargo check']
[arg('args', help="Additional cargo check arguments")]
check *args:
  cargo check {{ args }}

[group: 'check']
[doc: 'Run full CI checks']
ci: test clippy forbid
  cargo fmt --all -- --check
  cargo update --locked --package just-lsp

[group: 'check']
[doc: 'Run clippy lints']
[arg('args', help="Additional clippy arguments")]
clippy *args:
  cargo clippy --all --all-targets {{ args }}

[group: 'format']
[doc: 'Format Rust code']
fmt:
  cargo fmt

[group: 'format']
[doc: 'Format web frontend code']
fmt-web:
  cd www && bun run format

[group: 'format']
[doc: 'Check Rust formatting']
fmt-check:
  cargo fmt --all -- --check

[group: 'check']
[doc: 'Check for forbidden patterns']
forbid:
  ./bin/forbid

[group: 'dev']
[doc: 'Install from crates.io']
install:
  cargo install -f just-lsp

[group: 'dev']
[doc: 'Install dev build to cargo bin']
install-dev:
  cargo install --bin=just-lsp --path .

[group: 'dev']
[doc: 'Install dev dependencies (cargo-watch)']
install-dev-deps:
  cargo install cargo-watch

[group: 'release']
[doc: 'Publish to crates.io']
publish:
  ./bin/publish

[group: 'dev']
[doc: 'Run the LSP server']
[arg('args', help="Arguments passed to cargo run")]
run *args:
  cargo run {{ args }}

[group: 'test']
[doc: 'Run all tests']
[arg('args', help="Additional cargo test arguments")]
test *args:
  cargo test --all --all-targets {{ args }}

[group: 'test']
[doc: 'Test the release workflow with a test tag']
test-release-workflow:
  -git tag -d test-release
  -git push origin :test-release
  git tag test-release
  git push origin test-release

[group: 'release']
[doc: 'Append git log to CHANGELOG.md']
update-changelog:
  echo >> CHANGELOG.md
  git log --pretty='format:- %s' >> CHANGELOG.md

[group: 'dev']
[doc: 'Regenerate tree-sitter parser and run tests']
update-parser:
  cd vendor/tree-sitter-just && npx tree-sitter generate
  cd vendor/tree-sitter-just && npx tree-sitter test
  cargo test

[group: 'dev']
[doc: 'Watch for changes and run cargo command']
[arg('COMMAND', help="Cargo command to watch (default: test)")]
watch +COMMAND='test':
  cargo watch --clear --exec "{{COMMAND}}"
