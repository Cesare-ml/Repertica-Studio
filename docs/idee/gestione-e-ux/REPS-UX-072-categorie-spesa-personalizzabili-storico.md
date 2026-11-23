# REPS-UX-072 — Categorie di spesa personalizzabili senza alterare lo storico

- **ID stabile:** REPS-UX-072
- **Data:** 2026-11-23
- **Stato:** **Proposta — da valutare**
- **Filone:** Gestione e UX
- **Priorità ipotetica:** **P2** soltanto se la necessità di distinguere categorie aggiuntive è verificata con professionisti; **P3 o nessuno sviluppo** se «Altro» con descrizione e CSV sono sufficienti.
- **Decisione:** nessuna approvazione, nessuna assegnazione allo sviluppo. Documento esclusivamente ideativo, non una specifica implementativa autorizzata.

## Evidenza sul prodotto esistente e problema

Ricognizione sul branch `main` del 23 novembre 2026:

- `src/app/app.component.ts` (circa righe 161–170) dichiara **otto categorie fisse** nell'interfaccia: materiali pittura, spedizioni/imballaggi, attrezzatura, software/servizi, mostre/fiere, marketing, formazione e «Altro». Nella finestra della spesa (`src/app/app.component.html`, circa righe 966–970) si sceglie esclusivamente da questo elenco.
- La tabella economica mostra la categoria salvata e un'etichetta tradotta se il codice è conosciuto; altrimenti mostra il codice grezzo (`categoryLabel`, `app.component.ts`).
- Il backend `src-tauri/src/finance.rs` salva `expenses.category` come stringa non vuota, con limite di lunghezza; **non impone l'elenco della UI**. L'esportazione CSV degli acquisti conserva già il campo `category`. Il riepilogo mensile usa gli importi delle spese: il codice categoria non è oggi una ripartizione contabile autorevole.
- `docs/ECONOMY-I18N.md` distingue correttamente le **spese registrate** da uscite bancarie effettive e dagli incassi. Conservare questa distinzione.

**Problema di lavoro da verificare:** un artista che deve distinguere «Cornici», «Fonderia», «Affitto laboratorio» o «Trasporti per fiera» può scegliere soltanto una categoria generica («Altro» oppure una voce approssimativa), perdendo una classificazione leggibile e coerente tra registrazioni. Il fornitore e la descrizione non risolvono sempre il problema (lo stesso fornitore può erogare servizi diversi). Con archivi grandi, rettificare manualmente molte descrizioni può risultare poco pratico.

**Non è un bug dimostrato.** Il campo dati è già tecnicamente flessibile; la lacuna identificata riguarda soprattutto la **gestione comprensibile e stabile delle categorie nell'interfaccia**, non una presunta impossibilità strutturale di salvarle.

## Destinatari e casi d'uso

1. **Artista con laboratorio:** registra pigmenti, cornici, restauro, stampa e attrezzatura come voci diverse; verifica una categoria prima di confermare la nuova spesa.
2. **Galleria:** vuole distinguere allestimenti, movimentazioni e promozione senza forzare tutto in «Altro»; una categoria ritirata deve restare leggibile nelle spese degli anni precedenti.
3. **Archivista/professionista che riceve un archivio esistente:** apre registrazioni con codici di categoria che non figurano fra le otto voci standard; non deve perdere il valore originale né ritrovarsi una categoria «sconosciuta» falsamente normalizzata.
4. **Utente multilingue:** passa da italiano a inglese; i codici delle categorie predefinite rimangono invariati, mentre le categorie create dall'utente conservano il nome da lui inserito senza traduzione inventata.

## Specifica funzionale proposta (solo descrittiva)

**Fase minima da preferire, dopo verifica del bisogno:** nella selezione della categoria aggiungere «Crea categoria…», con nome obbligatorio e convalida preventiva. Mostrare l'elenco delle categorie personali accanto a quelle standard, senza sostituire le otto già presenti. Un comando «Gestisci categorie» consente di **rinominare l'etichetta di una categoria personale** senza cambiare l'identità degli acquisti già collegati e di **archiviarla**, non eliminarne l'identità quando è usata nello storico.

