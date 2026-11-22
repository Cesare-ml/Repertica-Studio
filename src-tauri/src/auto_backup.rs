//! Scheduled automatic backups, off by default.
//!
//! The artist picks how often (daily, weekly or monthly), at what local time and in which
//! folder. While the application is open a lightweight timer checks once a minute whether a
//! backup is due and runs the same verified backup as the manual command. If the computer was
//! off at the scheduled time, the missed backup runs at the next opportunity (once, never in
//! bursts). A failure (for instance an unplugged drive) is reported and retried after 30 minutes.
//!
//! Optionally only the most recent N automatic backups are kept. Only folders this feature
//! created itself are ever removed, and only after strict checks on their name and content.
use crate::{backup, db::Database, util::display_path};
use chrono::{Datelike, Duration, Local, NaiveDate, NaiveDateTime, NaiveTime};
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use std::{fs, path::Path};
use tauri::{Emitter, Manager, State};

const STAMP: &str = "%Y-%m-%dT%H:%M:%S";
const RETRY_MINUTES: i64 = 30;
const BACKUP_PREFIX: &str = "ReperticaStudio-backup-";

fn err(e: impl std::fmt::Display) -> String { crate::util::friendly(e) }
fn parse_stamp(value: &Option<String>) -> Option<NaiveDateTime> {
    value.as_deref().and_then(|v| NaiveDateTime::parse_from_str(v, STAMP).ok())
}

#[derive(Clone, Debug, PartialEq)]
pub struct Schedule {
    pub enabled: bool,
    /// `DAILY`, `WEEKLY` or `MONTHLY`.
    pub frequency: String,
    /// Local time, `HH:MM`.
    pub time_of_day: String,
    /// 0 = Monday … 6 = Sunday (weekly).
    pub weekday: i64,
    /// 1..=28 (monthly).
    pub day_of_month: i64,
}

impl Schedule {
    fn time(&self) -> NaiveTime {
        NaiveTime::parse_from_str(&self.time_of_day, "%H:%M").unwrap_or(NaiveTime::from_hms_opt(2, 0, 0).unwrap())
    }
    fn at(&self, date: NaiveDate) -> NaiveDateTime { date.and_time(self.time()) }

    /// The most recent scheduled moment that is not after `now`.
    pub fn previous_occurrence(&self, now: NaiveDateTime) -> NaiveDateTime {
        match self.frequency.as_str() {
            "WEEKLY" => {
                let back = (i64::from(now.weekday().num_days_from_monday()) - self.weekday).rem_euclid(7);
                let candidate = self.at(now.date() - Duration::days(back));
                if candidate > now { candidate - Duration::days(7) } else { candidate }
            }
            "MONTHLY" => {
                let day = self.day_of_month.clamp(1, 28) as u32;
                let this_month = self.at(NaiveDate::from_ymd_opt(now.year(), now.month(), day).unwrap());
                if this_month <= now { this_month } else {
                    let (year, month) = if now.month() == 1 { (now.year() - 1, 12) } else { (now.year(), now.month() - 1) };
                    self.at(NaiveDate::from_ymd_opt(year, month, day).unwrap())
                }
            }
            _ => {
                let candidate = self.at(now.date());
                if candidate > now { candidate - Duration::days(1) } else { candidate }
            }
        }
    }

    /// The first scheduled moment strictly after `now`.
    pub fn next_occurrence(&self, now: NaiveDateTime) -> NaiveDateTime {
        match self.frequency.as_str() {
            "WEEKLY" => self.previous_occurrence(now) + Duration::days(7),
            "MONTHLY" => {
                let previous = self.previous_occurrence(now);
                let day = self.day_of_month.clamp(1, 28) as u32;
                let (year, month) = if previous.month() == 12 { (previous.year() + 1, 1) } else { (previous.year(), previous.month() + 1) };
                self.at(NaiveDate::from_ymd_opt(year, month, day).unwrap())
            }
            _ => self.previous_occurrence(now) + Duration::days(1),
        }
    }
}

/// Decide whether a backup must run now.
pub fn is_due(
    schedule: &Schedule, now: NaiveDateTime, enabled_at: Option<NaiveDateTime>,
    last_success: Option<NaiveDateTime>, last_attempt_failed: Option<NaiveDateTime>,
) -> bool {
    if !schedule.enabled { return false; }
    let previous = schedule.previous_occurrence(now);
    // Enabling the schedule never triggers an immediate run for a moment that already passed.
    let since = [enabled_at, last_success].into_iter().flatten().max();
    if since.is_some_and(|s| s >= previous) { return false; }
    !last_attempt_failed.is_some_and(|t| now < t + Duration::minutes(RETRY_MINUTES))
}

#[derive(Clone, Debug, Deserialize)]
pub struct ScheduleInput {
    pub enabled: bool,
    pub frequency: String,
    pub time_of_day: String,
    pub weekday: i64,
    pub day_of_month: i64,
    pub destination: Option<String>,
    /// Keep only the newest N automatic backups; 0 keeps all of them.
    pub keep_last: i64,
}

