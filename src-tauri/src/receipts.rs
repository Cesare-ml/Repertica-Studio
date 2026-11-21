//! Explicit, local sale collections. These are manually reported, not bank-verified.
use crate::db::Database;
use chrono::{NaiveDate, Utc};
use rusqlite::{params, Connection, OptionalExtension, Row};
use serde::{Deserialize, Serialize};
use tauri::State;
use uuid::Uuid;

fn err(e: impl std::fmt::Display) -> String { e.to_string() }
fn valid_date(s:&str)->bool {
    NaiveDate::parse_from_str(s,"%Y-%m-%d")
        .map(|d|d.format("%Y-%m-%d").to_string()==s).unwrap_or(false)
}
fn clean(v:Option<String>,max:usize)->Result<Option<String>,String>{
    match v {None=>Ok(None),Some(s) if s.trim().is_empty()=>Ok(None),
        Some(s) if s.chars().count()<=max=>Ok(Some(s.trim().to_string())),
        Some(_)=>Err(format!("Text exceeds {max} characters"))}
}
#[derive(Debug,Clone,Serialize)]
pub struct SaleReceipt {
    pub id:String,pub sale_id:String,pub received_at:String,pub amount_cents:i64,
    pub method:String,pub reference:Option<String>,pub notes:Option<String>,pub created_at:String,
}
#[derive(Debug,Deserialize)]
pub struct SaleReceiptInput {
    pub sale_id:String,pub received_at:String,pub amount_cents:i64,
    pub method:String,pub reference:Option<String>,pub notes:Option<String>,
}
#[derive(Debug,Clone,Serialize)]
pub struct SaleBalance {
    pub sale_id:String,pub sale_cents:i64,pub received_cents:i64,pub outstanding_cents:i64,
}
const FIELDS:&str="id,sale_id,received_at,amount_cents,method,reference,notes,created_at";
fn row(r:&Row<'_>)->rusqlite::Result<SaleReceipt>{
    Ok(SaleReceipt{id:r.get(0)?,sale_id:r.get(1)?,received_at:r.get(2)?,amount_cents:r.get(3)?,
        method:r.get(4)?,reference:r.get(5)?,notes:r.get(6)?,created_at:r.get(7)?})
}
fn list_core(c:&Connection)->Result<Vec<SaleReceipt>,String>{
    let mut q=c.prepare(&format!("SELECT {FIELDS} FROM sale_receipts ORDER BY received_at DESC,created_at DESC,id DESC")).map_err(err)?;
    let rows=q.query_map([],row).map_err(err)?.collect::<Result<Vec<_>,_>>().map_err(err)?;
    Ok(rows)
}
fn balances_core(c:&Connection)->Result<Vec<SaleBalance>,String>{
    let mut q=c.prepare("SELECT s.id,s.amount_cents,COALESCE(SUM(r.amount_cents),0)
        FROM sales s LEFT JOIN sale_receipts r ON r.sale_id=s.id GROUP BY s.id
        ORDER BY s.sold_at DESC,s.created_at DESC,s.id").map_err(err)?;
    let rows=q.query_map([],|r|{
        let sale:i64=r.get(1)?;let received:i64=r.get(2)?;
        Ok(SaleBalance{sale_id:r.get(0)?,sale_cents:sale,received_cents:received,outstanding_cents:sale-received})
    }).map_err(err)?.collect::<Result<Vec<_>,_>>().map_err(err)?;
    Ok(rows)
}
fn record_core(c:&mut Connection,input:SaleReceiptInput)->Result<SaleReceipt,String>{
    if !valid_date(&input.received_at){return Err("Invalid receipt date".into())}
    if !(1..=100_000_000_000).contains(&input.amount_cents){return Err("Invalid receipt amount".into())}
    if !["bank_transfer","cash","card","other"].contains(&input.method.as_str()){
        return Err("Invalid receipt method".into())
    }
    let reference=clean(input.reference,200)?;
    let notes=clean(input.notes,2000)?;
    let tx=c.transaction().map_err(err)?;
    let (sale,received):(i64,i64)=tx.query_row(
        "SELECT s.amount_cents,
         (SELECT COALESCE(SUM(amount_cents),0) FROM sale_receipts WHERE sale_id=s.id)
         FROM sales s WHERE s.id=?1",
        params![input.sale_id],|r|Ok((r.get(0)?,r.get(1)?))
    ).optional().map_err(err)?.ok_or_else(||"Sale not found".to_string())?;
    if received.checked_add(input.amount_cents).is_none_or(|sum|sum>sale){
        return Err("Receipt exceeds outstanding sale amount".into())
    }
    let id=Uuid::new_v4().to_string();
    let stamp=Utc::now().to_rfc3339();
    tx.execute("INSERT INTO sale_receipts(id,sale_id,received_at,amount_cents,method,reference,notes,created_at)
        VALUES(?1,?2,?3,?4,?5,?6,?7,?8)",
        params![id,input.sale_id,input.received_at,input.amount_cents,input.method,reference,notes,stamp]).map_err(err)?;
    let saved=tx.query_row(&format!("SELECT {FIELDS} FROM sale_receipts WHERE id=?1"),params![id],row).map_err(err)?;
    tx.commit().map_err(err)?;
    Ok(saved)
}
fn delete_core(c:&Connection,id:&str)->Result<(),String>{
    if c.execute("DELETE FROM sale_receipts WHERE id=?1",params![id]).map_err(err)?==0{
        return Err("Receipt not found".into())
    }
    Ok(())
}
#[tauri::command]
pub fn list_sale_receipts(db:State<'_,Database>)->Result<Vec<SaleReceipt>,String>{
    let c=db.conn.lock().map_err(err)?;list_core(&c)
}
#[tauri::command]
pub fn list_sale_balances(db:State<'_,Database>)->Result<Vec<SaleBalance>,String>{
    let c=db.conn.lock().map_err(err)?;balances_core(&c)
}
#[tauri::command]
pub fn record_sale_receipt(db:State<'_,Database>,input:SaleReceiptInput)->Result<SaleReceipt,String>{
    let mut c=db.conn.lock().map_err(err)?;record_core(&mut c,input)
}
#[tauri::command]
pub fn delete_sale_receipt(db:State<'_,Database>,id:String)->Result<(),String>{
    let c=db.conn.lock().map_err(err)?;delete_core(&c,&id)
}
#[cfg(test)]
mod tests{
    use super::*;
    fn setup(c:&Connection){
        c.execute("INSERT INTO artworks(id,code,title,created_at,updated_at) VALUES('a','A','Art','t','t')",[]).unwrap();
        c.execute("INSERT INTO collectors(id,name,created_at,updated_at) VALUES('c','Collector','t','t')",[]).unwrap();
        c.execute("INSERT INTO sales(id,artwork_id,collector_id,amount_cents,sold_at,created_at)
            VALUES('s','a','c',12001,'2026-11-01','t')",[]).unwrap();
    }
    fn input(amount:i64)->SaleReceiptInput{
        SaleReceiptInput{sale_id:"s".into(),received_at:"2026-11-21".into(),amount_cents:amount,
            method:"bank_transfer".into(),reference:Some("  TR-123  ".into()),notes:None}
    }
    #[test]
    fn partial_receipts_block_overpayment_and_restrict_sale_deletion(){
        let temp=tempfile::tempdir().unwrap();let db=crate::db::open(temp.path().into()).unwrap();
        let mut c=db.conn.lock().unwrap();setup(&c);
        let first=record_core(&mut c,input(5000)).unwrap();
        assert_eq!(first.reference.as_deref(),Some("TR-123"));
        assert_eq!(balances_core(&c).unwrap()[0].outstanding_cents,7001);
        assert!(record_core(&mut c,input(7002)).is_err());
        let second=record_core(&mut c,input(7001)).unwrap();
        assert_eq!(balances_core(&c).unwrap()[0].outstanding_cents,0);
        assert!(c.execute("DELETE FROM sales WHERE id='s'",[]).is_err());
        delete_core(&c,&second.id).unwrap();
        assert_eq!(balances_core(&c).unwrap()[0].received_cents,5000);
        delete_core(&c,&first.id).unwrap();
        assert_eq!(balances_core(&c).unwrap()[0].received_cents,0);
    }
    #[test]
    fn rejects_invalid_inputs_and_sql_bypass(){
        let temp=tempfile::tempdir().unwrap();let db=crate::db::open(temp.path().into()).unwrap();
        let mut c=db.conn.lock().unwrap();setup(&c);
        let mut invalid=input(1);invalid.received_at="2026-02-30".into();
        assert!(record_core(&mut c,invalid).is_err());
        let mut invalid=input(1);invalid.method="unknown".into();
        assert!(record_core(&mut c,invalid).is_err());
        let mut invalid=input(1);invalid.sale_id="absent".into();
        assert!(record_core(&mut c,invalid).is_err());
        assert!(record_core(&mut c,input(0)).is_err());
        c.execute("INSERT INTO sale_receipts(id,sale_id,received_at,amount_cents,method,created_at)
            VALUES('r1','s','2026-11-21',10000,'cash','t')",[]).unwrap();
        assert!(c.execute("INSERT INTO sale_receipts(id,sale_id,received_at,amount_cents,method,created_at)
            VALUES('r2','s','2026-11-21',2002,'cash','t')",[]).is_err());
        assert!(c.execute("UPDATE sale_receipts SET amount_cents=12002 WHERE id='r1'",[]).is_err());
        assert!(c.execute("UPDATE sales SET amount_cents=9999 WHERE id='s'",[]).is_err());
        assert_eq!(list_core(&c).unwrap().len(),1);
    }
    #[test]
    fn v3_migration_does_not_invent_historical_receipts(){
        let temp=tempfile::tempdir().unwrap();let db=crate::db::open(temp.path().into()).unwrap();
        {let c=db.conn.lock().unwrap();setup(&c);
         c.execute_batch("DROP TRIGGER sale_receipts_limit_insert;
             DROP TRIGGER sale_receipts_limit_update;
             DROP TRIGGER sales_no_shrink_below_receipts;
             DROP TABLE sale_receipts; PRAGMA user_version=3;").unwrap();
        }
        drop(db);
        let upgraded=crate::db::open(temp.path().into()).unwrap();let c=upgraded.conn.lock().unwrap();
        let version:i64=c.pragma_query_value(None,"user_version",|r|r.get(0)).unwrap();
        assert_eq!(version,4);
        assert_eq!(balances_core(&c).unwrap()[0].outstanding_cents,12001);
        assert!(list_core(&c).unwrap().is_empty());
    }
}