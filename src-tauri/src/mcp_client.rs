//! One-click registration of the local MCP server in the AI assistants installed on this
//! computer, so nobody has to edit configuration files by hand.
//!
//! Supported: Claude Desktop, Codex (OpenAI), Cursor, Windsurf, Gemini CLI and VS Code. Only the
//! `repertica-studio` entry is ever added, updated or removed: every other setting and server in
//! the file is preserved (comments and layout too, for TOML), a copy of the original file is
//! kept, and a file that cannot be parsed is never modified.
use crate::{db::Database, util::display_path};
use serde::Serialize;
use serde_json::{json, Map, Value};
use std::{
    fs,
    path::{Path, PathBuf},
};
use tauri::State;
use toml_edit::{value, Array, DocumentMut, Item, Table};

const SERVER_KEY: &str = "repertica-studio";
const BACKUP_SUFFIX: &str = ".repertica-backup";

#[derive(Clone, Copy, Debug, PartialEq)]
enum Format {
    /// JSON object holding the servers under `key`; `typed` adds `"type": "stdio"` (VS Code).
    Json { key: &'static str, typed: bool },
    /// Codex: `[mcp_servers.<name>]` tables in `config.toml`.
    Toml,
}

#[derive(Clone, Copy, Debug)]
pub struct Client {
    pub id: &'static str,
    pub name: &'static str,
    format: Format,
}

pub const CLIENTS: [Client; 6] = [
    Client { id: "claude-desktop", name: "Claude Desktop", format: Format::Json { key: "mcpServers", typed: false } },
    Client { id: "codex", name: "Codex (OpenAI)", format: Format::Toml },
    Client { id: "cursor", name: "Cursor", format: Format::Json { key: "mcpServers", typed: false } },
    Client { id: "windsurf", name: "Windsurf", format: Format::Json { key: "mcpServers", typed: false } },
    Client { id: "gemini-cli", name: "Gemini CLI", format: Format::Json { key: "mcpServers", typed: false } },
    Client { id: "vscode", name: "Visual Studio Code", format: Format::Json { key: "servers", typed: true } },
];

#[derive(Debug, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ClientStatus {
    pub id: String,
    pub name: String,
    pub config_path: String,
    /// The assistant's settings folder exists, i.e. it has been installed and started once.
    pub app_found: bool,
    pub configured: bool,
    /// The registered command and data folder match this installation.
    pub up_to_date: bool,
    /// Why the configuration could not be read (it is then left untouched).
    pub error: Option<String>,
}

fn client(id: &str) -> Result<&'static Client, String> {
    CLIENTS.iter().find(|c| c.id == id).ok_or_else(|| "Assistente non supportato".to_string())
}

fn var(name: &str) -> Option<PathBuf> { std::env::var_os(name).filter(|v| !v.is_empty()).map(PathBuf::from) }
fn home() -> Option<PathBuf> { if cfg!(windows) { var("USERPROFILE") } else { var("HOME") } }
fn config_dir() -> Option<PathBuf> {
    if cfg!(windows) { var("APPDATA") }
    else if cfg!(target_os = "macos") { home().map(|h| h.join("Library").join("Application Support")) }
    else { var("XDG_CONFIG_HOME").or_else(|| home().map(|h| h.join(".config"))) }
}

/// Claude Desktop's configuration file. The Microsoft Store edition keeps it in a virtualised
/// folder, so an existing Store configuration wins over the classic location.
fn claude_desktop_path() -> Option<PathBuf> {
    const FILE: &str = "claude_desktop_config.json";
    if cfg!(windows) {
        let mut candidates: Vec<PathBuf> = Vec::new();
        if let Some(local) = var("LOCALAPPDATA") {
            if let Ok(packages) = fs::read_dir(local.join("Packages")) {
                for package in packages.flatten() {
                    if package.file_name().to_string_lossy().starts_with("Claude_") {
                        candidates.push(package.path().join("LocalCache").join("Roaming").join("Claude"));
                    }
                }
            }
        }
        candidates.extend(var("APPDATA").map(|p| p.join("Claude")));
        let with_file = candidates.iter().find(|dir| dir.join(FILE).is_file());
        let existing = candidates.iter().find(|dir| dir.is_dir());
        return with_file.or(existing).or(candidates.last()).map(|dir| dir.join(FILE));
    }
    Some(config_dir()?.join("Claude").join(FILE))
}

