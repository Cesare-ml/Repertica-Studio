//! Local studio economics. Monetary values are integer EUR cents and share ratios
//! are integer basis points. Sales are recorded turnover, not verified cash receipts.
use crate::db::Database;
use chrono::{NaiveDate, Utc};
use rusqlite::{params, Transaction};
use serde::{Deserialize, Serialize};
use tauri::State;
use uuid::Uuid;

fn error(e: impl std::fmt::Display) -> String { crate::util::friendly(e) }
fn date_ok(date: &str) -> bool {
    NaiveDate::parse_from_str(date, "%Y-%m-%d")
        .map(|d| d.format("%Y-%m-%d").to_string() == date)
        .unwrap_or(false)
}
fn opt_text(v: Option<String>, max: usize) -> Result<Option<String>, String> {
    match v {
        None => Ok(None),
        Some(s) if s.trim().is_empty() => Ok(None),
        Some(s) if s.chars().count() <= max => Ok(Some(s.trim().to_string())),
        Some(_) => Err(format!("Text exceeds {max} characters")),
    }
}
fn text(v: String, max: usize) -> Result<String,String> {
    opt_text(Some(v), max)?.ok_or_else(||"Required field is empty".to_string())
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct StudioSettings {
    pub artist_coefficient: f64,
    pub base_price_cm2_cents: i64,
    pub personal_bp: i64,
    pub activity_bp: i64,
    pub savings_bp: i64,
    pub treasury_start_date: Option<String>,
    pub default_tax_bp: i64,
    pub ui_language: Option<String>,
    /// Name printed on certificates as the issuing artist.
    #[serde(default)] pub artist_name: Option<String>,
    /// Stage name: shown on certificates and brochures instead of the full name.
    #[serde(default)] pub artist_alias: Option<String>,
}
fn valid_settings(s: &StudioSettings) -> Result<(),String> {
    if !s.artist_coefficient.is_finite() || s.artist_coefficient <= 0.0 || s.artist_coefficient > 100000.0 {
        return Err("Artist coefficient must be positive".into());
    }
    if !(0..=100000).contains(&s.base_price_cm2_cents) || !(0..=10000).contains(&s.default_tax_bp) {
        return Err("Invalid price rate or tax percentage".into());
    }
    if [s.personal_bp, s.activity_bp, s.savings_bp].iter().any(|v| !(0..=10000).contains(v)) ||
       s.personal_bp + s.activity_bp + s.savings_bp != 10000 {
        return Err("The three allocations must total exactly 100%".into());
    }
    if s.ui_language.as_ref().is_some_and(|lang| !["it","en","es","fr","de"].contains(&lang.as_str())) {
        return Err("Unsupported language code".into());
    }
    if s.treasury_start_date.as_ref().is_some_and(|date| !date_ok(date)) {
        return Err("Invalid treasury start date".into());
    }
    Ok(())
}
fn settings_tx(c: &rusqlite::Connection) -> Result<StudioSettings,String> {
    c.query_row("SELECT artist_coefficient,base_price_cm2_cents,personal_bp,activity_bp,savings_bp,treasury_start_date,default_tax_bp,ui_language,artist_name,artist_alias FROM studio_settings WHERE singleton=1",
        [], |r|Ok(StudioSettings{
            artist_coefficient:r.get(0)?,base_price_cm2_cents:r.get(1)?,personal_bp:r.get(2)?,
            activity_bp:r.get(3)?,savings_bp:r.get(4)?,treasury_start_date:r.get(5)?,default_tax_bp:r.get(6)?,ui_language:r.get(7)?,artist_name:r.get(8)?,artist_alias:r.get(9)?
        })).map_err(error)
}
#[tauri::command]
pub fn get_studio_settings(db:State<'_,Database>) -> Result<StudioSettings,String> {
    let c=db.conn.lock().map_err(error)?;
    settings_tx(&c)
}
// Persist certificate identity, price parameters and allocation defaults together.
// SQLite files imported from another application can contain triggers that ignore
// updates or silently rewrite values. Check the exact stored row before committing.
fn save_studio_settings_core(db: &Database, mut input: StudioSettings) -> Result<StudioSettings, String> {
    valid_settings(&input)?;
    input.artist_name = crate::commands::clean(input.artist_name, 240)?;
    input.artist_alias = crate::commands::clean(input.artist_alias, 240)?;
    let mut conn = db.conn.lock().map_err(error)?;
    let tx = conn.transaction().map_err(error)?;
    let updated = tx.execute(
        "UPDATE studio_settings SET artist_coefficient=?1,base_price_cm2_cents=?2,personal_bp=?3,activity_bp=?4,savings_bp=?5,treasury_start_date=?6,default_tax_bp=?7,ui_language=?8,artist_name=?9,artist_alias=?10 WHERE singleton=1",
        params![input.artist_coefficient,input.base_price_cm2_cents,input.personal_bp,input.activity_bp,input.savings_bp,input.treasury_start_date,input.default_tax_bp,input.ui_language,input.artist_name,input.artist_alias]
    ).map_err(error)?;
    if updated != 1 {
        return Err("SQLite did not save studio settings; reload and retry".into());
    }
    let saved = settings_tx(&tx)?;
    if saved != input {
        return Err("SQLite did not preserve studio settings; no changes were saved".into());
    }
    tx.commit().map_err(error)?;
    Ok(saved)
}

#[tauri::command]
pub fn save_studio_settings(db: State<'_, Database>, input: StudioSettings) -> Result<StudioSettings, String> {
    save_studio_settings_core(&db, input)
}

pub fn attach_sale_finances(tx: &Transaction<'_>,sale_id: &str,gross:i64,taxes:i64,shipping:i64)->Result<(),String>{
    if gross<=0 || taxes<0 || shipping<0 || taxes.checked_add(shipping).is_none_or(|s|s>gross) {
        return Err("Taxes and shipping must not exceed the sale amount".into());
    }
    let s=settings_tx(tx)?;
    // All writes share the caller's sale transaction. Imported SQLite files can
    // contain triggers with RAISE(IGNORE), or AFTER triggers which erase/rewrite
    // rows while reporting a successful INSERT. Check both the affected count
    // and the persisted values before the transaction is allowed to commit.
    let written=tx.execute("INSERT INTO sale_finances(sale_id,tax_cents,shipping_cents,personal_bp,activity_bp,savings_bp) VALUES(?1,?2,?3,?4,?5,?6)",
        params![sale_id,taxes,shipping,s.personal_bp,s.activity_bp,s.savings_bp]).map_err(error)?;
    if written != 1 {
        return Err("SQLite did not save the sale financial details".into());
    }
    // The shares in force today are frozen on this sale; later plan changes
    // never rewrite history, even if the configuration is subsequently disabled.
    let mut expected=crate::allocations::shares_for_new_sale(tx)?;
    for (part_id,bp) in &expected {
        let written=tx.execute("INSERT INTO sale_allocations(sale_id,part_id,share_bp) VALUES(?1,?2,?3)",
            params![sale_id,part_id,bp]).map_err(error)?;
        if written != 1 {
            return Err("SQLite did not save the frozen sale allocations".into());
        }
    }
    let saved:(i64,i64,i64,i64,i64)=tx.query_row(
        "SELECT tax_cents,shipping_cents,personal_bp,activity_bp,savings_bp FROM sale_finances WHERE sale_id=?1",
        params![sale_id],|r| Ok((r.get(0)?,r.get(1)?,r.get(2)?,r.get(3)?,r.get(4)?))
    ).map_err(|_| "SQLite did not preserve the sale financial details".to_string())?;
    if saved != (taxes,shipping,s.personal_bp,s.activity_bp,s.savings_bp) {
        return Err("SQLite did not preserve the sale financial details".into());
    }
    let mut statement=tx.prepare(
        "SELECT part_id,share_bp FROM sale_allocations WHERE sale_id=?1 ORDER BY part_id"
    ).map_err(error)?;
    let actual=statement.query_map(params![sale_id],
        |r| Ok((r.get::<_,String>(0)?,r.get::<_,i64>(1)?))
    ).map_err(error)?.collect::<Result<Vec<_>,_>>().map_err(error)?;
    expected.sort_by(|a,b|a.0.cmp(&b.0));
    if actual != expected {
        return Err("SQLite did not preserve the frozen sale allocations".into());
    }
    Ok(())
}
#[derive(Debug, Clone, Serialize)]
pub struct Expense {
    pub id:String,pub expense_date:String,pub category:String,pub description:String,
    pub amount_cents:i64,pub fund:Option<String>,pub supplier:Option<String>,pub purchase_url:Option<String>,
    pub notes:Option<String>,pub created_at:String,pub updated_at:String,
}
#[derive(Debug,Deserialize)]
pub struct ExpenseInput {
    pub id:Option<String>,pub expense_date:String,pub category:String,pub description:String,
    pub amount_cents:i64,pub fund:Option<String>,pub supplier:Option<String>,pub purchase_url:Option<String>,pub notes:Option<String>,
}
fn expense_row(r:&rusqlite::Row<'_>)->rusqlite::Result<Expense>{
    Ok(Expense{id:r.get(0)?,expense_date:r.get(1)?,category:r.get(2)?,description:r.get(3)?,
        amount_cents:r.get(4)?,fund:r.get::<_,Option<String>>(5)?,supplier:r.get(6)?,purchase_url:r.get(7)?,
        notes:r.get(8)?,created_at:r.get(9)?,updated_at:r.get(10)?})
}
#[tauri::command]
pub fn list_expenses(db:State<'_,Database>)->Result<Vec<Expense>,String>{
    let c=db.conn.lock().map_err(error)?;
    let mut q=c.prepare("SELECT id,expense_date,category,description,amount_cents,fund,supplier,purchase_url,notes,created_at,updated_at FROM expenses ORDER BY expense_date DESC,created_at DESC").map_err(error)?;
    let rows=q.query_map([],expense_row).map_err(error)?.collect::<Result<Vec<_>,_>>().map_err(error)?;
    Ok(rows)
}
fn save_expense_core(db:&Database,input:ExpenseInput)->Result<Expense,String>{
    if !date_ok(&input.expense_date) {return Err("Invalid expense date".into())}
    if input.amount_cents<=0 || input.amount_cents>100_000_000_000 {return Err("Invalid amount".into())}
    let category=text(input.category,120)?;
    let description=text(input.description,1500)?;
    let supplier=opt_text(input.supplier,400)?;
    let purchase_url=opt_text(input.purchase_url,2000)?;
    if purchase_url.as_ref().is_some_and(|v|!v.starts_with("https://")&&!v.starts_with("http://")){
        return Err("Purchase link must use HTTPS or HTTP".into());
    }
    let notes=opt_text(input.notes,20000)?;
    let mut c=db.conn.lock().map_err(error)?;
    // The write and its read-back share a transaction: an imported SQLite
    // archive may have AFTER triggers that change a row without an error.
    // Returning Err rolls back both a new expense and an attempted edit.
    let tx=c.transaction().map_err(error)?;
    let fund=input.fund.clone().map(|f|f.trim().to_string()).filter(|f|!f.is_empty());
    if let Some(fund)=&fund {
        let valid:bool=tx.query_row("SELECT EXISTS(SELECT 1 FROM allocation_parts WHERE id=?1 AND pays_expenses=1)",
            params![fund],|r|r.get(0)).map_err(error)?;
        if !valid {return Err("Fondo di spesa non valido".into())}
    }
    let updating=input.id.is_some();
    let id=input.id.unwrap_or_else(||Uuid::new_v4().to_string());
    let exists:bool=tx.query_row("SELECT EXISTS(SELECT 1 FROM expenses WHERE id=?1)",
        params![&id],|r|r.get(0)).map_err(error)?;
    let now=Utc::now().to_rfc3339();
    let written=if exists {
        tx.execute("UPDATE expenses SET expense_date=?2,category=?3,description=?4,amount_cents=?5,fund=?6,supplier=?7,purchase_url=?8,notes=?9,updated_at=?10 WHERE id=?1",
            params![&id,&input.expense_date,&category,&description,input.amount_cents,&fund,&supplier,&purchase_url,&notes,&now]).map_err(error)?
    }else{
        if updating {return Err("Expense does not exist".into());}
        tx.execute("INSERT INTO expenses(id,expense_date,category,description,amount_cents,fund,supplier,purchase_url,notes,created_at,updated_at) VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?10)",
            params![&id,&input.expense_date,&category,&description,input.amount_cents,&fund,&supplier,&purchase_url,&notes,&now]).map_err(error)?
    };
    if written != 1 {
        return Err("Expense was not updated by SQLite; reload the record and retry".into());
    }
    let saved=tx.query_row("SELECT id,expense_date,category,description,amount_cents,fund,supplier,purchase_url,notes,created_at,updated_at FROM expenses WHERE id=?1",
        params![&id],expense_row).map_err(|_|"SQLite did not preserve the saved expense".to_string())?;
    if saved.id != id || saved.expense_date != input.expense_date
        || saved.category != category || saved.description != description
        || saved.amount_cents != input.amount_cents || saved.fund != fund
        || saved.supplier != supplier || saved.purchase_url != purchase_url
        || saved.notes != notes || saved.updated_at != now
        || (!exists && saved.created_at != now) {
        return Err("SQLite did not preserve the saved expense".into());
    }
    tx.commit().map_err(error)?;
    Ok(saved)
}
#[tauri::command]
pub fn save_expense(db:State<'_,Database>,input:ExpenseInput)->Result<Expense,String>{
    save_expense_core(&db,input)
}
// Verify the row really disappeared before committing: a trigger may restore
// it after DELETE while SQLite still reports one affected row.
fn delete_expense_core(db:&Database,id:&str)->Result<(),String>{
    let mut c=db.conn.lock().map_err(error)?;
    let tx=c.transaction().map_err(error)?;
    if tx.execute("DELETE FROM expenses WHERE id=?1",params![id]).map_err(error)?!=1 {
        return Err("Expense not found or deletion ignored by SQLite".into());
    }
    let still_present:bool=tx.query_row("SELECT EXISTS(SELECT 1 FROM expenses WHERE id=?1)",
        params![id],|r|r.get(0)).map_err(error)?;
    if still_present {
        return Err("SQLite did not remove the expense".into());
    }
    tx.commit().map_err(error)
}
#[tauri::command]
pub fn delete_expense(db:State<'_,Database>,id:String)->Result<(),String>{
    delete_expense_core(&db,&id)
}
#[derive(Debug,Clone,Serialize)]
pub struct PartAmount { pub part_id:String, pub share_cents:i64, pub expense_cents:i64 }
#[derive(Debug,Clone,Serialize)]
pub struct MonthlyFinance {
    pub month:String,pub gross_cents:i64,pub taxes_cents:i64,pub shipping_cents:i64,
    pub net_cents:i64,pub expense_cents:i64,pub margin_cents:i64,
    /// One entry per part of the plan, in display order.
    pub parts:Vec<PartAmount>,
}
#[derive(Debug,Clone,Serialize)]
pub struct PartTotal {
    pub part_id:String,pub name:String,pub active:bool,pub pays_expenses:bool,
    pub share_cents:i64,pub expense_cents:i64,pub balance_cents:i64,
}
#[derive(Serialize)]
pub struct FinanceOverview {
    pub year:i32,pub months:Vec<MonthlyFinance>,pub gross_cents:i64,pub net_cents:i64,
    pub expense_cents:i64,pub margin_cents:i64,
    pub allocation_enabled:bool,pub parts:Vec<PartTotal>,
}
// Imported SQLite data may bypass database CHECK constraints. Report calculations
// must return a recoverable error instead of panicking or silently wrapping cents.
fn add_cents(total:&mut i64,value:i64,field:&str)->Result<(),String>{
    *total=total.checked_add(value).ok_or_else(||format!("Financial summary overflow: {field}"))?;
    Ok(())
}
fn subtract_cents(left:i64,right:i64,field:&str)->Result<i64,String>{
    left.checked_sub(right).ok_or_else(||format!("Financial summary overflow: {field}"))
}
fn finance_overview_core(c:&rusqlite::Connection,year:i32)->Result<FinanceOverview,String>{
    use std::collections::HashMap;
    if !(1900..=2200).contains(&year) {return Err("Invalid year".into())}
    let settings=settings_tx(c)?;
    let parts=crate::allocations::all_parts(c)?;
    let position:HashMap<&str,usize>=parts.iter().enumerate().map(|(i,p)|(p.id.as_str(),i)).collect();
    let mut months=(1..=12).map(|m|{
        let mut month=MonthlyFinance{month:format!("{year}-{m:02}"),gross_cents:0,taxes_cents:0,shipping_cents:0,
            net_cents:0,expense_cents:0,margin_cents:0,parts:vec![]};
        month.parts=parts.iter().map(|p|PartAmount{part_id:p.id.clone(),share_cents:0,expense_cents:0}).collect();
        month
    }).collect::<Vec<_>>();
    // Shares frozen on each sale, in display order of the parts.
    let mut shares:HashMap<String,Vec<(String,i64)>>=HashMap::new();
    {
        // LEFT JOIN is intentional: an imported database may contain an orphaned
        // frozen allocation if a third-party tool bypassed SQLite foreign keys.
        // An inner join would silently discard its share and understate balances.
        let mut q=c.prepare("SELECT a.sale_id,a.part_id,a.share_bp FROM sale_allocations a
            LEFT JOIN allocation_parts p ON p.id=a.part_id ORDER BY a.sale_id,p.position,p.id").map_err(error)?;
        let rows=q.query_map([],|r|Ok((r.get::<_,String>(0)?,r.get::<_,String>(1)?,r.get::<_,i64>(2)?))).map_err(error)?
            .collect::<Result<Vec<_>,_>>().map_err(error)?;
        for (sale,part,bp) in rows {shares.entry(sale).or_default().push((part,bp));}
    }
    // SQLite files edited outside Studio can contain sales with broken foreign keys.
    // Check owners alongside the monetary figures: a report must not silently
    // count turnover for an artwork or collector that no longer exists.
    let mut q=c.prepare("SELECT s.id,s.amount_cents,s.sold_at,sf.tax_cents,sf.shipping_cents,
            a.id,co.id
        FROM sales s
        LEFT JOIN sale_finances sf ON sf.sale_id=s.id
        LEFT JOIN artworks a ON a.id=s.artwork_id
        LEFT JOIN collectors co ON co.id=s.collector_id
        WHERE substr(s.sold_at,1,4)=?1").map_err(error)?;
    let rows=q.query_map(params![year.to_string()],|r| Ok((r.get::<_,String>(0)?,r.get::<_,i64>(1)?,r.get::<_,String>(2)?,
        r.get::<_,Option<i64>>(3)?,r.get::<_,Option<i64>>(4)?,r.get::<_,Option<String>>(5)?,r.get::<_,Option<String>>(6)?))).map_err(error)?
        .collect::<Result<Vec<_>,_>>().map_err(error)?;
    for (sale,gross,date,taxes,shipping,artwork,collector) in rows {
        if settings.treasury_start_date.as_ref().is_some_and(|start|date.as_str()<start.as_str()){continue;}
        if !date_ok(&date) {return Err(format!("Invalid sale date in archive: {sale}"));}
        if artwork.is_none() {return Err(format!("Archived sale has missing artwork: {sale}"));}
        if collector.is_none() {return Err(format!("Archived sale has missing collector: {sale}"));}
        // Sales are created with a detail row, and schema v2 backfills legacy sales.
        // If imported SQLite lost that row, never invent zero taxes/shipping.
        let taxes=taxes.ok_or_else(||format!("Archived sale has missing financial details: {sale}"))?;
        let shipping=shipping.ok_or_else(||format!("Archived sale has missing financial details: {sale}"))?;
        if gross<=0 || taxes<0 || shipping<0 {
            return Err(format!("Invalid sale finances in archive: {sale}"));
        }
        let net=subtract_cents(gross,taxes,"sale taxes")
            .and_then(|v|subtract_cents(v,shipping,"sale shipping"))?;
        if net<0 {return Err(format!("Sale deductions exceed the amount: {sale}"));}
        let index=date[5..7].parse::<usize>().map_err(error)?-1;
        let month=&mut months[index];
        add_cents(&mut month.gross_cents,gross,"monthly gross")?;
        add_cents(&mut month.taxes_cents,taxes,"monthly taxes")?;
        add_cents(&mut month.shipping_cents,shipping,"monthly shipping")?;
        add_cents(&mut month.net_cents,net,"monthly net")?;
        if let Some(frozen)=shares.get(&sale) {
            if frozen.iter().any(|(_,bp)|!(0..=10_000).contains(bp))
                || frozen.iter().map(|(_,bp)|*bp).sum::<i64>() != 10_000 {
                return Err(format!("Invalid frozen allocation percentages: {sale}"));
            }
            let amounts=crate::allocations::split(net,&frozen.iter().map(|(_,bp)|*bp).collect::<Vec<_>>());
            for ((part,_),amount) in frozen.iter().zip(amounts) {
                let slot=position.get(part.as_str()).ok_or_else(||format!(
                    "Unknown allocation part in archived sale {sale}: {part}"
                ))?;
                add_cents(&mut month.parts[*slot].share_cents,amount,"monthly allocation")?;
            }
        }
    }
    let mut q=c.prepare("SELECT expense_date,amount_cents,fund FROM expenses WHERE substr(expense_date,1,4)=?1").map_err(error)?;
    let rows=q.query_map(params![year.to_string()],|r|Ok((r.get::<_,String>(0)?,r.get::<_,i64>(1)?,r.get::<_,Option<String>>(2)?)))
        .map_err(error)?.collect::<Result<Vec<_>,_>>().map_err(error)?;
    for (date,amount,fund) in rows {
        if settings.treasury_start_date.as_ref().is_some_and(|start|date.as_str()<start.as_str()){continue;}
        if !date_ok(&date) {return Err(format!("Invalid expense date in archive: {date}"));}
        if amount<=0 {return Err(format!("Invalid expense amount in archive: {date}"));}
        let index=date[5..7].parse::<usize>().map_err(error)?-1;
        let month=&mut months[index];
        add_cents(&mut month.expense_cents,amount,"monthly expenses")?;
        if let Some(fund)=fund.as_deref() {
            let slot=position.get(fund).ok_or_else(||format!(
                "Unknown expense fund in archived record ({date}): {fund}"
            ))?;
            add_cents(&mut month.parts[*slot].expense_cents,amount,"monthly allocated expenses")?;
        }
    }
    let mut summary=FinanceOverview{year,months,gross_cents:0,net_cents:0,expense_cents:0,margin_cents:0,
        allocation_enabled:crate::allocations::enabled(c)?,parts:vec![]};
    let mut totals:Vec<PartTotal>=parts.iter().map(|p|PartTotal{part_id:p.id.clone(),name:p.name.clone(),active:p.active,
        pays_expenses:p.pays_expenses,share_cents:0,expense_cents:0,balance_cents:0}).collect();
    for m in &mut summary.months {
        m.margin_cents=subtract_cents(m.net_cents,m.expense_cents,"monthly margin")?;
        add_cents(&mut summary.gross_cents,m.gross_cents,"annual gross")?;
        add_cents(&mut summary.net_cents,m.net_cents,"annual net")?;
        add_cents(&mut summary.expense_cents,m.expense_cents,"annual expenses")?;
        for (slot,amount) in m.parts.iter().enumerate() {
            add_cents(&mut totals[slot].share_cents,amount.share_cents,"annual allocation")?;
            add_cents(&mut totals[slot].expense_cents,amount.expense_cents,"annual allocated expenses")?;
        }
    }
    for t in &mut totals {
        t.balance_cents=subtract_cents(t.share_cents,t.expense_cents,"allocation balance")?;
    }
    // Inactive parts appear only while they still carry figures for the year.
    summary.parts=totals.into_iter().filter(|t|t.active||t.share_cents!=0||t.expense_cents!=0).collect();
    summary.margin_cents=subtract_cents(summary.net_cents,summary.expense_cents,"annual margin")?;
    Ok(summary)
}
#[tauri::command]
pub fn finance_overview(db:State<'_,Database>,year:i32)->Result<FinanceOverview,String>{
    let c=db.conn.lock().map_err(error)?;
    finance_overview_core(&c,year)
}
/// Price of an artwork in cents. A fixed price always wins; otherwise width × height × rate per cm²
/// × coefficient, where the coefficient is the artwork's own or, when it has none, the artist's.
/// `None` when the size or an imported price parameter is invalid.
/// Never replace a corrupted fixed price with a plausible computed estimate.
pub fn price_cents(width_cm:Option<f64>,height_cm:Option<f64>,rate_cm2_cents:i64,artist_coefficient:f64,own_coefficient:Option<f64>,fixed_price_cents:Option<i64>)->Option<i64>{
    // Match save_artwork's fixed-price limits, including the valid zero price.
    // A fixed price takes precedence even if the other inputs are invalid.
    if let Some(fixed)=fixed_price_cents {
        return (0..=100_000_000_000).contains(&fixed).then_some(fixed);
    }
    let (width,height)=(width_cm?,height_cm?);
    let coefficient=own_coefficient.unwrap_or(artist_coefficient);
    // Legacy/imported SQLite data can bypass the settings/form validators.
    // In that case show no price rather than a negative or misleading estimate.
    if !(0..=100_000).contains(&rate_cm2_cents)
        || !coefficient.is_finite() || coefficient<=0.0 || coefficient>100_000.0
        || !width.is_finite() || !height.is_finite()
        || width<=0.0 || height<=0.0 || width>100000.0 || height>100000.0 {return None}
    let cents=width*height*rate_cm2_cents as f64*coefficient;
    if !cents.is_finite() || cents<0.0 || cents>100_000_000_000.0 {return None}
    Some(cents.round() as i64)
}
#[cfg(test)]
mod tests{
    use super::*;
    #[test]
    fn price_follows_fixed_price_then_own_coefficient_then_artist_coefficient(){
        // 30 x 40 cm at 0,16 EUR/cm2 with coefficient 1 = 192 EUR
        assert_eq!(price_cents(Some(30.0),Some(40.0),16,1.0,None,None),Some(19_200));
        assert_eq!(price_cents(Some(30.0),Some(40.0),16,2.0,None,None),Some(38_400),"the artist's coefficient");
        assert_eq!(price_cents(Some(30.0),Some(40.0),16,2.0,Some(1.5),None),Some(28_800),"the artwork's own coefficient wins");
        assert_eq!(price_cents(Some(30.0),Some(40.0),16,2.0,Some(1.5),Some(50_000)),Some(50_000),"a fixed price wins over everything");
        assert_eq!(price_cents(None,None,16,2.0,Some(1.5),Some(50_000)),Some(50_000),"a fixed price needs no size");
        assert_eq!(price_cents(Some(90.0),Some(120.0),16,1.25,None,None),Some(216_000));
        for (w,h) in [(None,Some(10.0)),(Some(0.0),Some(10.0)),(Some(-1.0),Some(10.0)),(Some(f64::NAN),Some(10.0)),(Some(1e6),Some(10.0))] {
            assert_eq!(price_cents(w,h,16,1.0,None,None),None,"{w:?} {h:?}");
        }
        assert_eq!(price_cents(Some(1e5),Some(1e5),100000,1.0,None,None),None,"absurd totals are refused");
    }
    #[test]
    fn imported_invalid_prices_never_create_misleading_estimates() {
        let normal = || price_cents(Some(30.0), Some(40.0), 16, 2.0, None, None);
        assert_eq!(normal(), Some(38_400));
        // Invalid stored fixed cents must not fall back to the calculated price.
        for fixed in [-1, i64::MIN, 100_000_000_001, i64::MAX] {
            assert_eq!(price_cents(Some(30.0), Some(40.0), 16, 2.0, None, Some(fixed)), None, "{fixed}");
        }
        assert_eq!(price_cents(None, None, -1, f64::NAN, None, Some(0)), Some(0));
        assert_eq!(price_cents(None, None, -1, f64::NAN, None, Some(100_000_000_000)), Some(100_000_000_000));
        // Invalid rate/coefficient from an imported archive must not appear as a real quote.
        for rate in [-1, 100_001, i64::MAX] {
            assert_eq!(price_cents(Some(30.0), Some(40.0), rate, 2.0, None, None), None, "{rate}");
        }
        for coefficient in [0.0, -1.0, 100_000.1, f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
            assert_eq!(price_cents(Some(30.0), Some(40.0), 16, coefficient, None, None), None, "{coefficient}");
            assert_eq!(price_cents(Some(30.0), Some(40.0), 16, 2.0, Some(coefficient), None), None, "{coefficient}");
        }
        // An explicit valid artwork override takes precedence over a damaged studio default.
        assert_eq!(price_cents(Some(30.0), Some(40.0), 16, f64::NAN, Some(1.5), None), Some(28_800));
        assert_eq!(price_cents(Some(30.0), Some(40.0), 0, 1.0, None, None), Some(0));
    }
    #[test]
    fn ignored_studio_settings_write_is_reported_and_does_not_claim_success() {
        let dir = tempfile::tempdir().unwrap();
        let db = crate::db::open(dir.path().to_owned()).unwrap();
        let before = settings_tx(&db.conn.lock().unwrap()).unwrap();
        let mut wanted = before.clone();
        wanted.default_tax_bp = 850;
        wanted.artist_name = Some("Autrice del certificato".into());
        db.conn.lock().unwrap().execute_batch(
            "CREATE TRIGGER skip_studio_settings BEFORE UPDATE ON studio_settings
             BEGIN SELECT RAISE(IGNORE); END;"
        ).unwrap();
        let message = save_studio_settings_core(&db, wanted.clone()).unwrap_err();
        assert!(message.contains("did not save studio settings"), "{message}");
        assert_eq!(settings_tx(&db.conn.lock().unwrap()).unwrap(), before);
        db.conn.lock().unwrap().execute_batch("DROP TRIGGER skip_studio_settings").unwrap();
        let saved = save_studio_settings_core(&db, wanted.clone()).unwrap();
        assert_eq!(saved, wanted);
        assert_eq!(settings_tx(&db.conn.lock().unwrap()).unwrap(), wanted);
    }

    #[test]
    fn tampered_certificate_identity_rolls_back_all_studio_settings() {
        let dir = tempfile::tempdir().unwrap();
        let db = crate::db::open(dir.path().to_owned()).unwrap();
        let before = settings_tx(&db.conn.lock().unwrap()).unwrap();
        let mut wanted = before.clone();
        wanted.artist_name = Some("Nome certificante".into());
        wanted.artist_alias = Some("  Firma artistica  ".into());
        wanted.default_tax_bp = 1200;
        db.conn.lock().unwrap().execute_batch(
            "CREATE TRIGGER change_studio_identity AFTER UPDATE ON studio_settings
             BEGIN UPDATE studio_settings SET artist_alias='Sostituito'
                   WHERE singleton=1; END;"
        ).unwrap();
        let message = save_studio_settings_core(&db, wanted.clone()).unwrap_err();
        assert!(message.contains("did not preserve studio settings"), "{message}");
        assert_eq!(settings_tx(&db.conn.lock().unwrap()).unwrap(), before,
            "even unrelated tax and artist-name changes must be rolled back");
        db.conn.lock().unwrap().execute_batch("DROP TRIGGER change_studio_identity").unwrap();
        let saved = save_studio_settings_core(&db, wanted).unwrap();
        assert_eq!(saved.artist_name.as_deref(), Some("Nome certificante"));
        assert_eq!(saved.artist_alias.as_deref(), Some("Firma artistica"));
        assert_eq!(saved.default_tax_bp, 1200);
    }

    #[test]
    fn settings_validate_exact_percentages(){
        let good=StudioSettings{artist_coefficient:1.0,base_price_cm2_cents:16,personal_bp:3334,activity_bp:3333,savings_bp:3333,
            treasury_start_date:None,default_tax_bp:0,ui_language:None,artist_name:None,artist_alias:None};
        assert!(valid_settings(&good).is_ok());
        let mut bad=good.clone();bad.savings_bp=3332;
        assert!(valid_settings(&bad).is_err());
        bad=good.clone();bad.artist_coefficient=f64::NAN;
        assert!(valid_settings(&bad).is_err());
    }
    #[test]
    fn expense_update_rejects_silently_ignored_sqlite_writes() {
        let dir = tempfile::tempdir().unwrap();
        let db = crate::db::open(dir.path().to_owned()).unwrap();
        let input = |id: Option<String>, description: &str| ExpenseInput {
            id, expense_date: "2026-11-20".into(), category: "materials".into(),
            description: description.into(), amount_cents: 3500,
            fund: None, supplier: None, purchase_url: None, notes: None,
        };
        let original = save_expense_core(&db, input(None, "Canvas")).unwrap();
        {
            let conn = db.conn.lock().unwrap();
            conn.execute_batch("CREATE TRIGGER ignore_expense_edit BEFORE UPDATE ON expenses
                BEGIN SELECT RAISE(IGNORE); END;").unwrap();
        }
        let message = save_expense_core(&db, input(Some(original.id.clone()), "Paint"))
            .expect_err("ignored update cannot be reported as a successful save");
        assert!(message.contains("not updated"), "{message}");
        {
            let conn = db.conn.lock().unwrap();
            let description: String = conn.query_row(
                "SELECT description FROM expenses WHERE id=?1",
                params![&original.id], |r| r.get(0)).unwrap();
            assert_eq!(description, "Canvas", "the original expense must remain intact");
            conn.execute_batch("DROP TRIGGER ignore_expense_edit").unwrap();
        }
        let updated = save_expense_core(&db, input(Some(original.id), "Paint")).unwrap();
        assert_eq!(updated.description, "Paint", "editing works after the block is removed");
    }

    #[test]
    fn expense_insert_and_update_tampering_roll_back_atomically() {
        let input=|id:Option<String>,description:&str|ExpenseInput{
            id,expense_date:"2026-11-22".into(),category:"materials".into(),
            description:description.into(),amount_cents:3500,
            fund:None,supplier:Some("Art shop".into()),purchase_url:None,notes:None,
        };
        for trigger in [
            "CREATE TRIGGER sabotage AFTER INSERT ON expenses BEGIN
                 UPDATE expenses SET amount_cents=1 WHERE id=NEW.id; END;",
            "CREATE TRIGGER sabotage AFTER INSERT ON expenses BEGIN
                 DELETE FROM expenses WHERE id=NEW.id; END;",
        ] {
            let dir=tempfile::tempdir().unwrap();
            let db=crate::db::open(dir.path().to_owned()).unwrap();
            db.conn.lock().unwrap().execute_batch(trigger).unwrap();
            let message=save_expense_core(&db,input(None,"Canvas"))
                .expect_err("altered expense inserts must fail atomically");
            assert!(message.contains("preserve"),"{message}");
            let c=db.conn.lock().unwrap();
            let count:i64=c.query_row("SELECT count(*) FROM expenses",[],|r|r.get(0)).unwrap();
            assert_eq!(count,0,"failed insert must roll back completely");
            c.execute_batch("DROP TRIGGER sabotage").unwrap();
            drop(c);
            assert_eq!(save_expense_core(&db,input(None,"Canvas")).unwrap().amount_cents,3500);
        }
        for trigger in [
            "CREATE TRIGGER sabotage AFTER UPDATE ON expenses BEGIN
                 UPDATE expenses SET amount_cents=1 WHERE id=NEW.id; END;",
            "CREATE TRIGGER sabotage AFTER UPDATE ON expenses BEGIN
                 DELETE FROM expenses WHERE id=NEW.id; END;",
        ] {
            let dir=tempfile::tempdir().unwrap();
            let db=crate::db::open(dir.path().to_owned()).unwrap();
            let original=save_expense_core(&db,input(None,"Canvas")).unwrap();
            db.conn.lock().unwrap().execute_batch(trigger).unwrap();
            let message=save_expense_core(&db,input(Some(original.id.clone()),"Paint"))
                .expect_err("altered expense updates must fail atomically");
            assert!(message.contains("preserve"),"{message}");
            let c=db.conn.lock().unwrap();
            let saved:Expense=c.query_row(
                "SELECT id,expense_date,category,description,amount_cents,fund,supplier,purchase_url,notes,created_at,updated_at FROM expenses WHERE id=?1",
                params![&original.id],expense_row).unwrap();
            assert_eq!(saved.description,"Canvas");
            assert_eq!(saved.amount_cents,3500);
            assert_eq!(saved.updated_at,original.updated_at);
            c.execute_batch("DROP TRIGGER sabotage").unwrap();
            drop(c);
            assert_eq!(save_expense_core(&db,input(Some(original.id),"Paint")).unwrap().description,"Paint");
        }
    }

    #[test]
    fn expense_delete_rejects_silent_resurrection_and_preserves_original() {
        let dir=tempfile::tempdir().unwrap();
        let db=crate::db::open(dir.path().to_owned()).unwrap();
        let original=save_expense_core(&db,ExpenseInput{
            id:None,expense_date:"2026-11-22".into(),category:"materials".into(),
            description:"Canvas".into(),amount_cents:3500,
            fund:None,supplier:None,purchase_url:None,notes:None,
        }).unwrap();
        let id=original.id.clone();
        {
            let c=db.conn.lock().unwrap();
            c.execute_batch("CREATE TRIGGER resurrect_expense AFTER DELETE ON expenses BEGIN
                INSERT INTO expenses(id,expense_date,category,description,amount_cents,fund,supplier,purchase_url,notes,created_at,updated_at)
                VALUES(OLD.id,OLD.expense_date,OLD.category,OLD.description,OLD.amount_cents,OLD.fund,
                       OLD.supplier,OLD.purchase_url,OLD.notes,OLD.created_at,OLD.updated_at);
                END;").unwrap();
        }
        assert!(delete_expense_core(&db,&id).unwrap_err().contains("did not remove"));
        {
            let c=db.conn.lock().unwrap();
            let row:i64=c.query_row("SELECT amount_cents FROM expenses WHERE id=?1",
                params![&id],|r|r.get(0)).unwrap();
            assert_eq!(row,3500,"failed deletion must not remove the expense");
            c.execute_batch("DROP TRIGGER resurrect_expense").unwrap();
        }
        delete_expense_core(&db,&id).unwrap();
        let c=db.conn.lock().unwrap();
        let count:i64=c.query_row("SELECT count(*) FROM expenses WHERE id=?1",
            params![&id],|r|r.get(0)).unwrap();
        assert_eq!(count,0);
    }

    #[test]
    fn sale_financial_details_and_frozen_shares_must_survive_sqlite_triggers() {
        // A valid SQLite page / foreign-key check cannot detect a silently
        // ignored row or an AFTER trigger deleting/changing a just-written row.
        for (trigger, expected_error) in [
            ("CREATE TRIGGER sabotage BEFORE INSERT ON sale_finances
              BEGIN SELECT RAISE(IGNORE); END;", "sale financial details"),
            ("CREATE TRIGGER sabotage AFTER INSERT ON sale_finances
              BEGIN DELETE FROM sale_finances WHERE sale_id=NEW.sale_id; END;", "sale financial details"),
            ("CREATE TRIGGER sabotage AFTER INSERT ON sale_finances
              BEGIN UPDATE sale_finances SET tax_cents=0 WHERE sale_id=NEW.sale_id; END;", "sale financial details"),
            ("CREATE TRIGGER sabotage BEFORE INSERT ON sale_allocations
              BEGIN SELECT RAISE(IGNORE); END;", "frozen sale allocations"),
            ("CREATE TRIGGER sabotage AFTER INSERT ON sale_allocations
              BEGIN DELETE FROM sale_allocations WHERE sale_id=NEW.sale_id AND part_id=NEW.part_id; END;", "frozen sale allocations"),
        ] {
            let directory=tempfile::tempdir().unwrap();
            let db=crate::db::open(directory.path().to_owned()).unwrap();
            let mut conn=db.conn.lock().unwrap();
            conn.execute("INSERT INTO artworks(id,code,title,created_at,updated_at)
                VALUES('a','ART-1','Opera','t','t')",[]).unwrap();
            conn.execute("INSERT INTO collectors(id,name,created_at,updated_at)
                VALUES('c','Cliente','t','t')",[]).unwrap();
            conn.execute("INSERT INTO sales(id,artwork_id,collector_id,amount_cents,sold_at,created_at)
                VALUES('s','a','c',10000,'2026-11-22','t')",[]).unwrap();
            conn.execute_batch(trigger).unwrap();

            let tx=conn.transaction().unwrap();
            let message=attach_sale_finances(&tx,"s",10000,1200,300)
                .expect_err("silently corrupted financial rows must prevent sale commit");
            assert!(message.contains(expected_error), "{message}");
            tx.rollback().unwrap();
            for table in ["sale_finances","sale_allocations"] {
                let count:i64=conn.query_row(
                    &format!("SELECT count(*) FROM {table} WHERE sale_id='s'"),
                    [],|r|r.get(0)).unwrap();
                assert_eq!(count,0,"failed sale must leave no partial {table}");
            }

            conn.execute_batch("DROP TRIGGER sabotage").unwrap();
            let tx=conn.transaction().unwrap();
            attach_sale_finances(&tx,"s",10000,1200,300).unwrap();
            tx.commit().unwrap();
            let saved:(i64,i64)=conn.query_row(
                "SELECT tax_cents,shipping_cents FROM sale_finances WHERE sale_id='s'",
                [],|r|Ok((r.get(0)?,r.get(1)?))).unwrap();
            assert_eq!(saved,(1200,300));
            let shares:i64=conn.query_row(
                "SELECT sum(share_bp) FROM sale_allocations WHERE sale_id='s'",
                [],|r|r.get(0)).unwrap();
            assert_eq!(shares,10000);
        }
    }

    #[test]
    fn monthly_sales_and_expenses_are_separate(){
        let dir=tempfile::tempdir().unwrap();
        let db=crate::db::open(dir.path().to_owned()).unwrap();
        let mut c=db.conn.lock().unwrap();
        c.execute("INSERT INTO artworks(id,code,title,status,created_at,updated_at) VALUES('a','a','a','sold','now','now')",[]).unwrap();
        c.execute("INSERT INTO collectors(id,name,created_at,updated_at) VALUES('c','Collector','now','now')",[]).unwrap();
        c.execute("INSERT INTO sales(id,artwork_id,collector_id,amount_cents,sold_at,created_at) VALUES('s','a','c',10000,'2026-02-15','now')",[]).unwrap();
        let tx=c.transaction().unwrap();
        attach_sale_finances(&tx,"s",10000,1200,800).unwrap();
        tx.commit().unwrap();
        c.execute("INSERT INTO expenses(id,expense_date,category,description,amount_cents,fund,created_at,updated_at) VALUES('e','2026-02-20','materials','Paint',1000,'activity','now','now')",[]).unwrap();
        let result=finance_overview_core(&c,2026).unwrap();
        assert_eq!(result.gross_cents,10000);
        assert_eq!(result.net_cents,8000);
        assert_eq!(result.expense_cents,1000);
        assert_eq!(result.margin_cents,7000);
        assert_eq!(result.months[1].gross_cents,10000);
        assert_eq!(result.months[0].gross_cents,0);
    }
    fn sale(c:&mut rusqlite::Connection,id:&str,amount:i64,date:&str,taxes:i64,shipping:i64){
        c.execute("INSERT INTO artworks(id,code,title,status,created_at,updated_at) VALUES(?1,?1,?1,'sold','t','t')",params![id]).unwrap();
        c.execute("INSERT OR IGNORE INTO collectors(id,name,created_at,updated_at) VALUES('c','Collector','t','t')",[]).unwrap();
        c.execute("INSERT INTO sales(id,artwork_id,collector_id,amount_cents,sold_at,created_at) VALUES(?1,?1,'c',?2,?3,'t')",params![id,amount,date]).unwrap();
        let tx=c.transaction().unwrap();
        attach_sale_finances(&tx,id,amount,taxes,shipping).unwrap();
        tx.commit().unwrap();
    }
    #[test]
    fn default_plan_reproduces_the_historical_three_way_split(){
        let dir=tempfile::tempdir().unwrap();
        let db=crate::db::open(dir.path().to_owned()).unwrap();
        let mut c=db.conn.lock().unwrap();
        sale(&mut c,"s1",10_001,"2026-03-05",0,0);
        c.execute("INSERT INTO expenses(id,expense_date,category,description,amount_cents,fund,created_at,updated_at) VALUES('e','2026-03-06','m','Colori',500,'activity','t','t')",[]).unwrap();
        let o=finance_overview_core(&c,2026).unwrap();
        let by_id=|id:&str|o.parts.iter().find(|p|p.part_id==id).unwrap();
        assert_eq!((by_id("personal").share_cents,by_id("activity").share_cents,by_id("savings").share_cents),(3334,3333,3334));
        assert_eq!(by_id("activity").balance_cents,3333-500);
        assert_eq!(o.parts.iter().map(|p|p.share_cents).sum::<i64>(),o.net_cents);
        assert!(o.allocation_enabled);
    }
    #[test]
    fn a_custom_plan_applies_to_new_sales_while_old_sales_keep_their_frozen_shares(){
        let dir=tempfile::tempdir().unwrap();
        let db=crate::db::open(dir.path().to_owned()).unwrap();
        {let mut c=db.conn.lock().unwrap();sale(&mut c,"old",10_000,"2026-01-10",0,0);}
        let plan=crate::allocations::save_plan_core(&db,crate::allocations::PlanInput{enabled:true,parts:vec![
            crate::allocations::PartInput{id:Some("personal".into()),name:"Io".into(),share_bp:7000,pays_expenses:false},
            crate::allocations::PartInput{id:None,name:"Materiali".into(),share_bp:3000,pays_expenses:true}]}).unwrap();
        assert_eq!(plan.parts.len(),2);
        let mut c=db.conn.lock().unwrap();
        sale(&mut c,"new",10_000,"2026-02-10",0,0);
        let o=finance_overview_core(&c,2026).unwrap();
        let by_id=|id:&str|o.parts.iter().find(|p|p.part_id==id).unwrap();
        // Old sale: 3334/3333/3333 kept; new sale: 7000/3000 on the new plan.
        assert_eq!(by_id("personal").share_cents,3334+7000);
        assert_eq!(by_id("activity").share_cents,3333);
        assert_eq!(by_id("savings").share_cents,3333);
        assert!(by_id("activity").pays_expenses);
        let materials=o.parts.iter().find(|p|p.name=="Materiali").unwrap();
        assert_eq!(materials.share_cents,3000);
        assert!(!by_id("activity").active && !by_id("savings").active,"historical parts remain visible but inactive");
        assert_eq!(o.parts.iter().map(|p|p.share_cents).sum::<i64>(),o.net_cents);
        let new_part=materials.part_id.clone();
        c.execute("INSERT INTO expenses(id,expense_date,category,description,amount_cents,fund,created_at,updated_at) VALUES('e','2026-02-11','m','Telai',400,?1,'t','t')",params![new_part]).unwrap();
        let o=finance_overview_core(&c,2026).unwrap();
        assert_eq!(o.parts.iter().find(|p|p.name=="Materiali").unwrap().balance_cents,3000-400);
    }
    #[test]
    fn with_the_split_switched_off_sales_have_no_shares_and_expenses_need_no_fund(){
        let dir=tempfile::tempdir().unwrap();
        let db=crate::db::open(dir.path().to_owned()).unwrap();
        let kept:Vec<_>=crate::allocations::load_plan(&db.conn.lock().unwrap()).unwrap().parts.into_iter().map(|p|
            crate::allocations::PartInput{id:Some(p.id),name:p.name,share_bp:p.share_bp,pays_expenses:p.pays_expenses}).collect();
        crate::allocations::save_plan_core(&db,crate::allocations::PlanInput{enabled:false,parts:kept}).unwrap();
        let mut c=db.conn.lock().unwrap();
        sale(&mut c,"s",20_000,"2026-05-05",1_000,500);
        let rows:i64=c.query_row("SELECT count(*) FROM sale_allocations WHERE sale_id='s'",[],|r|r.get(0)).unwrap();
        assert_eq!(rows,0);
        c.execute("INSERT INTO expenses(id,expense_date,category,description,amount_cents,fund,created_at,updated_at) VALUES('e','2026-05-06','m','Cornici',700,NULL,'t','t')",[]).unwrap();
        let o=finance_overview_core(&c,2026).unwrap();
        assert!(!o.allocation_enabled);
        assert_eq!((o.gross_cents,o.net_cents,o.expense_cents,o.margin_cents),(20_000,18_500,700,17_800));
        assert!(o.parts.iter().all(|p|p.share_cents==0&&p.expense_cents==0));
    }

    #[test]
    fn imported_sales_with_invalid_dates_deductions_or_shares_are_not_silently_reported(){
        let dir=tempfile::tempdir().unwrap();
        let db=crate::db::open(dir.path().to_owned()).unwrap();
        let mut c=db.conn.lock().unwrap();
        sale(&mut c,"s",10_000,"2026-02-15",0,0);
        c.execute("UPDATE sales SET sold_at='2026-02-31' WHERE id='s'",[]).unwrap();
        assert!(finance_overview_core(&c,2026).err().unwrap().contains("Invalid sale date"));
        c.execute("UPDATE sales SET sold_at='2026-02-15' WHERE id='s'",[]).unwrap();
        c.execute("UPDATE sale_finances SET tax_cents=10001 WHERE sale_id='s'",[]).unwrap();
        assert!(finance_overview_core(&c,2026).err().unwrap().contains("deductions exceed"));
        c.execute("UPDATE sale_finances SET tax_cents=0 WHERE sale_id='s'",[]).unwrap();
        c.execute("UPDATE sale_allocations SET share_bp=3332 WHERE sale_id='s' AND part_id='savings'",[]).unwrap();
        assert!(finance_overview_core(&c,2026).err().unwrap().contains("frozen allocation"));
    }
    #[test]
    fn financial_report_rejects_overflow_in_imported_sale_totals(){
        let dir=tempfile::tempdir().unwrap();
        let db=crate::db::open(dir.path().to_owned()).unwrap();
        let c=db.conn.lock().unwrap();
        c.execute("INSERT INTO collectors(id,name,created_at,updated_at) VALUES('c','Collector','t','t')",[]).unwrap();
        for id in ["a","b"] {
            c.execute("INSERT INTO artworks(id,code,title,created_at,updated_at) VALUES(?1,?1,?1,'t','t')",params![id]).unwrap();
            c.execute("INSERT INTO sales(id,artwork_id,collector_id,amount_cents,sold_at,created_at)
                VALUES(?1,?1,'c',?2,'2026-03-01','t')",params![id,i64::MAX]).unwrap();
            c.execute("INSERT INTO sale_finances(sale_id,tax_cents,shipping_cents,personal_bp,activity_bp,savings_bp)
                VALUES(?1,0,0,3334,3333,3333)",params![id]).unwrap();
        }
        assert!(finance_overview_core(&c,2026).err().unwrap().contains("overflow"));
    }
    #[test]
    fn imported_sale_without_financial_details_is_not_reported_with_fake_zero_deductions(){
        let dir=tempfile::tempdir().unwrap();
        let db=crate::db::open(dir.path().to_owned()).unwrap();
        let mut c=db.conn.lock().unwrap();
        sale(&mut c,"s",10_000,"2026-06-12",1_200,300);
        assert_eq!(finance_overview_core(&c,2026).unwrap().net_cents,8_500);

        // An external editor can delete the financial row with foreign keys
        // enabled; normal SQLite referential integrity alone does not detect it.
        c.execute("DELETE FROM sale_finances WHERE sale_id='s'",[]).unwrap();
        let message=finance_overview_core(&c,2026).err().expect("missing details must fail");
        assert!(message.contains("missing financial details"),"{message}");
        assert!(message.contains("s"),"{message}");

        // Repairing the row restores the report without rewriting the sale.
        c.execute("INSERT INTO sale_finances(sale_id,tax_cents,shipping_cents,personal_bp,activity_bp,savings_bp)
            VALUES('s',1200,300,3334,3333,3333)",[]).unwrap();
        assert_eq!(finance_overview_core(&c,2026).unwrap().net_cents,8_500);
    }
    #[test]
    fn financial_report_rejects_invalid_imported_expense_dates(){
        let dir=tempfile::tempdir().unwrap();
        let db=crate::db::open(dir.path().to_owned()).unwrap();
        let c=db.conn.lock().unwrap();
        c.execute("INSERT INTO expenses(id,expense_date,category,description,amount_cents,created_at,updated_at)
            VALUES('e','2026-02-31','materials','Paint',500,'t','t')",[]).unwrap();
        assert!(finance_overview_core(&c,2026).err().unwrap().contains("Invalid expense date"));
    }
    #[test]
    fn imported_orphaned_allocation_and_expense_funds_are_not_silently_ignored(){
        let dir=tempfile::tempdir().unwrap();
        let db=crate::db::open(dir.path().to_owned()).unwrap();
        let mut c=db.conn.lock().unwrap();
        sale(&mut c,"s",10_000,"2026-04-11",0,0);
        assert_eq!(finance_overview_core(&c,2026).unwrap().net_cents,10_000);

        // Simulate an externally edited SQLite archive with foreign keys disabled.
        // Frozen shares still sum to 100%, so aggregate-only checks miss this.
        c.execute_batch("PRAGMA foreign_keys=OFF").unwrap();
        c.execute("UPDATE sale_allocations SET part_id='missing-part'
            WHERE sale_id='s' AND part_id='savings'",[]).unwrap();
        c.execute_batch("PRAGMA foreign_keys=ON").unwrap();
        let message=finance_overview_core(&c,2026).err().expect("corrupt archive must fail");
        assert!(message.contains("Unknown allocation part"), "{message}");
        assert!(message.contains("missing-part"), "{message}");

        c.execute_batch("PRAGMA foreign_keys=OFF").unwrap();
        c.execute("UPDATE sale_allocations SET part_id='savings'
            WHERE sale_id='s' AND part_id='missing-part'",[]).unwrap();
        c.execute("INSERT INTO expenses
            (id,expense_date,category,description,amount_cents,fund,created_at,updated_at)
            VALUES('e','2026-04-12','materials','Paint',500,'missing-fund','t','t')",[]).unwrap();
        c.execute_batch("PRAGMA foreign_keys=ON").unwrap();
        let message=finance_overview_core(&c,2026).err().expect("corrupt archive must fail");
        assert!(message.contains("Unknown expense fund"), "{message}");
        assert!(message.contains("missing-fund"), "{message}");

        // An intentionally unallocated expense is valid and remains in the total.
        c.execute("UPDATE expenses SET fund=NULL WHERE id='e'",[]).unwrap();
        let overview=finance_overview_core(&c,2026).unwrap();
        assert_eq!((overview.net_cents,overview.expense_cents,overview.margin_cents),
            (10_000,500,9_500));
        assert_eq!(overview.parts.iter().map(|p|p.expense_cents).sum::<i64>(),0);
    }

    #[test]
    fn imported_sales_with_missing_artwork_or_collector_cannot_inflate_financial_reports(){
        let dir=tempfile::tempdir().unwrap();
        let db=crate::db::open(dir.path().to_owned()).unwrap();
        let mut c=db.conn.lock().unwrap();
        sale(&mut c,"s",10_000,"2026-04-11",0,0);
        assert_eq!(finance_overview_core(&c,2026).unwrap().gross_cents,10_000);

        // A third-party editor can bypass foreign keys without breaking SQLite
        // page integrity. Do not count those records as valid studio turnover.
        c.execute_batch("PRAGMA foreign_keys=OFF").unwrap();
        c.execute("UPDATE sales SET artwork_id='removed-artwork' WHERE id='s'",[]).unwrap();
        c.execute_batch("PRAGMA foreign_keys=ON").unwrap();
        let message=finance_overview_core(&c,2026).err().expect("orphaned artwork must fail");
        assert!(message.contains("missing artwork"), "{message}");
        assert!(message.contains("s"), "{message}");

        c.execute_batch("PRAGMA foreign_keys=OFF").unwrap();
        c.execute("UPDATE sales SET artwork_id='s',collector_id='removed-collector' WHERE id='s'",[]).unwrap();
        c.execute_batch("PRAGMA foreign_keys=ON").unwrap();
        let message=finance_overview_core(&c,2026).err().expect("orphaned collector must fail");
        assert!(message.contains("missing collector"), "{message}");

        // Repairing the references restores the original report; no records
        // are written or removed by the reporting path.
        c.execute_batch("PRAGMA foreign_keys=OFF").unwrap();
        c.execute("UPDATE sales SET collector_id='c' WHERE id='s'",[]).unwrap();
        c.execute_batch("PRAGMA foreign_keys=ON").unwrap();
        let overview=finance_overview_core(&c,2026).unwrap();
        assert_eq!((overview.gross_cents,overview.net_cents),(10_000,10_000));
    }

    #[test]
    fn migration_moves_historical_shares_and_expense_funds_into_the_new_model(){
        let dir=tempfile::tempdir().unwrap();
        let db=crate::db::open(dir.path().to_owned()).unwrap();
        {
            let mut c=db.conn.lock().unwrap();
            sale(&mut c,"s",9_000,"2026-02-11",0,0);
            c.execute("UPDATE sale_finances SET personal_bp=5000,activity_bp=3000,savings_bp=2000 WHERE sale_id='s'",[]).unwrap();
            c.execute("DELETE FROM sale_allocations",[]).unwrap();
            c.execute("INSERT INTO expenses(id,expense_date,category,description,amount_cents,fund,created_at,updated_at) VALUES('e','2026-02-12','m','Carta',100,'savings','t','t')",[]).unwrap();
            c.execute_batch("CREATE TABLE expenses_old AS SELECT * FROM expenses; DROP TABLE expenses;
                DROP TABLE sale_allocations; DROP TABLE allocation_parts; DROP TABLE allocation_settings;
                CREATE TABLE expenses (id TEXT PRIMARY KEY, expense_date TEXT NOT NULL, category TEXT NOT NULL, description TEXT NOT NULL,
                    amount_cents INTEGER NOT NULL CHECK(amount_cents > 0), fund TEXT NOT NULL CHECK(fund IN ('activity','savings')),
                    supplier TEXT, purchase_url TEXT, notes TEXT, created_at TEXT NOT NULL, updated_at TEXT NOT NULL);
                INSERT INTO expenses SELECT * FROM expenses_old; DROP TABLE expenses_old;
                PRAGMA user_version=8;").unwrap();
        }
        drop(db);
        let upgraded=crate::db::open(dir.path().to_owned()).unwrap();
        let c=upgraded.conn.lock().unwrap();
        let shares:Vec<(String,i64)>=c.prepare("SELECT part_id,share_bp FROM sale_allocations WHERE sale_id='s' ORDER BY part_id").unwrap()
            .query_map([],|r|Ok((r.get(0)?,r.get(1)?))).unwrap().map(Result::unwrap).collect();
        assert_eq!(shares,vec![("activity".to_string(),3000),("personal".to_string(),5000),("savings".to_string(),2000)]);
        let fund:Option<String>=c.query_row("SELECT fund FROM expenses WHERE id='e'",[],|r|r.get(0)).unwrap();
        assert_eq!(fund.as_deref(),Some("savings"));
        assert!(c.execute("INSERT INTO expenses(id,expense_date,category,description,amount_cents,fund,created_at,updated_at) VALUES('x','2026-02-12','m','d',1,'inesistente','t','t')",[]).is_err());
        assert!(c.execute("INSERT INTO expenses(id,expense_date,category,description,amount_cents,fund,created_at,updated_at) VALUES('y','2026-02-12','m','d',1,NULL,'t','t')",[]).is_ok());
    }
}
