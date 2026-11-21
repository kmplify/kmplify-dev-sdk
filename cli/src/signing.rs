//! Developer keys and the signed envelope.
//!
//! A signed app is one JSON document:
//!
//! ```json
//! {"format":"kmplify.app.signed/v1","manifest":"<canonical JSON text>",
//!  "pubkey":"<hex>","signature":"<hex>"}
//! ```
//!
//! The Ed25519 signature covers the bytes `kmplify-app-signature-v1\n`
//! followed by the UTF-8 bytes of `manifest` exactly as they stand in the
//! envelope. A verifier checks the signature first and parses second, so two
//! implementations never have to agree on a canonical JSON form. That is the
//! rule the fabric's signed container catalog already follows
//! (kmplify-compute-fabric docs/SIGNED-CATALOG.md, "As built"). The prefix
//! keeps a developer key's signature on an app from ever being read as a
//! signature on anything else.
//!
//! This is the DEVELOPER's signature: it says who published these exact
//! bytes. It is not approval. An app only runs on a node after review, under
//! the Community publisher's countersignature, which this tool cannot make.

use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

use crate::manifest::Manifest;

pub const ENVELOPE_FORMAT: &str = "kmplify.app.signed/v1";
pub const SIGNED_FILE: &str = "kmplify.app.signed.json";
const CONTEXT: &[u8] = b"kmplify-app-signature-v1\n";

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Envelope {
    pub format: String,
    pub manifest: String,
    pub pubkey: String,
    pub signature: String,
}

fn message(manifest_text: &str) -> Vec<u8> {
    let mut m = Vec::with_capacity(CONTEXT.len() + manifest_text.len());
    m.extend_from_slice(CONTEXT);
    m.extend_from_slice(manifest_text.as_bytes());
    m
}

pub fn sign(manifest: &Manifest, key: &SigningKey) -> Envelope {
    let text = manifest.canonical_json();
    let signature = key.sign(&message(&text));
    Envelope {
        format: ENVELOPE_FORMAT.to_string(),
        manifest: text,
        pubkey: hex::encode(key.verifying_key().to_bytes()),
        signature: hex::encode(signature.to_bytes()),
    }
}

/// Check the signature, then parse and re-validate the manifest inside.
/// `expected_pubkey`, when given, must be the key that signed: without it a
/// valid signature only proves the envelope is intact, not whose it is.
pub fn verify(envelope: &Envelope, expected_pubkey: Option<&str>) -> Result<Manifest, String> {
    if envelope.format != ENVELOPE_FORMAT {
        return Err(format!("unknown envelope format {:?}", envelope.format));
    }
    if let Some(expected) = expected_pubkey {
        if !expected.eq_ignore_ascii_case(&envelope.pubkey) {
            return Err("signed by a different key than the one expected".into());
        }
    }
    let key_bytes: [u8; 32] = hex::decode(&envelope.pubkey)
        .ok()
        .and_then(|b| b.try_into().ok())
        .ok_or("pubkey is not 32 bytes of hex")?;
    let sig_bytes: [u8; 64] = hex::decode(&envelope.signature)
        .ok()
        .and_then(|b| b.try_into().ok())
        .ok_or("signature is not 64 bytes of hex")?;
    let key =
        VerifyingKey::from_bytes(&key_bytes).map_err(|_| "pubkey is not a valid Ed25519 key")?;
    key.verify(
        &message(&envelope.manifest),
        &Signature::from_bytes(&sig_bytes),
    )
    .map_err(|_| "signature does not match: the manifest was altered or signed by another key")?;
    let manifest = Manifest::from_json(&envelope.manifest)
        .map_err(|e| format!("signed manifest does not parse: {e}"))?;
    let problems = manifest.validate();
    if !problems.is_empty() {
        let list: Vec<String> = problems.iter().map(ToString::to_string).collect();
        return Err(format!("signed manifest is not valid: {}", list.join("; ")));
    }
    Ok(manifest)
}

pub fn sha256_file(path: &Path) -> Result<String, String> {
    let bytes = fs::read(path).map_err(|e| format!("cannot read {}: {e}", path.display()))?;
    Ok(hex::encode(Sha256::digest(&bytes)))
}

/// A `.wasm` file starts with `\0asm` and version 1. Cheap, and it catches
/// the common mistake of pointing the manifest at the wrong file.
pub fn looks_like_wasm(path: &Path) -> Result<bool, String> {
    let bytes = fs::read(path).map_err(|e| format!("cannot read {}: {e}", path.display()))?;
    Ok(bytes.len() >= 8 && bytes[..8] == [0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00])
}

// ----- key store --------------------------------------------------------------

/// Where keys live: `$KMPLIFY_HOME/keys`, else `~/.config/kmplify/keys`
/// (`%APPDATA%\kmplify\keys` on Windows).
pub fn keys_dir() -> Result<PathBuf, String> {
    if let Some(home) = std::env::var_os("KMPLIFY_HOME").filter(|v| !v.is_empty()) {
        return Ok(PathBuf::from(home).join("keys"));
    }
    if cfg!(windows) {
        if let Some(appdata) = std::env::var_os("APPDATA").filter(|v| !v.is_empty()) {
            return Ok(PathBuf::from(appdata).join("kmplify").join("keys"));
        }
    }
    if let Some(xdg) = std::env::var_os("XDG_CONFIG_HOME").filter(|v| !v.is_empty()) {
        return Ok(PathBuf::from(xdg).join("kmplify").join("keys"));
    }
    let home = std::env::var_os("HOME")
        .filter(|v| !v.is_empty())
        .ok_or("cannot find a home directory; set KMPLIFY_HOME")?;
    Ok(PathBuf::from(home)
        .join(".config")
        .join("kmplify")
        .join("keys"))
}

