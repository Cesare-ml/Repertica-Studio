# REPS-UX-034 — Analisi delle vendite per caratteristiche delle opere, con metriche dichiarate

- **ID stabile:** REPS-UX-034
- **Data della ricognizione:** 2026-11-22
- **Stato:** Proposta — da valutare
- **Filone:** Gestione e UX
- **Priorità proposta:** P2, *ipotetica*. P1–P2 soltanto dopo verifica con artisti/gallerie che hanno uno storico sufficiente; P3 se i riepiloghi mensili e un CSV già bastano.
- **Natura:** ricerca e specifica esclusivamente documentale, **senza approvazione né incarico di sviluppo**.

## 1. Problema concreto, destinatari e cosa è stato verificato

Un artista o una galleria con decine o centinaia di opere vendute vorrebbe capire, con prudenza, **quali tecniche, supporti e periodi di vendita** caratterizzano il proprio storico: «quante opere a olio ho venduto quest'anno?», «a quali importi complessivi corrispondono?», «quanti record hanno tecnica non compilata?». Il riepilogo mensile delle entrate non risponde direttamente a queste domande; esportare e incrociare manualmente registri distinti richiede lavoro e può portare a somme duplicate.

**Evidenza circoscritta a `main` del 22 novembre 2026:** in [`src/app/models.ts`](../../../src/app/models.ts) `Artwork` espone `technique`, `support`, `year` e `status`; `Sale` collega `artwork_id`, `collector_id`, `amount_cents` e `sold_at`. In [`src-tauri/src/finance.rs`](../../../src-tauri/src/finance.rs), `finance_overview_core` produce totali per mese di vendite, spese e quote. [`docs/ECONOMY-I18N.md`](../../ECONOMY-I18N.md) distingue ricavo **registrato** da denaro **effettivamente incassato**. [`README.md`](../../../README.md) descrive ricerca e filtri del catalogo ma non, nei percorsi consultati, una tavola di analisi delle vendite per tecnica/supporto. **Non è una prova di difetto né di domanda di mercato:** la frequenza della necessità va indagata.

**A chi potrebbe servire:** artista con catalogo storico; galleria che presenta opere di tecniche differenti; piccolo archivio che prepara un bilancio descrittivo non fiscale. A un artista con poche vendite può bastare l'export CSV: non imporre il modulo alla UI principale.

## 2. Anti-doppione e confini rigorosi

