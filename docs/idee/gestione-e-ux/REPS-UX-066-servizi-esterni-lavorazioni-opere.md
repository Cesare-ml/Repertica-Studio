# REPS-UX-066 — Lavorazioni affidate a laboratori esterni: incarichi, scadenze ed esito dichiarato

- **ID stabile:** REPS-UX-066
- **Data:** 2026-11-23
- **Stato:** Proposta — da valutare
- **Filone:** Gestione e UX
- **Priorità proposta:** P2 **solo se** almeno 3–5 professionisti confermano casi ricorrenti e insufficienza degli strumenti esistenti; altrimenti P3, sola checklist o nessuna funzione
- **Natura:** esclusivamente ideativa e documentale. Nessuna approvazione, assegnazione allo sviluppo, implementazione, issue operativa o autorizzazione al merge.

## 1. Problema, destinatari, fatti e ipotesi

**Scenario professionale plausibile, non esigenza utente già dimostrata.** Chi realizza opere spesso commissiona a un terzo un servizio relativo a un'opera già catalogata: incorniciatura, montaggio, stampa fine-art, fusione in fonderia, trattamento del supporto o produzione di un elemento espositivo. Può dover ricordare cosa è stato concordato, *con chi*, entro quale data, quali prove o approvazioni mancano e se il lavoro è stato dichiarato concluso. Le note libere dell'opera o il registro delle spese non sono un elenco affidabile di **lavori ancora aperti**. Al tempo stesso non sarebbe corretto creare una spesa o spostare fisicamente un'opera quando si registra un semplice incarico.

**Destinatari da verificare:** artisti che affidano lavorazioni a corniciai, stampatori o fonderie; piccoli studi con collaboratori esterni; gallerie che coordinano preparazione e allestimento; archivi che documentano interventi di produzione commissionati dall'autore. Un artista che fa tutto in studio può non averne alcuna utilità.

**Riscontrato con lettura statica del branch `main` il 23/11/2026, tramite connettore GitHub:**

