# REPS-TECH-020 — Accessibilità verificabile dei flussi critici desktop

- **ID stabile:** REPS-TECH-020
- **Data:** 2026-11-21
- **Stato:** **Proposta — da valutare**
- **Filone:** Strumenti e architettura
- **Priorità proposta:** **P1–P2**, ipotetica: accesso universale alle operazioni critiche prima di affinamenti estetici
- **Natura:** ricerca, requisiti e criteri descrittivi; **nessuna approvazione, implementazione o assegnazione**

## 1. Motivazione, destinatari e fatti osservati

Un archivio personale perde una parte della propria indipendenza se backup, ripristino, export, gestione vendite o configurazione MCP risultano difficili da azionare senza mouse o con uno screen reader. Le persone coinvolte possono essere artisti con disabilità visive o motorie, assistenti di studio, galleristi anziani e professionisti che usano solo tastiera o zoom elevato. Un'operazione di ripristino è particolarmente sensibile: **focus e messaggi di conferma devono rendere sempre chiaro quale archivio verrà sostituito**.

**Ricognizione statica di `main` del 21 novembre 2026:**
- [README](../../../README.md) e [MVP](../../MVP.md) prevedono desktop Tauri/Angular, lingua di sistema con fallback inglese, backup/ripristino locali, nove CSV, certificati, incassi, collegamento MCP e collaudi futuri di accessibilità. La suite non dichiara una certificazione WCAG per l'app installata.
- [Interfaccia Angular](../../../src/app/app.component.html) utilizza già pulsanti nativi, etichette ARIA, messaggi `role="status"`, titoli e overlay con `role="dialog"`, `aria-modal="true"` e `aria-labelledby`. Questi sono **elementi positivi da preservare**.
- Nel [componente TS](../../../src/app/app.component.ts) non emerge una gestione esplicita di focus iniziale, restituzione del focus al trigger o navigazione `Tab`/ `Shift+Tab` confinata ai dialoghi; il markup non mostra un meccanismo esplicito di gestione `Escape`. La sola presenza di `aria-modal` non realizza queste interazioni. **Si tratta di un indizio di rischio da verificare in esecuzione, non della prova di un difetto osservato con tecnologie assistive.**
- Il [CSS](../../../src/app/app.component.css) specifica alcuni `:focus-visible` per menu e azioni, ma non costituisce una verifica globale di contrasto, scala o focus visibile. Alcune conferme passano da finestre del sistema o `window.confirm`; la loro accessibilità va valutata **separatamente** dagli overlay Angular.

**Necessità da verificare:** quali utenti e sistemi assistivi sono effettivamente presenti tra i destinatari? Quali flussi risultano bloccanti? Senza prove manuali non si dichiarano conformità né non conformità definitive.

## 2. Confini e assenza di duplicati

