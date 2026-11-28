//! Updates from the project's GitHub Releases: check, download, verify, install and restart.
//!
//! * The feed address is fixed in the program: nothing coming from the interface can change it.
//! * Packages are signed with the project's key (minisign). The public key is compiled in from the
//!   build environment (`REPERTICA_UPDATER_PUBLIC_KEY`); a build without it simply reports that
//!   updates are not configured, so development builds never install anything.
//! * An update is only installed after the user asks for it. If the automatic backup is switched on,
//!   a fresh copy of the archive is made first.
use serde::Serialize;
use std::sync::Mutex;
use std::time::Duration;
use tauri::{ipc::Channel, AppHandle, Manager};
use tauri_plugin_updater::{Update, UpdaterExt};

use crate::db::Database;

const FEED: &str = "https://github.com/Cesare-ml/Repertica-Studio/releases/latest/download/latest.json";
const DOWNLOAD_PREFIX: &str = "/Cesare-ml/Repertica-Studio/releases/download/";
/// Public key of the project's update signature, set when the release is built.
pub const PUBLIC_KEY: &str = match option_env!("REPERTICA_UPDATER_PUBLIC_KEY") { Some(key) => key, None => "" };

#[derive(Default)]
pub struct UpdateState(Mutex<Inner>);
#[derive(Default)]
struct Inner { busy: bool, pending: Option<Update> }

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateReply {
    /// `available`, `current`, `not_configured`, `check_failed`, `invalid_release`, `busy`, `stale`,
    /// `backup_failed`, `download_failed`, `signature_invalid`, `install_failed`, `installing`.
    pub status: &'static str,
    pub current_version: String,
    pub version: Option<String>,
    pub notes: Option<String>,
    pub date: Option<String>,
}
#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DownloadProgress { downloaded: u64, total: Option<u64> }

fn reply(app: &AppHandle, status: &'static str) -> UpdateReply {
    UpdateReply { status, current_version: app.package_info().version.to_string(), version: None, notes: None, date: None }
}
/// Only packages published in this project's own releases, over HTTPS, are accepted.
fn allowed_artifact(url: &tauri::Url) -> bool {
    url.scheme() == "https" && url.host_str() == Some("github.com") && url.path().starts_with(DOWNLOAD_PREFIX)
}
/// Pre-releases (`1.2.0-beta.1`) never reach people using the normal feed.
fn is_stable(version: &str) -> bool { !version.contains('-') }
/// Only the published identifier installs updates: a development build never replaces itself with the release.
fn is_configured() -> bool { crate::identity::is_release() && !PUBLIC_KEY.trim().is_empty() }
fn limit_notes(notes: &str) -> String { notes.chars().take(4000).collect() }
fn failure_status(error: &str) -> &'static str {
    let error = error.to_lowercase();
    if error.contains("signature") || error.contains("minisign") { "signature_invalid" } else { "download_failed" }
}

async fn find_update(app: &AppHandle) -> Result<Option<Update>, ()> {
    let updater = app.updater_builder()
        .endpoints(vec![FEED.parse().map_err(|_| ())?]).map_err(|_| ())?
        .timeout(Duration::from_secs(20))
        .build().map_err(|_| ())?;
    updater.check().await.map_err(|_| ())
}

#[tauri::command]
pub async fn check_for_update(app: AppHandle) -> UpdateReply {
    if !is_configured() { return reply(&app, "not_configured"); }
    let state = app.state::<UpdateState>();
    {
        let Ok(mut inner) = state.0.lock() else { return reply(&app, "check_failed") };
        if inner.busy { return reply(&app, "busy"); }
        inner.busy = true;
        inner.pending = None;
    }
    let result = find_update(&app).await;
    let Ok(mut inner) = state.0.lock() else { return reply(&app, "check_failed") };
    inner.busy = false;
    match result {
        Ok(Some(update)) if is_stable(&update.version) && allowed_artifact(&update.download_url) => {
            let mut out = reply(&app, "available");
            out.version = Some(update.version.clone());
            out.notes = update.body.as_deref().map(limit_notes);
            out.date = update.date.map(|d| d.to_string());
            inner.pending = Some(update);
            out
        }
        Ok(Some(_)) => reply(&app, "invalid_release"),
        Ok(None) => reply(&app, "current"),
        Err(()) => reply(&app, "check_failed"),
    }
}

