# REPS-TECH-005 — Anteprime derivate e consultazione fluida di archivi fotografici grandi

- **Data ricognizione:** 2026-11-21
- **Stato:** **Proposta — da valutare**
- **Priorità proposta:** **P2**, subordinata a misure reali e revisione proprietario; non è un mandato operativo.
- **Categoria:** prestazioni, sostenibilità dello spazio locale, esperienza con fotografie di opere; nessun caricamento su server.

## Motivazione, destinatari e stato osservato

Repertica conserva le fotografie in `media/` dentro la cartella privata dell'app e permette immagini principali PNG/JPEG/WebP fino a 12 MiB ([README](../../../README.md), [commands.rs](../../../src-tauri/src/commands.rs)). Il comando di lettura corrente restituisce i dati dell'immagine in **base64**. I backup copiano i media integri e li verificano con SHA-256 ([backup.rs](../../../src-tauri/src/backup.rs)); **non** sono un sistema di anteprime. Per raccolte con migliaia di opere, leggere o decodificare originali interi per mostrare una griglia può aumentare memoria, attese e uso del disco.

La [PR #17](https://github.com/Cesare-ml/Repertica-Studio/pull/17) è stata **integrata in `main`**: la galleria di fino a 20 immagini aggiuntive e il caricamento su richiesta sono già disponibili. Questa scheda è **complementare**: non propone un'altra galleria né ripropone il suo lazy loading, ma considera cache di miniature derivate solo se giustificate da benchmark.

**Destinatari:** fotografi e artisti con più immagini per lavoro, studi con cataloghi estesi, gallerie con grandi inventari e utenti su laptop con spazio/memoria limitati. Validare con archivi sintetici e reali prima di introdurre nuovi livelli di cache.

**Confini:**
- [TECH-003](REPS-TECH-003-diagnostica-integrita-media.md) controlla **file mancanti, orfani e anomalie** e recupero guidato; non va duplicata. Qui si parla di **prestazioni** e file *derivati eliminabili*.
- I backup attuali devono rimanere **autosufficienti**: originali + DB + manifest. Nessuna anteprima derivata è necessaria per il ripristino e nessun backup deve dipendere dalla cache.
- L'eventuale funzionalità futura deve evitare compressioni o modifiche degli **originali** senza azione specifica dell'utente. Non è un servizio di hosting, un media cloud o un catalogo online.

## Scenari e flussi UX proposti

1. **Consultazione griglia:** l'utente apre 2.000 opere; l'elenco mostra progressivamente miniature locali di dimensione adeguata alla vista, senza decodificare migliaia di immagini originali. Il testo e i filtri rimangono utilizzabili anche se una miniatura manca o la generazione è ancora in corso.
2. **Dettaglio su richiesta:** aprendo una scheda o ingrandendo, il sistema carica una versione per la vista e soltanto quando richiesto l'originale. La miniatura non deve essere confusa con file d'archivio o materiale idoneo alla stampa; l'originale rimane accessibile.
3. **Primo avvio dopo migrazione:** nessuna elaborazione massiva obbligatoria. Miniature prodotte progressivamente su richiesta, con possibile operazione manuale «Prepara anteprime» accompagnata da spazio stimato, avanzamento, pausa e interruzione. Nessun job nascosto con consumo elevato.
4. **Supporti lenti o scollegati:** la griglia resta navigabile e segnala chiaramente foto non disponibili; non mostra una vecchia miniatura come prova che il file originale esista ancora. Se una fonte non è disponibile, non tenta ricerche su altri volumi senza consenso.
5. **Gestione spazio:** Impostazioni → Media e prestazioni → occupazione delle anteprime, limite massimo per cache, pulsante «Svuota **solo le anteprime**» e messaggio «Non verranno eliminate le fotografie originali». Prima dell'operazione, anteprima del risparmio possibile.
6. **Cambio fotografia/ritocco/rotazione:** la vista invalida l'anteprima corrispondente, la rigenera quando necessario e non modifica silenziosamente l'originale. Se il file è corrotto o non decodificabile, mostra errore non distruttivo, rimandando alla futura diagnostica TECH-003.

## Ipotesi di modello dati e API (solo descrizioni)

**Separazione di responsabilità:**
- **Archivio autorevole:** database SQLite, record delle immagini e file originali nel media store già esistente. Gli UUID delle opere restano stabili.
- **Cache derivata ricostruibile:** file miniature con chiave logica `media_id / media_relative_path`, `original_fingerprint` (eventuale hash + dimensione + mtime per controllo veloce), `variant` (dimensioni/codec/profilo), `transform_version`, `created_at`, `last_accessed`, `bytes`. Nessuna cache è fonte autorevole del catalogo.
- **Manifest cache locale:** può essere una piccola tabella indice o struttura privata separata dal DB dell'archivio; evitare di contaminare esportazioni, backup e migrazioni con file derivati. Valutare se è più semplice l'invalidazione deterministica senza database cache.
- **Varianti minime:** miniatura per griglia e anteprima per dettaglio; dimensioni e formato da definire tramite benchmark, non prescritti qui. Conservare orientamento, ritaglio corretto e proporzioni; applicare controlli a immagini di grandi dimensioni o malformate.

API **ipotetiche**: `requestPreview(media_ref, variant) -> Pending|Ready|Unavailable`; `invalidatePreview(media_ref) -> Outcome`; `inspectCache() -> CacheSummary`; `pruneCache(quota, policy) -> Report`; `rebuildPreviews(scope, cancellation) -> Progress`. Queste non sono comandi Tauri approvati né endpoint MCP.

**Niente deduplicazione automatica degli originali**: due opere possono condividere fotografie intenzionalmente; rimuovere o unificare i file originali cambia implicazioni del backup, cancellazione e provenienza. Valutare deduplicazione solo con un progetto futuro e separato, verifiche esplicite e riferimenti multipli sicuri.

## Offline, sicurezza, privacy, compatibilità, impostazioni e lingue

- Tutte le anteprime sono calcolate **sul dispositivo**, senza CDN, browser remoto, chiamate a servizi di riconoscimento immagini, account, upload o telemetria. Non dipendono da Internet, modelli IA o codec proprietari obbligatori.
- Non accettare percorso arbitrario dalle schede: risoluzione limitata a media autorizzati, contrasto a `../`, symlink e traversal, validazione dei magic bytes oltre all'estensione, limiti a decompression bomb, pixelloni, memoria/CPU e concorrenza. I file di cache devono essere scritti in cartelle app-owned con nomi generati e pubblicazione atomica.
- **Privacy:** le anteprime possono ritrarre persone, firme o documenti. Trattarle come dati personali; app-local-data protetta dal profilo OS, possibilità di rimozione, nessuna inclusione in log, report o pacchetti di export predefiniti. Considerare rimozione dei metadati EXIF/GPS **dai derivati** (non dagli originali). L'eliminazione della cache non deve simulare eliminazione degli originali.
- File originali e cache restano locali anche su USB/NAS montato se in futuro l'archivio si sposta; percorso della cache configurabile solo se necessario, evitando che utenti la confondano con destinazione di backup. Se si consente un percorso diverso, deve essere scelto esplicitamente e mai dentro la cartella di backup.
- Compatibilità: supportare schemi v1–v4 e media già esistenti, inclusa la galleria integrata dalla PR #17, senza introdurre ora schema o limiti nuovi. Non alterare hash, percorsi e manifest dei backup, né lo snapshot immutabile dei certificati. Le copie derivate devono poter essere cancellate, ricostruite e ignorate dopo aggiornamenti dell'app.
- UI in lingua di sistema con fallback inglese e scelta manuale già presente; messaggi sulle distinzioni **originale/anteprima**, peso cache, avanzamento e errori localizzati. Accessibilità: non basare la navigazione su sole immagini; testi alternativi derivati dai metadati dell'opera senza IA, indicatori di caricamento annunciabili, pulsanti da tastiera, opzione di riduzione dell'animazione e nessun salto del focus.

## Import/export e rapporti con altri strumenti

- **Backup/ripristino:** include solo originali e riferimenti autorevoli; dopo ripristino le anteprime vengono generate nuovamente. Il recupero non dipende dalla presenza della cache e non aumenta la dimensione del backup per colpa delle miniature.
- **CSV e pacchetto TECH-001:** esportano dati e, ove previsto, **originali**, non copie derivate spacciate per master. Un eventuale catalogo di immagini leggero per la stampa/condivisione sarebbe un'altra funzionalità con consenso esplicito e non è definito qui.
- Nessun formato cache proprietario che diventi prerequisito per usare l'archivio. Ogni original file deve restare accessibile con strumenti comuni e mai essere criptato o sostituito automaticamente.

## Alternative, dipendenze, rischi e complessità

**Alternative:** (A) soltanto lazy loading degli originali, come già disponibile dopo la PR #17: meno manutenzione e forse sufficiente finché gli archivi rimangono piccoli; (B) miniature soltanto in memoria: semplici ma ricreate dopo ogni avvio e rischiose per RAM; (C) miniature persistenti e quota LRU (questa proposta) con maggiore complessità e privacy. **Non** introdurre la C senza benchmark che mostrino un vantaggio netto.

**Dipendenze ipotetiche:** pipeline sicura di decoding immagini lato Rust/Tauri o altro componente locale verificato, lettura media on demand, identità stabili dei media, algoritmi di invalidazione e raccolta cache, UI non bloccante, test multi-piattaforma. Non aggiungere librerie prima di scelta esplicita. Rispettare le licenze dei codec e dei componenti open source.

**Rischi:** miniatura non aggiornata, foto mancante mascherata, cache in crescita incontrollata, immagini personali rimaste dopo cancellazione, decompression bomb, consumo energetico su laptop, complicazioni su archivi rimovibili. Mitigazioni: validazione iniziale, quota e pulizia sicura, etichette di stato, priorità alle anteprime visibili, concorrenza limitata, revoca/pulizia, logging non sensibile.

**Complessità:** **media** per una miniatura dell'immagine principale, **medio-alta** per gallerie/varianti/quote/dispositivi lenti e compatibilità cross-platform. Il beneficio deve essere dimostrato.

## Criteri di accettazione e prove future

1. Benchmark **ripetibile** su archivio sintetico con 100, 2.000 e 20.000 opere e media di dimensione/risoluzione variabile: confrontare memoria massima, tempo di prima visualizzazione, scroll e dimensione cache rispetto al lazy loading senza cache. Le **soglie numeriche** vanno concordate prima di scegliere l'implementazione.
2. Dopo generazione anteprime, hash degli originali e riferimenti DB invariati. Backup creato/ripristinato senza anteprime mantiene immagini e certificati; cache rigenerabile completamente offline.
3. Sostituzione di un originale invalida la relativa anteprima senza mostrare a lungo quella vecchia; eliminazione di un'opera non lascia in cache copie indebitamente accessibili e non altera file di altre opere.
4. Interruzione, errore di decodifica, disco pieno, mancanza permessi, USB/NAS disconnesso e riavvio improvviso non bloccano consultazione, non danneggiano il DB né eliminano originali.
5. Cache limitata alla quota e svuotabile da UI; indicatore «solo anteprime» verificabile con test. Un'immagine avvelenata non provoca esecuzione di codice né letture fuori dai percorsi autorizzati.
6. Test Windows/macOS/Linux su schermi HiDPI e a scala standard, navigazione da tastiera e screen reader, fuso/lingua OS e fallback inglese; nessun invio dati via rete.

## Certezze, ipotesi e domande di revisione

**Certo dal `main`, ricontrollato il 2026-11-21:** foto principale e fino a 20 aggiuntive per opera, media locali con limite 12 MiB a immagine, lettura via base64 su richiesta, backup con checksum, archiviazione SQLite e UI di ricerca e filtri. La PR #17 è stata **integrata** e non va confusa con la proposta, ancora non approvata, di una cache di miniature persistenti. La diagnostica integrità è già oggetto di TECH-003.

**Ipotesi:** carichi abbastanza grandi da giustificare miniature persistenti, due varianti, quota configurabile e pipeline di generazione. Non è stata misurata oggi la lentezza reale.

**Domande al proprietario:** quanti lavori e quante foto per opera sono realistici? Esistono foto ad altissima risoluzione fuori dal limite di 12 MiB? Preferire generazione su richiesta o preparazione manuale prima di una mostra? Quale massimo di spazio disco è accettabile per cache? Le foto potranno risiedere su unità esterne senza essere copiate nell'app? Serve una cache condivisa fra archivi o è meglio tenerle rigorosamente separate?

## Fasi proposte per valutazione, non sviluppo autorizzato

1. Raccogliere casi reali, dati di prestazione e budget memoria/disco.
2. Prototipare e confrontare **solo su carta e benchmark dedicati se approvati**: lazy loading originale vs due livelli di anteprima.
3. Se il proprietario lo autorizzerà, definire formato delle copie derivate, privacy e test di integrità prima di eventuale codice.
4. Solo in seguito, valutare cache persistente per immagini aggiuntive e gestione su storage esterno.

> Nessuna funzionalità viene approvata, implementata o assegnata con questa scheda.
