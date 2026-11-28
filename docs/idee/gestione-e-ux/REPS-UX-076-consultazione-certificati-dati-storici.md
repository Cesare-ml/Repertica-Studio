# REPS-UX-076 — Ritrovare un certificato emesso tramite seriale e dati storici

- **ID stabile:** REPS-UX-076
- **Data:** 2026-11-23
- **Stato:** Proposta — da valutare
- **Filone:** Gestione e UX, solo documentazione
- **Priorità ipotetica:** P2 se il recupero dei certificati è un'esigenza ricorrente; P3 o nessuna funzione se bastano il registro attuale e il CSV.
- **Decisione:** nessuna approvazione, assegnazione o autorizzazione all'implementazione.

## Problema, destinatari ed evidenza

Un collezionista può chiedere all'artista una nuova copia del certificato citando il numero del documento, il titolo stampato anni prima o il codice dell'opera. Il titolo di catalogo può essere stato corretto dopo l'emissione. Una ricerca sul solo titolo corrente non dovrebbe far perdere la possibilità di ritrovare il documento storico. Si tratta di recuperare **un certificato già emesso**, non di attestare autenticità, proprietà attuale o avvenuta consegna.

**Ricognizione statica del main, 23 novembre 2026:** [template Angular](../../../src/app/app.component.html), righe 199–211: il registro mostra numero, opera, artista, data, Anteprima e Stampa/PDF, iterando direttamente l'intero array certificates; non si vede un controllo di ricerca o filtro del registro. [commands.rs](../../../src-tauri/src/commands.rs), righe 390–440: list_certificates ordina per created_at decrescente e legge a.title dal JOIN con l'opera corrente; issue_certificate_core serializza l'opera nello snapshot JSON prima dell'inserimento transazionale. [models.ts](../../../src/app/models.ts) definisce Certificate con serial, artwork_id, artwork_title, snapshot_json, issued_at, artist_name, collector_name, language, issued_place e altri campi. [DATA-PORTABILITY](../../DATA-PORTABILITY.md) chiarisce che current_artwork_title nel CSV è il titolo attuale mentre snapshot_json è il contenuto storico. [README](../../../README.md) e [MVP](../../MVP.md) confermano registro immutabile e stampa locale. Il codice attuale offre già anteprima e ristampa: non sono nuove funzioni.

**Destinatari ipotetici:** artisti con emissioni pluriennali, assistenti di studio, archivi d'artista e piccole gallerie che recuperano documentazione pregressa. Non ci sono interviste, segnalazioni verificate, misure di tempo o prove desktop in questa ricognizione: la lacuna di consultazione è osservata nel codice, il valore professionale resta da validare.

## Confini e confronto con i cataloghi

