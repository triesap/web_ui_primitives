use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::time::{SystemTime, UNIX_EPOCH};

fn fixture_manifest() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("leptos_render_modes")
        .join("Cargo.toml")
}

fn unique_target_dir(name: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("time")
        .as_nanos();
    std::env::temp_dir().join(format!("web_ui_primitives_{name}_{nanos}"))
}

fn run_fixture(name: &str, command: &str, target: Option<&str>, features: &str) -> Output {
    let target_dir = unique_target_dir(name);
    let mut cargo = Command::new(std::env::var("CARGO").unwrap_or_else(|_| "cargo".to_owned()));
    cargo
        .arg(command)
        .arg("--quiet")
        .arg("--locked")
        .arg("--manifest-path")
        .arg(fixture_manifest())
        .arg("--target-dir")
        .arg(&target_dir)
        .arg("--no-default-features")
        .arg("--features")
        .arg(features);
    if let Some(target) = target {
        cargo.arg("--target").arg(target);
    }

    let output = cargo.output().expect("fixture cargo command");
    fs::remove_dir_all(target_dir).ok();
    output
}

fn assert_fixture_succeeds(name: &str, command: &str, target: Option<&str>, features: &str) {
    let output = run_fixture(name, command, target, features);
    if !output.status.success() {
        panic!(
            "fixture `{name}` failed\nstdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
    }
}

#[test]
fn supports_native_ssr_consumer() {
    assert_fixture_succeeds("ssr", "test", None, "ssr");
}

#[test]
fn supports_wasm_csr_consumer() {
    assert_fixture_succeeds("csr", "check", Some("wasm32-unknown-unknown"), "csr");
}

#[test]
fn supports_wasm_hydrate_consumer() {
    assert_fixture_succeeds(
        "hydrate",
        "check",
        Some("wasm32-unknown-unknown"),
        "hydrate",
    );
}

#[test]
fn rejects_conflicting_render_modes() {
    let output = run_fixture("conflicting", "check", None, "csr,ssr");
    assert!(!output.status.success());
    assert!(
        String::from_utf8_lossy(&output.stderr).contains(
            "`csr`, `hydrate`, and `ssr` are mutually exclusive \
             web_ui_primitives_leptos render modes"
        ),
        "unexpected stderr:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
}
