## just-lsp

[![release](https://img.shields.io/github/release/terror/just-lsp.svg?label=release&style=flat&labelColor=1d1d1d&color=424242&logo=github)](https://github.com/terror/just-lsp/releases/latest)
[![crates.io](https://img.shields.io/crates/v/just-lsp.svg?style=flat&labelColor=1d1d1d&color=424242&logo=rust)](https://crates.io/crates/just-lsp)
[![build](https://img.shields.io/github/actions/workflow/status/terror/just-lsp/ci.yaml?branch=master&style=flat&labelColor=1d1d1d&color=424242&logo=GitHub%20Actions&logoColor=white&label=build)](https://github.com/terror/just-lsp/actions/workflows/ci.yaml)
[![codecov](https://img.shields.io/codecov/c/gh/terror/just-lsp?style=flat&labelColor=1d1d1d&color=424242&logo=Codecov&logoColor=white)](https://codecov.io/gh/terror/just-lsp)
[![downloads](https://img.shields.io/github/downloads/terror/just-lsp/total.svg?style=flat&labelColor=1d1d1d&color=424242)](https://github.com/terror/just-lsp/releases)

`just-lsp` is a server implementation of the
[language server protocol](https://microsoft.github.io/language-server-protocol/)
for [just](https://github.com/casey/just), the command runner.

[Website](https://www.just-lsp.systems/) ·
[Documentation](https://www.just-lsp.systems/documentation) ·
[Playground](https://www.just-lsp.systems/playground)

<img width="1667" alt="demo" src="https://raw.githubusercontent.com/terror/just-lsp/master/screenshot.png" />

`just-lsp` brings rich editor support to your justfiles, including completions,
hover docs, diagnostics, navigation, renaming, formatting, and running recipes.

## Installation

The easiest way to install it is by using
[cargo](https://doc.rust-lang.org/cargo/index.html), the Rust package manager:

```bash
cargo install just-lsp
```

See the
[installation guide](https://www.just-lsp.systems/documentation#installation) for
other package managers and pre-built binaries.

## Usage

See the documentation for
[editor setup](https://www.just-lsp.systems/documentation#usage),
[command-line usage](https://www.just-lsp.systems/documentation#cli),
[configuration](https://www.just-lsp.systems/documentation#configuration), and
[features](https://www.just-lsp.systems/documentation#features).

## Development

See [CONTRIBUTING.md](CONTRIBUTING.md) for build instructions, editor setup for
local development, and parser changes.

## Support

If you need help with `just-lsp`, please feel free to
[open an issue](https://github.com/terror/just-lsp/issues) or ping me on
[Discord](https://discord.gg/ezYScXR). Feature requests and bug reports are
always welcome!
