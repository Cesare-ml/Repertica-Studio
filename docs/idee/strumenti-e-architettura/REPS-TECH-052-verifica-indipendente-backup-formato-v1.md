# REPS-TECH-052 — Specifica pubblica e verifica indipendente dei backup completi, senza Repertica in esecuzione

- **ID stabile:** REPS-TECH-052
- **Data:** 2026-11-23
- **Stato:** **Proposta — da valutare**
- **Priorità proposta:** **P2 per specifica e prove indipendenti**; **P3 per un eventuale verificatore autonomo distribuito** (solo se necessario e sostenibile). Nessuna decisione approvata né sviluppo assegnato.
- **Ambito:** leggibilità e *verifica tecnica in sola lettura* dei **backup completi esistenti** da strumenti indipendenti dall'applicazione; **non** ripristino, conversione, reimportazione o nuovo formato di backup.

## 1. Motivazione, evidenza e destinatari

Un artista può conservare per anni copie su una cartella scelta liberamente, USB o NAS. Se il programma non è più installabile sul computer disponibile, oppure si deve controllare un backup prima di affidarlo a un restauratore digitale, una verifica che dipende solo dall'interfaccia corrente di Repertica non offre un percorso completamente indipendente. Questo contraddirebbe *nella pratica*, pur senza infrangere le funzioni attuali, l'obiettivo di massima libertà dell'utente nel lungo periodo.

**Verificato sul branch `main` in data 2026-11-23:**
- [`README.md`](../../../README.md), [`docs/MVP.md`](../../MVP.md) e [`docs/DATA-PORTABILITY.md`](../../DATA-PORTABILITY.md) descrivono backup autonomi in cartelle, snapshot `repertica.sqlite`, `media/` e `manifest.json` con SHA-256, ispezione prima del ripristino e conservazione dell'archivio precedente. L'interfaccia già verifica una singola copia: **non** si propone di rifare tale funzione.
- [`src-tauri/src/backup.rs`](../../../src-tauri/src/backup.rs) definisce il formato `repertica-studio-backup` versione `1`. Il manifest contiene `format`, `format_version`, `created_at`, `schema_version`, `files[]` con `path`, `bytes`, `sha256`. Il controllo nativo rifiuta record duplicati e percorsi non sicuri, sidecar SQLite `-wal`/`-shm`/`-journal`, file media non elencati, file speciali e collegamenti simbolici, e controlla SQLite, relazioni, determinati vincoli economici, riferimenti ai media e coerenza degli allegati agli incassi.
- In `main` lo schema dichiarato dal codice è attualmente `18` (`src-tauri/src/db.rs`); il verificatore presente nel prodotto accetta solo schemi supportati dalla propria versione. Le colonne e i vincoli specifici **cambiano con lo schema**.
- Non emerge dai documenti consultati una **specifica pubblica autonoma per verificare un backup v1 con un lettore indipendente da Repertica** o una fixture sintetica completa pensata per un verificatore di terzi. **Non è stato constatato alcun incidente di perdita dati né verificata domanda reale**: è un'opportunità di resilienza, da convalidare.

**Destinatari:** artista che cambia computer/OS, galleria con backup conservati su lungo periodo, archivista digitale, tecnico incaricato di valutare una copia prima del recupero. Il lettore deve avere accesso locale **autorizzato** alla cartella del backup.

## 2. Confini e confronto con le proposte precedenti

