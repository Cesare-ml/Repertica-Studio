# REPS-TECH-007 — Archivio media esterno gestito, rilocalizzabile e recuperabile

- **Data ricognizione:** 2026-11-21
- **Stato:** **Proposta — da valutare**
- **Priorità suggerita:** **P2**, da motivare con esigenze reali di capacità del disco e frequenza di utilizzo di unità rimovibili.
- **Ambito:** scelta facoltativa della **cartella che custodisce gli originali** delle fotografie, distinta da destinazione di backup e cache delle anteprime; **solo analisi**, senza mandato di sviluppo.

## Motivazione, destinatari e fatti verificati

La [documentazione MVP](../../MVP.md), il [README](../../../README.md), [db.rs](../../../src-tauri/src/db.rs) e [gallery.rs](../../../src-tauri/src/gallery.rs) descrivono un archivio SQLite nella cartella privata dell'applicazione e fotografie originali in `media/`. La galleria locale (PR [#17](https://github.com/Cesare-ml/Repertica-Studio/pull/17), **già integrata in `main`**) consente, per ogni opera, una foto principale e fino a 20 aggiuntive, PNG/JPEG/WebP fino a 12 MiB per immagine. I percorsi persistiti nel DB sono relativi, ad esempio `media/<uuid>.jpg`, risolti oggi rispetto alla directory dati dell'app. Il [backup Rust](../../../src-tauri/src/backup.rs) produce una cartella autosufficiente contenente database, media originali e manifest SHA-256; un ripristino sostituisce l'archivio attivo dopo verifica.

Per un artista con fotografie numerose, uno studio con SSD di sistema piccolo o una galleria che conserva gli originali su un volume capiente, l'obbligo di ospitare **tutti** i media nella cartella privata del profilo può diventare scomodo. L'opportunità è **poter scegliere e cambiare in sicurezza il volume degli originali** senza rendere il database dipendente da link assoluti o corrompere backup e certificati. Non è stato verificato un volume reale di utenti che richieda questa funzione: la domanda resta da validare.

**Destinatari potenziali:** fotografo con migliaia di scatti catalogati, galleria con NAS già montato, archivista che migra a un nuovo computer, artista con disco interno limitato. La cartella esterna è **locale o montata dal sistema operativo**, non uno spazio cloud proprietario né un servizio remoto di Repertica.

### Confini con le altre idee

