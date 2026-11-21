//! End-to-end check of `repertica-studio --mcp`: the real executable, real pipes, no window.
use serde_json::{json, Value};
use std::{
    io::{BufRead, BufReader, Write},
    process::{Command, Stdio},
};

fn archive() -> tempfile::TempDir {
    let dir = tempfile::tempdir().unwrap();
    let db = repertica_studio_lib::db::open(dir.path().to_path_buf()).unwrap();
    db.conn.lock().unwrap().execute_batch(
        "INSERT INTO artworks(id,code,title,status,created_at,updated_at)
         VALUES('a','OP-1','Prima opera','available','t','t');",
    ).unwrap();
    drop(db);
    dir
}

#[test]
fn executable_speaks_mcp_over_stdio_and_stops_when_input_closes() {
    let dir = archive();
    let mut child = Command::new(env!("CARGO_BIN_EXE_repertica-studio"))
        .args(["--mcp", "--data-dir"])
        .arg(dir.path())
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let mut stdin = child.stdin.take().unwrap();
    let mut stdout = BufReader::new(child.stdout.take().unwrap());
    let mut ask = |request: Value| -> Value {
        writeln!(stdin, "{request}").unwrap();
        stdin.flush().unwrap();
        let mut line = String::new();
        stdout.read_line(&mut line).unwrap();
        serde_json::from_str(&line).unwrap_or_else(|_| panic!("not JSON: {line:?}"))
    };
    let init = ask(json!({"jsonrpc": "2.0", "id": 1, "method": "initialize",
        "params": {"protocolVersion": "2025-06-18", "capabilities": {}, "clientInfo": {"name": "test", "version": "0"}}}));
    assert_eq!(init["result"]["serverInfo"]["name"], "repertica-studio");
    let found = ask(json!({"jsonrpc": "2.0", "id": 2, "method": "tools/call",
        "params": {"name": "search_artworks", "arguments": {"query": "prima"}}}));
    let text = found["result"]["content"][0]["text"].as_str().unwrap();
    assert!(text.contains("OP-1"), "{text}");
    drop(ask);
    drop(stdin); // closing stdin ends the session
    let status = child.wait().unwrap();
    assert!(status.success());
}

#[test]
fn missing_archive_is_a_clear_startup_error_not_a_hang() {
    let empty = tempfile::tempdir().unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_repertica-studio"))
        .args(["--mcp", "--data-dir"])
        .arg(empty.path())
        .stdin(Stdio::null())
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&output.stderr).contains("Nessun archivio"));
    assert!(output.stdout.is_empty(), "stdout is reserved for protocol messages");
}
