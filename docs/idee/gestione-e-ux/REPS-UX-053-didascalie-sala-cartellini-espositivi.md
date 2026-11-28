# REPS-UX-053 — Didascalie di sala e cartellini espositivi stampabili, senza duplicare le schede

- **ID stabile:** REPS-UX-053
- **Data:** 2026-11-22
- **Stato:** Proposta — da valutare
- **Filone:** Gestione e UX
- **Priorità ipotetica:** P2 per artisti e gallerie che allestiscono mostre frequenti; P3 o nessuno sviluppo se basta una semplice esportazione verso LibreOffice
- **Natura:** esclusivamente analisi/documentazione; **nessuna approvazione, implementazione, migrazione, assegnazione o modifica del prodotto autorizzata**

## 1. Problema professionale, persone e riscontri

**Esigenza da verificare:** durante una mostra, open studio, fiera o presentazione in galleria bisogna produrre **cartellini destinati al pubblico**, spesso uno per opera, con titolo, autore, anno, tecnica, supporto, dimensioni e talvolta un testo curatoriale breve. Riportare a mano i campi su un elaboratore di testi richiede tempo e introduce difformità, soprattutto quando le schede vengono aggiornate poco prima della stampa. Un artista con poche opere lo fa facilmente a mano; una galleria che cambia allestimenti frequentemente può invece avere un costo ricorrente.

**Destinatari:** artisti che organizzano esposizioni, piccole gallerie, curatori indipendenti, assistenti di studio e archivisti che producono materiale di sala. Il pubblico visita la mostra ma **non è utente del gestionale**.

**Verificato nel repository pubblico `main` il 2026-11-22:** [README](../../../README.md), [MVP](../../MVP.md), [modello delle opere e comandi](../../../src-tauri/src/commands.rs), [brochure offline](../../BROCHURES-OFFLINE.md) e ricerca sul codice. L'app conserva codice, titolo, anno, tecnica, supporto, misure, descrizione, stato e immagini; possiede un **editor di brochure PDF** con tre template, ma non emerge un flusso specifico per **più cartellini brevi su un foglio, con layout e controlli per la stampa di sala**. La ricerca non equivale a collaudo nativo completo: l'assenza va riverificata se `main` evolve. I formati e i testi degli esempi che seguono sono **ipotesi**, non requisiti di museo né standard imposti.

**Beneficio misurabile da validare:** quante etichette si preparano a mostra, quante correzioni si fanno, quanti minuti richiede oggi il passaggio schede → documento → stampa, quanti errori occorrono. Prima di progettare un modulo, testare un foglio modello LibreOffice con esportazione dei dati già disponibili.

## 2. Confini anti-doppione

