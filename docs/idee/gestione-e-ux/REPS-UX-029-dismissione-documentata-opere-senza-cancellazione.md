# REPS-UX-029 — Dismissione documentata delle opere senza cancellare la storia

- **ID stabile:** REPS-UX-029
- **Data della ricognizione:** 2026-11-21
- **Stato:** Proposta — da valutare
- **Filone:** Gestione e UX
- **Priorità proposta:** P2 per archivi, gallerie e studi con opere ritirate/dismesse; da verificare con professionisti reali. Non è una priorità approvata.
- **Natura:** esclusivamente analisi documentale. Nessuna richiesta di implementazione, issue operativa, modifica di codice/configurazioni o merge.

## 1. Problema, destinatari e valore concreto

Un'opera può **uscire definitivamente dal patrimonio o dal catalogo operativo** per una decisione esplicita, senza essere stata *venduta*: donazione, trasferimento permanente, distruzione deliberata di uno studio preparatorio o ritiro documentato dalla raccolta. In questi casi cancellare la scheda fa perdere identificativo, fotografie e storia; marcarla genericamente «in archivio» non spiega **quale decisione**, **quando** e **sulla base di quali fonti** è stata presa.

Destinatari prioritari: archivi d'artista che devono mantenere traccia anche delle opere non più esistenti, gallerie che possiedono opere non destinate alla vendita, collezioni professionali e studi che conservano una cronologia pluriennale. Per chi cataloga poche opere proprie e non pratica dismissioni potrebbe bastare lo stato «in archivio»: il bisogno è un'**ipotesi da verificare**, non un difetto misurato.

L'obiettivo non è cancellare, vendere, valutare o movimentare opere: è poter conservare un **fascicolo della decisione di uscita definitiva**, leggibile e correggibile in modo tracciato, senza attribuirgli automaticamente valore legale. **Il trasferimento giuridico del titolo non è deducibile da un pulsante o da un documento dichiarato dall'utente.**

### Evidenza diretta su `main` (21 novembre 2026)

- [`src/app/models.ts`](../../../src/app/models.ts) espone `ArtworkStatus = available | reserved | sold | archived`; non distingue una mera archiviazione dalla dismissione motivata.
- [`commands.rs`](../../../src-tauri/src/commands.rs) ammette questi quattro stati; una vendita impone `sold` e ne preserva l'associazione. `delete_artwork_core` esegue un'eliminazione della riga e, se autorizzata dai vincoli, pulisce riferimenti a media; rifiuta la cancellazione collegata a vendite o certificati. La UI chiede conferma per «Eliminare definitivamente» e dichiara l'azione irreversibile.
- [`docs/MVP.md`](../../MVP.md) e [`docs/DATA-PORTABILITY.md`](../../DATA-PORTABILITY.md) descrivono schede d'opera, certificati storici, backup completo verificato, 12 registri CSV e assenza di import generale. Nessuna procedura specifica di *dismissione documentata* è emersa dai percorsi esaminati.
- **Limite dell'evidenza:** ricognizione statica dei modelli, comandi e documentazione; non sono state provate tutte le schermate in una build installata. Questa è un'opportunità gestionale, non un malfunzionamento dimostrato.

### Confronto con proposte esistenti — perimetro senza doppioni

