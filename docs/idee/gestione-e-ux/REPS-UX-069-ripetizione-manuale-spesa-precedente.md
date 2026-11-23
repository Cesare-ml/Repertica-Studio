# REPS-UX-069 — Nuova spesa a partire da una spesa precedente, senza registrazioni automatiche

- **ID stabile:** REPS-UX-069
- **Data:** 2026-11-23
- **Stato:** **Proposta — da valutare**
- **Filone:** Gestione e UX; ideazione esclusivamente documentale
- **Priorità proposta:** **P2 soltanto con uso ripetitivo verificato**; P3 o nessun intervento se poco frequente
- **Decisione:** non approvata, non assegnata, non autorizzata allo sviluppo

## 1. Problema, destinatari e riscontro nel prodotto

Un artista o una piccola galleria registra spesso **nuove spese simili a precedenti**: acquisti ripetuti presso lo stesso fornitore, materiali da imballaggio, stampa, cornici, servizi periodici. Ripetere a mano categoria, descrizione, fornitore e fondo fa perdere tempo e aumenta il rischio di selezionare un fondo diverso per errore. Tuttavia **due righe apparentemente simili rappresentano due spese reali distinte** solo se l'utente decide di registrarne una nuova: un acquisto passato non dimostra che quello successivo sia stato fatto, pagato o dovuto.

**Riscontro su `main` al 23/11/2026, non prova di un disagio utente:** in `src/app/app.component.html`, sezione «Acquisti e spese», ogni riga propone «Modifica» ed «Elimina», mentre «+ Registra spesa» apre una maschera nuova. In `src/app/app.component.ts`, `newExpense()` parte da un form quasi vuoto; `editExpense(e)` riutilizza l'**ID originale**, quindi modifica la stessa spesa; `saveExpense()` salva il record. In `src-tauri/src/finance.rs`, `save_expense` distingue creazione da modifica tramite `ExpenseInput.id` e applica vincoli a data, importo, descrizione e fondo. Non emerge nel flusso controllato un'azione esplicita **«Usa come base per una nuova spesa»**. Non sono stati svolti test UX con professionisti, né osservate registrazioni duplicate realmente avvenute.

**Utenti candidati:** artisti che acquistano materiali frequentemente, gallerie con spedizioni ripetute, studi con spese di produzione simili. Per pochi movimenti annuali l'attuale form può bastare.

## 2. Differenze rispetto alle idee già documentate