- [REPS-TECH-003](REPS-TECH-003-diagnostica-integrita-media.md): controlli di file mancanti/orfani e ripristino di singole immagini, **non** gestione del volume che ospita il media store.
- [REPS-TECH-005](REPS-TECH-005-anteprime-media-grandi-archivi.md): miniature **derivate ed eliminabili**, non scelta dello spazio degli **originali**.
- [REPS-TECH-001](REPS-TECH-001-pacchetto-dati-aperto.md): interscambio semantico e reimportazione; l'archivio originale deve restare trasportabile indipendentemente dai percorsi specifici del computer.
- [REPS-TECH-004](REPS-TECH-004-ricerca-federata-archivi-locali.md): ricerca **read-only** in fonti distinte, non apertura o gestione di un archivio media attivo.
- [REPS-TECH-006](REPS-TECH-006-backup-cifrati-portabili.md): protezione opzionale dei backup; **questa scheda non cifra** e non sostituisce backup/ripristino.
- [PR #14](https://github.com/Cesare-ml/Repertica-Studio/pull/14): processi gestionali, dossier e movimenti di opere fisiche; nessuna modifica a quel catalogo.

## Scenari professionali e flussi UX proposti

**1. Primo utilizzo, invariato.** L'app continua a creare media e SQLite nella cartella privata. Nessuna domanda obbligatoria, nessun account o servizio aggiuntivo.

**2. Scelta consapevole di uno spazio maggiore.** In Impostazioni → Archivio e spazio, ipotetico comando «Sposta le fotografie originali…». L'utente sceglie una cartella via dialogo nativo: disco interno, cartella esterna, USB o NAS *già montato*. La UI mostra numero immagini, byte da trasferire, spazio libero stimato, volume di destinazione, rischio di disconnessione, permessi e presenza di dati personali. Esclude destinazione identica alla sorgente, cartelle di backup, sottocartelle interne ai media esistenti o altri archivi non identificati. Non effettua discovery di cartelle o device.

**3. Anteprima non distruttiva.** Il software controlla accessibilità della destinazione, scrittura, tipo dei file e collisioni, produce un piano di migrazione; chiede conferma separata. L'utente sa che **i backup restano necessari e autonomi** e che la scelta di uno spazio su rete rimovibile comporta indisponibilità quando scollegato.

**4. Migrazione verificata e commutazione.** In un progetto futuro, un'operazione esplicita deve copiare (non spostare distruttivamente) originali in una struttura gestita, verificare *ogni* file rispetto a una misura acquisita alla fonte, validare i riferimenti SQLite, quindi attivare il nuovo volume **soltanto dopo verifica completa**. La sorgente non viene eliminata automaticamente: l'utente riceve un report e può scegliere di rimuoverla **manualmente**, dopo aver verificato almeno un backup recuperabile. Se un file cambia mentre si copia, la migrazione fallisce e il volume attivo resta quello iniziale.

**5. Volume scollegato o permessi revocati.** Le schede opere e i dati SQLite rimangono consultabili, ma le immagini non disponibili sono segnate chiaramente «Originale non raggiungibile». **Non** sostituire silenziosamente con una vecchia anteprima; nessuna nuova acquisizione di foto verso un'altra cartella in fallback automatico. Comandi che richiedono gli originali (backup completo, esportazione completa, emissione di un documento che li incorpora) devono essere sospesi o fallire con messaggio comprensibile, non dichiarare successo parziale.

**6. Riaggancio e cambi di computer.** L'utente seleziona nuovamente una cartella media dopo cambio lettera USB, percorso di montaggio macOS/Linux o migrazione di dispositivo. Prima di riprendere la scrittura si verifica che si tratti dello **stesso** archivio media: identificatore locale, inventario e impronte di verifica, non semplice coincidenza del nome della cartella. Se il volume non corrisponde, nessun collegamento automatico. Una reinstallazione da backup **autosufficiente** deve funzionare anche senza volume esterno, ripristinando nel media store interno per default.

**7. Ritorno alla cartella interna.** Simmetrico al trasferimento verso l'esterno: piano, spazio disponibile, conferma, copia, verifica e attivazione; non distruggere i file precedenti. Distinguere in UI «Cambia percorso di un volume già spostato» da «Trasferisci tutti i file».

## Dati e API ipotetici, senza implementazione

**Scelta iniziale preferibile da studiare:** un **solo media store gestito** per archivio attivo, con un'unica radice scelta dall'utente. I valori di `image_path` restano relativi alla struttura logica `media/<nome sicuro>` già presente; una configurazione locale alla singola installazione identifica la radice fisica. Valutare una identità persistente di store (`store_id`), `format_version`, impronte dei file, totale oggetti, stato di disponibilità e timestamp di ultima verifica. L'eventuale ID è un aiuto contro l'aggancio erroneo, **non** prova sufficiente di autenticità o di integrità dei file.

La configurazione del **percorso assoluto** del volume è *specifica del dispositivo*: non deve diventare un prerequisito nascosto di un backup portabile, di TECH-001 o dei CSV. La directory gestita va trattata come contenuto **autoritativo** dell'archivio, a differenza delle anteprime ricostruibili. Non trasformare `image_path` in URL arbitrari né consentire riferimento a percorsi assoluti scelti per singola opera in questa prima proposta.

Le API sono **esclusivamente concettuali**:
- `inspectMediaStore(source, destination) -> Plan|Warnings`
- `migrateMediaStore(plan, explicitApproval, cancelToken) -> Verified|Failed`
- `inspectStoreIdentity(folder) -> Match|Mismatch|Unknown`
- `relinkMediaStore(verifiedFolder, approval) -> Result`
- `inspectStoreAvailability() -> Available|Unavailable|Degraded`

Occorre definire un protocollo di commit crash-safe tra SQLite e filesystem, con journal/marker di migrazione e recovery deterministico al riavvio. **Non** si assume che una transazione SQLite renda atomiche copie di file su altri dischi.

## Offline, sicurezza, privacy, impostazioni, accessibilità e lingue

- **Offline-first pieno:** nessun account, abbonamento, telemetria, scansione LAN o sincronizzazione; i percorsi SMB/NAS funzionano solo se il sistema operativo li ha già montati. I dati restano su device e destinazioni scelti consapevolmente.
- **Sicurezza dei percorsi:** accettare soltanto directory regolari autorizzate, rifiutare symlink/reparse point pericolosi, path traversal, hard link non sicuri, file speciali e cartelle annidate in sorgente/destinazione. Rivalidare i confini dopo riavvio e prima di ogni scrittura, anche su filesystem case-insensitive e con normalizzazioni Unicode differenti.
- **Integrità e rischio operativo:** protezione da TOCTOU, permessi cambiati, disco pieno, supporto USB rimosso, NAS intermittente, file in modifica, collisioni e copie incomplete. Nessuna cancellazione automatica degli originali, nessun aggiornamento dei percorsi di backup precedenti, nessun ripristino «parziale».
- **Privacy:** la cartella esterna potrebbe essere leggibile da altri utenti del computer o della rete. Mostrare avviso su clienti/firme/metadati eventualmente presenti nelle foto, ma non promettere cifratura; rimandare a protezioni filesystem/OS. Nessuna esposizione degli indirizzi fisici nel protocollo MCP senza autorizzazione specifica.
- **Impostazioni:** indicare percorso attivo, stato «collegato/scollegato», spazio usato e conteggio, ultima verifica, «Trova cartella», «Trasferisci originali», «Torna a spazio interno» e report esportabile in cartella scelta. Non aggiungere monitoraggio di dispositivi in background a programma chiuso.
- **Lingue e accessibilità:** lingua OS con fallback inglese e scelta manuale; messaggi localizzati per disconnessione, rischio di eliminazione, progressi e rollback. Focus e pulsanti accessibili da tastiera, stato annunciato da screen reader, nessun feedback solo cromatico, cancellazione dell'operazione senza messaggi fuorvianti.

## Compatibilità, backup, import/export e mantenibilità

- **Vincolo più importante:** il backup esistente deve restare **autosufficiente**, includendo i byte degli originali nel proprio `media/` e la verifica SHA-256. Una copia contenente solo link o percorsi della cartella esterna non è un backup. Se la cartella è irraggiungibile o cambia durante la copia, **nessun manifest valido** deve essere pubblicato e l'archivio attivo non viene modificato.
- I backup attuali devono continuare a essere ripristinabili senza conoscere la configurazione di dischi esterni. L'installazione destinazione può usare la cartella interna e, solo dopo, decidere di spostare i media.
- La presenza del volume non cambia UUID delle opere, snapshot immutabili dei certificati, seriali, vendite, pagamenti o registri. La galleria e le immagini principali devono essere trattate nello stesso modo.
- `REPS-TECH-001` deve includere *i contenuti* degli allegati se previsti dal pacchetto, mai affidarsi alla permanenza di percorsi assoluti. I cinque CSV esistenti rimangono esportazioni tabellari, non copie complete delle fotografie.
- Quando il media store è esterno, una cache di TECH-005 va conservata e invalidata separatamente; la sua presenza non dimostra che l'originale sia raggiungibile.

## Dipendenze, alternative, rischi e complessità

**Dipendenze eventuali:** collaudo delle attuali funzioni backup/ripristino (anche su USB/NAS, [issue #7](https://github.com/Cesare-ml/Repertica-Studio/issues/7)); diagnosi media (TECH-003) utile ma non obbligatoria; disegno di una radice media per archivio, verifica dell'identità dei media, sincronizzazione delle operazioni Tauri, protocollo di migrazione riprendibile o annullabile, test cross-platform.

**Alternative da valutare:**
1. **Non cambiare nulla:** lasciare gli originali in app-local-data, raccomandare sistemi operativi con profili/dati su disco capiente. È la scelta con meno rischi e migliore portabilità, sensata per la maggioranza degli utenti.
2. **Spostare tutta la cartella dati tramite strumenti OS:** possibile su alcuni dispositivi, ma fuori dal controllo dell'app e potenzialmente fragile con migrazioni, permessi e backup.
3. **Supportare media esterni per singola opera:** massimo controllo, ma molti riferimenti fragili, backup più complessi e pericolo di link fuori scope; *non suggerita come fase iniziale*.
4. **Media store unico esterno opzionale (questa scheda):** compromesso ipotetico tra flessibilità e tracciabilità, con costo progettuale alto.

**Rischi principali:** indisponibilità dei file su unità rimovibili, corruzione o split-brain quando l'utente modifica due copie, fuga di dati su share comuni, rallentamento massivo NAS, falsi successi di backup, bug di recovery cross-device, confusione fra media originali e cartelle di backup. **Complessità stimata alta**; non promuovere a roadmap senza interviste e prove con archivi di dimensioni realistiche.

## Criteri di accettazione e prove da definire

1. Migrazione con almeno **10.000 fotografie sintetiche** e file Unicode su Windows/macOS/Linux: hash sorgente/destinazione uguali, nessun link rotto, SQLite/UUID/seriali invariati.
2. Simulare rimozione USB, NAS irraggiungibile, mancanza di spazio, permessi negati, errore di copia e crash **prima, durante e dopo** la commutazione: l'archivio rimane nello stato precedente oppure riprende in modo verificabile; non lascia percorsi autorevoli ambigui.
3. Un backup completo creato con media esterni include **tutti** gli originali, passa manifest/hash/integrità SQLite e si ripristina offline su computer pulito **senza** dispositivo sorgente.
4. Un backup tentato con media non disponibili deve fallire senza dichiarare copia completa; uno creato precedentemente non viene toccato.
5. Riagganciare una cartella diversa con stesso nome ma contenuto differente causa **rifiuto**, senza associarla a opere esistenti né scrivere file.
6. Eliminazione delle vecchie copie consentita solo come scelta separata e manuale, con avviso che tale cancellazione potrebbe rendere non recuperabili originali non salvati.
7. Schede, ricerca testuale e statistica rimangono disponibili se il volume è scollegato, con indicatori accessibili per foto mancanti; comandi che richiedono originali non fanno fallback silenzioso.
8. Nessun account, invio rete, credenziali o server obbligatorio; percorsi non divulgati in log/export non necessari; lingua sistema e fallback inglese collaudati.

## Certezze, ipotesi e decisioni per il proprietario

**Accertato in `main`, 2026-11-21:** app-local-data SQLite + `media/`, foto principali e galleria locale fino a 20 extra, percorso relativo, salvataggio file gestito dall'app, backup integrale con manifest SHA-256. Non è stata individuata nelle API correnti una scelta della cartella degli originali indipendente dalla cartella dati.

**Ipotesi da verificare:** esistono utenti con raccolte tanto grandi da giustificare gestione del volume separato; un solo media store esterno per archivio è sufficiente; la migrazione dovrebbe essere possibile senza duplicazione permanente dei file dopo verifica.

**Domande per revisione:** quanto spazio occupano realmente le librerie degli utenti? È preferibile lasciare tutto nella cartella del programma e migliorare solo le anteprime? Vale la pena assumere il rischio di supporti esterni rimovibili? Si deve permettere il media store su NAS condiviso, e con quali limiti? Come devono essere gestite copie manuali «divergenti» e media originali modificati fuori dall'app? È sufficiente un solo volume o è richiesta segmentazione per anno/progetto? In quale ordine rispetto al collaudo del backup esistente?

## Fasi ipotetiche (non autorizzate)

1. **Verifica del bisogno:** dimensioni di archivi reali, dispositivi e interviste; valutare alternativa di sola gestione delle anteprime.
2. **Prototipo solo descrittivo/testabile:** contratto di identità della cartella, percorso relativo e backup autosufficiente, analisi failure mode.
3. **Solo se approvato:** migrazione guidata verso un unico store esterno con fallback manuale esplicito e recovery crash-safe.
4. **Dopo collaudi indipendenti:** riaggancio, restituzione a store interno e supporto a volumi lenti/rimovibili. Nessuna estensione multi-volume implicita.

> **Nessuna funzionalità implementata o approvata.** Questa è una proposta di ricerca, distinta da qualsiasi piano operativo.