pub fn config_path(client: &Client) -> Option<PathBuf> {
    match client.id {
        "claude-desktop" => claude_desktop_path(),
        "codex" => Some(var("CODEX_HOME").or_else(|| home().map(|h| h.join(".codex")))?.join("config.toml")),
        "cursor" => Some(home()?.join(".cursor").join("mcp.json")),
        "windsurf" => Some(home()?.join(".codeium").join("windsurf").join("mcp_config.json")),
        "gemini-cli" => Some(home()?.join(".gemini").join("settings.json")),
        "vscode" => Some(config_dir()?.join("Code").join("User").join("mcp.json")),
        _ => None,
    }
}

/// The server entry for JSON based assistants.
fn json_entry(client: &Client, command: &str, data_dir: &str) -> Value {
    let mut entry = json!({ "command": command, "args": ["--mcp", "--data-dir", data_dir] });
    if matches!(client.format, Format::Json { typed: true, .. }) {
        entry.as_object_mut().expect("object").insert("type".into(), json!("stdio"));
    }
    entry
}

/// Copy of the original file, kept once: it is the untouched original.
fn back_up(path: &Path) -> Result<(), String> {
    if path.is_file() {
        let backup = PathBuf::from(format!("{}{BACKUP_SUFFIX}", path.display()));
        if !backup.exists() { fs::copy(path, &backup).map_err(crate::util::friendly)?; }
    }
    Ok(())
}

fn write_atomically(path: &Path, text: &str) -> Result<(), String> {
    let parent = path.parent().ok_or("Percorso di configurazione non valido")?;
    let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("config");
    let temporary = parent.join(format!(".{name}.repertica-tmp"));
    fs::write(&temporary, text).map_err(crate::util::friendly)?;
    if let Err(e) = fs::rename(&temporary, path) {
        let _ = fs::remove_file(&temporary);
        return Err(crate::util::friendly(e));
    }
    Ok(())
}

fn read_text(path: &Path) -> Result<String, String> {
    match fs::read_to_string(path) {
        Ok(text) => Ok(text.trim_start_matches('\u{feff}').to_string()),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(String::new()),
        Err(e) => Err(crate::util::friendly(e)),
    }
}

fn invalid(path: &Path, what: &str) -> String {
    format!("Il file di configurazione non è {what} valido e non è stato modificato: {}", display_path(path))
}

// ---------------------------------------------------------------- JSON

fn read_json(path: &Path) -> Result<Map<String, Value>, String> {
    let text = read_text(path)?;
    if text.trim().is_empty() { return Ok(Map::new()); }
    match serde_json::from_str::<Value>(&text) {
        Ok(Value::Object(map)) => Ok(map),
        _ => Err(invalid(path, "un JSON")),
    }
}

fn json_servers<'a>(config: &'a Map<String, Value>, key: &str, path: &Path) -> Result<Option<&'a Map<String, Value>>, String> {
    match config.get(key) {
        None => Ok(None),
        Some(Value::Object(servers)) => Ok(Some(servers)),
        _ => Err(invalid(path, &format!("una sezione «{key}»"))),
    }
}

// ---------------------------------------------------------------- TOML

fn read_toml(path: &Path) -> Result<DocumentMut, String> {
    let text = read_text(path)?;
    text.parse::<DocumentMut>().map_err(|_| invalid(path, "un TOML"))
}

fn toml_entry(command: &str, data_dir: &str) -> Table {
    let mut table = Table::new();
    table["command"] = value(command);
    let mut args = Array::new();
    for a in ["--mcp", "--data-dir", data_dir] { args.push(a); }
    table["args"] = value(args);
    table
}

fn toml_matches(existing: &Item, command: &str, data_dir: &str) -> bool {
    let Some(table) = existing.as_table_like() else { return false };
    let args: Vec<&str> = table.get("args").and_then(Item::as_array).map(|a| a.iter().filter_map(|v| v.as_str()).collect()).unwrap_or_default();
    table.get("command").and_then(Item::as_str) == Some(command) && args == ["--mcp", "--data-dir", data_dir]
}

// ---------------------------------------------------------------- operations

pub fn status_at(client: &Client, path: &Path, command: &str, data_dir: &str) -> Result<ClientStatus, String> {
    let (configured, up_to_date) = match client.format {
        Format::Json { key, .. } => {
            let config = read_json(path)?;
            let existing = json_servers(&config, key, path)?.and_then(|s| s.get(SERVER_KEY));
            (existing.is_some(), existing == Some(&json_entry(client, command, data_dir)))
        }
        Format::Toml => {
            let doc = read_toml(path)?;
            let existing = doc.get("mcp_servers").and_then(Item::as_table_like).and_then(|t| t.get(SERVER_KEY));
            (existing.is_some(), existing.is_some_and(|e| toml_matches(e, command, data_dir)))
        }
    };
    Ok(ClientStatus { id: client.id.into(), name: client.name.into(), config_path: display_path(path),
        app_found: path.parent().is_some_and(Path::is_dir), configured, up_to_date, error: None })
}

