# REPS-TECH-011 — Profili di archivi locali indipendenti e apertura sicura

- **ID stabile:** REPS-TECH-011
- **Data della proposta e ricognizione:** 2026-11-21
- **Stato:** **Proposta — da valutare**
- **Priorità proposta:** **P1–P2**, da verificare con utenti che gestiscono davvero archivi separati.
- **Ambito:** selezione e gestione sicura dell'**archivio attivo**, non sincronizzazione, multiutenza, fusione, ricerca federata o backup automatici.
- **Decisione:** nessuna; la scheda non approva né avvia sviluppo.

## Motivazione verificata e destinatari

Il [README](../../../README.md) e [MVP](../../MVP.md) descrivono un solo archivio attivo nella directory privata dell'app. In [lib.rs](../../../src-tauri/src/lib.rs), all'avvio Tauri apre la directory restituita da app_local_data_dir e registra una singola istanza di database; [db.rs](../../../src-tauri/src/db.rs) apre il file repertica.sqlite e la cartella media. Il ripristino attuale **sostituisce** i dati dell'archivio attivo e conserva una copia precedente: non è un comando per passare da un archivio all'altro. Il MVP dichiara esplicitamente che più archivi nella stessa installazione non sono ancora gestiti.

Un artista può voler distinguere catalogo personale e archivio amministrato per un'associazione; una galleria può lavorare su raccolte appartenenti a titolari diversi. Ripristinare di volta in volta backup completi solo per cambiare contesto sarebbe lento e rischierebbe di confondere dati privati, certificati e vendite.

**Destinatari ipotetici:** artisti con cataloghi distinti, piccoli studi, gallerie e archivisti che lavorano sul **medesimo computer** con archivi indipendenti. Non si assume che questa esigenza sia universale; per la maggioranza degli artisti il singolo archivio deve rimanere semplice.

## Confini e non duplicazione

