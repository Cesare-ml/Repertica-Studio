# REPS-TECH-018 — Esportazioni CSV scalabili e pubblicazione sicura dei file

- **ID stabile:** REPS-TECH-018
- **Data:** 2026-11-21
- **Stato:** **Proposta — da valutare**
- **Priorità proposta:** **P2** (potenzialmente P1 se i test evidenziano blocchi su archivi reali; non approvata)
- **Ambito:** robustezza e prestazioni dell'**esportatore CSV già presente**, non nuovi registri, sincronizzazione, importazione o nuove funzioni gestionali.
- **Natura del testo:** esclusivamente analisi e specifica ipotetica. Nessun incarico di implementazione.

## Motivazione, evidenze e destinatari

**Riscontri verificati su main (21 novembre 2026):**

1. In **src-tauri/src/export.rs**, la funzione **render_csv** accumula tutti i record del registro in una singola **String** Rust prima della scrittura. Durante la generazione mantiene il mutex della connessione SQLite condivisa con i comandi dell'app; solo la successiva scrittura avviene fuori dal lock. Per esportazioni molto grandi, il consumo di memoria del testo cresce con l'output e la generazione può tenere occupato l'archivio. **È un rischio architetturale osservabile dal codice, non un malfunzionamento misurato**.
2. **export_file** usa correttamente **create_new** (nessuna sovrascrittura) e **sync_all**; se la scrittura restituisce errore tenta di eliminare il file. Tuttavia il percorso finale viene creato **prima** che l'export sia completato: un crash, un arresto forzato o la perdita del volume possono lasciare un CSV troncato che sembra definitivo. Non è documentata una pubblicazione finale atomica.
3. **docs/DATA-PORTABILITY.md** descrive **otto** registri: artworks, gallery, collectors, certificates, sales, balances, receipts, expenses. Contratto corrente: UTF-8 con BOM, separatore punto e virgola, righe CRLF, celle testuali tra virgolette, protezione contro formule, centesimi interi. Sono esportazioni **integrali**, non un backup e non un formato reimportabile.
4. **src/app/api.ts** espone un comando Tauri che restituisce soltanto il percorso a lavoro concluso; **src/app/csv-registers.ts** definisce gli otto tipi e il testo della UI. Non emerge un protocollo di progresso o annullamento per questi export.

**Utenti:** artista con migliaia di opere e fotografie catalogate, galleria che esporta periodicamente clienti e vendite, archivio con note molto estese, consulente che lavora su unità USB o NAS montati. La frequenza reale di export di dimensioni elevate è **da verificare**, non un bisogno già dimostrato.

**Problema che la proposta affronta:** con archivi crescenti, mantenere un'esportazione locale completa, prevedibile nella memoria utilizzata e inequivocabile quanto al suo completamento, senza perdere la semplicità dei normali CSV.

## Scenari d'uso e flussi UX proposti

### A. Esportazione ordinaria

L'utente seleziona il registro come oggi, apre il dialogo nativo e sceglie nome e cartella anche su disco esterno o NAS già montato. La UI segnala «Preparazione», «Esportazione», «Verifica e completamento»; mostra righe elaborate e byte prodotti **soltanto se disponibili e attendibili**. In caso di successo visualizza il percorso del **solo file completo**, senza apertura o condivisione automatica. I filtri della lista e l'anno mostrato sullo schermo **non restringono** l'export.

### B. Archivio grande o note molto estese

La generazione procede per record/buffer limitati, senza creare una copia dell'intero CSV in RAM. L'utente può continuare almeno le operazioni di lettura, compatibilmente con il modello di snapshot scelto; l'eventuale impossibilità temporanea di scrivere deve essere segnalata con chiarezza. Una futura opzione «Annulla» ferma la generazione in un punto sicuro e non lascia mai un file finale dichiarato riuscito.

### C. Spazio insufficiente, USB scollegata o arresto inatteso

Durante il lavoro esiste al più un file temporaneo **riconoscibile come incompleto** nella destinazione, mai un CSV finale apparentemente valido. Un errore produce un esito esplicito, non un messaggio di successo. Alla riapertura la UI può spiegare come riconoscere una copia parziale **creata dall'app** e proporne l'eliminazione manualmente confermata; non scansiona o rimuove indiscriminatamente i file altrui.