- **[REPS-UX-020 — Etichette inventariali](REPS-UX-020-etichette-inventariali-identificativi-scansionabili.md):** identifica **internamente** opere e contenitori mediante codice umano/QR. La presente proposta riguarda **testi pubblici di sala**, con tipografia leggibile, crediti e campi curatoriali; non richiede QR né identifica/scansiona l'opera. La stampa di un cartellino **non prova presenza fisica**.
- **[REPS-UX-007 — Dossier](REPS-UX-007-dossier-presentazione-opere.md)** e [brochure già reali](../../BROCHURES-OFFLINE.md): producono **pagine editoriali** e presentazioni per uno o più lavori. Un cartellino è una **unità piccola, spesso ripetuta su A4/A5, senza fotografia, copertina o indice**. Riutilizzare i componenti di stampa solo se sensato, senza creare un secondo editor generale.
- **[REPS-UX-009 — Progetti espositivi](REPS-UX-009-progetti-espositivi-checklist.md):** può prevedere preparazione delle didascalie fra i compiti di una mostra. Qui si descrive **esclusivamente la produzione del documento**; nessuna entità mostra o checklist è introdotta o aggiornata.
- **[REPS-UX-041 — Testi multilingua](REPS-UX-041-titoli-descrizioni-opere-multilingua.md)** e **[REPS-UX-031 — Autorialità](REPS-UX-031-autorialita-opere-attribuzioni-collaborazioni.md):** fonti future possibili, non approvate né prerequisiti. Nessuna traduzione, identità d'autore o attribuzione viene inventata.
- **[REPS-UX-023 — Listini](REPS-UX-023-prezzi-richiesti-listini-storico.md):** i prezzi richiesti, se in futuro disponibili, **non** si mostrano per impostazione predefinita; il cartellino non è un listino o una prova del prezzo di vendita.
- **[REPS-TECH-032 — PDF Unicode](https://github.com/Cesare-ml/Repertica-Studio/blob/docs/idee-strumenti-architettura-20261121/docs/idee/strumenti-e-architettura/REPS-TECH-032-pdf-brochure-unicode-font-incorporati-offline.md):** studia limiti/soluzioni del renderer PDF e caratteri non latini. Questa scheda definisce **il flusso professionale e la gerarchia visiva**, non prescrive un nuovo renderer, font, dipendenza o motore PDF.
- **[REPS-TECH-025 — Condivisione selettiva](https://github.com/Cesare-ml/Repertica-Studio/blob/docs/idee-strumenti-architettura-20261121/docs/idee/strumenti-e-architettura/REPS-TECH-025-esportazioni-tabellari-minimizzate-condivisione.md):** affronta l'export selettivo di registri; un cartellino impaginato non sostituisce export completo o CSV.

**Fuori ambito:** cartellonistica per intere sale, editor grafico generalista, etichette adesive sull'opera, codici di autenticazione, QR pubblico obbligatorio, publishing web, traduzione automatica, prestiti/movimentazioni, biglietteria, vendita o fatturazione, accesso a file di terzi senza consenso.

## 3. Scenari professionali concreti

1. **Personale con 18 opere.** L'artista seleziona i lavori in archivio, decide ordine e composizione delle righe, controlla che le dimensioni siano espresse chiaramente e salva un PDF con 4/6 cartellini per foglio A4. Ogni cartellino è revisionato prima della stampa.
2. **Collettiva.** Una piccola galleria presenta artisti diversi. Se un'opera non ha autore esplicito registrato, il campo resta **«da verificare»** oppure viene compilato per *questa stampa*, senza assegnare automaticamente l'artista proprietario del database all'opera.
3. **Opere con titoli lunghi e tecniche miste.** L'anteprima segnala se il testo eccede il riquadro; l'utente accorcia **solo il testo della didascalia** o sceglie un cartellino più grande, senza taglio silenzioso né modifica della scheda.
4. **Allestimento bilingue.** Stessi dati comuni e due versioni testuali curate separatamente. Un'eventuale lingua di esposizione non coincide necessariamente con la lingua dell'app. Mancanza di una traduzione dichiarata esplicitamente, mai sostituita automaticamente con testo creato dall'IA.
5. **Cambio di titolo a mostra in corso.** Il PDF già esportato rimane una **fotografia del contenuto di allora** e non si aggiorna; una nuova esportazione deve rendere evidente che l'opera corrente potrebbe avere metadati diversi. Nessuna vecchia didascalia diventa certificato.
6. **Catalogo privato e sala pubblica.** Anche se le schede comprendono collezionisti, ubicazioni, prezzo, spese o note di vendita, il cartellino esportato contiene solo i **campi selezionati per pubblicazione**; nessun contatto o nome proprietario deve uscire per errore.

## 4. Flusso UX proposto (non implementato)

### Preparazione

- Da **Archivio opere → Seleziona → Didascalie di sala** oppure da una singola scheda, visualizzare opere scelte, codice interno visibile **solo nell'anteprima gestionale**, titolo, ordine e indicatori di completezza. La funzione deve funzionare anche senza UX-009.
- Per ogni lavoro scegliere contenuto pubblico: **titolo**, eventuale **autore dichiarato**, **anno**, **tecnica/supporto**, **dimensioni** (unità esplicite e ordine stabile), eventuale breve nota curatoriale e **crediti espositivi verificati**. L'inclusione di codice archivio è un'opzione separata **disattivata** per default; prezzi e disponibilità sono esclusi nella fase iniziale.
- Testi modificabili **soltanto nella bozza di stampa**, con etichetta «personalizzazione per questa esportazione», senza aggiornamenti involontari ai campi dell'opera. **Mai inventare** date, attribuzioni o dimensioni mancanti. Per un valore ignoto mostrare «non compilato / da verificare» nel redattore, e consentire omissione consapevole sul documento pubblico.
- Selezionare un layout sobrio (inizialmente **uno solo**), dimensione fisica e margini reali; anteprima foglio/pagine con righelli o misure in mm; scelta del numero di cartellini per foglio coerente con dimensione e contenuti. Il prodotto non deve garantire che ogni stampante rispetti automaticamente i margini.

### Revisione e consegna

- Schermata di controllo **prima dell'export**: numero opere, numero cartellini, pagine, campi mancanti, testi che non entrano, lingua dichiarata, eventuali campi riservati inclusi su richiesta. Gli avvisi devono avere descrizione testuale, non solo colore.
- **PDF/stampa locale** tramite salvataggio o dialogo del sistema; nessuna stampa automatica e nessun invio a email o servizi. In caso di errore di impaginazione, **non generare file troncato senza avviso**. Per uno schema di impaginazione fisso, anteprima e PDF devono concordare su contenuto, ordine e interruzioni, altrimenti riportare l'anteprima come indicativa e offrire prova prima della stampa.
- Il documento finale può riportare un identificativo breve di sessione **solo se richiesto** su una pagina tecnica separata, mai come marchio visibile al pubblico. Non assumere che venga conservato nell'archivio: un PDF esportato in cartella esterna non è compreso automaticamente nel backup.
- **Accessibilità:** selezione e riordinamento tramite tastiera (pulsanti sposta su/giù), indicazioni di overflow testuali, focus coerente, preview con zoom e pagina descritta anche in elenco, testo leggibile e alto contrasto; la resa PDF deve poter essere verificata con font grandi e stampa in bianco e nero.

### In caso di utilizzo frequente

Solo se richiesta da utenti reali, valutare un **modello di cartellino salvabile** e una **bozza/snapshot di selezione**, con scelta esplicita tra «ricarica dati correnti» e «mantieni testo dell'ultima stampa». Nessuna sincronizzazione silenziosa, nessun progetto espositivo implicito e nessun blocco alle modifiche dell'opera.

## 5. Dati necessari e API solo descrittive

**Fase minima senza migrazione obbligatoria**, da valutare: una selezione in memoria degli UUID di opere già presenti e una struttura di stampa temporanea. Concetti esemplificativi:

- `label_sheet_draft` (solo stato temporaneo, non tabella imposta): lingua del **documento**, ordine UUID selezionati, formato pagina, margini in mm, cartellini per foglio, intestazioni visibili, data di preparazione.
- `wall_label_item`: `artwork_id`, `source_updated_at` come indicazione di confronto, campi presi dalla scheda, eventuali override **solo del documento**, `credit_line` e `display_language`, errori/avvisi di compilazione. `artwork_id` è un riferimento locale, non un certificato di identità pubblica.
- `wall_label_layout`: dimensioni fisiche, spaziatura, gerarchia tipografica, impostazioni del testo; niente riferimenti assoluti ai file personali.
- Possibile `renderWallLabels(selection, overrides, layout)` come **interfaccia concettuale**, non API da creare ora. Produzione senza scritture a `artworks`, `sales`, `receipts`, `expenses`, `certificates`, `gallery`.
- **Fonte autore:** in assenza di un modello di autorialità, nome libero **verificato manualmente per il documento** o campo omesso. Non inferirlo da nome utente, titolare, collezionista, provenienza o artista predefinito.
- **Dati espositivi** eventuali, come sala/numero parete, sono etichette redazionali *non* prova di collocazione fisica. Se si mantiene una bozza, le varianti per lingua devono rimanere distinguibili senza traduzioni presunte.

## 6. Preferenze, offline, multilingua, privacy

- **Offline-first:** dati letti solo dal SQLite locale, nessun server, account, abbonamento, AI cloud, download di font all'uso, telemetria, catalogo remoto o sincronizzazione richiesta. I dialoghi di stampa/file sono nativi del dispositivo; un'unità esterna/NAS è usata solo se scelta dall'utente.
- **Lingua UI:** lingua di sistema; scelta manuale se già configurata; **fallback inglese** per locale non supportata o stringhe mancanti, come da [ECONOMY-I18N](../../ECONOMY-I18N.md). La lingua dei **testi pubblici** è invece selezionata esplicitamente per il documento. Titolo e descrizione non si traducono da soli.
- **Impostazioni facoltative:** A4/A5, orientamento, misure mm, margini, dimensione caratteri, campo codice interno sì/no, ordine delle righe; salvare un modello locale solo se utile. Partire con un layout e pochissime opzioni anziché introdurre un designer visivo.
- **Privacy-by-default:** whitelist pubblica rigorosa; **esclusi senza eccezioni implicite** email, indirizzi, note interne, ubicazione di deposito, nome proprietario/collezionista, prezzo pagato, margini, fondi e percentuali di ripartizione. Campi ulteriori richiedono scelta esplicita e anteprima dei byte/testi da pubblicare. L'export PDF/spooler di stampa possono restare fuori dal controllo dell'app; avvisare se il percorso è condiviso.
- **Crediti e diritti:** riportare solo crediti forniti dall'utente; non stabilire titolarità del diritto d'autore o autorizzazioni di pubblicazione. Un cartellino non è una prova di autenticità, proprietà, datazione o perizia.

## 7. Import/export, backup, compatibilità

- **Esportazione primaria:** PDF stampabile creato localmente e salvato nel percorso scelto dall'utente. Per la prima fase **nessun import di cartellini**: un PDF non è una fonte autorevole da reimportare come opera.
- **Alternativa/interoperabilità prima dello sviluppo:** esportazione selettiva dei campi già disponibili in CSV UTF-8, per un modello LibreOffice Writer/Calc; intestazioni in codici stabili, testo con escape e protezione contro formule da fogli di calcolo. Non spacciare questa esportazione come formato canonico di backup o come prova di identità del record.
- **Se in seguito si salvassero bozze:** formato JSON versionato e documentato con ID, selezione, lingua, override e layout; esportazione/reimportazione con anteprima, UUID mancanti o conflitti dichiarati e nessuna scrittura automatica alle opere. Dati aggiunti inclusi nel normale backup SQLite+media su cartella arbitraria; se nessun salvataggio locale è implementato, indicare chiaramente che i PDF esterni **non** sono nel backup.
- **Compatibilità:** Windows/macOS/Linux e stampanti con margini differenti; layout robusto per IT/EN/ES/FR/DE, accenti, apostrofi tipografici, testi molto lunghi e alfabeti non latini. Il renderer brochure esistente usa Helvetica/WinAnsi e può sostituire alcuni caratteri: **non promettere Unicode completo** prima di una verifica tecnica con TECH-032. Fallback della UI in inglese **non** deve sostituire o latinizzare il testo dell'opera.
- **Archivi precedenti:** nessuna retrocompilazione di autori o crediti, nessuna modifica dei certificati immutabili. Un eventuale schema additivo deve mantenere leggibili gli archivi preesistenti e non compromettere restore verificato da USB/NAS.

## 8. Rischi, complessità, dipendenze e alternative

**Complessità stimata:** bassa–media per selezione e generazione semplice basata sui campi attuali; media per impaginazione fisica robusta, overflow, anteprima fedele e gestione multilingue; medio-alta per modelli persistenti, più lingue e round-trip di bozze. **Non proporre un editor grafico da zero.**

**Dipendenze potenziali:** identificativi delle opere, modelli UI Tauri/Angular, sistema di stampa/PDF locale, i18n esistente. Il modulo brochure potrebbe condividere renderer e misurazione del testo, **ma solo dopo analisi**: il suo PDF non garantisce ad oggi tutti i glifi Unicode. Nessuna dipendenza o file di implementazione viene toccato.

**Rischi principali:** (a) autore/anno/tecnica presentati come fatti verificati quando sono dichiarazioni dell'utente; (b) taglio invisibile di titoli e testi; (c) misure senza unità o formattazione ambigua; (d) perdita di accenti; (e) divulgazione di dati commerciali o personali; (f) differenze stampa-anteprima fra sistemi; (g) confusione fra cartellino stampato e presenza effettiva in mostra; (h) duplicazione di UX-007/UX-020 e crescita inutile della UI. Mitigare con confini stretti, prove di stampa, revisione per campo, whitelist privacy e test con professionisti.

**Alternative preferibili da provare:**
1. CSV selettivo e modello Writer/Calc riutilizzabile, totalmente offline, con nessuna nuova UI.
2. Impaginare manualmente in un documento esterno: razionale con poche opere/mostre.
3. Un template molto compatto **nel modulo brochure esistente** anziché una voce di menu distinta, purché mantenga anteprima e limiti di privacy; da confrontare con UX-007 prima di qualsiasi sviluppo.

## 9. Criteri di accettazione ipotetici

1. Selezionando cinque opere, la stampa include esattamente cinque cartellini, nell'ordine confermato; nessun duplicato involontario o opera non selezionata.
2. Se titolo, tecnica, anno o dimensioni mancano, l'anteprima avvisa e permette omissione o override locale **senza alterare il database**; non mostra valori inventati.
3. Campi privati di vendita, collezionista, spese, ripartizioni e ubicazioni interne **non appaiono** nel PDF predefinito né nei metadati aggiunti dall'app.
4. Un titolo lungo e uno con accenti/segni Unicode non vengono troncati in silenzio; quando il renderer non li supporta, l'utente riceve un errore/avviso verificabile prima di esportare.
5. Ogni foglio in PDF mantiene margini e separazione fra cartellini su almeno Windows e macOS; stampa di prova su A4 reale; verificare Linux quando incluso nel piano di test.
6. La lingua dell'app e quella del cartellino sono indipendenti; lingua di sistema non riconosciuta → UI inglese; nessuna traduzione automatica dei testi dell'artista.
7. Annullare la stampa o incontrare una cartella non accessibile non modifica opera, certificati, vendite, file già esistenti o backup. Eventuali PDF incompleti non devono essere presentati come pronti.
8. Se in futuro si salvano bozze, backup/ripristino a cartella scelta preservano ordine, lingue e personalizzazioni; esportazione/reimportazione dichiara opere mancanti e non ricrea vendite o certificati.
9. La generazione è utilizzabile da tastiera, con avvisi comprensibili da screen reader e layout consultabile senza soltanto colori o drag-and-drop.
10. Test con almeno **due** espositori reali e **15–20 opere**: confrontare tempo/errori del flusso proposto contro CSV + modello LibreOffice; se nessun vantaggio apprezzabile, **non procedere**.

## 10. Possibile suddivisione in fasi — non roadmap approvata

- **Fase 0, senza codice:** osservare due sessioni di preparazione reali; raccogliere etichette autorizzate, dimensioni fisiche, scelte linguistiche e problemi di layout; provare CSV + Writer/Calc.
- **Fase A, solo se serve:** selezione opere → cartellini brevi con un modello fisso → revisione → stampa/PDF locale, senza nuove tabelle o interfaccia grafica generalista.
- **Fase B, opzionale:** due formati pagina/cartellino, lingue documentali distinte, preset locale e avvisi di campi mancanti, senza traduzioni IA.
- **Fase C, solo con forte domanda:** bozze/snapshot di un allestimento, versioni consultabili, eventuale collegamento *manuale* a UX-009 e testi curatoriali localizzati da UX-041. Nessuna sincronizzazione o presenza in sala presunta.

## 11. Requisiti certi, ipotesi e domande per la revisione del proprietario

**Requisiti certi del prodotto:** app gratuita/open source Tauri e SQLite locale, offline-first senza account/abbonamento; lingue di sistema con fallback inglese; catalogo opere e brochure già esistenti; esportabilità e backup verificato con cartella scelta dall'utente; certificati storici non modificabili.

**Ipotesi non accertate:** preparazione dei cartellini è abbastanza frequente da giustificare una funzione dedicata; modelli fissi riducono errori più di un template Writer; la maggior parte desidera un cartellino senza fotografia; il flusso è utilizzabile anche senza un progetto mostra. Nessuna prova utente raccolta durante questa analisi.

**Domande aperte al proprietario, senza risposta presunta:** (1) cartellini per opere proprie o anche per artisti terzi? (2) A4 in multipli o formato singolo A5/A6? (3) le dimensioni in cm con quale ordine e precisione? (4) il nome artista è disponibile da un'eventuale impostazione studio o va inserito nel documento? (5) preferire un mini-template nelle brochure invece di una funzione distinta? (6) quali formati di stampa sono realmente usati e quanto spesso? (7) si deve conservare in archivio la versione esatta di ogni cartellino consegnato o basta il PDF esterno? (8) serve doppia lingua sullo stesso cartellino o due serie separate?

**Esito:** soltanto proposta documentale da valutare personalmente; nessuna idea approvata o assegnata allo sviluppo.
