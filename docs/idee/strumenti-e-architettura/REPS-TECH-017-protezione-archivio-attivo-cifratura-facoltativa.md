# REPS-TECH-017 — Protezione facoltativa dell'archivio attivo a riposo

- **ID stabile:** REPS-TECH-017
- **Data della ricognizione:** 2026-11-21
- **Stato:** **Proposta — da valutare**
- **Priorità suggerita:** **P2–P3, condizionata a una verifica del bisogno e delle alternative OS**. Non è un'approvazione né un incarico di sviluppo.
- **Ambito:** riservatezza del *database e dei file dell'archivio di lavoro a riposo*, non cifratura dei backup, sincronizzazione, account o controllo degli accessi tra operatori.

## Problema, destinatari e riscontri verificabili

Repertica custodisce dati anagrafici dei collezionisti, prezzi e ricevute, note, immagini di opere e copie storiche associate ai certificati. Un computer portatile smarrito, un disco dismesso senza bonifica o una cartella dati copiata da un altro utilizzatore del computer possono esporre dati che non dovrebbero essere divulgati. La cifratura del disco gestita dal sistema operativo può mitigare alcune di queste minacce, ma non tutti gli utenti sanno se sia attiva o come gestirne le chiavi.

**Riscontrato su `main` il 21 novembre 2026:** `src-tauri/src/lib.rs` apre all'avvio una sola directory `app_local_data_dir`, richiamando `db::open`; `src-tauri/Cargo.toml` usa `rusqlite` con `bundled` senza indicare un motore di cifratura; SQLite attivo e `media/` risiedono nella directory privata dell'app. Il database attuale usa WAL; le fotografie e gli allegati degli incassi sono file locali. I backup standard e i CSV restano **in chiaro**, come dichiarato in `README.md`, `docs/MVP.md` e `docs/DATA-PORTABILITY.md`. Il fatto che una cartella sia privata all'app non equivale di per sé a cifratura del contenuto. **Non è stato accertato alcun incidente di accesso abusivo o perdita di dati.**

**Potenziali destinatari:** artista che viaggia con il portatile; galleria con recapiti dei collezionisti; archivista che tratta opere e documenti sensibili; professionista che usa computer condivisi. Per uno studio su un disco già cifrato con protezioni OS adeguate questa funzione potrebbe risultare superflua: non renderla un prerequisito.

### Confronto per evitare duplicazioni