### D. Esportazione verso NAS

L'applicazione segnala che garanzie di atomicità, sincronizzazione e disponibilità dopo un guasto dipendono anche dal filesystem remoto. Se il volume non offre le garanzie richieste, non promettere «file sicuramente atomico». Valutare un messaggio mirato o un flusso di copia verificata con precauzioni supplementari.

## Specifica tecnica ipotetica — non implementata

### Generazione progressiva e consistenza

- **Streaming:** codifica delle righe direttamente in un writer con buffer limitato e gestione degli errori di I/O; nessuna **String** proporzionale alla dimensione dell'intero registro. Preservare esattamente ordine delle colonne, escape, BOM, CRLF, ordinamenti SQL, contenuti Unicode, newline incorporati, NULL vuoti, centesimi e protezione dalle formule dell'attuale esportatore.
- **Snapshot coerente:** il registro deve riflettere un singolo stato SQLite. **Ipotesi A:** connessione read-only separata con transazione di lettura e snapshot WAL; valutare compatibilità con SQLite, busy timeout e chiusura della transazione. Non assumere che una query su connessione distinta sia automaticamente consistente senza transazione. **Ipotesi B:** snapshot SQLite temporaneo coerente, poi export dalla copia (più spazio/tempo, meno lock prolungati). **Alternativa C:** mantenere il mutex condiviso durante l'intera generazione (semplice ma blocca a lungo le scritture). La decisione richiede misure.
- **Attenzione al WAL:** una lettura lunga può impedire il checkpoint e far crescere il WAL mentre altri comandi scrivono. Progettare monitoraggio, limiti ragionevoli, annullamento e messaggi; non trasformare la proposta in un servizio di sincronizzazione.
- **Finalizzazione sicura:** creare un file temporaneo con nome univoco **nella cartella scelta**, scriverlo, controllare gli errori, flush/sync e solo dopo pubblicare il nome definitivo con un'operazione che **non sovrascriva** una destinazione concorrente. Una semplice verifica «esiste?» seguita da rename non basta: va evitata la race. Le garanzie di rename e durabilità della directory differiscono fra Windows, macOS, Linux e filesystem di rete e vanno accertate prima di formulare promesse. Non sostituire mai documenti già presenti.
- **Interruzioni:** l'annullamento restituisce uno stato distinto dal fallimento; la cancellazione dei temporanei deve riguardare soltanto quelli creati e identificati in modo affidabile dalla specifica operazione. Se la rimozione fallisce, segnalarne la presenza senza fornire un falso successo.
- **Coesistenza:** al massimo un numero limitato di esportazioni attive per archivio, da definire dopo misure; non avviare processi, schedulazioni o invii senza un gesto esplicito.

### Dati e ipotesi di API (solo descrittive)

Non occorre necessariamente cambiare lo schema SQLite nella prima fase. Un job in memoria potrebbe avere campi concettuali:

| Campo | Significato |
| --- | --- |
| export_id | Identificativo effimero del lavoro, non dell'archivio |
| register_kind | Uno degli otto tipi CSV esistenti |
| phase | queued / preparing / writing / finalizing / completed / cancelled / failed |
| processed_rows | Numero effettivo di record codificati, se disponibile |
| output_bytes | Byte effettivamente scritti, se disponibili |
| started_at | Istante locale di avvio normalizzato, per il solo progresso |
| error_code | Codice tecnico minimizzato; non contenuti o path privati |

API concettuali, **non** comandi presenti: **startCsvExport(kind, target) → export_id**; **getCsvExportStatus(id) → progress**; **cancelCsvExport(id) → esito**. Un canale di eventi Tauri locale potrebbe ridurre il polling, ma non è una dipendenza o scelta approvata. Le notifiche non devono esporre il CSV né i dati dei clienti.

**Conservazione del contratto:** tutti gli otto export restano individuali, completi e non filtrati; non generare indici commerciali, aggregazioni fiscali, import o pacchetti semantici in questa proposta. Una eventuale esportazione coordinata di tutti i registri con uno snapshot comune richiede una decisione separata e coordinamento con **REPS-TECH-001**.

