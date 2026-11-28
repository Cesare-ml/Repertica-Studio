# REPS-TECH-062 — Operazioni locali lunghe: avanzamento affidabile e annullamento nei soli punti sicuri

- **ID stabile:** REPS-TECH-062
- **Data ricognizione:** 2026-11-23
- **Stato:** **Proposta — da valutare**
- **Filone:** Strumenti e architettura; **analisi e documentazione soltanto**
- **Priorità ipotetica:** **P2 condizionata** alla frequenza di operazioni lunghe; P3 o nessun nuovo componente se i tempi restano brevi. Una tutela P1 richiederebbe prima una prova di rischio concreto di perdita dati.
- **Decisioni:** nessuna approvazione del proprietario, assegnazione o autorizzazione allo sviluppo.

## Problema, destinatari ed evidenze

Backup, verifica e ripristino possono leggere e copiare molti gigabyte di fotografie, RAW e allegati da USB o NAS. Un artista potrebbe non distinguere un'operazione ancora in corso da una bloccata, chiudere l'applicazione, staccare il disco o ripetere il comando. Al contrario, proporre un pulsante «Annulla» quando l'archivio sta cambiando posizione può far credere erroneamente che interrompere sia sempre sicuro. La proposta riguarda **un contratto trasversale per stato, avanzamento e richiesta di stop**, non una nuova forma di backup o un nuovo processo gestionale.

**Verificato staticamente su main il 23/11/2026:**

- [src/app/app.component.ts](../../../src/app/app.component.ts), nei metodi makeBackup, chooseRestoreFolder e restoreSelectedBackup, usa un indicatore generale busy e aspetta il risultato finale. [src/app/api.ts](../../../src/app/api.ts) invoca create_backup, inspect_backup e restore_backup senza canale eventi di avanzamento o richiesta esplicita di annullamento.
- [src-tauri/src/backup.rs](../../../src-tauri/src/backup.rs) esegue i tre comandi tramite spawn_blocking. create_backup_core fa snapshot SQLite con VACUUM INTO, copia il media store sotto un mutex, verifica la cartella parziale, la pubblica mediante rinomina e la verifica ancora. restore_backup_core valida, prepara una copia in staging, quindi scambia la cartella dell'archivio conservando un rollback. Queste garanzie **non devono essere indebolite**.
- [src-tauri/src/export.rs](../../../src-tauri/src/export.rs) genera e salva il CSV in un comando sincrono, senza un modello di job cancellabile; evita già la sovrascrittura e cancella il file incompleto quando una scrittura fallisce.
- [src-tauri/src/updater.rs](../../../src-tauri/src/updater.rs) e [src/app/api.ts](../../../src/app/api.ts) usano già un Channel Tauri per il progresso del download di aggiornamenti. È una prova di fattibilità per gli eventi, **non** un contratto automaticamente applicabile ai backup o una garanzia che il pulsante Annulla esista già.
- [docs/MVP.md](../../MVP.md) richiede ancora prove native su supporti esterni e NAS. **Non** è stato eseguito qui un benchmark, non è dimostrato un blocco reale e non sono state intervistate persone.

**Destinatari:** artisti e gallerie con molte fotografie, archivi professionali con RAW, utenti che fanno backup su supporti rimovibili, persone con connessioni NAS lente e tecnici che verificano copie di sicurezza.

## Perimetro, anti-doppioni e alternative esistenti

