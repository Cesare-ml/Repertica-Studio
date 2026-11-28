# REPS-UX-040 — Rimborsi e rettifiche di spese e acquisti, con storia preservata

- **ID stabile:** REPS-UX-040
- **Data della ricognizione:** 2026-11-22
- **Stato:** Proposta — da valutare
- **Filone:** Gestione e UX
- **Priorità ipotetica:** P2; P1–P2 solo se emerge frequenza elevata di resi di materiali e acquisti
- **Natura:** esclusivamente ideativa e documentale. Nessuna approvazione, assegnazione o autorizzazione allo sviluppo.

## 1. Motivo, utenti e riscontri verificabili

**Problema.** Un artista acquista colori, tele o cornici, registra una spesa e in seguito restituisce parte della merce, riceve un rimborso parziale, una nota di credito, uno storno o un buono acquisto. Se cancella o modifica la spesa originaria, il registro può perdere il valore storico dell'acquisto; se la lascia immutata, l'analisi gestionale dei costi sostenuti rimane sovrastimata. Serve un modo prudente di **collegare gli eventi successivi alla spesa sorgente**, mantenendo separati l'accordo col fornitore, la restituzione fisica e il denaro eventualmente ricevuto.

**Lettura statica di `main` il 22/11/2026**, senza riprodurre un caso reale: [`src-tauri/src/finance.rs`](../../../src-tauri/src/finance.rs) definisce `Expense` con UUID, data, categoria, descrizione, importo positivo in centesimi, fondo opzionale, fornitore, URL, note e timestamp. `save_expense` aggiorna la stessa riga oppure ne crea una; `delete_expense` elimina la riga. Non emerge un registro dedicato a rimborsi/rettifiche **collegati a una spesa**. [`src-tauri/src/db.rs`](../../../src-tauri/src/db.rs) vincola l'importo positivo; [`docs/ECONOMY-I18N.md`](../../ECONOMY-I18N.md) chiarisce che i totali delle spese sono valori registrati, **non** pagamenti bancari verificati. [`docs/DATA-PORTABILITY.md`](../../DATA-PORTABILITY.md) elenca un CSV delle spese senza registro di rimborsi associati.

**Destinatari:** artisti che comprano materiali frequentemente; studi con acquisti per più commesse; piccole gallerie che sostengono spese di cornici, trasporto e allestimento; archivisti con resi o rimborsi su servizi di conservazione. Per attività con acquisti sporadici, il costo di nuovi campi e stati può non essere giustificato.

**Requisiti certi del prodotto:** desktop gratuito/open source, Tauri, offline-first e senza account o abbonamento, archivio SQLite locale, backup/ripristino verificati in cartella selezionabile liberamente, esportabilità completa, lingua OS con fallback inglese. **Non è accertato** quante persone richiedano un flusso di rimborsi: bisogno, priorità e modello sono ipotesi.

## 2. Quattro casi d'uso distinti

1. **Reso parziale con restituzione dichiarata:** un artista registra l'acquisto di materiali per **120,00 €** in luglio. In agosto restituisce una confezione da **30,00 €**. La scheda originale resta di 120,00 €; un evento collegato dichiara «rimborso di 30,00 € ricevuto il 10 agosto», con fonte manuale e data. La scheda mostra **costo rettificato informativo: 90,00 €**. Luglio resta il mese dell'acquisto; agosto è il mese del rimborso dichiarato.
2. **Nota di credito promessa, non ancora rimborsata:** il fornitore comunica una rettifica di 18,00 €, ma non è avvenuto accredito. Lo stato «concordato / da ricevere» **non** genera un movimento di cassa e non deve essere sommato come rimborso ricevuto.
3. **Buono per acquisti futuri:** il negozio emette un voucher di 25,00 € invece di restituire denaro. Va rappresentato come **credito commerciale non monetario** (sezione separata o nota) e non come pagamento bancario; il suo eventuale utilizzo richiede una scelta successiva, senza generare in automatico una nuova spesa negativa.
4. **Errore di trascrizione:** la spesa originale di 47,80 € è stata digitata 478,00 €. Questa è una **correzione dell'errore**, non un rimborso reale. Il percorso deve spiegare quando usare la normale modifica e quando invece registrare un evento successivo. Non creare rimborsi fittizi per far tornare i conti.

