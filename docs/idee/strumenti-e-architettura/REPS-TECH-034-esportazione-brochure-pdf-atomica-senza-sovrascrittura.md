# REPS-TECH-034 — Esportazione delle brochure PDF senza sovrascritture e con pubblicazione sicura

- **ID stabile:** REPS-TECH-034
- **Data della ricognizione:** 2026-11-22
- **Stato:** **Proposta — da valutare**
- **Filone:** Strumenti e architettura, esclusivamente analisi e documentazione
- **Priorità proposta:** **P1 per la protezione dei file già esistenti**, P2 per completamento atomico e resilienza multipiattaforma; priorità **non approvate**
- **Ambito:** sola affidabilità del salvataggio **PDF delle brochure già presenti**; nessuna modifica di codice, test, build, configurazioni o workflow eseguita

## 1. Problema, motivazione e destinatari

Un artista può esportare una brochure nella stessa cartella in cui conserva il PDF già inviato a una galleria, magari scegliendo involontariamente lo stesso nome. Un'esportazione deve **non distruggere documenti preesistenti** e non annunciare come concluso un documento incompleto dopo un errore di disco. Il rischio è particolarmente importante quando si lavora su cartelle condivise, USB o NAS montati, senza che Repertica debba dipendere da un cloud o da un server.

**Riscontri puntuali nel repository `main` (consultato 22/11/2026):**

