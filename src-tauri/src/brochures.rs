//! Offline brochure editor and A4 PDF exporter.
//! Adapted from Gestionale-Opere's Publication Engine (schema v2 / Classic,
//! Full Bleed and Hero Details concepts). No HTTP, cloud account or Chromium.
use crate::db::Database;
use chrono::Utc;
use image::codecs::jpeg::JpegEncoder;
use rusqlite::{params, OptionalExtension};
use serde::{Deserialize, Serialize};
use std::{collections::HashSet, fs, path::{Component, Path}, sync::MutexGuard};
use tauri::State;
use uuid::Uuid;

fn error(e: impl std::fmt::Display) -> String { crate::util::friendly(e) }

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct BrochureItem {
    pub artwork_id: String,
    pub template: String,
    pub gallery_image_id: Option<String>,
    pub title_override: String,
    pub description_override: String,
    pub include_description: bool,
}
impl Default for BrochureItem {
    fn default() -> Self {
        Self { artwork_id: String::new(), template: "classic".into(), gallery_image_id: None,
            title_override: String::new(), description_override: String::new(), include_description: true }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Brochure {
    pub id: String,
    pub title: String,
    pub subtitle: String,
    pub artist: String,
    pub introduction: String,
    pub contact_email: String,
    pub contact_website: String,
    pub contact_instagram: String,
    pub language: String,
    pub items: Vec<BrochureItem>,
    pub created_at: String,
    pub updated_at: String,
}
fn bounded(value: &str, maximum: usize, label: &str) -> Result<(), String> {
    if value.chars().count() > maximum { Err(format!("{label}: testo troppo lungo")) } else { Ok(()) }
}
fn validate(value: &Brochure) -> Result<(), String> {
    if value.title.trim().is_empty() { return Err("Inserisci un titolo per la brochure".into()); }
    if !["it","en","es","fr","de"].contains(&value.language.as_str()) {
        return Err("Lingua brochure non valida".into());
    }
    for (name, value, max) in [
        ("Titolo", value.title.as_str(), 200), ("Sottotitolo", &value.subtitle, 400),
        ("Artista", &value.artist, 200), ("Introduzione", &value.introduction, 10_000),
        ("Email", &value.contact_email, 250), ("Sito", &value.contact_website, 300),
        ("Instagram", &value.contact_instagram, 300)
    ] { bounded(value, max, name)?; }
    if value.items.len() > 100 { return Err("La brochure può contenere al massimo 100 opere".into()); }
    let mut seen = HashSet::new();
    for item in &value.items {
        if item.artwork_id.trim().is_empty() || !seen.insert(&item.artwork_id) {
            return Err("La selezione contiene opere duplicate o senza identificativo".into());
        }
        if !["classic", "full_bleed", "details"].contains(&item.template.as_str()) {
            return Err("Layout brochure non consentito".into());
        }
        bounded(&item.title_override, 300, "Titolo opera")?;
        bounded(&item.description_override, 10_000, "Descrizione opera")?;
    }
    Ok(())
}
fn connection<'a>(db: &'a Database) -> Result<MutexGuard<'a, rusqlite::Connection>, String> {
    db.conn.lock().map_err(error)
}
#[tauri::command]
pub fn list_brochures(db: State<'_, Database>) -> Result<Vec<Brochure>, String> {
    let conn=connection(&db)?;
    let mut statement=conn.prepare("SELECT payload_json FROM brochures ORDER BY updated_at DESC").map_err(error)?;
    let values=statement.query_map([], |row| row.get::<_, String>(0)).map_err(error)?;
    values.map(|item| serde_json::from_str::<Brochure>(&item.map_err(error)?).map_err(error)).collect()
}
#[tauri::command]
pub fn save_brochure(db: State<'_, Database>, mut input: Brochure) -> Result<Brochure, String> {
    validate(&input)?;
    let now=Utc::now().to_rfc3339();
    if input.id.is_empty() { input.id=Uuid::new_v4().to_string(); }
    bounded(&input.id, 80, "ID")?;
    let conn=connection(&db)?;
    let created=conn.query_row("SELECT created_at FROM brochures WHERE id=?1", [&input.id],
        |row| row.get::<_, String>(0)).optional().map_err(error)?;
    input.created_at=created.unwrap_or_else(|| now.clone());
    input.updated_at=now;
    let json=serde_json::to_string(&input).map_err(error)?;
    conn.execute("INSERT INTO brochures(id,title,payload_json,created_at,updated_at) VALUES(?1,?2,?3,?4,?5)
        ON CONFLICT(id) DO UPDATE SET title=excluded.title,payload_json=excluded.payload_json,updated_at=excluded.updated_at",
        params![input.id,input.title,json,input.created_at,input.updated_at]).map_err(error)?;
    Ok(input)
}
#[tauri::command]
pub fn delete_brochure(db: State<'_, Database>, id: String) -> Result<(), String> {
    let conn=connection(&db)?;
    conn.execute("DELETE FROM brochures WHERE id=?1", [id]).map_err(error)?;
    Ok(())
}

