# REPS-UX-067 — Protezione delle modifiche non salvate nei moduli di opere, collezionisti e spese

- **ID stabile:** REPS-UX-067
- **Data della ricognizione:** 2026-11-23
- **Stato:** Proposta — da valutare
- **Filone:** Gestione e UX
- **Priorità ipotetica:** P2 se confermata la perdita accidentale di modifiche; P3 o nessun intervento se le prove non evidenziano problemi
- **Natura:** esclusivamente analisi/documentazione. Nessuna approvazione, assegnazione, modifica a codice, test, build, schema, impostazioni, workflow o dipendenze.

## 1. Problema concreto e destinatari

Un artista può dedicare minuti a titolo, tecnica, misure, descrizione e foto di un'opera; un archivista corregge l'indirizzo e le note di un collezionista; una galleria registra un acquisto di materiali e il relativo fornitore. Se si preme **Chiudi** o **Annulla**, si naviga verso un'altra schermata o si apre la galleria senza prima salvare, il programma non dovrebbe eliminare silenziosamente i valori modificati.

**Utenti:** artisti individuali, operatori di galleria, segreterie di studio, archivisti e chi usa mouse, tastiera, tecnologie assistive o finestre piccole. La funzionalità non richiede utenti online o account e **non deve rendere più lento il percorso di chi non ha modificato nulla**.

**Evidenze circoscritte a `main`, lette il 23 novembre 2026:**

- In [`src/app/app.component.html`](../../../src/app/app.component.html), il dialogo **collezionista** chiude con `collectorDialog=false` sia dalla X sia da **Annulla**; anche il dialogo **spesa** chiude direttamente con `expenseDialog=false`. I campi sono associati a `ngModel`. È un percorso da provare: la chiusura elimina il contenuto della maschera non salvata, non la scheda persistita.
- In [`src/app/app.component.ts`](../../../src/app/app.component.ts), `cancelArtworkDialog()` chiude la maschera opera e chiama `finishPhotoSession(new Set())`, che sposta nel cestino gli import temporanei non utilizzati. Nella stessa maschera, l'azione **Galleria** imposta direttamente `artworkDialog=false` prima di aprire la galleria; le due uscite non passano da un unico controllo.
- In [`src/app/business-mail.component.ts`](../../../src/app/business-mail.component.ts) esiste già il precedente `hasUnsavedComposer` / `canLeave()`: la composizione email chiede se scartare modifiche non salvate. La proposta riguarda la **coerenza UX degli altri moduli**, non un secondo editor di posta.
- Non è stata eseguita una prova end-to-end della build nativa; questa è una **lacuna verificabile nel flusso sorgente**, non una statistica di incidenti reali. La frequenza degli errori va misurata osservando professionisti.

## 2. Confini e verifica anti-doppioni

