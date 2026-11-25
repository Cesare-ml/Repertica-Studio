//! Explicit, local sale collections. These are manually reported, not bank-verified.
use crate::db::Database;
use chrono::{NaiveDate, Utc};
use rusqlite::{params, Connection, OptionalExtension, Row};
use serde::{Deserialize, Serialize};
use tauri::State;
use uuid::Uuid;

fn err(e: impl std::fmt::Display) -> String { crate::util::friendly(e) }
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
    // Imported databases can have triggers that ignore DELETE or recreate the
    // row. Keep the attachment check and the final result in one transaction:
    // never report success or commit trigger side effects for a failed removal.
    let tx = c.unchecked_transaction().map_err(err)?;
    let attached: bool = tx.query_row(
        "SELECT EXISTS(SELECT 1 FROM sale_receipt_attachments WHERE receipt_id=?1)",
        params![id], |r| r.get(0),
    ).map_err(err)?;
    if attached { return Err("Rimuovi l'allegato prima di cancellare l'incasso".into()); }
    if tx.execute("DELETE FROM sale_receipts WHERE id=?1",params![id]).map_err(err)? != 1 {
        return Err("Incasso non trovato o eliminazione ignorata da SQLite".into());
    }
    let remains: bool = tx.query_row(
        "SELECT EXISTS(SELECT 1 FROM sale_receipts WHERE id=?1)",
        params![id], |r| r.get(0),
    ).map_err(err)?;
    if remains {
        return Err("SQLite ha ricreato l'incasso: eliminazione annullata".into());
    }
    tx.commit().map_err(err)?;
    Ok(())
}