struct ArtPage {
    title: String, code: String, year: Option<String>, technique: Option<String>,
    support: Option<String>, width: Option<f64>, height: Option<f64>,
    description: Option<String>, image_path: Option<String>,
}
fn selected_artwork(db: &Database, id: &str, gallery_id: Option<&str>) -> Result<ArtPage, String> {
    let conn=connection(db)?;
    let mut artwork=conn.query_row(
        "SELECT title,code,year,technique,support,width_cm,height_cm,description,image_path
          FROM artworks WHERE id=?1", [id],
        |row| Ok(ArtPage {title:row.get(0)?,code:row.get(1)?,year:row.get(2)?,
            technique:row.get(3)?,support:row.get(4)?,width:row.get(5)?,height:row.get(6)?,
            description:row.get(7)?,image_path:row.get(8)?})
    ).optional().map_err(error)?.ok_or_else(|| "Una delle opere della brochure non è più nell’archivio".to_string())?;
    if let Some(image_id)=gallery_id {
        artwork.image_path=conn.query_row(
            "SELECT image_path FROM artwork_images WHERE artwork_id=?1 AND id=?2",
            params![id,image_id], |row| row.get(0)
        ).optional().map_err(error)?
        .ok_or_else(|| format!("Fotografia aggiuntiva non trovata per {}",artwork.title)).map(Some)?;
    }
    Ok(artwork)
}
struct Photo { jpg: Vec<u8>, width:u32, height:u32 }
fn load_photo(db: &Database, relative: &str) -> Result<Photo, String> {
    let path=Path::new(relative);
    let mut parts=path.components();
    if !matches!(parts.next(),Some(Component::Normal(root)) if root=="media")
        || !matches!(parts.next(),Some(Component::Normal(_))) || parts.next().is_some()
        || relative.contains('\\') {
        return Err("Percorso fotografia non valido".into());
    }
    let media_dir=db.directory.join("media");
    if fs::symlink_metadata(&media_dir).map_err(error)?.file_type().is_symlink() {
        return Err("La cartella delle fotografie non è attendibile".into());
    }
    let base=fs::canonicalize(&media_dir).map_err(error)?;
    let full=db.directory.join(path);
    let meta=fs::symlink_metadata(&full).map_err(error)?;
    if !meta.file_type().is_file() || meta.file_type().is_symlink() || meta.len()>32*1024*1024 {
        return Err("Fotografia non valida o troppo grande".into());
    }
    if fs::canonicalize(&full).map_err(error)?.parent()!=Some(base.as_path()) {
        return Err("Fotografia fuori dall'archivio".into());
    }
    let (width,height)=image::image_dimensions(&full).map_err(error)?;
    if (width as u64)*(height as u64)>36_000_000 {
        return Err("Immagine troppo grande per la brochure".into());
    }
    let decoded=image::open(&full).map_err(error)?;
    let normalized=decoded.thumbnail(2400,2400).to_rgb8();
    let (w,h)=normalized.dimensions();
    let mut jpg=Vec::new();
    JpegEncoder::new_with_quality(&mut jpg,86).encode_image(&image::DynamicImage::ImageRgb8(normalized)).map_err(error)?;
    Ok(Photo {jpg,width:w,height:h})
}
struct Page { drawing: String, photo: Option<Photo> }
fn win_ansi(c: char)->u8 {
    match c {
        '\u{20AC}'=>0x80, '\u{201A}'=>0x82, '\u{0192}'=>0x83, '\u{201E}'=>0x84,
        '\u{2026}'=>0x85, '\u{2020}'=>0x86, '\u{2021}'=>0x87, '\u{02C6}'=>0x88,
        '\u{2030}'=>0x89, '\u{0160}'=>0x8A, '\u{2039}'=>0x8B, '\u{0152}'=>0x8C,
        '\u{017D}'=>0x8E, '\u{2018}'=>0x91, '\u{2019}'=>0x92, '\u{201C}'=>0x93,
        '\u{201D}'=>0x94, '\u{2022}'=>0x95, '\u{2013}'=>0x96, '\u{2014}'=>0x97,
        '\u{02DC}'=>0x98, '\u{2122}'=>0x99, '\u{0161}'=>0x9A, '\u{203A}'=>0x9B,
        '\u{0153}'=>0x9C, '\u{017E}'=>0x9E, '\u{0178}'=>0x9F,
        c if (c as u32)>=32 && (c as u32)<=255 => c as u8,
        _ => b'?'
    }
}
fn hex_text(s:&str)->String {
    s.chars().map(|c| format!("{:02X}",win_ansi(c))).collect()
}
fn put_text(out:&mut String, text:&str, x:f32,y:f32,size:f32,bold:bool) {
    if !text.is_empty() {
        let font=if bold {"F2"}else{"F1"};
        out.push_str(&format!("BT /{font} {size:.1} Tf 1 0 0 1 {x:.1} {y:.1} Tm <{}> Tj ET\n",hex_text(text)));
    }
}
fn lines(input:&str, width:usize)->Vec<String> {
    let mut result=Vec::new();
    for paragraph in input.split('\n') {
        if paragraph.trim().is_empty() {result.push(String::new());continue;}
        let mut current=String::new();
        for word in paragraph.split_whitespace() {
            // Unicode-aware split prevents any individual long URL from spilling off-page.
            let chunks:Vec<String>=word.chars().collect::<Vec<_>>().chunks(width.max(1))
                .map(|part| part.iter().collect()).collect();
            for chunk in chunks {
                if !current.is_empty() && current.chars().count()+chunk.chars().count()+1>width {
                    result.push(std::mem::take(&mut current));
                }
                if !current.is_empty() { current.push(' '); }
                current.push_str(&chunk);
            }
        }
        if !current.is_empty() {result.push(current);}
    }
    result
}
fn draw_lines(out:&mut String, lines:&[String], x:f32,mut y:f32,font:f32,leading:f32) {
    for line in lines {
        put_text(out,line,x,y,font,false);
        y-=leading;
    }
}
fn frame(out:&mut String,page:u32) {
    out.push_str("0.16 0.23 0.21 rg 50 771 495 1 re f\n");
    out.push_str("0.42 0.45 0.43 rg\n");
    put_text(out,"REPERTICA  /  STUDIO",52.0,788.0,9.0,true);
    put_text(out,&page.to_string(),522.0,40.0,9.0,false);
}
fn photo_ops(photo:&Photo,x:f32,y:f32,w:f32,h:f32,cover:bool)->String {
    let ratio=(photo.width as f32)/(photo.height as f32);
    let box_ratio=w/h;
    let (draw_w,draw_h)=if (ratio>box_ratio)==cover {(h*ratio,h)}else{(w,w/ratio)};
    let dx=x+(w-draw_w)/2.0;
    let dy=y+(h-draw_h)/2.0;
    format!("q {x:.2} {y:.2} {w:.2} {h:.2} re W n {draw_w:.2} 0 0 {draw_h:.2} {dx:.2} {dy:.2} cm /Im0 Do Q\n")
}
fn cover(b:&Brochure)->Page {
    let mut out=String::from("0.97 0.96 0.94 rg 0 0 595 842 re f\n0.18 0.27 0.23 rg\n");
    put_text(&mut out,match b.language.as_str() {"en"=>"SELECTED WORKS","es"=>"OBRAS SELECCIONADAS","fr"=>"ŒUVRES CHOISIES","de"=>"AUSGEWÄHLTE WERKE",_=>"OPERE SELEZIONATE"},66.0,652.0,12.0,true);
    out.push_str("0.29 0.44 0.38 rg 66 624 68 3 re f\n0.17 0.23 0.20 rg\n");
    draw_lines(&mut out,&lines(&b.artist,31),66.0,572.0,27.0,36.0);
    draw_lines(&mut out,&lines(&b.title,30),66.0,425.0,26.0,36.0);
    draw_lines(&mut out,&lines(&b.subtitle,54),66.0,309.0,15.0,22.0);
    put_text(&mut out,"REPERTICA  /  STUDIO",66.0,79.0,10.0,true);
    Page{drawing:out,photo:None}
}
fn intro(b:&Brochure)->Page {
    let mut out=String::new();
    frame(&mut out,0);
    put_text(&mut out,match b.language.as_str(){"en"=>"Introduction","es"=>"Introducción","fr"=>"Introduction","de"=>"Einleitung",_=>"Introduzione"},60.0,702.0,27.0,true);
    draw_lines(&mut out,&lines(&b.introduction,84).into_iter().take(36).collect::<Vec<_>>(),60.0,644.0,12.5,18.0);
    Page{drawing:out,photo:None}
}
fn contacts(b:&Brochure)->Page {
    let mut out=String::from("0.97 0.96 0.94 rg 0 0 595 842 re f\n0.17 0.23 0.20 rg\n");
    put_text(&mut out,match b.language.as_str(){"en"=>"CONTACTS","es"=>"CONTACTOS","fr"=>"CONTACTS","de"=>"KONTAKT",_=>"CONTATTI"},67.0,652.0,13.0,true);
    out.push_str("0.29 0.44 0.38 rg 67 626 68 2 re f\n0.17 0.23 0.20 rg\n");
    draw_lines(&mut out,&lines(&b.artist,30),67.0,564.0,27.0,34.0);
    for (i,line) in [b.contact_email.as_str(),b.contact_website.as_str(),b.contact_instagram.as_str()].iter().filter(|s|!s.trim().is_empty()).enumerate() {
        draw_lines(&mut out,&lines(line,65),67.0,418.0-(i as f32)*70.0,13.0,20.0);
    }
    put_text(&mut out,"REPERTICA  /  STUDIO",67.0,81.0,10.0,true);
    Page{drawing:out,photo:None}
}
fn artwork_pages(item:&BrochureItem, art:&ArtPage,photo:Photo,page_num:u32)->Vec<Page> {
    let title=if item.title_override.trim().is_empty(){&art.title}else{&item.title_override};
    let desc=if item.description_override.trim().is_empty(){art.description.as_deref().unwrap_or("")}else{&item.description_override};
    let technical=format!("{}  ·  {}  ·  {}  ·  {}",
        art.code,art.year.as_deref().unwrap_or(""),
        [art.technique.as_deref().unwrap_or(""),art.support.as_deref().unwrap_or("")].iter().filter(|x|!x.is_empty()).copied().collect::<Vec<_>>().join(" / "),
        match (art.height,art.width) {(Some(h),Some(w))=>format!("{h:g} × {w:g} cm"),_=>String::new()});
    let (allowed_chars,description_capacity) = match item.template.as_str() {
        "details"=>(32,28), "full_bleed"=>(85,4),_=>(91,7)
    };
    let description=if item.include_description {lines(desc,allowed_chars)}else{Vec::new()};
    let (shown,rest)=description.split_at(description.len().min(description_capacity));
    let mut out=String::new();
    match item.template.as_str() {
        "full_bleed" => {
            out.push_str(&photo_ops(&photo,0.0,0.0,595.0,842.0,true));
            out.push_str("0.97 0.96 0.94 rg 0 0 595 242 re f\n0.17 0.23 0.20 rg\n");
            draw_lines(&mut out,&lines(title,43).into_iter().take(2).collect::<Vec<_>>(),53.0,194.0,22.0,27.0);
            put_text(&mut out,&technical,53.0,125.0,10.0,false);
            draw_lines(&mut out,shown,53.0,100.0,10.5,14.0);
        }
        "details" => {
            frame(&mut out,page_num);
            out.push_str(&photo_ops(&photo,52.0,283.0,289.0,455.0,false));
            draw_lines(&mut out,&lines(title,29).into_iter().take(4).collect::<Vec<_>>(),363.0,715.0,17.0,25.0);
            draw_lines(&mut out,&lines(&technical,38).into_iter().take(5).collect::<Vec<_>>(),363.0,585.0,9.0,16.0);
            draw_lines(&mut out,shown,363.0,483.0,10.5,15.0);
        }
        _ => {
            frame(&mut out,page_num);
            out.push_str(&photo_ops(&photo,56.0,315.0,483.0,438.0,false));
            draw_lines(&mut out,&lines(title,47).into_iter().take(2).collect::<Vec<_>>(),56.0,283.0,24.0,29.0);
            put_text(&mut out,&technical,56.0,235.0,10.0,false);
            draw_lines(&mut out,shown,56.0,211.0,11.0,18.0);
        }
    }
    let mut pages=vec![Page{drawing:out,photo:Some(photo)}];
    for (index,chunk) in rest.chunks(34).enumerate() {
        let mut continuation=String::new();
        frame(&mut continuation,page_num+index as u32+1);
        draw_lines(&mut continuation,&lines(title,47).into_iter().take(2).collect::<Vec<_>>(),60.0,706.0,20.0,27.0);
        draw_lines(&mut continuation,chunk,60.0,622.0,12.0,16.0);
        pages.push(Page{drawing:continuation,photo:None});
    }
    pages
}
struct Pdf { objects:Vec<Vec<u8>> }
impl Pdf {
    fn new()->Self {Self{objects:Vec::new()}}
    fn reserve(&mut self)->usize {self.objects.push(Vec::new());self.objects.len()}
    fn put(&mut self,id:usize,bytes:Vec<u8>){self.objects[id-1]=bytes;}
    fn object(&mut self,bytes:Vec<u8>)->usize{let id=self.reserve();self.put(id,bytes);id}
    fn stream(dict:&str,bytes:&[u8])->Vec<u8>{
        let mut value=format!("<< {dict} /Length {} >>\nstream\n",bytes.len()).into_bytes();
        value.extend_from_slice(bytes); value.extend_from_slice(b"\nendstream");value
    }
    fn finish(self)->Vec<u8>{
        let mut out=b"%PDF-1.4\n%\xE2\xE3\xCF\xD3\n".to_vec();
        let mut offsets=vec![0usize];
        for (index,object) in self.objects.iter().enumerate(){
            offsets.push(out.len());
            out.extend_from_slice(format!("{} 0 obj\n",index+1).as_bytes());
            out.extend_from_slice(object);
            out.extend_from_slice(b"\nendobj\n");
        }
        let start=out.len();
        out.extend_from_slice(format!("xref\n0 {}\n0000000000 65535 f \n",offsets.len()).as_bytes());
        for offset in offsets.iter().skip(1) {out.extend_from_slice(format!("{offset:010} 00000 n \n").as_bytes());}
        out.extend_from_slice(format!("trailer\n<< /Size {} /Root 1 0 R >>\nstartxref\n{start}\n%%EOF\n",offsets.len()).as_bytes());
        out
    }
}
fn render_pdf(pages:Vec<Page>)->Vec<u8>{
    let mut pdf=Pdf::new();
    let catalog=pdf.reserve();
    let tree=pdf.reserve();
    let font=pdf.object(b"<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica /Encoding /WinAnsiEncoding >>".to_vec());
    let bold=pdf.object(b"<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica-Bold /Encoding /WinAnsiEncoding >>".to_vec());
    let mut ids=Vec::new();
    for page in pages {
        let image_ref=page.photo.map(|image|{
            let dict=format!("/Type /XObject /Subtype /Image /Width {} /Height {} /ColorSpace /DeviceRGB /BitsPerComponent 8 /Filter /DCTDecode",image.width,image.height);
            pdf.object(Pdf::stream(&dict,&image.jpg))
        });
        let content=pdf.object(Pdf::stream("",page.drawing.as_bytes()));
        let id=pdf.reserve();
        let xobject=image_ref.map_or(String::new(),|r|format!("/XObject << /Im0 {r} 0 R >>"));
        pdf.put(id,format!("<< /Type /Page /Parent {tree} 0 R /MediaBox [0 0 595 842] /Resources << /Font << /F1 {font} 0 R /F2 {bold} 0 R >> {xobject} >> /Contents {content} 0 R >>").into_bytes());
        ids.push(id);
    }
    let kids=ids.iter().map(|id|format!("{id} 0 R")).collect::<Vec<_>>().join(" ");
    pdf.put(tree,format!("<< /Type /Pages /Kids [{kids}] /Count {} >>",ids.len()).into_bytes());
    pdf.put(catalog,format!("<< /Type /Catalog /Pages {tree} 0 R >>").into_bytes());
    pdf.finish()
}
fn validate_output(db:&Database,destination:&str)->Result<(),String>{
    let file=Path::new(destination);
    if !file.extension().and_then(|s|s.to_str()).is_some_and(|s|s.eq_ignore_ascii_case("pdf")){
        return Err("La destinazione deve essere un file .pdf".into());
    }
    let parent=fs::canonicalize(file.parent().ok_or("Cartella non valida")?).map_err(error)?;
    if !parent.is_dir() || parent.starts_with(fs::canonicalize(&db.directory).map_err(error)?) {
        return Err("Seleziona una cartella esterna all'archivio".into());
    }
    if file.exists() && !fs::symlink_metadata(file).map_err(error)?.file_type().is_file() {
        return Err("La destinazione non è un file ordinario".into());
    }
    if file.exists() && fs::symlink_metadata(file).map_err(error)?.file_type().is_symlink() {
        return Err("Non è consentito esportare su collegamenti simbolici".into());
    }
    Ok(())
}
#[tauri::command]
pub fn export_brochure_pdf(db: State<'_,Database>, brochure_id:String, destination:String)->Result<String,String>{
    validate_output(&db,&destination)?;
    let document={
        let conn=connection(&db)?;
        let json=conn.query_row("SELECT payload_json FROM brochures WHERE id=?1",[brochure_id],
            |row|row.get::<_,String>(0)).optional().map_err(error)?
            .ok_or_else(||"Brochure non trovata".to_string())?;
        serde_json::from_str::<Brochure>(&json).map_err(error)?
    };
    validate(&document)?;
    if document.items.is_empty(){return Err("Aggiungi almeno un'opera prima dell'esportazione".into());}
    let mut pages=vec![cover(&document)];
    if !document.introduction.trim().is_empty(){pages.push(intro(&document));}
    for item in &document.items{
        let artwork=selected_artwork(&db,&item.artwork_id,item.gallery_image_id.as_deref())?;
        let path=artwork.image_path.as_deref()
            .ok_or_else(||format!("Manca la fotografia per l'opera {}",artwork.title))?;
        let photo=load_photo(&db,path)?;
        pages.extend(artwork_pages(item,&artwork,photo,pages.len() as u32+1));
    }
    pages.push(contacts(&document));
    let pdf=render_pdf(pages);
    fs::write(&destination,&pdf).map_err(error)?;
    Ok(destination)
}
#[cfg(test)]
mod tests{
    use super::*;
    #[test]
    fn validates_drafts_and_template_ids(){
        let mut b=Brochure{title:"Selezione".into(),language:"it".into(),..Default::default()};
        assert!(validate(&b).is_ok());
        b.items.push(BrochureItem{artwork_id:"abc".into(),..Default::default()});
        b.items.push(BrochureItem{artwork_id:"abc".into(),..Default::default()});
        assert!(validate(&b).is_err());
        b.items.pop();
        b.items[0].template="javascript:".into();
        assert!(validate(&b).is_err());
    }
    #[test]
    fn generated_pdf_has_page_tree_and_xref(){
        let b=Brochure{title:"Ópera".into(),artist:"Artista".into(),language:"es".into(),..Default::default()};
        let pdf=render_pdf(vec![cover(&b),contacts(&b)]);
        assert!(pdf.starts_with(b"%PDF-1.4"));
        let text=String::from_utf8_lossy(&pdf);
        assert!(text.contains("/Count 2"));
        assert!(text.contains("xref\n"));
        assert!(text.contains("%%EOF"));
        assert!(text.contains(&hex_text("Ópera")));
    }
    #[test]
    fn line_breaks_do_not_truncate_extremely_long_words(){
        let s="x".repeat(200);
        assert_eq!(lines(&s,40).iter().map(|x|x.len()).sum::<usize>(),200);
    }
}