- **[TECH-006](REPS-TECH-006-backup-cifrati-portabili.md)** riguarda **copie di backup trasferibili** (USB/NAS) ed esclude esplicitamente la cifratura dell'archivio attivo. Qui si valuta la protezione di SQLite, media, allegati, file di lavoro e loro residui *mentre sono sul dispositivo*; le due protezioni non si implicano reciprocamente.
- **[TECH-007](REPS-TECH-007-archivio-media-esterno-rilocalizzabile.md)** riguarda *dove* stanno gli originali. Un volume esterno non è automaticamente protetto: scelta della posizione e scelta della cifratura sono indipendenti.
- **[TECH-009](REPS-TECH-009-catalogo-verifiche-conservazione-backup.md)** verifica e cataloga copie complete; **[TECH-012](REPS-TECH-012-migrazioni-archivio-preflight-copia-sicurezza.md)** propone checkpoint prima delle migrazioni. Nessuna delle due garantisce la riservatezza dei byte.
- **[TECH-001](REPS-TECH-001-pacchetto-dati-aperto.md)** tutela interoperabilità ed esportazione; la cifratura non deve trasformarsi in lock-in. **[TECH-014](REPS-TECH-014-bridge-mcp-locale-lettura-permessi.md)** definisce autorizzazioni verso assistenti esterni, non accesso fisico al disco.
- La [PR #14 — Gestione e UX](https://github.com/Cesare-ml/Repertica-Studio/pull/14) riguarda significato, consultazione e processi dei dati. Non si ridefiniscono anagrafiche, vendite, provenienza o documenti. Nessun file dell'altro filone va modificato.

## Scenari d'uso e flussi UX ipotetici

1. **Installazione ordinaria invariata:** Repertica parte subito, senza account, login, abbonamento o password Repertica. Nelle impostazioni una sezione facoltativa «Protezione dell'archivio» spiega la differenza tra permessi della cartella, cifratura del dispositivo e eventuale protezione propria dell'app. Prima scelta preferibile da studiare: **informazioni e controlli non invasivi sulla protezione del volume OS**, senza presumere di poterla attivare da Repertica.
2. **Valutazione informata:** un utente con dati riservati sceglie «Valuta archivio protetto». La schermata descrive cosa potrebbe essere protetto (SQLite, WAL, immagini, allegati) e cosa **non** lo sarebbe automaticamente (CSV già esportati, backup standard, copie preesistenti, file altrove, dati visibili a sessione sbloccata). Confronta alternativa «Usa cifratura del disco/volume» e variante «Protezione applicativa facoltativa», senza attribuire al software la capacità di verificare completamente la configurazione del sistema.
3. **Attivazione solo esplicita:** se in futuro fosse approvata una modalità cifrata nell'app, preflight non distruttivo: dimensione complessiva, spazio disponibile, integrità di SQLite e riferimenti media, compatibilità della build, percorso di recupero scelto dall'utente e verifica di un backup indipendente. Dopo conferma creare una **nuova copia protetta in staging controllato**, provarne apertura, hash e riferimenti, e soltanto allora commutare la directory attiva. Mai cifrare in-place senza strategia di recupero verificata.
4. **Sblocco locale all'avvio:** eventuale archivio protetto rimane in stato «Bloccato» fino a un segreto inserito **localmente** o sblocco OS *esplicitamente scelto*. In stato bloccato non vengono mostrate miniature, nomi di persone, anteprime di documenti o messaggi con dati personali. Se la chiave è errata, nessun tentativo di reinterpretare il file come SQLite ordinario o ricreare un archivio vuoto allo stesso percorso.
5. **Uso quotidiano:** una volta sbloccato, tutte le operazioni già disponibili continuano secondo i vincoli attuali. Eventuale blocco manuale richiede prima chiusura di transazioni, connessioni e viste sensibili; il solo oscuramento della finestra non è protezione crittografica. Blocco automatico per inattività: **opzione da studiare**, non assumere che possa rendere immediatamente illeggibili dati già in RAM o swap.
6. **Esportazione e backup:** i CSV e i PDF continuano a poter essere creati in una cartella scelta; **prima di salvare una copia in chiaro** l'utente è avvisato. I backup standard rimangono compatibili, ma una modalità «archivio protetto» non deve generare copie non cifrate *silenziose*, inclusi `VACUUM INTO`, staging e copie `before-restore`. L'integrazione con un eventuale backup cifrato di TECH-006 richiederebbe una specifica distinta, non è presunta.
7. **Cambio dispositivo, segreto o disattivazione:** bisogna poter trasferire dati completi a un'altra installazione offline senza account né keychain del computer originale, usando un segreto/chiave di recupero portabile; rotazione e rimozione della protezione richiedono un piano verificato, copia di sicurezza e conferma esplicita. Non eliminare mai automaticamente l'unico archivio recuperabile.
8. **Guasti:** disco pieno, directory esterna disconnessa, processo terminato e file incompatibile devono lasciare un archivio riconoscibile, recuperabile dalla copia precedente, senza rollback a una versione meno sicura nascosto all'utente.

**Principio UX:** una prima fase potrebbe limitarsi a **guida alla sicurezza del dispositivo + inventario locale delle superfici di esposizione**, senza sviluppare cifratura in-app. È un'alternativa a minor rischio, non una promessa di controllo dello stato OS.

## Requisiti certi, ipotesi e struttura dati/API puramente descrittive

**Requisiti già vincolanti:** app Tauri 2, Rust/SQLite e media locali, offline-first, nessuna autenticazione online, dati sempre esportabili, cartelle di backup liberamente scelte, certificati storici immutabili, transazioni economiche e allegati non da riscrivere in modo distruttivo. La protezione del dispositivo non può richiedere servizi di autenticazione esterni.

**Ipotesi NON approvate:** dopo analisi crittografica e di licenze si potrebbe valutare un backend SQLite con cifratura a livello di database, **insieme** alla cifratura autenticata degli originali binari. Proteggere solo `repertica.sqlite` lasciando `media/` e allegati in chiaro sarebbe una copertura **parziale da dichiarare**, non «archivio cifrato». L'eventuale scelta fra crittografia di volume OS, contenitore cifrato gestito e codec SQLite più file cifrati è ancora aperta.

```text
ArchiveProtectionDescriptor {       # metadati minimi, senza dati dell'utente
  format_version: integer,
  mode: "os_volume_guidance" | "app_managed_optional",
  state: "plain" | "locked" | "unlocked" | "migrating" | "recovery_required",
  archive_id: opaque?,               # non prova di proprietà o identità
  key_derivation_version: integer?,   # dettagli di interoperabilità pubblici
  wrapped_key_metadata: object?,     # MAI il segreto o la chiave in chiaro
  last_verified_at: timestamp?
}
ProtectionPlan { sources, destinations, estimated_bytes, findings, rollback_path, explicit_consent }
```

**API concettuali, non implementate né stabilite:** `inspectArchiveProtection() -> Status`; `planProtectionChange(mode) -> Plan|Blockers`; `commitProtectionChange(plan, userConsent) -> Result`; `unlockLocalArchive(secret) -> Unlocked|Denied|Unsupported`; `verifyProtectedArchive() -> Report`; `planKeyRotation() -> Plan`; `exportFullArchiveUnlocked(destination, format) -> Receipt`. In ogni fase serve una distinzione esplicita tra **ispezione senza scrittura** e azione autorizzata. Segreti mai nelle strutture serializzate, log, crash report, IPC più ampio del necessario o strumenti MCP.

**Proprietà tecniche da studiare:** migrazioni di schema solo **dopo** sblocco; una chiave per archivio indipendente, formato/versione recuperabili senza server; WAL/journal, copie temporanee, cache delle immagini, allegati incasso, file storici dei certificati, snapshot di backup e directory di rollback trattati come possibili fonti di dati in chiaro. La transizione multi-file non è resa atomica dalla sola transazione SQLite: serve protocollo di staging, verifica, attivazione e recupero dopo crash. Non elaborare protocolli crittografici artigianali.

## Offline, sicurezza, privacy, impostazioni e multilingua

- **Offline e indipendenza:** nessun servizio di cifratura remoto, recovery via email, account, telemetria o obbligo di cloud. Anche senza funzione opzionale si deve poter installare, lavorare ed esportare integralmente l'archivio.
- **Minacce coperte e limiti:** protezione a riposo contro lettura di supporti sottratti, non contro malware attivo, utente già autorizzato, screenshot, RAM, pagine di swap non protette, dispositivo compromesso, cancellazione/ransomware, errori di condivisione o file esportati in chiaro. Non promettere anonimato, prova di integrità legale, conformità normativa automatica o accesso multiutente sicuro.
- **Gestione delle chiavi:** segreto portabile scelto localmente e/o integrazione volontaria con il portachiavi OS, con percorso di recupero testato e documentato. Perdita dell'unica chiave può comportare perdita **definitiva** dei dati; nessuna password di default né recupero nascosto. Il keychain da solo non garantisce ripristino su una seconda macchina. Non lasciare segreti in manifest, percorsi, errori o report di assistenza.
- **Impostazioni ipotetiche:** stato protezione dell'archivio, indicazione dei limiti, «Valuta protezione», «Verifica recuperabilità», «Modifica modalità/ruota segreto» solo se supportato; tutte le operazioni con progressi e annullamento sicuro, niente attivazione implicita.
- **Lingue:** lingua di sistema; fallback inglese, override locale; italiano/inglese/spagnolo/francese/tedesco ove disponibili. Tradurre con attenzione differenze fra *password dimenticata*, *archivio danneggiato*, *versione non supportata*, *unità indisponibile* e *chiave errata* senza affermare una diagnosi impossibile. Nessuna traduzione dei token/ID persistiti. Messaggi in linguaggio semplice e avvertimenti di recupero accessibili a screen reader, tastiera, alto contrasto e focus prevedibile.
- **Portabilità e import/export:** nessun vincolo al medesimo OS, profilo o installazione; esportazione completa verificabile su cartella scelta, con avviso visibile quando il formato è non cifrato. Gli **otto CSV** correnti restano registri di analisi, non sostituti del backup e non cambiano semantica. I formati e la procedura di decifratura, se esistenti, dovrebbero poter essere documentati per strumenti indipendenti e recupero a lungo termine.

## Dipendenze, alternative, complessità e rischi

**Prerequisiti di valutazione:** collaudo reale del backup/restauro attuale su Windows/macOS/Linux, USB/NAS ([issue #7](https://github.com/Cesare-ml/Repertica-Studio/issues/7)); threat model per supporti rubati/utenti locali; inventario di tutti i percorsi dove possono restare copie; confronto e audit di biblioteche, formati, licenze open source e loro compatibilità con CPAL-1.0. Nessuna nuova dipendenza viene qui richiesta o introdotta. Il backend `rusqlite` attuale non deve essere trattato come un codec cifrato.

**Alternative da confrontare prima di intervenire:**
1. **Solo cifratura integrale di dispositivo/volume** già disponibile nei sistemi operativi (es. BitLocker/FileVault/LUKS), con guida dedicata: soluzione normalmente più semplice e con minore manutenzione applicativa, ma gestione dei volumi e del profilo esterna a Repertica.
2. **Cartella o contenitore cifrato gestito dall'utente**, senza cambiare database: più interoperabile con strumenti esterni, ma rischio di copie/spostamenti in chiaro e di percorsi temporanei non protetti.
3. **Cifratura applicativa opzionale integrale** (SQLite+media+derivati): maggiore controllo sul blocco per archivio, ma complessità elevata di chiavi, recupero e compatibilità.
4. **Solo cifratura di backup [TECH-006](REPS-TECH-006-backup-cifrati-portabili.md):** utile per USB/NAS e condivisioni di copie, **non** impedisce lettura del database attivo.

**Rischi principali:** falsa sensazione di sicurezza cifrando solo SQLite; perdita della chiave; migrazione interrotta che rende il solo archivio illeggibile; backup o snapshot `VACUUM INTO` prodotti in chiaro; residui e cache, swap e crash dump; aggiornamenti non retrocompatibili; performance per gallerie numerose; lock-in da formato/codec non più disponibile; incompatibilità di licenza; NAS/USB che scompaiono. Non distribuire una soluzione con cifratura «fatta in casa»; richiedere revisione indipendente e strategia di recupero su sistemi diversi.

**Complessità indicativa:** **alta/molto alta** per cifratura applicativa coerente di DB, WAL, media, allegati, file derivati e backup; **bassa-media** per informazione e guida OS, da verificare sui sistemi reali. **Priorità P2–P3 solo ipotetica**: una soluzione OS potrebbe risolvere il problema meglio, e per il MVP non è giustificato rischiare i dati dell'archivio senza bisogno documentato.

### Test e criteri di accettazione proposti (solo per eventuale decisione futura)

1. **Nessuna regressione per l'utente standard:** installazione, uso, backup e restore in chiaro continuano a funzionare senza password o account.
2. Un archivio protetto non rivela nomi, testi, fotografie leggibili o estratti personali nei file **dichiarati protetti** a riposo; audit separato di SQLite, WAL/journal, `media/`, ricevute, miniature, cache, staging e rollback. Ogni residuo non coperto è documentato senza nasconderlo.
3. Riapertura su Windows/macOS/Linux, su secondo computer offline senza accesso al portachiavi originario, con chiave di recupero; dati, UUID, media, documenti e certificati storici restano identici.
4. Password/chiave errata, formato più nuovo, file mancante e database danneggiato producono stati distinti e nessuna creazione o sovrascrittura di DB vuoto.
5. Interruzione in ciascuna fase (copia, cifratura, verifica, commutazione), disco pieno, NAS scollegato: l'archivio originario rimane recuperabile; nessuna migrazione dichiarata riuscita senza verifica completa.
6. Modifica di una foto, ricevuta, titolo o collezionista aggiorna correttamente dati protetti senza lasciare duplicati permanenti non cifrati; galleria, certificati, ricavi, incassi e vincoli SQL non cambiano significato.
7. Esportazione CSV/PDF/backup standard da archivio protetto mostra **prima** un avviso sulla copia in chiaro e salva solo nella cartella esplicitamente scelta. Nessuna copia `before-restore` pubblicata in chiaro senza piano di protezione e consenso.
8. Importazione/ripristino di vecchi archivi non cifrati e aggiornamenti di schema gestiti solo con preflight; una build precedente rifiuta in modo sicuro un formato protetto non riconosciuto.
9. Nessun segreto nei log, negli errori di diagnostica, nei report o nell'interfaccia MCP; test di accessibilità e localizzazione, inclusa lingua di sistema non supportata con fallback inglese.
10. Benchmark con archivi sintetici piccoli e molto grandi; valutazione di latenza allo sblocco, memoria, spazio addizionale, tempi di backup e recupero. **Nessun obiettivo numerico** prima di misurazioni reali.

## Fasi ipotetiche e domande per il proprietario

- **Fase 0 — Ricerca e minaccia reale:** chiedere se gli utenti già usano dischi cifrati, se condividono computer e quali dati vogliono proteggere; distinguere protezione contro furto del dispositivo da protezione rispetto ad altri utenti del medesimo account OS.
- **Fase 1 — Confronto senza codice:** valutare guida OS vs contenitori cifrati vs archivi gestiti; audit della superficie di plaintext e delle licenze, mockup UX e prova teorica di recupero offline.
- **Fase 2 — Decisione del proprietario:** se il valore non supera i rischi, **non sviluppare** cifratura integrata e limitarsi a raccomandazioni documentate; se approvato, specifica indipendente di interoperabilità, chiavi e ripristino.
- **Fase 3 — Solo dopo esplicita approvazione:** eventuale prototipo su dati sintetici, revisione crittografica esterna, test crash/migrazioni e collaudo nativo; nessuna release prima delle prove di recupero.

**Domande aperte:** qual è la minaccia prioritaria (furto laptop, USB condivisa, accesso di altro utente locale)? Sono accettabili password locali senza recupero via server? Deve essere possibile sbloccare lo stesso archivio su sistemi diversi senza assistenza? Proteggere le copie `before-restore` come requisito inderogabile? Per gli utenti ordinari basta una guida alle funzioni di cifratura OS? L'eventuale archivio cifrato deve interoperare con strumenti open source indipendenti da Repertica? Quali costi di manutenzione e collaudo sarebbero sostenibili da un progetto gratuito?

**Riferimenti informativi**, non scelte tecnologiche: [SQLite Encryption Extension (documentazione SQLite)](https://www.sqlite.org/com/see.html); [Microsoft — BitLocker](https://support.microsoft.com/en-us/windows/security/encryption/bitlocker-overview). L'eventuale adozione di una libreria/codec richiede valutazione indipendente di disponibilità del codice, licenza, compatibilità e manutenzione.

**Nessuna proposta è approvata o assegnata allo sviluppo. Nessun codice, test, workflow, dipendenza o configurazione va modificato per questa ricognizione.**
