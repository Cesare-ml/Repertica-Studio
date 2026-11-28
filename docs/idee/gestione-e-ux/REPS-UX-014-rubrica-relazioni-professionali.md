# REPS-UX-014 — Rubrica delle relazioni professionali e storico dei contatti trasversale

- **ID stabile:** REPS-UX-014
- **Data della ricognizione:** 2026-11-21
- **Stato:** Proposta — da valutare
- **Filone:** Gestione e UX
- **Priorità proposta:** P1–P2, da validare con utenti reali
- **Perimetro:** esclusivamente analisi e documentazione. Nessuna funzionalità approvata, nessuna assegnazione allo sviluppo.

## Problema verificato e destinatari

Il branch `main` dispone di `collectors` con nome, email, telefono, indirizzo e note, relazioni con opere acquistate e vendite; le spese hanno il campo testuale `supplier`. Non esiste, nei modelli pubblici consultati, una **rubrica trasversale di controparti con ruoli multipli e uno storico di relazioni non necessariamente commerciali**. Un curatore può essere anche collezionista; una galleria può fare da intermediario, prestare una sala e organizzare una mostra; un corniciaio è fornitore ma non acquirente. Inserire tutti in «Collezionisti» o duplicarne i dati in note e fornitori confonde il significato dei record.

Destinatari: artisti indipendenti che dialogano con curatori, artigiani e gallerie; studi che affidano lavori all'esterno; piccole gallerie e archivi che coltivano contatti non legati a una vendita immediata. **La domanda effettiva è un'ipotesi**: occorrono esempi d'uso e interviste, non basta l'elenco dei ruoli teoricamente possibili.

### Confine preciso con ciò che è già previsto