#[derive(Clone, Debug, Serialize)]
pub struct ScheduleStatus {
    pub enabled: bool,
    pub frequency: String,
    pub time_of_day: String,
    pub weekday: i64,
    pub day_of_month: i64,
    pub destination: Option<String>,
    pub keep_last: i64,
    pub last_success_at: Option<String>,
    pub last_attempt_at: Option<String>,
    /// `OK`, `ERROR` or absent when no run happened yet.
    pub last_status: Option<String>,
    pub last_message: Option<String>,
    pub last_directory: Option<String>,
    pub next_run_at: Option<String>,
}

fn validate(input: &ScheduleInput) -> Result<(), String> {
    if !["DAILY", "WEEKLY", "MONTHLY"].contains(&input.frequency.as_str()) { return Err("Frequenza non valida".into()); }
    if NaiveTime::parse_from_str(&input.time_of_day, "%H:%M").map(|t| t.format("%H:%M").to_string() != input.time_of_day).unwrap_or(true) {
        return Err("Orario non valido (usa HH:MM)".into());
    }
    if !(0..=6).contains(&input.weekday) { return Err("Giorno della settimana non valido".into()); }
    if !(1..=28).contains(&input.day_of_month) { return Err("Il giorno del mese deve essere tra 1 e 28".into()); }
    if !(0..=365).contains(&input.keep_last) { return Err("Numero di copie da conservare non valido".into()); }
    if input.enabled {
        let destination = input.destination.as_deref().map(str::trim).unwrap_or("");
        if destination.is_empty() { return Err("Scegli la cartella in cui salvare i backup automatici".into()); }
        if !Path::new(destination).is_absolute() { return Err("Seleziona la cartella tramite il dialogo di sistema".into()); }
    }
    Ok(())
}

fn schedule_of(status: &ScheduleStatus) -> Schedule {
    Schedule { enabled: status.enabled, frequency: status.frequency.clone(), time_of_day: status.time_of_day.clone(),
        weekday: status.weekday, day_of_month: status.day_of_month }
}

pub fn status(conn: &Connection, now: NaiveDateTime) -> Result<ScheduleStatus, String> {
    let mut status = conn.query_row(
        "SELECT enabled,frequency,time_of_day,weekday,day_of_month,destination,keep_last,last_success_at,
                last_attempt_at,last_status,last_message,last_directory FROM backup_schedule WHERE singleton=1",
        [], |r| Ok(ScheduleStatus {
            enabled: r.get::<_, i64>(0)? == 1, frequency: r.get(1)?, time_of_day: r.get(2)?, weekday: r.get(3)?,
            day_of_month: r.get(4)?, destination: r.get(5)?, keep_last: r.get(6)?, last_success_at: r.get(7)?,
            last_attempt_at: r.get(8)?, last_status: r.get(9)?, last_message: r.get(10)?, last_directory: r.get(11)?,
            next_run_at: None,
        })).map_err(err)?;
    if status.enabled {
        status.next_run_at = Some(schedule_of(&status).next_occurrence(now).format(STAMP).to_string());
    }
    Ok(status)
}

pub fn save_core(db: &Database, input: ScheduleInput, now: NaiveDateTime) -> Result<ScheduleStatus, String> {
    validate(&input)?;
    let conn = db.conn.lock().map_err(err)?;
    let destination = input.destination.as_deref().map(str::trim).filter(|d| !d.is_empty()).map(String::from);
    conn.execute(
        "UPDATE backup_schedule SET enabled=?1,frequency=?2,time_of_day=?3,weekday=?4,day_of_month=?5,
            destination=?6,keep_last=?7,
            enabled_at=CASE WHEN ?1=1 AND (enabled=0 OR enabled_at IS NULL) THEN ?8 ELSE enabled_at END
         WHERE singleton=1",
        params![i64::from(input.enabled), input.frequency, input.time_of_day, input.weekday, input.day_of_month,
            destination, input.keep_last, now.format(STAMP).to_string()],
    ).map_err(err)?;
    status(&conn, now)
}

/// Keep only the newest `keep` automatic backups **in this destination**.
/// Entries in other directories (e.g. an old external disk) and invalid/tampered
/// entries must not count toward the quota or cause legitimate copies to be deleted.
fn apply_retention(db: &Database, destination: &Path, keep: i64) -> Result<(), String> {
    apply_retention_with(db, destination, keep, |path| backup::inspect_backup_core(path).is_ok())
}

