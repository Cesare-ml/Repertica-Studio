# REPS-TECH-016 — Allegati documentali locali riutilizzabili, con riferimenti verificabili e nessun cloud

- **ID stabile:** REPS-TECH-016
- **Data:** 2026-11-21
- **Stato:** **Proposta — da valutare**
- **Filone:** Strumenti e architettura — **sola analisi e documentazione**
- **Priorità ipotetica:** **P2**; da elevare solo se prove d'uso mostrano un bisogno ricorrente di documenti collegati a più domini.
- **Decisione:** nessuna approvazione, incarico di sviluppo o modifica operativa.

## 1. Motivazione, destinatari e fatti osservati

Un archivio d'arte conserva non soltanto fotografie delle opere, ma anche documenti di contesto: perizie, schede di condizione, lettere di prestito, ricevute di trasporto, autorizzazioni fotografiche, estratti di catalogo ottenuti lecitamente e corrispondenza di provenienza. Lasciare questi file in cartelle esterne non collegate obbliga gli utenti a ricostruire manualmente il legame fra documento e scheda, soprattutto quando migrano computer o producono backup. Il problema è **conservare e recuperare il file indicato dall'utente**, non attribuirgli autenticità o valore giuridico.

**Riscontro sul branch `main`, 21/11/2026:** [README](../../../README.md), [MVP](../../MVP.md), [schema SQLite](../../../src-tauri/src/db.rs), [galleria](../../../src-tauri/src/gallery.rs), [allegati incassi](../../../src-tauri/src/receipt_attachments.rs), [specifica allegati](../../RECEIPT-ATTACHMENTS.md), [backup](../../../src-tauri/src/backup.rs) e [portabilità](../../DATA-PORTABILITY.md). È **già implementato** nello schema v6 un allegato locale **per singolo incasso**, PDF/JPEG/PNG/WebP fino a 12 MiB, copiato in `media/`, con tipo e SHA-256; i backup verificati includono gli allegati e la UI consente esportazione locale. Esistono anche foto principali e galleria delle opere, con finalità differente. **Non risulta invece un contenitore riutilizzabile per documenti associati ad altre entità**, né un registro di collegamenti a più schede con provenienza del file e permessi di condivisione. È un'opportunità progettuale, non un bug; nessun documento reale è stato esaminato.

**Destinatari:** archivisti e conservatori che devono ritrovare prove dichiarate; artisti che organizzano accordi e lettere; gallerie e professionisti che collegano documenti a più opere; piccoli studi che desiderano una copia ordinata, trasferibile e consultabile senza rete. Per archivi minimi potrebbe bastare la cartella gestita manualmente: verificare la domanda prima di sviluppare.

## 2. Confine rigoroso con funzioni e proposte già presenti

