set dotenv-load

default:
	just --list

alias f := fmt
alias r := run
alias t := test

all: build test clippy fmt-check

[group: 'misc']
build:
  cargo build

[group: 'check']
check:
 cargo check

[group: 'check']
clippy:
  cargo clippy --all --all-targets

[group: 'format']
fmt:
  cargo +nightly fmt

[group: 'format']
fmt-check:
  cargo +nightly fmt --all -- --check

[group: 'misc']
install:
  cargo install -f just-lsp

[group: 'dev']
install-dev-deps:
  rustup install nightly
  rustup update nightly
  cargo install cargo-watch

[group: 'release']
publish:
  #!/usr/bin/env bash
  set -euxo pipefail
  rm -rf tmp/release
  gh repo clone https://github.com/terror/just-lsp tmp/release
  cd tmp/release
  VERSION=`sed -En 's/version[[:space:]]*=[[:space:]]*"([^"]+)"/\1/p' Cargo.toml | head -1`
  git tag -a $VERSION -m "Release $VERSION"
  git push origin $VERSION
  cargo publish
  cd ../..
  rm -rf tmp/release

[group: 'dev']
run *args:
  cargo run -- --{{args}}

[group: 'test']
test:
  cargo test

[group: 'dev']
watch +COMMAND='test':
  cargo watch --clear --exec "{{COMMAND}}"