// Inspecting an entire backup can hash gigabytes on an external drive or NAS.
// The SQLite mutex is needed to read the log and prune its rows, but must not
// remain locked during that potentially slow disk scan.
fn apply_retention_with(
    db: &Database, destination: &Path, keep: i64, mut verified: impl FnMut(&Path) -> bool,
) -> Result<(), String> {
    if keep <= 0 { return Ok(()); }
    let entries: Vec<(String, String)> = {
        let conn = db.conn.lock().map_err(err)?;
        let mut statement = conn.prepare("SELECT directory,created_at FROM auto_backup_log ORDER BY created_at DESC,directory DESC").map_err(err)?;
        let rows = statement.query_map([], |r| Ok((r.get(0)?, r.get(1)?))).map_err(err)?
            .collect::<Result<Vec<_>, _>>().map_err(err)?;
        rows
    };
    let canonical_destination = fs::canonicalize(destination).map_err(err)?;
    // Filter first, *then* apply the quota. Otherwise a newer backup made on
    // another disk (or a malformed log row) can consume a retention slot here.
    let eligible: Vec<String> = entries.into_iter().filter_map(|(directory, _)| {
        let path = Path::new(&directory);
        let name_ok = path.file_name().and_then(|n| n.to_str())
            .is_some_and(|name| name.starts_with(BACKUP_PREFIX));
        let plain_directory = fs::symlink_metadata(path)
            .map(|meta| meta.is_dir() && !meta.file_type().is_symlink()).unwrap_or(false);
        let directly_inside = fs::canonicalize(path).ok()
            .and_then(|resolved| resolved.parent()
                .map(|parent| parent == canonical_destination.as_path()))
            .unwrap_or(false);
        let regular_file = |name: &str| fs::symlink_metadata(path.join(name))
            .map(|meta| meta.is_file() && !meta.file_type().is_symlink()).unwrap_or(false);
        // A backup with the right filenames can still have corrupted media,
        // mismatched hashes, broken relations or a forged manifest. Never count
        // it toward the quota, and never destroy its potentially salvageable
        // files: otherwise a damaged recent copy could evict the last valid one.
        if name_ok && plain_directory && directly_inside
            && regular_file("manifest.json") && regular_file("repertica.sqlite")
            && verified(path) {
            Some(directory)
        } else {
            None // Keep suspicious, damaged and unavailable copies untouched.
        }
    }).collect();
    // The artist may change the destination or quota while verification runs.
    // Check again under the lock before removing any files, so stale settings
    // cannot prune backups after retention has been disabled or expanded.
    let conn = db.conn.lock().map_err(err)?;
    let (enabled, current_destination, current_keep): (bool, Option<String>, i64) = conn.query_row(
        "SELECT enabled,destination,keep_last FROM backup_schedule WHERE singleton=1", [],
        |row| Ok((row.get::<_, i64>(0)? == 1, row.get(1)?, row.get(2)?)),
    ).map_err(err)?;
    // Disabling automatic backups while an external disk is being scanned must
    // also cancel pending automatic deletions, even if the quota is unchanged.
    if !enabled || current_keep != keep || current_destination.as_deref().map(Path::new) != Some(destination) {
        return Ok(());
    }
    for directory in eligible.into_iter().skip(keep as usize) {
        if fs::remove_dir_all(&directory).is_err() { continue; }
        conn.execute("DELETE FROM auto_backup_log WHERE directory=?1", params![directory]).map_err(err)?;
    }
    Ok(())
}

/// Only results from the *current* configuration can advance its scheduling
/// state. A backup on a slow/removable disk may finish after the user changes
/// the destination or the schedule. The old backup remains valid and logged,
/// but must not suppress the next backup to the new destination.
fn same_configuration(a: &ScheduleStatus, b: &ScheduleStatus) -> bool {
    a.enabled == b.enabled && a.frequency == b.frequency && a.time_of_day == b.time_of_day
        && a.weekday == b.weekday && a.day_of_month == b.day_of_month
        && a.destination == b.destination
}

fn record_outcome(
    conn: &Connection, started: &ScheduleStatus, now: NaiveDateTime,
    outcome: &Result<backup::BackupSummary, String>,
) -> Result<(), String> {
    let stamp = now.format(STAMP).to_string();
    if let Ok(summary) = outcome {
        // The backup exists even if its original settings changed in flight:
        // retain its provenance for later inspection and quota management.
        conn.execute("INSERT OR REPLACE INTO auto_backup_log(directory,created_at) VALUES(?1,?2)",
            params![summary.directory, stamp]).map_err(err)?;
    }
    if !same_configuration(started, &status(conn, now)?) {
        return Ok(());
    }
    match outcome {
        Ok(summary) => {
            conn.execute("UPDATE backup_schedule SET last_attempt_at=?1,last_success_at=?1,last_status='OK',last_message=NULL,last_directory=?2 WHERE singleton=1",
                params![stamp, summary.directory]).map_err(err)?;
        }
        Err(message) => {
            conn.execute("UPDATE backup_schedule SET last_attempt_at=?1,last_status='ERROR',last_message=?2 WHERE singleton=1",
                params![stamp, message]).map_err(err)?;
        }
    }
    Ok(())
}

/// Run a backup now and record the outcome. Used by the timer and by "Run now".
pub fn run_core(db: &Database, now: NaiveDateTime) -> Result<ScheduleStatus, String> {
    let started = {
        let conn = db.conn.lock().map_err(err)?;
        status(&conn, now)?
    };
    let keep = started.keep_last;
    let destination = started.destination.clone().filter(|d| !d.trim().is_empty());
    let outcome: Result<backup::BackupSummary, String> = match &destination {
        None => Err("Nessuna cartella di destinazione scelta".into()),
        Some(d) if !Path::new(d).is_dir() => Err(format!(
            "Cartella di destinazione non disponibile: {}. Collega l'unità e riprova.", display_path(Path::new(d)))),
        Some(d) => backup::create_backup_core(db, Path::new(d)),
    };
    {
        let conn = db.conn.lock().map_err(err)?;
        record_outcome(&conn, &started, now, &outcome)?;
    }
    if outcome.is_ok() {
        if let Some(d) = &destination { let _ = apply_retention(db, Path::new(d), keep); }
    }
    let conn = db.conn.lock().map_err(err)?;
    status(&conn, now)
}

