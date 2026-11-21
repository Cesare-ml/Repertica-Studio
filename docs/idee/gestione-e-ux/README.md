# Idee future — Gestione e UX

**Catalogo esclusivamente ideativo**, da valutare personalmente. Le schede non autorizzano implementazioni, modifiche al prodotto o assegnazioni agli sviluppatori.

Aggiornato: **2026-11-21** (decima ricognizione). Stato delle proposte: **Proposta — da valutare**.

| ID stabile | Idea | Priorità ipotetica | Stato |
| --- | --- | --- | --- |
| [REPS-UX-001](REPS-UX-001-provenienza-movimentazioni.md) | Provenienza, ubicazioni e movimentazioni delle opere | P1 | Proposta — da valutare |
| [REPS-UX-002](REPS-UX-002-trattative-prenotazioni.md) | Trattative, proposte e prenotazioni prima della vendita | P1–P2 | Proposta — da valutare |
| [REPS-UX-003](REPS-UX-003-costo-e-margine-per-opera.md) | Scheda economica per opera e costi attribuiti | P2 | Proposta — da valutare |
| [REPS-UX-004](REPS-UX-004-rapporti-stato-conservazione.md) | Rapporti di condizione e conservazione, con confronti datati | P2 | Proposta — da valutare |
| [REPS-UX-005](REPS-UX-005-edizioni-multipli-serie-numerate.md) | Edizioni numerate, multipli e varianti con esemplari individuali | P2 | Proposta — da valutare |
| [REPS-UX-006](REPS-UX-006-incarichi-opere-su-richiesta.md) | Incarichi per opere su richiesta, brief versionati e avanzamento | P2 | Proposta — da valutare |
| [REPS-UX-007](REPS-UX-007-dossier-presentazione-opere.md) | Dossier e schede di presentazione con dati condivisi selettivamente | P1–P2 | Proposta — da valutare |
| [REPS-UX-008](REPS-UX-008-scorte-materiali-consumi.md) | Inventario operativo delle scorte e consumi dei materiali | P2 | Proposta — da valutare |
| [REPS-UX-009](REPS-UX-009-progetti-espositivi-checklist.md) | Progetti espositivi, selezioni opere e checklist con scadenze | P2 | Proposta — da valutare |
| [REPS-UX-010](REPS-UX-010-diritti-riproduzione-crediti-autorizzazioni.md) | Diritti di riproduzione, crediti fotografici e permessi d'uso dichiarati | P2 | Proposta — da valutare |
| [REPS-UX-011](REPS-UX-011-cicli-creativi-relazioni-opere.md) | Cicli creativi di opere uniche, studi preparatori e relazioni tipizzate | P2 | Proposta — da valutare |
| [REPS-UX-012](REPS-UX-012-affidamenti-conto-vendita-rendiconti.md) | Affidamenti in conto vendita e rendiconti commerciali agli intermediari | P1–P2 | Proposta — da valutare |
| [REPS-UX-013](REPS-UX-013-resi-annullamenti-rettifiche-vendite.md) | Resi, annullamenti e rettifiche post-vendita con cronologia e tutele | P1 | Proposta — da valutare |
| [REPS-UX-014](REPS-UX-014-rubrica-relazioni-professionali.md) | Rubrica soggetti professionali, ruoli multipli e storico contatti trasversale | P1–P2 | Proposta — da valutare |
| [REPS-UX-015](REPS-UX-015-ricognizioni-inventariali-discrepanze.md) | Ricognizioni inventariali e discrepanze di presenza fisica delle opere | P1–P2 | Proposta — da valutare |
| [REPS-UX-016](REPS-UX-016-valutazioni-coperture-assicurative-scadenze.md) | Valutazioni documentate e riferimenti a coperture assicurative dichiarate, con scadenze da verificare | P2 | Proposta — da valutare |
| [REPS-UX-017](REPS-UX-017-vendita-multiopera-ripartizione-importi.md) | Vendite con più opere, riepilogo unitario e ripartizione esplicita degli importi | P1–P2 | Proposta — da valutare |

