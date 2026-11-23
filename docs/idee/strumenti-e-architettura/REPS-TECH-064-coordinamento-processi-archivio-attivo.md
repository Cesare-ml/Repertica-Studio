# REPS-TECH-064 — Coordinamento tra processi per proteggere l'archivio attivo

- **ID stabile:** REPS-TECH-064
- **Data ricognizione:** 2026-11-23
- **Stato:** **Proposta — da valutare**
- **Ambito:** confine tecnico fra più processi locali che accedono allo stesso archivio SQLite + `media/`; non collaborazione, non sincronizzazione e non editor multiutente.
- **Priorità proposta:** **P1–P2 soltanto se il conflitto è riproducibile** su almeno una piattaforma; diversamente **P2–P3** o sola documentazione preventiva. Nessuna approvazione né incarico allo sviluppo.
- **Complessità indicativa:** media per una barriera affidabile fra istanze della GUI; medio-alta per un protocollo fra GUI, processi MCP, ripristino/migrazione e filesystem diversi.

## 1. Problema, destinatari e motivazione

Un archivio locale non è necessariamente utilizzato da un solo *processo*: si può avviare Repertica Studio due volte, lasciarlo aperto mentre si avvia una seconda copia o usare in parallelo il processo `Repertica Studio --mcp` lanciato da un assistente IA. Un `Mutex<Connection>` protegge la connessione e le operazioni **nel singolo processo**, ma non impedisce che un secondo processo apra lo stesso database, modifichi un file `media/`, avvii una migrazione o conservi un handle SQLite mentre la prima istanza ripristina l'intera cartella.

**Destinatari:** artisti e gallerie che tengono aperta l'app per molte ore, archivisti su computer condivisi e utenti che consultano lo stesso catalogo mediante il server MCP locale. Il rischio diventa rilevante soprattutto per **backup, ripristino, migrazioni e file multimediali**, non per le ordinarie transazioni concorrenti di SQLite, che già dispongono dei propri meccanismi di locking. Lo scopo è evitare due "realtà" differenti del medesimo archivio senza imporre cloud, login, server, abbonamenti o limitazioni artificiali all'esportazione.

### Evidenze statiche su `main` (23 novembre 2026)

- [`src-tauri/src/lib.rs`](../../../src-tauri/src/lib.rs): `tauri::Builder` esegue `recover_interrupted_restore`, quindi `db::open` e avvia i backup programmati. Nei passaggi esaminati non compare una barriera esplicita *fra processi* prima del recupero o dell'apertura. Una ricerca del repository per `single_instance` non ha restituito occorrenze, né il [`Cargo.toml`](../../../src-tauri/Cargo.toml) letto elenca un plugin Tauri single-instance.
- [`src-tauri/src/db.rs`](../../../src-tauri/src/db.rs): `Database.conn` è un `std::sync::Mutex<rusqlite::Connection>`; `db::open` apre `repertica.sqlite`, attiva WAL e può applicare migrazioni attraverso `user_version`. Un mutex in memoria non vale come esclusione per altri processi.
- [`src-tauri/src/backup.rs`](../../../src-tauri/src/backup.rs): `restore_backup_core` prende il mutex, chiude la connessione di **quella** istanza e rinomina la directory attiva a rollback, poi quella in staging in directory attiva. `recover_interrupted_restore` può intervenire all'avvio. Questa logica è appropriata entro l'istanza corrente, ma non prova che nessun altro processo abbia ancora handle o media aperti.
- [`src-tauri/src/main.rs`](../../../src-tauri/src/main.rs) e [`mcp.rs`](../../../src-tauri/src/mcp.rs): `--mcp` è un processo distinto. La connessione al database è **in sola lettura**, con `SQLITE_OPEN_READ_ONLY` e `query_only`; è una protezione importante, **non** una garanzia che un lettore sia compatibile con la sostituzione della cartella o con una migrazione contemporanea.
- [`README.md`](../../../README.md), [`docs/MVP.md`](../../MVP.md): il modello d'uso è un archivio locale con backup verificati e ripristino sostitutivo; non è annunciata la modifica contemporanea multi-processo.

**Limite della verifica:** è un'**ipotesi di rischio derivata da lettura statica**, non una perdita dati osservata. Non sono stati avviati due processi né provate differenze Windows/macOS/Linux o NAS. Su Windows handle aperti possono impedire rinomine; su filesystem POSIX la rinomina di una directory non equivale a chiudere gli handle di un processo ancora in esecuzione. È indispensabile un test reale prima di decidere se introdurre protezioni.

## 2. Perimetro e anti-doppioni