/// One timer tick: run the backup if the schedule says it is due.
pub fn tick(db: &Database, now: NaiveDateTime) -> Result<Option<ScheduleStatus>, String> {
    let due = {
        let conn = db.conn.lock().map_err(err)?;
        let s = status(&conn, now)?;
        let (enabled_at, last_success, last_attempt, failed): (Option<String>, Option<String>, Option<String>, bool) = conn.query_row(
            "SELECT enabled_at,last_success_at,last_attempt_at,last_status='ERROR' FROM backup_schedule WHERE singleton=1", [],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get::<_, Option<bool>>(3)?.unwrap_or(false)))).map_err(err)?;
        is_due(&schedule_of(&s), now, parse_stamp(&enabled_at), parse_stamp(&last_success),
            if failed { parse_stamp(&last_attempt) } else { None })
    };
    if due { run_core(db, now).map(Some) } else { Ok(None) }
}

/// Background timer: checks every minute while the application is open.
pub fn start(app: tauri::AppHandle) {
    std::thread::spawn(move || loop {
        std::thread::sleep(std::time::Duration::from_secs(60));
        let db = app.state::<Database>();
        if let Ok(Some(result)) = tick(&db, Local::now().naive_local()) {
            let _ = app.emit("auto-backup-finished", result);
        }
    });
}

#[tauri::command]
pub fn get_backup_schedule(db: State<'_, Database>) -> Result<ScheduleStatus, String> {
    let conn = db.conn.lock().map_err(err)?;
    status(&conn, Local::now().naive_local())
}

#[tauri::command]
pub fn save_backup_schedule(db: State<'_, Database>, input: ScheduleInput) -> Result<ScheduleStatus, String> {
    save_core(&db, input, Local::now().naive_local())
}

#[tauri::command]
pub async fn run_backup_now(app: tauri::AppHandle) -> Result<ScheduleStatus, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let db = app.state::<Database>();
        run_core(&db, Local::now().naive_local())
    }).await.map_err(err)?
}

#[cfg(test)]
mod tests {
    use super::*;

