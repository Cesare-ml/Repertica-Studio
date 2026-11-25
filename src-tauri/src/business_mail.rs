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
use lettre::{Message, SmtpTransport, Transport, message::{Mailbox, MultiPart, SinglePart, Attachment, header::ContentType},
    transport::smtp::authentication::Credentials};
use mailparse::{MailHeaderMap, ParsedMail};
use rusqlite::{params, OptionalExtension};
use serde::{Serialize, Deserialize};
use std::{collections::{HashMap,HashSet}, fs, net::{TcpStream,ToSocketAddrs}, path::Path, sync::{Mutex,OnceLock}, time::{Duration,Instant}};
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
        && h.parse::<std::net::IpAddr>().is_err()
}
fn valid_email(email:&str)->bool {
    email.len()<=254 && email.len()>4 && !email.chars().any(|c| c.is_whitespace() || c.is_control())
        && email.split_once('@').is_some_and(|(local,domain)|
            !local.is_empty() && !local.starts_with('.') && !local.ends_with('.')
                && valid_host(domain) && local.len()<=64)
}
fn header_text(text:&str,max:usize)->bool{
    text.chars().count()<=max && !text.chars().any(char::is_control)
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
        if !header_text(&self.display_name,120) || !safe_text(&self.signature,3000)
            || !header_text(&self.imap_username,254) || !header_text(&self.smtp_username,254)
            || self.imap_username.trim().is_empty() || self.smtp_username.trim().is_empty()
            || (!self.sent_folder.is_empty() && !valid_folder(&self.sent_folder))
            || self.imap_password.as_ref().is_some_and(|s|s.len()>2048 || s.contains('\0'))
            || self.smtp_password.as_ref().is_some_and(|s|s.len()>2048 || s.contains('\0')) {
            return Err("Impostazioni email non valide".into());
        }
        Ok(())
    }
}
pub(crate) fn account_by_id(db:&Database,id:&str)->Result<MailAccount,String>{
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
    accounts(&db)
}
/// Repertica Studio connects a single mailbox: a second one can only be added after removing the first.
pub(crate) fn one_mailbox_only(existing:usize,is_new:bool)->Result<(),String>{
    if is_new && existing>=1 {
        Err("È possibile collegare una sola casella di posta: rimuovi quella attuale per usarne un'altra".into())
    } else { Ok(()) }
}
pub(crate) fn accounts(db:&Database)->Result<Vec<MailAccount>,String>{
    let conn=db.conn.lock().map_err(db_error)?;
    let mut stmt=conn.prepare("SELECT config_json FROM mail_accounts ORDER BY email COLLATE NOCASE").map_err(db_error)?;
    let result=stmt.query_map([],|r|r.get::<_,String>(0)).map_err(db_error)?
        .map(|v|serde_json::from_str(&v.map_err(db_error)?).map_err(db_error)).collect();
    result
}
#[tauri::command]
pub fn mail_save_account(db:State<'_,Database>,input:MailAccountInput)->Result<MailAccount,String>{
    input.validate()?;
    one_mailbox_only(accounts(&db)?.len(),input.id.is_none())?;
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
    // New password or server settings: the kept connection must not be reused.
    forget_session(&id);
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
    forget_session(&account_id);
    let conn=db.conn.lock().map_err(db_error)?;
    conn.execute("DELETE FROM mail_accounts WHERE id=?1",[account_id]).map_err(db_error)?;
    Ok(())
}
type TlsSession=Session<native_tls::TlsStream<TcpStream>>;
/// Connecting (TLS handshake, then login) takes seconds with most providers, so a logged-in session is kept
/// per mailbox and reused by the next command: opening the section, reading a message and the background
/// check no longer start from scratch each time.
struct KeptSession { session:TlsSession, used:Instant }
fn kept_sessions()->&'static Mutex<HashMap<String,KeptSession>>{
    static SESSIONS:OnceLock<Mutex<HashMap<String,KeptSession>>>=OnceLock::new();
    SESSIONS.get_or_init(||Mutex::new(HashMap::new()))
}
/// A session unused for this long is dropped: servers close idle connections after a few minutes.
const IDLE_LIMIT:Duration=Duration::from_secs(4*60);
const CONNECT_TIMEOUT:Duration=Duration::from_secs(15);
/// No answer for this long means the connection is dead (the command is retried on a new one).
const IO_TIMEOUT:Duration=Duration::from_secs(30);
fn take_session(id:&str)->Option<TlsSession>{
    let mut map=kept_sessions().lock().ok()?;
    let kept=map.remove(id)?;
    (kept.used.elapsed()<IDLE_LIMIT).then_some(kept.session)
}
fn keep_session(id:&str,session:TlsSession){
    if let Ok(mut map)=kept_sessions().lock(){ map.insert(id.to_string(),KeptSession{session,used:Instant::now()}); }
}
/// Forget the connection of a mailbox (after its password or server settings change, or when it is removed).
pub(crate) fn forget_session(id:&str){
    if let Ok(mut map)=kept_sessions().lock(){ map.remove(id); }
}
fn connect_session(account:&MailAccount)->Result<TlsSession,String>{
    let tls=native_tls::TlsConnector::builder().build().map_err(|_|remote_error())?;
    let address=(account.imap_host.as_str(),account.imap_port).to_socket_addrs().map_err(|_|remote_error())?
        .next().ok_or_else(remote_error)?;
    let stream=TcpStream::connect_timeout(&address,CONNECT_TIMEOUT).map_err(|_|remote_error())?;
    stream.set_read_timeout(Some(IO_TIMEOUT)).map_err(|_|remote_error())?;
    stream.set_write_timeout(Some(IO_TIMEOUT)).map_err(|_|remote_error())?;
    let _=stream.set_nodelay(true);
    let secured=tls.connect(account.imap_host.as_str(),stream).map_err(|_|remote_error())?;
    let mut client=imap::Client::new(secured);
    client.read_greeting().map_err(|_|remote_error())?;
    let password=read_key(&account.id,"imap")?;
    client.login(&account.imap_username,password).map_err(|_|remote_error())
}
/// Run a command on the mailbox's kept session; if that session turns out to be dead, once more on a new one.
pub(crate) fn with_imap<T>(account:&MailAccount,action:impl Fn(&mut TlsSession)->Result<T,String>)->Result<T,String>{
    if let Some(mut session)=take_session(&account.id){
        match action(&mut session){
            Ok(value)=>{ keep_session(&account.id,session); return Ok(value); }
            // Only a connection problem is worth another attempt; our own refusals are final.
            Err(error) if error!=remote_error()=>{ keep_session(&account.id,session); return Err(error); }
            Err(_)=>{}
        }
    }
    let mut session=connect_session(account)?;
    let result=action(&mut session);
    match &result {
        Ok(_)=>keep_session(&account.id,session),
        Err(error) if *error!=remote_error()=>keep_session(&account.id,session),
        Err(_)=>{}
    }
    result
}
#[derive(Clone,Debug,Serialize)]
pub struct MailFolder {
    /// Name on the server (what the commands take), e.g. `INBOX.Sent`.
    pub name:String,
    /// `inbox`, `sent`, `drafts`, `trash`, `junk`, `archive` or `other`.
    pub role:String,
    /// Readable name without prefixes or encoding, e.g. `Sent`; the interface translates well-known roles.
    pub display:String,
    /// Unread messages (only asked for the inbox and the spam folder).
    pub unread:Option<u32>,
}
/// Decode the modified UTF-7 used by IMAP for folder names (`&AOk-` is `é`).
pub(crate) fn imap_utf7_decode(input:&str)->String {
    use base64::Engine;
    let mut out=String::new();
    let mut rest=input;
    while let Some(start)=rest.find('&') {
        out.push_str(&rest[..start]);
        let after=&rest[start+1..];
        let Some(end)=after.find('-') else { out.push('&'); out.push_str(after); return out; };
        let chunk=&after[..end];
        if chunk.is_empty() { out.push('&'); }
        else {
            match base64::engine::general_purpose::STANDARD_NO_PAD.decode(chunk.replace(',',"/").as_bytes()) {
                Ok(bytes)=>{
                    let units:Vec<u16>=bytes.chunks_exact(2).map(|c|u16::from_be_bytes([c[0],c[1]])).collect();
                    out.push_str(&String::from_utf16_lossy(&units));
                }
                Err(_)=>{ out.push('&'); out.push_str(chunk); out.push('-'); }
            }
        }
        rest=&after[end+1..];
    }
    out.push_str(rest);
    out
}
/// Name shown to people: decoded, without the `INBOX.` / `[Gmail]/` prefix, sub-folders joined with " / ".
fn folder_display(name:&str,delimiter:Option<&str>)->String {
    let decoded=imap_utf7_decode(name);
    let separator=delimiter.and_then(|d|d.chars().next()).unwrap_or('/');
    let mut parts:Vec<&str>=decoded.split(separator).filter(|p|!p.is_empty()).collect();
    if parts.len()>1 && (parts[0].eq_ignore_ascii_case("INBOX") || parts[0].starts_with('[')) { parts.remove(0); }
    if parts.is_empty() { decoded } else { parts.join(" / ") }
}
/// Role of a folder: the server's special-use flags first (`\Sent`, `\Trash`...), then well-known names.
fn folder_role(name:&str,display:&str,attributes:&[String])->String {
    for attribute in attributes {
        match attribute.to_ascii_lowercase().as_str() {
            "\\sent"=>return "sent".into(), "\\drafts"=>return "drafts".into(), "\\trash"=>return "trash".into(),
            "\\junk"=>return "junk".into(), "\\archive"|"\\all"=>return "archive".into(),
            _=>{}
        }
    }
    let low=display.to_lowercase();
    if name.eq_ignore_ascii_case("inbox"){"inbox".into()}
    else if ["sent","inviat","envoy","gesendet","enviado"].iter().any(|k|low.contains(k)){"sent".into()}
    else if ["draft","bozz","brouillon","entwür","entwur","borrador"].iter().any(|k|low.contains(k)){"drafts".into()}
    else if ["trash","cestin","deleted","corbeille","papierkorb","papelera","bin"].iter().any(|k|low==*k||low.contains(k)&&*k!="bin"){"trash".into()}
    else if ["spam","junk","indesiderat","indésirable","indesirable"].iter().any(|k|low.contains(k)){"junk".into()}
    else if ["archive","archivi","archiv"].iter().any(|k|low.contains(k)){"archive".into()}
    else {"other".into()}
}
fn folders(session:&mut TlsSession)->Result<Vec<MailFolder>,String>{
    let listed=session.list(None,Some("*")).map_err(|_|remote_error())?;
    if listed.len()>300 {return Err("Troppi nomi di cartelle restituiti dal server".into());}
    let mut rows=listed.iter().filter(|n|valid_folder(n.name()))
        .filter(|n|!n.attributes().iter().any(|a|matches!(a,imap::types::NameAttribute::NoSelect)))
        .map(|n|{
            let attributes:Vec<String>=n.attributes().iter().filter_map(|a|match a{
                imap::types::NameAttribute::Custom(text)=>Some(text.to_string()), _=>None}).collect();
            let display=folder_display(n.name(),n.delimiter());
            MailFolder{name:n.name().into(),role:folder_role(n.name(),&display,&attributes),display,unread:None}
        }).collect::<Vec<_>>();
    rows.sort_by(|a,b| {
        let r=|s:&str|match s{"inbox"=>0,"drafts"=>1,"sent"=>2,"archive"=>3,"other"=>4,"junk"=>5,_=>6};
        r(&a.role).cmp(&r(&b.role)).then_with(||a.display.to_lowercase().cmp(&b.display.to_lowercase()))
    });
    // Unread counters for the folders where they matter; a failure just leaves the counter out.
    for row in rows.iter_mut().filter(|r|r.role=="inbox"||r.role=="junk") {
        row.unread=session.status(&row.name,"(UNSEEN)").ok().and_then(|m|m.unseen);
    }
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
/// The newest 40 messages of a folder (read-only: `EXAMINE`).
fn list_in_session(session:&mut TlsSession,folder:&str)->Result<MailList,String>{
    let selected=session.examine(folder).map_err(|_|remote_error())?;
    let validity=selected.uid_validity.ok_or_else(remote_error)?;
    let exists=selected.exists;
    if exists==0{return Ok(MailList{folder:folder.to_string(),uid_validity:validity,exists,items:vec![]});}
    let start=exists.saturating_sub(39).max(1);
    let messages=session.fetch(format!("{start}:{exists}"),"(UID FLAGS RFC822.HEADER RFC822.SIZE)")
        .map_err(|_|remote_error())?;
    let mut rows=messages.iter().filter_map(|item|message_header(item,folder,validity)).collect::<Vec<_>>();
    rows.sort_by(|a,b|b.uid.cmp(&a.uid));
    Ok(MailList{folder:folder.to_string(),uid_validity:validity,exists,items:rows})
}
#[tauri::command]
pub async fn mail_list_messages(db:State<'_,Database>,account_id:String,folder:String)->Result<MailList,String>{
    require_folder(&folder)?;
    let account=account_by_id(&db,&account_id)?;
    tauri::async_runtime::spawn_blocking(move||with_imap(&account,|session|list_in_session(session,&folder)))
        .await.map_err(|_|remote_error())?
}
/// Folders and the messages of one of them in a single command (and a single connection): what the section
/// needs when it opens or refreshes. Without a folder, or if it no longer exists, the inbox is shown.
#[derive(Debug,Serialize)]
pub struct MailOverview { pub folders:Vec<MailFolder>, pub listing:Option<MailList> }
#[tauri::command]
pub async fn mail_overview(db:State<'_,Database>,account_id:String,folder:Option<String>)->Result<MailOverview,String>{
    if let Some(name)=&folder { require_folder(name)?; }
    let account=account_by_id(&db,&account_id)?;
    tauri::async_runtime::spawn_blocking(move||with_imap(&account,|session|{
        let rows=folders(session)?;
        let chosen=folder.as_ref().filter(|name|rows.iter().any(|f|&f.name==*name)).cloned()
            .or_else(||rows.iter().find(|f|f.role=="inbox").or(rows.first()).map(|f|f.name.clone()));
        let listing=match chosen { Some(name)=>Some(list_in_session(session,&name)?), None=>None };
        Ok(MailOverview{folders:rows,listing})
    })).await.map_err(|_|remote_error())?
}
fn text_part(part:&ParsedMail<'_>,depth:usize)->Option<String>{
    if depth>12{return None;}
    if part.ctype.mimetype.eq_ignore_ascii_case("text/plain") && part.subparts.is_empty(){
        let disposition=part.get_content_disposition();
        if !matches!(disposition.disposition,mailparse::DispositionType::Attachment){
            return part.get_body().ok();
        }
    }
    for child in &part.subparts {
        if let Some(body)=text_part(child,depth+1){return Some(body);}
    }
    None
}
fn html_part(part:&ParsedMail<'_>,depth:usize)->Option<String>{
    if depth>12{return None;}
    if part.ctype.mimetype.eq_ignore_ascii_case("text/html") && part.subparts.is_empty() {
        let disposition=part.get_content_disposition();
        if !matches!(disposition.disposition,mailparse::DispositionType::Attachment) {
            return part.get_body().ok()
                .and_then(|html|html2text::from_read(html.as_bytes(),90).ok());
        }
    }
    for child in &part.subparts {
        if let Some(text)=html_part(child,depth+1){return Some(text);}
    }
    None
}
fn count_attachments(part:&ParsedMail<'_>,depth:usize)->usize{
    if depth>12{return 0;}
    if part.subparts.is_empty() {
        let disposition=part.get_content_disposition();
        return usize::from(matches!(disposition.disposition,mailparse::DispositionType::Attachment)
            || disposition.params.contains_key("filename"));
    }
    part.subparts.iter().map(|p|count_attachments(p,depth+1)).sum()
}
/// Mark a message as read or unread on the server. Opening a message in the interface does this, so
/// the unread counter and the other devices stay in step; reading itself never changes anything.
#[tauri::command]
pub async fn mail_set_seen(db:State<'_,Database>,account_id:String,folder:String,uid:u32,uid_validity:u32,seen:bool)->Result<(),String>{
    require_folder(&folder)?;
    if uid==0{return Err("UID IMAP non valido".into());}
    let account=account_by_id(&db,&account_id)?;
    tauri::async_runtime::spawn_blocking(move||with_imap(&account,|session|{
        let mailbox=session.select(&folder).map_err(|_|remote_error())?;
        require_uid_validity(uid_validity,mailbox.uid_validity)?;
        let flags=if seen {"+FLAGS.SILENT (\\Seen)"} else {"-FLAGS.SILENT (\\Seen)"};
        session.uid_store(uid.to_string(),flags).map_err(|_|remote_error())?;
        Ok(())
    })).await.map_err(|_|remote_error())?
}
/// What the background watcher needs to know about an inbox.
pub(crate) struct InboxSnapshot { pub uid_validity:u32, pub uid_next:u32, pub exists:u32, pub unseen:u32, pub fresh:Vec<MailHeader> }
/// Unread UIDs that arrived since the previous check: those from the previous `UIDNEXT` on, newest first,
/// and none at all when there is no previous check or the mailbox was renumbered (`UIDVALIDITY` changed).
pub(crate) fn fresh_unseen(baseline:Option<(u32,u32)>,uid_validity:u32,unseen:&[u32])->Vec<u32> {
    let Some((validity,next))=baseline else { return vec![] };
    if validity!=uid_validity {return vec![]}
    let mut fresh:Vec<u32>=unseen.iter().copied().filter(|uid|*uid>=next).collect();
    fresh.sort_unstable_by(|a,b|b.cmp(a));
    fresh.truncate(5);
    fresh
}
pub(crate) fn inbox_snapshot(account:&MailAccount,baseline:Option<(u32,u32)>)->Result<InboxSnapshot,String>{
    with_imap(account,|session|{
        let mailbox=session.examine("INBOX").map_err(|_|remote_error())?;
        let uid_validity=mailbox.uid_validity.ok_or_else(remote_error)?;
        let uid_next=mailbox.uid_next.unwrap_or(0);
        let unseen:Vec<u32>=session.uid_search("UNSEEN").map_err(|_|remote_error())?.into_iter().collect();
        let wanted=fresh_unseen(baseline,uid_validity,&unseen);
        let mut fresh=Vec::new();
        if !wanted.is_empty() {
            let list=wanted.iter().map(u32::to_string).collect::<Vec<_>>().join(",");
            let fetched=session.uid_fetch(list,"(UID FLAGS RFC822.HEADER RFC822.SIZE)").map_err(|_|remote_error())?;
            fresh=fetched.iter().filter_map(|item|message_header(item,"INBOX",uid_validity)).collect();
            fresh.sort_by(|a,b|b.uid.cmp(&a.uid));
        }
        Ok(InboxSnapshot{uid_validity,uid_next,exists:mailbox.exists,unseen:unseen.len() as u32,fresh})
    })
}
/// Name of the sender for a notification: `"Ada Lovelace" <ada@x.org>` gives `Ada Lovelace`.
pub(crate) fn sender_name(from:&str)->String {
    let from=from.trim();
    let name=from.split('<').next().unwrap_or("").trim().trim_matches('"').trim();
    if !name.is_empty() { return name.to_string(); }
    from.trim_start_matches('<').trim_end_matches('>').trim().to_string()
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
        let plain=text_part(&parsed,0);
        let html_only=plain.is_none();
        // HTML is converted to inert text: no HTML, CSS, scripts or remote assets
        // are ever interpreted by the WebView.
        let body=plain.or_else(||html_part(&parsed,0));
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
    pub subject:String,pub body:String,pub revision:u32,pub updated_at:String,
    #[serde(default)] pub attachments:Vec<String>
}
#[derive(Debug,Deserialize)]
pub struct MailDraftInput {
    pub id:Option<String>,pub account_id:String,pub to:String,pub cc:String,
    pub bcc:String,pub subject:String,pub body:String,pub revision:Option<u32>,
    #[serde(default)] pub attachments:Vec<String>
}
fn validate_draft(input:&MailDraftInput)->Result<(),String>{
    if !safe_text(&input.to,1024) || !safe_text(&input.cc,1024) || !safe_text(&input.bcc,1024)
        || !header_text(&input.subject,255) || !safe_text(&input.body,100_000)
        || input.attachments.len()>5 || input.attachments.iter().any(|p|p.len()>4096 || p.contains('\0')){
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
    let result=stmt.query_map([account_id],|r|r.get::<_,String>(0)).map_err(db_error)?
        .map(|s|serde_json::from_str(&s.map_err(db_error)?).map_err(db_error)).collect();
    result
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
    if (input.id.is_some() && previous.is_none()) || previous!=input.revision {
        return Err("Conflitto: la bozza è cambiata. Copia il testo prima di riaprirla.".into());
    }
    let draft=MailDraft{id,account_id:input.account_id,to:input.to,cc:input.cc,bcc:input.bcc,
        subject:input.subject,body:input.body,revision:previous.unwrap_or(0)+1,
        updated_at:Utc::now().to_rfc3339(),attachments:input.attachments};
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
    pub subject:String,pub body:String,pub confirm:bool,
    #[serde(default)] pub attachments:Vec<String>
}
fn load_outbound_attachments(paths:&[String])->Result<Vec<(String,Vec<u8>,ContentType)>,String>{
    if paths.len()>5{return Err("Massimo cinque allegati per messaggio".into());}
    let mut result=Vec::new();
    let mut seen=HashSet::new();
    let mut total=0u64;
    for path in paths{
        if !seen.insert(path){return Err("Allegati duplicati".into());}
        let file=Path::new(path);
        let meta=fs::symlink_metadata(file).map_err(|_|"Allegato non disponibile".to_string())?;
        if !meta.file_type().is_file() || meta.file_type().is_symlink() || meta.len()==0 || meta.len()>5*1024*1024{
            return Err("Allegato non valido: consentiti file ordinari fino a 5 MiB".into());
        }
        total+=meta.len();
        if total>15*1024*1024{return Err("Allegati troppo grandi (limite 15 MiB)".into());}
        let name=file.file_name().and_then(|s|s.to_str())
            .ok_or_else(||"Nome allegato non valido".to_string())?.to_string();
        if name.len()>180 || name.chars().any(|c|c.is_control() || ['/', '\\','<','>'].contains(&c)){
            return Err("Nome allegato non valido".into());
        }
        let bytes=fs::read(file).map_err(|_|"Allegato non leggibile".to_string())?;
        if bytes.len() as u64 != meta.len(){return Err("Allegato modificato durante la lettura".into());}
        let mime=match file.extension().and_then(|x|x.to_str()).unwrap_or("").to_ascii_lowercase().as_str(){
            "jpg"|"jpeg"=>"image/jpeg",
            "png"=>"image/png","webp"=>"image/webp","gif"=>"image/gif",
            "pdf"=>"application/pdf","txt"=>"text/plain","csv"=>"text/csv",
            "zip"=>"application/zip","docx"=>"application/vnd.openxmlformats-officedocument.wordprocessingml.document",
            "xlsx"=>"application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
            "pptx"=>"application/vnd.openxmlformats-officedocument.presentationml.presentation",
            _=>"application/octet-stream"
        };
        let kind=mime.parse::<ContentType>().map_err(|_|"Tipo allegato non valido".to_string())?;
        result.push((name,bytes,kind));
    }
    Ok(result)
}
#[derive(Debug,Serialize)]
pub struct MailSendResult { pub delivery:String,pub sent_copy:String }
#[tauri::command]
pub async fn mail_send(db:State<'_,Database>,input:MailSendInput)->Result<MailSendResult,String>{
    if !input.confirm{return Err("L'invio richiede una conferma esplicita".into());}
    if !header_text(&input.subject,255)||input.subject.trim().is_empty()
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
        // Perform all local file checks before connecting or submitting SMTP DATA.
        let attachments=load_outbound_attachments(&input.attachments)?;
        let mailbox:Mailbox=if account.display_name.trim().is_empty(){account.email.parse()}
            else{format!("{} <{}>",account.display_name.replace(['<','>'], ""),account.email).parse()}
            .map_err(|_|"Mittente non valido".to_string())?;
        let mut builder=Message::builder().from(mailbox).subject(&input.subject);
        for address in to {builder=builder.to(address.parse().map_err(|_|"Destinatario non valido".to_string())?);}
        for address in cc {builder=builder.cc(address.parse().map_err(|_|"Destinatario CC non valido".to_string())?);}
        for address in bcc {builder=builder.bcc(address.parse().map_err(|_|"Destinatario CCN non valido".to_string())?);}
        let message=if attachments.is_empty(){
            builder.header(ContentType::TEXT_PLAIN).body(input.body)
        }else{
            let mut multipart=MultiPart::mixed().singlepart(SinglePart::plain(input.body));
            for (name,bytes,content_type) in attachments {
                multipart=multipart.singlepart(Attachment::new(name).body(bytes,content_type));
            }
            builder.multipart(multipart)
        }.map_err(|_|"Impossibile preparare il messaggio".to_string())?;
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
        assert!(!header_text("subject\r\nBcc:other@example.com",255));
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
            cc:String::new(),bcc:String::new(),subject:"Hello".into(),body:"ok".into(),revision:None,attachments:vec![]};
        assert!(validate_draft(&draft).is_ok());
    }
}


