# REPS-TECH-014 — Bridge MCP locale in sola lettura, dati minimizzati e autorizzazioni revocabili

- **ID stabile:** REPS-TECH-014
- **Data:** 2026-11-21
- **Stato:** **Proposta — da valutare**
- **Priorità ipotetica:** **P1–P2**, da validare sulla domanda reale e sulla fattibilità del modello di permessi; **non approvata né assegnata allo sviluppo**.
- **Ambito:** progettazione tecnica dell'integrazione facoltativa MCP, **non** un'implementazione, né un servizio IA incluso o un account obbligatorio.

## Aggiornamento di verifica — 22 novembre 2026: il bridge di base esiste già

**Questa sezione prevale sulle osservazioni storiche del 21 novembre presenti più sotto.** La scheda mantiene ID e stato **«Proposta — da valutare»** perché **non** è un incarico di implementazione né una dichiarazione di completamento delle garanzie proposte.

- **Ora implementato su `main`:** [`src-tauri/src/mcp.rs`](../../../src-tauri/src/mcp.rs) include un server MCP avviabile tramite `--mcp`, con trasporto STDIO, connessione SQLite `SQLITE_OPEN_READ_ONLY`, `PRAGMA query_only` e sei strumenti di sola lettura: overview, ricerca/dettaglio opere, elenchi collezionisti, vendite e certificati. [`docs/MCP.md`](../../MCP.md) ne documenta l'uso; [`src-tauri/src/mcp_client.rs`](../../../src-tauri/src/mcp_client.rs) prepara le configurazioni di client esterni supportati e permette di rimuovere tali configurazioni. La funzione non va più descritta come «ancora non presente».
- **Da valutare, non verificato come implementato:** non emerge dal percorso di dispatch MCP esaminato una politica di **consenso per singolo client, strumento o campo**, una scadenza di concessione o una revoca applicata dal backend a un processo MCP già avviabile. Il comando `--mcp --data-dir` avvia l'accesso al database consentito dai privilegi del sistema operativo. Rimuovere una voce dalla configurazione di un assistente **non equivale** a vietare a ogni processo locale di avviare il server. Queste sono differenze progettuali rispetto alla proposta originaria, **non** prove di uso improprio o fuga dati.
- **Precisione privacy:** i contatti (email/telefono/indirizzo), note private e file fotografici non risultano fra i campi restituiti dai sei strumenti esaminati; sono invece accessibili **nominativi dei collezionisti e dati commerciali** (compratore, prezzo, incassato, residuo e totali), inclusi in alcune risposte del server. Non chiamarli «solo dati pubblici». [`docs/MCP.md`](../../MCP.md) avverte correttamente che un client IA esterno può trasmettere i risultati al proprio provider, anche se Repertica non invia richieste di rete.
- **Decisione per il proprietario:** il valore di una **modalità catalogo minimo**, di profili di consenso o della rinuncia al collegamento in ambienti condivisi va discusso alla luce dell'MCP **già disponibile**, senza duplicarne l'implementazione. Una prima misura potrebbe essere soltanto una presentazione molto chiara dei campi oggi esposti. Eventuali policy e nuovi tool richiedono analisi del modello di minaccia e collaudi su Windows/macOS/Linux.
- **Confini:** lo scopo restante della presente scheda è studiare *minimizzazione dei campi, controllo dell'accesso e revoca effettiva*, non proporre un secondo server MCP. Nessun bug di sicurezza sfruttato è stato osservato. Priorità P1–P2 resta **ipotetica**.

---

## 1. Evidenza, motivazione, destinatari e perimetro

**L'esigenza non è nuova:** [issue #3](https://github.com/Cesare-ml/Repertica-Studio/issues/3) già richiede, come obiettivo futuro, un server MCP locale tramite **STDIO**, senza porte di rete, con strumenti read-only limitati (ricerca/dettaglio opere, eventuale elenco collezionisti), distribuito con l'app senza obbligo di installare Node. [docs/MVP.md](../../MVP.md) precisa che l'MCP **non è presente nella build iniziale** e prevede una futura integrazione locale. Questa scheda **approfondisce le scelte di sicurezza, consenso e compatibilità dell'issue**, senza duplicarla come nuova decisione di prodotto.

