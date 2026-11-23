# REPS-TECH-065 — Rientro controllato delle fotografie modificate con editor esterni

- **ID stabile:** REPS-TECH-065
- **Data della ricognizione:** 2026-11-23
- **Stato:** **Proposta — da valutare**
- **Filone:** Strumenti e architettura; esclusivamente analisi e documentazione
- **Priorità ipotetica:** **P3**, eventualmente **P2** soltanto con casi professionali ricorrenti e riscontro di fattibilità; non approvata e non assegnata
- **Decisione riservata al proprietario:** nessun avvio di sviluppo, modifica dati, nuova dipendenza, API, migrazione o workflow è autorizzato da questa scheda

## 1. Problema, destinatari e riscontri verificabili

**Esigenza da validare:** un fotografo d'arte corregge il colore in Darktable, ritocca un dettaglio in GIMP o prepara un derivato in Photoshop/Affinity Photo. Vuole conservare nel Media Center **la fotografia originale, il file elaborato e il rapporto esplicito fra loro**, senza reinserire manualmente metadati o creare due elementi apparentemente indipendenti. È un flusso distinto dalle correzioni di ritaglio/prospettiva/luminosità già presenti nell'editor interno; non obbliga a installare programmi proprietari.

**Destinatari possibili:** fotografi di opere d'arte, artisti, laboratori fotografici, gallerie e archivi che richiedono lavorazioni non coperte dall'editor incorporato. Chi importa e ritaglia una singola immagine non dovrebbe vedere nuove procedure obbligatorie.

**Ricognizione statica di `main` al 23/11/2026:** letti [README](../../../README.md), [MVP](../../MVP.md), [DATA-PORTABILITY](../../DATA-PORTABILITY.md), [media.rs](../../../src-tauri/src/media.rs) e [imaging.rs](../../../src-tauri/src/imaging.rs).

- Il Media Center possiede già `media_items`, `media_revisions`, ID stabili, hash SHA-256, revisioni non distruttive, `apply_edit_core`, `restore_revision_core` e `export_core`. Può esportare il file originale, corrente oppure un derivato, senza sovrascrivere file esistenti.
- `import_core` registra una fotografia importata come **nuovo elemento** della libreria. Nei comandi Tauri pubblicati nel tratto esaminato (`media_import`, `media_apply_edit`, `media_restore_revision`, `media_export`) **non emerge un comando dedicato a collegare un risultato modificato fuori dall'app come nuova revisione di un `media_id` già presente**.
- `set_current` e `repoint_references` fanno sì che una revisione resa corrente venga utilizzata dalle schede opera, dalle fotografie in galleria e dai riferimenti dei collezionisti che impiegano lo stesso `media_id`. **Non è quindi prudente attivare automaticamente un file rientrato dall'esterno**.
- I formati fotografici gestiti dal percorso di decodifica del Media Center sono JPEG, PNG e WebP, con limiti di dimensione/pixel; i file RAW fotografici sono gestiti in un modulo distinto. Non presumere supporto a PSD, XCF, TIFF o sidecar finché non venga valutato separatamente.
- I **12 CSV attuali** esportano registri e riferimenti, non una storia riimportabile dei file. Il backup verificato comprende SQLite e media ed è distinto da questo percorso editoriale.

Questa lettura **non dimostra un bug né una domanda di utenti già osservata**. Non sono stati eseguiti test, benchmark o prove native.

## 2. Perimetro e confronto anti-doppioni

