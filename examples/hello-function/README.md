# hello-function

The smallest possible function app: `hello.wasm` is a 36-byte WebAssembly
module that exports `_start` and returns. It exists so the sign and verify
path can be tried without a toolchain.

```bash
kmplify keys create                 # once
kmplify app validate .
kmplify app sign .
kmplify app verify kmplify.app.signed.json --pubkey "$(kmplify keys show)" --wasm hello.wasm
```

A real function is any `wasm32-wasip1` command module that reads stdin and
writes stdout, for example from Rust:

```bash
rustup target add wasm32-wasip1
cargo build --release --target wasm32-wasip1
```

The text form of `hello.wasm` is `(module (func (export "_start")))`.
