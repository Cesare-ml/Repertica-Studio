# REPS-TECH-009 — Catalogo delle copie, verifica periodica e conservazione prudente dei backup

- **ID stabile:** REPS-TECH-009
- **Data:** 2026-11-21
- **Stato:** **Proposta — da valutare**
- **Ambito:** strumenti tecnici, affidabilità del backup locale e gestione dello spazio; **non** implementazione.
- **Priorità suggerita:** **P2**, con una possibile prima fase P1 solo se il proprietario rileva problemi ricorrenti di ripristino o saturazione dei dischi.
- **Decisione del proprietario:** nessuna; tutte le opzioni seguenti sono ipotesi da discutere.

## Motivazione e destinatari

Il backup corrente è già una funzione reale: in **Impostazioni** si sceglie liberamente una cartella, il motore nativo crea una nuova sottocartella autonoma con snapshot SQLite, `media/` e `manifest.json`, calcola SHA-256, verifica database e immagini, poi pubblica la copia completa. Il ripristino verifica una singola cartella e conserva l'archivio precedentemente attivo. Tutto opera offline ([README](../../../README.md), [MVP](../../MVP.md), [backup.rs](../../../src-tauri/src/backup.rs)).

**Lacuna concreta:** `create_backup_core` genera sempre una nuova copia integrale; `inspect_backup_core` esamina un backup alla volta. Il manifest attuale (`format_version: 1`) include data, versione schema ed elenco di file/hash, **non** un identificativo stabile dell'archivio di origine. Non emerge un catalogo della storia delle copie in una cartella scelta, né una procedura assistita per rivalidarle o liberare spazio in sicurezza. Dopo mesi e molte foto un artista potrebbe non sapere quale copia sia sana, quante occupino spazio, se un disco USB sia effettivamente disponibile e quali copie sia prudente conservare.

**Utenti:** artisti che eseguono copie regolari su SSD/USB; gallerie con cartelle NAS montate; archivi professionali con molte fotografie; chi deve testare un ripristino su un secondo computer. Il beneficio e la frequenza del problema **non sono misurati**: richiedono riscontri reali.

### Confini anti-duplicazione