- [TECH-053](REPS-TECH-053-manifest-aperto-storia-revisioni-media.md) studia **esportare verso l'esterno** la storia delle versioni con manifest leggibile e verificabile, esplicitamente senza reimport. Qui l'oggetto esclusivo è il **rientro manuale** di *un* file elaborato, con associazione controllata a una fotografia esistente. Le due proposte possono condividere solo convenzioni per ID e hash, non duplicare interfacce.
- [TECH-021](REPS-TECH-021-gestione-colore-icc-derivati-fotografici.md) tratta la correttezza dei profili ICC; [TECH-024](REPS-TECH-024-metadati-fotografici-iptc-xmp-interoperabili.md) IPTC/XMP e sidecar; [TECH-013](REPS-TECH-013-copie-fotografiche-condivisibili-privacy-metadati.md) copie destinate alla condivisione. Il round-trip **non promette** fedeltà cromatica, lettura di sidecar o rimozione di metadati privati e deve riutilizzare eventuali decisioni future su questi temi.
- [TECH-045](REPS-TECH-045-associazione-guidata-foto-importate-opere.md) lega **nuove fotografie** alle **schede delle opere**; questa ipotesi lega un derivato a **una revisione di un singolo elemento Media Center**, non abbina opere a fotografie.
- [TECH-003](REPS-TECH-003-diagnostica-integrita-media.md), [TECH-033](REPS-TECH-033-salvataggio-condizionale-schede-obsolete.md), [TECH-038](REPS-TECH-038-staging-media-backup-concorrenza-snapshot.md), [TECH-064](REPS-TECH-064-coordinamento-processi-archivio-attivo.md) descrivono controlli da rispettare, senza duplicarli.
- Nell'altro filone, [UX-057](https://github.com/Cesare-ml/Repertica-Studio/blob/docs/idee-gestione-ux-20261121/docs/idee/gestione-e-ux/REPS-UX-057-opere-digitali-file-master-istruzioni-presentazione.md) studia i **file costitutivi di un'opera digitale**, non fotografie documentarie dell'opera; [UX-026](https://github.com/Cesare-ml/Repertica-Studio/blob/docs/idee-gestione-ux-20261121/docs/idee/gestione-e-ux/REPS-UX-026-diario-lavorazione-opera-fasi-datate.md) riguarda il processo creativo. Nessun file Gestione e UX va toccato.

**Esplicitamente esclusi:** apertura/esecuzione automatica di editor esterni, installazione di plugin o driver, sorveglianza periodica di cartelle, sincronizzazione bidirezionale, sostituzione tacita di originali, scrittura di sidecar, supporto generale ai formati proprietari, pubblicazione o invio in rete, modifiche retroattive a certificati emessi o documenti storici.

## 3. Scenari professionali e flussi UX ipotetici

**A — Correzione esterna di una foto.** Nel Media Center l'utente seleziona la foto e usa l'export `original` o `current` già esistente, salvando esplicitamente in una propria cartella. Modifica il file con il programma che preferisce. Tornato in Repertica sceglie *«Importa risultato come revisione di questa fotografia…»*, seleziona manualmente il file, vede un confronto visivo con la revisione corrente e conferma l'aggiunta. Il file importato **non diventa corrente per default**.

**B — Una foto utilizzata in più punti.** La stessa fotografia è associata a più schede o al profilo di un collezionista. Prima di rendere corrente la revisione esterna, Repertica presenta gli usi effettivi e avverte che, con il comportamento odierno di `repoint_references`, cambierebbero **tutti i riferimenti attivi** a quel `media_id`. Se la modifica deve valere soltanto per una destinazione, la prima versione **non procede**: l'utente può invece importare il file come elemento distinto con il flusso attuale. Nessuna deduzione automatica sulla licenza delle immagini.

**C — File cambiato o lavoro obsoleto.** Nel frattempo un collega o un'altra sessione ha creato una nuova revisione. Il file elaborato può ancora essere presentato, ma una schermata segnala che la base di partenza non è più quella corrente; l'utente sceglie se **archiviare come revisione non attiva** oppure annullare. Non attiva mai automaticamente il risultato. Un file identico a una revisione già presente produce un avviso di duplicato, senza sovrascritture.

**D — Esito negativo.** File mancante, contenuto non supportato, file troppo grande, supporto USB disconnesso, hash o decodifica non valida, spazio insufficiente: l'app mantiene integra la revisione corrente, segnala il motivo e lascia la fotografia già presente utilizzabile. Non pubblica un file parziale nel catalogo.

**Flusso minimo suggerito:** fotografia → «Esporta» corrente/originale (già disponibile) → modifica esterna senza coinvolgere Repertica → «Aggiungi revisione da file…» → scelta file tramite dialogo OS → preflight formato/dimensione/hash/usi esistenti → anteprima vecchio/nuovo → conferma **solo aggiunta, non attivazione** → eventuale successiva azione separata «Rendi corrente» dopo avviso sugli utilizzi. Una sola fotografia per volta nella prima fase; nessuna cartella monitorata.

## 4. Dati e ipotesi di API — SOLO descrittive

Riutilizzare ove possibile il modello esistente `media_items`/`media_revisions`: `media_id`, nuovo `revision_id` UUID, `number`, `kind` che distingua un import esterno da `EDIT` interno **solo se realmente necessario**, `image_path` relativo, `sha256`, dimensioni, byte e `created_at`. Il riferimento alla revisione di base è un'**ipotesi di tracciabilità**, non prova che le modifiche derivino matematicamente da quella revisione: la provenienza rimane *dichiarata dall'utente*. Nessun percorso temporaneo assoluto deve essere indispensabile per ripristinare il backup.

Esempio puramente concettuale, **non contratto approvato**:

