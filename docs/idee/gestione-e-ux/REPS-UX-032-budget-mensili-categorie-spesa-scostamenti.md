# REPS-UX-032 — Budget mensili facoltativi per categoria di spesa e scostamenti leggibili

- **ID stabile:** REPS-UX-032
- **Data della ricognizione:** 2026-11-22
- **Stato:** Proposta — da valutare
- **Filone:** Gestione e UX
- **Priorità ipotetica:** P2; P1 soltanto dopo riscontro da studi con spese periodiche significative
- **Ambito:** analisi e documentazione, **non** decisione di prodotto, implementazione o assegnazione

## 1. Esigenza e destinatari

**Problema verificabile nel prodotto:** Repertica Studio registra spese con data, categoria testuale, importo in centesimi, fornitore e parte/fondo opzionale. La vista economica calcola spese e margine **mensili consuntivi**, ma nel modello e nel codice di `main` consultati non emerge un budget preventivo per **mese e categoria** né un confronto esplicito fra importo preventivato e spesa registrata. Riferimenti: [finance.rs](../../../src-tauri/src/finance.rs), [modelli Angular](../../../src/app/models.ts), [Economia e i18n](../../ECONOMY-I18N.md). **Non è stato osservato un errore applicativo:** si tratta di un bisogno da validare.

**Utenti possibili:** artista indipendente che pianifica tele, colori, imballaggi e costi di studio; piccola galleria che desidera limitare spese operative per categoria; laboratorio che vuole distinguere oscillazioni stagionali. Il valore sta nel decidere *prima* quanto si intende spendere e nel vedere, *dopo*, quali registrazioni si discostano dal piano, senza mantenere un secondo foglio di calcolo.

**Principio cardine:** un budget è **un obiettivo dichiarato, non una spesa**, un pagamento, un saldo di cassa, una disponibilità bancaria o un utile fiscale. Non deve generare movimenti, alterare le ripartizioni dei guadagni né alimentare artificialmente le statistiche già esistenti.

## 2. Casi d'uso da mettere alla prova

1. **Studio indipendente:** per marzo l'artista imposta 100,00 € «Materiali» e 50,00 € «Stampe». Registra due spese «Materiali» da 72,00 € e 48,00 €, e una stampa da 30,00 €. La pagina indica **Materiali: 120,00 € registrati / 100,00 € previsti / +20,00 €**, **Stampe: 30,00 € / 50,00 € / −20,00 €**. Nessuna voce viene inserita una seconda volta nella contabilità.
2. **Spesa non preventivata:** compare una spesa «Trasporti» di 25,00 €, senza budget. Deve emergere sotto «Spese senza budget»; non deve sparire dal totale o essere rappresentata come budget di 0 € scelto dall'utente.
3. **Piano stagionale:** un laboratorio prepara target mensili differenti (non un solo tetto annuale diviso automaticamente per dodici). Può copiare i valori di un mese all'altro, **con anteprima e conferma**, correggendo manualmente i mesi speciali.
4. **Cambio di categoria:** una spesa già registrata passa da «Materiali» a «Stampe». Il consuntivo viene ricalcolato in base alla categoria attuale; il piano originario rimane visibile e non viene riscritto automaticamente.
5. **Categorie scritte diversamente:** «Cornici», «cornici» e «Cornici e telai» non devono essere fuse mediante supposizioni. Un'eventuale associazione semantica è decisa e revisionabile dall'utente dopo aver visto le righe interessate.

## 3. Esperienza utente proposta

**Economia → Budget spese** (voce/sezione facoltativa, non schermata obbligatoria): anno e mese coerenti con la vista finanziaria, griglia di categorie previste, importi e colonna «registrato». Per ogni categoria mostrare obiettivo, consuntivo, scostamento *consuntivo meno obiettivo*, percentuale solo con denominatore positivo, e quantità di spese collegate. Valori con etichette testuali, non solo barre rosse/verdi.

**Prima configurazione:** pulsante «Prepara un budget», scelta del mese, aggiunta di una o più categorie già comparse nel registro oppure categoria dichiarata nuova; importo di **0 € esplicito** permesso per una categoria che si vuole monitorare, distinto da una categoria senza alcuna riga di budget. Nessun piano inserito all'installazione e nessun target dedotto dai mesi precedenti. Se il budget è vuoto: «Nessun obiettivo impostato».

**Anteprima categorie:** mostrare l'elenco delle diverse etichette del registro e la quantità/importo delle voci che una categoria di budget includerebbe; consentire l'associazione esplicita delle varianti, con azione «Mostra spese incluse». Le correzioni ai raggruppamenti non modificano automaticamente la categoria testuale delle spese esistenti. Una categoria non riconosciuta resta nel gruppo «Non associata», visibile, senza creare budget impliciti.

