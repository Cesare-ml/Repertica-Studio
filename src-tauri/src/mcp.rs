//! Local, read-only Model Context Protocol (MCP) server over STDIO.
//!
//! Started as `Repertica Studio --mcp`. It reads JSON-RPC messages (one per line) from
//! standard input and answers on standard output; it never opens a network port, never
//! writes to the archive and never exposes personal contact details or image bytes.
use crate::{db::Database, util::display_path};
use rusqlite::{params, Connection, OpenFlags};
use serde_json::{json, Map, Value};
use std::{
    io::{BufRead, Write},
    path::{Path, PathBuf},
    time::Duration,
};
use tauri::State;

/// Schema written by this version of the app (`PRAGMA user_version`).
const SUPPORTED_SCHEMA: i64 = crate::db::SCHEMA_VERSION;
const PROTOCOLS: [&str; 3] = ["2025-06-18", "2025-03-26", "2024-11-05"];
const INSTRUCTIONS: &str = "Archivio locale di Repertica Studio in SOLA LETTURA: opere, collezionisti, \
vendite e certificati. Non è possibile modificare nulla. Email, telefono, indirizzi, note private \
e fotografie non sono esposti. Gli importi sono in centesimi di euro (campi *_cents) e in testo (*_eur).";

#[derive(Debug)]
pub struct Archive {
    conn: Connection,
}

impl Archive {
    /// Open the archive strictly read-only. SQLite refuses every write on this connection.
    pub fn open(directory: &Path) -> Result<Archive, String> {
        let file = directory.join("repertica.sqlite");
        if !file.is_file() {
            return Err(format!(
                "Nessun archivio in {}: apri prima Repertica Studio almeno una volta",
                display_path(directory)
            ));
        }
        let conn = Connection::open_with_flags(
            &file,
            OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NO_MUTEX,
        )
        .map_err(|e| format!("Impossibile aprire l'archivio: {e}"))?;
        conn.busy_timeout(Duration::from_secs(5)).map_err(|e| e.to_string())?;
        conn.pragma_update(None, "query_only", true).map_err(|e| e.to_string())?;
        let version: i64 = conn
            .pragma_query_value(None, "user_version", |r| r.get(0))
            .map_err(|e| e.to_string())?;
        if version != SUPPORTED_SCHEMA {
            return Err(format!(
                "Schema archivio {version} non supportato dal server MCP (atteso {SUPPORTED_SCHEMA}): \
                 apri Repertica Studio per aggiornare l'archivio o aggiorna l'applicazione"
            ));
        }
        Ok(Archive { conn })
    }
}

// ---------------------------------------------------------------- helpers

fn sql<T>(result: rusqlite::Result<T>) -> Result<T, String> {
    result.map_err(|e| format!("Errore di lettura dell'archivio: {e}"))
}

/// Lower-case and strip common Latin accents so that "Cafè" matches "cafe".
fn fold(text: &str) -> String {
    text.chars()
        .flat_map(char::to_lowercase)
        .map(|c| match c {
            'à' | 'á' | 'â' | 'ã' | 'ä' | 'å' => 'a',
            'ç' => 'c',
            'è' | 'é' | 'ê' | 'ë' => 'e',
            'ì' | 'í' | 'î' | 'ï' => 'i',
            'ñ' => 'n',
            'ò' | 'ó' | 'ô' | 'õ' | 'ö' => 'o',
            'ù' | 'ú' | 'û' | 'ü' => 'u',
            'ý' | 'ÿ' => 'y',
            other => other,
        })
        .collect()
}

/// Words are matched anywhere; text in double quotes is matched as consecutive words.
fn search_terms(query: &str) -> Vec<String> {
    let mut terms = Vec::new();
    let mut current = String::new();
    let mut quoted = false;
    for c in query.chars().chain(std::iter::once(' ')) {
        if c == '"' {
            if !current.trim().is_empty() {
                terms.push(fold(current.trim()));
            }
            current.clear();
            quoted = !quoted;
        } else if c.is_whitespace() && !quoted {
            if !current.is_empty() {
                terms.push(fold(&current));
            }
            current.clear();
        } else {
            current.push(c);
        }
    }
    terms
}

fn matches_all(haystack: &str, terms: &[String]) -> bool {
    let haystack = fold(haystack);
    terms.iter().all(|term| haystack.contains(term.as_str()))
}

fn eur(cents: i64) -> String {
    let sign = if cents < 0 { "-" } else { "" };
    format!("{sign}{}.{:02}", cents.abs() / 100, cents.abs() % 100)
}

fn text_arg<'a>(args: &'a Value, key: &str) -> Option<&'a str> {
    args.get(key).and_then(Value::as_str).map(str::trim).filter(|v| !v.is_empty())
}

