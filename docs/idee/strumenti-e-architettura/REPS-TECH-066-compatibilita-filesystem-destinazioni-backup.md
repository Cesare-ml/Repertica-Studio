# REPS-TECH-066 — Compatibilità e affidabilità delle destinazioni di backup su USB e NAS

- **ID stabile:** REPS-TECH-066
- **Data:** 2026-11-23
- **Stato:** **Proposta — da valutare**
- **Filone:** Strumenti e architettura
- **Priorità proposta:** **P2 solo se emergono limiti reali su supporti diffusi o archivi grandi**; altrimenti P3, test/guida senza nuova funzione
- **Natura:** documento di ricerca e specifica ipotetica. Nessuna implementazione, approvazione, incarico o decisione del proprietario.

## 1. Motivazione, persone interessate e riscontri

**Problema da verificare.** Repertica consente di salvare un backup completo in qualunque cartella accessibile, compresi dischi USB e NAS già montati. La dimensione complessiva di un archivio e le caratteristiche del filesystem di destinazione non coincidono necessariamente con ciò che quel filesystem può scrivere e pubblicare in modo affidabile. Un supporto può avere spazio complessivo sufficiente ma non supportare un singolo file di database molto grande; alcune unità di rete possono avere comportamenti differenti per rinomina, sincronizzazione e perdita temporanea della connessione. Un hash valido durante la verifica non equivale alla garanzia di durabilità dopo l'espulsione prematura o un'interruzione di corrente.

**Fatti verificati su `main`, 23/11/2026:**
- [README](../../../README.md) e [MVP](../../MVP.md) dichiarano backup e ripristino su cartelle scelte liberamente, USB e NAS montati, manifest SHA-256 e snapshot SQLite con `VACUUM INTO`; i collaudi su dispositivi esterni e NAS reali sono ancora elencati fra i lavori da svolgere.
- In [`src-tauri/src/backup.rs`](../../../src-tauri/src/backup.rs), `create_backup_core` crea una cartella parziale **dentro la destinazione**, acquisisce lo snapshot `repertica.sqlite`, copia i media, verifica il manifest e tenta `fs::rename(partial, target)` per pubblicare il backup, seguito da una nuova ispezione. Gli errori sono già restituiti, i file sono sincronizzati durante la copia e i residui parziali sono oggetto di pulizia. **Non si propone di sostituire questo sistema già presente.**
- Nel perimetro letto non emerge un controllo preventivo **specifico** dei limiti di dimensione per singolo file o delle capacità del filesystem di destinazione. Questo non prova un malfunzionamento: un errore I/O è già possibile e gestito; il valore aggiunto sarebbe un messaggio preventivo corretto e una matrice di compatibilità fondata su prove.

**Destinatari:** artisti con grandi archivi fotografici, gallerie con backup programmati su dischi esterni, archivi professionali che usano un NAS montato dal sistema operativo, volontari che collaudano Windows/macOS/Linux. La funzione **non** serve a chi usa piccoli archivi su un disco locale affidabile se gli errori esistenti sono comprensibili.

**Scenari concreti da testare, non incidenti osservati:**
1. Un database SQLite cresce fino ad avvicinarsi al massimo di un singolo file su un volume FAT32 (limite tipico inferiore a 4 GiB): la disponibilità di 100 GiB di spazio libero non rende possibile quel file. Distinguere **dimensione del singolo file** da dimensione totale del backup.
2. Un NAS sembra accessibile ma la connessione si interrompe fra copia e pubblicazione. L'utente deve distinguere `backup non pubblicato`, `backup pubblicato ma verifica finale fallita`, `backup verificato nella sessione` e `durabilità futura non dimostrabile`.
3. Una chiavetta viene rimossa durante la copia oppure un volume di rete ha permessi di lettura ma non di scrittura: la diagnosi non deve cancellare altre cartelle o suggerire falsamente che i dati siano salvati.

## 2. Confini anti-doppione

