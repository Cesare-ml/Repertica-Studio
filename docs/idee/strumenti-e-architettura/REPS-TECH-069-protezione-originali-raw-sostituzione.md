# REPS-TECH-069 — Protezione degli originali RAW alla sostituzione, con conservazione esplicita e recuperabile

- **ID stabile:** REPS-TECH-069
- **Data ricognizione:** 2026-11-23
- **Stato:** **Proposta — da valutare**
- **Filone:** Strumenti tecnici e architettura; esclusivamente ricerca e documentazione.
- **Priorità proposta, non approvata:** **P1–P2 solo per la salvaguardia minima se professionisti confermano una perdita inattesa**; **P2–P3** per una vera cronologia di RAW, da valutare soltanto in presenza di richiesta e spazio disponibile. Nessun incarico di sviluppo.
- **Principio:** la libertà di sostituire/rimuovere un allegato non deve far presumere che un file originale indispensabile sia recuperabile. Nessun account, cloud, abbonamento o automatismo remoto.

## 1. Problema professionale, destinatari ed evidenza verificata

Per un artista o un fotografo d'arte il RAW di fotocamera è spesso un **originale digitale di lavoro** e può essere molto più prezioso della JPEG di catalogo. Un archivista può sostituire una scansione o un file DNG associato all'opera credendo di aggiornare il riferimento *corrente*, non necessariamente di eliminare dall'archivio la **precedente copia RAW**. Se la copia sorgente esterna è stata eliminata e non esiste un backup antecedente, quella versione può non essere più recuperabile.

**Osservazioni statiche su `main` il 23 novembre 2026, senza esecuzione di test o segnalazione di incidente:**

1. [`src-tauri/src/db.rs`](../../../src-tauri/src/db.rs) registra un solo RAW corrente per opera in `artwork_raw_files` (chiave primaria `artwork_id`). La funzionalità è documentata in [`docs/MVP.md`](../../MVP.md) come RAW sostituibile/rimovibile, fino a 1 GiB, SHA-256 e backup integrale.
2. In [`src-tauri/src/raw_files.rs`](../../../src-tauri/src/raw_files.rs), `attach_core` prepara una nuova copia `media/raw-UUID.ext`, controlla l'integrità del vecchio file, usa `link` per cancellare la **riga del precedente collegamento** e associare il nuovo RAW, poi dopo il commit prova `cleanup_unlinked_raw`. Se il precedente file è regolare, conserva l'hash atteso e non è necessario ad altre relazioni, viene **eliminato dalla cartella media**. Il comportamento è **intenzionale**, come conferma il test `attaching_stores_hash_size_and_replaces_the_previous_file`. File alterati, collegati ad altre schede o necessari agli snapshot non vengono eliminati indiscriminatamente: le protezioni esistenti vanno preservate.
3. In [`src/app/app.component.ts`](../../../src/app/app.component.ts), `attachRaw` presenta il dialogo per scegliere un nuovo RAW e invoca subito `api.attachArtworkRaw`; **nel percorso esaminato non c'è una conferma ulteriore che il vecchio RAW interno sarà rimosso**. Invece `removeRaw` presenta già una conferma esplicita della cancellazione del file dell'archivio. L'HTML distingue già «Sostituisci file RAW…», «Salva una copia…» e «Rimuovi».
4. Il file sorgente **fuori** dall'archivio **non viene eliminato** dalla sostituzione; possono inoltre esistere backup precedenti. La lacuna da studiare non è una cancellazione generale del sorgente, ma la differenza fra **sostituzione del riferimento** e **conservazione della copia originale già importata** dentro Repertica.
5. Il registro CSV [`raw_files`](../../DATA-PORTABILITY.md) esporta soltanto il RAW correntemente associato e i relativi metadati; il backup include i byte presenti in `media/`, ma non può recuperare versioni già eliminate prima della creazione della copia.

**Destinatari da consultare:** artisti che conservano negativi RAW, fotografi di opere, studi, gallerie, archivi privati e responsabili di fondi fotografici. **Bisogno non validato da interviste**: non presumere che tutti vogliano più versioni di file grandi.

## 2. Confini e anti-doppioni

