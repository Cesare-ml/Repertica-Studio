# REPS-TECH-048 — Ricerca delle opere coerente fra interfaccia desktop e MCP locale

- **ID stabile:** REPS-TECH-048
- **Data della ricognizione:** 2026-11-22
- **Stato:** **Proposta — da valutare**
- **Priorità proposta:** **P2 condizionata** a divergenze riproducibili e all'uso reale del MCP; **P3 o nessuno sviluppo** se è sufficiente documentare le differenze.
- **Ambito:** contratto tecnico e verifica della *semantica della ricerca di opere* nei due percorsi già presenti, Angular e MCP STDIO read-only. Nessuna decisione approvata o assegnata allo sviluppo.

## 1. Motivazione, destinatari e riscontro sul repository

Un artista cerca «L’atelier 30,5 × 40 cm» nell'archivio dell'app e poi domanda al proprio assistente collegato tramite MCP di trovare la stessa opera. Risposte diverse alla **medesima ricerca, con lo stesso archivio e le stesse condizioni**, possono fare apparire un'opera assente o suggerire conclusioni inesatte. Per chi cataloga titoli con apostrofi tipografici, accenti o misure, la coerenza è più importante di nuove opzioni di ricerca.

Destinatari potenziali: artisti che usano la ricerca quotidiana, archivisti e gallerie che consultano il catalogo tramite assistenti locali/esterni, professionisti con titoli internazionali o misure rilevanti. **Non risulta una richiesta utenti convalidata** né un confronto end-to-end già eseguito; è un'opportunità tecnica circoscritta, verificabile con dati sintetici.

**Fatti ricavati dalla lettura di `main` in data 2026-11-22:**

- [Ricerca Angular](../../../src/app/archive-filters.ts): `foldSearch` applica normalizzazione Unicode **NFD**, rimozione dei combining marks, minuscole, equivalenza fra apostrofi e virgolette tipografiche/comuni, trattini tipografici, virgola decimale/punto e dimensioni espresse come «30,5 × 40 cm». `filterArtworks` cerca in più **campi distinti**, includendo rappresentazioni delle dimensioni.
- [Ricerca MCP Rust](../../../src-tauri/src/mcp.rs): `fold` trasforma in minuscole e sostituisce un insieme elencato di lettere latine accentate, **senza normalizzazione Unicode generale** né le equivalenze di punteggiatura/misura citate; `search_artworks` costruisce un testo con codice, titolo, anno, tecnica, supporto, descrizione e acquirente, **ma non aggiunge le misure al testo ricercabile**, pur restituendole nella risposta.
- Entrambi interpretano termini separati come AND e parole fra virgolette come frase; le differenze sopra indicate restano possibili anche quando le due API leggono gli stessi record. Esempi da verificare con fixture sintetiche: titolo con apostrofo curvo U+2019 versus interrogazione con apostrofo ASCII; testo memorizzato come `e + U+0301` versus query senza accento; query di dimensioni `30,5 × 40 cm`.
- [MCP.md](../../MCP.md) documenta un server già disponibile, in sola lettura, su STDIO, che **esclude** recapiti e fotografie, ma può esporre nomi acquirenti e importi; [README](../../../README.md) descrive ricerche accent-insensitive nell'archivio. Gli attuali test [Angular](../../../tests/archive-filters.test.mjs) e Rust in `mcp.rs` verificano casi nei rispettivi percorsi, non una **matrice comune** di equivalenza cross-runtime.
- La semantica dei **collezionisti non deve diventare identica**: l'interfaccia può ricercare indirizzi/contatti/annotazioni private, mentre `list_collectors` MCP cerca solo il nome. Questa differenza è una salvaguardia intenzionale di privacy, non un difetto da correggere.

Il confronto è **statico**, non una dimostrazione che l'app installata produca già un errore su casi reali. Il codice può evolvere: verificare nuovamente il branch principale prima di qualsiasi decisione.

## 2. Confini e controllo dei doppioni