**Riepilogo senza conti ingannevoli:** tre importi separati: (a) totale obiettivi configurati, (b) spese effettive delle categorie coperte, (c) spese effettive *senza* obiettivo. Mostrare anche il totale di tutte le spese del mese. **Non presentare un unico semaforo complessivo** come se tutto il registro fosse stato preventivato quando esistono spese fuori budget.

**Modifica volontaria:** «Modifica budget» apre il piano in anteprima, conferma e, se utile, una piccola nota di revisione «aumentato per costo telaio»; conservare una cronologia minima dei cambiamenti al target per spiegare differenze fra report salvati in date diverse. Una revisione non modifica spese, vendite, ricevute, prezzi, percentuali delle quote o certificati.

**Consultazione e accessibilità:** clic sulla riga per vedere le spese considerate, filtri per mese/categoria, ricerca testuale, esportazione dopo anteprima. Tasti, focus, lettori schermo, importi negativi dichiarati con parole («sotto/sopra obiettivo»), messaggi d'errore vicino ai campi e nessuna dipendenza dal colore. Evitare un diagramma come unica forma di lettura.

## 4. Regole contabili e casi-limite

- **Consuntivo categoria/mese** = somma in **centesimi interi** di `expenses.amount_cents` con `expense_date` nel mese, categorizzate secondo mapping *esplicito*. È un ammontare di **spese registrate**, non pagamenti bancari confermati. Il riferimento temporale è la **data della spesa**, come nell'attuale `finance_overview`, non la data di registrazione o una rata prevista.
- **Scostamento** = consuntivo − target (valore positivo = sopra obiettivo). Percentuale solo quando il target è maggiore di zero. Per target zero e spesa positiva: scrivere «oltre obiettivo di X €», **senza divisioni per zero**. Target assente ≠ target zero.
- **Totale mensile spese** già autorevole = somma di **tutte** le spese registrate, anche quelle non mappate; il nuovo pannello non calcola un nuovo saldo finanziario e non cambia il `margin_cents` esistente.
- **Fondi/parti percentuali:** il piano iniziale riguarda **categorie**, non fondi `personal/activity/savings` o quote congelate delle vendite. Una spesa imputata a un fondo viene conteggiata una volta sola. Eventuali budget distinti per fondo richiedono una decisione successiva esplicita, per non confondere due dimensioni differenti.
- **Periodo della tesoreria:** il campo `treasury_start_date` già filtra i report. La vista budget deve mostrare chiaramente se le spese precedenti a tale data non entrano nel confronto (comportamento proposto: seguire la stessa regola del riepilogo esistente, con avviso visibile per mesi parziali). I dati storici non devono essere cancellati.
- **Valute:** `main` contabilizza attualmente le spese in EUR; se in futuro fosse approvata [UX-027](REPS-UX-027-valute-estere-vendite-incassi-costi.md), ogni confronto dovrà rimanere nella stessa valuta o dichiarare un criterio esplicito di conversione. Non sommare USD a EUR e non recuperare cambi di rete senza richiesta.
- **Anni e ricalcoli:** un anno senza target mostra solo consuntivi; l'editing di una spesa storica aggiorna la vista dinamica, ma un report esportato in precedenza resta una fotografia datata dei numeri allora noti.

## 5. Strutture dati / API ipotetiche, *solo descrittive*

- **`expense_budget_category`:** `id` UUID, nome visualizzato libero, stato attivo/archiviato, data di creazione; eventuale elenco versionato di *etichette esatte* del registro spese che l'utente ha collegato manualmente. Non dedurre codici universali da una categoria in testo libero.
- **`expense_budget_target`:** `id` UUID, `month` nel formato `YYYY-MM`, `category_id`, `target_cents` intero ≥ 0, `currency` inizialmente EUR, `revision`, note opzionali e timestamp. Unicità di mese/categoria/valuta per la **revisione attiva**; target inesistente rappresentato da *assenza record*.
- **`expense_budget_revision`** (seconda fase, se serve): riferimento al target, importo precedente/nuovo, data e motivazione dichiarata. Non dedurre un utente autore in un'app senza login.
- **Proiezione di sola lettura:** `budget_vs_actual(month, category_id) → { target_cents?, actual_cents, variance_cents?, count, matched_expense_ids, source_revision }`. Non è una nuova API operativa approvata; i dati effettivi devono provenire dal registro `expenses`, non da duplicazioni.
- Eventuali mapping con categorie testuali vanno verificati all'import o in modifica. Evitare merge per maiuscole/minuscole, sinonimi, traduzioni o regole probabilistiche senza esplicita conferma; non riscrivere vecchie righe.

## 6. Impostazioni, lingua, offline, privacy e backup

