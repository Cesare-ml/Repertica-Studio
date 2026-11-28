# REPS-TECH-022 — Preflight dello spazio e dei volumi per operazioni locali di grandi dimensioni

- **ID stabile:** REPS-TECH-022
- **Data ricognizione:** 2026-11-21
- **Stato:** **Proposta — da valutare**
- **Filone:** strumenti e architettura; esclusivamente analisi e documentazione.
- **Priorità ipotetica:** **P2**, con possibile precedenza sui soli collaudi di backup/ripristino se emergono errori frequenti per spazio insufficiente. **Nessuna approvazione o assegnazione allo sviluppo.**
- **Ambito iniziale:** previsione prudente dello spazio necessario su **volume di destinazione del backup** e **volume che ospita l'archivio attivo durante il ripristino**. Solo in fasi successive, e previa conferma di necessità, import di media/RAW, elaborazioni derivate e CSV.

## Motivazione, destinatari ed evidenze nel codice

Un artista con migliaia di fotografie, una galleria con documenti e immagini RAW o uno studio che conserva backup su SSD/USB/NAS può avviare un'operazione di copia senza sapere in anticipo se il volume interessato la può contenere. Non è un problema di interfaccia cosmetico: un backup incompleto non è una copia recuperabile, mentre durante un ripristino devono coesistere temporaneamente dati vecchi e nuovi. Il controllo serve a **spiegare il fabbisogno prima della scrittura**, senza presentare una previsione come garanzia.

Ricognizione del branch `main`, 21 novembre 2026:

- [README](../../../README.md) e [MVP](../../MVP.md) documentano già backup completo verificato, ripristino con salvaguardia dell'archivio precedente e messaggi comprensibili in caso di disco pieno. **Queste protezioni esistenti non vanno replicate né descritte come mancanti.**
- In [`src-tauri/src/backup.rs`](../../../src-tauri/src/backup.rs), `create_backup_core` crea una sottocartella `.partial` nella destinazione scelta, produce con `VACUUM INTO` lo snapshot SQLite, copia i media e verifica manifest e hash prima della rinomina finale. Quindi il volume **scelto per il backup** deve avere spazio sufficiente per una nuova copia completa; la rinomina non crea una seconda copia dei contenuti sul medesimo filesystem.
- Nello stesso file, `restore_backup_core` verifica il backup scelto, copia database e media in uno **staging nella cartella padre dell'archivio attivo**, quindi rinomina l'archivio precedente in una cartella di sicurezza. Durante la preparazione devono quindi coesistere **l'archivio vecchio e lo staging nuovo sul volume dell'archivio**, anche quando la sorgente è una USB/NAS. La rinomina di sicurezza non è di per sé una seconda duplicazione del vecchio archivio.
- [MVP, sezione «Disco pieno»](../../MVP.md) dichiara che l'errore è riconosciuto e la cartella parziale viene rimossa, **ma non esiste un controllo preventivo dello spazio libero**. La presente proposta riguarda questo rischio *prima* che si esaurisca lo spazio, non la correzione dell'attuale recupero degli errori.
- [`docs/DATA-PORTABILITY.md`](../../DATA-PORTABILITY.md) distingue i **12 registri CSV** dall'archivio completo: l'eventuale estensione ai CSV non deve confonderli con un backup o modificare il formato.

**Non è stato misurato un incidente reale né un blocco ricorrente**: servono misure su archivi grandi e dispositivi fisici. Le capacità dichiarate dal filesystem, specie su NAS, volumi con quote, compressione o file sparse, possono essere incomplete o cambiare in qualsiasi momento.

**Destinatari:** chi utilizza cartelle selezionate liberamente per backup, supporti rimovibili, NAS montati o computer con spazio locale ridotto. Per un archivio piccolo su SSD capiente, il controllo dovrebbe essere quasi invisibile.

## Confini e anti-duplicazione

