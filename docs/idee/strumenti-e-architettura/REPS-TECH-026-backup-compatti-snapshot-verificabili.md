# REPS-TECH-026 — Backup compatti locali con snapshot verificabili e ripristino completo

- **ID stabile:** REPS-TECH-026
- **Data ricognizione:** 2026-11-21
- **Stato:** **Proposta — da valutare**
- **Filone:** strumenti e architettura; solo ricerca, nessuna implementazione.
- **Priorità proposta:** **P3** finché non emergano archivi reali sufficientemente grandi; **P2 condizionata** a misure che mostrino un beneficio concreto e a collaudi del backup completo attuale. Non approvata né assegnata.
- **Complessità stimata:** **molto alta** per un formato compatto portabile e recuperabile; non è una correzione urgente del backup esistente.

## Motivazione, destinatari e riscontri

Repertica Studio è un'app gratuita, open source, offline-first (Tauri 2, Angular, Rust, SQLite), senza account o abbonamenti. Chi conserva fotografie, documenti e RAW può accumulare molti gigabyte. La copia automatica quotidiana/settimanale, se è sempre integrale, può consumare spazio e tempo anche quando cambia solo una piccola parte dell'archivio.

**Riscontri verificati in `main` il 21 novembre 2026:**

- [`backup.rs`](../../../src-tauri/src/backup.rs): `create_backup_core` genera una nuova sottocartella `.partial`, uno snapshot SQLite con `VACUUM INTO`, copia **tutti** i file di `media/`, calcola SHA-256 di ciascun file, scrive il manifest `format_version: 1`, verifica il backup e solo dopo pubblica la directory; `restore_backup_core` verifica una **singola copia completa**, prepara lo staging e conserva l'archivio precedente.
- [`auto_backup.rs`](../../../src-tauri/src/auto_backup.rs): esiste già pianificazione facoltativa, disattivata di default, e conservazione delle ultime N copie automatiche create da questa funzione. Si usa il medesimo backup completo. La retention esistente non è una deduplicazione.
- [`MVP.md`](../../MVP.md) e [README](../../../README.md): backup e ripristino su cartelle liberamente scelte, anche USB/NAS montati, sono già disponibili; manca collaudo completo su dispositivi e sistemi reali. Sono già previsti allegati RAW fino a 1 GiB per opera e fotografie storiche dei certificati. I backup v1 sono in chiaro.
- [`DATA-PORTABILITY.md`](../../DATA-PORTABILITY.md): i 12 CSV non trasferiscono i byte dei media e **non** sostituiscono un backup completo.
- Il manifest v1 non possiede una identità stabile dell'archivio d'origine. Deducerla dal nome cartella o da opere comuni sarebbe pericoloso.

**Bisogno ipotizzato, non misurato:** un archivio con migliaia di foto e RAW sostanzialmente invariati potrebbe creare molte copie quasi identiche. Occorrono dimensioni, frequenza di modifica, tempi effettivi e filesystem reali prima di assegnare priorità.

**Destinatari:** studi d'artista con archivio multimediale in crescita, gallerie e archivi storici che fanno copie regolari su dischi esterni o NAS. Un archivio piccolo non deve pagare complessità aggiuntiva.

## Confini e coordinamento anti-doppione