- **REPS-UX-003** studia il **costo attribuito all'opera** e i margini per singola opera; qui si raggruppano *vendite già registrate*, senza inventare costi di produzione o utili.
- **REPS-UX-017** studia un **unico accordo per più opere** e la ripartizione dei prezzi: questa proposta non crea né modifica accordi, e deve continuare a contare ciascuna opera una sola volta se in futuro esisteranno accordi multiopera.
- **REPS-UX-023** riguarda prezzi **richiesti/listini**, non analisi dei prezzi di vendita effettivamente registrati. **REPS-UX-032** riguarda budget **di spesa**, non gruppi di vendite. **REPS-UX-014** riguarda la rubrica professionale, non profilazione dei clienti.
- **REPS-TECH-008** ([filone tecnico, PR #16](https://github.com/Cesare-ml/Repertica-Studio/pull/16)) riguarda filtri e **viste salvate delle schede**; qui il risultato principale è una lettura **aggregata** del registro vendite, con definizioni e limiti contabili espliciti. **REPS-TECH-025** studia esportazioni selettive riservate; questa scheda chiede un output statistico non identificativo per impostazione predefinita. Non toccare né replicare il motore di ricerca o i file del filone tecnico.
- **Fuori ambito:** forecast di domanda, ranking dei clienti, scoring commerciale, geolocalizzazione, invii marketing, attribuzione causale del successo artistico, analisi automatica delle immagini, dati di mercato esterni, fatturazione, IVA o collegamenti bancari.

## 3. Scenari di utilizzo da validare

1. **Valutazione del lavoro svolto:** l'artista sceglie il 2026 e vede che tre opere sono state vendute: due con tecnica attuale «Olio» (200 € e 300 €) e una «Acquerello» (500 €). Il raggruppamento mostra **Olio: 2 opere / 500 €; Acquerello: 1 opera / 500 €**. Sono vendite registrate, *non incassi*. Nessuna vendita viene modificata.
2. **Metadati incompleti:** cinque opere vendute hanno `technique = NULL`. Appaiono nel gruppo **«Tecnica non specificata»** e nel totale complessivo, non vengono escluse o classificate automaticamente come «Altro».
3. **Scheda corretta dopo la vendita:** una tecnica viene modificata da «Mista» a «Olio». Una vista che raggruppa *la classificazione attuale* cambia coerentemente: l'interfaccia dichiara **«Tecniche delle schede attuali, non fotografate alla data della vendita»**. Non si sostiene che sia cambiata la vendita storica.
4. **Confronto mensile:** il mese evidenziato mostra il numero di opere vendute, importi lordi registrati e un eventuale netto commerciale **solo se calcolabile dalle registrazioni storiche**; le rate incassate in mesi differenti non sono travestite da nuove vendite.
5. **Nessuna vendita o dati scarsi:** mostrare stato vuoto e spiegazione dei limiti; evitare medie o percentuali senza denominatore. Nessun indicatore «tecnica migliore» da un campione di una o due opere.
6. **Esportazione volontaria:** CSV dei soli gruppi e totali, senza nome/email del collezionista né ubicazione delle opere; dettagli riga per riga disponibili solo tramite scelta esplicita e anteprima.

## 4. Flusso UX proposto

**Economia → Analisi vendite** come pannello *facoltativo*, non un cruscotto che compete con la panoramica mensile. Apertura in sola lettura, con intervallo temporale sul campo **data vendita `sold_at`** e scelta **Raggruppa per tecnica / supporto / anno dell'opera**. La scelta dell'anno dell'opera non equivale all'anno di vendita; il testo deve distinguerli.

**Tabella principale prima dei grafici:** gruppo, numero di opere vendute, somma lorda delle vendite registrate, quota sul totale solo se significativa, eventuale valore medio per opera (se (n>0)). Riga «Non specificato» sempre visibile; totale di controllo sempre visibile. I filtri possono restringere il periodo o una tecnica, ma il loro stato deve essere indicato sopra la tabella e negli export.

**Dettaglio di gruppo:** azione «Mostra opere incluse» apre un elenco in sola lettura di codici/titoli/importi/date, con ritorno semplice al raggruppamento. **Non** esporre per default nome del collezionista, recapiti, note private, indirizzi o documenti economici; i dettagli resteranno nell'archivio locale.

**Definizioni comprensibili:** microtesto «Le somme riguardano le vendite registrate, non il denaro ricevuto né l'utile fiscale»; evidenza dei campi `technique/support/year` mancanti. Mostrare valori in **centesimi interi** internamente, formato locale solo a schermo. Etichette neutrali «Più frequente nel periodo» invece di «Miglior tecnica», perché il campione è selezionato e privo di inventario prodotto/esposto comparabile.

**Modifica scheda:** il pannello è strettamente di analisi; qualunque correzione di tecnica, supporto o anno passa dall'editor ordinario dell'opera, non da una modifica aggregata nella tabella. Quando la scheda cambia, i risultati sono ricalcolati o marcati chiaramente come da aggiornare.

**Accessibilità:** tabella navigabile da tastiera, intestazioni semantiche, numeri leggibili da screen reader, focus conservato tra filtro e dettaglio, zoom elevato e finestra piccola, indicazioni testuali oltre a eventuali grafici; nessuna informazione affidata unicamente a colori. Le viste devono essere utilizzabili anche senza grafici.

## 5. Dati e ipotesi di API soltanto descrittive

**Fase minima, nessuna nuova fonte di verità:** calcolo di sola lettura su vendite `sales` già registrate, collegate alle schede `artworks` mediante ID stabili, con eventuali componenti `sale_finances` soltanto per *netto commerciale* coerente con la definizione esistente. `amount_cents` e `sold_at` vengono dalla vendita; `technique/support/year` dalla **scheda corrente**, non da una testimonianza retroattiva dello stato passato.

Una possibile proiezione descrittiva: `sales_analysis({period_start, period_end, dimension}) -> {metadata_source: "current_artwork", sale_count, amount_gross_cents, missing_dimension_count, groups:[{key, label, count, gross_cents}], period, currency}`. **Non è un comando/API deciso, né autorizza uno schema.** Gruppi senza campo: chiave stabile `__missing__`, distinta da un testo utente «Altro». Nessuna somma che mescoli valute.

**Scelte da definire prima di qualunque implementazione:**
- **Categorie:** valori di `technique` e `support` sono testo libero, quindi «Olio», «olio» e «olio su tela» non vanno fusi mediante classificazione automatica senza un mapping dichiarato dall'utente. Per l'MVP mostrare etichette esatte; eventuali alias consensuali come fase separata.
- **Quantità:** conteggio di **opere con una vendita registrata** nel periodo, non numero di rate, incassi, documenti o fotografie. Nel futuro UX-017 la vista deve contare le righe per opera o la loro attribuzione esplicita, non duplicare il totale dell'accordo su ogni opera.
- **Temporalità:** `sold_at` determina il periodo della vendita. `artworks.year` è l'anno dichiarato dell'opera, talvolta non numerico/approssimato: non convertirlo in data precisa, non calcolare «tempo sul mercato» da `created_at` (data di archiviazione che può essere successiva alla realizzazione).
- **Rettifiche e resi:** se la futura UX-013 introduce annullamenti, la statistica dovrà rispettare il loro stato esplicito. Fino ad allora niente presentazione di «vendite definitive al netto dei resi» non verificabile.
- **Ambiti:** nessuna media per cliente/persona senza scelta ulteriore di privacy. Coefficienti artista e prezzi suggeriti non entrano nei totali delle vendite; l'analisi non ricostruisce quote storiche.
- **Prestazioni:** aggregazione locale paginata o tramite query, senza aprire media né caricare in memoria fotografie; la tabella deve restare reattiva su registri numerosi.

## 6. Impostazioni, offline, multilingua, backup, import/export

- **Impostazioni:** nessuna nuova configurazione obbligatoria; eventualmente ultima dimensione/intervallo selezionati nel database locale, con tasto «Ripristina filtri». Nessuna cronologia di navigazione esportata automaticamente. Le definizioni dei KPI devono restare costanti e consultabili.
- **Lingua:** lingua di sistema con override locale già previsto; **fallback inglese** per locale non riconosciuto o traduzione mancante. IT/EN/ES/FR/DE, date/numeri localizzati; tecniche/supporti scritti dall'utente non tradotti automaticamente; chiavi dei campi e CSV tecnici indipendenti dalla lingua UI.
- **Offline/privacy:** calcolo sul database SQLite locale, niente account, abbonamenti, telemetria, analytics esterna, rete, servizi IA o accesso automatico dal server MCP; valori e metadati commerciali sono riservati anche se aggregati. Evitare dettagli di compratori e microgruppi facilmente reidentificabili negli export predefiniti; avviso quando gruppo composto da una sola opera.
- **Backup e compatibilità:** in fase minima nessun nuovo dato persistente né migrazione; restano i backup SQLite+media completi, verificati, ripristinabili nella cartella scelta dall'utente. Se si aggiungessero impostazioni/alias, includerli in backup ed export completo; archivio vecchio produce dati mancanti, non record sintetici. Non cambiare il formato dei 12 CSV integrali.
- **Esportazione:** CSV UTF-8 con nome dimensione, filtro temporale, chiave gruppo, descrizione, count, gross_cents, currency, data esportazione, versione del criterio «classificazione corrente»; niente dati personali per default. Export *dettaglio* solo con anteprima dei campi e scelta esplicita, coordinato con TECH-025. Un eventuale JSON semantico versionato e reimportazione dati sorgente spettano a TECH-001/TECH-019: **un CSV aggregato non va reimportato come vendite**.
- **Import:** nessun import in fase minima; l'analisi usa esclusivamente registrazioni esistenti. Se un domani arrivassero nuovi dati via import guidato, devono passare dalle convalide ordinarie prima di comparire nei totali, senza doppioni.

## 7. Rischi, complessità, dipendenze, alternative

**Complessità:** **bassa-media** per tabella aggregata a sola lettura e filtri temporali; media per etichette libere/alias e test di performance; alta e potenzialmente fuorviante per snapshot storici, valute, resi e analisi «conversione del catalogo».

**Dipendenze:** `sales`, `artworks`, definizioni in `finance_overview`, eventuale logica esistente di tassazione/spedizione; nessun requisito di TECH-008 o UX-017 per una prima versione. Il sistema i18n, il backup e l'export sono vincoli invarianti, non nuove integrazioni cloud.

**Rischi principali:** interpretare vendite come incassi; trarre conclusioni da campioni piccoli; somme parziali da metadati mancanti; gruppi che cambiano dopo modifica delle schede; duplicazioni future con vendite multiopera; esposizione commerciale impropria. Mitigazioni: etichette esplicite, totali di quadratura, gruppo dati mancanti, indicazione «classificazione attuale», zero automatismi su dati, avvisi nei CSV e controllo di ciascun collegamento.

**Alternative da verificare:** (a) documentare come incrociare i CSV già disponibili in un foglio offline; (b) aggiungere un unico filtro nel riepilogo vendite senza nuovo pannello; (c) un semplice conteggio «opere vendute per tecnica» non monetario. Se queste rispondono al bisogno, evitare un modulo analitico dedicato.

## 8. Criteri di accettazione ipotetici, non test eseguiti

1. Con 3 vendite da 200 €, 300 € e 500 €, di cui 2 «Olio» e 1 «Acquerello», il totale lordo è 1.000 € e il conteggio è 3; la somma di **tutti i gruppi, inclusi i mancanti**, coincide con il totale delle vendite nel periodo.
2. Un'opera senza tecnica rimane nel gruppo «Non specificata»; «Altro» scritto nella scheda non viene trasformato in «Non specificata».
3. Con vendite rateali, **una vendita conta una sola volta**, indipendentemente da numero, data e somma di rate/incassi; non compare alcun «totale incassato» dedotto.
4. Un cambio della tecnica dopo la vendita aggiorna il gruppo corrente con chiaro avviso di prospettiva; non modifica `Sale`, quote percentuali congelate, documenti o certificati.
5. Cambio periodo e dimensione ricalcolano la tabella senza visualizzare righe stale. Media/percentuali vengono omesse con denominatore zero; i gruppi di una sola opera non ricevono un ranking di merito.
6. Il CSV base non contiene nomi/email/indirizzi dei collezionisti, ubicazioni, note o media; indica periodo, valuta e criterio di classificazione. Non viene importato come vendita.
7. L'archivio e i riepiloghi prima/dopo l'uso del pannello sono identici; nessun nuovo record o modifica SQLite. Offline, lingua OS/fallback inglese, tastiera/screen reader e ripristino del backup risultano coerenti su Windows/macOS/Linux.

## 9. Fasi eventuali e decisioni da chiedere al proprietario

- **Fase 0 — ricognizione con utenti:** parlare con 3–5 artisti/galleristi con archivi reali (anonimizzati negli esempi); verificare se serve questa analisi invece dei CSV, quali raggruppamenti comprendono e quale decisione prendono davvero con il risultato.
- **Fase A — minimo utile:** tabella read-only «vendite registrate per tecnica», periodo, gruppo mancanti, totali coerenti, CSV aggregato privato.
- **Fase B — soltanto se richiesta:** supporto/anno opera, dettaglio di gruppo, categorie normalizzate **confermate**, salvataggio locale dei filtri; nessun punteggio automatico.
- **Fase C — eventuale progetto separato:** storico classificazioni alla data della vendita, gestione multivaluta e resi solo dopo approvazione di rispettivi domini, con migrazioni e regole da discutere. Non deriva automaticamente da questa proposta.

**Requisiti certi:** prodotto gratuito/open source offline-first senza account obbligatorio; SQLite locale, dati esportabili e backup integrale nella cartella scelta; lingua OS/fallback inglese; registrazioni di vendita e spesa già distinte; nessuna approvazione di nuove funzioni.

**Ipotesi non dimostrate:** analisi per tecnica utile abbastanza spesso da meritare una pagina; campione di opere sufficiente per interpretare differenze; professionisti interessati a confrontare ricavi dichiarati per caratteristiche delle opere.

**Domande per il proprietario:** gli utenti che immagini vendono abbastanza opere da rendere utili i raggruppamenti? La dimensione più utile è tecnica, formato, supporto o soggetto (oggi non strutturato)? È accettabile raggruppare per *metadati attuali* con avvertenza, oppure l'analisi deve attendere un vero snapshot alla vendita? Basterebbe un CSV guidato? Vuoi impedire perfino l'export aggregato dei gruppi di una sola opera?

**Esito:** una proposta analitica autonoma, non approvata, non assegnata. Nessuna implementazione o modifica al codice è richiesta.