fn limit_arg(args: &Value, default: usize, max: usize) -> Result<usize, String> {
    match args.get("limit") {
        None | Some(Value::Null) => Ok(default),
        Some(v) => match v.as_u64() {
            Some(n) if (1..=max as u64).contains(&n) => Ok(n as usize),
            _ => Err(format!("limit deve essere un intero tra 1 e {max}")),
        },
    }
}

fn excerpt(text: &Option<String>, max: usize) -> Value {
    match text {
        None => Value::Null,
        Some(t) if t.chars().count() <= max => json!(t),
        Some(t) => json!(format!("{}…", t.chars().take(max).collect::<String>())),
    }
}

// ---------------------------------------------------------------- tools

fn overview(archive: &Archive) -> Result<Value, String> {
    let c = &archive.conn;
    let count = |table: &str| -> Result<i64, String> {
        sql(c.query_row(&format!("SELECT count(*) FROM {table}"), [], |r| r.get(0)))
    };
    let revenue: i64 = sql(c.query_row("SELECT COALESCE(SUM(amount_cents),0) FROM sales", [], |r| r.get(0)))?;
    let received: i64 = sql(c.query_row("SELECT COALESCE(SUM(amount_cents),0) FROM sale_receipts", [], |r| r.get(0)))?;
    let mut by_status = Map::new();
    let mut statement = sql(c.prepare("SELECT status,count(*) FROM artworks GROUP BY status"))?;
    let rows = sql(statement.query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, i64>(1)?))))?;
    for row in rows {
        let (status, n) = sql(row)?;
        by_status.insert(status, json!(n));
    }
    Ok(json!({
        "artworks": count("artworks")?, "artworks_by_status": by_status,
        "collectors": count("collectors")?, "sales": count("sales")?,
        "certificates": count("certificates")?,
        "sales_total_cents": revenue, "sales_total_eur": eur(revenue),
        "recorded_receipts_cents": received, "recorded_receipts_eur": eur(received),
        "outstanding_cents": revenue - received, "outstanding_eur": eur(revenue - received),
        "currency": "EUR", "schema_version": SUPPORTED_SCHEMA
    }))
}

fn search_artworks(archive: &Archive, args: &Value) -> Result<Value, String> {
    let terms = text_arg(args, "query").map(search_terms).unwrap_or_default();
    let status = text_arg(args, "status");
    if let Some(s) = status {
        if !["available", "reserved", "sold", "archived"].contains(&s) {
            return Err("status deve essere available, reserved, sold o archived".into());
        }
    }
    let limit = limit_arg(args, 20, 50)?;
    let mut statement = sql(archive.conn.prepare(
        "SELECT a.id,a.code,a.title,a.year,a.technique,a.support,a.width_cm,a.height_cm,a.depth_cm,
                a.description,a.status,a.updated_at,
                (SELECT c.name FROM sales s JOIN collectors c ON c.id=s.collector_id WHERE s.artwork_id=a.id)
         FROM artworks a ORDER BY a.updated_at DESC, a.title COLLATE NOCASE",
    ))?;
    let rows = sql(statement.query_map([], |r| {
        Ok((
            r.get::<_, String>(0)?, r.get::<_, String>(1)?, r.get::<_, String>(2)?,
            r.get::<_, Option<String>>(3)?, r.get::<_, Option<String>>(4)?, r.get::<_, Option<String>>(5)?,
            [r.get::<_, Option<f64>>(6)?, r.get::<_, Option<f64>>(7)?, r.get::<_, Option<f64>>(8)?],
            r.get::<_, Option<String>>(9)?, r.get::<_, String>(10)?,
            r.get::<_, String>(11)?, r.get::<_, Option<String>>(12)?,
        ))
    }))?;
    let (mut total, mut items) = (0_usize, Vec::new());
    for row in rows {
        let (id, code, title, year, technique, support, size, description, state, updated, buyer) = sql(row)?;
        if status.is_some_and(|s| s != state) {
            continue;
        }
        let haystack = [
            Some(code.as_str()), Some(title.as_str()), year.as_deref(), technique.as_deref(),
            support.as_deref(), description.as_deref(), buyer.as_deref(),
        ]
        .into_iter().flatten().collect::<Vec<_>>().join("\n");
        if !matches_all(&haystack, &terms) {
            continue;
        }
        total += 1;
        if items.len() < limit {
            items.push(json!({
                "id": id, "code": code, "title": title, "year": year, "technique": technique,
                "support": support, "height_cm": size[1], "width_cm": size[0], "depth_cm": size[2],
                "status": state, "buyer": buyer,
                "description_excerpt": excerpt(&description, 300), "updated_at": updated
            }));
        }
    }
    Ok(json!({ "total_matches": total, "returned": items.len(), "artworks": items }))
}

