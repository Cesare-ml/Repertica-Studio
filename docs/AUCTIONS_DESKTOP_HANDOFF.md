# Repertica Auctions in Studio Desktop — istruzioni per Claude Code

Stato: **incarico in definizione, non funzionalità già disponibile**. Ultimo aggiornamento: 25 novembre 2026. Le indicazioni confermate qui hanno precedenza sulle vecchie roadmap che richiedono il Desktop nella prima versione di Auctions. Aggiorna questo documento quando il proprietario precisa un requisito.

## Incarico

Realizza in Repertica Studio Desktop la gestione completa delle aste Repertica, prevista per **MVP 2**. L'utente potrà affidare questo file a Claude Code come punto di partenza. L'implementazione deve usare il servizio Auctions e le API comuni dell'ecosistema, senza replicare il motore d'asta nel computer. Il lavoro sul sito e sui servizi online è affidato a Codex; il relativo incarico è `docs/AUCTIONS_ONLINE_HANDOFF.md` nel repository Repertica-auctions.

**MVP 1:** Auctions funziona autonomamente da browser: l'artista inserisce un'opera e crea un'asta; l'opera viene inserita anche nel suo account Studio tramite API. Nessuna interfaccia asta in Studio Desktop o Studio Online è prerequisito.

**MVP 2:** Studio Desktop offre creazione e gestione completa delle aste; anche il sito offre la gestione completa delle offerte. **MVP 3:** integrazione bot Telegram, funzioni da definire: non anticiparla nella MVP 2.

## Stato e vincoli da verificare prima di modificare

- Il Desktop è un'app Angular 20, Tauri 2, Rust e SQLite, gratuita e offline-first. L'archivio locale deve restare utilizzabile senza account e senza Internet. Leggi `README.md`, `docs/MVP.md`, eventuali `AGENTS.md` e `CLAUDE.md`, lo stato Git e il codice attuale; non sovrascrivere modifiche preesistenti.
- Al momento della stesura non esistono nel Desktop un login Repertica né la gestione aste. Il sito Auctions mostra una demo con offerte simulate e un catalogo in sola lettura. Identità federata e API di scrittura sono ancora da completare e verificare: codice sperimentale o documenti non dimostrano che siano attivi.
- Gli endpoint e i formati effettivi vanno concordati e verificati con il progetto online prima di integrarli. `docs/STUDIO_API.md` nel repository Auctions descrive solo un precedente contratto sperimentale per anteprime: non trattarlo come un'API di produzione esistente.
- Non copiare interi archivi, credenziali, chiavi o configurazioni di produzione. Il collegamento al cloud è esplicito e limitato all'opera e alle immagini selezionate; nessuna sincronizzazione completa del Desktop è richiesta per usare Auctions.
- Preserva Studio e Sites, i dati esistenti e i backup. Nessun dato personale reale nei test o nei log. Evita cambiamenti di schema distruttivi; prevedi migrazione e rollback.

## Esperienza richiesta per l'artista

1. Offri una sezione Aste facilmente raggiungibile e una possibilità chiara di avviare un'asta dalla scheda di un'opera. Spiega quando serve connettersi o accedere, senza richiedere un account per le altre funzioni dell'archivio.
2. L'artista sceglie l'opera locale, le immagini da pubblicare e i dati consentiti. Visualizza una revisione dei soli dati che saranno inviati e pubblicati; note private, dati dei collezionisti e altre opere non devono partire implicitamente. Caricamento, salvataggio dell'opera nell'account Studio e associazione all'asta devono seguire il contratto online, evitando copie e pubblicazioni duplicate in caso di ritentativo.
3. Consenti creazione, bozza, revisione, pubblicazione, visualizzazione e gestione dell'asta tramite le capacità effettivamente supportate dal server. Mostra stato, importi, rilancio minimo, inizio/fine e risultati restituiti dal server, distinguendo chiaramente orari e fuso. Dopo una risposta incerta mostra che la conferma è in sospeso e recupera lo stato, senza dichiarare riuscita un'operazione non confermata.
4. Per ogni asta dell'artista mostra le offerte, il relativo importo e l'email dell'offerente. Consenti l'annullamento di un'offerta ritenuta sospetta attraverso l'API autorizzata, con conferma, eventuale motivazione se prevista e riscontro dell'esito. Dopo l'operazione ricarica stato, cronologia e offerta guida dal server; non calcolare o alterare localmente il risultato ufficiale. Rendere chiaro se l'operazione è impossibile o è contestata.
5. Mostra chiusura, vincitore/esito e notifiche disponibili. Aggiudicazione e pagamento sono eventi distinti: non creare o segnare automaticamente una vendita come pagata. L'eventuale registrazione locale segue una conferma esplicita e un contratto verificato.
6. In assenza di rete, lascia usare le funzioni locali e indica chiaramente che aste, offerte e annullamenti non sono aggiornabili. Eventuali bozze e cache devono essere contrassegnate come locali; non inventare conferme o offerte offline.

