# REPS-TECH-037 — Ripristino interrotto: avvio protetto e recupero assistito delle copie conservate

- **ID stabile:** REPS-TECH-037
- **Data ricognizione:** 2026-11-22
- **Stato:** **Proposta — da valutare**
- **Filone:** Strumenti e architettura; **esclusivamente analisi e documentazione**
- **Priorità proposta (non approvata):** **P1–P2 condizionata** per la prevenzione di selezioni accidentali dopo arresti durante un ripristino; **P2–P3** per un'interfaccia avanzata di ispezione se i test dimostrano che bastano istruzioni operative
- **Nessuna decisione, approvazione, assegnazione o modifica software autorizzata.**

## Motivazione, utenti e riscontro su `main`

Un archivio di opere e collezionisti è spesso irripetibile. Dopo uno spegnimento, un disco pieno o un'interruzione mentre si ripristina una copia, l'utente dovrebbe poter capire **quale archivio viene effettivamente aperto**, dove si trovano la versione precedente e quella candidata, e come recuperarle senza cancellazioni. Questo è particolarmente importante per artisti con certificati storici, gallerie con incassi/documenti e archivi grandi con fotografie e RAW.

**Comportamento verificato staticamente su `main` il 22/11/2026**:

1. [`backup.rs`](../../../src-tauri/src/backup.rs) implementa già un ripristino prudente: verifica il backup selezionato, copia in staging, controlla nuovamente SQLite, hash e media, mette l'archivio precedente nella cartella fissa `.repertica-studio-restore-rollback`, attiva il nuovo archivio mediante rinomina, e conserva il precedente con nome `ReperticaStudio-before-restore-...`. In caso di errore sono previsti tentativi di rollback. **Non è proposta la sostituzione di questa pipeline.**
2. `recover_interrupted_restore(data_dir)` viene invocato da [`lib.rs`](../../../src-tauri/src/lib.rs) **prima** di `db::open`. Se esiste il rollback e manca la cartella attiva, prova a rinominare il rollback nella posizione attiva. Se esistono entrambi, rinomina il rollback in una cartella `ReperticaStudio-before-restore-...` e prosegue: **non confronta prima la leggibilità/coerenza semantica dei due candidati**.
3. [`db.rs`](../../../src-tauri/src/db.rs) usa `create_dir_all`, poi `Connection::open`: all'avvio ordinario queste primitive possono creare una nuova cartella e un SQLite vuoto se il file non esiste. La combinazione **ipotetica** di directory attiva presente ma priva di database + rollback esistente merita una prova specifica: lo stato attivo non va automaticamente interpretato come archivio riuscito.
4. Lo staging di ripristino e le eventuali cartelle `.repertica-rejected-restore...` sono distinte dai backup formali con manifest; non emerge nei percorsi di avvio consultati **un'interfaccia dedicata** alla scelta assistita fra cartelle residue. La documentazione [MVP](../../MVP.md) afferma correttamente che un rollback interrotto viene rilevato al riavvio: il presente studio valuta **come esplicitare e convalidare** l'esito, senza negare le tutele già esistenti.
5. Non sono stati eseguiti spegnimenti forzati, benchmark, test su USB/NAS o prove native in questa ricognizione. **Nessuna perdita di dati o malfunzionamento reale è affermato.** L'opportunità è derivata dalla lettura del flusso.

**Destinatari:** chi ripristina un archivio dopo guasto o migrazione, chi lavora con centinaia di fotografie e documenti economici, chi fornisce assistenza in presenza e chi non ha competenze per esaminare cartelle nascoste nel filesystem.

## Perimetro e anti-doppioni

- [TECH-009](REPS-TECH-009-catalogo-verifiche-conservazione-backup.md) studia **catalogazione, verifica e conservazione dei backup completi in cartelle scelte**; qui si studia la decisione di **avvio dell'archivio attivo dopo un ripristino interrotto**, compresi rollback/staging interni che non costituiscono backup pubblicati.
- [TECH-012](REPS-TECH-012-migrazioni-archivio-preflight-copia-sicurezza.md) riguarda il checkpoint **prima di una migrazione di schema**; qui si limita l'analisi alla sequenza di **restore**, indipendentemente dalla versione del software. Se una migrazione fosse necessaria, non deve partire automaticamente durante un recupero incerto.
- [TECH-022](REPS-TECH-022-preflight-spazio-operazioni-locali.md) stima lo **spazio prima** dell'operazione; questa scheda tratta decisioni dopo arresto e visibilità delle copie, non quote di disco.
- [TECH-003](REPS-TECH-003-diagnostica-integrita-media.md) esamina media dell'archivio attivo; il recupero non deve trasformarsi in riparazione implicita di fotografie.
- [TECH-011](REPS-TECH-011-profili-multiarchivio-apertura-sicura.md) riguarda archivi indipendenti scelti intenzionalmente. Una cartella di rollback **non** è un nuovo profilo utente.
- Confrontato l'indice delle **41 proposte Gestione e UX** della [PR #14](https://github.com/Cesare-ml/Repertica-Studio/pull/14): nessuna di esse studia questa barriera di avvio post-interruzione. Nessun file di tale filone è toccato.

