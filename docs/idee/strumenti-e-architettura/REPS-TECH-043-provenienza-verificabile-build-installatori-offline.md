# REPS-TECH-043 — Provenienza verificabile delle build e degli installatori anche offline

**ID stabile:** REPS-TECH-043  
**Data ricognizione:** 2026-11-22  
**Stato:** **Proposta — da valutare**  
**Filone:** Strumenti e architettura — sicurezza della distribuzione e indipendenza dell'utente  
**Priorità ipotetica:** **P2** per un contratto minimo di provenienza nelle release; **P3** per strumenti di verifica integrati; nessuna priorità approvata.

> Studio documentale: nessun workflow, codice, dipendenza, configurazione, test, release o impostazione viene modificato da questa proposta.

## 1. Motivazione, destinatari e riscontri verificati

Repertica conserva localmente archivi di opere, certificati, contatti, fotografie e informazioni economiche. Il programma è gratuito/open source, Tauri + SQLite, utilizzabile senza account, abbonamenti o connessione permanente. Un artista che scarica un installer dal sito, un archivista che lo conserva su USB o una galleria che lo redistribuisce internamente hanno un problema specifico: **come verificare quale codice e quale processo abbiano prodotto esattamente quel pacchetto**, anche quando non c'è Internet?

Riscontri su `main` al 22/11/2026:
- `.github/workflows/release.yml` crea pacchetti Windows e macOS a partire da tag `v*`, controlla le versioni di `tauri.conf.json`, `package.json` e `Cargo.toml`, usa runner GitHub e pubblica con `tauri-apps/tauri-action@v0`; il job può essere rilanciato sullo stesso tag per aggiungere il solo macOS.
- `src-tauri/src/updater.rs` vincola il feed a GitHub Releases e richiede una **firma dell'aggiornamento** tramite la chiave del progetto. Questo già tutela il percorso dell'updater; non è necessario progettarne un altro.
- `docs/RELEASING.md` distingue chiaramente la firma del **pacchetto di aggiornamento** da quella dell'**app/installer secondo il sistema operativo**. Le build macOS attuali sono ad-hoc, non notarizzate; quelle Windows non hanno ancora firma di publisher OS. Non chiamarle «non firmate» senza questa distinzione.
- `.github/workflows/build-macos.yml` pubblica DMG **di prova** con `SHA256SUMS.txt`, non release ufficiali. `release.yml` e `ci.yml` usano `npm install` e una toolchain Rust `stable`; non emerge una **attestazione crittograficamente verificabile del commit e del processo di build**, un SBOM pubblicato né una guida di verifica offline degli installer ufficiali.
- I file `package-lock.json` e `src-tauri/Cargo.lock` sono già nel repository. Ciò facilita il controllo delle dipendenze, ma **non prova da solo build riproducibili**.
- Il README e `docs/MVP.md` dichiarano un MVP non ancora da usare come unica copia di archivi reali: questo studio **non attesta la sicurezza o affidabilità della release corrente**.

**Destinatari:** artisti senza conoscenze sistemistiche che scaricano il programma, gallerie con più postazioni, archivi con procedure di conservazione, manutentori open source e collaboratori che compilano l'app. **Esigenza da validare:** la procedura di verifica deve essere comprensibile senza imporre GitHub CLI, account o un'interfaccia tecnica a ogni artista.

## 2. Confini e confronto con le idee esistenti