- [REPS-UX-037](REPS-UX-037-scadenze-spese-ricorrenti-senza-registrazioni-presunte.md) riguarda **impegni futuri e scadenze** con cadenza; qui non esiste alcuna regola di ricorrenza, previsione o avviso. Si usa soltanto una **riga già registrata come modello per un nuovo inserimento manuale**.
- [REPS-UX-042](REPS-UX-042-pagamenti-dichiarati-spese-acquisti-differiti.md) riguarda scadenze e pagamenti relativi a un acquisto; qui non si inventano pagamenti o rate.
- [REPS-UX-028](REPS-UX-028-giustificativi-locali-spese-acquisti.md) tratta documenti probatori: non copiare automaticamente giustificativi del vecchio acquisto su quello nuovo.
- [REPS-UX-008](REPS-UX-008-scorte-materiali-consumi.md) tratta quantità fisiche/consumi. La ripetizione della spesa **non** aggiunge scorte né attribuisce consumi a un'opera.
- [REPS-UX-067](REPS-UX-067-protezione-modifiche-non-salvate-moduli.md) propone un'eventuale protezione trasversale dei form sporchi: se approvata in futuro, dovrebbe applicarsi anche a questo nuovo form; non implementarla qui.
- Filone tecnico: [REPS-TECH-002 — automazioni locali](https://github.com/Cesare-ml/Repertica-Studio/blob/docs/idee-strumenti-architettura-20261121/docs/idee/strumenti-e-architettura/REPS-TECH-002-automazioni-locali-consapevoli.md) è volutamente **non necessario**: nessuno scheduler, evento in background, creazione automatica o permesso di scrittura MCP. [REPS-TECH-018](https://github.com/Cesare-ml/Repertica-Studio/pull/16) sugli export riguarda sicurezza del CSV, non la nascita della riga contabile.
- **Regola anti-doppione:** si tratta di un'azione **manuale, puntuale e non persistente**, non di un secondo modulo spese o di una variante di «Modifica».

## 3. Casi d'uso e limiti da evidenziare

1. **Materiali:** una spesa «Colori e tele», €84,60, registrata in ottobre diventa base per un *nuovo acquisto reale* in novembre. L'utente cambia data e importo a €91,20, verifica il fondo e salva. Due UUID differenti, due movimenti soltanto dopo conferma.
2. **Spedizioni:** la stessa galleria ripete un imballaggio per un'opera diversa. Descrizione e fornitore possono essere riusati ma **nessun collegamento automatico a opera/vendita**, nessun costo generato da una consegna.
3. **Canone periodico:** l'utente può riutilizzare i dati dell'affitto di ottobre per registrare manualmente quello di novembre; il programma non crea il mese seguente, non garantisce che il canone sia stato versato e non popola lo scadenziario di UX-037.
4. **Importo cambiato:** il prezzo del fornitore è aumentato; il valore precedente è una *bozza da ricontrollare*, non il valore dovuto attuale.
5. **Fondo non più attivo:** una spesa storica appartiene a una ripartizione disattivata o non ammessa per nuove spese. Il form deve segnalare il problema e chiedere di scegliere un fondo valido oppure nessuno, **mai** ricreare silenziosamente il fondo storico.
6. **Click ripetuto o finestra chiusa:** cliccare più volte sull'azione non deve creare righe; chiudere la bozza non produce spese; un eventuale doppio salvataggio va prevenuto.
7. **Testo datato o sensibile:** note con numero ordine, indirizzo o riferimento a un cliente devono essere verificate prima di essere copiate; alternativamente **non copiare le note per default**.

## 4. Flusso UX minimo, con decisioni esplicite

**Ingresso:** `Economia → Acquisti e spese`, azione secondaria per riga «Nuova spesa da questa» / «Usa come base». Etichetta semanticamente distinta da «Modifica». Nelle viste compatte, la voce può stare nel menu azioni accessibile a tastiera. Niente pulsanti globali che agiscano su una selezione ambigua.

**Apertura:** `form` nuovo, indicato nel titolo come «Nuova spesa — dati suggeriti dalla spesa del [data]». Nuovo record senza `id`; **nessun salvataggio al click**. Prefill candidati: categoria, descrizione, fornitore, URL d'acquisto e fondo **solo se ancora valido**. Data nuova impostata al giorno locale corrente, visibilmente modificabile. Importo precedente mostrato in campo modificabile con un testo «Verifica il nuovo importo»; prima dell'effettivo salvataggio richiedere una **conferma di revisione dell'importo** oppure, come variante più prudente da testare, lasciare l'importo vuoto e visualizzare il precedente solo come riferimento. **Note vuote per default**; se un utente vuole recuperarle deve farlo deliberatamente, per evitare propagazione di dati legati al vecchio ordine.

**Revisione:** riepilogo non tecnico «Stai creando una seconda spesa; quella del [data] rimane invariata». Data e importo sempre espliciti. «Salva nuova spesa» solo con campi validi; il pulsante e la richiesta asincrona si bloccano durante il salvataggio. «Annulla» non scrive nulla. Eventuale protezione delle modifiche non salvate rimane responsabilità condivisa con UX-067.

**Dopo il salvataggio:** il registro mostra la nuova riga e quella originaria senza toccare la seconda; accesso diretto alla nuova voce e messaggio di esito. Nessuna annotazione «pagata», nessuna notifica, nessun CSV generato automaticamente, nessuna seconda registrazione involontaria.

**Pratiche di accessibilità:** etichette complete senza dipendere da icone, pulsante raggiungibile da tastiera, focus all'apertura sul titolo/primo campo, errori annunciati e associati ai campi, focus restituito alla riga di partenza alla chiusura, leggibilità con zoom 200%; messaggio comprensibile anche in assenza di colore.

## 5. Dati e possibile interfaccia — solo descrittivi

**Requisiti certi verificati:** `Expense` corrente possiede `id`, `expense_date`, `category`, `description`, `amount_cents`, `fund?`, `supplier?`, `purchase_url?`, `notes?`, `created_at`, `updated_at`. `saveExpense` inoltra al comando Rust `save_expense`; la creazione senza `id` produce un nuovo UUID. Gli importi sono centesimi interi EUR.

**Ipotesi minima:** usare una copia **solo nello stato del form** dei dati scelti da una spesa esistente, azzerare `id`, non trasportare `created_at/updated_at`, non assegnare una relazione permanente fra spesa originaria e copia. Usare la stessa validazione nativa del registro; il nuovo record è ordinario e resta esportabile come tutti gli altri. Un generico comando di «clonazione» sul backend **non è richiesto** in questa fase.

**Ipotesi per robustezza UX:** una variabile transitoria `sourceExpenseId` può servire esclusivamente per mostrare l'avviso di provenienza; **non è** un nuovo campo di database, un audit probatorio o una modifica alla contabilità. Il backend non deve accettare `id` del modello per un inserimento nuovo.

**Conferma dei dati:** controllare `amount_cents > 0`, data ISO corretta, descrizione non vuota, fondo ancora autorizzato, URL opzionale conforme ai vincoli attuali. I campi moneta dell'interfaccia non devono essere convertiti con approssimazioni non controllate: mantenere i centesimi e le regole esistenti. Se un registro esterno è alterato o il salvataggio fallisce, non attribuire esiti positivi e conservare la bozza recuperabile nella sessione.

**Non requisiti:** deduplicazione automatica delle spese, creazione batch, calcolo IVA, stock materiali, contabilizzazione di pagamenti, previsioni di cassa, link permanente alla spesa-madre, notifica di ricorrenze, creazione o cancellazione di documenti.

## 6. Offline, impostazioni, multilingua, privacy ed esportabilità

- **Offline:** nessuna rete, account, abbonamento, IA o sincronizzazione. L'azione usa il record già caricato in SQLite locale e funziona con chiavetta/NAS non disponibili. Nessuna automazione quando l'app è chiusa.
- **Impostazioni:** **nessuna nuova preferenza obbligatoria** per la fase minima. Eventuale preferenza «copia anche le note» *solo dopo ricerca utenti*, inizialmente disattiva; non aggiungere opzioni preventive senza valore provato.
- **Lingue:** lingua del sistema via impostazioni esistenti (IT/EN/ES/FR/DE), override opzionale, fallback inglese. Titoli, azioni, spiegazioni, messaggi di errore e avvisi traducibili; i contenuti personali di descrizione, fornitore e note **non** vanno tradotti automaticamente. Date memorizzate ISO, mostrate nel formato locale.
- **Privacy:** fornitori, URL, descrizioni e note possono includere dati personali. Non copiare note di default, nessuna copia negli appunti, nessuna telemetria o invio a un servizio. Nessun dato al server MCP o in un'email come effetto laterale.
- **Import/export e backup:** la nuova spesa salvata appare nei CSV spese e nel backup SQLite+media già disponibili **esattamente come una spesa inserita a mano**. Nessun nuovo formato di import, esportazione parallela, modifica di colonne o metadati di genealogia; non esportare bozze non salvate.
- **Compatibilità:** nessuna migrazione SQLite ipotizzata per il minimo; non modificare storia delle spese, quote congelate sulle vendite, saldi, certificati o date storiche. Compatibilità Windows/macOS/Linux e formati di input data/importo esistenti da verificare con collaudo nativo.

## 7. Dipendenze, complessità, rischi e alternative

**Dipendenze progettuali:** form spese, comando Rust `save_expense`, elenco spese, lista dei fondi ammissibili, i18n e convenzioni di conferma già presenti. **Nessuna dipendenza esterna o nuova libreria proposta.**

**Complessità orientativa:** bassa per comando manuale + form con `id` nullo e messaggi dedicati; medio-bassa se si vuole riuso robusto su dati migrati, accessibilità nativa e gestione di doppi clic. L'eventuale rilevamento dei doppioni tramite importo/data/fornitore introduce falsi positivi e va studiato **separatamente**.

**Rischi:** registrazione di due spese quando una sola è avvenuta; importo vecchio non verificato; note sensibili propagate; fondo inattivo o non valido; confusione fra «Modifica» e «Crea nuova»; errori di doppia pressione; percezione ingannevole di un pagamento verificato. Protezioni: etichette, conferma consapevole, validazioni, campo data nuovo, note vuote, blocco doppio invio.

**Alternative da provare prima:** mantenere inserimento attuale, consentire **solo copia testuale di fornitore/descrizione** in una nuova maschera, migliorare focus/tab nei campi; preparare un breve modello esterno per spese comuni. Scegliere UX-037 soltanto quando l'esigenza reale è **ricordare una scadenza futura**, non evitare la ridigitazione di una spesa presente.

## 8. Test e criteri di accettazione ipotetici (non eseguiti)

1. Da una spesa esistente, «Nuova spesa da questa» apre un **nuovo form**; l'UUID di origine non viene usato nel salvataggio.
2. Prima di «Salva» il conteggio e il totale delle spese restano identici; «Annulla» non crea né cambia record.
3. La data è intenzionalmente nuova e l'importo viene rivisto/riconfermato; due date uguali **non** comportano fusione o eliminazione automatica.
4. Il salvataggio riuscito produce **una sola** nuova spesa con nuovo UUID; quella precedente è identica byte-per-byte nei campi autorevoli.
5. Due click ravvicinati non generano doppia scrittura; errore nativo mantiene la bozza e non comunica successo.
6. Fondo non più valido: avviso e scelta esplicita di alternativa o nessun fondo; nessuna attribuzione automatica silenziosa.
7. Note del vecchio acquisto non vengono duplicate per default; eventuali URL sono convalidati come negli inserimenti ordinari.
8. La nuova spesa compare una sola volta nei riepiloghi mensili, nell'export CSV spese e nel backup; nessun movimento bancario o nuovo incasso.
9. Dialoghi utilizzabili con tastiera e screen reader, messaggi con significato testuale e focus gestito; test lingua sistema/override/fallback inglese.
10. Prove native Windows/macOS/Linux su descrizioni Unicode, date locali vicine alla mezzanotte, valute visualizzate ed elenco con molte spese; non creare script di test o codice in questa attività.

## 9. Priorità, fasi e domande riservate al proprietario

**Priorità proposta: P2 condizionata** a colloqui con almeno 3–5 artisti/galleristi che ripetono realmente dati di spese più volte al mese, e a un test in cui si misura il tempo/errori rispetto a «Nuova spesa». In caso contrario **P3 o nessuno sviluppo**. Nessuna approvazione implicita.

**Fase 0 — validazione senza codice:** mostrare form esistente a professionisti e chiedere una prova di 10 spese simili; misurare quali campi ripetono davvero e quanto sono rischiosi importi e note.

**Fase 1 — ipotesi di minimo:** singola azione «Nuova spesa da questa», nuova data, descrizione/categoria/fornitore/fondo riutilizzati con cautela, conferma importo, note vuote, nessun automatismo o schema nuovo.

**Fase 2 — solo se necessario:** confronto dettagliato prima/dopo o avviso prudente su possibile duplicazione nello stesso giorno, **senza blocchi automatici** né controllo indiscriminato del registro.

**Decisioni per il proprietario (aperte):** (a) l'azione va presentata per ogni riga o in un menu secondario? (b) lasciare importo vuoto o precompilarlo con conferma esplicita? (c) riusare il fondo solo se attivo oppure richiederne sempre la selezione? (d) vale la pena consentire un recupero volontario delle note, o è preferibile escluderle completamente? (e) quante spese simili vengono registrate in un mese e quanti errori emergono oggi?

**Distinzione finale:** confermati soltanto i dati e i flussi letti in `main`; bisogno, priorità, dettagli UX e tempi di realizzazione sono **ipotesi**. Questa scheda non è una specifica approvata né un incarico di implementazione.