- **Allegati incasso esistenti:** non proporre di riscrivere la tabella `sale_receipt_attachments`, togliere la regola «uno per incasso», migrare documenti storici automaticamente o duplicare il percorso PDF/immagine già funzionante. Un eventuale livello comune deve **convivere** con l'API esistente e dimostrare retrocompatibilità, eventualmente evitando ogni migrazione nella prima fase.
- **[UX-001](https://github.com/Cesare-ml/Repertica-Studio/pull/14) e UX-004/010/016/019:** provenienza, rapporti di condizione, diritti, assicurazioni, esposizioni e bibliografie definiscono **significato, campi e processi professionali**. Questa scheda tratta soltanto un'infrastruttura trasversale per salvare file, verificarli e associarli esplicitamente a record identificati: non progetta la validità delle attestazioni.
- **[TECH-001](REPS-TECH-001-pacchetto-dati-aperto.md):** definisce l'interscambio semantico generale. Qui si dettaglia il requisito di **custodia degli allegati** e dei collegamenti; l'esportatore futuro deve soltanto poterli rappresentare senza perdere dati.
- **[TECH-003](REPS-TECH-003-diagnostica-integrita-media.md):** diagnosticare file e recuperare media mancanti è un'attività separata, che può in seguito includere anche documenti. **[TECH-005](REPS-TECH-005-anteprime-media-grandi-archivi.md)** riguarda cache di immagini, **[TECH-007](REPS-TECH-007-archivio-media-esterno-rilocalizzabile.md)** il volume degli originali, **[TECH-013](REPS-TECH-013-copie-fotografiche-condivisibili-privacy-metadati.md)** copie derivate per condivisione. Nessuna di queste schede è approvata.
- **Issue #13:** acquisti, incassi e documenti finanziari hanno già un perimetro; questa proposta **non** crea flussi contabili, fatture o nuovi tipi di incasso.
- **Esclusioni:** verifica di autenticità, firma digitale, conservazione legale a norma, OCR, estrazione automatica di testi, interpretazione IA, sincronizzazione, scansione remota di cartelle, apertura automatica di file non fidati e pubblicazione online.

## 3. Scenari d'uso realmente verificabili

1. **Scheda opera e documento:** l'archivista acquisisce volontariamente una scansione PDF di una dichiarazione relativa all'opera; la associa tramite UUID e annota *solo* un titolo e una data dichiarata. L'interfaccia non mostra «provenienza verificata» solo perché il PDF esiste.
2. **Un documento, più opere:** una lettera di prestito elenca cinque opere; un unico file conservato nel repository locale è collegato a cinque schede. Scollegarlo da una scheda non elimina il file usato dalle altre.
3. **Ritrovamento dopo trasferimento:** l'artista ripristina un backup su un altro PC; un documento acquisito anni prima è ancora apribile e verificabile senza montare la cartella esterna originaria.
4. **Condivisione mirata:** il gallerista vuole una copia di un documento selezionato, non tutto l'archivio. Visualizza l'avvertenza sui dati sensibili ed esporta manualmente in una cartella scelta, senza far partire email, upload, MCP o collegamenti esterni.
5. **Documento inaccessibile o alterato:** un controllo segnala file mancante o checksum difforme. Nessun riferimento viene riparato, rimosso o sostituito in automatico; un backup apparentemente valido non deve includere silenziosamente riferimenti non soddisfatti.

## 4. Specifica funzionale e flussi UX, soltanto candidati

**Acquisizione:** da una scheda che, *se e quando approvata*, supporta documenti → «Collega documento» → «Aggiungi file dal dispositivo». Dialogo **nativo** esplicito; anteprima di nome, tipo, dimensione e spazio richiesto; nota che sarà creata una **copia gestita** nel database/media store, e che l'originale resta dove si trova. Nessuna importazione ricorsiva o monitoraggio delle cartelle. Prima fase suggerita: soltanto PDF/JPEG/PNG/WebP nei limiti già collaudati per gli incassi; file maggiori e TIFF richiedono un'analisi distinta, non un'estensione tacita.

**Collegamento:** scegliere una o più opere esistenti; mostrare il riepilogo di tutti i legami prima di confermare. Possibile associazione ad altri domini (mostre, accordi, perizie, ricevute) **solo dopo** che i relativi modelli siano definiti e approvati nel filone gestione/UX. Selezionare esplicitamente titolo descrittivo, data *dichiarata* (facoltativa), fonte testuale e nota privata; eventuali etichette di tipologia restano descrittive, **non stati di verifica legale**.

**Consultazione:** elenco locale per scheda con icona tipo, nome, data di acquisizione, numero di collegamenti, stato «Disponibile / Non disponibile / Alterato / Non verificato». L'apertura delle immagini può riusare un'anteprima protetta; il PDF, come già avviene per gli incassi, viene **esportato su scelta** e aperto col software del sistema, senza incorporarlo nella WebView né allentare la CSP. Nessuna esecuzione di script contenuti in PDF.

**Rimozione:** distinguere «Rimuovi collegamento da questa scheda» da «Elimina documento dall'archivio». La seconda opzione richiede l'assenza di altri collegamenti, controllo delle eventuali dipendenze storiche e conferma esplicita. Prima fase prudente: **solo scollegamento**, con conservazione del file non referenziato fino a gestione revisionata; mai pulizia automatica di presunti orfani.

**Esportazione di singolo file:** comando con riepilogo dei dati che il file *potrebbe* contenere e avviso che la selezione non ne rimuove i metadati. Directory liberamente scelta, nome sicuro, nessuna sovrascrittura. La privacy di una copia fotografica depurata dai metadati è tema distinto di TECH-013.

**Accessibilità:** percorso completo da tastiera e screen reader, errori associati ai campi, stati anche testuali, avvisi non dipendenti dal colore, possibilità di ingrandimento UI e focus restituito all'elemento che ha avviato il dialogo.

## 5. Modello dati e ipotesi di API **puramente descrittive**

Una possibile architettura, da confrontare **senza introdurla ora**, distingue l'oggetto binario da uno o più collegamenti:
- `local_documents`: `document_id` UUID stabile, `relative_path` sicuro e generato, `original_name` non usato come percorso, `mime`, `size_bytes`, `sha256_acquisition`, `created_at_utc`, `description`/data dichiarata opzionali, stato di conservazione.
- `artwork_document_links`: `document_id`, `artwork_id`, `linked_at_utc`, nota di contesto facoltativa; **FK esplicite** e vincolo univoco sulla coppia. Per ogni nuovo dominio scegliere una tabella di relazione tipizzata con FK, non un campo `owner_type/owner_id` privo di integrità referenziale.
- **Nessuna deduplicazione automatica** sulla base di SHA-256: due file identici potrebbero avere contesti e identità documentali diversi; unione solo dopo decisione consapevole e revisione di referenze storiche.
- Hash acquisito in occasione di copia controllata; **non** è firma di un autore, marcatura temporale qualificata né prova di autenticità. Distinguere sempre file acquisito da file originario.

API concettuali, *non comandi Tauri da aggiungere ora*: `inspectCandidate(path) -> TypeSizeSafetyReport`; `importLocalDocument(path, userConsent) -> DocumentId`; `linkDocument(documentId, artworkId) -> Link`; `listDocuments(artworkId) -> Metadata[]`; `verifyDocument(documentId) -> IntegrityState`; `exportDocumentCopy(documentId, destination) -> ExportReport`; `unlinkDocument(linkId) -> Result`. Ogni operazione deve rispettare l'autorizzazione locale scelta dall'utente e fallire senza effetti parziali sul record corrente.

## 6. Offline, privacy, sicurezza e preferenze

- **Offline senza eccezioni:** copia e verifica su filesystem locale, SQLite e cartella media gestita. Nessun account, abbonamento, cloud, telemetria o connessione di rete richiesta. NAS solo come percorso già montato, senza credenziali archiviate nell'app.
- **Validazioni:** bloccare symlink, file speciali, path traversal, nomi pericolosi, file vuoti e tipi non supportati; controllare firma, estensione, MIME, dimensione, checksum e condizioni di scrittura. Una semplice intestazione `%PDF-` non rende innocuo un PDF. Valutare protezioni contro file che cambiano tra verifica e copia, carichi grandi e importazione malevola; nessun rendering HTML attivo.
- **Coerenza DB/filesystem:** scrittura su file temporaneo gestito, verifica prima del riferimento SQLite, commit ordinato con crash recovery e chiara gestione dei file orfani; serializzazione con backup e rimozioni. Non usare l'hash come prova che l'intero archivio non sia stato alterato intenzionalmente.
- **Conservazione storica:** file già citati da un certificato immutabile o da una registrazione non cancellabile non vanno rimossi senza valutare le dipendenze. Non tradurre né modificare automaticamente i PDF originali.
- **Privacy:** default «privato». Niente condivisione pubblica o esposizione MCP predefinita; un futuro server MCP read-only [TECH-014](REPS-TECH-014-bridge-mcp-locale-lettura-permessi.md) non riceve accesso automatico ai byte degli allegati. Avvisi su documenti con recapiti, firme, indirizzi, informazioni finanziarie; *non* fare scansione automatica per individuare i dati personali, perché non sarebbe affidabile.
- **Impostazioni candidate:** massimo dimensione file configurabile solo entro limiti sicuri documentati; cartella dell'ultima importazione facoltativa e locale, che **non** viene salvata nei backup; nessun toggle «consenti qualsiasi formato». I percorsi di backup restano scelti liberamente come ora.
- **Multilingua:** lingua del sistema e selezione manuale IT/EN/ES/FR/DE, fallback inglese; stati, errori, unità e formati data localizzati. `document_id`, `mime`, stati/chiavi export in forma canonica indipendente dalla lingua; **nome originale e testo inserito dall'utente invariati**.

## 7. Interoperabilità, backup, versioni e dipendenze

I **backup attuali** sono completi SQLite + `media/` con manifest SHA-256 e controllo specifico per gli allegati degli incassi nello schema v6. Per gli **eventuali nuovi collegamenti** va progettata una verifica analoga: nessun backup considerato riuscito se una relazione a un documento obbligatorio punta a un file assente, fuori dalla cartella o incongruente. Il manifest resta un inventario dei byte, non certificazione giuridica. Verificare anche che il ripristino di versioni precedenti non perda né reinterpretI documenti; non promettere compatibilità in lettura con un'app più vecchia che non conosce il nuovo schema.

L'[esportazione CSV esistente](../../DATA-PORTABILITY.md) comprende **otto registri** (inclusa `gallery`), ma non è un archivio di binari reimportabile. Se approvata TECH-001, definire un contratto aperto versionato per **metadati + link UUID + file + hash** e round-trip in archivio vuoto, senza percorsi assoluti; non ritenere il solo CSV sufficiente. Gli allegati incasso correnti richiedono una decisione separata su coesistenza o eventuale convergenza **non distruttiva**.

**Dipendenze progettuali:** revisione delle assunzioni di `db.rs`, `backup.rs`, `receipt_attachments.rs`, `gallery.rs`, `media_audit.rs` e dei limiti di archiviazione; definizione nel filone UX dei contesti abilitati. La PR corrente **non modifica questi file**. Collaborazione eventuale via esportazione selettiva e file aperti, non sync cloud o account.

## 8. Alternative, complessità e rischi

| Alternativa | Beneficio | Limite / rischio |
| --- | --- | --- |
| **A. Continuare a usare cartelle esterne** | Nessun lavoro software | Legami manuali, perdita dei file durante trasferimento, backup non autosufficiente per documenti esterni |
| **B. Aggiungere un allegato specifico per ogni nuova scheda** | Semplice nel breve periodo, segue l'esempio degli incassi | Logiche ripetute, limiti incoerenti, difficile riuso fra opere, QA onerosa |
| **C. Livello comune locale e tabelle di link tipizzate (ipotesi favorita)** | Regole uniformi di custodia/verifica e riuso, coerenza del backup | Migrazioni e sicurezza complesse, rischio di ampliare troppo il perimetro MVP |
| **D. Conservare BLOB in SQLite** | Un unico file database | DB molto grande, I/O/backup più onerosi, differenze rispetto al media store esistente |

**Complessità:** **medio-alta** per acquisizione e collegamento a opere senza regressioni; **alta** per riferimento multidominio, versioning, grandi file e compatibilità completa. **Rischi:** perdita di allegati alla cancellazione, fuga di documenti privati tramite export/MCP, backup incompleti, file maligni, crash tra copia e DB, storage eccessivo, ambiguità «documento presente» vs «documento autentico». Mitigazioni: scope ridotto, originali intatti, backup autosufficiente verificato, nessuna cancellazione automatica, anteprima di condivisione e test di crash.

## 9. Test e criteri di accettazione per **un'eventuale** realizzazione

1. Archivio v6 con vecchie ricevute: aggiornamento futuro in staging e riapertura **senza modificare** documenti, hash, API di lettura/export e vincolo «uno per incasso». Migrazione con dati di prova, mai dati personali reali.
2. Collegamento dello **stesso** PDF a due opere; rimozione del solo collegamento della prima opera lascia entrambi il file e il legame della seconda. FK rifiutano entità inesistenti e duplicazioni dello stesso link.
3. Importazioni PDF/JPEG/PNG/WebP con nomi Unicode, file vuoti, estensione ingannevole, file troppo grandi, symlink e cartella scollegata: esiti spiegati, **nessun record monco** e nessuna modifica all'originale.
4. Interruzione durante copia e commit SQL: riapertura con stato coerente, nessun riferimento a file incompleti; orfani eventuali sono segnalabili senza cancellazioni silenziose.
5. Corruzione di un documento e ricalcolo del solo manifest del backup: ripristino **rifiutato** quando l'hash d'acquisizione non coincide, senza sostituire l'archivio in uso; verifica che il backup includa ogni documento linkato.
6. Esportazione di copia su cartella scelta (anche USB/NAS montato): hash uguale all'originale archiviato, niente sovrascritture o file lasciati parziali; nessun percorso assoluto interno rivelato per default.
7. Confronto di archivi di test di diversa dimensione: memoria limitata e controllata, tempo I/O e annullamento documentati; prove native Windows/macOS/Linux e test accessibilità tastiera/screen reader.
8. UI e report in IT/EN/ES/FR/DE con fallback inglese; nomi dei file e contenuti utente conservati byte-per-byte, nessuna chiamata di rete, nessuna esposizione automatica di file via MCP.
9. Eventuale esportazione aperta con manifest documentato: reimportazione in archivio vuoto di **byte, UUID, contesti e hash identici**, senza perdere ricevute storiche.

## 10. Fasi possibili **non approvate**

- **Fase 0 — verifica del bisogno:** chiedere a un piccolo campione di archivisti/artisti/gallerie quali documenti tengono fuori da Repertica, quali condividono e quanti sono. Raccogliere esempi sintetici e anonimi.
- **Fase 1 — studio di compatibilità e minacce:** solo prova descrittiva delle relazioni opere–documenti, riuso dell'acquisizione esistente, regole di backup/export, UX minimizzata; nessuna migrazione storica.
- **Fase 2 — eventuale pilota** *solo dopo approvazione:* associazione opzionale a opere, copia locale verificata, collegamenti multipli, esportazione consapevole; senza OCR, folder watch o automatismi.
- **Fase 3 — eventuale estensione:** altri contesti documentali soltanto se i relativi modelli del filone gestione sono approvati, accompagnati da round-trip del formato aperto e collaudi su grandi archivi.

## 11. Questioni aperte per revisione del proprietario

1. Quali **due tipi di documento** vengono più spesso cercati accanto a una scheda opera? Quanti utenti hanno esigenze oltre alla ricevuta d'incasso già gestita?
2. Conviene una prima associazione solo alle opere, o servono subito link a più entità? La semplicità iniziale deve prevalere sulla generalità?
3. Il limite corrente di **12 MiB** basta per le scansioni professionali, oppure è necessaria una fase separata di file grandi con streaming e quota spazio?
4. Documenti identici per byte devono poter avere titoli e contesti diversi? Quanti vengono riutilizzati su più opere?
5. Quale politica conservativa di scollegamento è accettabile quando il file è citato da documenti storici? È ammissibile **non cancellare** i file nella fase iniziale?
6. Deve esistere un export di **soli documenti selezionati** senza metadati sensibili, o è preferibile lasciare tutto all'esportazione consapevole del singolo file?
7. Quali criteri pratici giustificano introdurre questa funzione **prima** di completare QA backup/ripristino nativi e portabilità TECH-001?

**Conclusione:** proposta tecnica subordinata alla domanda reale e alla compatibilità con il sistema esistente. **Proposta — da valutare**, non approvata, non assegnata allo sviluppo.