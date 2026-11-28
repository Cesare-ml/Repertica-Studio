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
// An externally modified/imported SQLite archive may have invalid totals even
// when foreign_key_check succeeds (for example, its receipt triggers were disabled).
// Never display negative "outstanding" sums or use corrupted totals to accept a payment.
fn checked_balance(sale_id:String,sale:i64,received:i64,invalid_receipts:i64)->Result<SaleBalance,String>{
    if sale<=0 || received<0 || received>sale || invalid_receipts != 0 {
        return Err(format!("Invalid stored receipt balance for sale {sale_id}: check the archive before recording payments"));
    }
    let outstanding=sale.checked_sub(received)
        .ok_or_else(||format!("Receipt balance exceeds integer range for sale {sale_id}"))?;
    Ok(SaleBalance{sale_id,sale_cents:sale,received_cents:received,outstanding_cents:outstanding})
}
fn balances_core(c:&Connection)->Result<Vec<SaleBalance>,String>{
    let mut q=c.prepare("SELECT s.id,s.amount_cents,COALESCE(SUM(r.amount_cents),0),
        COUNT(CASE WHEN r.id IS NOT NULL AND (typeof(r.amount_cents) <> 'integer' OR r.amount_cents NOT BETWEEN 1 AND 100000000000) THEN 1 END)
        FROM sales s LEFT JOIN sale_receipts r ON r.sale_id=s.id GROUP BY s.id
        ORDER BY s.sold_at DESC,s.created_at DESC,s.id").map_err(err)?;
    let rows=q.query_map([],|r|Ok((
        r.get::<_,String>(0)?,r.get::<_,i64>(1)?,r.get::<_,i64>(2)?,r.get::<_,i64>(3)?
    ))).map_err(err)?.collect::<Result<Vec<_>,_>>().map_err(err)?;
    rows.into_iter().map(|(id,sale,received,invalid)|checked_balance(id,sale,received,invalid)).collect()
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
    let (sale,received,invalid):(i64,i64,i64)=tx.query_row(
        "SELECT s.amount_cents,
         (SELECT COALESCE(SUM(amount_cents),0) FROM sale_receipts WHERE sale_id=s.id),
         (SELECT COUNT(*) FROM sale_receipts WHERE sale_id=s.id
             AND (typeof(amount_cents) <> 'integer' OR amount_cents NOT BETWEEN 1 AND 100000000000))
         FROM sales s WHERE s.id=?1",
        params![input.sale_id],|r|Ok((r.get(0)?,r.get(1)?,r.get(2)?))
    ).optional().map_err(err)?.ok_or_else(||"Sale not found".to_string())?;
    let balance=checked_balance(input.sale_id.clone(),sale,received,invalid)?;
    if input.amount_cents>balance.outstanding_cents {
        return Err("Receipt exceeds outstanding sale amount".into())
    }
    let id=Uuid::new_v4().to_string();
    let stamp=Utc::now().to_rfc3339();
    let previous_count:i64=tx.query_row("SELECT COUNT(*) FROM sale_receipts WHERE sale_id=?1",
        params![input.sale_id],|r|r.get(0)).map_err(err)?;
    let changed=tx.execute("INSERT INTO sale_receipts(id,sale_id,received_at,amount_cents,method,reference,notes,created_at)
        VALUES(?1,?2,?3,?4,?5,?6,?7,?8)",
        params![id,input.sale_id,input.received_at,input.amount_cents,input.method,reference,notes,stamp]).map_err(err)?;
    if changed!=1 { return Err("Receipt insertion ignored by SQLite".into()); }
    let saved=tx.query_row(&format!("SELECT {FIELDS} FROM sale_receipts WHERE id=?1"),params![id],row).map_err(err)?;
    if saved.sale_id!=input.sale_id || saved.received_at!=input.received_at
        || saved.amount_cents!=input.amount_cents || saved.method!=input.method
        || saved.reference!=reference || saved.notes!=notes || saved.created_at!=stamp {
        return Err("Receipt changed during insertion".into());
    }
    let (actual_sale,actual_total,actual_count,actual_invalid):(i64,i64,i64,i64)=tx.query_row(
        "SELECT s.amount_cents,COALESCE(SUM(r.amount_cents),0),COUNT(r.id),
         COUNT(CASE WHEN r.id IS NOT NULL AND (typeof(r.amount_cents) <> 'integer'
             OR r.amount_cents NOT BETWEEN 1 AND 100000000000) THEN 1 END)
         FROM sales s LEFT JOIN sale_receipts r ON r.sale_id=s.id WHERE s.id=?1 GROUP BY s.id",
        params![input.sale_id],|r|Ok((r.get(0)?,r.get(1)?,r.get(2)?,r.get(3)?))).map_err(err)?;
    if actual_sale!=sale || Some(actual_total)!=received.checked_add(input.amount_cents)
        || Some(actual_count)!=previous_count.checked_add(1) || actual_invalid!=0 {
        return Err("Receipt balance changed unexpectedly during insertion".into());
    }
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
    // Validate individual rows before aggregation: a negative amount may be
    // hidden by a positive receipt in the same month, and a month prefix alone
    // cannot establish a valid calendar date. Keep this report read-only.
    let mut query=c.prepare("SELECT received_at,amount_cents FROM sale_receipts
        WHERE received_at>=?1 AND received_at<?2").map_err(err)?;
    let records=query.query_map(params![start,end],|r|Ok((
        r.get::<_,String>(0)?,r.get::<_,i64>(1)?
    ))).map_err(err)?;
    let mut total_cents=0i64;
    let mut count=0i64;
    for record in records {
        let (date,amount)=record.map_err(err)?;
        if !valid_date(&date) || !(1..=100_000_000_000).contains(&amount) {
            return Err("Invalid stored receipt date or amount in annual report".into());
        }
        let slot=months.iter_mut().find(|m|m.month==date[..7])
            .ok_or_else(||"Invalid stored receipt month".to_string())?;
        slot.amount_cents=slot.amount_cents.checked_add(amount).ok_or("Receipt total exceeds integer range")?;
        slot.count=slot.count.checked_add(1).ok_or("Receipt count exceeds integer range")?;
        total_cents=total_cents.checked_add(amount).ok_or("Receipt total exceeds integer range")?;
        count=count.checked_add(1).ok_or("Receipt count exceeds integer range")?;
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
    fn ignored_or_changed_receipt_insert_rolls_back_and_retry_succeeds() {
        let temp=tempfile::tempdir().unwrap();
        let db=crate::db::open(temp.path().into()).unwrap();
        let mut c=db.conn.lock().unwrap();
        setup(&c);
        let first=record_core(&mut c,input(2000)).unwrap();
        for trigger in [
            "CREATE TRIGGER adversarial_receipt BEFORE INSERT ON sale_receipts
             BEGIN SELECT RAISE(IGNORE); END;",
            "CREATE TRIGGER adversarial_receipt AFTER INSERT ON sale_receipts
             BEGIN UPDATE sale_receipts SET received_at='2026-11-22' WHERE id=NEW.id; END;",
            "CREATE TRIGGER adversarial_receipt AFTER INSERT ON sale_receipts
             BEGIN UPDATE sale_receipts SET amount_cents=amount_cents+1 WHERE id<>NEW.id AND sale_id=NEW.sale_id; END;",
            "CREATE TRIGGER adversarial_receipt AFTER INSERT ON sale_receipts
             BEGIN INSERT INTO sale_receipts(id,sale_id,received_at,amount_cents,method,created_at)
             VALUES(NEW.id||'-extra',NEW.sale_id,NEW.received_at,1,'cash',NEW.created_at); END;",
        ] {
            c.execute_batch(trigger).unwrap();
            assert!(record_core(&mut c,input(500)).is_err(), "{trigger}");
            let rows=list_core(&c).unwrap();
            assert_eq!(rows.len(),1);
            assert_eq!(rows[0].id,first.id);
            assert_eq!(rows[0].amount_cents,2000);
            assert_eq!(balances_core(&c).unwrap()[0].received_cents,2000);
            c.execute_batch("DROP TRIGGER adversarial_receipt").unwrap();
        }
        let retry=record_core(&mut c,input(500)).unwrap();
        assert_eq!(retry.amount_cents,500);
        assert_eq!(list_core(&c).unwrap().len(),2);
        assert_eq!(balances_core(&c).unwrap()[0].outstanding_cents,9501);
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
    fn annual_report_and_installments_preserve_receipt_dates_and_reject_bad_rows() {
        use crate::installments::{replace_plan, list_with_status, InstallmentInput};
        let temp=tempfile::tempdir().unwrap();
        let db=crate::db::open(temp.path().into()).unwrap();
        let mut c=db.conn.lock().unwrap();
        setup(&c);
        let tx=c.transaction().unwrap();
        replace_plan(&tx,"s",&[
            InstallmentInput{due_date:"2026-11-21".into(),amount_cents:6000,notes:None},
            InstallmentInput{due_date:"2026-12-21".into(),amount_cents:6001,notes:None},
        ]).unwrap();
        tx.commit().unwrap();
        record_core(&mut c,input(2000)).unwrap();
        let mut future=input(3000);
        future.received_at="2026-12-01".into();
        record_core(&mut c,future).unwrap();
        let today=NaiveDate::from_ymd_opt(2026,11,21).unwrap();
        assert_eq!(list_with_status(&c,today).unwrap()[0].paid_cents,2000);
        assert_eq!(balances_core(&c).unwrap()[0].received_cents,5000);
        let report=receipt_year_overview_core(&c,2026).unwrap();
        assert_eq!((report.total_cents,report.months[10].amount_cents,report.months[11].amount_cents),(5000,2000,3000));
        c.execute_batch("PRAGMA ignore_check_constraints=ON;").unwrap();
        c.execute("INSERT INTO sale_receipts(id,sale_id,received_at,amount_cents,method,created_at)
            VALUES('bad','s','2026-11-20',-1000,'cash','t')",[]).unwrap();
        assert!(receipt_year_overview_core(&c,2026).unwrap_err().contains("Invalid stored receipt"));
        assert!(balances_core(&c).is_err());
        assert!(list_with_status(&c,today).is_err());
        assert_eq!(list_core(&c).unwrap().len(),3);
        c.execute("UPDATE sale_receipts SET amount_cents=1000,received_at='2026-11-31' WHERE id='bad'",[]).unwrap();
        assert!(receipt_year_overview_core(&c,2026).unwrap_err().contains("Invalid stored receipt"));
        assert_eq!(list_core(&c).unwrap().len(),3);
        c.execute("DELETE FROM sale_receipts WHERE id='bad'",[]).unwrap();
        c.execute_batch("PRAGMA ignore_check_constraints=OFF;").unwrap();
        assert_eq!(receipt_year_overview_core(&c,2026).unwrap().total_cents,5000);
        assert_eq!(list_with_status(&c,today).unwrap()[0].paid_cents,2000);
    }

    #[test]
    fn invalid_imported_receipt_totals_do_not_allow_new_payments() {
        let temp=tempfile::tempdir().unwrap();
        let db=crate::db::open(temp.path().into()).unwrap();
        let mut c=db.conn.lock().unwrap();
        setup(&c);
        // Simulate a SQLite archive altered by external tools with triggers and
        // CHECK constraints bypassed. A negative historical receipt previously
        // allowed an extra receipt despite an invalid underlying balance.
        c.execute_batch("DROP TRIGGER sale_receipts_limit_insert;
            DROP TRIGGER sale_receipts_limit_update;
            DROP TRIGGER sales_no_shrink_below_receipts;
            PRAGMA ignore_check_constraints=ON;").unwrap();
        c.execute("INSERT INTO sale_receipts(id,sale_id,received_at,amount_cents,method,created_at)
            VALUES('negative','s','2026-11-20',-1000,'cash','t')",[]).unwrap();
        c.execute_batch("PRAGMA ignore_check_constraints=OFF;").unwrap();
        assert!(balances_core(&c).unwrap_err().contains("Invalid stored receipt balance"));
        assert!(record_core(&mut c,input(12001)).unwrap_err().contains("Invalid stored receipt balance"));
        assert_eq!(list_core(&c).unwrap().len(),1);

        // A negative receipt can be hidden by a larger positive receipt: the
        // aggregate alone would look legitimate, but individual rows are invalid.
        c.execute("INSERT INTO sale_receipts(id,sale_id,received_at,amount_cents,method,created_at)
            VALUES('positive','s','2026-11-20',2000,'cash','t')",[]).unwrap();
        assert!(balances_core(&c).unwrap_err().contains("Invalid stored receipt balance"));
        assert!(record_core(&mut c,input(1)).is_err());
        assert_eq!(list_core(&c).unwrap().len(),2);
        c.execute("DELETE FROM sale_receipts WHERE id='positive'",[]).unwrap();

        // Overpayment introduced with the trigger missing also fails closed.
        c.execute("DELETE FROM sale_receipts WHERE id='negative'",[]).unwrap();
        c.execute("INSERT INTO sale_receipts(id,sale_id,received_at,amount_cents,method,created_at)
            VALUES('overpaid','s','2026-11-20',12002,'cash','t')",[]).unwrap();
        assert!(balances_core(&c).unwrap_err().contains("Invalid stored receipt balance"));
        assert!(record_core(&mut c,input(1)).is_err());
        assert_eq!(list_core(&c).unwrap().len(),1);

        // Removing the offending imported row restores normal behavior without
        // rewriting historical sales or inventing receipt entries.
        c.execute("DELETE FROM sale_receipts WHERE id='overpaid'",[]).unwrap();
        assert_eq!(balances_core(&c).unwrap()[0].outstanding_cents,12001);
        let saved=record_core(&mut c,input(500)).unwrap();
        assert_eq!(saved.amount_cents,500);
        assert_eq!(balances_core(&c).unwrap()[0].outstanding_cents,11501);
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