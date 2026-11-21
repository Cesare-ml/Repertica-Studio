//! Automatic archive codes (e.g. `ART-0001` or `ML-2026-003`).
//!
//! The artist chooses a prefix, whether numbering restarts every year, the number of
//! digits and the separator. Numbers are never reused: a counter per prefix and scope
//! only moves forward, and codes already present in the archive are skipped.
use crate::db::Database;
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use tauri::State;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CodePolicy {
    pub prefix: String,
    /// `GLOBAL` (ART-0001) or `YEARLY` (ART-2026-001).
    pub scope: String,
    pub padding: i64,
    pub separator: String,
}

impl Default for CodePolicy {
    fn default() -> Self {
        CodePolicy { prefix: "ART".into(), scope: "GLOBAL".into(), padding: 4, separator: "-".into() }
    }
}

fn err(e: impl std::fmt::Display) -> String { crate::util::friendly(e) }

pub fn validate(policy: &CodePolicy) -> Result<CodePolicy, String> {
    let prefix = policy.prefix.trim().to_uppercase();
    if prefix.is_empty() || prefix.chars().count() > 12 || !prefix.chars().all(|c| c.is_ascii_alphanumeric()) {
        return Err("Il prefisso deve avere da 1 a 12 tra lettere e cifre (senza spazi)".into());
    }
    if !["GLOBAL", "YEARLY"].contains(&policy.scope.as_str()) {
        return Err("Numerazione non valida: scegli progressiva o annuale".into());
    }
    if !["-", ".", "_"].contains(&policy.separator.as_str()) {
        return Err("Separatore non valido (usa - . oppure _)".into());
    }
    if !(2..=8).contains(&policy.padding) {
        return Err("Le cifre del numero devono essere da 2 a 8".into());
    }
    Ok(CodePolicy { prefix, scope: policy.scope.clone(), padding: policy.padding, separator: policy.separator.clone() })
}

pub fn load(conn: &Connection) -> Result<CodePolicy, String> {
    conn.query_row(
        "SELECT prefix,scope,padding,separator FROM archive_code_policy WHERE singleton=1",
        [],
        |r| Ok(CodePolicy { prefix: r.get(0)?, scope: r.get(1)?, padding: r.get(2)?, separator: r.get(3)? }),
    ).map_err(err)
}

/// The year used by yearly numbering: the artwork's own year when it is a plain year
/// (also inside text such as "2023-2024"), otherwise the current local year.
pub fn year_for(artwork_year: Option<&str>) -> i32 {
    use chrono::Datelike;
    let digits: Vec<char> = artwork_year.unwrap_or("").chars().collect();
    let mut index = 0;
    while index + 4 <= digits.len() {
        let window = &digits[index..index + 4];
        let before_ok = index == 0 || !digits[index - 1].is_ascii_digit();
        let after_ok = index + 4 == digits.len() || !digits[index + 4].is_ascii_digit();
        if window.iter().all(char::is_ascii_digit) && before_ok && after_ok {
            let year: i32 = window.iter().collect::<String>().parse().unwrap_or(0);
            if (1000..=9999).contains(&year) { return year; }
        }
        index += 1;
    }
    chrono::Local::now().year()
}

fn format_code(policy: &CodePolicy, year: i32, number: i64) -> String {
    let digits = format!("{number:0width$}", width = policy.padding as usize);
    if policy.scope == "YEARLY" {
        format!("{p}{s}{year}{s}{digits}", p = policy.prefix, s = policy.separator)
    } else {
        format!("{p}{s}{digits}", p = policy.prefix, s = policy.separator)
    }
}

fn scope_key(policy: &CodePolicy, year: i32) -> String {
    if policy.scope == "YEARLY" { format!("{}|{year}", policy.prefix) } else { format!("{}|GLOBAL", policy.prefix) }
}

/// The highest number already used by an existing code of this prefix and scope.
fn highest_existing(conn: &Connection, policy: &CodePolicy, year: i32) -> Result<i64, String> {
    let head = if policy.scope == "YEARLY" {
        format!("{}{}{year}{}", policy.prefix, policy.separator, policy.separator)
    } else {
        format!("{}{}", policy.prefix, policy.separator)
    }
    .to_lowercase();
    let mut statement = conn.prepare("SELECT code FROM artworks").map_err(err)?;
    let codes = statement.query_map([], |r| r.get::<_, String>(0)).map_err(err)?;
    let mut highest = 0;
    for code in codes {
        let code = code.map_err(err)?.to_lowercase();
        if let Some(rest) = code.strip_prefix(&head) {
            if !rest.is_empty() && rest.chars().all(|c| c.is_ascii_digit()) {
                if let Ok(n) = rest.parse::<i64>() { highest = highest.max(n); }
            }
        }
    }
    Ok(highest)
}

