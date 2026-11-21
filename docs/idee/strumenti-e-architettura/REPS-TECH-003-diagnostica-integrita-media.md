# REPS-TECH-003 — Diagnostica integrità dei media nell'archivio attivo e recupero assistito

- **Data:** 2026-11-21
- **Stato:** **Proposta — da valutare**
- **Priorità proposta:** **P2**, dopo collaudo dei backup esistenti.
- **Ambito:** salute e recuperabilità dei file dell'archivio **attivo**. Non è un nuovo backup, una nuova scheda di provenienza o gestione prestiti.

## Problema concreto e confini

Nel [MVP](../../MVP.md) ogni opera conserva una foto principale locale e, dopo la [PR #17 integrata in `main`](https://github.com/Cesare-ml/Repertica-Studio/pull/17), fino a 20 fotografie aggiuntive (JPEG/PNG/WebP, massimo **12 MiB** ciascuna). Il codice del [backup](../../../src-tauri/src/backup.rs) già genera manifest SHA-256 e verifica consistenza dei file **nel backup**, oltre a integrità SQLite. Non risulta un pannello di **diagnostica preventiva dell'archivio attivo**, con elenco di file mancanti, file senza riferimenti e percorso di recupero assistito. Un fotografo o una galleria potrebbero scoprire troppo tardi che alcune immagini locali sono state rimosse accidentalmente.

**Destinatari:** artisti con migliaia di opere, gallerie e archivi con media che occupano molto spazio o che migrano computer. **Non** riguarda la veridicità storica o legale dei documenti di provenienza (filone [UX-001, PR #14](https://github.com/Cesare-ml/Repertica-Studio/pull/14)).

## Esperienza proposta e scenari d'uso

1. **Diagnostica non distruttiva:** Impostazioni → Salute archivio → «Analizza media». Riepilogo cartella locale, numero file referenziati e dimensione stimata, richiesta di avvio scansione, barra avanzamento e **Annulla**.
2. **Report per categoria:** a) file valido e raggiungibile; b) riferimento presente in SQLite ma file mancante; c) file presente ma non leggibile/corrotto; d) file presente nella cartella media senza alcun riferimento («orfano»); e) stesso contenuto su più file. Non qualificare automaticamente un orfano come cancellabile o due foto identiche come errori.
3. **Proposta di recupero per singola voce:** «Trova il file» con dialogo nativo su cartella scelta. Verifica dimensione, formato e hash atteso *se disponibile*. Mostrare anteprima e chiedere conferma prima di copiare e aggiornare il riferimento. Se hash storico non esiste, dichiarare **«identità non verificabile automaticamente»** e richiedere approvazione esplicita.
4. **Confronto con backup scelto:** opzionalmente cercare un media mancante in un backup già esistente indicato dall'utente, **senza** invocare un ripristino globale. Prima validare il manifest del backup e la corrispondenza del file. Mai modificare una cartella di backup.
5. **Solo report per file orfani:** elenco, dimensione, data/modifica e possibile origine; consentire **esportazione di un report locale**. Nessuna cancellazione, consolidamento, deduplica o ricompressione automatica.

## Disegno dati e ipotesi di API, soltanto descrittive

- Nel modello corrente `Artwork.image_path` e `artwork_images.image_path` sono percorsi locali relativi, rispettivamente per foto principale e galleria; la proposta potrebbe introdurre, **solo se approvata**, un `media_id` stabile, `relative_path`, `bytes`, `sha256`, `verified_at` e `origin`. Distinguere hash misurato *ora* da hash attestato al momento dell'acquisizione: il primo **non prova** che la foto sia quella corretta.
- Per non riscrivere subito lo schema, una **fase pilota** può produrre report effimeri dal confronto tra `image_path` e nomi nella cartella `media/`. Hash storici e associazioni multiple richiedono una migrazione progettata; nessuna inferenza automatica dei dati pregressi.
- **API concettuali**, non funzioni implementate: `scanActiveMedia(snapshot, budget, cancelToken) -> HealthReport`; `inspectCandidate(path, expectedHash?) -> MatchReport`; `planRelink(artworkId, candidate) -> Plan`; `confirmRelink(plan) -> Result`.
- `HealthReport` deve conservare ID e codici delle opere, codici errore, checksum disponibili e timestamp, evitando un catalogo permanente di percorsi esterni con dati identificanti.

## Offline, sicurezza, prestazioni e compatibilità

