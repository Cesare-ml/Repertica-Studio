//! Installment (rateale) payment plans for sales and the payment recorded together with a sale.
//!
//! A plan is a list of scheduled payments (due date and amount) whose total is exactly the
//! sale price. Plans only describe what is expected: money actually received is always a
//! manually recorded receipt, and the status of each installment is derived by applying the
//! receipts of the sale to the installments in due-date order.
use crate::db::Database;
use chrono::{Datelike, Local, NaiveDate};
use rusqlite::{params, Connection, Transaction};
use serde::{Deserialize, Serialize};
use tauri::State;
use uuid::Uuid;

const MIN_INSTALLMENTS: usize = 2;
const MAX_INSTALLMENTS: usize = 60;
const METHODS: [&str; 4] = ["bank_transfer", "cash", "card", "other"];

fn err(e: impl std::fmt::Display) -> String { crate::util::friendly(e) }
fn date_text(date: NaiveDate) -> String { date.format("%Y-%m-%d").to_string() }
fn canonical(value: &str) -> Option<NaiveDate> {
    NaiveDate::parse_from_str(value, "%Y-%m-%d").ok().filter(|d| date_text(*d) == value)
}

#[derive(Clone, Debug, Deserialize)]
pub struct InstallmentInput {
    pub due_date: String,
    pub amount_cents: i64,
    #[serde(default)] pub notes: Option<String>,
}

#[derive(Clone, Debug, Serialize, PartialEq)]
pub struct Installment {
    pub id: String,
    pub sale_id: String,
    pub position: i64,
    pub due_date: String,
    pub amount_cents: i64,
    pub notes: Option<String>,
    /// Part of this installment already covered by the receipts of the sale.
    pub paid_cents: i64,
    /// `PAID`, `PARTIAL` (started, not yet due), `OVERDUE` or `DUE`.
    pub status: String,
}

#[derive(Clone, Debug, Serialize, PartialEq)]
pub struct PlanLine { pub due_date: String, pub amount_cents: i64 }

/// The same date `months` later, clamped to the last day of the target month.
fn add_months(date: NaiveDate, months: u32) -> NaiveDate {
    let index = date.year() * 12 + date.month0() as i32 + months as i32;
    let (year, month) = (index.div_euclid(12), index.rem_euclid(12) as u32 + 1);
    let mut day = date.day();
    loop {
        if let Some(candidate) = NaiveDate::from_ymd_opt(year, month, day) { return candidate; }
        day -= 1;
    }
}

/// Split `total_cents` into `count` monthly installments; the cents left over go to the first ones.
pub fn generate(total_cents: i64, count: usize, first_due: NaiveDate) -> Result<Vec<PlanLine>, String> {
    if !(MIN_INSTALLMENTS..=MAX_INSTALLMENTS).contains(&count) {
        return Err(format!("Il numero di rate deve essere tra {MIN_INSTALLMENTS} e {MAX_INSTALLMENTS}"));
    }
    if total_cents < count as i64 { return Err("L'importo è troppo piccolo per questo numero di rate".into()); }
    let base = total_cents / count as i64;
    let remainder = (total_cents - base * count as i64) as usize;
    Ok((0..count).map(|i| PlanLine {
        due_date: date_text(add_months(first_due, i as u32)),
        amount_cents: base + i64::from(i < remainder),
    }).collect())
}

fn validate(sale_cents: i64, items: &[InstallmentInput]) -> Result<(), String> {
    if items.len() > MAX_INSTALLMENTS { return Err(format!("Massimo {MAX_INSTALLMENTS} rate")); }
    let mut total = 0_i64;
    for item in items {
        if canonical(&item.due_date).is_none() { return Err("Data di scadenza non valida".into()); }
        if item.amount_cents <= 0 || item.amount_cents > 100_000_000_000 { return Err("Importo della rata non valido".into()); }
        if item.notes.as_ref().is_some_and(|n| n.chars().count() > 500) { return Err("Nota troppo lunga (massimo 500 caratteri)".into()); }
        total = total.checked_add(item.amount_cents).ok_or("Importo non valido")?;
    }
    if !items.is_empty() && total != sale_cents {
        return Err(format!("Le rate devono sommare esattamente al prezzo di vendita (differenza: {} centesimi)", (total - sale_cents).abs()));
    }
    Ok(())
}

