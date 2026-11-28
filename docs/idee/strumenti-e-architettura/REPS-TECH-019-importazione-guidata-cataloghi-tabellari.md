# REPS-TECH-019 — Importazione guidata di cataloghi tabellari esterni (CSV/TSV)

- **ID stabile:** REPS-TECH-019
- **Data:** 2026-11-21
- **Stato:** **Proposta — da valutare**
- **Filone:** Strumenti e architettura
- **Priorità proposta:** **P1–P2**, soltanto ipotetica per utenti con archivi esistenti in fogli elettronici
- **Natura:** analisi e specifica descrittiva, senza approvazione o mandato di sviluppo

## 1. Motivazione, destinatari e riscontri

Un artista può possedere 500 opere annotate in un foglio Calc/Excel; una galleria può avere due elenchi separati di opere e contatti. Ricopiare ogni riga in Repertica rende costosa la migrazione e può introdurre errori su codici, misure e anagrafiche. Il bisogno è **importare dati da fogli già posseduti senza dipendere dal software di origine**, ma solo dopo aver visto come saranno interpretati.

**Riscontri verificati su `main` (21 novembre 2026):**
- [README](../../../README.md), [MVP](../../MVP.md), [portabilità](../../DATA-PORTABILITY.md): app locale SQLite/Tauri, backup completo verificato a cartella scelta; **nove registri CSV di sola esportazione** (`artworks`, `gallery`, `collectors`, `certificates`, `sales`, `allocations`, `balances`, `expenses`, `receipts`), non reimportabili.
- [`src-tauri/src/db.rs`](../../../src-tauri/src/db.rs): schema **v10**; identificativi UUID; `artworks.code` univoco senza distinzione maiuscole/minuscole; vincoli sulle vendite e sui certificati. [`src/app/models.ts`](../../../src/app/models.ts) distingue input di opera, collezionista, vendita e documenti.
- [Issue #5](https://github.com/Cesare-ml/Repertica-Studio/issues/5): migrazione **specifica** dal Gestionale Opere, già prevista con mapping, controlli e rollback, ma **non** import generico da CSV arbitrari di fogli terzi.
- Non è stato trovato un importatore tabellare generico nelle funzionalità documentate. L'effettiva domanda degli utenti **non è ancora misurata**: la priorità resta proposta, non un bug confermato.

**Destinatari:** artisti che migrano da spreadsheet, gallerie che incorporano inventari storici, piccoli archivi con elenchi compilati da collaboratori. Non riguarda l'inserimento manuale di poche opere.

## 2. Confini e non duplicazione

- **TECH-001** definisce il *pacchetto aperto canonico* per esportazione fedele e reimportazione di dati già strutturati. **TECH-019** studia invece l'*acquisizione da tabelle non standard*: intestazioni non uniformi, codici esterni, celle ambigue, mapping manuale e prova di import. Potrà eventualmente produrre gli stessi record validati, **non definisce un secondo formato Repertica**.
- **Issue #5** riguarda la sorgente *Gestionale Opere*, con propri identificativi e relazioni: qui nessun codice né dati del gestionale privato, e nessuna promessa di sostituire quell'importazione.
- **UX-018** tratta inserimento sequenziale manuale, non parser o mapping di file; **UX-005/011/021** definiscono rispettivamente edizioni, cicli e acquisizioni, che **non** vengono inferite da colonne libere; **UX-020** tratta identificativi stampati, non import.
- **TECH-018** rende più robuste le **esportazioni** CSV esistenti, non ne assume la reimportabilità. **TECH-011/012** studiano profili e migrazioni di schema dell'archivio: l'import tabellare è un processo separato.
- **Esclusioni iniziali:** vendite, incassi, spese, ripartizioni, certificati, diritti, provenance probatoria, fotografie, documenti e relazioni non verranno importati in automatico. Evitare di creare contabilità o certificati da una semplice cella.

## 3. Scenari d'uso e flusso UX ipotetico

1. **Catalogo studio:** file `opere.csv` con colonne «Codice», «Titolo», «Tecnica», «Altezza (cm)», «Larghezza (cm)». L'utente seleziona la tabella, conferma separatore, codifica, intestazioni e collega le colonne ai campi Repertica. Il sistema mostra 250 righe valide, 12 da correggere, 3 potenziali duplicati; **nessuna scrittura** prima della conferma.
2. **Anagrafica contatti:** import separato con nome e recapiti. Campi non riconosciuti non vengono persi silenziosamente: sono marcati «non importati» ed eventualmente conservabili nel report scelto dall'utente, con avviso privacy.
3. **Ambiguità internazionale:** «1,50», «1.50», «1 500», date come «03/04/2020», `0018` e accenti non devono essere interpretati o convertiti senza regola esplicita. Si selezionano unità, notazione decimale e significato della data; per codici con zeri iniziali si preserva la stringa testuale se già leggibile dal CSV.
4. **Conflitti:** due righe con lo stesso codice, o un codice già presente nel database, appaiono in un confronto affiancato. La somiglianza del titolo è un **indizio**, non prova d'identità. Scelte esplicite: correggere, ignorare o interrompere; mai aggiornare automaticamente record esistenti.
5. **Interruzione:** se il file cambia, la cartella non è più disponibile o finisce lo spazio, l'import viene annullato senza archivio parziale. Il report distingue le righe *analizzate* dalle righe effettivamente *create*.

**Percorso proposto:** `Impostazioni → Dati e portabilità → Importa da tabella` → scelta locale del file → analisi di intestazione e anteprima campione → associazione colonne/campi → regole unità e stati → preflight completo con errori per riga/cella → scelta destinazione protetta → conferma esplicita → report locale con contatori e ID creati. Un primo rilascio potrebbe richiedere un **archivio nuovo e vuoto**; l'aggiunta a uno già popolato è una fase da valutare separatamente.

**Accessibilità:** tabella di anteprima percorribile da tastiera e screen reader, errori con numero riga/colonna e testo (non solo colore), possibilità di filtrare gli errori, focus mantenuto dopo la correzione, scala 200%, messaggi sintetici di avanzamento e annullamento. Anteprima numerata, **mai** mostrare «importato» finché il commit non è riuscito.

## 4. Specifica funzionale e dati (solo ipotesi)

- **Fase minima:** CSV/TSV locali UTF-8 con o senza BOM; selezione esplicita di separatore (`;`, `,`, tab), delimitatore testo, riga intestazione e modalità righe multilinea; supporto ad altre codifiche soltanto con scelta e anteprima affidabili. **XLSX/ODS come opzioni future**, se motivate: mai eseguire macro, formule o collegamenti remoti, valutare parser isolati e relativi limiti.
- **Mapping dichiarativo:** `SourceProfile {source_kind, header_signature, encoding, delimiter, header_row, field_map, units, date_format, status_map, schema_version}`, conservabile localmente **solo su scelta**. I profili non contengono righe né dati dei clienti; le colonne sconosciute restano escluse e vengono conteggiate.
- **Staging descrittivo:** `ImportSession {id, source_hash, target_archive_id, target_revision, entity_kind, row_count, profile_hash, diagnostics, intended_actions, expires_at}`; `RowFinding {row_number, column, severity, code, message_key}`. Hash/versione della sorgente e revisione dell'archivio devono essere ricontrollati alla conferma. Sono **ipotesi di struttura**, non tabelle o API attualmente esistenti.
- **API concettuali, non implementate:** `inspectTable(file) -> StructurePreview`; `validateMapping(profile, sample) -> Findings`; `dryRunRows(file, profile, target) -> ImportPlan`; `applyPlan(plan_id, confirmation) -> ImportReport`.
- **Integrità:** mai fidarsi di input esterno per SQL o nomi di file; validare lunghezza e tipo di ogni campo, valori ammessi e unicità del codice, date, misure positive, limiti Unicode e riferimenti. Conflitti bloccanti non vengono «riparati» per supposizione. L'ID sorgente può essere conservato come traccia *solo se* viene progettato un campo esplicito e la sua provenienza è dichiarata.
- **Identità e riesecuzioni:** generare UUID nuovi quando la tabella esterna non ne fornisce uno valido e autorizzato; conservare un report locale di corrispondenza `riga/id sorgente → UUID Repertica` per audit e riuso consapevole. Non basarsi su nomi/titoli per deduplicare. Una riesecuzione **non** deve creare doppioni silenziosi: stessa sorgente rilevata, confronto prima della conferma.
- **Transazione e rollback:** nessuna scrittura durante preview/dry-run; validazione totale in staging, snapshot/preflight del target e commit atomico (eventualmente in archivio nuovo). Un crash non deve lasciare vendite, immagini o schede parziali. La cancellazione di dati preesistenti non fa parte del flusso.

## 5. Offline, privacy, sicurezza, impostazioni, multilingua

- Tutto su dispositivo locale: niente account, cloud, API remota, fogli Google obbligatori, tracciamento o invio automatico. File scelti liberamente su disco, USB o NAS montato. Nessuna dipendenza da applicazioni Office installate per il CSV.
- Il contenuto del CSV può contenere **nomi, recapiti e informazioni commerciali**. Il report diagnostico minimale deve preferire codici di errore e numero riga, **non** copiare per default l'intera riga. Eventuali anteprime con dati personali rimangono dentro l'app; nessun log con i valori. Cache/staging temporanei con permessi prudenti, cancellazione sicura nei limiti del filesystem.
- CSV e formule: trattare celle iniziate da `=`, `+`, `@`, tab o altri prefissi sospetti **come testo non eseguibile**, con indicatore e decisione per campo; nessuna valutazione di formule, DDE, hyperlink attivi o download da URL. Non presumere che una mitigazione di export risolva automaticamente l'import. Guida [OWASP: CSV Injection](https://owasp.org/www-community/attacks/CSV_Injection).
- Limiti configurabili/sensati: dimensione file, numero righe, lunghezza cella, tempo/memoria, dimensione anteprima, profondità o numero di contenuti di eventuali futuri XLSX/ODS. Un CSV molto grande non deve essere caricato tutto in memoria. Non seguire path o symlink indicati nel file.
- Lingua UI dal sistema operativo e fallback **inglese**; intestazioni sorgente e note originali **mai tradotte**. Errori/validazioni localizzati per codice stabile; notazioni e regole di parsing visibili indipendentemente dalla lingua UI. Profili riusabili tra lingue.
- Impostazioni facoltative: formato di lettura prescelto, profilo mapping, unità dei campi, modalità import iniziale, destinazione del report e cancellazione del profilo; nessuna impostazione trasforma automaticamente un CSV in modifica permanente.

## 6. Compatibilità, dipendenze, alternative e rischi

**Dipendenze progettuali:** comandi e validazioni Rust esistenti, policy dei codici archivio, vincoli SQLite v10, schema degli input, gestione transazionale/staging e preflight dei backup. Non è necessaria alcuna dipendenza nuova per *questa analisi*: la scelta di parser e librerie è rinviata alla revisione. Un formato di profilo di mapping dev'essere versionato quando e se sarà adottato.

**Alternative:** (A) usare manualmente i moduli attuali: ottimo per poche opere, poco pratico per grandi elenchi; (B) convertire prima i file con script esterni nel pacchetto canonico TECH-001: flessibile ma richiede competenze tecniche; (C) un importatore generico XLSX completo subito: più confortevole ma aumenta superficie d'attacco, varianti e costi di manutenzione. Il CSV/TSV con anteprima è la prima ipotesi più sobria.

**Rischi principali:** importi, date e misure interpretati male; codici duplicati/zeri persi; consenso non informato su dati personali; righe saltate silenziosamente; mismatch tra preview e commit; dati ostili e file enormi; differenze tra Excel e Calc; import parziale dopo crash; confusione fra catalogazione e certificazione storica. Mitigazioni: campi espliciti, preflight di **tutte** le righe, report, validazione autoritativa, transazioni e suite di prova con input ostili. **Complessità:** medio-alta per CSV di sole opere/contatti, alta per merge, media e XLSX/ODS.

**Riferimenti di interoperabilità:** [W3C CSVW — Metadata Vocabulary](https://www.w3.org/TR/tabular-metadata/) per tipi e metadati opzionali, senza imporre CSVW ai file degli utenti; [W3C — Model for Tabular Data](https://www.w3.org/TR/tabular-data-model/) per robustezza di encoding e parsing; [OWASP CSV Injection](https://owasp.org/www-community/attacks/CSV_Injection) per minacce di fogli elettronici.

## 7. Criteri di accettazione / prove future (non eseguite)

1. Import di almeno 1000 opere di prova e di un'anagrafica con Unicode, apostrofi, multilinea, titoli simili e codici con zeri: **conteggi, codici e valori coincidenti** con il piano confermato; nessuna vendita o certificato fabbricato.
2. Mapping `width_cm`/`height_cm` e stati validi controllato; ambiguità come `1,234`, `03/04/2020`, unità `mm` vs `cm` e codice `00042` producono diagnosi o conversioni esplicitamente accettate, mai interpretazioni silenziose.
3. Righe duplicate, codice già in uso, campo obbligatorio assente, file mutato dopo la preview, testo malformato e valori oltre limite bloccano il commit o richiedono una scelta riga per riga; nessuna modifica occulta di opere esistenti.
4. Crash, errore I/O, disco pieno, NAS scollegato, file ostile, 100k righe: nessun mezzo import visibile come completo; memoria limitata e tempi/risposte misurati su macchine rappresentative.
5. Riesecuzione dello stesso file con mapping invariato non produce doppioni senza avvertimento; report di prova e report finale dichiarano esplicitamente righe incluse/escluse e UUID creati.
6. Nessuna formula eseguita; dati non inviati in rete, profili senza dati personali, anteprime e messaggi accessibili da tastiera e screen reader, lingua OS/fallback inglese verificati.
7. Esportazione successiva dei record con i CSV **già supportati** e loro presenza nel backup verificato; **non** si dichiara che i nove CSV di export siano automaticamente reimportabili.

## 8. Fasi ipotetiche e questioni per revisione del proprietario

1. **Fase A — ricerca utenti + specifica minima:** raccogliere **solo esempi sintetici/autorizzati** di colonne e lingue, concordare ambito (opere, contatti), regole dei codici e politiche conflitti; nessuna implementazione autorizzata.
2. **Fase B — eventuale import CSV/TSV in archivio nuovo e vuoto:** mapping, anteprima, validazione completa, conferma, report e test di recuperabilità.
3. **Fase C — eventuale aggiunta a archivio esistente:** snapshot preventivo, confronto duplicati, azioni esplicite, immutabilità delle entità già storicizzate.
4. **Fase D — solo se richiesta:** XLSX/ODS locale, immagini da cartella selezionata, template aperti e CSVW facoltativo; mai macro, immagini da URL o sincronizzazione obbligatoria.

**Domande aperte al proprietario:** quanti utenti arrivano da fogli con più di 100 opere? Il primo rilascio dovrebbe creare sempre un archivio separato? Quali due entità importare per prime? Con quale frequenza il codice esterno deve restare identico? Servono conversioni di `mm` e pollici? Importare immagini associate prima o dopo la validazione dei metadati? Serve XLSX subito o basta CSV? Con quali regole distinguere *contatto* da *collezionista*? Come rappresentare omissioni e qualità delle fonti senza attribuire autenticità?

**Certezze:** vincoli, esportazioni e issue sopra verificati su `main`. **Ipotesi:** ogni nuovo menu, tipo, API, formato e fase descritti. **Decisioni:** tutte riservate al proprietario. **Stato invariato: Proposta — da valutare.** Nessun codice, issue implementativa, PR operativa, approvazione o merge.
