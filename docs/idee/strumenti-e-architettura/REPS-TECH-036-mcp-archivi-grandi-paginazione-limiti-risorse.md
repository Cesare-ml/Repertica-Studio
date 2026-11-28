# REPS-TECH-036 — MCP locale per archivi grandi: paginazione stabile e limiti di risorse

- **ID stabile:** REPS-TECH-036
- **Data:** 2026-11-22
- **Stato:** **Proposta — da valutare**
- **Priorità proposta:** **P2 condizionata** a misurazioni su archivi ampi; **P3** se i volumi reali non giustificano la complessità. Nessuna approvazione, assegnazione o autorizzazione allo sviluppo.
- **Perimetro:** robustezza e interoperabilità del **server MCP locale già implementato**, in sola lettura. Nessun nuovo assistente, cloud, login, API HTTP, accesso remoto o ampliamento delle informazioni condivise.

## 1. Problema, destinatari e riscontri verificabili

**Problema professionale da validare:** uno studio con migliaia di opere, collezionisti, certificati o vendite può voler esplorare l'archivio da un assistente MCP senza attendere una scansione completa a ogni richiesta e senza poter ottenere solo i primi 50/200 risultati. Per un archivista o una galleria, l'impossibilità di continuare una lista ordinata dopo la prima pagina rende meno utile la consultazione *anche se ogni chiamata è in sola lettura*.

**Riscontri statici su `main`, 22/11/2026:**

- [`docs/MCP.md`](../../MCP.md) descrive sei tool MCP **già funzionanti** via STDIO: `archive_overview`, `search_artworks`, `get_artwork`, `list_collectors`, `list_sales`, `list_certificates`. Nessun account o porta di rete. Le informazioni esposte **includono** nomi dei collezionisti, importi e descrizioni delle opere; **escludono** contatti, note private e byte/percorso delle fotografie. L'uso di un client IA esterno può comunque inviare al suo provider le informazioni lette.
- In [`src-tauri/src/mcp.rs`](../../../src-tauri/src/mcp.rs), `search_artworks` (righe indicative 166–215) legge e ordina le righe delle opere, applica `fold` e i filtri nel ciclo Rust, conta tutti i risultati e conserva **solo i primi `limit`** (massimo 50); non riceve cursore né offset. `list_collectors` esamina tutti i nomi e conta le corrispondenze; `list_sales` scorre tutte le vendite del periodo anche quando restituisce solo le prime 200.
- `list_certificates` (righe indicative 348–365) **raccoglie i certificati in un `Vec`** e solo dopo applica `.take(limit)`: il limite della risposta non delimita i dati intermedi caricati. Le definizioni dei tool non hanno `cursor`/`next_cursor` né una via documentata per leggere la pagina successiva.
- `serve` (righe indicative 468–480) usa `input.lines()` e `handle_message` analizza ogni riga JSON; nel percorso letto **non appare un limite esplicito di dimensione della singola richiesta**. Questo è un *rischio di consumo risorse da esaminare*, non la prova di una vulnerabilità sfruttata.
- [`src-tauri/src/db.rs`](../../../src-tauri/src/db.rs) include indici su alcune colonne, ma non dimostra di per sé che la ricerca accent-insensitive e per sottostringhe di `mcp.rs` possa essere trasformata senza differenze semantiche in un `LIKE` o in FTS.

**Limite della ricognizione:** non sono stati eseguiti benchmark su archivi grandi né test MCP desktop reali; non si afferma che l'app attuale sia inutilizzabile, che i limiti di output siano errati o che ci sia una perdita di dati. La priorità deve dipendere dal numero effettivo dei record e dai tempi misurati.

## 2. Distinzione dalle idee esistenti