## Casi d'uso e flussi UX ipotetici

1. **Ripristino normale concluso.** Nessun passaggio aggiuntivo: dopo successo verificato, Repertica apre l'archivio attivo come oggi, esplicitando nelle impostazioni la posizione della copia precedente già conservata.
2. **Interruzione prima della rinomina dell'attivo.** Se esiste soltanto uno staging incompleto e l'archivio originale è valido, mostrare in modo discreto una spiegazione: «Ripristino precedente non completato; archivio corrente conservato». Non promuovere uno staging parziale e non cancellarlo prima di una valutazione sicura.
3. **Cartella attiva mancante e rollback presente.** Prima di interpretare `rollback` come fonte autoritativa, verificare **senza mutazioni** che contenga un archivio riconoscibile (SQLite, versione, integrità, media pertinenti). Se gli esiti sono incerti, aprire una **modalità recupero** invece di creare/sostituire dati.
4. **Cartella attiva e rollback entrambi presenti.** Distinguere «attivo candidato A» e «copia precedente candidata B» senza dedurre quale sia più nuova dal nome; mostrare conteggi, versione schema, ora di ultimo controllo e limiti di verifica. **Non selezionare né eliminare una copia automaticamente quando la validità/identità è incerta.** La scelta di aprire/ripristinare un candidato richiede conferma comprensibile e conserva l'altro.
5. **Archivio attivo esistente ma SQLite mancante/non leggibile.** Bloccare la normale inizializzazione di un **nuovo database vuoto** in presenza di segnali di recupero ancora pendente; mostrare «Impossibile determinare l'archivio da aprire», con percorsi da copiare e possibilità di esaminare le copie. È una protezione proposta, non un difetto riprodotto.
6. **Supporto esterno non disponibile.** In caso di backup originale su USB/NAS, il recupero deve distinguere copia locale precedente, staging locale e fonte esterna scomparsa. Non supporre che l'assenza della USB significhi che un archivio sia vuoto o cancellato.
7. **Utente che non vuole decidere.** «Chiudi senza modificare nulla» e «Mostra istruzioni per recupero manuale» devono restare disponibili. È preferibile un'app in modalità sicura rispetto a un'attivazione non verificata.

La modalità di recupero, se realizzata, deve comparire **prima di qualunque `db::open` che crei o migri un database**, o usare un percorso bootstrap isolato non dipendente dall'archivio stesso. Non si presume che il frontend Angular corrente sia avviabile in assenza di database.

## Specifica funzionale descrittiva

**Principi da valutare:**

- Riconoscimento **soltanto** di nomi/percorsi riservati al prodotto, nella cartella padre conosciuta; non scandire arbitrariamente il computer, non usare pattern generici per cancellare directory.
- Ispezione read-only dei candidati per capire se contengano un archivio Repertica valido. `PRAGMA integrity_check` e `foreign_key_check`, controlli su media e schemi supportati, ma **senza** usare `db::open` perché può creare/migrare file. Le cartelle precedenti senza `manifest.json` non possono usare direttamente `inspect_backup_core` (pensato per backup completi): occorre discutere un preflight specifico, **non fingere** che il precedente archivio abbia hash storici firmati.
- Una «copia precedente» deve rimanere distinguibile da una «copia di backup completa verificata». Se priva di manifest, **non** dichiararla verificata SHA-256. Il nome `before-restore` non dimostra integrità o completezza.
- Indicare stati `Recognized`, `Suspect`, `Incompatible`, `Unavailable`, `NotInspected`; `Recognized` non significa file immutabili né certificazione di ogni contenuto multimediale.
- Nessuna cancellazione, sovrascrittura, modifica di schema o ripetizione automatica del ripristino per sbloccare l'avvio. Le rinomine di recupero andrebbero effettuate **solo dopo** scelta esplicita e verifica delle condizioni, considerando gli esiti di interruzioni precedenti.
- Una scheda di confronto tra candidati non deve mescolare contenuti di archivi diversi; UUID, contatori e date possono essere indizi, non prova univoca di identità o superiorità.

## Dati e API **puramente concettuali**

Nessuna nuova tabella o libreria è approvata. Un modello possibile, preferibilmente effimero:

```text
RestoreRecoveryScan {
  detected_state: NoRecovery | Interrupted | Ambiguous | NeedsManualAssistance
  candidates: RecoveryCandidate[]
  warnings: LocalizedCode[]
}
RecoveryCandidate {
  origin: Active | Rollback | Staging | Rejected | BeforeRestore
  path: LocalPath
  schema_version?: Int
  integrity: NotChecked | Passed | Failed | Uncertain
  media_check: NotChecked | Partial | Passed | Failed
  counts?: { artworks, collectors, sales, certificates }
  last_inspected_at?: Timestamp
  can_open_read_only: Bool
}
RecoveryPlan {
  selected_candidate_id
  preserved_candidates[]
  required_space?: EstimatedBytes
  operations_preview[]
  requires_explicit_confirmation: true
}
```

Chiamate eventuali, **non endpoint di rete né API pubbliche approvate**: `scanInterruptedRestoreReadOnly() -> RestoreRecoveryScan`, `inspectCandidateReadOnly(id) -> RecoveryCandidate`, `previewRecoveryChoice(id) -> RecoveryPlan`, `applyConfirmedRecovery(plan, userConsent) -> RecoveryResult`. Il token di un piano scade se directory, inodes/file ID, hash o disponibilità cambiano: nessuna garanzia esclusiva con la sola comparazione di path.

**Versionamento e interoperabilità:** il formato backup `manifest.json` v1 rimane importabile e verificabile; il recupero non trasforma silenziosamente cartelle di rollback senza manifest in backup portabili. Ogni archivio resta esportabile mediante il normale backup integrale; eventuali report di recupero sono file **JSON/Markdown aperti**, facoltativi e privi di dati personali per impostazione predefinita.

## Offline, sicurezza, privacy, impostazioni, multilingua

- **Offline-first rigoroso:** nessun account, abbonamento, endpoint, telemetria o sincronizzazione. Un NAS esiste per l'app soltanto come directory già montata dall'OS; non gestire credenziali di rete.
- Le cartelle candidate contengono potenzialmente **contatti, indirizzi, importi e fotografie**; non includere titoli, nominativi, note, percorsi completi o dati binari nei log/report condivisibili di default. Il report minimizzato può includere solo codici esito e conteggi, con scelta esplicita di esportazione.
- Ispezione senza eseguire SQL da contenuti non attendibili; impedire traversal, link simbolici/junction/reparse point, cartelle sostituite tra anteprima e operazione; evitare `rename`/cancellazioni con path generici. Il riconoscimento dei supporti esterni non deve «seguire» automaticamente mount cambiati.
- **Impostazioni ipotetiche:** nessun interruttore che permetta cancellazioni automatiche. Eventuale «Mostra posizione delle copie di recupero» e «Esporta rapporto tecnico locale» soltanto dopo l'apertura ordinaria o nella modalità di emergenza. I dati del recupero non devono richiedere impostazioni conservate nello SQLite che può non aprirsi.
- **Lingua:** usare lingua sistema con **fallback inglese** anche senza DB, e poi preferenza manuale IT/EN/ES/FR/DE quando disponibile. Codici di errore stabili non tradotti nei dati; testi, date, numeri, istruzioni e pulsanti localizzati. Focus iniziale, ordine da tastiera, lettori di schermo, informazioni non soltanto cromatiche, comandi distruttivi mai default.

## Dipendenze, rischi, alternative, complessità

**Dipendenze da esaminare:** `recover_interrupted_restore`, `restore_backup_core`, `inspect_backup_core` in Rust, sequenza `setup` di Tauri, `db::open` con creazione/migrazioni, stato dei media/SQLite WAL, guida di primo avvio e localizzazione pre-DB. Non scegliere ora librerie o architettura UI.

**Alternative:**
1. **Documentare un playbook di emergenza**, con nomi di cartelle, copie preventive e procedure per assistenza. Bassa complessità, ma l'utente scopre il problema fuori dall'app.
2. **Solo guardia di avvio**, che blocchi la creazione di un database vuoto in presenza di segnali ambigui, più messaggio con percorsi da copiare. Priorità relativamente alta, impatto funzionale contenuto.
3. **Modalità di recupero in sola lettura**, con validazione separata e scelta guidata, senza compiere azioni su richiesta implicita. Più utile ma richiede bootstrap e test nativi.
4. **Recupero integralmente automatico** basato su «esiste cartella attiva» o timestamp: **sconsigliato come prima alternativa** perché l'esistenza di una directory non ne certifica i contenuti.

**Rischi:** blocco di avvio per falsi segnali di recupero; scelta sbagliata che sostituisce dati buoni; confusione fra manifest di backup e archivi di sicurezza senza manifest; WAL/SHM che cambiano risultato di ispezione; spostamenti cross-volume non atomici; NAS intermittenti; disco pieno; due istanze di Repertica; vecchio schema che provoca una migrazione involontaria; credenziali/dati personali mostrati durante l'emergenza; test di crash non rappresentativi. Non dichiarare mai «recupero garantito».

