# REPS-UX-017 — Vendita con più opere: riepilogo unico e ripartizione verificabile degli importi

- **ID stabile:** REPS-UX-017
- **Data:** 2026-11-21
- **Stato:** Proposta — da valutare
- **Filone:** Gestione e UX
- **Priorità proposta:** P1–P2, subordinata alla frequenza reale delle vendite multiopera
- **Ambito:** esclusivamente ricerca e documentazione; nessuna implementazione, approvazione o assegnazione

## 1. Problema, destinatari e riscontri verificati

**Utenti:** artisti che vendono più opere durante un open studio, piccole gallerie che concordano una selezione di lavori con un unico acquirente e studi che rendicontano più opere in un'unica trattativa. Se il prezzo complessivo concordato include uno sconto, i professionisti devono distinguere il **totale dell'accordo** dai valori **attribuiti a ciascuna opera**, senza duplicare ricavi o incassi.

**Fatti verificati su `main` (21/11/2026):**
- `src-tauri/src/db.rs`: la tabella `sales` ha `artwork_id UNIQUE`, `collector_id`, `amount_cents` intero positivo e valuta `EUR`; `sale_finances` conserva quote personali/attività/risparmio e imposte/spedizione per singola vendita.
- `src-tauri/src/commands.rs`: `record_sale` registra **una** opera per transazione applicativa, aggiornandone lo stato `sold`; non esiste una conferma coordinata per una selezione di opere. La singola opera non può avere due vendite.
- `docs/SALE-RECEIPTS.md`: gli incassi sono **dichiarazioni manuali** attribuite a una vendita, anche parziali; il totale per vendita non può superare l'importo lordo registrato. Ricavo, incasso e saldo bancario non coincidono.
- `docs/MVP.md` menziona espressamente le **transazioni multi-opera** fra le evoluzioni non ancora disponibili. La proposta approfondisce *come presentare e rendicontare l'accordo al professionista*, non assume che la funzionalità sia già approvata.
- `docs/ECONOMY-I18N.md`: il coefficiente artista produce **solo un prezzo suggerito**; i parametri percentuali sono copiati sulla vendita e non riscrivono retroattivamente le quote delle vendite precedenti. Le statistiche mensili non sono rendiconti fiscali.

**Bisogno da verificare:** non sono state svolte interviste; per un artista che vende quasi sempre una singola opera il modulo potrebbe essere complessità ingiustificata. Scegliere una soluzione che non appesantisca l'attuale inserimento di vendita singola.

## 2. Perimetro e non-duplicazione