fn next_number(conn: &Connection, policy: &CodePolicy, year: i32) -> Result<i64, String> {
    let counter: i64 = conn.query_row(
        "SELECT COALESCE((SELECT last_value FROM archive_code_sequences WHERE scope_key=?1),0)",
        params![scope_key(policy, year)], |r| r.get(0),
    ).map_err(err)?;
    let mut number = counter.max(highest_existing(conn, policy, year)?) + 1;
    // Manual codes may use different padding: never hand out a code that already exists.
    loop {
        let taken: bool = conn.query_row(
            "SELECT EXISTS(SELECT 1 FROM artworks WHERE code=?1 COLLATE NOCASE)",
            params![format_code(policy, year, number)], |r| r.get(0),
        ).map_err(err)?;
        if !taken { return Ok(number); }
        number += 1;
    }
}

/// The code the next artwork would receive. Reserves nothing.
pub fn preview(conn: &Connection, artwork_year: Option<&str>) -> Result<String, String> {
    let policy = load(conn)?;
    let year = year_for(artwork_year);
    Ok(format_code(&policy, year, next_number(conn, &policy, year)?))
}

/// Reserve the next code. Call inside the transaction that inserts the artwork so that
/// a failed save does not consume a number.
pub fn allocate(conn: &Connection, artwork_year: Option<&str>) -> Result<String, String> {
    let policy = load(conn)?;
    let year = year_for(artwork_year);
    let number = next_number(conn, &policy, year)?;
    conn.execute(
        "INSERT INTO archive_code_sequences(scope_key,last_value) VALUES(?1,?2)
         ON CONFLICT(scope_key) DO UPDATE SET last_value=excluded.last_value",
        params![scope_key(&policy, year), number],
    ).map_err(err)?;
    Ok(format_code(&policy, year, number))
}

#[tauri::command]
pub fn get_archive_code_policy(db: State<'_, Database>) -> Result<CodePolicy, String> {
    let conn = db.conn.lock().map_err(err)?;
    load(&conn)
}

#[tauri::command]
pub fn save_archive_code_policy(db: State<'_, Database>, input: CodePolicy) -> Result<CodePolicy, String> {
    let policy = validate(&input)?;
    let conn = db.conn.lock().map_err(err)?;
    conn.execute(
        "UPDATE archive_code_policy SET prefix=?1,scope=?2,padding=?3,separator=?4 WHERE singleton=1",
        params![policy.prefix, policy.scope, policy.padding, policy.separator],
    ).map_err(err)?;
    load(&conn)
}