- **[UX-002](REPS-UX-002-trattative-prenotazioni.md)** registra offerte, riserve e attività *dentro una trattativa relativa a un'opera*: questa scheda riguarda il **rapporto continuativo con una persona/organizzazione**, anche senza trattative né opere collegate. Quando esiste una trattativa, la sua cronologia rimane autorevole: nessuna copia automatica di eventi.
- **[UX-012](REPS-UX-012-affidamenti-conto-vendita-rendiconti.md)** riguarda condizioni degli accordi di conto vendita e rendiconti, non l'anagrafica generale; una controparte potrà essere collegata a un accordo solo in futuro e con regole esplicite.
- **[UX-001](REPS-UX-001-provenienza-movimentazioni.md)** registra custodia, luoghi e movimenti delle opere; **[UX-009](REPS-UX-009-progetti-espositivi-checklist.md)** pianifica eventi: la rubrica descrive *chi* è il soggetto, senza creare movimenti o mostre.
- **[Issue #13](https://github.com/Cesare-ml/Repertica-Studio/issues/13)** tratta spese, fornitori e documenti economici; qui non si introduce un nuovo registro acquisti. **[TECH-001](https://github.com/Cesare-ml/Repertica-Studio/pull/16)** studia il formato portabile di tutti i dati: questa proposta ne specifica solo l'esperienza gestionale, senza modificare il filone tecnico.
- **Già presente:** un collezionista è una persona/entità collegata a vendite; l'eventuale evoluzione della rubrica **non deve cancellare, riassegnare o alterare silenziosamente** quei collegamenti o i certificati storici.

## Scenari d'uso concreti

1. **Curatrice e mostra:** un'artista registra la curatrice con organizzazione, ruolo, contatto e nota della telefonata. Due mesi dopo può trovare i precedenti senza crearle una vendita o una trattativa fittizia.
2. **Galleria con ruoli multipli:** un referente di galleria può essere sia interlocutore per una mostra sia potenziale acquirente. La scheda mostra ruoli sovrapponibili; non duplica la stessa persona in diverse rubriche.
3. **Fornitore:** il laboratorio di cornici compare come fornitore in più spese. In futuro può essere collegato alla rubrica tramite conferma; i fornitori testuali preesistenti restano **immutati** fino a una scelta deliberata.
4. **Passaggio di consegne interno:** il titolare consulta un promemoria locale «richiamare entro venerdì» dopo una conversazione. Nessuna notifica a terzi, nessuna lettura di email o calendario.
5. **Richiesta di esportazione o correzione:** un contatto chiede quali recapiti sono conservati; l'utente può esportare la sua scheda e distinguere informazioni documentali da note riservate, senza fingere di automatizzare tutti gli obblighi normativi.

## Flussi UX proposti

**Ingresso leggero.** Dalla navigazione «Relazioni» oppure dalla scheda opera/collezionista: **Nuovo contatto** → tipo *persona* o *organizzazione*, nome, ruolo/i facoltativi, almeno un dato identificativo utile; per il solo nome non chiedere email o telefono obbligatori. La distinzione persona/organizzazione è modificabile con avviso se ha relazioni già collegate.

**Scheda unificata.** Intestazione con nome, eventuale organizzazione, ruoli non esclusivi (*collezionista, galleria, curatore, fornitore, restauratore, trasportatore, altro*), recapiti e preferenza di contatto **dichiarata**, non inferita. Pannelli separati: *Dati*, *Relazioni*, *Note e interazioni*, *Collegamenti*. I collegamenti esistenti a vendite o certificati sono **sola lettura** e rimandano alle schermate originali; non si trasferiscono con un clic.

**Registra interazione manuale.** Tipologia (telefonata, incontro, email annotata, visita, altro), data anche solo giorno, breve sintesi, argomento, eventuale opera/progetto/accordo referenziato, prossima azione e data suggerita. Nessuna cattura automatica del contenuto dei messaggi, nessuna promessa di reminder a programma chiuso. Se l'interazione appartiene a una trattativa UX-002, preferire un **riferimento al record originario**, mai una seconda cronologia divergen­te.

**Panoramica del lavoro.** Elenco con ricerca per nome/organizzazione/ruolo e filtri *da ricontattare*, *senza recapito*, *senza attività da X mesi*, *con rapporti attivi*. Le scadenze sono locali, mostrate quando l'app viene aperta. Non trasformare «inattività» in punteggio delle persone o marketing automatico.

**Duplicati e correzioni.** Segnalazione non bloccante per contatti apparentemente omonimi; confronto fianco a fianco e proposta di unione **solo manuale**, con riepilogo delle relazioni coinvolte e undo/rollback prima di confermare. Non fondere per sola email o nome, che possono essere condivisi. In caso di ambiguità, mantenere record separati.

**Accessibilità.** Tutti i pannelli e le azioni disponibili via tastiera e screen reader; etichette, stato focus, errori non veicolati dal solo colore; non esporre recapiti sensibili nei tooltip involontari. Navigazione coerente con l'app senza richiedere una UI di tipo CRM aziendale.

## Dati necessari — ipotesi descrittive, non schema approvato

- `party`: UUID, `kind` (persona/organizzazione), nome visualizzato, denominazione facoltativa, recapiti etichettati, indirizzo, lingua preferita facoltativa, note private, creato/aggiornato; evitare di raccogliere data di nascita, identificativi fiscali o altre informazioni non necessarie.
- `party_role`: UUID parte, ruolo codificato, data inizio/fine facoltativa, nota. **Più ruoli per parte**, senza trasformare automaticamente il ruolo «collezionista» in una vendita.
- `party_relation`: collegamento fra parti (persona→organizzazione, referente di, collaboratore di), eventuali date e natura; prevenire cicli incoerenti solo dove la tipologia lo richieda.
- `interaction`: UUID, parte, data, tipo, sommario, note, riferimento opzionale a opera/trattativa/mostra/affidamento, stato e scadenza di una prossima azione. L'annotazione manuale non dimostra l'effettiva ricezione di email o l'esito di un contatto.
- `legacy_collector_link`: riferimento stabile e univoco alla riga `collectors.id` **solo dopo scelta della strategia di compatibilità**. Non duplicare email/indirizzi come copie autonome non sincronizzate senza una regola esplicita. Documentare preferenza per una singola fonte di verità nel modello futuro.
- Campi necessari a migrazioni e relazioni sempre con UUID e vincoli referenziali; storico di modifiche rilevanti e unione contatti possibilmente rettificabile, senza alterare snapshot dei certificati.
- Le note libere restano private per impostazione predefinita; eventuali allegati futuri richiedono schema media, limiti di dimensione e backup completi, non URL caricati automaticamente.

## Impostazioni, offline, multilingua e privacy

- **Impostazioni facoltative:** ruoli predefiniti modificabili/etichette personalizzate, intervallo suggerito di ricontatto, visibilità delle note nell'export, ordinamento; evitare configurazione iniziale obbligatoria o profili utente. Non introdurre account.
- **Offline-first:** SQLite locale e archivio privato; nessuna sincronizzazione cloud, import automatico dei contatti del sistema, apertura URL esterni, invio email, telefonate o notifiche desktop senza autorizzazione futura esplicita. Le attività restano disponibili senza Internet; scadenze superate ricalcolate all'apertura.
- **Multilingua:** lingua del sistema; IT/EN/ES/FR/DE con fallback inglese e override locale coerente con `docs/ECONOMY-I18N.md`. Ruoli codificati con chiavi invarianti e traduzione UI; ruoli personalizzati, nomi e note restano nella lingua scritta.
- **Privacy:** raccolta minima, niente profilazione o scoring, tracciamento email, rubriche cloud, inoltri invisibili. Schermate e documenti condivisibili devono escludere per default note interne e contatti privati; l'utente sceglie esplicitamente quali campi esportare. Non definire automaticamente un recapito «consenso al marketing»; eventuale registrazione di autorizzazione necessita fonte, data e finalità, da validare giuridicamente. Backup correnti in chiaro: esporre avviso coerente anche per la rubrica.
- **Conflitti d'identità:** l'utente può mantenere due soggetti omonimi; la relazione con opere o vendite non costituisce verifica di identità o titolarità legale.

## Import/export e compatibilità

- **Esportazione leggibile**: CSV UTF-8 (UUID, nome, ruoli, organizzazione, recapiti selezionati), e CSV separato delle interazioni **solo con conferma**; distinguere sempre esportazione completa per portabilità da un dossier selettivo che omette dati riservati. Gli attuali sei CSV integrali restano disponibili e inalterati.
- **Possibile round-trip**: JSON versionato con UUID, ruoli, riferimenti e provenienza del dato. L'import futuro richiede anteprima dei contatti duplicati, scelta per campo, validazione referenziale, registro degli esiti e rollback; non creare vendite, spese o eventi durante il solo import contatti.
- **Formati esterni**: vCard solo come opzione da valutare per recapiti basilari, segnalando la perdita di ruoli, note di attività e collegamenti; **non** scambiare vCard per backup completo. Allineamento solo concettuale con TECH-001; nessun vincolo a provider esterni.
- **Dati legacy**: preservare `collectors.id`, vendite, certificati e registri spese. Per l'MVP di valutazione evitare migrazione automatica di tutti i collezionisti in parti generiche; preferire collegamenti espliciti e un piano successivo verificato. I fornitori testuali non diventano organizzazioni dedotte; omonimie non implicano identità.
- **Backup/ripristino:** tutte le nuove relazioni e cronologie incluse in copie complete a cartella liberamente scelta, senza alterare snapshot storici; migrazioni additive per archivi precedenti, avviso che versioni vecchie non apriranno schemi futuri. Windows/macOS/Linux, Unicode, ordinamenti e date indipendenti dalla locale.

## Complessità, dipendenze, rischi e alternative

- **Complessità stimata:** media per rubrica + ruoli + interazioni manuali; alta per deduplicazione, migrazione dell'anagrafica `collectors`, rettifiche e import idempotente.
- **Dipendenze funzionali:** UUID, vincoli SQLite, backup e i18n; integrazione con le schede UX-001/002/009/012 **solo se approvate separatamente**. Nessun server, SDK di posta o rubrica OS necessari.
- **Rischi:** CRM troppo pesante per artisti singoli; dati personali e note improprie; rottura dei collegamenti alle vendite; merge distruttivi; duplicazione delle attività della trattativa; contatto invadente non autorizzato.
- **Mitigazioni:** ruoli opzionali, ricerca accessibile, dati minimi, nessun merge implicito, separazione delle fonti autorevoli e blocco di funzioni di outreach, anteprima export/merge e prove di recupero backup.
- **Alternativa minima:** migliorare l'anagrafica esistente con etichette/ruoli e schede note senza introdurre una nuova `party`; però la distinzione fra «collezionista» e «fornitore/curatore» rimarrebbe semanticamente ambigua.
- **Alternativa senza sviluppo:** usare rubrica locale esterna e note nelle schede attuali. È più economico, ma manca il legame affidabile alle opere e produce duplicazioni.

## Criteri di accettazione ipotetici

1. Una persona può essere insieme *collezionista* e *curatore*, collegata a un'organizzazione, senza duplicare i suoi recapiti e senza creare una vendita.
2. Aggiungere una telefonata annotata e una scadenza non modifica lo stato dell'opera, gli incassi, le trattative o i totali mensili.
3. La scheda di un collezionista storico mantiene esattamente gli stessi UUID e collegamenti con vendite/certificati dopo ogni migrazione; nessun documento storico viene riscritto.
4. Un contatto senza email o numero può essere salvato, ricercato, esportato e ritrovato offline.
5. Due omonimi non vengono fusi automaticamente; un'operazione di unione mostra prima tutte le relazioni e, se fallisce, non lascia riferimenti orfani.
6. Le attività delle trattative UX-002 (se presenti in futuro) restano uniche e referenziate, non duplicate silenziosamente nel diario generale.
7. Export filtrato esclude note/recapiti privati selezionati, mentre una futura esportazione completa e un ripristino verificato conservano l'intero archivio e i legami.
8. La rubrica resta fruibile completamente via tastiera e screen reader con locale OS, override e fallback EN, su Windows/macOS/Linux, anche senza rete.

## Fasi di valutazione ed eventuale evoluzione — non autorizzate

- **A — Verifica del bisogno:** raccogliere 4–6 casi reali presso artisti/gallerie, inventariare ruoli usati e osservare quanto siano sufficienti le note dell'anagrafica attuale.
- **B — Modello minimo:** rubrica per soggetti esterni e ruoli multipli, collegamento opzionale e conservativo agli UUID dei collezionisti; ricerca, niente fusione automatica.
- **C — Relazioni e follow-up:** collegamenti persona/organizzazione, interazioni inserite manualmente, promemoria visibili all'apertura, deduplica assistita.
- **D — Interoperabilità:** export selettivo, JSON round-trip, valutazione vCard e casi di unione complessi. Integrazioni con altre schede solo dopo separata approvazione.

## Certezze, ipotesi e domande per il proprietario

- **Certo su `main`:** esiste `Collector` con recapiti e note, `Expense.supplier` testuale, relazioni UUID collezionista–vendita–opera, certificati immutabili, SQLite offline, backup e sei CSV; l'app usa lingua del sistema con fallback inglese. Nessuna rubrica trasversale è dichiarata nella documentazione attuale.
- **Ipotesi:** utenti reali gestiscono abbastanza controparti non acquirenti da rendere utile una rubrica dedicata; follow-up manuali fuori dalle trattative sono frequenti; la distinzione fra persona e organizzazione è sostenibile.
- **Domande da decidere personalmente:** (1) il primo pubblico include anche gallerie e archivi o soprattutto artisti singoli? (2) meglio estendere «Collezionisti» o introdurre un'entità «Soggetto» mantenendo compatibilità? (3) i rapporti con fornitori richiedono solo il nome o anche persone referenti? (4) le interazioni fuori da una trattativa sono abbastanza importanti? (5) quali campi privati devono essere esclusi per default dalle esportazioni? (6) è accettabile rimandare unione duplicati e vCard per ridurre la complessità?

**Esito: Proposta — da valutare. Nessuna implementazione, approvazione, assegnazione o modifica al software.**