pub fn configure_at(client: &Client, path: &Path, command: &str, data_dir: &str) -> Result<ClientStatus, String> {
    if !path.parent().is_some_and(Path::is_dir) {
        return Err(format!("{} non è stato trovato su questo computer: installalo e avvialo almeno una volta", client.name));
    }
    match client.format {
        Format::Json { key, .. } => {
            let mut config = read_json(path)?;
            json_servers(&config, key, path)?; // reject an invalid section before changing anything
            let servers = config.entry(key).or_insert_with(|| json!({}));
            servers.as_object_mut().expect("checked above").insert(SERVER_KEY.into(), json_entry(client, command, data_dir));
            back_up(path)?;
            write_atomically(path, &format!("{}\n", serde_json::to_string_pretty(&Value::Object(config)).map_err(crate::util::friendly)?))?;
        }
        Format::Toml => {
            let mut doc = read_toml(path)?;
            if doc.get("mcp_servers").is_some_and(|i| !i.is_table_like()) { return Err(invalid(path, "una sezione «mcp_servers»")); }
            if doc.get("mcp_servers").is_none() {
                let mut servers = Table::new();
                servers.set_implicit(true); // only the `[mcp_servers.repertica-studio]` header is written
                doc["mcp_servers"] = Item::Table(servers);
            }
            let servers = doc["mcp_servers"].as_table_like_mut().expect("checked above");
            servers.insert(SERVER_KEY, Item::Table(toml_entry(command, data_dir)));
            back_up(path)?;
            write_atomically(path, &doc.to_string())?;
        }
    }
    status_at(client, path, command, data_dir)
}

pub fn remove_at(client: &Client, path: &Path, command: &str, data_dir: &str) -> Result<ClientStatus, String> {
    match client.format {
        Format::Json { key, .. } => {
            let mut config = read_json(path)?;
            json_servers(&config, key, path)?;
            let removed = config.get_mut(key).and_then(Value::as_object_mut).and_then(|s| s.shift_remove(SERVER_KEY));
            if removed.is_some() {
                back_up(path)?;
                write_atomically(path, &format!("{}\n", serde_json::to_string_pretty(&Value::Object(config)).map_err(crate::util::friendly)?))?;
            }
        }
        Format::Toml => {
            let mut doc = read_toml(path)?;
            if doc.get("mcp_servers").is_some_and(|i| !i.is_table_like()) { return Err(invalid(path, "una sezione «mcp_servers»")); }
            let removed = doc.get_mut("mcp_servers").and_then(Item::as_table_like_mut).and_then(|t| t.remove(SERVER_KEY));
            if removed.is_some() {
                back_up(path)?;
                write_atomically(path, &doc.to_string())?;
            }
        }
    }
    status_at(client, path, command, data_dir)
}

fn identity(db: &Database) -> Result<(String, String), String> {
    let executable = std::env::current_exe().map_err(crate::util::friendly)?;
    Ok((display_path(&executable), display_path(&db.directory)))
}

fn path_of(client: &Client) -> Result<PathBuf, String> {
    config_path(client).ok_or_else(|| format!("Cartella di configurazione di {} non determinabile", client.name))
}

/// Every assistant, with its state. A broken configuration file is reported per assistant.
pub fn statuses(command: &str, data_dir: &str) -> Vec<ClientStatus> {
    CLIENTS.iter().map(|client| {
        let Some(path) = config_path(client) else {
            return ClientStatus { id: client.id.into(), name: client.name.into(), config_path: String::new(), app_found: false,
                configured: false, up_to_date: false, error: None };
        };
        status_at(client, &path, command, data_dir).unwrap_or_else(|error| ClientStatus {
            id: client.id.into(), name: client.name.into(), config_path: display_path(&path),
            app_found: path.parent().is_some_and(Path::is_dir), configured: false, up_to_date: false, error: Some(error) })
    }).collect()
}

#[tauri::command]
pub fn mcp_clients_status(db: State<'_, Database>) -> Result<Vec<ClientStatus>, String> {
    let (command, data_dir) = identity(&db)?;
    Ok(statuses(&command, &data_dir))
}

#[tauri::command]
pub fn mcp_client_configure(db: State<'_, Database>, client_id: String) -> Result<ClientStatus, String> {
    let (command, data_dir) = identity(&db)?;
    let client = client(&client_id)?;
    configure_at(client, &path_of(client)?, &command, &data_dir)
}

