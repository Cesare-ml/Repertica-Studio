//! Local studio economics. Monetary values are integer EUR cents and share ratios
//! are integer basis points. Sales are recorded turnover, not verified cash receipts.
use crate::db::Database;
use chrono::{Datelike, NaiveDate, Utc};
use rusqlite::{params, OptionalExtension, Transaction};
use serde::{Deserialize, Serialize};
use tauri::State;
use uuid::Uuid;

fn error(e: impl std::fmt::Display) -> String { e.to_string() }
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
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StudioSettings {
    pub artist_coefficient: f64,
    pub base_price_cm2_cents: i64,
    pub personal_bp: i64,
    pub activity_bp: i64,
    pub savings_bp: i64,
    pub treasury_start_date: Option<String>,
    pub default_tax_bp: i64,
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
    if s.treasury_start_date.as_ref().is_some_and(|date| !date_ok(date)) {
        return Err("Invalid treasury start date".into());
    }
    Ok(())
}
fn settings_tx(c: &rusqlite::Connection) -> Result<StudioSettings,String> {
    c.query_row("SELECT artist_coefficient,base_price_cm2_cents,personal_bp,activity_bp,savings_bp,treasury_start_date,default_tax_bp FROM studio_settings WHERE singleton=1",
        [], |r|Ok(StudioSettings{
            artist_coefficient:r.get(0)?,base_price_cm2_cents:r.get(1)?,personal_bp:r.get(2)?,
            activity_bp:r.get(3)?,savings_bp:r.get(4)?,treasury_start_date:r.get(5)?,default_tax_bp:r.get(6)?
        })).map_err(error)
}
#[tauri::command]
pub fn get_studio_settings(db:State<'_,Database>) -> Result<StudioSettings,String> {
    let c=db.conn.lock().map_err(error)?;
    settings_tx(&c)
}
#[tauri::command]
pub fn save_studio_settings(db:State<'_,Database>,input:StudioSettings)->Result<StudioSettings,String>{
    valid_settings(&input)?;
    let c=db.conn.lock().map_err(error)?;
    c.execute("UPDATE studio_settings SET artist_coefficient=?1,base_price_cm2_cents=?2,personal_bp=?3,activity_bp=?4,savings_bp=?5,treasury_start_date=?6,default_tax_bp=?7 WHERE singleton=1",
    params![input.artist_coefficient,input.base_price_cm2_cents,input.personal_bp,input.activity_bp,input.savings_bp,input.treasury_start_date,input.default_tax_bp]).map_err(error)?;
    settings_tx(&c)
}

