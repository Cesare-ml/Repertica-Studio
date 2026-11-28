# REPS-TECH-058 — Ricognizione post-ripristino su un nuovo computer: impostazioni e integrazioni dipendenti dal dispositivo

- **ID stabile:** REPS-TECH-058
- **Data ricognizione:** 2026-11-23
- **Stato:** **Proposta — da valutare**
- **Ambito:** strumenti tecnici e portabilità tra installazioni. **Solo analisi e documentazione**, senza approvazione o assegnazione allo sviluppo.
- **Priorità ipotetica:** **P2** se un campione di utenti conferma trasferimenti fra macchine e configurazioni incomprensibili dopo il recupero; **P3/nessuna nuova funzione** se basta una guida. Un'eventuale priorità maggiore richiede incidenti verificati.

## 1. Motivazione, destinatari ed evidenze

**Problema circoscritto.** Il backup integrale protegge **l'archivio dei dati**; non può ricreare automaticamente l'ambiente operativo del computer di origine. Per esempio, dopo la sostituzione di un portatile l'artista può recuperare tutte le opere e fotografie, ma avere una destinazione di backup automatico non più raggiungibile, password della posta assenti o una connessione MCP che riguarda un'installazione differente. Il rischio non è necessariamente perdita di opere: è credere che il trasferimento sia completamente concluso quando alcune **impostazioni dipendenti dal dispositivo** devono essere verificate.

**Verificato nel branch `main` il 2026-11-23:**

- [README.md](../../../README.md), [MVP.md](../../MVP.md), [backup.rs](../../../src-tauri/src/backup.rs): backup in cartella scelta, snapshot `repertica.sqlite` + `media/` + `manifest.json`, controlli d'integrità e ripristino che **sostituisce** l'archivio attivo conservando la copia precedente. È già portabile il contenuto documentale; **non si propone di reinventare backup o ripristino**.
- [db.rs](../../../src-tauri/src/db.rs) e [auto_backup.rs](../../../src-tauri/src/auto_backup.rs): la tabella `backup_schedule` persiste in SQLite **abilitazione, frequenza, destinazione assoluta, quota e storico delle esecuzioni**. `run_core` controlla la presenza della cartella e segnala errori; la destinazione salvata su un dispositivo può essere impropria sull'altro. Non è stato riprodotto un errore reale in un trasloco.
- [BUSINESS-MAIL-DESKTOP.md](../../BUSINESS-MAIL-DESKTOP.md) e [business_mail.rs](../../../src-tauri/src/business_mail.rs): configurazione delle caselle IMAP/SMTP in SQLite; password nel portachiavi dell'OS, **escluse dai backup**. Il documento già esplicita la necessità di reinserire le password dopo un ripristino su altro computer.
- [MCP.md](../../MCP.md) e [mcp_client.rs](../../../src-tauri/src/mcp_client.rs): server MCP locale già disponibile; per collegarlo agli assistenti vengono modificati i **file di configurazione dei client esterni** (Claude, Codex, ecc.) con percorsi dell'eseguibile e dell'archivio. Questi file non sono parte della cartella di backup Repertica. Esistono già comandi di collegamento/aggiornamento.
- [app.component.ts](../../../src/app/app.component.ts): lingua scelta dell'interfaccia in `studio_settings.ui_language` (nel DB); alcune preferenze minori sono in `localStorage` della WebView, per esempio la guida iniziale, la preferenza sugli aggiornamenti automatici e l'ultimo luogo di emissione. Non tutte le preferenze della macchina accompagnano quindi il database.
- **Non verificato:** frequenza dei trasferimenti, problemi sperimentati da utenti, UX effettiva del primo avvio post-ripristino su macOS/Windows/Linux e comportamento su percorsi assoluti che per caso esistano anche nel dispositivo nuovo.

**Destinatari:** artista senza supporto IT che cambia PC, galleria che ripristina su macchina sostitutiva, archivista che esegue un recupero di emergenza, tecnico che assiste una migrazione fra Windows/macOS/Linux. Non riguarda collaborazione simultanea, cloud o accesso remoto.

## 2. Confini rispetto alle altre idee