pub fn valid_key_name(name: &str) -> bool {
    !name.is_empty()
        && name.len() <= 64
        && name
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_')
}

fn key_path(name: &str) -> Result<PathBuf, String> {
    if !valid_key_name(name) {
        return Err("a key name is 1..64 letters, digits, - or _".into());
    }
    Ok(keys_dir()?.join(format!("{name}.key")))
}

/// Create a key. Never overwrites: losing a key that apps were signed with
/// means those apps can no longer be updated under the same identity.
pub fn create_key(name: &str) -> Result<(PathBuf, String), String> {
    let path = key_path(name)?;
    let dir = path.parent().expect("key path has a parent");
    fs::create_dir_all(dir).map_err(|e| format!("cannot create {}: {e}", dir.display()))?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = fs::set_permissions(dir, fs::Permissions::from_mode(0o700));
    }
    let key = SigningKey::generate(&mut rand_core::OsRng);
    let mut options = fs::OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    let mut file = options.open(&path).map_err(|e| match e.kind() {
        std::io::ErrorKind::AlreadyExists => format!(
            "a key named {name} already exists at {}; it was not replaced",
            path.display()
        ),
        _ => format!("cannot write {}: {e}", path.display()),
    })?;
    writeln!(file, "{}", hex::encode(key.to_bytes()))
        .map_err(|e| format!("cannot write {}: {e}", path.display()))?;
    Ok((path, hex::encode(key.verifying_key().to_bytes())))
}

pub fn load_key(name: &str) -> Result<SigningKey, String> {
    let path = key_path(name)?;
    let text = fs::read_to_string(&path).map_err(|_| {
        format!(
            "no key named {name} (looked at {}); create one with `kmplify keys create`",
            path.display()
        )
    })?;
    let seed: [u8; 32] = hex::decode(text.trim())
        .ok()
        .and_then(|b| b.try_into().ok())
        .ok_or_else(|| format!("{} is not a key file", path.display()))?;
    Ok(SigningKey::from_bytes(&seed))
}

/// Name and public key of every key in the store, sorted by name.
pub fn list_keys() -> Result<Vec<(String, String)>, String> {
    let dir = keys_dir()?;
    let mut out = Vec::new();
    let Ok(entries) = fs::read_dir(&dir) else {
        return Ok(out);
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("key") {
            continue;
        }
        let Some(name) = path.file_stem().and_then(|s| s.to_str()) else {
            continue;
        };
        if let Ok(key) = load_key(name) {
            out.push((
                name.to_string(),
                hex::encode(key.verifying_key().to_bytes()),
            ));
        }
    }
    out.sort();
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn manifest() -> Manifest {
        Manifest::from_yaml("apiVersion: kmplify.app/v1\nid: acme/slugify\nversion: 0.1.0\nlane: function\nartifact:\n  wasm: {path: slugify.wasm}\n").unwrap()
    }

    fn key(seed: u8) -> SigningKey {
        SigningKey::from_bytes(&[seed; 32])
    }

    #[test]
    fn a_signed_manifest_verifies_and_round_trips() {
        let env = sign(&manifest(), &key(7));
        let back = verify(&env, None).unwrap();
        assert_eq!(back, manifest());
        assert!(verify(&env, Some(&env.pubkey.to_uppercase())).is_ok());
    }

    #[test]
    fn any_change_to_the_manifest_breaks_the_signature() {
        let mut env = sign(&manifest(), &key(7));
        env.manifest = env.manifest.replace("0.1.0", "0.1.1");
        assert!(verify(&env, None).unwrap_err().contains("does not match"));
    }

    #[test]
    fn another_keys_signature_is_refused() {
        let env = sign(&manifest(), &key(7));
        let other = hex::encode(key(8).verifying_key().to_bytes());
        assert!(verify(&env, Some(&other))
            .unwrap_err()
            .contains("different key"));
        // Swapping the pubkey in the envelope does not help either.
        let mut forged = env.clone();
        forged.pubkey = other;
        assert!(verify(&forged, None).is_err());
    }

    #[test]
    fn a_validly_signed_but_invalid_manifest_is_refused() {
        // Signing does not launder a manifest: verify re-runs the rules.
        let mut m = manifest();
        m.id = "Not Valid".into();
        let env = sign(&m, &key(7));
        assert!(verify(&env, None).unwrap_err().contains("not valid"));
    }

    #[test]
    fn the_signature_is_bound_to_this_purpose() {
        // A raw signature over the manifest text alone must not verify.
        let k = key(7);
        let mut env = sign(&manifest(), &k);
        env.signature = hex::encode(k.sign(env.manifest.as_bytes()).to_bytes());
        assert!(verify(&env, None).is_err());
    }

    #[test]
    fn the_envelope_is_pinned() {
        // The server side verifies this exact document; changing the format
        // must be a deliberate, versioned act.
        let env = sign(&manifest(), &key(7));
        assert_eq!(
            env.manifest,
            r#"{"apiVersion":"kmplify.app/v1","artifact":{"wasm":{"path":"slugify.wasm"}},"id":"acme/slugify","lane":"function","version":"0.1.0"}"#
        );
        assert_eq!(
            env.pubkey,
            "ea4a6c63e29c520abef5507b132ec5f9954776aebebe7b92421eea691446d22c"
        );
    }
}