- Nessuna rete, account, SaaS o upload di immagini; accesso ai file solo tramite cartelle consentite dall'utente. L'interfaccia usa la lingua del sistema, fallback inglese; report con terminologia chiara («manca il file», non «hash rotto»), navigabile da tastiera e screen reader.
- Scansione in **streaming** a blocchi per non caricare migliaia di foto in RAM; stabilire budget di I/O configurabili, sospensione, annullamento, attenzione a USB/NAS lenti e processo in background **solo mentre Repertica è aperta** (non servizio occulto).
- Verifiche di percorsi relativi, canonicalizzazione, rifiuto di symlink/path traversal e file speciali. Evitare race fra scansione, modifica e backup; un report deve essere marcato «potenzialmente non aggiornato» se i dati cambiano.
- Evitare che «ripara» sostituisca un originale: prima copia di sicurezza e validazione a staging, verifica post-copia, rollback se fallisce l'aggiornamento DB. No cancellazione del file sorgente e no riscrittura di certificati immutabili.
- La diagnostica **non garantisce** la qualità o genuinità artistica dei contenuti; SHA-256 misura solo uguaglianza dei byte rispetto a un hash disponibile e attendibile.
- Non assumere un generico «file store esterno con collegamenti assoluti»: per ora il backup include la sottocartella `media/` e i link esterni porrebbero rischi seri di portabilità. Eventuali media esterni richiedono un'altra decisione architetturale.
- Le foto aggiuntive della galleria sono già presenti in `main`; ricevute e documenti futuri ([issue #13](https://github.com/Cesare-ml/Repertica-Studio/issues/13)) possono riutilizzare lo strato tecnico; questa scheda **non** introduce quel dominio funzionale.

## Alternative, difficoltà e rischi

- **Alternativa minima:** controllare soltanto durante la creazione del backup (già presente), senza pannello separato; semplice ma non fornisce diagnosi prima di selezionare una cartella di backup o recupero puntuale.
- **Alternativa esterna:** script manuali di controllo file, poco accessibili, rischiosi per i non tecnici e non integrati con gli UUID delle opere.
- **Rischi:** lettura lenta su archivi grandi, hash di un file mutato mentre lo si legge, false associazioni per nome, report troppo dettagliati con dati di terzi, falsa percezione di backup sostitutivo, sovrascritture accidentali. Mitigare con scansioni read-only, hash/size, anteprime, report non autoritativi e interventi espliciti.
- **Complessità:** **media** per diagnosi read-only; **medio-alta** per recupero puntuale robusto; **alta** per versioning e deduplica reale multi-allegato.

## Criteri di accettazione e test di progetto

1. Una scansione su un archivio con **100.000 riferimenti sintetici** termina senza caricare tutti i media in RAM; interruzione/riavvio non lascia transazioni né scrive nei dati.
2. File mancante, non leggibile, orfano, nome Unicode, hash differente e directory NAS scollegata sono classificati distintamente, con report e stati UI localizzati.
3. Uno SHA-256 misurato su media senza hash storico è dichiarato **misurazione attuale**, non prova di originalità; proposte di riparazione non fanno merge automatico per titolo/nome di file.
4. Tentativi con `../`, symlink e tipi speciali non consentono lettura fuori dal perimetro scelto; non vengono aperte porte di rete.
5. Recupero da backup valido copia un solo file e **non** ripristina l'intero database; backup danneggiato o nuovo file incoerente impedisce l'azione e lascia invariato l'archivio attivo.
6. Qualsiasi elemento classificato «orfano» resta sul disco. Una nuova scansione su dati immutati restituisce conteggi coerenti; tastiera, screen reader, progresso e annullamento funzionano.

## Stato delle informazioni

**Certo dal repository (ricontrollato il 2026-11-21):** foto principale e fino a 20 aggiuntive locali fino a 12 MiB ciascuna; hash SHA-256 nel manifest di backup; verifica prima del ripristino; memorizzazione SQLite dei percorsi relativi delle immagini. La collocazione media è locale.

**Ipotesi:** dashboard, schema `media_id`, hash all'acquisizione, report persistenti o effimeri, relink da backup e controlli periodici.

**Questioni aperte al proprietario:** interessa prima una sola scansione read-only? Quale volume di media è realistico per gli utenti? Una prova di identità deve essere impossibile senza hash d'acquisizione, oppure basta conferma manuale? È opportuno supportare media su dischi esterni senza includerli nei backup (scelta ad alto rischio)? La pulizia degli orfani va lasciata sempre manuale fuori dall'app?

## Fasi eventuali (non approvate)

1. **Diagnostica read-only** basata sullo schema presente, con report e categorie.
2. **Hash alla nuova acquisizione**, policy di retrocompatibilità e dati di integrità distinti dagli hash dei backup.
3. **Recupero guidato** da file selezionato o backup valido, con staging e nessuna modifica silenziosa.
4. Solo dopo decisione: struttura multi-media, versioning e possibile deduplica non distruttiva.

> Il documento non avvia interventi sull'archivio o sul software.