- [TECH-009](REPS-TECH-009-catalogo-verifiche-conservazione-backup.md) inventaria, verifica nel tempo e aiuta a valutare la conservazione delle **copie già create**; cita esplicitamente i backup incrementali come possibile **progetto separato**, escluso dal proprio ambito. Qui si studia *come non riscrivere ogni volta contenuti invariati*, non la lista delle copie o una politica di cancellazione.
- [TECH-006](REPS-TECH-006-backup-cifrati-portabili.md) riguarda la cifratura facoltativa; l'eventuale deduplicazione **non** deve aggirare privacy o confini fra archivi/chiavi. Nessuna convergent encryption proposta.
- [TECH-022](REPS-TECH-022-preflight-spazio-operazioni-locali.md) valuta lo spazio **prima** di un'operazione; non riduce i byte dei backup. Eventuali stime vanno concordate.
- [TECH-001](REPS-TECH-001-pacchetto-dati-aperto.md) definisce un **interscambio semantico** di dati, distinto dagli snapshot di sicurezza; [TECH-007](REPS-TECH-007-archivio-media-esterno-rilocalizzabile.md) studia la posizione degli originali, non la ridondanza nelle copie.
- Il filone [Gestione e UX — PR #14](https://github.com/Cesare-ml/Repertica-Studio/pull/14) propone processi d'archivio e amministrazione, non un motore di deduplicazione. **Nessun suo file viene toccato.**

Non è una proposta di sincronizzazione multiutente, cloud, archivio remoto, fusione di database, backup parziali presentati come completi o eliminazione automatica di copie.

## Specifica funzionale ipotetica e flussi UX

**Principio di sicurezza:** il backup completo v1 esistente rimane **predefinito e sempre disponibile**. Un eventuale modo «risparmia spazio» è opt-in, separato, e non viene mai descritto come una cartella autonoma se ha dipendenze. Non sostituisce l'esportazione di una copia completa e indipendente, ripristinabile su un secondo computer senza archivi ausiliari.

1. **Valutazione senza scrivere:** Impostazioni → Backup → «Verifica possibile risparmio». Dopo scelta di una cartella e analisi locale, mostra spazio *logico*, occupazione *fisica stimata*, quantità di file cambiati e limiti. Se manca un formato sicuro per quel volume, raccomandare il backup completo; **nessun upgrade implicito** delle copie esistenti.
2. **Attivazione consapevole:** opzione facoltativa «Copie compatte (sperimentali)», con testo che indica chiaramente che uno snapshot può dipendere da oggetti condivisi nella stessa raccolta. Se la destinazione contiene backup v1 o file estranei, non modificarli e non adottarli automaticamente.
3. **Creazione:** prendere sempre uno **snapshot SQLite coerente** e verificare tutti i file media/autorevoli, inclusi quelli senza riferimenti correnti e le foto storiche dei certificati. Riutilizzare soltanto byte la cui impronta SHA-256 è stata verificata, non soltanto grandezza e `mtime`. Pubblicare un nuovo snapshot logico solo dopo completamento e verifica.
4. **Elenco delle versioni:** data, dimensione apparente, spazio realmente aggiunto quando conoscibile, disponibilità del deposito e ultima verifica completa. Nessun «backup sano» se manca un oggetto, se il supporto è scollegato o se la verifica è soltanto parziale.
5. **Ripristino di qualunque punto:** selezionare uno snapshot, ispezionare **tutte** le sue dipendenze, materializzare in staging una cartella completa e validarla tramite i controlli di backup/SQLite/media correnti, poi usare il ripristino protetto già previsto. L'archivio attivo e la copia precedente restano intatti in caso di errore. Non ripristinare direttamente file isolati dal deposito nell'archivio attivo.
6. **«Esporta copia indipendente»:** dalla raccolta compatta creare su una cartella libera un **normale backup completo** autosufficiente, verificato, trasportabile e leggibile senza il deposito; disponibile anche per utenti che vogliono uscire per sempre dalla modalità compatta. Il formato v1 resta il percorso compatibile quando lo schema lo consente.
7. **Rimozione / recupero spazio:** *non* prevista in fase iniziale. La rimozione di un indice/snapshot non implica che i byte condivisi siano liberabili. Un eventuale garbage collector futuro deve calcolare l'insieme delle dipendenze di **tutti** gli snapshot validi, ricontrollare e ottenere conferma distinta; nessuna cancellazione per liberare automaticamente spazio su supporto pieno.
8. **Destinazione rimossa, interruzione o power loss:** progressi annullabili, nessuno snapshot «completato» pubblicato se incompleto; alla riapertura rilevare staging incompleto e offrirne un resoconto, **senza eliminare oggetti potenzialmente utili** in modo silenzioso.

Interfaccia accessibile con tastiera/screen reader; barra di progresso distinguendo «verifica», «copia», «pubblicazione» e «ripristino»; nessun messaggio basato solo sul colore.

## Modelli dati e API *solo descrittivi*

Schema alternativo da valutare, **non autorizzato**: raccolta locale versionata con manifest degli snapshot che referenziano oggetti immutabili identificati da hash del contenuto; nessuno snapshot fa riferimento alla presenza di **un altro snapshot** (quindi niente catene base→delta), ma tutti quelli della raccolta condividono l'**object store**. L'object store resta una dipendenza: la cartella dello snapshot da sola **non** è un backup autosufficiente.

| Concetto | Campi/semantica ipotetici |
| --- | --- |
| `CompactStore` | `format_version`, `store_id`, radice del volume scelta, stato di compatibilità; identità locale verificabile, non autorizzazione a leggere altre cartelle |
| `ArchiveNamespace` | identificativo stabile dell'archivio se disponibile in un futuro schema; per backup v1 **`Unknown`**, senza associazione automatica |
| `SnapshotDescriptor` | `snapshot_id`, creazione, `schema_version`, elenco completo di `logical_path`, `bytes`, `sha256`, `object_key`, firma/versione del formato se applicabile |
| `ContentObject` | byte verificati con SHA-256, dimensione, nome fisico sicuro derivato dal digest; scrittura temporanea, `fsync`/rename o strategia equivalente con semantica verificata sul volume |
| `VerificationResult` | `CompleteVerifiedAt` / `Incomplete` / `Unavailable` / `Incompatible`, elenco errori non sensibili, timestamp; un hash non equivale a firma di autenticità |
| `MaterializationPlan` | snapshot selezionato, oggetti richiesti, destinazione, occupazione stimata e spazio per staging/ripristino non distruttivo |

Contratti ipotetici: `inspectCompactStore(folder) -> Report`; `previewCompactBackup(archive, folder) -> Estimate`; `createVerifiedSnapshot(...) -> SnapshotId`; `verifySnapshot(snapshotId) -> VerificationResult`; `materializeFullBackup(snapshotId, chosenFolder) -> VerifiedStandaloneBackup`; `restoreVerifiedSnapshot(...) -> RestoreSummary`. I nomi **non** rappresentano API pianificate.

Le strutture devono conservare byte originali, snapshot SQLite, UUID, vendite/incassi, allegati, fotografie e tracce storiche. Una futura esportazione aperta richiede **specifica documentata** del manifest, ordinamento canonico, versione, codifica e strategia per verificare/esportare senza software proprietario. Un file JSON non è automaticamente un formato recuperabile.

## Alternative e scelta architetturale da decidere

1. **Mantenere solo backup integrali v1 (baseline):** massima semplicità, affidabilità e indipendenza. Accompagnare da buone istruzioni e collaudi reali.
2. **Snapshot completi con copie reflink/CoW quando supportate:** una directory autonoma per snapshot e potenziale risparmio, ma capacità e semantiche variabili su Windows/macOS/Linux, exFAT e NAS; **fallback a copia completa**, senza supporre che `copy` sia CoW. Misurare anche l'occupazione reale.
3. **Deposito a oggetti condivisi e snapshot logici completi:** massimo riuso dei RAW identici, indipendenza tra manifest degli snapshot, ma dipendenza dal deposito, nuove regole di verifica, export di copia autosufficiente e complessa gestione dei riferimenti. Solo modello di studio, non scelta approvata.
4. **Hard link fra backup:** appaiono efficienti ma modificare un file in una copia può alterare silenziosamente tutte quelle che condividono l'inode. **Non preferiti**; richiederebbero barriere contro alterazioni esterne che non si possono garantire in modo generale.
5. **Catene differenziali SQLite o patch binarie:** massima complessità e dipendenza da basi precedenti; **escluse dalla prima valutazione**. Snapshot SQLite completo a ogni punto resta l'ipotesi conservativa.

Anche con deduplicazione, mantenere **copie su dispositivi fisicamente separati**: risparmiare spazio su un unico disco non protegge da perdita/furto del disco, ransomware o incendio.

## Offline, sicurezza, privacy, impostazioni e lingue

- Nessun account, abbonamento, telemetria, cloud o endpoint obbligatorio. NAS solo come cartella già montata dall'OS; non creare credenziali o scansioni di rete.
- Un deposito contiene SQLite, media, dati economici e personali **in chiaro**, come il backup attuale; mostrarlo. I nomi fisici basati su hash non rendono privati i contenuti. Cifratura futura = scelta separata da [TECH-006](REPS-TECH-006-backup-cifrati-portabili.md), con approfondimento sui possibili leak da deduplicazione tra utenti/archivi e chiavi.
- Percorsi: rifiutare symlink/junction/reparse point non sicuri, path traversal, file speciali, percorsi annidati sorgente/destinazione, manomissione del manifest e file mutati tra scansione e utilizzo (TOCTOU). Non assumere atomicità o `fsync` equivalenti su NAS.
- Non combinare automaticamente archivi differenti soltanto perché hanno file SHA-256 uguali; proteggere identità e autorizzazioni. Una copia del deposito fatta a mano mentre è in scrittura non costituisce automaticamente un backup valido.
- **Impostazioni:** modalità `Completa (predefinita)` / `Compatta (solo dopo studio e opt-in)`, percorso scelto liberamente, riepilogo delle dipendenze, verifica e «Esporta copia indipendente». Nessun controllo occulto su altri volumi.
- **Lingue:** lingua di sistema, fallback inglese; selezione manuale IT/EN/ES/FR/DE dove disponibile. Numeri, date e unità localizzati; identificativi/formati tecnici invarianti. Terminologia comprensibile: distinguere «snapshot logico», «copia indipendente» e «spazio condiviso».
- **Compatibilità:** Windows/macOS/Linux, filesystem locali, USB, NAS montati; le funzioni non supportate provocano fallback sicuro, non falso successo. Vecchi backup v1 leggibili/ripristinabili senza nuova raccolta; i 12 registri CSV e il backup integrale non subiscono alterazioni.

## Rischi, dipendenze e criteri di accettazione

**Dipendenze preliminari:** completare prima QA effettiva backup/ripristino su Windows/macOS/Linux e USB/NAS (cfr. [issue #7](https://github.com/Cesare-ml/Repertica-Studio/issues/7)), dati su archivi grandi, stabilità del formato v1, valutazione di limiti del volume, separazione di versioning e retention ([TECH-009](REPS-TECH-009-catalogo-verifiche-conservazione-backup.md)). Nessuna libreria o dipendenza selezionata in questa scheda.

**Rischi principali:** perdita di più versioni per singolo oggetto corrotto o mancante, garbage collection errata, collisione/misassociazione di archivi, performance dovute alla ri-verifica dei byte, storage pieno durante materializzazione, differenze di inode/reflink/CoW, permessi e ACL, NAS instabile, copie manuali incomplete, crescita nascosta della cache, ransomware e illusione che «compatto» significhi «più copie indipendenti». Un deposito condiviso aumenta la **correlazione dei guasti**: non dichiarare equivalenza con N copie complete su N supporti.

**Criteri/test ipotetici, non eseguiti:**

1. Archivio con 10.000 fotografie e diversi RAW sintetici, snapshot T1; varia solo un metadato e un file a T2. Entrambi i punti possono essere **materializzati e ripristinati integralmente**; misurare byte realmente scritti e occupati vs due copie complete, non inventare percentuali di risparmio.
2. Modificare, rimuovere o corrompere un oggetto condiviso: ogni snapshot dipendente deve fallire verifica, senza esito verde né toccare l'archivio attivo. La perdita di un oggetto condiviso deve essere esplicitata come rischio più ampio.
3. Rimuovere un manifest di snapshot (se in futuro consentito) non deve cancellare oggetti ancora utilizzati da altri snapshot. Garbage collection, se mai autorizzata, provata contro interruzioni, copie non raggiungibili e cataloghi incompleti.
4. Materializzare una copia v1 autosufficiente su disco nuovo e ripristinarla **senza** il deposito compatto, senza account, server e rete. Confrontare SHA-256, UUID, foto storiche dei certificati, allegati ricevute e media non referenziati.
5. Crash o rimozione USB/NAS in ogni fase di scrittura, `fsync`, pubblicazione, verifica, export e restore: nessuna pubblicazione incompleta, nessuna cancellazione di backup v1 precedenti; recupero spiegato.
6. Due archivi differenti nella stessa cartella padre e un backup v1 senza identificativo: nessuna associazione per nome o somiglianza; nessuna deduplica/retention tra archivi senza perimetro verificato.
7. File con stesso `mtime` e dimensione ma contenuti diversi: non devono essere considerati identici; prova di hash sul contenuto. Percorsi Unicode, file speciali, symlink e permessi negati gestiti in sicurezza.
8. Su Windows/macOS/Linux, filesystem con e senza CoW, USB exFAT e NAS: fallback al backup completo sempre funzionante. Valutare prestazioni, CPU/RAM, latenza, occupazione e prove screen reader/tastiera/EN fallback.
9. Backup automatico e operazioni concorrenti non devono produrre due writer sul deposito né esporre snapshot validi prima della verifica. Il monitoraggio resta limitato all'app aperta, come oggi.

## Fasi *eventuali* (non approvate)

- **Fase 0 — Convalida del bisogno:** misurare dimensione, tasso di modifica, frequenza e costo dei backup completi su casi sintetici e volontari, senza raccogliere file personali. Se il risparmio non giustifica il rischio, **chiudere l'idea senza sviluppo**.
- **Fase 1 — Analisi comparativa:** specificare su carta i failure mode di CoW e deposito content-addressed, portabilità aperta e materializzazione di una copia completa; evitare prototipi sui dati reali.
- **Fase 2 — Solo previo consenso del proprietario:** sperimentazione isolata con archivi sintetici; nessuna modifica alla modalità v1, nessuna cancellazione/retention automatica.
- **Fase 3 — Solo dopo collaudi e decisione esplicita:** eventuale modalità compatta opt-in, verifica periodica, export indipendente e migrazione di ritorno; governance dello spazio condiviso da approvare separatamente.

## Domande aperte per il proprietario

1. Quali sono **dimensioni e numero** tipici degli archivi? Quante copie si vogliono mantenere e con quale frequenza? Le foto/RAW cambiano dopo l'importazione?
2. È accettabile una **raccolta compatta con dipendenze interne**, purché si possa sempre creare una copia integrale indipendente, oppure ogni backup deve rimanere autosufficiente?
3. Se il filesystem non offre CoW, è meglio eseguire sempre un backup completo o adottare un deposito condiviso più complesso?
4. Conviene affrontare prima la verifica periodica e il catalogo di [TECH-009](REPS-TECH-009-catalogo-verifiche-conservazione-backup.md), mantenendo i backup v1?
5. Quanto è importante il risparmio di spazio rispetto a rischio di perdita correlata, durata del backup e semplicità di recupero su un altro computer?
6. Sarebbe utile una funzione di **copia completa indipendente su un secondo supporto** come obiettivo principale, invece di deduplicazione/retention?
7. Si vuole escludere per principio qualsiasi cancellazione automatica in un deposito condiviso finché non vi sia una governance espressamente approvata?

---

**Esito:** proposta esplorativa soltanto documentale. Non configura né attiva backup incrementali, non approva un formato, non modifica codice, test, workflow o impostazioni, non autorizza sviluppo, rimozioni o merge.