## Offline, privacy, sicurezza, impostazioni e multilingua

- **Offline e indipendenza:** nessuna rete, account, licenza a pagamento, abbonamento, telemetria, API esterna o cloud imposto. Il NAS è trattato esclusivamente come filesystem già montato dall'utente.
- **Privacy:** i CSV sono intenzionalmente in chiaro e possono contenere contatti, indirizzi, prezzi e note. Avvertimento chiaro prima del salvataggio in cartelle pubbliche/condivise. File temporanei e finali sono entrambi sensibili; minimizzare permessi d'accesso in base all'OS e non depositare frammenti in cache o log. Non registrare righe, nomi cliente, indirizzi o percorsi assoluti in report generali.
- **Sicurezza:** cartella e nome finale scelti esplicitamente; controlli di destinazione e symlink, rifiuto di scrittura dentro l'archivio attivo, nessuna sovrascrittura, attenzione a cambi di volume/percorso tra selezione e apertura. Assenza di script o formule eseguibili derivanti dai valori esportati; preservare l'escaping già esistente.
- **Impostazioni:** nessuna preferenza obbligatoria. Eventuale memorizzazione **locale e facoltativa** dell'ultima cartella; opzione di disabilitare progresso dettagliato se costoso; nessun export programmato predefinito.
- **Multilingua e accessibilità:** lingua del sistema con fallback inglese e selezione manuale come già previsto. Tradurre fasi, errori, conferme, annullamento e messaggi di spazio/disco scollegato; preservare nomi stabili delle colonne per interoperabilità. Progress bar e stato accessibili da tastiera e screen reader, annunci non invasivi, indicazione testuale quando la percentuale non è nota. Non tradurre valori inseriti dall'utente.

## Dipendenze, alternative, rischi e complessità

**Dipendenze progettuali:** attuale esportatore Rust, SQLite in WAL, modello di mutex dei comandi, dialoghi Tauri, localizzazione UI, otto registri e relativi test. Nessuna nuova libreria è richiesta **per questa analisi**; l'eventuale sviluppo deve giustificare ogni dipendenza. Non si postulano cambiamenti a backup, schema, certificati o modelli economici.

**Alternative:**
1. **Mantenere l'esportatore attuale**, corredato da benchmark e limite documentato: costo zero, sensato se gli archivi reali sono piccoli.
2. **Solo file temporaneo e pubblicazione finale**, senza streaming: mitigazione più semplice del rischio crash ma mantiene consumo RAM crescente e lock lungo.
3. **Solo streaming con mutex esclusivo**, senza connessione separata: semplifica la consistenza ma può congelare le modifiche dell'utente.
4. **Snapshot temporaneo SQLite**, poi esportazione: semplifica l'isolamento a costo di spazio e doppia I/O. Va confrontato sul campo.

**Rischi specifici:** file parziali scambiati per completi; crescita WAL; esaurimento spazio; progressi inaccurati; race tra controllo del nome e pubblicazione; filesystem che non garantisce rename atomico/durabile; annullamento durante sync; doppio evento di completamento; incompatibilità CSV per applicazioni terze. Possibile maggiore complessità ingiustificata per dataset modesti. Non si promette recupero in caso di perdita fisica del disco.

**Complessità stimata:** media per writer a flusso e test di compatibilità; **medio-alta/alta** per progressi/annullamento, consistenza con scritture concorrenti e pubblicazione robusta cross-platform/NAS. L'utilità dipende dai benchmark.

## Criteri di accettazione per una futura valutazione tecnica