```text
inspect_external_revision(
  media_id, source_file_selected_by_user, expected_current_revision_id?
) -> {
  source_sha256, supported_format, pixel_size, byte_size,
  same_as_existing_revision?, concurrent_change?,
  impacted_active_usages, warnings
}

attach_external_revision(
  media_id, user_selected_source, expected_base_revision_id,
  source_sha256_confirmed, activate: false
) -> { new_revision_id, persisted_sha256, current_revision_id_unchanged }
```

L'eventuale attivazione dovrebbe essere una **seconda operazione** con verifica di ID/revisione e usi aggiornati. Evitare di inventare un'interfaccia di modifica ai certificati storici; le revisioni e i riferimenti già congelati restano intatti. Valutare la necessità di una migrazione additiva solo dopo aver dimostrato che i campi `kind`/`operations_json` esistenti non bastano: `operations_json` dell'editor interno **non** deve fingersi una ricetta per Photoshop o GIMP. La prima fase potrebbe evitare uno schema aggiuntivo.

## 5. Offline, sicurezza, privacy, impostazioni e lingue

- **Offline-first reale:** file scelti localmente, nessun servizio remoto, account Repertica, abbonamento, upload, telemetria o chiamata all'editor esterno. Non è necessario rilevare la presenza di programmi. Utenti liberi di scegliere qualsiasi editor o limitarsi al Media Center interno.
- **Integrità:** convalida di tipo *effettivo* e limiti di decodifica/memoria; controllo di file regolari e rifiuto dei symlink in ogni punto sensibile; staging privato sicuro e hash ricontrollato al commit per ridurre i rischi TOCTOU; pubblicazione atomica dei media con pulizia di errori; transazione SQLite coerente, senza produrre riferimenti a file assenti; revisione di base e identità del media ricontrollate prima della scrittura. È un insieme di criteri di sicurezza per una **eventuale** implementazione, non una descrizione di protezioni già esistenti per questo futuro percorso.
- **Privacy:** i file esterni possono contenere GPS, profili personali, crediti, nomi di lavoro e identificativi; il rientro archivia i byte originali selezionati, non promette rimozione di tali dati. Nessun percorso esterno o nome di cliente nei log; non trasmettere a fornitori IA. I backup esistenti sono in chiaro.
- **Impostazioni:** nessuna opzione globale obbligatoria; se richiesto dai test, un semplice interruttore facoltativo «Mostra strumenti avanzati per fotografie» sarebbe locale. La cartella di export resta libera: nessun lock-in verso percorsi gestiti dal software.
- **Localizzazione/accessibilità:** lingua di sistema con fallback inglese, scelta manuale già prevista in Repertica; etichette ed errori traducibili IT/EN/ES/FR/DE senza tradurre gli ID macchina; tabulazione logica, focus stabile, etichette screen reader, confronto non basato soltanto sul colore, pulsanti «Annulla» e «Conferma» inequivocabili. Nessun testo hardcoded nel flusso finale.
- **Portabilità:** media nuovi conservati nel percorso relativo interno e inclusi nei backup verificati; eventuali metadati di provenienza futuri esportabili in formati aperti. Esportare i file come JPEG/PNG/WebP supportati, senza obbligo di conservare la sessione in un formato di progetto proprietario.

## 6. Dipendenze, compatibilità, alternative e rischi

**Dipendenze proposte solo concettuali:** infrastruttura attuale Tauri/Rust/SQLite, modello revisioni, validazione media, dialoghi file nativi e backup. Nessuna libreria esterna, servizio web, integrazione specifica con Adobe o API di desktop automation va introdotta senza decisione del proprietario. Compatibilità da verificare su Windows/macOS/Linux e dischi USB/NAS; rispettare gli attuali limiti su formato, memoria, dimensioni e archivio attivo.

**Alternativa zero-sviluppo prioritaria:** esportare dalla libreria, modificare con editor esterno, quindi **importare il risultato come nuovo elemento** del Media Center e assegnare un nome o tag descrittivo. Con 3–5 professionisti confrontare il costo cognitivo di questa soluzione con il prototipo del collegamento come revisione. Se il flusso attuale basta, **non sviluppare** questa proposta.

Altre alternative: semplice link manuale nelle note di una fotografia (minimo ma fragile), archiviazione delle versioni in cartelle esterne (fuori dal backup autosufficiente), attivazione automatica del nuovo file (sconsigliata), automazione tramite watchers/plugin (sconsigliata per consenso, sicurezza e portabilità).

