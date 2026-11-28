# REPS-UX-064 — Revisione guidata di metadati su più opere già catalogate

- **ID stabile:** REPS-UX-064
- **Data:** 2026-11-23
- **Stato:** **Proposta — da valutare**
- **Filone:** Gestione e UX
- **Priorità proposta:** **P2 condizionata** a casi reali di correzione ripetitiva; P3 o nessun sviluppo per piccoli archivi
- **Natura:** ricerca e specifica concettuale, senza autorizzazione, assegnazione, implementazione o modifica dei dati
- **Ambito:** correzione umana e controllata di **metadati non economici** di opere *già esistenti*, non inserimento seriale, import tabellare, revisione storica o gestione del magazzino

## 1. Problema, destinatari e riscontri verificati

**Problema da verificare.** Un professionista importa o cataloga decine di opere e scopre che un termine descrittivo ricorre in modo inesatto: per esempio «olio» in 26 schede nelle quali intende precisare «olio su tela», oppure il campo *supporto* lasciato vuoto in una selezione di opere omogenee. La correzione record per record comporta apertura, modifica e salvataggio ripetuti, con possibilità di intervenire sull'opera sbagliata. **Non è una richiesta utente già misurata.** Prima del codice occorre stabilire quante correzioni ripetitive si presentino in studi reali e se l'editor singolo basti.

**Destinatari:** studi d'artista con cataloghi estesi, gallerie che ricevono archivi eterogenei, archivisti che normalizzano manualmente la qualità descrittiva, persone che assistono artisti nella catalogazione. Un artista con poche opere non deve subire nuovi comandi o complessità nel percorso ordinario.

**Verificato sul branch main il 23 novembre 2026 tramite connettore GitHub:**

- [Interfaccia delle opere](../../../src/app/app.component.html) presenta ricerca, filtri per stato/collezionista, ordinamento, e pulsante **Modifica** per ogni riga. Nel percorso esaminato non risulta una selezione multipla con anteprima di cambiamenti per riga.
- [Componente Angular](../../../src/app/app.component.ts) usa una singola bozza articolo (artworkDraft); editArtwork apre la scheda e saveArtwork chiama il salvataggio di **una** opera. Esistono invece altre funzioni, come la creazione con foto e vendita, che hanno invarianti più complessi e non vanno riutilizzate ingenuamente.
- [Backend Rust](../../../src-tauri/src/commands.rs), save_artwork_core, aggiorna i metadati con una transazione e impone vincoli sull'unicità del codice e sullo stato di vendita. Un invio seriale di schede complete potrebbe sovrascrivere campi diversi da quello che si voleva cambiare.
- [Sicurezza modifica archivio](../../ARCHIVE-EDIT-SAFETY.md) dichiara la protezione contro la ricreazione di un record cancellato, **non** un controllo generale di concorrenza su record modificati nel frattempo.
- [README](../../../README.md) e [MVP](../../MVP.md) descrivono database locale, UUID stabili, ricerca, backup verificato e media. La presente proposta non presume disponibili ulteriori API di modifica multipla.

**Non verificato:** frequenza dell'esigenza, carichi reali di centinaia di schede, tasso di errore umano, presenza di più finestre concorrenti, prestazioni su installazioni native. Una lacuna nel percorso esaminato non prova da sola che la funzione debba essere costruita.

## 2. Confini: confronto esplicito con le proposte già presenti

