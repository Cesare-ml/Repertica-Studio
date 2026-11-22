# REPS-TECH-028 — Ricerca testuale offline nei PDF allegati, con indice locale eliminabile

- **ID stabile:** REPS-TECH-028
- **Data:** 2026-11-22
- **Stato:** **Proposta — da valutare**
- **Filone:** strumenti e architettura; analisi e specifica **senza autorizzazione a implementare**
- **Priorità proposta:** **P2–P3, condizionata a documenti effettivamente numerosi e richieste verificabili**
- **Ambito iniziale:** ricerca di testo *già presente* nei PDF locali allegati agli incassi; estensione ad altri documenti soltanto se saranno disponibili altri collegamenti e se approvati separatamente

## 1. Problema, destinatari, stato attuale verificato

Un artista o una galleria può ritrovare una vendita per importo e acquirente, ma avere difficoltà a rintracciare in quale documento PDF fosse scritto, per esempio, un riferimento a un trasporto, a un'opera o a un numero pratica. L'obiettivo potenziale è **ritrovare il documento** per aprirlo consapevolmente, non estrarre fatture, creare incassi, attribuire autenticità, fare OCR automatico o interpretare clausole contrattuali.

**Riscontri nel branch `main` del 22/11/2026:**
- [`docs/RECEIPT-ATTACHMENTS.md`](../../RECEIPT-ATTACHMENTS.md) e [`src-tauri/src/receipt_attachments.rs`](../../../src-tauri/src/receipt_attachments.rs) descrivono **un allegato per incasso**, PDF/JPEG/PNG/WebP fino a 12 MiB, copiato localmente, identificato da SHA-256 e incluso nei backup. La UI può esportarlo, ma **non incorpora un lettore PDF** per mantenere la CSP restrittiva della WebView.
- [`src/app/archive-filters.ts`](../../../src/app/archive-filters.ts) effettua ricerca nei **campi testuali di opere e collezionisti**, non nel corpo binario dei PDF.
- [`src-tauri/Cargo.toml`](../../../src-tauri/Cargo.toml) non dichiara una dipendenza specializzata nell'estrazione del testo PDF; dai componenti esaminati non emerge un indicizzatore dei PDF degli incassi. Questa è una **ricognizione statica**, non prova dell'assenza di qualunque soluzione interna o di una domanda reale.
- La [roadmap MVP](../../MVP.md) prevede ricerche più avanzate, ma l'archivio non deve cambiare necessariamente per aggiungere questa funzione: prima stimare quanti PDF consultano effettivamente gli utenti.

**Destinatari da intervistare:** gallerie con ricevute e documenti multipli, amministratori di studio, archivisti con documentazione digitale. Per pochi PDF, i nomi dei file e l'apertura nel lettore di sistema potrebbero bastare.

## 2. Confine e verifica anti-doppione

