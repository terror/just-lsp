alias b := build
alias c := check
alias dr := dry-run
alias sw := switch
alias t := test
alias u := update
alias ui := update-input

rebuildArgs := "--verbose"
rebuild := if os() == "macos" { "darwin-rebuild" } else { "nixos-rebuild" }

default:
    @just --choose

[private]
rebuild subcmd:
    {{ rebuild }} {{ subcmd }} {{ rebuildArgs }} --flake .

build: (rebuild "build")

dry-run: (rebuild "dry-run")

switch: (rebuild "switch")

test: (rebuild "test")

ci:
    nix run \
      --inputs-from . \
      --override-input nixpkgs nixpkgs \
      github:Mic92/nix-fast-build -- \
      --no-nom \
      --skip-cached \
      --option accept-flake-config true \
      --option allow-import-from-derivation false \
      --flake '.#hydraJobs'

check:
    nix flake check \
      --print-build-logs \
      --show-trace \
      --accept-flake-config

update:
    nix flake update

update-input input:
    nix flake lock \
      --update-input {{ input }} \
      --commit-lock-file \
      --commit-lockfile-summary "flake: update {{ input }}"

deploy system:
    nix run \
      --inputs-from . \
      'nixpkgs#deploy-rs' -- \
      -s '.#{{ system }}'

deploy-all:
    nix run \
      --inputs-from . \
      'nixpkgs#deploy-rs' -- -s

a:

clean:
    rm -rf \
      result* \
      repl-result-out* \
      config.tf.json \
      .terraform*