**Complessità stimata:** **media** per una guardia conservativa di bootstrap e spiegazione dell'errore; **medio-alta/alta** per ispezione dei candidati, recupero guidato robusto e simulazione dei crash multipiattaforma. Sono stime progettuali, non attività assegnate.

## Collaudi e criteri di accettazione **ipotetici**

1. Matrice di arresti: prima dello staging, durante copia/validazione, dopo rinomina `active→rollback`, dopo `staging→active`, prima/dopo `db::open` e prima della rinomina finale `rollback→before-restore`. Per ogni scenario: enumerazione delle cartelle, comportamento al riavvio, hash e UUID preservati, nessuna nuova inizializzazione nascosta.
2. Caso critico: `rollback` valido + directory `active` esistente **senza** `repertica.sqlite`. Non eseguire apertura ordinaria che crei un archivio vuoto senza che l'utente conosca e valuti il rollback. Nessuna sostituzione silenziosa.
3. `rollback` e `active` entrambi validi ma distinti: mostrare entrambi e l'incertezza; il candidato non selezionato resta fisicamente intatto anche se l'operazione viene annullata o fallisce.
4. Solo `staging` parziale: non trattarlo come backup pubblicato né promuoverlo; non toccare l'archivio attivo se leggibile.
5. SQLite vecchio, futuro, corrotto, con WAL attivo, media RAW pesanti, snapshot storici dei certificati e allegati incasso; controllo read-only senza migrare o modificare gli originali. Distinguere righe SQLite leggibili da disponibilità effettiva dei media.
6. Spazio esaurito, permessi negati, directory sostituita, symlink/junction, supporto esterno assente, due istanze concorrenti, riavvio ripetuto durante tentativi di recupero: nessuna sovrascrittura implicita né eliminazione automatica.
7. Tests su Windows/macOS/Linux, SSD/USB/NAS fisici e interruzioni realistiche; verificare comportamento di `rename` e file descriptor su Windows. Nessuna soglia prestazionale o successo recovery dichiarato senza evidenza.
8. UI accessibile prima di caricare impostazioni DB, lingua OS/fallback inglese; vecchi backup v1 continuano a essere verificati e ripristinati; export integrale del recuperato rimane disponibile in una cartella scelta liberamente.

## Fasi **ipotetiche** (senza approvazione)

- **Fase 0 — Indagine e validazione:** riprodurre *in ambiente di prova* stati di arresto costruiti artificialmente, verificare in quali stati la pipeline attuale preservi/apra correttamente i dati, misurare frequenza/gravità e raccogliere esigenze di recupero dal proprietario e utenti pilota.
- **Fase 1 — Barriera minima:** controlli read-only all'avvio quando esistono segnali di restore pendente; blocco di inizializzazioni ambigue; spiegazione e istruzioni manuali, senza cambio automatico della sorgente.
- **Fase 2 — Ispezione guidata:** elenco locale dei candidati e loro limiti di affidabilità; relazione su cosa è verificato e cosa no; nessuna cancellazione.
- **Fase 3 — Recupero confermato:** anteprima dei percorsi, scelta esplicita, controllo pre-azione e mantenimento verificabile di tutte le alternative, a seguito di test di crash e filesystem.

## Domande riservate al proprietario

1. Vuoi che Repertica **preferisca bloccare l'avvio** e chiedere aiuto, piuttosto che creare un archivio nuovo quando compare una cartella `rollback` ambigua?
2. Ti basta una guida con percorsi e istruzioni, oppure serve un pannello grafico di recupero utilizzabile anche quando il database non si apre?
3. È accettabile conservare indefinitamente le cartelle `before-restore` (attuale protezione), oppure occorre un messaggio che spieghi lo spazio occupato senza proporre rimozioni automatiche?
4. Il caso d'uso prioritario è spegnimento/disco pieno oppure migrazione di versione? Quest'ultima rimane distinta da TECH-012.
5. In caso di due archivi SQLite tecnicamente sani, è desiderabile solo una scelta informata con istruzioni, o un confronto di poche informazioni non sensibili?
6. Quali sistemi operativi, filesystem, unità USB/NAS e condizioni di crash possono essere effettivamente provati prima di considerare una soluzione affidabile?
7. Quanto è frequente il ripristino presso gli utenti reali e quale sarebbe il costo di complicare l'avvio ordinario con un rilevamento più severo?

---

**Esito esclusivamente documentale:** **Proposta — da valutare**. Non conferma malfunzionamenti, non autorizza nuove funzioni, non prevede cancellazioni, non modifica codice, configurazioni, test o workflow.