- [TECH-004](REPS-TECH-004-ricerca-federata-archivi-locali.md) esplora la **consultazione read-only** di backup selezionati e risultati con provenienza; questa idea riguarda **aprire e modificare un archivio alla volta**, senza ricerca trasversale.
- [TECH-001](REPS-TECH-001-pacchetto-dati-aperto.md) propone un **formato aperto di interscambio e reimportazione**; qui non si cambiano formati o importano record.
- [TECH-009](REPS-TECH-009-catalogo-verifiche-conservazione-backup.md) cataloga **copie storiche** di sicurezza; qui le fonti sono **archivi di lavoro attivi**, non snapshot da navigare o ripristinare.
- [TECH-007](REPS-TECH-007-archivio-media-esterno-rilocalizzabile.md) studia la **collocazione degli originali fotografici**; qui l'unità logica è l'intero archivio (SQLite, media e configurazione specifica).
- [PR #14, gestione e UX](https://github.com/Cesare-ml/Repertica-Studio/pull/14) riguarda processi professionali **all'interno** di un archivio; non viene replicata la catalogazione di opere, clienti o attività.
- Non è collaborazione concorrente fra più utenti: nessun server, nessun accesso remoto, nessuna condivisione automatica di directory.

## Scenari e specifica funzionale proposta

1. **Primo utilizzo:** Repertica continua a creare un archivio locale predefinito, senza chiedere account e senza imporre una scelta. Solo chi ne ha bisogno apre una sezione «Archivi locali».
2. **Creare archivio indipendente:** «Nuovo archivio» propone un nome descrittivo e chiede una cartella attraverso il dialogo nativo. Anteprima di destinazione, contenuto previsto, spazio e verifica dei permessi; non riutilizza o sovrascrive file di terzi. La cartella può essere liberamente scelta, anche su USB o NAS già montato, con avviso sui rischi di disconnessione e concorrenza.
3. **Registrare archivio esistente:** «Collega cartella archivio» chiede una directory riconoscibile con repertica.sqlite e media, verifica integrità SQLite, schema/versione, percorsi e relazione tra media e record **senza migrazioni implicite** in fase di sola ispezione. Non tratta una directory di backup come archivio attivo: per trasformare un backup in archivio distinto occorrerebbe un'azione esplicita «Crea archivio da copia verificata», con clonazione in una **nuova** cartella e verifica integrale.
4. **Passare da un archivio all'altro:** intestazione persistente con **nome e percorso dell'archivio attivo**, conteggi orientativi e eventuale stato «unità esterna». Prima del cambio: blocco di nuove operazioni, completamento o annullamento di transazioni e scritture media, chiusura della connessione e verifica delle operazioni pendenti. Una prima soluzione **potrebbe richiedere riavvio controllato** dell'app invece di sostituire in corsa lo stato Tauri: è un trade-off progettuale, non una scelta già approvata.
5. **Archivio non raggiungibile:** se USB/NAS viene scollegato, non creare un nuovo database vuoto sul medesimo percorso e non proseguire come se tutto fosse disponibile. Mostrare «Archivio non disponibile», proporre «Riprova», «Individua la cartella» o «Apri un altro archivio», con riepilogo read-only dell'ultima configurazione solo se non espone dati riservati.
6. **Rimozione da elenco:** «Dimentica questo archivio» rimuove **solo il collegamento locale**, non il database e non le foto. Un'eventuale cancellazione fisica è esclusa dalla prima proposta; distinguere chiaramente «dimenticare» da «eliminare».
7. **Backup, esportazioni e attestati:** ogni operazione indica in modo evidente l'archivio **di origine** e la cartella di destinazione. Backup completo, esportazioni CSV e certificati riguardano **soltanto quello aperto**; nessuna copia o collisione di seriali tra archivi implica un'unione. Prima di un ripristino mostrare il nome del **destinatario** e avvertire che il suo contenuto sarà sostituito.

## Modello dati e possibili API — solo descrittivi

Il modello corrente usa un solo file repertica.sqlite e un media store sotto la directory privata. Un eventuale **registro delle aperture** risiederebbe nelle preferenze locali dell'installazione, non in un servizio esterno:

| Campo ipotetico | Significato e cautela |
| --- | --- |
| profile_id | Identificatore locale della voce nell'elenco; **non** identità certificata dell'archivio |
| display_name | Etichetta modificabile, non univoca |
| canonical_directory | Cartella canonica autorizzata, dato **locale e sensibile**, non esportato per default |
| last_opened_at | Istante dell'ultima apertura riuscita, non prova di modifica dei dati |
| last_seen_schema | Versione schema effettivamente ispezionata |
| availability | Stato osservato: disponibile, scollegato, incompatibile, in verifica |
| archive_identity | **Solo eventuale** UUID persistente scritto nel singolo archivio, dopo decisione su migrazioni e backup |

Non dedurre che due cartelle identiche per titolo o UUID delle opere siano lo stesso archivio. Un identificativo persistente dell'archivio **non esiste nel manifest backup v1** ([backup.rs](../../../src-tauri/src/backup.rs)); bisogna scegliere se introdurlo, come versionarlo e come trattare cloni e vecchi backup. Il registro è una **comodità ricostruibile**: il possesso della cartella, di backup e di esportazioni deve bastare a recuperare i dati dopo una reinstallazione.

Possibili contratti ad alto livello, **non API definite**:

- inspectArchiveDirectory(directory) → compatibilità, integrità, dimensioni, rischi e informazioni di sola lettura.
- registerArchive(directory, label) → profilo locale, senza modificare il contenuto dell'archivio durante la sola registrazione.
- createArchive(directory, label) → nuova struttura dopo preflight e conferma.
- switchActiveArchive(profile_id) → piano di chiusura/apertura o riavvio controllato, esito esplicito e recupero se fallisce.
- forgetArchive(profile_id) → elimina il riferimento locale, **mai** i byte sorgente.
- cloneVerifiedBackupToArchive(backup_directory, new_directory) → operazione futura separata, non import/merge.

**Impostazioni di studio** come coefficiente artista, quote e lingua sono oggi collegate al database tramite studio_settings ([db.rs](../../../src-tauri/src/db.rs)); occorre distinguere preferenze **di installazione** (elenco profili, ultimo archivio, tema/lingua preferita) da valori **per archivio** (prezzi, quote, eventuale autore/emittente), senza trasferimenti silenziosi fra titolari. La lingua di sistema resta il valore iniziale con fallback inglese; un override locale segue la politica già definita nel progetto.

## Offline, sicurezza, privacy, lingue e accessibilità

- Nessun cloud, account, login, abbonamento, discovery di cartelle sulla rete, server locale obbligatorio o telemetria. I NAS sono **filesystem già montati** dal sistema operativo; Repertica non gestisce credenziali SMB né sincronizza dati.
- L'utente sceglie **liberamente** cartelle locali o esterne. Rifiutare file speciali, symlink pericolosi, percorsi relativi con traversal e duplicazioni di profili che puntano allo stesso archivio; le decisioni sui symlink normali del sistema operativo richiedono test Windows/macOS/Linux.
- Non aprire **due connessioni applicative in scrittura contemporanee** allo stesso archivio: valutare lock di processo cross-platform, comportamento alla chiusura anomala e limiti reali su NAS. I lock non garantiscono collaborazione concorrente sicura su tutti i filesystem.
- Ispezionare archivi potenzialmente non fidati in modalità read-only, con limiti di dimensione e versioni supportate; niente migrazione, avvio di script, estensioni SQLite o modifiche prima di conferma e backup. Una versione schema futura deve produrre incompatibilità esplicita, mai downgrade automatico.
- Evitare che dati del precedente archivio rimangano in componenti Angular, notifiche, cache immagini, cronologia, log o clipboard quando si cambia profilo. Se l'archivio contiene informazioni riservate di clienti diversi, mostrare il contesto attivo in ogni schermata sensibile.
- Messaggi in lingua di sistema (IT/EN/ES/FR/DE se disponibili), fallback inglese e possibile override locale. Nome proprio e contenuti utente non si traducono; date, importi e numeri si visualizzano secondo locale. Cambio archivio e schermate di errore accessibili da tastiera, con focus conservato, lettore di schermo e stati non indicati soltanto dal colore.
- **Esportabilità:** ogni archivio deve restare un insieme locale autonomo recuperabile senza il registro profili; backup nella cartella desiderata e sei CSV integrali esistenti continuano a valere per l'archivio selezionato. Un eventuale pacchetto semantico aperto resta materia di TECH-001.

## Dipendenze, alternative, rischi e complessità

**Dipendenze progettuali:** gestione della connessione singola in Tauri e dello stato Angular; database/migrazioni SQLite v1–v5; backup e restore con staging e rollback; media store e snapshot certificati; permessi e dialoghi di cartella multipiattaforma; test di apertura/chiusura su WAL. Non richiede un servizio aggiuntivo, API cloud o libreria proprietaria.

**Alternative:** (A) continuare con **un unico archivio** e backup multipli: zero nuova interfaccia ma passaggio di contesto rischioso e distruttivo; (B) creare profili separati del sistema operativo: isolamento forte, ma scomodo; (C) partire con **solo cambio al riavvio**, evitando hot-swap Tauri e cache: esperienza meno fluida ma probabilmente più facile da verificare; (D) federazione read-only TECH-004: adatta alla consultazione, non al lavoro di modifica.

**Rischi:** confusione di titolarità, apertura accidentale di uno snapshot di backup in scrittura, perdita di operazioni pendenti, due istanze che scrivono sullo stesso SQLite, DB vuoto creato su USB scollegata, cambio di percorso con copia parziale, immagini storiche mancanti, divergenza tra path canonico e alias, segreti nei percorsi memorizzati. Le mitigazioni richiedono isolamento esplicito, validazione, lock, conferme, log senza dati privati e rollback di cambio profilo.

**Complessità indicativa:** **medio-alta** per un selettore sicuro con riavvio; **alta** per hot-swap, archivi su NAS, clonazione da backup e identità robusta. Nessuna priorità è approvata: l'interesse va confermato e il collaudo backup già aperto in [MVP](../../MVP.md) rimane preliminare.

## Criteri di accettazione e test ipotetici

1. Su due archivi di prova A e B, con **UUID, titoli e seriali potenzialmente uguali ma contenuti diversi**, ogni schermata, esportazione, certificato e backup usa **soltanto** l'archivio attivo. Nessun trasferimento involontario o fusione.
2. Passaggio A→B durante modifica di scheda, fotografia, backup e stampa: o attende/completa l'azione, o annulla **esplicitamente** il cambio, senza perdere dati. Un riavvio e un crash simulato non lasciano metà cambio.
3. Archivio su USB scollegata, NAS in sola lettura, cartella rinominata o piena, schema SQLite futuro, DB corrotto, file symlink ostile: errore localizzato e recuperabile, **senza creare un archivio vuoto in quel luogo** o sovrascrivere byte.
4. Registrare due volte la stessa cartella canonica non crea due profili operativi distinti; copie clonate sono identificate come **copie** quando riconoscibili, senza inferire equivalenza assoluta.
5. «Dimentica» non cancella dati o fotografie; reinstallando l'app si può riaprire la cartella senza il vecchio registro locale. Backup e ripristino di ciascun archivio mantengono UUID, foto, incassi, vendite e certificati.
6. Nessuna porta TCP, telemetria o account; prova offline completa su Windows/macOS/Linux, su filesystem locale e almeno un supporto rimovibile, con navigazione da tastiera e screen reader.
7. Test di contemporaneità con due istanze della stessa app e tentativi di accesso in scrittura: eventuale conflitto viene **rilevato e bloccato**, senza corruzione. Un filesystem non affidabile deve essere rifiutabile oppure esplicitamente dichiarato non supportato.

## Requisiti accertati, ipotesi, questioni al proprietario

**Accertato nel repository al 21 novembre 2026:** singolo SQLite attivo caricato da Tauri all'avvio; media locale; backup con scelta cartella, manifest e verifica; ripristino sostitutivo; impostazioni economiche nel DB; supporto multilingue; assenza di selettore di archivi attivi; il backup v1 non documenta un ID stabile di archivio.

**Ipotesi:** sufficiente domanda per gestire più cataloghi; elenco locale di profili, cambio al riavvio, possibilità di collocare un intero archivio su USB/NAS, eventuale UUID d'archivio, clonazione assistita. Non sono requisiti approvati.

**Domande aperte per revisione personale:**
- Gli utilizzatori lavorano davvero su **due o più archivi modificabili** sullo stesso PC, oppure basta il catalogo unico con viste salvate o federazione read-only?
- L'archivio secondario deve poter vivere **fuori** dalla directory privata Tauri? Su NAS in scrittura, oppure soltanto su dischi locali affidabili?
- È preferibile il cambio sicuro **con riavvio** nella prima fase? Quale messaggio e workflow sono accettabili?
- Le impostazioni di lingua e presentazione devono essere **globali**, mentre coefficiente, quote e seriale certificati restano **per archivio**?
- Occorre un vero **ID stabile dell'archivio** da introdurre in DB e backup futuri, oppure iniziare senza identità persistente riconosciuta?
- Ha senso l'azione «Crea archivio da backup» o basta nuova cartella vuota e collegamento di un archivio esistente? Chi decide eventuali collisioni tra cloni?

## Fasi eventuali — non autorizzate

1. **Validazione del bisogno:** interviste brevi a professionisti con archivi distinti; mappa dei processi che oggi richiedono backup/restore per cambiare archivio.
2. **Specifiche e prove isolate:** contratto del registro profili, matrice sicurezza per OS/filesystem, separazione preferenze/archivio, test di collisione, disconnessione e crash.
3. **MVP ipotetico, solo dopo approvazione:** creare/registrare/aprire un archivio alla volta con riavvio controllato; indicatore sempre visibile; nessuna apertura diretta di backup in scrittura.
4. **Estensioni eventuali:** clonazione da backup verificato, archivi esterni, riconnessione assistita, identità persistente e solo se davvero necessaria migliore UX senza riavvio.

> Documento esclusivamente ideativo: nessun codice, issue, workflow, dipendenza o implementazione viene autorizzato da questa scheda.