#[cfg(test)]
mod folder_tests {
    use super::*;

    #[test]
    fn folder_names_are_decoded_and_shown_without_technical_prefixes() {
        assert_eq!(imap_utf7_decode("Posta &AOk-"), "Posta é");
        assert_eq!(imap_utf7_decode("Caf&AOk-s &- co"), "Cafés & co");
        assert_eq!(imap_utf7_decode("&2D3eAA-"), "\u{1F600}", "surrogate pairs");
        assert_eq!(imap_utf7_decode("Sent"), "Sent");
        assert_eq!(imap_utf7_decode("rotto &AOk"), "rotto &AOk", "an unterminated sequence is left as it is");
        assert_eq!(folder_display("INBOX.Sent", Some(".")), "Sent");
        assert_eq!(folder_display("INBOX/Clienti/Roma", Some("/")), "Clienti / Roma");
        assert_eq!(folder_display("[Gmail]/Sent Mail", Some("/")), "Sent Mail");
        assert_eq!(folder_display("INBOX", Some("/")), "INBOX");
        assert_eq!(folder_display("&AMk-l&AOk-ments envoy&AOk-s", None), "Éléments envoyés");
    }

    #[test]
    fn roles_come_from_special_use_flags_then_from_names() {
        let none: Vec<String> = vec![];
        assert_eq!(folder_role("INBOX", "INBOX", &none), "inbox");
        assert_eq!(folder_role("X", "Qualunque", &["\\Sent".into()]), "sent", "flags win over names");
        assert_eq!(folder_role("X", "Bozze", &["\\HasNoChildren".into(), "\\Drafts".into()]), "drafts");
        for (display, role) in [("Sent", "sent"), ("Posta inviata", "sent"), ("Elementi inviati", "sent"), ("Bozze", "drafts"), ("Drafts", "drafts"),
            ("Cestino", "trash"), ("Trash", "trash"), ("Deleted Items", "trash"), ("Spam", "junk"), ("Posta indesiderata", "junk"),
            ("Archivio", "archive"), ("Archive", "archive"), ("Clienti", "other"), ("Binario", "other")] {
            assert_eq!(folder_role("INBOX.X", display, &none), role, "{display}");
        }
    }

