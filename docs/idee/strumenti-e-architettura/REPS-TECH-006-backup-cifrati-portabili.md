# REPS-TECH-006 — Backup cifrati facoltativi, portabili e ripristinabili

- **Data ricognizione:** 2026-11-21
- **Stato:** **Proposta — da valutare**
- **Priorità proposta:** **P1–P2** per utenti che trasportano archivi o conservano copie in cartelle condivise; da validare con il proprietario e con collaudi dei backup attuali.
- **Ambito:** riservatezza **delle copie di backup a riposo**, non cifratura automatica del database attivo, gestione account, sincronizzazione, archiviazione cloud o sostituzione dei backup esistenti.
- **Nessuna autorizzazione allo sviluppo** derivante da questo documento.

## Problema osservato, utenti e confini

Il [README](../../../README.md) e [MVP](../../MVP.md) dichiarano esplicitamente che i backup attuali sono **in chiaro**: una cartella scelta dall'utente contiene `repertica.sqlite`, `media/` e `manifest.json`; il controllo SHA-256 verifica integrità accidentale ma **non** impedisce a chi accede ai file di leggere clienti, indirizzi, fotografie, note, prezzi o spese. Questo è ragionevole per copie su dispositivi già protetti, ma crea un rischio concreto per artisti, gallerie e archivi che conservano copie su USB rimovibili, NAS condivisi o dischi affidati a collaboratori.

**Destinatari:** artista che viaggia con una copia su disco esterno; galleria con backup in una cartella NAS condivisa; archivista che passa un supporto a un tecnico; collaboratore che conserva una copia di emergenza senza accesso ordinario ai dati. La proposta **non** rende automaticamente sicuro un computer infetto e non protegge il database quando l'app è aperta.