- **[TECH-042](REPS-TECH-042-compatibilita-aggiornamenti-recupero-versione-offline.md)** si occupa di **compatibilità binario/schema, downgrade e recupero operativo**: qui solo **origine e identità degli artifact distribuiti**, senza promesse sulla capacità di aprire un archivio.
- **[TECH-012](REPS-TECH-012-migrazioni-archivio-preflight-copia-sicurezza.md)** e **[TECH-037](REPS-TECH-037-recupero-ripristino-interrotto-avvio-protetto.md)** riguardano sicurezza e recupero **dei dati**; qui non si modifica il backup.
- **[TECH-006](REPS-TECH-006-backup-cifrati-portabili.md)** e **[TECH-009](REPS-TECH-009-catalogo-verifiche-conservazione-backup.md)** trattano crittografia/cataloghi dei **backup locali**, non firme o provenienza degli eseguibili.
- La [PR #14, Gestione e UX](https://github.com/Cesare-ml/Repertica-Studio/pull/14) comprende **47 proposte** per attività professionali, non per attestazioni di build; nessuno dei suoi file viene modificato.
- **Non** è un'alternativa al codice firmato Apple/Windows, un programma bug bounty, uno scanner antivirus, un servizio cloud, un aggiornamento automatico aggiuntivo o una certificazione generale del software. L'attestazione prova **una relazione verificabile con una pipeline**, non l'assenza di vulnerabilità.

## 3. Requisiti certi, ipotesi e alternative

**Vincoli certi di prodotto:** desktop gratuito/open source; dati SQLite e media locali; funzionamento ordinario offline-first; nessun login, server o canone obbligatorio; piena esportabilità e backup in cartelle arbitrarie; lingua del sistema con fallback inglese e override; consenso esplicito a installare aggiornamenti.

**Ipotesi:** l'utente potrebbe voler verificare un pacchetto conservato su disco esterno o ricevuto tramite un collaboratore. Un file di sole impronte SHA-256 è utile contro errori di copia, **non dimostra autenticità** se file e impronte provengono dallo stesso canale non affidabile. Una prova di provenienza firmata e legata a tag, commit, repository e workflow può fornire un controllo più forte, a patto di verificare l'identità attesa e la catena di fiducia.

**Alternativa più leggera:** pagina di rilascio chiara, installer ufficiali identificati per architettura, hash e procedura per confrontarli con un riferimento ottenuto tramite un canale verificato. Se la domanda è bassa, non introdurre UI o dipendenze aggiuntive. **Alternativa avanzata non predefinita:** attestazione [GitHub Artifact Attestations](https://docs.github.com/en/actions/how-tos/secure-your-work/use-artifact-attestations/use-artifact-attestations), con bundle conservato insieme al file e root di fiducia verificabile offline. Valutare attentamente la dipendenza dal servizio GitHub e la disponibilità futura dei bundle.

## 4. Specifica funzionale e flussi UX ipotetici

### 4.1 Identità di release e inventario degli artefatti

Per ciascun tag ufficiale, un **manifest descrittivo** potrebbe elencare: versione, tag, commit immutabile, data e identificativo del workflow, piattaforma/architettura, nome e dimensione degli installer, SHA-256, tipo di firma disponibile, stato dell'attestazione e link alle evidenze. Etichette distinte per **installer ufficiale**, **pacchetto updater firmato** e **DMG di prova**. Se si completa una release in due passaggi (`only_macos`), dichiarare quando l'elenco è **completo**: una piattaforma assente non deve essere simulata o «promessa» dal manifest precedente.

Non assumere che gli installer di due build dello stesso tag siano identici. Una ricompilazione deve creare evidenza riferita **a quel preciso digest**. Non sostituire in silenzio il download di una release già verificata né riutilizzare l'attestazione di un artifact con byte diversi.

### 4.2 Esperienza per chi scarica

1. **Dal sito o da GitHub Releases:** mostrare versione, sistema e architettura, natura ufficiale/di prova e una voce discreta «Verifica origine e integrità». Nessun check obbligatorio all'uso ordinario.
2. **Controllo semplice di un file scaricato:** mostrare il digest SHA-256 e istruzioni locali verificabili per Windows/macOS; risultato «byte uguali/differenti», **non** «software sicuro».
3. **Controllo avanzato della provenienza:** ove esista attestazione, controllare firma, soggetto (digest dell'artifact), repository/identificatore, workflow, ref/tag e commit atteso. Risultati distinti: *verificato*, *digest differente*, *identità non corrispondente*, *attestazione non presente*, *impossibile verificare offline*. Nessun fallimento va mascherato da successo.
4. **Chiavetta o rete assente:** l'utente può portare con sé installer, eventuale attestazione e radici di fiducia previamente acquisite da fonte autentica. La verifica offline deve chiarire che **revoche, nuovi certificati o rotazioni non sono automaticamente conoscibili** senza consultazioni successive.
5. **Aggiornamento in-app:** lasciare invariato il controllo minisign già presente; le evidenze di provenienza sono un **livello informativo opzionale**, non un secondo canale di installazione o una scorciatoia per saltare la firma Tauri.
6. **Errore o dubbio:** nessun pulsante «Installa comunque» basato soltanto sul mancato controllo delle attestazioni; mostrare canale ufficiale e possibilità di annullare, senza inviare a terzi file dell'archivio.

### 4.3 Eventuale controllo automatico (solo dopo validazione)

Un esperto potrebbe scegliere un file locale e una fonte attendibile preconfigurata per confrontare le evidenze. L'operazione sarebbe **read-only**, strettamente circoscritta ai file selezionati e attivata dall'utente. Nessuna scansione di cartelle, lettura di opere/collezionisti, telemetria, download/installazione automatica o account. Per il MVP è preferibile una procedura documentale esterna all'app.

## 5. Dati e ipotesi API — esclusivamente descrittivi

Nessuna nuova tabella SQLite necessaria: la provenienza riguarda **gli artefatti di distribuzione**, non l'archivio professionale. Possibili strutture di documentazione **non normative**:

| Oggetto concettuale | Campi | Regole |
| --- | --- | --- |
| `ReleaseIdentity` | `tag`, `source_commit_sha`, `workflow_identity`, `run_id`, `published_at` | Tag e SHA vanno verificati; data e titolo da soli non autenticanti |
| `ArtifactIdentity` | `filename`, `platform`, `arch`, `size_bytes`, `sha256`, `kind` | Digest dei **byte finali esatti**; distinguere updater/installer/test |
| `SignatureEvidence` | `mechanism`, `bundle_ref?`, `subject_digest`, `signer_identity?`, `trusted_root_version?` | Firma Tauri, firma OS e attestazione sono evidenze **diverse**, non intercambiabili |
| `VerificationReport` | `observed_digest`, `expected_digest`, `identity_match`, `verification_status`, `limitations` | Report locale minimizzato; nessuna falsa etichetta di sicurezza totale |

API ipotetiche: *inspectLocalInstaller(path)*, *compareDigestToTrustedRecord(path, record)*, *verifyAttestationOffline(artifact, bundle, trustedRoots, expectedIdentity)*, *renderVerificationExplanation(result, locale)*. **Non** sono comandi, endpoint pubblici, dipendenze approvate o contratto MCP. Eventuale verifica non deve consentire di avviare il pacchetto come effetto collaterale.

## 6. Sicurezza, privacy, offline, impostazioni e multilingua

- Nessun dato personale, nome opera, importo, fotografia, percorso dell'archivio o statistica locale nel manifest di distribuzione. Eventuali log della verifica restano locali, copiabili solo su richiesta. Mai inviare hash di documenti privati a servizi di attestazione.
- Il modello di fiducia deve considerare **compromissione di tag, workflow, chiavi di firma e runner**. Un artefatto attestato può comunque essere dannoso se il sorgente o il processo autorizzato è compromesso. Separare credenziali di pubblicazione, revisioni e gestione delle chiavi; non pubblicare mai chiavi private.
- L'uso dell'app e l'apertura dei dati rimangono offline. La verifica di un installer può essere offline **solo se** artefatto, bundle, informazioni attese e root attendibili sono già disponibili. Chiarire i limiti sulla validità temporale e rotazione della root; non richiedere GitHub CLI installata a tutti.
- Le impostazioni, qualora servissero, mostrerebbero solo informativa e collegamento alla guida di verifica, senza nuove impostazioni obbligatorie. Tutti gli avvisi in italiano/inglese e lingue supportate, lingua di sistema con fallback inglese e override; hash, nomi release e SHA non si traducono. La guida deve essere leggibile da tastiera, screen reader, zoom e testo selezionabile.
- La «provenienza verificata» non significa «versione compatibile con l'archivio» (TECH-042), «installer notarizzato», «reputazione SmartScreen acquisita» o «copia dell'archivio al sicuro». Queste garanzie non vanno fuse in un indicatore unico.

## 7. Dipendenze, compatibilità, rischi e complessità

**Dipendenze ipotetiche:** GitHub Releases e workflow già presenti, tag versionati, lockfile Node/Rust; eventuali attestazioni GitHub/Sigstore come alternativa da valutare. L'uso di `stable` per Rust, `npm install`, action riferite a tag major e runner mutabili va analizzato nel quadro della riproducibilità. **Non modificare automaticamente** il workflow e non promettere build byte-identiche: firme, timestamp, notarizzazione, toolchain e packaging possono legittimamente variare.

**Compatibilità:** Windows installer, DMG macOS Intel/Apple Silicon e pacchetti updater hanno bytes e meccanismi di verifica diversi. L'assenza di firma OS non annulla la firma updater; la firma updater non rende un DMG notarizzato. Un SBOM eventuale (SPDX/CycloneDX) aiuta inventario/audit delle dipendenze ma non sostituisce attestazione o audit del codice. Nessuna modifica agli archivi o schema dei backup.

**Rischi:** falsa sicurezza da checksum co-pubblicati; introduzione di tool complessi per utenti non tecnici; dipendenza dal servizio attestazioni e sue radici; duplicazione con workflow esistente; errore nel tag/commit dichiarato; costi di mantenimento cross-platform; pubblicazione in più tempi che produce manifest incompleti; differenze tra compilazioni. **Mitigazioni:** esiti trasparenti, prove periodiche realmente ripetibili, fonte verificabile, nessun obbligo operativo per gli utenti e documentazione «non verificato» quando manca una prova.

**Complessità stimata:** bassa per inventario rigoroso + guida con SHA-256; media per pubblicare e verificare attestazioni per ogni artifact di release; medio-alta per un vero processo di compilazione riproducibile cross-platform; alta e probabilmente ingiustificata per introdurre una UI nativa dedicata prima di casi concreti.

## 8. Criteri di accettazione e prove future (non eseguite)

1. Da un tag ufficiale si risale **senza ambiguità** a commit sorgente, workflow, runner e **digest di ogni installer realmente distribuito**; niente artifact inesistenti, né prova riutilizzata dopo ricompilazione.
2. Il controllo dell'hash rileva un byte alterato; l'attestazione, se adottata, rifiuta digest/identità/tag/commit/workflow sbagliati, bundle malformati o root di fiducia non corretta. «Impossibile verificare» non equivale a «valido».
3. Provare il percorso offline con solo pacchetto, bundle e root preacquisiti; documentare che l'offline non aggiorna informazioni su revoche o rotazioni. Vedi [guida GitHub alla verifica offline](https://docs.github.com/en/actions/how-tos/secure-your-work/use-artifact-attestations/verify-attestations-offline).
4. Simulare rilascio prima Windows e poi macOS con `only_macos`; verificare che ogni artifact e il manifest finale si riferiscano ai **byte pubblicati**, non a un aggregato preesistente.
5. Conservare in documentazione e in output separate le tre nozioni: **firma updater Tauri**, **firma/notarizzazione OS** e **provenienza della build**; nessuna viene presentata come prova di assenza di vulnerabilità.
6. Non imporre modifiche a SQLite/backup; nessun contatto ai servizi di rete quando l'utente usa l'app senza verificare installer; nessun accesso ai dati personali per creare il report.
7. Prove manuali delle istruzioni su Windows/macOS, tastiera e screen reader; localizzazione dei messaggi con fallback inglese. I test sarebbero pianificati **soltanto dopo eventuale approvazione**.

## 9. Fasi proposte e domande aperte per il proprietario

**Fase 0 — nessuno sviluppo:** raccogliere casi di uso offline e distribuzione da USB/galleria; chiarire se gli installer passano attraverso il sito Repertica o direttamente da GitHub. Documentare le evidenze già disponibili e i limiti delle firme esistenti.

**Fase 1 — eventuale minimo documentale:** definire un inventario univoco di artifact e commit per release, una guida di confronto SHA-256 e regole per distinguere build di test; collaudare le istruzioni **senza** modificare il programma.

**Fase 2 — condizionata:** valutare attestazioni firmate verificabili e bundle da conservare vicino agli artifact, con test di identità e offline; stimare costi operativi, dipendenza GitHub e processo di rilascio multipiattaforma.

**Fase 3 — solo se serve:** SBOM, rafforzamento di tracciabilità/ripetibilità delle build e strumenti di verifica più accessibili. Non impegnarsi a riproducibilità bit-per-bit senza prove e senza considerare firma/notarizzazione.

**Questioni da sottoporre al proprietario:**
1. Occorre una prova di origine per chi **scarica direttamente un installer** (non tramite l'updater già firmato)?
2. Il download di test macOS dal workflow deve restare chiaramente distinto dalle release pubbliche sul sito?
3. È accettabile che una parte della verifica di provenienza dipenda da GitHub/Sigstore, se l'utilizzo dell'archivio e una verifica offline preparata rimangono indipendenti?
4. Chi cura e conserva le evidenze di ogni artifact quando una release viene completata in più job o ricompilata sul medesimo tag?
5. Quale livello di istruzioni è sostenibile: checksum e guida, bundle verificabile per esperti, o futura UI? Sono arrivate richieste reali di verifica autonoma?
6. Quali garanzie **non** promettere prima della firma OS e della notarizzazione macOS?

**Conclusione:** una proposta tecnica separata dal recupero dell'app e dalla salvaguardia dell'archivio. **Da valutare**, non approvata, non assegnata e non autorizzata allo sviluppo.
