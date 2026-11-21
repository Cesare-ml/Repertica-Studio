# REPS-TECH-002 — Automazioni locali consapevoli: simulazione, permessi e tracciabilità

- **Data:** 2026-11-21
- **Stato:** **Proposta — da valutare**
- **Priorità proposta:** **P2**, dopo affidabilità dei fondamentali; non è un piano approvato.
- **Ambito:** eventuale motore interno di regole *offline*; nessuna esecuzione remota o «IA autonoma» con permessi impliciti.

## Problema e destinatari

In uno studio artistico si ripetono controlli e manutenzioni: verificare backup, controllare che manchino foto, ricordare esportazioni mensili o preparare elenchi di opere filtrati. Un automatismo non deve però creare, cancellare o inviare documenti senza comprensione del suo effetto. L'[MVP](../../MVP.md) oggi non prevede un motore di regole. L'[issue #3](https://github.com/Cesare-ml/Repertica-Studio/issues/3) descrive **un MCP locale in sola lettura** via STDIO: questa idea riguarda invece **azioni interne originate dall'app**, non l'MCP né sue autorizzazioni.

**Destinatari:** artista che lavora da solo; studio con molti archivi e file; assistenti autorizzati sullo stesso computer. Non presumere multiutente, permessi OS trasferibili o servizio cloud.

## Casi d'uso e flusso UX

1. **Controllo periodico proposto:** «Quando avvio Repertica e non verifico un backup da 30 giorni, mostrami un avviso». Nessun backup viene creato senza scelta esplicita della cartella e del momento.
2. **Inventario immagini:** «Quando apro l'archivio, segnalami le opere la cui foto non è disponibile»; in seguito potrà usare una diagnostica dedicata (si veda [TECH-003](REPS-TECH-003-diagnostica-integrita-media.md)), senza correggere i riferimenti autonomamente.
3. **Report di fine mese:** «Il primo avvio dopo la chiusura del mese, proponi un CSV delle spese»; il software esegue la scrittura solo dopo autorizzazione specifica e conferma percorso. I CSV attuali restano la fonte, senza alterare i dati.
4. **Editor di regole limitato:** Impostazioni → Automazioni locali → Nuova regola → evento consentito → condizioni semplici → effetto consentito → **Simula su dati correnti** (conteggio e anteprima dei record coinvolti) → autorizza → attiva/disattiva. Mostrare ultimo esito, prossima occasione di esecuzione e revoca.
5. **Registro leggibile:** pagina «Cronologia automazioni» mostra «proposto», «approvato», «eseguito», «saltato», «fallito» con data, regola, input sintetico e motivazione, *senza riprodurre nomi o note personali se non necessari*.

## Confine operativo e modello/API solo illustrativi

- **MVP eventuale:** regole in sola lettura, avvisi e suggerimenti; non modificano opere, clienti, vendite o certificati.
- **Successivo, opzionale:** azioni di output locale *allowlisted* (es. export CSV o verifica backup) con cartelle approvate singolarmente, limite alla frequenza e anteprima. Vietati script arbitrari, shell, comandi OS, HTTP, upload, SMTP, invio a terzi e accesso libero a filesystem.
- Il motore utilizza eventi **interni espliciti** (apertura app, completamento di una vendita, risultato verifica backup), mai esecuzione continua nascosta. Un'azione non genera in cascata nuove azioni salvo regola esplicitamente consentita e testata per evitare loop.
- **Ipotesi dati locali:** `rule_id` UUID; `enabled`; `trigger_kind`; `filter_version`; `condition` tipizzata; `action_kind`; `grant_scope` (cartella e modalità approvate); `last_success`; `last_run_key`; `rule_revision`; log a durata limitata. Non includere segreti né stringhe di codice eseguibile nelle regole.
- **API concettuali:** `previewRule(rule, snapshot) -> Preview`; `requestGrant(scope) -> Decision`; `evaluateRule(event) -> Proposal`; `executeApproved(proposal, grant) -> Outcome`; `revokeGrant(id)`. Sono descrizioni, non endpoint proposti per l'MCP.

**Idempotenza:** chiave su (`rule_id`, `rule_revision`, `event_id` oppure periodo calendario definito); doppio avvio o crash non crea esportazioni duplicate. Evitare che «primo giorno del mese» dipenda ambiguamente da DST, cambio fuso e computer spento.

## Sicurezza, offline, UX e multilingua

- **Default: nessuna regola attiva.** Una regola va abilitata con consenso comprensibile, revocabile e con chiara distinzione fra *avviso*, *anteprima* e *azione*. Permessi granolari per destinazione e tipo di file; un cambio a regola o percorso richiede nuova autorizzazione.
- Tutto locale in database protetto dal profilo OS, senza account, cloud obbligatorio, cron remoto, plugin scaricati o telemetria. L'app non deve interpretare dati di una scheda come comandi.
- In prima fase si esegue **soltanto quando l'app è aperta**; al riavvio, eventi periodici saltati si gestiscono con una sola proposta esplicita, non una raffica di azioni. Esecuzione quando l'app è chiusa richiederebbe un futuro processo OS separato, **non assunto**.
- I messaggi di risultato, gli stati e i permessi devono usare la lingua di sistema, fallback inglese, e una terminologia accessibile; navigazione da tastiera, lettura screen reader, conferma testuale senza affidarsi solo ai colori.
- Per evitare fughe di dati: nei log conservare ID e riepiloghi minimizzati; nel report visibile oscurare campi non necessari e prevedere una pulizia manuale/retention locale. File esportati possono contenere nominativi: l'avviso prima dell'azione è obbligatorio.
- Se la cartella scelta non esiste, disco pieno o NAS disconnesso, fermare l'azione e chiedere nuova conferma. Non tentare percorsi alternativi o scrivere in directory interne.

## Alternative e dipendenze

- **Alternative semplici:** reminder manuali nell'app (nessun motore), task del sistema operativo configurati dall'utente, plugin esterni. Le prime due evitano costi di gestione di un rule engine; plugin e task esterni riducono portabilità e sicurezza.
- **Dipendenze eventuali:** eventi interni tipizzati, valutatore di condizioni senza `eval`, permessi Tauri isolati, audit locale, export/backup con flussi già verificati, diagnostica futura dei media. **Nessuna dipendenza** da server MCP, servizi IA o framework di automazione di terzi necessaria.
- **Rischi:** regole dimenticate, doppie esecuzioni, perdita di dati tramite export, spam di notifiche, errori dovuti al fuso orario, modifiche non intenzionali, log che accumulano dati personali. Mitigare con simulazione, quota, idempotenza, revoca, durata log configurabile, **no azioni distruttive**.
- **Complessità stimata:** **media** per avvisi interni, **alta** se si aggiungono scritture approvate e scheduling OS; non giustificare un motore generale prima di validare 2-3 necessità ricorrenti con utenti reali.

## Verifiche e criteri di accettazione futuri

1. Installazione pulita: zero regole o permessi attivi. Un utente può vedere, simulare, autorizzare, disattivare e cancellare ogni regola, interamente offline.
2. La simulazione restituisce contatori coerenti e **non** modifica DB, foto, backup, CSV o file esterni.
3. Un avviso mensile non viene ripetuto al doppio avvio; fuso diverso, DST e app chiusa per 50 giorni non creano decine di notifiche.
4. Esportazione non autorizzata, destinazione disconnessa, cambio cartella e cartella non scrivibile producono errore chiaro senza bypass e senza rivelazione di dati.
5. Regole malformate o contenuti con stringhe di shell/URL/script non vengono eseguiti; non esiste alcuna azione arbitraria né porta di rete.
6. Inversione lingua a runtime, fallback inglese, tastiera/screen reader; log senza duplicazione massiva di dati anagrafici.

## Certezze, ipotesi, decisioni aperte

**Certo dal repository:** l'MVP è offline-first, SQLite; backup, verifica e due export CSV hanno UI o core dedicati; MCP read-only è issue separata. Nessun motore di automazioni interno è descritto come implementato.

**Ipotesi:** editor di regole, schema eventi, simulazione, log, avvisi solo con app aperta e futura estensione agli export autorizzati.

**Da decidere col proprietario:** bastano promemoria manuali prima di introdurre regole? Il programma è pensato per un unico profilo per installazione? Quali azioni (se qualcuna) possono essere svolte *senza conferma immediata* dopo un consenso preventivo? È desiderabile l'esecuzione ad app chiusa, nonostante i costi e rischi aggiuntivi?

## Fasi soltanto proposte

1. Test qualitativi dei promemoria più richiesti e catalogo eventi/effetti permessi.
2. Notifiche informative e simulazione **read-only**; UI revoca e audit.
3. Se approvato, operazioni scrivibili su **file esportati** con consensi limitati, senza cambiare dati.
4. Solo dopo validazione, considerare ulteriori azioni o scheduling, con revisione sicurezza distinta.

> Nessuna implementazione o permesso operativo è autorizzato da questa scheda.