- [TECH-053 — Manifest aperto della storia delle revisioni Media Center](REPS-TECH-053-manifest-aperto-storia-revisioni-media.md) tratta l'**esportazione della storia già presente** delle immagini elaborate in Media Center. Il RAW per opera è un allegato distinto, con una singola riga corrente; questa scheda studia **prima della cancellazione** come evitare una perdita inattesa. Nessun nuovo motore di revisioni fotografiche.
- [TECH-065 — Rientro da editor esterni](REPS-TECH-065-round-trip-revisioni-fotografiche-editor-esterni.md) riguarda fotografie/modifiche JPEG/PNG/WebP e revisioni Media Center; non è conservazione degli originali RAW precedenti.
- [TECH-046 — TIFF/BigTIFF](REPS-TECH-046-master-fotografici-tiff-bigtiff-offline.md) studia un **altro formato** di master, non il ciclo di vita di un RAW già supportato.
- [TECH-006](REPS-TECH-006-backup-cifrati-portabili.md), [TECH-009](REPS-TECH-009-catalogo-verifiche-conservazione-backup.md) e [TECH-052](REPS-TECH-052-verifica-indipendente-backup-formato-v1.md) riguardano copie di sicurezza e verifiche, **non una cronologia consultabile dei RAW associati a una singola opera**. Un backup è una rete di sicurezza, non una garanzia di versionamento infinito.
- [TECH-003 — Diagnostica media](REPS-TECH-003-diagnostica-integrita-media.md) riguarda integrità e file mancanti; non deve diventare un motore di ripristino di versioni.
- [UX-057 — Opere digitali](https://github.com/Cesare-ml/Repertica-Studio/blob/docs/idee-gestione-ux-20261121/docs/idee/gestione-e-ux/REPS-UX-057-opere-digitali-file-master-istruzioni-presentazione.md) descrive il **significato delle versioni costitutive di opere digitali**, non le cautele tecniche dello specifico allegato RAW di una fotografia. Verificato l'indice Gestione e UX della [PR #14](https://github.com/Cesare-ml/Repertica-Studio/pull/14): evitare nuovi campi di gestione opere e duplicazione del flusso UX.
- **Alternativa zero sviluppo:** rendere ben visibile l'attuale semantica e raccomandare un export o backup **prima** della sostituzione; verificare se per gli utenti è già sufficiente.

## 3. Scenari d'uso da verificare

**A — RAW iniziale sbagliato.** Un'artista ha associato per errore uno scatto RAW e lo sostituisce con il file corretto. Può volere eliminare il precedente per non occupare inutilmente spazio. Il prodotto non deve imporre conservazione permanente né moltiplicare automaticamente file da 1 GiB.

**B — Nuova acquisizione dello stesso dipinto.** Un fotografo riprende un'opera con una fotocamera diversa. Vuole usare il nuovo RAW per la consultazione ma **mantenere recuperabile lo scatto originale**, con data, nome e hash, fino a una cancellazione intenzionale.

**C — Archivio senza originale esterno.** Dopo anni, il file importato nell'archivio è l'unica copia reperibile. Un comando «Sostituisci» non dovrebbe sorprendere eliminandola; la conservazione deve essere una decisione informata, con preflight e spazio reale.

**D — Disco quasi pieno.** L'utente non può conservare dieci versioni da 1 GiB. La funzionalità deve proporre l'export su cartella liberamente scelta o un backup verificato; non deve affermare che un backup non creato sia disponibile.

**E — RAW alterato o condiviso.** Se l'hash non coincide, un symlink sostituisce il file, o una fotografia storica continua a riferirsi al percorso, la proposta non deve bypassare le salvaguardie attuali né eliminare materiale «orfano» per comodità.

## 4. Specifica funzionale e flussi UX candidati

### Fase minima senza schema nuovo (preferita per la validazione)

1. Da «Modifica opera → File RAW → Sostituisci», dopo la scelta di un file, **se esiste già un RAW interno** mostrare un riepilogo leggibile: nome e dimensione del file attuale; nome e dimensione del nuovo; differenza fra «RAW corrente» e «copia interna precedente»; esito della verifica d'integrità e, quando affidabile, disponibilità di una copia esportata scelta dall'utente.
2. Presentare azioni esplicite: **«Annulla»**, **«Salva prima una copia del RAW attuale…»** in una cartella scelta dall'utente, oppure **«Sostituisci e rimuovi la copia precedente»** con descrizione inequivocabile. La sola scelta del file **non** deve dare per confermata la rimozione. Nessun pulsante ambiguo come «Continua» o opzioni pre-spuntate per la cancellazione.
3. Se il professionista esporta prima il vecchio RAW, verificarne byte e SHA-256 come già previsto dall'esportatore, senza sovrascrivere file preesistenti; solo dopo una conferma separata effettuare l'associazione nuova. **Non equiparare «file salvato» a «backup verificato dell'intero archivio».**
4. Se il RAW è identico per hash e byte alla versione corrente, valutare un avviso di import superfluo; **non effettuare deduplicazioni silenziose** su filesystem o SQLite.
5. La rimozione manuale già presente conserva la propria conferma. Nessuna rimozione retroattiva o «pulizia» automatica di versioni storiche.

### Estensione eventuale: cronologia RAW opt-in

Solo se la necessità viene confermata, studiare un elenco facoltativo **«RAW correntemente associato / RAW conservati»** per opera, con data di acquisizione nell'archivio, nome originale, byte, hash verificato e stato. Una sostituzione, quando l'utente sceglie **«Conserva il precedente nell'archivio»**, trasferisce il vecchio collegamento a una voce storica senza riscrivere o ricodificare il file. La nuova associazione diventa corrente; una versione conservata può essere esportata in copia, riassociata **esplicitamente** come corrente e rimossa soltanto con conferma e verifica di riferimenti esterni. Nessuna versione diventa «autentica», «migliore» o «certificata» per effetto del software.

- Mostrare l'**aumento reale stimato** dello spazio usato e l'avvertenza che conservare vecchi RAW aumenta il volume del backup. Non far affidamento su un conteggio teorico se il filesystem comprime/deduplica i byte.
- Mantenere l'interfaccia di default leggera: chi allega un solo RAW non deve imparare un complesso gestore di versioni.
- Se un export da una versione storica non è materialmente disponibile, mostrare «file non disponibile» e non inventare un collegamento recuperato dal backup.
- Le versioni storiche non implicano modifiche ai certificati già emessi, alle fotografie di copertina, ai titoli o alle vendite.

## 5. Dati e ipotesi di API esclusivamente descrittive

**Fase minima:** nessuna nuova tabella richiesta; lettura della riga corrente `artwork_raw_files`, verifica `size_bytes`/`sha256` e risultati di export; la conferma nel percorso utente non autorizza nuove scritture fuori dal comando scelto.

**Eventuale schema futuro, non approvato:** `artwork_raw_versions` con `id` UUID, `artwork_id`, `relative_media_path` sicuro, `original_name`, `size_bytes`, `sha256`, `imported_at`, `retained_at?`, `status` (ad esempio `current|retained`) e `retention_note?` facoltativa. Non presumere di derivare la data di scatto dall'importazione. La tabella corrente potrebbe restare una **vista o relazione di compatibilità**: contratti esistenti di GUI, MCP e CSV non vanno cambiati di nascosto. Un identificativo versione stabile serve per esportare o riassociare senza confondere due nomi file uguali.

**API concettuali (non esistenti):**
- `previewRawReplacement(artworkId, candidatePath) -> {current, candidate, integrity, estimatedSpace, choices}`
- `exportCurrentRawCopy(artworkId, destination) -> {verifiedHash, destination}` (riuso della capacità esistente)
- `replaceRawWithDecision(artworkId, candidate, expectedCurrentHash, retentionChoice) -> Outcome` (ipotesi di controllo concorrente, non firma API)
- `listRetainedRawVersions(artworkId) -> Version[]` e `restoreRetainedRawVersion(versionId, expectedCurrentVersion) -> Outcome` **solo nell'estensione futura**.

Requisiti di integrità da studiare: decisione calcolata su stato corrente, verifica al momento del commit per non eliminare un RAW differente da quello visto nella conferma, transazione del collegamento, file staged sicuro, recupero se un processo si arresta fra commit e cleanup (in raccordo con [TECH-064](REPS-TECH-064-coordinamento-processi-archivio-attivo.md) e [TECH-038](REPS-TECH-038-staging-media-backup-concorrenza-snapshot.md)). La presenza di un file `media/` non deve essere l'unico indice di una versione recuperabile.

## 6. Offline, sicurezza, privacy, backup e interoperabilità

- **Tutto offline:** import, hashing, anteprima, versioni ed esportazione operano su filesystem/SQLite locali. Nessun account, invio online, cloud obbligatorio, telemetria o assistente IA necessario. I file sorgente possono trovarsi su USB/NAS scelti dall'utente; niente copia in una cartella remota non autorizzata.
- **Integrità e sicurezza:** preservare `safe_relative`/validazione dei percorsi e verifiche di file regolare/assenza di symlink, limite attuale 1 GiB, SHA-256, stream a memoria limitata, `create_new` e comportamento fail-closed sui file alterati. Nessuna pulizia di RAW storici per liberare spazio senza un'azione esplicita. Distinguere errore di copia, permessi, spazio, sostituzione, export e hash discordante.
- **Backup:** se una cronologia interna fosse introdotta, **ogni file storico dichiarato conservato** deve entrare nel backup autocontenuto e nella verifica di integrità e ripristino, inclusi test su volumi esterni. Il backup corrente già verifica tutti i media fisici nel manifesto; resta però necessario rendere **riconoscibili e ricostruibili i collegamenti alle versioni** tramite i dati, non solo conservare byte anonimi.
- **Portabilità:** il CSV attuale `raw_files` deve continuare a rappresentare il RAW corrente senza modificare silenziosamente le colonne. Eventuale esportazione *aggiuntiva* e separata della cronologia con UUID, hash, stato e nomi, oppure manifest aperto; niente reimport automatico non validato. Backup integrale e restore restano la via per trasferire l'archivio.
- **Privacy:** i RAW possono contenere coordinate EXIF, firme, volti, dettagli del laboratorio o informazioni non pubblicabili. Nessun thumbnail, EXIF o identificativo personale nei log diagnostici condivisibili per default; export solo nella destinazione scelta, con un avviso sulla natura sensibile dei byte. Hash non prova titolarità dei diritti né autenticità.
- **Impostazioni:** nella fase minima nessuna preferenza obbligatoria. Per cronologia eventuale preferenza **per singola sostituzione**, con possibile default conservativo da decidere dopo test; evitare retention arbitraria, quota per tutti e scadenze che cancellano dati senza consenso.
- **Multilingua e accessibilità:** lingua di sistema, fallback **inglese**, traduzioni UI separate dal nome originale; mantenere IT/EN/ES/FR/DE dove disponibili. Avvisi comprensibili anche con tastiera/screen reader, nomi file troncati visualmente ma disponibili per intero, focus corretto e contrasto adeguato. I pulsanti devono distinguere chiaramente conservazione, esportazione e cancellazione.

## 7. Dipendenze, alternative, rischi e complessità

**Dipendenze presenti:** SQLite locale, Rust, hashing SHA-256, gestione `media/` e backup verificati; l'analisi non autorizza nuove librerie, workflow o migrazioni. Eventuale cronologia richiede una revisione controllata dello schema, dei percorsi di ripristino, eliminazione opera, manutenzione media, esportazioni e compatibilità con installazioni vecchie.

**Alternative da confrontare:**
- **A. Nessun cambiamento:** spiegare la semantica attuale e chiedere di fare un export o un backup verificato prima della sostituzione.
- **B. Conferma mirata + export prima della sostituzione:** complessità **bassa/media**, senza nuova tabella; riduce la sorpresa ma non crea storia recuperabile dentro l'app.
- **C. Conservazione di una sola precedente versione opt-in:** complessità **media/alta**, più spazio e nuovi percorsi di recupero.
- **D. Cronologia completa opt-in:** complessità **alta**, conservazione e cleanup, compatibilità schema e backup, rischio crescita incontrollata dello spazio. Valutare solo con prove d'uso.

**Rischi:** confondere originale esterno con copia interna, falso senso di sicurezza, duplicazione di gigabyte, file rimasti orfani dopo crash, perdita di collegamenti durante migrazione, backup troppo grandi, denominazioni identiche, associazioni di un RAW di un'altra opera, cancellazioni concorrenti, hash non corrispondenti, incompatibilità del vecchio software. **L'assenza di crash attuali non rende dimostrata alcuna perdita reale**: l'osservazione riguarda una politica deliberata di ciclo di vita e una possibile aspettativa diversa.

## 8. Test e criteri di accettazione da valutare

1. **Baseline solo descrittiva:** riprodurre in ambiente di prova, con file RAW sintetici compatibili e senza dati personali, la sostituzione attuale; verificare che il RAW precedente venga rimosso **solo quando non esistono riferimenti che lo preservano**, e che il sorgente esterno rimanga intatto. Documentare l'interfaccia attuale.
2. **Fase minima:** ogni sostituzione di un RAW già presente deve mostrare nome/dimensione e conseguenza della rimozione **prima** del commit; «Annulla» conserva byte, record e backup esistenti. Il percorso «Salva una copia…» verifica la copia e non sovrascrive un file esistente; una mancata copia non autorizza implicitamente la sostituzione.
3. **File guasto:** RAW interno con hash divergente o symlink, sorgente illeggibile, nome anomalo, permessi negati o supporto scollegato: nessuna cancellazione non richiesta, nessuna sostituzione apparentemente riuscita.
4. **Concorrenza e arresto:** se cambia il RAW fra conferma e commit, annullare e richiedere nuova revisione; simulated crash e spazio insufficiente non devono lasciare una versione registrata ma inesistente, né eliminare il precedente prima di una nuova associazione valida.
5. **Cronologia, solo se scelta:** su 1, 2 e 5 versioni grandi, assicurare accesso all'originale conservato, hash esatti, export in copia, selezione corrente esplicita, backup/restore con **tutte** le versioni, eliminazione con conferma e sicurezza dei riferimenti. Non confondere `retained` con certificazione di conservazione digitale a norma.
6. **Compatibilità:** invariati CSV corrente, app precedente finché lo schema lo permette, fotografie in Media Center, certificati, vendita e MCP; nessuna esposizione di byte RAW nel server MCP. Verificare Windows/macOS/Linux e almeno un disco esterno.
7. **Accessibilità/I18N:** test tastiera, lettura annunci, messaggi di spazio/hash ed errori nelle lingue disponibili, fallback inglese e nomi file Unicode.

**Criterio di stop:** se 4–6 professionisti confermano che sanno già che «Sostituisci» cancella la copia interna e usano sistematicamente backup/export, limitarsi eventualmente a testo di avviso o documentazione e **non creare cronologia**.

## 9. Priorità e fasi possibili, senza autorizzazione allo sviluppo

- **Fase 0 — ricerca:** confermare aspettative su «Sostituisci», frequenza di rinnovo RAW, dimensioni file, routine backup e recuperi richiesti; analizzare eventi di conservazione durante sostituzione ed eliminazione di opere. Nessuna funzionalità approvata.
- **Fase 1 — salvaguardia minima, se giustificata:** eventuale conferma esplicita e scelta di export prima della sostituzione, con test anti-regressione; senza cronologia o migrazioni.
- **Fase 2 — studio tecnico con casi reali:** confrontare singola versione precedente e cronologia opt-in, misurare costi di backup, spazio e ripristino; definire modello compatibile prima di una possibile implementazione futura.
- **Fase 3 — solo con approvazione successiva:** specifica di versione, migrazione/rollback dei metadati, test su perdita alimentazione, backup/restore, portabilità e collaudo con professionisti.

**Priorità ipotetica:** P1–P2 per un avviso anti-perdita soltanto se la sorpresa/il danno sono confermati; P2–P3 per retention versionata; **altrimenti nessuna modifica**.

## 10. Questioni aperte per la revisione del proprietario

1. I RAW importati sono considerati **copie accessorie** oppure **originali di conservazione**? L'utente conserva sempre la sorgente altrove?
2. «Sostituisci» deve continuare a eliminare la copia interna precedente dopo avviso, oppure offrire una conservazione facoltativa?
3. Il professionista desidera **una sola versione precedente** oppure una storia completa? Quanti GB in più accetterebbe per opera?
4. Prima di una sostituzione, basta il comando «Salva una copia…» con conferma, o è indispensabile una versione consultabile e ripristinabile dall'app?
5. Come comunicare che **un backup creato dopo la sostituzione non recupera il RAW già eliminato**?
6. Cosa succede quando la precedente copia è referenziata da altri materiali storici, e come rendere comprensibile un RAW trattenuto per sicurezza?
7. Quale forma di test nativi e feedback di 4–6 professionisti giustifica davvero una nuova struttura dati?

**Nessuna domanda è risolta a nome del proprietario. Stato invariato: «Proposta — da valutare». Nessun codice o test da implementare per effetto della presente scheda.**