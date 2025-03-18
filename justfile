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
  cargo clippy --all-targets --all-features

fmt:
	cargo +nightly fmt

fmt-check:
  cargo +nightly fmt --all -- --check
  @echo formatting check done

install:
	cargo install --path .

run *args:
	cargo run -- --{{args}}

test:
	cargo test

usage:
	cargo run -- --help | pbcopy

watch +COMMAND='test':
	cargo watch --clear --exec "{{COMMAND}}"