fn get_artwork(archive: &Archive, args: &Value) -> Result<Value, String> {
    let key = text_arg(args, "id").or_else(|| text_arg(args, "code"))
        .ok_or("Indica id oppure code dell'opera")?;
    let c = &archive.conn;
    let found = c.query_row(
        "SELECT id,code,title,year,technique,support,width_cm,height_cm,depth_cm,description,status,
                image_path IS NOT NULL,created_at,updated_at,signed,final_varnish,framed,artist_coefficient_override,fixed_price_cents
         FROM artworks WHERE id=?1 OR code=?1 COLLATE NOCASE",
        params![key],
        |r| {
            Ok((
                r.get::<_, String>(0)?, r.get::<_, String>(1)?, r.get::<_, String>(2)?,
                r.get::<_, Option<String>>(3)?, r.get::<_, Option<String>>(4)?, r.get::<_, Option<String>>(5)?,
                [r.get::<_, Option<f64>>(6)?, r.get::<_, Option<f64>>(7)?, r.get::<_, Option<f64>>(8)?],
                r.get::<_, Option<String>>(9)?, r.get::<_, String>(10)?,
                r.get::<_, bool>(11)?, r.get::<_, String>(12)?, r.get::<_, String>(13)?,
                [r.get::<_, bool>(14)?, r.get::<_, bool>(15)?, r.get::<_, bool>(16)?],
                r.get::<_, Option<f64>>(17)?, r.get::<_, Option<i64>>(18)?,
            ))
        },
    );
    let (id, code, title, year, technique, support, size, description, status, has_cover, created, updated, flags, own_coefficient, fixed_price) =
        match found {
            Ok(v) => v,
            Err(rusqlite::Error::QueryReturnedNoRows) => return Err("Opera non trovata".into()),
            Err(e) => return Err(format!("Errore di lettura dell'archivio: {e}")),
        };
    let (rate, artist_coefficient): (i64, f64) = sql(c.query_row("SELECT base_price_cm2_cents,artist_coefficient FROM studio_settings WHERE singleton=1", [], |r| Ok((r.get(0)?, r.get(1)?))))?;
    let price = crate::finance::price_cents(size[0], size[1], rate, artist_coefficient, own_coefficient, fixed_price);
    let gallery: i64 = sql(c.query_row("SELECT count(*) FROM artwork_images WHERE artwork_id=?1", params![id], |r| r.get(0)))?;
    let sale = c.query_row(
        "SELECT s.id,s.sold_at,s.amount_cents,c.name,
                (SELECT COALESCE(SUM(amount_cents),0) FROM sale_receipts WHERE sale_id=s.id)
         FROM sales s JOIN collectors c ON c.id=s.collector_id WHERE s.artwork_id=?1",
        params![id],
        |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?, r.get::<_, i64>(2)?, r.get::<_, String>(3)?, r.get::<_, i64>(4)?)),
    );
    let sale = match sale {
        Ok((sale_id, sold_at, amount, buyer, received)) => json!({
            "id": sale_id, "sold_at": sold_at, "collector_name": buyer,
            "amount_cents": amount, "amount_eur": eur(amount),
            "received_cents": received, "outstanding_cents": amount - received,
            "outstanding_eur": eur(amount - received)
        }),
        Err(rusqlite::Error::QueryReturnedNoRows) => Value::Null,
        Err(e) => return Err(format!("Errore di lettura dell'archivio: {e}")),
    };
    let mut statement = sql(c.prepare("SELECT serial,issued_at FROM certificates WHERE artwork_id=?1 ORDER BY created_at"))?;
    let certificates = sql(statement.query_map(params![id], |r| Ok(json!({
        "serial": r.get::<_, String>(0)?, "issued_at": r.get::<_, String>(1)?
    }))))?
    .collect::<Result<Vec<_>, _>>();
    let certificates = sql(certificates)?;
    Ok(json!({
        "id": id, "code": code, "title": title, "year": year, "technique": technique, "support": support,
        "height_cm": size[1], "width_cm": size[0], "depth_cm": size[2],
        "description": excerpt(&description, 8000), "status": status,
        "signed": flags[0], "final_varnish_applied": flags[1], "framed": flags[2],
        "price_cents": price, "price_eur": price.map(eur),
        "price_source": if fixed_price.is_some() { "fixed_price" } else if own_coefficient.is_some() { "own_coefficient" } else { "artist_coefficient" },
        "artist_coefficient_override": own_coefficient,
        "has_cover_photo": has_cover, "gallery_photo_count": gallery,
        "sale": sale, "certificates": certificates, "created_at": created, "updated_at": updated
    }))
}