- [TECH-022](REPS-TECH-022-preflight-spazio-operazioni-locali.md) studia **spazio disponibile e preflight della capacità**, non il **limite per singolo file** e l'affidabilità della pubblicazione su filesystem differenti.
- [TECH-060](REPS-TECH-060-identita-destinazione-backup-programmati.md) confronta **l'identità del volume** durante backup automatici quando un percorso può riferirsi a un supporto diverso; qui interessa il **comportamento/capacità** di qualunque destinazione, anche manuale, indipendentemente dall'identità.
- [TECH-009](REPS-TECH-009-catalogo-verifiche-conservazione-backup.md), [TECH-052](REPS-TECH-052-verifica-indipendente-backup-formato-v1.md) e [TECH-050](REPS-TECH-050-confronto-backup-senza-ripristino.md) trattano conservazione/verifica/confronto di copie **esistenti**. Qui si esamina la compatibilità della *scrittura/pubblicazione* sulla destinazione.
- [TECH-062](REPS-TECH-062-progresso-annullamento-sicuro-operazioni-locali.md) riguarda stato e annullamento delle operazioni; questa proposta non introduce un nuovo motore di avanzamento.
- [TECH-064](REPS-TECH-064-coordinamento-processi-archivio-attivo.md) riguarda accesso di processi concorrenti **all'archivio attivo**, non le garanzie del filesystem di backup. Nessuna funzione gestionale dell'altra [PR #14](https://github.com/Cesare-ml/Repertica-Studio/pull/14) viene duplicata o modificata.

**Esclusioni:** non scegliere al posto dell'utente un altro disco; non imporre filesystem o servizi cloud; non modificare il formato `manifest.json`; non reinterpretare il backup come una sincronizzazione, un archivio incrementale o una copia sicuramente durevole.

## 3. Specifica funzionale e flussi UX — ipotesi

**Prima opzione a basso costo, preferibile:** documentare una *matrice di collaudo* con esempi su filesystem e supporti effettivamente provati, e migliorare i soli messaggi di errore già emessi. Una nuova schermata è giustificata soltanto se le prove dimostrano errori evitabili e difficili da interpretare.

**Eventuale diagnostica prima del backup:**
1. In **Impostazioni → Backup**, l'utente seleziona liberamente la cartella esistente, anche USB/NAS; un controllo read-only verifica che il percorso sia raggiungibile e che non sia nell'archivio attivo, senza installare provider o chiedere credenziali.
2. Stima in modo prudente la dimensione del **file SQLite risultante**, che può differire da quella del database live dopo `VACUUM INTO`; quando la dimensione non è prevedibile, mostra `Non determinabile con affidabilità` invece di una falsa garanzia.
3. Rileva **solo se affidabile sull'OS** il tipo filesystem e un eventuale limite noto per singolo file. Esempio: per un volume identificato come FAT32, segnala che un file vicino o oltre il limite non sarà scrivibile; **non** vieta il backup per la sola dimensione totale superiore al limite per file.
4. Se i limiti del NAS/remoto non sono conoscibili, evita etichette `compatibile garantito`: avvisa che il risultato va verificato con una copia di prova. Per verificare scrittura/rinomina su quel volume potrebbe offrire un **test facoltativo esplicitamente autorizzato**, in sottocartella nuova con nome univoco, senza toccare file altrui; dimensione e costo del test vanno dichiarati.
5. Il normale backup resta disponibile e continua a utilizzare il proprio manifest e la verifica esistenti. Un errore nella pubblicazione riporta **fase** e **destinazione** (con percorso visibile solo localmente), senza dichiarare completato il backup.
6. Dopo un esito positivo viene mostrato `Backup verificato in questa sessione`, accompagnato dall'avvertenza che hash e lettura immediata non sostituiscono espulsione sicura, più copie e prova di ripristino.

**Accessibilità:** messaggi leggibili con screen reader, non basati sul solo colore, dettaglio tecnico espandibile, focus conservato e prova da tastiera; il controllo non deve rendere più difficile il normale salvataggio manuale.

## 4. Strutture dati / API illustrative (non esistenti)

Nessuna nuova tabella SQLite sarebbe necessariamente richiesta. Eventuali risultati transitori, **solo descrittivi**:

```text
DestinationCapabilityReport {
  canonical_destination: LocalPathDisplayedOnly,
  fs_family: Option<FilesystemName>,
  fs_detection_confidence: known | inferred | unknown,
  max_single_file_bytes: Option<u64>,
  estimated_largest_file_bytes: Option<u64>,
  estimated_total_bytes: Option<u64>,
  available_bytes: Option<u64>,
  checks: [ { code, severity: info | warning | block, evidence } ],
  checked_at_local: LocalTimestamp
}
BackupPublishOutcome {
  phase: copying | validating_partial | publishing | validating_published | completed,
  result: success | failed | uncertain,
  published_directory: Option<LocalPath>,
  verification_in_session: Option<bool>
}
```

Le API ipotetiche `inspectBackupDestination(path, estimatedContent) -> DestinationCapabilityReport` e `runOptionalDestinationProbe(path, explicitConsent) -> ProbeReport` sarebbero separate dai comandi esistenti. Se una piattaforma non espone le informazioni in modo affidabile, restituire `unknown`. **Non** aggiungere flag o campi al manifest di backup per questi controlli ambientali; il manifest resta formato portabile dei dati, non un profilo del computer.

## 5. Offline, privacy, impostazioni, multilingua e sicurezza

- **Offline totale:** nessuna ricerca online del NAS, nessuna API remota, account, telemetria, invio automatico o servizio esterno. Solo cartelle/volumi già montati dall'OS e autorizzati dall'utente.
- **Privacy:** percorso del NAS, nomi delle directory, struttura dei dischi e dettagli d'errore sono dati locali potenzialmente sensibili; niente log o rapporti condivisi automaticamente. Per eventuale export del report, anonimizzazione/anteprima esplicita e consenso.
- **Impostazioni:** controllo non distruttivo opzionale o contestuale; possibilità di disattivare gli avvisi informativi; gli errori bloccanti reali restano visibili. Non memorizzare identificatori di volume, test scritti o password se non strettamente necessario; nessun dispositivo diventa `autorizzato per sempre`.
- **Lingue:** UI dalla lingua del sistema con fallback inglese e scelta manuale già prevista; messaggi tramite codici stabili traducibili nelle lingue supportate, dati del filesystem non tradotti, unità file spiegate in byte e GiB senza confusione fra GB e GiB.
- **Sicurezza e persistenza:** nessuna prova di scrittura se l'utente non l'ha chiesta; se eseguita, usare una nuova sottocartella esclusiva app-owned, limiti di byte/tempo e nessuna scansione della rete. Non inseguire symlink o rinominare cartelle di terzi. In caso di timeout non assicurare che i dati di prova siano stati cancellati dal NAS: mostrare percorso di eventuali residui.
- **Durabilità:** `fsync` dei file, rinomina e immediata verifica degli hash migliorano la fiducia ma non dimostrano la persistenza dopo blackout, cache NAS/USB o distacco. Non assegnare un punteggio di affidabilità inventato ai filesystem.

## 6. Dipendenze, alternative, rischi e complessità

**Dipendenze progettuali:** `backup.rs` (snapshot, copia, verifica e pubblicazione), dialogo nativo di selezione cartella e capacità di leggere metadata locali sul sistema operativo. L'eventuale rilevazione del filesystem richiederebbe API native specifiche di Windows/macOS/Linux; **nessuna dipendenza è richiesta o approvata da questa scheda**. La verifica operativa è condizionata a test reali con permessi ridotti e filesystem remoti.

**Alternative:** (A) sola matrice/documentazione e migliori messaggi d'errore, preferita se i fallimenti sono rari; (B) diagnosi preventiva read-only su file/volume, senza test di scrittura; (C) test temporaneo di scrittura/rinomina facoltativo, soltanto se la matrice mostra che aggiunge valore; (D) richiedere esclusivamente dischi locali: più semplice ma contraddice il principio di libertà della cartella.

**Rischi:** falso `OK` su NAS instabile; confusione fra singolo file e spazio totale; supporti identificati erroneamente; probe che lascia residui o consuma spazio; comportamento diverso fra driver/filesystem/OS; messaggi talmente allarmanti da indurre l'utente a non fare backup. La mitigazione è uno schema di risposte `conosciuto/sconosciuto`, diagnostica non distruttiva e test di guasto documentati. **Complessità:** bassa per guida e matrice, media per messaggi/diagnostica read-only, medio-alta per probe e portabilità nativa.

## 7. Test e criteri di accettazione futuri (non eseguiti)

1. **Baseline senza nuova funzione:** Windows, macOS e Linux; disco locale, USB exFAT, eventuale FAT32 e almeno un NAS SMB montato. Rilevare esiti di creazione, verifica e ripristino **su copia di prova**; mai utilizzare l'unico archivio reale.
2. **Dimensione:** archivio con file SQLite molto grande su volume con limite per file noto, controprova su volume senza quel limite; spazio totale sufficiente ma file non scrivibile produce avviso o errore corretto **senza creare un falso backup valido**. Un backup complessivo oltre 4 GiB fatto di file più piccoli non deve essere rifiutato per questo motivo.
3. **Rinomina/publicazione:** connessione NAS interrotta durante copia, prima/dopo rinomina e subito dopo verifica; l'esito non deve indicare successo se non confermato. Cartelle altrui e backup precedenti intatti. Un file già pubblicato ma non verificabile non deve diventare `verified`.
4. **Dispositivo rimosso / permessi revocati / disco pieno:** report preciso, nessuna sovrascrittura e nessuna perdita dell'archivio attivo; i residui incerti vengono segnalati, non rimossi con ricerca ampia.
5. **Privacy:** nessuna connessione Internet generata dal controllo, nessuna credenziale memorizzata, diagnostica locale con report esportabile soltanto a scelta.
6. **Accessibilità/lingue:** italiano/inglese e fallback inglese, messaggi non ambigui, tastiera e screen reader, volume non identificabile indicato come `sconosciuto`.
7. **Regressione:** nessuna modifica al formato di backup v1, alla possibilità di selezionare liberamente cartelle, al ripristino verificato o alle politiche di conservazione; nessun test invasivo silenzioso su USB/NAS.

## 8. Fasi ipotetiche e domande riservate al proprietario

1. **Ricerca senza nuovo codice:** campioni sintetici di archivi grandi e test non distruttivi su filesystem reali; catalogare errori e capacità effettive, distinguendo database vs media. Si può concludere che **non serve una nuova funzione**.
2. **Solo se utile:** guida su limiti per singolo file, messaggi della fase di fallimento e visualizzazione prudente dell'esito.
3. **Solo se richiesto:** diagnostica read-only delle capacità della destinazione, sempre con valori sconosciuti espliciti.
4. **Eventuale sperimentazione separata:** test volontario di scrittura/rinomina e prove approfondite di resilienza NAS; nessuna promessa di durabilità assoluta.

**Questioni aperte:** esistono utenti che usano FAT32/vecchi USB? Qual è la dimensione massima osservata di `repertica.sqlite` e dei backup completi? Quali filesystem e NAS sono prioritari fra Windows/macOS/Linux? È accettabile un avviso prudente anziché un controllo attivo? Il test di destinazione deve essere sempre opt-in? Quali messaggi danno valore reale senza trasmettere falsa sicurezza?

**Certezze:** codice e limiti di collaudo citati da `main`; verifica SHA-256, cartella parziale e rinomina già presenti. **Ipotesi:** bisogno di diagnosi preventiva e tutte le strutture UX/API illustrate. **Decisioni:** integralmente riservate al proprietario; stato **Proposta — da valutare**. Non è stato eseguito alcun test nativo né modificato codice.
