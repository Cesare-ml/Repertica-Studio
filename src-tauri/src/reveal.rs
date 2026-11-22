//! Show an exported file (or a folder) in the system file manager.
//! No shell is involved: the program and its arguments are built here and the path is never interpreted.
use std::path::Path;
use std::process::Command;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Target { Windows, MacOs, Other }

/// What to launch: a program, its normal arguments and, on Windows, one raw argument
/// (Explorer needs `/select,"path"` with the quotes inside the argument itself).
#[derive(Debug, PartialEq)]
pub struct Plan { pub program: &'static str, pub args: Vec<String>, pub raw: Option<String> }

pub fn plan(target: Target, path: &str, is_dir: bool) -> Plan {
    match (target, is_dir) {
        (Target::Windows, true) => Plan { program: "explorer.exe", args: vec![], raw: Some(format!("\"{path}\"")) },
        (Target::Windows, false) => Plan { program: "explorer.exe", args: vec![], raw: Some(format!("/select,\"{path}\"")) },
        (Target::MacOs, true) => Plan { program: "open", args: vec![path.to_string()], raw: None },
        (Target::MacOs, false) => Plan { program: "open", args: vec!["-R".into(), path.to_string()], raw: None },
        (Target::Other, true) => Plan { program: "xdg-open", args: vec![path.to_string()], raw: None },
        (Target::Other, false) => {
            // No portable "select this file": open the folder that contains it.
            let parent = Path::new(path).parent().map(|p| p.to_string_lossy().into_owned()).unwrap_or_else(|| path.to_string());
            Plan { program: "xdg-open", args: vec![parent], raw: None }
        }
    }
}

fn current_target() -> Target {
    if cfg!(target_os = "windows") { Target::Windows } else if cfg!(target_os = "macos") { Target::MacOs } else { Target::Other }
}

/// A path the file manager may be asked to show: absolute, existing and free of control characters and quotes.
fn checked(path: &str) -> Result<bool, String> {
    if path.trim().is_empty() || path.len() > 4096 || path.chars().any(|c| c.is_control() || c == '"') {
        return Err("Percorso non valido".into());
    }
    let p = Path::new(path);
    if !p.is_absolute() { return Err("Percorso non valido".into()); }
    let metadata = std::fs::symlink_metadata(p).map_err(|_| "Il file o la cartella non esiste più".to_string())?;
    Ok(metadata.is_dir())
}

#[tauri::command]
pub fn reveal_in_folder(path: String) -> Result<(), String> {
    let is_dir = checked(&path)?;
    // The verbatim prefix added by canonicalisation is not understood by Explorer.
    let shown = crate::util::display_path(Path::new(&path));
    let plan = plan(current_target(), &shown, is_dir);
    let mut command = Command::new(plan.program);
    command.args(&plan.args);
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        if let Some(raw) = &plan.raw { command.raw_arg(raw); }
    }
    // Explorer reports a non-zero exit status even when it works, so only a failure to start is an error.
    command.spawn().map(|_| ()).map_err(|_| "Impossibile aprire la cartella".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn explorer_gets_one_raw_argument_with_the_quotes_inside() {
        let file = plan(Target::Windows, r"C:\Utenti\Anna Rossi\Brochure 2026.pdf", false);
        assert_eq!(file.program, "explorer.exe");
        assert_eq!(file.raw.as_deref(), Some(r#"/select,"C:\Utenti\Anna Rossi\Brochure 2026.pdf""#));
        assert!(file.args.is_empty());
        assert_eq!(plan(Target::Windows, r"C:\Utenti", true).raw.as_deref(), Some(r#""C:\Utenti""#));
    }
    #[test]
    fn other_systems_select_the_file_or_open_its_folder_without_a_shell() {
        assert_eq!(plan(Target::MacOs, "/Users/a/b.pdf", false), Plan { program: "open", args: vec!["-R".into(), "/Users/a/b.pdf".into()], raw: None });
        assert_eq!(plan(Target::MacOs, "/Users/a", true).args, vec!["/Users/a"]);
        assert_eq!(plan(Target::Other, "/home/a/b.pdf", false).args, vec!["/home/a"]);
        assert_eq!(plan(Target::Other, "/home/a", true).program, "xdg-open");
    }
    #[test]
    fn only_existing_absolute_clean_paths_are_accepted() {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("brochure.pdf");
        std::fs::write(&file, b"%PDF").unwrap();
        assert_eq!(checked(file.to_str().unwrap()), Ok(false));
        assert_eq!(checked(dir.path().to_str().unwrap()), Ok(true));
        assert!(checked(dir.path().join("manca.pdf").to_str().unwrap()).is_err());
        for bad in ["", "   ", "relativo/file.pdf", "C:\\a\"b.pdf", "/tmp/a\nb"] { assert!(checked(bad).is_err(), "{bad:?}"); }
        assert!(checked(&"a".repeat(5000)).is_err());
    }
}