- **[REPS-UX-018](REPS-UX-018-catalogazione-seriale-schede-bozza.md)** propone catalogazione in serie e, in una fase evoluta, **bozze persistenti di nuove opere**. Qui la prima fase protegge la semplice *uscita da una maschera modificata*, senza creare record incompleti, bozze permanenti né una nuova modalità di inserimento.
- **[REPS-TECH-033 — salvataggi condizionali](https://github.com/Cesare-ml/Repertica-Studio/blob/docs/idee-strumenti-architettura-20261121/docs/idee/strumenti-e-architettura/REPS-TECH-033-salvataggio-condizionale-schede-obsolete.md)** studia conflitti fra **contenuto del database cambiato dopo l'apertura** e maschera obsoleta. Qui si riconosce che **l'utente ha cambiato la maschera ma non ha premuto Salva**. Un avviso di uscita non sostituisce il controllo di concorrenza e non lo implementa.
- **[REPS-UX-064](REPS-UX-064-revisione-guidata-metadati-opere-esistenti.md)** studia aggiornamenti a più schede; non è il caso di uscita accidentale da un singolo editor. **[REPS-TECH-062](https://github.com/Cesare-ml/Repertica-Studio/blob/docs/idee-strumenti-architettura-20261121/docs/idee/strumenti-e-architettura/REPS-TECH-062-progresso-annullamento-sicuro-operazioni-locali.md)** riguarda operazioni native di lunga durata, non campi di un modulo.
- Il composer email offre il **pattern già esistente** da valutare, ma non è modificato. Non si propone un sistema di autosalvataggio generalizzato né di recupero dopo crash: questa esigenza va studiata a parte, senza sovrapporre UX-018.

## 3. Scenari d'uso

1. **Opera esistente, descrizione riscritta.** Dopo una modifica, l'artista preme X: può **Continua a modificare**, **Scarta modifiche** o **Salva**. Scarta non altera la scheda salvata; Salva usa le validazioni normali e chiude solo a successo confermato.
2. **Nuovo collezionista con dati personali.** L'archivista chiude dopo aver inserito telefono e note: viene avvisato; nessuna bozza viene inviata a servizi esterni o registrata senza esplicita scelta.
3. **Spesa di materiali.** L'utente cambia importo e categoria, poi seleziona Annulla. Con dati modificati appare l'avviso; senza modifiche la chiusura resta immediata.
4. **Opera appena creata con fotografie temporanee.** L'uscita deve spiegare l'effetto di **Scarta** sugli import della sessione, che potranno finire nel cestino del Media Center. Rimanendo nel modulo non deve partire la pulizia.
5. **Scheda già esistente con editor media.** Modificare un file nel Media Center può produrre **subito una versione persistita**. L'avviso deve distinguere i *campi ancora da salvare* dalle modifiche a fotografie **già applicate**: Scarta non promette di annullare quelle versioni.
6. **Navigazione verso Galleria e altre pagine.** Una transizione non dovrebbe eludere il controllo sulle modifiche in sospeso; se l'utente rimane, dialogo e focus devono restare stabili.

## 4. Flusso UX proposto (solo progettazione)

**Apertura:** per le modifiche di record esistenti, conservare un'istantanea normalizzata dei **soli campi del modulo** all'apertura; per i nuovi record, definire quali campi precompilati sono valori iniziali e quali sono stati davvero cambiati. L'assenza di differenze apre la pagina successiva senza dialogo aggiuntivo.

**Rilevamento di variazioni significative:** confrontare valori normalizzati (es. stringhe vuote e `null` ove semanticamente equivalenti) e le nuove fotografie in sessione. Non marcare come «modificato» un semplice focus, una visualizzazione dell'anteprima o una traduzione UI. I campi commerciali o di stato protetti dai normali controlli rimangono protetti.

**Tentativo di uscita:** tutte le azioni previste per lasciare il modulo (X, Annulla, navigazione interna, Galleria, cambio sezione e, nei limiti delle API native, chiusura finestra) confluiscono idealmente nella stessa decisione:
- **Continua a modificare** (default/focus iniziale, nessun dato perso).
- **Scarta le modifiche del modulo** (solo dopo conferma; conseguenze degli import temporanei rese esplicite).
- **Salva e chiudi**, se il modulo e le sue validazioni lo consentono. In caso di errore di salvataggio resta aperto, con messaggio contestuale; mai dichiarare riuscito ciò che non è persistito.

**Casi critici:** durante un salvataggio in corso, impedire doppia conferma o una chiusura incoerente; dopo una risposta positiva del backend, aggiornare la baseline prima di qualsiasi uscita. Per le fotografie di un'opera, elencare chiaramente ciò che verrà cestinato e ciò che è già persistito; nessun rollback fittizio. In assenza di una garanzia affidabile sull'intercettazione della chiusura forzata di OS/app, **non promettere recupero automatico**.

**Accessibilità:** usare un dialogo propriamente etichettato, focus intrappolato mentre aperto e restituito all'azione originaria, ordine tastiera prevedibile; Escape con semantica di «continua», non «scarta». Etichette testuali chiare, nessun solo colore, supporto screen reader e finestre piccole. Nessuna conferma invasiva quando il modulo è invariato.

## 5. Dati, configurazioni e API — ipotesi descrittive

**Prima fase senza migrazioni:** `formBaseline` per tipo editor e identità record, `hasMeaningfulChanges`, `pendingNavigation` e riferimenti agli import fotografici della sessione sono **stato volatile dell'interfaccia** (nomi esemplificativi, non contratti definiti). Il confronto deve usare una whitelist di campi rilevanti, non serializzare indiscriminatamente oggetti che comprendono thumbnail, percorsi assoluti o segreti.

**Solo eventuale fase evoluta:** un contratto descrittivo `requestLeave(formKind, recordId, intendedAction)` restituisce una scelta fra resta/scarta/salva e può coordinarne l'esito; non è un endpoint remoto. Nessuna nuova tabella, registro o background job è indispensabile per la prima fase. Un'eventuale persistenza di bozze dopo riavvio è **fuori perimetro** e va coordinata con UX-018.

**Preferenze:** inizialmente comportamento protettivo predefinito, senza interruttore globale da aggiungere. Eventuale scelta «non chiedere ancora» solo dopo evidenze d'uso, con rischi e reversibilità espliciti. L'avviso non offre autosave né invia notifiche.

## 6. Offline, lingue, privacy, import/export, compatibilità

- **Offline-first:** nessuna rete, account, abbonamento, telemetria, assistente IA o servizio richiesto. Database locale SQLite e media esistenti rimangono fonti autorevoli: l'avviso non effettua scritture se si sceglie **Resta**.
- **Multilingua:** testi e nomi azione nelle lingue già supportate (IT/EN/ES/FR/DE) secondo lingua di sistema e fallback inglese, con scelta manuale già presente. La lingua non cambia la semantica dei tre esiti e non traduce note libere.
- **Privacy:** i dati in compilazione possono includere indirizzi, note riservate, dettagli economici e fotografie non pubbliche. Non riportarli nel testo della conferma, in log o in analytics. Le bozze non devono essere scritte in file di debug o accessibili dal server MCP; cancellare lo stato volatile al termine reale dell'editor. Il backup attuale è in chiaro: questa proposta non aggiunge copie persistenti.
- **Import/export:** nessun nuovo formato nella prima fase. **Scarta** non altera registri CSV, media già salvati, backup esistenti, certificati o storici. **Salva** conserva il normale schema e gli export attuali; nessuna bozza deve essere spacciata per record importabile o esportabile. Se un futuro recupero persistente venisse approvato, richiedere specifica separata di export, cancellazione e backup.
- **Compatibilità:** non introdurre migrazioni DB, modifiche agli IPC Rust o dipendenze per la sola conferma; verificare la navigazione in Tauri su Windows/macOS/Linux, con tastiera e screen reader. L'eventuale controllo di conflitti di TECH-033 resta autonomo e prevale quando il backend rifiuta un record obsoleto.

## 7. Dipendenze, complessità, rischi e alternative

- **Dipendenze probabili:** state management attuale di `app.component.ts/html`, ciclo degli editor, `run()`/salvataggi esistenti, gestione degli import temporanei nel Media Center e stringhe i18n. Nessuna dipendenza esterna necessaria come ipotesi iniziale.
- **Complessità ipotetica:** **bassa–media** per singoli moduli con snapshot e conferma coerente; **media** per unificare tutte le uscite, le transizioni verso la galleria, i side effect fotografici e il blocco durante salvataggi; **alta** per autosave/crash recovery (esclusi).
- **Rischi:** troppe conferme (assuefazione), falsi positivi da normalizzazione, immagini temporanee cestinate prima della decisione, confusione tra modifica del modulo e versione media persistita, salvataggi doppi, falsa promessa di protezione da spegnimenti improvvisi. Mitigazioni: test reali, confronto dei soli campi significativi, messaggi precisi, conferma prima di pulizia e prova con errori simulati.
- **Alternative minime:** un avviso testuale solo sui moduli più lunghi; una piccola dicitura «Modifiche non salvate» e conferma soltanto su X/Annulla; formazione all'uso del tasto Salva. **Non** proporre autosave indiscriminato: per spese, importi e dati personali può consolidare valori non ancora verificati.

## 8. Criteri di accettazione e prove ipotetiche

1. Modulo opera/collezionista/spesa **invariato**: uscita immediata, senza avviso.
2. Modulo con cambiamento significativo: X e Annulla mostrano tre scelte chiaramente distinte, con **Continua** non distruttivo; prova da tastiera/screen reader.
3. Per un nuovo collezionista si conferma **Scarta**: nessun record è creato e nessuna nota appare in export, backup o MCP.
4. Premendo **Salva** con errori di validazione o backend indisponibile, il modulo rimane con i valori inseriti e mostra l'errore senza segnalare falso successo.
5. Con fotografie importate nella sessione, **Continua** non le cestina; **Scarta** le tratta secondo la politica esistente, esplicitata in UX; gli originali/media già consolidati non sono cancellati.
6. Entrando in **Galleria** da un'opera modificata, la navigazione non ignora la scelta; l'utente comprende che eventuali interventi già salvati nel Media Center **non sono annullati**.
7. Nessuna nuova tabella, migrazione, chiamata cloud, modifica ai file già esportati o promozione di bozze a schede ufficiali.
8. Le prove sono ripetute in italiano e inglese (fallback), con test di layout delle ulteriori lingue e su finestre di dimensione ridotta; la navigazione e i focus restano validi.
9. Test esplorativo con 4–6 professionisti: misurare uscite accidentali evitate, false conferme e tempo aggiunto; se il costo UX supera il beneficio, preferire un avviso mirato.

## 9. Fasi indicative (non roadmap approvata)

- **A — verificare il problema:** sessioni osservate su tre moduli principali, screenshot/flussi e classificazione delle uscite per rischio; distinguere bug riproducibili da aspettative.
- **B — soluzione minima da valutare:** conferma su modifiche reali in opera, collezionista e spesa, inclusa la pulizia media; senza persistenza di bozze.
- **C — eventuale coerenza trasversale:** ulteriori moduli e navigazioni, solo dove emergono rischi equivalenti; accessibilità e prevenzione dei falsi positivi. **Mai** estendere automaticamente a backup/versionamento o alla posta già protetta.

## 10. Requisiti certi, ipotesi e decisioni riservate al proprietario

**Certi dalla documentazione e dal codice esaminato:** app Tauri/Angular offline-first, SQLite locale, salvataggi espliciti, X e Annulla diretti su moduli, precedente anti-perdita nella posta, sessioni di media temporanei, backup in cartella scelta, multilingua sistema/fallback inglese.

**Ipotesi non validate:** gli utenti chiudono questi moduli per errore con sufficiente frequenza; una conferma condizionale riduce incidenti senza risultare fastidiosa; le transizioni verso Galleria andrebbero coordinate con il medesimo controllo.

**Da decidere personalmente:** vale P2 su tre moduli o basta una protezione mirata? Mostrare sempre **Salva e chiudi** o solo se tutti i campi essenziali sono validi? Quale testo descrive senza ambiguità il cestino di import fotografici? Quali azioni fuori dai moduli richiedono un comportamento analogo? Le prove native mostrano altri percorsi già protetti non emersi dalla ricognizione statica?

**Questa scheda non costituisce approvazione né mandato allo sviluppo.**