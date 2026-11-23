# REPS-TECH-060 — Destinazione dei backup automatici: distinguere il percorso dal supporto effettivo

- **ID stabile:** REPS-TECH-060
- **Data ricognizione:** 2026-11-23
- **Stato:** **Proposta — da valutare**
- **Ambito:** affidabilità e riservatezza dei backup locali programmati su USB, dischi rimovibili e NAS montati; analisi soltanto, **nessuna approvazione o assegnazione**.
- **Priorità ipotetica:** **P2 condizionata** a rischi riproducibili e utenti che impiegano backup programmati su supporti rimovibili; **P3 o sola guida** se non emerge domanda. Un caso di scrittura di dati personali sul volume errato, realmente riproducibile, potrebbe giustificare una valutazione P1 per la guardia minima.
- **Invarianti:** desktop Tauri gratuito/open source, SQLite e media locali, offline-first, nessun account, cloud o abbonamento obbligatorio; backup e ripristino completi in cartelle liberamente scelte, nessuna cancellazione o trasferimento automatico non richiesto.

## 1. Problema, utenti ed evidenze nel codice

**Problema circoscritto:** un percorso raggiungibile non identifica necessariamente il disco o la destinazione che l'utente aveva scelto. Esempio: l'artista configura `E:\\CopieRepertica` sul proprio SSD USB; al riavvio Windows la lettera viene assegnata a un altro supporto contenente la stessa sottocartella. Il backup programmato può apparire riuscito, pur essendo stato salvato su un dispositivo **diverso**, con implicazioni di disponibilità e privacy. Anche una directory montata su Linux/macOS o una condivisione NAS riconnessa a un'altra risorsa può mantenere lo stesso *nome di percorso*. Non è stato dimostrato che il caso sia già accaduto a un utente Repertica.

**Riscontro diretto su `main` il 2026-11-23:**

- [`auto_backup.rs`](../../../src-tauri/src/auto_backup.rs): `backup_schedule.destination` è una stringa di percorso assoluto. `run_core` verifica che la directory esista (`is_dir`) e passa il percorso a `backup::create_backup_core`. Lo storico e la quota di conservazione distinguono la stringa del percorso; il codice consultato non rileva un'identità del volume persistente tra due esecuzioni.
- [`backup.rs`](../../../src-tauri/src/backup.rs): `create_backup_core` risolve il percorso, verifica che sia una directory e che non ricada nell'archivio attivo, crea una nuova sottocartella parziale e pubblica solo dopo snapshot SQLite + media + manifest SHA-256 e ispezione. **Queste protezioni sono già presenti e vanno mantenute**: non risolvono però l'equivalenza fra vecchio e nuovo supporto con lo stesso percorso.
- [`README.md`](../../../README.md) e [`docs/MVP.md`](../../MVP.md): backup programmati facoltativi con cartella scelta liberamente, inclusi supporti esterni e NAS. Il software funziona senza rete, salvo che l'utente usi deliberatamente un NAS montato.
- `apply_retention` di `auto_backup.rs` applica già controlli conservativi sulle cartelle dei backup creati dall'app, verifica le copie candidate e ricontrolla la configurazione prima di eliminare quelle eccedenti. **Non affermiamo che oggi cancelli backup altrui**: lo scenario principale qui è la *nuova copia destinata al volume inatteso*, non un difetto dimostrato della retention.
- Nessun test con sostituzione fisica dei volumi o riproduzione su Windows/macOS/Linux è stato eseguito in questa ricognizione. Non si assume che tutte le lettere di unità, mount o NAS cambino frequentemente.

**Destinatari:** artisti che ruotano dischi USB di backup, gallerie con postazioni condivise, studi che salvano su volumi cifrati rimovibili, archivisti che usano NAS montati. Il controllo ha utilità solo se distingue un problema reale meglio dell'attuale messaggio «cartella non disponibile».

## 2. Confini e deduplicazione