- **TECH-009:** censimento, verifica periodica e conservazione dei backup *già creati*; qui interessa il **contesto del computer dopo il ripristino**, anche con backup perfettamente integro.
- **TECH-042:** compatibilità fra versioni del programma e schema SQLite, downgrade e aggiornamenti falliti; qui non si cambia il software installato e non si propone downgrade.
- **TECH-052:** verifica indipendente di un backup senza eseguire Repertica; qui si presume un ripristino già riuscito e si valutano solo **dipendenze ambientali**.
- **TECH-035:** protezione dei file di configurazione esterni durante la loro modifica; qui il possibile controllo si limita a segnalare collegamenti mancanti/da riconfigurare, riutilizzando in futuro la gestione esistente, senza editarli implicitamente.
- **TECH-006/007/012/022/037:** cifratura di copie, collocazione media, migrazioni schema, spazio e ripristino interrotto; nessun nuovo motore di copia, media store o recupero dati.
- **TECH-056:** credenziali OAuth *facoltative* dei provider di posta; qui le password e i token restano esclusi dal backup anche se il metodo d'autenticazione cambierà.
- **UX-015/UX-060**, nel filone Gestione e UX ([PR #14](https://github.com/Cesare-ml/Repertica-Studio/pull/14)), riguardano opere e ricognizioni, non la macchina. Consultato l'indice completo di **62 proposte UX**; nessun suo file viene modificato.

## 3. Casi d'uso e flussi UX ipotetici

### A. Trasloco volontario su un altro PC

1. L'utente crea **un backup verificato** con le funzioni esistenti e lo conserva su supporto locale scelto.
2. Sul nuovo computer installa Repertica e seleziona manualmente la cartella di backup; usa il ripristino attuale, che preserva una copia dell'archivio preesistente.
3. **Solo dopo il ripristino riuscito**, potrebbe comparire un riepilogo informativo «Controlli su questo computer»: archivio disponibile; backup automatici (destinazione e accessibilità); posta (credenziali da reinserire); assistenti MCP (stato collegamenti); preferenze locali non trasferite.
4. L'utente apre la sezione esistente relativa al singolo elemento, configura **di propria iniziativa** nuovi percorsi o credenziali e marca «verificato». Il riepilogo non modifica alcuna impostazione da sé.
5. Il pannello è richiudibile; se l'utente non usa posta/MCP non mostra allarmi generici su funzioni facoltative.

### B. Ripristino di emergenza

L'utente non sa se il backup provenga da un altro dispositivo: la verifica può essere invocata **manualmente**, senza presumere un trasferimento né introdurre un identificatore hardware permanente. Una nota spiega la differenza tra «archivio ripristinato» e «ambiente configurato». Il normale utilizzo offline di opere/vendite/certificati resta possibile indipendentemente dall'esito dei controlli facoltativi.

### C. Destinazione backup con percorso coincidente ma supporto diverso

Un vecchio percorso può esistere anche sul nuovo PC, ma riferirsi a un altro disco/cartella. Non basta `Path::exists` per dichiarare sicura una destinazione. Visualizzare il percorso e chiedere conferma **prima di considerarlo validato per il nuovo dispositivo**. Non ispezionare file di terzi o cercare dischi in automatico; niente test di scrittura silenziosi. L'eventuale prova di scrittura sarebbe una seconda fase, esplicitamente autorizzata e non distruttiva.

## 4. Specifica funzionale e UX proposta (non approvata)

**Fase minima preferita:** una guida Markdown/manuale, oppure una checklist *senza nuove tabelle*, che richiami funzioni già presenti. Valutare soltanto se questa prima soluzione non basta un pannello nativo dopo il ripristino. Ogni voce distingue `non_usato`, `da_controllare`, `verificato_da_utente`, `non_verificabile_automaticamente`, `errore`. Non usare un generico bollino «tutto sicuro».

- **Archivio:** risultato già restituito dal ripristino, non un secondo controllo crittografico concorrente.
- **Backup programmati:** esporre `enabled`, `frequency`, `destination`, data/risultato storico con dicitura «registrato sul computer d'origine o in passato»: non confondere `last_success_at` con una prova su questo computer. La modifica richiede dialogo cartella e salvataggio ordinari.
- **Posta:** distinguere configurazione casella presente da password OS disponibile; evitare un tentativo di login o rete senza comando esplicito. Niente visualizzazione della password.
- **MCP:** consultare lo stato esistente dei client (collegato, da aggiornare, assente), non toccare JSON/TOML degli assistenti senza consenso puntuale.
- **Preferenze WebView:** dichiarare quali non sono incluse nella copia. Non importare automaticamente `localStorage` o impostazioni specifiche di Windows/macOS.
- **Accessibilità:** riepilogo in testo, focus stabile, percorso con tastiera e screen reader, nessuna dipendenza dal solo colore, azioni con nome e stato distinguibili.
- **Report facoltativo:** checklist esportabile in Markdown/JSON semplice **senza credenziali, nomi di clienti, importi né percorsi assoluti**, salvo scelta consapevole dell'utente in un'esportazione privata. Non confondere il report con un backup.

## 5. Dati e ipotesi di API esclusivamente descrittive

**Nessun nuovo dato indispensabile nella fase documentale.** Le fonti proposte sono `backup_schedule` nel DB recuperato, disponibilità locale del portachiavi OS per account configurati, stato restituito dai collegamenti MCP già presenti e alcune preferenze locali della WebView.

Un possibile risultato effimero — **esempio concettuale, non contratto implementato**:

```json
{
  "schema": "repertica-device-readiness-draft-1",
  "checks": [
    {"kind": "scheduled_backup_target", "state": "needs_user_confirmation"},
    {"kind": "mail_credentials", "state": "not_checked"},
    {"kind": "mcp_clients", "state": "optional"},
    {"kind": "webview_preferences", "state": "device_local"}
  ],
  "recovery_completed": true,
  "device_configuration_verified": false
}
```

API *ipotetiche*: `inspectLocalRecoveryContext() -> DeviceReadiness` in sola lettura; `acknowledgeCheck(kind)` solo per conferma locale facoltativa; `openExistingSettings(kind)` naviga al flusso già presente. Non dedurre in automatico se il backup provenga dal medesimo dispositivo tramite fingerprint hardware, né scrivere `device_id` permanenti nel backup. Un eventuale stato di verifica persistente è **questione aperta**, non requisito: va invalidato quando l'utente cambia percorso o configurazione e non deve risultare trasferito come attestazione valida a un altro computer.

## 6. Offline, sicurezza, privacy, multilingua e compatibilità

- Tutta la ricognizione opera **offline**, senza cloud, account, abbonamento, telemetria o rete. Solo i successivi usi volontari di IMAP/SMTP o di assistenti esterni possono implicare rete a carico del rispettivo programma/provider.
- Mai esportare, cercare, reimpostare o sincronizzare segreti dal keyring. Anche in eventuali report evitare percorsi assoluti con nome utente, indirizzi email, dettagli di macchina e dati commerciali.
- Non cercare o modificare configurazioni di assistenti su volumi arbitrari. Riutilizzare i controlli e il consenso granulari esistenti. Nessun collegamento automatico.
- Non cancellare copie del computer precedente, non alterare la directory sorgente di backup né attivare/disattivare backup programmati senza conferma. La riconfigurazione deve rispettare le protezioni di retention già presenti.
- Lingua del sistema, con **fallback inglese** e scelta manuale della lingua; IT/EN/ES/FR/DE per i messaggi previsti dal prodotto. Stati/API semantici indipendenti dalla lingua; date e percorsi presentati secondo il sistema senza cambiarne il significato.
- Il ripristino tra piattaforme deve rimanere possibile secondo gli attuali vincoli di **compatibilità dello schema**; nessuna promessa di compatibilità verso versioni precedenti del programma. File JSON e Markdown opzionali in UTF-8, senza percorsi o codifiche Windows obbligatori.
- Su computer condivisi la checklist non deve rivelare dati privati sul blocco schermo o in messaggi di errore. Niente accesso a portachiavi multipli di altri utenti.

## 7. Dipendenze, alternative, rischi e complessità

**Dipendenze concettuali:** backup/ripristino e verifiche già implementati; pagina Impostazioni; stato programmato dei backup; API del keyring/posta e dei client MCP; i18n e accessibilità esistenti. Nessun nuovo servizio. Le future idee TECH-035, TECH-042 e TECH-056 possono interagire, ma **non sono prerequisiti da implementare**.

**Alternativa preferita:** pagina di documentazione «Cambiare computer in sicurezza» con elenco manuale dei controlli e spiegazione delle password assenti, dei percorsi specifici del dispositivo e dello stato degli assistenti. Seconda alternativa: mantenere soltanto gli avvisi specifici nelle pagine già esistenti, senza pannello riassuntivo.

**Rischi:** sovraccarico UX, interpretazione di una checklist come certificazione d'integrità, falsi allarmi a chi non usa posta o MCP, letture del keyring che attivano prompt OS, indicazioni errate su dischi omonimi, percorso predefinito che per caso esiste ma punta ad altre copie, stato persistente obsoleto. Contromisure: prima informazione e verifica manuale, risposte `sconosciuto` quando necessario, niente login o connessioni automatiche, integrazioni con consenso.

**Complessità:** **bassa** per sola documentazione; **medio-bassa** per report read-only che riusa API; **media** per stato persistente con invalidazione e prove native Windows/macOS/Linux. Prima misurare il bisogno.

## 8. Criteri di accettazione e prove ipotetiche

1. Un backup integro con un'installazione Windows e una destinazione `C:\\...` ripristinato su macOS conserva **opere, immagini, vendite e documenti**; la checklist segnala la destinazione da rivedere, senza modificarla o dichiararla valida.
2. Una destinazione assoluta che **esiste** sul nuovo dispositivo non viene attestata automaticamente come destinazione appropriata: serve conferma esplicita e visibile dell'utente.
3. Le caselle ripristinate non vengono dichiarate operative in assenza delle credenziali OS; nessuna password è letta, esportata o sostituita in chiaro; nessuna connessione IMAP/SMTP parte per la sola checklist.
4. Un client MCP assente, uno collegato e uno «da aggiornare» vengono descritti distintamente, senza modificare i file degli assistenti. Le impostazioni già offerte da Repertica restano la sede delle eventuali azioni confermate.
5. Se la WebView perde preferenze locali, nessuna pagina mostra erroneamente «recuperate dal backup» opzioni mai presenti in SQLite. La lingua auto segue il sistema, con fallback inglese, salvo preferenza `ui_language` effettivamente ripristinata.
6. Con posta/MCP non configurati non compaiono errori bloccanti; il lavoro offline e il ripristino completato non sono revocati per mancate verifiche facoltative.
7. Prova con filesystem lenti, backup di grandi dimensioni e prova accessibile da tastiera/screen reader: il riepilogo non richiede una seconda scansione integrale dei media né blocca il frontend.
8. Nessun controllo invia dati in rete, altera archivi o backup, cancella file, modifica portachiavi o tocca configurazioni esterne senza azione dell'utente. Il report di prova non contiene segreti e non è trattato come backup.

## 9. Possibili fasi e decisioni per il proprietario

1. **Validazione senza codice:** test di cambio Windows → macOS e Windows → Windows con backup sintetici, elenco manuale dei punti che confondono l'utente; intervistare 3–5 professionisti. Verificare prima le istruzioni attuali.
2. **Solo documentazione**, se sufficiente: checklist per il trasloco, credenziali posta fuori dal backup, necessità di confermare destinazione delle copie e riconfigurare eventualmente MCP.
3. **Eventuale UI di ricognizione read-only:** dopo ripristino completato e accessibile manualmente, senza nuovi dati o alterazioni.
4. **Eventuale memoria locale delle verifiche:** soltanto se utile e con invalidazione affidabile per dispositivo/percorso; non trasferire automaticamente tale stato con l'archivio.

**Questioni aperte (nessuna approvata):** i professionisti trasferiscono davvero l'archivio fra macchine? Quale informazione del trasloco genera più errori — backup, posta, MCP o preferenze? Il solo manuale risolve il problema? Il controllo va mostrato sempre o soltanto su comando? Quali opzioni sono essenziali, e quali sarebbe meglio non verificare affatto per non esporre informazioni sul dispositivo? È accettabile che un ripristino riuscito mostri avvisi non bloccanti? Il rischio di scrivere su una vecchia destinazione va gestito in questo flusso o all'interno della configurazione dei backup?

**Distinzione epistemica:** la *separazione dei dati* (SQLite/media, keyring, file MCP esterni, localStorage) è confermata dai file sopra; la gravità, la frequenza del problema e la necessità di una nuova UI sono **ipotesi da verificare**, non malfunzionamenti riscontrati né autorizzazioni a intervenire.

> Questa scheda non cambia codice, test, build, configurazioni, dipendenze, workflow o dati; non autorizza implementazioni.
