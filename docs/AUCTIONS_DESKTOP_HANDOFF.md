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

**Provider Studio reale (26 novembre).** Il client nativo `repertica-desktop-auctions` è già implementato nel provider Studio Online sul ramo di integrazione degli agenti (`integration/identity-artwork-qa-20261126`, non ancora in `main`): validazione del redirect loopback, pagina `/login`, scambio senza `Origin`, token con `sid`, `session-status`. La stessa prova end-to-end è passata anche contro **quel provider reale** in esecuzione sul suo runtime di test isolato con PostgreSQL 17 usa-e-getta, con il server Auctions che verifica i token tramite il `createStudioVerifier` di produzione (JWKS e `session-status` reali). L'harness e la configurazione di esempio del client nativo sono nel ramo `feat/desktop-native-client-e2e-20261126` di Repertica-Studio-Online (`api/scripts/desktop-native-e2e-harness.cjs`, procedura in `docs/SSO_CONTRACT.md`); per questa modalità il test Rust usa anche `REPERTICA_E2E_BROWSER`, che simula il passaggio di `/login` dopo l'accesso. Non coperti: l'interfaccia reale di `/login` nel browser, la finestra Tauri, la revoca dell'account vista dal Desktop.

## Prova nella finestra Tauri (26 novembre 2026)

Il Desktop è stato provato **nella sua finestra vera** (`tauri dev`, build di debug) contro il provider Studio Online reale (runtime di test isolato, PostgreSQL 17 usa-e-getta, account fittizio `@example.test`) e il server Auctions in anteprima sintetica con il verificatore di produzione. Archivio isolato: identificatore `it.repertica.studio.e2e` passato con `--config`, quindi l'archivio reale `it.repertica.studio` non viene aperto. La pagina `/login` vera è stata compilata in un browser reale; la finestra è stata pilotata tramite il debug remoto di WebView2 e i dialoghi nativi tramite UI Automation di Windows.

Percorso verificato: accesso PKCE con la pagina `/login` reale; secondo accesso immediato grazie alla sessione Studio già aperta nel browser (single sign-on); profili, aste, offerte con email; annullamento di un'offerta (con "Annulla" nel dialogo non parte nulla, con "OK" il server lo conferma); bozza locale, revisione e pubblicazione (stesso comportamento del dialogo), rifiuto di un secondo invio con lo stesso indirizzo; uscita (token dimenticato, nessuna richiesta inviata); dopo il riavvio la sessione è persa; **revoca**: dopo un cambio password sul provider il server Auctions rifiuta il vecchio token (`session-status`) e il Desktop torna a chiedere l'accesso.

Difetti trovati solo nella finestra e corretti su questo branch:

- **Conferme mai mostrate.** In Tauri il plugin di dialogo sostituisce `window.confirm` con una versione asincrona, che inoltre richiede il permesso `dialog:allow-message`, assente. Senza permesso `confirm()` fallisce senza mostrare nulla e `if (!confirm(...))` prosegue comunque (una Promise è "vera"). Aggiunti il permesso e `src/app/confirm-dialog.ts` (`await confirmed(...)`, un dialogo non mostrato vale "no"), usato nella sezione Aste. **Lo stesso difetto colpisce il resto dell'app, compresa la v0.1.2 rilasciata** (eliminazione di opere, collezionisti, certificati, vendite, immagini, ripristino dell'archivio, invio di email): va corretto a parte, partendo da `main`.
- Date in formato americano: ora nella lingua dell'interfaccia.
- Offerte con lo stesso istante riordinate per identificativo casuale: ora resta l'ordine del server.
- Pulsanti e campi senza stile: ora come nel resto dell'app.
- Dopo la pubblicazione l'elenco online non si aggiornava e la bozza restava ripubblicabile (il server rifiutava con un codice grezzo `http_409`): ora l'elenco si ricarica, la bozza si chiude e un rifiuto mostra il motivo del server (`public_details_exist` e altri) in chiaro.