- [TECH-009](REPS-TECH-009-catalogo-verifiche-conservazione-backup.md) censisce e verifica **copie già prodotte** e la loro conservazione; questa proposta controlla **prima** dell'avvio programmato *dove* si sta per scrivere.
- [TECH-022](REPS-TECH-022-preflight-spazio-operazioni-locali.md) verifica capacità e precondizioni, non la continuità dell'identità della destinazione.
- [TECH-058](REPS-TECH-058-ricognizione-post-ripristino-nuovo-computer.md) ricorda di confermare impostazioni **dopo un ripristino su un altro PC**; qui il cambio indesiderato può verificarsi **sullo stesso PC**, senza ripristini e fra backup periodici.
- [TECH-006](REPS-TECH-006-backup-cifrati-portabili.md) riguarda cifratura e recupero della copia, non una scelta errata di destinazione. Cifrare facoltativamente ridurrebbe la divulgazione, ma non la perdita di ridondanza.
- [TECH-012](REPS-TECH-012-migrazioni-archivio-preflight-copia-sicurezza.md), [TECH-026](REPS-TECH-026-backup-compatti-snapshot-verificabili.md), [TECH-037](REPS-TECH-037-recupero-ripristino-interrotto-avvio-protetto.md) e [TECH-052](REPS-TECH-052-verifica-indipendente-backup-formato-v1.md) affrontano rispettivamente migrazione, dimensioni, recupero e verifica formato: nessun nuovo motore di backup o manifest proposto qui.
- Il filone [Gestione e UX, PR #14](https://github.com/Cesare-ml/Repertica-Studio/pull/14), esaminato fino a **REPS-UX-064**, tratta processi di opere, clienti e vendite. Questa guardia tecnica non propone nuove viste inventariali o amministrative e non ne modifica i file.

## 3. Specifica e flussi UX ipotetici

### A. Configurazione volontaria di una destinazione controllata

1. L'utente mantiene il selettore nativo di **qualsiasi cartella** accessibile. In Impostazioni → Backup automatici può scegliere **«Avvisa se il supporto della destinazione cambia»** (opzione proposta, non presente né approvata).
2. Una scheda leggibile mostra il percorso, il tipo di destinazione rilevabile (`locale`, `rimovibile`, `rete`, `sconosciuta`) e il grado di affidabilità del controllo: mai dichiarare che un identificatore è infallibile.
3. Se necessario, l'utente autorizza una *piccola marcatura locale della cartella selezionata*, in aggiunta agli indizi del filesystem. La marcatura contiene solo identificativo casuale e versione, niente nomi di opere, utenti, credenziali o ID hardware personali. **Non creare nulla nella cartella senza conferma**. La marcatura non costituisce prova crittografica: può essere copiata insieme alla directory.
4. L'utente conferma la destinazione; l'app non cambia la frequenza, le copie esistenti, la retention o i percorsi di altre cartelle. Se il sistema non espone un'identità stabile affidabile (per esempio particolari NAS), il pannello dichiara il limite e permette di preferire una riconferma manuale ai controlli ambigui.

### B. Backup programmato quando il dispositivo è cambiato

1. Prima di avviare una nuova copia viene effettuato un controllo *read-only* della destinazione attualmente risolta, confrontato con le evidenze memorizzate al consenso. Il percorso visibile può coincidere mentre volume/marcatura non coincidono.
2. Se il supporto è **assente, diverso o non verificabile** nella modalità protetta, **non scrivere il backup sulla destinazione dubbia**. Mostrare stato separato: «saltato: supporto da confermare» invece di «riuscito» o «archivio danneggiato». Nessuna nuova richiesta di credenziali, rete o installazione.
3. Quando l'utente apre Repertica, un avviso accessibile consente **Riprova verifica**, **Scegli altra cartella** o **Conferma questa nuova destinazione**. Quest'ultima è un'azione esplicita che non trasferisce né cancella copie precedenti. L'utente può continuare a catalogare offline anche senza backup riuscito.
4. Allo scadere successivo, la normale pianificazione recupera le esecuzioni mancate con le regole già previste; nessun accumulo di backup in raffica. Non presentare l'ultimo successo sull'unità vecchia come prova della nuova.
5. Prima di applicare qualsiasi retention, riusare i controlli conservativi esistenti e **non eliminare nulla su destinazione ambigua**. La logica di retention attuale è una garanzia da preservare, non da allentare.

### C. Backup manuale e migrazione

- Il backup manuale continua a consentire ogni cartella scelta con dialogo nativo; l'utente vede l'avviso sull'eventuale volume differente **prima di confermare**. Non imporre identificativi hardware o una sincronizzazione dell'account.
- Dopo cambio computer o ripristino, integrare concettualmente con TECH-058: una destinazione protetta configurata altrove risulta **da riconfermare**, non «danneggiata». Non ripristinare automaticamente la marcatura sull'altro dispositivo e non effettuare scansioni delle altre unità.

## 4. Ipotesi di dati e API (solo descrittive)

Un eventuale record separato dalle copie di backup, locale e minimizzato:

```text
DestinationBinding {
  chosen_path: string
  mode: "legacy" | "verify_before_scheduled_write"
  filesystem_hint?: { platform_kind, opaque_volume_ref?, reliability }
  folder_marker_id?: random_nonsecret_id
  confirmed_at?: local_datetime
  confirmed_on_this_installation?: boolean
}
CheckResult {
  status: "matched" | "different" | "missing" | "uncertain" | "unsupported"
  reason_code: string
  checked_at: local_datetime
}
```

API solo concettuali: `inspectBackupDestination(path) -> CheckResult`, `confirmBackupDestination(path, explicitConsent) -> DestinationBinding`, `checkDestinationBeforeAutomaticWrite(binding) -> CheckResult`. Lo storico `auto_backup_log` esistente rimane autorevole per sapere quali copie sono state realmente create; non usare una marcatura come sostituto del manifest, degli hash o dell'ispezione. Non si fissa in questa scheda lo schema SQLite, né un nome di comando Tauri.

**Vincolo di concorrenza (decisivo):** un check prima della scrittura soffre di *time-of-check/time-of-use*: un volume può essere rimosso o rimpiazzato dopo la verifica. La fattibilità va studiata per OS e filesystem (eventuali handle/identità legati alla directory usata per la scrittura, ricontrollo coerente prima della pubblicazione, fail-closed quando la garanzia manca). Un semplice file sentinella o `canonicalize` **non garantiscono** l'identità della destinazione né impediscono una sostituzione malevola. Non promettere «backup su volume corretto garantito» senza un modello di minaccia e test che lo dimostrino.

## 5. Offline, privacy, impostazioni e localizzazione

- Funzionamento interamente locale; nessuna rete aggiunta, chiamata remota, telemetria o trasmissione di inventario dispositivi. Un NAS già configurato può ovviamente richiedere connettività sulla rete locale: l'app non la abilita né effettua discovery automatica.
- Non memorizzare seriali hardware leggibili, nomi di rete, identificatori pubblicitari o dati personali non necessari. Se viene usato un identificativo OS opaco, esplicitare durata, affidabilità, possibilità di cambio e protezione rispetto al backup del DB. La marcatura facoltativa è visibile e revocabile, ma la sua rimozione richiede consenso e non tocca copie o file estranei.
- Nessun file dei clienti, foto, percentuali economiche o credenziali nel report di verifica. Eventuali messaggi di supporto dovrebbero essere codici minimizzati, non percorsi sensibili; mostrare il percorso completo **solo nell'interfaccia privata**.
- Lingua automatica del sistema con fallback **inglese**, override manuale esistente. Messaggi traducibili in italiano, inglese, spagnolo, francese e tedesco; testo esplicito «supporto non verificabile», non icone/colore soltanto. Controlli da tastiera e screen reader, focus sullo stato senza dialoghi bloccanti durante il lavoro normale.
- Nessuna modifica automatica alle destinazioni già salvate nelle installazioni esistenti. Per gli utenti preesistenti, modalità iniziale `legacy` con spiegazione e adesione esplicita; se viene accertato un rischio di perdita effettiva, il proprietario dovrà decidere separatamente se e come cambiare il default.

## 6. Compatibilità, dipendenze, alternative e rischi

**Dipendenze logiche:** scheduler nativo in `auto_backup.rs`, I/O e manifest in `backup.rs`, dialogo cartella Tauri, possibili API specifiche dei filesystem Windows/macOS/Linux. Nessuna dipendenza nuova scelta o introdotta. Valutare l'effettiva possibilità di identificare volumi USB con lettere riassegnate, APFS/mount point, bind mounts e SMB/NFS, inclusi volumi cifrati sbloccati in momenti differenti. Non assumere che UUID volume o metadati siano stabili in ogni ambiente.

**Alternative da provare prima di creare UI o API:** istruzioni per assegnare un punto di montaggio/lettera stabile, usare una cartella dedicata univoca e controllare manualmente l'ultimo backup; cifrare il supporto; eseguire backup manuali su unità rimovibili; prova con cartelle e supporti sintetici. Per un NAS non verificabile, chiedere conferma anziché produrre falsi positivi di sicurezza.

**Rischi:** falsa sicurezza se la marcatura viene copiata, supporti clonati o mount sostituiti fra controllo e scrittura; falsi allarmi dopo formattazione, migrazione o sostituzione lecita del disco; copie non prodotte se l'utente ignora avvisi; regressioni della retention; possibile raccolta superflua di fingerprint del dispositivo. Non dichiarare una protezione contro amministratori locali o sistemi compromessi.

**Complessità stimata:** media per una guida o avviso sulle destinazioni; **alta** per una verifica attendibile, multipiattaforma e robusta rispetto alle condizioni di concorrenza. Se impossibile rispettare la garanzia dichiarata su un OS o un filesystem, meglio **non implementare** il controllo per quel contesto piuttosto che simulare sicurezza.

## 7. Criteri di accettazione e verifiche proposte, NON eseguite

1. Su un archivio di prova, percorso valido e volume invariato: nessun cambiamento della forma del backup, hash e ripristino; backup programmato compiuto normalmente.
2. Cartella mancante o disco scollegato: fallimento comprensibile senza scrivere altrove; l'ultimo successo non viene aggiornato. Recupero deliberato al successivo tentativo, senza raffiche.
3. Stesso percorso, **supporto diverso ma cartella omonima**: la modalità protetta rileva la differenza o dichiara incertezza e non scrive; l'utente può scegliere il nuovo supporto esplicitamente.
4. Marcatura mancante/coperta/coperta da una copia della cartella: risultato non può essere pubblicizzato come prova forte; test negativo con marcatura copiata e identità volume diversa.
5. Dispositivo cambiato durante controllo, copia e pubblicazione: nessuna promessa di atomicità non dimostrata; definire comportamento di arresto/backup parziale e recupero.
6. NAS SMB/NFS, USB Windows, disco rimovibile macOS, mount Linux, directory con link simbolici: matrix con casi supportati, non supportati e risultati ambigui espliciti. Testare in modo particolare supporto non disponibile *prima* e *durante* la retention.
7. Nessuna eliminazione di file estranei o backup validi su un volume non confermato; nessun accesso fuori dalla cartella scelta. Qualsiasi modifica alle preferenze richiede un'azione consapevole.
8. Localizzazione di avvisi, lettura da screen reader, gestione da tastiera, messaggi di errore comprensibili e archivi ripristinati da altre macchine senza riconferma implicita.
9. Dati reali/feedback: 3–5 studi che già usano supporti rimovibili o NAS e almeno un esperimento riproducibile; senza questi dati, la proposta resta ipotetica.

## 8. Fasi suggerite, priorità e decisioni aperte

- **Fase 0 (nessuno sviluppo):** intervistare utenti e documentare 2–3 scenari di percorso/volume ricreati con dati fittizi; pubblicare eventualmente una guida per punti di montaggio stabili. Se l'attuale comportamento non presenta un rischio pratico, fermarsi qui.
- **Fase 1 (eventuale, dopo approvazione):** prototipo separato di sola verifica e report, senza schedulare backup, senza modificare record e senza scrivere marcature. Misurare affidabilità OS per OS.
- **Fase 2 (eventuale):** consenso per associare la destinazione a un'indicazione verificabile; blocco prudente dei soli backup automatici verso un supporto dubbio; nessuna modifica delle copie storiche.
- **Fase 3 (eventuale):** esperienza di riconferma e test end-to-end su Windows/macOS/Linux, NAS, backup interrompibili, migrazione fra computer e retention.

**Domande per il proprietario, senza presumerne la risposta:** quanti utenti usano backup automatici su USB/NAS? Preferire il blocco del backup in caso di dubbio, con rischio di restare senza nuova copia, oppure un avviso e prosecuzione? Quali sistemi/filesystem devono essere supportati prioritariamente? È accettabile una marcatura facoltativa nella cartella scelta? Una guida esterna risolve il problema più economicamente? Come presentare la differenza fra «supporto verificato» e «contenuto del backup verificato»?

**Decisione:** nessuna. Questa scheda non autorizza sviluppo, modifiche a codice/test/build/workflow/configurazione/dipendenze, nuove issue o merge. La priorità è una proposta da discutere dopo evidenze e verifiche tecniche.
