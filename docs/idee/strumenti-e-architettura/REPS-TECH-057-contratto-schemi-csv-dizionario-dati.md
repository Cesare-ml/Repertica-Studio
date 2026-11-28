# REPS-TECH-057 — Contratto versionato e dizionario dei dodici registri CSV

- **ID stabile:** REPS-TECH-057
- **Data:** 2026-11-23
- **Stato:** **Proposta — da valutare**
- **Priorità proposta:** **P2 condizionata** a utilizzi periodici esterni dei CSV; **P3 o nessun intervento** se la documentazione attuale basta.
- **Perimetro:** descrizione verificabile, versioni e compatibilità dei **CSV già disponibili**. Non nuovi registri, non import, non backup, non modifica dei dati gestionali, non sviluppo autorizzato.

## 1. Motivazione, utenti e riscontri verificati

**Riscontri sul branch `main` (23 novembre 2026):** `src-tauri/src/export.rs` espone dodici `kind` e associa a ciascuno una stringa di intestazioni e una query SQL; `src/app/csv-registers.ts` ne elenca gli stessi dodici tipi. `docs/DATA-PORTABILITY.md` documenta contenuti e convenzioni. I CSV sono **integrali, unidirezionali e non reimportabili**: UTF-8 con BOM, separatore `;`, terminatori CRLF, testo tra virgolette, denaro in centesimi, identificativi UUID e mitigazione delle formule dei fogli elettronici. Il codice esporta i `NULL` come celle vuote; l'escape di sicurezza può anteporre un apostrofo a valori testuali che potrebbero essere interpretati come formule. Non si deduce dal solo testo CSV una corrispondenza perfettamente reversibile con SQLite.

Il codice verificato **non inserisce nel file CSV un identificatore esplicito di versione del singolo contratto di colonne**. La documentazione segnala già che l'aggiunta di sei colonne agli allegati degli incassi può richiedere modifiche ai programmi esterni che si attendono un numero fisso di colonne. `PRAGMA user_version` del database e versione dell'applicazione non sono di per sé versioni di un formato pubblico.

**Problema da verificare, non guasto dimostrato:** un consulente o un archivista che usa LibreOffice, Python o software proprio potrebbe interpretare male `*_cents`, quote `*_bp`, date, campi mancanti e relazioni, oppure non accorgersi che una versione dell'app ha aggiunto o cambiato una colonna. Non sono disponibili misurazioni sul numero di utenti che importano i CSV in strumenti esterni.

**Destinatari:** artisti che consegnano periodicamente registri economici, gallerie con analisi locali, archivi con processi periodici di estrazione, consulenti indipendenti e futuri manutentori del formato.

## 2. Distinzione dalle idee già esistenti

