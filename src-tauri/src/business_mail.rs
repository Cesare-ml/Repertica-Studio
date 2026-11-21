//! Personal/business correspondence for Repertica Studio.
//! Ports concepts from Gestionale-Opere/backend/modules/business-mail:
//! independent accounts, guarded IMAP UIDVALIDITY, read-only mailbox access,
//! local optimistic drafts, explicit SMTP submission and a native secret vault.
//!
//! No message body is cached by Studio. Passwords go to the OS credential store,
//! never to SQLite, JSON results, backups, logs or notification events.
use crate::db::Database;
use chrono::Utc;
use imap::Session;
use keyring::Entry;
use lettre::{Message, SmtpTransport, Transport, message::{Mailbox, header::ContentType},
    transport::smtp::authentication::Credentials};
use mailparse::{MailHeaderMap, ParsedMail};
use rusqlite::{params, OptionalExtension};
use serde::{Serialize, Deserialize};
use std::{collections::HashSet, net::TcpStream};
use tauri::State;
use uuid::Uuid;

const KEYRING_SERVICE: &str = "repertica-studio-mail";
const MAX_MESSAGE_BYTES: u32 = 1024 * 1024;
fn db_error(e: impl std::fmt::Display) -> String { crate::util::friendly(e) }
fn remote_error() -> String { "Connessione al server di posta non riuscita. Controlla le impostazioni, TLS e la password.".into() }
fn credentials_error() -> String { "Credenziali non accessibili nel portachiavi del sistema. Verifica la configurazione del sistema operativo.".into() }
fn valid_host(host:&str)->bool {
    let h=host.trim();
    h.len()>=4 && h.len()<=253 && h.contains('.') && !h.ends_with('.')
        && h.split('.').all(|part| !part.is_empty() && part.len()<=63
            && !part.starts_with('-') && !part.ends_with('-')
            && part.bytes().all(|b| b.is_ascii_alphanumeric() || b==b'-'))
        && !h.eq_ignore_ascii_case("localhost")
}
fn valid_email(email:&str)->bool {
    email.len()<=254 && email.len()>4 && !email.chars().any(|c| c.is_whitespace() || c.is_control())
        && email.split_once('@').is_some_and(|(local,domain)|
            !local.is_empty() && !local.starts_with('.') && !local.ends_with('.')
                && valid_host(domain) && local.len()<=64)
}
fn safe_text(text:&str, max:usize)->bool {
    text.chars().count()<=max && !text.chars().any(|c| c=='\0' || (c.is_control() && c!='\n' && c!='\t' && c!='\r'))
}
fn valid_folder(value:&str)->bool {
    !value.trim().is_empty() && value.len()<=240 &&
    !value.chars().any(|c| c.is_control() || ['"', '\\', '{', '}'].contains(&c))
}
fn clean_preview(input:&str,max:usize)->String {
    input.chars().filter(|c|!c.is_control() || *c=='\t').take(max).collect::<String>()
}
#[derive(Clone,Debug,Serialize,Deserialize)]
pub struct MailAccount {
    pub id:String, pub email:String, pub display_name:String,
    pub imap_host:String, pub imap_port:u16, pub imap_username:String,
    pub smtp_host:String, pub smtp_port:u16, pub smtp_username:String,
    pub sent_folder:String, pub signature:String,
    pub created_at:String, pub updated_at:String,
}
#[derive(Debug,Deserialize)]
pub struct MailAccountInput {
    pub id:Option<String>, pub email:String, pub display_name:String,
    pub imap_host:String, pub imap_port:u16, pub imap_username:String,
    pub smtp_host:String, pub smtp_port:u16, pub smtp_username:String,
    pub sent_folder:String, pub signature:String,
    pub imap_password:Option<String>, pub smtp_password:Option<String>,
}
impl MailAccountInput {
    fn validate(&self)->Result<(),String>{
        if !valid_email(&self.email) || !valid_host(&self.imap_host) || !valid_host(&self.smtp_host) {
            return Err("Indirizzo email o server non valido".into());
        }
        if self.imap_port!=993 || ![465,587].contains(&self.smtp_port) {
            return Err("Sono supportati solo IMAP TLS (993), SMTP TLS (465) e SMTP STARTTLS (587)".into());
        }
        if !safe_text(&self.display_name,120) || !safe_text(&self.signature,3000)
            || !safe_text(&self.imap_username,254) || !safe_text(&self.smtp_username,254)
            || self.imap_username.trim().is_empty() || self.smtp_username.trim().is_empty()
            || (!self.sent_folder.is_empty() && !valid_folder(&self.sent_folder))
            || self.imap_password.as_ref().is_some_and(|s|s.len()>2048 || s.contains('\0'))
            || self.smtp_password.as_ref().is_some_and(|s|s.len()>2048 || s.contains('\0')) {
            return Err("Impostazioni email non valide".into());
        }
        Ok(())
    }
}
fn account_by_id(db:&Database,id:&str)->Result<MailAccount,String>{
    let conn=db.conn.lock().map_err(db_error)?;
    let json=conn.query_row("SELECT config_json FROM mail_accounts WHERE id=?1",[id],
        |r|r.get::<_,String>(0)).optional().map_err(db_error)?
        .ok_or_else(|| "Account email non trovato".to_string())?;
    serde_json::from_str(&json).map_err(db_error)
}
fn key(id:&str,protocol:&str)->Result<Entry,String>{
    Entry::new(KEYRING_SERVICE,&format!("{id}:{protocol}")).map_err(|_|credentials_error())
}
fn read_key(id:&str,protocol:&str)->Result<String,String>{
    key(id,protocol)?.get_password().map_err(|_|credentials_error())
}
fn set_key(id:&str,protocol:&str,secret:&str)->Result<(),String>{
    key(id,protocol)?.set_password(secret).map_err(|_|credentials_error())
}
fn remove_key(id:&str,protocol:&str)->Result<(),String>{
    let k=key(id,protocol)?;
    match k.delete_credential(){
        Ok(())|Err(keyring::Error::NoEntry)=>Ok(()),
        Err(_)=>Err(credentials_error())
    }
}
#[tauri::command]
pub fn mail_list_accounts(db:State<'_,Database>)->Result<Vec<MailAccount>,String>{
    let conn=db.conn.lock().map_err(db_error)?;
    let mut stmt=conn.prepare("SELECT config_json FROM mail_accounts ORDER BY email COLLATE NOCASE").map_err(db_error)?;
    stmt.query_map([],|r|r.get::<_,String>(0)).map_err(db_error)?
        .map(|v|serde_json::from_str(&v.map_err(db_error)?).map_err(db_error)).collect()
}
#[tauri::command]
pub fn mail_save_account(db:State<'_,Database>,input:MailAccountInput)->Result<MailAccount,String>{
    input.validate()?;
    let id=input.id.clone().unwrap_or_else(||Uuid::new_v4().to_string());
    if Uuid::parse_str(&id).is_err(){return Err("ID account non valido".into());}
    let existing=account_by_id(&db,&id).ok();
    if input.id.is_some() && existing.is_none(){return Err("Account non trovato".into());}
    if existing.is_none() && input.imap_password.as_ref().is_none_or(String::is_empty){
        return Err("Inserisci la password IMAP o la password per le app del provider".into());
    }
    let now=Utc::now().to_rfc3339();
    let account=MailAccount {
        id:id.clone(),email:input.email.trim().into(),display_name:input.display_name.trim().into(),
        imap_host:input.imap_host.trim().to_ascii_lowercase(),imap_port:input.imap_port,
        imap_username:input.imap_username.trim().into(),
        smtp_host:input.smtp_host.trim().to_ascii_lowercase(),smtp_port:input.smtp_port,
        smtp_username:input.smtp_username.trim().into(),
        sent_folder:input.sent_folder.trim().into(),signature:input.signature,
        created_at:existing.as_ref().map(|v|v.created_at.clone()).unwrap_or_else(||now.clone()),
        updated_at:now
    };
    if let Some(password)=input.imap_password.as_deref().filter(|s|!s.is_empty()) {
        set_key(&id,"imap",password)?;
    }
    if let Some(password)=input.smtp_password.as_deref().filter(|s|!s.is_empty()){
        set_key(&id,"smtp",password)?;
    }
    let json=serde_json::to_string(&account).map_err(db_error)?;
    let conn=db.conn.lock().map_err(db_error)?;
    conn.execute("INSERT INTO mail_accounts(id,email,config_json,created_at,updated_at) VALUES(?1,?2,?3,?4,?5)
       ON CONFLICT(id) DO UPDATE SET email=excluded.email, config_json=excluded.config_json,
       updated_at=excluded.updated_at",
       params![account.id,account.email,json,account.created_at,account.updated_at]).map_err(db_error)?;
    Ok(account)
}
#[tauri::command]
pub fn mail_remove_account(db:State<'_,Database>,account_id:String)->Result<(),String>{
    account_by_id(&db,&account_id)?;
    // Removing credentials first avoids leaving a usable orphaned secret after removal.
    remove_key(&account_id,"imap")?;
    remove_key(&account_id,"smtp")?;
    let conn=db.conn.lock().map_err(db_error)?;
    conn.execute("DELETE FROM mail_accounts WHERE id=?1",[account_id]).map_err(db_error)?;
    Ok(())
}
type TlsSession=Session<native_tls::TlsStream<TcpStream>>;
fn with_imap<T>(account:&MailAccount,action:impl FnOnce(&mut TlsSession)->Result<T,String>)->Result<T,String>{
    let tls=native_tls::TlsConnector::builder().build().map_err(|_|remote_error())?;
    let client=imap::connect((account.imap_host.as_str(),account.imap_port),
        account.imap_host.as_str(),&tls).map_err(|_|remote_error())?;
    let password=read_key(&account.id,"imap")?;
    let mut session=client.login(&account.imap_username,password).map_err(|_|remote_error())?;
    let result=action(&mut session);
    let _=session.logout();
    result
}
#[derive(Clone,Debug,Serialize)]
pub struct MailFolder { pub name:String, pub role:String }
fn folder_role(name:&str)->String {
    let low=name.to_ascii_lowercase();
    if low=="inbox"{"inbox".into()}
    else if low.contains("sent")||low.contains("inviat"){"sent".into()}
    else if low.contains("draft")||low.contains("bozz"){"drafts".into()}
    else if low.contains("trash")||low.contains("cestin"){"trash".into()}
    else if low.contains("spam")||low.contains("junk"){"junk".into()}
    else {"other".into()}
}
fn folders(session:&mut TlsSession)->Result<Vec<MailFolder>,String>{
    let listed=session.list(None,Some("*")).map_err(|_|remote_error())?;
    if listed.len()>300 {return Err("Troppi nomi di cartelle restituiti dal server".into());}
    let mut rows=listed.iter().filter(|n|valid_folder(n.name())).map(|n|MailFolder{
        name:n.name().into(),role:folder_role(n.name())
    }).collect::<Vec<_>>();
    rows.sort_by(|a,b| {
        let r=|s:&str|match s{"inbox"=>0,"sent"=>1,"drafts"=>2,"other"=>3,"junk"=>4,_=>5};
        r(&a.role).cmp(&r(&b.role)).then_with(||a.name.cmp(&b.name))
    });
    Ok(rows)
}
fn require_folder(folder:&str)->Result<(),String>{
    if !valid_folder(folder){Err("Cartella IMAP non valida".into())}else{Ok(())}
}
fn require_uid_validity(value:u32,actual:Option<u32>)->Result<(),String>{
    if value==0 || actual!=Some(value) {
        Err("La cartella è stata modificata sul server. Aggiorna l'elenco dei messaggi.".into())
    } else {Ok(())}
}
#[tauri::command]
pub async fn mail_list_folders(db:State<'_,Database>,account_id:String)->Result<Vec<MailFolder>,String>{
    let account=account_by_id(&db,&account_id)?;
    tauri::async_runtime::spawn_blocking(move||with_imap(&account,folders))
        .await.map_err(|_|remote_error())?
}
#[derive(Debug,Serialize)]
pub struct MailConnectionCheck { pub ok:bool,pub folders:usize }
#[tauri::command]
pub async fn mail_test_account(db:State<'_,Database>,account_id:String)->Result<MailConnectionCheck,String>{
    let account=account_by_id(&db,&account_id)?;
    tauri::async_runtime::spawn_blocking(move||with_imap(&account,|session|{
        let rows=folders(session)?;
        Ok(MailConnectionCheck{ok:true,folders:rows.len()})
    })).await.map_err(|_|remote_error())?
}
#[derive(Clone,Debug,Serialize)]
pub struct MailHeader {
    pub uid:u32,pub uid_validity:u32,pub folder:String,
    pub subject:String,pub from:String,pub date:String,
    pub seen:bool,pub flagged:bool,pub size:u32
}
#[derive(Debug,Serialize)]
pub struct MailList { pub folder:String,pub uid_validity:u32,pub exists:u32,pub items:Vec<MailHeader> }
fn headers_from_raw(raw:&[u8])->(String,String,String){
    match mailparse::parse_headers(raw){
        Ok((headers,_))=>{
            let subject=headers.get_first_value("Subject").unwrap_or_default();
            let from=headers.get_first_value("From").unwrap_or_default();
            let date=headers.get_first_value("Date").unwrap_or_default();
            (clean_preview(&subject,300),clean_preview(&from,300),clean_preview(&date,120))
        },
        Err(_)=>(String::new(),String::new(),String::new())
    }
}
fn message_header(fetch:&imap::types::Fetch,folder:&str,uid_validity:u32)->Option<MailHeader>{
    let uid=fetch.uid?;
    let (subject,from,date)=fetch.header().map(headers_from_raw).unwrap_or_default();
    let flags=fetch.flags();
    Some(MailHeader{uid,uid_validity,folder:folder.into(),subject,from,date,
        seen:flags.iter().any(|f|matches!(f,imap::types::Flag::Seen)),
        flagged:flags.iter().any(|f|matches!(f,imap::types::Flag::Flagged)),
        size:fetch.size.unwrap_or(0)})
}
#[tauri::command]
pub async fn mail_list_messages(db:State<'_,Database>,account_id:String,folder:String)->Result<MailList,String>{
    require_folder(&folder)?;
    let account=account_by_id(&db,&account_id)?;
    tauri::async_runtime::spawn_blocking(move||with_imap(&account,|session|{
        let selected=session.examine(&folder).map_err(|_|remote_error())?;
        let validity=selected.uid_validity.ok_or_else(remote_error)?;
        let exists=selected.exists;
        if exists==0{return Ok(MailList{folder,uid_validity:validity,exists,items:vec![]});}
        let start=exists.saturating_sub(39).max(1);
        let messages=session.fetch(format!("{start}:{exists}"),"(UID FLAGS RFC822.HEADER RFC822.SIZE)")
            .map_err(|_|remote_error())?;
        let mut rows=messages.iter().filter_map(|item|message_header(item,&folder,validity)).collect::<Vec<_>>();
        rows.sort_by(|a,b|b.uid.cmp(&a.uid));
        Ok(MailList{folder,uid_validity:validity,exists,items:rows})
    })).await.map_err(|_|remote_error())?
}
fn text_part(part:&ParsedMail<'_>,depth:usize)->Option<String>{
    if depth>12{return None;}
    if part.ctype.mimetype.eq_ignore_ascii_case("text/plain") && part.subparts.is_empty(){
        let disposition=part.get_content_disposition();
        if !disposition.disposition.eq_ignore_ascii_case("attachment"){
            return part.get_body().ok();
        }
    }
    for child in &part.subparts {
        if let Some(body)=text_part(child,depth+1){return Some(body);}
    }
    None
}
fn count_attachments(part:&ParsedMail<'_>,depth:usize)->usize{
    if depth>12{return 0;}
    if part.subparts.is_empty() {
        let disposition=part.get_content_disposition();
        return usize::from(disposition.disposition.eq_ignore_ascii_case("attachment")
            || disposition.params.contains_key("filename"));
    }
    part.subparts.iter().map(|p|count_attachments(p,depth+1)).sum()
}
#[derive(Debug,Serialize)]
pub struct MailDetail { pub header:MailHeader,pub body:String,pub html_only:bool,pub attachment_count:usize }
#[tauri::command]
pub async fn mail_read_message(db:State<'_,Database>,account_id:String,folder:String,uid:u32,uid_validity:u32)
    ->Result<MailDetail,String>{
    require_folder(&folder)?;
    if uid==0{return Err("UID IMAP non valido".into());}
    let account=account_by_id(&db,&account_id)?;
    tauri::async_runtime::spawn_blocking(move||with_imap(&account,|session|{
        let mailbox=session.examine(&folder).map_err(|_|remote_error())?;
        require_uid_validity(uid_validity,mailbox.uid_validity)?;
        let sizes=session.uid_fetch(uid.to_string(),"(UID RFC822.SIZE)").map_err(|_|remote_error())?;
        let size=sizes.iter().next().and_then(|f|f.size)
            .ok_or_else(||"Messaggio non più disponibile".to_string())?;
        if size>MAX_MESSAGE_BYTES{return Err("Messaggio troppo grande: download allegati non ancora disponibile".into());}
        drop(sizes);
        let messages=session.uid_fetch(uid.to_string(),"(UID FLAGS RFC822.HEADER RFC822.SIZE BODY.PEEK[])")
            .map_err(|_|remote_error())?;
        let f=messages.iter().next().filter(|f|f.uid==Some(uid))
            .ok_or_else(||"Messaggio non più disponibile".to_string())?;
        let raw=f.body().ok_or_else(||"Corpo del messaggio non disponibile".to_string())?;
        if raw.len()>MAX_MESSAGE_BYTES as usize{return Err("Messaggio troppo grande".into());}
        let parsed=mailparse::parse_mail(raw).map_err(|_|"Messaggio MIME non leggibile".to_string())?;
        let body=text_part(&parsed,0);
        let html_only=body.is_none();
        let header=message_header(f,&folder,uid_validity)
            .or_else(||{let(subject,from,date)=headers_from_raw(raw);Some(MailHeader{
               uid,uid_validity,folder:folder.clone(),subject,from,date,
               seen:false,flagged:false,size
            })}).ok_or_else(remote_error)?;
        Ok(MailDetail{header,body:body.unwrap_or_default().chars().take(100_000).collect(),
            html_only,attachment_count:count_attachments(&parsed,0)})
    })).await.map_err(|_|remote_error())?
}
#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct MailDraft {
    pub id:String,pub account_id:String,pub to:String,pub cc:String,pub bcc:String,
    pub subject:String,pub body:String,pub revision:u32,pub updated_at:String
}
#[derive(Debug,Deserialize)]
pub struct MailDraftInput {
    pub id:Option<String>,pub account_id:String,pub to:String,pub cc:String,
    pub bcc:String,pub subject:String,pub body:String,pub revision:Option<u32>
}
fn validate_draft(input:&MailDraftInput)->Result<(),String>{
    if !safe_text(&input.to,1024) || !safe_text(&input.cc,1024) || !safe_text(&input.bcc,1024)
        || !safe_text(&input.subject,255) || !safe_text(&input.body,100_000){
        return Err("La bozza contiene campi non validi o troppo lunghi".into());
    }
    Ok(())
}
#[tauri::command]
pub fn mail_list_drafts(db:State<'_,Database>,account_id:String)->Result<Vec<MailDraft>,String>{
    account_by_id(&db,&account_id)?;
    let conn=db.conn.lock().map_err(db_error)?;
    let mut stmt=conn.prepare("SELECT payload_json FROM mail_drafts WHERE account_id=?1 ORDER BY updated_at DESC")
        .map_err(db_error)?;
    stmt.query_map([account_id],|r|r.get::<_,String>(0)).map_err(db_error)?
        .map(|s|serde_json::from_str(&s.map_err(db_error)?).map_err(db_error)).collect()
}
#[tauri::command]
pub fn mail_save_draft(db:State<'_,Database>,input:MailDraftInput)->Result<MailDraft,String>{
    validate_draft(&input)?;
    account_by_id(&db,&input.account_id)?;
    let id=input.id.clone().unwrap_or_else(||Uuid::new_v4().to_string());
    if Uuid::parse_str(&id).is_err(){return Err("ID bozza non valido".into());}
    let mut conn=db.conn.lock().map_err(db_error)?;
    let tx=conn.transaction().map_err(db_error)?;
    let previous=tx.query_row("SELECT revision FROM mail_drafts WHERE id=?1 AND account_id=?2",
        params![id,input.account_id],|r|r.get::<_,u32>(0)).optional().map_err(db_error)?;
    if previous!=input.revision{return Err("Conflitto: la bozza è cambiata. Copia il testo prima di riaprirla.".into());}
    let draft=MailDraft{id,account_id:input.account_id,to:input.to,cc:input.cc,bcc:input.bcc,
        subject:input.subject,body:input.body,revision:previous.unwrap_or(0)+1,
        updated_at:Utc::now().to_rfc3339()};
    let json=serde_json::to_string(&draft).map_err(db_error)?;
    tx.execute("INSERT INTO mail_drafts(id,account_id,payload_json,revision,updated_at) VALUES(?1,?2,?3,?4,?5)
        ON CONFLICT(id) DO UPDATE SET payload_json=excluded.payload_json,
         revision=excluded.revision,updated_at=excluded.updated_at",
        params![draft.id,draft.account_id,json,draft.revision,draft.updated_at]).map_err(db_error)?;
    tx.commit().map_err(db_error)?;
    Ok(draft)
}
#[tauri::command]
pub fn mail_remove_draft(db:State<'_,Database>,account_id:String,draft_id:String,revision:u32)->Result<(),String>{
    let conn=db.conn.lock().map_err(db_error)?;
    let count=conn.execute("DELETE FROM mail_drafts WHERE id=?1 AND account_id=?2 AND revision=?3",
        params![draft_id,account_id,revision]).map_err(db_error)?;
    if count==0{return Err("La bozza è stata modificata. Aggiorna l'elenco.".into());}
    Ok(())
}
fn recipients(value:&str)->Result<Vec<String>,String>{
    if value.trim().is_empty(){return Ok(vec![]);}
    let items=value.split([',',';']).map(str::trim).map(str::to_string).collect::<Vec<_>>();
    if items.iter().any(|s|!valid_email(s)){return Err("Uno degli indirizzi destinatari non è valido".into());}
    Ok(items)
}
#[derive(Debug,Deserialize)]
pub struct MailSendInput {
    pub account_id:String,pub to:String,pub cc:String,pub bcc:String,
    pub subject:String,pub body:String,pub confirm:bool
}
#[derive(Debug,Serialize)]
pub struct MailSendResult { pub delivery:String,pub sent_copy:String }
#[tauri::command]
pub async fn mail_send(db:State<'_,Database>,input:MailSendInput)->Result<MailSendResult,String>{
    if !input.confirm{return Err("L'invio richiede una conferma esplicita".into());}
    if !safe_text(&input.subject,255)||input.subject.trim().is_empty()
        || !safe_text(&input.body,100_000)||input.body.trim().is_empty(){
        return Err("Oggetto e testo del messaggio sono obbligatori".into());
    }
    let to=recipients(&input.to)?;let cc=recipients(&input.cc)?;let bcc=recipients(&input.bcc)?;
    let all=to.iter().chain(&cc).chain(&bcc).map(|s|s.to_ascii_lowercase()).collect::<Vec<_>>();
    if all.is_empty()||all.len()>10||all.iter().collect::<HashSet<_>>().len()!=all.len(){
        return Err("Indica da 1 a 10 destinatari, senza duplicati".into());
    }
    let account=account_by_id(&db,&input.account_id)?;
    tauri::async_runtime::spawn_blocking(move|| {
        let mailbox:Mailbox=if account.display_name.trim().is_empty(){account.email.parse()}
            else{format!("{} <{}>",account.display_name.replace(['<','>'], ""),account.email).parse()}
            .map_err(|_|"Mittente non valido".to_string())?;
        let mut builder=Message::builder().from(mailbox).subject(&input.subject)
            .header(ContentType::TEXT_PLAIN);
        for address in to {builder=builder.to(address.parse().map_err(|_|"Destinatario non valido".to_string())?);}
        for address in cc {builder=builder.cc(address.parse().map_err(|_|"Destinatario CC non valido".to_string())?);}
        for address in bcc {builder=builder.bcc(address.parse().map_err(|_|"Destinatario CCN non valido".to_string())?);}
        let message=builder.body(input.body).map_err(|_|"Impossibile preparare il messaggio".to_string())?;
        let password=read_key(&account.id,"smtp").or_else(|_|read_key(&account.id,"imap"))?;
        let creds=Credentials::new(account.smtp_username.clone(),password);
        // TLS certificate validation is mandatory; port 587 must STARTTLS or fail.
        let builder=if account.smtp_port==465 {
            SmtpTransport::relay(&account.smtp_host)
        }else{
            SmtpTransport::starttls_relay(&account.smtp_host)
        }.map_err(|_|remote_error())?;
        let transport=builder.port(account.smtp_port).credentials(creds).build();
        // Any server error after DATA is ambiguous: never retry automatically.
        transport.send(&message).map_err(|_|"Esito SMTP non confermato: verifica Inviati prima di riprovare; non reinviare automaticamente.".to_string())?;
        let sent_copy=if account.sent_folder.is_empty(){"not_configured".to_string()}
        else {
            match with_imap(&account,|s|s.append(&account.sent_folder,&message.formatted()).map_err(|_|remote_error())) {
                Ok(())=>"saved".into(),Err(_)=>"not_saved".into()
            }
        };
        Ok(MailSendResult{delivery:"accepted".into(),sent_copy})
    }).await.map_err(|_|remote_error())?
}
#[cfg(test)]
mod tests{
    use super::*;
    #[test] fn rejects_insecure_hosts_and_ports(){
        assert!(!valid_host("localhost"));assert!(!valid_host("127.0.0.1"));
        assert!(!valid_host("example.com\nINJECT"));assert!(valid_host("mail.example.com"));
        assert!(valid_email("hello@example.com"));assert!(!valid_email("hello@localhost"));
    }
    #[test] fn folder_and_recipient_validation(){
        assert!(!valid_folder("INBOX\r\nDELETE"));
        assert!(valid_folder("INBOX/Sent"));
        assert!(recipients("x@example.com,y@example.com").is_ok());
        assert!(recipients("bad input").is_err());
        assert!(require_uid_validity(44,Some(44)).is_ok());
        assert!(require_uid_validity(44,Some(43)).is_err());
    }
    #[test] fn drafts_enforce_length_limits(){
        let draft=MailDraftInput{id:None,account_id:"x".into(),to:"a@example.com".into(),
            cc:String::new(),bcc:String::new(),subject:"Hello".into(),body:"ok".into(),revision:None};
        assert!(validate_draft(&draft).is_ok());
    }
}
