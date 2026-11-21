# REPS-UX-021 — Acquisizioni di opere da terzi e documentazione del titolo dichiarato

- **ID stabile:** REPS-UX-021
- **Data:** 2026-11-21
- **Stato:** Proposta — da valutare
- **Filone:** Gestione e UX
- **Priorità ipotetica:** P2 per gallerie, archivi e collezioni con acquisizioni; da rivalutare solo dopo confronto con utenti
- **Natura:** esclusivamente analisi e specifica documentale. Non autorizza sviluppo, modifiche al codice, modelli o database, approvazioni, assegnazioni o merge.

## 1. Problema e destinatari, senza confondere custodia e proprietà

Un artista che cataloga le **proprie creazioni** non ha bisogno di registrare un'acquisizione per ogni opera: il flusso normale deve restare semplice. Una **galleria che acquista opere di altri artisti**, un archivio che riceve una donazione oppure una collezione che acquisisce un gruppo di lavori hanno invece un problema diverso: sapere *come* una specifica opera è entrata nel patrimonio dichiarato, da chi, a quali condizioni, con quali documenti, e distinguere l'arrivo fisico dall'eventuale acquisizione conclusa.

La mancanza di un fascicolo strutturato rischia di mescolare la consegna materiale, una trattativa di acquisto, la spesa finanziaria, la provenienza dichiarata e la prova documentale del trasferimento del titolo. **Nessuna spunta o formula può certificare la proprietà legale**, l'autenticità, la provenienza lecita, il pagamento o la titolarità dei diritti d'autore: occorre conservare ciò che la persona ha registrato e le fonti disponibili, evitando affermazioni automatiche.

**Destinatari principali:** piccole gallerie che comprano per rivendere; archivi e fondazioni che ricevono donazioni; collezionisti professionali con ingresso di opere da più fonti. **Secondari:** artisti che acquistano opere altrui o rientrano in possesso di opere cedute in precedenza. La frequenza reale di tali casi, nell'utenza Repertica, è **da verificare**.

### Riscontro verificato sul repository (main, 21/11/2026)

Consultati [README](../../../README.md), [MVP](../../MVP.md), [economia e i18n](../../ECONOMY-I18N.md), [portabilità](../../DATA-PORTABILITY.md), [modelli Angular](../../../src/app/models.ts), [modelli Rust](../../../src-tauri/src/models.rs) e [schema SQLite](../../../src-tauri/src/db.rs). L'opera attuale ha UUID, codice, titolo, descrizione, stato commerciale, ubicazione e media, ma **non** una pratica distinta di acquisizione da terzi, né un campo strutturato per autore dell'opera o provenienza dell'acquisto. Il registro `expenses` conserva uscite per causale/fornitore; `sales` registra la **vendita in uscita** dell'opera a un collezionista. Non sono la prova di un acquisto/ingresso in proprietà. Lo schema visionato è v6. Sono già disponibili backup verificati e **otto** registri CSV integrali, ma non un fascicolo di acquisizione. È una lacuna di dominio, **non** un bug o prova di gestione errata.

### Fondamento esterno, senza importare obblighi museali nel prodotto