- [TECH-001](REPS-TECH-001-pacchetto-dati-aperto.md) progetta un **nuovo pacchetto semantico** con export/reimport in software terzi: qui **nessun nuovo pacchetto**, solo documentazione e verifica del backup v1 già prodotto.
- [TECH-009](REPS-TECH-009-catalogo-verifiche-conservazione-backup.md) progetta **catalogo, verifica periodica e conservazione dall'app**: qui verifica autonoma di **una copia selezionata**, anche quando l'app non è disponibile; niente scansione programmata o cancellazione.
- [TECH-006](REPS-TECH-006-backup-cifrati-portabili.md) riguarda cifratura futura; un controllo v1 in chiaro **non** deve fingere di supportare contenitori cifrati. [TECH-012](REPS-TECH-012-migrazioni-archivio-preflight-copia-sicurezza.md), [TECH-037](REPS-TECH-037-recupero-ripristino-interrotto-avvio-protetto.md) e [TECH-042](REPS-TECH-042-compatibilita-aggiornamenti-recupero-versione-offline.md) trattano migrazione, crash e compatibilità dell'app, non un validatore indipendente.
- [TECH-050](REPS-TECH-050-confronto-backup-senza-ripristino.md) confronta **due** copie già verificate: questa idea controlla **una** copia e **non** cerca differenze fra archivi. [TECH-043](REPS-TECH-043-provenienza-verificabile-build-installatori-offline.md) verifica installer, **non** i backup.
- [Filone Gestione e UX — PR #14](https://github.com/Cesare-ml/Repertica-Studio/pull/14): non si modifica la catalogazione, la gestione delle opere, le vendite, i contatti o l'interfaccia gestionale. Nessun file di quel filone viene toccato.

## 3. Casi d'uso e flusso di esperienza proposto

**Caso A — nuovo PC senza Repertica.** Una galleria possiede `ReperticaStudio-backup-.../` su SSD esterno. Consulta una descrizione pubblica del formato e, se davvero utile, esegue **un verificatore locale indipendente** su quella cartella. Ottiene un risultato comprensibile senza installare/avviare Repertica, aprire file di opere o inviare dati online.

**Caso B — archivio conservato per molti anni.** Un tecnico può riconoscere il manifest e calcolare gli hash con strumenti generici, aprire lo SQLite con software standard in sola lettura e capire ciò che *può* e ciò che *non può* concludere, anche quando lo schema supera la versione del verificatore.

**Caso C — copia incompleta o manomessa.** Il controllo segnala separatamente: percorso pericoloso, file mancante, hash/size differenti, struttura SQLite non valida, versione sconosciuta, vincoli specifici non verificabili. **Non** dichiara una copia recuperabile quando ha controllato solo gli hash.

**UX ipotetica, non implementata:**
1. Dalla documentazione pubblica l'utente vede «Come verificare indipendentemente un backup», differenza tra **corrispondenza degli hash**, **integrità strutturale** e **ripristino realmente collaudato**.
2. Se necessario, un comando/strumento autonomo offline riceve **una sola cartella scelta esplicitamente** (per esempio tramite selezione percorso del sistema o parametro locale). Prima di analizzare mostra che la copia contiene dati personali in chiaro.
3. Visualizza progressivamente fase, file verificati e controlli applicabili; al termine espone **Esito tecnico**, **Controlli non eseguiti**, **Versione di formato/schema riconosciuta** e avvertenza «La verifica non è un ripristino».
4. La persona può **scegliere** di salvare un report locale **minimizzato**, senza nomi di opere, persone, percorsi assoluti o contenuti dei documenti per impostazione predefinita. Nessun pulsante «Ripara», «Ripristina» o «Carica nel cloud».
5. Chiude senza alcuna modifica al backup o all'archivio attivo. La procedura ufficiale di ripristino resta quella di Repertica, col proprio controllo completo.

**Accessibilità:** progresso e errori leggibili testualmente, output utile anche a screen reader/terminale e senza colori obbligatori; errori recuperabili con codici stabili e spiegazioni semplici.

## 4. Contratto dei dati e ipotesi di API — solo descrittive

**Il manifest v1 è un fatto del codice attuale, non una nuova proposta di modifica:**

```json
{
  "format": "repertica-studio-backup",
  "format_version": 1,
  "created_at": "<data RFC3339>",
  "schema_version": 18,
  "files": [
    {"path": "repertica.sqlite", "bytes": 12345, "sha256": "<64 caratteri esadecimali>"},
    {"path": "media/<nome-sicuro>", "bytes": 4321, "sha256": "<64 caratteri esadecimali>"}
  ]
}
```

I numeri sono **esemplificativi**, non un backup generato o un nuovo contratto approvato. La specifica autonoma dovrebbe documentare **esattamente** limiti e semantica realmente applicati dal codice, inclusi: sole voci `repertica.sqlite` e `media/<file>`, nomi media ASCII sicuri, nessun percorso assoluto/`..`, unicità dei riferimenti, file regolari, directory `media/` non collegata simbolicamente, byte originali, `SHA-256` di ogni file, ordine non semanticamente significativo di `files[]`, dimensione massima del manifest e soglie di risorse. Tutti i dettagli vanno riconciliati con codice e test **alla data di pubblicazione della specifica**, non inventati in anticipo.

**Interfaccia descrittiva facoltativa, non esistente:**

```text
inspectIndependentBackup(folder, profile = "basic" | "full")
  -> {format, format_version, schema_version, inspected_at,
      manifest_checks, file_checks, sqlite_checks, domain_checks,
      skipped_checks, status, warnings, verifier_version}
```

- `basic`: validazione del manifest, dei percorsi, dei file media e degli hash; non rivendica validità SQLite o ripristinabilità.
- `full`: oltre a `basic`, verifica SQLite `integrity_check`, `foreign_key_check` e **solo i vincoli di dominio conosciuti e applicabili a quello schema**. Un verificatore indipendente non deve essere dichiarato equivalente a `inspect_backup_core` finché non sono testate tutte le condizioni correnti, comprese rate, quote, incassi, allegati, RAW, fotografie di collezionisti, revisioni e riferimenti storici dei certificati.
- `status` concettuale: `valid_for_checked_rules` / `invalid` / `incomplete_or_unsupported` / `interrupted`. La dicitura «backup verificato» senza specificare il **profilo** e la **copertura** è vietata.
- Report JSON opzionale (o testo semplice) contenente codici esito e contatori, non contenuti del database. L'utente può esportarlo su una **cartella liberamente scelta**, creando nuovo file senza sovrascrittura; anche il report tecnico può svelare informazioni sul volume/archivio, quindi avviso prima della condivisione.
- Nessun endpoint HTTP, account, servizio esterno o permesso MCP. Non si modifica l'API di Repertica, SQLite, il manifest v1 né i registri CSV. Nessun import/reimport o scrittura su archivi.

## 5. Comportamento offline, sicurezza, privacy e compatibilità

**Offline obbligatorio:** la lettura del backup avviene senza rete, credenziali, telemetria o obbligo di iscrizione. La cartella può essere su unità USB o NAS **già montato** dall'OS; Repertica non deve gestire provider cloud o password di rete. Il controllo segue soltanto il percorso selezionato.

**Backup come input non attendibile:** non seguire symlink/junction o percorsi fuori dalla directory, non eseguire codice incorporato nel database o nei documenti, non aprire automaticamente allegati, non lanciare SQL presente nei dati. Verificare in streaming gli hash con memoria limitata, bloccare manifest troppo grandi e rilevare file cambiati tra prima e dopo la lettura. I sidecar `-wal`, `-shm` e `-journal` non fanno parte del backup pubblicato: non lasciarli influenzare la lettura del solo `repertica.sqlite`. Un controllo che apre SQLite deve rispettare `read-only/query-only` ed evitare creazioni/migrazioni/temp nell'origine: **verificarlo con test reali**, non presumerlo dal nome del flag.

**Riservatezza:** il backup v1 è **in chiaro**, anche quando il controllo non visualizza record personali. Niente report con nomi, email, importi, fotografie, digest o percorsi completi per default; hash e nomi dei file rimangono nella cartella originale e non vengono inviati. Le eventuali copie temporanee devono essere giustificate, in posizione scelta e rimosse senza false promesse di cancellazione sicura su SSD/NAS.

**Limite di fiducia essenziale:** SHA-256 confrontato col manifest rileva errori/modifiche **rispetto al manifest**; non è firma o prova di autenticità. Se un soggetto modifica insieme file e manifest, un confronto hash può risultare positivo. Un controllo completo non dimostra che l'opera sia autentica, che le somme siano riconciliate con la banca o che il software futuro possa ripristinare ogni versione.

**Compatibilità:** il formato v1 e gli archivi correnti devono rimanere leggibili; versione del manifest, versione SQLite, versione del verificatore e versione dell'app sono concetti distinti. Se arriva uno schema futuro non conosciuto, **i controlli sui byte possono comunque essere eseguiti**, ma i controlli di dominio vanno riportati come **non supportati**, mai come «database corrotto». Prove su Windows/macOS/Linux, percorsi con spazi e Unicode nella **cartella esterna**, dischi rimovibili/NAS con prestazioni e permessi variabili. Non promettere parità prima dei test.

**Impostazioni e multilingua:** nessuna nuova opzione obbligatoria o account. Un eventuale strumento riusa lingua di sistema con fallback inglese e, se ha UI, consente override manuale coerente con Repertica (IT/EN/ES/FR/DE secondo copertura). Chiavi JSON e codici macchina non tradotti; data, unità e testo umano localizzati. Nessun invio periodico automatico.

## 6. Alternative, dipendenze, rischi e complessità

| Alternativa | Beneficio | Limite e decisione |
| --- | --- | --- |
| **A — Pubblicare solo descrizione tecnica del v1 e procedura con strumenti SQLite/hash standard** | **Costo e manutenzione minimi**, indipendenza sostanziale senza nuovo programma | Più passaggi manuali, copertura dei vincoli di dominio limitata. **Prima scelta da provare** |
| B — Fixture sintetiche e suite di conformità pubblica, senza eseguibile distribuito | Rende possibile a terzi verificare i propri lettori e individuare incompatibilità | Richiede gestione versionata delle fixture; non è una GUI per l'artista |
| C — Verificatore CLI open source autonomo, solo lettura | Un comando locale, utilizzabile anche se Repertica non si avvia | Nuove build, distribuzione, audit dei parser e manutenzione a ogni evoluzione dello schema |
| D — App autonoma con schermata grafica | Più semplice per utenti non tecnici | Complessità maggiore, rischia di duplicare `inspect_backup_core` già presente |
| E — Non fare nulla | Nessun costo | Verifica completa legata alla disponibilità di una versione funzionante di Repertica |

**Dipendenze di ricerca:** stabilità dei backup v1, test esistenti del validatore nativo, definizione pubblica di casi validi/invalidi, documentazione SQLite e SHA-256, test dei supporti esterni/NAS reali, competenze per distinguere controlli fisici e di dominio. Nessuna libreria o packaging viene selezionato in questa scheda.

**Rischi:** scambiare un controllo parziale per successo del restore; divergenza fra due verificatori; falsi «corrotto» su schema futuro; data leak nel report; riproduzione incompleta di regole economiche; path traversal/TOCTOU; costi eccessivi di un secondo prodotto; sovraccarico su archivi grandi; imprecisioni della specifica rispetto al codice. **Complessità stimata:** bassa–media per una specifica e un protocollo di prova; media per fixture di conformità; medio-alta per un CLI completo; alta per app GUI e compatibilità retroattiva prolungata.

## 7. Criteri di accettazione e test ipotetici

1. **Fixture sintetiche, senza dati reali:** backup v1 valido, media storico non più referenziato ma nel manifest, allegato incasso valido, copertina in snapshot di certificato; testarlo prima con il validatore ufficiale. La specifica e il verificatore devono riportare esplicitamente quali regole hanno controllato.
2. Alterazioni negative: file mancante, hash/size diversi, media aggiunto fuori manifest, percorso `../`, duplicato, link simbolico o junction, file speciale, directory `media/` sostituita, sidecar SQLite e `repertica.sqlite` danneggiato → esiti non fuorvianti, nessun file cancellato o corretto.
3. Errori di dominio con SQLite ancora strutturalmente valido: FK mancante, ripartizione diversa dal 100%, incassi eccessivi, rate incoerenti, allegato con SHA presente nel DB diverso dai byte oppure formato/MIME incongruente → il profilo `full` li riconosce **solo dove applicabili**; il profilo `basic` dichiara di non averli controllati.
4. Manifest oltre la soglia, elenco file eccessivo, file enorme, interruzione a metà, cartella scollegata durante il controllo → limiti di risorse, stato `interrupted`/`incomplete`; **nessun falso verde**.
5. Schema superiore a quello noto, schema legacy supportato, file con Unicode/spazi nel percorso genitore, differenze filesystem fra Windows/macOS/Linux → distinguerne i limiti e mantenere i controlli sugli hash quando sicuro.
6. **Sola lettura effettiva:** comparazione byte prima/dopo sulla copia, nessuna creazione di `-wal`/`-shm`/`-journal`, niente scritture, niente rete. Il report viene prodotto **solo** su consenso, fuori dalla cartella del backup.
7. Prova indipendente: installazione Repertica assente, applicazione chiusa, sistema senza credenziali online; un lettore documentato deve riuscire a distinguere «hash corretti» da «ripristino provato» senza dipendere dal software Repertica.
8. Validazione con 2–3 professionisti o tecnici: la sola guida e gli strumenti standard sono sufficienti? Se sì, **nessun verificatore da sviluppare**. Nessun criterio richiede una nuova funzione in assenza di domanda.

## 8. Fasi ipotetiche e domande riservate al proprietario

- **Fase 0 — Ricerca/riscontro:** raccogliere 2–3 casi concreti (recupero su altro OS, software non disponibile, deposito istituzionale) senza usare backup personali; valutare guida/manuale prima del codice.
- **Fase 1 — Solo documentazione, previa valutazione:** bozza autonoma di specifica v1 e matrice di compatibilità contro il codice e i test esistenti; esempi inventati, licenza e modalità di citazione del formato, senza cambiare i backup.
- **Fase 2 — Eventuale conformità:** fixture e casi negativi, valutazione di lettori indipendenti e dei risultati parziali.
- **Fase 3 — Soltanto se giustificata:** scelta fra CLI autonomo e nessun software aggiuntivo; test nativi e verifica della sostenibilità manutentiva prima di una release.

**Questioni aperte (nessuna decisione presunta):**
1. Ci sono artisti, gallerie o archivi che richiedono già una verifica **senza installare Repertica**? Con quale frequenza?
2. Basta pubblicare la specifica e un manuale per SQLite/SHA-256, oppure serve davvero un eseguibile autonomo?
3. Chi garantirebbe nel tempo l'allineamento dei vincoli di dominio tra verificatore indipendente e release Repertica?
4. Quali report, se presenti, possono essere condivisi senza rivelare struttura o dati riservati?
5. Una futura copia cifrata richiederebbe un progetto separato; va esplicitamente esclusa finché non esiste un formato verificabile?

**Esito della ricognizione:** **solo proposta documentale**. Nessun codice, test, workflow, dipendenza, configurazione, issue operativa o merge autorizzato. La scelta può legittimamente essere **nessuno sviluppo**.
