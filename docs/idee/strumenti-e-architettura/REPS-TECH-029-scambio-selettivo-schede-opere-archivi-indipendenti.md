# REPS-TECH-029 — Scambio selettivo di schede-opera fra archivi indipendenti, senza sincronizzazione

- **ID stabile:** REPS-TECH-029
- **Data:** 2026-11-22
- **Stato:** **Proposta — da valutare**
- **Filone:** Strumenti e architettura — esclusivamente analisi e documentazione
- **Priorità ipotetica:** **P2–P3**, soltanto se artisti e gallerie confermano scambi ripetuti; nessuna approvazione o assegnazione allo sviluppo.
- **Oggetto:** un **pacchetto locale, aperto e circoscritto** per consegnare una o più schede di opere e media deliberatamente selezionati, e un'eventuale **ispezione/importazione assistita** in un archivio Repertica già popolato. Non è un backup, un certificato, una vendita o un sistema di sincronizzazione.

## 1. Motivazione, destinatari e riscontri verificati

**Scenario professionale da convalidare:** un artista cataloga un'opera nel proprio Repertica e deve fornire a una galleria titolo, tecnica, dimensioni e fotografie autorizzate; la galleria desidera usare quelle informazioni nel proprio archivio locale, senza riscriverle a mano, né ricevere clienti, incassi, ubicazioni private o l'intero database. Analoga necessità può presentarsi per prestiti a curatori o consegne ad archivisti. In un altro caso, una galleria trasferisce le informazioni documentali di una scheda a un conservatore, ma non può promettere autenticità, proprietà o diritto di riproduzione soltanto tramite file.

**Destinatari:** artisti indipendenti, galleristi, registrars, archivisti, curatori che lavorano con archivi separati e non vogliono dipendere da un servizio online. Per chi mantiene un unico archivio o scambia raramente opere, PDF e CSV già disponibili potrebbero bastare.

**Ricognizione sul branch `main`, 22/11/2026:** [README](../../../README.md), [MVP](../../MVP.md), [DATA-PORTABILITY](../../DATA-PORTABILITY.md), [models.ts](../../../src/app/models.ts), [gallery.rs](../../../src-tauri/src/gallery.rs), [media.rs](../../../src-tauri/src/media.rs), [raw_files.rs](../../../src-tauri/src/raw_files.rs), [backup.rs](../../../src-tauri/src/backup.rs) e [MCP.md](../../MCP.md). Attualmente:
- le opere hanno UUID, codice archivio, descrizione, misure, stato e foto principale; la galleria ha foto aggiuntive, categoria, didascalia e ordine; Media Center conserva revisioni e metadati delle immagini;
- i RAW possono essere allegati con hash e dimensione; i media di un archivio sono gestiti localmente;
- i **12 CSV** sono registri **integrali** senza byte dei file e non costituiscono un importatore transazionale; il backup completo contiene SQLite e media, verifica SHA-256 e al ripristino **sostituisce** l'archivio;
- esistono brochure e copie di fotografie, ma dai percorsi esaminati non risulta un interscambio strutturato **di singole schede con i rispettivi media e una scelta consapevole dei campi da condividere**, importabile in un archivio non vuoto.

Quest'ultima osservazione è limitata ai file consultati: **nessun test runtime, intervista utente o problema di perdita dati è stato condotto o dimostrato**.

## 2. Confini e controllo delle duplicazioni

