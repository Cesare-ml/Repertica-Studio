# REPS-TECH-050 — Confronto descrittivo di due backup verificati, senza ripristino

- **ID stabile:** REPS-TECH-050
- **Data della ricognizione:** 2026-11-22
- **Stato:** **Proposta — da valutare**
- **Priorità ipotetica:** **P2–P3**, subordinata a una necessità reale e a prove su backup sintetici; nessuno sviluppo se i flussi esistenti bastano.
- **Ambito:** strumento tecnico di consultazione **in sola lettura** di **due backup completi** scelti manualmente. Non è sincronizzazione, fusione, ripristino, riparazione, storico delle modifiche né gestione automatica dei backup.
- **Decisione del proprietario:** nessuna; questo documento non approva o assegna attività.

## Problema e riscontro nel repository

Un artista salva periodicamente copie complete di un archivio su SSD o NAS. Prima di recuperare una vecchia copia vuole capire se contiene opere, vendite o fotografie non presenti in una copia più recente. I soli conteggi non distinguono «due opere aggiunte» da «una rimossa e tre aggiunte»; confrontare manualmente due SQLite e centinaia di media espone a errori. La domanda utile è **«quali differenze documentabili esistono fra queste due copie?»**, non «quale backup è migliore?».

**Accertato su `main` (22/11/2026):** [`backup.rs`](../../../src-tauri/src/backup.rs) crea directory autosufficienti con `repertica.sqlite`, `media/`, manifest JSON v1 e SHA-256. `inspect_backup_core` valida una singola copia, inclusi SQLite, hash, riferimenti e allegati, e restituisce *soltanto* conteggi di opere, collezionisti, vendite e certificati, spazio, schema e data dichiarata. Il restore **sostituisce** l'archivio attivo e ne conserva il precedente; non esiste nel flusso ispezionato una comparazione semantica tra **due** backup. Le entità principali hanno ID testuali e schema SQLite fino a v18 ([`db.rs`](../../../src-tauri/src/db.rs)). Il manifest v1 **non contiene un identificativo affidabile dell'archivio originario**. Questo è un bisogno possibile, **non** un incidente o una richiesta di utenti già confermati.

**Destinatari:** artisti con copie manuali/automatiche, gallerie che custodiscono fotografie e documenti storici, professionisti che recuperano dati dopo migrazione o ripristino non distruttivo. Utilità maggiore quando i backup differiscono significativamente o ci sono errori umani; per un archivio piccolo può bastare l'ispezione già implementata.

## Scenari e flussi UX ipotizzati

1. **Scegli A e B.** In Impostazioni → Backup → possibile voce futura «Confronta due copie»; selezione **esplicita** di due sottocartelle `ReperticaStudio-backup-...` (anche su dischi differenti). Nessuna scansione dell'intero computer e nessuna modifica dell'archivio attivo. La UI distingue origine A/B da «precedente/migliore», senza attribuire automaticamente superiorità alle date o ai nomi.
2. **Verifica le fonti.** Prima di confrontare, usare l'ispezione integrale esistente su ciascuna copia; indicare schema, data *dichiarata dal manifest*, numero file, dimensione ed esito con ora effettiva della verifica. Se una fonte è corrotta, incompatibile, scomparsa o il controllo è interrotto, **non produrre un confronto qualificato come attendibile**.
3. **Mostra differenze, non conclusioni.** Prima schermata aggregata per entità compatibili (opere, collezionisti, vendite, incassi, spese, certificati e allegati laddove lo schema li supporta): «solo in A», «solo in B», «ID presenti in entrambi ma dati differenti», «nessuna differenza rilevata nel perimetro confrontato». Se i dati disponibili non consentono un confronto, mostrare «non confrontabile», **mai zero**.
4. **Apri dettagli su richiesta.** Elenco per ID/codice, campi distinti, valori A/B ed eventuali revisioni delle fotografie; **nominativi, prezzi, note o immagini nascosti per impostazione iniziale** e rivelati volontariamente, in particolare durante condivisioni schermo. Permettere filtri e navigazione da tastiera. Un certificato immutabile con ID diverso è una voce distinta, non una «variante» dedotta.
5. **Metti in guardia sull'identità.** Due cartelle possono provenire da archivi diversi, da una copia clonata o da esportazioni con contenuti divergenti. Anche UUID coincidenti non bastano per attestare provenienza comune. In assenza di prova esplicita mostrare «confronto di contenuti selezionati, non cronologia certificata»; **«solo in A» non significa cancellato in B**.
6. **Esci senza modifiche.** Nessun pulsante «rendi B come A», merge, sync, eliminazione di backup, ripristino o sostituzione. Chiudendo non resta un indice sensibile obbligatorio. Report locale **facoltativo**, con conferma separata ed esportazione iniziale di soli conteggi/codici di stato.

