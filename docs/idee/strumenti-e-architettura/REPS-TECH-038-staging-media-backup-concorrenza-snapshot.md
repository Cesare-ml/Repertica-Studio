# REPS-TECH-038 — Staging dei media e backup coerenti durante importazioni concorrenti

- **ID stabile:** REPS-TECH-038
- **Data di ricognizione:** 2026-11-22
- **Stato:** **Proposta — da valutare**
- **Filone:** strumenti e architettura; esclusivamente analisi e documentazione.
- **Priorità proposta:** **P1–P2 condizionata** alla riproduzione del rischio e ai test di concorrenza; P2–P3 per eventuali strumenti di recupero avanzati. Non approvata né assegnata.
- **Complessità stimata:** media per test deterministici e protocollo di coordinamento; medio-alta per staging uniforme e gestione dei crash su più piattaforme.

## Motivazione, destinatari e riscontri verificati in `main`

Repertica Studio è un'app Tauri/Rust/Angular con archivio SQLite e media locali; un backup completo crea una copia SQLite coerente, copia l'intera directory `media/`, calcola SHA-256, verifica e pubblica una nuova cartella indipendente. L'importazione di fotografie, documenti e RAW cerca invece, opportunamente, di evitare che una lunga copia da disco esterno blocchi l'interfaccia e le altre interrogazioni.

**Riscontro statico circoscritto (22 novembre 2026):**

- [`backup.rs`](../../../src-tauri/src/backup.rs), `create_backup_core`: prende `db.conn.lock()` durante `VACUUM INTO` e la scansione/copia di **tutti** i file di `media/`. Il commento attribuisce al mutex la coerenza di SQLite e media.
- [`gallery.rs`](../../../src-tauri/src/gallery.rs), `add_gallery_bytes_with_stage`: prima di prendere il mutex salva un file temporaneo `media/<uuid>.<ext>.tmp`, poi lo rinomina in `media/<uuid>.<ext>`; **solo dopo** collega il file alla fotografia in una transazione SQLite.
- [`receipt_attachments.rs`](../../../src-tauri/src/receipt_attachments.rs), `stage_receipt_file`, e [`raw_files.rs`](../../../src-tauri/src/raw_files.rs), `stage`: scrivono file direttamente nella directory `media/`, con nomi finali, **fuori dal mutex SQLite**; il collegamento del record avviene successivamente sotto mutex. I RAW possono essere particolarmente grandi.
- `backup.rs::safe_filename` ammette anche nomi terminanti in `.tmp`; `inspect_backup_core` richiede che ogni file fisico del backup compaia nel manifest e che ogni **riferimento SQLite** sia incluso, ma **non** impone che ogni file elencato nel manifest sia referenziato: è una scelta corretta per preservare media storici/orfani potenzialmente importanti.

**Lacuna ipotizzata:** il solo mutex SQLite non rende la directory fisica quiescente. Un backup può osservare una foto/RAW/allegato già creato ma non ancora collegato al database copiato; potrebbe includere un file in corso di scrittura, un `.tmp` o un file in seguito scartato. Un file così acquisito può anche essere correttamente hashato e superare la verifica di backup perché non è referenziato: **un hash valido prova i byte copiati, non che fossero uno snapshot logico desiderato**. Al contrario, un file rimosso durante la lettura può far fallire il backup. È un *rischio dedotto dal controllo del codice*, non un incidente osservato, né una prova che i dati attivi siano stati persi. Occorrono test con barriere di concorrenza per quantificarlo.

**Utenti:** artisti con archivi di fotografie RAW, gallerie che allegano ricevute mentre partono backup automatici, professionisti che lavorano da USB/NAS lenti. L'obiettivo non è impedire di lavorare durante il backup, ma evitare copie logicamente ambigue o inutilmente grandi e false rassicurazioni sulla loro coerenza.

## Confini e verifica anti-doppioni