/// Replace the whole plan of a sale. An empty list removes the plan.
pub fn replace_plan(tx: &Connection, sale_id: &str, items: &[InstallmentInput]) -> Result<(), String> {
    let sale_cents: i64 = tx.query_row("SELECT amount_cents FROM sales WHERE id=?1", params![sale_id], |r| r.get(0))
        .map_err(|_| "Vendita non trovata".to_string())?;
    validate(sale_cents, items)?;
    tx.execute("DELETE FROM sale_installments WHERE sale_id=?1", params![sale_id]).map_err(err)?;
    let mut ordered: Vec<(usize, &InstallmentInput)> = items.iter().enumerate().collect();
    ordered.sort_by(|a, b| a.1.due_date.cmp(&b.1.due_date).then(a.0.cmp(&b.0)));
    for (position, (_, item)) in ordered.iter().enumerate() {
        let notes = item.notes.as_ref().map(|n| n.trim().to_string()).filter(|n| !n.is_empty());
        tx.execute("INSERT INTO sale_installments(id,sale_id,position,due_date,amount_cents,notes) VALUES(?1,?2,?3,?4,?5,?6)",
            params![Uuid::new_v4().to_string(), sale_id, position as i64 + 1, item.due_date, item.amount_cents, notes]).map_err(err)?;
    }
    Ok(())
}

/// The payment registered together with a new sale.
#[derive(Clone, Debug, Deserialize)]
pub struct PaymentInput {
    /// `UNPAID`, `PAID`, `PARTIAL` or `INSTALLMENTS`.
    pub mode: String,
    /// Money received now: the full price for `PAID`, an advance for `PARTIAL` and `INSTALLMENTS`.
    #[serde(default)] pub received_cents: Option<i64>,
    #[serde(default)] pub received_at: Option<String>,
    #[serde(default)] pub method: Option<String>,
    #[serde(default)] pub reference: Option<String>,
    #[serde(default)] pub installments: Vec<InstallmentInput>,
}

pub fn apply_payment(tx: &Transaction<'_>, sale_id: &str, sale_cents: i64, payment: &PaymentInput) -> Result<(), String> {
    let received = match payment.mode.as_str() {
        "UNPAID" => return Ok(()),
        "PAID" => sale_cents,
        "PARTIAL" => {
            let cents = payment.received_cents.unwrap_or(0);
            if cents <= 0 || cents >= sale_cents { return Err("L'importo incassato deve essere maggiore di zero e minore del prezzo".into()); }
            cents
        }
        "INSTALLMENTS" => {
            if payment.installments.len() < MIN_INSTALLMENTS { return Err(format!("Un piano rateale richiede almeno {MIN_INSTALLMENTS} rate")); }
            replace_plan(tx, sale_id, &payment.installments)?;
            // An optional advance is a receipt like any other.
            match payment.received_cents.unwrap_or(0) {
                0 => return Ok(()),
                cents if cents > 0 && cents < sale_cents => cents,
                _ => return Err("L'acconto deve essere maggiore di zero e minore del prezzo".into()),
            }
        }
        _ => return Err("Modalità di incasso non valida".into()),
    };
    let received_at = payment.received_at.as_deref().unwrap_or("");
    if canonical(received_at).is_none() { return Err("Data di incasso non valida".into()); }
    let method = payment.method.as_deref().unwrap_or("bank_transfer");
    if !METHODS.contains(&method) { return Err("Metodo di incasso non valido".into()); }
    let reference = payment.reference.as_ref().map(|r| r.trim().to_string()).filter(|r| !r.is_empty());
    if reference.as_ref().is_some_and(|r| r.chars().count() > 200) { return Err("Riferimento troppo lungo (massimo 200 caratteri)".into()); }
    tx.execute("INSERT INTO sale_receipts(id,sale_id,received_at,amount_cents,method,reference,notes,created_at) VALUES(?1,?2,?3,?4,?5,?6,NULL,?7)",
        params![Uuid::new_v4().to_string(), sale_id, received_at, received, method, reference, chrono::Utc::now().to_rfc3339()]).map_err(err)?;
    Ok(())
}