pub fn attach_sale_finances(tx: &Transaction<'_>,sale_id: &str,gross:i64,taxes:i64,shipping:i64)->Result<(),String>{
    if gross<=0 || taxes<0 || shipping<0 || taxes.checked_add(shipping).is_none_or(|s|s>gross) {
        return Err("Taxes and shipping must not exceed the sale amount".into());
    }
    let s=settings_tx(tx)?;
    tx.execute("INSERT INTO sale_finances(sale_id,tax_cents,shipping_cents,personal_bp,activity_bp,savings_bp) VALUES(?1,?2,?3,?4,?5,?6)",
        params![sale_id,taxes,shipping,s.personal_bp,s.activity_bp,s.savings_bp]).map_err(error)?;
    Ok(())
}
#[derive(Debug, Clone, Serialize)]
pub struct Expense {
    pub id:String,pub expense_date:String,pub category:String,pub description:String,
    pub amount_cents:i64,pub fund:String,pub supplier:Option<String>,pub purchase_url:Option<String>,
    pub notes:Option<String>,pub created_at:String,pub updated_at:String,
}
#[derive(Debug,Deserialize)]
pub struct ExpenseInput {
    pub id:Option<String>,pub expense_date:String,pub category:String,pub description:String,
    pub amount_cents:i64,pub fund:String,pub supplier:Option<String>,pub purchase_url:Option<String>,pub notes:Option<String>,
}
fn expense_row(r:&rusqlite::Row<'_>)->rusqlite::Result<Expense>{
    Ok(Expense{id:r.get(0)?,expense_date:r.get(1)?,category:r.get(2)?,description:r.get(3)?,
        amount_cents:r.get(4)?,fund:r.get(5)?,supplier:r.get(6)?,purchase_url:r.get(7)?,
        notes:r.get(8)?,created_at:r.get(9)?,updated_at:r.get(10)?})
}
#[tauri::command]
pub fn list_expenses(db:State<'_,Database>)->Result<Vec<Expense>,String>{
    let c=db.conn.lock().map_err(error)?;
    let mut q=c.prepare("SELECT id,expense_date,category,description,amount_cents,fund,supplier,purchase_url,notes,created_at,updated_at FROM expenses ORDER BY expense_date DESC,created_at DESC").map_err(error)?;
    let rows=q.query_map([],expense_row).map_err(error)?.collect::<Result<Vec<_>,_>>().map_err(error)?;
    Ok(rows)
}
#[tauri::command]
pub fn save_expense(db:State<'_,Database>,input:ExpenseInput)->Result<Expense,String>{
    if !date_ok(&input.expense_date) {return Err("Invalid expense date".into())}
    if input.amount_cents<=0 || input.amount_cents>100_000_000_000 {return Err("Invalid amount".into())}
    if input.fund!="activity" && input.fund!="savings" {return Err("Invalid fund".into())}
    let category=text(input.category,120)?;
    let description=text(input.description,1500)?;
    let supplier=opt_text(input.supplier,400)?;
    let purchase_url=opt_text(input.purchase_url,2000)?;
    if purchase_url.as_ref().is_some_and(|v|!v.starts_with("https://")&&!v.starts_with("http://")){
        return Err("Purchase link must use HTTPS or HTTP".into());
    }
    let notes=opt_text(input.notes,20000)?;
    let c=db.conn.lock().map_err(error)?;
    let id=input.id.unwrap_or_else(||Uuid::new_v4().to_string());
    let exists:bool=c.query_row("SELECT EXISTS(SELECT 1 FROM expenses WHERE id=?1)",params![id],|r|r.get(0)).map_err(error)?;
    let now=Utc::now().to_rfc3339();
    if exists {
        c.execute("UPDATE expenses SET expense_date=?2,category=?3,description=?4,amount_cents=?5,fund=?6,supplier=?7,purchase_url=?8,notes=?9,updated_at=?10 WHERE id=?1",
            params![id,input.expense_date,category,description,input.amount_cents,input.fund,supplier,purchase_url,notes,now]).map_err(error)?;
    }else{
        if input.id.is_some() {return Err("Expense does not exist".into());}
        c.execute("INSERT INTO expenses(id,expense_date,category,description,amount_cents,fund,supplier,purchase_url,notes,created_at,updated_at) VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?10)",
            params![id,input.expense_date,category,description,input.amount_cents,input.fund,supplier,purchase_url,notes,now]).map_err(error)?;
    }
    c.query_row("SELECT id,expense_date,category,description,amount_cents,fund,supplier,purchase_url,notes,created_at,updated_at FROM expenses WHERE id=?1",
        params![id],expense_row).map_err(error)
}
#[tauri::command]
pub fn delete_expense(db:State<'_,Database>,id:String)->Result<(),String>{
    let c=db.conn.lock().map_err(error)?;
    if c.execute("DELETE FROM expenses WHERE id=?1",params![id]).map_err(error)?==0 {return Err("Expense not found".into());}
    Ok(())
}
#[derive(Debug,Clone,Serialize)]
pub struct MonthlyFinance {
    pub month:String,pub gross_cents:i64,pub taxes_cents:i64,pub shipping_cents:i64,
    pub net_cents:i64,pub expense_cents:i64,pub margin_cents:i64,
    pub personal_cents:i64,pub activity_cents:i64,pub savings_cents:i64,
    pub activity_expenses_cents:i64,pub savings_expenses_cents:i64,
}
impl MonthlyFinance {
    fn new(month:String)->Self{
        Self{month,gross_cents:0,taxes_cents:0,shipping_cents:0,net_cents:0,expense_cents:0,margin_cents:0,
        personal_cents:0,activity_cents:0,savings_cents:0,activity_expenses_cents:0,savings_expenses_cents:0}
    }
}
#[derive(Serialize)]
pub struct FinanceOverview {
    pub year:i32,pub months:Vec<MonthlyFinance>,pub gross_cents:i64,pub net_cents:i64,
    pub expense_cents:i64,pub margin_cents:i64,pub personal_cents:i64,
    pub activity_balance_cents:i64,pub savings_balance_cents:i64,
}
fn allocate(net:i64,p:i64,a:i64)->(i64,i64,i64){
    let personal=(i128::from(net)*i128::from(p)/10000) as i64;
    let activity=(i128::from(net)*i128::from(a)/10000) as i64;
    (personal,activity,net-personal-activity)
}
fn finance_overview_core(c:&rusqlite::Connection,year:i32)->Result<FinanceOverview,String>{
    if !(1900..=2200).contains(&year) {return Err("Invalid year".into())}
    let settings=settings_tx(c)?;
    let mut months=(1..=12).map(|m|MonthlyFinance::new(format!("{year}-{m:02}"))).collect::<Vec<_>>();
    let mut q=c.prepare("SELECT s.amount_cents,s.sold_at,COALESCE(sf.tax_cents,0),COALESCE(sf.shipping_cents,0),
        COALESCE(sf.personal_bp,3334),COALESCE(sf.activity_bp,3333)
        FROM sales s LEFT JOIN sale_finances sf ON sf.sale_id=s.id WHERE substr(s.sold_at,1,4)=?1").map_err(error)?;
    let rows=q.query_map(params![year.to_string()],|r| Ok((r.get::<_,i64>(0)?,r.get::<_,String>(1)?,
        r.get::<_,i64>(2)?,r.get::<_,i64>(3)?,r.get::<_,i64>(4)?,r.get::<_,i64>(5)?))).map_err(error)?
        .collect::<Result<Vec<_>,_>>().map_err(error)?;
    for (gross,date,taxes,shipping,personal_bp,activity_bp) in rows {
        if settings.treasury_start_date.as_ref().is_some_and(|start|date.as_str()<start.as_str()){continue;}
        let Some(index)=date.get(5..7).and_then(|m|m.parse::<usize>().ok()).filter(|m|(1..=12).contains(m)).map(|m|m-1) else {continue};
        let month=&mut months[index];
        let net=gross-taxes-shipping;
        let (personal,activity,savings)=allocate(net,personal_bp,activity_bp);
        month.gross_cents+=gross;month.taxes_cents+=taxes;month.shipping_cents+=shipping;
        month.net_cents+=net;month.personal_cents+=personal;month.activity_cents+=activity;month.savings_cents+=savings;
    }
    let mut q=c.prepare("SELECT expense_date,amount_cents,fund FROM expenses WHERE substr(expense_date,1,4)=?1").map_err(error)?;
    let rows=q.query_map(params![year.to_string()],|r|Ok((r.get::<_,String>(0)?,r.get::<_,i64>(1)?,r.get::<_,String>(2)?)))
        .map_err(error)?.collect::<Result<Vec<_>,_>>().map_err(error)?;
    for (date,amount,fund) in rows {
        if settings.treasury_start_date.as_ref().is_some_and(|start|date.as_str()<start.as_str()){continue;}
        let Some(index)=date.get(5..7).and_then(|m|m.parse::<usize>().ok()).filter(|m|(1..=12).contains(m)).map(|m|m-1) else {continue};
        let month=&mut months[index];
        month.expense_cents+=amount;
        if fund=="savings" {month.savings_expenses_cents+=amount}
        else {month.activity_expenses_cents+=amount}
    }
    let mut summary=FinanceOverview{year,months,gross_cents:0,net_cents:0,expense_cents:0,margin_cents:0,
        personal_cents:0,activity_balance_cents:0,savings_balance_cents:0};
    for m in &mut summary.months {
        m.margin_cents=m.net_cents-m.expense_cents;
        summary.gross_cents+=m.gross_cents;
        summary.net_cents+=m.net_cents;
        summary.expense_cents+=m.expense_cents;
        summary.personal_cents+=m.personal_cents;
        summary.activity_balance_cents+=m.activity_cents-m.activity_expenses_cents;
        summary.savings_balance_cents+=m.savings_cents-m.savings_expenses_cents;
    }
    summary.margin_cents=summary.net_cents-summary.expense_cents;
    Ok(summary)
}
#[tauri::command]
pub fn finance_overview(db:State<'_,Database>,year:i32)->Result<FinanceOverview,String>{
    let c=db.conn.lock().map_err(error)?;
    finance_overview_core(&c,year)
}
#[tauri::command]
pub fn price_preview(db:State<'_,Database>,width_cm:f64,height_cm:f64)->Result<Option<i64>,String>{
    if !width_cm.is_finite() || !height_cm.is_finite() || width_cm<=0.0 || height_cm<=0.0 ||
        width_cm>100000.0 || height_cm>100000.0 {return Ok(None)}
    let c=db.conn.lock().map_err(error)?;
    let s=settings_tx(&c)?;
    let cents=width_cm*height_cm*s.base_price_cm2_cents as f64*s.artist_coefficient;
    if !cents.is_finite() || cents>100_000_000_000.0 {return Ok(None)}
    Ok(Some(cents.round() as i64))
}
#[cfg(test)]
mod tests{
    use super::*;
    #[test]
    fn settings_validate_exact_percentages(){
        let good=StudioSettings{artist_coefficient:1.0,base_price_cm2_cents:16,personal_bp:3334,activity_bp:3333,savings_bp:3333,
            treasury_start_date:None,default_tax_bp:0};
        assert!(valid_settings(&good).is_ok());
        let mut bad=good.clone();bad.savings_bp=3332;
        assert!(valid_settings(&bad).is_err());
        bad=good.clone();bad.artist_coefficient=f64::NAN;
        assert!(valid_settings(&bad).is_err());
    }
    #[test]
    fn allocates_exact_cents(){
        let (p,a,s)=allocate(10001,3334,3333);
        assert_eq!(p+a+s,10001);
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
}
