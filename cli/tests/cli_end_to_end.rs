//! The binary, as a developer uses it: init, validate, keys, sign, verify.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

fn kmplify(home: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_kmplify"))
        .env("KMPLIFY_HOME", home)
        .args(args)
        .output()
        .unwrap()
}

fn out(o: &Output) -> String {
    format!(
        "{}{}",
        String::from_utf8_lossy(&o.stdout),
        String::from_utf8_lossy(&o.stderr)
    )
}

fn example(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../examples")
        .join(name)
}

fn copy_example(name: &str, to: &Path) {
    for entry in std::fs::read_dir(example(name)).unwrap().flatten() {
        std::fs::copy(entry.path(), to.join(entry.file_name())).unwrap();
    }
}

#[test]
fn function_sign_and_verify() {
    let home = tempfile::tempdir().unwrap();
    let app = tempfile::tempdir().unwrap();
    copy_example("hello-function", app.path());
    let dir = app.path().to_str().unwrap();

    assert!(kmplify(home.path(), &["app", "validate", dir])
        .status
        .success());

    // No key yet: signing says how to get one.
    let o = kmplify(home.path(), &["app", "sign", dir]);
    assert!(
        !o.status.success() && out(&o).contains("kmplify keys create"),
        "{}",
        out(&o)
    );

    assert!(kmplify(home.path(), &["keys", "create"]).status.success());
    // A second create must not replace the key.
    let o = kmplify(home.path(), &["keys", "create"]);
    assert!(
        !o.status.success() && out(&o).contains("not replaced"),
        "{}",
        out(&o)
    );

    let pubkey = String::from_utf8(kmplify(home.path(), &["keys", "show"]).stdout)
        .unwrap()
        .trim()
        .to_string();
    assert_eq!(pubkey.len(), 64);

    let o = kmplify(home.path(), &["app", "sign", dir]);
    assert!(o.status.success(), "{}", out(&o));
    let signed = app.path().join("kmplify.app.signed.json");
    let wasm = app.path().join("hello.wasm");

    let o = kmplify(
        home.path(),
        &[
            "app",
            "verify",
            signed.to_str().unwrap(),
            "--pubkey",
            &pubkey,
            "--wasm",
            wasm.to_str().unwrap(),
        ],
    );
    assert!(o.status.success(), "{}", out(&o));

    // Another key is refused.
    let o = kmplify(
        home.path(),
        &[
            "app",
            "verify",
            signed.to_str().unwrap(),
            "--pubkey",
            &"0".repeat(64),
        ],
    );
    assert!(
        !o.status.success() && out(&o).contains("different key"),
        "{}",
        out(&o)
    );

    // Changed module bytes are refused.
    let mut bytes = std::fs::read(&wasm).unwrap();
    bytes.extend_from_slice(&[0, 1, 0]); // an empty custom section: still wasm, different bytes
    std::fs::write(&wasm, bytes).unwrap();
    let o = kmplify(
        home.path(),
        &[
            "app",
            "verify",
            signed.to_str().unwrap(),
            "--wasm",
            wasm.to_str().unwrap(),
        ],
    );
    assert!(
        !o.status.success() && out(&o).contains("not the module that was signed"),
        "{}",
        out(&o)
    );

    // A tampered envelope is refused.
    let text = std::fs::read_to_string(&signed)
        .unwrap()
        .replace("0.1.0", "9.9.9");
    std::fs::write(&signed, text).unwrap();
    let o = kmplify(home.path(), &["app", "verify", signed.to_str().unwrap()]);
    assert!(
        !o.status.success() && out(&o).contains("does not match"),
        "{}",
        out(&o)
    );
}

#[test]
fn init_then_validate_and_the_placeholder_digest_is_caught() {
    let home = tempfile::tempdir().unwrap();
    let app = tempfile::tempdir().unwrap();
    let dir = app.path().join("my-app");
    let dir_s = dir.to_str().unwrap();

    let o = kmplify(
        home.path(),
        &[
            "app",
            "init",
            dir_s,
            "--id",
            "acme/my-app",
            "--lane",
            "container",
        ],
    );
    assert!(o.status.success(), "{}", out(&o));
    // init never overwrites.
    let o = kmplify(
        home.path(),
        &[
            "app",
            "init",
            dir_s,
            "--id",
            "acme/my-app",
            "--lane",
            "container",
        ],
    );
    assert!(
        !o.status.success() && out(&o).contains("not replaced"),
        "{}",
        out(&o)
    );

    let o = kmplify(home.path(), &["app", "validate", dir_s]);
    assert!(
        !o.status.success() && out(&o).contains("placeholder digest"),
        "{}",
        out(&o)
    );

    let o = kmplify(
        home.path(),
        &[
            "app",
            "init",
            app.path().join("bad").to_str().unwrap(),
            "--id",
            "NotAnId",
        ],
    );
    assert!(!o.status.success());
}

#[test]
fn a_file_that_is_not_wasm_is_not_signed() {
    let home = tempfile::tempdir().unwrap();
    let app = tempfile::tempdir().unwrap();
    copy_example("hello-function", app.path());
    std::fs::write(app.path().join("hello.wasm"), b"#!/bin/sh\necho hi\n").unwrap();
    assert!(kmplify(home.path(), &["keys", "create"]).status.success());
    let o = kmplify(home.path(), &["app", "sign", app.path().to_str().unwrap()]);
    assert!(
        !o.status.success() && out(&o).contains("not a WebAssembly module"),
        "{}",
        out(&o)
    );
}

#[test]
fn the_container_example_validates_and_signs() {
    let home = tempfile::tempdir().unwrap();
    let app = tempfile::tempdir().unwrap();
    copy_example("hello-container", app.path());
    assert!(kmplify(home.path(), &["keys", "create", "--name", "ci"])
        .status
        .success());
    let o = kmplify(
        home.path(),
        &["app", "sign", app.path().to_str().unwrap(), "--key", "ci"],
    );
    assert!(o.status.success(), "{}", out(&o));
    let o = kmplify(
        home.path(),
        &[
            "app",
            "verify",
            app.path().join("kmplify.app.signed.json").to_str().unwrap(),
        ],
    );
    assert!(
        o.status.success() && out(&o).contains("not who signed it"),
        "{}",
        out(&o)
    );
}
