# Esportazioni CSV locali — registri dell'archivio

L'esportatore Rust `export_ledger_csv` supporta cinque valori di `kind`:

| Tipo | Righe esportate | Contenuti |
| --- | --- | --- |
| `artworks` | Tutte le opere | UUID, codice, titolo, anno, tecnica, supporto, dimensioni in cm, descrizione, ubicazione, stato, percorso dell'immagine principale e date |
| `collectors` | Tutti i collezionisti | UUID, contatti e note private, date |
| `certificates` | Tutti i certificati | UUID del certificato e opera, seriale, date, emittente, destinatario, note e **snapshot_json originale immutabile** |
| `sales` | Tutte le vendite | UUID opera/collezionista, importi in centesimi e quote storiche |
| `expenses` | Tutte le spese | UUID, date, causali, fornitori e importi in centesimi |

Tutti i registri sono esportati integralmente, senza dipendere dai filtri dell'interfaccia. L'output è UTF-8 con BOM, separatore `;`, righe CRLF, campi testuali tra virgolette con escape delle virgolette, e difesa contro formule di fogli elettronici. Le dimensioni sono numeri decimali con punto (non valori monetari). I campi vuoti non conservano la distinzione fra stringa vuota e `NULL`.

Il campo `current_artwork_title` nei certificati mostra il **titolo attuale** collegato all'opera; il contenuto storico autentico dell'emissione rimane in `snapshot_json`. Gli UUID permettono di riconciliare i registri tra loro. Nessuna esportazione altera opere, certificati o vendite.

## Sicurezza e ambito

- Il dialogo nativo di salvataggio accetta una cartella scelta dall'utente, inclusa un'unità esterna o NAS montato. Non viene sovrascritto alcun file esistente e non si può esportare nella cartella privata dell'app.
- L'esportatore lavora offline e non crea account o servizi remoti.
- I CSV contengono **dati personali in chiaro**. Conservare e condividere responsabilmente.
- Il formato CSV è destinato a analisi e interoperabilità **in sola lettura**, non è un formato di reimportazione transazionale. Le immagini, i file associati e l'intero schema SQLite non sono contenuti nei CSV: per una copia completa portabile usare sempre **Impostazioni → Backup** e la sua verifica SHA-256.
- L'esportazione `sales` registra vendite **non necessariamente incassate**: non deve essere descritta come saldo bancario.
- **Stato interfaccia**: i pulsanti di esportazione per vendite e spese esistono già. I tre registri aggiuntivi sono disponibili nel comando Tauri, ma richiedono ancora collegamento ai pulsanti dell'interfaccia Angular; questo intervento evita deliberatamente i file UI e di migrazione modificati dalla PR parallela della galleria (#17).

## Collaudo

`cargo test --manifest-path src-tauri/Cargo.toml` include test per UUID, campi storici dei certificati, quote di escaping, importi e dimensioni, creazione senza sovrascrittura e divieto di scrivere nell'archivio interno. Eseguire una prova desktop reale su Windows/macOS/Linux con file grandi e cartelle rimovibili prima di una release stabile.