**Rischi:** duplicazione dei byte, crescita dei backup, foto rientrata associata al media sbagliato, modifica involontaria dei riferimenti condivisi, divergenza cromatica (TECH-021), supporto parziale ai formati esterni, file sostituito fra selezione e lettura, collisioni e percorsi di rete, falsa impressione che l'origine del file sia verificata, e UI troppo complessa per l'artista occasionale. Nessun automatismo può confermare autorialità, autenticità o diritti.

**Complessità ipotetica:** **media** per aggiunta singola non attiva con controlli e persistente; **medio-alta** per confronto visivo, controlli di concorrenza e attivazione sicura con più utilizzi; **alta** per integrazioni avanzate o nuovi codec, che non sono proposti per la prima fase.

## 7. Criteri di accettazione e prove da fare solo dopo eventuale approvazione

1. Confronto preliminare con **4–6 fotografi/artisti/archivisti**, su tre file di prova e con l'export+import attuale. Annotare quante volte desiderano mantenere una relazione di revisione e quanti errori commettono; non assumere una domanda senza evidenza.
2. Un JPEG/PNG/WebP valido selezionato dall'utente può essere aggiunto a **un `media_id` esistente** come nuova revisione; UUID stabile, file nuovo, hash e dimensioni coerenti, file originale invariato.
3. L'aggiunta non cambia `current_revision_id`, percorsi in schede, foto profilo o certificati: confronto degli stati prima/dopo, inclusi media condivisi.
4. La futura attivazione, **se scelta separatamente**, presenta il numero e tipo di utilizzi, rifiuta una selezione obsoleta e non altera certificati precedenti.
5. File corrotto, enorme, non supportato, symlink, permessi insufficienti, disco rimosso, crash in staging o stato concorrente: nessuna revisione visibile incompleta, nessun file esterno toccato; pulizia dei temporanei verificabile.
6. Backup→verifica→ripristino su altro computer: la nuova revisione è disponibile offline senza i file dell'editor esterno; il file originale e le revisioni precedenti restano leggibili.
7. Accessibilità da tastiera e screen reader, stringhe localizzate in 5 lingue/fallback inglese, percorsi di filesystem con Unicode; prova nativa Windows/macOS/Linux e almeno un supporto rimovibile.
8. Nessun invio di dati o avvio di editor da parte di Repertica; nessuna nuova dipendenza di rete; nessuna alterazione dei CSV esistenti o del comportamento di export.

Questi sono **test ipotetici**, non test eseguiti né accettazione del prodotto.

## 8. Priorità condizionata e fasi opzionali

- **Fase 0 — nessun codice:** testare i flussi esistenti di export+import con professionisti e documentare decisione *procedere/non procedere*.
- **Fase 1 — soltanto se utile:** specifica confermata per aggiungere **una** revisione da file esterno senza attivarla; trasparenza di origine dichiarata; nessun editor integrato nuovo.
- **Fase 2 — soltanto se richiesta:** avviso degli utilizzi effettivi e attivazione separata, con controllo delle modifiche concorrenti.
- **Fase 3 — non implicitamente autorizzata:** profili avanzati, batch, sidecar o round-trip semiautomatici: solo un'eventuale decisione successiva, e preferibilmente riuso di TECH-021/024/053.

**Priorità proposta P3** perché la libreria contiene già editor, esportazione e importazione. **P2 condizionata** a casi ripetuti in cui l'archivio perde davvero un legame importante tra versioni, e alla possibilità di assicurare un ripristino coerente. **Nessuna implementazione** se i flussi attuali sono sufficienti.

## 9. Questioni aperte per revisione del proprietario

1. Quanti utenti di Repertica lavorano davvero fotografie con programmi esterni, e per quali casi l'editor interno non basta?
2. La **sola aggiunta come revisione non attiva** è un valore professionale autonomo, o gli utenti chiederebbero inevitabilmente l'attivazione?
3. Una fotografia usata contemporaneamente su più opere o contatti deve essere modificabile globalmente con avviso, o è preferibile richiedere sempre un nuovo elemento separato?
4. Bastano JPEG/PNG/WebP per un primo scenario verificato, senza PSD/XCF/TIFF e senza promettere la fedeltà colore?
5. Serve conservare un riferimento informativo alla revisione di base, distinguendolo da una garanzia verificata di derivazione?
6. Qual è la soglia accettabile di aumento della dimensione del backup quando le revisioni esterne sono numerose?
7. Prima di progettare UI: un semplice tutorial «esporta, modifica, importa come nuova fotografia» risolve già il problema?

**Conclusione:** ipotesi esclusivamente documentale. **Stato: Proposta — da valutare.** Nessuna idea è approvata, assegnata o autorizzata allo sviluppo.
