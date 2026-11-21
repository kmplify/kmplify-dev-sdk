//! The JSON Schema and the Rust rules describe one format. These tests keep
//! them from drifting: every example and starter must pass both, and a set of
//! manifests that are wrong must fail both.

use kmplify_cli::manifest::{Lane, Manifest};
use kmplify_cli::scaffold;
use std::path::PathBuf;

fn repo() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..")
}

fn schema() -> jsonschema::Validator {
    let text = std::fs::read_to_string(repo().join("schema/kmplify.app.v1.schema.json")).unwrap();
    jsonschema::validator_for(&serde_json::from_str(&text).unwrap()).unwrap()
}

fn as_json(yaml: &str) -> serde_json::Value {
    serde_yaml_ng::from_str(yaml).unwrap()
}

#[test]
fn examples_and_starters_pass_both() {
    let schema = schema();
    let mut texts = vec![
        std::fs::read_to_string(repo().join("examples/hello-function/kmplify.app.yaml")).unwrap(),
        std::fs::read_to_string(repo().join("examples/hello-container/kmplify.app.yaml")).unwrap(),
    ];
    texts.push(scaffold::manifest_text("acme/hello-app", Lane::Function));
    texts.push(scaffold::manifest_text("acme/hello-app", Lane::Container));
    for text in texts {
        let value = as_json(&text);
        let errors: Vec<String> = schema.iter_errors(&value).map(|e| e.to_string()).collect();
        assert!(errors.is_empty(), "schema refused:\n{text}\n{errors:?}");
        assert_eq!(
            Manifest::from_yaml(&text).unwrap().validate(),
            vec![],
            "{text}"
        );
    }
}

#[test]
fn wrong_manifests_fail_both() {
    let schema = schema();
    let base = "apiVersion: kmplify.app/v1\nid: acme/app\nversion: 1.0.0\n";
    let digest = "a".repeat(64);
    let wrong = [
        format!("{base}lane: container\nartifact: {{image: 'registry.kmplify.io/acme/app:latest'}}\nentrypoint: {{port: 80}}\n"),
        format!("{base}lane: container\nartifact: {{image: 'r.io/acme/app@sha256:{digest}'}}\nentrypoint: {{port: 80}}\nprivileged: true\n"),
        format!("{base}lane: container\nartifact: {{image: 'r.io/acme/app@sha256:{digest}'}}\nentrypoint: {{port: 80}}\nnetwork: {{mode: egress, allow: ['*.example.com:443']}}\n"),
        format!("{base}lane: container\nartifact: {{image: 'r.io/acme/app@sha256:{digest}'}}\nentrypoint: {{port: 80}}\npermissions: [host.docker]\n"),
        "apiVersion: kmplify.app/v2\nid: acme/app\nversion: 1.0.0\nlane: function\nartifact: {wasm: {path: a.wasm}}\n".to_string(),
        "apiVersion: kmplify.app/v1\nid: Acme/App\nversion: 1.0.0\nlane: function\nartifact: {wasm: {path: a.wasm}}\n".to_string(),
        format!("{base}lane: function\nartifact: {{wasm: {{path: a.wasm}}, image: 'r.io/acme/app@sha256:{digest}'}}\n"),
    ];
    for text in wrong {
        assert!(
            !schema.is_valid(&as_json(&text)),
            "schema accepted:\n{text}"
        );
        let rust_ok = Manifest::from_yaml(&text)
            .map(|m| m.validate().is_empty())
            .unwrap_or(false);
        assert!(!rust_ok, "rules accepted:\n{text}");
    }
}
