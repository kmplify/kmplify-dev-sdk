//! The `kmplify.app/v1` manifest: types, parsing and the rules a manifest
//! must satisfy before it can be signed.
//!
//! The JSON Schema in `schema/kmplify.app.v1.schema.json` describes the same
//! shape for editors and for the server side; `tests/schema_agrees.rs` keeps
//! the two from drifting. The rules that a schema cannot express (lane versus
//! artifact, the isolation floor, the function lane's hard limits) live in
//! [`Manifest::validate`], and every one of them is enforced again by the
//! gateway and the node. This file makes mistakes visible early; it is not
//! what keeps a provider's machine safe.

use serde::{Deserialize, Serialize};
use std::fmt;

pub const API_VERSION: &str = "kmplify.app/v1";
pub const MANIFEST_FILE: &str = "kmplify.app.yaml";

/// The node's hard caps for the function lane (kmplify-node:
/// `PROVIDER_MAX_FUNCTION_MB` tops out at 1024, `…_MS` at 300 s).
pub const FUNCTION_MAX_MEM_MB: u64 = 1024;
pub const FUNCTION_MAX_RUNTIME_S: u64 = 300;
/// The marketplace's own ceiling for a listing price (shim `price_cents`).
pub const MAX_PRICE_CENTS: u64 = 10_000_000;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Lane {
    Function,
    Container,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Isolation {
    Wasm,
    Gvisor,
    Microvm,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum NetworkMode {
    None,
    Egress,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PricingModel {
    Free,
    OneOff,
    Monthly,
    PerRun,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DataClass {
    None,
    Personal,
    Document,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Wasm {
    pub path: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sha256: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Artifact {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub wasm: Option<Wasm>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub image: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Gpu {
    pub min_vram_mb: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub accelerator: Option<String>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Resources {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cpus: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mem_mb: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub disk_mb: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub gpu: Option<Gpu>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Network {
    pub mode: NetworkMode,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub allow: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Entrypoint {
    pub port: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub health: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ready_timeout_s: Option<u64>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Limits {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_runtime_s: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_concurrency: Option<u64>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Pricing {
    pub model: PricingModel,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub price_cents: Option<u64>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Manifest {
    #[serde(rename = "apiVersion")]
    pub api_version: String,
    pub id: String,
    pub version: String,
    pub lane: Lane,
    pub artifact: Artifact,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub resources: Option<Resources>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub isolation: Option<Isolation>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub network: Option<Network>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub permissions: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub data_class: Option<DataClass>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub entrypoint: Option<Entrypoint>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub limits: Option<Limits>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pricing: Option<Pricing>,
}

/// One thing wrong with a manifest, with where it is.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Problem {
    pub field: &'static str,
    pub message: String,
}

impl fmt::Display for Problem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.field, self.message)
    }
}

const PERMISSIONS: &[&str] = &["dws.kv", "dws.bucket", "dws.queue", "dws.secrets"];

impl Manifest {
    pub fn from_yaml(text: &str) -> Result<Self, String> {
        serde_yaml_ng::from_str(text).map_err(|e| e.to_string())
    }

    pub fn from_json(text: &str) -> Result<Self, String> {
        serde_json::from_str(text).map_err(|e| e.to_string())
    }

    /// The text that gets signed: JSON with sorted keys and no whitespace.
    /// (serde_json's map is ordered by key unless `preserve_order` is on, and
    /// this crate does not turn it on.) Verifiers never rebuild this form:
    /// they check the signature over the bytes as sent, then parse.
    pub fn canonical_json(&self) -> String {
        let value = serde_json::to_value(self).expect("a manifest is always serialisable");
        serde_json::to_string(&value).expect("a JSON value is always serialisable")
    }

    /// The sandbox this app will actually be held to.
    pub fn effective_isolation(&self) -> Isolation {
        self.isolation.unwrap_or(match self.lane {
            Lane::Function => Isolation::Wasm,
            Lane::Container => Isolation::Gvisor,
        })
    }

    /// Every rule at once, so a developer fixes a manifest in one pass.
    pub fn validate(&self) -> Vec<Problem> {
        let mut out = Vec::new();
        let mut bad = |field: &'static str, message: String| out.push(Problem { field, message });

        if self.api_version != API_VERSION {
            bad("apiVersion", format!("must be {API_VERSION}"));
        }
        if !valid_id(&self.id) {
            bad("id", "must be <handle>/<name>, lowercase letters, digits and hyphens (e.g. acme/pdf-summarizer)".into());
        }
        if !valid_semver(&self.version) {
            bad("version", "must be a semantic version such as 1.4.0".into());
        }

        match self.lane {
            Lane::Function => {
                match (&self.artifact.wasm, &self.artifact.image) {
                    (Some(w), None) => {
                        if w.path.trim().is_empty() {
                            bad("artifact.wasm.path", "must name the .wasm file".into());
                        }
                        if let Some(h) = &w.sha256 {
                            if !is_hex64(h) {
                                bad(
                                    "artifact.wasm.sha256",
                                    "must be 64 lowercase hex characters".into(),
                                );
                            }
                        }
                    }
                    _ => bad(
                        "artifact",
                        "a function needs artifact.wasm and no artifact.image".into(),
                    ),
                }
                if let Some(i) = self.isolation {
                    if i != Isolation::Wasm {
                        bad("isolation", "a function always runs in the Wasm sandbox; leave isolation out or set wasm".into());
                    }
                }
                if self
                    .network
                    .as_ref()
                    .is_some_and(|n| n.mode != NetworkMode::None)
                {
                    bad(
                        "network",
                        "a function has no network: it sees stdin, stdout, stderr and argv only"
                            .into(),
                    );
                }
                if !self.permissions.is_empty() {
                    bad(
                        "permissions",
                        "a function cannot hold platform permissions; use a container".into(),
                    );
                }
                if self.entrypoint.is_some() {
                    bad(
                        "entrypoint",
                        "a function has no port; it reads stdin and writes stdout".into(),
                    );
                }
                if let Some(r) = &self.resources {
                    if r.gpu.is_some() {
                        bad("resources.gpu", "a function cannot use a GPU".into());
                    }
                    if r.mem_mb.is_some_and(|m| m > FUNCTION_MAX_MEM_MB) {
                        bad(
                            "resources.mem_mb",
                            format!("a function may use at most {FUNCTION_MAX_MEM_MB} MB"),
                        );
                    }
                }
                if self
                    .limits
                    .as_ref()
                    .and_then(|l| l.max_runtime_s)
                    .is_some_and(|s| s > FUNCTION_MAX_RUNTIME_S)
                {
                    bad(
                        "limits.max_runtime_s",
                        format!("a function may run at most {FUNCTION_MAX_RUNTIME_S} s"),
                    );
                }
            }
            Lane::Container => {
                match (&self.artifact.image, &self.artifact.wasm) {
                    (Some(image), None) => {
                        if !pinned_image(image) {
                            bad("artifact.image", "must be pinned by digest: repository@sha256:<64 hex>. A tag can move; a signature has to name exact bytes".into());
                        }
                    }
                    _ => bad(
                        "artifact",
                        "a container needs artifact.image and no artifact.wasm".into(),
                    ),
                }
                if self.isolation.is_some_and(|i| i < Isolation::Gvisor) {
                    bad(
                        "isolation",
                        "third-party containers never run below gvisor".into(),
                    );
                }
                if self.entrypoint.is_none() {
                    bad("entrypoint", "a container needs entrypoint.port".into());
                }
            }
        }

        if let Some(e) = &self.entrypoint {
            if e.port == 0 || e.port > 65535 {
                bad("entrypoint.port", "must be 1..65535".into());
            }
            if e.health.as_ref().is_some_and(|h| !h.starts_with('/')) {
                bad("entrypoint.health", "must be a path starting with /".into());
            }
        }

        if let Some(n) = &self.network {
            match n.mode {
                NetworkMode::None if !n.allow.is_empty() => {
                    bad(
                        "network.allow",
                        "has entries but network.mode is none".into(),
                    );
                }
                NetworkMode::Egress if n.allow.is_empty() => {
                    bad(
                        "network.allow",
                        "egress needs an explicit list of host:port; there is no allow-all".into(),
                    );
                }
                _ => {}
            }
            for entry in &n.allow {
                if let Err(why) = check_allow_entry(entry) {
                    bad("network.allow", format!("{entry}: {why}"));
                }
            }
        }
        for p in &self.permissions {
            if !PERMISSIONS.contains(&p.as_str()) {
                bad(
                    "permissions",
                    format!("unknown permission {p}; known: {}", PERMISSIONS.join(", ")),
                );
            }
        }

        if let Some(r) = &self.resources {
            if r.cpus.is_some_and(|c| !(c > 0.0 && c <= 64.0)) {
                bad("resources.cpus", "must be above 0 and at most 64".into());
            }
            if r.mem_mb.is_some_and(|m| m < 16) {
                bad("resources.mem_mb", "must be at least 16".into());
            }
            if let Some(g) = &r.gpu {
                if g.min_vram_mb < 1024 {
                    bad("resources.gpu.min_vram_mb", "must be at least 1024".into());
                }
                if g.accelerator.as_ref().is_some_and(|a| a != "cuda") {
                    bad("resources.gpu.accelerator", "only cuda is supported".into());
                }
            }
        }

        if let Some(p) = &self.pricing {
            let price = p.price_cents.unwrap_or(0);
            match p.model {
                PricingModel::Free if price != 0 => {
                    bad("pricing.price_cents", "a free app has no price".into())
                }
                PricingModel::Free => {}
                _ if price == 0 => bad(
                    "pricing.price_cents",
                    "a paid model needs a price above 0".into(),
                ),
                _ if price > MAX_PRICE_CENTS => {
                    bad("pricing.price_cents", format!("at most {MAX_PRICE_CENTS}"))
                }
                _ => {}
            }
        }

        out
    }
}

fn valid_segment(s: &str, min: usize, max: usize) -> bool {
    let len = s.len();
    len >= min
        && len <= max
        && s.bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
        && !s.starts_with('-')
}

pub fn valid_id(id: &str) -> bool {
    match id.split_once('/') {
        Some((handle, name)) => valid_segment(handle, 2, 39) && valid_segment(name, 2, 63),
        None => false,
    }
}

fn valid_semver(v: &str) -> bool {
    let (core, pre) = match v.split_once('-') {
        Some((c, p)) => (c, Some(p)),
        None => (v, None),
    };
    let parts: Vec<&str> = core.split('.').collect();
    let numeric = |p: &&str| {
        !p.is_empty()
            && p.bytes().all(|b| b.is_ascii_digit())
            && (p.len() == 1 || !p.starts_with('0'))
    };
    parts.len() == 3
        && parts.iter().all(numeric)
        && pre.is_none_or(|p| {
            !p.is_empty()
                && p.bytes()
                    .all(|b| b.is_ascii_alphanumeric() || b == b'.' || b == b'-')
        })
}

pub fn is_hex64(s: &str) -> bool {
    s.len() == 64
        && s.bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}

fn pinned_image(image: &str) -> bool {
    match image.split_once("@sha256:") {
        Some((repo, digest)) => {
            is_hex64(digest)
                && !repo.is_empty()
                && repo.as_bytes()[0].is_ascii_alphanumeric()
                && repo
                    .bytes()
                    .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b"._/:-".contains(&b))
        }
        None => false,
    }
}

/// `host:port`, where host is a DNS name. An IP literal is refused: an
/// allow-list of names can be reviewed, and a literal address is how an app
/// would ask for a router, a NAS or a cloud metadata service. The node drops
/// private, link-local and metadata ranges after resolution in any case.
fn check_allow_entry(entry: &str) -> Result<(), &'static str> {
    let (host, port) = entry.rsplit_once(':').ok_or("must be host:port")?;
    let port: u32 = port.parse().map_err(|_| "port must be a number")?;
    if port == 0 || port > 65535 {
        return Err("port must be 1..65535");
    }
    if host.is_empty() || host.len() > 253 {
        return Err("host must be a DNS name");
    }
    if !host
        .bytes()
        .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'.' || b == b'-')
    {
        return Err("host must be a lowercase DNS name, no wildcards");
    }
    if host
        .split('.')
        .all(|l| !l.is_empty() && l.bytes().all(|b| b.is_ascii_digit()))
    {
        return Err("an IP address is not accepted, name the host");
    }
    if !host.contains('.')
        || host == "localhost"
        || host.ends_with(".local")
        || host.ends_with(".internal")
        || host.ends_with(".lan")
    {
        return Err("must be a public DNS name");
    }
    if host
        .split('.')
        .any(|l| l.is_empty() || l.starts_with('-') || l.ends_with('-'))
    {
        return Err("host is not a valid DNS name");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn container() -> Manifest {
        Manifest::from_yaml(&format!(
            "apiVersion: kmplify.app/v1\nid: acme/pdf-summarizer\nversion: 1.4.0\nlane: container\nartifact:\n  image: registry.kmplify.io/acme/pdf-summarizer@sha256:{}\nentrypoint: {{port: 8080, health: /healthz}}\n",
            "a".repeat(64)
        ))
        .unwrap()
    }

    fn function() -> Manifest {
        Manifest::from_yaml("apiVersion: kmplify.app/v1\nid: acme/slugify\nversion: 0.1.0\nlane: function\nartifact:\n  wasm: {path: slugify.wasm}\n").unwrap()
    }

    fn fields(m: &Manifest) -> Vec<&'static str> {
        m.validate().into_iter().map(|p| p.field).collect()
    }

    #[test]
    fn minimal_manifests_are_valid() {
        assert_eq!(container().validate(), vec![]);
        assert_eq!(function().validate(), vec![]);
    }

    #[test]
    fn defaults_to_the_lanes_sandbox() {
        assert_eq!(container().effective_isolation(), Isolation::Gvisor);
        assert_eq!(function().effective_isolation(), Isolation::Wasm);
    }

    #[test]
    fn a_container_cannot_ask_for_less_than_gvisor() {
        let mut m = container();
        m.isolation = Some(Isolation::Wasm);
        assert_eq!(fields(&m), vec!["isolation"]);
        m.isolation = Some(Isolation::Microvm);
        assert_eq!(m.validate(), vec![]);
    }

    #[test]
    fn a_tagged_image_is_refused() {
        let mut m = container();
        m.artifact.image = Some("registry.kmplify.io/acme/pdf-summarizer:latest".into());
        assert_eq!(fields(&m), vec!["artifact.image"]);
    }

    #[test]
    fn unknown_fields_do_not_parse() {
        let err = Manifest::from_yaml("apiVersion: kmplify.app/v1\nid: a1/b1\nversion: 1.0.0\nlane: function\nartifact: {wasm: {path: x.wasm}}\nprivileged: true\n").unwrap_err();
        assert!(err.contains("privileged"), "{err}");
    }

    #[test]
    fn a_function_gets_no_network_gpu_port_or_permissions() {
        let mut m = function();
        m.network = Some(Network {
            mode: NetworkMode::Egress,
            allow: vec!["api.example.com:443".into()],
        });
        m.permissions = vec!["dws.kv".into()];
        m.entrypoint = Some(Entrypoint {
            port: 80,
            health: None,
            ready_timeout_s: None,
        });
        m.resources = Some(Resources {
            gpu: Some(Gpu {
                min_vram_mb: 8192,
                accelerator: None,
            }),
            mem_mb: Some(4096),
            ..Default::default()
        });
        m.limits = Some(Limits {
            max_runtime_s: Some(900),
            max_concurrency: None,
        });
        let f = fields(&m);
        for expected in [
            "network",
            "permissions",
            "entrypoint",
            "resources.gpu",
            "resources.mem_mb",
            "limits.max_runtime_s",
        ] {
            assert!(f.contains(&expected), "missing {expected} in {f:?}");
        }
    }

    #[test]
    fn egress_needs_named_public_hosts() {
        let mut m = container();
        m.network = Some(Network {
            mode: NetworkMode::Egress,
            allow: vec![],
        });
        assert_eq!(fields(&m), vec!["network.allow"]);
        for bad in [
            "192.168.1.1:80",
            "169.254.169.254:80",
            "localhost:5432",
            "*.example.com:443",
            "router.lan:80",
            "example.com",
            "example.com:0",
            "nas.local:445",
        ] {
            m.network = Some(Network {
                mode: NetworkMode::Egress,
                allow: vec![bad.into()],
            });
            assert_eq!(fields(&m), vec!["network.allow"], "{bad} should be refused");
        }
        m.network = Some(Network {
            mode: NetworkMode::Egress,
            allow: vec!["api.openai.com:443".into()],
        });
        assert_eq!(m.validate(), vec![]);
    }

    #[test]
    fn pricing_matches_its_model() {
        let mut m = container();
        m.pricing = Some(Pricing {
            model: PricingModel::Free,
            price_cents: Some(5),
        });
        assert_eq!(fields(&m), vec!["pricing.price_cents"]);
        m.pricing = Some(Pricing {
            model: PricingModel::PerRun,
            price_cents: None,
        });
        assert_eq!(fields(&m), vec!["pricing.price_cents"]);
        m.pricing = Some(Pricing {
            model: PricingModel::PerRun,
            price_cents: Some(5),
        });
        assert_eq!(m.validate(), vec![]);
    }

    #[test]
    fn ids_and_versions() {
        assert!(valid_id("acme/pdf-summarizer"));
        for bad in [
            "acme",
            "Acme/x1",
            "acme/",
            "-acme/app",
            "acme/app/extra",
            "a/app",
        ] {
            assert!(!valid_id(bad), "{bad}");
        }
        assert!(valid_semver("1.4.0") && valid_semver("0.1.0-rc.1"));
        for bad in ["1.4", "v1.4.0", "01.4.0", "1.4.0-"] {
            assert!(!valid_semver(bad), "{bad}");
        }
    }

    #[test]
    fn canonical_json_is_sorted_and_stable() {
        let a = container().canonical_json();
        assert!(
            a.starts_with(r#"{"apiVersion":"kmplify.app/v1","artifact":"#),
            "{a}"
        );
        assert_eq!(a, Manifest::from_json(&a).unwrap().canonical_json());
    }
}