- [README](../../../README.md), [MVP](../../MVP.md) e [modelli Rust](../../../src-tauri/src/models.rs) descrivono opere, fotografie, vendite e spese, ma non un registro autonomo di **servizi di lavorazione affidati a fornitori**.
- [Economia e lingue](../../ECONOMY-I18N.md) distingue le spese economiche per data, categoria, importo, descrizione e `supplier` (testuale) dai prezzi suggeriti e dalle quote delle vendite. **La spesa registrata non attesta che una lavorazione sia stata richiesta o completata.**
- La [scheda dell'opera](../../../src/app/models.ts) conserva `framed` e altri attributi descrittivi, ma una casella «incorniciata» non fornisce un preventivo, una data di consegna prevista, un fornitore o uno storico dell'incarico.
- L'attuale `ArtworkStatus` (`available`, `reserved`, `sold`, `archived`) ha significato **commerciale**: non deve diventare un indicatore di stato dei lavori.
- [Backup verificato](../../MVP.md) ed esportazioni CSV esistono; l'eventuale nuova entità deve rientrare nella portabilità completa e non richiedere rete, account o nuovi servizi esterni.

**Limite dell'evidenza:** l'assenza di un modulo in questi percorsi non è una richiesta reale di utenti né prova che il flusso attuale sia inadeguato. Non è stata eseguita una sessione d'uso né un test nativo.

### Riferimento metodologico, senza confondere ambiti

Le procedure Spectrum di Collections Trust distinguono **intervento conservativo**, **uscita fisica** e **movimentazione** dell'oggetto: [Collections care and conservation](https://collectionstrust.org.uk/spectrum/procedures/collections-care-and-conservation-spectrum-5-0/) e [Object exit](https://collectionstrust.org.uk/spectrum/primary-procedures/object-exit-spectrum-5-0-primary-procedures/). Sono riferimenti per mantenere separati fatti e documentazione, **non** una richiesta di conformità a uno standard museale né un modello normativo universale per un piccolo studio. Restauro e trattamento conservativo specialistico restano nel perimetro della [UX-004](REPS-UX-004-rapporti-stato-conservazione.md), non in questo registro generico.

## 2. Confronto anti-doppioni: che cosa NON è questa idea

- **[UX-006 — Incarichi per opere su richiesta](REPS-UX-006-incarichi-opere-su-richiesta.md):** riguarda l'artista che **riceve** un incarico da un committente e il suo brief; UX-066 studia l'artista o la galleria che **affida** a un fornitore una lavorazione su un'opera esistente. Un medesimo progetto potrebbe richiedere entrambe, senza duplicare lo stato o le scadenze.
- **[UX-003 — Costi e margine per opera](REPS-UX-003-costo-e-margine-per-opera.md):** attribuisce importi alle opere; UX-066 riguarda **lo stato del lavoro promesso**, non introduce un secondo registro contabile. Un preventivo non è una spesa, un lavoro completato non è un pagamento.
- **[UX-008 — Scorte materiali](REPS-UX-008-scorte-materiali-consumi.md):** quantità di materiali in deposito, non un servizio acquistato all'esterno.
- **[UX-014 — Rubrica professionale](REPS-UX-014-rubrica-relazioni-professionali.md):** identità e relazioni dei fornitori. UX-066 può inizialmente usare una denominazione testuale e **non** deve creare una rubrica parallela o un collezionista fittizio.
- **[UX-001 — Provenienza e movimentazioni](REPS-UX-001-provenienza-movimentazioni.md), [UX-004 — Condizione](REPS-UX-004-rapporti-stato-conservazione.md) e [UX-025 — Istruzioni di allestimento/cura](REPS-UX-025-istruzioni-allestimento-cura-opera.md):** una richiesta di incorniciatura **non prova** passaggio di custodia, uscita dal deposito, controllo dello stato fisico o trattamento conservativo. Collegamenti futuri soltanto se ciascun registro esiste ed è confermato.
- **[UX-026 — Diario di produzione](REPS-UX-026-diario-lavorazione-opera-fasi-datate.md):** descrive *attività svolte e documentate dall'artista*; UX-066 gestisce un **impegno esterno ancora da eseguire o confermare**, con un fornitore e una scadenza. Non creare copie di passaggi nel diario.
- **[UX-044 — Attrezzature dello studio](REPS-UX-044-attrezzature-studio-manutenzioni-garanzie.md):** manutenzione di strumenti durevoli dello studio, non interventi su opere.
- **[UX-046 — Consegne post-vendita](REPS-UX-046-consegne-spedizioni-post-vendita-checklist.md):** trasporto dell'opera al compratore; non coincide con incarico a stampatore, corniciaio o fonderia.
- **Filone tecnico, [PR #16](https://github.com/Cesare-ml/Repertica-Studio/pull/16):** TECH-016 riguarda documenti locali riutilizzabili; TECH-033 schede obsolete; TECH-001 export completo; TECH-008 ricerca e filtri. Sono possibili dipendenze **concettuali non approvate**. Nessun file in `docs/idee/strumenti-e-architettura/` è toccato.

**Escluso:** ordini elettronici a fornitori, invio automatico di mail, firma di contratti, gestione fiscale, pagamenti, obblighi IVA, magazzino, fatturazione, gestione di dipendenti, certificazione di qualità, conservazione scientifica, spedizione automatica, cambio automatico di disponibilità o proprietà e marketplace.

## 3. Casi d'uso realizzabili e distinzione fra promessa e fatto

1. **Cornice per una tela disponibile.** L'artista annota «Cornice noce, battuta 3 cm» e il preventivo *indicativo* del laboratorio, con data prevista di fine lavori. Quando riceve notizia della fine, segna «Lavorazione conclusa — dichiarato»; solo una verifica successiva può confermare `framed` nella scheda opera. Il lavoro non cambia il prezzo né la disponibilità.
2. **Stampa fine-art in laboratorio.** Per una fotografia già registrata viene richiesto un campione, poi una stampa definitiva. Stato «in revisione» o nota sull'approvazione del campione. Nessun esemplare aggiuntivo o numero di tiratura viene creato automaticamente (rimane distinta [UX-005](REPS-UX-005-edizioni-multipli-serie-numerate.md)).
3. **Scadenza mancata.** La consegna stimata passa; in elenco appare «Data prevista superata — stato da verificare», non «fornitore in ritardo» come fatto accertato. L'artista può aggiornare la data senza riscrivere retroattivamente la precedente se era confermata.
4. **Lavorazione annullata.** Il laboratorio non prende più in carico il lavoro; l'incarico viene segnato annullato con motivazione facoltativa. Non crea uno storno economico: una spesa già registrata resta tale finché corretta nel suo registro.
5. **Opera venduta ma ancora da preparare.** Il servizio può essere collegato a un'opera marcata `sold`. L'incarico non riscrive la vendita né dichiara avvenuta la consegna all'acquirente.
6. **Tre opere dallo stesso corniciaio.** Nella fase minima restano tre incarichi autonomi, chiaramente collegati ai rispettivi codici opera. Una «commessa al fornitore» multiopera è un'**alternativa successiva**, non una relazione automatica fra schede.
7. **Restauro specialistico.** Se il caso è un trattamento conservativo post-produzione, la scheda invita ad annotarlo secondo un futuro registro di conservazione UX-004, **non** lo semplifica a una casella «riparato».

## 4. Flussi UX proposti e stati (solo ipotesi)

### Versione minima: una sezione facoltativa nella scheda opera

`Opere → [opera] → Lavorazioni esterne → Nuova lavorazione`. Campi visibili all'inizio: **tipo o titolo libero**, **laboratorio/fornitore come testo**, **data richiesta** e **data prevista facoltativa**; nota su cosa si deve consegnare/ricevere e **stato**. Non aprire un nuovo modulo nell'onboarding né obbligare gli artisti ad annotare il fornitore. L'elenco nell'opera mostra «Da definire / Richiesto / In esecuzione / Esito da verificare / Concluso / Annullato». Codici stabili separati dalle etichette tradotte.

**Passaggio di stato manuale.** `Apri → Aggiorna stato → Conferma`, con un riepilogo «Hai dichiarato questa fase; non significa pagamento, consegna fisica o cambiamento di proprietà». `Concluso` richiede una data effettiva **dichiarata dall'utente**, eventualmente solo giorno, e una nota facoltativa sulla fonte («confermato per telefono», «ritirato di persona»); non richiede ricevuta, email o firma digitale.

**Vista trasversale SOLO se la navigazione per singola opera non basta.** `Lavorazioni esterne` o un filtro in «Opere»: lavori aperti, ordinati per data prevista, ricerca per nome fornitore/testo/codice opera, con evidenza distinta per *scadenza prevista trascorsa*. Pulsante «Apri opera» e «Apri incarico» sempre presenti. Nessun punteggio di efficienza del fornitore, nessun allarme in background a programma chiuso.

**Prove di usabilità:** in una vista stretta usare righe/pannelli sequenziali, non una Kanban che dipenda da drag-and-drop. Focus visibile e restauro del focus dopo conferma, tastiera completa, testo delle fasi e delle date leggibile da screen reader, non solo colori, zoom al 200%, comunicazioni di errore collegate ai campi.

### Vincoli temporali e di significato

- Data richiesta, promessa e completamento sono **concetti diversi**. `null` non equivale a «oggi»; una data prevista trascorsa non cambia automaticamente lo stato.
- Un incarico «in esecuzione» non significa che l'oggetto si trovi presso il fornitore. La custodia e i movimenti appartengono a UX-001.
- Un preventivo accettato non significa costo pagato; la contabilità registrata resta nella tabella `expenses`. Le percentuali di guadagno delle vendite restano immutabili.
- Il servizio può essere terminato senza che siano disponibili documenti; gli allegati, se introdotti, richiedono esplicita selezione e controllo, non crawling automatico della posta.
- Un'opera può avere più lavorazioni in tempi differenti; una data di conclusione non muta `signed`, `framed`, `final_varnish` o il certificato.

## 5. Dati necessari (ipotesi descrittive, NON schema autorizzato)

**Versione leggera proposta:**
`external_work_order` = `id` UUID, `artwork_id` UUID (FK a opera esistente), `title` o tipo testuale, `supplier_display` testuale, `requested_on` (data facoltativa), `expected_on` (data facoltativa), `completed_on` (data manuale facoltativa), `status` codificato, `work_notes` private facoltative, `created_at`, `updated_at`. Un riferimento opzionale futuro `supplier_party_id` a UX-014 non deve essere inferito dal testo o dal nome del collezionista.

**Eventuali incrementi separati e condizionati:** `external_work_event` (ordine, data dichiarata, tipo di variazione, nota minima, fonte), `estimated_cost_cents` + `currency` e marcatura **preventivo non contabilizzato**, `expense_id` (relazione opzionale, non nuova spesa), riferimenti a file del Media Center/allegati gestiti. Per più opere associate alla medesima commessa, studiare successivamente `external_work_order_item` con costi allocati esplicitamente, **non** replicare un importo su tutte le opere.

**Invarianti:** nessuna riscrittura dei codici/UUID delle opere, `sales`, `certificates`, `sale_allocations`, `expenses` o documenti storici; nessun `collector_id` obbligatorio, nessuna generazione di nuovo cliente per il fornitore; nessun importo dedotto dal coefficiente artista. Un riferimento a opera eliminata deve essere gestito esplicitamente (rifiuto conservativo oppure archivio della relazione), senza cancellazioni a cascata silenziose dei documenti professionali.

**Alternative di modellazione:** iniziare senza nuove tabelle usando note e una checklist stampabile esterna, e misurare se l'impossibilità di filtrare le lavorazioni aperte è davvero un problema. Evitare schemi ricchi di stati prima di aver osservato le parole realmente usate dai professionisti.

## 6. Impostazioni, offline, lingua e privacy

- **Impostazioni:** nessuna necessaria nella prima fase; possibili tipi frequenti (cornice, stampa, fusione, altro) **facoltativi**, ordinamento preferito e vista «aperti» come default soltanto dopo test. Niente timer, notifiche push o automazioni locali non richieste.
- **Funzionamento offline:** dati in SQLite locale dell'archivio attivo; nessun server, rete, account, abbonamento o cloud. L'eventuale lavoro su NAS riguarda solo un backup manuale/verificato scelto dall'utente, non il servizio al fornitore.
- **Multilingua:** lingua UI rilevata dal sistema, fallback inglese e override nelle lingue già supportate IT/EN/ES/FR/DE. Le etichette degli stati sono tradotte a partire da codici stabili; nomi dei fornitori e note mantengono esattamente la lingua scritta. Date in archiviazione con semantica univoca, visualizzazione secondo locale senza interpretare ambiguamente `03/04` in importazione.
- **Privacy e riservatezza:** dettagli di lavorazioni, accordi, costi preventivati, foto non pubblicate e recapiti di terzi sono **interni** per default; non compaiono in brochure, certificati, dossier per clienti o MCP read-only senza un'eventuale separata revisione autorizzativa. Nessun parsing automatico di caselle IMAP, nessun invio a laboratori, niente profilazione dei fornitori. I backup in chiaro richiedono la medesima cautela del resto dell'archivio.
- **Accessibilità:** interazioni pienamente raggiungibili da tastiera, testi non solo cromatici, date etichettate come previste o dichiarate, modali con focus gestito e nomi significativi anche con screen reader.

## 7. Import/export, compatibilità e conservazione

- **Backup:** ogni nuovo record e gli allegati interni, *se introdotti*, compresi nel backup SQLite/media verificato; ripristino che preserva UUID, collegamenti alle opere e fasi dichiarate. Una destinazione arbitraria USB/NAS resta scelta liberamente dall'utente. Non creare backup parziali mascherati da copie complete.
- **Esportazione completa:** scheda in CSV UTF-8 per ordini e (solo eventualmente) eventi, campi espliciti `work_order_id`, `artwork_id`, codice opera per lettura, fornitore, stato, date previste/reali, note riservate, costo stimato separato dall'eventuale `expense_id`; mantenere un formato/versione pubblica coerente con TECH-001/057, senza sostituire il backup integrale. Per condivisioni esterne serve selezione deliberata dei campi; l'export completo **non** è un dossier innocuo da mandare al fornitore.
- **Import futuro solo dopo valutazione:** JSON versionato o CSV mappato con UUID, idempotenza e controllo referenziale prima del commit; ordine che punta a opera inesistente = conflitto esplicito, non creazione silenziosa di una nuova opera o di una spesa. Import/export testati con Unicode (nomi fornitori, descrizioni), date e percorsi non ASCII.
- **Compatibilità:** migrazioni SQLite additive, archivi preesistenti con zero lavorazioni; non generare eventi retroattivi da `framed`, da `expenses.supplier` o da note. Le versioni vecchie dell'app non devono dichiararsi compatibili con schemi nuovi. Windows/macOS/Linux, database locale e stampa esterna facoltativa.
- **Eliminazione e correzione:** distinguere un incarico annullato da un record erroneamente creato; se ci sono eventi collegati o costi riferiti, prima mostrare collegamenti e conseguenze. Evitare cancellazioni orfane o modifiche a spese, vendite, stato commerciale e certificati.

## 8. Complessità, rischi e alternative

**Complessità stimata:** bassa se resta una checklist testuale esterna; **media** per registro leggero locale, filtri, date, validazioni, backup/export; **medio-alta** per storico eventi, fornitori strutturati, multiopera e integrazione spese/documenti. Il costo di manutenzione del modello e delle traduzioni può eccedere il beneficio per i singoli artisti.

**Dipendenze potenziali, non approvate:** UUID opere e SQLite (presenti), backup/esportazione e i18n (presenti); UX-014 fornitori, UX-003 costi attribuiti, UX-001 movimenti, TECH-016 allegati, TECH-033 salvataggi obsoleti **solo se ognuna è approvata autonomamente**.

**Rischi:** proliferazione di stati amministrativi, confusione fra «richiesto», «promesso» e «concluso», doppi conteggi economici, dati privati esposti in PDF, ritenere che il fornitore sia proprietario o custode, collegamenti spezzati dopo cancellazioni, promesse di conformità museale, UI ingombrante per la maggioranza.

**Alternative prima dello sviluppo:** (1) nota strutturata e data prevista nel campo `description`/note operative **senza** cambiare il DB, con limiti chiari; (2) foglio CSV/Calc locale con codice opera e checklist; (3) seguire il lavoro con il fornitore nel proprio calendario e registrare solo la spesa finale; (4) riusare in futuro un modulo di task generico, ma soltanto se esiste davvero e senza forzare il modello. Se questi rimedi bastano, non sviluppare UX-066.

## 9. Criteri di accettazione ipotetici e test da proporre (NON eseguiti)

1. Una scheda opera esistente può avere due lavorazioni esterne indipendenti, una conclusa e una programmata; entrambe rimangono collegate allo stesso UUID e codice.
2. Incarico senza data prevista o senza costo è valido; «non indicato» non appare mai come scaduto o importo zero.
3. Aggiornare data/stato non crea vendite, incassi, spese, quote, variazioni `framed`, cambi di ubicazione, clienti, certificati o messaggi email.
4. Il passaggio «Concluso» richiede conferma dell'utente e non attesta di per sé una movimentazione o un pagamento; una scadenza oltrepassata non conclude né annulla automaticamente nulla.
5. Un preventivo di 120,00 EUR non aumenta la somma `expenses` neppure se l'incarico è completato; un'eventuale spesa collegata resta unica nel suo registro autorevole.
6. Una lavorazione su opera venduta non modifica vendite o collegamenti al collezionista; lo stato commerciale originale resta immutato.
7. Un ripristino verificato in una cartella scelta recupera tutte le lavorazioni e le loro relazioni, mentre l'export integrale conserva ID e date con formato dichiarato.
8. Un report selettivo non rivela per default note riservate, preventivi o recapiti, né genera invio automatico verso fornitori.
9. Dopo errore di scrittura o conflitto su una scheda cambiata in altra finestra nessun incarico «mezzo salvato» diventa visibile; l'utente può riesaminare i dati.
10. Etichette IT/EN/ES/FR/DE, fallback inglese, lingue OS, dati Unicode, tastiera, screen reader e zoom 200% permettono il flusso completo senza rete su Windows/macOS/Linux.
11. Nessuna inferenza automatica «lavoro completato = incorniciata» o «lavoro inviato = opera trasferita» viene fatta nel sistema e nell'interfaccia.

## 10. Sequenza prudente di valutazione (nessuna fase autorizzata)

- **Fase 0 — Ricerca sul bisogno:** osservare 3–5 professionisti con lavorazioni effettive recenti (corniciaio, laboratorio stampa, fonderia, galleria), valutare frequenza e perdita di informazioni; confrontare come oggi annotano richieste, date e costi. Misurare se il codice opera basta per trovare il lavoro e quante azioni vanno perse.
- **Fase A — Nessun nuovo modulo:** provare una scheda/checklist Markdown o CSV esportabile, compilata a mano in un archivio sintetico. Se risolve il problema, archiviare la proposta senza implementarla.
- **Fase B — Solo se emerge domanda:** record leggero per lavorazione **di singola opera**, data prevista, stato manuale e filtro aperte, senza costi, documenti o contatti strutturati.
- **Fase C — Solo con evidenze distinte:** eventuali preventivi, collegamento alla spesa registrata e allegati selezionati; attivare con revisioni separate dei confini contabili e privacy.
- **Fase D — Eventuale complessità ulteriore:** ordini multiopera, cronologia datata e connessioni ai movimenti/condizione se gli altri moduli saranno approvati; nessuna sincronizzazione obbligatoria.

## 11. Certezze, ipotesi e domande per il proprietario

**Requisiti certi del progetto:** desktop gratuito e open source, offline-first, Tauri/Angular/Rust/SQLite, archivio locale con backup a cartella scelta e dati sempre esportabili, senza account/abbonamento; lingua UI del sistema con fallback inglese. Registri presenti di opere, spese, vendite e loro stati semantici distinti. **La nuova funzione non fa parte dei requisiti approvati.**

**Ipotesi:** artisti/gallerie affidano abbastanza spesso lavorazioni a terzi da aver bisogno di un registro dedicato; le scadenze sono più importanti del costo stimato; una semplice checklist per opera può avere valore senza tracciare consegne o proprietà.

**Domande alla valutazione personale del proprietario:**

1. Il pubblico prioritario sono artisti che commissionano cornici/stampe o archivi/gallerie che coordinano molti laboratori?
2. È sufficiente una **checklist per opera** o serve veramente una vista di tutte le lavorazioni aperte?
3. L'utente registra più spesso *una sola opera per incarico* oppure una fattura di laboratorio con più opere?
4. Le date previste e quelle effettive sono abbastanza affidabili da meritare campi separati, anziché una nota?
5. È opportuno lasciare **per sempre** i servizi senza rilevanza contabile, o solo valutare in una fase distinta il collegamento a `expenses`?
6. Un lavoro di restauro va deliberatamente escluso per evitare di creare un secondo registro di conservazione?
7. Come comunicare più chiaramente che «lavorazione conclusa» **non prova** rientro fisico dell'opera o soddisfazione dell'artista?

**Decisione:** **Proposta — da valutare**. Nessuna idea approvata o assegnata. Nessuna implementazione.
