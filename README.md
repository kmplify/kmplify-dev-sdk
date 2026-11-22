# kmplify-dev-sdk

The developer SDK for **serverless apps** in the KMPLIFY ecosystem: what a
developer uses to build an app and sign it, before handing it in on
[marketplace.kmplify.ai](https://marketplace.kmplify.ai), free or paid.

**Step-by-step guide with screenshots:**
[kmplify.io/docs/serverless-apps](https://kmplify.io/docs/serverless-apps)
(German: [kmplify.io/de/docs/serverless-apps](https://kmplify.io/de/docs/serverless-apps)).

A serverless app runs on the KMPLIFY Compute Fabric: on servers KMPLIFY
operates and on peer hardware (KMPLIFY Desktop, `kmplify-node`). Because that
includes other people's machines, an app is always a **signed, sandboxed
package** and never just an image name.

## Status

| Piece | State |
|---|---|
| `kmplify.app/v1` manifest and its JSON Schema | here |
| `kmplify` CLI: `app init`, `app validate`, `keys create/list/show`, `app sign`, `app verify` | here |
| Hand in a signed app for review | on the marketplace: **Settings > Your apps** at [marketplace.kmplify.ai/settings/developer-apps](https://marketplace.kmplify.ai/settings/developer-apps) |
| `kmplify app test` (run locally in the same sandbox a node uses) | planned |
| `kmplify login`, `kmplify app publish` (hand in from the terminal) | planned |
| Runtime libraries (KV, bucket, queue, secrets) for Python, JS, Rust | planned |

See [docs/ROADMAP.md](docs/ROADMAP.md).

## Quick start

You need Rust (stable) to build the CLI, and for a real function the
`wasm32-wasip1` target (`rustup target add wasm32-wasip1`).

```bash
git clone https://github.com/kmplify/kmplify-dev-sdk
cd kmplify-dev-sdk/cli && cargo install --path .   # installs the `kmplify` binary

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

## From your computer to the marketplace

1. **Sign it.** `kmplify app sign` writes `kmplify.app.signed.json` and proves
   who published these exact bytes. Your private key never leaves your machine.
2. **Register once.** Sign in on [marketplace.kmplify.ai](https://marketplace.kmplify.ai)
   and open **Settings > Your apps**. Enter your publisher handle (every app id
   starts with it, for example `acme/word-count`; it cannot be changed later)
   and paste the output of `kmplify keys show`. **Replace key** registers a new
   key later.
3. **Hand it in.** On the same page choose `kmplify.app.signed.json` and, for a
   function, its `.wasm` module, then **Hand in for review**. The page checks
   the signature, your key and your handle before anything is sent.
4. **KMPLIFY reviews it.** Every app, free or paid, waits in one queue and is
   reviewed by a person in the order it arrived. **Your apps** shows the
   position and the decision; a rejection comes with its reason.
5. **KMPLIFY countersigns it.** An approved version gets the Community
   publisher's signature and a draft listing. Nodes only run apps that carry
   that signature; this SDK cannot make it.
6. **Publish it.** In **Your listings** choose **Make public**. People can now
   add the app on the marketplace for the devices it names in `platforms`,
   run it on the KMPLIFY network from the web app, or offline in KMPLIFY
   Desktop.

A new version is signed and handed in the same way and goes through the same
review; once approved, the listing shows it. A copy someone already added keeps
the version they added.

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