/// Installments of every sale with their status as of `today`.
pub fn list_with_status(conn: &Connection, today: NaiveDate) -> Result<Vec<Installment>, String> {
    let mut statement = conn.prepare(
        "SELECT i.id,i.sale_id,i.position,i.due_date,i.amount_cents,i.notes,
                COALESCE((SELECT SUM(amount_cents) FROM sale_receipts WHERE sale_id=i.sale_id),0)
         FROM sale_installments i ORDER BY i.sale_id,i.position").map_err(err)?;
    let rows = statement.query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?, r.get::<_, i64>(2)?,
        r.get::<_, String>(3)?, r.get::<_, i64>(4)?, r.get::<_, Option<String>>(5)?, r.get::<_, i64>(6)?)))
        .map_err(err)?.collect::<Result<Vec<_>, _>>().map_err(err)?;
    let today = date_text(today);
    let mut result = Vec::with_capacity(rows.len());
    let (mut current_sale, mut remaining) = (String::new(), 0_i64);
    for (id, sale_id, position, due_date, amount_cents, notes, received) in rows {
        if sale_id != current_sale { current_sale = sale_id.clone(); remaining = received; }
        let paid_cents = remaining.clamp(0, amount_cents);
        remaining -= paid_cents;
        let status = if paid_cents == amount_cents { "PAID" }
            else if due_date < today { "OVERDUE" }
            else if paid_cents > 0 { "PARTIAL" } else { "DUE" };
        result.push(Installment { id, sale_id, position, due_date, amount_cents, notes, paid_cents, status: status.into() });
    }
    Ok(result)
}

#[tauri::command]
pub fn list_sale_installments(db: State<'_, Database>) -> Result<Vec<Installment>, String> {
    let conn = db.conn.lock().map_err(err)?;
    list_with_status(&conn, Local::now().date_naive())
}

#[tauri::command]
pub fn save_sale_installments(db: State<'_, Database>, sale_id: String, items: Vec<InstallmentInput>) -> Result<(), String> {
    let mut conn = db.conn.lock().map_err(err)?;
    let tx = conn.transaction().map_err(err)?;
    if !items.is_empty() && items.len() < MIN_INSTALLMENTS { return Err(format!("Un piano rateale richiede almeno {MIN_INSTALLMENTS} rate")); }
    replace_plan(&tx, &sale_id, &items)?;
    tx.commit().map_err(err)
}

