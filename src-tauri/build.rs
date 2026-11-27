//! Tauri's build step, plus the identifier of this build baked into the binary as
//! `REPERTICA_APP_IDENTIFIER` (read by `src/identity.rs`). It is `tauri.conf.json`'s identifier,
//! replaced by the one in the override that `tauri dev|build --config <file>` passes in
//! `TAURI_CONFIG` (the same merge Tauri applies), so `npm run tauri:dev` gets
//! `it.repertica.studio.dev` and the release keeps `it.repertica.studio`. Code that runs without a
//! Tauri app (the `--mcp` server) knows which installation it belongs to as well; at startup the
//! app checks that this value matches the identifier Tauri itself uses.
use serde_json::Value;

fn main() {
    tauri_build::build();
    println!("cargo:rerun-if-changed=tauri.conf.json");
    println!("cargo:rerun-if-env-changed=TAURI_CONFIG");
    let base: Value = serde_json::from_str(&std::fs::read_to_string("tauri.conf.json").expect("tauri.conf.json"))
        .expect("tauri.conf.json is valid JSON");
    let mut identifier = base["identifier"].as_str().expect("identifier in tauri.conf.json").to_owned();
    if let Ok(merge) = std::env::var("TAURI_CONFIG") {
        let merge: Value = serde_json::from_str(&merge).expect("TAURI_CONFIG is JSON");
        if let Some(value) = merge.get("identifier").and_then(Value::as_str) {
            identifier = value.to_owned();
        }
    }
    println!("cargo:rustc-env=REPERTICA_APP_IDENTIFIER={identifier}");
}