Possibili controlli:
- Anteprima immediata del nome della categoria e della sua eventuale somiglianza (normalizzazione solo per confronto: spazi, maiuscole, accenti); avviso **non bloccante**, nessuna fusione automatica.
- Elenco delle categorie «Attive» e «Archiviate». Le archiviate non compaiono per default nelle nuove spese, ma restano identificabili, selezionabili in modifica storica quando appropriate, esportabili e ripristinabili.
- Nelle registrazioni esistenti, mostrare **nome leggibile** e, se necessario, «Categoria non presente nell'elenco» senza cancellare il codice proveniente da versioni precedenti o da database importati.
- Nessuna assegnazione automatica delle categorie in base a fornitore, descrizione, AI o allegati. Nessuna riclassificazione retroattiva di massa come conseguenza della rinomina.

### Flussi UX

**A. Nuova spesa:** Economia → «Registra spesa» → Categoria → «Crea categoria» → nome e possibile avviso di somiglianza → conferma → categoria selezionata nella bozza → l'utente completa importo/data/fondo e **salva esplicitamente la spesa**. Uscire dalla finestra prima del salvataggio non deve creare alcuna spesa (coordinarsi con REPS-UX-067, modifiche non salvate).

**B. Revisione dell'elenco:** Impostazioni → «Categorie delle spese» → consulta categorie di sistema e personalizzate → rinomina o archivia la categoria personale → vede quanti record potrebbero usarla → conferma. Per le categorie standard, si preferisce inizialmente **nessuna rinomina/disattivazione**: evitare conflitti con localizzazione e comparabilità dei dati.

**C. Consultazione storica:** apre una spesa registrata due anni prima con categoria poi archiviata → legge sempre l'etichetta e l'eventuale indicazione «archiviata» → può salvare altre modifiche senza dover cambiare categoria. Se la categoria sconosciuta proviene da dati legacy, mostrarne il valore originario senza trasformarlo automaticamente.

**D. Ripristino/esportazione:** esporta CSV → ritrova codice originale, non etichetta tradotta; ripristina un backup completo → recupera sia le spese sia il vocabolario personale, senza dover ricreare categorie a mano.

### Dati necessari e compatibilità (ipotesi, non API da implementare)

- **Già presenti e da preservare:** `expenses.id`, `category` (stringa), `amount_cents`, `expense_date`, `fund`, dati fornitore e note. L'importo resta in centesimi e la ripartizione per fondi non cambia.
- **Solo se necessaria una gestione persistente delle etichette:** piccolo registro locale di categorie personali con identificatore **immutabile**, etichetta utente, stato attiva/archiviata, eventuali date e un ordine di visualizzazione. Esempio concettuale: `expense_category { id, label, archived_at?, sort_order? }`. Questa struttura non è richiesta nella prima fase di sola ricerca utente.
- **Identità vs etichette:** mai usare il testo tradotto o una semplice rinomina come chiave del dato. I codici delle otto categorie native devono restare interpretabili dalle vecchie versioni e dagli export. Una categoria personalizzata deve restare distinta da quelle standard anche se ha lo stesso nome visualizzato.
- **Categorie sconosciute:** conservare e mostrare il valore originario; nessuna cancellazione, conversione o correzione automatica. Valutare prefissi/namespace dei nuovi codici per scongiurare collisioni con future categorie native.
- **Compatibilità:** prima dello sviluppo verificare schema SQLite attuale, retrocompatibilità dei CSV, backup/restore completo, migrazioni e comportamento dei client vecchi quando incontrano codici personali. Non promettere apertura di archivi futuri da versioni vecchie dell'app.
- **Non include:** cambi automatici alle categorie delle spese storiche; nuove formule per margini/fondi; piani di budget; pagamenti; estratti bancari; modifica dei registri tramite CSV; tassonomie gestite via server.

## Offline, privacy, lingua, impostazioni e accessibilità

