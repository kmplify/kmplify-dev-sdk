//! `kmplify app init`: a starting point that validates as written.

use std::fs;
use std::path::Path;

use crate::manifest::{valid_id, Lane, MANIFEST_FILE};

/// A placeholder digest so a fresh container manifest parses; `validate`
/// names it so nobody signs it by accident.
pub const PLACEHOLDER_DIGEST: &str =
    "0000000000000000000000000000000000000000000000000000000000000000";

pub fn manifest_text(id: &str, lane: Lane) -> String {
    let name = id.split_once('/').map(|(_, n)| n).unwrap_or(id);
    match lane {
        Lane::Function => format!(
            "# KMPLIFY serverless app. Reference: docs/MANIFEST.md\n\
             apiVersion: kmplify.app/v1\n\
             id: {id}\n\
             version: 0.1.0\n\
             lane: function            # a WebAssembly module: runs on every node, no network, no files\n\
             artifact:\n\
             \x20 wasm:\n\
             \x20   path: {name}.wasm   # wasm32-wasip1 command: read stdin, write stdout, exit 0\n\
             resources:\n\
             \x20 mem_mb: 64            # at most 1024\n\
             limits:\n\
             \x20 max_runtime_s: 10     # at most 300\n\
             data_class: none          # none | personal | document\n\
             pricing:\n\
             \x20 model: free           # free | one_off | monthly | per_run\n"
        ),
        Lane::Container => format!(
            "# KMPLIFY serverless app. Reference: docs/MANIFEST.md\n\
             apiVersion: kmplify.app/v1\n\
             id: {id}\n\
             version: 0.1.0\n\
             lane: container           # an OCI image: runs only inside gVisor or a microVM\n\
             artifact:\n\
             \x20 # Pin by digest, never by tag. Replace the zeros with your image's digest.\n\
             \x20 image: registry.kmplify.io/{id}@sha256:{PLACEHOLDER_DIGEST}\n\
             resources:\n\
             \x20 cpus: 1\n\
             \x20 mem_mb: 512\n\
             isolation: gvisor         # gvisor | microvm\n\
             network:\n\
             \x20 mode: none            # none | egress (then list every host:port under allow)\n\
             entrypoint:\n\
             \x20 port: 8080\n\
             \x20 health: /healthz\n\
             limits:\n\
             \x20 max_runtime_s: 900\n\
             \x20 max_concurrency: 4\n\
             data_class: none          # none | personal | document\n\
             pricing:\n\
             \x20 model: free           # free | one_off | monthly | per_run\n"
        ),
    }
}

/// Write the manifest into `dir`. Refuses to replace an existing one.
pub fn init(dir: &Path, id: &str, lane: Lane) -> Result<std::path::PathBuf, String> {
    if !valid_id(id) {
        return Err("--id must be <handle>/<name>, lowercase letters, digits and hyphens (e.g. acme/pdf-summarizer)".into());
    }
    fs::create_dir_all(dir).map_err(|e| format!("cannot create {}: {e}", dir.display()))?;
    let path = dir.join(MANIFEST_FILE);
    if path.exists() {
        return Err(format!(
            "{} already exists; it was not replaced",
            path.display()
        ));
    }
    fs::write(&path, manifest_text(id, lane))
        .map_err(|e| format!("cannot write {}: {e}", path.display()))?;
    Ok(path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::manifest::Manifest;

    #[test]
    fn both_starters_parse_and_validate() {
        for lane in [Lane::Function, Lane::Container] {
            let m = Manifest::from_yaml(&manifest_text("acme/hello-app", lane)).unwrap();
            assert_eq!(m.validate(), vec![], "{lane:?}");
            assert_eq!(m.lane, lane);
        }
    }
}
