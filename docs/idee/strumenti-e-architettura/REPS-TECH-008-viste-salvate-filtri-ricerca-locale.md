# REPS-TECH-008 — Viste salvate e ricerca strutturata nel singolo archivio

- **Data ricognizione:** 2026-11-21
- **Stato:** **Proposta — da valutare**
- **Priorità ipotetica:** **P1–P2** (prima validare volumi di catalogo e frequenza d'uso)
- **Ambito:** strumenti di ricerca e consultazione **del solo archivio locale attivo**; non è un incarico di sviluppo.

## Problema osservato e destinatari

Sul branch `main`, `src/app/archive-filters.ts` implementa per le opere una ricerca libera in memoria, un filtro per stato, uno per collezionista e ordinamenti «recenti/titolo/anno»; per i collezionisti una ricerca testuale. Il comando Rust `list_artworks` restituisce tutte le opere ordinate per aggiornamento; l'interfaccia filtra l'array ricevuto. Funziona ragionevolmente per una prima MVP ma non rappresenta interrogazioni professionali ripetibili come «opere disponibili, su carta, anteriori al 2020, prive di fotografia» o «opere vendute al collezionista X con certificato emesso».

La necessità di filtri combinati, di un elenco di viste salvate e di risultati riproducibili è **un'ipotesi di prodotto**, non una richiesta verificata da utenti. Destinatari possibili: artisti con molte opere, assistenti di studio, galleristi e archivisti che selezionano opere per visite, cataloghi, inventari e controlli. Per piccoli cataloghi la ricerca semplice deve restare immediata.

## Confini e mancata duplicazione

- **Già presente in `main`:** ricerca libera di titolo, codice, anno, tecnica, supporto, ubicazione, descrizione e nome dell'acquirente; filtri per stato/collezionista; ordinamento; esportazione CSV completa di cinque registri. Non riproporli come novità.
- **[TECH-004](REPS-TECH-004-ricerca-federata-archivi-locali.md):** cerca in *più backup o archivi* in sola lettura; questa proposta interroga **unicamente l'archivio in uso** e non aggiunge fonti, fusioni o sincronizzazione.
- **[TECH-001](REPS-TECH-001-pacchetto-dati-aperto.md):** interscambio semantico completo, non un motore di ricerca. La definizione di una vista non è una copia dei dati.
- **[UX-007 nella PR #14](https://github.com/Cesare-ml/Repertica-Studio/pull/14):** crea dossier da presentare; un eventuale filtro salvato serve solo a trovare i record, non pubblica né impagina informazioni. **[UX-011](https://github.com/Cesare-ml/Repertica-Studio/pull/14)** riguarda relazioni artistiche tra opere, non criteri di ricerca.
- **Roadmap `docs/MVP.md`:** menziona genericamente «ricerche avanzate», ma non specifica le viste, la semantica dei filtri o le condizioni di privacy. Questa scheda approfondisce quel bisogno senza modificarne priorità approvate.
- Nessun server MCP, endpoint HTTP, funzione di IA, algoritmo di classificazione o ricerca su Internet è implicato.

## Scenari e flussi UX illustrativi

1. **Ricerca quotidiana semplice:** la barra attuale continua a funzionare. Un pulsante facoltativo «Filtri avanzati» apre un pannello con criteri espliciti, contatore dei risultati e azione «Azzera». L'utente può sempre tornare alla vista standard senza perdere i dati.
2. **Combinazione di criteri:** artista/archivista seleziona *stato disponibile* AND *tecnica contiene «acquerello»* AND *anno numerico 2015–2023* AND *senza foto*. La vista mostra immediatamente numero di risultati, criteri attivi in forma testuale e possibilità di rimuovere singoli criteri; i risultati non sono modificati.
3. **Dati incompleti:** criteri «senza immagine», «senza ubicazione», «anno non specificato», «certificato presente/assente» aiutano a individuare lacune del catalogo **senza correggerle automaticamente**. Anni liberi come «circa 2000» non vanno trasformati forzatamente in 2000: si distinguono «anno esatto», «anno non interpretabile» e «non indicato».
4. **Vista nominata:** «Acquerelli disponibili» può essere salvata, rinominata, duplicata, ordinata ed eliminata. Una vista memorizza *criteri*, non risultati congelati; all'apertura mostra la data di esecuzione e i record correnti. Avverte che il numero può cambiare se l'archivio viene modificato.
5. **Riapertura e recupero:** all'avvio resta predefinita la ricerca semplice; l'utente sceglie una vista dal menu. Se un campo di una vista non è più supportato da una versione futura, mostrarla come «da aggiornare», indicare il criterio incompatibile e impedire risultati silenziosamente incompleti.
6. **Riservatezza:** viste basate sul nome di un collezionista sono segnalate come contenenti criteri personali; nessuna visualizzazione pubblica, notifica sul desktop, invio esterno o storico automatico delle parole cercate.
7. **Accessibilità:** filtri con etichette persistenti, comando da tastiera, focus ordinato, risultati annunciati in modo non invasivo e stati leggibili anche senza colore.

## Specifica funzionale proposta

**Campi fase iniziale:** titolo, codice, stato, tecnica, supporto, ubicazione, presenza foto principale, anno testuale/esatto, data di aggiornamento, collezionista acquisitore, presenza certificato. Le eventuali proprietà delle foto aggiuntive, incassi parziali o relazioni future richiedono uno studio separato delle definizioni e prestazioni, non entrano automaticamente nel primo set.

**Predicati:** uguaglianza/elenco per stato, contiene/inizia per testo, presente/assente per campi nullabili, intervalli inclusivi per date o numeri realmente strutturati; `AND` fra filtri in prima fase, `OR` soltanto dentro un elenco dello stesso campo. Una ricerca libera non deve trasformarsi in SQL arbitrario. Etichette comprensibili («Contiene», «È vuoto», «Tra») e anteprima della logica in lingua corrente. La ricerca per nome cliente deve risolvere l'ID del collezionista e distinguere omonimie.

**Ordinamento:** prevedere stabilità deterministica con UUID come criterio finale per evitare risultati che cambiano ordine a parità di titolo/anno. Conservare «recenti/titolo/anno» per compatibilità. In futuro valutare paginazione o cursori *solo* se le misure su dati realistici lo giustificano.

**Semantica dei risultati:** criteri applicati all'archivio attivo al momento dell'interrogazione, conteggi coerenti con gli stessi criteri, nessun risultato «fresco» da cache non aggiornata dopo una modifica. Mostrare esplicitamente che le viste sono dinamiche, non report storici certificati.

## Modello dati e ipotesi API — solo descrizione, NON codice

Una possibile tabella locale `saved_views` con `view_id` UUID, `name`, `scope=artworks`, `filter_schema_version`, `criteria_json`, `sort_json`, `created_at`, `updated_at`, `is_default` (falso all'inizio). `criteria_json` rappresenta un albero di **predicati tipizzati in lista positiva di campi/operatori**, non frammenti SQL, espressioni eseguibili o codice JavaScript; profondità, lunghezza e numero criteri limitati.

Interfacce concettuali: `validateCriteria(criteria) -> ValidationReport`; `previewQuery(criteria, order, page?) -> {items, total, warnings}`; `saveView(name, criteria, order) -> ViewReference`; `openView(viewId) -> ValidatedCriteria`; `deleteView(viewId) -> Confirmation`. Le operazioni di lettura non alterano opere, vendite, clienti o certificati. L'interprete dei filtri deve verificare campo, tipo, ambito, operatore e valori **prima** di compilare eventuali query parametrizzate lato Rust.

Per una prima fase è possibile riutilizzare la ricerca in memoria se i cataloghi restano piccoli; per volumi maggiori valutare query SQLite parametrizzate e indici mirati. **Non** si propone ora alcuna migrazione né API implementata. Mantenere i contratti correnti di `list_artworks` finché non sia dimostrata la necessità di nuove query.

## Offline, sicurezza, privacy, portabilità e impostazioni

- Operatività integralmente **offline-first** su Tauri 2 e SQLite locale, senza account, cloud, telemetria, abbonamenti, servizi esterni, motori di ricerca remoti o permessi di rete.
- Le viste devono rimanere nell'archivio **locale**, incluse nel backup integrale e nel ripristino soltanto dopo una migrazione esplicita e verificata; nessun percorso assoluto o credenziale nei criteri. In questa prima fase evitare ricerche persistenti e registri delle query eseguite.
- Dati personali: un nome o ID di collezionista può apparire nel criterio; trattare anche la definizione della vista come informazione privata. Conferma esplicita per qualsiasi **futuro** export delle viste o condivisione; i cinque CSV attuali restano **integrali**, indipendenti dai filtri, come documentato in [DATA-PORTABILITY](../../DATA-PORTABILITY.md). Non introdurre di nascosto esportazioni filtrate. Eventuale export di *sole definizioni delle viste* in JSON versionato e documentato è una scelta da valutare, non una sostituzione dell'esportazione completa dei dati.
- Le definizioni importate, se previste, si validano e si mostrano in anteprima prima della conferma; nessuna esecuzione di SQL/code esterno. Backup su cartelle liberamente scelte continua a contenere database e media completi; non richiede dipendenze di rete.
- Lingua predefinita da sistema operativo, fallback inglese e override locale per italiano, inglese, spagnolo, francese e tedesco. **I token/nomi tecnici dei campi restano stabili**, le etichette cambiano con la lingua; numeri e date si presentano secondo il locale, ma si interrogano in rappresentazione canonica. Attenzione agli accenti, alle collazioni e alle stringhe non italiane: il filtro attuale usa `toLocaleLowerCase('it')`, che va considerato nel progetto dei confronti e dei test.
- Accessibilità: tastiera, lettori schermo, gestione focus, conteggi, messaggi d'errore localizzabili; i risultati non devono essere codificati soltanto a colori.

## Alternative, dipendenze, rischi e complessità

**Alternative:** mantenere filtri semplici; aggiungere solo due filtri preimpostati per problemi di completezza; introdurre viste soltanto quando richieste da utenti con archivi grandi; usare CSV completi in un foglio elettronico (meno adatto a dati personali e lavoro quotidiano). Per dataset piccoli, una nuova infrastruttura di interrogazione sarebbe sproporzionata.

**Dipendenze potenziali:** definizione tipizzata dei filtri, disponibilità degli indici vendita e certificato, schema SQLite versionato, componenti Angular accessibili, test su build native e migrazioni avanti/indietro. Nessuna dipendenza esterna obbligatoria.

**Rischi:** filtri ambigui su anni testuali; differenze di collazione/locale; ordinamenti non stabili; false impressioni di «report immutabile»; query lente; esposizione dei dati dei collezionisti nella vista salvata; migrazioni future che ignorano predicati; perdita delle viste durante un ripristino; confusione con CSV completi. Mitigare con semantica documentata, validazione, avvisi e test ripetibili.

**Complessità indicativa:** **media**, potenzialmente **medio-alta** solo se paginazione, ricerca full-text o strutture `OR` annidate risultassero necessarie. P1–P2 è una valutazione editoriale, non una roadmap.

## Criteri di accettazione ipotetici (da discutere)

1. Filtri combinati nel singolo archivio producono risultati e conteggi coerenti con i record originali, senza alcuna modifica ai dati.
2. Il comportamento della ricerca semplice e dei CSV integrali resta invariato quando il pannello avanzato è chiuso.
3. Test con campi `NULL`, testo accentato, apici, Unicode, anni liberi/non interpretabili, codici simili, collezionisti omonimi e risultati vuoti; impossibilità di iniettare SQL.
4. Una vista salvata conserva i criteri dopo riavvio, backup e ripristino; risultati aggiornati quando un'opera viene modificata.
5. Vista con criterio obsoleto segnala precisamente l'incompatibilità e non produce risultati troncati mascherati da validi.
6. Test su Windows/macOS/Linux, sola tastiera e screen reader; presentazione corretta in cinque lingue e fallback inglese.
7. Benchmark con cataloghi sintetici di **1.000, 10.000 e 100.000 opere** (numeri per test, non limiti o prestazioni promesse), misurando latenza, memoria, conteggi e necessità di paginazione; nessun obiettivo numerico inventato senza misura.
8. Nessuna richiesta HTTP, trasmissione di query o logging di dati personali.

## Fasi possibili — tutte subordinate ad approvazione

- **Fase 0, ricerca utenti:** raccogliere esempi di selezioni ripetute, dimensioni degli archivi, lingua dei dati e controlli desiderati; decidere se bastano i filtri già disponibili.
- **Fase 1, specifica:** definire predicati minimi, semantica anni/assenze, UX e convenzioni di accessibilità; disegnare viste di esempio non operative.
- **Fase 2, eventuale MVP futuro:** soltanto combinazioni `AND` e viste nominative con validazione/retrocompatibilità.
- **Fase 3, eventuali ottimizzazioni:** query lato database, ordinamenti/paginazione misurati e, solo se utili, estensioni a certi altri registri.
- **Fase 4, opzionale:** formati aperti per esportare le *definizioni* delle viste; nessuna condivisione automatica né riuso implicito per i dossier.

## Requisiti verificati, ipotesi e domande al proprietario

**Verificato nel repository:** ricerca e filtri attuali in `archive-filters.ts`; elenco opere completo da `list_artworks`; cinque CSV completi non filtrati; SQLite locale e backup ripristinabile; la roadmap cita ricerche avanzate ma non viste nominative.

**Ipotesi:** utenti professionali hanno query ripetitive abbastanza complesse da giustificare salvataggio delle viste; le prestazioni della ricerca in memoria potrebbero degradare su dataset molto grandi, senza misure al momento; lo studio desidera salvare le viste nel database anziché nelle sole preferenze UI.

**Da decidere personalmente:** (a) quali query ricorrenti giustificano l'intervento? (b) le viste devono essere trasferibili tra archivi o restare proprietà del singolo archivio? (c) quali campi personali si possono filtrare e come segnalarli? (d) conservare solo `AND` o introdurre `OR` combinato? (e) serve davvero filtrare certificati/vendite o solo opere? (f) i risultati selezionati dovrebbero servire in futuro a dossier, senza cambiare il comportamento dei CSV? (g) quali prestazioni minime sono accettabili su hardware reale?

**Decisione:** nessuna. Non implementare, non aprire issue operative, non unire questa PR e non assegnare attività di sviluppo senza approvazione esplicita del proprietario.
