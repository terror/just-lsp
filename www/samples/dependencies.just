default: check

alias b := build
alias t := test

check: format-check test

format-check:
  cargo fmt --check

test: build
  cargo test

build:
  cargo build

package target: (compile target 'release')
  @echo 'Packaging {{target}}'

compile target profile:
  @echo 'Compiling {{target}} with profile {{profile}}'

release: (package 'app') && clean
  @echo 'Publishing app'

clean:
  @echo 'Cleaning temporary files'