- [TECH-003](REPS-TECH-003-diagnostica-integrita-media.md) menziona l'avanzamento della *scansione media*; qui si definiscono semantica, limiti e sicurezza **comuni** dei job esistenti, senza reintrodurre quella diagnostica o la riparazione.
- [TECH-009](REPS-TECH-009-catalogo-verifiche-conservazione-backup.md) tratta elenco, verifiche periodiche e retention delle copie; qui **solo lo stato della singola operazione avviata** e i punti in cui può essere interrotta.
- [TECH-018](REPS-TECH-018-esportazioni-csv-scalabili-atomiche.md) propone writer CSV a flusso e pubblicazione sicura: un eventuale job di export **dipende da quella scelta**; questa scheda non ridisegna il file CSV.
- [TECH-022](REPS-TECH-022-preflight-spazio-operazioni-locali.md) valuta capacità *prima* del lavoro; qui si riferisce ciò che accade *durante*. [TECH-037](REPS-TECH-037-recupero-ripristino-interrotto-avvio-protetto.md) studia l'avvio dopo crash, non un annullamento volontario. [TECH-038](REPS-TECH-038-staging-media-backup-concorrenza-snapshot.md) studia gli effetti di import concorrenti, non la comunicazione del lavoro all'utente. [TECH-060](REPS-TECH-060-identita-destinazione-backup-programmati.md) verifica il supporto, non il progresso.
- [PR #14 — Gestione e UX](https://github.com/Cesare-ml/Repertica-Studio/pull/14) documenta flussi di opere, clienti e vendite: **questa proposta non aggiunge schermate gestionali né modifica file UX**.
- Prima alternativa **a costo minimo**: mantenere il busy attuale, rendere più esplicite le istruzioni «Non scollegare il disco» e verificare i tempi reali. Seconda alternativa: eventi specifici per il solo backup, senza infrastruttura generalizzata. Il contratto condiviso è utile solo se più operazioni ne hanno davvero bisogno.

## Scenari d'uso e flussi UX ipotetici

### A. Backup di molte fotografie su SSD o NAS

1. L'utente sceglie liberamente la cartella e avvia il backup come oggi; nessun agente esterno, account o servizio remoto.
2. Un pannello discreto mostra **fase effettiva**, contatore verificabile e percorso *dichiarato in forma abbreviata* solo nell'interfaccia locale: «Preparazione», «Snapshot SQLite», «Copia dei media», «Verifica dei file», «Pubblicazione», «Verifica finale».
3. Se si conoscono numero/byte totali, mostrare unità completate e totale; se il totale è ignoto, un indicatore indeterminato. **Non** esporre una percentuale complessiva inventata o «100%» prima della verifica post-pubblicazione.
4. «Richiedi annullamento» è disponibile **solo quando il backend può ancora abbandonare una cartella temporanea senza toccare l'archivio attivo né una copia già pubblicata**. La richiesta non equivale all'esito: mostrare «Arresto richiesto; conclusione della fase sicura».
5. Esito distinto: copia verificata e pubblicata; operazione annullata **prima** della pubblicazione; errore con cartella parziale da valutare; stato incerto. Una nuova copia non rimpiazza mai automaticamente una precedente.

### B. Ripristino con disconnessione del disco

1. Conferma esplicita della sostituzione dell'archivio come oggi, con copia precedente conservata.
2. La sola ispezione/lettura dello staging può essere candidabile all'annullamento, **se tecnicamente verificato**. Quando inizia lo scambio delle directory e il recupero della connessione SQLite, l'interfaccia mostra «Fase critica: annullamento non disponibile».
3. Il pulsante non deve mai uccidere il thread o forzare una chiusura nel mezzo di rinomina/rollback. Se il processo termina in modo improvviso, valgono le protezioni di recupero esistenti e la ricerca separata di TECH-037; nessuna «ripresa automatica» è promessa.
4. Disco disconnesso o hash incoerente: segnalare errore o recupero necessario, **mai** «annullato in sicurezza» senza esito controllato.

### C. CSV e operazioni automatiche

- Per i 12 CSV, un eventuale progresso può indicare «Generazione» e «Salvataggio». Finché render_csv produce una stringa intera, **non** presentare una finta percentuale per riga. L'eventuale cancellazione granulare dipende da TECH-018 e da una prova di consistenza.
- Il backup pianificato esistente resta opt-in, con frequenza e destinazione scelte dall'utente. Un job automatico dovrebbe dichiarare origine «Pianificazione autorizzata» e rendere distinguibile l'interruzione del job corrente dalla disattivazione delle **esecuzioni future**. Nessuna nuova attività silenziosa o nuova schedulazione proposta.

## Specifica funzionale, dati e API **solo descrittive**

**Contratto di lavoro ipotetico**, preferibilmente in memoria e senza nuova tabella SQLite nella prima fase:

- Operation: operation_id casuale per esecuzione; kind (BACKUP_CREATE / BACKUP_INSPECT / BACKUP_RESTORE / CSV_EXPORT); trigger (MANUAL / SCHEDULED_EXISTING); started_at; phase; state; cancellable_now; completed_units; total_units opzionale; processed_bytes e total_bytes opzionale; last_seq; outcome.
- OperationEvent: operation_id, seq monotono, timestamp, phase, unit_kind (FILES/BYTES/ROWS/UNKNOWN), completed, total opzionale, cancellable_now, message_code **localizzabile**. Perdita o riordinamento eventi → richiedere fotografia corrente dello stato, non ricostruire un successo da una percentuale.
- Outcome: SUCCEEDED_VERIFIED, FAILED, CANCELLED_SAFE, RECOVERY_REQUIRED o UNKNOWN; distinguere evento finale backend da semplice chiusura della finestra.
- Endpoint concettuali, **non da implementare ora**: startOperation(input) → operation_id; subscribeOperation(operation_id) → eventi; getOperationStatus(operation_id) → stato corrente; requestCancel(operation_id) → REQUESTED / NOT_SAFE / ALREADY_FINISHED. Il comando reale può restare Tauri IPC locale, senza porte HTTP.
- Modello di autorizzazione: solo l'interfaccia della sessione attiva può chiedere stop al proprio lavoro; nessun comando MCP per avviare o annullare operazioni. Un identificativo di job **non** è un token di accesso da inviare a software esterni.
- Riconciliazione con le primitive attuali: conservare hash, manifest v1, regole sul media store, snapshot SQLite, lock e rollback. Un evento è **informazione** e non giustifica un cambiamento di semantica del backup. Eventuali punti di cancellazione nel codice richiederebbero analisi transazionale e collaudo separati.

**Stati suggeriti:** QUEUED → RUNNING (fase nota o ignota) → {SUCCEEDED_VERIFIED, FAILED, CANCELLED_SAFE, RECOVERY_REQUIRED}. CANCEL_REQUESTED è uno stato intermedio; può diventare CANCELLED_SAFE solo **dopo** una verifica. Un errore di cleanup non deve essere etichettato come successo dell'annullamento.

**Concorrenza:** documentare quali job sono incompatibili e quale lock utilizzano. Non avviare in parallelo due ripristini, un ripristino con scritture dell'archivio o backup che distruggano la consistenza dello snapshot. Per verifiche read-only e file system lenti considerare limiti I/O; non introdurre una coda permanente o un servizio sempre attivo senza necessità reale.

## Offline, sicurezza, privacy, impostazioni e multilingua

- Il motore rimane **100% locale**, senza login, abbonamento, telemetria, socket, cloud obbligatorio o invio di dati. Destinazioni scelte dal dialogo nativo: cartelle locali, USB o NAS già montati. Il software non monta unità e non gestisce le loro credenziali.
- Eventi e report di default **non** contengono nomi di collezionisti, testi delle opere, percorsi assoluti, password, nomi originali degli allegati o payload di file; se servono dettagli diagnostici, mostrarli localmente e solo su richiesta consapevole.
- Validare input, path traversal, collegamenti simbolici, punti di ricontrollo e identità delle directory con le tutele esistenti; un pulsante «Annulla» non deve aprire una via per cancellare file estranei o un backup pubblicato. Unità rimossa, Windows file-lock e NAS instabile richiedono stati di errore espliciti.
- Preferenze: **nessuna impostazione obbligatoria nuova**; eventuali notifiche sonore/di sistema solo opt-in e rispettose della privacy, nessun avvio in background implicito. La chiusura dell'app durante un'operazione mostra cautela ma non può garantire protezione da crash/kill del sistema.
- Lingua di sistema con fallback inglese, selezione manuale già presente: phase e message_code stabili non tradotti; etichette, unità, errori e annunci screen reader localizzati. Non basarsi su colore o animazioni; aggiornamenti di progresso moderati per non sovraccaricare le tecnologie assistive.

## Dipendenze, compatibilità e complessità

**Dipendenze di progetto:** prima misurare tempi e volumi reali. Il Channel già disponibile per gli aggiornamenti può essere studiato come meccanismo di notifica, ma la sua presenza non risolve di per sé pause, annullamento, backpressure, perdita degli eventi, mutex Rust e consistenza. Evitare librerie o servizi nuovi senza una motivazione misurata.

**Compatibilità:** non alterare il formato backup v1, schema SQLite, dati delle vendite/certificati, CSV, scelta dei percorsi e politiche di pulizia. Se l'interfaccia non riceve eventi, deve mostrare il normale busy e attendere l'esito, non presumere fallimento o cancellazione. La prima fase può essere interamente informativa.

**Complessità stimata:** bassa-media per un progresso di fase coerente con i comandi attuali; **medio-alta** per cancellazione garantita nei punti sicuri e gestione eventi durante I/O grande, OS differenti e rollback; alta per una vera ripresa dopo riavvio, **esclusa** dalla proposta iniziale.

## Rischi e cautele

1. «Annulla» induce un falso senso di sicurezza: preferire «Richiedi arresto», con backend autorevole e possibilità di rifiutarlo in fase critica.
2. Byte copiati non equivalgono a copia verificata; controlli doppi, fsync e manifest rendono inattendibile una percentuale costruita per semplice peso dei file.
3. Emissione troppo frequente di eventi, mutex mantenuto durante copia, USB/NAS disconnessi e canale IPC senza subscriber possono peggiorare il comportamento rispetto a oggi.
4. Un job dichiarato «annullato» quando restano staging o rollback non classificati può nascondere dati recuperabili. Non eliminare automaticamente cartelle non riconosciute.
5. Troppa UI per archivi piccoli. Prima provare l'alternativa con etichetta di fase o semplice messaggio informativo.

## Prove e criteri di accettazione **ipotetici**

1. Backup piccolo e archivio sintetico con migliaia di media: eventi monotoni per fase, byte/contatori plausibili, memoria e CPU misurati; niente totale inventato prima della scoperta.
2. Confronto degli esiti con inspect_backup_core: «Backup riuscito» solo dopo la verifica completa della cartella pubblicata; copia annullata non presentata come utilizzabile.
3. Richieste di stop in ciascuna fase (inclusi VACUUM INTO, copia, verifica, rename e riapertura SQLite): classificazione REQUESTED/NOT_SAFE coerente, nessuna perdita dell'archivio attivo e delle copie precedenti. Se non esiste un punto sicuro, pulsante disabilitato e spiegazione.
4. Disconnessione USB/NAS, spazio esaurito, permesso revocato, file bloccato su Windows, Unicode e cartelle omonime: errori distinguibili, nessun percorso esterno sovrascritto o cancellato.
5. Crash forzato in staging e nel passaggio critico di restore: il comportamento è verificato tramite i protocolli di recupero, non simulato con il solo annullamento cooperativo.
6. Interfaccia con eventi persi, ricevuti tardi o duplicati, chiusura finestra e doppio clic: stato finale non contraddittorio, niente due job incompatibili.
7. Test di tastiera, screen reader e lingue IT/EN/ES/FR/DE, fallback inglese per locale sconosciuto; progresso descrivibile senza colore e senza annunci continui.
8. Nessuna richiesta di rete, password, account, nuova dipendenza e nessun cambiamento di byte del backup/CSV valido o dei dati dell'archivio.

## Fasi da sottoporre a revisione

- **Fase 0 — Evidenze, senza codice:** cronometrare copie e ripristini su piccoli/grandi archivi sintetici, SSD locale e NAS reale; raccogliere 3–5 scenari osservati di utenti. Se il busy è sufficiente, chiudere la valutazione senza nuova funzione.
- **Fase 1 eventuale — Trasparenza:** soli stati di fase ed esito accurati per un backup manuale; nessun annullamento e nessuna modifica ai percorsi di scrittura. Misurare miglioramento di comprensione.
- **Fase 2 eventuale — Annullamento limitato:** progettare punti di stop e cleanup **soltanto** dove il rischio è dominabile, con test di interruzione e invarianti del backup. Ripristino critico non annullabile.
- **Fase 3 eventuale — Riuso mirato:** ispezione, CSV (in coordinamento con TECH-018) e backup schedulati esistenti, solo se la qualità del contratto condiviso giustifica la complessità.

## Questioni aperte al proprietario

- Quante fotografie/RAW e quanti GB hanno gli archivi tipici, e quali operazioni superano una soglia di attesa percepita come problema?
- È più utile mostrare **la fase attuale** o offrire un vero «Richiedi arresto», con il limite che alcune fasi non si possono fermare?
- Per i backup automatici basta già il registro degli esiti oppure serve una notifica locale mentre l'app è aperta?
- Vale la pena un contratto comune dei job o conviene solo una piccola indicazione per i backup manuali?
- Prima di qualsiasi sviluppo: quali test nativi Windows/macOS/Linux, USB/NAS e con archivi grandi sono effettivamente disponibili?

**Conclusione:** proposta di affidabilità e chiarezza, **non** bug dimostrato, requisito approvato, nuova automazione o autorizzazione al codice. Nessun file del filone Gestione e UX è interessato.