## Identità, dati e interfaccia

- Usa l'identità Repertica condivisa e le autorizzazioni effettive. Per l'accesso Desktop adotta browser di sistema e Authorization Code con PKCE quando il provider sarà pronto; non incorporare segreti nell'app. Proteggi e revoca le sessioni usando i meccanismi supportati dalla piattaforma e dal provider. I ruoli e l'isolamento degli account sono verificati dal server, non da un identificativo fidato dal client.
- Le email degli offerenti sono dati riservati: esporle solo all'artista autorizzato nella gestione privata; mai in catalogo pubblico, log o notifiche visibili ad altri. Limita cache e conservazione in base al contratto e alle scelte definite per il prodotto.
- Integra UI, accessibilità, gestione errori e lingue nel sistema già presente nel Desktop. Verifica gli stati vuoti, scadenze, sessione scaduta, autorizzazione negata, rete assente, richieste duplicate e aggiornamenti concorrenti.
- Il server è autorevole su offerte, scadenze, annullamenti, identità e risultato. Il Desktop non deve richiedere che il computer resti acceso durante l'asta.

## Integrazione con il lavoro online

Prima di collegare un flusso reale, concorda con Codex un contratto versionato per: identità/sessione e revoca; selezione e proprietà dell'opera; upload delle immagini; bozze/pubblicazione; elenco e dettaglio delle aste; lettura privata delle offerte con email; annullamento autorizzato e motivazioni; notifiche e sincronizzazione dell'esito; idempotenza e recupero dopo risposta persa. Definisci esempi, errori, ruoli, paginazione e regole temporali con test comuni. Se un'API manca, sviluppa e testa in isolamento ciò che non dipende da essa; non presentare il collegamento come operativo e non aggirare le verifiche di sicurezza.

## Criteri di completamento

- Con dati sintetici, un'artista seleziona una sola opera locale, vede i dati che invierà, pubblica un'asta e la ritrova sia sul sito sia nel Desktop; l'opera associata esiste nell'account Studio corretto.
- Due account distinti restano isolati: un artista non può vedere email, offerte o aste private altrui né annullare offerte su aste altrui. La pagina pubblica non espone email.
- Offerte simultanee e annullamento sono regolati dal server; importo guida e cronologia coincidono sui due canali. Ritentativi, risposta persa, riavvio, sessione scaduta e rete assente non producono duplicati o falsi successi.
- La chiusura dell'asta non registra automaticamente un pagamento; gestione locale, migrazioni, backup e ripristino continuano a funzionare. Verifica UI accessibile e multilingua, test mirati di integrazione e regressione, build e test pertinenti del progetto. Registra ciò che è passato e ciò che non è stato eseguito.
- Lavora con modifiche compatibili con lo stato presente; al termine fai commit e push su `main` solo dei tuoi cambiamenti, senza includere o scartare lavoro preesistente. Non distribuire una release Desktop né attivare aste reali finché i rispettivi flussi non sono pronti e verificati.

## Decisioni ancora aperte

Il proprietario sta definendo la roadmap. Raccogli progressivamente: campi obbligatori per asta/opera, ruoli e condizioni dell'annullamento, motivazione e visibilità dell'offerta annullata, conseguenze su tempi e offerte successive, notifiche e conservazione dei dati. Non inventare regole vincolanti: dove una decisione è necessaria, prepara il lavoro indipendente e segnala il punto preciso. Telegram appartiene alla MVP 3.

## Stato dell'implementazione Desktop (25 novembre 2026)

Verificato nel repository Repertica-auctions (stesso giorno): non esiste un'API di produzione per identità reale, pubblicazione, elenco aste, lettura delle offerte con email o annullamento; `docs/STUDIO_API.md` descrive solo anteprime non attivate (flag spento) e l'accesso Repertica non è completato. Il Desktop quindi **non è collegato** e non finge di esserlo (`AUCTION_API_CONNECTED = false` in `src/app/auction-contract.ts`).