- [UX-018 — Catalogazione seriale](REPS-UX-018-catalogazione-seriale-schede-bozza.md): inserisce **nuove** opere con campi riutilizzabili e bozze; REPS-UX-064 correggerebbe **solo record già consolidati**, senza crearne alcuno.
- [UX-055 — Possibili doppioni](REPS-UX-055-revisione-possibili-doppioni-schede-opera.md): confronta somiglianze e non propone un editor di massa; qui non si fondono schede né si decide che due opere sono identiche.
- [UX-005](REPS-UX-005-edizioni-multipli-serie-numerate.md), [UX-011](REPS-UX-011-cicli-creativi-relazioni-opere.md), [UX-031](REPS-UX-031-autorialita-opere-attribuzioni-collaborazioni.md) e [UX-048](REPS-UX-048-identificativi-storici-riferimenti-esterni-opere.md): non inferire edizioni, cicli, autorialità o codici storici da una correzione ripetuta.
- [TECH-030 — Vocabolari locali](https://github.com/Cesare-ml/Repertica-Studio/blob/docs/idee-strumenti-architettura-20261121/docs/idee/strumenti-e-architettura/REPS-TECH-030-vocabolari-locali-tecniche-supporti-interoperabili.md): studia termini e corrispondenze; qui si progetta **la conferma per ciascuna opera**, anche senza introdurre vocabolari. «Olio» e «olio su tela» non sono necessariamente equivalenti.
- [TECH-019 — Import tabellare](https://github.com/Cesare-ml/Repertica-Studio/blob/docs/idee-strumenti-architettura-20261121/docs/idee/strumenti-e-architettura/REPS-TECH-019-importazione-guidata-cataloghi-tabellari.md): importa dati esterni; nessun CSV in ingresso è necessario qui.
- [TECH-033 — Salvataggio condizionale](https://github.com/Cesare-ml/Repertica-Studio/blob/docs/idee-strumenti-architettura-20261121/docs/idee/strumenti-e-architettura/REPS-TECH-033-salvataggio-condizionale-schede-obsolete.md) è **dipendenza concettuale di sicurezza**: un'operazione multiopera deve controllare le versioni correnti. La sua approvazione non è implicita. [TECH-010 — Storico revisioni](https://github.com/Cesare-ml/Repertica-Studio/blob/docs/idee-strumenti-architettura-20261121/docs/idee/strumenti-e-architettura/REPS-TECH-010-storico-revisioni-metadati.md) offrirebbe una storia recuperabile, **ma non è un sostituto del controllo preventivo**. [TECH-008](https://github.com/Cesare-ml/Repertica-Studio/blob/docs/idee-strumenti-architettura-20261121/docs/idee/strumenti-e-architettura/REPS-TECH-008-viste-salvate-filtri-ricerca-locale.md) potrebbe fornire filtri riutilizzabili, senza imporli.
- [UX-062 — Coerenza brochure](REPS-UX-062-verifica-coerenza-brochure-rispetto-archivio.md) segnala un effetto importante: modificare un testo catalografico oggi non modifica un PDF passato ma può cambiare una ristampa. Mostrare l'avviso, non riscrivere brochure esistenti.

**Esclusioni:** cancellazioni, sostituzione fotografie, RAW e file del Media Center, modifiche di status commerciale, codici d'archivio, UUID, prezzi, coefficienti, titolarità, quote economiche, vendite, incassi, spese, certificati e dati personali. Non si suggerisce un comando «Applica a tutte» senza vedere esattamente tutte le righe destinatarie.

## 3. Casi d'uso e comportamento previsto

1. **Supporto mancante.** L'archivista filtra 18 opere e seleziona manualmente 9 schede per le quali ha verificato la documentazione originale. Sceglie «Compila supporto solo dove vuoto», inserisce «tela», legge i nove valori precedenti e conferma. Le altre nove opere non cambiano.
2. **Rettifica descrittiva non universale.** Ventiquattro schede contengono «olio» come tecnica, ma due riguardano olio su tavola. La tabella di anteprima evidenzia quelle due e permette di escluderle. Non si esegue una sostituzione globale sulla base di una ricerca testuale.
3. **Archivio cambiato durante la revisione.** Prima del comando definitivo un'opera selezionata viene modificata da un altro percorso. La conferma deve interrompersi o richiedere nuovo confronto, **senza usare la copia obsoleta**. La bozza della revisione resta visibile.
4. **Opera venduta e certificata.** Una correzione della tecnica corrente può essere legittima ma non rettifica il certificato storico immutabile, la vendita o la descrizione pubblicata anni prima. La riga segnala «documenti storici collegati» e offre l'apertura dei riferimenti, **non** il cambiamento degli snapshot.
5. **Selezione ampia o filtri variabili.** Cambiare il filtro o ordinamento non deve aggiungere implicitamente opere alla selezione. La conferma mostra sempre conteggio esatto e codici di tutte le opere coinvolte; per un lotto lungo, una lista paginata con «mostra tutte le N selezionate».
6. **Errore o annullamento.** «Annulla» prima del commit non cambia dati. Se un aggiornamento fallisce, il riepilogo deve distinguere con precisione **nessuna modifica** da **alcune modifiche confermate**; la prima fase dovrebbe preferire un'unica transazione atomica per il lotto.

## 4. Flusso UX ipotetico, non approvato

**Percorso minimo:** Opere → filtri esistenti → «Seleziona opere» (modalità esplicita) → selezione puntuale → «Rivedi un campo» → campo e operazione (sostituisci valore dichiarato / compila solo se vuoto) → anteprima per riga → esclusioni manuali → riepilogo finale → conferma consapevole → risultato con contatori e collegamenti alle schede.

- La modalità di selezione è separata dalla normale tabella e può essere abbandonata senza perdita dei filtri. Evitare azioni distruttive vicine a «Modifica».
- Selezione iniziale **per riga**, non «tutti i risultati di ogni pagina» senza confini. Un'eventuale azione «seleziona tutti i risultati filtrati» richiede conteggio, identificazione dell'ambito, esclusioni e un'ulteriore conferma, da valutare solo in fase successiva.
- **Prima fase proposta:** consentire soltanto **un campo alla volta** e preferibilmente tecnica o supporto; sostituzione di un valore dichiarato solo su righe in cui il valore corrente corrisponde. Anno, descrizione libera, dimensioni, booleane firmato/verniciato/incorniciato e prezzi restano fuori dalla fase minima. Non precompilare con il testo della prima riga.
- L'anteprima mostra per ogni opera **codice, titolo, valore corrente, nuovo valore, azione, eventuale ragione di esclusione**. «Vuoto» e «non modificato» sono distinti. Una revisione batch non equivale a una perizia.
- Conferma con formula esplicita: «Modifica il campo Supporto in 9 opere». Niente invio da Enter in un campo di testo; pulsante conferma separato con focus prevedibile.
- Stato senza colore come unica indicazione, controlli da tastiera, checkbox con nome associato, screen reader con annunci non ripetuti, zoom 200%, elenco adattivo in finestra ridotta. Consentire l'apertura della scheda completa per un dubbio e tornare alla selezione.
- Un lotto vuoto, con valori identici o con riga ormai non modificabile produce una spiegazione; mai un successo apparente con zero righe aggiornate.

## 5. Dati, regole e API — esclusivamente descrittivi

**Requisiti certi dell'app:** UUID locali, codice archivio univoco, backend Rust autorevole, SQLite e backup completo verificato, nessun account/cloud obbligatorio; le vendite/certificati storici hanno legami e invarianti che una correzione descrittiva non deve alterare. La lingua di sistema e il fallback inglese sono già descritti in [ECONOMY-I18N](../../ECONOMY-I18N.md).

**Ipotesi di stato temporaneo** (nessuna tabella obbligatoria nella prima fase):

~~~text
BatchMetadataDraft {
  archive_context: opaque_local_id,
  selected_artwork_ids: UUID[],
  field: "technique" | "support",
  operation: "fill_if_empty" | "replace_exact",
  expected_old_value: string | null,
  proposed_value: string,
  per_row_exclusions: UUID[],
  expected_record_revisions: map<UUID, opaque_revision>
}
BatchPreviewRow {
  artwork_id, artwork_code, artwork_title,
  old_value, proposed_value,
  result: "will_change" | "unchanged" | "excluded" | "conflict" | "not_found",
  has_historical_certificates: boolean
}
~~~

**Contratto concettuale:** previewBatchMetadata(draft) → {rows, counts, preview_token}; applyBatchMetadata(preview_token, expected_scope) → {updated_ids, unchanged_ids, refused_ids} **oppure nessun cambiamento e un conflitto**. Non implementare API senza audit dei campi monitorati. La preview **non autorizza** l'applicazione se l'archivio cambia dopo il calcolo: ricontrollo atomico lato Rust di identità, revisioni, valore atteso e vincoli; selezione immutabile al momento del commit.

**Atomicità scelta da valutare:** per la fase minima, **tutte o nessuna** delle modifiche consentite nello stesso database/transazione. È più semplice da spiegare; per archivi grandi occorre misurare durata, lock e limiti dimensione lotto. Una modalità parziale futura non deve essere silenziosa: conferma per sottoinsieme ed elenco degli errori da conservare localmente. Evitare una chiamata ripetuta a save_artwork con schede complete perché può sovrascrivere campi che l'utente non ha scelto. Nessun merge automatico né ripetizione del commit dopo timeout senza sapere l'esito.

**Documenti storici:** non modificare snapshot dei certificati, vendite, incassi o PDF già esportati. Le viste basate su dati correnti possono invece mostrare il nuovo valore: segnalarlo. Se TECH-010 venisse adottata, riutilizzare quel registro senza duplicarne uno; altrimenti non promettere undo per-record oltre un ripristino integrale. L'anteprima, da sola, non è un backup.

## 6. Offline, impostazioni, import/export, privacy e compatibilità

- **Offline e indipendenza:** elaborazione esclusivamente locale, nessun endpoint, autenticazione, AI, abbonamento, upload o sincronizzazione. Le preferenze di selezione durano la sessione; non servono profili cloud.
- **Impostazioni:** nessun interruttore globale in fase minima; comando avanzato volontario nell'archivio. Un eventuale limite massimo per lotto si stabilisce dopo benchmark. Preferire campo/operazione selezionati ogni volta, non ricordare azioni potenzialmente pericolose come default.
- **Multilingua:** lingua sistema con fallback inglese, override dove già previsto, dizionari it/en/es/fr/de. Tradurre comandi, errori e stati, **non** termini inseriti dall'artista, codici, UUID o contenuto delle opere. Etichette e annunci accessibili. Separare i valori «vuoto», «non disponibile», «nessun cambiamento» anche nel fallback.
- **Export/import:** nessun nuovo formato di import. I CSV esistenti esportano i **dati correnti** dopo il commit; gli export e PDF precedenti non cambiano. Backup completo nella cartella scelta include i valori aggiornati; un backup precedente conserva lo stato di allora e si ripristina integralmente, **non** funge da undo selettivo. Un riepilogo di revisione CSV opzionale e senza foto è da valutare *solo in fase separata* con chiara avvertenza sui dati catalografici che potrebbero essere riservati.
- **Privacy:** le descrizioni e i dati di opere possono essere confidenziali; niente registri remoti, clipboard automatica, telemetria, esportazioni spontanee o copia dei contenuti nei log applicativi. Una schermata visibile a ospiti deve evitare anteprime di schede non selezionate.
- **Compatibilità:** prima fase limitata a campi già presenti; se servono revisioni o tabella di audit, progettare una migrazione SQLite transazionale e testare backup e archivi vecchi. Su versioni precedenti il catalogo deve restare interpretabile; nessun cambio dei codici o delle relazioni FK. Windows/macOS/Linux, filesystem con percorsi scelti liberamente per i backup, niente dipendenza dalla rete.

## 7. Alternative, dipendenze, rischi e complessità

**Alternative senza codice:** usare gli editor individuali già disponibili per pochi record; preparare una checklist delle correzioni e utilizzare i filtri esistenti; valutare TECH-030 per individuare termini incoerenti senza aggiornare nulla; esportare un CSV di controllo per revisione esterna (non reimportarlo come verità). Prima di sviluppare, osservare almeno **3–5 professionisti** con lotti reali e cronometrare 15–30 correzioni.

**Dipendenze logiche:** protezione delle modifiche obsolette ispirata a TECH-033 (non dichiarata implementata), controlli transazionali Rust, ricerca/filtri attuali, possibile integrazione futura TECH-010 e TECH-030. **Nessuna PR tecnica viene aperta o modificata da questa scheda.**

**Rischi:** errore su decine di opere, terminologia resa falsa dall'uniformazione, pubblicazioni nuove incoerenti con certificati vecchi, conflitti con modifiche simultanee, salvataggi parziali, interfaccia troppo complessa, potenziale perdita di informazione storica. Mitigazioni: campo singolo e valori attesi, anteprima completa, esclusione per riga, revisione obbligata per conflitti, transazione atomica, campi sensibili esclusi e nessuna promessa di reversibilità.

**Complessità stimata:** **media** per l'interazione e l'elenco, **medio-alta** per condizioni di commit e test dei conflitti; aumenta molto se si introduce ripristino per-record o gestione file. Questa è una stima qualitativa, non un preventivo.

## 8. Criteri di accettazione ipotetici e fasi possibili

**Criteri da usare solo se la proposta fosse approvata:**

1. Con 20 opere filtrate, 7 selezionate, la preview nomina **esattamente** 7 codici/UUID e nessun cambiamento si applica prima della conferma.
2. Su 7 righe, 2 senza valore, 1 esclusa e 4 con valori diversi, «compila solo vuoti» aggiorna unicamente le righe eleggibili previste; i contatori coincidono con il database reale.
3. Modificare o eliminare una scheda tra preview e conferma non sovrascrive nulla in modo invisibile; in modalità atomica, il conflitto lascia l'intero lotto invariato e mantiene la bozza per revisione.
4. Nessuna variazione a UUID, codice, stato commerciale, prezzo, coefficiente, fotografie, vendita, importi, quote, certificati e snapshot storici.
5. Annullamento, errore di validazione, disco pieno o interruzione non producono successi ingannevoli né lotti parziali non dichiarati.
6. L'operazione non usa rete e conserva la portabilità/backup attuali; dati precedenti leggibili senza inventare revisione storica.
7. UI fruibile da tastiera e screen reader, con zoom 200%; sistema IT/EN/ES/FR/DE e fallback inglese senza tradurre il testo delle opere.
8. In prova su archivi sintetici medio-grandi, il tempo di anteprima e commit resta misurabile e accettabile senza freeze non annunciati; definire soglie solo dopo benchmark.

**Fasi da valutare, nessuna assegnata:**
- **Fase 0 — ricerca:** 3–5 sessioni con professionisti, confronto editor singolo/checklist, quantificare gli errori e verificare se la necessità esiste.
- **Fase 1 — minimo protetto:** selezione manuale di record esistenti; un solo campo (tecnica/supporto), confronto e anteprima completa, conferma transazionale, conflitti bloccanti, senza import né storico.
- **Fase 2 — qualità professionale opzionale:** supporto ai vocabolari se TECH-030 scelto; selezioni salvate se TECH-008 scelto; revisione più ampia soltanto per campi verificati e con rischi documentati.
- **Fase 3 — ripristino selettivo soltanto eventuale:** riuso di una cronologia se TECH-010 fosse approvata, senza inventare undo retroattivi.

## 9. Questioni aperte per revisione personale

- Quanti utenti correggono realmente 15–30 schede alla volta? Un filtro + checklist e l'editor attuale sono già sufficienti?
- È più frequente **compilare campi vuoti** o **rettificare valori esatti**, e quali campi sono sicuri da trattare insieme?
- È preferibile limitare il primo rilascio a tecnica e supporto, lasciando anno/dimensioni/descrizioni alla scheda singola?
- Un lotto bloccato integralmente per una sola riga cambiata è comprensibile, o serve un modello di sottoinsiemi confermati? La scelta richiede prove e garanzie backend.
- Quanto peso attribuire all'avviso di divergenza rispetto a certificati/brochure storici? Deve essere solo informativo?
- Quali dati sintetici e quali dimensioni d'archivio sono rappresentativi? Quale beneficio misurato giustificherebbe la maggiore complessità?

**Esito documentale:** proposta **non approvata**, senza sviluppo richiesto. Nessuna modifica a codice, test, workflow, configurazioni, dipendenze o documenti dell'altro filone.