**Casi limite da esplicitare:** più rimborsi parziali sullo stesso acquisto; promessa annullata; rimborso registrato nel nuovo anno; fornitore che rimborsa spese di spedizione e non materiali; spesa collegata a un fondo successivamente disattivato; eventuali spese in valuta diversa (UX-027, non ancora realizzata); contestazione senza rimborso.

## 3. Flusso UX proposto, minimale e reversibile

**Percorso A — prima soluzione da convalidare:** nella scheda **Economia → Spese → Dettaglio**, mostrare un riepilogo «Acquisto originale», «Variazioni dichiarate» e «Costo rettificato» senza modificare la riga storica. Il comando facoltativo **Registra rimborso** apre un modulo breve con importo, data, natura dell'evento, stato dichiarato, riferimento al fornitore e nota; il documento di riscontro sarebbe facoltativo e distinto dai giustificativi della spesa.

La conferma presenta una frase non ambigua: **«Registri una dichiarazione manuale. Non è una verifica bancaria, una nota di credito fiscale generata dal programma o un pagamento effettuato da Repertica.»** Per «da ricevere» il riepilogo separa valore concordato da valore ricevuto; per «buono acquisto» esclude il valore dalle somme monetarie.

**Consultazione:** elenco compatto sotto la spesa, ordinabile per data, con etichetta testuale di stato e riepilogo per fondo. Filtri ipotetici «con rimborsi», «da ricevere» e «da verificare». Nessun wizard obbligatorio per tutte le spese: le registrazioni attuali non devono cambiare aspetto né significato per chi non usa questa funzione.

**Correzioni di un evento:** evitare cancellazione distruttiva di una rettifica già usata nei riepiloghi. Preferire una correzione motivata oppure un evento di annullamento che conserva riferimento, data e precedente valore; una cancellazione semplice può restare ammessa per bozze non contabilizzate, previa scelta esplicita del proprietario.

**Accessibilità e qualità:** focus sul campo invalido; conferme navigabili integralmente da tastiera; testi espliciti invece di distinguere rimborsi e note di credito soltanto con colori o segni «+ / −»; importi tabulari, lettura con screen reader, zoom e finestre piccole, messaggi localizzati.

## 4. Dati ipotetici, senza definire schema o API operative

Una possibile entità **`expense_adjustment`** (solo idea, non implementazione) potrebbe contenere:

- UUID stabile, `expense_id` (FK alla spesa originale), tipo dichiarato (`refund`, `credit_note_pending`, `store_credit`, `correction_cancel` o vocabolario più semplice da valutare), stato e data effettiva/dichiarata;
- importo positivo in centesimi nella **stessa valuta della spesa**; per l'attuale MVP importi EUR; `recorded_at`, motivo/note, riferimento del fornitore; fonte documentale locale facoltativa e riferimenti a rettifica/annullamento;
- collegamento al **fondo storico** della spesa originaria come riferimento, senza riscrivere ripartizioni storiche delle vendite; eventuale data di accredito distinta dalla data di nota di credito.

**Invarianti proposti:** importi positivi e non superiori complessivamente all'acquisto originale **per i rimborsi dichiarati effettivamente ricevuti**, salvo casi particolari gestiti esplicitamente e mai in silenzio. La somma delle promesse **non** concorre al totale ricevuto. Nessun importo negativo dentro `expenses`; nessuna vendita o incasso da opera generato automaticamente. Collegamenti per UUID; il ripristino non deve lasciare eventi orfani; se esistono eventi collegati, eliminazione della spesa sorgente da gestire con blocco e spiegazione, non cascata muta.