- **Non** ripropone la creazione, verifica SHA-256 e ripristino già implementati, né la QA fisica USB/NAS della [issue #7](https://github.com/Cesare-ml/Repertica-Studio/issues/7).
- [TECH-006](REPS-TECH-006-backup-cifrati-portabili.md) tratta la *riservatezza* del backup, non l'inventario e la conservazione delle copie. Un catalogo futuro dovrà rispettare le copie cifrate senza sbloccarle automaticamente.
- [TECH-004](REPS-TECH-004-ricerca-federata-archivi-locali.md) cerca **opere** dentro più archivi read-only, non verifica nel tempo copie di sicurezza.
- [TECH-001](REPS-TECH-001-pacchetto-dati-aperto.md) studia interscambio/reimportazione semantica, non la retention.
- [TECH-003](REPS-TECH-003-diagnostica-integrita-media.md) controllerebbe gli allegati dell'**archivio attivo**. Qui si opera su copie già create.
- [TECH-002](REPS-TECH-002-automazioni-locali-consapevoli.md) propone regole generiche con permessi: questa scheda **non** autorizza operazioni pianificate o cancellazioni automatiche.
- Le [proposte gestione e UX, PR #14](https://github.com/Cesare-ml/Repertica-Studio/pull/14), riguardano processi professionali, non il ciclo di vita degli snapshot.

## Scenari e flussi UX proposti

1. **«Esamina una cartella di backup».** L'utente sceglie una cartella già accessibile (locale, USB o NAS montato). L'app elenca soltanto sottocartelle che sembrano backup Repertica; non apre altre cartelle per cercare dati personali. Mostra per ciascuna copia: data dichiarata, percorso, peso apparente, versione/formato, esito e *momento dell'ultima verifica*. L'elenco può includere backup di archivi differenti e file estranei: nessuna unificazione implicita.
2. **«Verifica ora»**, per una copia o una selezione esplicita. Il controllo riesegue l'ispezione completa nativa, inclusi hash del contenuto, coerenza SQLite e immagini referenziate. Visualizza avanzamento, file esaminati, errori spiegati, eventuale annullamento sicuro e tempo dell'ultima **verifica completa**. Una vecchia spunta non equivale a «integro adesso».
3. **«Prepara un piano di conservazione».** Anteprima in sola lettura di spazio occupato e delle copie **candidate** a una futura rimozione; ordinamento per data e stato, distinzione tra copia verificata, non verificata, incompatibile, corrotta e non raggiungibile. Esempio: un SSD con 18 backup completi, due dei quali ancora da verificare; nessuna eliminazione avviene nel solo calcolo.
4. **«Rimuovi copie selezionate» (solo eventuale fase successiva).** Richiesta distinta e irreversibile: mostra nomi esatti, numero di file, destinazione, spazio stimato recuperabile e avverte che la cancellazione può compromettere il recupero storico. Preferire inizialmente **nessuna cancellazione dall'app**: il pulsante potrebbe aprire la cartella affinché l'utente gestisca i file dal sistema operativo. Se si approvasse una funzione di rimozione, servirebbero conferma forte, riconferma dello stato dei file, esclusione di dati sconosciuti e audit specifico.
5. **Supporto scollegato o backup danneggiato.** Se il dispositivo scompare durante scansione o verifica, l'operazione si ferma senza cambiare nulla. Mostrare «unità non disponibile», non «copia cancellata». Una copia parziale `.*.partial` non viene considerata un backup integro, né rimossa automaticamente.
6. **«Prova il ripristino».** Proporre una verifica reale su computer di prova o ambiente isolato *soltanto dopo progetto e approvazione separati*. Un catalogo con hash verdi non certifica da solo la riuscita operativa del ripristino.

## Requisiti verificati e vincoli non negoziabili

**Verificato in `main` il 2026-11-21:**

- Copie **complete e autonome** in nuove sottocartelle `ReperticaStudio-backup-...`, con SQLite, media e manifest versione 1.
- `inspect_backup_core` valida hash, struttura, versione, database e riferimenti immagini; `restore_backup_core` lavora su una singola copia ed esegue staging e rollback.
- Possono coesistere backup di **più archivi** e file non Repertica nella medesima cartella; il manifest v1 **non espone un archive ID affidabile**.
- I backup attuali sono **in chiaro**: nomi, contatti e dati economici di terzi possono essere leggibili.
- Le verifiche desktop su supporti USB/NAS reali restano oggetto della issue #7: non dichiararle collaudate.

**Requisiti per un'eventuale approvazione:** nessun account, servizio remoto, cloud, abbonamento o percorso imposto; nessun cambiamento non richiesto all'archivio attivo; preservare la lettura e il ripristino dei vecchi backup completi v1. La cartella si sceglie sempre liberamente. Non rimuovere, sovrascrivere o ricollocare file di terzi.

## Dati e contratti descrittivi (non API approvate)

Possibile modello **temporaneo, ricostruibile**:

- `BackupLocation { chosen_path, last_scanned_at? }`: riferimento alla cartella selezionata, non obbligatoriamente salvato.
- `BackupEntry { path, format_version, schema_version?, created_at?, declared_bytes?, observed_bytes?, archive_identity?: Unknown|Verified(id), status, last_full_verified_at?, last_error_code? }`.
- `status` distinto in `NotChecked | VerifiedAt(timestamp) | Invalid | Incompatible | Unavailable | Locked`: lo stato «VerifiedAt» è **storico**, non garanzia in tempo reale. `Locked` avrebbe senso soltanto con TECH-006 in futuro.
- `RetentionPreview { selection, total_candidates, estimated_reclaimable_bytes, protected_entries, warnings, identity_confidence }`: risultato sempre non distruttivo.
- Chiamate concettuali: `scanBackupLocation(chosenFolder) -> Entry[]`, `verifySelectedBackups(paths) -> VerificationReport[]`, `previewRetention(selection) -> RetentionPreview`. Una eventuale `removeConfirmedBackups(...)` è **fuori dalla prima fase**, da riesaminare separatamente.

**Questione strutturale:** non dedurre che due backup appartengano allo stesso archivio confrontando soltanto numero di opere, schema, nome della cartella o UUID di una singola opera. Potrebbero essere copie derivate o archivi diversi. Per politiche «conserva l'ultima copia sana di ogni archivio», valutare un identificatore non sensibile, stabile e persistente, introdotto in un futuro manifest v2 con migrazione compatibile; in v1, identità **sconosciuta** e nessuna eliminazione automatica basata su raggruppamenti presunti.

Il catalogo può essere ricostruito per scansione; un'eventuale cache locale deve essere chiaramente invalidabile, escludibile dal backup e non fonte autoritativa per cancellazioni. **Formato aperto** per un eventuale rapporto di verifica: JSON documentato o CSV locale con stato, timestamp e riferimenti non sensibili, esportato solo su azione esplicita. Non includere nomi dei collezionisti, contenuti delle opere, hash esposti online o percorsi privati in un report condiviso per default.

## Offline, sicurezza, privacy, multilingua e accessibilità

- Nessun accesso di rete avviato dall'app; un NAS è una directory **già montata dall'OS**. Nessuna memorizzazione di credenziali di rete o telemetria.
- Non seguire symlink, junction o reparse point durante scansioni o eventuali cancellazioni; canonizzazione, controlli di contenimento e nessuna traversal. Per le verifiche su grandi archivi, evitare letture integrali in memoria; imporre limiti e possibilità di annullare dopo ogni file.
- Proteggere da race/TOCTOU: tra anteprima, conferma e operazione un disco può essere sostituito o i file possono cambiare. In una possibile fase di rimozione, verificare nuovamente la directory e limitare azioni ai **soli backup espressamente selezionati e riconosciuti**, mai a una cartella padre intera. Non promettere eliminazione sicura o irreversibile dei dati da SSD/NAS.
- La cache o un rapporto di verifica può rivelare nomi di directory e abitudini di backup; conservarli solo localmente e con minimizzazione. Backup standard in chiaro sempre segnalati come tali. Copie future cifrate: mostrare solo metadata non riservati senza chiedere chiavi implicitamente.
- Lingua: preferenza dalla lingua di sistema, **fallback inglese**, selezione manuale; testi localizzabili anche in italiano/spagnolo/francese/tedesco secondo le lingue del progetto. Date/ore/pesi formattati per locale, unità coerenti, mai stati tradotti dentro i file tecnici.
- Tabelle navigabili da tastiera, focus visibile, screen reader, progressi annunciati in modo non invasivo, motivazioni leggibili anziché soli badge/colori, conferme comprensibili su operazioni distruttive.

## Dipendenze, alternative, complessità e rischi

**Dipendenze:** consolidare prima i test di backup e ripristino su sistemi e filesystem reali ([issue #7](https://github.com/Cesare-ml/Repertica-Studio/issues/7)); riusare il validatore nativo; ottenere esempi di cartelle con backup sani, corrotti, vecchi e di più archivi; definire UX affidabile di scansione e cancellazione. Nessuna nuova libreria è presupposta.

**Alternative da confrontare:**

1. **Solo guida/documentazione** alla conservazione di tre o più copie e prova periodica di ripristino: quasi nessuna complessità ma scarsa visibilità sullo stato effettivo.
2. **Catalogo e verifica in sola lettura**, senza rimozione: miglior rapporto iniziale fra utilità e rischio, possibile P2.
3. **Gestione assistita manuale** della retention: più comoda, ma rischio alto di perdita storica se un'identità viene interpretata male.
4. **Backup incrementali / content-addressed / deduplicati:** risparmio potenziale di spazio ma **non** equivalente al formato attuale di cartelle autonome. Richiederebbe un progetto separato di catene, garbage collection, recovery, versioning, crittografia e materializzazione di copie complete indipendenti. **Non parte della proposta iniziale**, non eliminare la modalità full backup.

**Complessità indicativa:** **bassa–media** per elenco/verifica con riuso delle primitive correnti, **alta** per cancellazioni guidate e conservazione basata sull'identità dell'archivio, **molto alta** per deduplicazione. Tali stime non sono tempi di sviluppo approvati.

**Rischi principali:** falsa rassicurazione da verifiche obsolete, dati personali nei log, errore di riconoscimento backup diversi, filesystem con permessi ambigui, volumi rimossi, scansione molto lenta, perdita irreversibile in caso di pulizia impropria, conflitti con backup cifrati e falsi conteggi su dischi con hardlink/sparse files. Priorità assoluta alla sicurezza dei dati rispetto al recupero di spazio.

## Collaudi e criteri di accettazione ipotetici

- In una cartella con backup validi, vecchi, corrotti e file non correlati, il catalogo mostra gli stati corretti **senza modificare un solo byte**; file estranei restano invisibili alle operazioni distruttive.
- Sottocartelle con stesso nome logico ma contenuti diversi, e backup di due archivi diversi, **non** vengono raggruppati come un unico archivio sulla base di ipotesi.
- La verifica completa riconosce manifest mancante, hash errato, SQLite incoerente, immagini mancanti, schema incompatibile e disconnessione del supporto; non mostra «verificato» quando ha controllato solo i metadati.
- Stato salvato con timestamp non viene spacciato per verifica attuale. Modificare un file dopo la scansione obbliga a rilevarne la modifica con una nuova verifica.
- Su cartelle grandi: misurare durata, memoria massima, annullamento e disponibilità UI su Windows/macOS/Linux, SSD/USB/NAS reali; soglie da stabilire **dopo** dati misurati, non inventate.
- Test di accessibilità e localizzazione (assenza locale, fallback inglese), separazione tra backup in chiaro e potenziali backup cifrati e report senza dati personali di default.
- **Se** in futuro fosse autorizzata la rimozione: nessun file fuori dalle directory selezionate deve sparire; prove con symlink, directory sostituita, disco scollegato, rollback della UI e copia sana unica. Senza identità verificabile, disabilitare qualunque retention automatica per archivio.
- Un backup completo v1 verificato continua a essere ripristinabile con il flusso esistente; i test di ripristino non si limitano ai soli hash.

## Fasi suggerite (nessuna autorizzazione)

- **Fase 0 — Ricerca:** raccogliere esempi anonimi d'uso, numero di copie, crescita annua, filesystem impiegati e richieste di ripristino; verificare la fattibilità dell'inventario con manifest v1 senza trattarlo come fonte d'identità.
- **Fase 1 — Solo lettura:** catalogo temporaneo di una cartella selezionata, filtri per stato e verifica manuale di copie scelte, rapporti locali minimizzati.
- **Fase 2 — Supporto decisionale:** confronto spazio/età e preview di conservazione, senza cancellazioni in-app; spiegare quali copie sono davvero verificate.
- **Fase 3 — Da decidere a parte:** eventuale cancellazione guidata dopo analisi d'identità, sicurezza, coerenza e collaudi; nessun comportamento automatico o pianificato per default. Deduplicazione fuori ambito.

## Domande aperte per il proprietario

1. Gli utenti conservaranno normalmente 2–3 copie oppure decine di copie dello stesso archivio? Dove (SSD, USB, NAS)?
2. Conta maggiormente **sapere che un backup funziona** oppure recuperare spazio? Serve una vista solo in lettura?
3. È accettabile una **rimozione in-app** o è preferibile lasciare la cancellazione al file manager?
4. Le copie di archivi diversi convivranno spesso nella stessa cartella? È opportuno introdurre un identificativo stabile e non sensibile nel futuro formato backup v2?
5. Quale strategia storica è desiderata (giornaliera/settimanale/mensile) e chi autorizza l'eliminazione? Vietare qualsiasi pulizia automatica finché non vi sia una scelta esplicita?
6. I rapporti locali di verifica devono essere esportabili, e in quale formato? Come evitare che espongano percorsi o nomi riservati?
7. Quanto è prioritario il lavoro rispetto alla QA del ripristino già prevista dalla issue #7?

---

**Esito:** esclusivamente **Proposta — da valutare**. Non aggiunge requisiti approvati, non avvia sviluppo, non crea issue e non autorizza cancellazioni o modifiche all'archivio.