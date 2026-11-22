# kmplify-dev-sdk

The developer SDK for **serverless apps** in the KMPLIFY ecosystem: what a
registered developer uses to build an app, sign it, and (soon) submit it to
[marketplace.kmplify.ai](https://marketplace.kmplify.ai), free or paid.

A serverless app runs on the KMPLIFY Compute Fabric: on servers KMPLIFY
operates and on peer hardware (KMPLIFY Desktop, `kmplify-node`). Because that
includes other people's machines, an app is always a **signed, sandboxed
package** and never just an image name.

## Status

| Piece | State |
|---|---|
| `kmplify.app/v1` manifest and its JSON Schema | here |
| `kmplify` CLI: `app init`, `app validate`, `keys create/list/show`, `app sign`, `app verify` | here |
| `kmplify app test` (run locally in the same sandbox a node uses) | planned |
| `kmplify login`, `kmplify app publish` (submit to the review queue) | planned: needs the submission service |
| Runtime libraries (KV, bucket, queue, secrets) for Python, JS, Rust | planned |

See [docs/ROADMAP.md](docs/ROADMAP.md). Nothing here can publish an app yet.

## Quick start

```bash
cd cli && cargo install --path .        # installs the `kmplify` binary

kmplify keys create                     # your signing key, stays on this machine
kmplify app init my-app --id acme/my-app --lane function
kmplify app validate my-app
kmplify app sign my-app                 # writes my-app/kmplify.app.signed.json
kmplify app verify my-app/kmplify.app.signed.json --pubkey "$(kmplify keys show)"
```

`examples/hello-function` signs and verifies as it is.

## The two lanes

| Lane | What it is | Runs on | Sandbox |
|---|---|---|---|
| `function` | a WebAssembly module (`wasm32-wasip1`), stdin to stdout | every node, any OS | Wasm runtime: no files, no sockets, no environment |
| `container` | an OCI image pinned by digest, with an HTTP port | Linux nodes with a gVisor or microVM runtime | gVisor at least, on top of dropped capabilities, read-only rootfs and resource caps |

## How an app becomes runnable

1. **You sign it.** `kmplify app sign` proves who published these exact bytes.
2. **KMPLIFY reviews it.** Every submission, free or paid, waits in one queue
   and is reviewed by a person in the order it arrived.
3. **KMPLIFY countersigns it.** Nodes only run apps carrying the Community
   publisher's signature. This SDK cannot make that signature.

The rules the CLI checks are checked again by the gateway and by each node.
The CLI exists to show mistakes early, so changing it gains nothing.

## Layout

```
cli/        the `kmplify` binary and library (Rust)
schema/     kmplify.app.v1.schema.json, for editors and the server side
examples/   hello-function (signs as is), hello-container
docs/       MANIFEST.md, SIGNING.md, ROADMAP.md
```

## Development

```bash
cd cli
cargo test          # unit tests, schema agreement, end-to-end CLI tests
cargo clippy --all-targets -- -D warnings
cargo fmt --check
```

## Security

The SDK is open on purpose: see [SECURITY.md](SECURITY.md) for why that is safe
and for how to report a vulnerability privately.

## License

Apache License 2.0, see [LICENSE](LICENSE) and [NOTICE](NOTICE). Apps you build
with the SDK are yours and may carry any license you choose.
