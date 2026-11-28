# REPS-UX-075 — Caratteristiche dell'opera non verificate: distinguere assenza e informazione mancante

- **ID stabile:** REPS-UX-075
- **Data:** 2026-11-23
- **Stato:** **Proposta — da valutare**
- **Priorità ipotetica:** P2 se l'ambiguità ha conseguenze documentali o operative dimostrate; P3/nessuno sviluppo se bastano spiegazioni.
- **Perimetro:** solo studio; nessuna approvazione, assegnazione o autorizzazione a implementare.

## 1. Problema e riscontro su main

Un artista che cataloga una propria opera appena finita può sapere se è firmata, verniciata e incorniciata. Un archivista che riceve fotografie o importa un archivio storico può invece non saperlo. Un campo non verificato non dovrebbe essere interpretato come affermazione che la caratteristica sia assente, soprattutto in una scheda consegnata a una galleria o in un'analisi dei lavori da completare.

Ricognizione statica del 23/11/2026:
- [MVP — caratteristiche v14](../../MVP.md) descrive tre caselle e indica che le opere preesistenti partono da «no».
- [Migrazione v14 e test](../../../src-tauri/src/db.rs#L474-L489) aggiungono `signed`, `final_varnish` e `framed` come interi NOT NULL DEFAULT 0, vincolati a 0/1. Il [test di migrazione](../../../src-tauri/src/db.rs#L614-L633) verifica esplicitamente che un'opera v13 ottenga tre false. Quindi almeno per questi record false può significare semplicemente informazione non raccolta.
- [Modello Angular](../../../src/app/models.ts#L75-L86) e [modello Rust](../../../src-tauri/src/models.rs) usano booleani; gli input Rust hanno default false. [Componente principale](../../../src/app/app.component.ts) inizializza i tre campi a false anche per una nuova scheda; [template](../../../src/app/app.component.html) offre tre checkbox. Il [visualizzatore](../../../src/app/artwork-viewer.component.ts) mostra solo etichette positive, scelta già prudente.
- [Export](../../../src-tauri/src/export.rs#L142-L144) include i tre campi nel CSV artworks; [MCP](../../MCP.md) li include in get_artwork. La documentazione MVP segnala anche la loro presenza negli snapshot dei certificati successivi alla migrazione.

Non è dimostrato che qualcuno abbia interpretato erroneamente i dati o che il renderer del certificato stampi un falso «no»: il problema studiato è la semantica del dato, non un bug di emissione dimostrato. Non sono state eseguite app o prove con archivi reali.

## 2. Destinatari e scenari

- Archivio d'artista che cataloga opere di decenni precedenti: non conosce l'avvenuta verniciatura e vuole poterlo dichiarare senza inventare una risposta.
- Artista che inizia una scheda mentre lavora: sa che non ha ancora firmato il lavoro, ma non vuole che un campo lasciato intatto equivalga a una verifica.
- Galleria che riceve un'opera di terzi: desidera distinguere cornice assente, cornice presente e informazione non raccolta. Una fotografia etichettata «firma» non deve trasformarsi in verifica automatica.

## 3. Confini e confronto

Confrontati i 74 titoli/descrizioni UX e le 70 proposte tecniche nei due indici; approfondita [TECH-041 — Object ID](https://github.com/Cesare-ml/Repertica-Studio/blob/docs/idee-strumenti-architettura-20261121/docs/idee/strumenti-e-architettura/REPS-TECH-041-checklist-identificazione-object-id-offline.md). Questa proposta non amplia le categorie Object ID né genera un fascicolo di identificazione: chiarisce soltanto il significato dei tre flag esistenti. UX-004 studia rapporti di condizione datati, UX-026 il diario di lavorazione, UX-031 l'attribuzione autoriale, UX-064 la revisione di tecnica/supporto e UX-074 l'assenza di una fotografia; nessuna nuova funzione relativa a questi domini. TECH-057 riguarda il contratto degli export e TECH-010 la storia delle revisioni; eventuale evoluzione del contratto si coordina con questi studi senza richiederne l'implementazione. Nessun loro file modificato.

**Requisiti certi:** locale/offline-first, gratuito e open source; nessun account; backup completo e scelta della cartella; dati esportabili; lingua di sistema e fallback inglese; nessun cambiamento dei certificati storici o dei movimenti economici. **Ipotesi:** una distinzione persistente è necessaria; non basta interpretare false come «non marcato»; l'archivio storico trae beneficio da una scelta esplicita. Da validare.

## 4. Specifica e flussi UX ipotetici

Fase minima (nessun dato nuovo): spiegare vicino alle caselle e nel dizionario dei CSV/MCP che un campo non marcato può derivare da default e non costituisce attestazione di assenza. Mantenere le etichette positive del visualizzatore. Verificare con professionisti se questo risolve il problema prima di considerare una modifica dati.

Solo se insufficiente, studiare tre stati indipendenti per caratteristica: **Non verificata / Presente / Assente**, mediante controllo a scelta singola accessibile da tastiera con nome del campo e stato annunciato. Per nuova opera, default non verificata; selezione di presente o assente deliberata. Salvataggio normale della scheda, senza richieste obbligatorie. Per scheda esistente mostrare distintamente «Non verificata (valore storico non marcato)» dove pertinente; non forzare una revisione collettiva. I tre stati non costituiscono una perizia né una prova di autenticità.

Esempio: un dipinto noto senza firma ma verniciatura ignota e incorniciato può risultare rispettivamente assente, non verificata, presente. Il visualizzatore privato potrebbe mostrare il riepilogo; un documento destinato a terzi non deve trasformare «non verificata» in una negazione. Non introdurre in questa fase dashboard, attività automatiche, filtri globali o inferenze sulle fotografie.

Impostazioni: nessuna nuova preferenza obbligatoria. Etichette localizzate nelle lingue disponibili, scelta di sistema/fallback inglese e override esistente. Testi utente non tradotti automaticamente. Nessun colore come unica distinzione, niente checkbox indeterminate ambigue o sequenze cicliche non accessibili.

## 5. Dati e compatibilità (solo ipotesi descrittive)

Opzione A: nessuna migrazione, documentare semantica conservativa dei booleani esistenti. Opzione B: mantenere i booleani legacy e introdurre un qualificatore di conoscenza per ciascuno, o un modello versionato esplicito con valori stabili UNKNOWN/PRESENT/ABSENT. Non implementare entrambe le opzioni. L'eventuale rappresentazione deve impedire coppie incoerenti e definire un solo valore autorevole. Nessuna data di verifica inventata; un'eventuale fonte/data dichiarata è fuori dalla prima fase.

Per i record esistenti non è possibile distinguere in generale false scelto esplicitamente da false predefinito. Conservare il dato originale; una proiezione prudente può trattare ogni false legacy come non verificato, spiegando all'utente che anche una precedente negazione deliberata dovrà essere riconfermata se serve. I true legacy rimangono indicazioni positive già registrate, non prove indipendenti. Non usare created_at/updated_at o immagini per inferire una verifica. Nessuna retrodatazione o riscrittura degli snapshot certificati. Prima di una eventuale migrazione occorrono compatibilità con backup supportati e percorso di recupero già previsto dal progetto; versioni precedenti non devono aprire in scrittura uno schema incompatibile.

## 6. Import/export, offline, privacy

Funzione interamente locale, senza account, API esterna, cloud, telemetria o AI. Il backup integrale nella cartella scelta includerebbe gli eventuali nuovi stati nello SQLite. Nessuna copia separata di foto o dati personali. Nelle condivisioni mostrare solo informazioni scelte e mai introdurre un nuovo giudizio di autenticità.

Mantenere interpretabili i CSV storici: non cambiare silenziosamente il significato delle colonne booleane; valutare colonne esplicite aggiuntive con contratto documentato, oppure versione separata se necessaria. Il vecchio false esportato resta legacy non marcato, non assenza accertata. L'import futuro dovrebbe mantenere il valore originale e segnalare assenza di conoscenza, senza ipotizzare una verifica. Il server MCP read-only necessita di un contratto esplicito per non esporre unknown come false certo; nessun ampliamento dei dati personali. I certificati già emessi e i PDF già esportati non vengono modificati. Per nuovi documenti, eventuali indicazioni descrittive devono essere deliberate e coerenti con il template, non aggiunte implicitamente.

## 7. Alternative, costi, dipendenze e rischi

- Prima alternativa: sola spiegazione, mantenendo checkbox e tag positivi; costo basso, nessuna migrazione. Può bastare per un artista che conosce tutte le opere.
- Note libere: adatte a pochi casi, ma non distinguibili da strumenti esterni; evitare di imporre note obbligatorie.
- Stato esplicito: complessità media (UI, migrazione, IPC, export, MCP e nuovi snapshot ove applicabile), nessun nuovo servizio o libreria indispensabile. Rischi: regressioni nei booleani legacy, maggiore attrito di catalogazione, sovrainterpretazione delle parole «verificata»/«assente», falsi conteggi e incompatibilità con consumer CSV. Una cronologia completa o report di condizione avrebbe costi maggiori e duplicherebbe altri studi; esclusa.

## 8. Criteri di accettazione e ricerca da svolgere

1. Su casi sintetici legacy v13/v14, distinguere ciò che non è noto; nessuna inferenza sulle opere reali e conservazione dei valori storici. Testare anche false deliberati, true, e una nuova scheda con tutti gli stati non verificati.
2. Le tre caratteristiche possono essere modificate indipendentemente, con annullamento normale; riapertura, export, backup e ripristino preservano il significato. Nessuna modifica a prezzo, codice, vendita, incasso o certificato già emesso.
3. Una nuova dichiarazione negativa deve richiedere scelta esplicita; nessuna conversione derivata da immagini, anno o note. Nessun segnale di autenticità automatica.
4. Verificare keyboard/screen reader, traduzioni e fallback inglese; test su Windows/macOS/Linux, anche offline. Verificare i consumer del CSV e MCP con stato ignoto e nuovo contratto; nessun campo privato aggiuntivo.
5. Prova comparativa con 3–5 artisti e archivisti su una scheda nuova e una legacy: sanno spiegare senza aiuto la differenza fra non marcata, non verificata e assente? Se la spiegazione minima risolve il dubbio, fermarsi. Nessun test eseguito in questa ricognizione.

## 9. Fasi e questioni per il proprietario

- F0: osservare casi reali e misurare l'ambiguità; non assumere che tutti i vecchi record siano errati.
- F1, solo se valutata utile: chiarimenti di interfaccia/documentazione e contratto legacy.
- F2, soltanto dopo decisione esplicita: studio della compatibilità e degli stati persistenti; poi eventuale progettazione di collaudo, non autorizzata ora.

Domande aperte: il proprietario considera le caselle oggi come semplici indicatori positivi o come dichiarazioni anche negative? Serve davvero distinguere l'assenza nei tre campi, oppure solo nella firma? Gli archivi già popolati hanno valori false impostati deliberatamente? Quali destinatari esterni usano i CSV/MCP? L'interfaccia a tre stati rallenterebbe la catalogazione in modo sproporzionato? Se il bisogno non è dimostrato, mantenere il modello attuale.

**Stato finale: Proposta — da valutare.** Nessun incarico di sviluppo.
