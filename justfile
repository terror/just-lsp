set dotenv-load

default:
	just --list

alias f := fmt
alias r := run
alias t := test

all: build test clippy fmt-check

build:
  cargo build

check:
 cargo check

clippy:
  cargo clippy -- --deny warnings

[group: 'format']
fmt:
  cargo +nightly fmt

[group: 'format']
fmt-check:
  cargo +nightly fmt --all -- --check

run *args:
  cargo run -- --{{args}}

[group: 'test']
test:
  cargo test

watch +COMMAND='test':
  cargo watch --clear --exec "{{COMMAND}}"