#[tauri::command]
pub fn mcp_client_remove(db: State<'_, Database>, client_id: String) -> Result<ClientStatus, String> {
    let (command, data_dir) = identity(&db)?;
    let client = client(&client_id)?;
    remove_at(client, &path_of(client)?, &command, &data_dir)
}

#[cfg(test)]
mod tests {
    use super::*;

    const COMMAND: &str = r"C:\Program Files\Repertica Studio\Repertica Studio.exe";
    const DATA: &str = r"C:\Dati\it.repertica.studio";

    fn by_id(id: &str) -> &'static Client { client(id).unwrap() }
    fn workspace(file: &str) -> (tempfile::TempDir, PathBuf) {
        let dir = tempfile::tempdir().unwrap();
        let folder = dir.path().join("assistente");
        fs::create_dir(&folder).unwrap();
        (dir, folder.join(file))
    }
    fn read_json_value(path: &Path) -> Value { serde_json::from_str(&fs::read_to_string(path).unwrap()).unwrap() }
    fn backup_of(path: &Path) -> PathBuf { PathBuf::from(format!("{}{BACKUP_SUFFIX}", path.display())) }

    #[test]
    fn every_assistant_has_a_path_and_a_unique_id() {
        let ids: std::collections::HashSet<_> = CLIENTS.iter().map(|c| c.id).collect();
        assert_eq!(ids.len(), CLIENTS.len());
        for c in &CLIENTS {
            let path = config_path(c);
            assert!(path.is_some(), "{} has no configuration path", c.id);
        }
        assert!(config_path(by_id("codex")).unwrap().ends_with("config.toml"));
        assert!(config_path(by_id("cursor")).unwrap().ends_with("mcp.json"));
        assert!(client("sconosciuto").is_err());
    }

    #[test]
    fn json_assistants_get_the_entry_merged_without_touching_anything_else() {
        for id in ["claude-desktop", "cursor", "windsurf", "gemini-cli"] {
            let (_dir, path) = workspace("config.json");
            let original = "{\n  \"theme\": \"dark\",\n  \"mcpServers\": {\n    \"filesystem\": {\"command\": \"npx\"}\n  },\n  \"zeta\": 1\n}\n";
            fs::write(&path, original).unwrap();
            let status = configure_at(by_id(id), &path, COMMAND, DATA).unwrap();
            assert!(status.configured && status.up_to_date, "{id}");
            let config = read_json_value(&path);
            assert_eq!(config["mcpServers"]["filesystem"]["command"], "npx");
            assert_eq!(config["mcpServers"][SERVER_KEY]["args"], json!(["--mcp", "--data-dir", DATA]));
            assert_eq!(config.as_object().unwrap().keys().collect::<Vec<_>>(), ["theme", "mcpServers", "zeta"], "key order is preserved");
            assert_eq!(fs::read_to_string(backup_of(&path)).unwrap(), original);
            let removed = remove_at(by_id(id), &path, COMMAND, DATA).unwrap();
            assert!(!removed.configured);
            assert_eq!(read_json_value(&path)["mcpServers"]["filesystem"]["command"], "npx");
        }
    }