- [TECH-003](REPS-TECH-003-diagnostica-integrita-media.md) è diagnosi dell'**archivio attivo** e recupero di file già mancanti: qui si studiano le **interferenze temporali fra importazioni e backup**.
- [TECH-009](REPS-TECH-009-catalogo-verifiche-conservazione-backup.md) è verifica/conservazione delle copie già pubblicate; [TECH-026](REPS-TECH-026-backup-compatti-snapshot-verificabili.md) è risparmio spazio mediante snapshot compatti; **questa proposta mantiene il formato completo v1**.
- [TECH-022](REPS-TECH-022-preflight-spazio-operazioni-locali.md) controlla la capacità disponibile; [TECH-037](REPS-TECH-037-recupero-ripristino-interrotto-avvio-protetto.md) gestisce l'**avvio dopo restore interrotto**. Il coordinamento col restore attivo è solo un caso concorrente di questa scheda, non una riscrittura del recupero di TECH-037.
- [TECH-031](REPS-TECH-031-allegati-bozze-posta-ripristinabili.md) riguarda bozze email con **percorsi esterni**, non le importazioni già gestite in `media/`.
- Le **42 idee Gestione e UX** della [PR #14](https://github.com/Cesare-ml/Repertica-Studio/pull/14) riguardano il modello applicativo e i flussi professionali (es. UX-028 giustificativi delle spese), non il protocollo di pubblicazione fisica dei media durante uno snapshot. Nessun loro file è modificato.

Non propone nuovo cloud, sincronizzazione, deduplicazione, cancellazione automatica degli orfani, modifiche ai certificati o allegati economici aggiuntivi.

## Scenari e flussi UX ipotetici

1. **Importazione RAW lenta + backup automatico:** l'utente allega un RAW da un disco USB e un backup pianificato parte mentre la copia non è finita. L'importazione continua a prepararsi localmente; il backup deve contenere **o** lo stato precedente completo **o** quello successivo completo, mai un file semiscritto presentato come allegato autorevole. Eventuale attesa va indicata senza obbligare l'utente a interrompere il lavoro.
2. **Foto di galleria creata, non ancora confermata in SQLite:** se l'utente annulla o un vincolo rifiuta la foto, la copia di backup non dovrebbe conservare un frammento di staging come contenuto intenzionale. Se un file storico senza riferimento esiste già, **non** deve essere scartato o nascosto per somiglianza del nome.
3. **Allegato incasso durante backup su NAS:** il software mostra `Backup in preparazione`, `Copia verificata` o un errore specifico; non dice «copia coerente» sulla sola base di checksum di byte presi durante una scrittura.
4. **Interruzione, ripristino o chiusura durante lo staging:** la sessione successiva distingue materiale temporaneo dell'app da media validi senza toccarli automaticamente. Il restore non deve rinominare la directory attiva mentre un writer sta ancora pubblicando lì.

**Interfaccia minima preferita:** nessuna nuova schermata permanente. Solo stato e messaggi di attesa/errore negli attuali flussi di allegato e Backup; un eventuale rapporto di staging residuo è una seconda fase e solo dopo aver validato il bisogno.

## Specifica funzionale e contratto di coerenza proposto

**Requisito da valutare:** definire un **punto di linearizzazione** osservabile: per ogni nuovo media, bytes definitivi e riferimento SQLite devono entrare nello snapshot come unità logica. Poiché il filesystem e SQLite non condividono una transazione atomica, progettare un protocollo di pubblicazione prudente, verificabile anche in caso di crash.

- Preparazione dei byte potenzialmente lenta in un'area **privata di staging separata dalla directory `media/` sottoposta a backup**; percorso/capienza/permessi da verificare. Un file incompleto non deve essere visibile come un media definitivo.
- Un guardiano di coerenza condiviso da backup, pubblicazione media e fase critica del restore (mutex/lease dedicato, o altro meccanismo equivalente) deve proteggere lo snapshot SQLite **insieme** alla promozione finale dei file; la lenta lettura dal disco sorgente resta fuori dalla sezione critica. Non presupporre che `db.conn.lock()` copra già l'I/O esterno.
- Nel passaggio finale, validare nuovamente entità, limiti e percorsi sotto protezione; pubblicare un file verificato senza sovrascritture, confermare il legame SQLite e definire cosa accade se un passaggio fallisce. In assenza di atomicità filesystem+DB, considerare un registro di intenti/recupero **solo se giustificato**; non introdurre tabelle permanenti per una mera prova.
- Il backup conserva la **regola conservativa esistente**: includere tutti i media storici effettivi, anche se privi di un riferimento corrente. Non dedurre «temporaneo» o «cancellabile» dalla sola assenza di una FK: occorre origine nota e lifecycle di staging esplicito.
- Snapshot e manifest v1 restano un backup **integrale e autonomo**, senza riscritture delle copie già create; eventuali retry devono riavviare l'operazione in una nuova cartella sicura, non riusare implicitamente un backup non pubblicato.
- Ripristino e backup automatico devono rispettare lo stesso perimetro di concorrenza, senza blocchi incrociati tra mutex SQLite, media e interfaccia. Le condizioni già testate di rollback e conservazione delle copie precedenti rimangono invarianti.

## Struttura dati e API: soltanto ipotesi descrittive

Una possibile struttura **temporanea** di staging (non specifica di implementazione) contiene `staging_id`, `kind` (gallery/receipt/raw/cover/altro), `requested_entity_id`, `temporary_path`, `final_relative_path`, `byte_count`, `sha256`, `created_at`, `phase` (writing/ready/publishing/linked/abandoned) e `cleanup_eligibility` provata. Non contenere email, importi o percorsi sorgente privati nei log ordinari. Eventuale metadata recovery deve poter distinguere **staging noto** da orfani storici **non identificabili**.

API **puramente illustrative**: `prepareMedia(source, budget) -> StagingToken`; `publishMediaAndLink(token, entity, expectedRevision) -> LinkResult`; `createConsistentBackup(destination) -> BackupSummary`; `inspectInterruptedStaging() -> ReadOnlyReport`. Non sono API richieste, approvate o da implementare ora. Il token non abilita scritture arbitrarie, non è esposto al MCP locale e non identifica un path esterno liberamente modificabile.

## Offline, privacy, sicurezza, impostazioni e multilingua

- Solo filesystem dell'utente e cartelle scelte, senza account, SaaS, abbonamento, telemetria o rete obbligatoria. Backup su USB e NAS montati tramite il sistema operativo rimangono consentiti.
- Staging può contenere foto, documenti fiscali e nomi sensibili: mantenere permessi stretti, limiti di dimensione, verifica contro symlink/reparse point/path traversal e file speciali, cancellazione manualmente giustificata e gestione prudente di crash e disco pieno. Non generare file temporanei in directory condivise o in chiaro fuori dal perimetro dell'app senza informazione.
- Una directory di staging residua non va **mai** eliminata automaticamente se la sua identità e la sua assenza di riferimenti non sono dimostrate; non compromettere originali, snapshot dei certificati o allegati che potrebbero avere un valore storico. Verificare le relazioni anche dopo migrazioni e ripristini.
- Impostazioni: riusare scelta della cartella di backup, periodicità e retention correnti; nessun servizio di background extra. Un eventuale limite di attesa/ritentativo e un rapporto locale di staging devono essere facoltativi, visibili e cancellabili solo con verifica/consenso.
- Lingua del sistema con fallback inglese e selezione IT/EN/FR/ES/DE quando presente; messaggi come «Importazione ancora in corso» e «Backup non completato» traducibili e comprensibili. Date e dimensioni localizzate; hash, UUID e nomi tecnici invarianti. Progresso e annullamento con tastiera e screen reader.
- Compatibilità: Windows/macOS/Linux, APFS/NTFS/exFAT e NAS montati; non assumere che `rename` sia atomico su volumi diversi, né che un `fsync` garantisca identico comportamento su ogni filesystem. Il formato di backup v1, SQLite e i dodici CSV esistenti restano leggibili.

## Dipendenze, alternative, rischi e complessità

**Dipendenze di analisi:** test nativi Windows/macOS/Linux, un harness concorrenziale riproducibile, requisiti del formato manifest v1, verifica di crash/recovery e delle operazioni Tauri che scrivono in `media/` (non soltanto gallery, receipt, RAW). Nessuna nuova libreria o configurazione proposta a priori.

**Alternative da confrontare prima di scegliere:**

1. **Solo test e documentazione:** se i casi non sono riproducibili o i backup attuali già soddisfano gli invarianti, nessun cambiamento funzionale. È l'alternativa preferita come prima fase.
2. **Coordinamento di breve durata + staging esterno al set di backup:** mantiene le copie lente fuori dal lock; maggiore lavoro sui confini tra database e filesystem, ma possibile soluzione conservativa.
3. **Lock globale durante tutta l'importazione:** semplice da ragionare ma può congelare UI, backup e query per RAW di grandi dimensioni; non preferito senza misure.
4. **Filtrare tutti i file non referenziati dal backup:** **non accettabile** senza una diversa decisione sulla conservazione, perché elimina fotografie storiche e altri media non collegati.
5. **Verifica post-copia che tutti i media abbiano FK:** non corretta per lo stesso motivo; una directory che passa SHA-256 può ancora contenere temporanei.

**Rischi:** deadlock per lock order diverso, perdita di file intenzionalmente orfani, leak di documenti intermedi, blocchi su USB/NAS, TOCTOU fra scan/copia/rename, crash dopo move del file ma prima del commit SQLite, false riuscite dei backup, tempi di backup più lunghi, spazio aggiuntivo nello staging, policy di recupero troppo aggressiva. La priorità reale va definita su prove ripetibili, non sulla sola lettura del codice.

## Criteri di accettazione / test *proposti, non eseguiti*

1. Test deterministico con barriere: pausa durante scrittura di `media/*.tmp` per galleria e durante copia diretta RAW/documento, poi creazione backup. Una copia pubblicata deve rappresentare un punto coerente documentato; nessun file *in costruzione* deve essere spacciato per archivio di un allegato.
2. Pausa **dopo** il completamento del file ma **prima** del commit SQLite; verificare entrambe le possibili sequenze (backup prima/dopo publish), stabilendo quali media storici non referenziati rimangono conservati.
3. Importazione annullata, vincolo DB fallito, disco pieno, app terminata e USB rimossa: il backup non acquisisce automaticamente file incompleti; l'archivio attivo e i backup validi precedenti non subiscono modifiche distruttive.
4. Backup con media storici davvero non referenziati, inclusi quelli degli snapshot malformati: mantenerli integralmente, senza pulizia automatica; hash e manifest restano verificabili.
5. Copie successive con RAW grande (fino al limite attuale), molti file in parallelo e backup automatico: nessun deadlock; query desktop e import non sono bloccati dall'intera fase di copia sorgente; misurare attese CPU/I/O/RAM.
6. Restore mentre staging è aperto: non rinominare il percorso sotto al writer, nessuna scrittura in un archivio ormai sostituito; rollback/recupero restano protetti ([TECH-037](REPS-TECH-037-recupero-ripristino-interrotto-avvio-protetto.md)).
7. Windows/macOS/Linux, USB, exFAT e NAS: comportamenti di rename, lock, sync e disconnessione verificati realmente; fallback esplicito senza dichiarazioni di successo improprie.
8. Offline reale, nessuna modifica a CSV, manifest v1, certificati o importi; messaggi IT/EN (fallback) accessibili da tastiera e screen reader.

## Fasi eventuali, non approvate

- **Fase 0:** riprodurre con test controllati l'interleaving di staging/backup senza usare archivi personali; registrare esattamente se si ottiene copia valida con media parziali, orfani transitori o errore.
- **Fase 1:** definire invarianti e diagramma degli stati di file e DB; comparare lock breve e staging separato dal set di backup; verificare tutte le vie di ingresso media e la sicurezza del restore.
- **Fase 2:** **solo su autorizzazione** sperimentare con archivi sintetici una soluzione locale minima, senza nuovo formato di backup.
- **Fase 3:** **solo dopo collaudi e decisione** eventuale UX di diagnosi dei residui di staging; non introdurre pulizia automatica di file di origine incerta.

## Questioni aperte per revisione del proprietario

1. Quanto frequentemente si importano foto, RAW o ricevute mentre parte un backup automatico? Quali dimensioni e filesystem sono rappresentativi?
2. È accettabile attendere pochi secondi per il punto di pubblicazione dei file, purché la copia lenta da USB/NAS continui senza bloccare le altre operazioni?
3. Una cartella di staging interna e temporanea è preferibile a una barriera applicativa globale? Qual è il comportamento più comprensibile se un backup parte mentre c'è un import incompleto?
4. I media storici senza riferimento corrente devono **sempre** continuare a essere inclusi nei backup integrali (come oggi)?
5. Se una chiusura improvvisa lascia un file di staging integro ma senza collegamento, va offerto solo un report, o una scelta esplicita di recupero? Chi può autorizzarne la cancellazione?
6. Prima di una qualunque implementazione, i test di concorrenza dimostrano un problema effettivo o bastano chiarimenti documentali e collaudo nativo?

**Conclusione:** proposta da valutare personalmente; nessuna modifica a codice, test, workflow, impostazioni, dipendenze, né approvazione o assegnazione allo sviluppo.
