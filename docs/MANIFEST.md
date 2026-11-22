# The app manifest: `kmplify.app/v1`

One file, `kmplify.app.yaml`, says what the app is, what it may use and what
it costs. Unknown fields are refused. Schema:
[`schema/kmplify.app.v1.schema.json`](../schema/kmplify.app.v1.schema.json).

The marketplace title, description, icon and screenshots are **not** in the
manifest. They are edited on the marketplace; editing them never changes what
runs.

```yaml
apiVersion: kmplify.app/v1
id: acme/pdf-summarizer          # <handle>/<name>
version: 1.4.0                   # semantic version
lane: container                  # function | container
artifact:
  image: registry.kmplify.io/acme/pdf-summarizer@sha256:<64 hex>
resources:
  cpus: 2
  mem_mb: 4096
  disk_mb: 2048
  gpu: {min_vram_mb: 8192, accelerator: cuda}
isolation: gvisor                # gvisor | microvm
network:
  mode: egress                   # none | egress
  allow: [api.example.com:443]
permissions: [dws.kv, dws.bucket]
platforms: [desktop, web, mobile] # where people can use it
data_class: personal             # none | personal | document
entrypoint: {port: 8080, health: /healthz, ready_timeout_s: 120}
limits: {max_runtime_s: 900, max_concurrency: 4}
pricing: {model: per_run, price_cents: 5}
```

## Fields

| Field | Required | Meaning |
|---|---|---|
| `apiVersion` | yes | `kmplify.app/v1` |
| `id` | yes | `<handle>/<name>`: lowercase letters, digits, hyphens. The handle is your publisher handle |
| `version` | yes | `MAJOR.MINOR.PATCH`, optional pre-release. Every version is reviewed separately |
| `lane` | yes | `function` or `container` |
| `artifact.wasm.path` | function | the `.wasm` file, relative to the manifest. `kmplify app sign` adds its `sha256` |
| `artifact.image` | container | `repository@sha256:<digest>`. A tag is refused: a tag can move, a signature must name exact bytes |
| `resources` | no | what the app asks for. A function: at most 1024 MB, no GPU |
| `isolation` | no | the weakest sandbox the app accepts. Default `wasm` for a function, `gvisor` for a container. A container can never go below `gvisor` |
| `network` | no | default: no network. `egress` needs every `host:port` listed |
| `permissions` | no | platform services the app may use, each limited to the app's own namespace: `dws.kv`, `dws.bucket`, `dws.queue`, `dws.secrets`. Containers only |
| `platforms` | no | where people can use the app: `desktop` (KMPLIFY Desktop), `web` (the web app), `mobile`. Default: `desktop` and `web`. The marketplace offers the app only for the devices named here, and a person chooses which of them to add it to |
| `data_class` | no | `none`, `personal` or `document`. Decides which nodes may host the app |
| `entrypoint` | container | the port the app listens on. Reached through the gateway relay, never a public port |
| `limits` | no | `max_runtime_s` (a function: at most 300), `max_concurrency` |
| `pricing` | no | `free`, `one_off`, `monthly` or `per_run`, with `price_cents`. The marketplace's normal fee applies |

## Network rules

- `allow` entries are public DNS names with a port. IP addresses, wildcards,
  `localhost`, `.local`, `.lan` and `.internal` names are refused.
- On the node, addresses in private, link-local and cloud-metadata ranges are
  dropped after name resolution, whatever the manifest says. An app on
  someone's home PC must never reach their router or NAS.
- A function has no network at all.

## What a function sees

stdin, stdout, stderr and argv. No files, no sockets, no environment
variables. Read the input from stdin, write the result to stdout, exit 0.
Any toolchain that produces a `wasm32-wasip1` command module works: Rust, C,
Zig, TinyGo, AssemblyScript.