fn list_collectors(archive: &Archive, args: &Value) -> Result<Value, String> {
    let terms = text_arg(args, "query").map(search_terms).unwrap_or_default();
    let limit = limit_arg(args, 50, 200)?;
    let mut statement = sql(archive.conn.prepare(
        "SELECT c.id,c.name,
                (SELECT count(*) FROM sales s WHERE s.collector_id=c.id),
                (SELECT COALESCE(SUM(amount_cents),0) FROM sales s WHERE s.collector_id=c.id)
         FROM collectors c ORDER BY c.name COLLATE NOCASE",
    ))?;
    let rows = sql(statement.query_map([], |r| {
        Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?, r.get::<_, i64>(2)?, r.get::<_, i64>(3)?))
    }))?;
    let (mut total, mut items) = (0_usize, Vec::new());
    for row in rows {
        let (id, name, purchases, spent) = sql(row)?;
        if !matches_all(&name, &terms) {
            continue;
        }
        total += 1;
        if items.len() < limit {
            items.push(json!({
                "id": id, "name": name, "purchases": purchases,
                "total_purchases_cents": spent, "total_purchases_eur": eur(spent)
            }));
        }
    }
    Ok(json!({ "total_matches": total, "returned": items.len(), "collectors": items,
        "note": "Contatti e note private dei collezionisti non sono esposti." }))
}

fn list_sales(archive: &Archive, args: &Value) -> Result<Value, String> {
    let limit = limit_arg(args, 50, 200)?;
    let (from, to) = match args.get("year") {
        None | Some(Value::Null) => ("0000-01-01".to_string(), "9999-12-31".to_string()),
        Some(v) => {
            let year = v.as_i64().filter(|y| (1900..=2200).contains(y)).ok_or("year deve essere tra 1900 e 2200")?;
            (format!("{year}-01-01"), format!("{}-01-01", year + 1))
        }
    };
    let mut statement = sql(archive.conn.prepare(
        "SELECT s.id,s.sold_at,a.id,a.code,a.title,c.name,s.amount_cents,
                (SELECT COALESCE(SUM(amount_cents),0) FROM sale_receipts WHERE sale_id=s.id)
         FROM sales s JOIN artworks a ON a.id=s.artwork_id JOIN collectors c ON c.id=s.collector_id
         WHERE s.sold_at >= ?1 AND s.sold_at < ?2
         ORDER BY s.sold_at DESC, s.created_at DESC",
    ))?;
    let rows = sql(statement.query_map(params![from, to], |r| {
        Ok((
            r.get::<_, String>(0)?, r.get::<_, String>(1)?, r.get::<_, String>(2)?, r.get::<_, String>(3)?,
            r.get::<_, String>(4)?, r.get::<_, String>(5)?, r.get::<_, i64>(6)?, r.get::<_, i64>(7)?,
        ))
    }))?;
    let (mut total, mut total_cents, mut items) = (0_usize, 0_i64, Vec::new());
    for row in rows {
        let (id, sold_at, artwork_id, code, title, buyer, amount, received) = sql(row)?;
        total += 1;
        total_cents += amount;
        if items.len() < limit {
            items.push(json!({
                "id": id, "sold_at": sold_at, "artwork_id": artwork_id, "artwork_code": code,
                "artwork_title": title, "collector_name": buyer,
                "amount_cents": amount, "amount_eur": eur(amount),
                "received_cents": received, "outstanding_cents": amount - received,
                "outstanding_eur": eur(amount - received), "currency": "EUR"
            }));
        }
    }
    Ok(json!({ "total_sales": total, "total_cents": total_cents, "total_eur": eur(total_cents),
        "returned": items.len(), "sales": items }))
}

fn list_certificates(archive: &Archive, args: &Value) -> Result<Value, String> {
    let limit = limit_arg(args, 50, 200)?;
    let mut statement = sql(archive.conn.prepare(
        "SELECT c.serial,c.issued_at,a.id,a.code,a.title,c.artist_name,c.collector_name
         FROM certificates c JOIN artworks a ON a.id=c.artwork_id ORDER BY c.created_at DESC",
    ))?;
    let rows = sql(statement.query_map([], |r| Ok(json!({
        "serial": r.get::<_, String>(0)?, "issued_at": r.get::<_, String>(1)?,
        "artwork_id": r.get::<_, String>(2)?, "artwork_code": r.get::<_, String>(3)?,
        "artwork_title": r.get::<_, String>(4)?, "artist_name": r.get::<_, String>(5)?,
        "collector_name": r.get::<_, Option<String>>(6)?
    }))))?
    .collect::<Result<Vec<_>, _>>();
    let all = sql(rows)?;
    let total = all.len();
    let items: Vec<Value> = all.into_iter().take(limit).collect();
    Ok(json!({ "total": total, "returned": items.len(), "certificates": items }))
}

