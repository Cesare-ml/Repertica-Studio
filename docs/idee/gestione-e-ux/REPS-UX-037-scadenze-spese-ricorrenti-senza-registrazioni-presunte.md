# REPS-UX-037 — Scadenze delle spese ricorrenti, senza registrazioni o pagamenti presunti

- **ID stabile:** REPS-UX-037
- **Data:** 2026-11-22
- **Stato:** **Proposta — da valutare**
- **Filone:** Gestione e UX; analisi esclusivamente documentale
- **Priorità proposta:** **P2 condizionata**, P3 se è sufficiente un semplice elenco esterno; mai approvata
- **Decisione:** nessuna implementazione, assegnazione, modifica dei registri o automazione autorizzata

## 1. Problema professionale, destinatari ed evidenze

Un artista che paga ogni mese l'affitto dello studio, ogni trimestre un servizio di deposito o una volta l'anno l'assicurazione deve ricordare **impegni futuri** prima che diventino **spese registrate**. La dashboard economica attuale mostra il consuntivo, ma la mera presenza di una spesa del mese precedente non è una promessa né una scadenza futura. Se manca una registrazione, l'utente non può sapere dalla sola statistica se il costo non esiste, non è ancora stato sostenuto oppure è stato dimenticato.

**Riscontro puntuale su `main` (22/11/2026):** [`finance.rs`](../../../src-tauri/src/finance.rs) definisce `Expense` e `ExpenseInput` con `expense_date`, categoria, descrizione, importo in centesimi, fondo facoltativo, fornitore, link e note; `save_expense` crea o modifica un movimento, `list_expenses` ordina per data. Il calcolo `finance_overview_core` somma `expenses.amount_cents` **per data spesa**, non per obbligazioni future. [`ECONOMY-I18N.md`](../../ECONOMY-I18N.md) specifica che le statistiche non equivalgono a movimenti bancari verificati; [`DATA-PORTABILITY.md`](../../DATA-PORTABILITY.md) documenta dodici CSV, compreso `expenses`. Nei modelli, comandi e documenti letti non emerge un registro delle *scadenze ricorrenti delle spese*. **Non è un bug dimostrato, né una richiesta utente già convalidata.**

**Destinatari da verificare:** artisti autonomi con spese fisse di studio, gallerie piccole, laboratori con canoni/assicurazioni, gestori di archivi che devono preparare le uscite nei prossimi mesi. Gli artisti con spese quasi esclusivamente occasionali potrebbero non trarne beneficio.

**Confine irrinunciabile:** una scadenza prevista **non è** una spesa registrata, un pagamento, una passività fiscale certificata o denaro già uscito. L'eventuale funzionalità deve tenere separati impegni pianificati e registro consuntivo autorevole.

## 2. Distinzione dalle proposte già esistenti (anti-doppione)