**Due letture economiche differenti:** (a) spese originali registrate per **mese di acquisto** — metrica esistente intatta; (b) rettifiche dichiarate per **mese del loro evento** — metrica separata. Un eventuale «costo rettificato complessivo» per singola spesa può essere `12000 - 3000 = 9000` centesimi, ma **non** va presentato come cassa accertata né riscrivere il margine storico luglio. Una vista annuale combinata, se desiderata, deve dichiarare chiaramente convenzione, periodo e assenza di rilevanza fiscale; non mutare le statistiche senza decisione esplicita.

**API descrittive**, solo se necessarie dopo approvazione: «elenca rettifiche di una spesa», «registra evento con verifica dei limiti», «riepilogo rimborsi dichiarati per periodo». Autorizzazione e transazione restano locali Rust/SQLite; nessuna API pubblica o server remoto richiesto.

## 5. Offline, impostazioni, lingue, import/export, privacy

- **Offline-first:** nessun accesso a conti bancari, portali fiscali, provider di pagamento, lettura automatica delle email o notifiche esterne. L'utente dichiara gli eventi; i file dei documenti sono scelti e copiati localmente solo su conferma, se un giorno sarà approvata l'integrazione con UX-028.
- **Impostazioni:** funzione eventualmente nascosta fino al suo primo utilizzo; filtri vista memorizzabili nell'archivio locale, opzione «mostra importi lordi e rettificati» senza cambiare formule autorevoli. Nessun nuovo account, sincronizzazione o categoria obbligatoria.
- **Multilingua:** applicare locale OS e override già esistenti IT/EN/ES/FR/DE, fallback inglese per lingua o stringa mancante; tradurre stati, errori e tooltip, **non** i testi, causali e documenti dell'utente. Conservare codici enum stabili indipendenti dalle etichette tradotte; date/valuta visualizzate secondo locale, scambiate con formati canonici.
- **Import/export:** mantenere **immutato** l'attuale CSV `expenses` per compatibilità dei consumatori; nuovo CSV opzionale `expense_adjustments` UTF-8 con UUID, `expense_id`, tipo, stato, date, centesimi, valuta e riferimento, senza binari e senza riempimenti presunti. Il backup verificato SQLite + media **deve** comprendere tutti i nuovi record ed eventuali allegati prima di dichiarare completa la funzione. Eventuale JSON versionato per round-trip e import locale con anteprima, deduplica UUID, verifica dei riferimenti e rollback, coordinato concettualmente con TECH-001/019; i CSV non sono sostituti del backup.
- **Privacy:** nomi di fornitori, riferimenti di acquisto e ricevute possono identificare persone o abitudini di spesa. Niente telemetria, indicizzazione remota o invio di documenti. Nelle viste condivisibili anteprima e scelta esplicita dei campi, senza note private di default. I backup oggi sono **in chiaro**: spiegare il rischio quando si sceglie una cartella condivisa.

## 6. Dipendenze, complessità, rischi e alternative

**Complessità stimata:** medio-bassa per evento collegato e scheda; **media-alta** se si introducono rettifiche di rendiconti storici, cronologia completa, scambio strutturato e documenti. Necessari: UUID delle spese, SQLite e migrazioni transazionali, calcoli a centesimi interi, backup/ripristino e localizzazione esistenti. Nessuna nuova dipendenza software è decisa.

**Rischi principali:** doppi rimborsi, confusione fra *rimborso promesso*, *accredito dichiarato*, *buono* e *incasso verificato*; apparente contabilità fiscale; duplicazione di giustificativi; cancellazione di acquisti su cui insiste storia; sovrapposizione con futuro registro di movimenti bancari. Risposte: validazione per singola spesa, stati testuali, nessun movimento automatico, report con definizioni visibili, riferimenti ai giustificativi solo opzionali, preservazione della sorgente.

**Alternative da confrontare:** (1) nessun nuovo modulo: nota testuale «rimborso del…», e totale spese lordo invariato; (2) campo facoltativo «rimborso dichiarato» per spesa, sufficiente se è raro e singolo ma inadatto a rate, eventi ripetuti o promesse; (3) registro di eventi minimo come sopra; (4) integrazione futura con quadratura manuale UX-030, senza obbligo né vincolo di sviluppo. Non costruire un sistema di contabilità generale.