1. [`docs/BROCHURES-OFFLINE.md`](../../BROCHURES-OFFLINE.md) descrive il modulo brochure **già implementato**, con editor locale, dati in SQLite e generazione PDF Rust; il documento esportato è un derivato esterno all'archivio. La brochure è diversa dal certificato storico.
2. In [`src-tauri/src/brochures.rs`](../../../src-tauri/src/brochures.rs), `validate_output` controlla l'estensione `.pdf`, la cartella esterna all'archivio, e respinge alcuni tipi di file/symlink. **Se la destinazione esiste ed è un file ordinario, il controllo non la rifiuta.** In `export_brochure_pdf` l'output viene poi scritto con **`fs::write(&destination, &pdf)`**: la modalità standard di scrittura può troncare e sostituire il contenuto di un file esistente. Non si vede in questo percorso una creazione esclusiva né un passaggio su file temporaneo con pubblicazione finale.
3. [`docs/DATA-PORTABILITY.md`](../../DATA-PORTABILITY.md) dichiara **assenza di sovrascrittura** per i CSV. Per le brochure non è lecito presumere che tale regola sia già rispettata: l'implementazione dei PDF ha un percorso diverso.
4. [`REPS-TECH-018`](REPS-TECH-018-esportazioni-csv-scalabili-atomiche.md) studia pubblicazione sicura dei **CSV**, con attenzione a interruzioni, file temporanei, volumi esterni e race. Qui il problema si applica specificamente al PDF brochure, **già costruito in memoria**, senza richiedere streaming dei registri o nuove tipologie di esportazione.
5. Le [proposte Gestione e UX — PR #14](https://github.com/Cesare-ml/Repertica-Studio/pull/14), incluso [UX-007](https://github.com/Cesare-ml/Repertica-Studio/blob/docs/idee-gestione-ux-20261121/docs/idee/gestione-e-ux/REPS-UX-007-dossier-presentazione-opere.md), trattano la selezione dei **contenuti** di dossier e presentazioni. Non definiscono le garanzie del filesystem di **questo** comando Rust già esistente.

**Livello di evidenza:** il comportamento potenzialmente distruttivo è deducibile direttamente dal percorso di codice, **senza** aver effettuato una prova desktop, una riproduzione dell'errore o osservato perdita di dati reale. La protezione dei symlink presente in `validate_output` è un presidio utile, non una garanzia contro ogni situazione concorrente. Non dichiarare che tutte le piattaforme o i filesystem abbiano già manifestato il problema.

**Destinatari:** artisti che aggiornano frequentemente presentazioni, gallerie che archiviano versioni inviate a interlocutori differenti, studi con cartelle condivise e archivisti che conservano PDF esterni di riferimento.

## 2. Scenari d'uso

1. **Nome già occupato:** `Catalogo-mostre.pdf` contiene la brochure inviata il mese prima; una nuova esportazione con lo stesso nome non deve sovrascriverlo silenziosamente.
2. **Unità rimovibile:** durante la scrittura si scollega il disco, si esaurisce lo spazio o viene negato un permesso; il nome definitivo non deve identificare un nuovo PDF parziale come riuscito.
3. **Un altro programma modifica la cartella:** tra il dialogo di salvataggio e la pubblicazione nasce un file con lo stesso nome; il controllo iniziale `exists()` non può autorizzare la sovrascrittura.
4. **Percorso ambiguo:** un collegamento simbolico, anche *dangling* (link il cui bersaglio non esiste), o un cambiamento della directory richiede verifica robusta **al momento dell'apertura/pubblicazione**, non fiducia in un controllo precedente. Il possibile comportamento dipende dal sistema e va testato, non attribuito automaticamente a un exploit.
5. **Brochure grande o immagini costose:** il PDF viene costruito localmente; un errore tardivo nella pubblicazione deve lasciare disponibili le opere, i media, la bozza e gli eventuali PDF precedenti.

## 3. Specifica funzionale e flussi UX proposti

### A. Esportazione ordinaria

- Dall'editor brochure l'utente sceglie **Esporta PDF**, come oggi, e indica liberamente un file `.pdf` in una cartella locale, disco USB o NAS montato. Non sono richiesti account né connessione.
- Se il nome finale risulta occupato, mostrare un messaggio comprensibile: **«Esiste già un file con questo nome. Scegli un nuovo nome.»** Possibile proposta di suffisso non collidente, visibile e confermata dall'utente. **Non aggiungere** un'opzione nascosta di sostituzione automatica.
- Dopo la generazione, segnalare successo **solo** quando il file definitivo è stato pubblicato senza sostituire quello di terzi e tutti i controlli supportati sono riusciti. Percorso ed eventuale dimensione riportati all'utente; nessun invio automatico.
- Il dialogo di salvataggio e la UI non devono promettere una “versione archiviata internamente”: i PDF esterni non entrano per ciò solo nei backup di Repertica.

### B. Errore, interruzione e disconnessione

- Generare in memoria o in un **file temporaneo di proprietà dell'operazione**. Nel secondo caso il nome deve essere riconoscibile come incompleto e distinto dal PDF finale.
- Scrivere il file temporaneo nella destinazione effettiva (stesso filesystem quando possibile), gestire `write`, `flush`, `sync` e fallimenti di chiusura/durabilità secondo la piattaforma. **Solo dopo** pubblicare con un'operazione **no-clobber** che rifiuti una destinazione appena occupata.
- Se il programma termina, il volume scompare o la pubblicazione fallisce: non modificare un PDF già presente; se un temporaneo non può essere cancellato, segnalarlo senza rimuovere indiscriminatamente file di altri programmi.
- Su filesystem che non consentono di garantire pubblicazione e durata atomiche, distinguere chiaramente **integrità verificata del file finale** da **garanzia in caso di crash non dimostrata**. Non dichiarare atomicità universale su NAS.

### C. Output e contenuti

- La bozza, i dati SQLite delle opere, le immagini originali e le vendite non cambiano. Stessa selezione di foto, lingua documento, ordine, testi e layout della brochure.
- Questa proposta non corregge font Unicode, overflow o resa tipografica: quel problema è documentato in [TECH-032](REPS-TECH-032-pdf-brochure-unicode-font-incorporati-offline.md).
- Verifica facoltativa dell'header/footer PDF o dell'hash del risultato finale per rilevare errori di trasporto: **non** confondere un PDF formalmente apribile con un PDF editorialmente corretto.
- L'utente non deve perdere un PDF precedente nemmeno quando il nuovo documento risulta invalido o l'app subisce un errore.

## 4. Struttura dati e ipotesi di API (descrittive, non implementate)

**Prima fase senza migrazioni SQLite né impostazioni obbligatorie.** Il contratto concettuale può rappresentarsi così:

| Entità/valore | Campi ipotetici | Invarianti |
| --- | --- | --- |
| `BrochureExportRequest` | `brochure_id`, `destination_path`, `expected_extension` | Destinazione scelta esplicitamente; bozza identificata tramite UUID esistente |
| `BrochureExportOutcome` | `status` (`completed`/`name_taken`/`failed`), `published_path?`, `bytes?`, `sha256?` | Percorso finale solo su successo; nessuna informazione commerciale nel log |
| `TemporaryOutput` | `operation_id`, `temporary_path`, `owner_marker`, `created_at` | Eventuale stato transitorio; non nuova fonte autorevole né cronologia di documenti |
| `ExportFailure` | `reason_code` (collisione, permesso, spazio, unità rimossa, verifica, I/O), `recoverable` | Messaggio localizzabile; nessuna perdita silenziosa del file precedente |

API ipotetiche: `prepareBrochurePdf(id) -> BytesOrStagedOutput`; `publishNewPdfExclusive(staged, destination) -> Result`; `getSafeExportOutcome(result) -> UIMessage`. **Non sono comandi presenti** e non prescrivono la scelta di una libreria.

**Decisione tecnica aperta:** conservare il PDF in memoria (come oggi) e pubblicarlo tramite file temporaneo, oppure generarlo progressivamente. Il primo approccio limita la portata del cambiamento; il secondo va preso in considerazione solo se misure reali giustificano una memoria minore. La finalizzazione deve essere affidabile **in entrambi i casi**.

## 5. Offline, sicurezza, privacy, impostazioni e multilingua

- **Offline e indipendenza:** tutte le operazioni sono locali; nessun server, account, telemetria, licenza online, storage proprietario o cloud imposto. L'utente sceglie **qualsiasi cartella accessibile**; un NAS già montato è un percorso filesystem, non un'integrazione remota obbligatoria.
- **Sicurezza filesystem:** distinguere `exists()` da ispezione del **nodo** con semantica `lstat`/equivalente; rifiutare link simbolici anche quando il target manca; attenzione a directory sostituite, reparse point Windows e race check-then-use. Preferire le primitive sicure effettivamente disponibili su ogni sistema, non una sequenza `if !exists { rename }` vulnerabile a competizione.
- **No-clobber:** la creazione esclusiva `create_new` impedisce collisioni all'apertura ma non, da sola, la pubblicazione atomica di un file precedentemente generato. Valutare separatamente le primitive OS per rendere visibile il **solo documento completo** senza sovrascrivere. La soluzione esatta deve essere dimostrata su Windows/macOS/Linux e verificata sui filesystem supportati.
- **Privacy:** una brochure può includere contatti di presentazione, descrizioni non pubbliche o fotografie. Anche i file temporanei devono avere permessi restrittivi compatibili con l'OS, non essere inviati in report o cloud, e non lasciare copie non segnalate in cartelle condivise. L'output finale è un PDF **in chiaro** che l'utente sceglie di salvare.
- **Impostazioni:** nessuna nuova impostazione richiesta. Eventuale preferenza solo locale per ultima cartella proposta (senza forzare percorsi) e nessuna esportazione periodica. La regola di **non sovrascrivere** non può essere disabilitata silenziosamente.
- **Lingue e accessibilità:** seguire lingua di sistema con fallback inglese e override IT/EN/ES/FR/DE; tradurre errori, stati e avvisi, **non** modificare il testo redazionale salvato. Avviso di collisione con focus corretto, leggibile con screen reader, navigabile da tastiera e senza indicazioni affidate solo al colore.
- **Compatibilità:** preservare schema 15 e bozze esistenti, PDF già esportati, API editor e backup SQLite/media; nomi Unicode, file con spazi, percorsi lunghi e cartelle scelte dall'utente. Eventuali PDF esistenti non sono inseriti retroattivamente nei backup.

## 6. Dipendenze, alternative, rischi e complessità

**Dipendenze analitiche:** `src-tauri/src/brochures.rs`, dialoghi Tauri, Rust `std::fs`, permessi del filesystem host e test manuali nativi. Prima di qualsiasi implementazione futura servirebbe confrontare primitive di pubblicazione no-clobber supportate da Windows, macOS, Linux e NAS. **Nessuna nuova dipendenza è stata scelta o aggiunta.**

**Alternative:**
1. **Minimo cambiamento futuro:** almeno impedire la sovrascrittura di file ordinari usando apertura esclusiva al momento della scrittura. Protegge documenti già presenti, ma **non** assicura pubblicazione di soli file completi dopo crash: va dichiarato.
2. **Due fasi su file temporaneo + pubblicazione esclusiva:** maggiore robustezza, con costo di I/O e gestione della cancellazione/recupero temporanei; preferibile solo se validabile multipiattaforma.
3. **Esportare sempre con nome univoco automatico:** evita parte delle collisioni ma riduce il controllo sul nome; da valutare **come suggerimento**, non imposizione.
4. **Chiedere conferma per sovrascrivere:** sconsigliato come default perché documenti storici o di terzi possono essere persi; discutere solo dopo esplicita scelta del proprietario, non equivale a un salvataggio sicuro.

**Rischi:** collisioni concorrenti; time-of-check/time-of-use; symlink/reparse point e nomi anomali; sospensione dell'app durante scrittura; spazio insufficiente; pubblicazione parziale su NAS; temporanei con contenuti riservati; riduzione della compatibilità in caso di primitive OS non disponibili; consumo RAM attuale. **Complessità:** bassa-media per protezione minima dai nomi già esistenti; media-alta per integrità, durabilità e collaudo portabile senza promesse eccessive.

## 7. Criteri di accettazione e test suggeriti (solo proposti, non eseguiti)

1. Un `.pdf` esistente con byte noti **rimane identico** prima/dopo un tentativo di export con lo stesso nome; il risultato segnala nome occupato e non successo.
2. Due tentativi simultanei sullo stesso nome: **al massimo uno** può riuscire; nessun PDF preesistente viene troncato.
3. Errore dopo scrittura parziale, spazio esaurito, directory non scrivibile, arresto forzato e USB scollegata: il file finale preesistente non cambia. L'eventuale output temporaneo è gestito in modo dichiarato.
4. Test con link simbolico **valido e dangling**, reparse point Windows ove pertinente, cartella che cambia nel frattempo, percorso con estensione diversa e percorso dentro archivio: nessuna fuga dal perimetro/overwrite involontario. Non presupporre che l'esito sia identico su ogni filesystem.
5. Un nuovo PDF corretto si apre su lettori nativi, senza troncamento e con ordine di pagine e fotografie attesi; contenuti e impostazioni della brochure rimangono identici prima/dopo il salvataggio.
6. Prove native Windows/macOS/Linux, file grandi, USB e NAS; registrare quali garanzie (no-clobber, visibilità solo dopo completamento, persistenza post-crash) sono realmente ottenute per ciascuna piattaforma.
7. Avvisi localizzati nelle cinque lingue previste, fallback inglese e focus/tastiera/screen reader; log e messaggi non rivelano contenuti di collezionisti o fotografie.
8. Archivio SQLite, backup, certificati, esportazioni CSV e vecchi PDF **inalterati**; nessun account o traffico di rete richiesto.

## 8. Priorità proposta, fasi e domande al proprietario

- **Fase di valutazione P1 (non autorizzata):** confermare mediante code review e prova riproducibile in ambiente di test che un file ordinario preesistente possa essere troncato da `fs::write`; mappare gli analoghi punti di export senza estendere questa scheda ai CSV. Se confermato, priorità alta per evitare perdita di copie utente, indipendentemente dal numero di brochure generate.
- **Fase eventuale A:** protezione minimale delle destinazioni già occupate e degli input di percorso, con semantica di errore distinta da errore di rendering. Nessun redesign dell'editor.
- **Fase eventuale B:** pubblicazione finale no-clobber solo a PDF completo, pulizia prudente dei temporanei, test di durabilità e crash; precisare i limiti su NAS e filesystem non locali.
- **Fase eventuale C:** diagnostica di grandi esportazioni e progresso/annullamento **solo se misure e utenti lo richiedono**. Non introdurre una coda, scheduler o archivio storico delle brochure in automatico.

**Requisiti certi del prodotto:** desktop gratuito e open source, Tauri/SQLite locale, nessun account, backup e cartella di esportazione a libera scelta, lingua OS/fallback inglese, dati sempre portabili; la brochure PDF nativa è già presente. **Fatto statico del codice:** la destinazione regolare già esistente non viene respinta e il codice usa `fs::write`. **Ipotesi non convalidate:** frequenza degli overwrite reali, comportamento esatto su NAS e race/symlink in ciascun OS, domanda di un'esportazione con annullamento, utilità di un hash finale.

**Questioni per revisione esclusiva del proprietario:**

- Vuole una regola inderogabile **«non sostituire mai file esistenti»** per tutti gli export oppure una diversa politica esplicita per i PDF?
- È più utile suggerire un nome unico (es. data/versione) o chiedere di sceglierlo manualmente, mantenendo sempre la destinazione libera?
- Quali combinazioni reali di Windows/macOS/Linux e USB/NAS devono essere considerate supportate e verificate?
- Basta prima il rifiuto della collisione o serve dare priorità contemporaneamente alla pubblicazione atomica?
- È accettabile lasciare un file temporaneo chiaramente identificato dopo crash, purché mai scambiato per PDF concluso e cancellato solo con conferma?

**Decisione riservata al proprietario:** proposta esclusivamente documentale, **«Proposta — da valutare»**. Nessun codice/test/configurazione modificato, nessuna issue o attività di implementazione aperta, nessun merge o approvazione implicita.
