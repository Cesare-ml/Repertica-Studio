# REPS-TECH-042 — Compatibilità degli aggiornamenti e recupero offline della versione applicativa

- **ID stabile:** REPS-TECH-042
- **Data ricognizione:** 2026-11-22
- **Stato:** **Proposta — da valutare**
- **Filone:** Strumenti e architettura; analisi esclusivamente documentale
- **Priorità proposta (non approvata):** **P1–P2** per criteri minimi di distribuzione, diagnosi e collaudo di compatibilità; **P2–P3** per un eventuale percorso guidato di recupero, soltanto se il bisogno è confermato
- **Decisione del proprietario:** non espressa. Nessuna approvazione, assegnazione, implementazione o modifica di software è autorizzata.

## 1. Motivazione, destinatari e limiti della verifica

Una galleria può installare una nuova versione di Repertica, scoprire che non si avvia correttamente e voler ritornare temporaneamente alla versione precedente. Questa scelta è **diversa dal recuperare i dati**: un installatore precedente potrebbe ancora essere disponibile, ma non poter aprire uno SQLite già migrato. Un archivio aggiornato, inoltre, può contenere vendite e fotografie aggiunte *dopo* l'installazione; sostituirlo automaticamente con un vecchio backup cancellerebbe lavoro successivo.

**Riscontri statici effettuati sul branch main il 22/11/2026:**
- In [updater.rs](../../../src-tauri/src/updater.rs), l'aggiornamento è scaricato dal feed GitHub fissato nel programma, il pacchetto deve corrispondere alla release del progetto e avere firma verificata dal plugin. La nuova versione viene installata solo su comando. Il codice esegue un backup preventivo **solo se il backup automatico è attivo e dispone di destinazione**; in assenza di tale configurazione prosegue senza crearne uno. L'utente può anche confermare l'installazione dopo un fallimento del backup.
- In [db.rs](../../../src-tauri/src/db.rs), la versione schema corrente è **18**. L'apertura rifiuta un archivio con schema superiore a quello supportato e applica le migrazioni previste per schemi più vecchi. Una versione applicativa precedente non è quindi, di per sé, una strada per riaprire lo stesso database migrato.
- In [lib.rs](../../../src-tauri/src/lib.rs), l'apertura dell'archivio è nella fase di setup dell'app; non risulta, nei file controllati, un percorso indipendente dall'apertura di SQLite per mostrare una guida di recupero quando il setup fallisce.
- In [RELEASING.md](../../RELEASING.md), gli artefatti firmati e le istruzioni di pubblicazione sono documentati, ma non è descritto un **contratto di recupero di una versione applicativa precedente**, distinto dal ripristino dell'archivio. Il ciclo reale scarica → installa → riavvia è dichiarato ancora da collaudare con una release firmata.
- [MVP.md](../../MVP.md) avverte di non utilizzare ancora questa versione come unica copia di archivi di produzione. Non è stato riprodotto un guasto di aggiornamento o una perdita dati: questa è un'analisi preventiva di continuità operativa.

**Destinatari:** artisti indipendenti senza assistenza IT; gallerie e archivi con cronologie di vendite, certificati e collezionisti; professionisti che lavorano offline o su dispositivi rimovibili; persone che aggiornano il programma soltanto a intervalli lunghi.

**Problema concreto da evitare:** confondere «ripristinare il programma» con «ripristinare i dati». Una firma di aggiornamento autentica non garantisce la compatibilità retroattiva dello schema, né la riuscita dell'avvio sul dispositivo.

## 2. Confini e anti-doppioni