- **[TECH-009](REPS-TECH-009-catalogo-verifiche-conservazione-backup.md):** inventario e rivalidazione di **copie già prodotte**, conservazione prudente, eventuale recupero spazio. TECH-022 stima il fabbisogno **prima di una singola nuova operazione** e non cancella backup.
- **[TECH-012](REPS-TECH-012-migrazioni-archivio-preflight-copia-sicurezza.md):** checkpoint prima di **migrazione dello schema**, con propria logica di sicurezza. Qui il primo perimetro è il backup/ripristino esplicito già esistente; una possibile primitiva di stima dovrebbe poter essere riusata senza inglobare il progetto delle migrazioni.
- **[TECH-018](REPS-TECH-018-esportazioni-csv-scalabili-atomiche.md):** scrittura atomica e memoria limitata dei **CSV**. La stima di spazio è un possibile servizio condiviso futuro, non una riproposizione dell'esportatore.
- **[TECH-003](REPS-TECH-003-diagnostica-integrita-media.md):** verifica presenza/formato e possibile recupero selettivo dei media nell'archivio attivo; non calcola la capacità prima di duplicazioni.
- **[TECH-006](REPS-TECH-006-backup-cifrati-portabili.md)** e **[TECH-007](REPS-TECH-007-archivio-media-esterno-rilocalizzabile.md):** futuri formati o collocazioni possono cambiare i consumi stimati; non vengono presupposti.
- La [PR #14 — Gestione e UX](https://github.com/Cesare-ml/Repertica-Studio/pull/14) propone processi professionali di catalogazione e commercio, non il preflight dei filesystem: **nessuna sua scheda viene modificata**.
- **Non è** un sistema di ottimizzazione del disco, deduplicazione, cleanup automatico, deframmentazione, quota centralizzata o automazione di cancellazione.

## Scenari d'uso e flussi UX proposti

1. **Backup manuale in cartella scelta:** dopo la scelta della destinazione tramite dialogo nativo, mostrare un riepilogo discreto: «Copia stimata: circa …», «Spazio disponibile dichiarato: … / non verificabile» e spiegazione sintetica del margine. Se il volume è chiaramente insufficiente, fermarsi *prima* di creare `.partial`; offrire «Scegli un'altra cartella» e «Ricalcola». Non rimuovere copie precedenti per fare posto.
2. **Backup su USB/NAS:** controllare esistenza, permessi e disponibilità della destinazione. Se l'OS non espone affidabilmente lo spazio disponibile, mostrare «Spazio non verificabile su questo dispositivo», **non** un semaforo verde. Decidere separatamente la politica di avvio con conferma quando il dato è ignoto, continuando comunque a gestire gli errori di scrittura.
3. **Ripristino da disco esterno:** dopo `inspect_backup_core`, distinguere con chiarezza **sorgente del backup** (lettura), **destinazione dell'archivio** (staging nuovo) e **copia del precedente archivio** (conservata sul volume di destinazione). Lo spazio richiesto per lo staging va confrontato con lo spazio libero sul volume **dell'archivio attivo**, non con quello del backup esterno. Mostrare che il vecchio archivio non verrà eliminato per recuperare spazio.
4. **Archivio già su disco quasi pieno:** se la stima non consente di conservare vecchio e nuovo insieme, impedire il ripristino distruttivo. Le alternative sono liberare spazio *manualmente*, scegliere in futuro un diverso archivio di destinazione attraverso un progetto multiarchivio separato, o rimandare l'operazione. **Mai** eliminare la copia precedente o passare a «ripristina sul posto» senza protezione.
5. **Backup automatico (già presente):** se manca capacità, annotare un errore locale comprensibile e seguire le regole di ritentativo già esistenti; non cancellare backup storici fuori dalle regole di conservazione autorizzate e non saturare il volume con tentativi paralleli. Una semplice verifica preventiva riduce errori evitabili, ma non sostituisce il trattamento dell'errore reale durante la scrittura.
6. **Variazione del volume durante il lavoro:** disconnessione, montaggio sostituito, quota ridotta o spazio consumato da altre app invalidano la stima. Ripetere i controlli quanto più vicino possibile all'avvio della scrittura e trattare `ENOSPC`/errore equivalente come esito ancora possibile; mantenere cleanup e rollback già previsti.
7. **Accessibilità:** le quantità possono essere lette da screen reader; la possibilità di procedere o meno non dipende soltanto dal colore. Il dettaglio di percorso completo è visibile su richiesta e copiabile, non inviato a log o servizi esterni.

### Regole di calcolo prudente, da validare

- **Backup:** stima preliminare = dimensione osservata dei media regolari + grandezza stimata del nuovo snapshot SQLite + overhead del manifest e filesystem + margine configurato internamente. La dimensione del file SQLite attivo non coincide sempre con l'output di `VACUUM INTO`; le stime non devono essere mostrate come quantità esatte.
- **Ripristino:** dopo verifica del manifest, i byte dichiarati e verificati forniscono una buona base per lo **staging nuovo**; restano eventuali overhead di directory, block allocation, FS e margine. L'archivio vecchio occupa **già** spazio: non va erroneamente sommato come un'ulteriore nuova copia, né considerato liberabile.
- **Filesystem:** distinguere spazio **totale**, **disponibile per il processo** e **sconosciuto**. Quote per utente, snapshot del filesystem, sparse files, hardlink, deduplicazione e compressione non consentono un confronto numerico sempre esatto. Non fidarsi dell'etichetta di volume o di un path cache: ricontrollare il volume reale.
- **Politiche:** una percentuale di margine o una soglia fissa vanno valutate con benchmark e test; **nessun numero arbitrario** viene imposto da questa scheda. Se i valori sono inconcludenti, presentare esplicitamente incertezza anziché bloccare un backup valido solo sulla base di una previsione pessimistica.
- **Coerenza:** stimare non equivale a bloccare tutte le modifiche dell'archivio durante la scansione. Verificare eventuali cambiamenti fra preflight e acquisizione dello snapshot; mantenere le primitive di snapshot/lock e le verifiche finali già esistenti.

## Modello dati e ipotesi API — esclusivamente descrittive

Non è richiesta una nuova tabella SQLite né l'introduzione di dati sensibili in rete. Possibili strutture *temporanee* in memoria:

| Oggetto concettuale | Campi e significato |
| --- | --- |
| `StoragePlan` | `operation: Backup | Restore`, `source_volume_ref`, `target_volume_ref`, `estimate_bytes?`, `estimated_overhead_bytes?`, `existing_archive_bytes?`, `assumptions[]`; nessun importo economico |
| `VolumeObservation` | `volume_id_ephemeral?`, `available_bytes?`, `observed_at`, `reliability: Reported | Unknown`, `reason?`; non salvare il percorso privato nei report per default |
| `CapacityDecision` | `status: SufficientEstimate | InsufficientEstimate | Unknown`, `blocking_reason?`, `warnings[]`, `requires_recheck`; non significa che il successo della copia sia assicurato |
| `OperationSpaceOutcome` | `started_at`, `bytes_written?`, `error_code?`, `partial_removed?`, `previous_archive_preserved?`; non registra contenuti di opere o contatti |

Contratti ipotetici: `estimateBackupCapacity(archive, destination) -> StoragePlan`; `estimateRestoreCapacity(verifiedBackup, activeArchive) -> StoragePlan`; `observeTargetVolume(plan) -> VolumeObservation`; `revalidateBeforeWrite(plan) -> CapacityDecision`. I nomi **non sono API approvate**. Nella prima fase il piano è effimero, non parte dell'archivio e non deve invalidare backup v1 o lo schema SQLite v14.

## Offline, privacy, impostazioni, lingue e compatibilità

- **Offline-first:** nessun account, server, abbonamento, chiamata Internet o probe di dispositivi non scelti. Un NAS è un filesystem **già montato**, non un'integrazione cloud obbligatoria.
- **Privacy:** i percorsi e i nomi delle cartelle possono rivelare clienti e progetti. Non includerli nei log condivisibili, nelle telemetrie (non previste) o in esportazioni di default. Non aprire file per leggerne contenuto quando bastano metadati; rispetta permessi, traversal, symlink e cancellazione/rimontaggio del percorso.
- **Impostazioni:** inizialmente **nessuna opzione necessaria**: il riepilogo può apparire solo per grandi quantità, condizioni incerte o rischio reale. Da decidere se consentire una preferenza «Mostra sempre il dettaglio capacità»; mai disattivare in silenzio i controlli di sicurezza del ripristino.
- **Multilingua:** lingua di sistema, fallback inglese, possibile selezione manuale fra lingue supportate (IT/EN/ES/FR/DE). Tradurre spiegazioni e stati, non percorsi o contenuti utente. Dimensioni leggibili e locali; non mescolare MB decimali e MiB binari senza etichetta.
- **Compatibilità:** Windows, macOS, Linux, volumi locali, USB, NAS SMB/NFS montati; gestire errori e informazioni mancanti senza assumere equivalenza delle API per spazio libero e delle semantiche di rename. Nessuna dipendenza o libreria scelta adesso.
- **Portabilità:** backup e restore restano selezionabili su qualsiasi cartella accessibile; nessuna restrizione artificiale a un percorso gestito da Repertica. CSV, manifest e media esistenti restano esportabili integralmente.

## Dipendenze, alternative, rischi e complessità

**Dipendenze concettuali:** collaudi reali della pipeline di backup/ripristino su sistemi operativi e filesystem previsti ([issue #7](https://github.com/Cesare-ml/Repertica-Studio/issues/7)); misure di consumo per snapshot SQLite e media; prova della gestione attuale di `ENOSPC`/unità rimossa. La soluzione dovrebbe riusare il validatore di backup corrente anziché aggiungere un secondo interprete del manifest.

**Alternative valutabili:** (A) mantenere soltanto l'errore di scrittura già gestito e aggiungere indicazioni nella documentazione: minimo costo, ma scoperta tardiva; (B) **solo stima e avviso** prima dell'operazione, primo passo prudente; (C) bloccare soltanto i casi chiaramente impossibili, mantenendo esplicita l'incertezza; (D) preallocare o riservare spazio: più complesso, non affidabile né supportato uniformemente su filesystem e NAS, **non proposto per la fase iniziale**.

**Rischi:** falsa garanzia di successo, soglie troppo restrittive, scansione lenta o bloccante su archivi enormi, diritti di quota diversi da quelli osservati, variazione TOCTOU, duplicazione di stime fra moduli, snapshot SQLite più grande del previsto, sparse/compressed files, spazio condiviso da più utenti NAS, errori locali tradotti male, esposizione di percorsi privati. Non promettere né «spazio garantito» né prevenzione di tutti i fallimenti.

**Complessità preliminare:** **medio-bassa** per avviso non invasivo e sola capacità di backup/ripristino, **media–alta** per pianificatore robusto multipiattaforma, stime e recheck affidabili su NAS/quote; più alta per estensione a RAW/elaborazioni/cancellazioni. Stime indicative, non impegni di realizzazione.

## Test e criteri di accettazione proposti

1. Su backup manuale, preflight **senza alcuna scrittura** prima della conferma; rifiuto anticipato di una destinazione palesemente insufficiente senza lasciare cartelle `.partial`.
2. Ripristino da USB verso un archivio su SSD quasi pieno: confronto con **spazio del volume SSD**, non della USB, e nessuna alterazione dell'archivio attivo quando lo staging non entra.
3. Preservazione della precedente cartella `ReperticaStudio-before-restore-...` e delle fotografie storiche dei certificati, anche con errore imprevisto e rollback. Nessuna cancellazione automatica per guadagnare spazio.
4. Tolleranza a spazio ridotto dopo il controllo, unità estratta o rimontata, ACL/permessi negati, quota NAS, filesystem con valori ignoti; il fallimento reale non produce messaggio falso di «backup verificato».
5. Confronto misure stima/byte realmente scritti per archivi piccoli e grandi, con SQLite WAL, media RAW, tante fotografie, file compressi/sparse e blocchi di allocazione differenti. Soglie da fissare **dopo** misurazioni.
6. Nessuna regressione su backup v1 già validi, esportazioni CSV, archivio SQLite, backup automatici e ripristino già protetto. Non cambia la scelta libera della cartella.
7. Test nativi su Windows/macOS/Linux e supporti USB/NAS reali; misura di latenza/memoria della scansione; schermate accessibili via tastiera e screen reader, fallbacks EN e lingue disponibili.
8. Nessun dato di collezionisti, contenuto immagine o percorso sensibile in un eventuale report diagnostico condivisibile per default.

## Fasi eventuali, non approvate

- **Fase 0 — verifica necessità:** raccogliere casi anonimi di insufficienza spazio e misurare differenze fra dimensioni di origine, staging, snapshot e spazio effettivo. Verificare quali API OS danno dati utili nei filesystem destinatari.
- **Fase 1 — minimo utile:** stima prudente in sola lettura e messaggi chiari *solo* per backup e ripristino manuali; ricontrollo immediatamente prima della scrittura, lasciando attive tutte le protezioni già presenti.
- **Fase 2 — integrazione controllata:** unificare il preflight dei backup automatici con gestione degli errori esistente; aggiungere test con quota/USB/NAS e variabilità durante l'operazione.
- **Fase 3 — soltanto se richiesta:** riuso della stessa logica per import RAW, generazione derivati e grandi esportazioni; da coordinare con TECH-018, TECH-012 e TECH-007 senza modificare i loro documenti.

## Questioni aperte per il proprietario

1. Ci sono casi reali di backup/ripristino falliti per disco pieno, oppure basta una guida e l'errore attuale? Quali dimensioni tipiche degli archivi?
2. È più utile mostrare la stima **sempre**, solo quando lo spazio è scarso, o su richiesta in un pannello avanzato?
3. In caso di capacità **sconosciuta** su NAS, consentire il backup manuale dopo avviso ma applicare una policy più prudente al ripristino? Quali messaggi sono accettabili?
4. Quale margine prudenziale è giustificato dalle misurazioni, senza impedire inutilmente le operazioni su volumi compressi o con quota?
5. È preferibile limitare l'eventuale prima implementazione al backup/ripristino manuale, rimandando media/RAW ed esportazioni?

**Esito:** proposta di studio, non bug confermato, non autorizzazione allo sviluppo. Non modifica il comportamento dell'applicazione e non approva componenti, dipendenze o nuove API.
