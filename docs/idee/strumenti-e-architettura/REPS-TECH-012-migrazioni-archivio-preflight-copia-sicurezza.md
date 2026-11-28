# REPS-TECH-012 — Migrazioni dell'archivio con preflight e copia di sicurezza verificata

- **ID stabile:** REPS-TECH-012
- **Data della ricognizione:** 2026-11-21
- **Stato:** **Proposta — da valutare**
- **Priorità proposta:** **P1–P2**, prudenzialmente elevata per la salvaguardia dei dati, **non approvata**.
- **Ambito:** sicurezza del **passaggio tra versioni dello schema SQLite** quando una nuova versione di Repertica apre un archivio esistente. Non riguarda la distribuzione dell'aggiornamento software, i backup periodici, il versionamento delle singole opere o l'interoperabilità semantica.
- **Decisione e assegnazione:** nessuna. Specifica esclusivamente descrittiva; non autorizza interventi sul codice.

## Motivazione documentata e destinatari

**Lacuna riscontrata sul branch `main` il 21 novembre 2026.** In [`src-tauri/src/lib.rs`](../../../src-tauri/src/lib.rs) l'avvio chiama `recover_interrupted_restore` e poi `db::open`. In [`src-tauri/src/db.rs`](../../../src-tauri/src/db.rs), `open` crea directory e apre `repertica.sqlite`, legge `PRAGMA user_version`, rifiuta versioni future oltre lo schema 5 e applica le migrazioni v1→v5 con blocchi SQL transazionali. **Non si vede in questo flusso una copia integrale, autonoma e verificata dell'archivio precedente effettuata *prima* di una migrazione.** Le transazioni SQL sono una protezione utile, ma non equivalgono a un punto di recupero garantito dall'utente: migrazioni successive possono già essere state confermate, e il motore tratta anche immagini e dati che vivono fuori SQLite.

Esiste un [backup completo e verificato](../../../docs/MVP.md), con scelta libera della destinazione, manifest SHA-256, snapshot SQLite e media; il ripristino preserva una copia dell'archivio sostituito. Questo è **già implementato**, ma attualmente viene attivato dall'utente in Impostazioni, non è una barriera esplicita prima dell'upgrade di schema durante l'avvio. Il file [`docs/DATA-PORTABILITY.md`](../../DATA-PORTABILITY.md) documenta inoltre immagini storiche necessarie ai certificati e verifiche di inclusione del contenuto media.

**Utenti coinvolti:** tutti gli artisti, studi e gallerie che apriranno con versioni più nuove un archivio reale mantenuto negli anni; in particolare chi conserva molte fotografie, certificati e registrazioni economiche. La probabilità concreta di danni da migrazione non è misurata e non viene dichiarato un bug o una perdita già avvenuta. La motivazione è prevenire una classe di rischi prima che crescano il numero di versioni e la complessità dei dati.

## Perimetro e confronto per evitare duplicazioni