1. **Compatibilità:** per gli otto registri, confrontare output della versione attuale e della variante studiata su dataset fissi (vuoto, Unicode, multilinea, formula injection, allegati metadati, centesimi esatti, ordinamento, grandi note). Stesse righe, intestazioni e quoting; nessuna modifica silenziosa alle colonne.
2. **Memoria:** su archivi sintetici crescenti, dimostrare con misure ripetibili che il picco di RAM della sola generazione non cresce linearmente con i byte del CSV; riportare hardware, dataset, dimensioni e picco, senza fissare oggi un limite arbitrario.
3. **Coerenza:** durante modifiche concorrenti alla base dati, il singolo export corrisponde a uno snapshot consistente; misurare attesa UI, crescita del WAL e tempo di checkpoint. Nessun vincolo storico su otto esportazioni avviate separatamente.
4. **Errore/annullamento:** simulare disco pieno, cartella resa non scrivibile, USB rimossa, terminazione forzata e cancellazione durante la generazione. Il percorso finale non deve presentare una versione incompleta come riuscita; eventuali temporanei devono essere identificabili e rimovibili senza toccare file estranei.
5. **Concorrenza e filesystem:** due export allo stesso nome non sovrascrivono nulla; testare Windows, macOS, Linux, filesystem locali e NAS/USB reali, dichiarando quali garanzie non sono supportabili.
6. **Privacy/UX:** nessuna riga sensibile o percorso nei log/progressi, avviso esplicito per CSV in chiaro, messaggi nella lingua selezionata e fallback inglese, tastiera e lettori di schermo; nessuna connessione di rete necessaria.
7. **No regressioni:** restano validi i test attuali su transazioni, formule, nomi esistenti, export completi e integrità dell'archivio; i backup verificati restano il mezzo di ripristino.

## Fasi ipotetiche (non autorizzate)

- **Fase 0 — raccolta evidenze:** misurare export reali e sintetici; documentare picchi di memoria, file di note e volumi, lunghezza dei blocchi UI, tempi su SSD/USB/NAS; capire se serve davvero intervenire.
- **Fase 1 — completamento sicuro:** valutare file temporaneo e pubblicazione non distruttiva senza cambiare contratto CSV, UI o dati.
- **Fase 2 — streaming consistente:** confronto fra connessione read-only, snapshot temporaneo e mutex; benchmark e compatibilità.
- **Fase 3 — esperienza resiliente:** avanzamento, annullamento e recupero temporanei, localizzazione e accessibilità solo se l'utilità è confermata.
- **Fase 4 — collaudo multipiattaforma:** verifiche manuali con guasti e dispositivi reali prima di qualsiasi promessa di robustezza.

## Domande aperte al proprietario

1. Quante opere, ricevute e righe di galleria sono realistiche in un archivio professionale? Si sono verificati crash, lentezze o difficoltà pratiche durante export?
2. In caso di esportazioni lunghe, è più importante continuare a **modificare** le schede oppure garantire un'operazione semplice che blocchi temporaneamente le scritture?
3. È desiderabile un pulsante **Annulla** già nella prima evoluzione, oppure bastano pubblicazione finale sicura ed esito leggibile?
4. Quali filesystem (Windows/macOS/Linux, USB, NAS) sono davvero prioritari? Quali livelli di garanzia si possono documentare senza promettere l'impossibile?
5. Sono necessarie esportazioni dei registri **contemporanee e coordinate**, oppure rimangono processi separati come oggi (eventuale studio distinto da TECH-001)?
6. Quanto spesso gli utenti condividono CSV con terzi? Occorrono avvisi e istruzioni privacy più visibili, senza modificare l'esportazione integrale?

## Confini rispetto alle proposte esistenti

- **REPS-TECH-001:** pacchetto aperto e reimportazione semantica. Qui si studia **soltanto il motore degli otto CSV esistenti**; nessun nuovo formato o round-trip.
- **REPS-TECH-008:** viste e filtri salvati nell'archivio; questa proposta mantiene esplicitamente tutti i record, anche quando un filtro UI è attivo.
- **REPS-TECH-009 / 012:** catalogazione e preflight dei **backup/migrazioni**. I CSV non sono backup, non ripristinano media e non sostituiscono quelle salvaguardie.
- **REPS-TECH-015:** codici e report diagnostici generali. Questa scheda tratta affidabilità della scrittura e progressi specifici all'export, senza progettare un nuovo sistema generale di log.
- **REPS-UX-001…022 (PR #14):** catalogazione, rapporti e processi professionali; nessun nuovo processo gestionale è definito qui.

**Decisione sospesa:** validare dimensioni e bisogni reali prima di ritenere necessaria l'ottimizzazione. **Stato finale: Proposta — da valutare.**
