//! Configurable split of sale proceeds ("ripartizione"): the artist decides into how many
//! parts the net proceeds are divided, how each part is called and its percentage, and
//! can switch the whole feature off. Three parts are created by default.
//!
//! The shares in force when a sale is recorded are frozen on that sale (`sale_allocations`):
//! later changes to the plan never rewrite history.
use crate::db::Database;
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use tauri::State;
use uuid::Uuid;

const MAX_PARTS: usize = 12;

fn err(e: impl std::fmt::Display) -> String { crate::util::friendly(e) }

// SQLite may silently skip writes when an externally imported database defines
// a BEFORE INSERT/UPDATE/DELETE trigger with RAISE(IGNORE). Treat that as an
// error so the transaction rolls back instead of reporting a partial plan save.
fn require_plan_write(rows: usize, operation: &str) -> Result<(), String> {
    if rows == 1 { Ok(()) } else {
        Err(format!("SQLite did not apply the allocation plan change ({operation}); reload the settings and retry"))
    }
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Part {
    pub id: String,
    pub name: String,
    pub share_bp: i64,
    /// Expenses can be charged to this part (like the activity and savings funds).
    pub pays_expenses: bool,
    pub position: i64,
    pub active: bool,
}

#[derive(Clone, Debug, Deserialize)]
pub struct PartInput {
    #[serde(default)] pub id: Option<String>,
    pub name: String,
    pub share_bp: i64,
    pub pays_expenses: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Plan {
    pub enabled: bool,
    /// Active parts in display order.
    pub parts: Vec<Part>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct PlanInput {
    pub enabled: bool,
    pub parts: Vec<PartInput>,
}

// Third-party SQLite importers can bypass CHECK constraints. Values other than
// 0/1 must never be silently treated as "disabled" or "inactive": that could
// stop the allocation of new sales or hide historical funds.
fn persisted_flag(value: i64, field: &str) -> Result<bool, String> {
    match value {
        0 => Ok(false),
        1 => Ok(true),
        _ => Err(format!("Invalid {field} flag in imported allocation data; repair the archive before continuing")),
    }
}

// Imported databases can also bypass the allocation_parts CHECK constraints.
// Reject invalid history before displaying funds or freezing a distribution on
// a new sale; do not silently expose an invented percentage or an empty label.
// Historical inactive parts may share positions, so only require a positive
// position rather than globally unique/contiguous numbering.
fn validate_persisted_part(name: &str, share_bp: i64, position: i64) -> Result<(), String> {
    let length = name.trim().chars().count();
    if !(1..=60).contains(&length) {
        return Err("Invalid allocation part name in imported SQLite data; repair the archive".into());
    }
    if !(0..=10_000).contains(&share_bp) {
        return Err("Invalid allocation share_bp in imported SQLite data; repair the archive".into());
    }
    if position < 1 {
        return Err("Invalid allocation position in imported SQLite data; repair the archive".into());
    }
    Ok(())
}

pub fn enabled(conn: &Connection) -> Result<bool, String> {
    let value = conn.query_row(
        "SELECT enabled FROM allocation_settings WHERE singleton=1",
        [], |r| r.get::<_, i64>(0),
    ).map_err(err)?;
    persisted_flag(value, "allocation enabled")
}

/// Every part ever defined, active or not, in display order.
pub fn all_parts(conn: &Connection) -> Result<Vec<Part>, String> {
    let mut statement = conn.prepare(
        "SELECT id,name,share_bp,pays_expenses,position,active FROM allocation_parts ORDER BY position,id").map_err(err)?;
    let rows = statement.query_map([], |r| Ok((
        r.get::<_, String>(0)?, r.get::<_, String>(1)?, r.get::<_, i64>(2)?,
        r.get::<_, i64>(3)?, r.get::<_, i64>(4)?, r.get::<_, i64>(5)?,
    ))).map_err(err)?;
    let mut parts = Vec::new();
    for row in rows {
        let (id, name, share_bp, pays_expenses, position, active) = row.map_err(err)?;
        validate_persisted_part(&name, share_bp, position)?;
        parts.push(Part {
            id, name, share_bp,
            pays_expenses: persisted_flag(pays_expenses, "pays_expenses")?,
            position, active: persisted_flag(active, "active")?,
        });
    }
    Ok(parts)
}

pub fn load_plan(conn: &Connection) -> Result<Plan, String> {
    Ok(Plan { enabled: enabled(conn)?, parts: all_parts(conn)?.into_iter().filter(|p| p.active).collect() })
}

/// Shares to freeze on a sale recorded now: empty when the split is switched off.
pub fn shares_for_new_sale(conn: &Connection) -> Result<Vec<(String, i64)>, String> {
    if !enabled(conn)? { return Ok(vec![]); }
    let shares: Vec<_> = load_plan(conn)?.parts.into_iter().map(|p| (p.id, p.share_bp)).collect();
    // Imported or externally edited SQLite databases can contain an incomplete active
    // plan even when each individual basis-point value passes its CHECK constraint.
    // Refuse to freeze an invalid distribution on a new sale.
    if shares.is_empty() || shares.len() > MAX_PARTS ||
        shares.iter().any(|(_, bp)| !(0..=10_000).contains(bp)) ||
        shares.iter().map(|(_, bp)| *bp).sum::<i64>() != 10_000 {
        return Err("Il piano di ripartizione non è valido: le quote attive devono sommare al 100%".into());
    }
    Ok(shares)
}

/// Divide `net` cents by basis points; the last part takes the remainder so the cents always add up.
pub fn split(net: i64, shares_bp: &[i64]) -> Vec<i64> {
    let mut amounts = Vec::with_capacity(shares_bp.len());
    let mut assigned = 0;
    for (index, bp) in shares_bp.iter().enumerate() {
        let amount = if index + 1 == shares_bp.len() { net - assigned } else { (i128::from(net) * i128::from(*bp) / 10_000) as i64 };
        assigned += amount;
        amounts.push(amount);
    }
    amounts
}

pub fn validate(input: &PlanInput) -> Result<(), String> {
    // A disabled plan may be an unfinished draft, but must never contain the
    // same persisted part twice: UPSERT would silently collapse both rows.
    if input.parts.len() > MAX_PARTS {
        return Err(format!("La ripartizione non può superare {MAX_PARTS} parti"));
    }
    let mut ids = std::collections::HashSet::new();
    for part in &input.parts {
        if let Some(id) = part.id.as_deref() {
            if !ids.insert(id) {
                return Err("La stessa parte della ripartizione non può essere usata due volte".into());
            }
        }
    }
    if !input.enabled { return Ok(()); }
    if input.parts.is_empty() {
        return Err(format!("La ripartizione deve avere da 1 a {MAX_PARTS} parti"));
    }
    let mut names: Vec<String> = Vec::new();
    let mut total = 0_i64;
    for part in &input.parts {
        let name = part.name.trim();
        if name.is_empty() || name.chars().count() > 60 { return Err("Ogni parte deve avere un nome (massimo 60 caratteri)".into()); }
        if names.contains(&name.to_lowercase()) { return Err(format!("Nome duplicato: {name}")); }
        names.push(name.to_lowercase());
        if !(0..=10_000).contains(&part.share_bp) { return Err("Le percentuali devono essere tra 0% e 100%".into()); }
        total += part.share_bp;
    }
    if total != 10_000 { return Err("Le percentuali delle parti devono sommare esattamente al 100%".into()); }
    Ok(())
}

pub fn save_plan_core(db: &Database, input: PlanInput) -> Result<Plan, String> {
    validate(&input)?;
    let mut conn = db.conn.lock().map_err(err)?;
    let tx = conn.transaction().map_err(err)?;
    let existing = all_parts(&tx)?;
    let mut kept: Vec<String> = Vec::new();
    // Keep the exact expected state: an AFTER trigger can alter a row while
    // SQLite still reports that our INSERT/UPDATE/DELETE affected one row.
    let mut expected_parts: Vec<Part> = Vec::new();
    for (position, part) in input.parts.iter().enumerate() {
        let name = part.name.trim().to_string();
        let id = match &part.id {
            Some(id) if existing.iter().any(|old| old.id == *id) => id.clone(),
            Some(_) => return Err("Parte non trovata: ricarica le impostazioni".into()),
            None => Uuid::new_v4().to_string(),
        };
        // Parts of a disabled plan are stored too, so the artist's configuration is never lost.
        let changed = tx.execute(
            "INSERT INTO allocation_parts(id,name,share_bp,pays_expenses,position,active) VALUES(?1,?2,?3,?4,?5,1)
             ON CONFLICT(id) DO UPDATE SET name=excluded.name,share_bp=excluded.share_bp,
                pays_expenses=excluded.pays_expenses,position=excluded.position,active=1",
            params![id, name, part.share_bp, i64::from(part.pays_expenses), position as i64 + 1],
        ).map_err(err)?;
        require_plan_write(changed, "save part")?;
        expected_parts.push(Part {
            id: id.clone(), name, share_bp: part.share_bp,
            pays_expenses: part.pays_expenses, position: position as i64 + 1, active: true,
        });
        kept.push(id);
    }
    for old in existing.iter().filter(|old| !kept.contains(&old.id)) {
        let used: bool = tx.query_row(
            "SELECT EXISTS(SELECT 1 FROM sale_allocations WHERE part_id=?1) OR EXISTS(SELECT 1 FROM expenses WHERE fund=?1)",
            params![&old.id], |r| r.get(0)).map_err(err)?;
        if used {
            // History keeps pointing at the part; it just stops being offered.
            let changed = tx.execute("UPDATE allocation_parts SET active=0 WHERE id=?1", params![&old.id]).map_err(err)?;
            require_plan_write(changed, "deactivate historical part")?;
            expected_parts.push(Part { active: false, ..old.clone() });
        } else {
            let changed = tx.execute("DELETE FROM allocation_parts WHERE id=?1", params![&old.id]).map_err(err)?;
            require_plan_write(changed, "remove unused part")?;
        }
    }
    let changed = tx.execute("UPDATE allocation_settings SET enabled=?1 WHERE singleton=1", params![i64::from(input.enabled)]).map_err(err)?;
    require_plan_write(changed, "save enabled state")?;
    expected_parts.sort_by(|a, b| a.position.cmp(&b.position).then_with(|| a.id.cmp(&b.id)));
    if enabled(&tx)? != input.enabled || all_parts(&tx)? != expected_parts {
        return Err("SQLite changed the allocation plan unexpectedly; no changes were saved".into());
    }
    let plan = load_plan(&tx)?;
    tx.commit().map_err(err)?;
    Ok(plan)
}

#[tauri::command]
pub fn get_allocation_plan(db: State<'_, Database>) -> Result<Plan, String> {
    let conn = db.conn.lock().map_err(err)?;
    load_plan(&conn)
}

#[tauri::command]
pub fn save_allocation_plan(db: State<'_, Database>, input: PlanInput) -> Result<Plan, String> {
    save_plan_core(&db, input)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup() -> (tempfile::TempDir, Database) {
        let dir = tempfile::tempdir().unwrap();
        let db = crate::db::open(dir.path().to_path_buf()).unwrap();
        (dir, db)
    }
    fn part(name: &str, bp: i64, pays: bool) -> PartInput { PartInput { id: None, name: name.into(), share_bp: bp, pays_expenses: pays } }

    #[test]
    fn invalid_imported_boolean_flags_do_not_hide_allocations() {
        let (_dir, db) = setup();
        let conn = db.conn.lock().unwrap();
        let before = load_plan(&conn).unwrap();
        // An external importer can disable CHECK constraints while modifying
        // a perfectly valid SQLite file; opening the file must not mask this.
        conn.execute_batch("PRAGMA ignore_check_constraints=ON;").unwrap();
        conn.execute("UPDATE allocation_settings SET enabled=2 WHERE singleton=1", []).unwrap();
        assert!(enabled(&conn).unwrap_err().contains("allocation enabled"));
        assert!(shares_for_new_sale(&conn).is_err(),
            "corruption cannot silently turn a new sale's allocations off");
        conn.execute("UPDATE allocation_settings SET enabled=1 WHERE singleton=1", []).unwrap();

        conn.execute("UPDATE allocation_parts SET pays_expenses=2 WHERE id='activity'", []).unwrap();
        assert!(all_parts(&conn).unwrap_err().contains("pays_expenses"));
        assert!(load_plan(&conn).is_err());
        conn.execute("UPDATE allocation_parts SET pays_expenses=1 WHERE id='activity'", []).unwrap();

        conn.execute("UPDATE allocation_parts SET active=-1 WHERE id='savings'", []).unwrap();
        assert!(all_parts(&conn).unwrap_err().contains("active"));
        assert!(shares_for_new_sale(&conn).is_err(),
            "a corrupt active flag must not silently exclude the historic fund");
        conn.execute("UPDATE allocation_parts SET active=1 WHERE id='savings'", []).unwrap();
        conn.execute_batch("PRAGMA ignore_check_constraints=OFF;").unwrap();

        assert_eq!(load_plan(&conn).unwrap(), before);
        assert_eq!(shares_for_new_sale(&conn).unwrap().len(), 3);
    }

    #[test]
    fn invalid_imported_part_fields_cannot_be_shown_as_valid_funds() {
        let (_dir, db) = setup();
        let conn = db.conn.lock().unwrap();
        let original = all_parts(&conn).unwrap();
        // Simulate imports that bypass SQLite CHECK constraints.
        conn.execute_batch("PRAGMA ignore_check_constraints=ON;").unwrap();
        for (column, sql, expected) in [
            ("share_bp", "UPDATE allocation_parts SET share_bp=10001 WHERE id='activity'", "share_bp"),
            ("share_bp", "UPDATE allocation_parts SET share_bp=-1 WHERE id='activity'", "share_bp"),
            ("name", "UPDATE allocation_parts SET name='   ' WHERE id='activity'", "name"),
            ("name", "UPDATE allocation_parts SET name=printf('%061d',0) WHERE id='activity'", "name"),
            ("position", "UPDATE allocation_parts SET position=0 WHERE id='activity'", "position"),
        ] {
            conn.execute_batch(sql).unwrap();
            let message = all_parts(&conn).unwrap_err();
            assert!(message.contains(expected), "{column}: {message}");
            assert!(load_plan(&conn).is_err(), "{column} must not be shown as a valid fund");
            assert!(shares_for_new_sale(&conn).is_err(), "{column} must not be frozen onto a new sale");
            conn.execute(
                "UPDATE allocation_parts SET name='Attività',share_bp=3333,position=2 WHERE id='activity'",
                [],
            ).unwrap();
            assert_eq!(all_parts(&conn).unwrap(), original);
        }
        conn.execute_batch("PRAGMA ignore_check_constraints=OFF;").unwrap();
    }

    #[test]
    fn three_default_parts_are_created_and_enabled() {
        let (_dir, db) = setup();
        let plan = load_plan(&db.conn.lock().unwrap()).unwrap();
        assert!(plan.enabled);
        let summary: Vec<_> = plan.parts.iter().map(|p| (p.id.as_str(), p.name.as_str(), p.share_bp, p.pays_expenses)).collect();
        assert_eq!(summary, vec![("personal", "Personale", 3334, false), ("activity", "Attività", 3333, true), ("savings", "Risparmio", 3333, true)]);
    }

    #[test]
    fn the_artist_can_choose_any_number_of_parts_names_and_percentages() {
        let (_dir, db) = setup();
        let two = save_plan_core(&db, PlanInput { enabled: true, parts: vec![part("Io", 6000, false), part("Studio", 4000, true)] }).unwrap();
        assert_eq!(two.parts.len(), 2);
        let five = save_plan_core(&db, PlanInput { enabled: true, parts: vec![
            part("Personale", 4000, false), part("Materiali", 2000, true), part("Tasse", 2000, false),
            part("Risparmio", 1500, true), part("Viaggi", 500, true)] }).unwrap();
        assert_eq!(five.parts.iter().map(|p| p.share_bp).sum::<i64>(), 10_000);
        assert_eq!(five.parts[1].name, "Materiali");
        assert_eq!(five.parts.iter().map(|p| p.position).collect::<Vec<_>>(), vec![1, 2, 3, 4, 5]);
    }

    #[test]
    fn invalid_plans_are_rejected_without_changing_anything() {
        let (_dir, db) = setup();
        let before = load_plan(&db.conn.lock().unwrap()).unwrap().parts;
        for bad in [
            PlanInput { enabled: true, parts: vec![part("A", 5000, false), part("B", 4999, false)] },
            PlanInput { enabled: true, parts: vec![part("A", 10001, false)] },
            PlanInput { enabled: true, parts: vec![part("A", 5000, false), part("a", 5000, false)] },
            PlanInput { enabled: true, parts: vec![part("  ", 10000, false)] },
            PlanInput { enabled: true, parts: vec![] },
            PlanInput { enabled: true, parts: (0..13).map(|i| part(&format!("P{i}"), if i == 0 { 10000 } else { 0 }, false)).collect() },
        ] {
            assert!(save_plan_core(&db, bad).is_err());
        }
        assert_eq!(load_plan(&db.conn.lock().unwrap()).unwrap().parts, before);
    }

    #[test]
    fn disabling_keeps_the_configuration_and_stops_freezing_shares_on_new_sales() {
        let (_dir, db) = setup();
        let kept = load_plan(&db.conn.lock().unwrap()).unwrap().parts;
        let input = PlanInput { enabled: false, parts: kept.iter().map(|p| PartInput {
            id: Some(p.id.clone()), name: p.name.clone(), share_bp: p.share_bp, pays_expenses: p.pays_expenses }).collect() };
        let plan = save_plan_core(&db, input).unwrap();
        assert!(!plan.enabled);
        let conn = db.conn.lock().unwrap();
        assert!(shares_for_new_sale(&conn).unwrap().is_empty());
        assert_eq!(load_plan(&conn).unwrap().parts, kept, "parts survive while the split is off");
        // An incomplete plan is acceptable while disabled.
        drop(conn);
        assert!(save_plan_core(&db, PlanInput { enabled: false, parts: vec![part("Bozza", 1, false)] }).is_ok());
    }

    #[test]
    fn removing_a_part_used_by_history_deactivates_it_and_unused_parts_are_deleted() {
        let (_dir, db) = setup();
        {
            let c = db.conn.lock().unwrap();
            c.execute("INSERT INTO artworks(id,code,title,status,created_at,updated_at) VALUES('a','A','T','sold','t','t')", []).unwrap();
            c.execute("INSERT INTO collectors(id,name,created_at,updated_at) VALUES('c','C','t','t')", []).unwrap();
            c.execute("INSERT INTO sales(id,artwork_id,collector_id,amount_cents,sold_at,created_at) VALUES('s','a','c',1000,'2026-11-21','t')", []).unwrap();
            c.execute("INSERT INTO sale_allocations(sale_id,part_id,share_bp) VALUES('s','savings',3333)", []).unwrap();
        }
        let plan = save_plan_core(&db, PlanInput { enabled: true, parts: vec![
            PartInput { id: Some("personal".into()), name: "Personale".into(), share_bp: 5000, pays_expenses: false },
            PartInput { id: Some("activity".into()), name: "Attività".into(), share_bp: 5000, pays_expenses: true }] }).unwrap();
        assert_eq!(plan.parts.len(), 2);
        let conn = db.conn.lock().unwrap();
        let all = all_parts(&conn).unwrap();
        let savings = all.iter().find(|p| p.id == "savings").expect("kept for history");
        assert!(!savings.active);
        drop(conn);
        let plan = save_plan_core(&db, PlanInput { enabled: true, parts: vec![
            PartInput { id: Some("personal".into()), name: "Personale".into(), share_bp: 10000, pays_expenses: false }] }).unwrap();
        assert_eq!(plan.parts.len(), 1);
        let ids: Vec<String> = all_parts(&db.conn.lock().unwrap()).unwrap().into_iter().map(|p| p.id).collect();
        assert_eq!(ids, vec!["personal", "savings"], "the unused 'activity' part is removed, the historical one stays");
        assert!(save_plan_core(&db, PlanInput { enabled: true, parts: vec![
            PartInput { id: Some("sconosciuta".into()), name: "X".into(), share_bp: 10000, pays_expenses: false }] }).is_err());
    }

    #[test]
    fn duplicate_persisted_part_ids_are_rejected_before_writing_even_when_disabled() {
        let (_dir, db) = setup();
        let original = load_plan(&db.conn.lock().unwrap()).unwrap();
        for is_enabled in [true, false] {
            let bad = PlanInput { enabled: is_enabled, parts: vec![
                PartInput { id: Some("personal".into()), name: "Autore".into(), share_bp: 6000, pays_expenses: false },
                PartInput { id: Some("personal".into()), name: "Studio".into(), share_bp: 4000, pays_expenses: true },
            ] };
            assert!(save_plan_core(&db, bad).unwrap_err().contains("due volte"));
            assert_eq!(load_plan(&db.conn.lock().unwrap()).unwrap().parts, original.parts);
            assert_eq!(enabled(&db.conn.lock().unwrap()).unwrap(), original.enabled);
        }
    }

    #[test]
    fn corrupted_active_percentages_cannot_be_frozen_on_a_new_sale() {
        let (_dir, db) = setup();
        let mut c = db.conn.lock().unwrap();
        c.execute("UPDATE allocation_parts SET share_bp=3000 WHERE id='personal'", []).unwrap();
        assert!(shares_for_new_sale(&c).unwrap_err().contains("100%"));
        c.execute("INSERT INTO artworks(id,code,title,status,created_at,updated_at) VALUES('a','A','T','sold','t','t')", []).unwrap();
        c.execute("INSERT INTO collectors(id,name,created_at,updated_at) VALUES('c','C','t','t')", []).unwrap();
        c.execute("INSERT INTO sales(id,artwork_id,collector_id,amount_cents,sold_at,created_at) VALUES('s','a','c',1000,'2026-11-21','t')", []).unwrap();
        // The normal sale command performs this operation inside a transaction:
        // no partially written finance row should survive a failed share validation.
        let tx = c.transaction().unwrap();
        assert!(crate::finance::attach_sale_finances(&tx, "s", 1000, 0, 0).is_err());
        drop(tx);
        let count: i64 = c.query_row("SELECT COUNT(*) FROM sale_finances WHERE sale_id='s'", [], |r| r.get(0)).unwrap();
        assert_eq!(count, 0);
    }

    #[test]
    fn ignored_part_updates_are_rejected_without_changing_the_saved_plan() {
        let (_dir, db) = setup();
        let before = load_plan(&db.conn.lock().unwrap()).unwrap();
        let edited = || PlanInput {
            enabled: true,
            parts: before.parts.iter().map(|p| PartInput {
                id: Some(p.id.clone()),
                name: if p.id == "personal" { "Nuovo nome".into() } else { p.name.clone() },
                share_bp: p.share_bp,
                pays_expenses: p.pays_expenses,
            }).collect(),
        };
        {
            let conn = db.conn.lock().unwrap();
            conn.execute_batch("CREATE TRIGGER ignore_part_edit BEFORE UPDATE ON allocation_parts
                WHEN OLD.id='personal' BEGIN SELECT RAISE(IGNORE); END;").unwrap();
        }
        let message = save_plan_core(&db, edited()).unwrap_err();
        assert!(message.contains("SQLite did not apply"), "{message}");
        assert_eq!(load_plan(&db.conn.lock().unwrap()).unwrap(), before);
        db.conn.lock().unwrap().execute_batch("DROP TRIGGER ignore_part_edit").unwrap();
        let updated = save_plan_core(&db, edited()).unwrap();
        assert_eq!(updated.parts[0].name, "Nuovo nome");
    }

    #[test]
    fn ignored_settings_update_rolls_back_earlier_part_changes() {
        let (_dir, db) = setup();
        let before = load_plan(&db.conn.lock().unwrap()).unwrap();
        let edited = || PlanInput {
            enabled: false,
            parts: before.parts.iter().map(|p| PartInput {
                id: Some(p.id.clone()),
                name: format!("Modificato {}", p.name),
                share_bp: p.share_bp,
                pays_expenses: p.pays_expenses,
            }).collect(),
        };
        db.conn.lock().unwrap().execute_batch(
            "CREATE TRIGGER ignore_plan_setting BEFORE UPDATE ON allocation_settings
             BEGIN SELECT RAISE(IGNORE); END;"
        ).unwrap();
        let message = save_plan_core(&db, edited()).unwrap_err();
        assert!(message.contains("save enabled state"), "{message}");
        assert_eq!(load_plan(&db.conn.lock().unwrap()).unwrap(), before,
            "both the enabled flag and the earlier part changes must roll back");
        db.conn.lock().unwrap().execute_batch("DROP TRIGGER ignore_plan_setting").unwrap();
        let updated = save_plan_core(&db, edited()).unwrap();
        assert!(!updated.enabled);
        assert!(updated.parts[0].name.starts_with("Modificato"));
    }

    #[test]
    fn silently_ignored_new_part_does_not_delete_the_old_configuration() {
        let (_dir, db) = setup();
        let before = load_plan(&db.conn.lock().unwrap()).unwrap();
        db.conn.lock().unwrap().execute_batch(
            "CREATE TRIGGER ignore_new_part BEFORE INSERT ON allocation_parts
             WHEN NEW.name='Nuovo' BEGIN SELECT RAISE(IGNORE); END;"
        ).unwrap();
        let message = save_plan_core(&db, PlanInput {
            enabled: true,
            parts: vec![part("Nuovo", 10_000, true)],
        }).unwrap_err();
        assert!(message.contains("save part"), "{message}");
        assert_eq!(load_plan(&db.conn.lock().unwrap()).unwrap(), before);
    }

    #[test]
    fn after_update_triggers_cannot_change_the_saved_shares_or_enabled_flag() {
        let (_dir, db) = setup();
        let before = load_plan(&db.conn.lock().unwrap()).unwrap();
        let edited = || PlanInput {
            enabled: false,
            parts: before.parts.iter().map(|p| PartInput {
                id: Some(p.id.clone()), name: format!("Nuovo {}", p.name),
                share_bp: p.share_bp, pays_expenses: p.pays_expenses,
            }).collect(),
        };
        {
            let conn = db.conn.lock().unwrap();
            conn.execute_batch(
                "CREATE TRIGGER tamper_saved_part AFTER UPDATE OF name ON allocation_parts
                   WHEN NEW.id='personal' BEGIN
                   UPDATE allocation_parts SET share_bp=1234 WHERE id=NEW.id;
                 END;"
            ).unwrap();
        }
        let message = save_plan_core(&db, edited()).unwrap_err();
        assert!(message.contains("unexpectedly"), "{message}");
        assert_eq!(load_plan(&db.conn.lock().unwrap()).unwrap(), before,
            "a mutated part must roll back all changes");
        {
            let conn = db.conn.lock().unwrap();
            conn.execute_batch(
                "DROP TRIGGER tamper_saved_part;
                 CREATE TRIGGER tamper_saved_setting AFTER UPDATE OF enabled ON allocation_settings
                   BEGIN UPDATE allocation_settings SET enabled=1; END;"
            ).unwrap();
        }
        let message = save_plan_core(&db, edited()).unwrap_err();
        assert!(message.contains("unexpectedly"), "{message}");
        assert_eq!(load_plan(&db.conn.lock().unwrap()).unwrap(), before,
            "a mutated enabled flag must roll back all part edits");
        db.conn.lock().unwrap().execute_batch("DROP TRIGGER tamper_saved_setting;").unwrap();
        let saved = save_plan_core(&db, edited()).unwrap();
        assert!(!saved.enabled);
        assert!(saved.parts.iter().all(|p| p.name.starts_with("Nuovo ")));
    }

    #[test]
    fn after_delete_trigger_cannot_silently_recreate_an_unused_allocation_part() {
        let (_dir, db) = setup();
        let before = all_parts(&db.conn.lock().unwrap()).unwrap();
        db.conn.lock().unwrap().execute_batch(
            "CREATE TRIGGER recreate_deleted_part AFTER DELETE ON allocation_parts
             WHEN OLD.id='activity' BEGIN
               INSERT INTO allocation_parts(id,name,share_bp,pays_expenses,position,active)
               VALUES(OLD.id,OLD.name,OLD.share_bp,OLD.pays_expenses,OLD.position,OLD.active);
             END;"
        ).unwrap();
        let only_personal = || PlanInput {
            enabled: true,
            parts: vec![PartInput {
                id: Some("personal".into()), name: "Personale".into(),
                share_bp: 10_000, pays_expenses: false,
            }],
        };
        let message = save_plan_core(&db, only_personal()).unwrap_err();
        assert!(message.contains("unexpectedly"), "{message}");
        assert_eq!(all_parts(&db.conn.lock().unwrap()).unwrap(), before);
        db.conn.lock().unwrap().execute_batch("DROP TRIGGER recreate_deleted_part;").unwrap();
        let saved = save_plan_core(&db, only_personal()).unwrap();
        assert_eq!(saved.parts.len(), 1);
        assert_eq!(all_parts(&db.conn.lock().unwrap()).unwrap().len(), 1);
    }

    #[test]
    fn splitting_always_adds_up_to_the_exact_net_amount() {
        assert_eq!(split(10001, &[3334, 3333, 3333]).iter().sum::<i64>(), 10001);
        assert_eq!(split(10001, &[3334, 3333, 3333]), vec![3334, 3333, 3334]);
        assert_eq!(split(100, &[10000]), vec![100]);
        assert_eq!(split(7, &[5000, 5000]), vec![3, 4]);
        assert_eq!(split(-500, &[5000, 5000]), vec![-250, -250]);
        for net in [0_i64, 1, 99, 12345, 100_000_000_000] {
            assert_eq!(split(net, &[1234, 2345, 3456, 2965]).iter().sum::<i64>(), net);
        }
    }
}