- **Impostazioni:** visibilità del pannello budget (disattivo finché non viene creato un piano); possibilità di copiare un mese come **bozza confermata**; prefissati del calendario compatibili con anno già in uso. Evitare soglie e notifiche obbligatorie, profili cloud e automatismi di registrazione spese.
- **Multilingua:** stessa selezione della lingua di sistema/override locale in IT, EN, ES, FR, DE e **fallback inglese** anche per messaggi mancanti, secondo [ECONOMY-I18N](../../ECONOMY-I18N.md). I nomi delle categorie e le note *scritte dall'utente* non vanno tradotti automaticamente; numeri e date si visualizzano nel locale ma si serializzano in centesimi/ISO invarianti.
- **Offline e sicurezza:** SQLite e backup solo locali, nessuna API bancaria, analitica cloud, SaaS o login; nessuna esportazione o invio automatico. I budget sono dati commerciali privati: non esporli per default su brochure, certificati, schede da inviare, ricerca MCP locale o assistenti esterni. Per accessi MCP futuri la visibilità richiederebbe una scelta separata del proprietario.
- **Backup:** target, categorie, mapping e revisioni entrano nel **backup completo verificabile** già presente, su cartella liberamente selezionata. Il ripristino recupera i target e gli stessi collegamenti senza richiedere servizi esterni; un archivio vecchio rimane utilizzabile senza obiettivi.
- **Compatibilità:** schema additivo eventuale, nessuna riscrittura dei record `expenses`; i form di vendita, incasso e ripartizione economica restano indipendenti. Le copie antecedenti alla nuova funzione mantengono il loro formato storico, senza target sintetici inventati.

## 7. Import, export e interoperabilità

