set shell := ['bash', '-eu', '-o', 'pipefail', '-c']

[default]
list:
  @just --list

[group: 'development']
build:
  cargo build

[group: 'checks']
test:
  cargo test

[group: 'checks']
[parallel]
check: build test

[confirm: 'Publish this release?']
[group: 'release']
publish:
  cargo publish

[linux]
open:
  xdg-open .

[macos]
open:
  open .

[windows]
open:
  explorer .