fn tool_definitions() -> Value {
    let limit = |max: u64| json!({"type": "integer", "minimum": 1, "maximum": max, "description": "Numero massimo di risultati"});
    let read_only = json!({"readOnlyHint": true, "destructiveHint": false, "idempotentHint": true, "openWorldHint": false});
    json!([
        {"name": "archive_overview", "title": "Panoramica archivio",
         "description": "Conteggi di opere (per stato), collezionisti, vendite e certificati, con totali venduto, incassato e da incassare.",
         "inputSchema": {"type": "object", "properties": {}, "additionalProperties": false}, "annotations": read_only},
        {"name": "search_artworks", "title": "Cerca opere",
         "description": "Cerca opere per parole (senza distinguere maiuscole e accenti) in codice, titolo, anno, tecnica, supporto, descrizione e acquirente. Le frasi tra virgolette devono comparire come parole consecutive.",
         "inputSchema": {"type": "object", "properties": {
            "query": {"type": "string", "description": "Parole o \"frasi\" da cercare; vuoto = tutte le opere"},
            "status": {"type": "string", "enum": ["available", "reserved", "sold", "archived"]},
            "limit": limit(50)}, "additionalProperties": false}, "annotations": read_only},
        {"name": "get_artwork", "title": "Dettaglio opera",
         "description": "Scheda completa di un'opera (per id o codice) con vendita, saldo, certificati e numero di fotografie. Le immagini non sono incluse.",
         "inputSchema": {"type": "object", "properties": {
            "id": {"type": "string"}, "code": {"type": "string", "description": "Codice opera, senza distinzione di maiuscole"}},
            "additionalProperties": false}, "annotations": read_only},
        {"name": "list_collectors", "title": "Elenco collezionisti",
         "description": "Nomi dei collezionisti con numero e totale degli acquisti. I contatti non sono esposti.",
         "inputSchema": {"type": "object", "properties": {
            "query": {"type": "string"}, "limit": limit(200)}, "additionalProperties": false}, "annotations": read_only},
        {"name": "list_sales", "title": "Elenco vendite",
         "description": "Vendite registrate, dalla più recente, con incassato e residuo. Filtro opzionale per anno di vendita.",
         "inputSchema": {"type": "object", "properties": {
            "year": {"type": "integer", "minimum": 1900, "maximum": 2200}, "limit": limit(200)},
            "additionalProperties": false}, "annotations": read_only},
        {"name": "list_certificates", "title": "Elenco certificati",
         "description": "Certificati emessi (seriale, data, opera, artista), dal più recente.",
         "inputSchema": {"type": "object", "properties": {"limit": limit(200)}, "additionalProperties": false},
         "annotations": read_only}
    ])
}

fn call_tool(archive: &Archive, name: &str, args: &Value) -> Option<Result<Value, String>> {
    Some(match name {
        "archive_overview" => overview(archive),
        "search_artworks" => search_artworks(archive, args),
        "get_artwork" => get_artwork(archive, args),
        "list_collectors" => list_collectors(archive, args),
        "list_sales" => list_sales(archive, args),
        "list_certificates" => list_certificates(archive, args),
        _ => return None,
    })
}

// ---------------------------------------------------------------- JSON-RPC

fn failure(id: Value, code: i64, message: &str) -> Value {
    json!({"jsonrpc": "2.0", "id": id, "error": {"code": code, "message": message}})
}

/// Handle one JSON-RPC line. Notifications (no `id`) never produce a reply.
pub fn handle_message(archive: &Archive, line: &str) -> Option<Value> {
    let message: Value = match serde_json::from_str(line) {
        Ok(v) => v,
        Err(_) => return Some(failure(Value::Null, -32700, "JSON non valido")),
    };
    let Some(object) = message.as_object() else {
        return Some(failure(Value::Null, -32600, "Richiesta non valida (i batch non sono supportati)"));
    };
    let id = object.get("id").cloned()?; // notification or stray response
    let Some(method) = object.get("method").and_then(Value::as_str) else {
        return Some(failure(id, -32600, "Metodo mancante"));
    };
    let params = object.get("params").cloned().unwrap_or_else(|| json!({}));
    let reply = |result: Value| json!({"jsonrpc": "2.0", "id": id.clone(), "result": result});
    Some(match method {
        "initialize" => {
            let requested = params.get("protocolVersion").and_then(Value::as_str).unwrap_or("");
            let version = PROTOCOLS.iter().find(|p| **p == requested).unwrap_or(&PROTOCOLS[0]);
            reply(json!({
                "protocolVersion": version,
                "capabilities": {"tools": {"listChanged": false}},
                "serverInfo": {"name": "repertica-studio", "title": "Repertica Studio (sola lettura)",
                               "version": env!("CARGO_PKG_VERSION")},
                "instructions": INSTRUCTIONS
            }))
        }
        "ping" => reply(json!({})),
        "tools/list" => reply(json!({"tools": tool_definitions()})),
        "tools/call" => {
            let name = params.get("name").and_then(Value::as_str).unwrap_or("");
            let args = params.get("arguments").cloned().unwrap_or_else(|| json!({}));
            if !args.is_object() {
                return Some(failure(id, -32602, "arguments deve essere un oggetto"));
            }
            match call_tool(archive, name, &args) {
                None => failure(id, -32602, &format!("Strumento sconosciuto: {name}")),
                Some(Ok(value)) => reply(json!({
                    "content": [{"type": "text", "text": value.to_string()}], "isError": false
                })),
                Some(Err(message)) => reply(json!({
                    "content": [{"type": "text", "text": message}], "isError": true
                })),
            }
        }
        _ => failure(id, -32601, &format!("Metodo non supportato: {method}")),
    })
}