Questa scheda definisce **un contratto tecnico trasversale, verificabile** per controllo tastiera, focus, messaggi, ingrandimento e prove di qualità dei flussi già presenti. **Non** propone una nuova schermata gestionale.
- **TECH-008** riguarda filtri e viste, **TECH-014** autorizzazione/scope del bridge MCP, **TECH-015** codici e report diagnostici, **TECH-012** sicurezza delle migrazioni: nessuna definisce un protocollo di accessibilità dei dialoghi nativi e webview.
- **UX-001…024** della [PR documentale #14](https://github.com/Cesare-ml/Repertica-Studio/pull/14) descrivono processi professionali e loro flussi, non un contratto cross-platform per tastiera e screen reader. Nuove schermate UX potrebbero in futuro **adottare** questi criteri, senza modifica dei relativi file.
- Le funzioni esistenti di backup, ripristino, stampa, esportazione, impostazioni, collegamento Claude e salvataggio restano semanticamente invariate. Nessuna introduzione di scorciatoie che eseguono azioni distruttive senza conferma.

## 3. Scenari e flussi UX proposti

1. **Catalogazione senza mouse:** da elenco opere a «Aggiungi opera»; all'apertura il focus si sposta sul titolo o sul primo campo utile, lo screen reader legge nome e scopo della finestra. `Tab` e `Shift+Tab` restano nel dialogo; `Escape` chiude **solo** quando non perde dati senza avviso; al termine il focus torna al pulsante d'apertura o a un bersaglio coerente se il pulsante non esiste più.
2. **Vendita con errori:** dopo importo non valido, il testo associa l'errore al campo, il focus raggiunge il messaggio e il campo correggibile; nessuna comunicazione affidata al solo rosso. La conferma riuscita viene annunciata senza leggere importi o nomi non necessari.
3. **Ripristino/backup:** scelta cartella nel dialogo del sistema operativo; tornando all'app, il focus è prevedibile. Prima del ripristino la conferma indica che **l'archivio corrente sarà sostituito**, quale copia è stata selezionata e dove resta la copia precedente. «Annulla» è sempre raggiungibile; un doppio invio non provoca due operazioni.
4. **Consultazione tabelle e impostazioni con zoom:** fino al 200% e con finestre strette, etichette, tasti e messaggi rimangono leggibili e raggiungibili, senza che un overlay costringa a perdere il focus in una zona invisibile; le tabelle mantengono una navigazione semanticamente comprensibile.
5. **MCP in Impostazioni:** lo stato «collegato», «da aggiornare» o «errore» viene annunciato con testo localizzato; la configurazione mostrata in un blocco scorrevole è raggiungibile senza mouse. La conferma per modificare un file di configurazione rimane esplicita e reversibile secondo il processo già previsto.

**Flusso ricorrente:** azione utente → apertura/operazione → focus prevedibile e descrizione accessibile → errore o conferma non ambiguo → chiusura → ripristino del focus, senza azioni aggiuntive implicite.

## 4. Specifica funzionale e dati/API: solo ipotesi

**Contratto dei dialoghi modali** (confronto con [W3C APG, Modal Dialog](https://www.w3.org/WAI/ARIA/apg/patterns/dialog-modal/)):
- Ogni dialogo ha nome accessibile univoco; dove necessario descrizione associata e un contenuto iniziale leggibile. `aria-modal` è usato soltanto se il focus non può interagire con l'interfaccia sottostante.
- Focus iniziale valutato in base al rischio: primo campo per catalogazione; titolo/avviso o opzione non distruttiva per una conferma irreversibile. `Tab`/ `Shift+Tab` ciclici; `Escape` annulla se sicuro; chiusura conserva o conferma esplicitamente l'eventuale perdita di modifiche. Ritorno al trigger quando esiste.
- Un dialogo del sistema operativo non va simulato tramite ARIA; verificarne ritorno del focus e leggibilità su Windows/macOS/Linux. Se la WebView non rende affidabile un comportamento, documentare la limitazione e considerare alternative accessibili, non affidarsi a un attributo isolato.

**Contratto per annunci e moduli:** messaggi `status` non intrusivi per successi, allerta distinta per fallimenti che richiedono attenzione; etichette e testo di aiuto associati ai campi; errori identificati da codice e descrizione traducibile; evitare di far leggere automaticamente l'intero archivio o dati privati a voce. Indicatori di caricamento, disabilitazione e conclusione devono essere riconoscibili senza colore o movimento.

**Stato UI descrittivo**, non nuova tabella SQLite: `DialogA11yState { dialog_id, trigger_reference, initial_focus_target, dirty, pending_action, result_message_key }` e `OperationAnnouncement { message_key, severity, related_control_id }`. Sono tipi concettuali **effimeri**; non registrare cronologia dei tasti, nomi di clienti o campi in log. Eventuale preferenza futura di interfaccia (dimensioni testo o animazioni) vive **solo localmente e su scelta**, senza modificare il database delle opere.

**API/interfacce puramente illustrative:** `openAccessibleDialog(kind, trigger)`, `announceOperationResult(code, severity)`, `restoreFocusAfterClose(origin)`, `validateCriticalFlow(flow_id)`. Non sono API presenti e non costituiscono richiesta di librerie o struttura Angular specifica.

**Comportamento offline:** nessun servizio di accessibilità esterno, account, sincronizzazione, analisi remota o telemetria. Tutte le validazioni semantiche restano autorevoli nel backend Rust; l'accessibilità non aggira conferme, controllo permessi o integrità degli archivi.

## 5. Impostazioni, lingue e privacy

- Lingua di sistema rilevata come già previsto; **fallback inglese** per etichette, messaggi, azioni e descrizioni ARIA. Le combinazioni di tastiera si documentano per nome funzione, senza codificare lettere che cambiano con la traduzione o interferiscono con OS/screen reader.
- Evitare un «modo disabilità» obbligatorio: **accessibilità di base predefinita**. Solo eventuali personalizzazioni facoltative (densità, grandezza testo, riduzione animazioni) se dimostrano vantaggi reali e possono essere conservate offline.
- Annunci, clipboard e diagnosi non devono rivelare nomi di clienti, indirizzi, importi o percorsi non indispensabili. L'interfaccia continua a mostrare dati a chi ha già accesso al computer; questo studio non promette riservatezza da persone presenti nella stanza.
- I CSV, backup e documenti generati non cambiano per effetto della gestione focus. L'accessibilità **dei PDF stampati** è un problema separato: verificare documentazione/output prima di promettere PDF taggati o PDF/UA, senza alterare certificati storici.

## 6. Compatibilità, dipendenze, alternative, rischi e complessità

**Compatibilità:** Angular/WebView Tauri 2 su Windows (WebView2), macOS (WKWebView) e Linux (WebKitGTK); tastiera senza mouse, lettori di schermo tipici di ogni piattaforma (es. NVDA su Windows, VoiceOver su macOS, Orca su Linux). I loro risultati non si assumono identici. Nessuna modifica retroattiva a SQLite v10, identificativi, backup v1 o esportazioni a nove registri.

**Dipendenze:** progettuali da flussi, markup, CSS, i18n, dialoghi e checklist; non si prescrive qui nessuna libreria, pacchetto o workflow. Se si valutasse un gestore di focus, prima servirebbe analisi di licenza, peso, compatibilità con Tauri, mantenibilità e test nativi.

**Alternative:** (A) aggiungere descrizioni ARIA soltanto: minima manutenzione ma non risolve la gestione focus; (B) basarsi su dialoghi nativi per ogni funzione: accessibili su alcune piattaforme ma limitati per form complessi e conferme; (C) definire contratto di interazione, verifiche e remediation incrementale: più affidabile, da validare senza riscrittura generalizzata. Non promettere che test statici automatici sostituiscano test manuali.

**Rischi:** perdita di dati da `Escape` incontrollato, focus invisibile, tasti catturati due volte, screen reader che annuncia dati riservati, traduzioni con etichette ambigue, overlay scrollati fuori vista, regressioni con nuovi moduli e differenze tra WebView/OS. **Complessità ipotetica:** media per un singolo dialogo; medio-alta per definire e verificare l'intera matrice desktop con tecnologie assistive.

**Riferimenti metodologici:** [W3C WAI-ARIA APG, Dialog Modal](https://www.w3.org/WAI/ARIA/apg/patterns/dialog-modal/), [WCAG 2.2](https://www.w3.org/TR/WCAG22/) (tastiera, focus visibile/non oscurato, stato e messaggi, reflow/zoom, dimensione target), [W3C Understanding Focus Not Obscured](https://www.w3.org/WAI/WCAG22/Understanding/focus-not-obscured-minimum). Sono riferimenti di progettazione e verifica, **non una dichiarazione di conformità o certificazione**.

## 7. Criteri di accettazione e test futuri (non eseguiti)

1. Tutti i dialoghi realmente modali consentono apertura, compilazione, annullamento e chiusura con sola tastiera; `Tab`/ `Shift+Tab` non raggiungono controlli sottostanti; focus iniziale e di ritorno coerenti. `Escape` non perde modifiche senza consenso.
2. Ripristino, eliminazione, modifica di vendita/incasso e modifica configurazione MCP sono operabili senza mouse: conferma esplicita, annullamento affidabile, nessun doppio invio, feedback semantico e **nessuna azione distruttiva inattesa**.
3. In prova con NVDA/WebView2, VoiceOver/WKWebView e Orca/WebKitGTK, un tester può riconoscere nome del dialogo, etichette dei campi, errore preciso, esito e percorso verso l'azione di recupero. Se una combinazione non è testabile, il report lo dichiara e non asserisce supporto.
4. Con zoom/reflow e viewport equivalente a 200%, focus visibile/non occultato, pulsanti utilizzabili e scorrimento raggiungibile; verifica visiva e audit di contrasto per stati, placeholder e icone, senza usare il solo colore.
5. Lingue `it/en/es/fr/de` e lingua OS non producono pulsanti senza nome né messaggi ARIA mancanti; fallback inglese testato per stringhe non tradotte. Nessun testo personale letto automaticamente nei messaggi generici o riportato nella diagnostica.
6. La QA comprende scenari con conferma del sistema operativo, cartella USB scollegata, errore di permesso, backup interrotto, file dialog annullato e archivi consistenti; nessun test modifica dati reali e nessun flusso cambia l'esito economico o l'integrità dei dati.
7. Report della revisione separa **rilievi riproducibili**, **limitazioni per piattaforma** e **azioni suggerite**, prima di qualunque dichiarazione di conformità.

## 8. Fasi ipotetiche e decisioni riservate al proprietario

1. **Fase A — audit con casi riproducibili:** inventario dei dialoghi/operazioni critiche, navigazione manuale tastiera e lettori di schermo, riscontro di problemi reali con ambiente e passaggi, senza modifiche applicative.
2. **Fase B — contratto accessibile minimo:** se approvato separatamente, focus/annunci per salvataggi, vendite e apertura dialoghi; specifica dei criteri e prove di regressione.
3. **Fase C — flussi irreversibili e nativi:** se utile, ripristino, cancellazioni, stampa, file picker e configurazione MCP, sulla matrice Windows/macOS/Linux.
4. **Fase D — estensione e manutenzione:** se giustificata, verifiche continue di regressione, ingrandimento e preferenze opzionali; nessun automatismo esterno obbligatorio.

**Questioni aperte:** quali sistemi operativi sono prioritari per l'utenza? Quali tecnologie assistive vanno collaudate prima della prima release pubblica? Ci sono utenti disposti a verificare le fasi in test? Quali conferme devono essere non chiudibili con `Escape` senza secondo avviso? Esistono moduli in cui `Enter` attiva erroneamente l'azione primaria? Come evitare annunci di contenuti privati? Dove conservare un report di accessibilità che non riveli dati dell'archivio?

**Requisiti certi:** principi offline-first, dati locali, lingue dichiarate, markup e architettura esistenti su `main`. **Ipotesi:** necessità, modelli UI, API, livelli di priorità e fasi. **Nessuna segnalazione di mancata conformità verificata, nessuna approvazione o implementazione. Stato: Proposta — da valutare.**