| Ambito | Confine della nuova proposta |
| --- | --- |
| [UX-001 — Provenienza e movimentazioni](REPS-UX-001-provenienza-movimentazioni.md) | Documenta **dove** si trova un'opera e quando cambia custode. Un trasferimento fisico non dimostra una decisione di dismissione; una dismissione non registra da sola un movimento. |
| [UX-004 — Condizione/conservazione](REPS-UX-004-rapporti-stato-conservazione.md) | Registra **condizioni osservate** e interventi. Una condizione grave non autorizza automaticamente la distruzione, né questa scheda fornisce trattamenti conservativi. |
| [UX-015 — Ricognizioni inventariali](REPS-UX-015-ricognizioni-inventariali-discrepanze.md) | Segnala **non rinvenuta dopo controllo**; non prova perdita, furto o dismissione. Questa proposta esclude esplicitamente i **casi di scomparsa da accertare**, che rimangono una possibile analisi distinta. |
| [UX-021 — Acquisizioni da terzi](REPS-UX-021-acquisizioni-opere-terzi-titolo-documentato.md) | Documenta l'**entrata** deliberata; questa proposta documenta l'**uscita deliberata**, senza confondere acquisto, custodia o proprietà. |
| [UX-013 — Resi/rettifiche vendita](REPS-UX-013-resi-annullamenti-rettifiche-vendite.md) | Una **vendita ordinaria**, un reso o un rimborso restano nel dominio delle vendite; non si genera una pratica di dismissione in parallelo per duplicarne effetti economici. |
| [UX-016 — Valutazioni/assicurazioni](REPS-UX-016-valutazioni-coperture-assicurative-scadenze.md) | La dichiarazione di dismissione non modifica automaticamente valori, polizze o sinistri. |
| [TECH-010 — Revisioni metadati](https://github.com/Cesare-ml/Repertica-Studio/blob/docs/idee-strumenti-architettura-20261121/docs/idee/strumenti-e-architettura/REPS-TECH-010-storico-revisioni-metadati.md) e [TECH-016 — Allegati](https://github.com/Cesare-ml/Repertica-Studio/blob/docs/idee-strumenti-architettura-20261121/docs/idee/strumenti-e-architettura/REPS-TECH-016-allegati-documentali-locali-multi-entita.md) | Studiano infrastruttura trasversale per revisioni e file; questa proposta definisce **decisione, stati e UX di dominio**. Nessun file tecnico viene modificato. |

**Fuori ambito:** furti sospetti, opere cercate e non rinvenute, prestiti, singoli spostamenti, dismissione di materiali di consumo, acquisti, vendite, generazione di atti legali o contratti, calcoli fiscali e cancellazione definitiva automatica.

## 2. Scenari per verificare se il bisogno esiste

1. **Studio d'artista, opera mai venduta.** Un lavoro giovanile `ART-0142` viene distrutto deliberatamente. L'artista vuole continuare a cercarne codice, immagini e data di creazione; registra `distruzione dichiarata`, data nota o approssimata, motivazione e foto del fascicolo, lasciando la scheda storica consultabile.
2. **Galleria proprietaria, donazione.** Una scultura non destinata alla vendita viene donata a un'istituzione. La galleria conserva decisione, destinatario **dichiarato**, riferimento del documento e data; la consegna materiale resta un evento separato, se mai esisterà UX-001. La pratica non genera una vendita a zero euro o un costo negativo.
3. **Archivio, trasferimento fuori collezione.** Un'opera viene trasferita a una raccolta didattica e non deve comparire tra gli oggetti *in gestione attiva*. Rimangono chiari l'identificativo storico e la fonte, senza cancellare relazioni con fotografie e registri.
4. **Decisione corretta prima dell'esecuzione.** Una dismissione proposta per errore viene annullata *prima di essere dichiarata conclusa*: il fascicolo mantiene la rettifica e non finge che l'opera sia stata distrutta.
5. **Archivio con dati preesistenti.** Un'opera oggi `archived` non riceve un fascicolo di dismissione fittizio in fase di migrazione; l'utente può decidere in seguito se documentare una vera uscita storica con data e fonte.

### Un caso da NON trattare come dismissione

Un'opera non trovata in deposito durante un inventario: lo stato corretto resta «**non rinvenuta / da verificare**» nella ricognizione, non «distrutta», «ceduta», «rubata» o «dismessa». Un falso positivo potrebbe far perdere un oggetto ancora esistente. La gestione di incidenti, furti, ritrovamenti e segnalazioni esterne non fa parte di questa scheda.

## 3. Flussi UX proposti (soltanto descrittivi)

**Ingresso contestuale:** `Opera → Altre azioni → Documenta uscita definitiva`, visibile ma separato dal pulsante `Elimina definitivamente` e da `In archivio`. Intestazione sempre con **codice, titolo e fotografia**; breve avviso: «Questa pratica non cancella l'opera e non modifica vendite o incassi». Per una sola opera la compilazione deve richiedere pochi campi.

**A. Nuova pratica in bozza.** Scegliere motivo normalizzato ma non normativo: `destroyed_declared` (distruzione dichiarata), `donated_declared` (donazione dichiarata), `transferred_declared` (trasferimento permanente dichiarato), `removed_from_collection` (ritiro dal catalogo operativo), `other`. La scelta non deve imporre che l'utente sia proprietario o legittimato: è una *dichiarazione da documentare*, non una verifica legale. Un campo libero spiega le circostanze.

**B. Dettagli della decisione.** Data della **decisione** distinta dalla data della sua **esecuzione**; se sconosciuta, precisare l'incertezza anziché inventare un giorno. Fonte (verbale, lettera, nota personale, documento di trasporto, altro), eventuale referente/destinatario dichiarato e documenti locali. Per una semplice nota di ritiro dal catalogo è accettabile «documento non disponibile», ma la mancanza deve essere visibile. Nessun campo obbligatorio di nominativi/indirizzi di terzi.

**C. Confronto e protezioni.** Prima di confermare mostrare stato commerciale attuale, eventuali vendita e certificati, dati di ubicazione, eventuale ricognizione/affidamento futuro e cosa **non** sarà toccato. Se una vendita è già registrata, non consentire di convertire implicitamente la sua uscita in «donazione/dismissione»: spiegare il conflitto e indirizzare al registro autorevole, senza mutazioni automatiche. Un'opera con certificato emesso **non** perde il certificato.

**D. Decisione e completamento distinti.** Stati minimi ipotetici `draft` → `declared_decided` → `declared_completed`, con `cancelled` e `corrected` se motivati. Una decisione registrata non significa che l'opera abbia già lasciato lo studio o sia stata distrutta. Un completamento richiede scelta esplicita e anteprima; se l'esecuzione non è verificabile, mostrare «**completamento dichiarato, non verificato**».

**E. Consultazione storica e rettifiche.** La scheda opera rimane ricercabile tramite UUID, codice e titolo; nella lista operativa un filtro distingue *attive / in archivio / dismissioni documentate*. La linea temporale mostra chi ha **dichiarato** l'evento (testo, non identità verificata), quando e con quale fonte. Una correzione crea un evento di rettifica con motivo, senza sovrascrivere il fascicolo precedente né riciclare il codice.

**F. Report volontario.** Anteprima di una scheda di dismissione **non fiscale e non contrattuale**, stampabile/salvabile localmente, con informazioni fonte e avvertenza sui limiti. Documenti e indirizzi privati sono esclusi dalla copia condivisibile se non selezionati esplicitamente.

**Accessibilità:** tastiera e screen reader per tutte le azioni, riepilogo non color-only, conferma leggibile e focus ripristinato nella scheda originaria; date incomplete e conflitti spiegati in testo. Nessun dialogo a più passi obbligatorio per una pratica semplice.

## 4. Dati minimi e regole di dominio (ipotesi, NON schema/API autorizzati)

- **`artwork_retirement_case`**: UUID stabile, `artwork_id` FK, codice di motivo, descrizione, stato del fascicolo, data decisione e precisione della data, data esecuzione e precisione, `declared_by` testo opzionale, destinatario testo/ID soggetto **solo se approvato in un futuro filone**, note riservate, fonte, riferimenti allegati, `created_at`, `updated_at`; eventuale `effective_at` distinto dalla registrazione.
- **`artwork_retirement_event`**: UUID, `case_id`, tipo (`created`, `decision_recorded`, `completion_declared`, `correction`, `cancelled`), valore/note precedenti e nuovi **necessari** alla ricostruzione, data dichiarata, timestamp di registrazione, motivazione e riferimento alla versione precedente; niente firma di utente autenticato se l'app non ha identità utente.
- **`retirement_attachment`** (solo se serve): riferimento a documento locale copiato nell'archivio, nome originale sanificato, MIME verificato, dimensione, SHA-256 e riferimento al fascicolo; evitare link assoluti o dipendenze da drive originari.
- **Proiezione della situazione corrente:** lo stato del fascicolo e lo stato commerciale di `Artwork` restano **campi concettualmente diversi**. Non trasformare automaticamente `artworks.status=archived` in «dismesso»: oggi quella voce è ambigua. Se si decide di escludere le opere con pratica conclusa dal catalogo operativo, farlo tramite un filtro derivato e dichiarato, non cancellando il record né riscrivendo il passato.
- **Invarianti:** una pratica attiva/definitiva per opera nella prima fase (o regola documentata per revisioni successive); ID e codici storici mai riutilizzati; nessuna scrittura su `sales`, `sale_receipts`, `sale_allocations`, `expenses`, certificati emessi o fotografie originali. Gli altri fascicoli e le eventuali relazioni restano consultabili.
- **Conflitti:** pratica di donazione su opera già venduta → blocco o esplicita revisione manuale del caso tramite processo separato (scelta da decidere); due pratiche di completamento sovrapposte → avviso e nessuna seconda «uscita» automatica; data di completamento anteriore alla decisione → avviso con possibilità di documentare un evento storico registrato in ritardo.
- **Nessun effetto economico automatico:** una donazione non è una «vendita 0 €», una distruzione non costituisce da sola svalutazione contabile o sinistro assicurativo; eventuali costi della pratica passano soltanto per il registro spese esistente, con inserimento esplicito.

Non si prescrive un design di tabelle o comandi eseguibile: i nomi sono segnaposto semantici per valutare il dominio.

## 5. Impostazioni, offline, multilingua, privacy, compatibilità

**Impostazioni facoltative:** filtro predefinito del catalogo (opere operative / anche dismesse), categorie/motivi aggiuntivi con codici locali, opzione di stampa con o senza note interne, scelta se richiedere una fonte quando si segna «completata». Nessuna configurazione iniziale obbligatoria o policy museale fittiziamente imposta ai piccoli artisti.

**Offline:** dati, timeline ed eventuali PDF/foto allegati in SQLite + media locali, senza account, abbonamento, cloud, posta elettronica, OCR online, verifica automatica presso terzi o telemetria. Il backup completo esistente usa una **cartella scelta liberamente** dall'utente, USB/NAS montati compresi; se questa proposta venisse realizzata, pratica e allegati dovrebbero essere inclusi nella medesima verifica e ripristino. I backup correnti **non sono cifrati**: gli avvisi restano necessari.

**Multilingua:** lingua di sistema e override attuale IT/EN/ES/FR/DE, fallback **inglese** (inclusi testi non tradotti). Motivi, stati, etichette e avvisi sono traducibili tramite codici stabili; le dichiarazioni, i nomi e i contenuti dei documenti non vengono tradotti automaticamente. Date ISO nello scambio, data locale e precisione nell'interfaccia; importi assenti se non effettivamente dichiarati altrove.

**Privacy:** contengono potenzialmente nomi di destinatari, sedi di opere di valore, atti personali o accordi non pubblici. Minimizzazione dei campi, accesso locale, anteprima dei contenuti condivisibili, esclusione di default di indirizzi, recapiti, fotografie private, importi economici e note di provenienza riservate. Non creare attestati di proprietà/consenso o nomi di operatori autenticati inesistenti.

**Compatibilità:** possibile migrazione SQLite **additiva**, record preesistenti lasciati invariati e nessuna conversione automatica degli `archived` storici. Archivi con opere senza pratiche restano validi; uno schema futuro potrebbe non essere apribile con versioni vecchie dell'app, da comunicare. Certificati storici e ID rimangono identici. Prove native su Windows/macOS/Linux, con percorsi Unicode e file su dischi esterni. La nuova funzionalità non sostituisce il ripristino integrale con import parziale.

## 6. Import/export aperto e non ambiguo

- **CSV consultivo dedicato**, distinto dai 12 registri già disponibili, con UUID della pratica/opera, tipo codificato, stato, date e precisione, motivazione, indicazione di fonte, e **nessuna prova implicita di titolarità o esecuzione**. Per eventuali referenti, indirizzi o allegati: selezione/anteprima, non inclusione tacita. Protezione delle celle testuali contro formule spreadsheet come nell'esportatore esistente.
- **JSON versionato per eventuale round-trip**: pratica, eventi di rettifica, UUID di opere e metadati degli allegati. Il file JSON non sostituisce le fotografie o gli allegati; eventuale pacchetto deve includere copie e manifest per ogni file.
- **Import futuro soltanto guidato**, con controllo referenze, stato, collisioni UUID, date, duplicati e politica esplicita di conflitto; un import ripetuto non crea due uscite per la stessa pratica e non cambia vendite/status per effetto collaterale. Conferma finale e rollback transazionale.
- **Backup resta il formato di recupero completo**; CSV e stampa sono uscite di consultazione, non backup. Il vincolo tecnico di pacchetto generale spetta alla proposta [TECH-001](https://github.com/Cesare-ml/Repertica-Studio/blob/docs/idee-strumenti-architettura-20261121/docs/idee/strumenti-e-architettura/REPS-TECH-001-pacchetto-dati-aperto.md), non a questa scheda.

## 7. Complessità, rischi, alternative e dipendenze

**Complessità:** media per pratica singola, liste/filtri e timeline; medio-alta per rettifiche non distruttive, conflitti con vendite, documenti, export/import idempotente; alta per un vero processo istituzionale di autorizzazioni, firme e revisione legale, che qui **non si propone**.

**Dipendenze:** `artworks.id`, integrità referenziale e snapshot immutabili dei certificati, backup/media locali, i18n, UX tastiera; l'eventuale TECH-016 per allegati è un'opportunità di riuso ma **non** una precondizione approvata. Collegamenti a UX-001, UX-004, UX-021 e UX-015 sono ipotesi future separabili.

**Rischi principali:** presentare una decisione non documentata come legalmente valida; confondere «in archivio» con «distrutta»; usare `sold` per donazioni o viceversa; nascondere nella ricerca un'opera che deve restare storica; cancellare media e certificati; attribuire titolarità senza fonti; retrodatare senza mostrare quando è stata registrata la dichiarazione; esposizione di ubicazioni/contatti in report condivisi. Mitigazioni: etichette «dichiarato», fonte e precisione data, controlli prima della conferma, storia append-only, separazione dei domini, backup verificato, anteprima privacy.

**Alternative da confrontare in interviste:**
1. **Nessuna nuova funzione:** usare `archived` con una nota libera, e tenere documenti in cartella locale esterna; costo nullo ma poca tracciabilità.
2. **Alternativa minima consigliabile:** un **evento di ritiro/dismissione datato** con motivo e fonte opzionale nella scheda opera, senza workflow, stati multipli, destinatari e documenti. È probabilmente sufficiente per studi con poche opere.
3. **Modello strutturato completo:** fascicolo decisione/completamento/rettifiche come descritto sopra, solo se la frequenza d'uso e gli obblighi di documentazione lo giustificano.
4. **Policy istituzionale esterna:** un museo potrebbe dover mantenere propri verbali, autorizzazioni o registri normativi; Repertica non deve fingere di sostituirli.

### Orientamenti professionali, non obblighi generali del software

Le procedure **Spectrum / Collections Trust** distinguono [Deaccessioning and disposal](https://collectionstrust.org.uk/resource/deaccessioning-and-disposal-the-spectrum-standard/) dalla [Object exit](https://collectionstrust.org.uk/resource/object-exit-the-spectrum-standard/), e trattano [Damage and loss](https://collectionstrust.org.uk/resource/damage-and-loss-the-spectrum-standard/) come processo specifico. Sono riferimenti concettuali del settore museale, **non** un insieme di requisiti legali automaticamente applicabili ad artisti, gallerie o a tutti i paesi. Qui si propone soltanto una UX utilizzabile anche da piccoli studi senza rituali museali obbligatori.

## 8. Criteri di accettazione ipotetici, misurabili

1. Un'opera `ART-0142`, senza vendita, dichiarata distrutta conserva **stesso UUID, codice, testo, fotografie e certificati eventualmente emessi**; resta reperibile dalla ricerca storica, pur potendo sparire dal filtro «attive».
2. Una **decisione** senza completamento non appare «opera fisicamente eliminata/distrutta»; la data effettiva non è inventata. Una dichiarazione retrospettiva conserva la propria data di registrazione distinta.
3. Una donazione con importo non specificato non genera vendita a zero, incasso, spesa, cambi percentuali, margine o documenti fiscali.
4. Una vendita e i relativi incassi già registrati non vengono cancellati o riclassificati dalla nuova funzione; l'utente vede un conflitto comprensibile e un percorso separato per eventuali correzioni.
5. `archived` preesistente non equivale a pratica conclusa: dopo la migrazione non compaiono fascicoli storici inventati. Una pratica annullata o rettificata conserva le motivazioni.
6. Un'opera «non rinvenuta» in un inventario non viene automaticamente considerata donata, distrutta, rubata o dismessa, e non perde lo stato commerciale attuale.
7. Un export consultivo non condivide per default recapiti/indirizzi, note private e allegati; import ripetuto dei medesimi ID non duplica fascicoli o riscrive certificati.
8. Backup e ripristino verificati su cartella scelta riproducono pratiche, correzioni e byte degli allegati; se un file richiesto manca/è alterato, il ripristino non sostituisce l'archivio attivo.
9. Tutti i percorsi funzionano offline con file Unicode, tastiera/screen reader, lingua di sistema e fallback inglese su Windows/macOS/Linux.
10. La cancellazione definitiva di una scheda con fascicolo storico non può eliminare silenziosamente la cronologia; l'eventuale protezione dei riferimenti è dimostrabile con prova di vincolo.

## 9. Fasi suggerite (non approvate e non assegnate)

- **A — Ricerca sul campo:** raccogliere 3–5 esempi reali di opere ritirate/distrutte/trasferite; verificare se una nota associata a `archived` basti davvero e quali evidenze gli operatori conservino.
- **B — Opzione minima:** evento manuale di dismissione, data/precisione, motivazione e indicazione della fonte, filtro storico/attive, nessun effetto sui registri economici e nessuna cancellazione.
- **C — Solo se il bisogno è confermato:** stati decisione/completamento, rettifiche, fascicolo con documenti locali e anteprima privacy; test del restore e compatibilità.
- **D — Interoperabilità e UX avanzata:** export strutturato, import assistito, filtri e report professionali, eventuali collegamenti **espliciti** a movimentazioni e acquisizioni se approvate autonomamente.

## 10. Requisiti certi, ipotesi e domande per revisione del proprietario

**Requisiti certi (principi e fatti su `main`):** software gratuito/open source offline-first senza account o abbonamento; archivio SQLite locale; backup verificati in cartella liberamente scelta; lingua di sistema con fallback inglese; ID opere persistenti; attuale `archived` e cancellazione distinta; certificati storici non modificabili. Nessuna prova di una procedura di dismissione già implementata nei file esaminati.

**Ipotesi (NON decisioni):** gallerie e archivi hanno casi sufficientemente frequenti da preferire il fascicolo a una nota; la distinzione decisione/esecuzione è comprensibile; una vista storico/attive è necessaria; per artisti singoli potrebbe bastare un unico evento documentato.

**Domande da lasciare al proprietario:** (1) il prodotto deve coprire soltanto l'archiviazione delle opere proprie o anche la dismissione da collezioni di terzi? (2) Il ritiro dal **catalogo operativo** va gestito qui oppure basta `archived` con nota? (3) Quali motivi di uscita ricorrono davvero tra gli utenti? (4) Richiedere sempre una fonte o accettare «nessun documento disponibile»? (5) Serve distinguere *decisione* ed *esecuzione* già nella prima fase? (6) È preferibile la sola scheda evento, rinviando documenti e report? (7) Quali operazioni devono essere bloccate in presenza di vendita o certificati?

**Stato finale: Proposta — da valutare. Nessuna idea è approvata, assegnata o avviata allo sviluppo.**
