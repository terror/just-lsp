set dotenv-load

export CARGO_MSG_LIMIT := '1'

default:
	just --list

alias f := fmt
alias r := run
alias t := test

all: build test clippy fmt-check

[group: 'dev']
build:
  cargo build

[group: 'dev']
build-wasm:
  just -f vendor/tree-sitter-just/justfile build-wasm
  cp vendor/tree-sitter-just/tree-sitter-just.wasm www/public/tree-sitter-just.wasm

[group: 'check']
check:
 cargo check

[group: 'check']
ci: test clippy forbid
  cargo fmt --all -- --check
  cargo update --locked --package just-lsp

[group: 'check']
clippy:
  cargo clippy --all --all-targets

[group: 'format']
fmt:
  cargo fmt

[group: 'format']
fmt-web:
  cd www && bun run format

[group: 'format']
fmt-check:
  cargo fmt --all -- --check

[group: 'check']
forbid:
  ./bin/forbid

[group: 'dev']
install:
  cargo install -f just-lsp

[group: 'dev']
install-dev-deps:
  cargo install cargo-watch

[group: 'release']
publish:
  ./bin/publish

[group: 'dev']
run *args:
  cargo run -- --{{args}}

[group: 'test']
test:
  cargo test --all --all-targets

[group: 'test']
test-release-workflow:
  -git tag -d test-release
  -git push origin :test-release
  git tag test-release
  git push origin test-release

[group: 'release']
update-changelog:
  echo >> CHANGELOG.md
  git log --pretty='format:- %s' >> CHANGELOG.md

[group: 'dev']
update-parser:
  cd vendor/tree-sitter-just && npx tree-sitter generate
  cd vendor/tree-sitter-just && npx tree-sitter test
  cargo test

[group: 'dev']
watch +COMMAND='test':
  cargo watch --clear --exec "{{COMMAND}}"