- **[UX-002](REPS-UX-002-trattative-prenotazioni.md):** segue offerte, attività e prenotazioni *prima* della conclusione; questa scheda riguarda il **momento della registrazione di un accordo commerciale multiopera** e la sua ricostruzione dopo la vendita. Eventuale passaggio da più trattative sarebbe solo esplicito.
- **[UX-003](REPS-UX-003-costo-e-margine-per-opera.md):** attribuisce *costi di produzione* alle opere; qui si ripartiscono **prezzo e sconto dell'accordo**, non costi di materiali né un secondo registro spese.
- **[UX-012](REPS-UX-012-affidamenti-conto-vendita-rendiconti.md):** accordi di **conto vendita** e commissioni di intermediari, non un acquisto che contiene più opere. Non dedurre percentuali o commissioni da un lotto.
- **[UX-013](REPS-UX-013-resi-annullamenti-rettifiche-vendite.md):** tratta resi e correzioni post-vendita con cronologia. Un futuro storno parziale di una vendita multiopera dovrà usare **un'unica regola coerente** con quel filone, mai un secondo meccanismo concorrente.
- **[Issue #13](https://github.com/Cesare-ml/Repertica-Studio/issues/13):** incassi, documenti di acquisto e cashflow generale. La nuova idea **non** propone fatture, prima nota, pagamenti bancari, riconciliazione o un nuovo sistema di incassi; studia soltanto l'UX di attribuzione di incassi **già dichiarati** alle singole vendite associate.
- **[TECH-001](https://github.com/Cesare-ml/Repertica-Studio/blob/docs/idee-strumenti-architettura-20261121/docs/idee/strumenti-e-architettura/REPS-TECH-001-pacchetto-dati-aperto.md)** definisce interoperabilità generale; **[TECH-008](https://github.com/Cesare-ml/Repertica-Studio/blob/docs/idee-strumenti-architettura-20261121/docs/idee/strumenti-e-architettura/REPS-TECH-008-viste-salvate-filtri-ricerca-locale.md)** filtri e viste salvate. Nessuna riscrittura delle loro proposte o dei loro file.

**Esclusioni:** fatturazione fiscale, IVA multiregime, vendita di quote frazionate di un'opera, asta, carrello e-commerce, più clienti per singolo accordo, deduzioni automatiche di pagamento, calcolo di commissioni non concordate e multi-valuta automatica.

## 3. Scenari professionali

1. **Open studio, due opere e uno sconto globale.** Un collezionista acquista due opere, proposte a €700 e €300; prezzo concordato €900. Lo studio registra **€100 di sconto totale** ripartito espressamente €70/€30, producendo importi definitivi **€630 e €270**, il cui totale è esattamente €900. Nessuna opera risulta venduta prima della conferma dell'intera operazione.
2. **Incasso parziale unico.** Lo stesso collezionista versa, secondo dichiarazione manuale, €450 del totale €900. La schermata propone una ripartizione *in anteprima* di €315 e €135; l'utente la conferma o la corregge. I due incassi attribuiti restano separati dai €900 di fatturato e non diventano «pagamento bancario verificato».
3. **Conflitto su un'opera.** Prima di confermare la selezione, una delle opere risulta già venduta oppure archiviata. Si blocca **l'intero tentativo** con scelta esplicita fra tornare alla selezione e rinunciare: non vendere silenziosamente soltanto le opere valide.
4. **Accordo per una sola opera.** L'utente continua a usare il flusso attuale, senza dover creare lotti o gruppi vuoti. Le vendite storiche restano consultabili con i propri UUID e importi originari.
5. **Richiesta di restituzione di una sola opera.** Il riepilogo mostra l'appartenenza all'accordo comune, ma non inventa un rimborso, non cancella la vendita originaria e non ricalcola retroattivamente gli altri prezzi. La procedura post-vendita resta una decisione da coordinare con UX-013.

## 4. Flussi UX candidati

**A — Vendite → Nuova vendita.** Mantenere la modalità singola come principale, con azione facoltativa «Aggiungi un'altra opera». Ricerca nella lista locale, selezione di un unico acquirente già presente, data dell'accordo e riepilogo delle opere. Un'opera in elenco appare una volta sola. L'interfaccia distingue *prezzo suggerito*, *prezzo concordato prima dello sconto*, *sconto* e *importo definitivo per opera*.

**B — Ripartisci.** L'utente può immettere prezzi definitivi direttamente oppure un totale pattuito da ripartire, scegliendo **proporzione ai prezzi proposti** o **ripartizione manuale**. La strategia è visibile e non si sostituisce alla decisione umana. Gli arrotondamenti si calcolano **solo in centesimi interi**, con resto assegnato in modo deterministico e dichiarato; il riepilogo mostra sempre `somma righe = totale accordo`. Non proporre una ripartizione se manca una base attendibile: chiedere valori manuali. Per ogni riga prezzo definitivo strettamente positivo nell'attuale schema; omaggi gratuiti richiederebbero una decisione separata.

**C — Riepilogo prima di registrare.** Mostrare opere/codici, compratore, data, prezzo iniziale per riga, sconto allocato, importo netto concordato per riga, totale, imposte e trasporto *solo se esplicitamente registrati*, quote di destinazione e chiarimento «vendita registrata ≠ importo incassato». Indicare quali valori sono **storicizzati** e quali semplici suggerimenti del coefficiente artista. Tasto «Conferma tutte le opere», con esito unitario; errori associati alla riga responsabile, non un generico fallimento.

**D — Dettaglio accordo.** Vista di gruppo consultabile dalla scheda di ogni opera, dal cliente e dalla sezione vendite: numero/UUID interno, elenco delle singole vendite, totale concordato, stato di incassi manualmente registrati e residuo **dichiarato**; collegamento alle vendite e agli eventuali certificati senza modificarli. Non rappresentare un riepilogo commerciale come ricevuta fiscale. Se una singola riga è stata rettificata successivamente, mostrarlo anziché rifare il prezzo degli altri articoli.

**E — Incassi successivi.** Un'eventuale azione «Registra incasso del gruppo» deve restare **facoltativa e con conferma della ripartizione** verso le vendite già esistenti; nessuna riga può superare il proprio saldo residuo. Il sistema non deve creare un **settimo registro di incasso** o un conto bancario fittizio. Se manca una regola approvata per importi parziali, mantenere la registrazione esistente per singola vendita e mostrare solo un riepilogo dei saldi.

**Accessibilità:** tutti i valori editabili con tastiera; tabella leggibile da screen reader con intestazioni «prezzo base/sconto/prezzo finale», annunci testuali quando una somma non torna, focus sugli errori, etichette non basate solo su colore, zoom 200%, controllo di tabulazione coerente su Windows/macOS/Linux.

## 5. Dati e invarianti — ipotesi descrittive, NON schema approvato

Due alternative da confrontare prima di scegliere:
- **A — Contenitore leggero:** `sale_group` (UUID, data, `collector_id`, note, totale concordato in centesimi, criterio di ripartizione, data di creazione) con collegamento **facoltativo** fra le attuali `sales` e un gruppo. Ogni riga `sales` mantiene UUID, `artwork_id UNIQUE`, `amount_cents` positivo e `sale_finances`. Snapshot di prezzo iniziale e sconto per riga in metadati dedicati. Le vecchie vendite rimangono **senza gruppo**, non vengono raggruppate automaticamente.
- **B — Testata con righe:** nuovo `sale_order` + `sale_order_line` con righe collegate alle vendite. Richiede scelte importanti su migrazione, stampa, report, vincoli transazionali, compatibilità con incassi e import/export. Da non adottare senza dimostrare un vantaggio concreto rispetto ad A.

**Invarianti irrinunciabili:** uno e un solo acquirente per operazione (in prima fase); ogni opera al più una volta nel gruppo; importi in centesimi e valuta EUR, senza `float`; somma dei prezzi definitivi delle righe uguale al totale registrato, senza scarti da arrotondamenti; ogni riga rispetta le regole correnti di vendita e ha uno stato commerciale coerente; nessun importo pagato dedotto dal fatturato; quote finanziarie storicizzate per vendita; nessuna modifica implicita a certificati immutabili. Per imposte/spedizione comuni, **non dividere automaticamente** o duplicare costi: decidere prima se e come ripartirli in righe con somma invariata.

**Vincolo temporale:** una modifica successiva di listino, coefficiente o percentuali predefinite non ricalcola né gli importi definitivi né le quote delle vendite già confermate. Se UX-013 non sarà approvata, nessuna interfaccia di modifica post-conferma del gruppo deve promettere rettifiche parziali sicure.

## 6. Offline, privacy, impostazioni, multilingua

- **Offline e indipendenza:** dati unicamente nell'archivio SQLite locale, nessun account, abbonamento, cloud, pagamenti online o telemetria. Il normale backup a **cartella liberamente scelta** (USB/NAS montato inclusi) deve comprendere gruppo, righe, eventuali allegati gestiti internamente e incassi; ripristino verificato senza perdere la relazione fra righe. Backup preesistenti restano leggibili mediante migrazioni additive ove compatibili.
- **Privacy:** riepiloghi possono contenere nominativi del cliente, importi, margini e sconti riservati; esportare **solo su comando esplicito** e con anteprima, nascondendo note interne, recapiti, costi di produzione e quote personali nelle copie per terzi. I backup attuali sono in chiaro; avvertire prima di salvarli su cartelle condivise.
- **Impostazioni facoltative:** modalità iniziale singola, metodo suggerito per ripartire sconti (proporzionale/manuale), formato di stampa «interno/per cliente», valuta EUR vincolata al modello esistente, nessun campo obbligatorio oltre ai dati necessari all'accordo. Non modificare le percentuali di ripartizione dello studio senza conferma.
- **Lingue:** rilevamento lingua del sistema, scelta manuale IT/EN/ES/FR/DE, fallback inglese per locale o stringa non disponibile. Tradurre messaggi su somme, sconti, incassi e arrotondamenti; date/formati valuta resi secondo locale, **valori persistiti sempre in centesimi e codici stabili**. Le note inserite dall'utente non sono tradotte automaticamente.

## 7. Import/export e compatibilità

- **Export interno:** CSV UTF-8 con una riga per opera e colonne `group_id`, `sale_id`, `artwork_id`, `collector_id`, `agreed_total_cents`, `line_amount_cents`, `discount_allocated_cents`, valuta, date e criterio di ripartizione. Identificativi e intestazioni tecniche stabili, date ISO; non esporre percorsi assoluti o recapiti senza scelta. Mantenere i **sei CSV attuali** integrali per i loro domini; l'eventuale nuovo formato è aggiuntivo, non un sostituto di backup.
- **Riepilogo da stampare:** documento commerciale **non fiscale**, con una descrizione esplicita e data di generazione; versione «per cliente» priva di dati gestionali privati. Non chiamarlo fattura, scontrino o prova di pagamento.
- **Round trip futuro:** JSON versionato per gruppi e righe con UUID e riferimenti; preview, verifica totale, deduplicazione, risoluzione consapevole dei conflitti con vendite già esistenti, rollback atomico. Coordinarsi concettualmente con TECH-001 se mai approvata; **non** ridefinire o implementare la sua architettura.
- **Compatibilità storica:** le vecchie vendite **non** vengono riconciliate automaticamente in accordi collettivi sulla base di stesso acquirente/data; non riscrivere incassi, percentuali o certificati. Un archivio nuovo non deve richiedere la creazione di gruppi per usare le funzioni esistenti; i backup e la stampa storici restano accessibili con la versione capace di leggere il relativo schema.
- **Compatibilità piattaforme:** UI e stampa locale su Windows/macOS/Linux, locale e dialoghi nativi; nessuna dipendenza da provider esterni.

## 8. Rischi, alternative, complessità e dipendenze

**Rischi:** doppio conteggio dei ricavi (totale gruppo + vendite delle righe), ripartizioni arbitrarie che alterano margini, pagamento parziale attribuito due volte, errori di arrotondamento, sovrapposizione UX-013 per resi, cancellazione del gruppo con vendite già certificate o incassate, problemi privacy negli export. **Mitigazione centrale:** i report sommano **esclusivamente le vendite elemento**, non aggiungono mai anche il totale del gruppo; quest'ultimo è una vista/controllo di coerenza.

**Alternative:** (1) registrare manualmente vendite separate e annotare un riferimento comune nelle note — nessuna nuova funzionalità ma bassa verificabilità; (2) semplice etichetta comune di gruppo senza sconti, come MVP minimo; (3) software di contabilità esterno, mantenendo Repertica su vendite per opera. Evitare piattaforme ecommerce e contabilità completa.

**Complessità stimata:** media per raccolta e conferma atomica di più vendite; medio-alta per ripartizione sconti, tasse/spese condivise e incassi parziali; alta per resi parziali e generazione di documenti fiscali, **fuori ambito**.

**Dipendenze già disponibili:** UUID di opere/clienti/vendite, vincoli SQLite, registrazione finance, incassi per vendita, backup e i18n. **Dipendenze solo concettuali:** UX-002 (trattative), UX-013 (resi/storni), TECH-001 (formato aperto); nessuna è prerequisito approvato. Prima dell'eventuale scelta testare migrazione di archivi v1–v5 e tutela dei certificati.

## 9. Criteri di accettazione ipotetici

1. Vendere due opere a €630 e €270 come parte di un accordo da €900 produce **due vendite e un solo totale di €900**, mai €1.800; entrambe passano a `sold` **insieme**, oppure nessuna.
2. Un conflitto su una delle opere annulla il tentativo senza modificare disponibilità, vendite o quote delle altre.
3. Una ripartizione di sconto €70/€30 da basi €700/€300 conserva esattamente €100 di sconto e €900 di ricavo in centesimi; testare anche resti di 1 centesimo, tre righe e ordine delle righe variabile.
4. Registrare manualmente €450 di incassi attribuiti come €315+€135 incrementa **solo** il registro degli incassi per ciascuna vendita, senza aggiungere ricavi né superare saldi.
5. Una vendita storica senza gruppo e con certificato emesso resta identica dopo una futura migrazione: UUID, cliente, importi, percentuali e snapshot non cambiano.
6. Eliminare o rettificare un accordo collegato a certificati/incassi non aggira i vincoli della singola vendita; se non esiste UX-013, l'interfaccia non propone una cancellazione collettiva distruttiva.
7. CSV e futura reimportazione di prova preservano relazioni e importi, segnalano duplicati e non espongono di default recapiti o note private.
8. Intera operazione eseguibile senza rete e senza account; tastiera/screen reader, zoom elevato, lingue di sistema e fallback inglese sono coperti da verifiche Windows/macOS/Linux; backup/ripristino conserva la relazione di gruppo.

## 10. Fasi orientative, NON roadmap

- **Fase A:** osservazione di casi reali; scelta fra gruppo leggero e testata/righe; selezione multiopera con riepilogo e conferma atomica, **senza sconti condivisi né incassi di gruppo**.
- **Fase B:** sconto esplicito e ripartizione esatta; riepilogo gruppo e stampa locale senza dati riservati.
- **Fase C:** incasso manuale del gruppo con ripartizione confermata e interoperabilità semantica, **solo se** la validazione Finance 2 e i confini con UX-013 risultano chiari.

## 11. Requisiti certi, ipotesi e domande al proprietario

**Certi dal repository:** software desktop gratuito e open source senza account obbligatorio, SQLite locale, vendita una-per-opera, incassi manuali per vendita, backup completi a cartella scelta, importi EUR in centesimi, quote finanziarie snapshot, certificati immutabili, lingua OS e fallback inglese.

**Ipotesi da convalidare:** vendite multiopera sufficientemente frequenti; necessità di uno sconto **sull'intero accordo**; possibilità di una ripartizione del prezzo che professionisti trovino intuitiva; utilità del documento riepilogativo non fiscale.

**Questioni aperte per il proprietario:**
1. Quante vendite con 2+ opere avvengono realmente? Hanno un unico acquirente e una data comune?
2. Preferire **semplice gruppo di vendite autonome** oppure una testata di accordo con righe e sconti espliciti?
3. Ripartire sconto e costi accessori **per prezzo relativo**, manualmente o secondo altro criterio? Sono ammesse righe a importo zero (omaggi)?
4. Un acconto unico deve essere distribuito alle opere o può rimanere **non attribuito** fino alla riconciliazione manuale? Quest'ultima richiederebbe nuove regole, non presenti oggi.
5. Come trattare un **reso di una sola opera** senza alterare lo storico delle altre e gli eventuali documenti già emessi?
6. Serve un riepilogo cliente stampabile oppure bastano collegamenti e CSV privati?

**Nessuna proposta è approvata o assegnata. La decisione spetta esclusivamente al proprietario.**