**Non duplicare:**
- Il backup completo, il manifest con SHA-256, la scelta libera della destinazione, la verifica dei file e il ripristino con copia di sicurezza **esistono già** ([MVP](../../MVP.md), [backup.rs](../../../src-tauri/src/backup.rs)); non vengono riproposti.
- [TECH-001](REPS-TECH-001-pacchetto-dati-aperto.md) riguarda scambio **semantico** con software terzi, non la segretezza di copie complete.
- [TECH-003](REPS-TECH-003-diagnostica-integrita-media.md) riguarda la salute dei **media attivi**, non la cifratura.
- [TECH-004](REPS-TECH-004-ricerca-federata-archivi-locali.md) prevede fonti verificabili in **sola lettura**: i backup cifrati non devono essere indicizzati o decifrati di nascosto. Una futura compatibilità richiederebbe una decisione separata.
- [PR #14](https://github.com/Cesare-ml/Repertica-Studio/pull/14) raccoglie funzioni gestionali e UX; qui si tratta solo di protezione tecnica delle copie.

## Scenari, specifica e flussi UX proposti

1. **Backup ordinario invariato:** Impostazioni → Backup del tuo archivio continua a offrire «Backup standard (non cifrato)» con comportamento e compatibilità attuali. Non trasformare automaticamente le copie precedenti né rendere obbligatoria una password.
2. **Nuovo backup riservato:** scelta esplicita «Crea backup cifrato», descrizione dei dati protetti e avviso chiaro: *senza frase segreta o chiave di recupero, i dati potrebbero essere irrecuperabili*. L'utente sceglie **qualsiasi cartella accessibile** (locale, USB, NAS montato). Mostrare spazio disponibile/stimato, formato e versione, risultato della verifica e percorso finale.
3. **Scelta del segreto:** ipotesi di prima versione con **frase segreta scelta dall'utente**, inserita due volte, campo mostra/nascondi e indicazione non prescrittiva della robustezza. Nessun obbligo di account, salvataggio in cloud, recupero via email o invio del segreto a server. Eventuale integrazione *opt-in* con keychain OS solo in una fase separata; la **portabilità fra computer** deve restare possibile senza dipendere dal keychain originario.
4. **Verifica:** «Verifica copia» rileva il nuovo formato e chiede la frase segreta; autenticazione **di ogni blocco** e validazione del contenuto ricostruito, compresi integrità SQLite, relazioni e media. Distinguere in UI «file leggibile», «copia decifrata e verificata», «password o contenuto non valido». Non scrivere dati in archivio attivo durante la verifica.
5. **Ripristino guidato:** selezione cartella/file del backup, richiesta del segreto solo al momento necessario, anteprima minima non riservata prima di sbloccare; **dopo** sblocco mostra contatori, versione, data e compatibilità. Prima di sostituire l'archivio attivo: convalida completa in area isolata, conferma distinta e **conservazione della copia dell'archivio precedente** secondo il flusso corrente. Chiarire che tale copia precedente rimane in chiaro, se così prevede l'implementazione corrente.
6. **Recupero su altro dispositivo:** installazione pulita Repertica su Windows/macOS/Linux, nessun accesso al dispositivo sorgente, scelta del backup cifrato e apertura tramite sola frase segreta. Niente dipendenza da percorsi assoluti, profilo originario o identificativo macchina.
7. **Errori e interruzioni:** frase errata, contenuto manipolato, disco pieno, NAS disconnesso, interruzione durante la cifratura o mancanza della cartella producono un fallimento esplicito senza pubblicare una copia incompleta, senza eliminare originali e senza compromettere l'archivio attivo. Non provare silenziosamente un'altra destinazione.

## Modello di dati e API concettuali — solo descrizione

**Contratto ipotetico** da sottoporre a revisione crittografica, non schema definitivo:

- Un **contenitore cifrato autocontenuto** e versionato, con una *intestazione minima non sensibile* (identificativo formato, versione, parametri KDF, sale casuale, riferimento all'algoritmo, dimensione approssimativa facoltativa), seguita da dati cifrati e autenticati. Non esporre nomi di opere, nomi dei media, indirizzi di clienti o dettagli economici nell'intestazione o nei percorsi dei file.
- All'interno del contenuto autenticato, riusare per quanto possibile la struttura logica corrente (snapshot SQLite, `media/`, manifest/hash/versioni). I check SHA-256 interni **non sono** un meccanismo di autenticazione del contenitore: la riservatezza e l'integrità contro manomissione richiedono crittografia **AEAD** con autenticazione completa e verifica prima di qualsiasi ripristino.
- Derivazione della chiave da frase segreta con **KDF resistente a tentativi offline** (ad esempio Argon2id, parametri esplicitamente versionati e verificati per dispositivi lenti; valutare alternative dove richiesto dai sistemi supportati). Usare primitive mature e librerie mantenute, **mai** algoritmi crittografici inventati. Salt e nonces casuali indipendenti; gestione sicura di chunk e numerazione per impedire riordinamento, omissione, truncamento e riuso di nonce. La scelta concreta richiede audit esterno.
- Cifratura in **streaming autenticato** a memoria limitata; il numero e la dimensione dei file possono essere grandi. Non lasciare copie in chiaro in directory temporanee generiche. Se è necessario uno staging in chiaro, valutarne protezioni, pulizia dopo crash e rischio su dischi non cifrati; preferire flusso che cifri i dati dal backup già consistente senza pubblicarne una copia provvisoria.
- **Compatibilità:** `format_version` del backup cifrato separata dalla versione dello schema SQLite e dal formato aperto TECH-001. L'app deve **continuare a riconoscere** i backup in chiaro già esistenti; versioni vecchie possono non supportare il nuovo contenitore e devono ricevere un messaggio leggibile, non interpretarlo come backup guasto.
- **API ipotetiche**, non funzioni approvate: `planEncryptedBackup(destination) -> Estimate`; `createEncryptedBackup(plan, secret) -> Receipt`; `inspectEncryptedBackup(source, secret?) -> Locked|Verified|Unsupported|Invalid`; `restoreEncryptedBackup(source, secret, confirmation) -> Result`. Le API non devono serializzare né registrare segreti in log, cronologie, event bus o protocollo MCP.

## Offline, sicurezza, privacy, lingua e accessibilità

- Tutto il flusso resta **locale**, senza account, abbonamento, upload, server esterni, telemetria o servizi di custodia chiavi. La cartella è sempre scelta dall'utente. Nessuna cifratura automatica degli export CSV: questi ultimi restano in chiaro, con avvertimento specifico.
- Preferire segreti temporanei in memoria solo per la durata dell'operazione; niente recupero improvvisato, cache persistenti di frasi segrete, password di default, chiavi hardcoded o visualizzazione del segreto in errori e dump. Considerare la cancellazione della memoria *best effort*, evitando promesse assolute.
- Una copia cifrata può comunque essere cancellata, sottratta, bloccata da ransomware o persa. La cifratura **non sostituisce** copie ridondanti, verifica periodica, protezione OS né collaudi di ripristino. La privacy del database **attivo** rimane distinta.
- Rifiutare file speciali, symlink, percorsi traversal, header non validi, parametri KDF eccessivi, contenitori enormi o troncati e decompression bomb. Il limite di risorse va validato prima di allocare memoria o aprire file; scrittura atomica su percorso temporaneo controllato.
- Lingua di sistema con fallback **inglese**, override manuale e stringhe complete per italiano/inglese/spagnolo/francese/tedesco ove supportate. Messaggi comprensibili su perdita della frase segreta e limiti di recupero. Tastiera, screen reader, focus visibile, avvisi non affidati solo ai colori; evitare di incollare frasi segrete in report di errore.
- Proteggere soprattutto i dati di terzi (clienti e collezionisti), le fotografie originali e gli snapshot immutabili dei certificati. Nessuna ricerca o esportazione automatica del contenuto decifrato.

## Dipendenze, alternative, rischi e complessità

**Dipendenze ipotetiche:** backup attuale affidabile e collaudato su Windows/macOS/Linux e USB/NAS ([issue #7](https://github.com/Cesare-ml/Repertica-Studio/issues/7)); compatibilità con schemi SQLite presenti/futuri; progettazione di contenitore e migrazioni; libreria crittografica consolidata e verifiche indipendenti; flussi Tauri nativi per selezione cartelle e gestione errori.

**Alternative:**
1. Mantenere **solo backup in chiaro** e raccomandare BitLocker/FileVault/LUKS o contenitori cifrati del sistema: meno codice e meno rischio crittografico nell'app, ma minore portabilità e più passaggi manuali.
2. Integrare un formato archivio cifrato standard e diffuso, **solo dopo** una valutazione documentata di interoperabilità, autenticazione, metadata leakage e streaming. Non sceglierlo perché popolare senza valutare rischio e manutenzione.
3. Cifrare direttamente il database attivo: **problema differente**, con gestione chiavi, prestazioni e migrazione più impegnative. Non mescolarlo con questa proposta.
4. Richiedere password per **ogni** backup: scartata come default ipotetico perché può impedire ripristini ad utenti che non ne hanno bisogno.

**Rischi:** perdita della frase segreta = perdita della copia; falsa sicurezza e copia in chiaro rimasta in altre cartelle; attacco offline a passphrase deboli; implementazioni o librerie non auditabili; temp in chiaro; metadata non cifrati; prestazioni su grandi raccolte; formati non apribili dopo molti anni; recovery su macchine diverse; incompatibilità con backup già esistenti; falsa segnalazione di successo senza autenticazione completa. Mitigazioni: default opt-in e informativa, KDF parametrizzato, revisione indipendente, test interruzione, documentazione pubblica del formato e **prova su secondo dispositivo**.

**Complessità:** **alta** per contenitore realmente sicuro, streaming, ripristino e mantenibilità pluriennale; **media** per UX/lingue. È una delle proposte per cui l'alternativa esterna al software potrebbe risultare preferibile.

## Criteri di accettazione e test ipotetici

1. Backup standard del formato v1 ancora verificabile e ripristinabile senza password; nessun cambiamento o sovrascrittura silenziosa.
2. Backup cifrato con password corretta verificato fino all'ultimo byte su altra installazione offline, con conteggi, UUID, immagini e snapshot dei certificati identici a quelli sorgente.
3. Password errata, dati alterati in testa/metà/fine, blocco rimosso/riordinato, header maligno e troncamento causano rifiuto **prima di cambiare l'archivio attivo**; non pubblicare una copia incompleta come valida.
4. Disco pieno, lettura fallita, cartella remota scollegata e interruzione nel mezzo del processo mantengono il backup precedente e i dati attivi intatti, senza temporanei in chiaro residui non protetti.
5. Prova di portabilità Windows/macOS/Linux con percorsi Unicode, USB/NAS e raccolte media grandi: memoria limitata, progressi, possibilità di annullamento e riapertura della copia sulla macchina nuova senza account o keychain originario.
6. I file cifrati non contengono in chiaro stringhe caratteristiche prelevate da database/media/nomi dei clienti; il formato documenta esplicitamente eventuali metadati non protetti.
7. Nessun segreto in log, report, crash dump applicativi gestiti, export o integrazione MCP; uso di test vettoriali e revisione della procedura crittografica.
8. Prima di una release stabile, test con utenti non tecnici: sanno distinguere backup standard e cifrato, capiscono **chi** può recuperarli e riescono a completare una simulazione di ripristino.

## Accertato, ipotizzato e da decidere

**Accertato il 2026-11-21 dal repository `main`:** il backup crea cartelle complete e non cifrate con manifest SHA-256, snapshot SQLite e media; verifica prima di ripristinare e conserva un backup precedente. I CSV sono esportazioni in chiaro. Nessun flusso di cifratura della copia emerge da README/MVP/`backup.rs`.

**Ipotesi:** una percentuale significativa degli utenti archivia backup su dischi removibili o NAS condivisi; la cifratura in-app ridurrebbe rischio più di quanto aggiunga complessità. Non ci sono dati di utilizzo che la confermino.

**Domande per il proprietario:** quante persone desiderano davvero copie cifrate senza software esterno? Meglio integrare libreria/standard diffuso o documentare strumenti OS? Accettabile avvertire che **non esiste recupero password**? Serve un keyfile opzionale, o sarebbe un onere in più? Quale metadata leakage residuo è accettabile? Un backup cifrato deve poter essere aperto da un tool CLI indipendente? La copia di rollback locale resta in chiaro o si vuole una modalità protetta distinta?

## Fasi di valutazione (non roadmap approvata)

1. Raccogliere bisogni concreti e scenari di rischio (USB, NAS, disco esterno); verificare se raccomandazioni per cifratura OS bastano.
2. Stendere threat model e confronto formati/librerie **senza codificare**; definire metadati esposti, gestione segreti, interoperabilità e compatibilità vecchi backup.
3. Solo dopo consenso esplicito: prototipo isolato su dati sintetici e revisione crittografica indipendente; criteri di recuperabilità su piattaforme diverse.
4. Eventuale progetto di UX, implementazione, collaudo e pubblicazione di specifiche del formato **soltanto se approvato**.

> Questa scheda è una **proposta da valutare**, non approva né assegna alcun intervento di sviluppo.
