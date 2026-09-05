Build the WebAssembly analyzer with:

```sh
rustup target add wasm32-unknown-unknown
cargo build --package just-lsp-wasm --target wasm32-unknown-unknown
```

The C parser requires Clang with WebAssembly support and `llvm-ar`. Set
`CC_wasm32_unknown_unknown` and `AR_wasm32_unknown_unknown` to those executables
when they are not the defaults. On macOS, use the LLVM distribution instead of
Apple Clang.

The browser analyzer accepts source text in memory. Filesystem URL conversion
returns an unsupported-operation error on this target.
