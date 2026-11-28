# REPS-TECH-072 — Coerenza della singola risposta MCP durante le modifiche dell'archivio

- **ID stabile:** REPS-TECH-072
- **Data:** 2026-11-23
- **Stato:** **Proposta — da valutare**
- **Priorità ipotetica:** P2 se viene riprodotta una risposta incoerente di rilievo; P3/nessuno sviluppo altrimenti. Nessuna approvazione o assegnazione.
- **Perimetro:** coerenza delle letture del server MCP esistente, non nuovi tool, permessi o funzioni gestionali.

## 1. Motivazione e riscontri su main

Uno studio può registrare un incasso, una vendita o modificare il prezzo di un'opera mentre il suo assistente locale interroga Repertica. Una singola risposta dovrebbe descrivere uno stato coerente dell'archivio, anche se può essere già superato quando arriva all'utente. La sola lettura non garantisce di per sé la coerenza fra più istruzioni SELECT.

Ricognizione statica del 23/11/2026:
- [mcp.rs, Archive::open e overview/get_artwork](../../../src-tauri/src/mcp.rs): connessione SQLite READ_ONLY, query_only, attesa busy di cinque secondi. overview esegue separatamente somme di sales e sale_receipts, raggruppamento per stato e conteggi. get_artwork legge scheda, impostazioni prezzi, conteggio galleria, vendita/incassi e certificati in query distinte. Il dispatcher call_tool e handle_message non delimitano esplicitamente queste letture con una transazione per risposta.
- [db.rs](../../../src-tauri/src/db.rs): il processo desktop usa un'altra connessione con journaling WAL e mutex interno. Il server MCP può operare mentre la GUI è aperta, come dichiarato in [MCP.md](../../MCP.md). In assenza di una transazione esplicita, le singole istruzioni in autocommit possono acquisire viste differenti; alcuni statement attivi possono già mantenere temporaneamente una vista, quindi la portata reale va verificata, non presunta.
- Il codice mcp.rs serve una richiesta alla volta sulla pipe STDIO; non è necessario creare un nuovo server o rendere persistente una sessione dati.

**Esempio esclusivamente sintetico:** prima di una transazione GUI, venduto 100 EUR e incassato 50 EUR; dopo una nuova vendita e successivo incasso, venduto 300 EUR e incassato 250 EUR. Se le due somme di overview sono lette rispettivamente prima e dopo queste operazioni, la differenza calcolata potrebbe essere −150 EUR, non il residuo di uno stato reale. Non è stato osservato un incidente né eseguito un test concorrente in questa ricognizione. Una fotografia coerente per chiamata non prova la correttezza delle definizioni economiche o delle registrazioni.

**Destinatari:** artisti e gallerie che interrogano il proprio archivio mentre lavorano, archivisti che consultano dati e certificati, utenti di assistenti IA locali che vogliono verificare i risultati senza confondere informazioni correnti e storiche.

## 2. Confronto anti-doppioni