- [TECH-008](REPS-TECH-008-viste-salvate-filtri-ricerca-locale.md) propone **nuovi filtri e viste salvate** nell'app; qui si analizza soltanto la **coerenza del comportamento della ricerca libera già esistente** tra due implementazioni. Non si aggiungono filtri.
- [TECH-036](REPS-TECH-036-mcp-archivi-grandi-paginazione-limiti-risorse.md) valuta **paginazione e limiti di risorse** MCP; qui si definiscono casi di equivalenza dei match, senza promettere di ricevere oltre i primi 50 risultati né cambiare limiti.
- [TECH-014](REPS-TECH-014-bridge-mcp-locale-lettura-permessi.md) studia consensi/ambiti MCP e resta autorevole per i confini dei dati esposti: **nessuna parità di ricerca autorizza la lettura di nuovi campi**. [TECH-020](REPS-TECH-020-accessibilita-flussi-critici-tastiera-screen-reader.md) riguarda UX accessibile; [TECH-030](REPS-TECH-030-vocabolari-locali-tecniche-supporti-interoperabili.md) i vocabolari professionali, non l'algoritmo di ricerca.
- [PR #14 — Gestione e UX](https://github.com/Cesare-ml/Repertica-Studio/pull/14), esaminata fino a **REPS-UX-052**, tratta catalogazione, commercializzazione, codici storici (UX-048), contesti di vendita e filtri d'uso. Non presenta una specifica di parità di **normalizzazione Unicode/semantica di match Angular–MCP**. Nessun suo file è interessato.
- Non si propone un secondo motore di ricerca, FTS obbligatorio, motore semantico/IA, invio dei dati a servizi esterni, nuovo endpoint HTTP, telemetria o riscrittura degli originali testuali.

## 3. Scenari e flussi UX, soltanto descrittivi

1. **Titolo e apostrofi:** l'utente cerca in Repertica «L'atelier» e trova l'opera catalogata «L’atelier». Ripete il testo in un client MCP: l'eventuale funzione di ricerca dovrebbe produrre la stessa corrispondenza *nei campi condivisi*, senza riscrivere il titolo nel database.
2. **Composizione Unicode:** un titolo importato contiene «Cafe» più segno combinante di accento, non la singola lettera precomposta. Query senza accento: stessa identificazione sui due percorsi o segnalazione documentata dell'eccezione.
3. **Misure:** la ricerca desktop interpreta «30,5 × 40 cm»; in MCP il campo misura oggi è nella risposta ma **non nel testo cercabile**. Prima di allargare il match MCP, verificare con utenti se tale ricerca è effettivamente necessaria e se il contratto delle misure risulta chiaro.
4. **Frasi e AND:** `"olio su tela" marina` può cercare la frase **dentro un singolo campo**, più «marina» in un altro. Testare esplicitamente che una frase non sia creata attraversando il confine fra due campi, evitando falsi positivi.
5. **Archivio con più di 50 risultati:** la UI può mostrare l'intera lista; MCP limita il numero restituito. Il confronto si effettua sul **predicato di appartenenza** o sul conteggio totale a condizioni uguali, non sulla sola lunghezza delle due liste; la continuazione delle pagine appartiene a TECH-036.
6. **Differenze deliberate:** l'assistente non trova un collezionista per indirizzo privato, benché l'interfaccia possa farlo. Il messaggio di documentazione deve indicare che la differenza protegge i dati; **mai** ampliare l'MCP per ottenere falsa parità.
7. **Client IA non controllabile:** un assistente potrebbe riformulare automaticamente una domanda. La garanzia riguarda la **stessa query effettivamente inviata** al tool, non la qualità della risposta in linguaggio naturale generata dal modello.

**Flusso minimo di verifica, non una nuova schermata obbligatoria:** selezionare un archivio sintetico senza dati personali → inserire la stessa stringa di ricerca nei due percorsi → confrontare codici/UUID e condizioni → annotare differenze intenzionali e non intenzionali → decidere se basta migliorare la guida o se vale la pena uniformare parte della normalizzazione. Eventuale spiegazione nelle Impostazioni → Assistenti IA locali: «L'assistente consulta solo alcuni dati dell'archivio. I suoi risultati possono differire dai filtri dell'app». Nessuna sincronizzazione o azione automatica.

## 4. Specifica funzionale proposta: contratto di ricerca e compatibilità

**Invarianti certi:** il database e i testi delle opere restano originali; UUID e codici non cambiano; nessuna ricerca scrive SQLite; MCP continua a usare `SQLITE_OPEN_READ_ONLY` e `query_only`, STDIO e campi attualmente consentiti. Nessun account o abbonamento. Le funzionalità normali dell'app non devono dipendere dall'attivazione di un client MCP.

**Ipotesi da validare prima di sviluppare:**

- Un **contratto documentato di normalizzazione** per la ricerca libera delle opere, con regole esplicite su maiuscole, accenti precomposti/decomposti, apostrofi, virgolette, trattini, spazi, numeri decimali, simbolo di moltiplicazione e dimensioni. Non promettere copertura universale di tutti i sistemi di scrittura senza test.
- Un **insieme di campi comuni** (codice, titolo, anno, tecnica, supporto, descrizione e, dove consentito, nome acquirente) per cui confrontare i risultati. Misure: aggiunta al contratto solo se utile; nessun accesso MCP a note private, email, telefono, indirizzo, percorsi o file.
- **Parole non citate**: AND tra termini anche di campi diversi; **frase citata**: stringa contigua in un singolo campo normalizzato, non somma inventata di campi. Mai creare un indicatore che la frase provi una provenienza, vendita o attribuzione.
- **Gestione regressioni:** se una modifica fa comparire più opere in risposta MCP, valutarne effetti su client esistenti e minimizzazione dei dati. Eventuali nuove regole andrebbero documentate, testate e introdotte senza cambiare la forma JSON dei tool esistenti.
- **Ordine e limiti separati dal match:** gli ordinamenti UI/MCP non devono diventare identici per garantire la stessa appartenenza al set; parità di top-N non è promessa senza ordinamento e paginazione condivisi.
- **Performance:** prima fissare limiti o scegliere una libreria Unicode, misurare latenza e memoria con input realistici. Normalizzare la query e i campi solo nella memoria del processo; preferire riuso di fixture/utility del progetto rispetto a librerie aggiunte senza prova di necessità.

### Dati e API ipotetici — NON implementazione

Nessuna nuova tabella SQLite è richiesta per la **fase di specifica/test**. Una fixture di verifica potrebbe descrivere:

~~~text
SearchCase {
  case_id: string,
  query: string,
  shared_field_fixture: { artwork_uuid, code, title, year?, technique?, support?, description?, dimensions?, buyer? },
  expected_matching_artwork_uuids: string[],
  scope: "common_artwork_fields" | "ui_only" | "mcp_privacy_exclusion",
  reason: string
}
compareSearchMembership(case, uiResults, mcpResults)
  -> { equivalent: boolean, onlyUi: UUID[], onlyMcp: UUID[], expectedExceptions: string[] }
~~~

La struttura è soltanto **documentale**. Non includere mai nelle fixture dati di collezionisti reali. Possibili superfici future, da approvare separatamente: un `search_contract_version` **informativo** nella documentazione MCP oppure un pacchetto di casi di test condiviso TS/Rust; **non** una nuova API di rete o un campo persisted in SQLite. L'eventuale funzione `search_artworks(query, status, limit)` già presente rimane compatibile.

## 5. Offline, privacy, sicurezza, impostazioni, lingue e portabilità

- Tutta la verifica e l'eventuale uniformazione dovrebbero funzionare **offline** su Tauri/Angular/Rust e database SQLite locali. Nessuna indicizzazione cloud, invio delle query a terzi, logging dei nomi cercati, storico delle ricerche o token remoto.
- Il client MCP è **un soggetto esterno**: ciò che l'app rende consultabile può essere inviato online dall'assistente secondo le sue regole. Spiegare questo rischio senza insinuare che Repertica effettui upload in autonomia. Nessuna modifica a consensi, ambiti o file di configurazione dei client.
- Nessun dato nuovo necessario per svolgere ricerche; l'eventuale contratto/fixture è documentazione o parte della suite di test **solo se il proprietario decidesse** di implementarla. Export CSV integrale e backup SQLite + media nella cartella liberamente scelta restano invariati.
- La UI usa **lingua di sistema con fallback inglese**, con scelta manuale ove già prevista; eventuali testi informativi devono essere traducibili in italiano, inglese e lingue supportate, senza alterare titoli e descrizioni originali. Normalizzazione per il *matching* non equivale a traduzione, traslitterazione universale o correzione ortografica.
- Accessibilità: qualsiasi avviso su differenze di copertura deve essere leggibile da screen reader e tastiera, non dipendere solo dal colore, e non presentare un «nessun risultato MCP» come prova che l'opera manchi dall'archivio.
- Sicurezza: proteggere la memoria/tempo di elaborazione da query patologiche; le regole di limiti generali sono però già studiate in TECH-036. Evitare di aggiungere al MCP ricerca su campi personali soltanto per simulare uguaglianza di risultati.

## 6. Alternative, dipendenze e rischi

| Alternativa | Vantaggio | Limite/decisione |
| --- | --- | --- |
| **A. Nessuna modifica del software: documentare differenze** | Nessuna dipendenza o regressione; appropriata se uso MCP limitato | Query apparentemente uguali continueranno a produrre esiti diversi |
| **B. Contratto di fixture condivise e allineamento minimo** | Intervento potenzialmente circoscritto; tutela i casi realmente frequenti | TS e Rust implementano Unicode in modo distinto; servono test incrociati e verifica privacy |
| **C. Motore di ricerca unico o nuovo indice persistente** | Possibile maggiore uniformità architetturale | Molto più complesso, con migrazioni, performance e rischi; **non giustificato come prima scelta** |

**Dipendenze da valutare:** test Angular esistenti e test Rust MCP, regole UI su Unicode e misure, possibile libreria Unicode Rust solo dopo inventario delle librerie già disponibili; verifica degli effetti su client MCP e sui limiti di risposta. Nessuna dipendenza viene scelta o installata da questa scheda.

**Rischi:** cambio dei risultati attesi dai client; falsi positivi se si normalizzano eccessivamente punteggiatura o numeri; aumento dei tempi di scansione su cataloghi molto ampi; falsa promessa di parità quando filtri, quantità di risultati o campi ammessi divergono intenzionalmente; sovraesposizione dei dati se si volesse uniformare il solo contenuto degli output. La privacy prevale sull'equivalenza apparente.

**Complessità stimata, non impegnativa:** **bassa** per specifica e matrice di test, **media** per parità selettiva TS/Rust e regressioni, **alta** per motore condiviso o nuova indicizzazione (sconsigliati senza evidenze). Non sono stati eseguiti benchmark o test in questa ricognizione.

## 7. Criteri di accettazione ipotetici

1. Esiste una **matrice di casi anonimizzati** con testo NFC/NFD, accenti, apostrofi tipografici e semplici, virgolette, trattini, virgola/punto decimali, «×/x», dimensioni, query AND e frasi; ogni caso dichiara quali differenze sono intenzionali.
2. Per lo stesso snapshot/archivio e il medesimo filtro stato, i due percorsi concordano sull'**insieme degli UUID corrispondenti per i campi comuni**, oppure producono eccezioni documentate; il confronto non si basa sul solo primo blocco da 50 risultati MCP.
3. Una frase in doppie virgolette non è verificata artificialmente da testi provenienti da due campi distinti; casi con testo vuoto, accentazione decomposta e dati opzionali non producono crash.
4. **Nessun aumento dei campi privati MCP:** email, telefono, indirizzo, note dei collezionisti, file e percorsi media restano esclusi; i test di riservatezza preesistenti continuano a valere.
5. Nessuna modifica ai dati originali o certificati; esportazioni e backup/ripristino restano integri e pienamente locali; la UI non richiede rete o account.
6. Lingua del sistema/fallback inglese e accessibilità degli eventuali avvisi verificate; titoli originali non vengono modificati o tradotti.
7. Su una fixture sintetica con molte opere non emergono regressioni prestazionali sproporzionate, valutate con soglie **decise dopo misura** (non inventate ora).
8. Nessun intervento necessario se gli utenti non osservano il problema o accettano esplicitamente le differenze documentate.

## 8. Fasi eventuali e decisioni aperte

- **Fase 0 — convalida senza sviluppo:** raccogliere 5–10 query anonime reali da artisti/gallerie, riprodurre su fixture sintetiche le divergenze descritte, confrontare le regole e la documentazione attuale. Se il bisogno non è materiale, chiudere la valutazione senza implementare.
- **Fase 1 — specifica e QA proposto:** decidere un piccolo contratto di ricerca *per i soli campi comuni* e una matrice di fixture/versioni, mantenendo esclusioni privacy e API già esistenti.
- **Fase 2 — eventuale allineamento minimo:** solo se approvato e giustificato, valutare le poche equivalenze più utili e collaudare anche con client MCP reali e input multilingue.
- **Fase 3 — solo con misure:** riconsiderare prestazioni e possibili primitive condivise; coordinarsi con TECH-008/036 senza trasformare questo studio in filtri avanzati o paginazione.

**Domande per la revisione del proprietario:**

1. Gli artisti usano oggi davvero il MCP per ritrovare opere, o la funzione è soprattutto dimostrativa?
2. Quali fra titoli accentati, apostrofi, trattini e **ricerca per dimensioni** ricorrono nei cataloghi effettivi? Serve la stessa sintassi fra interfaccia e assistente?
3. Preferiresti dichiarare esplicitamente le differenze dei due percorsi oppure una parità limitata ai campi comuni?
4. Vale la pena sostenere una dipendenza Unicode aggiuntiva per casi che non sono ancora riprodotti su desktop reali?
5. Quali lingue e sistemi di scrittura vanno verificati oltre a italiano, inglese, francese, spagnolo e tedesco?

**Esito della ricognizione:** una sola **proposta da valutare**, senza approvazione, incarico, implementazione o modifica di codice/test/build/workflow/configurazioni/dipendenze. Il branch documentale e la PR rimangono in bozza per revisione personale.