- **[TECH-008](REPS-TECH-008-viste-salvate-filtri-ricerca-locale.md)** definisce filtri, viste e query sui **metadati dell'archivio attivo**. Questa scheda riguarda una sorgente differente: il testo *interno* di **file PDF acquisiti**, eventualmente associato in futuro a quei filtri senza duplicare la barra di ricerca.
- **[TECH-016](REPS-TECH-016-allegati-documentali-locali-multi-entita.md)** propone *custodia e collegamenti* di documenti a opere e altre schede; non descrive un motore di estrazione e ricerca. La prima fase qui usa soltanto **gli allegati incasso già implementati**, senza introdurre quella funzione futura né alterare i collegamenti.
- **[TECH-003](REPS-TECH-003-diagnostica-integrita-media.md)** protegge l'integrità dei file, non ne indicizza il contenuto. **TECH-001/018/025** studiano portabilità ed esportazione, non ricerche nel contenuto. **TECH-014** approfondisce il confine MCP: il testo indicizzato **non diventa automaticamente disponibile agli assistenti IA**.
- Le **32 proposte Gestione e UX** nella [PR #14](https://github.com/Cesare-ml/Repertica-Studio/pull/14) riguardano tra l'altro giustificativi di spesa (UX-028), quadratura (UX-030), dossier (UX-007) e altri processi. Nessuna viene riscritta: UX-028 potrebbe *un giorno* fornire ulteriori PDF da cercare, ma non è un prerequisito dell'idea.
- **Fuori ambito:** OCR di fotografie/scansioni, IA/LLM, riassunti o deduzione di dati, validazione delle ricevute, controlli fiscali, indicizzazione di email o di cartelle esterne, indicizzazione di backup e altri archivi, ricerca online, server HTTP, accesso MCP, importazione PDF generalizzata. In particolare, la parola **«trovato» non implica che una dichiarazione nel PDF sia vera**.

## 3. Scenari d'uso e flussi UX illustrativi

**A. Ricerca mirata, con consenso.** Dal Registro vendite → «Cerca nei documenti PDF» (funzione *solo ipotetica*), l'utente legge che il testo estratto potrebbe contenere dati personali. La funzione parte **disattivata**. L'utente decide di indicizzare soltanto uno o più PDF identificati nel suo archivio; prima vede numero, tipi, dimensione cumulata e spazio stimato. Non viene eseguita una scansione automatica delle cartelle, né viene interrogato Internet.

**B. Risultati e provenienza.** L'utente cerca «spedizione assicurata». La UI mostra la riga con **nome allegato, incasso e vendita collegati, data del documento se esplicitamente registrata**, ed eventualmente numero di pagina e porzione di testo *solo dopo un'ulteriore scelta di visualizzazione*. Il risultato non viene fatto passare per un campo certificato di Repertica. «Esporta e apri documento» riutilizza il percorso di esportazione sicura **già esistente**; non è necessario introdurre un viewer PDF nella WebView.

**C. Documento scannerizzato.** Il file è un PDF composto da immagini e non contiene testo selezionabile. Il risultato è **«Nessun testo estraibile»**, non «Nessuna corrispondenza» e non «Documento vuoto». È possibile aprirlo manualmente con il programma di sistema; nessun OCR invisibile o upload a un servizio remoto.

**D. Modifica, rimozione e ripristino.** Se un allegato viene rimosso, se il suo hash cambia o se l'utente ripristina un altro archivio, la corrispondente voce dell'indice diventa **non attendibile** e non deve essere mostrata come risultato corrente. Un indice eliminato può essere ricostruito volontariamente dai PDF esistenti: **l'indice non è un backup**, la sua perdita non fa perdere i documenti.

**E. Privacy nell'uso condiviso dello schermo.** Di default i risultati non mostrano importi, nomi di clienti o estratti di testo non necessari. Pulsante «Mostra estratto» esplicito, azione «Cancella indice locale», nessun suggerimento di ricerca tratto automaticamente da documenti riservati. Navigazione da tastiera, etichette accessibili, annunci di completamento e messaggi comprensibili per testo non leggibile, PDF protetti e indice obsoleto.

## 4. Specifica funzionale e confini di affidabilità

1. **Sorgenti esplicite.** Fase iniziale limitata ai PDF referenziati da `sale_receipt_attachments`; usare ID dell'incasso + percorso relativo sicuro + impronta del file **verificata prima dell'elaborazione**. Un file senza testo non va trattato come errore di integrità. Escludere JPEG/PNG/WebP, RAW, file esterni, copie storiche non scelte e backup.
2. **Separazione archivio/indice.** L'indice è un artefatto derivato, scartabile e non autorevole; non modifica il PDF, le righe SQLite delle vendite, gli incassi, le rate o il checksum originale. Una cancellazione dell'indice non cancella **mai** allegati. Ricalcolare solo se richiesto e previo controllo dell'impronta. Evitare la persistenza di una seconda copia integrale del testo se non indispensabile.
3. **Parsing prudente.** PDF cifrati, protetti da password, malformati, privi di testo, molto complessi o con mappature Unicode insufficienti devono avere stati distinti. Il sistema può non garantire tutti gli alfabeti, la posizione esatta della pagina o la ricostruzione della lettura: **i falsi negativi devono essere esplicitati**.
4. **Ricerca letterale controllata.** Termini e frasi con limite di lunghezza, normalizzazione Unicode documentata e comportamento su accenti/lingue misurato; nessun campo che accetta SQL o regex arbitraria. Le stringhe estratte non eseguono codice. I risultati collegano al documento originale verificato, mai a copie temporanee arbitrarie.
5. **Confine con la sicurezza esistente.** Il PDF viene **letto da Rust o da processo isolato**, non iniettato nell'HTML della WebView. Non indebolire `object-src 'none'` e `frame-src 'none'`, non interpretare JavaScript nei PDF, non aprire URI o allegati incorporati.

**Contatori da mostrare:** `documenti esaminati`, `PDF con testo`, `PDF senza testo`, `non elaborabili`, `indice obsoleto`. L'utente deve conoscere ciò che **non** è stato cercato.

## 5. Modello dati e API — solo ipotesi descrittive

Possibile **indice secondario** in un file locale separato dall'archivio autorevole, all'interno della directory privata dell'app, non incluso automaticamente nei backup completi. Un esempio puramente concettuale:

```text
DocumentIndexEntry {
  archive_fingerprint,          // identifica il contesto; non equivale all'UUID del documento
  attachment_receipt_id,
  attachment_sha256,
  parser_version,
  indexed_at_utc,
  status: text_found | no_text | protected | unsupported | error | stale,
  page_count?,
  searchable_tokens?,           // solo se realmente necessari
  indexed_bytes?,
  warnings: [stable_code]
}
SearchDocumentHit {
  receipt_id, attachment_sha256,
  page_number?,                 // solo se affidabile per quel parser
  match_count?,
  snippet_available: boolean,   // false di default
  trust: "text_extracted_not_verified"
}
inspectEligibleAttachments() -> EligibilityReport
previewIndexing(attachmentIds) -> ConsentPlan
buildLocalIndex(plan) -> IndexReport
searchIndexedDocumentText(query) -> SearchResults
forgetIndex() -> RemovalReport
```

Queste **non sono API approvate** e i tipi non corrispondono a tabelle esistenti. Se si valutasse SQLite FTS5, verificarne **realmente** disponibilità, normalizzazione, prestazioni e packaging per Windows/macOS/Linux: non assumere che sia abilitato solo perché SQLite è presente. Non archiviare il PDF come blob duplicato nell'indice. La chiave dell'indice deve incorporare identità dell'archivio corrente, identificativo dell'allegato e SHA-256, così un ripristino di un diverso archivio con UUID coincidenti non produrrà risultati incrociati. Un controllo di coerenza prima dei risultati resta necessario anche dopo riavvio.

**Portabilità:** gli originali PDF e le loro relazioni rimangono nello SQLite e nel `media/` dei backup completi scelti dall'utente, esattamente come oggi. L'indice scartabile **non** è richiesto per il ripristino e non va copiato nei 12 CSV completi o nei pacchetti aperti di TECH-001; su nuovo PC si può ricostruire offline, solo dopo consenso. Eventuali esportazioni di elenchi risultati richiederebbero in futuro scelta esplicita e anteprima privacy, senza modificare gli esportatori vigenti.

## 6. Offline, sicurezza, privacy e impostazioni

- **Offline:** nessuna chiamata API, account, abbonamento, OCR cloud, telemetria, connessione obbligatoria o autorizzazione di rete. Nessuna scansione di cartelle non selezionate; il server MCP locale non legge l'indice salvo **diversa decisione futura esplicita**.
- **Input ostili:** i PDF possono contenere stream compressi, font corrotti, oggetti annidati e payload progettati per consumare memoria/CPU. Imporre limiti di byte, pagine, oggetti, testo estratto, profondità e tempo; isolamento del parser in processo con privilegi minimi e convalida degli output quando praticabile; non fidarsi soltanto dell'intestazione `%PDF-`. Esaminare vulnerabilità, licenza e aggiornamenti di un eventuale parser prima di sceglierlo.
- **Riservatezza:** l'indice testuale è **un'altra copia di contenuti personali e finanziari**, anche quando gli allegati originali sono già protetti da checksum. Predefinito **spento**, costruzione deliberata e reversibile, cancellazione documentata dell'indice e reset dopo cambio archivio. Non promettere cancellazione forense su SSD, crash dump o swap: richiedere la protezione del disco OS; non salvare testo in log, cronologia query, notifiche o clipboard automaticamente.
- **Impostazioni potenziali:** «Attiva ricerca nel testo dei PDF» disattivata; «Seleziona documenti»; «Indice locale: numero file / dimensione / ultimo controllo»; «Cancella indice»; scelta facoltativa se visualizzare estratti nei risultati, **spenta**. Nessuna modalità automatica silenziosa o invio agli assistenti.
- **Multilingua:** lingua sistema con fallback **inglese**, override IT/EN/ES/FR/DE secondo il prodotto; descrizioni, errori di parsing e stati localizzati; testo originale **mai tradotto automaticamente**. Differenziare «testo non trovato», «PDF non indicizzato» e «PDF senza testo» in ciascuna lingua. Supporto lingue di estrazione e ricerca deve essere dichiarato e provato, non assunto.

## 7. Alternative, dipendenze, complessità e rischi

| Opzione | Pro | Contro / condizione |
| --- | --- | --- |
| **A. Restare con nome file e esportazione manuale** | Zero nuove dipendenze, nessun indice di dati privati | È lento per decine/centinaia di documenti simili |
| **B. Ricerca *solo su richiesta* in un PDF selezionato, senza indice persistente** | Minore accumulo di dati riservati e superficie di migrazione; prototipo di valore | Per ogni ricerca può dover riprocessare il PDF; il parser rimane necessario |
| **C. Indice locale cancellabile e opt-in** | Ricerca ripetuta veloce su un corpus definito, senza cloud | Dati personali derivati, invalidazioni, consumo disco e manutenzione |
| **D. OCR offline come fase futura distinta** | Potrebbe trattare scansioni prive di testo | Molto più costoso: riconoscimento multilingua, errori, pacchetti modello, privacy e performance; **non incluso qui** |

Dipendenze progettuali: collegamenti degli allegati incasso e loro SHA-256, backup verificato, gestione dei contesti di archivio, dialoghi accessibili, scelta di un parser locale compatibile e mantenuto, criteri di eliminazione e invalidazione dell'indice. **Complessità media-alta / alta** soprattutto per parsing sicuro e confini di riservatezza. La prima verifica di fattibilità dovrebbe confrontare l'opzione **B** senza persistenza e l'opzione **C**, non assumere che l'indice sia necessario.

Rischi: fuga di dati tramite estratti o indice, falsa completezza quando il PDF è scansione, risposte sbagliate con testo multicolonna, parser compromesso, lentezze di archivi grandi, risultati obsoleti dopo cancellazioni/ripristini, ambiguità Unicode e lingue diverse, duplicazione inutile di funzionalità del sistema operativo. **Nessun parser o formato è scelto**.

## 8. Criteri di accettazione e prove da eseguire soltanto se l'idea sarà approvata

1. Corpus di test **fittizio o autorizzato**: PDF con testo italiano e inglese, accenti, font embedded, più colonne, scansione senza OCR, PDF criptato, malevolo/malformato, file grandi, PDF senza testo e casi con nome uguale ma SHA diverso. Riportare *precisione, falsi negativi e tempo/memoria* senza dichiarare copertura universale.
2. Nessun file viene indicizzato o trasmesso prima di consenso; annullamento e fallimento lasciano l'archivio e gli allegati invariati. I PDF restano identici byte per byte e tutti i checksum continuano a valere.
3. Il risultato rimanda sempre al **file ancora presente e verificato**; sostituzione, scollegamento, cambio di archivio e ripristino invalidano i risultati relativi. L'indice cancellato non appare come «documenti senza corrispondenze».
4. La UI distingue chiaramente testo assente, documento non indicizzato, file non leggibile, nessun risultato e indice obsoleto. Snippet e nomi privati non vengono divulgati per impostazione predefinita.
5. Backup completo e ripristino tramite cartella scelta funzionano **senza** indice; nuovo PC offline ricostruisce, se confermato, soltanto le informazioni derivate. CSV e MCP esistenti restano invariati.
6. Test di isolamento del parser, limiti risorse, input ostili, clean-up, licenze e pacchettizzazione su Windows/macOS/Linux; accessibilità con tastiera e lettore schermo, scaling 200%, testo tradotto nelle cinque lingue e fallback inglese.
7. Casi reali anonimi: misurare quante ricerche documentali vengono fatte e se l'opzione senza indice è sufficiente. Nessuna approvazione automatica solo perché un benchmark tecnico passa.

## 9. Fasi ipotetiche, priorità e questioni per il proprietario

- **Fase 0 — ricerca necessità (raccomandata):** intervistare 3–5 operatori e misurare quantità di PDF, query ricorrenti, scansioni senza testo e sensibilità dei documenti. Se pochi PDF, **non proseguire**.
- **Fase 1 — concetto minimo:** valutare ricerca in un singolo PDF scelto, **senza persistenza**, con parser isolato e limiti chiari; definire criteri di errore e privacy.
- **Fase 2 — eventuale indice opt-in:** solo se il problema è frequente e il parser ha superato audit; UI per selezione, stato, rimozione, invalidazione e prove multipiattaforma.
- **Fase 3 — estensione subordinata:** ulteriori classi di documenti *solo se i relativi modelli sono approvati*, mai introducendo per via indiretta TECH-016 o UX-028. OCR e MCP restano proposte distinte, non «fasi obbligatorie».

**Requisiti certi del progetto:** offline-first, archivi SQLite locali, file originali e backup esportabili interamente, nessun account/cloud obbligatorio, multilingua sistema con fallback inglese, protezione della privacy. **Ipotesi non dimostrate:** frequenza delle ricerche nei PDF, utilità dell'indice, sostenibilità del parsing, richiesta di OCR. **Domande aperte:** quanti PDF e quali formati reali? Va bene cercare solo nei documenti degli incassi? Serve davvero un indice persistente? Qual è la soglia massima di rischio/complessità tollerata? Un OS-level search manuale risolve già il bisogno? Chi potrà vedere i risultati su computer condivisi? I documenti scanditi sono la maggioranza?

**Conclusione:** opportunità concreta da **valutare**, non una priorità approvata. Qualunque attuazione richiederà una decisione esplicita separata. Nessun codice, dipendenza, issue o PR di implementazione è richiesto o autorizzato da questa scheda.