- [TECH-036](REPS-TECH-036-mcp-archivi-grandi-paginazione-limiti-risorse.md): paginazione, costo e continuità fra chiamate. Qui solo l'unità di consistenza **interna a una singola chiamata**, senza cursori né snapshot fra pagine.
- [TECH-055](REPS-TECH-055-esportazione-coerente-registri-csv.md): coerenza fra file CSV esportati in una sessione; qui nessun file, manifest, esportatore o snapshot persistente.
- [TECH-064](REPS-TECH-064-coordinamento-processi-archivio-attivo.md): esclusività di migrazione/ripristino e accesso interprocesso. Non è risolta dalla transazione di lettura qui studiata; il cambio dell'archivio attivo resta fuori perimetro.
- [TECH-014](REPS-TECH-014-bridge-mcp-locale-lettura-permessi.md) e [TECH-049](REPS-TECH-049-confine-istruzioni-dati-mcp-prompt-injection.md): consenso, minimizzazione e confine dati/istruzioni. Garanzie da conservare, non sostituire.
- Confrontate le 76 proposte [Gestione e UX, PR #14](https://github.com/Cesare-ml/Repertica-Studio/pull/14), in particolare UX-039/050/073 (riepiloghi economici) e UX-076 (certificati storici): nessuna propone l'isolamento della singola risposta MCP. Nessun loro file viene modificato.

## 3. Scenari e flussi UX ipotetici

- **Economia:** mentre una vendita o un incasso viene registrato nell'app, l'assistente chiede la panoramica. La risposta mostra soltanto dati da una stessa vista SQLite: tutti prima oppure tutti dopo, mai una combinazione incompatibile.
- **Scheda opera:** modifica del coefficiente o di un allegato durante la lettura; scheda, valore suggerito, conteggio fotografie e riferimenti commerciali descrivono lo stesso stato, senza promettere che sia l'ultimo al momento della visualizzazione.
- **Archivio occupato:** in caso di errore di lettura, l'host riceve un errore esplicito e può riprovare su richiesta dell'utente. Nessuna risposta di successo parziale, nessun risultato memorizzato e presentato come nuovo.
- **Due domande successive:** possono riferirsi a stati differenti. La seconda non è vincolata alla prima e l'utente può chiedere un aggiornamento; non si mantengono transazioni aperte durante l'elaborazione del modello.

Non occorre una nuova schermata: Impostazioni → Assistenti IA locali potrebbe spiegare in una riga il significato di “dati letti durante questa richiesta”. Nell'host, errori e consigli di ripetizione sono testuali e non affidati al colore. Nessun comportamento di un client esterno è garantito. Se il bisogno è raro, è preferibile una nota esplicativa e nessun nuovo sviluppo.

## 4. Specifica funzionale e dati (ipotesi)

**Requisiti certi:** conservare SQLite READ_ONLY/query_only, pipe STDIO, tool e campi attuali, i limiti di risultato, distinzione fra vendite e incassi dichiarati, esclusione di contatti/foto/note private dei collezionisti, nessun account, rete, telemetria o modifica dell'archivio.

**Ipotesi da valutare soltanto dopo una riproduzione:** per i tool con letture multiple, acquisire una transazione SQLite di lettura a durata limitata, effettuare tutte le query e materializzare la risposta nel suo ambito; rilasciare statement e transazione prima di serializzare/spedire la risposta sullo stdout. La vista si stabilisce con la prima lettura, non necessariamente all'inizio della richiesta. Non usare una transazione di scrittura o un lock esclusivo della GUI; nessuna operazione su una copia permanente. Considerare anche una singola query aggregata quando rende più semplice ottenere la medesima vista, senza cambiare la semantica dei risultati.

**Contratto astratto, non codice o API approvata:**
- acquireReadView() → contesto effimero sulla connessione esistente, o errore.
- evaluateToolInView(context, name, validatedArguments) → risultato completo oppure errore.
- releaseReadView(context) → rilascio certo in ogni percorso, prima dell'I/O verso l'host.

Nessuna nuova tabella, UUID, migrazione, cache o timestamp obbligatorio. La prima fase mantiene invariati i sei tool e il JSON già documentato. Eventuale indicazione di consistenza nel contratto delle risposte è solo una seconda ipotesi, da verificare con client vecchi; non deve essere interpretata come marca temporale certificata, dato verificato fiscalmente o prova di autenticità.

## 5. Offline, impostazioni, multilingua, portabilità e privacy

- Funziona interamente in locale, senza dipendenze da fornitori di IA. La scelta di usare un host MCP resta facoltativa; l'host potrebbe trasferire all'esterno i dati che legge, come già avverte MCP.md. Nessun ampliamento delle informazioni disponibili.
- Nessuna impostazione nuova indispensabile. Un eventuale testo in Impostazioni segue lingua di sistema con fallback inglese, scelta manuale già disponibile; codici di errore macchina, se aggiunti, restano indipendenti dalla lingua. Non tradurre nomi e descrizioni archiviate né alterare le chiavi JSON esistenti. Il server attuale contiene messaggi italiani: un eventuale miglioramento della loro localizzazione va valutato separatamente e non è prerequisito.
- Non servono import/export aggiuntivi. CSV, backup integrali verificati in cartelle liberamente scelte e ripristino restano immutati; output MCP non è un backup.
- Non loggare query, nomi, importi o percorsi; fixture esclusivamente sintetiche. Nessuna cache o copia privata lasciata su disco dalla proposta. Eventuali sidecar SQLite già necessari al WAL rimangono sotto il normale controllo del motore: non si promette che ogni accesso SQLite abbia zero attività filesystem.
- Compatibilità prevista con schema corrente (18 al momento della ricognizione) e con i client/protocolli già documentati; nessun aggiornamento dipendenze proposto. Conservare controllo dello schema e validazioni. La transazione non protegge dal ripristino che sostituisce la directory: coordinamento TECH-064 separato.

## 6. Alternative, rischi e complessità

A. **Nessun intervento:** misurare e documentare; preferibile se la discrepanza non si riproduce o non è significativa.
B. **Singola query per aggregati:** possibile per overview; minor durata della lettura, ma più complessa per get_artwork e rischio di cambiare semantica (join e duplicazione aggregati).
C. **Transazione read-only per chiamata:** soluzione da studiare per più query; complessità stimata bassa–media, soprattutto per lifetimes Rust, gestione errori e prove concorrenti. Nessuna nuova dipendenza necessaria in ipotesi, riuso di rusqlite.
D. **Snapshot persistente o lock globale:** sproporzionati e scartati per il perimetro minimo; una copia temporanea aggiunge dati sensibili, disco e pulizia, un lock ostacola il lavoro.

Una lettura WAL prolungata può ritardare il checkpoint e far crescere il WAL: nessuna transazione deve attraversare richieste multiple, attese dell'assistente, output bloccato o operazioni lunghe non necessarie. Valutare impatto della paginazione futura TECH-036 senza introdurla qui. Non promettere tempi di risposta o libertà da SQLITE_BUSY. Un risultato temporalmente coerente può essere obsoleto appena consegnato e può comunque riflettere dati errati registrati dall'utente.

## 7. Criteri di accettazione e prove future (non eseguite)

1. Fixture sintetica con due connessioni, barriere deterministiche fra le SELECT di overview; committare modifiche tra le letture e verificare che ogni risposta appartenga integralmente allo stato prima o dopo. Non basta ripetere un race casuale. Includere il caso in cui la risposta storicamente negativa è un dato reale: nessun clamp del risultato.
2. Stessa prova per get_artwork con modifiche concorrenti di prezzo, foto e vendita; confrontare con gli stati committed conosciuti, non con l'ultimo stato in assoluto.
3. Chiamate consecutive ammettono aggiornamenti; nessun cursore o transazione persistente. Errore SQL simulato, schema non valido e client disconnesso non lasciano risorse di lettura trattenute e non producono un successo parziale.
4. Confrontare latenza, dimensione WAL, tempi di commit della GUI e checkpoint su cataloghi sintetici piccoli e grandi; soglie concordate dopo baseline, su Windows/macOS/Linux. La scrittura locale deve restare utilizzabile.
5. Verificare che i sei tool, i limiti e la struttura JSON restino compatibili; controllare assenza di nuovi dati riservati, contatti, media, log e scritture sul database. Provare una chiamata con client già supportato, senza account nuovi.
6. Verificare errore leggibile e fallback inglese degli eventuali nuovi testi UI, senza cambiare la lingua dei contenuti utente. Backup ed export invariati.

## 8. Fasi, priorità e domande al proprietario

- Fase 0: riproduzione mirata e confronto con test esistenti; verificare se le letture già attive mantengano di fatto la vista in tutti i percorsi interessati. Se il problema non emerge, fermarsi alla documentazione.
- Fase 1: se approvato separatamente, valutare la soluzione minima per overview/get_artwork, misurare effetto sulle scritture e verificare contratti. **P2 soltanto con riscontro concreto**; P1 da discutere solo se viene documentata una conseguenza grave reale, non presunta. P3/nessuno sviluppo se non utile.
- Fase 2 facoltativa: documentare il confine di consistenza per i client, coordinandosi con TECH-036; nessuna snapshot fra chiamate senza nuova valutazione.

**Questioni aperte:** gli utenti interrogano MCP mentre registrano vendite? Quali tool devono garantire uno stato coerente e quali consistono già in una singola SELECT? Quale durata di lettura è accettabile su archivi ampi? È sufficiente correggere overview o serve un contratto uniforme? Come spiegare “coerente ma non necessariamente aggiornato” senza suggerire affidabilità fiscale?

Tutte le risposte e le scelte di sviluppo sono riservate al proprietario. Questa scheda non è una specifica approvata e non autorizza cambi a codice, test, build, workflow, impostazioni o dipendenze.
