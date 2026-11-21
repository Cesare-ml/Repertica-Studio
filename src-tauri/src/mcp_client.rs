//! One-click registration of the local MCP server in Claude Desktop, so nobody has to edit
//! configuration files by hand. Only the `repertica-studio` entry is ever added, updated or
//! removed: every other setting and server in the file is preserved, a copy of the original
//! file is kept, and a file that is not valid JSON is never modified.
use crate::{db::Database, util::display_path};
use serde::Serialize;
use serde_json::{json, Map, Value};
use std::{
    fs,
    path::{Path, PathBuf},
};
use tauri::State;

const SERVER_KEY: &str = "repertica-studio";
const CONFIG_FILE: &str = "claude_desktop_config.json";
const BACKUP_SUFFIX: &str = ".repertica-backup";

#[derive(Debug, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ClientStatus {
    pub config_path: String,
    /// Claude Desktop's settings folder exists, i.e. the application has been installed and started.
    pub app_found: bool,
    pub configured: bool,
    /// The registered command and data folder match this installation.
    pub up_to_date: bool,
}

fn entry_for(command: &str, data_dir: &str) -> Value {
    json!({ "command": command, "args": ["--mcp", "--data-dir", data_dir] })
}

/// Claude Desktop's configuration file. The Microsoft Store edition keeps it in a virtualised
/// folder, so an existing Store configuration wins over the classic location.
pub fn default_config_path() -> Option<PathBuf> {
    let var = |name: &str| std::env::var_os(name).filter(|v| !v.is_empty()).map(PathBuf::from);
    if cfg!(windows) {
        let classic = var("APPDATA").map(|p| p.join("Claude"));
        let mut candidates: Vec<PathBuf> = Vec::new();
        if let Some(local) = var("LOCALAPPDATA") {
            if let Ok(packages) = fs::read_dir(local.join("Packages")) {
                for package in packages.flatten() {
                    let name = package.file_name().to_string_lossy().to_string();
                    if name.starts_with("Claude_") {
                        candidates.push(package.path().join("LocalCache").join("Roaming").join("Claude"));
                    }
                }
            }
        }
        candidates.extend(classic);
        let with_file = candidates.iter().find(|dir| dir.join(CONFIG_FILE).is_file());
        let existing = candidates.iter().find(|dir| dir.is_dir());
        return with_file.or(existing).or(candidates.last()).map(|dir| dir.join(CONFIG_FILE));
    }
    let directory = if cfg!(target_os = "macos") {
        var("HOME")?.join("Library").join("Application Support").join("Claude")
    } else {
        var("XDG_CONFIG_HOME").or_else(|| var("HOME").map(|h| h.join(".config")))?.join("Claude")
    };
    Some(directory.join(CONFIG_FILE))
}

fn read_config(path: &Path) -> Result<Map<String, Value>, String> {
    let text = match fs::read_to_string(path) {
        Ok(text) => text,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(Map::new()),
        Err(e) => return Err(crate::util::friendly(e)),
    };
    let text = text.trim_start_matches('\u{feff}');
    if text.trim().is_empty() {
        return Ok(Map::new());
    }
    match serde_json::from_str::<Value>(text) {
        Ok(Value::Object(map)) => Ok(map),
        _ => Err(format!(
            "Il file di configurazione di Claude non è un JSON valido e non è stato modificato: {}",
            display_path(path)
        )),
    }
}

fn servers_of(config: &Map<String, Value>) -> Result<Option<&Map<String, Value>>, String> {
    match config.get("mcpServers") {
        None => Ok(None),
        Some(Value::Object(servers)) => Ok(Some(servers)),
        Some(_) => Err("Il file di configurazione di Claude ha una sezione «mcpServers» non valida e non è stato modificato".into()),
    }
}

pub fn status_at(path: &Path, entry: &Value) -> Result<ClientStatus, String> {
    let config = read_config(path)?;
    let registered = servers_of(&config)?.and_then(|servers| servers.get(SERVER_KEY));
    Ok(ClientStatus {
        config_path: display_path(path),
        app_found: path.parent().is_some_and(Path::is_dir),
        configured: registered.is_some(),
        up_to_date: registered == Some(entry),
    })
}