## Cosa è stato verificato per evitare duplicati

- [README principale](../../../README.md), [MVP](../../MVP.md), [economia e lingue](../../ECONOMY-I18N.md): catalogazione, ricerca, anagrafica collezionisti, vendite, certificati immutabili, backup in cartelle scelte, ripristino, suggerimento prezzo con coefficiente e statistiche mensili esistono già come funzionalità o sviluppi documentati.
- [Issue #13](https://github.com/Cesare-ml/Repertica-Studio/issues/13): incassi reali, documenti di acquisto, spese ricorrenti ed export CSV finanziario sono proposte già presenti; nessuna delle sette idee le ripropone come novità.
- [Issue #5](https://github.com/Cesare-ml/Repertica-Studio/issues/5): import da Gestionale Opere, non duplicato.
- [Issue #3](https://github.com/Cesare-ml/Repertica-Studio/issues/3) e [#4](https://github.com/Cesare-ml/Repertica-Studio/issues/4): MCP locale e updater firmati appartengono al filone tecnico.
- Alla **prima** ricognizione sul branch `main` non esisteva ancora `docs/idee/`; la PR #14 ha introdotto il primo catalogo di gestione e UX. Le successive ricognizioni lavorano sulla medesima PR documentale in bozza.

## Seconda ricognizione (21 novembre 2026)

Le nuove proposte completano due lacune non coperte dalle prime schede o dalle issue aperte:

- **REPS-UX-004** documenta rilievi dello stato fisico e confronti conservativi. Si distingue da UX-001 (movimenti/provenienza) e da [TECH-003, PR #16](https://github.com/Cesare-ml/Repertica-Studio/pull/16) (integrità dei file digitali). La [PR #17](https://github.com/Cesare-ml/Repertica-Studio/pull/17) propone una galleria fotografica locale, ancora distinta dal registro dei rilievi e non assunta come presente su `main`.
- **REPS-UX-005** affronta la produzione e catalogazione professionale di edizioni numerate, copie fisiche e prove d'artista, non la sola immagine multipla di un'opera. Le regole correnti «una vendita per record opera» e «certificati immutabili» restano vincoli di progettazione.
- L'altra automazione lavora esclusivamente sotto `docs/idee/strumenti-e-architettura/` e ha già proposto portabilità semantica, automazioni locali e diagnostica media. Nessun file del suo filone è stato modificato.
- [Issue #13](https://github.com/Cesare-ml/Repertica-Studio/issues/13) comprende commissioni, incassi, documenti e cashflow: non viene duplicata in queste schede.
- Nessuna priorità proposta costituisce approvazione o incarico di implementazione.

## Terza ricognizione (21 novembre 2026)

Dopo aver riletto il branch `main` (README, MVP, ECONOMY-I18N, DATA-PORTABILITY e modelli UI) e le schede aggiornate nelle PR documentali [#14](https://github.com/Cesare-ml/Repertica-Studio/pull/14) e [#16](https://github.com/Cesare-ml/Repertica-Studio/pull/16), si approfondiscono due **processi differenti** senza riproporre funzionalità esistenti:

- **REPS-UX-006 — Incarichi per opere su richiesta:** brief e revisioni su opere **non ancora realizzate**, con fasi creative e conversione solo manuale in una scheda opera. Non è UX-002 (trattative/offerte su opere esistenti), non è UX-003 (attribuzione spese), né [issue #13](https://github.com/Cesare-ml/Repertica-Studio/issues/13) (commissioni economiche, incassi e documenti). Nessuna registrazione automatica di ricavi, acconti o certificati.
- **REPS-UX-007 — Dossier di presentazione e schede condivisibili:** documenti narrativi/visivi locali con scelta esplicita dei **soli dati divulgabili**, distinti dai certificati numerati immutabili già presenti, dai cinque CSV integrali e da TECH-001 (portabilità semantica). Non crea un sito, un canale di invio o una piattaforma marketing.
- **Filone tecnico rispettato:** TECH-001…005 riguardano portabilità, automazioni, diagnostica, ricerca federata e anteprime. Le nuove schede parlano **solo di UX e processi d'uso** e possono, in futuro e se approvate, interoperare con tali strumenti senza modificarne definizioni o file.
- **Validazione ancora necessaria:** l'effettiva domanda degli artisti che lavorano su incarico e la frequenza di condivisione dossier sono ipotesi; le priorità restano provvisorie. Tutte le sette schede sono **Proposta — da valutare**.

## Quarta ricognizione (21 novembre 2026)

Riletti `main` (README, docs/MVP.md, ECONOMY-I18N.md, DATA-PORTABILITY.md, src/app/models.ts, src-tauri/src/db.rs), [issue #13](https://github.com/Cesare-ml/Repertica-Studio/issues/13) e i cataloghi aggiornati delle PR [#14](https://github.com/Cesare-ml/Repertica-Studio/pull/14) e [#16](https://github.com/Cesare-ml/Repertica-Studio/pull/16). Due processi nuovi, non duplicati:

- **[REPS-UX-008](REPS-UX-008-scorte-materiali-consumi.md) — Scorte materiali e consumi:** il registro spese non rappresenta giacenze o prelievi fisici. Si propongono unità coerenti, registri di movimento e rettifiche, evitando di duplicare gli acquisti ricorrenti e i documenti previsti dalla issue #13, o i costi per opera di UX-003. Non si creano nuove spese automaticamente.
- **[REPS-UX-009](REPS-UX-009-progetti-espositivi-checklist.md) — Progetti espositivi:** programmazione locale di una mostra (opere candidate, checklist, scadenze), **non** un registro dei movimenti fisici (UX-001), una trattativa (UX-002) o un dossier da condividere (UX-007). Nessuna conferma della mostra trasferisce o prenota automaticamente opere.
- **Separazione dal filone tecnico:** TECH-001…006 riguardano portabilità, automazioni, integrità media, ricerca federata, cache immagini e cifratura backup; nessuna scheda né file della sua cartella è stato toccato.
- **Verifiche ancora da fare prima di qualsiasi scelta:** frequenza reale di riordino materiali e partecipazione a mostre tra utenti target, bisogno di unità multiple, eventuale interoperabilità fra progetti e storia dei movimenti. Le priorità P2 sono ipotesi, non decisioni.

## Quinta ricognizione (21 novembre 2026)

Verificati di nuovo il branch `main` (README, `docs/MVP.md`, `docs/ECONOMY-I18N.md`, `docs/DATA-PORTABILITY.md`, `docs/SALE-RECEIPTS.md`, `src/app/models.ts`, `src-tauri/src/db.rs`), la PR documentale [#14](https://github.com/Cesare-ml/Repertica-Studio/pull/14), la PR tecnica [#16](https://github.com/Cesare-ml/Repertica-Studio/pull/16) e le issue esistenti. Il modello corrente ha schema SQLite fino a v4, con galleria foto e incassi parziali registrati: nessuna delle nuove proposte ripropone queste funzioni già presenti.

- **[REPS-UX-010](REPS-UX-010-diritti-riproduzione-crediti-autorizzazioni.md) — Diritti di riproduzione e crediti:** conserva dichiarazioni e documenti sulla liceità **non verificata automaticamente** degli usi di opere e singole fotografie. Si distingue da UX-007 (impaginazione dossier), UX-009 (pianificazione mostra), UX-005 (edizioni fisiche) e dalla portabilità TECH-001. Una vendita non equivale a cessione automatica dei diritti. Questa scheda è ideativa, non consulenza legale.
- **[REPS-UX-011](REPS-UX-011-cicli-creativi-relazioni-opere.md) — Cicli creativi e relazioni tra opere:** raggruppa originali autonomi in cicli, dittici e studi preparatori, lasciando identità e stati di ogni opera indipendenti. **Non** è UX-005, che gestirebbe esemplari di una tiratura, né UX-009, che pianifica la partecipazione a eventi. Non deduce parentele artistiche da titoli o immagini.
- **Confini col filone tecnico:** le sette schede TECH-001…007 trattano strumenti trasversali (portabilità, automazioni, integrità media, ricerca federata, anteprime, backup cifrati, storage esterno). Nessun file sotto `docs/idee/strumenti-e-architettura/` è stato modificato.
- **Verifica della domanda reale ancora aperta:** frequenza delle richieste di licenza/credito da terzi e uso di cicli di originali unici. Priorità P2 ipotetiche, nessuna scelta attuata e nessuna idea assegnata agli sviluppatori.

## Sesta ricognizione (21 novembre 2026)

Riletti il branch `main` (README, MVP, ECONOMY-I18N, SALE-RECEIPTS, DATA-PORTABILITY, `src/app/models.ts`, `src-tauri/src/db.rs`, `src-tauri/src/commands.rs`), la documentazione delle undici schede precedenti, l'indice tecnico delle otto proposte nella [PR #16](https://github.com/Cesare-ml/Repertica-Studio/pull/16), la [issue #13](https://github.com/Cesare-ml/Repertica-Studio/issues/13) e la [PR #23](https://github.com/Cesare-ml/Repertica-Studio/pull/23) sugli incassi CSV, ancora aperta alla ricognizione.

- **[REPS-UX-012](REPS-UX-012-affidamenti-conto-vendita-rendiconti.md) — Affidamenti in conto vendita:** affronta *termini commerciali* tra artista/proprietario e intermediario, percentuali contrattuali ipotetiche, scadenze e rendiconti. Non crea una seconda vendita, non registra incassi né altera l'ubicazione fisica. Differisce da UX-001 (movimenti/custodia), UX-002 (trattative con acquirenti), UX-003 (costi di produzione), UX-009 (mostre) e issue #13 (contabilità generale e commissioni). La sovrapposizione sulla commissione è esplicitamente delimitata: questa proposta riguarda la **UX dell'accordo**, non il registro fiscale o di cassa.
- **[REPS-UX-013](REPS-UX-013-resi-annullamenti-rettifiche-vendite.md) — Resi e rettifiche post-vendita:** affronta la conservazione della **storia di una vendita annullata/restituita**, senza cancellare incassi dichiarati o certificati. In `main` una vendita normale è unica per opera, `delete_sale` restituisce lo stato `available` e le vendite con incassi non sono eliminabili: nessuna funzione attuale documenta una pratica completa di reso o una rivendita con storico. Distinta da UX-002 (prima della vendita), da UX-001 (movimenti fisici) e dall'export incassi della PR #23.
- **Rispetto del filone tecnico:** TECH-001…008 propongono portabilità, autorizzazioni locali, diagnostica media, ricerca federata, anteprime, cifratura backup, archivi media esterni e viste salvate. Le nuove schede descrivono **solo processi gestionali**; l'eventuale esportazione strutturata si allineerebbe concettualmente a TECH-001, senza modificarne alcun file.
- **Da convalidare con il proprietario:** frequenza del conto vendita rispetto alla vendita diretta; base e regole delle commissioni; frequenza di resi/cancellazioni; diritto di registrare una seconda vendita della medesima opera; criterio temporale dei report rettificati. P1/P2 sono solo priorità suggerite, mai decisioni di sviluppo.
- **Vincolo editoriale:** nessuna approvazione, issue operativa, modifica al software o merge. Stato uniforme: **Proposta — da valutare**.

## Settima ricognizione (21 novembre 2026)

Riletti `main` (README, `docs/MVP.md`, `docs/ECONOMY-I18N.md`, `docs/DATA-PORTABILITY.md`, `docs/SALE-RECEIPTS.md`, `src/app/models.ts`), la [issue #13](https://github.com/Cesare-ml/Repertica-Studio/issues/13), tutte le schede del filone UX-001…013 e l'indice tecnico TECH-001…009 nella [PR #16](https://github.com/Cesare-ml/Repertica-Studio/pull/16).

- **[REPS-UX-014](REPS-UX-014-rubrica-relazioni-professionali.md) — Rubrica delle relazioni professionali:** propone una scheda unica per persone e organizzazioni con ruoli multipli (curatori, fornitori, gallerie, collezionisti), cronologia manuale e follow-up **anche senza trattative o vendite**. La lacuna verificabile è che in `main` l'anagrafica `Collector` riguarda collezionisti con recapiti e note, mentre `Expense.supplier` è solo testo libero.
- **Evitati i duplicati:** UX-002 conserva esclusivamente proposte, attività e prenotazioni di una trattativa riferita a un'opera; UX-012 concerne accordi e commissioni di conto vendita; UX-001 e UX-009 riguardano ubicazioni e mostre. L'eventuale rubrica non sostituisce nessuna di queste cronologie e **non migra automaticamente** clienti, fornitori, vendite o certificati. La proposta tecnica TECH-001 sulla portabilità non viene riscritta.
- **Limiti riconosciuti:** non è ancora provato che il pubblico target abbia bisogno di una rubrica strutturata invece di note e recapiti; qualunque migrazione dell'anagrafica attuale richiederebbe validazioni, tutela degli UUID e prova di ripristino. È esplicitamente esclusa la cattura automatica di messaggi, l'invio a contatti o la profilazione.
- **Perimetro editoriale:** modificati soltanto questa scheda e l'indice, esclusivamente Markdown in `docs/idee/gestione-e-ux/`. Nessuna modifica all'altro filone, nessun codice, issue implementativa, approvazione o merge. Stato invariato: **Proposta — da valutare**.

## Ottava ricognizione (21 novembre 2026)

Riesaminati `main` (README, MVP, ECONOMY-I18N, DATA-PORTABILITY, `src/app/models.ts`), le quattordici schede del filone UX e le dieci del filone tecnico della [PR #16](https://github.com/Cesare-ml/Repertica-Studio/pull/16). Una sola nuova proposta, senza ipotesi spacciate per funzionalità presenti:

- **[REPS-UX-015](REPS-UX-015-ricognizioni-inventariali-discrepanze.md) — Ricognizioni inventariali:** permette di valutare un processo di inventario fisico per sessioni, separando opere «non ancora controllate» da opere «non rinvenute dopo controllo», e distinguendo snapshot iniziale, osservazione e catalogo aggiornato. Sul `main` attuale esistono ubicazione testuale e filtri, non un processo di verifica fisica periodica.
- **Non si duplica UX-001:** la ricognizione non è un registro di movimenti e non cambia automaticamente ubicazione o custodia. Si distingue anche da UX-004 (condizione), UX-009 (mostre), UX-008 (materiali) e dalle viste salvate TECH-008 o cronologia metadati TECH-010. Nessuna modifica ai file del filone tecnico.
- **Rischi verificabili:** un oggetto non controllato non è «smarrito»; una osservazione non prova proprietà o consegna. Snapshot/live, codici duplicati, privacy delle ubicazioni ed export selettivo richiederebbero attenzione prima di qualsiasi sviluppo.
- **Da validare:** frequenza e ampiezza delle ricognizioni reali, utilità per artisti individuali rispetto a gallerie, grado di dettaglio delle osservazioni. Priorità P1–P2 esclusivamente suggerita. Stato **Proposta — da valutare**, senza implementazione, approvazione o assegnazione.

## Nona ricognizione (21 novembre 2026)

Riletti il `main` (README, MVP, ECONOMY-I18N, DATA-PORTABILITY, `src-tauri/src/models.rs` e `finance.rs`), la [issue #13](https://github.com/Cesare-ml/Repertica-Studio/issues/13), le quindici schede UX-001…015 e le undici proposte TECH-001…011 dell'[altra PR documentale #16](https://github.com/Cesare-ml/Repertica-Studio/pull/16). È stata approfondita **una sola esigenza professionale**, da convalidare prima di qualsiasi sviluppo:

- **[REPS-UX-016](REPS-UX-016-valutazioni-coperture-assicurative-scadenze.md) — Valutazioni documentate e coperture assicurative dichiarate:** registrare fonti e date dei valori attribuiti alle opere, riferimenti a polizze individuali o collettive, intervalli annotati e distinte selettive per mostre/prestiti. Il `main` possiede coefficiente artista e prezzo suggerito, ma **non** un registro assicurativo o storico peritale; un prezzo suggerito o di vendita non è valore assicurato.
- **Confini espliciti:** UX-001 documenta movimenti/provenienza, UX-004 la condizione fisica, UX-009 pianifica mostre, UX-012 gestisce conto vendita, UX-003 e issue #13 trattano costi/finanza; TECH-001 studia portabilità aperta e TECH-006 backup cifrati. La nuova scheda non duplica questi processi, non crea premi o spese e non accerta né promette che un'opera sia coperta.
- **Rischi da sottoporre al proprietario:** falso senso di sicurezza, confusione tra massimale condiviso e valore della singola opera, esposizione di indirizzi e numeri di polizza, documenti personali in backup attualmente **in chiaro**, clausole non interpretabili automaticamente. La condivisione ha anteprima e redazione dei campi riservati, sempre offline e senza contattare compagnie.
- **Fattibilità e utilità ancora da verificare:** interviste professionali, uso effettivo delle polizze collettive, priorità per studi con poche opere, corretto confine con l'eventuale gestione di mostre. P2 è un'indicazione, non autorizza roadmap. Nessun file di `docs/idee/strumenti-e-architettura/` è stato modificato; nessuna PR implementativa o merge.

## Criteri editoriali

1. Ogni idea deve aggiungere un **processo professionale distinto**, non rinominare una funzione già presente. Chi propone cita la differenza rispetto all'MVP e alle issue.
2. Ogni scheda descrive problema, utenti, scenari, flussi, dati, offline, lingue, privacy, export/import, complessità, dipendenze, rischi, alternative, criteri e fasi; separa **certezze**, **ipotesi** e **decisioni da richiedere al proprietario**.
3. Non affermare autenticità della provenienza, saldo bancario reale, incasso o profitto fiscale senza prove; non trasformare un'ipotesi in una promessa del software.
4. Tutto resta **gratuito, open source, offline-first**, senza account, abbonamenti, sincronizzazione cloud o invio dei dati a servizi terzi; lingua del sistema con fallback inglese.
5. Prima di creare una nuova scheda verificare altre schede, issue e PR aperte, comprese quelle di `docs/idee/strumenti-e-architettura/` se aggiunte dall'altro filone. Non modificare i file dell'altro filone. Se un'idea interseca altre schede, linkarle e definire confini.
6. Ogni ulteriore proposta deve avere ID successivo `REPS-UX-017`, ecc., mantenendo gli ID già assegnati stabili; gli sviluppatori non devono implementarla senza revisione esplicita del proprietario.
7. Lavorare solo su file Markdown di questa cartella e tramite PR **documentale**; non effettuare merge automatici.

## Sequenza di valutazione suggerita (non autorizzazione)

Discutere per prime le esigenze davvero ricorrenti: **movimenti e custodia** (001), **trattative e prenotazioni** (002), **costi analitici** (003). Valutare separatamente l'interesse per **rilievi conservativi** (004), **edizioni e multipli** (005) e **incarichi su richiesta** (006), processi potenzialmente specialistici. Esaminare **dossier di presentazione** (007) quando è chiaro quali dati gli utenti desiderano condividere e quali debbano rimanere privati. Infine valutare **scorte e consumi materiali** (008) soltanto se acquisti e giacenze ricorrenti sono una necessità reale, e **progetti espositivi** (009) soltanto se l'organizzazione di mostre è abbastanza frequente da giustificare un modulo dedicato. Considerare **diritti e crediti di riproduzione** (010) solo dopo una verifica dei casi di pubblicazione più comuni, e **cicli creativi/relazioni fra opere uniche** (011) solo se i semplici filtri testuali sono insufficienti. Infine esaminare **conto vendita/intermediari** (012) se una quota significativa degli utenti opera tramite gallerie; la **tracciabilità di resi e annullamenti** (013) merita valutazione anticipata se si prevedono vendite reali con acconti, dato il rischio di perdere la storia delle rettifiche. La **rubrica trasversale dei contatti professionali** (014) è da valutare solo se i casi reali di curatori, fornitori e gallerie dimostrano i limiti dell'anagrafica collezionisti attuale; evitare un CRM pesante senza evidenze. La **ricognizione inventariale** (015) dovrebbe prima essere provata con professionisti che controllano materialmente opere, per accertare se semplici filtri e note siano insufficienti. La **gestione documentale di valori e coperture** (016) va esaminata soltanto se prestiti, trasporti e richieste assicurative risultano frequenti, evitando di trasformare il gestionale in un comparatore di polizze. La sequenza e le priorità sono opinioni progettuali, **non un piano approvato**.


## Decima ricognizione — 21 novembre 2026

Confrontati nuovamente `main` ([README](../../../README.md), [MVP](../../MVP.md), [economia e lingue](../../ECONOMY-I18N.md), [incassi](../../SALE-RECEIPTS.md), `src/app/models.ts`, `src-tauri/src/db.rs` e `src-tauri/src/commands.rs`), le sedici schede gestionali e l'[indice delle dodici proposte tecniche nella PR #16](https://github.com/Cesare-ml/Repertica-Studio/pull/16). È stata approfondita **una** lacuna professionale già menzionata in `docs/MVP.md` come possibile evoluzione, senza considerarla approvata.

- **[REPS-UX-017](REPS-UX-017-vendita-multiopera-ripartizione-importi.md) — Vendite con più opere:** un accordo commerciale può comprendere più opere e un prezzo complessivo. La proposta esamina registrazione atomica, sconti ripartiti con arrotondamenti esatti, riepilogo per cliente, saldi degli incassi manualmente dichiarati e protezione delle quote finanziarie storiche. Il totale di gruppo non deve mai aggiungersi di nuovo ai ricavi delle singole vendite.
- **Evitati i doppioni:** UX-002 riguarda trattative precedenti alla vendita; UX-003 attribuisce costi; UX-012 documenta conto vendita; UX-013 rimane il riferimento unico per eventuali resi/storni; [issue #13](https://github.com/Cesare-ml/Repertica-Studio/issues/13) riguarda i registri finanziari. TECH-001 (portabilità) e TECH-008 (filtri) sono soltanto punti di interoperabilità potenziali, non documenti modificati. Il riepilogo non costituisce una fattura o una prova di incasso.
- **Da validare:** frequenza reale delle vendite multiopera, bisogno di sconti collettivi, trattamento di arrotondamenti e incassi comuni, eventuali resi parziali. La vendita singola deve restare il percorso più semplice. Priorità P1–P2 **solo ipotetica**.
- **Perimetro:** aggiunti soltanto questa scheda e l'indice Markdown sotto `docs/idee/gestione-e-ux/`. Nessun codice, test, workflow, configurazione, issue di implementazione o documento dell'altro filone modificato. La [PR #14](https://github.com/Cesare-ml/Repertica-Studio/pull/14) resta in bozza, senza merge. **Stato: Proposta — da valutare**.