#[tauri::command]
pub fn generate_installment_plan(amount_cents: i64, count: usize, first_due: String) -> Result<Vec<PlanLine>, String> {
    let first = canonical(&first_due).ok_or("Data della prima scadenza non valida")?;
    if amount_cents <= 0 || amount_cents > 100_000_000_000 { return Err("Importo non valido".into()); }
    generate(amount_cents, count, first)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn d(s: &str) -> NaiveDate { NaiveDate::parse_from_str(s, "%Y-%m-%d").unwrap() }
    fn setup() -> (tempfile::TempDir, Database) {
        let dir = tempfile::tempdir().unwrap();
        let db = crate::db::open(dir.path().to_path_buf()).unwrap();
        {
            let c = db.conn.lock().unwrap();
            c.execute("INSERT INTO artworks(id,code,title,status,created_at,updated_at) VALUES('a','A','T','sold','t','t')", []).unwrap();
            c.execute("INSERT INTO collectors(id,name,created_at,updated_at) VALUES('c','C','t','t')", []).unwrap();
            c.execute("INSERT INTO sales(id,artwork_id,collector_id,amount_cents,sold_at,created_at) VALUES('s','a','c',100001,'2026-01-31','t')", []).unwrap();
        }
        (dir, db)
    }
    fn line(due: &str, cents: i64) -> InstallmentInput { InstallmentInput { due_date: due.into(), amount_cents: cents, notes: None } }

    #[test]
    fn generated_plans_split_exactly_and_clamp_to_the_end_of_the_month() {
        let plan = generate(100_001, 3, d("2026-01-31")).unwrap();
        assert_eq!(plan.iter().map(|p| p.amount_cents).collect::<Vec<_>>(), vec![33_334, 33_334, 33_333]);
        assert_eq!(plan.iter().map(|p| p.amount_cents).sum::<i64>(), 100_001);
        assert_eq!(plan.iter().map(|p| p.due_date.as_str()).collect::<Vec<_>>(), vec!["2026-01-31", "2026-02-28", "2026-03-31"]);
        let leap = generate(1_000, 2, d("2028-01-31")).unwrap();
        assert_eq!(leap[1].due_date, "2028-02-29");
        assert_eq!(generate(1_000, 13, d("2026-11-30")).unwrap()[2].due_date, "2027-01-30");
        assert_eq!(generate(2_000, 60, d("2026-01-15")).unwrap().len(), 60);
        assert!(generate(1_000, 1, d("2026-01-15")).is_err());
        assert!(generate(1_000, 61, d("2026-01-15")).is_err());
        assert!(generate(5, 10, d("2026-01-15")).is_err());
    }

    #[test]
    fn a_plan_must_total_the_sale_price_and_is_stored_in_due_date_order() {
        let (_dir, db) = setup();
        let mut c = db.conn.lock().unwrap();
        let tx = c.transaction().unwrap();
        assert!(replace_plan(&tx, "s", &[line("2026-02-01", 50_000), line("2026-03-01", 50_000)]).unwrap_err().contains("sommare"));
        assert!(replace_plan(&tx, "s", &[line("2026-02-30", 50_000), line("2026-03-01", 50_001)]).unwrap_err().contains("Data"));
        assert!(replace_plan(&tx, "s", &[line("2026-02-01", 0), line("2026-03-01", 100_001)]).unwrap_err().contains("Importo"));
        assert!(replace_plan(&tx, "assente", &[line("2026-02-01", 1)]).is_err());
        replace_plan(&tx, "s", &[line("2026-03-01", 40_001), line("2026-02-01", 60_000)]).unwrap();
        tx.commit().unwrap();
        let plan = list_with_status(&c, d("2026-01-01")).unwrap();
        assert_eq!(plan.iter().map(|i| (i.position, i.due_date.as_str(), i.amount_cents)).collect::<Vec<_>>(),
            vec![(1, "2026-02-01", 60_000), (2, "2026-03-01", 40_001)]);
        let tx = c.transaction().unwrap();
        replace_plan(&tx, "s", &[]).unwrap();
        tx.commit().unwrap();
        assert!(list_with_status(&c, d("2026-01-01")).unwrap().is_empty(), "an empty list removes the plan");
    }

    #[test]
    fn receipts_cover_installments_in_order_and_overdue_ones_are_flagged() {
        let (_dir, db) = setup();
        let mut c = db.conn.lock().unwrap();
        let tx = c.transaction().unwrap();
        replace_plan(&tx, "s", &[line("2026-02-01", 30_000), line("2026-03-01", 30_000), line("2026-04-01", 40_001)]).unwrap();
        tx.commit().unwrap();
        let status = |c: &Connection, today: &str| list_with_status(c, d(today)).unwrap().into_iter().map(|i| (i.status, i.paid_cents)).collect::<Vec<_>>();
        assert_eq!(status(&c, "2026-01-15"), vec![("DUE".into(), 0), ("DUE".into(), 0), ("DUE".into(), 0)]);
        c.execute("INSERT INTO sale_receipts(id,sale_id,received_at,amount_cents,method,created_at) VALUES('r1','s','2026-02-01',45000,'cash','t')", []).unwrap();
        assert_eq!(status(&c, "2026-02-10"), vec![("PAID".into(), 30_000), ("PARTIAL".into(), 15_000), ("DUE".into(), 0)]);
        assert_eq!(status(&c, "2026-03-10"), vec![("PAID".into(), 30_000), ("OVERDUE".into(), 15_000), ("DUE".into(), 0)]);
        assert_eq!(status(&c, "2026-05-01"), vec![("PAID".into(), 30_000), ("OVERDUE".into(), 15_000), ("OVERDUE".into(), 0)]);
        c.execute("INSERT INTO sale_receipts(id,sale_id,received_at,amount_cents,method,created_at) VALUES('r2','s','2026-03-02',55001,'cash','t')", []).unwrap();
        assert!(status(&c, "2026-05-01").iter().all(|(s, _)| s == "PAID"));
    }

    #[test]
    fn payments_registered_with_a_sale_create_receipts_and_plans_atomically() {
        let (_dir, db) = setup();
        let mut c = db.conn.lock().unwrap();
        let paid = |mode: &str, cents: Option<i64>| PaymentInput { mode: mode.into(), received_cents: cents, received_at: Some("2026-02-01".into()),
            method: Some("cash".into()), reference: None, installments: vec![] };
        let tx = c.transaction().unwrap();
        apply_payment(&tx, "s", 100_001, &paid("UNPAID", None)).unwrap();
        apply_payment(&tx, "s", 100_001, &paid("PARTIAL", Some(25_000))).unwrap();
        assert!(apply_payment(&tx, "s", 100_001, &paid("PARTIAL", Some(100_001))).is_err());
        assert!(apply_payment(&tx, "s", 100_001, &paid("PARTIAL", Some(0))).is_err());
        assert!(apply_payment(&tx, "s", 100_001, &paid("MAGIC", None)).is_err());
        let mut bad_date = paid("PAID", None);
        bad_date.received_at = Some("01/02/2026".into());
        assert!(apply_payment(&tx, "s", 100_001, &bad_date).is_err());
        let mut bad_method = paid("PARTIAL", Some(10));
        bad_method.method = Some("bitcoin".into());
        assert!(apply_payment(&tx, "s", 100_001, &bad_method).is_err());
        let mut plan = paid("INSTALLMENTS", Some(10_000));
        plan.installments = vec![line("2026-03-01", 50_000), line("2026-04-01", 50_001)];
        apply_payment(&tx, "s", 100_001, &plan).unwrap();
        let mut single = paid("INSTALLMENTS", None);
        single.installments = vec![line("2026-03-01", 100_001)];
        assert!(apply_payment(&tx, "s", 100_001, &single).unwrap_err().contains("almeno"));
        tx.commit().unwrap();
        let received: i64 = c.query_row("SELECT SUM(amount_cents) FROM sale_receipts WHERE sale_id='s'", [], |r| r.get(0)).unwrap();
        assert_eq!(received, 35_000, "partial 25000 plus the advance of the plan");
        let rows: i64 = c.query_row("SELECT count(*) FROM sale_installments", [], |r| r.get(0)).unwrap();
        assert_eq!(rows, 2);
        let tx = c.transaction().unwrap();
        // Paying in full after an advance would exceed the price: the database refuses it.
        assert!(apply_payment(&tx, "s", 100_001, &paid("PAID", None)).is_err());
    }
}