Il progetto può prendere a riferimento **come checklist concettuale**, non come standard certificato di Repertica, le distinzioni descritte da Collections Trust/Spectrum: [Object entry](https://collectionstrust.org.uk/resource/object-entry-the-spectrum-standard/) separa arrivo/custodia e motivo del deposito; [Acquisition and accessioning](https://collectionstrust.org.uk/resource/acquisition-and-accessioning-the-spectrum-standard/) richiede, nel contesto museale di riferimento, di documentare il trasferimento del titolo; la [procedura suggerita](https://collectionstrust.org.uk/resource/acquisition-and-accessioning-suggested-procedure/) elenca fonti utili. **Sono orientamenti di settore, non attestazioni giuridiche applicabili automaticamente a qualsiasi paese o compravendita.**

## 2. Confini anti-doppione: le altre schede non vengono modificate

| Ambito già presente/proposto | Confine di REPS-UX-021 |
| --- | --- |
| [UX-001 — Provenienza e movimentazioni](REPS-UX-001-provenienza-movimentazioni.md) | Descrive passaggi storici, luogo e custodia. **Qui** si propone il fascicolo della *specifica decisione di acquisire* (offerta, titolo/documenti, accettazione dichiarata, condizioni) di chi gestisce l'archivio. Un movimento di ingresso non costituisce acquisizione. Nessuna seconda cronologia fisica. |
| [UX-002 — Trattative e prenotazioni](REPS-UX-002-trattative-prenotazioni.md) | Riguarda un potenziale compratore delle opere gestite: qui il soggetto è l'**acquirente/beneficiario in entrata**. Non si introduce un secondo funnel di vendite ai collezionisti. |
| [UX-003 — Costi e margini per opera](REPS-UX-003-costo-e-margine-per-opera.md) | Gestisce attribuzioni economiche di spese già registrate. Questa proposta conserva **condizioni e fonte dell'acquisizione**; un corrispettivo di acquisto può essere collegato a una spesa **una sola volta**, senza crearne una seconda né alterare i margini. |
| [UX-012 — Affidamenti in conto vendita](REPS-UX-012-affidamenti-conto-vendita-rendiconti.md) | Un'opera ricevuta in conto vendita **non è acquisita** dalla galleria; gli accordi di intermediazione rimangono nel loro dominio. Conversione eventuale solo come nuova operazione esplicita, con evidenza. |
| [UX-014 — Rubrica professionale](REPS-UX-014-rubrica-relazioni-professionali.md) | Definisce chi è il soggetto. L'acquisizione definisce l'**operazione su una o più opere**, con controparte collegabile se una rubrica generale verrà approvata. |
| [UX-016 — Valutazioni e assicurazioni](REPS-UX-016-valutazioni-coperture-assicurative-scadenze.md) | Un prezzo pattuito non è una stima assicurativa, e una donazione non è automaticamente valorizzata. |
| [UX-017 — Vendite multiopera](REPS-UX-017-vendita-multiopera-ripartizione-importi.md) | È la **vendita in uscita**, non un lotto acquistato in entrata. Nessuna riutilizzazione impropria delle vendite. |
| [TECH-001 — Interoperabilità](https://github.com/Cesare-ml/Repertica-Studio/blob/docs/idee-strumenti-architettura-20261121/docs/idee/strumenti-e-architettura/REPS-TECH-001-pacchetto-dati-aperto.md) | Definisce il formato di scambio generale: questo documento descrive solo il significato e la UX del fascicolo. |
| [TECH-016 — Allegati documentali](https://github.com/Cesare-ml/Repertica-Studio/blob/docs/idee-strumenti-architettura-20261121/docs/idee/strumenti-e-architettura/REPS-TECH-016-allegati-documentali-locali-multi-entita.md) | Definisce conservazione tecnica dei documenti, non quali condizioni di acquisto documentare; integrazione eventuale e non obbligatoria. |

**Fuori ambito:** gestione fiscale dell'acquisto, registrazione IVA, contabilità civilistica o valorizzazione di magazzino, verifica automatica della legittimità della proprietà, controlli normativi/antiriciclaggio, aste e bidding, passaggi digitali di titolarità, autenticazioni, certificati firmati, integrazioni bancarie, riconoscimento di opere tramite immagini, consulenza legale o fiscale.

## 3. Casi d'uso realistici

1. **Acquisto da un privato da parte di una galleria.** Il referente documenta l'offerta di un dipinto, il prezzo pattuito di 1.200,00 EUR, una fonte/accordo e la data dichiarata di trasferimento. Registra separatamente l'eventuale uscita finanziaria nel registro spese, collegandola soltanto se verificata. L'ingresso fisico potrebbe avvenire giorni dopo e non è dedotto dal prezzo.
2. **Donazione a un archivio.** Una fondazione riceve tre opere con una lettera firmata. Registra il tipo *donazione*, le tre schede interessate, la controparte, le riserve esplicite e il riferimento al documento. **Non** crea automaticamente un incasso, una vendita, un costo pari al valore stimato o un certificato.
3. **Proposta non conclusa e opera in visione.** Una galleria prende in esame una scultura e la riceve temporaneamente. La pratica resta *in verifica* e l'opera fisicamente in custodia **non** passa a «acquisita». Può essere restituita: chiusura pratica e rientro fisico sono eventi separati.
4. **Lotto di acquisto con più opere.** Una galleria acquista quattro opere per 3.000,00 EUR complessivi. Il corrispettivo per ogni opera rimane **ignoto** salvo ripartizione esplicita dell'utente, evitando di inventare un costo unitario e successivi margini falsi.
5. **Fascicolo storico incompleto.** Per un'opera già nel catalogo la persona inserisce «provenienza riferita dal precedente proprietario» e «documentazione da verificare», senza retrodatare un contratto o generare una transazione economica inventata.
6. **Opera in conto vendita poi acquistata.** Il conto vendita resta nella sua cronologia (UX-012 se approvata). Un'eventuale acquisizione successiva ha **una pratica propria**, non modifica retroattivamente le condizioni dell'affidamento né crea un movimento fisico fittizio.

## 4. Flussi UX da sottoporre a valutazione

**A — Ingresso dalla scheda opera (opzionale).** `Opera → Titolo e acquisizioni` presenta *nessuna acquisizione registrata* oppure un elenco di pratiche. Non mostra «di proprietà» in assenza di fonti; non rende obbligatorio questo pannello per chi produce le proprie opere. L'utente sceglie **Nuova pratica** o **Registra riferimento storico**.

**B — Nuova pratica leggera.** Scelta di `acquisto`, `donazione`, `permuta`, `altro` oppure `da chiarire`; contatto/fornitore testuale o soggetto esistente **non obbligatorio**; almeno una descrizione e una data o intervallo facoltativi. Per documenti incerti mostrare `dato dichiarato` / `documento consultato` / `da verificare` e campo di provenienza dell'informazione. Non esigere data o prova inventate per completare una bozza.

**C — Collegamento opere, anche per lotto.** Ricerca e selezione delle schede per UUID/codice; anteprima dei titoli, attenzione a omonimie. Quando un'opera non è ancora catalogata, valutare una **riga provvisoria descrittiva non vendibile**; la creazione della scheda `Artwork` richiede conferma separata e preserva unicità codice/UUID. Nessun record `Artwork` viene creato solo per aver aperto il wizard.

**D — Documenti e condizioni.** Inserimento dei riferimenti a scrittura/accordo, ricevuta di consegna, dichiarazioni, foto e note sulla verifica; import locale di file soltanto se esiste un sistema di allegati approvato, altrimenti riferimento testuale o file selezionato con limitazioni esplicite. **L'app non esamina la validità della firma né dichiara certo il trasferimento del titolo.**

**E — Importi e fonti autorevoli.** In caso di acquisto: corrispettivo pattuito in EUR/centesimi o `non noto`, modalità `per singola opera` / `totale lotto non ripartito` / `quote attribuite manualmente`, data e fonte. Mostrare in colonne separate **prezzo pattuito**, **spesa registrata** e **pagamento verificato esternamente** (quest'ultimo non è verificabile dall'MVP: mostrare *non gestito*). Se una riga viene collegata a `expenses`, l'acquisizione non produce una seconda uscita.

**F — Esito e cronologia.** Stato di lavoro `bozza` → `da verificare` → `documentata come conclusa` oppure `non conclusa`; conferma con riepilogo delle prove mancanti e relativi avvisi. *Documentata come conclusa* significa **solo registrazione dell'operatore**. Rettifiche di fonte/prezzo/stato con data e motivazione, senza alterare in silenzio atti precedenti. Nessun cambio automatico di `artworks.status`, `location`, vendita, titolarità dichiarata o certificato.

**G — Consultazione e stampa.** Elenco pratiche per controparte, tipo, periodo, esito e incompletezza; schermata di dettaglio con **tracciabilità delle fonti** e collegamenti alle opere. Riepilogo stampabile chiaramente marcato `promemoria interno, non documento di trasferimento del titolo`. Prima di esportare per terzi, anteprima dei dati sensibili e scelta esplicita dei campi.

**Accessibilità:** maschere utilizzabili interamente da tastiera, focus ripristinato alla riga dopo il salvataggio, tabelle semantiche, etichette e messaggi di errore letti da screen reader, nessun controllo che dipenda soltanto dal colore o dal drag-and-drop. Date incerte non mostrate come date complete arbitrarie; importi distinguibili come `ignoto`, `zero` o `non applicabile`. Supportare zoom 200%, testi lunghi e Unicode su Windows/macOS/Linux.

## 5. Struttura dati concettuale — *solo ipotesi*, nessuna API/migrazione autorizzata

- `acquisition_case`: UUID, codice pratica locale facoltativo, tipo, scopo (`acquisto/donazione/permuta/altro`), stato di lavoro, data o periodo con precisione dichiarata, controparte testuale o `party_id` **solo se UX-014 approvata**, riferimento a fonte, note e timestamp locali. Non inserire `verified_owner=true` derivato da una spunta.
- `acquisition_item`: UUID della riga, `case_id`, `artwork_id` opzionale finché non esiste scheda, identificativo provvisorio, posizione nel lotto, descrizione originaria e note. Chiave per impedire due righe identiche dello stesso record nella medesima pratica, senza vietare pratiche storiche successive della stessa opera.
- `acquisition_consideration`: UUID, `case_id`, `item_id` opzionale, `kind` (prezzo pattuito/ripartizione dichiarata/nessun corrispettivo monetario/ignoto), `amount_cents` opzionale, `currency=EUR` nella prima proposta. **NULL non equivale a zero.** Se si inserisce una ripartizione di 3.000,00 EUR su quattro opere, somma di importi in centesimi = 300000; se non ripartita, ogni costo unitario rimane non noto.
- `acquisition_evidence`: UUID, pratica o riga, categoria di documento/testimonianza, riferimento locale o `document_id` solo con TECH-016 approvata, data della fonte, descrizione, stato `present/declared/unverified`, riservatezza, riferimento di provenienza. L'hash di un file prova soltanto integrità del contenuto, **non autenticità del titolo**.
- `acquisition_expense_link`: UUID, pratica/riga, UUID di **spesa esistente** e quota associata (se consentita). È un **riferimento**: non crea spese né incassi. Vincoli di quote e contabilità analitica restano, se approvati, autorevoli in UX-003.
- `acquisition_revision`: UUID, pratica, campo/e rettificato, momento, motivo, versione origine, senza attribuire un'identità utente autenticata che non esiste nel desktop offline. Sottolineare `registrato localmente`, non `firmato da`.
- `work_author` **non dedotto**: il modello `Artwork` attuale non ha un campo autore. Possibili scelte *future*, da decidere a parte: campo testuale semplice, relazioni a soggetti o modulo dedicato. Evitare di aggiungerlo silenziosamente o di dedurre l'autore dal nome del venditore.
- Una singola fonte di verità per ciascun dominio: catalogo opera; posizione/trasferimenti (UX-001); vendita in uscita (`sales`); spesa (`expenses`); accordo di intermediazione (UX-012); acquisizione documentata (questo fascicolo). Mai creare automaticamente record paralleli.

Le strutture e gli eventuali endpoint `previewAcquisitionCase`, `saveAcquisitionDraft`, `confirmAcquisition` e `exportAcquisitionSummary` sono **nomi descrittivi**, non interfacce da realizzare né promesse di integrazione.

## 6. Impostazioni, multilingua, funzionamento offline e privacy

- **Impostazioni opzionali:** mostra/nascondi il pannello acquisizioni per studi d'artista; categorie configurabili con codici interni stabili; numerazione pratica suggerita, mai obbligatoria; stampa di riepiloghi con dati personali esclusi per default; formato data/importi secondo locale.
- **Multilingua:** usare lingua di sistema e override già previsti per IT/EN/ES/FR/DE, con **fallback inglese** ([ECONOMY-I18N](../../ECONOMY-I18N.md)); tradurre stati, ruoli e messaggi di rischio. Nomi, formule contrattuali, descrizioni e documenti inseriti dall'utente restano nella lingua originale, senza traduzioni inventate.
- **Offline-first:** database locale SQLite, documenti locali soltanto con consenso esplicito al loro inserimento, nessuna rete, account, sincronizzazione, notifiche remote, telemetria, verifica online della proprietà, apertura automatica di URL o scansione di cartelle esterne.
- **Privacy:** nomi dei precedenti proprietari, indirizzi, prezzo di acquisto, identità dei donatori e contratti sono **dati sensibili per uso professionale**. Minimizzare campi, separare note riservate, non includere i prezzi di acquisto in dossier pubblici/certificati; `Esporta per terzi` richiede anteprima e opt-in per ogni gruppo di campi. I backup ordinari sono **in chiaro**: avvertire nella UX e rispettare la cartella liberamente scelta.
- **Conflitti:** un ingresso temporaneo non deve rendere vendibile un'opera; una conclusione amministrativa non dimostra possesso fisico o proprietà. Presentare conflitti come elementi da controllare, non come verdetti legali.

## 7. Import, export, portabilità e compatibilità

- **Vista consultiva:** CSV UTF-8 per pratiche e righe, separando `acquisition_cases`, `acquisition_items` e, se necessario, riferimenti alle fonti/quote; UUID, date ISO con precisione esplicita, importi **in centesimi interi**, tipo e stato con codici stabili. Vietato confondere un riepilogo di acquisizioni con un registro di spese pagate.
- **Pacchetto aperto ipotetico:** JSON versionato + manifest dei documenti locali eventualmente importati, ID e riferimenti relativi; in caso di export completo *dell'archivio* includere anche fonti e metadati riservati, evitando che un export redatto per terzi venga scambiato per portabilità integrale. Allineamento concettuale con TECH-001; **non** modifica la sua scheda né definisce qui un formato globale.
- **Import futuro:** scelta di file locale; anteprima di pratiche nuove/esistenti; corrispondenze opera per UUID e codice con verifica umana se ambigue; dati controparte non dedotti per omonimia; gestione idempotente, con transazione e rollback; nessuna modifica a record di vendita, spesa o certificato durante l'import di sole pratiche.
- **Compatibilità:** migrazioni additive e conservative dal modello v6 senza dedurre acquisizioni dalle opere esistenti o convertire automaticamente `supplier`/collezionisti in titolari; archivi vecchi restano leggibili dalle nuove versioni senza pratiche, mentre versioni vecchie potrebbero non aprire nuovi schemi (avviso). Backup/ripristino integrali, verificati su cartella scelta, includerebbero tutte le nuove tabelle e allegati senza alterare manifest storici o snapshot dei certificati.
- **Compatibilità piattaforme e supporti:** percorsi relativi e file Unicode su Windows/macOS/Linux, supporti USB/NAS già montati (nessun protocollo cloud nell'app), date dichiarate senza dipendere dalla timezone per eventi solo giorno/anno. Il *report esterno* salvato in cartella arbitraria non viene automaticamente incluso nel backup dell'archivio, da chiarire all'utente.

## 8. Complessità, dipendenze, rischi e alternative

**Complessità:** media per pratica semplice e lista di opere già catalogate; medio-alta per lotti, fonti/integrazione allegati, rettifiche e associazioni spese; alta per una gestione integrale di trasferimenti di proprietà, regole giuridiche per paesi, permute patrimoniali e importazioni da sistemi terzi — **escluse dalla proposta iniziale**.

**Dipendenze certe già disponibili:** UUID opere, SQLite, selettore file e backup locali, centesimi EUR, i18n. **Dipendenze soltanto possibili:** UX-001 per movimenti, UX-003 per costi, UX-014 per contatti, UX-012 per mandato, TECH-001 per round trip generale, TECH-016 per file condivisi. Nessuna di queste deve essere considerata approvata o requisito di avvio.

**Rischi principali e mitigazioni:**
- *«Acquisito» letto come titolo legale accertato*: etichette `documentata come conclusa` con fonte e avvertenza; nessun certificato di proprietà.
- *Doppio conteggio del costo*: link espliciti alle spese esistenti; preview `costo pattuito ≠ spesa registrata ≠ pagamento`; importi non noti restano NULL.
- *Confusione con conto vendita o deposito*: tipi e schermate separati, nessuna conversione per trascinamento o per cambio di ubicazione.
- *Sovrascrittura di storie e contratti*: rettifiche con motivo, nessuna eliminazione a cascata di prove collegate a opere con vendite/certificati.
- *Dati personali nell'esportazione*: anteprima, profilo interno/per terzi, documenti non inclusi automaticamente.
- *Aumento eccessivo della UI per artisti singoli*: modulo opzionale, nessun onboarding aggiuntivo.
- *Assenza di verifiche legali*: guida esplicita «Repertica archivia dichiarazioni e fonti, non verifica automaticamente l'efficacia degli atti».

**Alternative:** (1) **nessun nuovo modulo**, usare documento PDF in cartella scelta + descrizione opera, con perdita di collegamenti e filtri; (2) **minimo realmente utile**: pratica acquisizione singola con controparte, tipo, fonte e stato, senza calcolo importi né lotti; (3) espandere UX-001 con una nota «acquisita» — economica, ma rischia di confondere **evento di provenienza** e **operazione d'acquisto**. Decidere solo dopo prove d'uso.

## 9. Criteri di accettazione ipotetici (non test da implementare ora)

1. **Solo utenti pertinenti:** un artista che registra una propria opera non vede passaggi di acquisizione obbligatori; una galleria può aprire una pratica per opera esistente.
2. **Custodia ≠ proprietà:** contrassegnare «opera arrivata» non imposta il fascicolo su «documentata come conclusa»; chiudere la pratica non modifica `location`, `status` o i movimenti.
3. **Prezzi ≠ pagamenti:** inserire un corrispettivo di 1.200,00 EUR non cambia spese, cassa, incassi o vendite; collegare un'`expense` esistente non aggiunge una seconda uscita.
4. **Lotto trasparente:** quattro opere per totale 3.000,00 EUR restano con costi unitari `non attribuiti` fino a ripartizione confermata; una ripartizione di 700+800+650+850 EUR è accettata (totale esatto), mentre somme non coerenti sono respinte con errore leggibile.
5. **Donazione/permuta:** l'assenza di corrispettivo monetario non viene letta come prezzo di mercato pari a zero, né genera automaticamente movimenti finanziari.
6. **Prova/uncertainty:** una fonte descritta come `riferita` non appare «verificata». Rettificare un dato storico crea una nota e conserva il riferimento precedente.
7. **Preservazione:** creare/modificare/archiviare pratiche non altera certificati immutabili, vendite, ubicazioni o fotografie originali.
8. **Portabilità:** esportazione e reimportazione in archivio di prova conservano UUID, riga/lotto, fonti, importi interi e livelli di certezza; secondo import non duplica pratiche; backup/ripristino in cartella scelta conserva tutto.
9. **Privacy e accessibilità:** un report per terzi non include per default prezzo d'acquisto, recapiti e documento di identità; tutti i flussi sono utilizzabili offline, con tastiera e screen reader, lingua OS o fallback inglese.

## 10. Fasi possibili, **non** piano di sviluppo approvato

- **Fase 0 — Convalida della domanda:** osservare 3–5 pratiche reali con almeno una galleria e un archivio; verificare se bastano UX-001 + PDF e quali documenti siano effettivamente usati. Distinguere studio di artista singolo da galleria commerciale.
- **Fase A — Tracciamento essenziale:** fascicolo acquisizione per opera già censita, controparte minima, tipo, stato dichiarato, fonte e consulta/stampa in locale. Nessun calcolo economico né creazione automatica di schede.
- **Fase B — Lotti e importi:** più opere, prezzo totale vs quote dichiarate, fonte e link facoltativo alle spese autorevoli senza doppio conteggio.
- **Fase C — Portabilità e collegamenti:** documenti locali, import/export versionato, eventuale collegamento a provenienza/contatti in caso di approvazioni indipendenti; collaudi completi sui supporti desktop.
- **Fase D — Solo se richiesta dagli utenti:** studio di integrazioni con trattative d'acquisto e procedure specialistiche, dopo valutazione giuridica e di UX. Non inclusa nella presente approvazione (che **non esiste**).

## 11. Separazione tra certezze, ipotesi e decisioni del proprietario

**Requisiti certi del progetto:** app desktop gratuita/open source Tauri, archivio SQLite locale/offline-first, nessun account o abbonamento, esportazioni e backup completi su cartelle scelte, niente rete obbligatoria; ID stabili e registro spese/vendite in centesimi; interfaccia IT/EN/ES/FR/DE, lingua del sistema e fallback inglese; certificati già emessi immutabili.

**Ipotesi da validare:** utenti Repertica comprano opere realizzate da terzi; registrare il titolo dichiarato offre valore distinto da provenienza testuale; esistono frequenti lotti di acquisto; il rapporto qualità/complessità giustifica un nuovo pannello opzionale; la pratica può restare leggera senza allegati nativi.

**Domande al proprietario (non risposte in sua vece):**
1. Repertica deve servire anche gallerie che **acquistano e rivendono opere di più autori**, oltre allo studio dell'artista? Se sì, serve prima un modello di **autore dell'opera**?
2. Ha priorità la gestione delle **acquisizioni concluse**, oppure anche offerte in entrata e opere ricevute per visione prima della decisione?
3. Quali fonti/documenti sono davvero tipici (fattura d'acquisto, scrittura privata, lettera di donazione, contratto, semplice riferimento cartaceo), senza far produrre al software atti legali?
4. Lotti con prezzo indiviso sono frequenti? Una ripartizione economica per opera è realmente necessaria?
5. Il collegamento al registro spese esistente deve essere solo un riferimento oppure UX-003 è un prerequisito che si preferisce completare prima?
6. È sufficiente, all'inizio, un campo `documentazione di acquisizione da verificare` sulla scheda opera, senza una nuova entità?
7. Quali dati sono indispensabili nelle esportazioni verso altri professionisti, separandoli dal backup completo interno?

**Stato finale:** **Proposta — da valutare.** Nessuna approvazione, assegnazione, implementazione o promessa di conformità legale.