#[tauri::command]
pub fn next_archive_code(db: State<'_, Database>, year: Option<String>) -> Result<String, String> {
    let conn = db.conn.lock().map_err(err)?;
    preview(&conn, year.as_deref())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn conn() -> (tempfile::TempDir, Database) {
        let dir = tempfile::tempdir().unwrap();
        let db = crate::db::open(dir.path().to_path_buf()).unwrap();
        (dir, db)
    }
    fn add(conn: &Connection, code: &str) {
        conn.execute("INSERT INTO artworks(id,code,title,created_at,updated_at) VALUES(?1,?1,'T','t','t')", params![code]).unwrap();
    }
    fn policy(prefix: &str, scope: &str, padding: i64, separator: &str) -> CodePolicy {
        CodePolicy { prefix: prefix.into(), scope: scope.into(), padding, separator: separator.into() }
    }
    fn set(conn: &Connection, p: &CodePolicy) {
        conn.execute("UPDATE archive_code_policy SET prefix=?1,scope=?2,padding=?3,separator=?4",
            params![p.prefix, p.scope, p.padding, p.separator]).unwrap();
    }

    #[test]
    fn default_policy_numbers_artworks_progressively() {
        let (_dir, db) = conn();
        let c = db.conn.lock().unwrap();
        assert_eq!(load(&c).unwrap(), CodePolicy::default());
        assert_eq!(preview(&c, None).unwrap(), "ART-0001");
        assert_eq!(allocate(&c, None).unwrap(), "ART-0001");
        add(&c, "ART-0001");
        assert_eq!(allocate(&c, None).unwrap(), "ART-0002");
    }

    #[test]
    fn preview_reserves_nothing_and_numbers_are_never_reused_after_deletion() {
        let (_dir, db) = conn();
        let c = db.conn.lock().unwrap();
        assert_eq!(preview(&c, None).unwrap(), "ART-0001");
        assert_eq!(preview(&c, None).unwrap(), "ART-0001");
        let first = allocate(&c, None).unwrap();
        add(&c, &first);
        let second = allocate(&c, None).unwrap();
        add(&c, &second);
        c.execute("DELETE FROM artworks WHERE code=?1", params![second]).unwrap();
        assert_eq!(allocate(&c, None).unwrap(), "ART-0003", "the deleted code must not come back");
    }

    #[test]
    fn yearly_policy_restarts_each_year_and_uses_the_artwork_year() {
        let (_dir, db) = conn();
        let c = db.conn.lock().unwrap();
        set(&c, &policy("ML", "YEARLY", 3, "-"));
        let a = allocate(&c, Some("2024")).unwrap();
        add(&c, &a);
        let b = allocate(&c, Some("ca. 2024")).unwrap();
        add(&c, &b);
        let other = allocate(&c, Some("2023-2024")).unwrap();
        let c2025 = allocate(&c, Some("2025")).unwrap();
        assert_eq!((a.as_str(), b.as_str(), c2025.as_str()), ("ML-2024-001", "ML-2024-002", "ML-2025-001"));
        assert_eq!(other, "ML-2023-001", "a range uses its first year");
        let fallback = allocate(&c, Some("senza data")).unwrap();
        assert!(fallback.starts_with(&format!("ML-{}-", chrono::Local::now().format("%Y"))), "{fallback}");
    }

    #[test]
    fn existing_manual_codes_move_the_sequence_forward_and_are_never_duplicated() {
        let (_dir, db) = conn();
        let c = db.conn.lock().unwrap();
        add(&c, "ART-0041");
        add(&c, "art-0042");
        add(&c, "ART-7");
        assert_eq!(allocate(&c, None).unwrap(), "ART-0043");
        set(&c, &policy("ART", "GLOBAL", 2, "."));
        add(&c, "ART.44");
        // Different separator/padding: still never collides with an existing code.
        let next = allocate(&c, None).unwrap();
        assert_eq!(next, "ART.45");
        add(&c, &next);
    }

    #[test]
    fn changing_prefix_starts_its_own_numbering_and_validation_rejects_bad_policies() {
        let (_dir, db) = conn();
        let c = db.conn.lock().unwrap();
        let first = allocate(&c, None).unwrap();
        add(&c, &first);
        set(&c, &policy("MIRKO", "GLOBAL", 4, "-"));
        assert_eq!(allocate(&c, None).unwrap(), "MIRKO-0001");
        assert!(validate(&policy("", "GLOBAL", 4, "-")).is_err());
        assert!(validate(&policy("TROPPO-LUNGO", "GLOBAL", 4, "-")).is_err());
        assert!(validate(&policy("AB CD", "GLOBAL", 4, "-")).is_err());
        assert!(validate(&policy("ML", "MONTHLY", 4, "-")).is_err());
        assert!(validate(&policy("ML", "GLOBAL", 1, "-")).is_err());
        assert!(validate(&policy("ML", "GLOBAL", 9, "-")).is_err());
        assert!(validate(&policy("ML", "GLOBAL", 4, "/")).is_err());
        assert_eq!(validate(&policy(" ml ", "YEARLY", 3, "_")).unwrap().prefix, "ML");
        assert!(c.execute("UPDATE archive_code_policy SET scope='WEEKLY'", []).is_err(), "the database enforces it too");
    }

    #[test]
    fn year_extraction_handles_text_and_ignores_longer_numbers() {
        assert_eq!(year_for(Some("2019")), 2019);
        assert_eq!(year_for(Some(" 1998 ")), 1998);
        assert_eq!(year_for(Some("1999-2001")), 1999);
        let current: i32 = chrono::Local::now().format("%Y").to_string().parse().unwrap();
        assert_eq!(year_for(Some("12345")), current);
        assert_eq!(year_for(Some("")), current);
        assert_eq!(year_for(None), current);
    }
}