- [TECH-012 — Checkpoint pre-migrazione](REPS-TECH-012-migrazioni-archivio-preflight-copia-sicurezza.md) studia la salvaguardia **dell'archivio prima di una migrazione SQLite**. REPS-TECH-042 studia **versioni del software, disponibilità degli installatori, matrice compatibilità, esperienza di recupero e supporto offline**. Il checkpoint, se necessario, sarebbe il meccanismo di TECH-012, non una seconda implementazione dei backup.
- [TECH-037 — Ripristino interrotto](REPS-TECH-037-recupero-ripristino-interrotto-avvio-protetto.md) concerne la scelta sicura fra cartelle attive, rollback e staging **dopo un restore interrotto**. Questa scheda non modifica il flusso del restore e non promuove automaticamente cartelle.
- [TECH-009](REPS-TECH-009-catalogo-verifiche-conservazione-backup.md) inventaria e verifica copie complete già esistenti; [TECH-022](REPS-TECH-022-preflight-spazio-operazioni-locali.md) riguarda la capacità dei volumi. Eventuali informazioni di queste funzioni andrebbero riutilizzate, non duplicate.
- [TECH-014](REPS-TECH-014-bridge-mcp-locale-lettura-permessi.md) riguarda MCP; un processo MCP avviato con binario più vecchio deve semplicemente continuare a rifiutare uno schema incompatibile, senza scrivere o riparare dati.
- Confrontato anche l'[indice Gestione e UX, PR #14](https://github.com/Cesare-ml/Repertica-Studio/blob/docs/idee-gestione-ux-20261121/docs/idee/gestione-e-ux/README.md), con 46 proposte: nessuna descrive la recuperabilità **del pacchetto applicativo** dopo aggiornamento. La scheda non introduce un modulo gestionale, una funzionalità di contabilità o un pannello di backup parallelo.
- **Fuori ambito:** downgrade automatico dello schema, fusione automatica degli archivi, installazione di release senza firma o da sorgenti arbitrarie, endpoint proprietari, sincronizzazione, aggiornamenti silenziosi, disattivazione dei controlli di sicurezza di Windows/macOS, distribuzione di chiavi private.

## 3. Requisiti certi, ipotesi, alternative

**Vincoli già stabiliti:** app Tauri gratuita e open source, archivio SQLite e media locali, funzionamento offline per uso ordinario, nessun account/abbonamento, backup in cartelle scelte liberamente, esportazione completa dei dati, consenso all'installazione, lingua di sistema e fallback inglese. L'eventuale recupero non può ridurre questi diritti.

**Ipotesi da validare:** gli utenti potrebbero aver bisogno di mantenere, o reperire offline, un installer di una release precedente; una lista compatibilità versione-app/versione-schema potrebbe prevenire errori; l'impossibilità di avviare l'app dopo un aggiornamento potrebbe richiedere un percorso di assistenza leggibile **senza accesso al DB**. Non è stato dimostrato quanti casi reali giustifichino un'interfaccia dedicata.

**Alternativa minima, da preferire prima di creare un modulo:** una procedura documentata nella release e nel sito, con link agli installer ufficiali delle versioni precedenti, avviso «il programma vecchio non apre necessariamente l'archivio nuovo», istruzioni per copiare l'archivio attuale *prima* di ogni recupero e prova di avvio con un archivio di test. Questa alternativa richiede soprattutto disciplina di release e QA, non nuove tabelle.

## 4. Scenari e flussi UX ipotetici

1. **Aggiornamento senza migrazione.** L'utente vede la versione installata, la candidata e una descrizione affidabile dell'eventuale cambio di formato dati. Può scegliere «Aggiorna» o «Non ora». La mancanza di dati di compatibilità non deve trasformarsi in una promessa di rollback.
2. **Aggiornamento con possibile migrazione.** Prima dell'installazione, avviso comprensibile: «Il nuovo programma potrebbe aggiornare l'archivio. Una versione precedente potrebbe non riuscire ad aprirlo». Nessun linguaggio che presenti il backup automatico come attivo se non lo è. Eventuale checkpoint **delegato a TECH-012** e verificato prima che cambi il DB; se indisponibile, lasciare all'utente una decisione esplicita e prudente.
3. **Nuova versione non si avvia, archivio invariato.** La documentazione offline e/o un piccolo percorso di assistenza indipendente dal DB mostra numero versione, sistema operativo e percorsi d'installazione senza condividere dati. L'utente può reinstallare una versione ufficiale precedente, *se* il suo archivio è ancora compatibile; nessun dato cancellato.
4. **Nuova versione si è avviata e ha migrato SQLite.** Evitare il pulsante ambiguo «Torna indietro». Presentare due risorse diverse: (A) **archivio corrente**, da preservare integralmente e non modificare; (B) eventuale **checkpoint pre-migrazione**, da ispezionare e aprire solo in una copia o mediante recupero consapevole. Una versione vecchia non viene puntata automaticamente al DB più nuovo.
5. **Sono stati creati dati dopo l'aggiornamento.** Prima di qualunque recupero storico, avviso che certificati, pagamenti, media o altre modifiche potrebbero non essere nel checkpoint. Nessun rollback dei dati o eliminazione dei file recenti senza ispezione e consenso esplicito. Possibile esportazione degli elementi recenti per futura ricostruzione, ma non promettere una fusione automatica.
6. **Offline o release non più disponibile.** Repertica resta utilizzabile senza consultare GitHub. Un eventuale kit locale di recupero richiede scelta dell'utente e verifica di origine, firma, piattaforma e spazio; se il kit non esiste, la guida deve dirlo, non simulare un recupero possibile.
7. **Errore di firma/installazione.** Mostrare stato e azione di ripetizione, senza scorciatoie che disattivino la verifica o reinstallino da un URL alternativo non verificato. L'app precedente e i dati devono rimanere intatti per quanto controllabile dal prodotto; l'esito effettivo dipende anche dall'installer del sistema.