- **[TECH-001](REPS-TECH-001-pacchetto-dati-aperto.md)**: contratto generale di *portabilità completa* e reimportazione in archivio **vuoto**, con un eventuale merge generale lasciato al futuro. **Qui** si valuta soltanto la **consegna intenzionale di un sottoinsieme di opere fra soggetti/archivi indipendenti**, con minimizzazione di campi/media, provenienza dichiarata e ispezione senza sostituzione del database di destinazione. Se TECH-001 venisse approvata, riutilizzare obbligatoriamente convenzioni, parser e formato aperto; non mantenere due protocolli incompatibili.
- **[TECH-025](REPS-TECH-025-esportazioni-tabellari-minimizzate-condivisione.md)**: CSV mirati per leggere/analizzare dati, **non** trasferimento verificabile di foto e relazioni. **[TECH-013](REPS-TECH-013-copie-fotografiche-condivisibili-privacy-metadati.md)**: pulizia dei metadati di foto derivate, che rimane un problema separato e un prerequisito eventuale.
- **[TECH-004](REPS-TECH-004-ricerca-federata-archivi-locali.md) / [TECH-011](REPS-TECH-011-profili-multiarchivio-apertura-sicura.md)**: consultare o aprire più archivi, senza trasferire singoli record. **[TECH-019](REPS-TECH-019-importazione-guidata-cataloghi-tabellari.md)** importa CSV esterni con colonne mappate, non pacchetti Repertica con foto e riferimenti.
- **Filone Gestione e UX, [PR #14](https://github.com/Cesare-ml/Repertica-Studio/pull/14):** UX-001 provenienza e movimentazioni, UX-007 dossier di presentazione, UX-012 conto vendita, UX-021 acquisizione da terzi, UX-023 listini e UX-031 autorialità trattano significato, titoli, diritti e processi gestionali. Il pacchetto proposto **non** produce quelle attestazioni né modifica i relativi registri. UX-018 catalogazione seriale e UX-020 etichette non sono un import di schede ricevute. Nessun documento dell'altro filone è modificato.
- **Esclusi:** trasferimento di proprietà, stock, vendite, clienti, prezzi, percentuali, saldi, certificati storici, diritti e licenze impliciti; sincronizzazione bidirezionale, collaborazione live, account, invio email/upload automatico, IA, accesso MCP al pacchetto, riconoscimento automatico di «stessa opera» da foto o titolo.

## 3. Casi d'uso e flussi UX descrittivi

1. **Preparazione da parte dell'artista:** seleziona 1–N opere dalla ricerca locale → «Prepara pacchetto per altro archivio» → anteprima dell'elenco e di **tutte** le categorie di dati/byte. Default: codice sorgente come riferimento testuale, titolo, anno, tecnica, supporto, dimensioni e descrizione; **esclusi** ubicazione, stato commerciale, riferimenti a clienti/vendite/certificati, note private e fotografie. Nessun dato è condiviso finché non sceglie deliberatamente la destinazione.
2. **Fotografie:** per ogni opera si possono includere **copie esplicitamente selezionate** della copertina e della galleria, con tipo/didascalia solo se autorizzati. Distinguere **originale**, **revisione del Media Center** e **derivato condivisibile**; mostrare dimensione, metadati potenzialmente privati e crediti/dichiarazioni di diritti. Le fotografie **non** sono automaticamente liberate per terzi. RAW originali esclusi per default (file potenzialmente molto grandi e sensibili), eventualmente valutabili come opzione separata a forte conferma. Se la copia sicura di TECH-013 non esiste, dichiarare che non si possono garantire metadati rimossi.
3. **Esportazione:** scegliere cartella locale/USB/NAS montato → controllo spazio e riepilogo privacy → creare nuova sottocartella verificabile; nessuna sovrascrittura, nessuna rete, nessun invio automatico. Possibile annullamento con cleanup dei soli file temporanei creati dall'app; il file originale di ogni foto resta inalterato.
4. **Ricezione e verifica:** destinatario sceglie manualmente il pacchetto → vista read-only con mittente *dichiarato*, data, versione, inventario, SHA-256, anteprime autorizzate, campi assenti e avviso «dati forniti da terzi, non verificati». Anche un pacchetto formalmente integro **non prova** provenienza/autenticità.
5. **Eventuale importazione futura, solo dopo specifica e approvazione:** in un archivio **già popolato**, il destinatario confronta ogni opera con possibili corrispondenze locali usando codice/titolo/media esclusivamente come **segnali**, non matching certo. Scelte per scheda: «Ignora» oppure «Crea nuova scheda» con **nuovo UUID e codice secondo la regola locale**. Niente sovrascrittura o merge di opere esistenti nella prima versione. Scegliere manualmente lo stato adeguato nel contesto destinatario soltanto dopo verifica della propria legittimazione; la ricezione **non** significa disponibilità commerciale o possesso fisico. Se ciò non può essere espresso correttamente dal modello attuale, rinviare l'import attivo e offrire soltanto consultazione.
6. **Conflitti e ripetizioni:** pacchetto già acquisito, hash di foto già presenti, stessi titoli, codice/UUID uguali o scheda candidata duplicata → avviso con possibilità di annullare/ignorare; **nessuna unione automatica**, nessun recupero dati commerciali dal mittente. Errori mostrati per record e il commit non deve lasciare mezze schede.

**Accessibilità:** riepilogo su più passi con focus controllato, navigazione da tastiera e screen reader, check espliciti per condivisione di immagini, annunci di progressi/errori, niente stati solo a colore, dialoghi ridimensionabili.

## 4. Formato aperto e strutture dati ipotetiche, NON API implementate

Preferire un sotto-profilo **compatibile con TECH-001** da concordare prima di qualsiasi codice, non un formato proprietario aggiuntivo. Una rappresentazione concettuale:

```text
Repertica-artworks-exchange-<exchange-id>/
  manifest.json            # profilo/formato/versione, UUID pacchetto, data, scope, hash e byte
  README.md                # interpretazione, licenze dichiarate, limiti, assenze
  data/artworks.jsonl      # ID+codice di origine, dati scelti, null distinti
  data/media-links.jsonl   # ID media e tipo uso, ordine e didascalia selezionati
  media/<safe-name>        # SOLO copie selezionate, SHA-256
```

Campi *ipotetici* del manifest: `format_id`, `format_version`, `profile="artwork-selective"`, `exchange_id`, `created_at_utc`, `origin_label_claimed` (opzionale, **non identità certificata**), `scope_manifest`, `items[] {path, bytes, sha256}`, `omitted_categories`, `tool_version`. Non includere percorsi locali, credenziali o identità/personali del mittente senza consenso.

Per opera: `source_artwork_id` (UUID sorgente, **non** primario in destinazione), `source_code`, `title`, `year`, `technique`, `support`, dimensioni in cm con valore numerico o null (mai conversione tacita), `description`, `source_updated_at` solo informativo. Evitare `location`, `status`, `collector_id`, `sale_id`, `certificate_id`, `price` e quote. File: nome relativo, tipo `cover/gallery`, `photo_kind`, `sort_order`, `caption`, `sha256`, `media_variant=selected_copy/original`; nessun percorso assoluto o URL obbligatorio. Un pacchetto parziale **non deve mai dichiararsi backup o archivio completo**.

Possibili operazioni puramente concettuali:
`inspectArtworkExchangeScope(selection) -> DisclosurePreview`;
`createArtworkExchange(selection, approved_fields, approved_media, destination) -> VerifiedPackage`;
`inspectIncomingArtworkPackage(folder) -> ReadOnlyImportReport`;
`planIncomingArtworkCreation(package, selected_ids, target_archive) -> CandidatePlan`;
`commitIncomingArtworkCreation(plan, explicit_authorization) -> ImportReport`.
Il piano si invalida se cambia il manifest, un file o lo stato rilevante del database. **Nessun endpoint HTTP/MCP.**

**Identità e anti-duplicazione:** `exchange_id + source_artwork_id` può essere memorizzato **solo se approvato** come riferimento di importazione distinto dall'UUID e dal codice dell'opera locale; non prova che due schede rappresentino lo stesso oggetto. Secondo invio dello stesso pacchetto non deve creare automaticamente un'altra opera: richiedere consultazione esplicita. La conservazione del riferimento all'origine è traccia di trasmissione, non autenticazione dell'autore o provenienza legale.

## 5. Offline, privacy, sicurezza, impostazioni e multilingua

- **Offline-first integrale:** nessun account, abbonamento, cloud, telemetria, scansione di cartelle o LAN; per trasferire file fra PC l'utente usa liberamente supporti USB/NAS già montati o qualunque mezzo esterno sotto il suo controllo. Niente autorizzazione di rete e nessuna configurazione di credenziali.
- **Privacy per impostazione predefinita:** nessun cliente/contatto o storico economico; immagini solo su scelta deliberata e con avvertenza su GPS/EXIF/IPTC, volto, firma, interni/ubicazione e diritto di terzi. La protezione dei metadati delle copie, se desiderata, va specificata separatamente. Le informazioni escluse **non** finiscono nel README, manifest, nomi di file, cache, errori o log.
- **Sicurezza di import:** limiti su quantità, bytes, dimensioni di decompressione, profondità e tempo; file regolari soltanto, niente symlink/hardlink speciali o path traversal, controllo checksum e tipo, parser JSON limitato, gestione di omografie Unicode e case-folding, revisione dei decoder immagini, nessuna esecuzione di contenuti attivi. Preferire cartella non compressa inizialmente: ZIP richiederebbe difese specifiche contro zip bomb.
- **Integrità e concorrenza:** esportazione consistente di SQLite e file media durante modifiche/backup; scrittura temporanea e finalizzazione atomica se il filesystem lo consente; import eventuale con staging, transazione SQLite e gestione degli orfani dopo crash. Non cancellare/rinominare media esistenti né riscrivere snapshot di certificati. Verificare comportamento con USB/NAS scollegati e dischi pieni.
- **Impostazioni:** nessuna condivisione globale abilitata; preferenze facoltative di cartella e profilo di campi (con avviso e revisione **a ogni esportazione**); impostazioni sensibili e path macchina-specifici esclusi dal pacchetto. Nessuna automatizzazione periodica.
- **Lingue:** lingua di sistema con fallback **inglese** e scelta manuale; testo UI e avvisi localizzati IT/EN/ES/FR/DE, ma codici del formato in inglese stabile. Testi immessi dall'utente, UUID, quantità in cm e SHA-256 mantenuti intatti; date di visualizzazione secondo locale, sul disco UTC ISO 8601.

## 6. Dipendenze, alternative, rischi e complessità

**Dipendenze progettuali, non approvate:** contratto/versione di TECH-001, mappatura esatta di `artworks`/galleria/Media Center, gestione copie privacy di TECH-013 (se adottata), serializzazione media+SQLite, validatore schema, compatibilità import di foto e limiti 12 MiB/20 foto per opera, creazione codici localmente e criteri di autorizzazione per schede ricevute. Nessuna nuova libreria è oggi richiesta o scelta. La presenza di fotografie RAW non significa che il destinatario possa legalmente riprodurle.

**Alternative:**
- **A. CSV o brochure/PDF correnti:** soluzione più semplice per scambi occasionali; non reintegra foto e collegamenti in un database.
- **B. Backup completo e ripristino:** conserva tutto ma espone dati non pertinenti e **sostituisce** l'archivio destinatario: non usare come modalità di scambio di singole opere.
- **C. Pacchetto leggibile solo, senza importazione:** minimizza rischi; potrebbe essere il **primo passo preferibile**.
- **D. Scambio selettivo con futura importazione assistita:** più produttivo per trasferimenti frequenti, ma comporta modello di identità, sicurezza, media e collisioni molto più complesso.

**Rischi maggiori:** divulgazione involontaria di una fotografia originale o di una posizione privata; interpretare schede trasferite come autenticazioni/diritti; perdita dell'ordine galleria/crediti; corruzione dopo import parziale; duplicazione di schede dopo più invii; conflitto tra Media Center e copia fisica; modello stato commerciale non adeguato per ricezioni non proprietarie. **Complessità:** medio-alta per export read-only selettivo; **alta** per import transazionale in archivio già popolato.

**Priorità:** P2 per gallerie/artisti che scambiano **regolarmente** schede corredate da foto, altrimenti P3. La priorità è proposta, **non** richiesta di implementazione.

## 7. Criteri di accettazione/test progettati per un futuro eventuale sviluppo

1. Con due opere, una con 20 foto/galleria e una con Media Center revisionato, il pacchetto contiene **solo** le schede e le immagini scelte; nessuna stringa cliente/ubicazione/saldo/certificato appare nei file, manifest, nomi e errori del pacchetto.
2. Senza consenso foto, il pacchetto non include **alcun** byte fotografico. Una foto selezionata compare con dimensione e SHA-256 corretti; nessuna trasformazione invisibile di RAW o revisioni. Le copie con metadata non ripuliti sono etichettate come tali.
3. File eliminato, checksum non valido, manifest manipolato, schema futuro, symlink, percorso `../`, file non regolare, media pericoloso e ZIP non supportato vengono rifiutati in ispezione **senza modifiche** all'archivio attivo.
4. Se è autorizzata in futuro l'importazione in archivio non vuoto: nuovo UUID/codice locale, relazione media e ordine coerenti; import ripetuto avverte e non duplica silenziosamente; conflitti non riscrivono né vendite né certificati né il file originale già presente.
5. Arresto forzato, disco pieno o disconnessione durante export/import: pacchetto incompleto non viene pubblicato come verificato; DB originale e foto restano indenni; staging recuperabile/smaltibile in sicurezza.
6. Verifica dei round-trip e delle omissioni con valori Unicode, null, decimali in cm, file grandi, stessi titoli/codici su archivi diversi; non inferire proprietà o autenticità.
7. Prove native Windows/macOS/Linux, cartelle USB/NAS, diversa sensibilità alle maiuscole nei nomi, accessibilità tastiera/screen reader, UI IT/EN/ES/FR/DE e fallback inglese; **nessun traffico di rete** generato dall'app.

## 8. Requisiti accertati, ipotesi e domande al proprietario

**Accertato:** struttura locale delle opere e dei media su `main`, CSV integrali non reimportabili, backup sostitutivo verificato; i formati di galleria/Media Center e RAW sono già rappresentati nel codice. Nelle schede tecniche precedenti esistono progettazione di portabilità completa (TECH-001), CSV per condivisione (TECH-025) e federazione read-only (TECH-004). **Non è dimostrata la domanda di scambio fra utenti**, né provata l'assenza assoluta di implementazioni alternative fuori dai percorsi verificati.

**Ipotesi:** professionisti vogliono ricevere **schede + fotografie** nell'archivio proprio, anziché solo PDF/CSV; sono disposti a revisionare campi e diritti, senza sincronizzazione. Il profilo del pacchetto potrebbe condividere lo stesso contratto aperto di TECH-001.

**Domande per la revisione personale:**
- Quante volte al mese un artista consegna **dati strutturati** (non solo PDF/foto) a una galleria?
- Basta un pacchetto consultabile con software generico, oppure serve davvero importazione nel database **già popolato**?
- Sono consentiti metadati e fotografie originali, oppure soltanto copie derivate e controllate? Quali diritti vengono concessi, e come dichiarare esplicitamente che l'app **non li certifica**?
- Se l'opera ricevuta è in semplice custodia, come evitare che `available/reserved/sold/archived` ne rappresenti falsamente la proprietà o disponibilità? Occorre attendere UX-001/012/021?
- Il destinatario desidera conservare un riferimento al mittente? Quali campi minimi senza diffondere dati personali?
- Quali limiti per numero di opere e byte, e quale comportamento se una scheda cambia dopo la trasmissione?

## 9. Fasi di valutazione, **non** piano approvato

1. **Fase 0 — verifica del bisogno:** 3–5 casi reali anonimi artista↔galleria/archivio, confronto dei tempi e delle informazioni condivise oggi tramite PDF/CSV/immagini.
2. **Fase 1 — specifica descrittiva:** definire il profilo minimo aperto e compatibile TECH-001, matrice disclosure, gestione revisioni/foto e compatibilità con i domini UX. Nessuna implementazione.
3. **Fase 2 — eventuale MVP solo se approvato:** esportazione esplicita read-only e **ispezione senza import**, con test di omissione privacy e verifica; valutare se risolve già il bisogno.
4. **Fase 3 — eventuale estensione soltanto dopo decisioni su stato/identità:** importazione assistita di nuove schede in archivio popolato, senza merge di record, con transazione e controllo conflitti.
5. **Fase 4 — eventuali evoluzioni:** profili di condivisione ulteriori solo su bisogni misurati; mai sync/cloud obbligatorio.

> **Solo proposta da valutare personalmente.** Questo documento non autorizza nuove funzioni, test, modifiche a codice/configurazioni, issue, PR operative o merge.
