# REPS-TECH-070 — Consultazione fluida degli archivi grandi: costo dei filtri e rendering delle liste desktop

- **ID stabile:** REPS-TECH-070
- **Data ricognizione:** 2026-11-23
- **Stato:** **Proposta — da valutare**
- **Filone:** strumenti e architettura; analisi e documentazione soltanto.
- **Priorità ipotetica:** **P2 solo dopo misure riproducibili su archivi realistici**; P3, semplice QA o nessun intervento se la UX attuale resta soddisfacente. Non approvata e non assegnata.
- **Perimetro:** fluidità dell'interfaccia desktop di consultazione di un **singolo archivio**, senza nuovi filtri, viste salvate, server, cloud o modifiche obbligatorie ai dati.

## Motivazione, destinatari e riscontri osservati

Un archivista, una galleria o uno studio d'artista possono consultare migliaia di opere e collezionisti, applicare ricerca e ordinamento e scorrere l'elenco durante una visita. La capacità di trovare un risultato già supportato dall'app può perdere valore se la digitazione o lo scorrimento diventano lenti. **La lentezza non è stata riprodotta né misurata in questa ricognizione**; si propone prima un'indagine circoscritta, non una riscrittura preventiva.

**Evidenza statica su `main`, 23 novembre 2026:**

1. In [`src-tauri/src/commands.rs`](../../../src-tauri/src/commands.rs), `list_artworks` legge le opere in una `Vec<Artwork>` ordinata per aggiornamento, senza una finestra o un limite SQL. `list_sales` e `list_certificates` restituiscono a loro volta registri completi.
2. [`src/app/app.component.ts`](../../../src/app/app.component.ts), `refresh()`, richiede in parallelo opere, collezionisti, vendite, certificati e altri registri, quindi aggiorna l'array `artworks`. Costruisce già **indici per vendita** una sola volta per refresh: conservare questa ottimizzazione.
3. Il getter `visibleArtworks` invoca ogni volta `filterArtworks(this.artworks, ...)`. In [`src/app/archive-filters.ts`](../../../src/app/archive-filters.ts) questa funzione scandisce le opere, prepara i campi di ricerca normalizzati per confrontarli con i termini, quindi ordina i risultati quando richiesto. Nella pagina [`src/app/app.component.html`](../../../src/app/app.component.html) il getter compare in **conteggio, iterazione `*ngFor` e stato vuoto**. Ogni invocazione produce un nuovo array; **non equivale necessariamente** a ricreare tutte le righe DOM a ogni rilevazione Angular.
4. L'elenco attuale materializza nel template una riga per ogni opera visibile; nel tratto esaminato non si vede una paginazione o virtualizzazione della **tabella delle opere**. Il visualizzatore a schermo intero acquisisce intenzionalmente uno snapshot della lista filtrata per navigare tra le opere.
5. `refresh()` limita già a 35 nuove copertine le richieste iniziali e a 60 quelle per i profili: **non** affermare che scarichi immediatamente tutte le immagini. Il Media Center possiede già una propria API con `offset` e `limit`; questa scheda non ne richiede la sostituzione.
6. [`tests/archive-filters.test.mjs`](../../../tests/archive-filters.test.mjs) documenta la semantica della ricerca. Non sono stati eseguiti test prestazionali, profiler Angular né prove Windows/macOS/Linux con cataloghi numerosi.

**Problema da verificare:** tempi di risposta a digitazione, filtro e ordinamento; numero di valutazioni/ricalcoli, dimensione del DOM, memoria del WebView e tempo di apertura o navigazione nel visualizzatore. Il costo eventuale di trasferire tutti i record via IPC va misurato **separatamente**, non presunto come collo di bottiglia.

## Distinzione dalle altre proposte

