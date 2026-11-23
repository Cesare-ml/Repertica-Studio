//! Small helpers shared by every command module.
use std::path::Path;

/// Turn low-level SQLite / operating-system failures into messages a person can act on.
/// Unknown errors are returned unchanged so no diagnostic detail is lost.
pub fn friendly(error: impl std::fmt::Display) -> String {
    let text = error.to_string();
    let has = |needle: &str| text.contains(needle);
    let message = if has("UNIQUE constraint failed: artworks.code") {
        "Esiste già un'opera con questo codice"
    } else if has("UNIQUE constraint failed: certificates.serial") {
        "Esiste già un certificato con questo numero"
    } else if has("UNIQUE constraint failed: sales.artwork_id") {
        "Questa opera ha già una vendita registrata"
    } else if has("UNIQUE constraint failed: artwork_images.image_path") || has("UNIQUE constraint failed: collector_images.image_path") {
        "Questa immagine è già usata in una galleria"
    } else if has("certificate_immutable") {
        "I certificati emessi non possono essere modificati o eliminati"
    } else if has("receipt_exceeds_sale") {
        "L'incasso supera l'importo ancora da incassare per questa vendita"
    } else if has("sale_below_receipts") {
        "L'importo della vendita non può scendere sotto gli incassi già registrati"
    } else if has("FOREIGN KEY constraint failed") {
        "Operazione non consentita: il record è collegato ad altri dati oppure non esiste più"
    } else if has("CHECK constraint failed") {
        "Uno dei valori non è consentito dai controlli dell'archivio"
    } else if is_disk_full(&text) {
        "Spazio su disco insufficiente: libera spazio sull'unità indicata e riprova"
    } else if has("database is locked") || has("database table is locked") {
        "L'archivio è occupato da un altro programma: chiudilo e riprova"
    } else if has("readonly database") {
        "L'archivio è in sola lettura: controlla i permessi della cartella dati"
    } else if has("disk I/O error") {
        "Errore di lettura o scrittura sul disco: verifica l'unità e riprova"
    } else if has("(os error 5)") {
        "Accesso negato dal sistema: controlla i permessi della cartella o chiudi i programmi che usano questi file"
    } else {
        return text;
    };
    message.to_string()
}

/// Lower-case and fold common Latin accents, including decomposed Unicode.
/// Artists and collectors may use names such as "Straße", "Œuvre" or "Łódź":
/// searching for "strasse", "oeuvre" or "lodz" should find the local media.
/// This is intentionally a small Latin-oriented search fold, not a complete
/// transliterator. Characters outside the mappings are preserved.
pub fn fold(text: &str) -> String {
    let mut folded = String::with_capacity(text.len());
    for c in text.chars().flat_map(char::to_lowercase) {
        // macOS filenames and imported metadata can contain decomposed accents.
        if matches!(c,
            '\u{0300}'..='\u{036f}' | '\u{1ab0}'..='\u{1aff}' |
            '\u{1dc0}'..='\u{1dff}' | '\u{20d0}'..='\u{20ff}' |
            '\u{fe20}'..='\u{fe2f}') {
            continue;
        }
        match c {
            // Multi-letter equivalents must be expanded, not dropped.
            'æ' => folded.push_str("ae"),
            'œ' => folded.push_str("oe"),
            'ß' => folded.push_str("ss"),
            'þ' => folded.push_str("th"),
            'à' | 'á' | 'â' | 'ã' | 'ä' | 'å' | 'ā' | 'ă' | 'ą' | 'ǎ' => folded.push('a'),
            'ç' | 'ć' | 'ĉ' | 'ċ' | 'č' => folded.push('c'),
            'ď' | 'đ' | 'ð' => folded.push('d'),
            'è' | 'é' | 'ê' | 'ë' | 'ē' | 'ĕ' | 'ė' | 'ę' | 'ě' => folded.push('e'),
            'ĝ' | 'ğ' | 'ġ' | 'ģ' => folded.push('g'),
            'ĥ' | 'ħ' => folded.push('h'),
            'ì' | 'í' | 'î' | 'ï' | 'ĩ' | 'ī' | 'ĭ' | 'į' | 'ı' | 'ǐ' => folded.push('i'),
            'ĵ' => folded.push('j'),
            'ķ' => folded.push('k'),
            'ĺ' | 'ļ' | 'ľ' | 'ŀ' | 'ł' => folded.push('l'),
            'ñ' | 'ń' | 'ņ' | 'ň' => folded.push('n'),
            'ò' | 'ó' | 'ô' | 'õ' | 'ö' | 'ō' | 'ŏ' | 'ő' | 'ø' | 'ǒ' => folded.push('o'),
            'ŕ' | 'ŗ' | 'ř' => folded.push('r'),
            'ś' | 'ŝ' | 'ş' | 'š' => folded.push('s'),
            'ţ' | 'ť' | 'ŧ' => folded.push('t'),
            'ù' | 'ú' | 'û' | 'ü' | 'ũ' | 'ū' | 'ŭ' | 'ů' | 'ű' | 'ų' | 'ǔ' => folded.push('u'),
            'ŵ' => folded.push('w'),
            'ý' | 'ÿ' | 'ŷ' => folded.push('y'),
            'ź' | 'ż' | 'ž' => folded.push('z'),
            other => folded.push(other),
        }
    }
    folded
}

