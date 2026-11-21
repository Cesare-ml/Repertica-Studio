# Allegati locali degli incassi — interfaccia vendite + SQLite v6

Gli incassi sono registrazioni **manuali**, non pagamenti verificati dalla banca.
La migrazione transazionale SQLite **v5 → v6** aggiunge la tabella
`sale_receipt_attachments`, con **al massimo un allegato per incasso**.
Gli incassi preesistenti restano invariati e non vengono dotati di documenti
fittizi. La relazione con `sale_receipts` ha una foreign key; il record di
incasso non può essere eliminato finché esiste un allegato collegato.

## API native già implementate

- `attach_sale_receipt_file(receipt_id, source_path)`: copia un PDF/JPEG/PNG/WebP
  scelto dall'utente da un percorso **assoluto** in `media/receipt-<UUID>.<ext>`;
  verifica file regolare, firma del formato, limite **12 MiB**, unicità
  dell'allegato, collegamento all'incasso e persistenza. Non mantiene riferimenti
  al percorso originale: scollegare il disco esterno non interrompe l'archivio.
- `get_sale_receipt_attachment(receipt_id)`: restituisce i metadati, tra cui
  il nome originale, il MIME, la dimensione, la data e lo SHA-256.
- `sale_receipt_attachment_data(receipt_id)`: legge il file solo da `media/`
  con controllo del percorso, del tipo di file e dello SHA-256 originale e
  restituisce un data URL per una successiva anteprima/stampa frontend.
- `remove_sale_receipt_attachment(receipt_id)`: elimina il collegamento e
  prova a cancellare la copia locale. Un errore dopo il commit viene esposto;
  il file residuo rimane incluso nei backup completi per evitare perdite silenziose.

Le operazioni sono serializzate dal mutex del database, lo stesso del backup.
La rimozione verifica **prima della modifica SQLite** che la directory `media/`
non sia stata sostituita da un collegamento simbolico. Se un archivio importato
o alterato riutilizza lo stesso file in un'opera, nella galleria, in un
collezionista, nel catalogo immagini, nei RAW o in un certificato storico,
scollega l'incasso ma **conserva il file condiviso**. I certificati non leggibili
sono trattati in modo conservativo: non autorizzano cancellazioni di file.
Restano possibili file orfani dopo un errore I/O di cancellazione, ma un errore
viene mostrato e il backup li conserva.

Se la copia del file riesce ma l'inserimento SQLite fallisce, il nuovo file
viene rimosso. Nessun servizio remoto, account o libreria di upload cloud.

**Stato:** l'interfaccia Angular del registro vendite ora consente di
scegliere tramite dialogo nativo un PDF, JPEG, PNG o WebP per ogni incasso,
mostrare il nome originale, consultare le immagini, esportare qualsiasi
documento su un percorso scelto dall'utente e rimuovere l'allegato dopo conferma.
La cancellazione dell'incasso viene disabilitata se ha un allegato collegato:
occorre prima rimuovere il documento. Il salvataggio rispetta il limite Rust
di 12 MiB e il file originale non deve restare accessibile dopo l'acquisizione.

**PDF:** per non allentare le restrizioni di sicurezza della WebView Tauri
(`object-src 'none'`, `frame-src 'none'`), non viene incorporato un visualizzatore
PDF. Usa **Esporta documento** per creare una copia verificata e aprirla
con il lettore PDF del sistema operativo. La scrittura richiede estensione
coerente col formato, rifiuta sovrascritture e impedisce di salvare all'interno
della cartella dell'archivio attivo. La lettura previa verifica
l'SHA-256 originale del file; gli errori vengono mostrati senza
sostituire file già esistenti.

**Da collaudare:** dialoghi nativi, anteprima delle immagini ed esportazione
PDF su Windows/macOS/Linux, inclusi dischi esterni e file protetti.

## Portabilità e ripristino

Tutti gli allegati si trovano dentro `media/` e sono compresi nel backup
SQLite + media con manifest SHA-256. Dallo schema v6 in poi la verifica dello
snapshot richiede che **ogni percorso allegato ancora collegato** esista tra i
file registrati nel manifest; un documento mancante causa il rifiuto del
ripristino **prima** di modificare l'archivio attivo.

La verifica dei checksum contenuti nel DB avviene alla **lettura** dell'allegato
e ora anche durante la **creazione, ispezione e il ripristino** di backup v6:
dimensione e SHA-256 nel record SQLite devono combaciare con il file del
manifest; inoltre firma del formato, MIME e estensione devono concordare.
Un backup con un file alterato e il solo manifest ricalcolato viene rifiutato.
**Limite:** il backup non è firmato digitalmente: modificando in modo coerente
file, database SQLite e manifest si possono produrre altri checksum validi.
Il formato resta una directory portabile e leggibile offline; i PDF e le immagini non vengono cifrati.
Conservare i backup in luoghi sicuri perché possono contenere documenti
personali o bancari.

## Controlli

I test Rust coprono la migrazione di un archivio v5 con incassi storici,
relazioni, tipo file, file vuoti e link simbolici Unix, l'inserimento,
la lettura e la rimozione, oltre al rifiuto di un backup cui è stato tolto
un allegato sia dal disco sia dal manifest. Test aggiuntivi verificano backup
con manifest ricalcolato dopo alterazioni del file, dell'hash SQLite o del MIME,
nonché il rifiuto della pubblicazione di backup da una sorgente già corrotta.
Restano necessari i test reali su Windows/macOS/Linux, USB e NAS, compreso il flusso interattivo Angular. Il test Node controlla i collegamenti IPC e la CSP, non simula una WebView o un dialogo nativo.
