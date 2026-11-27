//! Which installation this build belongs to.
//!
//! The published app uses the identifier `it.repertica.studio`. Development runs
//! (`npm run tauri:dev`, which adds `tauri.dev.conf.json`) use `it.repertica.studio.dev`, and test
//! runs may pass another one with `--config`. Tauri derives the data folder (archive, media,
//! WebView storage) from the identifier. Everything else that a build could otherwise share with
//! the owner's installed release is scoped here: the restore rollback folder beside the data
//! folder, the keychain service of the mail passwords, the MCP entry written into the AI
//! assistants' configuration, the update feed and the automatic backup timer. So a development
//! build never opens, migrates or overwrites the real archive, and the release keeps every name
//! it has always used.

/// Identifier of the published app. Never change it: every installed archive lives under it.
pub const RELEASE_IDENTIFIER: &str = "it.repertica.studio";

/// Identifier this binary was built with: `tauri.conf.json` plus the override passed with
/// `tauri dev|build --config` (see `build.rs`).
pub const IDENTIFIER: &str = env!("REPERTICA_APP_IDENTIFIER");

/// Opt-in for the automatic backup timer in a non-release build (see [`backup_timer_allowed`]).
pub const DEV_AUTO_BACKUP_VARIABLE: &str = "REPERTICA_DEV_AUTO_BACKUP";

/// True only for the published identifier.
pub fn is_release() -> bool {
    IDENTIFIER == RELEASE_IDENTIFIER
}

/// `name` in the release, `name-<flavor>` in any other build (`repertica-studio-mail-dev`).
pub fn scoped(name: &str) -> String {
    scoped_for(IDENTIFIER, name)
}

/// Title of the main window: other builds say which one they are.
pub fn window_title() -> String {
    title_for(IDENTIFIER)
}

/// "dev" for `it.repertica.studio.dev`; `None` for the release.
fn flavor_of(identifier: &str) -> Option<String> {
    if identifier == RELEASE_IDENTIFIER {
        return None;
    }
    let label = identifier
        .strip_prefix(RELEASE_IDENTIFIER)
        .and_then(|rest| rest.strip_prefix('.'))
        .filter(|rest| !rest.is_empty())
        .unwrap_or(identifier);
    Some(label.chars().map(|c| if c.is_ascii_alphanumeric() { c.to_ascii_lowercase() } else { '-' }).collect())
}

fn scoped_for(identifier: &str, name: &str) -> String {
    match flavor_of(identifier) {
        None => name.to_owned(),
        Some(flavor) => format!("{name}-{flavor}"),
    }
}

fn title_for(identifier: &str) -> String {
    match flavor_of(identifier) {
        None => "Repertica Studio".to_owned(),
        Some(flavor) => format!("Repertica Studio ({flavor})"),
    }
}

/// A debug build (`tauri dev`, `cargo run`) never runs with the release identifier: it would open,
/// and with a newer schema migrate, the owner's real archive, which the installed release then
/// refuses to open. `npm run tauri:dev` passes the development identifier.
pub fn startup_check(debug_build: bool, identifier: &str) -> Result<(), String> {
    if debug_build && identifier == RELEASE_IDENTIFIER {
        return Err(format!(
            "Questa build di sviluppo usa l'identificatore della release ({RELEASE_IDENTIFIER}) e aprirebbe \
             l'archivio reale. Avviala con `npm run tauri:dev` (identificatore it.repertica.studio.dev, \
             vedi docs/DEV-DATA.md)."
        ));
    }
    Ok(())
}

/// The automatic backup timer writes into the folder saved in the archive and prunes old copies
/// there. An archive restored into a development build still names the owner's backup folder, so
/// outside the release the timer runs only when asked for (`REPERTICA_DEV_AUTO_BACKUP=1`).
pub fn backup_timer_allowed(release: bool, opt_in: Option<&str>) -> bool {
    release || opt_in == Some("1")
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::Value;

    const DEV: &str = "it.repertica.studio.dev";

    fn config(text: &str) -> Value {
        serde_json::from_str(text).unwrap()
    }

    #[test]
    fn the_release_keeps_every_name_it_has_always_used() {
        assert_eq!(config(include_str!("../tauri.conf.json"))["identifier"], RELEASE_IDENTIFIER);
        assert!(config(include_str!("../tauri.release.conf.json")).get("identifier").is_none());
        assert_eq!(flavor_of(RELEASE_IDENTIFIER), None);
        for name in ["repertica-studio", "repertica-studio-mail", ".repertica-studio-restore-rollback"] {
            assert_eq!(scoped_for(RELEASE_IDENTIFIER, name), name);
        }
        assert_eq!(title_for(RELEASE_IDENTIFIER), "Repertica Studio");
    }

    #[test]
    fn the_development_config_gets_its_own_folder_and_names() {
        let dev = config(include_str!("../tauri.dev.conf.json"));
        assert_eq!(dev["identifier"], DEV);
        assert_eq!(flavor_of(DEV).as_deref(), Some("dev"));
        assert_eq!(scoped_for(DEV, "repertica-studio-mail"), "repertica-studio-mail-dev");
        assert_eq!(scoped_for(DEV, "repertica-studio"), "repertica-studio-dev");
        assert_eq!(scoped_for(DEV, ".repertica-studio-restore-rollback"), ".repertica-studio-restore-rollback-dev");
        assert_eq!(title_for(DEV), "Repertica Studio (dev)");
    }

    #[test]
    fn any_other_identifier_is_scoped_too() {
        assert_eq!(scoped_for("it.repertica.studio.e2e", "repertica-studio"), "repertica-studio-e2e");
        assert_eq!(scoped_for("com.example.Other App", "x"), "x-com-example-other-app");
        assert_eq!(scoped_for("it.repertica.studio.", "x"), "x-it-repertica-studio-");
        assert_eq!(scoped_for("it.repertica.studiox", "x"), "x-it-repertica-studiox");
    }

    #[test]
    fn a_debug_build_refuses_the_release_identifier() {
        assert!(startup_check(true, RELEASE_IDENTIFIER).unwrap_err().contains("npm run tauri:dev"));
        assert!(startup_check(true, DEV).is_ok());
        assert!(startup_check(false, RELEASE_IDENTIFIER).is_ok());
        assert!(startup_check(false, DEV).is_ok());
    }

    #[test]
    fn the_backup_timer_runs_outside_the_release_only_on_request() {
        assert!(backup_timer_allowed(true, None));
        assert!(!backup_timer_allowed(false, None));
        assert!(!backup_timer_allowed(false, Some("0")));
        assert!(backup_timer_allowed(false, Some("1")));
    }

    #[test]
    fn this_build_knows_its_identifier() {
        // `cargo test` builds without `--config`, so the compiled identifier is the one in tauri.conf.json;
        // with an override it is a different, scoped one.
        assert!(!IDENTIFIER.is_empty());
        assert_eq!(is_release(), IDENTIFIER == RELEASE_IDENTIFIER);
        assert_eq!(scoped("x") == "x", is_release());
    }
}