Confrontate le 75 proposte Gestione e UX della [PR #14](https://github.com/Cesare-ml/Repertica-Studio/pull/14) e le 71 tecniche della [PR #16](https://github.com/Cesare-ml/Repertica-Studio/pull/16), con lettura mirata delle schede sovrapposte.
- [UX-059](REPS-UX-059-controllo-preemissione-certificati.md) riguarda verifica prima dell'emissione; [UX-045](REPS-UX-045-rettifica-sostituzione-certificati-opere.md) riguarda rettifiche dopo l'emissione. Qui nessuna modifica, revoca, sostituzione o emissione.
- [UX-060](REPS-UX-060-attivita-registrate-opera-vista-unificata.md) parte da un'opera già individuata, mentre qui il punto di partenza è un documento o numero storico, anche se l'opera corrente ha un altro titolo.
- [UX-048](REPS-UX-048-identificativi-storici-riferimenti-esterni-opere.md) propone identificativi esterni delle opere; qui si usano soltanto seriali e snapshot già registrati.
- [TECH-008](https://github.com/Cesare-ml/Repertica-Studio/blob/docs/idee-strumenti-architettura-20261121/docs/idee/strumenti-e-architettura/REPS-TECH-008-viste-salvate-filtri-ricerca-locale.md) definisce scope=artworks e criteri di presenza certificato per opere, non la ricerca di un preciso documento storico. Riutilizzare eventuali convenzioni comuni senza introdurre viste salvate.
- TECH-023 riguarda conservazione PDF/A, TECH-048 parità ricerca opere/MCP, TECH-070 prestazioni lista opere, TECH-020 accessibilità. Non sviluppare motori di ricerca generali, integrazioni MCP, nuovi formati o PDF in questa proposta. Nessun documento tecnico modificato.

## Scenari e percorso UX

1. **Seriale comunicato da un collezionista.** Aprire Certificati, digitare RS-2024-000123 (esempio fittizio), ottenere il record corrispondente, controllare numero, data e opera, aprire l'anteprima già esistente. Non generare un nuovo numero; la stampa resta un'azione separata.
2. **Titolo cambiato.** Il certificato riportava «Studio in blu», oggi la scheda è «Studio n. 4». Cercare uno dei due titoli oppure il codice storico se presente nello snapshot. Mostrare «Titolo all'emissione» e, se diverso, «Titolo attuale in archivio» come dato secondario esplicito; il documento resta quello emesso. Non sostituire il titolo storico mancante con il corrente fingendo che sia storico.
3. **Più certificati per una stessa opera o titoli uguali.** Mostrare tutti i risultati con seriale, data e codice quando disponibile. Non scegliere automaticamente il più recente come documento valido; nessuna presunzione di sostituzione, revoca o autenticità.
4. **Dati legacy o snapshot non interpretabile.** Il record resta ricercabile tramite seriale e dati di registro; indicare «Dati all'emissione non leggibili» con etichetta testuale. Non bloccare tutti gli altri risultati, inventare informazioni o riparare JSON. Rimandare agli strumenti già disponibili e al backup.
5. **Ricerca senza risultati.** Distinguere archivio senza certificati da ricerca senza corrispondenze; offrire azzeramento. Non dedurre che l'opera non abbia mai avuto certificati al di fuori del registro locale.

Percorso minimo: Certificati → campo «Cerca per numero, titolo o codice dell'opera» → conteggio risultati → selezione del record → Anteprima esistente → chiusura e ritorno alla stessa ricerca. Filtro anno di emissione solo se utile, con opzione Tutti; non confondere anno dell'opera con anno del documento. Ordine iniziale attuale, risultati con ordine deterministico; mantenere azione di emissione separata dai risultati. Nessuna azione automatica a seguito di ricerca esatta. Per una richiesta riservata, non includere intestatario e note nella lista o ricerca minima.

Accessibilità: label persistente, azzeramento con tastiera, focus prevedibile al ritorno, contatore annunciato senza annunciare ogni carattere, titoli storico/attuale distinguibili anche senza colore, stato di dati non leggibili testuale, verifica zoom e screen reader sulle WebView native.

## Dati e specifica descrittiva

**Fase minima:** vista derivata in memoria dai Certificate già caricati; nessuna tabella, migrazione, persistenza di query o nuovo comando. Usare UUID del certificato come identità di riga, artwork_id soltanto come collegamento, serial e issued_at dal registro, titolo/codice storici solo da un oggetto snapshot valido con campi stringa. Un possibile modello concettuale di lettura comprende certificate_id, serial, issued_at, historical_title?, historical_code?, current_title?, snapshot_read_status e matched_fields. Distinguere campo assente da snapshot non interpretabile, trattare JSON come dati e renderizzare testi tramite binding sicuri, mai HTML grezzo. Non usare un cast TypeScript come verifica di struttura. Il codice corrente dell'opera è facoltativo come criterio aggiuntivo se disponibile tramite UUID e identificato come corrente.

**Semantica:** seriale completo confrontabile senza distinzione maiuscole/minuscole, spazi esterni ignorati, trattini e zeri preservati; eventuali corrispondenze parziali indicate esplicitamente. Titoli con ricerca testuale senza accenti coerente con l'archivio opere, senza alterare le stringhe registrate. Nessuna ricerca fuzzy sui seriali in prima fase. Una corrispondenza esatta può essere evidenziata, non aperta automaticamente. Controllare le stringhe storiche una volta per aggiornamento dei record, non a ogni rendering. Query di lettura ipotetica listCertificateMatches(criteria) con lista di ID, campi corrispondenti e conteggio; non è un contratto approvato. Se i volumi lo richiedono, misurare prima di proporre paginazione o query SQLite. Nessuna dipendenza obbligatoria dalle altre schede.

## Offline, privacy, impostazioni, localizzazione

Tutto locale, senza account, abbonamenti, cloud, telemetria o invio automatico. Le query restano temporanee e vengono azzerate al cambio/restore dell'archivio; nessuna nuova impostazione necessaria inizialmente. Non cercare note, indirizzi, intestatari o altri campi personali in fase minima; eventuale estensione a intestatario richiede consenso del proprietario e revisione privacy. Le anteprime possono mostrare dati personali già presenti: aprirle soltanto su azione deliberata. Nessuna verifica su Internet, link pubblici, QR o funzione di validazione da parte di terzi. Il nome dell'artista e i titoli sono dati originali, non stringhe da tradurre automaticamente.

Interfaccia nella lingua del sistema o override esistente, fallback inglese; etichette nuove nelle cinque lingue supportate, date localizzate per visualizzazione, criterio anno su date canoniche. La lingua del documento già emesso (it/en/bilingual) resta quella registrata, indipendente dalla UI. Unicode, seriali e codici non vengono riscritti.

## Import/export e compatibilità

I CSV dei certificati rimangono **integrali** e indipendenti dai filtri; conservare le colonne e lo snapshot originale, niente export filtrato nuovo in fase minima. Non promettere ricostruzione di file PDF dal CSV né importazione di documenti esterni. Backup completo SQLite + media, cartelle liberamente scelte, verifica e restore esistenti intatti; nessun indice o file aggiuntivo da salvare. Compatibilità con archivi precedenti e snapshot privi di titolo/codice, tipi non validi, testo Unicode e documenti emessi prima dei campi lingua/luogo: usare solo informazioni effettivamente presenti. Nessun aggiornamento storico dei dati, contatori o fotografie. Non alterare API/MCP, schema o dipendenze. Stessa esperienza Windows/macOS/Linux da verificare su installazioni reali.

## Alternative, complessità, rischi

- **Nessuna modifica:** registro cronologico e ricerca nel CSV locale con Calc, consultando snapshot_json quando necessario. Adeguato per pochi certificati; CSV contiene dati personali, non condividerlo indiscriminatamente.
- **Opzione ridotta:** solo ricerca del seriale, costo basso, da preferire se le richieste reali citano sempre il numero. Non risolve il problema del titolo storico.
- **Opzione estesa descritta sopra:** complessità bassa-media senza schema; richiede parsing prudente, etichette di provenienza e test di accessibilità. Paginazione o ricerca backend solo dopo misure, complessità maggiore.
- **Rischi:** risultati storici confusi con dati attuali, seriali simili e omonimie, esposizione dei dati personali, JSON malformato, rallentamenti su archivi grandi, sovrapposizione con UX-060. Mitigazioni: distinguere fonte dei campi, nessuna selezione automatica, niente campi privati aggiuntivi, limiti e misurazioni, riuso della preview. Un record presente non dimostra autenticità o possesso attuale.

## Criteri di accettazione proposti (non test eseguiti)

- Un seriale esatto individua il relativo UUID senza scritture; un prefisso o seriale parziale non viene spacciato per un'identificazione univoca.
- Un certificato emesso con titolo A, opera oggi B, è rintracciabile da A e B, con etichette corrette e snapshot originale invariato. Codice storico presente rintracciabile, assente dichiarato.
- Due documenti della stessa opera e opere con titoli uguali restano distinti; non viene scelto o ristampato nulla automaticamente.
- Snapshot JSON malformato, array, null e campo non stringa non interrompono ricerca o render; seriale e data restano consultabili, nessun dato fittizio.
- Ricerca vuota ripristina elenco completo e ordine previsto; ricerca senza risultati indica chiaramente il filtro attivo. Dopo preview, ritorno e focus coerenti; dopo restore nessun risultato stantio.
- Nessuna variazione di seriali, snapshot, contatori, media, registri CSV, schema e backup dopo ricerca/preview. Nessuna rete. Verificare anche locale non supportato e override, documenti bilingui, date al cambio d'anno, caratteri accentati e nomi Unicode.
- Prove con 50, 500 e 5.000 certificati sintetici: misurare ricerca, rendering e uso tastiera prima di fissare soglie prestazionali. Validazione con utenti: recuperare un documento dal seriale o dal titolo precedente senza consultare più schede; confrontare tempo, errori di identificazione e passaggi con registro e CSV attuali.

## Priorità e fasi, decisioni riservate al proprietario

**Requisiti certi:** indipendenza, offline-first, nessun account, dati esportabili e backup a cartella scelta; immutabilità di certificati già emessi; funzioni attuali di anteprima/stampa e CSV non sostituite. **Ipotesi:** frequenza delle richieste storiche, necessità di trovare documenti per titoli non più correnti, sufficienza dei dati già presenti e della ricerca temporanea. **Non è stata provata una perdita di documenti.**

Fase 0: chiedere a 4–6 professionisti (artisti, assistenti e archivisti) quali riferimenti ricevono nelle richieste di ristampa, quanti certificati gestiscono e se il CSV basta; confrontare recuperi su dataset sintetico. Fase 1 eventuale: ricerca seriale e titoli/codice esistenti, distinzione esplicita delle fonti, test offline e accessibilità. Fase 2 soltanto su evidenza: filtro anno, ottimizzazione prestazionale, eventuale criterio privato di intestatario con revisione separata. Nessuna fase autorizzata da questo documento.

**Domande aperte:** è sufficiente il seriale? Va mostrato il titolo storico come colonna primaria e il corrente solo quando diverso? Serve anno o intervallo di date? Si vuole mai cercare per intestatario, con quali limiti? La stampa deve restare esattamente nel percorso attuale? Se nessuno riscontra il bisogno, conservare questa scheda come studio senza sviluppo.

**Perimetro della ricognizione:** solo Markdown in questa cartella, PR #14 documentale in bozza verso main, nessun merge; il filone tecnico resta intatto.