- **Export umano:** CSV locale dei target per mese, categoria, valuta e importo in centesimi; secondo CSV opzionale di confronto con target, consuntivo, scostamento, periodo, data esportazione e **copertura del budget**, distinguendo «senza obiettivo» da 0. Anteprima dei valori e della destinazione; nessun overwrite implicito. Formato coerente con gli attuali CSV UTF-8 e protezione da formule.
- **Export completo/round trip futuro:** JSON versionato con UUID, categorie, mapping, revisioni e target; integrazione soltanto concettuale con [TECH-001](https://github.com/Cesare-ml/Repertica-Studio/blob/docs/idee-strumenti-architettura-20261121/docs/idee/strumenti-e-architettura/REPS-TECH-001-pacchetto-dati-aperto.md). Non usare i CSV come backup; il backup SQLite+media continua ad essere autorevole.
- **Import opzionale:** file scelto manualmente, anteprima categorie duplicate, date/valuta, target zero, conflitti mese/categoria e mapping mancanti; transazione annullabile, chiavi UUID idempotenti e scelta esplicita «sostituisci questo obiettivo / salta». Non importare o generare spese da un file budget.
- **Dati sensibili:** l'export dei target potrebbe essere condivisibile, ma quello di confronto può rivelare strategie d'acquisto e fornitori: nessun nome di cliente, indirizzo, nota privata o lista dettagliata delle spese viene incluso automaticamente.

## 8. Dipendenze, rischi, alternative e complessità

- **Dipendenze attuali:** registro `expenses`, data/categoria/importo, `finance_overview`, i18n, esportazione su cartella scelta e backup. Nessun nuovo servizio o dipendenza tecnica obbligatoria.
- **Complessità:** **bassa-media** per target manuale mensile e confronto con categorie esatte; **media** per mapping guidato, revisioni e report senza doppio conteggio; **medio-alta** per import coerente e supporto di più valute. Evitare di trasformarla in contabilità analitica generale.
- **Rischio di falsa precisione:** categoria libera, voci non previste, data errata o spesa modificata fanno oscillare i numeri. Mitigazione: anteprima spese incluse, categoria non associata visibile, «dati registrati» e indicatore di copertura.
- **Rischio gestionale:** prendere «budget residuo» per liquidità realmente disponibile o confondere obiettivi con fondi di ripartizione. Mitigazione: lessico «obiettivo/registrato» e nessuna metrica bancaria.
- **Rischio UX:** troppi campi per artisti che non pianificano per categoria. Mitigazione: attivazione facoltativa, uno o due target iniziali, alternativa minima e nessun blocco di spese sopra obiettivo.
- **Alternativa minima consigliata:** un **solo target mensile totale** e un indicatore «spese registrate / obiettivo», senza categoria, mapping, revisioni o import: costa meno e può bastare a uno studio piccolo. **Alternativa esterna:** foglio Calc/Excel locale con CSV esportato manualmente.
- **Confini anti-doppione:** [UX-003](REPS-UX-003-costo-e-margine-per-opera.md) attribuisce spese **alle opere**, non definisce un piano **per mese/categoria**; [UX-008](REPS-UX-008-scorte-materiali-consumi.md) controlla **quantità fisiche**; [UX-009](REPS-UX-009-progetti-espositivi-checklist.md) gestisce **mostre e attività**, non budget di studio; [UX-030](REPS-UX-030-quadratura-facoltativa-estratti-conto-locali.md) verifica corrispondenze con movimenti bancari, non obiettivi preventivi. [TECH-002](https://github.com/Cesare-ml/Repertica-Studio/blob/docs/idee-strumenti-architettura-20261121/docs/idee/strumenti-e-architettura/REPS-TECH-002-automazioni-locali-consapevoli.md) riguarda **regole/avvisi/azioni autorizzate**, che questa idea non richiede. [TECH-025](https://github.com/Cesare-ml/Repertica-Studio/blob/docs/idee-strumenti-architettura-20261121/docs/idee/strumenti-e-architettura/REPS-TECH-025-esportazioni-tabellari-minimizzate-condivisione.md) riguarda selezione sicura di colonne per terzi: è una possibile futura integrazione, non una sovrapposizione. **Non modificare i loro file.**

## 9. Criteri di accettazione *ipotetici*

1. Budget marzo: Materiali 10.000 centesimi, Stampe 5.000; spese Materiali 12.000, Stampe 3.000, Trasporti **senza budget** 2.000. Mostra +2.000 e −2.000 per le prime due, 2.000 «non preventivati», **17.000** come totale spese registrate. Il pannello non finge che la copertura budget sia totale.
2. Impostare target zero e avere una spesa produce avviso comprensibile senza percentuale infinita; una categoria senza target resta «senza budget», non uguale a zero.
3. Se la stessa spesa ha un `fund` e un collegamento a un'opera (UX-003 futura), il consuntivo categoria la conta **una volta sola**, e la vista economica mensile continua a riportare lo stesso importo.
4. Copiare il piano di marzo ad aprile non crea `expenses` né conferma spese; una modifica di target è tracciabile e non altera i dati del mese precedente.
5. Correggere la categoria di una spesa o la data del suo record provoca il ricalcolo della vista, con rinvio alla riga originale e senza correzioni automatiche a vendite/incassi.
6. Import ripetuto del medesimo piano non duplica target; invalidità di importo, mese, mapping o valuta impediscono una modifica parziale non confermata. Backup/ripristino conserva target/revisioni/UUID, in cartella scelta e senza rete.
7. Test con lingua sistema non supportata → fallback inglese; date, importi, navigazione da tastiera e screen reader coerenti su Windows/macOS/Linux. Nessun obbligo di configurare budget per usare Economia.

## 10. Fasi di esplorazione, **non roadmap approvata**

- **A — Validazione del bisogno:** 5–8 prove qualitative con artisti e piccole gallerie; verificare se un solo obiettivo mensile risolve il problema, frequenza delle categorie effettivamente usate e confusione fra «preventivato» e «pagato».
- **B — MVP eventuale:** budget per 1–3 categorie, confronto locale leggibile, spese non coperte ben visibili; nessuna automazione, nessun import.
- **C — Qualità e interoperabilità:** mapping esplicito delle etichette, eventuali revisioni, CSV/JSON e collaudi backup con categorie/Unicode/mesi parziali.
- **D — Solo se richiesto:** import/versionamento più articolato e scenari multi-valuta; non introdurre contabilità di cassa o strumenti previsionali automatici.

## 11. Certezze, ipotesi e domande al proprietario

**Requisiti certi del progetto:** desktop Tauri open source gratuita; offline-first e database locale, nessun account/abbonamento, backup e scelta cartella, dati esportabili; spese categorizzate per data, importo e parte facoltativa, riepiloghi mensili, lingua OS con fallback inglese. `main` non descrive target mensili per categoria nel modello consultato.

**Ipotesi da validare:** budget per mese/categoria più utile di un unico tetto mensile; le categorie esistenti sono abbastanza stabili per un confronto; piccoli studi desiderano pianificazione senza report fiscali; i nomi duplicati sono un problema pratico e non solo teorico.

**Domande per revisione personale:** È preferibile il minimo «tetto spese mensile» o categorie nominate? Serve anche il budget annuale, oppure bastano dodici mesi? Il target deve riguardare soltanto le spese o anche ricavi **previsti** (campo semanticamente diverso)? Che rapporto deve avere con `treasury_start_date` quando il mese è parziale? Le varianti di categoria vanno gestite con mappatura o è sufficiente correggere manualmente il testo nella spesa? Quando un budget viene modificato, è indispensabile la cronologia?

**Esito:** soltanto proposta documentale. **Nessuna approvazione, implementazione o assegnazione**.