- **Offline-first:** elenco, rinomina, archiviazione e consultazione risiedono soltanto nel database locale. Nessun account, chiamata di rete, pacchetto di categorie remoto o abbonamento. L'utente può sempre usare categorie native senza configurarne altre.
- **Privacy:** i nomi possono contenere informazioni economiche o su progetti riservati; non inviarli a terzi. Backup e CSV possono includerli e vanno trattati come gli altri dati dello studio. Nessuna telemetria.
- **Multilingua:** lingua dell'interfaccia automaticamente rilevata dal sistema, selezione manuale già esistente e fallback **inglese**. Le categorie native sono etichette tradotte tramite il dizionario applicativo; le personali conservano il **testo letterale dell'utente**, senza falsa traduzione, con supporto Unicode e lettura assistiva. Facoltativa in futuro una traduzione manuale per lingua, ma non nella fase minima.
- **Impostazioni:** se necessario, una singola voce «Categorie delle spese» nell'area preferenze; evitare una nuova sezione di contabilità. Preferire archiviazione alla cancellazione per le categorie usate e azioni testualmente chiare «Archivia» / «Riattiva».
- **Accessibilità:** selezione con tastiera e screen reader; etichette comprensibili, focus e annunci dopo creazione/archiviazione; niente distinzione affidata solo al colore; contrasto e zoom nativi, ordinamento stabile; errori con istruzioni. Coordinamento con la proposta tecnica REPS-TECH-020 senza modificarla.

## Import/export e indipendenza dell'utente

Il **CSV delle spese rimane esportabile con il valore stabile `category`**, come oggi. L'eventuale registro delle categorie personali deve essere incluso nel **backup completo verificato**; valutare, prima di aggiungere nuove esportazioni, se basti una semplice tabella CSV/JSON aperta contenente identificativo e etichetta, così da leggere le categorie anche senza Repertica. Non perdere record se la tabella opzionale non è disponibile: mostrare il codice esistente come dato di fallback. Nessuna importazione automatica né trasformazione opaca dei CSV.

## Complessità, dipendenze, rischi e alternative

- **Complessità:** **bassa–media** se basta inserire nuove voci UX senza rinomina; **media** se servono registro persistente, archiviazione, vecchi codici e migrazione; **medio-alta** se si tentano riclassificazione massiva, gerarchie o traduzioni multiple, **qui escluse**.
- **Dipendenze concettuali:** registro spese locale, schema e export esistenti, dizionario i18n, dialoghi accessibili e backup. Nessun cloud o componente esterno necessario.
- **Rischio 1 — deriva semantica:** rinominare una categoria esistente cambia il significato apparente di acquisti passati. Mitigazione: codice stabile, storico immutato, rinomina solo dell'etichetta, avviso su quantità di record interessati e possibilità di mantenere vecchio nome se rilevante.
- **Rischio 2 — frammentazione:** proliferano sinonimi «Cornice», «Cornici», «Incorniciatura». Mitigazione: suggerimento esplicito di categorie simili, nessuna fusione automatica.
- **Rischio 3 — categorie invisibili:** un record importato o una categoria archiviata scompare dal selettore e viene risalvato con default. Mitigazione: mantenere esplicitamente il valore originale nella modifica e testare la riapertura.
- **Rischio 4 — portabilità retroattiva:** client precedenti non riconoscono le categorie custom. Mitigazione: i vecchi record conservano codice e CSV; documentare che le versioni vecchie possono non mostrare etichette e non garantire migrazioni inverse.
- **Alternativa A, preferibile prima di sviluppare:** restare con «Altro», descrizioni e CSV; verificare se la domanda è solo una migliore vista dei dati già registrati.
- **Alternativa B:** campo categoria testuale libero per singola spesa, senza registro: più semplice, ma favorisce frammentazione e non soddisfa l'obiettivo delle categorie persistenti.
- **Alternativa C:** vocabolario controllato con sole categorie native e una breve guida agli usi: quasi nessuna complessità, ma non copre la diversità di artisti/gallerie.

## Requisiti certi, ipotesi, domande al proprietario