**Riscontri su `main` del 21/11/2026:** [`src-tauri/src/lib.rs`](../../../src-tauri/src/lib.rs) registra comandi Tauri sia di lettura sia di **scrittura** (opere, collezionisti, vendite, certificati, incassi, file, backup/ripristino); [`src/app/api.ts`](../../../src/app/api.ts) li invoca dall'interfaccia; [`src-tauri/capabilities/default.json`](../../../src-tauri/capabilities/default.json) limita le capacità della finestra principale, **ma non è una politica di autorizzazione per processi MCP esterni**. [README](../../../README.md) documenta SQLite e media locali e sette esportazioni CSV; attualmente non risultano un server MCP né un processo che esponga una API remota. La documentazione non garantisce la protezione contro un malware locale con gli stessi privilegi OS.

**Problema concreto da verificare:** un archivista può voler interrogare rapidamente il catalogo da un assistente o da un'applicazione compatibile senza copiare manualmente dati e senza fornire al programma esterno accesso indiscriminato a SQLite, ai file media o ai recapiti dei collezionisti. **Read-only significa non modificare l'archivio; non significa che la lettura sia priva di rischio privacy.** Un host IA, anche collegato a un bridge locale, **può inviare ciò che legge a servizi online** secondo la propria configurazione: Repertica non deve promettere un trattamento esclusivamente locale da parte di software terzo.

**Destinatari:** artisti, gallerie, archivi, assistenti di studio e professionisti con cataloghi estesi; utile anche a strumenti accessibili di ricerca o analisi **soltanto se** l'interoperabilità con MCP viene effettivamente richiesta. Per l'utente che non utilizza client MCP, Repertica rimane completamente funzionale senza configurazioni aggiuntive.