- **[TECH-001](REPS-TECH-001-pacchetto-dati-aperto.md)**: nuovo pacchetto **semantico completo e possibilmente reimportabile** con JSONL/media. Questa scheda non crea un secondo formato di migrazione: documenta soltanto i dodici CSV esistenti.
- **[TECH-018](REPS-TECH-018-esportazioni-csv-scalabili-atomiche.md)**: streaming, snapshot e sicurezza fisica nella generazione di **un singolo file**. Qui si propone un contratto di *significato e compatibilità delle colonne*, non un diverso writer.
- **[TECH-055](REPS-TECH-055-esportazione-coerente-registri-csv.md)**: esportazione **coordinata da una stessa snapshot** con manifest di hash e conteggi. Un dizionario comune potrebbe essere riutilizzato, ma **non garantisce** snapshot, coerenza temporale o autenticità.
- **TECH-025**: minimizzazione di righe/colonne per destinatari esterni; qui le descrizioni riguardano tutti i registri, **senza** crearne versioni ridotte o pubblicabili.
- **TECH-019**: importazione guidata da CSV di terzi, distinta dalla formalizzazione del significato degli export Repertica.
- Il filone **[Gestione e UX — PR #14](https://github.com/Cesare-ml/Repertica-Studio/pull/14)** propone nuove esperienze e dati professionali (tra cui UX-050, rendiconto mensile); non definisce il contratto macchina degli attuali dodici CSV. **Nessun file di quel filone va modificato.**

## 3. Scenari concreti e flussi UX ipotetici

### A. Esportazione normale, nessuna frizione aggiunta

L'artista continua a utilizzare **Esporta CSV** come oggi. Il file rimane apribile in Calc/Excel senza righe tecniche preliminari, colonne occultate o modifica di delimitatore/BOM. Un'indicazione discreta può rimandare alla pagina locale «Come leggere questo registro». **Prima fase: sola documentazione**, senza nuovi comandi né dialoghi obbligatori.

### B. Consulente che automatizza un foglio

Il consulente consulta un **dizionario dati pubblicamente leggibile**, senza necessità di avviare Repertica: `kind`, nomi/ordine delle colonne, tipo logico (non semplicemente tipo SQLite), unità, semantica delle date, sensibilità, chiavi, relazioni, righe escluse/assenze e limiti. Il suo script riconosce la versione di contratto dichiarata dalla documentazione e confronta effettivamente l'header ricevuto. Se sconosciuto, **si ferma con diagnostica**, invece di riassegnare una colonna a un altro significato.

### C. Evoluzione del software

Una futura modifica di una query o dell'intestazione è valutata confrontando il contratto precedente: aggiunta di una colonna, modifica di denominazione, tipo, significato, granularità o regola di protezione testo devono essere **registrate esplicitamente**. Il mantenimento del nome della colonna **non dimostra** compatibilità se cambia il significato. Non promettere la disponibilità perpetua di tutte le versioni storiche senza una decisione del proprietario.

### D. Metadati opzionali, fase eventuale

Solo se la documentazione non è sufficiente, valutare un comando distinto «**Esporta descrizione del registro**» che produca un **sidecar** `<nome>.schema.json` nella cartella scelta, **senza modificare il CSV**. Prima conferma di destinazione e controllo dei conflitti; se già esiste un file omonimo, nessuna sovrascrittura. Il sidecar descrive colonne e versione, non include dati di opere/clienti; non viene creato automaticamente in cartelle condivise o attraverso un'esportazione ordinaria. **Alternativa più economica:** scaricare manualmente lo schema statico dal repository in modo indipendente dall'app.

## 4. Contratto e struttura dati proposti — soltanto descrittivi

**Possibile schema di dizionario**, non approvato né implementato:

```json
{
  "format_id": "repertica-ledger-csv",
  "register_kind": "receipts",
  "contract_version": "1.0",
  "encoding": "UTF-8-BOM",
  "delimiter": ";",
  "record_separator": "CRLF",
  "header_required": true,
  "scope": "all-records-not-UI-filtered",
  "columns": [
    {
      "position": 1,
      "name": "receipt_id",
      "logical_type": "uuid",
      "nullable": false,
      "role": "primary-identifier",
      "privacy": "internal-identifier"
    },
    {
      "position": 9,
      "name": "amount_cents",
      "logical_type": "integer",
      "unit": "minor-currency-unit",
      "related_currency_column": "currency",
      "nullable": false,
      "privacy": "commercial"
    }
  ]
}
```

*L'esempio è abbreviato intenzionalmente: il file definitivo dovrebbe elencare **tutte** le colonne dei **dodici** registri, senza inferire tipi e nullabilità soltanto dal loro nome.* Elementi da specificare:

- **Identità e versione indipendenti** per `kind`: distinzione tra evoluzione compatibile (es. colonna nuova documentata) e incompatibile (rinomina, rimozione, significato cambiato); regola sui consumer che leggono le colonne **per nome** e quelli che le leggono **per posizione**. Un'aggiunta in coda può comunque rompere i consumer che richiedono cardinalità fissa: la compatibilità va qualificata, non promessa.
- **Grana del record e cardinalità**: `sales` è una riga per vendita, `receipts` una riga per incasso inserito manualmente, `allocations` una riga per parte associata alla vendita; `balances` è un riepilogo calcolato, **non** una transazione bancaria. Nessun dato presume ricevute pagate o spese fiscalmente verificate.
- **Tipi e unità**: denaro intero in centesimi, percentuali in punti base, dimensioni in centimetri con punto decimale, stringhe di codice da non trasformare in numeri, date solo se note e nel formato effettivamente prodotto; data calendario e timestamp non sono intercambiabili.
- **Vuoti e stringhe protette**: `NULL` e stringa vuota non sono distinguibili nel CSV corrente; l'apostrofo aggiunto contro formula injection è **un carattere nel file**, non prova di una modifica nel database. Documentarne esplicitamente la regola; nessuna «reversione» automatica sicura in uno script esterno senza conoscere il contesto.
- **Relazioni e provenienza**: UUID stabili e FK concettuali fra registri; percorsi media relativi **non trasferiscono i byte**; `current_artwork_title` su un certificato è il titolo corrente, mentre `snapshot_json` è il contenuto storico; le quote di vendita non significano somme già liquidate.
- **Classificazione privacy per colonna**: contenuto personale, dato economico, percorso locale relativo, testo liberamente inserito, dato pubblicabile **solo su revisione**. Nessuna etichetta «anonimo» per i registri integrali.
- **Header e controlli**: mappare nome, numero/ordine colonne ed eventuali valori enumerati del `kind`. Un documento di schema **non** certifica hash del CSV, unicità degli ID o qualità dei dati. Se si richiede integrità dei byte, fare riferimento al manifest di TECH-055, non inventarne un altro.

**API solo ipotetiche:** `describeCsvRegister(kind) -> ContractDescriptor`; `compareCsvHeader(header, declaredVersion) -> CompatibilityReport`; `exportCsvSchemaSidecar(kind, chosenPath) -> Result`. Non sono comandi presenti nel repository, né c'è una decisione di aggiungerli.

## 5. Offline, privacy, sicurezza, impostazioni, lingue

- **Offline-first:** dizionario statico leggibile senza applicazione e senza Internet una volta copiato, nessun login, servizio terzo, SaaS, abbonamento o server; il CSV resta proprietà dell'utente e può essere analizzato liberamente.
- **Privacy:** lo schema descrive nomi di campi, non valori; i CSV reali contengono nomi, note, contatti e importi. Eventuale sidecar non deve riportare percorsi di cartelle personali, UUID dell'archivio, nomi di clienti, esempi reali o campioni di righe. Nessun invio automatico a consulenti.
- **Sicurezza:** non allentare la protezione contro formule; non interpretare contenuti CSV come istruzioni/eseguibili. Se si esporta un sidecar, usare percorso esplicito, nessuna sovrascrittura né link simbolici non attendibili, gestione di unità USB/NAS scollegate; non promettere scrittura atomica sui filesystem remoti.
- **Preferenze:** nessuna configurazione necessaria per consultare documentazione; eventuale sidecar soltanto con scelta esplicita, preferenza non preattivata e rimovibile. Non introdurre versioni compatibili selezionabili dall'utente prima di conoscerne necessità e costi.
- **Multilingua:** UI nella lingua di sistema, fallback **inglese** e selezione manuale secondo il prodotto; descrizioni di aiuto traducibili (IT/EN/ES/FR/DE), mentre `kind`, chiavi JSON, nomi di colonna, unità macchina e numeri di versione non dipendono dal locale. Le convenzioni CSV **non** mutano in base alla lingua del desktop.
- **Accessibilità:** versione testuale Markdown consultabile con lettore di schermo, esempi semplici e tabella ricercabile per `kind`; eventuale controllo header comunica errori in testo, non solo colore.

## 6. Dipendenze, alternative, rischio e complessità

**Dipendenze concettuali:** codice e specifica reale dell'esportatore in `src-tauri/src/export.rs`, elenco `src/app/csv-registers.ts`, modelli SQLite, `docs/DATA-PORTABILITY.md`, evoluzione dello schema. Non sono necessarie nuove dipendenze per la **fase documentale**. Una validazione machine-readable futura richiede definire un formato/validator affidabile, senza imporre software proprietario.

| Percorso | Beneficio | Limite |
| --- | --- | --- |
| **0. Nessun intervento**: usare documentazione e header attuali | Costo nullo; già possibile | Contratti non versionati per consumer ripetuti |
| **1. Dizionario Markdown e changelog per registro** | Molto portabile, testabile manualmente | La conformità resta verificata a mano |
| **2. Dizionario machine-readable statico facoltativo** | I consumer possono controllare i tipi dichiarati e l'header | Manutenzione di dodici contratti e rischio di divergenza |
| **3. Sidecar esportato su richiesta** | Associa una descrizione al CSV consegnato | Nuova UI e nuovi file da mantenere; non prova autenticità |

**Rischi principali:** descrittori obsoleti rispetto al codice, falsa certezza di retrocompatibilità, malinterpretazione fiscale dei registri, confusione fra `NULL` e vuoto, perdita dell'escaping protettivo nei parser esterni, sovrapposizione con il formato semantico TECH-001. Mitigazione: confronto delle intestazioni, campioni sintetici e dichiarazione esplicita delle limitazioni.

**Complessità:** **bassa-media** per censimento/changelog e regole; **media** per contratti machine-readable mantenuti e prove di compatibilità; **media-alta** se si introducono opzioni di produzione di versioni legacy e migrazioni dei consumer. Non presupporre quest'ultima fase.

## 7. Criteri di accettazione e test per una decisione futura (non eseguiti)

1. Per ognuno dei **12** `kind`, ogni nome/posizione nell'header del codice coincide con il dizionario versionato, senza omissioni; `collectors`, `receipts`, `certificates`, `allocations` e `raw_files` sono verificati anche su casi sintetici con campi mancanti.
2. Un consumer indipendente che legge **per nome** e uno che legge **per posizione** riconoscono differenze e interrompono l'elaborazione su schema incompatibile; nessuna correzione silenziosa di dati storici.
3. Fixture di prova con accenti, separatori, testo multilinea, virgolette, codice con zeri iniziali, data senza ora, denaro in centesimi, percentuali in punti base, `NULL` e stringhe che iniziano con `=`, `+`, `-`, `@`: il contratto descrive i byte effettivi e i limiti di reversibilità.
4. Nel caso di una colonna aggiunta in una futura versione, changelog ed esito di compatibilità rendono evidente la variazione prima che lo script confronti colonne sbagliate; il numero `PRAGMA user_version` non è scambiato per quello del CSV.
5. CSV individuali **identici ai precedenti** nella fase di sola documentazione; niente righe di versione dentro i dati, niente nuove informazioni personali, nessuna regressione delle difese per fogli elettronici.
6. Se si valuta il sidecar, Windows/macOS/Linux e USB/NAS vengono provati con nomi esistenti, dischi scollegati, permessi insufficienti; nessun file precedente sovrascritto e nessun sidecar creato implicitamente.
7. Prova con **3–5 professionisti** o veri script consumer (su **dati fittizi**) per verificare che il dizionario risolva ambiguità concrete; senza riscontri, archiviare la proposta come non necessaria.

## 8. Requisiti certi, ipotesi e domande al proprietario

**Certo dal repository:** dodici registri integrali individuali, intestazioni scritte nel codice, convenzioni CSV e protezione contro formule, dati con UUID, importi in centesimi, `NULL` non distinguibile dal vuoto, CSV non reimportabili e backup completo separato. `docs/DATA-PORTABILITY.md` segnala già un cambiamento di colonne con potenziale impatto sui consumer.

**Ipotesi:** esistenza di flussi periodici con consulenti o script, valore di un contratto versionato per registro, JSON di esempio, regole esatte dei numeri di versione, UI facoltativa per sidecar, costo giustificato da consumatori esterni reali.

**Domande per la revisione personale:**
1. Gli utenti esportano abitualmente CSV verso gestionali/consulenti o usano soprattutto Excel/Calc una tantum?
2. È sufficiente completare `docs/DATA-PORTABILITY.md` con un dizionario stabile e storico delle colonne, senza nuove funzioni?
3. Il contratto dovrebbe versionare ciascun `kind` separatamente o un insieme complessivo, sapendo che gli aggiornamenti non riguardano sempre tutti i registri?
4. Occorre distinguere compatibilità dei parser per nome e per posizione o basta dichiarare formalmente che solo la lettura per nome è sostenuta?
5. È utile un file sidecar per chi condivide un solo CSV, oppure creerebbe confusione con il manifest multi-CSV di TECH-055?

## 9. Fasi ipotetiche, nessuna autorizzata

1. **Ricognizione di consumer reali**, casi sintetici e identificazione degli errori di lettura più ricorrenti. Se il problema non è confermato, fermarsi.
2. **Documentazione umana** di colonne/semantica/cronologia per i 12 registri, con esempi privi di dati reali; non richiede cambiare il software.
3. **Solo se utile:** contratto machine-readable statico versionato, con confronti delle intestazioni e golden fixture.
4. **Solo dopo una decisione esplicita:** sidecar opzionale e relativo UX/test multi-OS. Nessuna retroesportazione legacy garantita per default.

> Esclusivamente proposta documentale, **non approvata**, non assegnata allo sviluppo e priva di modifiche operative.