**Messaggi accessibili:** «Versione del programma», «Versione dell'archivio», «Dati non modificati», «Archivio convertito: non apribile con questa versione», «Conserva una copia prima di procedere». Mai chiamare «recupero completo» una procedura non verificata.

## 5. Dati e API solo descrittive — non approvate

Non si propone una tabella obbligatoria né una telemetria. Possibili oggetti *informativi*, derivati da file locali o metadati firmati di release:

| Oggetto concettuale | Campi descrittivi | Note |
| --- | --- | --- |
| AppBuildIdentity | version, target_os, architecture, installer_type, signature_identity? | La firma applicativa e quella updater possono avere significati diversi |
| ArchiveCompatibility | schema_version, app_supported_min?, app_supported_max?, media_format_version?, read_only_possible | Se non è verificato un intervallo, mostrare «non noto» |
| RecoveryInventory | local_installers[], available_checkpoints[], current_archive_schema?, observed_at | Inventario locale consultivo; non scandire tutto il disco senza permesso |
| RecoveryChoice | action, source, destination, explicit_consent, expected_data_loss_warning | Mai migrare all'indietro o cambiare l'archivio attivo implicitamente |
| RecoveryOutcome | verified, app_version, archive_version, diagnostics, preserved_paths[] | Esito verificato solo dopo collaudo, non al semplice download |

Possibili contratti ad alto livello: *inspectInstalledVersion()*; *inspectArchiveCompatibilityReadOnly(path)*; *listUserSelectedRecoverySources()*; *explainRecoveryPlan(app, archive, checkpoint)*; *verifyRecoveredCopyWithoutMutatingOriginal()*. Non sono endpoint HTTP, comandi operativi approvati o specifiche Tauri esecutive.

**Contratto di compatibilità:** separare (1) versione semanticamente pubblicata dell'app, (2) versione dello schema SQLite, (3) formato manifest dei backup e (4) compatibilità dei media. Una release può aggiornare il programma senza migrare lo schema; un checkpoint SQLite compatibile non garantisce che tutti i media, i documenti e le funzionalità funzionino con la vecchia UI. Non far dedurre la compatibilità dal solo confronto dei numeri di versione o da un singolo hash.

## 6. Offline, privacy, sicurezza, impostazioni e multilingua

- Il recupero è **offline-first**: istruzioni locali, selezione manuale di cartelle/file, nessun account, contatto con assistenza o cloud obbligatorio. GitHub serve solo per ottenere release ufficiali quando una connessione è disponibile; gli installer salvati dall'utente rimangono sotto il suo controllo.
- Nessun invio di percorsi, nomi di collezionisti, certificati, importi, fotografie o log senza una scelta esplicita. Un report di compatibilità, se previsto, usa per default soli numeri di schema/versione e codici d'errore; i percorsi restano locali o copiabili a richiesta.
- Non attivare automaticamente eseguibili vecchi e non applicare downgrade SQLite. Verificare provenienza e autenticità dei pacchetti secondo il canale di distribuzione; un file «con il nome giusto» non è considerato verificato. Il file dei dati precedenti e quello più recente vanno mantenuti separati per evitare confusione e perdita.
- Le impostazioni potrebbero offrire solo un collegamento a «Informazioni di compatibilità e recupero» e la scelta facoltativa della cartella per installer/kit locali. Non è necessaria una scansione automatica delle cartelle personali, né una copia nascosta permanente dei programmi.
- Tutti i messaggi e le azioni seguono la lingua del sistema, con fallback inglese e override già previsto. Date e dimensioni sono localizzate; UUID, versioni e codici tecnici sono copiabili invariati. Interfaccia da tastiera, focus prevedibile, annunci screen reader e istruzioni consultabili quando la finestra principale non parte.

## 7. Dipendenze, compatibilità, complessità, rischi

**Dipendenze solo ipotetiche:** dati di versione dello schema e file già presenti; processo di release con artefatti ufficiali; proposta TECH-012 per checkpoint pre-migrazione; TECH-037 per eventuale esame di resti di restore. Non introdurre un sistema di backup, un installer personalizzato o un demone residente per questa funzione.