fn write_config(path: &Path, config: Map<String, Value>) -> Result<(), String> {
    let parent = path.parent().ok_or("Percorso di configurazione non valido")?;
    if path.is_file() {
        let backup = PathBuf::from(format!("{}{BACKUP_SUFFIX}", path.display()));
        // Keep the very first copy: it is the untouched original.
        if !backup.exists() {
            fs::copy(path, &backup).map_err(crate::util::friendly)?;
        }
    }
    let temporary = parent.join(format!(".{CONFIG_FILE}.repertica-tmp"));
    let text = serde_json::to_string_pretty(&Value::Object(config)).map_err(crate::util::friendly)?;
    fs::write(&temporary, format!("{text}\n")).map_err(crate::util::friendly)?;
    if let Err(e) = fs::rename(&temporary, path) {
        let _ = fs::remove_file(&temporary);
        return Err(crate::util::friendly(e));
    }
    Ok(())
}

pub fn configure_at(path: &Path, entry: &Value) -> Result<ClientStatus, String> {
    if !path.parent().is_some_and(Path::is_dir) {
        return Err("Claude Desktop non è stato trovato su questo computer: installalo e avvialo almeno una volta".into());
    }
    let mut config = read_config(path)?;
    servers_of(&config)?; // reject an invalid section before changing anything
    let servers = config.entry("mcpServers").or_insert_with(|| json!({}));
    servers.as_object_mut().expect("checked above").insert(SERVER_KEY.into(), entry.clone());
    write_config(path, config)?;
    status_at(path, entry)
}

pub fn remove_at(path: &Path, entry: &Value) -> Result<ClientStatus, String> {
    let mut config = read_config(path)?;
    servers_of(&config)?;
    let removed = config
        .get_mut("mcpServers")
        .and_then(Value::as_object_mut)
        .and_then(|servers| servers.shift_remove(SERVER_KEY));
    if removed.is_some() {
        write_config(path, config)?;
    }
    status_at(path, entry)
}

fn current_entry(db: &Database) -> Result<Value, String> {
    let executable = std::env::current_exe().map_err(crate::util::friendly)?;
    Ok(entry_for(&display_path(&executable), &display_path(&db.directory)))
}

fn config_path() -> Result<PathBuf, String> {
    default_config_path().ok_or_else(|| "Cartella di configurazione di Claude non determinabile".to_string())
}

#[tauri::command]
pub fn mcp_client_status(db: State<'_, Database>) -> Result<ClientStatus, String> {
    status_at(&config_path()?, &current_entry(&db)?)
}

#[tauri::command]
pub fn mcp_client_configure(db: State<'_, Database>) -> Result<ClientStatus, String> {
    configure_at(&config_path()?, &current_entry(&db)?)
}

