# Esportazioni CSV locali — registri dell'archivio

L'esportatore Rust `export_ledger_csv` supporta dodici valori di `kind`:

| Tipo | Righe esportate | Contenuti |
| --- | --- | --- |
| `artworks` | Tutte le opere | UUID, codice, titolo, anno, tecnica, supporto, dimensioni in cm, descrizione, ubicazione, stato, percorso dell'immagine principale e date |
| `gallery` | Tutte le fotografie aggiuntive della galleria | UUID foto e opera, codice e titolo attuali, percorso relativo del file, didascalia, ordine di visualizzazione, data e tipo (`kind`: `DETAIL`, `SIGNATURE`, `DIGITAL_MOCKUP`, `IN_SITU`, `OTHER`, `FRAMED`, `WORK_IN_PROGRESS`); non contiene i byte delle immagini |
| `collectors` | Tutti i collezionisti | UUID, nome completo, contatti e note private, date; in coda nome, cognome, dati di nascita, profili social, percorso della foto profilo e numero di fotografie |
| `collector_photos` | Tutte le fotografie dei collezionisti, profilo e aggiuntive | UUID foto/collezionista, nome attuale, tipo (`PROFILE` o `PHOTO`), percorso relativo, UUID Media Center facoltativo e data; nessun file binario |
| `certificates` | Tutti i certificati | UUID del certificato e opera, seriale, date, emittente, destinatario, note e **snapshot_json originale immutabile** |
| `raw_files` | Tutti i file RAW associati alle opere | UUID opera, codice/titolo attuali, percorso relativo, nome originario, dimensione, SHA-256 registrato e data; nessun file binario |
| `sales` | Tutte le vendite | UUID opera/collezionista, importi in centesimi e le tre quote storiche predefinite (personale, attività, risparmio; 0 per le vendite che non le usano: le ripartizioni personalizzate sono nel registro `allocations`) |
| `installments` | Tutte le rate dei piani rateali | UUID rata/vendita, posizione, scadenza, importo in centesimi, note, codice e titolo dell'opera, nome del collezionista |
| `allocations` | Una riga per vendita e per parte di ripartizione | UUID vendita/opera, netto in centesimi, UUID e nome della parte, quota in punti base congelata sulla vendita |
| `balances` | Tutti i saldi delle vendite | UUID vendita/opera/collezionista, data, valuta, lordo, incassato dichiarato e residuo in centesimi (anche vendite senza incassi) |
| `expenses` | Tutte le spese | UUID, date, causali, fornitori, importi in centesimi, fondo (UUID e, in coda, nome della parte) |
| `receipts` | Tutti gli incassi registrati manualmente | UUID incasso/vendita/opera/collezionista, data, importo in centesimi, valuta, metodo, riferimento, note e data di registrazione; **se presente**, percorso relativo, nome originale, MIME, dimensione, SHA-256 e data dell'allegato locale dell'incasso |

Tutti i registri sono esportati integralmente, senza dipendere dai filtri dell'interfaccia. L'output è UTF-8 con BOM, separatore `;`, righe CRLF, campi testuali tra virgolette con escape delle virgolette, e difesa contro formule di fogli elettronici. Le dimensioni sono numeri decimali con punto (non valori monetari). I campi vuoti non conservano la distinzione fra stringa vuota e `NULL`.

**Registri aggiuntivi (`collector_photos`, `raw_files`):** espongono riferimenti stabili e metadati per ricostruire il legame con collezionisti, opere e Media Center. Una foto profilo precede le foto aggiuntive dello stesso collezionista; i file RAW sono ordinati per codice opera. I due `kind` sono disponibili anche dai pulsanti CSV in **Opere → Metadati RAW** e **Collezionisti → Fotografie**; il comando Rust `export_ledger_csv` salva i registri nella cartella scelta dall'utente. Non contengono byte di immagini/RAW, né verificano nuovamente gli SHA-256; per trasferire effettivamente i file resta necessario un backup completo SQLite + `media/`.

**Allegati degli incassi nel CSV:** le ultime sei colonne di `receipts` (`attachment_path`, `attachment_original_name`, `attachment_mime`, `attachment_size_bytes`, `attachment_sha256`, `attachment_created_at`) sono vuote per gli incassi senza documento. Il collegamento è tramite `receipt_id` della stessa riga, senza duplicare né omettere incassi. Il percorso è **relativo alla cartella dell'archivio**, non un URL né un collegamento al disco da cui il documento fu acquisito. Il nome originale viene protetto contro formule CSV come gli altri testi. **Non vengono esportati i byte degli allegati né verificati di nuovo i file durante l'esportazione del registro**: SHA-256 è il valore registrato in SQLite. Per trasferire o verificare documenti effettivi usa il backup completo SQLite + `media/`, non il CSV. L'estensione delle colonne è introdotta nel formato CSV successivo allo schema locale v6; eventuali elaborazioni esterne che si aspettano un numero fisso di colonne vanno aggiornate.

Il campo `current_artwork_title` nei certificati mostra il **titolo attuale** collegato all'opera; il contenuto storico autentico dell'emissione rimane in `snapshot_json`. Gli UUID permettono di riconciliare i registri tra loro. Il registro `gallery` conserva l'ordine (`sort_order`) e le didascalie delle immagini aggiuntive, ma non include le immagini binarie; la copertina è nel registro `artworks`, le immagini storiche negli snapshot dei `certificates`. Per trasferire realmente le fotografie serve un backup completo. Nessuna esportazione altera opere, certificati o vendite.

## Sicurezza e ambito

- Il dialogo nativo di salvataggio accetta una cartella scelta dall'utente, inclusa un'unità esterna o NAS montato. Non viene sovrascritto alcun file esistente e non si può esportare nella cartella privata dell'app.
- L'esportatore lavora offline e non crea account o servizi remoti.
- I CSV contengono **dati personali in chiaro**. Conservare e condividere responsabilmente.
- Il formato CSV è destinato a analisi e interoperabilità **in sola lettura**, non è un formato di reimportazione transazionale. Le immagini, i file associati e l'intero schema SQLite non sono contenuti nei CSV: per una copia completa portabile usare sempre **Impostazioni → Backup** e la sua verifica SHA-256.
- L'esportazione `sales` registra vendite **non necessariamente incassate**: non deve essere descritta come saldo bancario. L'esportazione distinta `receipts` elenca **gli incassi dichiarati dall'utente**, non movimenti bancari verificati: un incasso può essere parziale; lo stesso `sale_id` può comparire più volte. Il registro `balances` riporta una riga per vendita con importo lordo, incassato dichiarato e residuo: non considera automaticamente pagate le vendite prive di incassi.
- **Interfaccia**: Opere (registri separati per schede e metadati galleria), Collezionisti, Certificati, Vendite (tre pulsanti separati per vendite, incassi e saldi) e Acquisti e spese dispongono del pulsante CSV. Il dialogo nativo propone un nome `ReperticaStudio-<registro>-AAAA-MM-GG.csv` e lascia scegliere qualsiasi cartella accessibile. Se il percorso è già occupato, il backend rifiuta la sovrascrittura e mostra un errore; scegliere un nuovo nome. Le esportazioni includono **tutti** i record, anche se nell'interfaccia sono attivi filtri o un anno specifico.

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

Dalla versione con schema v17 il registro `certificates` termina con le colonne `language` (`it`, `en` o `bilingual`) e `issued_place`. Tutte le esportazioni CSV si trovano in **Impostazioni → Esportazioni**.