    #[test]
    fn only_one_mailbox_can_be_connected() {
        assert!(one_mailbox_only(0, true).is_ok(), "the first mailbox");
        assert!(one_mailbox_only(1, false).is_ok(), "editing the connected mailbox");
        let refused = one_mailbox_only(1, true).unwrap_err();
        assert!(refused.contains("una sola casella"), "{refused}");
    }

    #[test]
    fn only_unread_mail_that_arrived_since_the_last_check_is_new() {
        assert!(fresh_unseen(None, 7, &[1, 2, 3]).is_empty(), "the first check only sets the baseline");
        assert!(fresh_unseen(Some((7, 10)), 8, &[11, 12]).is_empty(), "a renumbered mailbox never notifies");
        assert_eq!(fresh_unseen(Some((7, 10)), 7, &[3, 9, 10, 12, 11]), vec![12, 11, 10], "old unread mail is not new; newest first");
        assert_eq!(fresh_unseen(Some((7, 10)), 7, &(10..30).collect::<Vec<_>>()).len(), 5, "a burst is capped");
        assert!(fresh_unseen(Some((7, 10)), 7, &[]).is_empty());
    }

    #[test]
    fn the_sender_is_shown_by_name_when_there_is_one() {
        assert_eq!(sender_name("\"Ada Lovelace\" <ada@example.org>"), "Ada Lovelace");
        assert_eq!(sender_name("Ada <ada@example.org>"), "Ada");
        assert_eq!(sender_name("<ada@example.org>"), "ada@example.org");
        assert_eq!(sender_name("ada@example.org"), "ada@example.org");
        assert_eq!(sender_name(""), "");
    }
}
