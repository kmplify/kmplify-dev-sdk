# Roadmap

What exists, what comes next, and what each step waits for.

## Here now (0.1)

- Manifest `kmplify.app/v1` with JSON Schema; lanes `function` and `container`.
- CLI: `app init`, `app validate`, `keys create|list|show`, `app sign`, `app verify`.
- Examples and tests that hold the schema and the CLI's rules together.

## Next

| Step | What | Waits for |
|---|---|---|
| `kmplify app test` | run the app locally with the flags a node uses: Wasm limits for a function, gVisor plus the egress allow-list for a container | nothing external; needs Docker and runsc on the developer's machine for containers |
| `kmplify login`, key registration | bind your public key to your KMPLIFY developer account | publisher-key routes in `kmplify-developer-api` |
| `kmplify app publish` | push the artifact to the KMPLIFY registry and create a submission in the review queue | the registry mirror and the submission service |
| `kmplify app status|logs|rollout|deprecate` | follow a submission through review and operate a published app | the developer pages' API in the Control Plane |
| Runtime libraries | KV, bucket, queue, secrets and structured logs for Python, JS and Rust | the node-side sidecar that hands out scoped tokens |
| GitHub Action | build, test, sign and publish from CI | `app publish` |
| `composite` lane | an app made of several functions and containers plus storage, as a `kmplify.dws/v1` manifest | container lane in production |

## Decided

- Every submission, free or paid, is reviewed by a person, first in, first out.
- Paid apps use the marketplace's own revenue split and pricing models.
- Third-party containers never run below gVisor.