#[derive(Debug,Clone,Serialize)]
pub struct MonthlyReceipt {
    pub month:String,pub amount_cents:i64,pub count:i64,
}
#[derive(Debug,Clone,Serialize)]
pub struct ReceiptYearOverview {
    pub year:i32,pub months:Vec<MonthlyReceipt>,pub total_cents:i64,pub count:i64,
}
// This report is grouped by receipt date, NOT by sale date. No sale is
// considered paid until an explicit receipt exists in sale_receipts.
fn receipt_year_overview_core(c:&Connection,year:i32)->Result<ReceiptYearOverview,String>{
    if !(1900..=2200).contains(&year){return Err("Invalid receipt report year".into())}
    let mut months=(1..=12).map(|m|MonthlyReceipt{
        month:format!("{year}-{m:02}"),amount_cents:0,count:0
    }).collect::<Vec<_>>();
    let start=format!("{year}-01-01");
    let end=format!("{}-01-01",year+1);
    let mut query=c.prepare("SELECT substr(received_at,1,7),SUM(amount_cents),COUNT(*)
        FROM sale_receipts WHERE received_at>=?1 AND received_at<?2
        GROUP BY substr(received_at,1,7) ORDER BY 1").map_err(err)?;
    let records=query.query_map(params![start,end],|r|Ok((
        r.get::<_,String>(0)?,r.get::<_,i64>(1)?,r.get::<_,i64>(2)?
    ))).map_err(err)?.collect::<Result<Vec<_>,_>>().map_err(err)?;
    let mut total_cents=0i64;
    let mut count=0i64;
    for (month,amount,items) in records {
        let slot=months.iter_mut().find(|m|m.month==month)
            .ok_or_else(||"Invalid stored receipt month".to_string())?;
        slot.amount_cents=amount;
        slot.count=items;
        total_cents=total_cents.checked_add(amount).ok_or("Receipt total exceeds integer range")?;
        count=count.checked_add(items).ok_or("Receipt count exceeds integer range")?;
    }
    Ok(ReceiptYearOverview{year,months,total_cents,count})
}
#[tauri::command]
pub fn receipt_year_overview(db:State<'_,Database>,year:i32)->Result<ReceiptYearOverview,String>{
    let c=db.conn.lock().map_err(err)?;
    receipt_year_overview_core(&c,year)
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
    fn receipt_with_attachment_needs_explicit_detach_before_delete() {
        let tmp = tempfile::tempdir().unwrap();
        let db = crate::db::open(tmp.path().into()).unwrap();
        let mut c = db.conn.lock().unwrap();
        setup(&c);
        let receipt = record_core(&mut c, input(150)).unwrap();
        c.execute(
            "INSERT INTO sale_receipt_attachments
            (receipt_id,media_path,original_name,mime_type,size_bytes,sha256,created_at)
            VALUES(?1,'media/receipt-1.pdf','proof.pdf','application/pdf',8,?2,'now')",
            params![receipt.id, "0".repeat(64)],
        ).unwrap();
        assert!(delete_core(&c, &receipt.id).unwrap_err().contains("allegato"));
        assert_eq!(list_core(&c).unwrap().len(), 1);
        c.execute("DELETE FROM sale_receipt_attachments WHERE receipt_id=?1",
            params![receipt.id]).unwrap();
        delete_core(&c, &receipt.id).unwrap();
    }
    #[test]
    fn ignored_or_recreated_receipt_deletes_roll_back_without_changing_balances() {
        let temp = tempfile::tempdir().unwrap();
        let db = crate::db::open(temp.path().into()).unwrap();
        let mut c = db.conn.lock().unwrap();
        setup(&c);
        let receipt = record_core(&mut c, input(2300)).unwrap();
        let before = balances_core(&c).unwrap()[0].received_cents;

        c.execute_batch("CREATE TRIGGER ignore_receipt_delete BEFORE DELETE ON sale_receipts
            BEGIN SELECT RAISE(IGNORE); END;").unwrap();
        let error = delete_core(&c, &receipt.id).unwrap_err();
        assert!(error.contains("ignorata"), "{error}");
        assert_eq!(list_core(&c).unwrap().len(), 1);
        assert_eq!(balances_core(&c).unwrap()[0].received_cents, before);
        c.execute_batch("DROP TRIGGER ignore_receipt_delete").unwrap();

        // The DELETE claims to affect one row, but an AFTER trigger restores it.
        c.execute_batch("CREATE TRIGGER recreate_receipt AFTER DELETE ON sale_receipts
            BEGIN
              INSERT INTO sale_receipts(id,sale_id,received_at,amount_cents,method,reference,notes,created_at)
              VALUES(OLD.id,OLD.sale_id,OLD.received_at,OLD.amount_cents,
                     OLD.method,OLD.reference,OLD.notes,OLD.created_at);
            END;").unwrap();
        let error = delete_core(&c, &receipt.id).unwrap_err();
        assert!(error.contains("ricreato"), "{error}");
        assert_eq!(list_core(&c).unwrap().len(), 1);
        assert_eq!(balances_core(&c).unwrap()[0].received_cents, before);
        c.execute_batch("DROP TRIGGER recreate_receipt").unwrap();

        delete_core(&c, &receipt.id).unwrap();
        assert!(list_core(&c).unwrap().is_empty());
        assert_eq!(balances_core(&c).unwrap()[0].received_cents, 0);
    }

    #[test]
    fn receipt_year_overview_uses_collection_date_not_sale_date_and_includes_empty_months(){
        let temp=tempfile::tempdir().unwrap();
        let db=crate::db::open(temp.path().into()).unwrap();
        let mut c=db.conn.lock().unwrap();
        setup(&c);
        let november=record_core(&mut c,input(3000)).unwrap();
        let mut december=input(1001);
        december.received_at="2026-12-02".into();
        record_core(&mut c,december).unwrap();
        let mut next_year=input(8000);
        next_year.received_at="2027-01-01".into();
        record_core(&mut c,next_year).unwrap();

        let current=receipt_year_overview_core(&c,2026).unwrap();
        assert_eq!(current.year,2026);
        assert_eq!(current.months.len(),12);
        assert_eq!((current.total_cents,current.count),(4001,2));
        assert_eq!((current.months[10].amount_cents,current.months[10].count),(3000,1));
        assert_eq!((current.months[11].amount_cents,current.months[11].count),(1001,1));
        assert_eq!((current.months[0].amount_cents,current.months[0].count),(0,0));
        let later=receipt_year_overview_core(&c,2027).unwrap();
        assert_eq!((later.total_cents,later.count),(8000,1));
        let empty=receipt_year_overview_core(&c,2025).unwrap();
        assert_eq!((empty.total_cents,empty.count),(0,0));
        assert!(empty.months.iter().all(|m|m.amount_cents==0 && m.count==0));

        delete_core(&c,&november.id).unwrap();
        assert_eq!(receipt_year_overview_core(&c,2026).unwrap().total_cents,1001);
        assert_eq!(balances_core(&c).unwrap()[0].outstanding_cents,3000);
        assert!(receipt_year_overview_core(&c,1899).is_err());
        assert!(receipt_year_overview_core(&c,2201).is_err());
    }
    #[test]
    fn v3_migration_does_not_invent_historical_receipts(){
        let temp=tempfile::tempdir().unwrap();let db=crate::db::open(temp.path().into()).unwrap();
        {let c=db.conn.lock().unwrap();setup(&c);
         c.execute_batch("DROP TRIGGER sale_receipts_limit_insert;
             DROP TRIGGER sale_receipts_limit_update;
             DROP TRIGGER sales_no_shrink_below_receipts;
             DROP TABLE sale_receipt_attachments;
             DROP TABLE sale_receipts;
             DROP INDEX idx_artwork_images_position;
             ALTER TABLE artwork_images DROP COLUMN sort_order;
             PRAGMA user_version=3;").unwrap();
        }
        drop(db);
        let upgraded=crate::db::open(temp.path().into()).unwrap();let c=upgraded.conn.lock().unwrap();
        let version:i64=c.pragma_query_value(None,"user_version",|r|r.get(0)).unwrap();
        assert_eq!(version,crate::db::SCHEMA_VERSION);
        assert_eq!(balances_core(&c).unwrap()[0].outstanding_cents,12001);
        assert!(list_core(&c).unwrap().is_empty());
    }
}