- **[TECH-011](REPS-TECH-011-profili-multiarchivio-apertura-sicura.md):** poter **scegliere** e cambiare archivi indipendenti. Questa scheda riguarda invece **più processi sullo stesso archivio**, anche senza introdurre profili.
- **[TECH-033](REPS-TECH-033-salvataggio-condizionale-schede-obsolete.md):** conflitti di **versione del singolo record** in maschere già aperte. La protezione interprocesso non sostituisce il controllo delle revisioni.
- **[TECH-038](REPS-TECH-038-staging-media-backup-concorrenza-snapshot.md):** snapshot coerenti e file staging durante operazioni **della stessa istanza**; non estende automaticamente il mutex ai processi esterni.
- **[TECH-012](REPS-TECH-012-migrazioni-archivio-preflight-copia-sicurezza.md)** e **[TECH-037](REPS-TECH-037-recupero-ripristino-interrotto-avvio-protetto.md):** preflight/migrazioni e recupero dopo restore fallito. Qui si valuta **chi può iniziare** operazioni esclusive mentre altri processi sono ancora attivi.
- **[TECH-014](REPS-TECH-014-bridge-mcp-locale-lettura-permessi.md)** e **[TECH-036](REPS-TECH-036-mcp-archivi-grandi-paginazione-limiti-risorse.md):** permessi e scalabilità della lettura MCP; qui solo coordinamento del ciclo di vita di un lettore esterno.
- **[Gestione e UX — PR #14](https://github.com/Cesare-ml/Repertica-Studio/pull/14):** le schede professionali, tra cui UX-067 sulle modifiche non salvate, non trattano l'esclusione *fra processi*. Non si modificano file dell'altro filone.

Nessun ruolo utente, rete, server di coordinamento, lock cloud, trasferimento di dati fra installazioni o modifica ai formati di backup viene proposto in questa scheda.

## 3. Flussi d'uso proposti (solo descrittivi)

**A — Doppio avvio della GUI.** L'artista apre una seconda volta il programma con lo stesso archivio. Comportamento minimo da valutare: mettere in primo piano l'istanza esistente, senza aprire una seconda sessione di scrittura e senza mostrare un errore incomprensibile. Se il sistema non consente l'attivazione della finestra, mostrare un messaggio traducibile "Archivio già aperto su questo computer" e un'azione innocua per chiudere la seconda istanza. Nessuna chiusura forzata della prima.

**B — Assistente MCP in lettura e ripristino.** Un assistente sta consultando l'archivio tramite `--mcp`; l'utente sceglie "Ripristina" in Repertica. Prima della **fase distruttiva/sostitutiva** l'app richiede una disponibilità esclusiva dell'archivio; se ci sono lettori attivi, propone di terminare la sessione dall'applicazione che l'ha avviata e poi "Riprova" oppure "Annulla". Non terminare arbitrariamente processi esterni, non procedere dopo un timeout inventato e non assumere che il client IA abbia smesso di trattenere dati già letti. Durante un normale uso non sostitutivo, più lettori MCP potrebbero restare compatibili **solo se** i test dimostrano isolamento corretto.

**C — Seconda finestra e migrazione.** Una nuova versione dell'app tenta di aggiornare lo schema mentre un'altra vecchia istanza mantiene l'archivio aperto. Nessuna migrazione prima della decisione di esclusività. In caso di conflitto: nessuna scrittura, nessun database inizializzato ex novo per errore, percorso dell'archivio mostrato all'utente con cautela.

**D — Crash o scollegamento del volume.** Al riavvio, un vecchio marcatore testuale non prova che l'archivio sia ancora utilizzato. Il controllo distingue **lock effettivamente detenuto**, file marcatore eventualmente residuo e volume indisponibile; non cancellare né "sbloccare" automaticamente file o backup altrui. Se la destinazione USB/NAS non supporta in modo affidabile i lock previsti, mostrare incompatibilità e restare conservativi invece di garantire sicurezza inesistente.

**E — Due processi di sola lettura.** Se la compatibilità è accertata, due MCP possono consultare dati con il proprio snapshot SQLite senza bloccare tutta la GUI. La modalità MCP resta read-only; nessuna scrittura aggiunta per "facilitare" il protocollo. Un lettore con schema non supportato deve ricevere un errore esplicito, non una migrazione effettuata dal processo MCP.

## 4. Specifica funzionale ipotetica e struttura dati/API descrittiva

**Contratto minimo da verificare:**

1. Determinare un'identità della directory attiva tramite percorso canonico più informazioni prudenti sul volume; non confondere percorso uguale, clone di backup e archivio realmente identico. Evitare che collegamenti simbolici, alias o differenze di maiuscole permettano due sessioni distinte sullo stesso archivio. Dove l'identità non è certa, preferire l'errore e il riavvio esplicito.
2. Acquisire prima di `recover_interrupted_restore` e `db::open` un permesso **interprocesso** appropriato all'operazione. Non usare un semplice PID scritto su disco come unica prova: PID riciclabili e crash rendono la marcatura non autorevole. Il meccanismo deve usare primitive OS/file-lock realmente verificate su ciascun filesystem supportato.
3. Il riferimento di coordinamento deve trovarsi in una **posizione stabile**, non dentro la directory che `restore_backup_core` rinomina. I suoi dati non sono l'archivio e non vanno confusi col manifest del backup; è da decidere se risiedano in una directory dell'app o a fianco dell'archivio, rispettando la portabilità e i permessi del volume.
4. Se si consentono lettori contemporanei, valutare **permessi condivisi di lettura / esclusivi per restore, recupero e migrazione**; occorre misurare fairness e rischio di starvation. SQLite WAL rimane responsabile delle proprie transazioni: il permesso interprocesso **non** deve sostituire i lock transazionali del DB.
5. Tutti i punti d'ingresso pertinenti (GUI, `--mcp`, backup programmato, restore, futura scelta profili) devono consultare il medesimo contratto. L'acquisizione deve essere resistente alla finestra fra verifica e uso (*TOCTOU*); evitare un preflight che dice "libero" e poi procede senza mantenere l'esclusività.
6. Liberare il permesso durante la normale chiusura e, dove supportato, sfruttare il rilascio OS al crash. Se lo stato è ambiguo, **non** cancellare file estranei per sbloccare un archivio.

**Ipotesi di interfacce interne, NON implementate:**

| Concetto descrittivo | Campi minimi e significato | Nota di sicurezza |
| --- | --- | --- |
| `ArchiveAccessRequest` | `archive_path`, `purpose` = `interactive_write` / `mcp_read` / `restore` / `migration` | Solo input locale; path non esposto a provider IA |
| `ArchiveAccessLease` | `identity`, `mode`, `held_until_process_exit` o guardia RAII | Non basta memorizzare un booleano o PID; guardia effettivamente detenuta |
| `ArchiveBusy` | `operation` compatibile/incompatibile, `retryable`, messaggio localizzabile | Non esporre nomi di utenti, processi o percorsi privati senza consenso |
| `archive_access_status` | Stato sintetico: disponibile, occupato, percorso indisponibile, lock non supportato | Query informativa, **non** autorizzazione al restore: l'esclusività va acquisita atomicamente |

Si può iniziare con **solo una GUI per archivio** e verificare i casi MCP/restore, anziché costruire subito un gestore generale di sessioni o introdurre nuove tabelle SQLite. Una scelta più radicale (bloccare *tutti* gli MCP mentre la GUI è avviata) è semplice ma potrebbe annullare un caso d'uso dichiarato; va valutata, non imposta.

## 5. Offline, privacy, impostazioni, lingue e interoperabilità

- **Offline totale:** nessuna chiamata remota, nessun server, nessun discovery di rete. Anche senza Internet la protezione deve funzionare. Nessun account né servizio obbligatorio.
- **Dati privati:** eventuali metadati locali di diagnostica non devono includere contatti, titoli opere, testi, fotografie, chiavi, token MCP o contenuti di posta. I messaggi su processi e percorsi devono essere minimizzati, soprattutto su computer condivisi.
- **Impostazioni:** nessun interruttore "disabilita la sicurezza" come default. Solo eventuale preferenza informativa per richiamare la finestra già aperta, senza indebolire restore/migrazioni; niente scansioni continue inutili.
- **Multilingua:** lingua del sistema e fallback inglese, con localizzazione di stati/azioni/errori in tutte le lingue supportate (italiano, inglese, spagnolo, francese, tedesco). Terminologia accessibile: "Archivio aperto altrove", non codice OS grezzo; feedback testuale leggibile da screen reader, focus e tastiera.
- **Import/export:** il meccanismo non altera CSV, media, UUID, snapshot, manifest o recuperabilità dei backup. Copiare un archivio mentre è in uso **non** diventa automaticamente un export valido. Backup su cartella scelta, export completo e restore verificato restano disponibili, ma restore deve ricevere esclusività *prima* di mutare la directory.
- **Compatibilità:** Windows/macOS/Linux, processi GUI e MCP, filesystem locali, USB e NAS montati. Le primitive di locking possono comportarsi diversamente su SMB/NFS e nei filesystem non usuali: non dichiarare compatibilità senza prove. Le copie storiche devono rimanere leggibili.

## 6. Dipendenze, alternative e rischi

**Dipendenze tecniche da valutare:** primitive OS di single-instance e locking di file/processo; eventuale plugin Tauri; semantica SQLite WAL; gestione della chiusura della connessione in `backup.rs`; ciclo di vita MCP, migrazioni schema e ripristino; testi UI/i18n. Sono categorie di progettazione, **non** richieste di aggiungere dipendenze adesso.

**Alternative in ordine di costo:**
1. Documentare che non si devono aprire due istanze e, soprattutto, **provare** prima se questa condizione crea una reale finestra di rischio: nessuna implementazione.
2. Protezione minima: un solo processo GUI per archivio, seconda apertura che richiama la prima; verificare separatamente come gestire il processo MCP durante restore.
3. Protocollo condiviso/ esclusivo interprocesso solo per operazioni che ne hanno davvero bisogno, con test su tre OS; più complesso ma coerente con l'uso MCP simultaneo.
4. Coordinatore locale/daemon: da evitare nella prima fase perché introdurrebbe un servizio persistente e maggiori superfici d'errore; non serve presumere collaborazione.

**Rischi:** deadlock tra mutex SQLite e lock esterno, blocchi lunghi o starvation dei lettori, errata identificazione di alias/volumi, falsa sicurezza su rete, finestra TOCTOU, installazione di più versioni sullo stesso archivio, trattamento improprio del crash, blocchi dopo update, difficoltà ad avviare MCP mentre la GUI è aperta. Un locking **cooperativo** non può impedire a un programma esterno non cooperativo di modificare direttamente SQLite/media: resta indispensabile il backup verificato.

## 7. Criteri di accettazione e prove (futuri, non eseguiti)

- Due GUI avviate contemporaneamente sullo **stesso** percorso non producono due istanze autorizzate alla scrittura; la seconda non modifica media, DB o impostazioni del profilo.
- Apertura simultanea da alias del percorso, symlink e differenze di case verifica lo stesso archivio, oppure rifiuta prudentemente l'ambiguità. Archivi diversi rimangono indipendenti.
- Una GUI con SQLite aperto e un processo MCP in lettura, seguiti da richiesta di restore, non consentono una sostituzione mentre restano handle incompatibili: nessuno stato spezzato e nessuna perdita silenziosa; feedback di occupazione comprensibile.
- Migrazione schema, recupero restore e backup pianificato non attraversano un conflitto interprocesso non rilevato; l'operazione rifiutata lascia invariati archivio e copie già valide.
- Uccisione forzata di un processo prima, durante e dopo l'acquisizione non lascia un blocco permanente "fantasma"; non si sblocca automaticamente un lock ancora detenuto.
- Verifica con processi veri su **Windows 11, macOS Intel/Apple Silicon e Linux**, su disco locale, USB e almeno un NAS se dichiarato supportato. Monitorare timeout, errori, stato WAL, hash media e copie precedenti; non basta un test unitario intra-processo.
- Nessun peggioramento dell'MCP read-only, della disponibilità offline, delle prestazioni ordinarie, del fallback inglese o degli strumenti di accessibilità.

## 8. Fasi ipotetiche e domande riservate al proprietario

**Fase 0 — Solo ricognizione e prove:** riprodurre apertura di due GUI, GUI+MCP, GUI vecchia/nuova, restore e migrazione, distinguendo fra errori visibili e danno verificabile; misurare su Windows/macOS/Linux. Se nessun rischio sensibile viene riprodotto, fermarsi.

**Fase 1 — Eventuale difesa minima:** definire comportamento della seconda GUI e guardia prima delle fasi critiche, con messaggi tradotti; preservare tutte le copie e nessun intervento sui file esterni.

**Fase 2 — Solo se utile:** lettori MCP compatibili tramite permessi condivisi e restore esclusivo, prova sistematica dei crash e delle unità rimovibili.

**Fase 3 — Solo se approvata un'evoluzione multiarchivio:** riutilizzare la medesima identità/guardia di accesso per i profili, senza introdurre un secondo protocollo.

**Questioni da decidere personalmente:** è importante poter aprire deliberatamente più finestre della stessa installazione? Il client MCP deve continuare a interrogare l'archivio mentre la GUI lavora? Su NAS si vogliono supportare realmente archivi **attivi**, oppure soltanto destinazioni di backup? Come presentare un errore "archivio occupato" su un computer condiviso senza esporre processi dell'altro utente? Quale livello di test reali giustificherebbe P1 rispetto a una guardia P2?

**Decisione:** nessuna. Questa è **solo una proposta documentale**, senza implementazione, approvazione o assegnazione.