**Anti-doppioni:** [UX-013](REPS-UX-013-resi-annullamenti-rettifiche-vendite.md) riguarda **resi e rimborsi di vendite di opere**, non acquisti di materiali; [UX-028](REPS-UX-028-giustificativi-locali-spese-acquisti.md) conserva **documenti** delle spese, non le rettifica; [UX-030](REPS-UX-030-quadratura-facoltativa-estratti-conto-locali.md) confronta estratti bancari, senza decidere il trattamento dei rimborsi; [UX-033](REPS-UX-033-entrate-professionali-non-vendita-opere.md) tratta **corrispettivi da attività**, non restituzioni di spese; [UX-037](REPS-UX-037-scadenze-spese-ricorrenti-senza-registrazioni-presunte.md) prevede scadenze future, non eventi successivi. Nel filone [tecnico — PR #16](https://github.com/Cesare-ml/Repertica-Studio/pull/16), TECH-001, 018, 019, 025 discutono **formati e meccanismi tecnici**, non la semantica dei rimborsi. Nessun loro file modificato.

## 7. Criteri di accettazione ipotetici

1. Su spesa da 120,00 € registrare due rimborsi dichiarati da 20,00 € e 10,00 €: vedere 90,00 € come **costo rettificato informativo**, senza modificare i 120,00 € originali né creare vendite/incassi fittizi.
2. Registrare una nota di credito **promessa** da 15,00 €: il totale «rimborsi ricevuti dichiarati» resta invariato. Lo stesso vale per un buono non utilizzato.
3. Rifiutare operazione che porti il totale dei rimborsi ricevuti dichiarati sopra l'importo originario; non creare eventi parziali in caso di errore e lasciare intatta la spesa.
4. Rimborso nell'anno successivo visibile nel periodo dell'evento, acquisto originario invariato nel suo periodo; la vista economica di default non cambia in modo silenzioso.
5. Spesa con fondo storico disattivato mantiene referenze e rendiconti; nessuna ripartizione delle vendite congelate è riscritta.
6. Esportazione nuovo registro con UUID e riferimenti, senza alterare il CSV `expenses` esistente; backup/ripristino completo conserva i legami e gli allegati eventualmente adottati.
7. Conferme, errori e stati comprensibili con tastiera, screen reader, zoom e lingue supportate; fallback inglese verificabile.
8. Database di versioni precedenti resta apribile via migrazione sicura; nessuna richiesta di account o collegamento di rete; nessuna attestazione bancaria o fiscale simulata.

## 8. Fasi eventuali e decisioni del proprietario

- **Fase 0 — validazione:** interviste brevi a 3–5 artisti/studi con acquisti frequenti: quante restituzioni annue, quali documenti, promessa vs denaro ricevuto, convenzione desiderata nei riepiloghi? Verificare se basta una nota.
- **Fase A — solo se approvata:** eventi minimi collegati a spese, scheda consultabile, distinzione promesso/ricevuto/buono, validazione e preservazione del valore originale. Nessuna modifica ai report storici o import.
- **Fase B — eventuale:** export aggiuntivo, filtri e tabella separata di rettifiche per periodo, supporto documenti locali in coordinamento con UX-028.
- **Fase C — eventuale:** import strutturato non distruttivo, riferimenti manuali a quadrare con estratti conti (UX-030), senza automatismi di cassa.

**Domande da sottoporre al proprietario, non risposte presunte:** conviene rappresentare soltanto rimborsi **effettivamente dichiarati ricevuti** o anche importi attesi? I buoni sono abbastanza frequenti da richiedere gestione strutturata? Il prezzo al netto dei rimborsi deve apparire nelle statistiche mensili, e **secondo quale data**, oppure basta una vista per singolo acquisto? È importante distinguere reso merce, rettifica prezzo e rimborso del servizio? Quale grado di traccia delle correzioni è necessario?

**Esito:** proposta non approvata e non assegnata allo sviluppo. Nessun codice, test, workflow o configurazione deve cambiare a seguito di questa scheda.
