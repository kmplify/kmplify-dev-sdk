# Security

## Why this repository is public

Nothing in the SDK is a secret, and nothing in the platform relies on it being
one. The SDK only describes and signs an app. Whether an app runs is decided
elsewhere:

- The server validates every submitted manifest again. The checks in
  `kmplify app validate` exist to give you early feedback, not to protect the
  platform, and a modified CLI gains nothing.
- A person reviews every submission before KMPLIFY countersigns it. Nodes only
  run apps that carry that countersignature. The countersigning key is not in
  this repository and never will be.
- At run time the app is confined by the node's sandbox (Wasm, or a hardened
  container under gVisor), with the resources, network destinations and
  platform services its signed manifest names, and nothing more.
- A published app can be revoked on every node at any time.

Your own signing key stays on your machine, in `~/.config/kmplify/keys` with
owner-only permissions. Never commit it. If it leaks, revoke it in the
developer console and create a new one.

## What must never be committed here

Private keys of any kind, API tokens, internal host names, and the review
criteria or scanner rules used on the server side.

## Reporting a vulnerability

Please do not open a public issue. Use GitHub's private reporting on this
repository (Security, then "Report a vulnerability"). Include the SDK version
(`kmplify --version`), what you did and what happened. We will confirm receipt
and keep you informed until it is fixed.