#[tauri::command]
pub fn mcp_client_remove(db: State<'_, Database>) -> Result<ClientStatus, String> {
    remove_at(&config_path()?, &current_entry(&db)?)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry() -> Value { entry_for(r"C:\Program Files\Repertica Studio\Repertica Studio.exe", r"C:\Dati\it.repertica.studio") }
    fn workspace() -> (tempfile::TempDir, PathBuf) {
        let dir = tempfile::tempdir().unwrap();
        let claude = dir.path().join("Claude");
        fs::create_dir(&claude).unwrap();
        (dir, claude.join(CONFIG_FILE))
    }
    fn read(path: &Path) -> Value { serde_json::from_str(&fs::read_to_string(path).unwrap()).unwrap() }

    #[test]
    fn configuring_creates_the_file_when_claude_is_installed_but_has_no_config_yet() {
        let (_dir, path) = workspace();
        let before = status_at(&path, &entry()).unwrap();
        assert!(before.app_found && !before.configured && !before.up_to_date);
        let after = configure_at(&path, &entry()).unwrap();
        assert!(after.configured && after.up_to_date);
        assert_eq!(read(&path)["mcpServers"][SERVER_KEY], entry());
        assert!(!PathBuf::from(format!("{}{BACKUP_SUFFIX}", path.display())).exists(), "nothing to back up");
    }

    #[test]
    fn existing_servers_preferences_and_key_order_survive_and_the_original_is_backed_up() {
        let (_dir, path) = workspace();
        let original = "{\n  \"theme\": \"dark\",\n  \"mcpServers\": {\n    \"filesystem\": {\"command\": \"npx\", \"args\": [\"x\"]}\n  },\n  \"zeta\": 1\n}\n";
        fs::write(&path, original).unwrap();
        configure_at(&path, &entry()).unwrap();
        let config = read(&path);
        assert_eq!(config["theme"], "dark");
        assert_eq!(config["zeta"], 1);
        assert_eq!(config["mcpServers"]["filesystem"]["command"], "npx");
        assert_eq!(config["mcpServers"][SERVER_KEY], entry());
        let keys: Vec<&String> = config.as_object().unwrap().keys().collect();
        assert_eq!(keys, ["theme", "mcpServers", "zeta"], "the user's key order is preserved");
        let backup = PathBuf::from(format!("{}{BACKUP_SUFFIX}", path.display()));
        assert_eq!(fs::read_to_string(&backup).unwrap(), original);
        // A second change keeps the first, untouched backup.
        configure_at(&path, &entry_for("C:\\Altro\\app.exe", "C:\\Dati")).unwrap();
        assert_eq!(fs::read_to_string(&backup).unwrap(), original);
    }

    #[test]
    fn a_moved_application_is_reported_as_out_of_date_and_can_be_updated() {
        let (_dir, path) = workspace();
        configure_at(&path, &entry()).unwrap();
        let moved = entry_for(r"D:\Portable\Repertica Studio.exe", r"C:\Dati\it.repertica.studio");
        let status = status_at(&path, &moved).unwrap();
        assert!(status.configured && !status.up_to_date);
        assert!(configure_at(&path, &moved).unwrap().up_to_date);
    }

    #[test]
    fn removing_deletes_only_this_server() {
        let (_dir, path) = workspace();
        fs::write(&path, r#"{"mcpServers":{"other":{"command":"x"}}}"#).unwrap();
        configure_at(&path, &entry()).unwrap();
        let status = remove_at(&path, &entry()).unwrap();
        assert!(!status.configured);
        let config = read(&path);
        assert!(config["mcpServers"].get(SERVER_KEY).is_none());
        assert_eq!(config["mcpServers"]["other"]["command"], "x");
        // Removing when nothing is registered changes nothing and does not fail.
        assert!(!remove_at(&path, &entry()).unwrap().configured);
    }

    #[test]
    fn an_unreadable_or_unexpected_file_is_never_overwritten() {
        let (_dir, path) = workspace();
        for broken in ["{ not json", "[1,2]", "\"text\"", r#"{"mcpServers": []}"#] {
            fs::write(&path, broken).unwrap();
            assert!(configure_at(&path, &entry()).is_err(), "{broken}");
            assert!(remove_at(&path, &entry()).is_err(), "{broken}");
            assert_eq!(fs::read_to_string(&path).unwrap(), broken, "the file must stay untouched");
        }
        fs::write(&path, "\u{feff}{\"a\":1}").unwrap();
        assert!(configure_at(&path, &entry()).is_ok(), "a UTF-8 byte order mark is tolerated");
        fs::write(&path, "  \n").unwrap();
        assert!(configure_at(&path, &entry()).is_ok(), "an empty file is treated as no configuration");
    }

    #[test]
    fn nothing_is_created_when_claude_desktop_is_not_installed() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("Claude").join(CONFIG_FILE);
        assert!(!status_at(&path, &entry()).unwrap().app_found);
        assert!(configure_at(&path, &entry()).unwrap_err().contains("non è stato trovato"));
        assert!(!dir.path().join("Claude").exists());
    }

    #[test]
    fn the_registered_command_uses_the_mcp_flag_and_the_archive_folder() {
        let value = entry();
        assert_eq!(value["args"], json!(["--mcp", "--data-dir", r"C:\Dati\it.repertica.studio"]));
        assert!(default_config_path().map(|p| p.ends_with(CONFIG_FILE)).unwrap_or(true));
    }
}
