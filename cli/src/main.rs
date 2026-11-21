//! `kmplify`: the KMPLIFY developer CLI.
//!
//! This release covers what can be done on the developer's own machine:
//! scaffold an app, check its manifest, create a signing key, sign, verify.
//! Publishing to the marketplace is not here yet: it needs the submission
//! service and the review queue, which do not exist yet (docs/ROADMAP.md).

use clap::{Parser, Subcommand, ValueEnum};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use kmplify_cli::manifest::{Lane, Manifest, MANIFEST_FILE};
use kmplify_cli::scaffold::{self, PLACEHOLDER_DIGEST};
use kmplify_cli::signing::{self, Envelope, SIGNED_FILE};

#[derive(Parser)]
#[command(
    name = "kmplify",
    version,
    about = "Build, sign and verify serverless apps for the KMPLIFY ecosystem"
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Work with a serverless app (kmplify.app.yaml)
    App {
        #[command(subcommand)]
        command: AppCommand,
    },
    /// Manage the keys you sign apps with
    Keys {
        #[command(subcommand)]
        command: KeysCommand,
    },
}

#[derive(Clone, Copy, ValueEnum)]
enum LaneArg {
    Function,
    Container,
}

#[derive(Subcommand)]
enum AppCommand {
    /// Write a starter kmplify.app.yaml
    Init {
        /// Directory to create the app in
        #[arg(default_value = ".")]
        dir: PathBuf,
        /// <handle>/<name>, e.g. acme/pdf-summarizer
        #[arg(long)]
        id: String,
        #[arg(long, value_enum, default_value = "function")]
        lane: LaneArg,
    },
    /// Check a manifest against every rule, and say what is wrong
    Validate {
        /// App directory or manifest file
        #[arg(default_value = ".")]
        path: PathBuf,
    },
    /// Sign the app with your developer key
    Sign {
        /// App directory or manifest file
        #[arg(default_value = ".")]
        path: PathBuf,
        /// Name of the key to sign with
        #[arg(long, default_value = "default")]
        key: String,
        /// Where to write the signed envelope [default: kmplify.app.signed.json beside the manifest]
        #[arg(long)]
        out: Option<PathBuf>,
    },
    /// Check a signed envelope: signature, manifest rules and, for a function, the .wasm bytes
    Verify {
        /// The signed envelope
        #[arg(default_value = SIGNED_FILE)]
        file: PathBuf,
        /// Require this public key (hex). Without it, only integrity is proven, not authorship
        #[arg(long)]
        pubkey: Option<String>,
        /// For a function: the .wasm file to check against the signed sha256
        #[arg(long)]
        wasm: Option<PathBuf>,
    },
}

#[derive(Subcommand)]
enum KeysCommand {
    /// Create a new Ed25519 signing key on this machine
    Create {
        #[arg(long, default_value = "default")]
        name: String,
    },
    /// List your keys and their public halves
    List,
    /// Print one key's public half (what you register with KMPLIFY)
    Show {
        #[arg(long, default_value = "default")]
        name: String,
    },
}

fn main() -> ExitCode {
    match run(Cli::parse()) {
        Ok(()) => ExitCode::SUCCESS,
        Err(message) => {
            eprintln!("kmplify: {message}");
            ExitCode::FAILURE
        }
    }
}

fn manifest_path(path: &Path) -> PathBuf {
    if path.is_dir() {
        path.join(MANIFEST_FILE)
    } else {
        path.to_path_buf()
    }
}

/// Parse and validate; on problems, list all of them.
fn load_valid(path: &Path) -> Result<(PathBuf, Manifest), String> {
    let file = manifest_path(path);
    let text =
        fs::read_to_string(&file).map_err(|e| format!("cannot read {}: {e}", file.display()))?;
    let manifest = Manifest::from_yaml(&text).map_err(|e| format!("{}: {e}", file.display()))?;
    let mut problems: Vec<String> = manifest
        .validate()
        .iter()
        .map(ToString::to_string)
        .collect();
    if manifest
        .artifact
        .image
        .as_deref()
        .is_some_and(|i| i.ends_with(PLACEHOLDER_DIGEST))
    {
        problems.push(
            "artifact.image: still carries the placeholder digest from `kmplify app init`".into(),
        );
    }
    if problems.is_empty() {
        Ok((file, manifest))
    } else {
        Err(format!(
            "{} has {} problem(s):\n  - {}",
            file.display(),
            problems.len(),
            problems.join("\n  - ")
        ))
    }
}

