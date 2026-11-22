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
#[derive(Clone, Debug, Serialize, Deserialize)]
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
#[tauri::command]
pub fn save_studio_settings(db:State<'_,Database>,mut input:StudioSettings)->Result<StudioSettings,String>{
    valid_settings(&input)?;
    input.artist_name=crate::commands::clean(input.artist_name,240)?;
    input.artist_alias=crate::commands::clean(input.artist_alias,240)?;
    let c=db.conn.lock().map_err(error)?;
    c.execute("UPDATE studio_settings SET artist_coefficient=?1,base_price_cm2_cents=?2,personal_bp=?3,activity_bp=?4,savings_bp=?5,treasury_start_date=?6,default_tax_bp=?7,ui_language=?8,artist_name=?9,artist_alias=?10 WHERE singleton=1",
    params![input.artist_coefficient,input.base_price_cm2_cents,input.personal_bp,input.activity_bp,input.savings_bp,input.treasury_start_date,input.default_tax_bp,input.ui_language,input.artist_name,input.artist_alias]).map_err(error)?;
    settings_tx(&c)
}

pub fn attach_sale_finances(tx: &Transaction<'_>,sale_id: &str,gross:i64,taxes:i64,shipping:i64)->Result<(),String>{
    if gross<=0 || taxes<0 || shipping<0 || taxes.checked_add(shipping).is_none_or(|s|s>gross) {
        return Err("Taxes and shipping must not exceed the sale amount".into());
    }
    let s=settings_tx(tx)?;
    tx.execute("INSERT INTO sale_finances(sale_id,tax_cents,shipping_cents,personal_bp,activity_bp,savings_bp) VALUES(?1,?2,?3,?4,?5,?6)",
        params![sale_id,taxes,shipping,s.personal_bp,s.activity_bp,s.savings_bp]).map_err(error)?;
    // The shares in force today are frozen on this sale; later plan changes never rewrite it.
    for (part_id,bp) in crate::allocations::shares_for_new_sale(tx)? {
        tx.execute("INSERT INTO sale_allocations(sale_id,part_id,share_bp) VALUES(?1,?2,?3)",
            params![sale_id,part_id,bp]).map_err(error)?;
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
#[tauri::command]
pub fn save_expense(db:State<'_,Database>,input:ExpenseInput)->Result<Expense,String>{
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
    let c=db.conn.lock().map_err(error)?;
    // An expense is optionally charged to one of the parts that can pay expenses.
    let fund=input.fund.clone().map(|f|f.trim().to_string()).filter(|f|!f.is_empty());
    if let Some(fund)=&fund {
        let valid:bool=c.query_row("SELECT EXISTS(SELECT 1 FROM allocation_parts WHERE id=?1 AND pays_expenses=1)",
            params![fund],|r|r.get(0)).map_err(error)?;
        if !valid {return Err("Fondo di spesa non valido".into())}
    }
    let updating=input.id.is_some();
    let id=input.id.unwrap_or_else(||Uuid::new_v4().to_string());
    let exists:bool=c.query_row("SELECT EXISTS(SELECT 1 FROM expenses WHERE id=?1)",params![id],|r|r.get(0)).map_err(error)?;
    let now=Utc::now().to_rfc3339();
    if exists {
        c.execute("UPDATE expenses SET expense_date=?2,category=?3,description=?4,amount_cents=?5,fund=?6,supplier=?7,purchase_url=?8,notes=?9,updated_at=?10 WHERE id=?1",
            params![id,input.expense_date,category,description,input.amount_cents,fund,supplier,purchase_url,notes,now]).map_err(error)?;
    }else{
        if updating {return Err("Expense does not exist".into());}
        c.execute("INSERT INTO expenses(id,expense_date,category,description,amount_cents,fund,supplier,purchase_url,notes,created_at,updated_at) VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?10)",
            params![id,input.expense_date,category,description,input.amount_cents,fund,supplier,purchase_url,notes,now]).map_err(error)?;
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
    let mut q=c.prepare("SELECT s.id,s.amount_cents,s.sold_at,COALESCE(sf.tax_cents,0),COALESCE(sf.shipping_cents,0),
            a.id,co.id
        FROM sales s
        LEFT JOIN sale_finances sf ON sf.sale_id=s.id
        LEFT JOIN artworks a ON a.id=s.artwork_id
        LEFT JOIN collectors co ON co.id=s.collector_id
        WHERE substr(s.sold_at,1,4)=?1").map_err(error)?;
    let rows=q.query_map(params![year.to_string()],|r| Ok((r.get::<_,String>(0)?,r.get::<_,i64>(1)?,r.get::<_,String>(2)?,
        r.get::<_,i64>(3)?,r.get::<_,i64>(4)?,r.get::<_,Option<String>>(5)?,r.get::<_,Option<String>>(6)?))).map_err(error)?
        .collect::<Result<Vec<_>,_>>().map_err(error)?;
    for (sale,gross,date,taxes,shipping,artwork,collector) in rows {
        if settings.treasury_start_date.as_ref().is_some_and(|start|date.as_str()<start.as_str()){continue;}
        if !date_ok(&date) {return Err(format!("Invalid sale date in archive: {sale}"));}
        if artwork.is_none() {return Err(format!("Archived sale has missing artwork: {sale}"));}
        if collector.is_none() {return Err(format!("Archived sale has missing collector: {sale}"));}
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
/// `None` when the size is missing or the result is not a sensible amount.
pub fn price_cents(width_cm:Option<f64>,height_cm:Option<f64>,rate_cm2_cents:i64,artist_coefficient:f64,own_coefficient:Option<f64>,fixed_price_cents:Option<i64>)->Option<i64>{
    if let Some(fixed)=fixed_price_cents {return Some(fixed)}
    let (width,height)=(width_cm?,height_cm?);
    if !width.is_finite() || !height.is_finite() || width<=0.0 || height<=0.0 || width>100000.0 || height>100000.0 {return None}
    let cents=width*height*rate_cm2_cents as f64*own_coefficient.unwrap_or(artist_coefficient);
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
        }
        assert!(finance_overview_core(&c,2026).err().unwrap().contains("overflow"));
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