Aggancio aggiunto, solo nelle build di debug: `REPERTICA_DEV_LOGIN_URL_FILE` scrive l'indirizzo della pagina di login in un file invece di aprire il browser di sistema, per aprirlo in un browser di prova. Nell'harness del provider (`api/scripts/desktop-native-e2e-harness.cjs`) un "front" locale permette a un browser vero di usare `/login` sul loopback, e `REPERTICA_E2E_CREDENTIALS_FILE` fornisce le credenziali dell'account fittizio.

Non verificati: l'apertura del browser di sistema predefinito (`rundll32`) nella finestra (la sua logica resta coperta dai test), una build di release, macOS e Linux. Il server accetta più aste per la stessa opera con indirizzi diversi (il corpo inviato non contiene l'opera di Studio): da decidere lato server.

## Regole del Gestionale Opere (26 novembre 2026, sera)

Il proprietario ha confermato che valgono le regole del Gestionale Opere. Nel Desktop: annullamento con uno dei cinque motivi, nota riservata facoltativa e blocco facoltativo dell'email, possibile anche dopo la chiusura; nome dell'offerente nella cronologia; bozza con prezzo di riserva facoltativo e condizioni di pagamento del vincitore (spedizione in Italia, beneficiario, IBAN verificato, BIC/SWIFT, banca), precompilate con le ultime usate su questo computer; alla pubblicazione l'immagine principale dell'opera viene ricodificata sul computer (JPEG, lato massimo 2400 px, senza EXIF/GPS; l'originale resta intatto) e caricata dal lato nativo con il comando `auction_upload_artwork_image`, insieme ad anno, tecnica, supporto e misure. Un'opera senza fotografia principale non può essere pubblicata. L'anti-sniping (2 + 2 minuti) lo applica il server. Test: unità 181, Rust `auction_online` 14 + 1 ignorato, build Angular. Non provato nella finestra dopo queste modifiche.

Per accendere il collegamento in produzione servono, nell'ordine: unione del ramo di integrazione del provider in `main` di Studio Online e sua distribuzione con il client nativo `repertica-desktop-auctions` in `OAUTH_CLIENTS_JSON`; servizio Auctions distribuito (`deploy/service/` nel repository Auctions) con le chiavi del provider; una prova nella finestra di una build di release contro i servizi distribuiti; quindi `ONLINE_ENABLED = true` e una release. Le immagini dell'opera non sono ancora inviate dal Desktop.

## Collegamento acceso (27 novembre 2026)

`ONLINE_ENABLED = true`: il provider di identità è in produzione su `https://studio.repertica.com` con la pagina di accesso sotto `/id/login` (accanto al pilot Studio, stesso issuer) e il servizio Auctions su `https://auctions.repertica.com`. Verificato da internet il percorso del client nativo (accesso, autorizzazione, scambio del codice senza Origin, chiamata ad Auctions, revoca dopo l'uscita). Le offerte sono spente lato server; la pubblicazione è aperta al profilo `mirko-lancerotto`. Le build di debug con `REPERTICA_DEV_*` continuano a usare i servizi locali (pagina `/login`). Per gli utenti serve una nuova versione dell'app costruita da `main` dopo l'unione di questa PR e della #174 (conferme): non ancora preparata né distribuita.

## Indirizzo della pagina e contatto con il vincitore (27 novembre 2026)

Decisioni del proprietario, che sostituiscono nel modulo d'asta l'indirizzo scelto a mano, la spedizione e il bonifico:

- **Indirizzo della pagina.** Non lo sceglie mai l'utente: lo crea il server dal titolo pubblico (regola del Gestionale, con `-2`, `-3`… se già usato). Il Desktop non invia più uno `slug` e riconosce una pubblicazione dall'id dell'asta. Le richieste in sospeso salvate dalle versioni precedenti, che usavano l'indirizzo, sono ancora riconosciute.
- **Contatto con il vincitore.** Nella sezione "Quando l'asta si chiude con un vincitore" l'artista inserisce la propria email, che riceve nome ed email del vincitore. Poi sceglie:
  - **"Ricevo i dati del vincitore e lo contatto io":** il vincitore riceve solo un avviso;
  - **"Il vincitore riceve una mia email di vittoria":** oggetto e testo liberi, con `{name}`, `{title}`, `{amount}`, `{link}` e `{artist}`. Rispondendo, il compratore scrive direttamente all'artista.
- **Modello dell'email.** Viene offerto nella lingua dell'interfaccia (`auction-winner-email.ts`) e contiene già i dati bancari salvati dalle versioni precedenti, se ci sono. "Ripristina il modello" lo ricarica. Le scelte restano su questo computer per le bozze successive.
- **Contratto.** È in Repertica-auctions `docs/ARTIST_API_V1.md`, sezione "Contatto con il vincitore". Il server deve essere aggiornato prima di distribuire questo Desktop: un server precedente rifiuta il nuovo formato con `invalid_terms`.

## Gestione completa dell'asta pubblicata (27 novembre 2026)

Richiesta del proprietario: una volta pubblicata, l'asta si gestisce tutta dal Desktop, con la stessa interfaccia del Gestionale Opere (letto, mai modificato). Dalla lista delle aste, "Gestisci" apre la schermata dell'asta:

- **Riepilogo e comunicazioni.** Stato, offerta in testa, partecipanti, ultimo aggiornamento ("Aggiorna dal server"). "Email dei partecipanti" produce l'elenco da incollare nel campo Ccn, solo di chi ha verificato l'email, con "Escludi chi è in testa" e "Escludi gli indirizzi bloccati".
- **Offerte.** Cronologia con nome ed email dell'offerente, "Annulla offerta" e "Annulla e blocca" (motivo tra i cinque del Gestionale, nota riservata, blocco facoltativo dell'email), possibili anche dopo la chiusura.
- **Partecipanti.** Nome, email, stato (verificato o bloccato), offerte valide, data di verifica, "In testa". Se il server non può leggere i contatti lo dice e non mostra elenchi vuoti.
- **Dettagli e impostazioni.** Indirizzi pubblici italiano e inglese con "Copia indirizzo", prezzi, rilancio minimo, riserva, proroga finale, versione del regolamento, visibilità dell'asta conclusa, contatto con il vincitore (la sola lettura delle condizioni scelte alla pubblicazione).
- **Email bloccate.** Blocco per indirizzo, con motivo facoltativo, e "Rimuovi blocco"; il server restituisce solo l'indirizzo mascherato (`b***@dominio`). L'elenco dei domini temporanei è della piattaforma, uguale per tutti: da qui si consulta soltanto.
- **Scrivi al vincitore.** Con l'asta aggiudicata e l'email del vincitore leggibile, apre la finestra Posta con la bozza del Gestionale (`auction-winner-mail.ts`): destinatario, oggetto e testo si leggono e si modificano, nulla parte da solo. Senza un account di posta compare "Per scrivere al vincitore aggiungi prima un account di posta".

Contratto: Repertica-auctions `docs/ARTIST_API_V1.md`, sezione "Gestione completa" (`GET /api/artist/auctions/:id/participants`, `GET …/terms`, `GET /api/artist/email-policy`, `POST …/email-policy/blocks`, `POST …/blocks/:id/removal`); nel lato nativo sono le sole nuove voci della lista fissa `allowed_request`. Il server deve essere distribuito prima di questo Desktop.

Prova: unità 306, Rust `auction_online`, build Angular; nella finestra Tauri isolata (identificatore `it.repertica.studio.e2e`, dati di esempio con indirizzi `example.test` simulati nel servizio Angular) la schermata si disegna e il percorso "Gestisci → Scrivi al vincitore → Posta" funziona con soli clic reali. Non ancora provato contro il server distribuito.

Non coperto rispetto al Gestionale: modifica, ripubblicazione ed eliminazione di un'asta prima dell'inizio; cronologia degli eventi; blocco o sblocco per singolo dominio temporaneo; pulsante "Apri la pagina pubblica" (c'è l'indirizzo con "Copia"); link all'informativa privacy nella finestra di offerta della pagina pubblica, da aggiungere quando l'informativa sarà pubblicata.
