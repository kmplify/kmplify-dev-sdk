# kmplify-dev-sdk

The developer SDK for **serverless apps** in the KMPLIFY ecosystem: what a
developer uses to build an app, sign it, and hand it in for
[marketplace.kmplify.ai](https://marketplace.kmplify.ai), free or paid.

A serverless app runs on the KMPLIFY Compute Fabric: on servers KMPLIFY
operates, on peer hardware (KMPLIFY Desktop, `kmplify-node`) and, for
functions, offline in KMPLIFY Desktop. Because that includes other people's
machines, an app is always a **signed, sandboxed package** and never just an
image name.

**Step-by-step guide with screenshots:**
[kmplify.io/docs/serverless-apps](https://kmplify.io/docs/serverless-apps)
(German: [kmplify.io/de/docs/serverless-apps](https://kmplify.io/de/docs/serverless-apps)).

## Status

| Piece | State |
|---|---|
| `kmplify.app/v1` manifest and its JSON Schema | available |
| `kmplify` CLI: `app init`, `app validate`, `keys create/list/show`, `app sign`, `app verify` | available |
| Hand in a signed app for review | available on the marketplace: Settings > [Your apps](https://marketplace.kmplify.ai/settings/developer-apps) |
| `kmplify app test` (run locally in the same sandbox a node uses) | planned; any WASI runtime such as `wasmtime` works meanwhile |
| `kmplify login`, `kmplify app publish` (hand in from the command line) | planned |
| Runtime libraries (KV, bucket, queue, secrets) for Python, JS, Rust | planned |

See [docs/ROADMAP.md](docs/ROADMAP.md).

## Quick start

You need [Rust](https://rustup.rs) and, for a function, the `wasm32-wasip1`
target (`rustup target add wasm32-wasip1`).

```bash
git clone https://github.com/kmplify/kmplify-dev-sdk
cd kmplify-dev-sdk/cli && cargo install --path .   # installs the `kmplify` binary
kmplify --version

kmplify keys create                     # your signing key, stays on this machine
kmplify keys show                       # the public half: register it on the marketplace
kmplify app init my-app --id acme/my-app --lane function
# write and build your module, then:
kmplify app validate my-app
kmplify app sign my-app                 # writes my-app/kmplify.app.signed.json
kmplify app verify my-app/kmplify.app.signed.json --pubkey "$(kmplify keys show)"
```

`examples/hello-function` signs and verifies as it is.

## From signed app to marketplace

1. **Register once.** Sign in on [marketplace.kmplify.ai](https://marketplace.kmplify.ai),
   open Settings > Your apps, and enter your handle (the part before the `/`
   in your app id, fixed for good) and the output of `kmplify keys show`.
2. **Hand in.** Choose `kmplify.app.signed.json` and, for a function, the
   `.wasm` module, then *Hand in for review*. The marketplace checks the
   signature and the module's SHA-256 again. Up to 5 apps can wait per account.
3. **Review.** A person at KMPLIFY reviews every submission in the order it
   arrived. A rejection shows the reviewer's reason on the same page.
4. **Publish.** An approved app gets a private draft listing. Add a
   description and picture, then *Make public* under Your listings. People
   then add it to KMPLIFY Desktop, the web app or their phone, wherever its
   manifest's `platforms` allows, and run it.

A new version is a new submission with a higher `version`, signed with the
same key. The full walkthrough, including run answers and how to stop an app,
is in the [guide](https://kmplify.io/docs/serverless-apps).

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
