# Allegati locali degli incassi — prima fase backend (SQLite v6)

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
Se la copia del file riesce ma l'inserimento SQLite fallisce, il nuovo file
viene rimosso. Nessun servizio remoto, account o libreria di upload cloud.

**Stato:** è pronta l'API Rust con test; **la selezione file e la visualizzazione
degli allegati nel registro vendite Angular sono ancora da integrare**.
Non considerare la funzionalità completata finché il flusso UI non è collaudato.

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
Un backup con file manipolati e manifest ricalcolato viene quindi rifiutato.
Il formato resta una directory
portabile e leggibile offline; i PDF e le immagini non vengono cifrati.
Conservare i backup in luoghi sicuri perché possono contenere documenti
personali o bancari.

## Controlli

I test Rust coprono la migrazione di un archivio v5 con incassi storici,
relazioni, tipo file, file vuoti e link simbolici Unix, l'inserimento,
la lettura e la rimozione, oltre al rifiuto di un backup cui è stato tolto
un allegato sia dal disco sia dal manifest. Test aggiuntivi verificano backup
con manifest ricalcolato dopo alterazioni del file, dell'hash SQLite o del MIME,
nonché il rifiuto della pubblicazione di backup da una sorgente già corrotta. Restano necessari i test reali su
Windows/macOS/Linux, USB e NAS, compreso il flusso interattivo Angular.