    #[test]
    fn vscode_uses_its_own_section_and_marks_the_server_as_stdio() {
        let (_dir, path) = workspace("mcp.json");
        fs::write(&path, r#"{"inputs": [], "servers": {"other": {"type": "http", "url": "https://x.test"}}}"#).unwrap();
        configure_at(by_id("vscode"), &path, COMMAND, DATA).unwrap();
        let config = read_json_value(&path);
        assert_eq!(config["servers"][SERVER_KEY]["type"], "stdio");
        assert_eq!(config["servers"]["other"]["url"], "https://x.test");
        assert!(config.get("mcpServers").is_none());
        assert!(status_at(by_id("vscode"), &path, COMMAND, DATA).unwrap().up_to_date);
    }

    #[test]
    fn codex_gets_a_toml_table_and_keeps_comments_and_other_servers() {
        let (_dir, path) = workspace("config.toml");
        let original = "# my Codex settings\nmodel = \"gpt\"\n\n[mcp_servers.other]\ncommand = \"node\"\nargs = ['a.js']  # keep me\n\n[mcp_servers.other.env]\nKEY = \"secret\"\n\n[features]\nx = true\n";
        fs::write(&path, original).unwrap();
        let status = configure_at(by_id("codex"), &path, COMMAND, DATA).unwrap();
        assert!(status.configured && status.up_to_date);
        let text = fs::read_to_string(&path).unwrap();
        assert!(text.starts_with("# my Codex settings\nmodel = \"gpt\""));
        assert!(text.contains("args = ['a.js']  # keep me"), "comments and quoting survive");
        assert!(text.contains("[mcp_servers.other.env]\nKEY = \"secret\""));
        assert!(text.contains("[features]\nx = true"));
        assert!(text.contains("[mcp_servers.repertica-studio]"));
        let parsed: DocumentMut = text.parse().unwrap();
        let ours = &parsed["mcp_servers"][SERVER_KEY];
        assert_eq!(ours["command"].as_str(), Some(COMMAND), "backslashes survive");
        assert_eq!(ours["args"].as_array().unwrap().iter().filter_map(|v| v.as_str()).collect::<Vec<_>>(), ["--mcp", "--data-dir", DATA]);
        assert_eq!(fs::read_to_string(backup_of(&path)).unwrap(), original);
        let removed = remove_at(by_id("codex"), &path, COMMAND, DATA).unwrap();
        assert!(!removed.configured);
        let after = fs::read_to_string(&path).unwrap();
        assert!(!after.contains("repertica-studio") && after.contains("args = ['a.js']  # keep me") && after.contains("[features]"));
    }

    #[test]
    fn codex_without_any_server_gets_only_our_table() {
        let (_dir, path) = workspace("config.toml");
        configure_at(by_id("codex"), &path, COMMAND, DATA).unwrap();
        let text = fs::read_to_string(&path).unwrap();
        assert!(text.contains("[mcp_servers.repertica-studio]") && !text.contains("[mcp_servers]\n"), "{text}");
        assert!(!backup_of(&path).exists(), "nothing to back up");
    }

    #[test]
    fn a_moved_application_is_reported_as_out_of_date_for_both_formats() {
        for (id, file) in [("cursor", "mcp.json"), ("codex", "config.toml")] {
            let (_dir, path) = workspace(file);
            configure_at(by_id(id), &path, COMMAND, DATA).unwrap();
            let moved = status_at(by_id(id), &path, r"D:\Portable\Repertica Studio.exe", DATA).unwrap();
            assert!(moved.configured && !moved.up_to_date, "{id}");
            assert!(configure_at(by_id(id), &path, r"D:\Portable\Repertica Studio.exe", DATA).unwrap().up_to_date);
        }
    }

    #[test]
    fn unreadable_files_are_never_overwritten() {
        for (id, file, broken) in [("cursor", "mcp.json", vec!["{ not json", "[1,2]", r#"{"mcpServers": []}"#]),
            ("vscode", "mcp.json", vec![r#"{"servers": 5}"#]), ("codex", "config.toml", vec!["[mcp_servers\nx", "mcp_servers = 5\n"])] {
            for text in broken {
                let (_dir, path) = workspace(file);
                fs::write(&path, text).unwrap();
                assert!(configure_at(by_id(id), &path, COMMAND, DATA).is_err(), "{id}: {text}");
                assert!(remove_at(by_id(id), &path, COMMAND, DATA).is_err(), "{id}: {text}");
                assert_eq!(fs::read_to_string(&path).unwrap(), text, "the file must stay untouched");
                assert!(!backup_of(&path).exists());
            }
        }
    }

    #[test]
    fn nothing_is_created_when_the_assistant_is_not_installed() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("assente").join("mcp.json");
        assert!(!status_at(by_id("cursor"), &path, COMMAND, DATA).unwrap().app_found);
        assert!(configure_at(by_id("cursor"), &path, COMMAND, DATA).unwrap_err().contains("non è stato trovato"));
        assert!(!dir.path().join("assente").exists());
    }

    #[test]
    fn a_bom_or_an_empty_file_is_tolerated() {
        let (_dir, path) = workspace("mcp.json");
        fs::write(&path, "\u{feff}{\"a\":1}").unwrap();
        assert!(configure_at(by_id("cursor"), &path, COMMAND, DATA).is_ok());
        fs::write(&path, "  \n").unwrap();
        assert!(configure_at(by_id("cursor"), &path, COMMAND, DATA).is_ok());
        let (_dir, toml) = workspace("config.toml");
        fs::write(&toml, "").unwrap();
        assert!(configure_at(by_id("codex"), &toml, COMMAND, DATA).is_ok());
    }

    #[test]
    fn the_listing_reports_a_broken_file_per_assistant_without_failing_the_others() {
        let list = statuses(COMMAND, DATA);
        assert_eq!(list.len(), CLIENTS.len());
        assert!(list.iter().all(|s| !s.name.is_empty()));
    }
}