fn run(cli: Cli) -> Result<(), String> {
    match cli.command {
        Command::App { command } => match command {
            AppCommand::Init { dir, id, lane } => {
                let lane = match lane {
                    LaneArg::Function => Lane::Function,
                    LaneArg::Container => Lane::Container,
                };
                let path = scaffold::init(&dir, &id, lane)?;
                println!("wrote {}", path.display());
                println!(
                    "next: edit it, then `kmplify app validate {}`",
                    dir.display()
                );
                Ok(())
            }
            AppCommand::Validate { path } => {
                let (file, manifest) = load_valid(&path)?;
                println!("{} is valid", file.display());
                println!("  app       {} {}", manifest.id, manifest.version);
                println!("  lane      {:?}", manifest.lane);
                println!("  sandbox   {:?}", manifest.effective_isolation());
                match &manifest.network {
                    Some(n) if !n.allow.is_empty() => {
                        println!("  network   egress to {}", n.allow.join(", "))
                    }
                    _ => println!("  network   none"),
                }
                Ok(())
            }
            AppCommand::Sign { path, key, out } => {
                let (file, mut manifest) = load_valid(&path)?;
                let base = file.parent().map(Path::to_path_buf).unwrap_or_default();
                if let Some(wasm) = manifest.artifact.wasm.as_mut() {
                    let wasm_path = base.join(&wasm.path);
                    if !signing::looks_like_wasm(&wasm_path)? {
                        return Err(format!(
                            "{} is not a WebAssembly module",
                            wasm_path.display()
                        ));
                    }
                    let actual = signing::sha256_file(&wasm_path)?;
                    match &wasm.sha256 {
                        Some(declared) if *declared != actual => {
                            return Err(format!(
                                "artifact.wasm.sha256 in the manifest does not match {}\n  manifest {declared}\n  file     {actual}",
                                wasm_path.display()
                            ));
                        }
                        _ => wasm.sha256 = Some(actual),
                    }
                }
                let signing_key = signing::load_key(&key)?;
                let envelope = signing::sign(&manifest, &signing_key);
                let out = out.unwrap_or_else(|| base.join(SIGNED_FILE));
                let text = serde_json::to_string_pretty(&envelope)
                    .expect("an envelope is always serialisable");
                fs::write(&out, text + "\n")
                    .map_err(|e| format!("cannot write {}: {e}", out.display()))?;
                println!("signed {} {} with key {key}", manifest.id, manifest.version);
                println!("  pubkey  {}", envelope.pubkey);
                if let Some(w) = &manifest.artifact.wasm {
                    println!("  wasm    sha256 {}", w.sha256.as_deref().unwrap_or(""));
                }
                println!("  wrote   {}", out.display());
                println!("this is your signature as the developer. An app runs on the network only after KMPLIFY's review.");
                Ok(())
            }
            AppCommand::Verify { file, pubkey, wasm } => {
                let text = fs::read_to_string(&file)
                    .map_err(|e| format!("cannot read {}: {e}", file.display()))?;
                let envelope: Envelope = serde_json::from_str(&text)
                    .map_err(|e| format!("{} is not a signed envelope: {e}", file.display()))?;
                let manifest = signing::verify(&envelope, pubkey.as_deref())?;
                if let Some(wasm_path) = &wasm {
                    let declared = manifest
                        .artifact
                        .wasm
                        .as_ref()
                        .and_then(|w| w.sha256.as_deref())
                        .ok_or("--wasm was given but the signed manifest carries no wasm sha256")?;
                    let actual = signing::sha256_file(wasm_path)?;
                    if declared != actual {
                        return Err(format!("{} is not the module that was signed\n  signed {declared}\n  file   {actual}", wasm_path.display()));
                    }
                }
                println!("signature valid: {} {}", manifest.id, manifest.version);
                println!("  signed by {}", envelope.pubkey);
                if pubkey.is_none() {
                    println!("  note: no --pubkey given, so this proves the envelope is intact, not who signed it");
                }
                if wasm.is_some() {
                    println!("  wasm bytes match the signed sha256");
                }
                Ok(())
            }
        },
        Command::Keys { command } => match command {
            KeysCommand::Create { name } => {
                let (path, pubkey) = signing::create_key(&name)?;
                println!("created key {name}");
                println!(
                    "  private key  {}  (keep it; never commit or share it)",
                    path.display()
                );
                println!("  public key   {pubkey}");
                println!(
                    "back the private key up: apps signed with it can only be updated with it."
                );
                Ok(())
            }
            KeysCommand::List => {
                let keys = signing::list_keys()?;
                if keys.is_empty() {
                    println!("no keys yet; create one with `kmplify keys create`");
                }
                for (name, pubkey) in keys {
                    println!("{name}\t{pubkey}");
                }
                Ok(())
            }
            KeysCommand::Show { name } => {
                let key = signing::load_key(&name)?;
                println!("{}", hex::encode(key.verifying_key().to_bytes()));
                Ok(())
            }
        },
    }
}