pub fn serve<R: BufRead, W: Write>(archive: &Archive, input: R, mut output: W) -> std::io::Result<()> {
    for line in input.lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }
        if let Some(response) = handle_message(archive, &line) {
            writeln!(output, "{response}")?;
            output.flush()?;
        }
    }
    Ok(())
}

// ---------------------------------------------------------------- entry points

/// Where the desktop app keeps its archive (Tauri's app-local-data directory, named after the identifier
/// this binary was built with: a development build reads its own archive, never the release's).
pub fn default_data_dir() -> Option<PathBuf> {
    let var = |name: &str| std::env::var_os(name).filter(|v| !v.is_empty()).map(PathBuf::from);
    let base = if cfg!(windows) {
        var("LOCALAPPDATA")?
    } else if cfg!(target_os = "macos") {
        var("HOME")?.join("Library").join("Application Support")
    } else {
        var("XDG_DATA_HOME").or_else(|| var("HOME").map(|h| h.join(".local").join("share")))?
    };
    Some(base.join(crate::identity::IDENTIFIER))
}

/// `Repertica Studio --mcp [--data-dir <cartella>]`; returns the process exit code.
pub fn main(args: &[String]) -> i32 {
    let mut directory = std::env::var_os("REPERTICA_DATA_DIR").filter(|v| !v.is_empty()).map(PathBuf::from);
    let mut iterator = args.iter();
    while let Some(argument) = iterator.next() {
        match argument.as_str() {
            "--data-dir" => match iterator.next() {
                Some(path) => directory = Some(PathBuf::from(path)),
                None => {
                    eprintln!("--data-dir richiede una cartella");
                    return 2;
                }
            },
            other => {
                eprintln!("Argomento sconosciuto: {other}");
                return 2;
            }
        }
    }
    let Some(directory) = directory.or_else(default_data_dir) else {
        eprintln!("Cartella dati non trovata: usa --data-dir");
        return 2;
    };
    let archive = match Archive::open(&directory) {
        Ok(archive) => archive,
        Err(message) => {
            eprintln!("{message}");
            return 1;
        }
    };
    match serve(&archive, std::io::stdin().lock(), std::io::stdout().lock()) {
        Ok(()) => 0,
        Err(error) => {
            eprintln!("Errore di comunicazione: {error}");
            1
        }
    }
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct McpSetup {
    /// Name of the entry in the assistant's configuration (`repertica-studio`, or `repertica-studio-dev`).
    server_name: String,
    command: String,
    args: Vec<String>,
    data_dir: String,
}

/// Information shown in Settings so the user can register this server in an MCP client.
#[tauri::command]
pub fn mcp_setup_info(db: State<'_, Database>) -> Result<McpSetup, String> {
    let executable = std::env::current_exe().map_err(|e| e.to_string())?;
    Ok(McpSetup {
        server_name: crate::mcp_client::server_key(),
        command: display_path(&executable),
        args: vec!["--mcp".into(), "--data-dir".into(), display_path(&db.directory)],
        data_dir: display_path(&db.directory),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture() -> (tempfile::TempDir, Archive) {
        let dir = tempfile::tempdir().unwrap();
        {
            let db = crate::db::open(dir.path().to_path_buf()).unwrap();
            let c = db.conn.lock().unwrap();
            c.execute_batch(
                "INSERT INTO artworks(id,code,title,year,technique,support,description,location,status,created_at,updated_at)
                   VALUES('a1','OP-001','Notte d''estate','2024','olio su tela','tela di lino','Paesaggio notturno','Studio','sold','t','2026-02-01'),
                         ('a2','OP-002','Mattino','2025','acquerello','carta','Luce del mattino','Magazzino','available','t','2026-01-01');
                 INSERT INTO collectors(id,name,email,phone,address,notes,created_at,updated_at)
                   VALUES('c1','José Pérez','segreto@example.org','+39 333 0000000','Via Segreta 1','nota privata','t','t');
                 INSERT INTO sales(id,artwork_id,collector_id,amount_cents,sold_at,created_at)
                   VALUES('s1','a1','c1',150050,'2026-03-10','t');
                 INSERT INTO sale_receipts(id,sale_id,received_at,amount_cents,method,created_at)
                   VALUES('r1','s1','2026-03-11',50000,'cash','t');
                 INSERT INTO certificates(id,artwork_id,serial,issued_at,artist_name,collector_name,snapshot_json,created_at)
                   VALUES('k1','a1','RS-2026-000001','2026-03-12','Artista','José Pérez','{}','t');",
            )
            .unwrap();
        }
        let archive = Archive::open(dir.path()).unwrap();
        (dir, archive)
    }

    fn rpc(archive: &Archive, request: Value) -> Value {
        handle_message(archive, &request.to_string()).expect("a reply")
    }

    fn call(archive: &Archive, name: &str, arguments: Value) -> (bool, Value) {
        let reply = rpc(archive, json!({"jsonrpc": "2.0", "id": 7, "method": "tools/call",
            "params": {"name": name, "arguments": arguments}}));
        let result = &reply["result"];
        let text = result["content"][0]["text"].as_str().unwrap();
        let parsed = serde_json::from_str(text).unwrap_or_else(|_| json!(text));
        (result["isError"].as_bool().unwrap(), parsed)
    }

    #[test]
    fn handshake_negotiates_protocol_and_advertises_only_read_only_tools() {
        let (_dir, archive) = fixture();
        let init = rpc(&archive, json!({"jsonrpc": "2.0", "id": 1, "method": "initialize",
            "params": {"protocolVersion": "2024-11-05", "capabilities": {}, "clientInfo": {"name": "t", "version": "1"}}}));
        assert_eq!(init["result"]["protocolVersion"], "2024-11-05");
        assert_eq!(init["id"], 1);
        let unknown = rpc(&archive, json!({"jsonrpc": "2.0", "id": 2, "method": "initialize", "params": {"protocolVersion": "1999-01-01"}}));
        assert_eq!(unknown["result"]["protocolVersion"], "2025-06-18");
        assert!(handle_message(&archive, r#"{"jsonrpc":"2.0","method":"notifications/initialized"}"#).is_none());
        let tools = rpc(&archive, json!({"jsonrpc": "2.0", "id": 3, "method": "tools/list"}));
        let tools = tools["result"]["tools"].as_array().unwrap();
        assert_eq!(tools.len(), 6);
        assert!(tools.iter().all(|t| t["annotations"]["readOnlyHint"] == true && t["inputSchema"]["type"] == "object"));
        assert_eq!(rpc(&archive, json!({"jsonrpc": "2.0", "id": 4, "method": "ping"}))["result"], json!({}));
    }

    #[test]
    fn protocol_errors_are_reported_without_crashing() {
        let (_dir, archive) = fixture();
        assert_eq!(handle_message(&archive, "{not json").unwrap()["error"]["code"], -32700);
        assert_eq!(handle_message(&archive, "[]").unwrap()["error"]["code"], -32600);
        assert_eq!(rpc(&archive, json!({"jsonrpc": "2.0", "id": 1, "method": "delete_everything"}))["error"]["code"], -32601);
        let unknown = rpc(&archive, json!({"jsonrpc": "2.0", "id": 1, "method": "tools/call", "params": {"name": "drop_table"}}));
        assert_eq!(unknown["error"]["code"], -32602);
        let (is_error, _) = call(&archive, "search_artworks", json!({"limit": 0}));
        assert!(is_error);
        let (is_error, _) = call(&archive, "search_artworks", json!({"status": "deleted"}));
        assert!(is_error);
    }

    #[test]
    fn search_ignores_case_and_accents_and_supports_quoted_phrases() {
        let (_dir, archive) = fixture();
        let (_, hits) = call(&archive, "search_artworks", json!({"query": "NOTTE ESTATE"}));
        assert_eq!(hits["total_matches"], 1);
        assert_eq!(hits["artworks"][0]["code"], "OP-001");
        assert_eq!(hits["artworks"][0]["buyer"], "José Pérez");
        let (_, by_buyer) = call(&archive, "search_artworks", json!({"query": "jose perez"}));
        assert_eq!(by_buyer["total_matches"], 1);
        let (_, phrase) = call(&archive, "search_artworks", json!({"query": "\"olio su tela\""}));
        assert_eq!(phrase["total_matches"], 1);
        let (_, reversed) = call(&archive, "search_artworks", json!({"query": "\"tela su olio\""}));
        assert_eq!(reversed["total_matches"], 0);
        let (_, available) = call(&archive, "search_artworks", json!({"status": "available"}));
        assert_eq!(available["artworks"][0]["code"], "OP-002");
        let (_, all) = call(&archive, "search_artworks", json!({"limit": 1}));
        assert_eq!((all["total_matches"].as_i64(), all["returned"].as_i64()), (Some(2), Some(1)));
    }

    #[test]
    fn artwork_detail_includes_sale_balance_and_certificates_by_code() {
        let (_dir, archive) = fixture();
        let (error, detail) = call(&archive, "get_artwork", json!({"code": "op-001"}));
        assert!(!error);
        assert_eq!(detail["sale"]["amount_eur"], "1500.50");
        assert_eq!(detail["sale"]["outstanding_cents"], 100050);
        assert_eq!(detail["certificates"][0]["serial"], "RS-2026-000001");
        assert_eq!(detail["gallery_photo_count"], 0);
        assert!(call(&archive, "get_artwork", json!({"code": "missing"})).0);
        assert!(call(&archive, "get_artwork", json!({})).0);
    }

    #[test]
    fn totals_balances_and_filters_use_exact_cents() {
        let (_dir, archive) = fixture();
        let (_, overview) = call(&archive, "archive_overview", json!({}));
        assert_eq!(overview["sales_total_eur"], "1500.50");
        assert_eq!(overview["recorded_receipts_cents"], 50000);
        assert_eq!(overview["outstanding_eur"], "1000.50");
        assert_eq!(overview["artworks_by_status"]["available"], 1);
        let (_, sales) = call(&archive, "list_sales", json!({"year": 2026}));
        assert_eq!(sales["total_sales"], 1);
        assert_eq!(sales["sales"][0]["artwork_title"], "Notte d'estate");
        assert_eq!(call(&archive, "list_sales", json!({"year": 2025})).1["total_sales"], 0);
        assert!(call(&archive, "list_sales", json!({"year": 1800})).0);
        let (_, certificates) = call(&archive, "list_certificates", json!({}));
        assert_eq!(certificates["total"], 1);
        assert_eq!(eur(-5), "-0.05");
    }

    #[test]
    fn private_contact_details_never_leave_the_archive() {
        let (_dir, archive) = fixture();
        let (_, collectors) = call(&archive, "list_collectors", json!({"query": "perez"}));
        assert_eq!(collectors["collectors"][0]["purchases"], 1);
        // Exercise every tool and make sure no private field value appears in any reply.
        let mut everything = String::new();
        for (tool, args) in [
            ("archive_overview", json!({})), ("search_artworks", json!({})), ("get_artwork", json!({"id": "a1"})),
            ("list_collectors", json!({})), ("list_sales", json!({})), ("list_certificates", json!({})),
        ] {
            everything.push_str(&call(&archive, tool, args).1.to_string());
        }
        for secret in ["segreto@example.org", "+39 333", "Via Segreta", "nota privata"] {
            assert!(!everything.contains(secret), "{secret} leaked");
        }
    }

    #[test]
    fn the_connection_is_read_only_at_the_database_level() {
        let (_dir, archive) = fixture();
        assert!(archive.conn.execute("DELETE FROM artworks", []).is_err());
        assert!(archive.conn.execute("UPDATE sales SET amount_cents=1", []).is_err());
        assert!(archive.conn.execute("CREATE TABLE x(a)", []).is_err());
    }

    #[test]
    fn opening_reports_missing_or_incompatible_archives() {
        let empty = tempfile::tempdir().unwrap();
        assert!(Archive::open(empty.path()).unwrap_err().contains("Nessun archivio"));
        let old = tempfile::tempdir().unwrap();
        {
            let c = Connection::open(old.path().join("repertica.sqlite")).unwrap();
            c.pragma_update(None, "user_version", 3).unwrap();
        }
        assert!(Archive::open(old.path()).unwrap_err().contains("non supportato"));
    }

    #[test]
    fn serve_answers_one_line_per_request_and_skips_notifications() {
        let (_dir, archive) = fixture();
        let input = "{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"ping\"}\n\n\
                     {\"jsonrpc\":\"2.0\",\"method\":\"notifications/initialized\"}\n\
                     {\"jsonrpc\":\"2.0\",\"id\":2,\"method\":\"ping\"}\n";
        let mut output = Vec::new();
        serve(&archive, input.as_bytes(), &mut output).unwrap();
        let lines: Vec<_> = String::from_utf8(output).unwrap().lines().map(String::from).collect();
        assert_eq!(lines.len(), 2);
        assert!(lines.iter().all(|l| !l.contains('\n') && serde_json::from_str::<Value>(l).is_ok()));
    }

    #[test]
    fn default_data_directory_matches_the_desktop_app_identifier() {
        if let Some(path) = default_data_dir() {
            assert!(path.ends_with(crate::identity::IDENTIFIER));
        }
        let config: Value = serde_json::from_str(include_str!("../tauri.conf.json")).unwrap();
        assert_eq!(config["identifier"], crate::identity::RELEASE_IDENTIFIER);
    }
}
