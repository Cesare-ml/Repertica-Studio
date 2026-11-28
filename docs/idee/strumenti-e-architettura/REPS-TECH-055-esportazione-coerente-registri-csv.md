# REPS-TECH-055 — Esportazione coerente di più registri CSV con manifest locale

- **ID stabile:** REPS-TECH-055
- **Data della ricognizione:** 2026-11-23
- **Stato:** **Proposta — da valutare**
- **Priorità proposta:** **P2 condizionata** a impieghi ricorrenti di più registri insieme; **P3 o nessuna implementazione** se bastano gli export singoli attuali.
- **Filone:** strumenti tecnici, interoperabilità e qualità del dato. **Nessuna approvazione o assegnazione allo sviluppo.**

## 1. Problema, destinatari e riscontri certi

Un artista o un consulente può dover analizzare insieme opere, vendite, quote di ripartizione, incassi e spese. Se esporta i registri separatamente mentre l'archivio cambia, i file possono descrivere **momenti diversi**: ad esempio il registro vendite viene creato prima di registrare un nuovo incasso e il registro saldi dopo. I file possono essere validi singolarmente ma **non costituire un insieme temporalmente coerente**. Questo è un rischio dedotto dal flusso, **non** una discrepanza rilevata negli archivi degli utenti.

**Fatti verificati su main il 23/11/2026:**
- La [documentazione sulla portabilità](../../DATA-PORTABILITY.md) elenca **12 registri CSV**: artworks, gallery, collectors, collector_photos, raw_files, certificates, sales, installments, allocations, balances, expenses, receipts. Sono esportazioni integrali, indipendenti dai filtri, non ripristinabili e prive dei byte dei media.
- Nel [modulo Rust export.rs](../../../src-tauri/src/export.rs), ogni chiamata a **export_ledger_csv** rende **un solo** registro sotto il mutex SQLite e poi crea **un file CSV singolo** senza sovrascrivere il percorso esistente. La snapshot è coerente per quel singolo output; **non** emerge un comando che renda coerenti temporalmente più output distinti.
- Restano già disponibili la protezione dai contenuti interpretati come formule di foglio elettronico, i centesimi interi, gli UUID, i controlli sulle relazioni SQLite, i file UTF-8 con BOM e separatore punto e virgola. **Non si propone di risolvere problemi già risolti.**
- Il CSV serializza NULL come campo testuale vuoto, aggiunge un apostrofo di difesa davanti ad alcuni testi e non comprende gli originali delle immagini. Non è un round-trip esatto del database, né un backup.
- Il [MVP](../../MVP.md) e [README](../../../README.md) descrivono backup completi SQLite/media/manifest SHA-256 nella cartella scelta, da lasciare distinti e invariati.

**Destinatari ipotetici:** artista che prepara statistiche per il commercialista, galleria che incrocia vendite e incassi, archivista che documenta periodicamente lo stato di cataloghi e registri, professionista che usa Calc o uno script locale. **La frequenza di questo bisogno non è stata verificata.**

## 2. Ambito distinto dalle idee esistenti

