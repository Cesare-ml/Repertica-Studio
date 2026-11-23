# REPS-TECH-068 — Esportazione PDF delle brochure con memoria limitata anche per selezioni grandi

- **ID stabile:** REPS-TECH-068
- **Data della ricognizione:** 2026-11-23
- **Stato:** **Proposta — da valutare**
- **Filone:** Strumenti tecnici e architettura; solo analisi e documentazione
- **Priorità proposta:** **P2 condizionata** a misure di picco memoria/fallimenti riproducibili su brochure da 50–100 opere; P3 o nessuna modifica se il renderer attuale è adeguato. **Non approvata né assegnata allo sviluppo.**
- **Ambito:** gestione delle risorse durante la generazione di un PDF brochure **già disponibile**, senza modificare contenuti, numero massimo di opere, dati di archivio o modello commerciale.

## 1. Problema, destinatari e riscontri sul codice

Un artista, una galleria o un archivista può voler esportare una brochure con decine di opere fotografate ad alta risoluzione. Un PDF elaborato integralmente in RAM può richiedere memoria proporzionale alla selezione e alle fotografie, risultando lento o fallendo su computer poco potenti, soprattutto se altre applicazioni sono aperte. **È un rischio architetturale da misurare, non un incidente osservato.**

**Osservazioni statiche sul branch `main` (23 novembre 2026):**

1. [`docs/BROCHURES-OFFLINE.md`](../../BROCHURES-OFFLINE.md) conferma editor e generatore PDF A4 offline nativi Rust, fotografie e bozze locali SQLite. Il PDF esportato resta un derivato esterno, non uno snapshot archiviato automaticamente.
2. [`src-tauri/src/brochures.rs`](../../../src-tauri/src/brochures.rs) ammette fino a **100 opere** in `validate`. `load_photo` legge/decodifica un'immagine, impone limiti di dimensioni/file, crea un derivato RGB/JPEG con lato massimo 2400 e qualità 86: sono protezioni già presenti, **da preservare**.
3. `export_brochure_pdf` costruisce un `Vec<Page>` completo: ogni `Page` può trattenere i byte JPEG. Solo dopo tutte le pagine chiama `render_pdf(pages)`. Il renderer accumula a sua volta `Vec<Vec<u8>>` di oggetti PDF e `Pdf::finish` costruisce il `Vec<u8>` finale. Il risultato viene scritto con `fs::write(&destination, &pdf)`. Il codice mostra quindi **accumulazione dell'intero output** prima della scrittura; non misura però quanto incida in RAM sulle macchine degli utenti.
4. Un errore su fotografia assente/invalidata viene rilevato mentre si costruiscono le pagine: per selezioni lunghe può arrivare dopo lavoro già svolto. Non confondere questa osservazione con una prova di leak, crash o corruzione.
5. Non risultano nel flusso esaminato benchmark pubblicati del picco RSS/working set di esportazioni da 10, 50 e 100 opere su Windows/macOS, né una politica specifica di consumo RAM per il PDF. Il limite per immagine non equivale a un limite per **l'intero documento**.

**Destinatari:** artisti che creano cataloghi ampi, gallerie con molte fotografie per opera, archivi e fondazioni che lavorano su portatili, professionisti che condividono documenti via USB/NAS, contributori che devono verificare prestazioni e regressioni.

## 2. Confini e confronto anti-doppioni

