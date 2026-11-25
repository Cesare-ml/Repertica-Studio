//! Automatic mail synchronisation: while the application is open, every account's inbox is checked in the
//! background, the unread counter is kept up to date and new mail is reported to the interface (which
//! shows the badge in the menu and, if enabled, a system notification).
//!
//! Only the inbox is checked (a TLS connection per account and minute): the first check of a session just
//! records where the mailbox stands, so opening the application never floods the desktop with
//! notifications for mail that was already there.
use crate::business_mail::{accounts, inbox_snapshot, sender_name, MailAccount};
use crate::db::Database;
use serde::Serialize;
use std::{collections::HashMap, sync::Mutex, time::Duration};
use tauri::{AppHandle, Emitter, Manager};

/// Seconds between two checks.
pub const POLL_SECONDS: u64 = 60;
const FIRST_CHECK_SECONDS: u64 = 4;

#[derive(Clone, Debug, Serialize, PartialEq)]
pub struct FreshMail {
    pub account_id: String,
    pub account_email: String,
    pub uid: u32,
    pub sender: String,
    pub subject: String,
}
#[derive(Clone, Debug, Default, Serialize, PartialEq)]
pub struct AccountSync {
    pub account_id: String,
    pub email: String,
    pub unread: u32,
    pub exists: u32,
    pub uid_validity: u32,
    pub uid_next: u32,
    /// The last check of this account failed (the previous figures are kept).
    pub error: bool,
}
#[derive(Clone, Debug, Default, Serialize, PartialEq)]
pub struct SyncState {
    pub unread_total: u32,
    pub accounts: Vec<AccountSync>,
    pub checked_at: Option<String>,
    /// Mail that arrived since the previous check (newest first); empty on the first check.
    pub fresh: Vec<FreshMail>,
}

#[derive(Default)]
struct Inner {
    state: SyncState,
    /// `(UIDVALIDITY, UIDNEXT)` seen at the previous check, per account.
    baselines: HashMap<String, (u32, u32)>,
}
#[derive(Default)]
pub struct MailWatch { inner: Mutex<Inner>, polling: Mutex<()> }

/// Merge one account's result into the figures: a failed check keeps the previous numbers of that account.
fn merge(previous: &[AccountSync], account: &MailAccount, checked: Result<AccountSync, ()>) -> AccountSync {
    match checked {
        Ok(fresh) => fresh,
        Err(()) => {
            let mut kept = previous.iter().find(|a| a.account_id == account.id).cloned().unwrap_or(AccountSync {
                account_id: account.id.clone(), email: account.email.clone(), ..Default::default()
            });
            kept.error = true;
            kept
        }
    }
}
fn total(accounts: &[AccountSync]) -> u32 { accounts.iter().map(|a| a.unread).sum() }

/// One check of every account. Never runs twice at the same time; a second caller gets the last figures.
pub fn poll(app: &AppHandle) -> SyncState {
    let watch = app.state::<MailWatch>();
    let Ok(_running) = watch.polling.try_lock() else { return snapshot(&watch) };
    let database = app.state::<Database>();
    let list = accounts(&database).unwrap_or_default();
    let (previous, baselines) = {
        let inner = watch.inner.lock().expect("mail watch state");
        (inner.state.accounts.clone(), inner.baselines.clone())
    };
    let mut next_baselines = HashMap::new();
    let mut rows = Vec::new();
    let mut fresh = Vec::new();
    for account in &list {
        let baseline = baselines.get(&account.id).copied();
        let checked = inbox_snapshot(account, baseline).map(|snap| {
            next_baselines.insert(account.id.clone(), (snap.uid_validity, snap.uid_next));
            for message in &snap.fresh {
                fresh.push(FreshMail {
                    account_id: account.id.clone(), account_email: account.email.clone(), uid: message.uid,
                    sender: sender_name(&message.from), subject: message.subject.clone(),
                });
            }
            AccountSync {
                account_id: account.id.clone(), email: account.email.clone(), unread: snap.unseen, exists: snap.exists,
                uid_validity: snap.uid_validity, uid_next: snap.uid_next, error: false,
            }
        }).map_err(|_| ());
        if checked.is_err() {
            // Keep the old baseline so mail that arrives during an outage is still reported afterwards.
            if let Some(old) = baseline { next_baselines.insert(account.id.clone(), old); }
        }
        rows.push(merge(&previous, account, checked));
    }
    let state = SyncState { unread_total: total(&rows), accounts: rows, checked_at: Some(chrono::Utc::now().to_rfc3339()), fresh };
    {
        let mut inner = watch.inner.lock().expect("mail watch state");
        inner.baselines = next_baselines;
        // The mail reported as fresh is not kept: it is delivered once, with this event.
        inner.state = SyncState { fresh: vec![], ..state.clone() };
    }
    let _ = app.emit("mail-status", &state);
    state
}
fn snapshot(watch: &MailWatch) -> SyncState { watch.inner.lock().expect("mail watch state").state.clone() }

/// Background timer: first check shortly after start, then every minute.
pub fn start(app: AppHandle) {
    std::thread::spawn(move || {
        std::thread::sleep(Duration::from_secs(FIRST_CHECK_SECONDS));
        loop {
            let has_accounts = accounts(&app.state::<Database>()).map(|list| !list.is_empty()).unwrap_or(false);
            if has_accounts { poll(&app); }
            std::thread::sleep(Duration::from_secs(POLL_SECONDS));
        }
    });
}

/// Last known figures (what the menu badge shows before the next event arrives).
#[tauri::command]
pub fn mail_sync_state(watch: tauri::State<'_, MailWatch>) -> SyncState { snapshot(&watch) }

/// Check right now, for example after reading a message or changing the accounts.
#[tauri::command]
pub async fn mail_sync_now(app: AppHandle) -> Result<SyncState, String> {
    tauri::async_runtime::spawn_blocking(move || poll(&app)).await.map_err(|_| "Controllo della posta non riuscito".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn account(id: &str) -> MailAccount {
        MailAccount {
            id: id.into(), email: format!("{id}@example.test"), display_name: String::new(), imap_host: "imap.example.test".into(), imap_port: 993,
            imap_username: String::new(), smtp_host: String::new(), smtp_port: 465, smtp_username: String::new(), sent_folder: String::new(),
            signature: String::new(), created_at: String::new(), updated_at: String::new(),
        }
    }
    #[test]
    fn a_failed_check_keeps_the_previous_figures_and_flags_the_error() {
        let before = vec![AccountSync { account_id: "a".into(), email: "a@example.test".into(), unread: 4, exists: 40, ..Default::default() }];
        let kept = merge(&before, &account("a"), Err(()));
        assert_eq!((kept.unread, kept.exists, kept.error), (4, 40, true));
        let unknown = merge(&before, &account("b"), Err(()));
        assert_eq!((unknown.unread, unknown.error, unknown.email.as_str()), (0, true, "b@example.test"));
        let fine = merge(&before, &account("a"), Ok(AccountSync { account_id: "a".into(), unread: 1, ..Default::default() }));
        assert_eq!((fine.unread, fine.error), (1, false));
    }
    #[test]
    fn the_badge_counts_unread_mail_of_every_account() {
        let rows = vec![AccountSync { unread: 3, ..Default::default() }, AccountSync { unread: 2, error: true, ..Default::default() }];
        assert_eq!(total(&rows), 5);
        assert_eq!(total(&[]), 0);
    }
    #[test]
    fn the_timer_is_every_minute_and_the_first_check_is_prompt() {
        assert_eq!(POLL_SECONDS, 60);
        assert!(FIRST_CHECK_SECONDS < 10);
    }
}