- [TECH-018](REPS-TECH-018-esportazioni-csv-scalabili-atomiche.md) studia memoria, durata del lock e pubblicazione sicura di **un CSV**. Qui la novità è la **coerenza di più registri tra loro**, più un manifest con copertura esplicita; streaming e pubblicazione sicura di TECH-018 sarebbero prerequisiti riutilizzabili, non una seconda implementazione.
- [TECH-001](REPS-TECH-001-pacchetto-dati-aperto.md) progetta un **nuovo interscambio semantico completo con possibile reimportazione e media**; qui soltanto un insieme di **CSV già disponibili**, senza ricostruzione dell'archivio, import o nuovi formati per le entità.
- [TECH-025](REPS-TECH-025-esportazioni-tabellari-minimizzate-condivisione.md) riguarda scelta di colonne/righe **da condividere**: questo pacchetto mantiene **registri integrali**, non minimizzati, e ne avverte il rischio privacy. [TECH-052](REPS-TECH-052-verifica-indipendente-backup-formato-v1.md) verifica i **backup**, non tabelle CSV.
- [TECH-054](REPS-TECH-054-pacchetto-catalogo-web-statico-offline.md) genera un sottoinsieme **pubblicabile** di opere e foto per il web, non registri interni economici.
- Nel filone [Gestione e UX, PR #14](https://github.com/Cesare-ml/Repertica-Studio/pull/14) si progettano modelli gestionali, documenti e presentazioni, **non** un contratto tecnico di esportazione multipla. Non se ne modificano le schede.

## 3. Scenari d'uso e flusso UX ipotetico

**A — Analisi economica locale.** L'artista sceglie vendite, ripartizioni, incassi, saldi e spese. Un'unica acquisizione coerente rende questi registri relativi allo **stesso stato**. Il programma non inventa contabilizzazioni, importi riscossi o corrispondenze con estratti bancari.

**B — Consegna controllata a un consulente.** Una galleria può scegliere tutti i registri o solo un gruppo, ma prima vede chiaramente quali includono contatti, note, importi, riferimenti ai certificati e percorsi relativi. Una selezione ridotta è dichiarata **parziale**, mai “anonima” o completa.

**C — USB rimossa, disco pieno o archivio modificato.** Se la preparazione fallisce, non deve comparire una cartella finale con manifest che sembri attestare il completamento. Nessun CSV preesistente o backup deve essere sovrascritto.

**Flusso proposto (non presente nel prodotto):**
1. **Impostazioni → Esportazioni → Pacchetto di registri CSV (proposta)**; l'esportazione singola attuale resta invariata.
2. Scegliere i registri fra i 12 disponibili; mostrare elenco completo, numero stimato di righe se calcolabile senza costi eccessivi, categorie sensibili e **cosa non viene incluso**: foto, RAW, allegati, altre impostazioni e valori NULL distinguibili.
3. Confermare un'operazione **solo locale**, scegliere liberamente una cartella di destinazione esistente, su disco/USB/NAS già montato, e produrre **una nuova sottocartella** senza collisioni di nome.
4. Eseguire i registri contro una stessa vista immutabile dello SQLite, mostrando avanzamento e stato “in preparazione”; un eventuale annullamento deve lasciare intatta la destinazione preesistente.
5. Verificare file effettivamente scritti, conteggi, checksum, e solo allora pubblicare il manifest e il nome finale; riepilogo: “Esportati N registri dalla stessa snapshot”, **non** “Backup creato”.
6. Possibilità di aprire la cartella o copiare il percorso; nessuna apertura automatica dei file in software esterni, invio email, upload o pubblicazione.

**Accessibilità:** lista di registri e avvisi con etichette testuali, navigazione tastiera, focus recuperabile, lettura tramite screen reader, progressi ed errori non affidati al colore, zoom elevato. Il consenso non può essere una casella preselezionata nascosta.

## 4. Dati e API puramente descrittivi

**Nessun nuovo modello di opera, vendita o contatto.** Riutilizzarebbe le query di export e la semantica attuale. Il solo metadato addizionale ipotetico è un manifest **di esportazione CSV**, distinto da quello dei backup:

    {
      "format": "repertica-csv-collection",
      "format_version": 1,
      "exported_at": "2026-11-23T03:00:00Z",
      "source_schema_version": "<valore effettivo di PRAGMA user_version>",
      "snapshot_id": "<identificatore effimero della sessione>",
      "selected_kinds": ["sales", "receipts", "balances"],
      "excluded_kinds": ["artworks", "..."],
      "files": [
        {"kind": "sales", "path": "sales.csv",
         "rows": 0, "bytes": 0, "sha256": "<digest calcolato>"}
      ]
    }

**Esempio schematico, non un formato approvato.** I placeholder non rappresentano un file prodotto; l'identificativo di snapshot è effimero e **non** identifica il proprietario o l'archivio nel tempo. Chiavi macchina stabili, eventuale versione del contratto CSV per ciascun registro **solo dopo** censimento delle colonne effettive; non dedurre compatibilità dal solo numero dello schema SQLite. Niente identificativi o percorsi assoluti del computer nel manifest; no contenuti personali se non nei CSV scelti.

**Contratti astratti, inesistenti oggi:**
- **planCsvCollection(kinds, destination)** → registri inclusi, rischi privacy, capienza stimata, possibili blocchi.
- **captureConsistentView()** → riferimento temporaneo a uno **stesso** stato SQLite, con rilascio certo delle risorse.
- **writeCsvCollection(view, plan)** → file e conteggi verificati; nessuna scrittura sul database sorgente.
- **inspectCsvCollection(folder)** → manifest/bytes/hash/registri mancanti; **non** prova l'integrità dei dati originali o la ripristinabilità.

La consistenza può essere ottenuta con **transazione di lettura coerente** o con **copia SQLite temporanea verificata**. Non scegliere anticipatamente: una transazione lunga può far crescere il WAL e influire sulle scritture; una copia richiede spazio, permessi e politiche severe per la sua eliminazione. Evitare di mantenere il mutex globale per minuti se gli archivi sono grandi. Il manifest descrive file e snapshot, **non** dimostra autenticità crittografica, né certifica contabilità.

## 5. Offline, sicurezza, privacy, impostazioni, lingue e compatibilità

- **Offline completo:** nessun account, login, abbonamento, cloud, API remota o server di sincronizzazione; lettura SQLite locale, filesystem scelto dall'utente, nessun consenso implicito a condividere i risultati.
- **Riservatezza:** i 12 CSV possono contenere nomi, contatti, note riservate, vendite, importi e snapshot dei certificati. Preview dei nomi dei registri e categorie, **non** copia dei valori privati nel manifest, nei log o in messaggi di errore. Se il supporto è condiviso, avviso prima della conferma; i file rimangono **in chiaro**. Non definire “sicuro per la pubblicazione” un set parziale.
- **Destinazioni:** nuova sottocartella e nessuna sovrascrittura, blocco della cartella interna dell'archivio attivo, rifiuto di link/percorso non attendibili, limite ragionevole alle dimensioni e nomi dei file fissi, no esecuzione di contenuti del CSV. La scrittura temporanea e la finalizzazione vanno testate realmente su NTFS, APFS, ext4, unità rimovibili e NAS; l'atomicità non può essere garantita universalmente dai filesystem remoti.
- **CSV attuale da preservare:** UTF-8 BOM, separatore “;”, CRLF, quoting, difesa formule, numeri in centesimi interi, date/codici e UUID. **NULL vs stringa vuota resta indistinguibile** nel CSV: non promettere reversibilità. Un'applicazione di fogli elettronici può riformattare codici e date all'apertura/salvataggio: il manifest deve descrivere **i byte originariamente esportati**, non l'interpretazione di Excel/Calc.
- **Compatibilità:** non modificare né rinominare i 12 comandi/registri individuali. Eventuali colonne nuove richiedono aggiornamento della documentazione e controllo di compatibilità dei consumer, senza pretendere che export con schemi diversi abbiano le stesse colonne. Non trattare il pacchetto come backup SQLite/media o come sostituto degli export semantici futuri.
- **Impostazioni facoltative:** memorizzare localmente solo ultime selezioni o cartella, **se l'utente lo desidera**; niente schedulazione automatica o condivisione ricorrente per default. UI nella lingua di sistema con fallback **inglese** e scelta manuale coerente con IT/EN/ES/FR/DE; etichette tradotte, nomi macchina dei registri e chiavi JSON invarianti, data/ora presentate nel fuso locale ma marcate UTC nel manifest.

## 6. Dipendenze, alternative, complessità e rischi

**Dipendenze logiche:** esportatore Rust esistente e suoi controlli, schema e migrazioni SQLite, snapshot/coerenza e test di [TECH-018](REPS-TECH-018-esportazioni-csv-scalabili-atomiche.md), preflight dello spazio, writer/checksum progressivo e dialogo nativo della cartella. Nessuna dipendenza aggiuntiva o libreria è approvata.

| Alternativa | Vantaggi | Limiti |
| --- | --- | --- |
| **A. Nessuna funzione nuova: esportare i CSV singolarmente mentre l'archivio resta fermo** | Minimo costo, già utilizzabile | Richiede disciplina; non fornisce una prova meccanica di snapshot comune |
| **B. Guida documentale “come analizzare insieme i registri”** | Indipendenza e chiarimenti semantici a basso costo | Non impedisce esportazioni temporalmente discordanti |
| **C. Pacchetto multi-CSV coerente e manifest minimo (questa ipotesi)** | Analisi integrata senza inventare nuovi formati dei record | Complessità di snapshot, risorse, dati riservati ed esperienza su supporti esterni |
| **D. Formato semantico completo TECH-001** | Potenzialmente più fedele e reimportabile | Progetto più ampio, non giustificato solo dall'analisi finanziaria |

**Rischi:** confondere file coerenti con contabilità fiscale verificata; riferimenti senza byte dei media; spesa spazio e WAL; uso improprio di un manifest come prova di autenticità; esportazioni incomplete su volumi scollegati; file privati lasciati in temporanei; differenza fra conteggi attesi e righe risultanti; aumento delle superfici di manutenzione quando i registri cambiano. **Complessità:** media-alta per snapshot/bundle; bassa per una guida e test manuali; più alta se si pretende isolamento senza alcun impatto sulle modifiche concorrenti.

## 7. Criteri di accettazione e prove proposte (nessun test creato)

1. **Fixture sintetica con relazioni:** opere, collezionisti, vendite con ripartizioni, rate, incassi parziali, spese, certificati, foto/RAW; tutti i registri inclusi devono avere stessa origine temporale e riferimenti coerenti. Testare anche set parziali dichiarati e set vuoto rifiutato.
2. **Concorrenza controllata:** modificare incasso/spesa mentre il pacchetto viene preparato; nessuna combinazione “vendita prima / saldo dopo” nella stessa esportazione. Nessuna scrittura al database e nessuna modifica ai certificati storici.
3. **Controlli anti-falso successo:** interrompere processo, simulare disco pieno, USB scollegata, scrittura parziale, manifest mancante, file modificato o hash errato, permessi mancanti; nessun nome finale dichiarato completo, nessuna sovrascrittura.
4. **Semantica finanziaria:** centesimi e quote storiche identici al singolo CSV; saldi coerenti con incassi **registrati**, non estratti bancari; nomi Unicode, testo multilinea, prefissi da formula, quote, date, NULL e stringhe vuote trattati come nel comportamento attuale.
5. **Privacy:** nessun recapito o nota nei metadati del manifest; anteprima dei rischi, nessun invio di rete, nessun dato personale in log; set completo mai classificato automaticamente come condivisibile.
6. **Prestazioni e dispositivi:** misurare uso RAM, tempo sotto lock e crescita WAL con archivi grandi; prove native Windows/macOS/Linux, unità USB/NAS e cartelle con caratteri Unicode.
7. **Interoperabilità:** apertura con Calc e parser CSV standard, verifica indipendente di SHA-256/numero righe (con newline dentro i campi); un CSV alterato dopo l'esportazione viene segnalato. Nessuna pretesa di reimportabilità.
8. **Confronto pratico:** almeno tre operatori devono spiegare la differenza fra questo set, un backup completo e TECH-001. Se una guida/esportazione individuale è sufficiente, **non implementare il pacchetto**.

## 8. Fasi proposte e questioni per il proprietario

- **Fase 0 — Solo ricerca:** chiedere ad artisti/gallerie/consulenti quante volte uniscono davvero più registri, quali, con che dimensioni e frequenza; provare il workaround dei singoli CSV in una sessione senza modifiche.
- **Fase 1 — Eventuale specifica:** definizione di snapshot, copertura, schema/manifest con dati sintetici, separazione esplicita dai backup e dal formato TECH-001.
- **Fase 2 — Solo dopo valutazione:** prova tecnica di snapshot e writer usando i registri correnti, con misure di RAM, lock, WAL e fallimenti su filesystem; non è autorizzata da questa scheda.
- **Fase 3 — Solo se utile:** UX accessibile per selezione, avvisi privacy, annullamento e report; test nativi e compatibilità prima di una eventuale release.

**Domande aperte:**
1. Quali combinazioni reali di registri vengono analizzate insieme, e quanto spesso?
2. Basta “esporta tutto in una sessione senza modifiche” oppure la coerenza temporale va garantita dal programma?
3. Se si selezionano solo alcuni registri, quali dipendenze e omissioni devono essere segnalate?
4. Un manifest di hash e conteggi serve ai professionisti o crea solo complessità? Sarebbe preferibile un documento umano leggibile?
5. Quali limiti di dimensione, spazio temporaneo e tempo di blocco sono accettabili su USB/NAS?
6. È preferibile aspettare TECH-018 e TECH-001 anziché avviare un modulo intermedio?

**Esito:** esclusivamente **Proposta — da valutare**. Questa scheda non autorizza creazione di codice, test, workflow, impostazioni, dipendenze, issue, PR di implementazione o merge.