### Ambito minimo e confini anti-doppioni

- [TECH-009](REPS-TECH-009-catalogo-verifiche-conservazione-backup.md) elenca, rivalida e studia la *conservazione* delle copie; qui si analizzano le **differenze fra dati all'interno di due snapshot**, senza cataloghi né retention.
- [TECH-037](REPS-TECH-037-recupero-ripristino-interrotto-avvio-protetto.md) affronta il **bootstrap dopo un crash** e il confronto *identificativo minimo* dei candidati di recupero; qui solo backup completi verificati, dall'app funzionante, senza selezione/attivazione di candidati.
- [TECH-010](REPS-TECH-010-storico-revisioni-metadati.md) è cronologia **delle modifiche dentro un archivio attivo**; due snapshot indipendenti non costituiscono un event log né spiegano chi o perché ha modificato i dati.
- [TECH-001](REPS-TECH-001-pacchetto-dati-aperto.md) definisce un formato interoperabile e il relativo import: **nessun nuovo formato di backup** qui. [TECH-006](REPS-TECH-006-backup-cifrati-portabili.md) riguarda la cifratura, che in futuro richiederebbe esplicito sblocco prima del confronto.
- [REPS-UX-050](https://github.com/Cesare-ml/Repertica-Studio/blob/docs/idee-gestione-ux-20261121/docs/idee/gestione-e-ux/REPS-UX-050-rendiconto-mensile-snapshot-verifiche.md) studia un **rendiconto economico mensile datato**, non differenze di backup a livello di record. Le altre proposte UX non gestiscono questo percorso tecnico. **Nessun file UX va modificato.**

## Specifica funzionale e dati/API — descrittivi, non approvati

**Fonti:** due directory di backup del formato attualmente supportato, ciascuna validata con gli stessi controlli di `inspect_backup_core` e senza aprire lo SQLite con `db::open` (che può creare o migrare). Nessuna richiesta di nuova tabella; nessuna alterazione di `manifest.json`. In una prima fase confrontare **solo backup con schema identico** e domini esplicitamente mappati; fra schemi diversi segnalare copertura parziale/incompatibilità, **non migrare** i file originali per renderli comparabili.

**Regole ipotetiche per confronto:**
- Entità: chiave `(nome_dominio, id)`; quando in entrambe, confrontare solo i campi dichiarati nel contratto della stessa versione, con tipi/NULL, denaro in centesimi e date **senza** conversioni implicite. Non dedurre un collegamento tra record con stesso titolo, codice o nome ma ID diversi.
- Distinguerne il contenuto dalla presentazione: un diverso `updated_at` può essere un indizio, non prova che un campo sostanziale sia cambiato. Escludere deliberatamente campi solo tecnici se documentati, mantenendo la possibilità di mostrare «metadati tecnici diversi». Nessuna «storia della modifica» retroattiva.
- Manifest media: confrontare per **percorso relativo + lunghezza + SHA-256**, con categorie «file solo in A/B», «stesso path, digest diverso», «stessi byte/hash, path diverso (possibile corrispondenza, non identità semantica)». Le revisioni del Media Center non devono essere collassate in una sola immagine. **Nessun file va eliminato come duplicato.**
- Certificati e relativi snapshot, allegati di incasso, RAW e media storici: mantenere esatta distinzione fra record riferiti e file fisici non referenziati. Un hash valido dimostra integrità dei byte *rispetto al manifest esaminato*, non autenticità/provenienza esterna.
- Impostazioni economiche e totali: trattare eventuali differenze come valori memorizzati, non come bilancio fiscale; niente ricostruzione di pagamenti, profitti o incassi da sole date. Se la versione non consente di leggere un dominio, mostrare «non esaminato».
- Per archivi grandi: letture incrementali, limiti espliciti, progressi e annullamento; non caricare fotografie o tutte le righe in memoria per default.

```text
BackupComparisonRequest {
  backup_a_folder: SelectedLocalDirectory
  backup_b_folder: SelectedLocalDirectory
  domain_scope: SelectedDomain[]
  details_consent: false
}
ComparisonSource {
  format_version: Int
  schema_version: Int
  declared_created_at: String
  checked_at: Timestamp
  integrity: VerifiedNow | Invalid | Incompatible | Interrupted
  origin_relationship: Unknown
}
DomainDifference {
  domain: StableDomainCode
  only_a_count: Int
  only_b_count: Int
  same_id_changed_count: Int
  comparable_fields: String[]
  coverage: Complete | Partial | Unsupported
}
ComparisonReport {
  sources: [ComparisonSource, ComparisonSource]
  differences: DomainDifference[]
  media_summary: { only_a, only_b, same_path_different_digest, same_digest_different_path }
  warnings: StableWarningCode[]
}
```

**API puramente ipotetiche:** `inspectSelectedBackup(path) -> SourceStatus`, `compareVerifiedBackups(A, B, domains) -> ComparisonReport`, `readDifferenceDetails(domain, page, consent) -> DifferencePage`, `exportMinimizedComparison(report, chosenPath) -> Result`. Nessun endpoint HTTP, nessuna scrittura su SQLite, nessun import. L'eventuale report aperto è JSON/CSV UTF-8 **generato su richiesta**: schema e copertura dichiarati, nessun dato personale di default.

## Funzionamento offline, sicurezza, privacy, impostazioni e lingue

- **Offline integralmente:** due cartelle scelte dall'utente su disco locale, USB o NAS **già montato dall'OS**. Nessun account, abbonamento, rete avviata, telemetria, autenticazione cloud o copia remota implicita. I dati originali restano disponibili mediante export/backup completi.
- Due backup contengono dati personali e commerciali in chiaro. Il confronto **non** crea una nuova «cartella di lavoro» persistente obbligatoria, non registra nei log nomi, descrizioni o percorsi privati e non invia mai dati a un assistente MCP. L'export di dettagli sensibili richiede esplicita selezione e anteprima, distinto dal report sintetico.
- Rieseguire le verifiche dei due backup prima dell'analisi; rifiutare symlink e file non regolari e distinguere la fonte rimossa o sostituita durante lettura. La validazione preventiva non elimina il rischio TOCTOU su NAS/dischi rimovibili: per risultati considerati affidabili servono controlli dell'identità dei file e trattamento dei cambiamenti durante l'operazione, senza fingere uno snapshot atomico di un'unità esterna.
- Il contenuto SQLite e i nomi dei file sono **dati non attendibili**: sola lettura, nessuna istruzione SQL esterna eseguita, query predefinite, limiti per righe/stringhe, niente interpretazione HTML/macro/testo come comandi, niente apertura automatica di allegati. L'accesso effettivo ai file segue i permessi OS dell'utente.
- **Impostazioni:** preferibile nessuna opzione persistente nella prima fase. Eventuali «ricorda ultime due cartelle» e «mostra dettagli privati» devono essere **opt-in**; quest'ultimo non deve restare attivo permanentemente. Nessuna politica di eliminazione o backup automatico derivata da un confronto.
- **Multilingua:** UI lingua sistema, fallback inglese, preferenza manuale esistente IT/EN/ES/FR/DE; codici tecnici stabili, date e dimensioni formattate per locale, *nessuna* traduzione automatica dei dati catalografici. L'utente distingue A/B anche senza colore; focus, screen reader, tabelle navigabili, filtri e progressi accessibili.

## Dipendenze, alternative, rischi e complessità

**Dipendenze da studiare, non da installare ora:** controlli del backup `inspect_backup_core`, versione schema in `db.rs`, rappresentazione dei media/revisioni, snapshot certificati, allegati degli incassi, eventuale paginazione read-only. La QA nativa dei backup su USB/NAS precede ogni pretesa di affidabilità. Non è necessaria una nuova libreria come requisito della proposta.

**Alternative prima dello sviluppo:**
1. **Nessuna funzione nuova:** verificare le copie da UI e utilizzare CSV + LibreOffice Calc / strumenti SQLite offline, confrontando manualmente i registri. Prima provarla con 3–5 casi reali anonimizzati.
2. **Comparazione minimale** dei quattro conteggi già restituiti da `BackupSummary` e del manifest media: pochissima complessità, ma **non** rivela sostituzioni a totale invariato.
3. **Comparazione read-only a ID/record + manifest** (questa proposta): più utile, costo e privacy maggiori.
4. **Sync/fusione/ripristino selettivo:** esplicitamente **fuori ambito**; potrebbe alterare certificati, vendite, allegati, percentuali e codici senza mantenere le invarianti.

**Rischi:** due archivi indipendenti spacciati per versioni successive; schema diverso e falsi «nessuna differenza»; abuso dei nomi dei file; backup modificati durante scansione; consumo eccessivo di memoria/CPU su archivi grandi; dati di collezionisti mostrati o esportati involontariamente; comparazioni finanziarie interpretate come riconciliazione contabile; tempo e costo di manutenzione sproporzionati rispetto a un uso sporadico. **Complessità stimata:** **media** per verifica+conteggi e manifest, **medio-alta/alta** per confronti semantici completi fra versioni e interfaccia dettagliata.

## Collaudi/criteri di accettazione ipotetici

1. Due backup v1 sintetici, entrambi integri e con stesso schema: contatori A-only/B-only/changed corretti per UUID e categorie supportate, **anche se i totali complessivi sono uguali**; nessuna «cancellazione accertata».
2. Due backup validi ma provenienti da **archivi indipendenti** o da un clone divergente: presentare identità di origine *sconosciuta*, nessuna conclusione su cronologia/cancellazioni/merge.
3. Coppie con schemi incompatibili, vecchi, futuri e media missing/hash errato: rifiutare comparazione qualificata; mostrare fonte e motivo, nessun dato importato o schema migrato.
4. Foto con stesso digest e diverso path, stesso path e hash differente, file storico certificato non referenziato dalla copertina corrente, RAW e allegati incasso: nessun asset ignorato o suggerito per cancellazione automatica.
5. Nominativi, prezzi, note multilinea, accenti Unicode, titoli identici con UUID distinti: visualizzazione iniziale priva di contenuti personali, reveal manuale e differenze determinate da **dati**, non da presentazione/localizzazione.
6. Scollegamento USB, sostituzione directory o modifica di file dopo la prima ispezione, permessi negati, backup con symlink/junction, SQLite con sidecar non ammessi: errore coerente, **nessun report presentato come pienamente verificato**.
7. Grandi archivi su Windows/macOS/Linux con SSD/USB/NAS: misurare memoria, UI reattiva, annullamento e risorse; non stabilire soglie inventate senza benchmark; verificare che non ci siano modifiche a sorgenti e archivio attivo (hash prima/dopo, nessuna query write).
8. Report JSON/CSV minimizzato e opzionale, percorso liberamente scelto, senza overwrite di file esistenti; codici errore localizzati, fallback inglese, lettore schermo e tastiera. Nessuna esposizione implicita a MCP o cloud.

## Fasi ipotetiche e questioni per la revisione del proprietario

**Fase 0 — Validazione d'uso:** raccogliere 3–5 storie anonime di recupero e due coppie di copie con differenze reali; provare CSV + Calc prima di investire nello strumento.

**Fase 1 — Ricognizione minima:** due backup scelti, verificati uno per uno, riepilogo affiancato e confronto dei **manifest** con avvisi di provenienza incerta; **nessuna** lettura dettagliata dei dati personali oltre i contatori esistenti.

**Fase 2 — Solo se utile:** confronto per UUID e campi selezionati nello **stesso schema**, paginato e con dettagli nascosti per default. Successivamente valutare i domini aggiuntivi e report minimizzati.

**Fase 3 — Eventuale ricerca separata:** compatibilità semantica di schemi differenti e possibile confronto dell'archivio *attivo* tramite snapshot temporaneo isolato: non accesso diretto non verificato al live WAL, non ripristino.

**Domande aperte:** quale evento concreto porta gli utenti a confrontare due backup? È sufficiente vedere differenze di file/contatori? Quanto sono frequenti copie di archivi differenti nella stessa cartella? Quali domini e dati personali sono davvero necessari? Conviene aspettare un identificativo stabile di archivio nel backup v2? Sono accettabili tempi di verifica SHA-256 di due archivi fotografici grandi? Il proprietario preferisce non integrare nulla e documentare un confronto manuale?

---

**Questa scheda è soltanto «Proposta — da valutare». Non chiede modifiche al codice, non autorizza lo sviluppo e non comporta approvazione, merge o operazioni sui backup.**
