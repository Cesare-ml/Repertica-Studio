# REPS-TECH-056 — Autenticazione OAuth 2.0 facoltativa per le caselle IMAP/SMTP, senza account Repertica

- **ID stabile:** REPS-TECH-056
- **Data della ricognizione:** 2026-11-23
- **Stato:** **Proposta — da valutare**
- **Filone:** Strumenti e architettura; soltanto ricerca e documentazione
- **Priorità ipotetica:** **P2 condizionata** alla domanda degli utenti e alla sostenibilità dei requisiti dei provider; **P1–P2 solo se caselle professionali realmente necessarie risultano inutilizzabili** con le modalità correnti; P3 o nessuno sviluppo se gli utenti usano password per app o il client di posta esterno.
- **Decisione del proprietario:** nessuna. Questa scheda non autorizza sviluppo, integrazioni, registrazioni presso provider, spese o nuove dipendenze.

## 1. Problema, destinatari e riscontri

**Problema circoscritto.** Repertica Studio è un archivio desktop locale ma contiene già una funzione facoltativa di posta IMAP/SMTP. Alcuni utenti professionali potrebbero voler collegare la propria casella senza utilizzare password tradizionali, oppure non poterla collegare perché il provider o l'amministratore ammette soltanto autorizzazioni moderne. Il fine non è creare un servizio email Repertica: è **conservare la scelta di usare la posta del proprio fornitore**, senza introdurre un login all'app, un cloud proprietario o dipendenze per consultare opere, clienti, vendite e documenti offline.

**Verificato nel repository pubblico, branch `main`, il 23/11/2026:**
- [README](../../../README.md) ribadisce app Tauri/Angular/Rust/SQLite gratuita e senza account.
- [BUSINESS-MAIL-DESKTOP.md](../../BUSINESS-MAIL-DESKTOP.md) descrive caselle multiple, IMAP TLS 993, SMTP TLS 465/STARTTLS 587, password e password per app conservate nel portachiavi nativo Windows/macOS/Linux; configurazioni prive di segreti in SQLite; backup senza credenziali, reinserimento sul nuovo computer. **OAuth2/XOAUTH2 è esplicitamente indicato come limite non ancora coperto** e sviluppo solo suggerito.
- [business_mail.rs](../../../src-tauri/src/business_mail.rs) usa `imap::connect` con `client.login(..., password)` e `lettre::SmtpTransport` con `Credentials`; gestisce segreti tramite `keyring::Entry`. Non è emerso, in questo modulo, un percorso di autorizzazione browser, refresh token o autenticazione SASL XOAUTH2.
- Le bozze locali già funzionano offline e l'invio SMTP necessita sempre conferma esplicita. La lettura IMAP non segna i messaggi come letti; se l'esito dell'invio è incerto **non** deve scattare un secondo invio automatico.
- Non è stata eseguita una prova con account Google/Microsoft, né raccolta una richiesta di utenti: **non si presenta il limite come un guasto già osservato**.