**Compatibilità Windows/macOS/Linux:** installazione, disinstallazione e riavvio differiscono fra sistemi. La [documentazione Tauri v2 Updater](https://v2.tauri.app/plugin/updater/) chiarisce la verifica obbligatoria della firma dei pacchetti, l'uscita dell'app durante l'installazione su Windows e la possibilità configurabile (non automatica) di ammettere versioni precedenti. Queste capacità del plugin **non equivalgono** a un rollback sicuro dei dati. Verificare il comportamento reale dei pacchetti di Repertica su ogni OS prima di promettere recuperabilità.

**Complessità stimata:** bassa–media per matrice compatibilità/release e guida offline; media per uno strumento read-only di diagnosi avviabile indipendentemente dall'archivio; alta per recupero guidato completo e protezioni cross-platform. Proposta deliberatamente frazionata per non creare un secondo updater.

**Rischi e mitigazioni:** installer non ufficiali (verifica origine/firma); ripristino distruttivo da checkpoint vecchio (non sovrascrivere, avviso dati più recenti); false promesse di compatibilità (stato «ignoto» e prove native); consumo di disco (nessun caching automatico illimitato); apertura accidentale di dati sensibili in un assistente esterno (report minimizzato); conflitti con migrazioni future (unico proprietario del checkpoint in TECH-012).

## 8. Criteri di accettazione e prove da svolgere solo dopo eventuale approvazione

La presente ricognizione **non esegue test né modifica test**. Se il proprietario approverà una fase futura, definire almeno:

- Matrice di compatibilità pubblicata per ciascuna release: versione programma, intervallo schemi letti, possibili migrazioni, piattaforme supportate, documenti/media da verificare e limitazioni note.
- Installazione di una release firmata con database compatibile: nuovo programma apre e verifica dati campione, media, certificati storici, incassi e impostazioni; senza contattare servizi obbligatori.
- Installazione con migrazione: distinguere checkpoint verificato, schema iniziale/finale e stato di compatibilità col binario precedente; non indicare «reversibile» una migrazione senza percorso provato.
- Simulare installer fallito, rilancio fallito, spazio pieno, dispositivo esterno assente, interruzione, avvio offline e reinstallazione precedente. Dimostrare che il file dati più recente rimane identificabile e che nessuna azione elimina contenuti senza conferma.
- Un vecchio eseguibile deve rifiutare uno schema più nuovo **senza creare o convertire database**; una vecchia copia di archivio può essere provata in directory separata ma non sostituisce l'attuale automaticamente.
- Se ci sono vendite o fotografie successive al checkpoint, un tentativo di recupero deve avvertire che mancherebbero e deve conservare entrambe le versioni. Nessuna fusione tacita dei dati.
- Report privo di dati personali per default; pulsanti e messaggi verificati con sola tastiera e lettore di schermo, lingue italiano/inglese e fallback inglese.

## 9. Fasi ipotetiche e domande per il proprietario

**Fase 0 — validazione senza sviluppo:** verificare se ci sono casi reali di fallimento aggiornamento; definire una matrice release/schema e una procedura manuale che distingua programma e dati; confrontare le risorse già presenti nelle release.

**Fase 1 — documentazione di recupero verificata:** testare le istruzioni offline su release firmate e Windows/macOS (Linux solo se distribuito); chiarire cosa fare quando il DB è già migrato, senza promettere un installer reversibile.

**Fase 2 — facoltativa:** interfaccia diagnostica read-only indipendente dall'apertura DB, o inventario di copie/installatori scelti dall'utente; solo se la procedura manuale risulta insufficiente.

**Fase 3 — eventuale, condizionata:** percorso di recupero assistito per professionisti con dati importanti, riusando TECH-012/037 e senza sincronizzazione o downgrade implicito.

**Questioni aperte per il proprietario:**
1. È utile mantenere pubblicamente gli installer di più release precedenti e un quadro chiaro versione-app/versione-schema?
2. Una guida offline nella documentazione e nel sito è sufficiente o serve un'esperienza di recupero **anche se la UI principale non si avvia**?
3. Quale comportamento consentire quando non è configurato il backup automatico prima dell'aggiornamento, distinguendolo dal checkpoint pre-migrazione progettato in TECH-012?
4. Quali piattaforme verranno ufficialmente supportate con installatori firmati e collaudati? Le build non notarizzate/ad hoc vanno considerate solo di test?
5. Quanto è accettabile conservare più installer e copie locali su un disco piccolo? Preferire un kit opzionale, senza caching obbligatorio?
6. Chi ha la responsabilità di verificare le tabelle di compatibilità prima di pubblicare una release? Come si segnala una combinazione non testata?

**Conclusione:** proposta **non approvata**. Priorità condizionata alla robustezza dei rilasci e alla frequenza di problemi reali. Nessuna implementazione, issue, merge o assegnazione.