    fn at(s: &str) -> NaiveDateTime { NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M").unwrap() }
    fn daily(time: &str) -> Schedule { Schedule { enabled: true, frequency: "DAILY".into(), time_of_day: time.into(), weekday: 0, day_of_month: 1 } }
    fn weekly(day: i64, time: &str) -> Schedule { Schedule { enabled: true, frequency: "WEEKLY".into(), time_of_day: time.into(), weekday: day, day_of_month: 1 } }
    fn monthly(day: i64, time: &str) -> Schedule { Schedule { enabled: true, frequency: "MONTHLY".into(), time_of_day: time.into(), weekday: 0, day_of_month: day } }

    #[test]
    fn daily_occurrences_around_the_scheduled_time() {
        let s = daily("21:30");
        assert_eq!(s.previous_occurrence(at("2026-11-21 22:00")), at("2026-11-21 21:30"));
        assert_eq!(s.previous_occurrence(at("2026-11-21 21:30")), at("2026-11-21 21:30"));
        assert_eq!(s.previous_occurrence(at("2026-11-21 09:00")), at("2026-11-20 21:30"));
        assert_eq!(s.next_occurrence(at("2026-11-21 09:00")), at("2026-11-21 21:30"));
        assert_eq!(s.next_occurrence(at("2026-11-21 21:30")), at("2026-11-22 21:30"));
        assert_eq!(s.previous_occurrence(at("2026-01-01 00:10")), at("2025-12-31 21:30"));
    }

    #[test]
    fn weekly_occurrences_follow_the_chosen_weekday() {
        // 2026-11-21 is a Saturday (weekday 5); Monday is 0.
        let monday = weekly(0, "08:00");
        assert_eq!(monday.previous_occurrence(at("2026-11-21 12:00")), at("2026-11-16 08:00"));
        assert_eq!(monday.next_occurrence(at("2026-11-21 12:00")), at("2026-11-23 08:00"));
        let saturday = weekly(5, "08:00");
        assert_eq!(saturday.previous_occurrence(at("2026-11-21 12:00")), at("2026-11-21 08:00"));
        assert_eq!(saturday.previous_occurrence(at("2026-11-21 07:00")), at("2026-11-14 08:00"));
        assert_eq!(saturday.next_occurrence(at("2026-11-21 07:00")), at("2026-11-21 08:00"));
        let sunday = weekly(6, "23:59");
        assert_eq!(sunday.previous_occurrence(at("2026-11-21 12:00")), at("2026-11-15 23:59"));
    }

    #[test]
    fn monthly_occurrences_handle_month_and_year_boundaries() {
        let s = monthly(15, "03:00");
        assert_eq!(s.previous_occurrence(at("2026-11-21 12:00")), at("2026-11-15 03:00"));
        assert_eq!(s.previous_occurrence(at("2026-11-10 12:00")), at("2026-10-15 03:00"));
        assert_eq!(s.previous_occurrence(at("2026-01-10 12:00")), at("2025-12-15 03:00"));
        assert_eq!(s.next_occurrence(at("2026-11-21 12:00")), at("2026-12-15 03:00"));
        assert_eq!(s.next_occurrence(at("2026-12-20 12:00")), at("2027-01-15 03:00"));
        assert_eq!(monthly(28, "00:00").previous_occurrence(at("2026-03-01 00:00")), at("2026-02-28 00:00"));
    }

    #[test]
    fn a_backup_is_due_only_for_a_scheduled_moment_not_yet_covered() {
        let s = daily("09:00");
        let now = at("2026-11-21 15:00");
        // Just enabled at 14:00: today's 09:00 moment is already in the past and must not trigger a run.
        assert!(!is_due(&s, now, Some(at("2026-11-21 14:00")), None, None));
        // Enabled yesterday and never run: the missed 09:00 backup runs now.
        assert!(is_due(&s, now, Some(at("2026-11-20 14:00")), None, None));
        // Already backed up after today's scheduled moment.
        assert!(!is_due(&s, now, Some(at("2026-11-01 10:00")), Some(at("2026-11-21 09:05")), None));
        // Last success was yesterday: due again today.
        assert!(is_due(&s, now, Some(at("2026-11-01 10:00")), Some(at("2026-11-20 09:05")), None));
        // Before today's moment nothing is due yet.
        assert!(!is_due(&s, at("2026-11-21 08:59"), Some(at("2026-11-01 10:00")), Some(at("2026-11-20 09:05")), None));
        let mut off = s.clone();
        off.enabled = false;
        assert!(!is_due(&off, now, None, None, None));
    }

    #[test]
    fn a_failed_attempt_is_retried_only_after_thirty_minutes() {
        let s = daily("09:00");
        let enabled = Some(at("2026-11-01 10:00"));
        assert!(!is_due(&s, at("2026-11-21 09:20"), enabled, Some(at("2026-11-20 09:00")), Some(at("2026-11-21 09:01"))));
        assert!(is_due(&s, at("2026-11-21 09:31"), enabled, Some(at("2026-11-20 09:00")), Some(at("2026-11-21 09:01"))));
    }

    fn setup() -> (tempfile::TempDir, Database, std::path::PathBuf) {
        let dir = tempfile::tempdir().unwrap();
        let db = crate::db::open(dir.path().join("archive")).unwrap();
        db.conn.lock().unwrap().execute("INSERT INTO collectors(id,name,created_at,updated_at) VALUES('c','Persona','t','t')", []).unwrap();
        let destination = dir.path().join("backups");
        fs::create_dir(&destination).unwrap();
        (dir, db, destination)
    }
    fn input(destination: &Path, keep: i64) -> ScheduleInput {
        ScheduleInput { enabled: true, frequency: "DAILY".into(), time_of_day: "09:00".into(),
            weekday: 0, day_of_month: 1, destination: Some(destination.to_string_lossy().into_owned()), keep_last: keep }
    }

    #[test]
    fn the_schedule_is_off_by_default_and_validated_before_saving() {
        let (_dir, db, destination) = setup();
        let now = at("2026-11-21 10:00");
        let initial = status(&db.conn.lock().unwrap(), now).unwrap();
        assert!(!initial.enabled && initial.next_run_at.is_none() && initial.destination.is_none());
        let good = input(&destination, 0);
        let rejected = |change: &dyn Fn(&mut ScheduleInput)| {
            let mut candidate = good.clone();
            change(&mut candidate);
            save_core(&db, candidate, now).unwrap_err()
        };
        assert!(rejected(&|i| i.frequency = "HOURLY".into()).contains("Frequenza"));
        assert!(rejected(&|i| i.time_of_day = "9:00".into()).contains("Orario"));
        assert!(rejected(&|i| i.time_of_day = "25:00".into()).contains("Orario"));
        assert!(rejected(&|i| i.weekday = 7).contains("settimana"));
        assert!(rejected(&|i| i.day_of_month = 31).contains("1 e 28"));
        assert!(rejected(&|i| i.keep_last = -1).contains("copie"));
        assert!(rejected(&|i| i.destination = None).contains("cartella"));
        assert!(rejected(&|i| i.destination = Some("relativa/percorso".into())).contains("dialogo"));
        // A disabled schedule needs no destination.
        let mut off = good.clone();
        off.enabled = false;
        off.destination = None;
        assert!(!save_core(&db, off, now).unwrap().enabled);
        let saved = save_core(&db, good, now).unwrap();
        assert!(saved.enabled);
        assert_eq!(saved.next_run_at.as_deref(), Some("2026-11-22T09:00:00"));
    }

    #[test]
    fn a_due_run_creates_a_verified_backup_records_the_result_and_is_not_repeated() {
        let (_dir, db, destination) = setup();
        save_core(&db, input(&destination, 0), at("2026-11-19 10:00")).unwrap();
        let now = at("2026-11-21 09:05");
        let ran = tick(&db, now).unwrap().expect("a missed backup must run");
        assert_eq!(ran.last_status.as_deref(), Some("OK"));
        let directory = ran.last_directory.clone().unwrap();
        assert!(Path::new(&directory).join("manifest.json").is_file());
        assert!(Path::new(&directory).starts_with(display_path(&fs::canonicalize(&destination).unwrap())));
        assert!(tick(&db, at("2026-11-21 09:06")).unwrap().is_none(), "the same moment is never backed up twice");
        let again = tick(&db, at("2026-11-22 09:00")).unwrap();
        assert!(again.is_some(), "the next day's moment runs");
        assert_eq!(fs::read_dir(&destination).unwrap().count(), 2);
    }

    #[test]
    fn an_unavailable_destination_is_reported_and_does_not_hammer_the_disk() {
        let (dir, db, _destination) = setup();
        let gone = dir.path().join("unita-scollegata");
        save_core(&db, input(&gone, 0), at("2026-11-19 10:00")).unwrap();
        let failed = tick(&db, at("2026-11-21 09:05")).unwrap().unwrap();
        assert_eq!(failed.last_status.as_deref(), Some("ERROR"));
        assert!(failed.last_message.unwrap().contains("non disponibile"));
        assert!(tick(&db, at("2026-11-21 09:20")).unwrap().is_none(), "no retry within 30 minutes");
        fs::create_dir(&gone).unwrap();
        let recovered = tick(&db, at("2026-11-21 09:40")).unwrap().unwrap();
        assert_eq!(recovered.last_status.as_deref(), Some("OK"));
    }

    #[test]
    fn retention_removes_only_older_automatic_backups_created_by_this_feature() {
        let (_dir, db, destination) = setup();
        save_core(&db, input(&destination, 2), at("2026-11-10 08:00")).unwrap();
        // A manual backup and unrelated files live in the same folder and must survive.
        let manual = backup::create_backup_core(&db, &destination).unwrap();
        fs::write(destination.join("note.txt"), "personale").unwrap();
        let mut directories = Vec::new();
        for day in 11..=14 {
            let ran = run_core(&db, at(&format!("2026-11-{day} 09:00"))).unwrap();
            directories.push(ran.last_directory.unwrap());
        }
        assert!(!Path::new(&directories[0]).exists() && !Path::new(&directories[1]).exists(), "older automatic backups removed");
        assert!(Path::new(&directories[2]).exists() && Path::new(&directories[3]).exists(), "the newest two are kept");
        assert!(Path::new(&manual.directory).exists(), "a manual backup is never removed");
        assert!(destination.join("note.txt").exists());
        let logged: i64 = db.conn.lock().unwrap().query_row("SELECT count(*) FROM auto_backup_log", [], |r| r.get(0)).unwrap();
        assert_eq!(logged, 2);
    }

    #[test]
    fn retention_counts_backups_per_destination_even_after_switching_disks() {
        let (dir, db, first_destination) = setup();
        let second_destination = dir.path().join("second-disk");
        fs::create_dir(&second_destination).unwrap();
        save_core(&db, input(&first_destination, 2), at("2026-11-10 08:00")).unwrap();
        let a1 = run_core(&db, at("2026-11-11 09:00")).unwrap().last_directory.unwrap();
        let a2 = run_core(&db, at("2026-11-12 09:00")).unwrap().last_directory.unwrap();

        save_core(&db, input(&second_destination, 2), at("2026-11-12 11:00")).unwrap();
        let b1 = run_core(&db, at("2026-11-13 09:00")).unwrap().last_directory.unwrap();
        let b2 = run_core(&db, at("2026-11-14 09:00")).unwrap().last_directory.unwrap();

        save_core(&db, input(&first_destination, 2), at("2026-11-14 11:00")).unwrap();
        let a3 = run_core(&db, at("2026-11-15 09:00")).unwrap().last_directory.unwrap();
        assert!(!Path::new(&a1).exists(), "only the oldest backup on disk A is pruned");
        assert!(Path::new(&a2).is_dir() && Path::new(&a3).is_dir(),
            "two newest copies on disk A survive despite later backups on disk B");
        assert!(Path::new(&b1).is_dir() && Path::new(&b2).is_dir(),
            "copies on the other disk are untouched");
        let count: i64 = db.conn.lock().unwrap().query_row(
            "SELECT count(*) FROM auto_backup_log", [], |r| r.get(0)).unwrap();
        assert_eq!(count, 4);
    }

    #[test]
    fn invalid_newer_log_entry_does_not_consume_a_retention_slot() {
        let (_dir, db, destination) = setup();
        save_core(&db, input(&destination, 2), at("2026-11-10 08:00")).unwrap();
        let oldest = run_core(&db, at("2026-11-11 09:00")).unwrap().last_directory.unwrap();
        let keep = run_core(&db, at("2026-11-12 09:00")).unwrap().last_directory.unwrap();

        let fake = destination.join("ReperticaStudio-backup-not-a-backup");
        fs::create_dir(&fake).unwrap();
        fs::write(fake.join("note.txt"), "non e un backup").unwrap();
        db.conn.lock().unwrap().execute(
            "INSERT INTO auto_backup_log(directory,created_at) VALUES(?1,'2099-01-01T00:00:00')",
            params![fake.to_string_lossy()],
        ).unwrap();

        let newest = run_core(&db, at("2026-11-13 09:00")).unwrap().last_directory.unwrap();
        assert!(!Path::new(&oldest).exists(), "oldest real copy can be pruned");
        assert!(Path::new(&keep).is_dir() && Path::new(&newest).is_dir(),
            "a bogus log row must not cost a real copy its retention slot");
        assert!(fake.join("note.txt").exists(), "unrecognized folder is never removed");
    }

    #[test]
    fn damaged_recent_backup_cannot_evict_the_last_verified_copies() {
        let (_dir, db, destination) = setup();
        save_core(&db, input(&destination, 2), at("2026-11-10 08:00")).unwrap();
        let first = run_core(&db, at("2026-11-11 09:00")).unwrap().last_directory.unwrap();
        let damaged = run_core(&db, at("2026-11-12 09:00")).unwrap().last_directory.unwrap();
        // The directory still looks like a backup (manifest + SQLite exist),
        // but its manifest no longer describes the actual files.
        fs::write(Path::new(&damaged).join("manifest.json"), "{}").unwrap();
        assert!(backup::inspect_backup_core(Path::new(&damaged)).is_err());

        let newest = run_core(&db, at("2026-11-13 09:00")).unwrap().last_directory.unwrap();
        assert!(Path::new(&first).is_dir(), "the older valid copy must survive");
        assert!(Path::new(&newest).is_dir(), "the newly verified copy must survive");
        assert!(Path::new(&damaged).is_dir(), "damaged backups may contain salvageable files");
        assert!(backup::inspect_backup_core(Path::new(&first)).is_ok());
        assert!(backup::inspect_backup_core(Path::new(&newest)).is_ok());
        let logged: i64 = db.conn.lock().unwrap()
            .query_row("SELECT count(*) FROM auto_backup_log", [], |r| r.get(0)).unwrap();
        assert_eq!(logged, 3, "damaged log records must be retained for manual recovery");
    }

    #[test]
    fn damaged_old_backup_is_preserved_when_valid_older_copies_are_pruned() {
        let (_dir, db, destination) = setup();
        save_core(&db, input(&destination, 2), at("2026-11-10 08:00")).unwrap();
        let damaged = run_core(&db, at("2026-11-11 09:00")).unwrap().last_directory.unwrap();
        fs::write(Path::new(&damaged).join("manifest.json"), "{}").unwrap();
        let oldest_valid = run_core(&db, at("2026-11-12 09:00")).unwrap().last_directory.unwrap();
        let next = run_core(&db, at("2026-11-13 09:00")).unwrap().last_directory.unwrap();
        let newest = run_core(&db, at("2026-11-14 09:00")).unwrap().last_directory.unwrap();

        assert!(Path::new(&damaged).is_dir(), "corrupted data must not be deleted automatically");
        assert!(!Path::new(&oldest_valid).exists(), "oldest verified copy may be pruned");
        assert!(Path::new(&next).is_dir() && Path::new(&newest).is_dir(),
            "the two newest verified copies must be preserved");
    }

    #[test]
    fn retention_rechecks_settings_after_verifying_backups_without_locking_sqlite() {
        let (_dir, db, destination) = setup();
        save_core(&db, input(&destination, 0), at("2026-11-10 08:00")).unwrap();
        let first = run_core(&db, at("2026-11-11 09:00")).unwrap().last_directory.unwrap();
        let second = run_core(&db, at("2026-11-12 09:00")).unwrap().last_directory.unwrap();
        save_core(&db, input(&destination, 1), at("2026-11-12 11:00")).unwrap();

        let mut changed = false;
        apply_retention_with(&db, &destination, 1, |path| {
            // try_lock avoids a hanging test if a future refactor holds the
            // archive mutex while hashing the external backup contents.
            let conn = db.conn.try_lock().expect("SQLite should be unlocked during backup inspection");
            if !changed {
                conn.execute("UPDATE backup_schedule SET keep_last=0 WHERE singleton=1", []).unwrap();
                changed = true;
            }
            drop(conn);
            backup::inspect_backup_core(path).is_ok()
        }).unwrap();

        assert!(changed, "the verified backup scanner must run");
        assert!(Path::new(&first).is_dir() && Path::new(&second).is_dir(),
            "disabling retention during an external-disk scan must preserve all backups");
    }


    #[test]
    fn a_completed_backup_does_not_mark_a_new_destination_as_backed_up() {
        let (dir, db, first) = setup();
        let second = dir.path().join("new-external-disk");
        fs::create_dir(&second).unwrap();
        save_core(&db, input(&first, 0), at("2026-11-19 10:00")).unwrap();
        let started = status(&db.conn.lock().unwrap(), at("2026-11-21 09:05")).unwrap();
        // Simulate a slow backup finishing *after* the artist switches disks.
        let completed = backup::create_backup_core(&db, &first).unwrap();
        save_core(&db, input(&second, 0), at("2026-11-21 09:05")).unwrap();
        {
            let conn = db.conn.lock().unwrap();
            record_outcome(&conn, &started, at("2026-11-21 09:05"), &Ok(completed.clone())).unwrap();
            let logged: i64 = conn.query_row(
                "SELECT count(*) FROM auto_backup_log WHERE directory=?1",
                params![completed.directory], |r| r.get(0)).unwrap();
            assert_eq!(logged, 1, "the valid backup on the first disk is still tracked");
        }
        let unchanged = status(&db.conn.lock().unwrap(), at("2026-11-21 09:06")).unwrap();
        assert!(unchanged.last_success_at.is_none(), "an old disk must not satisfy the new schedule");
        assert!(unchanged.last_directory.is_none());
        let new_run = tick(&db, at("2026-11-21 09:06")).unwrap().expect("new destination is still due");
        // Windows can return a verbatim (\\\\?\\) path for the temporary directory,
        // while the backup API deliberately returns a user-facing path without it.
        // Compare resolved filesystem locations rather than their display spelling.
        let created = fs::canonicalize(new_run.last_directory.unwrap()).unwrap();
        let selected = fs::canonicalize(&second).unwrap();
        assert!(created.starts_with(&selected),
            "the new backup must be inside the newly selected destination");
    }

    #[test]
    fn an_obsolete_backup_failure_does_not_replace_new_schedule_status() {
        let (dir, db, first) = setup();
        let second = dir.path().join("second-disk");
        fs::create_dir(&second).unwrap();
        save_core(&db, input(&first, 0), at("2026-11-19 10:00")).unwrap();
        let started = status(&db.conn.lock().unwrap(), at("2026-11-21 09:05")).unwrap();
        save_core(&db, input(&second, 0), at("2026-11-21 09:05")).unwrap();
        {
            let conn = db.conn.lock().unwrap();
            record_outcome(&conn, &started, at("2026-11-21 09:05"),
                &Err("Unità precedente scollegata".into())).unwrap();
        }
        let state = status(&db.conn.lock().unwrap(), at("2026-11-21 09:06")).unwrap();
        assert!(state.last_attempt_at.is_none() && state.last_status.is_none());
        assert!(tick(&db, at("2026-11-21 09:06")).unwrap().is_some(),
            "an old failure must not impose a 30-minute retry delay on the new disk");
    }

    #[test]
    fn turning_off_auto_backup_during_verification_cancels_pruning() {
        let (_dir, db, destination) = setup();
        save_core(&db, input(&destination, 0), at("2026-11-10 08:00")).unwrap();
        let oldest = run_core(&db, at("2026-11-11 09:00")).unwrap().last_directory.unwrap();
        let newest = run_core(&db, at("2026-11-12 09:00")).unwrap().last_directory.unwrap();
        save_core(&db, input(&destination, 1), at("2026-11-12 11:00")).unwrap();
        let mut disabled = false;
        apply_retention_with(&db, &destination, 1, |path| {
            let conn = db.conn.try_lock().expect("SQLite must remain unlocked during SHA verification");
            if !disabled {
                conn.execute("UPDATE backup_schedule SET enabled=0 WHERE singleton=1", []).unwrap();
                disabled = true;
            }
            drop(conn);
            backup::inspect_backup_core(path).is_ok()
        }).unwrap();
        assert!(disabled);
        assert!(Path::new(&oldest).is_dir() && Path::new(&newest).is_dir(),
            "disabling automatic backups must not remove old copies");
        let count: i64 = db.conn.lock().unwrap()
            .query_row("SELECT count(*) FROM auto_backup_log", [], |r| r.get(0)).unwrap();
        assert_eq!(count, 2);
    }

    #[test]
    fn a_tampered_log_entry_can_never_delete_arbitrary_folders() {
        let (dir, db, destination) = setup();
        save_core(&db, input(&destination, 1), at("2026-11-10 08:00")).unwrap();
        let precious = dir.path().join("documenti-importanti");
        fs::create_dir(&precious).unwrap();
        fs::write(precious.join("tesi.docx"), "x").unwrap();
        let lookalike = destination.join("ReperticaStudio-backup-finto");
        fs::create_dir(&lookalike).unwrap();
        fs::write(lookalike.join("dati.txt"), "x").unwrap();
        {
            let conn = db.conn.lock().unwrap();
            for (path, stamp) in [(&precious, "2020-01-01T00:00:00"), (&lookalike, "2020-01-02T00:00:00")] {
                conn.execute("INSERT INTO auto_backup_log(directory,created_at) VALUES(?1,?2)", params![path.to_string_lossy(), stamp]).unwrap();
            }
        }
        run_core(&db, at("2026-11-11 09:00")).unwrap();
        run_core(&db, at("2026-11-12 09:00")).unwrap();
        assert!(precious.join("tesi.docx").is_file(), "folders outside the destination are untouched");
        assert!(lookalike.join("dati.txt").is_file(), "a folder without a manifest is untouched");
    }

    #[test]
    fn run_now_reports_a_missing_destination_clearly() {
        let (_dir, db, _destination) = setup();
        let status = run_core(&db, at("2026-11-21 10:00")).unwrap();
        assert_eq!(status.last_status.as_deref(), Some("ERROR"));
        assert!(status.last_message.unwrap().contains("Nessuna cartella"));
    }
}