**Contesto esterno, da rivalidare prima di ogni scelta progettuale:** Google documenta SASL XOAUTH2 per IMAP/SMTP; Microsoft documenta OAuth per IMAP/POP/SMTP e la dismissione della Basic Authentication per IMAP in Exchange Online. Le condizioni di pubblicazione e verifica delle applicazioni OAuth, in particolare per accessi sensibili/restricted di Google, possono essere onerose. Fonti ufficiali consultate il 23/11/2026:
- [Google — XOAUTH2 per IMAP/SMTP](https://developers.google.com/workspace/gmail/imap/xoauth2-protocol)
- [Microsoft — OAuth per IMAP/POP/SMTP](https://learn.microsoft.com/en-us/exchange/client-developer/legacy-protocols/how-to-authenticate-an-imap-pop-smtp-application-by-using-oauth)
- [Microsoft — Basic Authentication in Exchange Online](https://learn.microsoft.com/en-us/exchange/clients-and-mobile-in-exchange-online/deprecation-of-basic-authentication-exchange-online)
- [Google — Verifica per scope soggetti a restrizioni](https://developers.google.com/identity/protocols/oauth2/production-readiness/restricted-scope-verification)

**Destinatari:** artisti che usano la posta professionale per rispondere ai clienti, gallerie con caselle Microsoft 365/Google Workspace e piccoli studi con più provider. Non tutti hanno bisogno di questa integrazione: chi usa soltanto l'archivio Repertica deve continuare a non vedere né configurare account online.

## 2. Confini e confronto anti-doppioni

- [TECH-031](REPS-TECH-031-allegati-bozze-posta-ripristinabili.md) studia la portabilità degli **allegati delle bozze** salvati come riferimenti a file. Qui si studia **soltanto il metodo di autenticazione** ai server, senza cambiare l'archiviazione dei file o il modello delle bozze.
- [TECH-039](REPS-TECH-039-salvataggio-selettivo-email-eml-portabile.md) studia l'esportazione di messaggi EML: non introduce login. [TECH-040](REPS-TECH-040-scambio-contatti-vcard-offline.md) riguarda la rubrica, non gli account di posta.
- [TECH-017](REPS-TECH-017-protezione-archivio-attivo-cifratura-facoltativa.md) concerne cifratura a riposo dei dati locali, diversa dai **token OAuth conservati nel portachiavi OS**. [TECH-014](REPS-TECH-014-bridge-mcp-locale-lettura-permessi.md) e [TECH-049](REPS-TECH-049-confine-istruzioni-dati-mcp-prompt-injection.md) riguardano l'MCP: **nessun accesso dell'assistente alla posta, ai token o al flusso OAuth**.
- Nell'[indice Gestione e UX — PR #14](https://github.com/Cesare-ml/Repertica-Studio/blob/docs/idee-gestione-ux-20261121/docs/idee/gestione-e-ux/README.md), UX-035 studia le **preferenze di contatto** dei collezionisti e le comunicazioni individuali; non l'autorizzazione tecnica alle caselle. Nessuna proposta UX viene modificata.
- **Escluso:** un account Repertica o SSO per usare l'app, centralizzazione dei token su un servizio Repertica, campagne email, sincronizzazione automatica di tutte le caselle, API Graph/Gmail obbligatorie, accesso email via MCP, import di contatti senza consenso, invio automatico e acquisizione di privilegi amministrativi sulle organizzazioni.

## 3. Requisiti certi, ipotesi e scenari da verificare

**Certi (vincoli già fissati):** uso generale offline-first; account/abbonamento Repertica mai obbligatori; SQLite e dati d'archivio locali ed esportabili; backup liberamente collocabili; lingua di sistema e fallback inglese; posta solo come opzione volontaria; invio sempre confermato dall'utente. Un account del **provider di posta scelto dall'utente**, necessario per la sola posta, non equivale a un account Repertica.

**Ipotesi da validare:**
1. Casella individuale Microsoft 365 con Basic Auth disabilitata: l'utente vorrebbe continuare a leggere e inviare tramite l'IMAP/SMTP esistente, con autorizzazione del proprio tenant. L'amministratore potrebbe vietare OAuth a client non approvati: **non garantire compatibilità**.
2. Casella Google: l'artista autorizza con browser di sistema e torna a Repertica, ma i requisiti di scope/consent screen/verification potrebbero rendere insostenibile una distribuzione generalizzata; verificare la fattibilità prima dell'interfaccia.
3. Casella generica: il provider accetta password o password per app; l'utente conserva il flusso corrente e **non** deve passare a OAuth.
4. Nuovo PC da backup: configurazione non segreta riconosciuta, ma token non trasferiti; l'app indica chiaramente **«Ricollega la casella»**, senza collegarsi ad account sbagliati.
5. Connessione assente: opere, clienti, economia e bozze restano disponibili; autorizzazione, refresh, lettura remota e invio email sono indisponibili, senza bloccare l'app.

## 4. Specifica e flussi UX — solamente descrittivi

**A. Aggiunta della casella.** In Posta → Account, conservare l'opzione attuale «Password/password per app». Solo per provider effettivamente supportati e valutati, proporre «Accedi con il provider (OAuth)». La scelta non è preimpostata per qualsiasi server IMAP e non cambia le credenziali di altri account.

**B. Anteprima della concessione.** Prima di aprire il browser di sistema mostrare: provider/tenant dichiarato, indirizzo della casella, permessi di lettura e/o invio necessari, dati che restano locali e che cosa avviene sul sito del provider. Nessuna opzione di accesso a contatti, calendario o file se la funzione non la richiede. Il consenso OAuth non è consenso commerciale a contattare clienti.

**C. Autorizzazione volontaria.** Aprire il browser OS sul dominio di autorizzazione riconosciuto, usare Authorization Code con PKCE, `state` imprevedibile e redirect locale **limitato alla durata del collegamento**, oppure altra modalità native-app approvata dal provider. Il ritorno deve essere verificato rispetto all'account/operazione avviata; annullamento, timeout e finestra chiusa non devono alterare una casella già funzionante. Nessun web server persistente, endpoint remoto Repertica o segreto client incorporato nell'eseguibile.

**D. Collegamento verificato.** Prima di sostituire il metodo precedente, effettuare un test esplicito e minimizzato di autenticazione IMAP e, separatamente, SMTP **senza inviare un messaggio reale**. Distinguere «autorizzazione browser concessa» da «login IMAP riuscito» e da «SMTP effettivamente autorizzato». Se l'account ha utenti/alias o server distinti, richiedere verifica delle identità; non assumere che uno scope di lettura abiliti l'invio.

**E. Uso ordinario.** Riusare vista cartelle, lettura in sola consultazione, bozze offline e conferma SMTP già presenti. All'occorrenza usare token a scadenza breve; il rinnovo non comporta apertura della posta in background o invii automatici. In assenza di token valido offrire «Ricollega»; non ripiegare silenziosamente su password o su un account diverso.

**F. Gestione e revoca.** Mostrare per casella: metodo `password`/`oauth`, provider dichiarato, stato locale (`collegata`, `scaduta`, `richiede accesso`, `bloccata dal provider`) ed eventuale ultimo test. «Scollega da questo computer» elimina i token **locali** dopo conferma; «Revoca dal provider» può guidare a una pagina ufficiale o tentare una revoca documentata, ma non promettere che la sola cancellazione locale abbia revocato l'autorizzazione remota. Non eliminare account di posta o messaggi sul server.

**G. Accessibilità.** Tutti i passaggi avviabili da tastiera, focus ripristinato al ritorno dal browser, feedback testuale leggibile da screen reader, errori con azioni specifiche, distinzione degli stati non affidata al colore e progressi/annullamenti verificabili.

## 5. Dati e API ipotetiche — nessuna modifica di schema approvata

La configurazione non segreta può rimanere collegata all'ID locale della casella già gestita, mentre **access token e refresh token devono stare esclusivamente nel portachiavi nativo**, non in `mail_accounts`, log, URL persistenti, report diagnostici, CSV, backup o strumenti MCP. Non si assume che tutti i sistemi offrano portachiavi funzionante: come già per le password, l'assenza di protezione OS deve essere un errore esplicito, **mai fallback in chiaro**.

Possibili concetti, senza nuovi endpoint reali:
- `MailAuthProfile { account_id, mode: Password | OAuth, provider_kind?, issuer?, tenant_hint?, imap_identity, smtp_identity, scope_set, last_verified_at?, state }`. `tenant_hint` e `issuer` sono riferimenti di configurazione da validare e non attribuzioni di proprietà/autenticità.
- `SecretHandle { account_id, protocol, credential_generation }` è un riferimento opaco al portachiavi, non un token serializzato.
- `beginOAuthForMail(account, provider, scopes) -> ConsentPlan`; `finishOAuthForMail(correlation_id, callback) -> VerifiedAuthState`; `testMailCapability(account, protocol) -> CapabilityResult`; `disconnectMailAuthorization(account) -> LocalRemovalResult` (**tutte API illustrative**).
- Un metodo di autenticazione **separato per protocollo** va valutato perché IMAP e SMTP possono avere autorizzazioni/scope diversi; non presumere token riutilizzabili su audience/server differenti.
- I codici di errore e gli stati persistiti devono essere versionabili e in inglese tecnico stabile; mai serializzare token, auth code, PKCE verifier, stato segreto o URL di callback in eventi UI permanenti.
- Nessun cambiamento implicito alle tabelle di opere, vendite, incassi, spese, certificati, galleria, RAW o agli attuali manifest di backup. Migrazioni eventuali conservano configurazioni password esistenti e rispettano la scelta dell'utente.

## 6. Offline, privacy, sicurezza, impostazioni e multilingua

- **Confine rete chiaro:** OAuth e IMAP/SMTP sono integrazioni remote opzionali e richiedono Internet. Il software e l'archivio locale non devono mai richiederlo, né eseguire login/refresh all'avvio per chi non usa Posta. Un eventuale listener di loopback per il callback non è un server pubblico o sempre attivo e deve legarsi solo alla sessione utente autorizzata.
- **Token e credenziali:** consenso granulare, minima durata degli access token, rinnovo solo quando pertinente, eliminazione locale sicura per quanto consentito dall'OS; tenere conto di grant revocati, password aggiornate, account cancellati, MFA, restrizioni tenant e rotazioni del token. La rimozione di una casella non deve lasciare chiavi orfane. Non sincronizzare i token tramite backup o Git.
- **Native-app security:** PKCE, `state`, validazione di issuer, audience, scope, redirect e TLS; confronto rigoroso del callback con la richiesta iniziata e finestra temporale breve; blocco di token/codici in argomenti di processo e logging. Verificare il rischio di callback localhost intercettato, deep-link hijacking e browser embedded non autorizzati. Mai usare client secret statici nell'app open source.
- **Principio di minimizzazione:** non richiedere autorizzazioni a tutta la posta quando esistono alternative più ristrette **realmente compatibili con il protocollo scelto**. I protocolli legacy con SASL XOAUTH2 potrebbero richiedere scope ampi e provider approval: evidenziarlo prima di scegliere l'architettura, senza promettere privilegi non concessi.
- **Sicurezza invio:** l'esito SMTP incerto resta incerto; nessun retry automatico, nessun messaggio spedito durante test autenticazione o rinnovo, nessun uso di email o destinatari come verifica implicita della connessione.
- **Backup e ripristino:** database e bozze continuano nei backup esistenti; portachiavi OS e token **non** sono portabili. Dopo ripristino su un altro sistema si richiede nuova autorizzazione; nessuna perdita o alterazione di opere e dati economici.
- **Impostazioni:** disabilitato per default; scelta per singola casella, test distinti lettura/invio, possibilità di ricollegare/rimuovere, indicazioni su requisiti del provider, nessun «collega tutte le caselle». Nessuna sincronizzazione o automazione in background introdotta da questa idea.
- **Lingue:** lingua del sistema, fallback **inglese**, scelta manuale e localizzazioni IT/EN/ES/FR/DE delle nuove etichette; testo di consenso del provider rimane nella lingua gestita dal provider. Codici, nomi dei protocolli e scope tecnici non tradotti; evitare di tradurre indirizzi email, tenant e codici errore. Stati e messaggi anche accessibili a screen reader.

## 7. Dipendenze, alternative, rischi e complessità

**Dipendenze da esplorare, non autorizzate:** capacità SASL XOAUTH2 delle librerie IMAP/SMTP Rust attuali o eventuale adattatore, keyring già presente, flusso OAuth native-app appropriato al provider, gestione configurazioni account, QA Windows/macOS/Linux, packaging/redirect della build Tauri, documentazione pubblica delle autorizzazioni e degli ambiti concessi. **Non** sono proposte in questa scheda nuove crate, SDK, workflow, registrazioni OAuth o modifiche di impostazioni.

**Nodo principale di sostenibilità:** per una distribuzione pubblica e gratuita, chi registra e mantiene i client ID OAuth e l'eventuale verifica di Google/Microsoft? Un'app open source non può custodire un segreto statico nell'eseguibile. Gli eventuali costi, verifiche periodiche e restrizioni tenant/provider **devono essere stimati prima** di impegnarsi: il diritto degli utenti a usare l'archivio resta indipendente da questa integrazione.

**Alternative realistiche:**
1. **Nessuna nuova funzione:** mantenere password per app dove supportate e fornire indicazioni per utilizzare un client esterno. Nessun costo di manutenzione OAuth.
2. **Aprire l'email nel client predefinito OS:** evita autentiche e permessi di Repertica, ma non aggiunge lettura IMAP interna.
3. **OAuth limitato a un provider e un profilo testato:** riduce superficie e costo iniziali, ma comporta scelte di supporto da dichiarare; non generalizzare a tutti i server.
4. **Configurazione di un client OAuth personale/organizzativo:** possibile solo se le policy provider la consentono e l'UX per utenti non tecnici resta accettabile; non scaricare sui singoli la gestione di segreti non adatti a client pubblici.
5. **API proprietarie del provider (Microsoft Graph/Gmail API):** talvolta scope più granulari ma richiedono un secondo motore di posta, ulteriore manutenzione e un vincolo al provider; non confonderle con semplice XOAUTH2 per l'IMAP/SMTP esistente.

**Complessità:** **media–alta** per un singolo provider con librerie adeguate e QA credenziali; **alta** per supporto multiprovider robusto e distribuzione pubblica soggetta a verifiche. Rischi: login con account errato, concessioni eccessive, token non revocati, callback dirottato, chiavi non disponibili su Linux, differenze tra IMAP e SMTP, tenant che blocca l'app, regressione degli account password, provider che cambia policy, manutenzione continua non sostenibile.

## 8. Criteri di accettazione e test ipotetici

1. App e archivi esistenti funzionano **senza account né rete**; disattivare OAuth non disattiva Posta/password o il resto dell'app. Una casella password già configurata mantiene configurazione e segreti dopo aggiornamento.
2. Un test OAuth completo, **solo con account sintetici autorizzati**, esegue browser + PKCE + callback verificato e autentica IMAP e SMTP indipendentemente. Consenso concesso **non** viene mostrato come login testato se manca la prova di protocollo.
3. Token scaduto, revocato, rinnovato, assenza di refresh, MFA, tenant che nega consenso, autorizzazione annullata, callback/issuer/state errati e cambio account non causano accesso a una casella diversa, fallback insicuro o cancellazione delle bozze.
4. Prove di no-secrets su SQLite, backup, file temporanei, stdout/stderr, diagnostica, log OS accessibili e payload Angular: nessun token/password/codice verificatore; dove necessario l'IPC trasporta solo dati effimeri e a privilegi minimi.
5. Portachiavi indisponibile, chiuso o bloccato → errore leggibile, nessun deposito alternativo in chiaro. Ripristino su PC diverso → richiesta di autorizzazione nuova, ma dati locali/bozze leggibili.
6. IMAP resta EXAMINE/PEEK; il test SMTP **non** invia posta. Invio manuale con esito ambiguo non viene ripetuto automaticamente.
7. Windows/macOS/Linux: browser predefinito, richiamo callback, annullamento, screen reader, tastiera e fallback inglese collaudati su dispositivi reali; provider e domini di test dichiarati, senza assumere il successo in tutti i tenant.
8. Nessuna rete o listener attivo finché non si avvia consapevolmente l'integrazione; listener temporaneo chiuso a successo, timeout o annullamento. Nessuna porta LAN o server Repertica.
9. Permessi richiesti e dati accessibili documentati chiaramente; verifica pubblicazione/consent e relative condizioni del provider **prima** di presentare la funzione come disponibile agli utenti.
10. Backup v1, SQLite, export CSV, certificati e server MCP locale mantengono compatibilità e privacy; nessun token o accesso alla posta compare nelle risposte MCP.

## 9. Fasi ipotetiche e domande per la revisione

- **Fase 0 — Validazione prima di sviluppare:** chiedere a 4–6 artisti/gallerie quali provider e quali errori incontrano realmente; provare password per app e client di sistema come alternative. Verificare policy ufficiali, costi/tempi di verifica e possibilità di account test; scrivere una matrice provider/protocollo/permessi.
- **Fase 1 — Solo specifica e fattibilità:** prototipazione concettuale del contratto di segreti, callback, scope e test indipendenti IMAP/SMTP, senza impegnare provider, nuovi SDK o codice di produzione.
- **Fase 2 — Solo se approvata in seguito:** un provider scelto e chiaramente etichettato, collegamento/disconnessione, portachiavi, test IMAP e SMTP; modalità password sempre disponibile per gli altri. Qualunque implementazione necessita autorizzazione separata.
- **Fase 3 — Solo se sostenibile:** secondo provider, tenant enterprise, localizzazione completa e QA cross-platform; considerare revoca guidata e gestione errori avanzata in base ai problemi reali.

**Questioni aperte per il proprietario (nessuna scelta fatta):**
1. Quanti utenti hanno caselle Microsoft 365, Google Workspace/Gmail o altri provider che **non** accettano il percorso attuale?
2. La posta integrata è un elemento essenziale oppure un'app email esterna è preferibile per preservare l'indipendenza?
3. È accettabile che un'integrazione **facoltativa** possa richiedere verifiche del provider o manutenzione annuale, pur lasciando Repertica completamente utilizzabile senza di essa?
4. Quale identità pubblica, se mai, dovrebbe registrare un client OAuth desktop? È sostenibile il rischio di dover sospendere un provider?
5. Meglio primo supporto solo a lettura (se compatibile con gli scope) o lettura e invio, e come spiegare permessi ampi?
6. È opportuno offrire account OAuth aziendali configurati da un amministratore senza rendere complicata l'esperienza di un artista individuale?

**Conclusione:** questa è una **proposta da valutare**, non una decisione di architettura o uno sviluppo avviato. Prima possibilità senza codice: documentare chiaramente le caselle attualmente supportate, le password per app e l'uso del client email esterno.