/// ENOSPC (28), ERROR_DISK_FULL (112), ERROR_HANDLE_DISK_FULL (39) and SQLite's "database or disk is full".
fn is_disk_full(text: &str) -> bool {
    text.contains("database or disk is full")
        || text.contains("No space left")
        || text.contains("(os error 112)")
        || text.contains("(os error 39)")
        || text.contains("(os error 28)")
}

/// `fs::canonicalize` returns `\\?\C:\...` on Windows; show people the path they know.
pub fn display_path(path: &Path) -> String {
    let text = path.to_string_lossy();
    if let Some(rest) = text.strip_prefix(r"\\?\UNC\") {
        format!(r"\\{rest}")
    } else if let Some(rest) = text.strip_prefix(r"\\?\") {
        rest.to_string()
    } else {
        text.into_owned()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn constraint_and_trigger_errors_become_actionable_messages() {
        assert!(friendly("UNIQUE constraint failed: artworks.code").contains("codice"));
        assert!(friendly("certificate_immutable").contains("certificati"));
        assert!(friendly("receipt_exceeds_sale").contains("incasso"));
        assert!(friendly("FOREIGN KEY constraint failed").contains("collegato"));
        assert!(friendly("database is locked").contains("occupato"));
        assert_eq!(friendly("something unexpected"), "something unexpected");
    }

    #[test]
    fn full_disk_is_recognised_for_sqlite_windows_and_unix() {
        for raw in [
            "database or disk is full",
            "Spazio su disco insufficiente. (os error 112)",
            "There is not enough space on the disk. (os error 112)",
            "No space left on device (os error 28)",
            "Il disco è pieno (os error 39)",
        ] {
            assert!(friendly(raw).contains("Spazio su disco"), "{raw}");
        }
        let io = std::io::Error::from_raw_os_error(if cfg!(windows) { 112 } else { 28 });
        assert!(friendly(io).contains("Spazio su disco"));
        assert!(friendly("Accesso negato. (os error 5)").contains("Accesso negato"));
    }

    #[test]
    fn local_media_search_folds_decomposed_unicode_accents() {
        // Media filenames and tags may originate on macOS, where combining
        // characters are common. Normalise both query and stored metadata.
        assert_eq!(fold("Café"), "cafe");
        assert_eq!(fold("Cafe\u{301}"), "cafe");
        assert_eq!(fold("CAFÉ"), fold("cafe\u{301}"));
        assert_eq!(fold("Piñata"), fold("Pin\u{303}ata"));
        assert_eq!(fold("İSTANBUL"), "istanbul");

        let title = fold("Antichita\u{300} · SÖLE");
        assert!(title.contains(&fold("antichità")));
        assert!(title.contains(&fold("sole")));
        assert_eq!(fold("Studio #42 & 📷"), "studio #42 & 📷",
            "unrelated characters and punctuation are preserved");
    }

    #[test]
    fn local_media_search_matches_ligatures_and_extended_latin_names() {
        // File names, artwork titles and photo credits can come from other
        // European languages; searches should not require the same keyboard.
        for (stored, typed) in [
            ("Straße", "STRASSE"),
            ("Œuvre", "oeuvre"),
            ("Æther", "aether"),
            ("Sølv", "SOLV"),
            ("Łódź", "lodz"),
            ("Dvořák", "dvorak"),
            ("Český", "cesky"),
            ("İstanbul", "istanbul"),
            ("Þing", "thing"),
        ] {
            assert_eq!(fold(stored), fold(typed), "{stored} versus {typed}");
        }
        // Non-Latin text and emoji must not be transliterated or stripped.
        assert_eq!(fold("日本語 · 📷"), "日本語 · 📷");
    }

    #[test]
    fn verbatim_windows_prefixes_are_hidden_from_users() {
        assert_eq!(display_path(Path::new(r"\\?\C:\Users\Anna\backup")), r"C:\Users\Anna\backup");
        assert_eq!(display_path(Path::new(r"\\?\UNC\nas\share\backup")), r"\\nas\share\backup");
        assert_eq!(display_path(Path::new("/home/anna/backup")), "/home/anna/backup");
    }
}