- [TECH-008](REPS-TECH-008-viste-salvate-filtri-ricerca-locale.md): nuovi criteri, viste e semantica della ricerca; menziona una futura paginazione solo come possibile conseguenza di filtri avanzati. **Qui nessuna vista/criterio nuovo:** si studia il costo dell'**attuale** getter, della tabella e del refresh; la prima soluzione potrebbe non toccare Rust o SQLite.
- [TECH-036](REPS-TECH-036-mcp-archivi-grandi-paginazione-limiti-risorse.md): paginazione e budget del **server MCP STDIO**. Questa proposta riguarda soltanto il desktop Angular; nessuna nuova API MCP.
- [TECH-005](REPS-TECH-005-anteprime-media-grandi-archivi.md): generazione, conservazione e invalidazione delle anteprime fotografiche; qui non si studiano codec, qualità o cache dei media.
- [TECH-048](REPS-TECH-048-parita-ricerca-interfaccia-mcp-unicode.md): parità semantica tra motori UI e MCP. Una futura ottimizzazione deve **preservare** questa semantica, non sostituirla unilateralmente.
- [TECH-020](REPS-TECH-020-accessibilita-flussi-critici-tastiera-screen-reader.md): accessibilità dei dialoghi e flussi critici; eventuale paginazione o virtualizzazione **deve** essere verificata con tastiera e screen reader, non essere assunta accessibile.
- [UX-071](https://github.com/Cesare-ml/Repertica-Studio/blob/docs/idee-gestione-ux-20261121/docs/idee/gestione-e-ux/REPS-UX-071-ritorno-contesto-consultazione-archivio.md): ritorno da opera a collezionista senza perdere criteri. [UX-074](https://github.com/Cesare-ml/Repertica-Studio/blob/docs/idee-gestione-ux-20261121/docs/idee/gestione-e-ux/REPS-UX-074-ricognizione-copertine-fotografiche-mancanti.md): nuovo filtro di copertine assenti. Nessuno di questi due flussi viene aggiunto qui. Consultato l'[indice Gestione e UX — PR #14](https://github.com/Cesare-ml/Repertica-Studio/pull/14) senza modificarlo.

## Scenari d'uso e flussi UX descrittivi

1. **Studio piccolo, 100 opere:** aprire Archivio, cercare per titolo e tecnica, ordinare, aprire una scheda. Nessun obbligo di comprendere pagine, cursori, opzioni o nuove modalità.
2. **Galleria con 10.000 opere sintetiche:** digitare una frase con accenti, applicare stato e collezionista, spostarsi nell'elenco. La barra resta utilizzabile durante l'interazione; contatore «risultati / totale» corrisponde ai filtri effettivi. Un miglioramento va dimostrato da confronto prima/dopo, non soltanto percepito.
3. **Collezionista di un'opera:** dalla riga filtrata aprire l'acquirente, poi tornare con il comportamento attuale. Non introdurre qui il ritorno contestuale di UX-071.
4. **Visualizzatore:** aprire un'opera e usare precedente/successiva secondo l'ordine filtrato al momento dell'apertura; una eventuale finestra visibile della tabella **non deve ridurre implicitamente** l'universo delle opere navigabili.
5. **Dati appena cambiati:** modificare titolo, stato o vendita e aggiornare; non mostrare risultati stantii, conteggi non corrispondenti o miniature associate all'UUID errato.
6. **Tecnologie assistive:** percorrere le righe via tastiera, leggere intestazioni e posizione nella lista, tornare alla riga di partenza dopo l'apertura del visualizzatore. Se una soluzione incrementale non lo consente, non adottarla.

### Possibile esperienza minima (solo dopo prova)

L'utente vede gli stessi input, lista, messaggi e azioni. Eventuali ottimizzazioni di calcolo sono invisibili. Solo **se** migliaia di righe rendono il DOM davvero oneroso, confrontare una paginazione esplicita con comandi «Prima / Precedente / Successiva», numero totale e focus prevedibile, oppure una lista virtualizzata realmente accessibile. Non introdurre «scorrimento infinito» come requisito: rischia di nascondere la posizione e peggiorare l'uso da tastiera.

## Specifica funzionale proposta, non implementativa

**Requisiti certi da conservare:** dati dell'archivio attivo; filtri per termini, frasi nello stesso campo, accenti, punteggiatura, misure, stato e collezionista; ordinamenti attuali; contatore completo; relazioni opera/vendita; navigazione del visualizzatore; UUID persistenti; backup e CSV integrali indipendenti dai filtri.

**Ipotesi di valutazione in ordine di invasività:**

1. **Misurazione:** dataset sintetici, ad esempio 100, 1.000, 10.000 e 30.000 schede con testi realistici, registri di vendite e collezionisti, varianti di lingua e fotografie opzionali. Misurare con ambiente e versione registrati tempi p50/p95 di risposta alla ricerca, uso memoria, quantità di nodi, caricamento iniziale e latenze apertura/chiusura viewer. Le soglie di accettabilità vanno concordate dopo osservazioni di utenti e hardware reale, non inventate ora.
2. **Riduzione dei ricalcoli della vista:** valutare una derivazione stabile dei risultati aggiornata solo quando cambiano record, filtri, query, associazioni o ordine, preservando immutabilità, consistenza dopo `refresh()` e regole di ricerca. Evitare cache per UUID che contaminino un altro archivio. Confrontare con l'implementazione attuale sotto profiler.
3. **Rendering progressivo soltanto se necessario:** confronto di paginazione accessibile, virtualizzazione in tabella e rendering attuale; `trackBy` o identità stabile delle righe non bastano da soli a risolvere il costo di filtering. Con pagine, conteggio riferito all'intero risultato, non alla pagina. Il viewer naviga l'insieme filtrato e non può interpretare silenziosamente una pagina come elenco completo.
4. **Solo se misurato un limite IPC/memoria:** studiare una query locale Rust di sola lettura con filtri parametrizzati, ordinamento deterministico (UUID a parità), indice mirato e strategia di continuità. **Non** passare meccanicamente a `LIKE`/FTS: normalizzazione, accenti, frasi e misure devono restare equivalenti; vedi TECH-048. La ricerca avanzata e le viste salvate restano nell'ambito separato TECH-008.

**Comportamento degli errori:** richiesta di consultazione interrotta o fuori data non sovrascrive una risposta più recente; dopo un errore mostrare spiegazione recuperabile e permettere una ricerca nuova. Nessuna cancellazione automatica di dati o cache durevoli. Verificare anche scansione con 0 risultati, modifica durante filtro e record eliminati durante una consultazione.

## Dati e ipotesi di API (solamente descrittive)

**Prima ipotesi, senza migrazioni:** usare in memoria le già esistenti liste `Artwork[]`, `Sale[]`, `Collector[]`, insieme a `ArtworkFilters` e `SaleIndexes`, e una derivazione effimera `VisibleArtworkResult { ids: UUID[], total: number, revision: number }`. `revision` è una generazione *dell'interfaccia*, non un campo aggiunto a SQLite né un audit dell'archivio.

**Soltanto eventuale fase successiva:** contratto puramente ipotetico `queryArtworks({ query, status, collector_id, order, cursor?, limit? }) -> { records, total_matching, next_cursor?, applied_filter_revision }`. Il cursore non è un URI, non espone chiavi personali né consente accessi esterni; scade quando cambia archivio o insieme dei filtri. Se la semantica è legata a uno snapshot, esplicitare limiti di consistenza fra modifiche concorrenti. Non aggiungere un'API di scrittura o trasmettere dati oltre Tauri IPC locale.

La vista eventualmente derivata non è un nuovo registro: non si esporta come dati autorevoli né si include come obbligo nel backup. Le opere, i contatti, le vendite e i documenti restano integralmente esportabili tramite gli strumenti già presenti.

## Offline, privacy, sicurezza, impostazioni, multilingua, compatibilità

- **Offline-first:** tutta la ricerca e il rendering restano nel processo desktop e nel database SQLite locale; nessun account, abbonamento, telemetria, cloud, servizio indicizzatore permanente o richiesta di connessione.
- **Privacy:** dati dei collezionisti e query non finiscono in log, analytics o cache temporanee non cancellabili. Dataset di prova soltanto **sintetici**, mai export privati prelevati di default dai professionisti; niente invio automatico a strumenti IA.
- **Sicurezza:** se si valuta un endpoint Rust, filtri e ordinamenti ammessi da una lista chiusa, query parametrizzate, budget di tempo/memoria studiati e gestione non distruttiva degli errori. Un cursore non autorizza accesso a un archivio diverso. Nessuna istruzione SQL arbitraria.
- **Impostazioni:** per la fase iniziale nessuna nuova preferenza; eventuale numero di risultati per pagina solo dopo prova UX, con valore conservativo, persistenza opzionale locale, ripristino del default e nessun blocco di accesso alle opere.
- **Multilingua:** lingua di sistema con fallback inglese come già previsto; tutte le nuove etichette/messaggi, se introdotti, tradotti nelle lingue supportate; preservare ricerca e ordinamento Unicode senza trasformare i valori originali del catalogo.
- **Compatibilità:** Windows/macOS/Linux, desktop Tauri 2 e archivi SQLite precedenti; nessuna migrazione obbligatoria per la fase minima, nessuna variazione di CSV, backup, export immagini, certificati, MCP, codice opera o storico delle vendite.
- **Accessibilità:** contatore annunciato senza ripetizioni invasive, focus su controllo/riga corretta, intestazioni leggibili, indicatori di pagina non dipendenti dal colore, comandi da tastiera e screen reader; evitare virtualizzazione che sottragga le righe all'esplorazione assistita senza alternativa.

## Dipendenze, alternative, rischi e complessità

**Dipendenze esistenti:** Angular + WebView Tauri 2, `archive-filters.ts`, indici vendite, renderer lista e visualizzatore; SQLite/Rust solo per l'eventuale terza fase. **Nessuna libreria esterna è assunta necessaria.**

**Alternative:** mantenere tutto invariato se le misure sono soddisfacenti; indicazioni d'uso su filtri già disponibili; memoizzazione locale senza nuove UI; paginazione classica; virtualizzazione accessibile; query SQLite solo dopo confronto semantico. Non imporre nuovi controlli a chi possiede poche opere.

**Rischi:** invalidazione incompleta di risultati memorizzati dopo modifiche; ordini incoerenti a parità di dati; selezione/posizione del viewer errata; conteggi incompleti spacciati per totali; normalizzazione diversa rispetto alla ricerca attuale; pagine con contenuti duplicati/saltati; perdite di focus; peggioramento screen reader; aumento dello stato applicativo; benchmark non rappresentativi. Nessun miglioramento va proclamato senza un confronto riproducibile.

**Complessità indicativa:** bassa per analisi/profilazione e piccoli miglioramenti di derivazione; media per paginazione e accessibilità; alta per nuova interrogazione Rust con parità Unicode, coerenza, cursori e migrazione del flusso di viewer. La complessità concreta si decide solo dopo la fase misurativa.

## Test proposti e criteri di accettazione per un'eventuale decisione

1. Evidenziare una **regressione misurabile** o un disagio confermato su hardware rappresentativo; registrare baseline e strumenti di misura, senza dati privati.
2. Tutti i casi di ricerca già coperti da `tests/archive-filters.test.mjs` conservano lo stesso insieme di UUID, ordine e conteggi; includere accenti, virgolette, segni, misure, valori vuoti e collezionisti omonimi.
3. Test con dati sintetici a 100/1k/10k/30k opere e vendite collegate; osservare memoria, tempi, DOM e viewer in tre OS se possibile; non imporre soglie arbitrarie prima del benchmark.
4. Dopo modifica/eliminazione di un'opera o variazione di una vendita, risultati e contatore aggiornati senza informazioni stale; l'utente non vede una selezione di un altro archivio.
5. Viewer precedente/successiva riferito all'intero insieme filtrato; accessibilità con Tab, frecce, screen reader, zoom OS e focus dopo chiusura.
6. Nessun export troncato o cambiato; backup/ripristino conservano lo stesso contenuto; nessuna rete né comunicazione automatica.
7. Se il benchmark non mostra vantaggi netti senza peggiorare compatibilità/UX, **criterio di successo della ricognizione è decidere di non sviluppare**.

## Fasi suggerite — tutte subordinate a revisione del proprietario

- **Fase 0 — sola verifica:** raccogliere 3–5 esempi di volumi reali dichiarati da artisti, gallerie e archivi; costruire benchmark sintetici e misurare UI/IPC. Nessun intervento sul prodotto.
- **Fase 1 — eventuale miglioramento interno minimo:** ottimizzare derivazione dei risultati solo se il profiler individua lì il problema; confronto regressioni sui filtri.
- **Fase 2 — eventuale rendering progressivo:** scelta UX fra paginazione e alternativa accessibile, prototipo valutato con professionisti, senza promettere che sia necessario.
- **Fase 3 — solo se inevitabile:** interrogazione paginata Rust/SQLite per archivi molto grandi, subordinata a parità di ricerca e sicurezza dell'archivio.

## Questioni aperte per il proprietario

1. Quante opere/collezionisti hanno realisticamente gli utilizzatori target: centinaia, migliaia o decine di migliaia?
2. Sono stati osservati rallentamenti di ricerca, scroll o apertura viewer, su quale sistema e con quanti record?
3. Si preferirebbe mantenere la tabella continua o, solo per archivi grandi, una paginazione chiaramente navigabile?
4. Quali tecnologie assistive devono essere provate prima di cambiare le modalità di visualizzazione?
5. Quale degrado misurato giustificherebbe la complessità extra anziché conservare il funzionamento attuale?

**Esito:** proposta da sottoporre a valutazione personale, **non approvata, non assegnata e non autorizzata allo sviluppo**. Nessun codice, test, build, workflow, impostazione o dipendenza da modificare per questa ricognizione.
