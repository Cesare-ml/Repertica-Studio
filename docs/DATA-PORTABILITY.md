# Esportazioni CSV locali — registri dell'archivio

L'esportatore Rust `export_ledger_csv` supporta sette valori di `kind`:

| Tipo | Righe esportate | Contenuti |
| --- | --- | --- |
| `artworks` | Tutte le opere | UUID, codice, titolo, anno, tecnica, supporto, dimensioni in cm, descrizione, ubicazione, stato, percorso dell'immagine principale e date |
| `collectors` | Tutti i collezionisti | UUID, contatti e note private, date |
| `certificates` | Tutti i certificati | UUID del certificato e opera, seriale, date, emittente, destinatario, note e **snapshot_json originale immutabile** |
| `sales` | Tutte le vendite | UUID opera/collezionista, importi in centesimi e quote storiche |
| `balances` | Tutti i saldi delle vendite | UUID vendita/opera/collezionista, data, valuta, lordo, incassato dichiarato e residuo in centesimi (anche vendite senza incassi) |
| `expenses` | Tutte le spese | UUID, date, causali, fornitori e importi in centesimi |
| `receipts` | Tutti gli incassi registrati manualmente | UUID incasso/vendita/opera/collezionista, data, importo in centesimi, valuta, metodo, riferimento, note e data di registrazione |

Tutti i registri sono esportati integralmente, senza dipendere dai filtri dell'interfaccia. L'output è UTF-8 con BOM, separatore `;`, righe CRLF, campi testuali tra virgolette con escape delle virgolette, e difesa contro formule di fogli elettronici. Le dimensioni sono numeri decimali con punto (non valori monetari). I campi vuoti non conservano la distinzione fra stringa vuota e `NULL`.

Il campo `current_artwork_title` nei certificati mostra il **titolo attuale** collegato all'opera; il contenuto storico autentico dell'emissione rimane in `snapshot_json`. Gli UUID permettono di riconciliare i registri tra loro. Nessuna esportazione altera opere, certificati o vendite.

## Sicurezza e ambito

- Il dialogo nativo di salvataggio accetta una cartella scelta dall'utente, inclusa un'unità esterna o NAS montato. Non viene sovrascritto alcun file esistente e non si può esportare nella cartella privata dell'app.
- L'esportatore lavora offline e non crea account o servizi remoti.
- I CSV contengono **dati personali in chiaro**. Conservare e condividere responsabilmente.
- Il formato CSV è destinato a analisi e interoperabilità **in sola lettura**, non è un formato di reimportazione transazionale. Le immagini, i file associati e l'intero schema SQLite non sono contenuti nei CSV: per una copia completa portabile usare sempre **Impostazioni → Backup** e la sua verifica SHA-256.
- L'esportazione `sales` registra vendite **non necessariamente incassate**: non deve essere descritta come saldo bancario. L'esportazione distinta `receipts` elenca **gli incassi dichiarati dall'utente**, non movimenti bancari verificati: un incasso può essere parziale; lo stesso `sale_id` può comparire più volte. Il registro `balances` riporta una riga per vendita con importo lordo, incassato dichiarato e residuo: non considera automaticamente pagate le vendite prive di incassi.
- **Interfaccia**: Opere, Collezionisti, Certificati, Vendite (tre pulsanti separati per vendite, incassi e saldi) e Acquisti e spese dispongono del pulsante CSV. Il dialogo nativo propone un nome `ReperticaStudio-<registro>-AAAA-MM-GG.csv` e lascia scegliere qualsiasi cartella accessibile. Se il percorso è già occupato, il backend rifiuta la sovrascrittura e mostra un errore; scegliere un nuovo nome. Le esportazioni includono **tutti** i record, anche se nell'interfaccia sono attivi filtri o un anno specifico.

## Collaudo

`cargo test --manifest-path src-tauri/Cargo.toml` include test per UUID, campi storici dei certificati, quote di escaping, importi e dimensioni, creazione senza sovrascrittura e divieto di scrivere nell'archivio interno. Eseguire una prova desktop reale su Windows/macOS/Linux con file grandi e cartelle rimovibili prima di una release stabile.

## Fotografie storiche dei certificati nei backup

Il certificato conserva uno snapshot immutabile della scheda al momento dell'emissione. Quando l'immagine principale viene sostituita, il file originale può non comparire più né nella scheda attuale né nella galleria: rimane però necessario per il certificato storico. La verifica del backup controlla ora anche i percorsi `image_path` presenti negli snapshot JSON **validi** dei certificati. Un backup che ometta una di queste immagini viene rifiutato **prima** di sostituire l'archivio attivo, anche se SQLite e tutti gli hash dei file ancora elencati nel manifest risultano corretti.

I file storici sono copiati nel backup insieme agli altri media. Gli snapshot storici malformati, che possono esistere in vecchi archivi, vengono conservati senza tentare di dedurne percorsi: non è possibile certificare automaticamente le loro immagini. Il controllo non recupera fotografie già eliminate; per quelle occorre una copia precedente. Il ripristino richiede sempre collaudo desktop reale da dischi esterni e NAS.

## Verifica completa della directory media

La verifica del backup controlla anche che la cartella `media/` sia una directory
reale, non un collegamento simbolico, e che **ogni file fisicamente presente**
in `media/` sia elencato nel `manifest.json`. Il controllo riguarda anche i
file che non sono referenziati da opere o certificati: potrebbero essere
materiali storici ancora importanti per l'utente. In precedenza un file
aggiunto manualmente a una cartella di backup senza aggiornarne il manifest
poteva essere ignorato durante il ripristino e quindi andare perduto
silenziosamente. Adesso l'ispezione e il ripristino rifiutano quel backup
prima di sostituire l'archivio attivo.

I file e i nomi non regolari, le sottocartelle e i link simbolici dentro
`media/` sono respinti. Non modificare direttamente un backup verificato:
se occorre integrare file, importarli prima nell'archivio attivo e creare un
nuovo backup da Impostazioni. L'archivio precedente resta preservato in
caso di errore. Sono disponibili test automatici per file aggiuntivi non
registrati e, su Unix, per una cartella media collegata all'esterno.
