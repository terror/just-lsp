set dotenv-load

export EDITOR := 'nvim'

alias f := fmt
alias t := test

default:
  just --list

[group: 'build']
build:
  bun run build

[group: 'format']
fmt:
  prettier --write .

[group: 'release']
deploy:
  bun run build && bunx gh-pages -d dist

[group: 'local']
dev:
  bun run dev

test:
  bun test
