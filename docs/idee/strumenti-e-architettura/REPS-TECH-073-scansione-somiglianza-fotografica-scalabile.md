# REPS-TECH-073 — Ricerca di immagini simili: scansione scalabile e copertura dichiarata

- **ID stabile:** REPS-TECH-073
- **Data ricognizione:** 2026-11-23
- **Stato:** **Proposta — da valutare**
- **Filone:** Strumenti e architettura; esclusivamente analisi documentale.
- **Priorità proposta:** P2 se tempi, memoria o copertura risultano problematici con archivi professionali; altrimenti P3/sola guida o nessuno sviluppo.
- **Decisione:** non approvata né assegnata; nessuna autorizzazione all'implementazione.

## 1. Motivazione, destinatari e riscontro su main

Il Media Center permette già di trovare immagini identiche e visivamente simili. Un fotografo d'opere, uno studio o un archivio che importa ripetutamente sessioni con molte fotografie può usare questa funzione per individuare scatti ridondanti senza confondere immagini diverse della stessa opera. Il bisogno è rendere affidabile e utilizzabile **la ricerca esistente**, non introdurre deduplicazione automatica, riconoscimento di opere o classificazione IA.

Ricognizione statica del 23 novembre 2026 su [README](https://github.com/Cesare-ml/Repertica-Studio/blob/main/README.md), [MVP](https://github.com/Cesare-ml/Repertica-Studio/blob/main/docs/MVP.md), [media.rs](https://github.com/Cesare-ml/Repertica-Studio/blob/main/src-tauri/src/media.rs), [media-center.component.ts](https://github.com/Cesare-ml/Repertica-Studio/blob/main/src/app/media-center.component.ts):

- `similar_core` (media.rs, circa righe 685–720) sincronizza i riferimenti, carica gli elementi ACTIVE e usa l'impronta dHash a 64 bit della revisione corrente. Riusa la cache `dhash`/`dhash_revision_id` se valida; altrimenti legge e calcola l'impronta, aggiornando la cache. La soglia di distanza di Hamming è 6.
- Una lettura o un calcolo fallito causa `continue` senza comunicare all'interfaccia il numero di immagini saltate. La UI, una volta ricevuta una lista vuota, mostra «Nessuna immagine simile trovata» (circa righe 129–132). La frase non distingue una scansione completa da una parziale.
- Il confronto visita tutte le coppie delle impronte disponibili. Con N impronte complete ci sono N(N−1)/2 confronti: 499.500 per 1.000; 49.995.000 per 10.000; 449.985.000 per 30.000. Questo è un conteggio matematico, **non** un benchmark. I confronti sono economici singolarmente; il costo di decodifica, SQL e serializzazione va misurato separatamente.
- Ogni coppia ammessa contiene due `MediaItem` completi, viene accumulata e ordinata, poi restituita alla GUI; `scanSimilar` carica anche le miniature dei risultati e il template li itera. Un insieme di molte impronte uguali o ravvicinate può quindi aumentare memoria e volume IPC anche con un indice efficiente.
- Il commento sul confronto esatto non basta a garantire esclusione dei contenuti byte-identici: la condizione osservata confronta i percorsi, non gli hash SHA-256. Non modificare questa semantica nel progetto di scalabilità; verificare separatamente se escludere o mantenere in entrambi i registri quei risultati.

Non sono state eseguite scansioni, benchmark o test su archivi reali. Non è dimostrato un rallentamento, crash o falso negativo reale. La rilevazione statica di un possibile esito parziale è motivo per verificare la chiarezza del messaggio, anche su archivi piccoli.

## 2. Scenari d'uso

- Una galleria importa migliaia di immagini da varie campagne fotografiche; cerca possibili copie a risoluzioni diverse senza toccare le fotografie.
- Un artista cerca ridondanze in una serie di immagini molto somiglianti; deve poter distinguere un risultato visivo da un'identità certa e mantenere tutti gli originali.
- Un archivista avvia una scansione con un'immagine non leggibile; vuole sapere che i risultati si riferiscono soltanto alle immagini effettivamente analizzate e poi usare la diagnostica già disponibile.
- Durante una scansione viene modificata una revisione fotografica. I risultati riferiti alla revisione precedente non devono essere scambiati per risultati aggiornati.

## 3. Specifica funzionale e flusso UX (ipotesi)

**Minimo utile:** conservare il pulsante attuale «Cerca immagini simili». Mostrare copertura: candidati iniziali, impronte utilizzabili, non analizzate, eventuale risultato parziale. Se 999 immagini su 1.000 sono analizzate, dichiarare che la ricerca copre quelle 999; se non vi sono coppie, evitare una conclusione assoluta. Non mostrare percentuali come probabilità che due opere siano uguali; la percentuale attuale è derivata dalla distanza dHash, non una confidenza statistica.

1. L'utente entra in Media Center → Duplicati, sceglie il comando esistente. Nessuna scansione automatica in background.
2. La ricerca lavora sulle revisioni attive identificate all'inizio; cache valida riutilizzabile. Una revisione cambiata o rimossa durante il lavoro può invalidare il suo risultato; non mostrare uno stato definitivo non verificato.
3. Esito completo, parziale, interrotto o fallito chiaramente distinto; un esito parziale indica il motivo per categoria (senza divulgare percorsi assoluti). Un collegamento alla diagnostica corrente può aiutare nei casi di file mancanti.
4. Se i risultati sono molti, mostrare una prima pagina con ordinamento deterministico (distanza, ID stabili), possibilità di proseguire e indicazione esplicita di eventuale limite. Non chiamare «totale» un conteggio troncato; nessun limite nascosto. Anteprime solo delle righe consultate.
5. Aprire e confrontare i due elementi tramite le azioni esistenti. Nessuna fusione, rimozione, modifica di revisione, certificato o opera; una coppia è un indizio.
6. Ripetizione manuale dopo modifica delle immagini. Eventuale progresso/annullamento deve riusare i principi di TECH-062, non introdurre un secondo motore di job.

## 4. Alternative tecniche e struttura dati descrittiva

**A — mantenere l'algoritmo attuale:** misurare prima; aggiungere solo conteggi di copertura e stato esplicito se necessario. Ottimizzare letture e ridurre copie dei metadati se i profili lo motivano. È l'alternativa preferita per archivi normali.

**B — risultato compatto e a pagine:** riferimenti `media_id` + `revision_id` + distanza, con metadati e miniature risolti a richiesta. Separare limite di memoria e limite del numero di coppie visitate; anche con un indice, una ricerca con N impronte identiche può produrre N(N−1)/2 coppie. Preferire un limite esplicito dei risultati, o proseguimento controllato, a un accumulo illimitato. Una pagina non deve implicare copertura completa se la ricerca è stata fermata.

**C — indice esatto per distanza di Hamming:** solo se il confronto quadratico è il collo di bottiglia misurato. Un indice BK-tree in memoria è una possibilità; altra possibilità, un indice per blocchi disgiunti. Con 7 blocchi disgiunti su 64 bit e soglia 6, due impronte a distanza non superiore a 6 devono condividere almeno un blocco identico (principio dei cassetti); l'unione dei candidati va deduplicata e ogni distanza ricontrollata. Questo evita falsi negativi introdotti dal filtro, ma i blocchi molto popolati possono annullare il vantaggio. Nessun passaggio a una soglia diversa o a un modello di similarità approssimato senza decisione separata.

Struttura concettuale, non un'API proposta come definitiva:
- Snapshot temporanea: identificatori di elementi/revisioni, impronte valide, versione del criterio di confronto. Non conservare altre fotografie.
- `ScanSummary`: stato, candidati, analizzati, saltati per motivo, eventuali cambi di revisione, soglia e criterio, copertura delle coppie, limite raggiunto. Distinguere copertura delle immagini e dei confronti.
- `PairRef`: due ID media/revisione e distanza intera; coppia non ordinata normalizzata e senza duplicati.
- `scanSimilar(scope, budget)` e `readSimilarPage(scanId, cursor)` sono descrizioni, non nuovi comandi Tauri approvati; mantenere compatibilità del comando esistente finché possibile. Un cursore è valido per una sessione locale e non deve restituire dati di un archivio ripristinato o di revisioni non più valide.

Nessun indice persistente o migrazione nella fase minima. L'impronta esistente è derivata e può essere rigenerata. Un'eventuale struttura persistente richiederebbe una valutazione distinta.

## 5. Offline, portabilità, sicurezza, privacy, impostazioni e lingue

- Tutto locale e facoltativo. Non richiede rete, account, servizi IA, nuove autorizzazioni OS o invio di immagini/impronte.
- Non ampliare MCP, né esporre contatti, foto dei collezionisti o percorsi. Il Media Center contiene anche fotografie non pubbliche: la scansione e la cache restano dentro il perimetro attuale dell'archivio.
- Nessuna nuova cancellazione, modifica di originali, revisione storica o certificati. Le sole scritture eventualmente ammesse sono aggiornamenti della cache derivata già prevista, legati alla revisione effettivamente analizzata. Non trattenere il mutex SQLite per un confronto di tutte le coppie.
- Rispettare limiti di decodifica e controlli dei file già esistenti; errori localizzati senza nomi sensibili in log esportabili. Budget e cancellazione cooperativa, senza processi o job permanenti. Nessuna telemetria.
- Nessuna nuova impostazione globale nella prima fase. Un ambito filtrato o budget manuale è un'ipotesi successiva: deve essere dichiarato e non cambiare silenziosamente la soglia. Lingua di sistema con fallback inglese, preferenza manuale corrente; tradurre stati, contatori, errori, etichette e descrizioni accessibili in IT/EN/ES/FR/DE. Numeri formattati secondo locale, ID e distanza invariati.
- Backup e ripristino integrali, CSV e formato archivio esistenti restano invariati. Nessuna reimportazione di risultati né dipendenza da una cache: un nuovo computer può ricalcolare. Export opzionale di un report locale CSV/JSON (solo ID, revisioni e distanze, stato e copertura, senza immagini né percorsi) esclusivamente se richiesto da utenti; destinazione scelta e protezione da sovrascritture. Non è un backup né una prova di identità.
- Accessibilità: esito leggibile da screen reader, focus stabile, confronto raggiungibile via tastiera, nessuna informazione affidata solo al colore, pagine o gruppi navigabili.

## 6. Dipendenze, complessità, rischi

Tauri, Angular, Rust, SQLite, funzioni dHash e Hamming esistenti sono sufficienti per lo studio e per l'ipotesi minima. Nessuna nuova dipendenza proposta; un eventuale indice deve essere valutato rispetto a licenza, mantenibilità e precisione. Complessità bassa/media per copertura e riduzione delle copie; media/alta per paginazione con risultati consistenti e concorrenza; alta se si pretende una scansione completa di output enormi.

Rischi: falsi negativi con filtri approssimati; coppie esplosive in serie visivamente omogenee; cache riferita a revisione superata; risultati obsoleti dopo ripristino; pagine duplicate o mancanti; lock prolungati; UI che indica erroneamente una scansione completa; percezione di similarità come identità certa. Ridurre lavoro senza cambiare il significato del risultato. Non imporre costi di manutenzione per un problema non misurato.

## 7. Criteri di accettazione e verifiche future (non eseguite)

- Benchmark riproducibile 100, 1.000, 10.000 e 30.000 impronte sintetiche; distribuzione casuale, cluster, tutte identiche; separare cache calda/fredda, decodifica, SQL, confronti, serializzazione IPC, rendering e picco RAM. Usare anche piccole fixture fotografiche non personali; registrare OS, hardware e versione.
- Su fixture gestibili, parità di insieme e distanza con l'enumerazione esaustiva per il medesimo criterio (inclusi confini 5/6/7); nessun falso negativo aggiuntivo; coppie uniche e ordine stabile. Definire a parte l'eventuale trattamento degli identici byte-per-byte.
- File mancante, non decodificabile, cache assente o non valida: contatori e stato corretto; nessuna conclusione assoluta se la copertura è parziale.
- Budget raggiunto, zero risultati parziali, molti risultati, chiusura della vista, ripristino e modifica concorrente di una revisione: non confondere incompletezza e assenza, non mostrare risultati di revisioni non verificate come correnti. Nessuna perdita di dati.
- Verificare che l'archivio rimanga utilizzabile e che originali, vendite, certificati, backup e CSV non cambino. Nessun traffico di rete.
- Test accessibilità e localizzazione, incluso fallback EN. Concordare con il proprietario soglie di tempo e RAM su hardware di riferimento **dopo** baseline, non promettere obiettivi arbitrari. Nessuna modifica se l'attuale algoritmo soddisfa il bisogno e il messaggio risulta adeguato.

## 8. Fasi e priorità ipotetica

0. Verificare domanda professionale e casi reali; raccogliere baseline e verificare significato della copertura. Nessun lavoro operativo avviato.
1. Se serve, chiarire l'esito e i contatori; ottimizzare solo dove misurato, mantenendo il criterio attuale.
2. Solo se volume dei risultati problematico, studiare riferimenti compatti e pagine con limiti dichiarati.
3. Solo se CPU dei confronti ancora dominante, confrontare indice esatto in memoria con baseline. Eventuale export o ambito filtrato separato, dopo domanda confermata.

**P2 condizionata**, P3/QA o nessuno sviluppo se la funzione attuale è sufficientemente rapida e comprensibile. Non è una priorità approvata.

## 9. Confini e domande per il proprietario

Confrontati gli indici delle 72 proposte TECH e 77 UX preesistenti nella [PR #14](https://github.com/Cesare-ml/Repertica-Studio/pull/14). TECH-003 diagnostica integrità e recupero; TECH-005 cache delle anteprime; TECH-062 progresso e stop trasversali; TECH-070 filtraggio e rendering della lista opere; UX-055 possibili duplicati di **schede-opera**, non immagini. Qui si tratta solo del motore di ricerca fotografica già presente, della completezza dichiarata e della quantità di risultati. Nessuna nuova deduplicazione.

**Requisiti certi:** principi locali/open source, originali e storia intatti, trasparenza degli esiti, backup portabili. **Ipotesi:** gli archivi grandi incontrano un problema di risorse; la copertura esplicita ha valore per professionisti; un indice migliora i tempi. Da verificare, non presupporre.

Questioni aperte:
- Quante immagini e quali campagne fotografiche rappresentano gli utenti reali? Quanto spesso usano «Cerca immagini simili»?
- Preferire una lista limitata esplicitamente ai migliori risultati o consentire consultazione completa anche per cluster enormi?
- È utile mostrare separatamente le immagini identiche e quelle simili, mantenendo comunque l'accesso a tutti i casi?
- È necessario un report portabile o basta il confronto in app?
- Quale macchina di riferimento e quale durata di attesa è accettabile?

Nessuna implementazione, test eseguito, modifica di codice, configurazione, dipendenza, issue o approvazione. Decisione riservata al proprietario.