- [TECH-034](REPS-TECH-034-esportazione-brochure-pdf-atomica-senza-sovrascrittura.md) affronta **collisioni del nome, sovrascritture e pubblicazione sicura del file finale**. La presente proposta affronta **solo RAM, dimensione dell'output e lavoro progressivo**; un'eventuale implementazione dovrebbe riutilizzare la politica di pubblicazione di TECH-034 invece di reinventarla.
- [TECH-062](REPS-TECH-062-progresso-annullamento-sicuro-operazioni-locali.md) definisce il **contratto trasversale** di progresso e annullamento di lavori lunghi. Qui interessa la possibilità tecnica di costruire un PDF progressivamente; non si propone una seconda infrastruttura di job.
- [TECH-032](REPS-TECH-032-pdf-brochure-unicode-font-incorporati-offline.md) studia i font; [TECH-063](REPS-TECH-063-brochure-pdf-accessibili-lettura-assistita.md) la struttura accessibile. Questa scheda **non cambia** layout, tipografia, testo, tag semantici o standard PDF.
- Le idee [Gestione e UX — PR #14](https://github.com/Cesare-ml/Repertica-Studio/pull/14) trattano selezione/contenuto delle opere, dossier e consultazione. Qui si studia **solo l'efficienza del renderer nativo**, senza modificare l'altra cartella.
- Alternativa zero-sviluppo: misurare e dichiarare limiti realistici delle brochure, con suggerimenti di suddividere un documento troppo pesante. Non alzare arbitrariamente il massimo di 100 opere.

## 3. Scenari reali da convalidare

1. **Catalogo di 80–100 opere:** un artista seleziona fotografie diverse, con testi lunghi e pagine supplementari. L'app deve completare il lavoro senza bloccare o terminare per esaurimento RAM, se il computer soddisfa requisiti esplicitati.
2. **Portatile con poca memoria:** il costo cresce col numero di fotografie; si vuole evitare di trattenere contemporaneamente le immagini codificate e tutte le copie interne del PDF quando non serve.
3. **PDF pesante su unità esterna:** la scrittura può rallentare o fallire. Un intervento sulle risorse non deve compromettere la sicurezza del file finale, oggetto di TECH-034.
4. **Fotografia mancante in fondo alla selezione:** possibile con una bozza conservata dopo modifiche all'archivio. Un preflight **in sola lettura** può segnalare gli elementi mancanti prima della conversione, ma non può garantire che il disco o il file non cambino successivamente.
5. **Brochure breve:** 2–5 opere su computer ordinario. Un nuovo percorso di output non deve peggiorare inutilmente latenza, qualità o semplicità del flusso.

## 4. Specifica funzionale e flussi UX proposti

**Fase preliminare — diagnosi senza nuova UI obbligatoria.** Misurare su build di prova il tempo di export, il picco di memoria del processo, byte del PDF e pagina/opera a cui si verifica un eventuale errore. Usare dataset sintetici **senza fotografie né contatti personali**, su Windows e macOS, includendo fotografie grandi ma già ammesse dai limiti correnti. Non proporre un nuovo comando prima di avere misure.

**Flusso A — esportazione ordinaria.** Nell'editor brochure l'utente sceglie `Esporta PDF`, decide percorso e nome come oggi. Il backend verifica bozza e riferimenti necessari, compone il documento con consumo RAM **misurabilmente contenuto**, pubblica il PDF solo a completamento effettivo e restituisce un esito verificabile. La bozza e i record di vendita non cambiano. Nessun cloud, login o server.

**Flusso B — risorse insufficienti o file mancante.** Se un preflight trova un'immagine assente, indicare **l'opera specifica** e interrompere prima di scrivere il risultato definitivo. Se esaurisce memoria/spazio o un volume viene scollegato, distinguere `errore di generazione`, `errore di scrittura` e `pubblicazione non riuscita` senza dichiarare un PDF valido. Eventuali parziali e cleanup devono rispettare il progetto di TECH-034; non promettere cancellazione universale su NAS.

**Flusso C — molte opere.** Se i benchmark mostrano che il lavoro dura a lungo, valutare un indicatore di fase reale ed eventualmente annullamento **solo nei punti sicuri**, riusando TECH-062. Non mostrare una percentuale inventata o un pulsante che uccide il thread nel mezzo della pubblicazione.

**Strategie tecniche *alternative* da confrontare, non da scegliere ora:**

- **A. Mantenere l'architettura**, dopo ottimizzazioni locali comprovate: limitare copie temporanee e ricodifiche duplicate, senza nuova dipendenza. È la scelta conservativa se il picco è accettabile.
- **B. Writer PDF incrementale a file temporaneo** con tabella degli offset e riferimenti agli oggetti, emettendo una pagina alla volta: costi di memoria indipendenti dalla dimensione complessiva del PDF **salvo strutture piccole** (offset/indici), mantenendo un PDF finale valido. Le fasi di serializzazione non devono alterare z-order, disposizione né paginazione.
- **C. Staging di risorse su disco** e costruzione differita: riduce il picco RAM ma aumenta I/O, spazio temporaneo, cleanup e complessità; occorre preflight prudente. Non conservare file temporanei nell'archivio attivo come dati permanenti.

Per ogni scelta, prima definire un **profilo prestazionale misurato** e prove con immagini diverse. Evitare la promessa di RAM assolutamente costante: gli offset PDF, i metadati, il carico GUI, la singola immagine decodificata e la conversione JPEG occupano comunque memoria.

## 5. Dati e API ipotetiche, soltanto descrittive

Prima fase **senza migrazione SQLite**. Le bozze `Brochure` e `BrochureItem` rimangono la fonte, con UUID delle opere e selezione fotografica già presenti.

| Entità concettuale | Campi/ruolo | Invarianti |
| --- | --- | --- |
| `ExportPlan` | `brochure_id`, `selected_items`, `expected_page_count?`, `missing_media[]`, `destination` | Sola lettura; il preflight non è un blocco permanente sui file |
| `PdfRenderSession` | `operation_id`, `page_index`, `object_offset_index`, `temporary_output`, `bytes_written` | Staging temporaneo confinato alla destinazione; mai fonte autorevole |
| `RenderMetrics` | `page_count`, `photo_count`, `render_duration_ms`, `peak_memory_bytes?`, `pdf_size_bytes` | Diagnostica locale; niente nomi, titoli, immagini o contatti nei log condivisibili |
| `ExportResult` | `status`, `output_path?`, `error_code?`, `completed_at?` | Successo solo dopo la pubblicazione verificata, coerente con TECH-034 |

API *esemplificative e non esistenti*: `inspectBrochureMedia(brochureId) -> Finding[]`, `renderBrochureIncrementally(plan, sink) -> RenderResult`, `publishBrochureOutput(staged, path) -> Outcome`. Non fissano crate, protocollo, API IPC o struttura del database.

## 6. Offline, privacy, sicurezza, multilingua e configurazione

- **Offline-first:** generazione, benchmark di regressione e verifica strutturale operano localmente. Dipendenze eventuali devono essere redistribuibili, compatibili con la licenza del progetto e senza rete. Nessun account, telemetria automatica, servizio PDF esterno o abbonamento.
- **Privacy:** il PDF può contenere dati di contatto inseriti nella brochure. Non includere testi, nomi, percorsi assoluti o anteprime nei log prestazionali; eventuali report sono locali ed esportati solo su richiesta. Non esporre fotografie tramite MCP.
- **Integrità:** non modificare originali Media Center, UUID, snapshot dei certificati o vendite. Confinare i temporanei, rispettare protezioni contro symlink e traversal già presenti e predisporre pulizia prudente dopo errore. Nessuna pubblicazione parziale col nome finale: requisito da coordinare con TECH-034.
- **Impostazioni:** nessuna nuova preferenza obbligatoria, quota RAM o limite di pagine da aggiungere per default. Se emergesse bisogno di modalità `risparmio memoria`, valutarla prima con dati reali e mantenere l'export semplice. Il percorso finale resta liberamente scelto.
- **Lingue:** etichette e codici d'errore localizzabili secondo la lingua di sistema (IT/EN/ES/FR/DE esistenti), con fallback inglese. **La lingua della brochure scelta dall'utente** non deve cambiare con quella dei messaggi UI.
- **Portabilità/compatibilità:** PDF standard apribile con lettori indipendenti su Windows/macOS/Linux; non introdurre dipendenze runtime cloud. L'export rimane un derivato esterno non automaticamente incluso nel backup; nessuna modifica al contratto dei CSV o all'importazione del database.

## 7. Dipendenze, alternative, rischi e complessità

- **Dipendenze concettuali:** renderer Rust e libreria immagini già presenti, contratto di pubblicazione sicura di TECH-034, modello di operazioni lunghe di TECH-062 **solo se necessario**. Nessuna nuova dipendenza approvata.
- **Complessità:** **media** per diagnosi e riduzione di copie; **medio-alta/alta** per writer PDF incrementale corretto su tutti i layout, font futuri, xref, gestione I/O e recovery. Un cambiamento del renderer potrebbe complicare TECH-032/063: progettare interfacce che non ne impediscano l'evoluzione.
- **Rischi:** regressioni di impaginazione, immagini duplicate, ordine errato di oggetti, PDF troncati, crescita imprevista di temporanei, introduzione di un worker troppo complesso per 100 opere, differenze fra filesystem, test prestazionali instabili. Conservare una baseline dei PDF con controlli visivi e funzionali.
- **Alternative:** (1) nessuno sviluppo se misure soddisfacenti; (2) snellire i buffer attuali; (3) generazione a flusso; (4) dividere volontariamente una brochure in documenti più piccoli **solo come guida**, non come modifica automatica della selezione.

## 8. Test proposti e criteri di accettazione

1. **Baseline riproducibile** su selezioni sintetiche da **5, 25, 50 e 100 opere** con immagini piccole, grandi e descrizioni multipagina; rilevare picco di memoria, durata, byte output e spazio temporaneo su Windows e macOS. Documentare le condizioni macchina; non dichiarare miglioramenti senza dati comparabili.
2. Se si sceglie lo streaming, dimostrare che **il picco di memoria aggiuntiva non cresce linearmente con il numero delle pagine/JPEG già emessi** nelle prove, salvo l'immagine/pagina corrente e indici limitati; fissare soglie realistiche solo dopo la baseline.
3. Verificare struttura PDF (xref, numero pagine, visualizzazione su almeno due lettori), ordine delle opere, foto selezionata, orientamento, layout e testo; confrontare a campione output nuovo e attuale, distinguendo differenze di byte non sostanziali.
4. Simulare immagini mancanti, corrotti, raggiungibilità persa, cartella non scrivibile, disco pieno, interruzione controllata e file finale preesistente; nessuna modifica a PDF precedenti, bozza, opere o fotografie. Requisiti di pubblicazione coerenti con TECH-034.
5. Dimostrare che la generazione **non trasmette** fotografie o dati commerciali e non rende obbligatori account o connessioni. Messaggi comprensibili, tradotti e accessibili da tastiera.
6. Verificare che limitazioni attuali (100 opere, limiti immagini, scelta di lingua, tre layout, nessuna modifica a SQLite) siano invariate salvo futura decisione esplicita.

La verifica statica identifica un'opportunità; **nessun benchmark, test di carico, prova nativa o intervista è stato eseguito in questa ricognizione**.

## 9. Fasi e priorità proposte

- **Fase 0 — ricerca (preferita):** misurare memoria e durata su 5–100 opere, verificare se la RAM è davvero un limite per gli utenti e annotare risultati. **Nessuna modifica autorizzata ora**.
- **Fase 1 — opzione minima:** se utile, documentare limiti/diagnostica o ridurre buffer superflui; misurare regressioni e qualità.
- **Fase 2 — solo se motivata:** progettare writer incrementale e staging concordato con TECH-034, integrare eventuali eventi TECH-062 senza duplicarli.
- **Fase 3 — validazione:** matrice multipiattaforma, lettori PDF indipendenti, regressioni di layout e gestione errori, incluso un caso 100 opere.

**Priorità non vincolante: P2 se i test mostrano rallentamenti/crash reali su archivi utili, altrimenti P3 o nessun intervento.**

## 10. Questioni aperte per la revisione del proprietario

1. Le brochure di utenti reali superano **20–30 opere**? Su quali computer e con fotografie di quali dimensioni?
2. Qual è il picco RAM e la durata **misurati** nell'implementazione attuale a 50 e 100 opere?
3. È più importante mantenere il renderer leggero e originale, oppure investire in un writer progressivo se la differenza è misurabile?
4. Va privilegiato l'export identico per tutti i sistemi o una maggiore efficienza con eventuali differenze di struttura PDF invisibili?
5. A chi spetta la politica comune di staging/pubblicazione (TECH-034) e quando avrebbe senso mostrare progresso (TECH-062)?
6. Una semplice guida a dividere cataloghi molto ampi risolve già il bisogno senza aumentare la complessità del prodotto?

**Nessuna risposta a queste domande è presunta; nessuna idea è approvata, assegnata o pronta per implementazione.**
