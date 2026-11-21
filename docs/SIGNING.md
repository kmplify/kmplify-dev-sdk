# Keys and signatures

## Your key

`kmplify keys create` makes an Ed25519 key on your machine and stores the
private half in `~/.config/kmplify/keys/<name>.key` (`%APPDATA%\kmplify\keys`
on Windows, or `$KMPLIFY_HOME/keys`), readable only by you. It is never sent
anywhere. `kmplify keys show` prints the public half, which is what you
register with your KMPLIFY developer account.

- Back the private key up. Apps signed with it can only be updated with it.
- Never commit it. This repository's `.gitignore` excludes `*.key`.
- `keys create` never overwrites an existing key.
- For CI, create a separate key (`kmplify keys create --name ci`) and keep it
  in your CI system's secret store.

## The signed envelope

`kmplify app sign` writes `kmplify.app.signed.json`:

```json
{
  "format": "kmplify.app.signed/v1",
  "manifest": "{\"apiVersion\":\"kmplify.app/v1\",...}",
  "pubkey": "<32 bytes, hex>",
  "signature": "<64 bytes, hex>"
}
```

- `manifest` is the manifest as JSON text with sorted keys and no whitespace.
  For a function it includes the sha256 of the `.wasm` file; for a container
  the image digest is already in it.
- `signature` is Ed25519 over the bytes `kmplify-app-signature-v1\n` followed
  by the UTF-8 bytes of `manifest` exactly as they stand in the envelope.
- A verifier checks the signature first and parses the manifest second. No
  two implementations have to agree on a canonical JSON form. The fabric's
  signed container catalog works the same way.

## What a signature does and does not mean

Your signature proves that the holder of your key published exactly these
bytes. It is not permission to run. An app runs on the network only after a
person at KMPLIFY has reviewed that version and the Community publisher key
has countersigned it. If an app turns out to be harmful, KMPLIFY can revoke
it on every node at once.

`kmplify app verify <file> --pubkey <hex>` checks who signed. Without
`--pubkey` it only proves the envelope was not altered.