**Confini anti-duplicazione:**
- [TECH-002](REPS-TECH-002-automazioni-locali-consapevoli.md) propone regole interne autorizzate avviate **da Repertica**; qui un *client esterno* chiede letture. Non condividere permessi implicitamente tra i due sistemi.
- [TECH-001](REPS-TECH-001-pacchetto-dati-aperto.md) descrive export/import completo; qui si consulta solo un insieme ristretto di record correnti senza creare pacchetti.
- [TECH-004](REPS-TECH-004-ricerca-federata-archivi-locali.md) consulta backup scelti; l'eventuale MCP iniziale legge **un solo archivio attivo**, senza federazione.
- [TECH-008](REPS-TECH-008-viste-salvate-filtri-ricerca-locale.md) riguarda filtri e viste **nell'app**, non l'esposizione a terze parti.
- [PR #14, filone Gestione e UX](https://github.com/Cesare-ml/Repertica-Studio/pull/14): non si progettano nuovi processi gestionali, CRM, vendite, dossier o pubblicazioni.

## 2. Scenari realistici

1. **Catalogo per una domanda puntuale:** l'artista abilita temporaneamente il collegamento e un client autorizzato chiede «quali opere ad acquerello del 2024 sono in archivio?». L'output mostra codice, titolo, anno, tecnica e stato, **non** nome acquirente, ubicazione privata, foto o note libere per impostazione predefinita.
2. **Consultazione di una singola opera:** dopo una ricerca l'assistente apre il dettaglio per UUID, con il medesimo insieme minimo di campi. Un ID inesistente o un UUID malformato non fa emergere percorsi o dati di altre opere.
3. **Anagrafica strettamente autorizzata (fase successiva, da validare):** l'utente abilita esplicitamente `list_collectors`, scegliendo campi e durata. Il semplice «collezionisti» **non** include automaticamente email, telefono, indirizzo, note o cronologia acquisti.
4. **Revoca e chiusura:** una sessione scade, l'utente la revoca o l'archivio viene cambiato/ripristinato; le chiamate successive falliscono **senza** tornare a concessioni permanenti e senza utilizzare risultati di un altro archivio.
5. **Client non attendibile:** un client ignoto tenta di usare nomi di tool non previsti, recuperare tutte le tabelle, leggere file arbitrari o chiedere un export CSV integrale. Deve ricevere un rifiuto senza eseguire alcuna scrittura.

## 3. Flusso UX proposto (solo descrittivo)

**A. Attivazione volontaria:** Impostazioni → Integrazioni locali → «Collegamento MCP (sperimentale)». Disabilitato inizialmente. Breve spiegazione: «Consente a programmi esterni di leggere i dati che autorizzi. Il programma esterno potrebbe elaborarli anche online». Pulsante **«Mostra dati accessibili»** con anteprima esatta, non solo una formula generica di consenso.

**B. Concessione ristretta:** scelta dell'archivio attualmente aperto; ambito iniziale «opere: campi essenziali»; opzionalmente singole categorie aggiuntive, **mai** selezionate di default; durata limitata e revoca immediata. La UI nomina chiaramente il client configurato, ma deve avvertire che **l'identità dichiarata dal client non è autenticazione sufficiente**.

**C. Collegamento del client:** istruzioni locali e copiabili per host MCP compatibili, per sistema operativo, con percorso del componente già incluso nell'installer: nessun Node o SaaS necessario. **Ipotesi da verificare:** bridge/companion Rust installato insieme all'app, avviato dal client come sottoprocesso STDIO. Non esiste un server HTTP in ascolto. Le configurazioni e i percorsi d'esempio non devono contenere credenziali a lunga durata o dati personali.

**D. Sessione e controllo:** prima di esporre dati il backend verifica ambito e validità del consenso. La UI mostra collegamento corrente, origine dichiarata (non verificata), strumenti concessi, ultima attività e azioni «Sospendi»/«Revoca». Non promettere finestre di conferma per ogni tool quando l'app è chiusa o un host non supporta richieste interattive: **se il consenso necessario non è verificabile, fallire in modo chiuso**.

**E. Spegnimento e revoca:** chiudere/scollegare il bridge e invalidare nuovi accessi. Se un client ha **già ricevuto** record, la revoca non può cancellare copie, log o memoria dell'host esterno: dirlo esplicitamente.

**Accessibilità:** sequenza tastiera completa; etichette screen reader; descrizioni testuali dei campi esposti e del rischio, non solo colore o icone; scorciatoia per revoca; messaggi di errore distinguibili e localizzati.

## 4. Contratto dati e API descrittivi, non implementati

Proposta di **allowlist separata dai comandi Tauri**: non inoltrare automaticamente il registro `invoke_handler` al client, non riutilizzare i comandi di scrittura come un proxy universale, non esporre `data_directory`, path di foto, `create_backup`, `restore_backup`, `export_ledger_csv`, comandi di vendita/incasso, certificati completi o query SQL libere.

Possibili strumenti, con argomenti tipizzati e limiti vincolanti sul server:
- `search_artworks({query?, year_from?, year_to?, technique?, status?, limit?, cursor?})` → risultati paginati: `id` UUID, `code`, `title`, `year`, `technique`, `status`; tutto il resto **escluso** salvo ambito esplicito.
- `get_artwork({id})` → dettaglio con campi **identici all'ambito approvato**, non il record SQLite o l'oggetto frontend completo; mai esporre `image_path` o note private per inerzia.
- `list_collectors({limit?, cursor?})` → **non nella fase 1**, attivabile solo dopo revisione privacy; lista eventualmente limitata a ID e nominativo, senza recapiti per default. La sola presenza di una vendita non crea consenso.
- `get_capabilities()` o metadata statici → strumenti e schema di output autorizzati senza conteggi che rivelino dati riservati.

**Contratti di risultato:** `schema_version` del **contratto MCP Repertica** distinto sia dalla versione del protocollo MCP sia da `PRAGMA user_version`; `archive_scope` pseudonimo locale non sensibile; campi stabili in inglese, date ISO/UTC quando pertinenti, importi eventuali in **centesimi interi** e valuta esplicita; null distinguibile. La UI localizza i messaggi, **non** nomi delle chiavi e identificatori.

**Policy locale ipotetica**, separata dai dati archivistici e dai backup:
`grant_id, archive_fingerprint, allowed_tools, allowed_fields, created_at, expires_at, revoked_at, consent_version, last_access_at, client_label_unverified`.
Non considerare UUID dell'opera, percorso filesystem o nome client come credenziali. Se un archivio cambia contesto, la concessione deve essere invalidata o rinegoziata; il solo path non è un identificatore sicuro. Non depositare token segreti nei documenti di configurazione del client o negli argomenti della riga di comando.

**Limiti indicativi da misurare, non valori approvati:** `limit` predefinito 20, massimo 50; massimo lunghezza query 200 caratteri, timeout e budget risultati per sessione; paginazione non enumerabile oltre lo scope autorizzato; niente wildcard che conceda accesso all'intero DB. La cancellazione del client interrompe il lavoro, con comportamento definito in presenza di letture lunghe.

## 5. Architetture alternative e fattibilità di STDIO

**Opzione A — broker applicativo + helper STDIO (più controllabile ma complessa):** il client lancia un piccolo binario locale; questo parla con l'app, **quando è aperta**, su un IPC OS con permessi minimizzati. La policy e la revoca sono applicate dal backend Rust; l'app può mostrare stato e consenso. Se l'app è chiusa, il collegamento non apre il database autonomamente e fallisce in modo comprensibile. Verificare come proteggere pipe/socket, autorizzazione e lifecycle su Windows/macOS/Linux **senza aprire porte TCP**.

**Opzione B — helper STDIO autonomo (più semplice da distribuire, difficile per la revoca):** il client lancia il componente che apre una connessione SQLite in **sola lettura**, senza migrazioni, e consulta policy locali già approvate. Va verificato come gestire WAL, lock, schema futuro, rollback, cambio archivio, grant scaduti e avvio mentre l'app è chiusa. Senza un'autorità in esecuzione, la conferma interattiva immediata non è affidabile. Non assumere questa opzione sicura senza un threat model e test sul filesystem reale.

Il trasporto **STDIO** MCP è un sottoprocesso lanciato dal client con messaggi JSON-RPC sui flussi standard; **stdout deve contenere solo traffico del protocollo** e le diagnostiche devono usare stderr (vedi [specifica MCP, revisione 2026-07-28, trasporti](https://modelcontextprotocol.io/specification/2026-07-28/basic/transports)). Il protocollo cambia nel tempo: verificare negoziazione/versioni e compatibilità di host reali senza legarsi a un SDK JavaScript o a installazioni globali. Le **annotazioni `readOnlyHint` sono metadati informativi, non un meccanismo di sicurezza** ([analisi del progetto MCP, marzo 2026](https://blog.modelcontextprotocol.io/posts/2026-03-16-tool-annotations/)); i limiti vanno applicati dal codice Rust e da policy verificabili.

## 6. Offline, privacy, sicurezza e minacce

- Nessuna connessione, apertura porta, discovery LAN, login, cloud o account **richiesti da Repertica**. L'utente può scegliere un host IA completamente offline; il bridge non impone né impedisce al **client terzo** di usare Internet. Mostrare questo confine prima dell'abilitazione.
- Default **disabilitato**, zero tool esposti senza una concessione verificata e **sola lettura in ogni circostanza**. Anche un client con scope consentito non può chiedere eliminazioni, modifiche, stampa certificati, backup/ripristini, upload, chiamate HTTP, shell o scrittura di file.
- **Separazione dei privilegi:** non basta nascondere i tool nella UI o fidarsi delle annotazioni MCP; applicare una allowlist nel servizio, validare input, privilegiare proiezioni SQL parametrizzate, connessioni in lettura e verifiche esplicite sul DB/archivio selezionato. Niente SQL definito dal client, path arbitrari, URI di file, estensioni SQLite, lettura automatica di media o di snapshot privati.
- **Prompt injection e catena di strumenti:** descrizioni, titoli e note di archivio sono **dati non attendibili**, non istruzioni del modello. Anche un tool locale read-only può essere combinato da un host con tool esterni per far uscire informazioni: la sola assenza di HTTP nel bridge **non** impedisce l'esfiltrazione tramite host.
- **Minimizzazione:** default senza email, telefoni, indirizzi, note, foto, ubicazioni esatte, prezzi/incassi o certificati. Le aggiunte di ambito devono essere separate, comprensibili e revocabili. Il catalogo stesso può essere riservato.
- **Log locali:** solo timestamp, tipo tool, esito, durata, campo/concessione; evitare argomenti di ricerca, nomi, email, risposte e token. Conservazione limitata e cancellabile; nessuna telemetria esterna.
- **Limite dell'isolamento:** un software già in esecuzione con gli stessi privilegi dell'utente potrebbe leggere direttamente i file SQLite non cifrati, indipendentemente dal bridge. Le policy MCP limitano **l'API di Repertica**, non costituiscono una sandbox universale contro programmi locali malevoli.
- Errori e interruzioni: SQLite occupato, app chiusa, percorso/USB non disponibile, timeout, grant revocato, schema non supportato, archivio sostituito → errori minimizzati, nessun fallback su DB precedenti o altra directory.

## 7. Impostazioni, multilingua, interoperabilità, import/export

**Impostazioni possibili:** interruttore globale off/on, scope opere essenziali, eventuali scope extra, scadenza di consenso, dispositivo/archivio, registro minimo degli accessi, revoca e reset. Nessuna necessità di login. L'installazione resta utilizzabile anche senza integrazione.

**Lingue:** lingua del sistema per impostazione predefinita e fallback **inglese**; opzioni esistenti per override manuale. Esporre privacy, errore, stato e revoca con traduzioni testabili; i campi MCP restano stabili per i client e i valori forniti dall'utente **non sono tradotti automaticamente**. Accessibilità da tastiera e screen reader obbligatoria prima di presentare l'integrazione come pronta.

**Import/export:** MCP read-only **non** reimporta né esporta automaticamente l'archivio. I sette CSV e il backup SQLite+media continuano ad essere i canali attuali; [TECH-001](REPS-TECH-001-pacchetto-dati-aperto.md) è il progetto separato di portabilità semantica. Grant, cache/telemetria e percorsi del client non devono contaminare gli export dei dati professionali; se vengono conservate preferenze trasferibili, documentare ciò che contiene la copia e chiedere consenso esplicito prima di esportarle.

**Compatibilità:** ipotizzare distribuzione del bridge binario con pacchetti Tauri per Windows, macOS e Linux; non promettere che tutti i client supportino la stessa versione MCP o la stessa configurazione STDIO. Verificare firma/distribuzione, aggiornamenti compatibili e annullamento installazione; non includere codice eseguibile di terze parti dentro l'archivio dati. Con UI e binario in versioni incompatibili, negare l'accesso invece di degradare le autorizzazioni.

## 8. Dipendenze, alternative, rischi, complessità

**Dipendenze tecniche ipotetiche:** interfaccia Rust read-only separata dai comandi Tauri mutanti; policy locale e persistenza consensi, collegamento IPC o lettura DB sicura; protocollo MCP STDIO versionato; paginazione e limiti; test con host reali; packaging del companion su tre OS. **Non** introdurre librerie, dipendenze o workflow in questa PR.

**Alternative da esaminare:**
1. **Nessuna integrazione MCP:** ricerca in UI e sette CSV già presenti; minima esposizione e minima manutenzione.
2. **Esportazione locale su richiesta:** file selezionato manualmente per un assistente, senza accesso continuo all'archivio; meno ergonomica, ma per casi sporadici forse sufficiente.
3. **Bridge STDIO read-only con app aperta (opzione A):** più controllabile e coerente con consenso revocabile; complessità cross-platform.
4. **Helper autonomo (opzione B):** meno passaggi quando l'app è chiusa, ma consensi e isolamento più difficili.
5. **Server HTTP locale:** **fuori dal primo perimetro**; amplifica superficie di rete e deployment, non necessario per l'obiettivo di issue #3.

**Rischi principali:** falsa impressione «dati sempre offline» quando si usa un host cloud; esposizione di contatti e dati commerciali; client che ignora o inventa la propria identità; tool generici che aggirano policy; accesso a un archivio sostituito; query costose; blocchi sul database in uso; log troppo dettagliati; manutenzione delle revisioni MCP e packaging; abuso del contenuto testuale come istruzioni. **Complessità stimata alta** per integrazione cross-platform con consenso realmente revocabile, **media** per un prototipo limitato e non distribuibile. La P1 dell'issue non è un'autorizzazione di sviluppo.

## 9. Criteri di accettazione e test futuri (nessun test scritto qui)

1. **Installazione pulita:** bridge disabilitato, zero grant, nessun listener TCP, nessun accesso a SQLite o media da un client non autorizzato; Repertica funziona senza host MCP.
2. **Scope minimo:** ricerca e dettaglio restano paginati, restituiscono solo campi concessi; test su opere vendute, record con descrizioni riservate, caratteri Unicode, query lunghe, UUID malformati e valori simili a prompt/SQL.
3. **Assenza di scritture:** snapshot hash/DB e test transazionali prima/dopo 1.000 chiamate; zero modifiche a opere, clienti, vendite, certificati, ricevute, backup, media e preferenze professionali; i soli audit/consensi app-owned sono distinti.
4. **Privacy:** senza consenso esplicito, `list_collectors`, recapiti, note, path locali, foto e numeri finanziari non sono leggibili neppure usando tool name inventati, cursor malevoli, argument injection o molti request in parallelo.
5. **Revoca:** scadenza, stop, cambio archivio, chiusura app (nell'opzione A), aggiornamento schema e policy cambiata bloccano immediatamente **nuove** letture, senza fallback permissivo. I dati già consegnati restano fuori dal controllo Repertica e l'UI lo comunica.
6. **Compatibilità:** client MCP reale con STDIO su Windows/macOS/Linux; version negotiation, stdout pulito, interruzione e cancellazione, timeout, app chiusa, due processi concorrenti, dischi esterni scollegati; nessuna installazione Node obbligatoria.
7. **Usabilità:** un non tecnico capisce quali dati verranno letti, completa la configurazione opzionale, vede ultima attività e revoca senza cercare file di sistema; interfaccia accessibile, stringhe IT/EN/ES/FR/DE e fallback EN.
8. **Rischio IA esterna:** il consenso avverte in modo inequivoco che l'host può trasferire dati in rete. Non si dichiara sicuro il trattamento di dati personali da parte di client terzi non valutati.

## 10. Certezze, ipotesi e questioni per il proprietario

**Certezze dal repository:** issue #3 esistente per MCP STDIO read-only; docs/MVP lo colloca dopo l'MVP; core Rust/Tauri con SQLite e comandi sia di lettura sia di scrittura; sette CSV e backup locali. Non si è trovato nel `main` esaminato un server MCP. Il vincolo software è gratuito/open source/offline-first senza account, con lingua di sistema e fallback inglese.

**Ipotesi progettuali, non fatti:** companion Rust distribuito con app; UI consensi e audit; scope predefinito ristretto; revoca collegata a broker; limiti numerici indicativi; supporto multi-client. Non è provato che gli artisti richiedano davvero la consultazione tramite IA più spesso di un export manuale.

**Questioni aperte da sottoporre al proprietario:**
- Esistono casi d'uso reali con Codex o client MCP **che giustifichino** il costo di packaging e sicurezza?
- Accettabile richiedere che **Repertica sia aperta** durante le sessioni, oppure è requisito fondamentale l'avvio da client con app chiusa?
- Esporre solo opere (con campi minimizzati) in fase 1, rinviando l'anagrafica clienti? I riferimenti alle vendite e le ubicazioni sono mai necessari?
- Consentire un grant a durata limitata senza approvazione per ogni chiamata, oppure solo consenso per sessione? Come deve funzionare la revoca sul sistema operativo reale?
- Quali host e revisioni MCP supportare per primi? È desiderabile supportare strumenti IA **interamente locali** come prima dimostrazione, senza imporne alcuno?
- Chi verifica che guide, installer e avvisi non inducano l'utente a credere che il client terzo non invii dati online?

## 11. Fasi ipotetiche di valutazione — NON roadmap approvata

1. **Interviste e matrice dei dati:** dimostrare almeno due scenari professionali che i filtri e i CSV attuali non risolvono comodamente; classificare campi e rischi.
2. **Threat model/decisione architetturale:** scegliere fra broker con app aperta e helper autonomo; formalizzare grant/revoca, archivi, permessi OS e limiti; prototipo *solo concettuale* finché non c'è approvazione.
3. **Eventuale fase iniziale, solo dopo approvazione:** tool `search_artworks`/`get_artwork` su campi essenziali, STDIO, UI opt-in, niente contatti; verifiche indipendenti.
4. **Eventuale estensione separata:** campi o elenchi collezionisti con permessi specifici; compatibilità più ampia e audit, mantenendo sempre un'alternativa senza MCP.

> **Nessun codice, dipendenza, processo, impostazione o permesso è stato implementato o autorizzato da questo documento.** L'issue #3 resta un riferimento già esistente; la presente scheda è soltanto un'analisi da discutere.