**Certi (principi richiesti):** gratuito e open source; Tauri e database locale; offline anche senza account; nessun abbonamento; esportazione piena; backup in cartella scelta; lingua di sistema con fallback inglese; nessuna modifica automatica di importi, fondi o dati storici.

**Osservati nel repository:** elenco front-end di otto categorie; stringa `category` libera nel backend; esportazione CSV; statistica economica aggregata; supporto multilingue esistente.

**Ipotesi da verificare:** gli utenti hanno bisogno di **categorie persistenti aggiuntive** e le riutilizzano abbastanza spesso da giustificare un registro; non basta un campo descrittivo; l'archiviazione delle categorie è comprensibile.

**Domande riservate al proprietario (non bloccanti per questa ricerca):**
1. È prioritario distinguere spese di laboratorio, mostre/fiere e costi fissi senza ricorrere a «Altro»?
2. Vuoi consentire soltanto categorie aggiuntive, oppure anche rinominare/disattivare quelle native, accettando maggiore complessità?
3. È sufficiente raggrupparle visivamente nell'elenco/CSV, senza introdurre grafici per categoria in questa proposta?
4. Nel rinominare una categoria, preferisci mantenere una fotografia del nome storico, oppure la nuova etichetta può comparire anche nelle consultazioni passate?

## Fasi e criteri di accettazione proposti (nessun test eseguito)

**Fase 0 — domanda reale:** interviste/osservazioni con 4–6 professionisti che usano regolarmente il registro spese. Chiedere due casi in cui «Altro» provoca errori o ripetizione, e provare un semplice uso del CSV prima di qualunque implementazione. **Stop** se il bisogno non è frequente.

**Fase 1 — prototipo UX senza migrazione:** provare con utenti il selettore, la creazione, i nomi simili, le categorie archiviate, il ritorno a una vecchia spesa e la lettura con tastiera/screen reader. Valutare se una sola categoria custom risolve già il lavoro quotidiano.

**Fase 2 — solo se validata:** definizione del contratto dati e compatibilità, struttura locale minima, dialoghi tradotti e documentazione del CSV/backup. Il codice non è oggetto della presente PR.

**Fase 3 — collaudo futuro, se approvato separatamente:**
1. Creare categoria personale e salvare una spesa **solo dopo conferma**; eliminando la bozza non si crea una spesa.
2. Ridenominare/archiviare/riattivare una categoria non cambia `id`, `category`, importi, data, fondo o fornitore delle spese.
3. Una spesa storica con codice archiviato/sconosciuto si apre, si modifica e si salva **senza rimpiazzare silenziosamente** la categoria.
4. Cambiando lingua IT → EN → DE, le categorie native sono tradotte, quelle personali restano identiche; per locale non supportato il resto della UI usa inglese.
5. Backup → ripristino conserva categorie personali ed etichette; l'export CSV mantiene codici stabili e importi in centesimi. Nessun invio di dati in rete.
6. Con tastiera e lettore assistivo si può creare e riconoscere una categoria archiviata; errori e conferme non dipendono solo dal colore.
7. Con migliaia di spese e categorie, il selettore resta utilizzabile e la ricerca per nome non richiede cloud.

## Distinzione dalle idee già presenti

- **REPS-UX-032**: budget mensili per categorie; qui **solo denominazione/riutilizzo/storicità delle categorie**, senza budget.
- **REPS-UX-069**: ripetere manualmente una spesa esistente; qui **vocabolario condiviso** per spese nuove o pregresse, senza clonazione.
- **REPS-UX-040/042**: rimborsi e pagamenti di spese; qui nessuna movimentazione finanziaria.
- **REPS-TECH-030**: vocabolari facoltativi di **tecniche e supporti delle opere**; qui categorie delle **spese**, senza intervenire in quel filone.
- **REPS-TECH-057**: dizionario dei registri CSV; questa proposta non ridefinisce il contratto CSV.
- **REPS-UX-064**: modifica guidata massiva dei metadati delle **opere**; qui nessuna riscrittura massiva delle spese.

La scheda rimane **Proposta — da valutare**: non implica sviluppo, merge, issue operative né approvazioni.