/// Make a fresh backup first when the automatic backup is on. Returns `false` if that failed.
fn backup_before_update(app: &AppHandle) -> bool {
    let db = app.state::<Database>();
    crate::auto_backup::backup_for_update(&db, chrono::Local::now().naive_local())
}

/// Download the update announced by the last check, verify its signature and install it.
/// The application restarts by itself (on Windows the installer relaunches it).
#[tauri::command]
pub async fn install_update(app: AppHandle, version: String, skip_backup: bool, progress: Channel<DownloadProgress>) -> UpdateReply {
    let state = app.state::<UpdateState>();
    let update = {
        let Ok(mut inner) = state.0.lock() else { return reply(&app, "install_failed") };
        if inner.busy { return reply(&app, "busy"); }
        match inner.pending.take() {
            Some(update) if update.version == version => { inner.busy = true; update }
            other => { inner.pending = other; return reply(&app, "stale"); }
        }
    };
    let finish = |status: &'static str, keep: Option<Update>| {
        if let Ok(mut inner) = state.0.lock() { inner.busy = false; if keep.is_some() { inner.pending = keep; } }
        reply(&app, status)
    };
    if !skip_backup {
        let handle = app.clone();
        let saved = tauri::async_runtime::spawn_blocking(move || backup_before_update(&handle)).await.unwrap_or(false);
        if !saved { return finish("backup_failed", Some(update)); }
    }
    let mut downloaded: u64 = 0;
    let bytes = match update.download(
        |chunk, total| { downloaded += chunk as u64; let _ = progress.send(DownloadProgress { downloaded, total }); },
        || {},
    ).await {
        Ok(bytes) => bytes,
        Err(error) => return finish(failure_status(&error.to_string()), Some(update)),
    };
    match update.install(bytes) {
        Ok(()) => {
            // On Windows the installer has taken over and this process is already ending.
            if let Ok(mut inner) = state.0.lock() { inner.busy = false; }
            #[cfg(not(target_os = "windows"))]
            app.restart();
            #[cfg(target_os = "windows")]
            reply(&app, "installing")
        }
        Err(_) => finish("install_failed", Some(update)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_https_packages_from_this_projects_releases_are_accepted() {
        let ok = |u: &str| allowed_artifact(&u.parse().unwrap());
        assert!(ok("https://github.com/Cesare-ml/Repertica-Studio/releases/download/v1.2.0/Repertica-Studio_1.2.0_x64-setup.exe"));
        assert!(!ok("http://github.com/Cesare-ml/Repertica-Studio/releases/download/v1.2.0/a.exe"));
        assert!(!ok("https://github.com/someone-else/Repertica-Studio/releases/download/v1.2.0/a.exe"));
        assert!(!ok("https://github.com/Cesare-ml/Repertica-Studio/archive/main.zip"));
        assert!(!ok("https://github.com.evil.test/Cesare-ml/Repertica-Studio/releases/download/v1/a.exe"));
        assert!(!ok("https://evil.test/Cesare-ml/Repertica-Studio/releases/download/v1/a.exe"));
    }
    #[test]
    fn prereleases_are_not_offered_and_notes_are_bounded() {
        assert!(is_stable("1.2.0") && !is_stable("1.2.0-beta.1") && !is_stable("2.0.0-rc.2"));
        assert_eq!(limit_notes(&"è".repeat(5000)).chars().count(), 4000);
    }
    #[test]
    fn failures_are_told_apart() {
        assert_eq!(failure_status("Signature verification failed"), "signature_invalid");
        assert_eq!(failure_status("minisign: bad key"), "signature_invalid");
        assert_eq!(failure_status("connection reset"), "download_failed");
    }
    #[test]
    fn the_feed_is_fixed_https_and_in_this_repository() {
        assert!(FEED.starts_with("https://github.com/Cesare-ml/Repertica-Studio/releases/latest/download/"));
        assert!(FEED.parse::<tauri::Url>().is_ok());
    }
}