Realizzato, senza alcuna chiamata di rete:

- **Sezione Aste** (`src/app/auctions.component.ts`) e pulsante **Avvia asta** sulle opere disponibili. Avviso chiaro «Non collegato ad Auctions», bozze locali marcate come tali (una per opera, nel `localStorage` del Desktop: solo campi pubblici, non incluse nei backup dell'archivio), revisione dei soli dati che verrebbero inviati, elenco di ciò che non parte, pulsante Pubblica disattivato. Orari nel fuso dell'utente con conversione in UTC esatto; importi in euro convertiti in centesimi.
- **Lettura delle risposte del server** (`auction-contract.ts`): parser con allowlist per aste, offerte (email riservata, con `maskEmail` per i log), pagine di storico (deduplica, ordine per tempo server) ed esito di annullamento; controllo di coerenza dell'offerta guida che segnala le discordanze senza correggerle; `awarded` non implica pagamento né vendita.
- **Operazioni con esito incerto** (`auction-operations.ts`): risposta persa = «conferma in sospeso»; ritentativo con stessa chiave e stesso corpo; conferma solo dopo rilettura dello stato dal server; rifiuti 4xx e annullamenti non consentiti/contestati distinti.
- **Orchestrazione su trasporto astratto** (`auction-manager.ts`): pubblicazione, caricamento aste, storico offerte paginato e annullamento con conferma solo dopo rilettura; offline non tenta nulla; 401 segnalato come «accedi di nuovo». Nessun trasporto reale esiste: è collaudato solo con un trasporto simulato (`tests/auction-manager.test.mjs`, 7). Non è ancora importato dall'app.
- Test: `tests/auction-desktop.test.mjs` (11), insieme a `auction-preview` (7). `npm run test:unit` 157/157, `npm run build` ok, `cargo test` 305/305.

Non realizzato perché dipende dal progetto online: login PKCE e sessioni, caricamento immagini, pubblicazione reale, elenco aste e offerte, annullamento, notifiche, registrazione esplicita di una vendita dopo l'aggiudicazione. Decisioni aperte invariate (campi obbligatori, regole di annullamento e motivazione, anti-sniping, conservazione dati). Non verificato: l'interfaccia nella finestra Tauri (solo build e test automatici).

Nota: i branch remoti `feat/auctions-offline-draft-storage-20261124` e `qa/auctions-core-combined-20261124` contengono altro lavoro Desktop non ancora in `main` (bozze in SQLite, PKCE, ledger). Non è stato incluso né sovrascritto; prima di unirlo va riconciliato con questa implementazione (in particolare `auction-preview.ts` e la migrazione v20). Attenzione: il loro client HTTP nativo usa il vecchio contratto Cloudflare `/api/studio/tenants` e `/api/studio/previews`, non quello del servizio EC2 (`artist-v1`); il modulo PKCE nativo (`auction_pkce.rs`) resta invece riutilizzabile.

## Aggiornamento 26 novembre 2026: contratto `artist-v1`

Lato online (Repertica-auctions, branch `feat/auctions-mvp1-mvp2-20261125`) esiste ora un contratto reale e collaudato per la gestione MVP 2: elenco aste dell'artista, cronologia completa delle offerte con email dell'offerente (solo per owner/editor), annullamento idempotente con motivazione e ricalcolo atomico dell'offerta guida. Documento: `docs/ARTIST_API_V1.md` in quel repository. Non è distribuito.

Nel Desktop (branch `feat/auctions-artist-v1-client-20261126`):

- `auction-contract.ts` legge le forme reali `artist-v1` (stati `upcoming/live/ended/awarded/not_awarded/cancelled`, `paymentRecorded` sempre `false`, email nullable, etichetta offerente, cursore); mappa i rifiuti documentati (`bid_already_cancelled`, `auction_closed`, `403`).
- `auction-preview.ts` prepara il corpo esatto di `POST /api/artist/publications` (istanti in millisecondi UTC, centesimi); l'id dell'asta deriva dalla chiave di idempotenza, quindi ogni ritentativo nomina la stessa asta.
- `auction-http-transport.ts` implementa `AuctionTransport` su una **porta HTTP iniettata**: percorsi fissi `/api/artist/...`, nessun URL scelto dalla webview, nessun token nella webview (lo aggiunge la porta nativa). La motivazione è obbligatoria (3–500 caratteri).
- `tests/fixtures/auctions-artist-v1/` contiene risposte catturate dal server reale in anteprima sintetica; `tests/auction-http-transport.test.mjs` le usa per vincolare parser, trasporto e gestore (compresa la risposta persa confermata solo dalla cronologia riletta).

## Collegamento del Desktop (26 novembre 2026, stesso branch)

Realizzato e collaudato, **spento in produzione** finché provider e servizio non sono distribuiti (`ONLINE_ENABLED = false` in `src-tauri/src/auction_online.rs`, unica fonte di verità; `AUCTION_API_CONNECTED` in TypeScript la rispecchia soltanto):

- **Accesso Repertica nativo** (`auction_online.rs`): Authorization Code + PKCE S256 nel browser di sistema, client nativo `repertica-desktop-auctions`, redirect loopback monouso `http://127.0.0.1:<porta>/repertica/oauth/callback` (RFC 8252), `state` confrontato in tempo costante, scambio del codice senza header `Origin`, risposta del token letta con allowlist (`Bearer`, `profile`, massimo 900 s). È il contratto del provider Studio nel ramo di sviluppo `test/browser-auctions-lifecycle-20261124` (`b97b920`, `api/identity-policy.js`). Nessun segreto nell'app; timeout 5 minuti; annullabile.
- **Token solo in memoria nativa**: mai restituito alla webview, mai su disco né nei log; dimenticato a uscita, scadenza, risposta 401 o riavvio. Un accesso che termina dopo «Esci» viene scartato.
- **Porta HTTP nativa** (`auction_api_request`): solo l'origine fissa di Auctions e solo le operazioni `artist-v1` (allowlist rigorosa di metodi, percorsi e parametri), corpo JSON ≤ 8 KB, chiave di idempotenza obbligatoria per le scritture, nessun redirect, timeout 15 s, risposta ≤ 256 KB. Senza sessione risponde 401 senza inviare nulla; un errore di rete resta «esito incerto». `reqwest`/`rustls` sono gli stessi già compilati per l'updater; il test strutturale ora vieta client HTTP in qualunque altro modulo.
- **Interfaccia** (`auction-online.component.ts`, `auction-online.service.ts`, `auction-native.ts`): accesso, profilo artista, aste con stato e orari (fuso locale e UTC), cronologia completa con email dell'offerente (non salvate), annullamento con motivazione e conferma, ritentativo con la stessa chiave, avvisi di incoerenza. **Pubblica asta** si abilita solo da collegati con un profilo abilitato e dopo la revisione, con conferma esplicita; le pubblicazioni non confermate restano in `auction-publications.ts` (solo campi pubblici) e si ritentano anche dopo un riavvio senza duplicare l'asta.
- **Build di debug**: con `REPERTICA_DEV_STUDIO_ORIGIN` e `REPERTICA_DEV_AUCTIONS_ORIGIN` (solo `http://127.0.0.1:<porta>`) il Desktop si collega a servizi locali di prova; le build di rilascio ignorano queste variabili.

Collaudo: `cargo test` 317 + 2 passati (13 nuovi in `auction_online`, compreso il flusso completo con browser simulato); prova **end-to-end reale** contro il server Auctions in anteprima sintetica con il provider Studio fittizio (`scripts/synthetic-studio.mjs` nel repository Auctions): accesso PKCE, profili, aste, offerte con email, annullamento 201 e ritentativo 200, token falso rifiutato con 401 (`cargo test -- --ignored live_` con `REPERTICA_E2E_STUDIO`/`REPERTICA_E2E_AUCTIONS`). `npm run test:unit` 175/175, `npm run build` ok. **Non verificato**: l'interfaccia dentro la finestra Tauri.

Per accendere il collegamento in produzione servono, nell'ordine: provider Studio distribuito con il client nativo `repertica-desktop-auctions` in `OAUTH_CLIENTS_JSON` e le sessioni federate (`sid`, `session-status`); servizio Auctions distribuito (`deploy/service/` nel repository Auctions) con le chiavi del provider; prova manuale nella finestra Tauri; quindi `ONLINE_ENABLED = true` e una release. Le immagini dell'opera non sono ancora inviate dal Desktop.