- **Funzione esistente:** backup/ripristino manuale verificato, con staging e copia precedente. Questa scheda valuta il *checkpoint pre-migrazione*, il momento in cui predisporlo, il comportamento in caso di blocco e la possibilità di ritornare ai byte precedenti.
- **[TECH-009](REPS-TECH-009-catalogo-verifiche-conservazione-backup.md):** elenca e rivalida nel tempo *copie già prodotte*. Non affronta l'accesso a uno schema più vecchio prima dell'applicazione automatica di una migrazione. I checkpoint eventuali dovrebbero essere riconoscibili nel catalogo futuro, senza delegargli le condizioni di sicurezza dell'upgrade.
- **[TECH-010](REPS-TECH-010-storico-revisioni-metadati.md):** revisione di *campi di una singola opera*, non recupero dell'intero archivio e della sua versione.
- **[TECH-001](REPS-TECH-001-pacchetto-dati-aperto.md):** esportazione/reimportazione *semantica*, non snapshot byte-coerente prima del cambio schema.
- **[TECH-006](REPS-TECH-006-backup-cifrati-portabili.md):** protezione facoltativa della riservatezza dei backup. Un eventuale checkpoint locale non deve presupporre l'esistenza della cifratura.
- **[TECH-011](REPS-TECH-011-profili-multiarchivio-apertura-sicura.md):** scelta fra archivi indipendenti; la migrazione deve essere valutata **per il solo archivio selezionato**, ma qui non si progetta un selettore.
- **[Issue #4](https://github.com/Cesare-ml/Repertica-Studio/issues/4):** aggiornamenti Tauri firmati e distribuzione software. La firma di una build non garantisce da sola la recuperabilità dei dati migrati.
- **[Issue #7](https://github.com/Cesare-ml/Repertica-Studio/issues/7):** collaudi reali dei backup/ripristini su sistemi e dispositivi. Una salvaguardia pre-migrazione dovrebbe usare quel piano di QA, senza sostituirlo.
- **[PR #14 — Gestione e UX](https://github.com/Cesare-ml/Repertica-Studio/pull/14):** provenienza, vendite, assicurazioni, inventari e altri processi di catalogazione. Nessuno propone una barriera di integrità all'apertura di uno schema SQLite precedente; non si modifica né si replica alcuna loro scheda.

## Casi d'uso e flussi UX proposti — senza funzionalità autorizzate

1. **Apertura ordinaria, schema già compatibile.** L'avvio rimane diretto: nessun obbligo di backup a ogni apertura e nessun nuovo passaggio per chi usa un archivio aggiornato. Un archivio nuovo (`user_version=0`, file assente) viene distinto da uno storico da migrare; il checkpoint di un database inesistente non ha senso.
2. **Rilevata migrazione (es. v4→v6).** Prima di qualunque mutazione, mostrare un riepilogo sintetico: versione sorgente e destinazione, motivo della protezione, quantità indicativa di spazio e percorso di copia scelto. Il controllo preliminare deve essere **realmente non mutante**: la semplice ispezione non deve creare cartelle, inizializzare un SQLite vuoto, cambiare `journal_mode` né completare migrazioni.
3. **Checkpoint pre-migrazione.** Creare una **copia autonoma** di SQLite coerente con WAL e di **tutti i media**, anche quelli conservati per certificati storici o non più referenziati; scrivere manifest e hash, verificare integrità, formato e corrispondenza dei file; pubblicare atomicamente in una nuova sottocartella, mai sovrascrivendo copie preesistenti. Riutilizzare in futuro contratti del backup già affidabili, ma non presumere che l'attuale `create_backup_core` possa essere invocato senza aprire e migrare prima il DB: è necessaria una soluzione architetturale separata da valutare.
4. **Cartella di destinazione.** Rispettare la scelta libera dell'utente: cartella locale, USB o NAS già montato se affidabile. Possibile default proposto: una cartella locale separata dall'archivio attivo, **esposta chiaramente e modificabile prima dell'operazione**. Da decidere se impedire un upgrade senza checkpoint verificato o offrire una modalità esclusivamente di consultazione quando non c'è spazio o supporto.
5. **Migrazione dopo verifica.** Solo dopo esito positivo e consenso secondo una politica da definire, applicare gli step necessari e controllare nuovamente `integrity_check`, vincoli FK, versione e invarianti di dominio (es. certificati immutabili, somme degli incassi non superiori alla vendita, percorsi media). Una migrazione transazionale per step non prova da sola la completezza del risultato.
6. **Errore o arresto improvviso.** Non procedere con una seconda migrazione cieca; al riavvio spiegare se esiste checkpoint verificato, versione attuale e possibilità di recupero. Qualunque recupero che sostituisca l'archivio dev'essere una **scelta esplicita** dopo ispezione, con conservazione anche dell'archivio fallito per diagnosi. Non cancellare automaticamente copie o media.
7. **Versione software precedente a quella dello schema.** Esporre «archivio creato con versione più recente» senza tentare downgrade o sostituire dati. Un checkpoint precedente l'upgrade può essere utile soltanto tramite **operazione esplicita** di recupero/clonazione, mai mediante conversione inversa implicita. Il software vecchio potrebbe non conoscere dati aggiunti dopo l'upgrade.
8. **Archivi esterni e futuro multiarchivio.** Se un dispositivo è disconnesso, non creare un database nuovo al percorso previsto; se si introdurranno più profili, ciascun checkpoint riguarda *solo* il profilo interessato, con origine e destinazione evidenti.

### Messaggi ed esperienza accessibile

Interfaccia breve: «È necessario aggiornare la struttura dei dati», «Proteggi prima l'archivio», «Copia verificata», «Aggiornamento completato» oppure «Aggiornamento interrotto: archivio non modificato / stato da esaminare». Evitare formule rassicuranti se il checkpoint è incompleto. Schermate utilizzabili da tastiera, stato annunciato ai lettori di schermo, progresso non comunicato solo con colori, errori con azioni comprensibili e con percorsi copiabili su richiesta (non esposti per default).

## Modello dati / API ipotetiche — **non** specifiche approvate

Possibili concetti, indipendenti da un'implementazione:

| Entità concettuale | Campi descrittivi e cautela |
| --- | --- |
| `MigrationPreflight` | `source_schema`, `target_schema`, `needs_migration`, `source_fingerprint?`, `estimated_bytes?`, `warnings[]`, `can_proceed`; nessuna scrittura durante l'ispezione |
| `PreMigrationCheckpoint` | `id`, `source_archive_identity?`, `created_at`, `schema_version`, `directory`, `manifest_format_version`, `verification_result`; non inferire identità dalla sola etichetta |
| `MigrationAttempt` | `attempt_id`, `source_schema`, `target_schema`, `checkpoint_ref`, `state`, `started_at`, `finished_at?`, `error_code?`; metadati minimizzati, non un registro di dati personali |
| `MigrationResult` | stato verificato, versione effettiva, percorso recuperabile, problemi da mostrare, *nessun* «successo» anticipato |

API puramente indicative: `inspectSchemaReadOnly(archive) → Preflight`; `createVerifiedPreMigrationCheckpoint(archive, destination) → Checkpoint`; `planMigration(preflight, checkpoint) → Plan`; `executeSchemaMigration(plan, consent) → Result`; `inspectRecoveryOptions(attempt) → Options`. Nessun endpoint HTTP, listener, integrazione MCP o protocollo remoto richiesto.

**Coerenza della copia:** se il database ha WAL attivo, evitare copia ingenua del file `repertica.sqlite` ignorando WAL/shm. Il checkpoint deve derivare da snapshot SQLite consistente e da una barriera alle modifiche concorrenti dei media. Per lo schema futuro prevedere un manifest con versionamento esplicito, distinguendo *versione del formato backup* da *versione dello schema* e da *versione del software*. **Compatibilità:** vecchi backup formato 1 già verificabili e ripristinabili non devono diventare invalidi; i vecchi programmi continuano a rifiutare schemi più nuovi, non li convertono.

**Integrità vs recuperabilità:** hash e integrità SQLite non dimostrano che un'app vecchia o nuova renderizzi correttamente tutti i dati. Il criterio comprende un'apertura verificata su una *copia di prova* senza alterare l'originale, in particolare per certificati, fotografie, vendite, incassi, spese, quote e impostazioni; non dichiarare recovery sicuro solo perché esiste un file.

## Offline, sicurezza, privacy, impostazioni e localizzazione

- Tutto avviene localmente, senza account, abbonamenti, cloud obbligatorio, telemetria né connessioni. Cartelle NAS eventualmente già montate restano filesystem, non servizi gestiti dall'app.
- Il checkpoint contiene **nomi, contatti, incassi, fotografie e percorsi privati**: gli snapshot correnti sono **in chiaro**. Avvertire chiaramente, suggerire una destinazione protetta e non copiare il contenuto nei log. L'eventuale cifratura facoltativa TECH-006 è questione distinta e non prerequisito.
- Impostazioni ipotetiche: cartella predefinita per checkpoint modificabile; promemoria della destinazione usata (senza scoperta di percorsi altrui); politica di conferma delle migrazioni; possibilità di esportare un resoconto tecnico minimizzato. **Non** introdurre silenziosamente una retention che cancelli backup precedenti.
- Verificare spazio, permessi, symlink/junction/reparse point, traversal, cartella bersaglio non inclusa nell'archivio, disconnessione, file speciali e rischio TOCTOU. Un percorso condiviso non deve indurre sovrascritture. Qualunque marcatura di tentativo interrotto non deve occultare l'originale.
- Lingua del sistema con fallback inglese e override già disponibile nell'app (IT/EN/ES/FR/DE); nuovi testi e codici errore stabili localizzabili, senza tradurre ID, nomi propri, versioni o valori persistiti. Se l'apertura del database fallisce prima di leggere `studio_settings.ui_language`, l'interfaccia di recupero deve poter usare **locale OS / fallback inglese**, non dipendere da un database già aperto.
- Nessun automatismo distruttivo e nessuna scrittura su backup preesistenti. L'utente continua a possedere ed esportare tutto senza un registro centrale obbligatorio.

## Dipendenze, alternative, rischi e complessità

**Dipendenze progettuali:** ciclo di avvio in `lib.rs`; migrazioni SQLite e controllo `PRAGMA user_version` in `db.rs`; snapshot `VACUUM INTO`, mutex, hashing e ispezione in `backup.rs`; media della galleria, file storici dei certificati e incassi; finestre native di selezione cartelle; traduzioni prima del DB; collaudi multipiattaforma e su storage rimovibile. Non è necessario un server né viene proposta una libreria proprietaria.

**Alternative da discutere:**

1. **Istruzioni «fai un backup prima di aggiornare»:** nessuna complessità aggiuntiva, ma l'utente può aggiornare l'app prima ancora di vedere il promemoria.
2. **Checkpoint automatico sempre nella cartella dell'app:** semplice per l'avvio ma può saturare il disco, concentrare tutte le copie sullo stesso volume o aggirare la preferenza di destinazione.
3. **Checkpoint verificato soltanto quando cambia schema:** compromesso proposto per limitare attrito e spazio. Necessita di preflight realmente read-only e gestione dell'errore.
4. **Migrazione su clone e sostituzione atomica:** protezione ulteriore ma maggiore duplicazione temporanea e complessità, soprattutto con WAL, media e dischi esterni. Va valutata rispetto al checkpoint+transazioni.
5. **Sola apertura in lettura per versioni incompatibili:** possibile estensione utile alla diagnosi, da non promettere senza specifiche per database e media.

**Rischi:** checkpoint incompleto scambiato per buono; copia consistente del DB ma incoerente con i media; disco pieno durante la copia; fallimento di più step consecutivi; avvio automatico ripetuto in loop; rollback che perde dati creati **dopo** l'upgrade; symlink o directory sostituiti; interazione con restore in sospeso; UX bloccante per archivi enormi; dati sensibili in copie inattese; migrazione di manifest/schema fuori compatibilità. Stimare spazio e durata **solo con misure reali**, senza garanzie inventate.

**Complessità indicativa:** **media–alta** per preflight non mutante, checkpoint verificato e flusso di errore; **alta** per migrazione su clone/attivazione atomica e compatibilità multiarchivio. Non viene proposta alcuna scadenza di sviluppo.

## Criteri di accettazione e piano di test **ipotetici**

1. Archivio schema v1/v2/v3/v4 con opere, collezionisti, galleria, certificati e media storici: il primo avvio con app più nuova produce un **checkpoint verificato del contenuto precedente** prima di qualunque modifica allo schema; dopo migrazione, i record, UUID, seriali, immagini, incassi dichiarati e quote sono integri. Non inventare dati in schemi che non li avevano.
2. `user_version=0` in un archivio nuovo non produce un checkpoint inutile; `user_version` futuro causa un messaggio chiaro senza downgrade né scritture durante il preflight.
3. L'interruzione simulata durante copia, hash, verifica, prima migrazione, tra migrazioni, verifica finale e primo riavvio non elimina l'originale o il checkpoint. Il sistema identifica uno stato recuperabile, senza cicli ciechi; il recupero richiede conferma e conserva le copie precedenti.
4. Disco pieno, percorso di destinazione non scrivibile, USB rimossa, NAS disconnesso, media mancante, file di origine modificato mentre si copia: **mai** mostrare «copia verificata» e **mai** procedere alla migrazione se la politica scelta richiede il checkpoint valido.
5. Con WAL attivo e molte foto, compresi media orfani o referenziati da snapshot storici, manifest/hash rilevano ogni mancanza. Eseguire prove Windows/macOS/Linux con filesystem reale, non solo test Rust isolati.
6. Due istanze del programma o un'operazione di backup/ripristino in corso non devono produrre una migrazione in concorrenza; testare lock, restart e crash. `integrity_check` e `foreign_key_check` non sono sostituti dei controlli di dominio.
7. Copie dei vecchi backup **formato 1** restano riconoscibili e ripristinabili da app compatibili; lo schema migra soltanto **dopo** che il ripristino completo è stato completato e se il checkpoint previsto è disponibile, senza compromettere la copia originale.
8. Interfaccia accessibile da tastiera, errori localizzati anche quando l'archivio non si apre, scelta autonoma della cartella, nessuna trasmissione a rete e nessuna sovrascrittura di file esistenti. Le misure di performance sono fissate solo dopo benchmark su archivi di taglie diverse.

## Requisiti certi, ipotesi e domande per il proprietario

**Accertato in `main`:** apertura dell'unico archivio Tauri prima di mostrare la UI; migrazioni SQL con `PRAGMA user_version` e transazioni; backup completi e verificati **su richiesta**; snapshot con media e certificati; ripristino sostitutivo con copia precedente; schema futuro rifiutato; lingua di sistema/fallback inglese e impostazione per-archivio. Il controllo `recover_interrupted_restore` tratta un ripristino interrotto, **non** documenta un recupero da migrazione interrotta. Nessun danno realmente avvenuto è qui dimostrato.

**Ipotesi da convalidare:** rischio sufficiente per anteporre il checkpoint a ogni migrazione, UI preliminare all'apertura del DB, cartella locale suggerita ma modificabile, migrazione su clone, stato persistente del tentativo, controllo applicativo post-migrazione. Sono specifiche di ricerca, non requisiti di prodotto.

**Questioni aperte:**

1. La salvaguardia prima del cambio schema dev'essere **obbligatoria** o si può permettere un esplicito «procedi senza copia»? Come trattare assenza di spazio?
2. La destinazione iniziale deve essere proposta in una cartella sicura e **liberamente modificabile**, oppure scelta ogni volta? Su USB/NAS è accettabile bloccare l'apertura in assenza del volume?
3. Per archivi grandi, meglio checkpoint completo + migrazione in sede o migrare **una copia** e attivarla soltanto dopo verifica?
4. La copia pre-migrazione deve essere contrassegnata da un nome speciale ma mantenere il **normale formato di backup verificabile**, senza introdurre una variante non recuperabile?
5. Dove registrare una sessione di migrazione interrotta se il DB non è apribile? Come evitare di mettere percorsi personali in log e crash report?
6. In presenza di backup molto vecchi e nuove versioni dell'app, quante versioni schema precedenti si garantiscono? Qual è la politica di supporto dei vecchi programmi?
7. La migrazione va differita quando la nuova versione del programma è stata installata ma l'utente vuole soltanto visualizzare un archivio, e come segnalare i limiti della consultazione?
8. La QA dei backup su USB/NAS della issue #7 va considerata condizione per abilitare nuovi schemi su archivi di produzione?

## Fasi eventuali — **non autorizzate**

- **Fase 0, studio e prove non operative:** confermare l'attuale sequenza di apertura e tutti i punti di mutazione, intervistare archivisti/artisti con database storici, studiare backup v1–v5 anonimizzati, testare concettualmente le opzioni di sicurezza.
- **Fase 1, progetto dettagliato:** contratto di preflight read-only, politica di scelta della destinazione e di blocco quando mancano spazio o verifica, matrice di compatibilità e dati dei checkpoint.
- **Fase 2, solo dopo approvazione specifica:** eventuale checkpoint completo prima di migrazioni, verifica, gestione dell'errore e integrazione di messaggi accessibili/localizzati.
- **Fase 3, decisione separata:** eventuale migrazione su clone, viewer read-only per schemi incompatibili, recupero guidato e integrazione con il catalogo TECH-009.

> Documento di ricerca: **Proposta — da valutare**. Nessuna modifica del software, nessuna decisione presa e nessuna assegnazione allo sviluppo.