- [UX-032 — Budget mensili](REPS-UX-032-budget-mensili-categorie-spesa-scostamenti.md): il budget definisce **quanto l'utente desidera spendere per categoria e mese**, non una specifica obbligazione verso un fornitore con data e cadenza. Un affitto previsto può essere confrontato con un budget, ma **non genera** una spesa né un budget.
- [UX-024 — Scadenziario incassi](REPS-UX-024-scadenziario-incassi-piani-pagamento-vendite.md): segue somme **da ricevere per una vendita**; qui si valutano uscite **dello studio**, senza toccare rate, saldi e vendite.
- [UX-003 — Costo e margine per opera](REPS-UX-003-costo-e-margine-per-opera.md) e [UX-008 — Scorte materiali](REPS-UX-008-scorte-materiali-consumi.md): attribuiscono consumi/costi alle opere e quantità fisiche, non canoni ripetuti a calendario.
- [UX-028 — Giustificativi delle spese](REPS-UX-028-giustificativi-locali-spese-acquisti.md): conserva documenti relativi a spese **già annotate**; una scadenza non equivale a fattura ricevuta. [UX-030 — Quadratura](REPS-UX-030-quadratura-facoltativa-estratti-conto-locali.md) riguarda collegamenti manuali a estratti conto e non deve segnare automaticamente una scadenza come pagata.
- [TECH-002 — Automazioni locali](https://github.com/Cesare-ml/Repertica-Studio/blob/docs/idee-strumenti-architettura-20261121/docs/idee/strumenti-e-architettura/REPS-TECH-002-automazioni-locali-consapevoli.md) propone un **motore generico** di regole/permessi; **questa scheda riguarda soltanto semantica e UX degli impegni di spesa**, realizzabile persino senza scheduler, processi o notifiche. [TECH-001/018/025](https://github.com/Cesare-ml/Repertica-Studio/pull/16) riguardano la portabilità e robustezza degli export, non il significato di una scadenza.
- **Coordinamento:** nessun file del filone tecnico deve essere modificato; non introdurre un secondo task engine, un budget parallelo, nuovi incassi o una contabilità generale.

## 3. Casi d'uso concreti e casi-limite

1. **Affitto mensile dello studio:** canone previsto di 350,00 €, scadenza il 5 di ogni mese. L'utente apre «Prossime scadenze», vede il 5 dicembre come *previsto*, poi registra manualmente una spesa di 350,00 € datata 6 dicembre e può collegarla all'occorrenza del 5. Il registro spese contiene **una sola riga**.
2. **Assicurazione annuale:** 300,00 € a marzo, importo indicativo soggetto a rinnovo. In aprile l'utente non deve visualizzare undici falsi mesi di spesa; in marzo compare un promemoria dichiarato. Può aggiornare solo l'anno seguente senza riscrivere quello trascorso.
3. **Costo variabile:** un servizio ricorrente indica «importo da confermare», senza `0` inventato; la scadenza resta consultabile ma **non entra in una somma monetaria prevista come importo esatto**.
4. **Addebito non avvenuto:** l'utente salta una scadenza o sospende l'abbonamento. L'occorrenza viene annotata *saltata* con motivo facoltativo, non diventa spesa e non modifica i mesi precedenti.
5. **Modifica del contratto:** il canone passa da 350,00 € a 370,00 € dal 1° gennaio. Le scadenze storiche già confermate/collegate mantengono il loro importo dichiarato. Nessuna modifica retroattiva silenziosa.
6. **Giorno 31 e febbraio:** «ultimo giorno del mese» è diverso da «giorno 31 del mese». Se una data non esiste, scegliere e **mostrare una regola esplicita**, senza spostamenti impliciti che facciano scattare una registrazione.
7. **Archivio fermo per due mesi:** al successivo avvio l'app può mostrare in un'unica vista le scadenze trascorse, **non** creare una raffica di spese o notifiche e non dedurre che siano insolute in banca.
8. **Due costi uguali:** due fornitori possono richiedere entrambi 50,00 € nello stesso giorno; importo/data coincidenti non autorizzano un collegamento automatico.

## 4. Flussi UX proposti, dal minimo alla versione facoltativa

**Ingresso facoltativo:** «Economia → Spese → Scadenze previste», separato visivamente e verbalmente da «Spese registrate». Installazione nuova: elenco vuoto con spiegazione «Qui puoi annotare impegni futuri; non vengono creati movimenti economici». Nessun canone preimpostato o dedotto dalle spese già presenti.

**Nuova ricorrenza:** «Aggiungi scadenza» → descrizione (es. «Affitto studio»), categoria testuale, fornitore facoltativo, importo *stimato e modificabile* oppure «da confermare», prima data di scadenza, frequenza iniziale limitata a mensile / trimestrale / annuale, regola del giorno, data fine/sospensione facoltativa, fondo facoltativo coerente con le parti correnti → **anteprima delle prossime tre date** → «Salva promemoria». Impostazione non richiesta agli utenti che usano soltanto il registro spese.

**Consultazione:** due sezioni «Da verificare» e «Già esaminate» con data prevista, nome, importo e stato; filtri per prossimo mese, arretrate, categoria e fornitore. Se l'importo manca, mostrare «Da confermare» e includerlo nel **numero** delle scadenze, non in un totale fittizio. Per budget personale eventuale tabella `stimato` in un pannello distinto dal `registrato`, con avvertenza «previsioni incomplete» quando esistono importi sconosciuti.

**Da scadenza a spesa:** pulsante «Registra questa spesa» apre il **form spesa esistente** precompilato, ma richiede data effettiva, importo, fondo e conferma. Solo il salvataggio riuscito genera `Expense` e un collegamento esplicito all'occorrenza. Per una spesa già registrata, «Collega a spesa esistente» propone candidati e chiede una scelta **manuale**, mai matching automatico per importo o data. «Rimanda», «Segna come non sostenuta» o «Non so ancora» non creano movimenti. Il semplice controllo di una casella non equivale a «pagata».

**Modifiche:** «Modifica le prossime scadenze» mostra data di entrata in vigore e un riepilogo delle date/importi interessati; le occorrenze storiche decise e i riferimenti a `expense_id` restano stabili. «Termina promemoria» non cancella spese né documenti; eliminazione della serie richiede un avviso sui riferimenti storici (preferire archiviazione alla distruzione).

**Accessibilità:** struttura navigabile da tastiera, focus visibile nei dialoghi e ritorno al punto d'origine, date leggibili dai lettori schermo, stato comunicato con testo e non solo colori/icona campanello, zoom 200%, errori associati ai campi; nel calendario deve esistere anche una vista elenco. Preferenze lingua coerenti con tutta l'app.

## 5. Semantica e dati / API soltanto descrittivi, non prescrittivi

**Concetti separati:**
- *Serie di scadenze*: regola proposta dall'utente, non un contratto validato.
- *Occorrenza*: singola data prevista, con stato esaminato o ancora da verificare.
- *Spesa registrata*: record `expenses` già autorevole per gli aggregati, mai generato alla sola maturazione della data.
- *Pagamento bancario*: fatto esterno che Repertica non conosce; anche un'occorrenza collegata a `expenses` significa «spesa registrata», **non** «pagamento provato».

**Ipotesi struttura:** `expense_recurrence` (`id` UUID, `title`, `category_label`, `supplier?`, `estimated_amount_cents?`, `currency='EUR'`, `fund_id?`, `start_due_date`, `frequency`, `day_rule`, `end_date?`, `paused`, `notes?`, `created_at`, `revision`). Eventuali modifiche a partire da data futura richiedono versionamento della regola o snapshot delle occorrenze, non riscrittura del passato.

**Occorrenza:** `occurrence_id` stabile, `recurrence_id`, `period_key` e data prevista, `amount_snapshot_cents?`, stato (`TO_REVIEW` / `LINKED_EXPENSE` / `SKIPPED` / `POSTPONED` / `CANCELLED`), `linked_expense_id?`, nota e data della decisione. Stati che implicano pagamento **non previsti**. Unicità logica di serie e periodo; il doppio avvio non duplica occorrenze. È possibile anche calcolare scadenze virtualmente e persistere solo le decisioni: **alternativa più leggera da studiare**.

**API concettuali, non comandi da implementare:** «anteprima prossime occorrenze», «leggi scadenze in intervallo», «proponi inserimento spesa», «collega manualmente un `expense_id`», «annota saltata/rinviata», «archivia regola». L'inserimento di una spesa e il collegamento, se accettati in futuro, devono essere atomici o recuperabili dopo errore, senza spesa orfana o doppio movimento. Dopo eliminazione/modifica della spesa, il collegamento deve risultare mancante/da verificare, **mai ricreare automaticamente una spesa**.

**Regole numeriche:** importi sempre in **centesimi interi**, non `float`; distinguere `null` da zero e da importo stimato. Importi positivi; nessuna valuta dedotta da simboli localizzati. Totali previsti e spese registrate sono due aggregati differenti, con periodi, basi e completezza dichiarati. Non riscrivere `finance_overview`, quote congelate, vendite, incassi, certificati o coefficiente artista.

## 6. Offline, impostazioni, multilingua, privacy, interoperabilità

- **Offline-first e autonomia:** SQLite locale e nessun account, abbonamento, cron cloud, provider bancario, servizio calendari o accesso alla rete. L'app non deve eseguire operazioni al di fuori dell'interfaccia senza consenso. La vista delle scadenze funziona con l'app aperta e usa l'orologio locale **solo per etichette «oggi/in ritardo»**, mai per creare spese o interpretare pagamenti.
- **Impostazioni proposte:** mostra/nascondi vista «Scadenze», orizzonte di consultazione (es. 30/90 giorni), eventuale avviso *solo all'apertura dell'app* e **disattivato per default**; prima fase preferibile **senza** notifiche OS né motore genericamente programmabile. Nessuna integrazione MCP o email automatica; eventuali reminder esterni necessitano di valutazione separata.
- **Lingue:** lingua OS e selezione manuale IT/EN/ES/FR/DE con **fallback inglese**. Le etichette di frequenza, gli stati, le validazioni e i testi d'aiuto sono tradotti; nomi fornitori, descrizioni, note e categorie **non** sono tradotti automaticamente. Salvare codici stabili e date ISO `YYYY-MM-DD`, mostrare le date secondo locale senza modificare la scadenza civile.
- **Giorni/fuso orario:** per date senza orario evitare conversioni UTC che possano spostare il giorno al cambio di fuso o DST. Per giorno 29/30/31 ed eventi a cadenza non disponibile occorre regola esplicita («ultimo giorno», «salta», o «giorno n se presente») scelta con anteprima; non assumere che +30 giorni equivalga a +1 mese. Il cambio manuale dell'orologio non cambia le registrazioni reali.
- **Privacy e sicurezza:** canoni, fornitori, date, eventuali note possono rivelare dati sensibili della vita professionale. Nessuna esposizione nelle brochure, nei certificati, negli export per terzi o nel server MCP locale per impostazione predefinita. I backup attuali sono **in chiaro** e possono contenere tali dettagli; presentare la cautela. Nessuna telemetria, contatto a servizi terzi o scadenza condivisa automaticamente.
- **Backup/ripristino:** le definizioni e le decisioni sulle occorrenze, se approvate, dovrebbero far parte del **backup completo verificato** in cartella liberamente selezionata (USB, NAS montato, cartella condivisa a scelta); dopo ripristino gli ID e i collegamenti a spese conservate restano verificabili. Nessun promemoria deve produrre una spesa «di recupero» al primo avvio dopo un restore.
- **Esportazione completa:** nuova sezione del formato aperto versionato o registro CSV dedicato con serie, regole, date, importi previsti, stati e riferimenti UUID a spese: **non** sostituire o reinterpretare il CSV `expenses` esistente. L'export integrale include tutte le scadenze e loro decisioni; il CSV `expenses` continua a includere **solo spese realmente registrate**. Esportazioni selettive per terzi richiedono anteprima e minimizzazione di note/fornitori.
- **Import futuro:** file locale scelto manualmente, preview delle ricorrenze, mappatura categorie/fondi, validazione delle date e codici, conflitti UUID e sovrapposizioni, transazione idempotente. Un file di «scadenze» importato **non** crea spese e non traduce automaticamente categorie o valuta. Non considerare CSV semplice come backup.
- **Compatibilità:** migrazione additiva e tollerante a archivi antecedenti, niente rigenerazione dello storico; vecchie versioni che non supportano uno schema nuovo possono non aprirlo, quindi usare la politica di compatibilità e backup prima delle migrazioni già prevista. Windows/macOS/Linux con stessi risultati su date civili, importi e traduzioni.

## 7. Dipendenze, alternative, complessità e rischi

**Dipendenze reali da valutare:** registro `expenses`, parti che pagano spese, calendario/date locali, SQLite, UI accessibile, i18n, backup/CSV. Nessun motore di automazione, notifiche OS, service worker o cloud richiesto. La possibile UX-032 sui budget sarebbe **indipendente** e non è una condizione necessaria.

**Alternativa 0 — non sviluppare:** usare calendario personale o un foglio Calc offline con ricorrenze, insieme al CSV delle spese. È la scelta giusta se le interviste dimostrano che le scadenze dello studio sono pochissime o gli utenti non registrano puntualmente le spese.

**Alternativa minima:** una tabella semplice di «prossime uscite» con data, descrizione, importo indicativo e flag «esaminato», **senza ripetizione automatica**. Se risolve il bisogno, non introdurre un modello ricorrente. Alternativa intermedia: modelli che precompilano il form spese solo su click, senza occorrenze persistite o promemoria.

**Complessità:** **bassa** per tabella di impegni manuali; **media** per ricorrenze mensili/trimestrali/annuali con date civili, stati e link manuali; **medio-alta** per revisioni, import/export round-trip, eccezioni e storicizzazione. Non progettare rate bancarie, fiscalità, fatture, addebiti o notifiche quando l'app è chiusa.

**Rischi e mitigazioni principali:** doppio conteggio previsto/consuntivo (schermate e CSV distinti); falsa pretesa di pagamento (stati «collegata a spesa», non «pagata»); scadenze errate al 31 febbraio o cambi fuso (regola esplicita e test calendario); template modificati che riscrivono storico (revisioni/snapshot); duplicazione form spese (precompilazione sul form già esistente); esposizione di fornitori nei CSV (preview e separazione export completo/selettivo); avvisi eccessivi dopo lunghe assenze (vista unica, nessun catch-up automatico); errore di importazione che crea molte spese (import serie soltanto, conferma e rollback). Nessun algoritmo deve inventare obbligazioni o considerare una previsione «pagata».

## 8. Criteri di accettazione verificabili (solo futuri)

1. App appena installata: nessuna ricorrenza o spesa automatica; il normale registro spese funziona come prima senza configurazione.
2. Affitto di 350,00 € ogni 5 del mese: per tre mesi sono visibili tre date previste, **zero** nuovi record `expenses` finché l'utente non salva una spesa.
3. «Registra questa spesa» apre il form esistente; conferma con data 6 anziché 5 crea **una** spesa e un link stabile, senza alterare altri movimenti.
4. Doppio click, riavvio, ripristino e cambio lingua non generano doppie spese, doppie occorrenze persistenti o duplicazioni dei costi mensili.
5. Sospendere una serie, annotare un mese saltato e modificare il canone per le date future non cambia scadenze storiche già esaminate né spese collegate.
6. Importo ignoto rimane `null`: la UI lo conta fra le scadenze ma non lo somma come 0 € o cifra certa; somma prevista esplicitamente parziale.
7. Giorno 31, 29 febbraio, anno bisestile, cambio fuso/DST e app chiusa per oltre un mese generano anteprime comprensibili e **mai** registrazioni retroattive.
8. L'utente può collegare manualmente una spesa già presente; il sistema rifiuta collegamenti duplicati o ambigui senza sua scelta.
9. Il totale delle spese e `margin_cents` per mese restano identici prima e dopo creazione/archiviazione di un promemoria, **a parità di record `expenses`**.
10. Un backup completo con cartella scelta e successivo ripristino preservano regole, stati, UUID e legami senza servizi online.
11. CSV attuale `expenses` conserva le sole spese effettive; eventuale CSV delle scadenze è separato e un export integrale documentato non perde importi sconosciuti e revisioni.
12. UI e messaggi navigabili da tastiera e screen reader, zoom 200%, lingua di sistema, override IT/EN/ES/FR/DE, fallback inglese; date civili e importi persistiti restano invariati.

## 9. Fasi, certezza / ipotesi / decisioni per il proprietario

**Fase 0 — riscontro utenti, nessuno sviluppo:** chiedere ad artisti/gallerie, con esempi anonimizzati, quanti impegni ripetuti devono ricordare, se oggi gestiscono già il problema con il calendario e se desiderano **promemoria** o solo **precompilazione delle spese**. Misurare il bisogno prima di progettare nuove tabelle.

**Fase A — minimo, solo se approvato:** elenco manuale di uscite future; nessuna ricorrenza automatica né scrittura nel registro contabile. Test di lessico e differenze fra previsto/registrato.

**Fase B — opzionale e separata:** serie periodiche con anteprima calendario e «Registra spesa» confermata, stati dei singoli periodi e link manuale a spese già registrate; nessun servizio in background.

**Fase C — soltanto con necessità dimostrata:** revisioni dei canoni, import/export versionato, avvisi non invasivi con app aperta, integrazione *solo visiva* con eventuali budget UX-032, senza duplicare il motore TECH-002.

**Requisiti certi dal progetto e dal `main`:** software open source gratuito, Tauri/Angular/Rust/SQLite locale; nessun account/abbonamento; spese in centesimi interi registrate con `expense_date`; riepiloghi mensili basati sulle spese registrate, non su pagamenti bancari; esportazioni CSV e backup/ripristino a cartella scelta, in chiaro; lingua di sistema con fallback inglese; integrità degli storici e delle percentuali di ripartizione delle vendite.

**Ipotesi non verificate:** gli utenti gestiscono abbastanza spese ricorrenti da giustificare una funzione; una vista di scadenze migliora la puntualità; tre frequenze sono sufficienti; i collegamenti a spese registrate risultano utili; un calendario interno sarebbe preferito a quello personale.

**Domande riservate al proprietario:** il target comprende soprattutto artisti con studio fisico o anche gallerie con molti contratti? È sufficiente un **elenco di promemoria** senza ricorrenza? Gli impegni devono essere visibili senza notifiche, oppure è desiderato un avviso all'apertura? È accettabile distinguere «esaminato», «saltato» e «collegato a spesa» senza mai dire «pagato»? Servono importi stimati variabili e scadenze annuali o sarebbe eccessivo? L'utente preferisce gestire i canoni nel calendario del sistema e mantenere Repertica solo come registro di spese?

**Esito:** proposta documentale da valutare personalmente. Nessuna idea approvata, attività di sviluppo assegnata, implementazione o modifica operativa.