- [TECH-014](REPS-TECH-014-bridge-mcp-locale-lettura-permessi.md) riguarda **autorizzazioni, ambiti e revoca** dei dati esposti dai tool; non il completamento degli elenchi su più pagine né i consumi di memoria per chiamata. Ogni eventuale pagina deve comunque rispettarne le restrizioni.
- [TECH-035](REPS-TECH-035-configurazioni-mcp-client-esterni-conflitti-recupero.md) tutela file di configurazione **di altri client** durante collegamento/scollegamento: non tocca query o protocollo di risposta del server.
- [TECH-008](REPS-TECH-008-viste-salvate-filtri-ricerca-locale.md) riguarda filtri e viste nell'**interfaccia dell'archivio**, [TECH-004](REPS-TECH-004-ricerca-federata-archivi-locali.md) più archivi e [TECH-020](REPS-TECH-020-accessibilita-flussi-critici-tastiera-screen-reader.md) accessibilità dei flussi UI. Qui si esamina solo la **consultazione di un archivio tramite il protocollo MCP**.
- Le **40 schede** di [Gestione e UX, PR #14](https://github.com/Cesare-ml/Repertica-Studio/pull/14) includono ricerche, relazioni, inventario e analisi delle vendite: non risultano una specifica del limite di lettura/ripresa delle risposte MCP. **Nessun file dell'altro filone viene modificato**.

## 3. Scenari d'uso e flussi UX proposti

1. **Catalogo esteso:** l'utente chiede a Codex «mostrami le opere disponibili ad acquerello». Una prima chiamata riceve al massimo 50 elementi; il client potrebbe chiedere volontariamente `next_cursor` per continuare. Nessun caricamento obbligatorio di tutte le pagine.
2. **Esame dei certificati:** un archivista consulta più di 200 certificati. Può proseguire in ordine deterministico senza che una richiesta costruisca un array intermedio di tutti i certificati.
3. **Vendite storiche:** una galleria cerca vendite di un determinato anno. `list_sales` può scorrere una pagina dopo l'altra; i totali globali (se richiesti) restano distinti dal numero di record restituiti.
4. **Archivio modificato durante la consultazione:** un'altra operazione registra un nuovo documento tra due chiamate. Il client vede che la continuazione non è uno snapshot storico garantito; una ripetizione o un riavvio dell'elenco è esplicito e non duplica/inventa record per convenienza.
5. **Richieste accidentali troppo pesanti:** un host passa una stringa di ricerca enorme o una riga JSON eccessiva. Il server deve restituire un errore prevedibile o chiudere in modo controllato, senza loggare la query privata o consumare memoria senza limite definito.
6. **Utente normale:** con 20 opere, i tool e i valori `limit` correnti restano utilizzabili senza dover comprendere paginazione o cambiare impostazioni.

**Flusso UX:** Impostazioni → Assistenti IA locali → eventuale area informativa «Lettura di archivi grandi» con spiegazione *facoltativa* di limiti di risultato e continuità. Nessun nuovo pannello indispensabile: il cursore è un dettaglio di protocollo trasparente per i client compatibili. In caso di errore, il client riceve un codice comprensibile; se il testo viene mostrato all'utente, è localizzato e suggerisce di restringere la ricerca o ripartire dalla prima pagina. **Nessun recupero automatico di tutte le pagine** e nessun consenso implicito ad allargare l'accesso ai dati.

## 4. Specifica funzionale ipotetica

**Requisiti certi da preservare:** sola lettura SQLite, un archivio selezionato, STDIO senza rete, assenza di account e abbonamenti, filtri e ordinamenti pubblici esistenti, limiti di output, nessun contatto/nota privata/fotografia aggiuntiva, compatibilità del più possibile con tool e client attuali.

**Possibile evoluzione, non decisione tecnica:**

- **Misurare prima:** collezioni sintetiche di 100, 10.000 e 100.000 righe; rilevare memoria di picco, latenza totale, quantità di righe lette e impatto sul database aperto dall'app. Registrare soltanto metriche di test locali senza dati dei clienti.
- **Risposte continuabili opzionali:** `limit` mantiene il significato attuale; per le funzioni elenco si valuta un `cursor` opaco e una risposta `next_cursor`/`has_more`. Non emettere un cursore utilizzabile su un archivio diverso. I client vecchi, senza cursore, conservano la prima pagina e i campi esistenti.
- **Ordini stabili:** criterio dichiarato per ciascun tool (es. data + UUID come spareggio, rispettando l'ordine attualmente promesso); evitare duplicazioni o salti dovuti a chiavi uguali. Meglio valutare paginazione *keyset* che `OFFSET` crescente, senza promettere snapshot fra processi.
- **Conteggi e costi distinti:** i campi attuali `total_matches`/`total_sales` hanno un significato e non si cambiano silenziosamente. Un conteggio esatto può richiedere l'intera scansione: valutare un'opzione esplicita di conteggio o una modalità compatibile con limitazioni documentate. Mai chiamare «totale» un conteggio parziale.
- **Budget per richiesta:** possibili soglie conservative (byte della richiesta, lunghezza query, dimensione risposta, tempo/righe di lavoro) e fallimento chiaro. I valori reali vanno scelti **solo dopo benchmark** e prova di compatibilità con client; non si propone un numero arbitrario.
- **Ricerca:** la semantica già documentata (accenti, parole su campi diversi, frasi esatte nello stesso campo) non deve regredire. `LIMIT` SQL e `FTS5` non sono sostituzioni automatiche di `fold` + `matches_all`: se si studia un indice, verificare normalizzazione, aggiornamento locale, ordinamento, backup, ricostruzione e costi.
- **Nessuna cache persistente necessaria** per la prima fase. Eventuali indici derivati locali devono essere ricostruibili e non diventare l'unica copia dei dati.

## 5. Dati e API solo descrittivi

Non si propone una migrazione SQLite obbligatoria. Possibile contratto **concettuale**:

```text
ListQuery { query?, status?, year?, limit?: integer, cursor?: opaque_string }
ListPage<T> {
  items: T[],
  returned: integer,
  has_more: boolean,
  next_cursor?: opaque_string,
  total_matches?: integer,        // se semanticamente esatto e richiesto
  consistency: "live_read"        // non snapshot multi-chiamata
}
Error { code: INVALID_CURSOR | ARCHIVE_CHANGED | RESOURCE_LIMIT |
              INVALID_ARGUMENT | ARCHIVE_UNAVAILABLE, safe_message }
```

Il cursore **non** deve contenere dati personali leggibili, percorsi, query in chiaro o chiavi segrete; non può essere considerato un'autorizzazione. Va legato alla selezione dell'archivio, versione del contratto, filtro e ordinamento, con durata/scope da definire. Cursori errati, manomessi o provenienti da altre query devono essere rifiutati. Se si usa un digest o identificatore locale, definire come evitare correlazione di archivi e perdite di privacy. Una pagina non implica che il database sia rimasto immutato dopo la lettura.

**Estensione protocollo da valutare:** aggiungere campi senza rimuovere gli output attuali e senza cambiare gli identificativi esistenti dei tool; la lettura da client terzi deve comunque fare una chiamata esplicita per ogni pagina. Stabilire una compatibilità misurabile con le versioni MCP dichiarate in `docs/MCP.md`.

## 6. Offline, sicurezza/privacy, impostazioni e multilingua

- **Offline-first:** tutto avviene nel processo locale e nel SQLite locale; nessun indice remoto, monitoraggio, sincronizzazione o dipendenza SaaS. L'app funziona identica senza aver collegato alcun client MCP.
- **Privacy:** non aumentare campi sensibili restituiti e **non presumere** che un limite di 50/200 record impedisca a un assistente autorizzato di leggere più pagine. La scelta di pagina non equivale a un nuovo consenso: le decisioni sulla visibilità dei dati appartengono a TECH-014. Nessun invio implicito al provider; chiarire che il client collegato potrebbe trasmettere gli output fuori dal PC.
- **Sicurezza e risorse:** limiti coerenti per input/JSON, esecuzione e risposta; niente stampa di query/risultati in log, errori o diagnostica. Valutare annullamento e interruzione pulita, evitando lock prolungati, letture di archivi non selezionati, creazione di file temporanei non autorizzata o consumo incontrollato di RAM.
- **Impostazioni:** inizialmente nessuna obbligatoria; eventuali preferenze avanzate solo con motivazione verificata, valori prudenti e ripristino predefiniti. Non creare credenziali, porte o servizio di background.
- **Multilingua e accessibilità:** spiegazioni e messaggi dell'UI nella lingua di sistema, fallback inglese, scelta manuale dove disponibile; **codici e nomi di campi del protocollo restano stabili** per compatibilità internazionale. Annunci di errore leggibili e navigabili da tastiera, senza colore come unico indizio.
- **Import/export e backup:** questa proposta **non** introduce importazioni o esportazioni di registri e non cambia i CSV né il formato di backup verificato. I dati rimangono sempre disponibili tramite esportazioni complete e backup su cartella liberamente scelta. Eventuali indici/cache derivati non devono contaminare il backup come dato autoritativo né impedire ripristino indipendente.
- **Compatibilità:** Windows, macOS e Linux; leggere SQLite con schema corrente come ora, senza aprire in scrittura. Testare app e server MCP contemporanei, client con e senza `cursor` e versioni dichiarate del protocollo.

## 7. Alternative, dipendenze, rischi e complessità

**Alternativa zero:** mantenere i limiti dei risultati e documentare che i tool danno solo la prima pagina; raccomandare i CSV per analisi grandi. **Alternativa minima:** eliminare la collezione completa dei certificati e imporre limiti espliciti agli input, mantenendo gli altri elenchi non paginabili. **Alternativa intermedia:** cursori per gli elenchi senza cambiare la ricerca testuale; lasciare i conteggi esatti finché misurati. **Alternativa più onerosa:** indice di ricerca locale derivato con semantica equivalente, dopo prove reali.

**Dipendenze note:** `rusqlite`, `serde_json`, schema SQLite esistente e protocollo MCP STDIO; non è approvata alcuna nuova dipendenza, migrazione o libreria. Per un eventuale indice FTS, usare solo se la disponibilità e i tokenizer dell'SQLite distribuito sono compatibili su tutti gli OS.

**Rischi:** modifiche concorrenti fra pagine, token non validi, comparazione di stringhe e accenti, instabilità negli ordinamenti con chiavi identiche, regressioni nei conteggi, errori nell'arrotondamento di importi (che devono restare in centesimi interi), richieste ripetute che impattano la UI, quantità di dati inviata a provider IA esterni, aspettative sbagliate di snapshot, compatibilità con host già collegati. Complessità: **bassa-media** per misurazioni/limiti/conservazione memoria dei certificati; **media-alta** per cursori robusti e ricerca indicizzata semanticamente equivalente.

## 8. Test e criteri di accettazione ipotetici (non eseguiti)

1. Con i dataset 100/10.000/100.000, misurare prima/dopo picco memoria, CPU, tempi e righe esplorate; i limiti prestazionali vanno approvati **dopo** una baseline, non inventati qui.
2. `search_artworks` conserva risultati di query multi-campo, virgolette, accenti e filtro stato; il numero di risultati attuale non cambia per dataset e query identici.
3. Le chiamate senza `cursor` mantengono i parametri e i campi conosciuti. Client vecchi non devono dover inviare pagine nuove per ottenere ciò che già ricevevano.
4. Richiedendo pagine successive in archivio invariato, ogni UUID compare una sola volta e l'unione coincide con l'elenco ordinato atteso, anche quando più record hanno la stessa data/titolo.
5. Con un archivio modificato fra pagine, nessuna promessa di snapshot: o comportamento live chiaramente documentato oppure rifiuto esplicito e ripartenza; non corrompere il DB.
6. Cursor di altro archivio, query diversa, versione incompatibile, stringa eccessiva e input JSON oltre la soglia → errori prevedibili, senza perdita di dati e senza eco di dati privati.
7. `list_certificates` con molti record non richiede in memoria un `Vec` dell'intero archivio per la sola pagina richiesta; verificare con profiling.
8. Le prove con SQLite aperto dall'app non bloccano a lungo salvataggi leciti. Nessuna scrittura MCP, rete, log personale o file cache autoritativo.
9. Su Windows/macOS/Linux e client MCP selezionati: test manuali di compatibilità, lingua sistema/fallback inglese nei messaggi UI, navigazione delle eventuali schermate informative da tastiera.

## 9. Fasi di possibile valutazione

- **Fase 0 — Misura e domanda:** ricavare (con consenso e senza telemetria) ordini di grandezza dagli utenti; benchmark sintetici e verifica degli host MCP. Si può chiudere la proposta qui se non emerge il problema.
- **Fase 1 — Robustezza minima:** valutare streaming dei certificati e limiti di input/contenuti, errori di risorsa, senza cambiare i tool pubblici.
- **Fase 2 — Consultazione continuabile facoltativa:** contratti con cursori ed esiti verificabili, test di compatibilità e semantica degli ordinamenti.
- **Fase 3 — Ottimizzazione misurata:** solo se il benchmark indica latenza problematica, confronto query SQL/FTS con `fold` esistente e indici opzionali ricostruibili.

## 10. Questioni aperte riservate al proprietario

1. Quante opere, vendite e certificati contiene realisticamente un archivio da supportare: centinaia, migliaia o centinaia di migliaia?
2. È importante poter leggere **tutte** le pagine tramite assistenti o per grandi quantità di dati sono preferibili CSV esportati localmente dall'utente?
3. I conteggi esatti a ogni chiamata sono necessari, oppure una modalità a costo ridotto sarebbe utile purché esplicita?
4. Per archivi che cambiano durante l'esplorazione, è preferibile mostrare dati live con limiti dichiarati o fermare la continuazione?
5. Quali host MCP concreti devono essere collaudati per primi e quanto sarebbe accettabile l'aumento di complessità dell'interfaccia?
6. L'eventuale pagina aggiuntiva espone più dati personali di quanto il proprietario desideri permettere per default? Da coordinare con TECH-014 prima di qualsiasi decisione.

**Sintesi di stato:** analisi statica e ipotesi di progettazione, **non** bug riprodotto o richiesta approvata. Nessuna implementazione, modifica a codice/test/build/workflow/configurazioni/dipendenze, issue o PR di sviluppo; nessuna decisione assunta al posto del proprietario.
