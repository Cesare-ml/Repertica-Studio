# REPS-TECH-059 — Acquisizione temporanea di fotografie da smartphone su rete locale, senza cloud

- **ID stabile:** REPS-TECH-059
- **Data della ricognizione:** 2026-11-23
- **Stato:** **Proposta — da valutare**
- **Filone:** Strumenti e architettura, **solo analisi e documentazione**.
- **Priorità ipotetica:** **P3 di partenza**; eventualmente **P2** se un bisogno ricorrente viene confermato e una revisione della sicurezza giustifica un trasferimento locale. **Nessuna approvazione, implementazione o assegnazione**.
- **Decisione raccomandata prima di qualsiasi sviluppo:** provare e documentare i flussi già disponibili (USB, condivisione OS, cartella locale e import multiplo). Un listener di rete non è giustificato dalla sola comodità ipotizzata.

## 1. Motivo, destinatari e riscontri su `main`

**Problema da verificare.** Un artista fotografa opere, firme o allestimenti con un telefono ma lavora sul catalogo nel desktop. Oggi deve prima far arrivare i file al computer (USB, AirDrop/Quick Share, cartella condivisa o trasferimento manuale), poi importarli in Repertica. Per uno studio con frequenti sessioni fotografiche questo passaggio può diventare ripetitivo. Un trasferimento **solo sulla stessa rete locale**, aperto esplicitamente dall'utente e chiuso subito dopo, potrebbe eliminare l'intermediazione di servizi online senza trasformare Repertica in un prodotto cloud.

**Destinatari possibili:** artisti individuali con telefono e desktop, assistenti di studio, piccole gallerie e archivisti che fotografano l'inventario. **Non esistono in questa ricognizione interviste, segnalazioni o misurazioni** che dimostrino un problema reale. È un'ipotesi di efficienza, non un bug.

**Verifiche statiche del repository `main` al 2026-11-23:**

- [README](../../../README.md), [MVP](../../MVP.md) e [portabilità](../../DATA-PORTABILITY.md) confermano: app Tauri/Angular/Rust con SQLite e media locali; foto nel backup completo e non nei CSV, backup destinabile a una cartella scelta liberamente, nessun account richiesto.
- In [Media Center](../../../src/app/media-center.component.ts) `importFiles()` usa **già** il dialogo nativo `open({ multiple: true })` per JPEG/PNG/WebP, seguito da `api.mediaImport(paths)`. L'import multiplo **esiste**; non proporlo nuovamente.
- [`src-tauri/src/media.rs`](../../../src-tauri/src/media.rs) `media_import`/`import_core` legge percorsi locali e registra gli oggetti nella libreria; l'associazione di media già importati a opere è un'altra operazione esplicita. Non è presente, fra i comandi registrati in [`lib.rs`](../../../src-tauri/src/lib.rs), un ricevitore HTTP/LAN per le foto. **È una constatazione sul codice esaminato, non un test dinamico della rete**.
- [`src-tauri/capabilities/default.json`](../../../src-tauri/capabilities/default.json) limita le capacità Tauri della finestra principale; ciò **non** costituisce già una politica di sicurezza per un ipotetico server Rust di rete.
- [`docs/MCP.md`](../../MCP.md) descrive un **server MCP STDIO, sola lettura e senza porte di rete**. Questa proposta non deve né riutilizzarlo come listener né ampliare il suo perimetro.
- Il backup completo attuale include i file media verificati. Il trasferimento dal telefono **non** è un backup e non deve produrre riferimenti a file remoti permanenti.

## 2. Confini e confronto anti-doppioni

- [TECH-027](REPS-TECH-027-importazione-fotografie-heic-heif-offline.md) tratta **decodifica, conservazione e anteprime HEIC/HEIF**, non il **canale** con cui un telefono consegna i byte. Anche un file ricevuto via LAN resterebbe soggetto ai formati effettivamente supportati: non si può accettare HEIC modificando solo il trasporto.
- [TECH-045](REPS-TECH-045-associazione-guidata-foto-importate-opere.md) propone abbinamenti **dopo** che i media sono già importati; qui si ipotizza **solo il passaggio del file al Media Center**. Nessun abbinamento automatico per titolo, nome file, IA o codice.
- [TECH-038](REPS-TECH-038-staging-media-backup-concorrenza-snapshot.md) studia staging e coerenza fra media e backup; qui andrebbero riutilizzate le medesime invarianti, non introdotta una via alternativa che le elude.
- [TECH-013](REPS-TECH-013-copie-fotografiche-condivisibili-privacy-metadati.md) e [TECH-024](REPS-TECH-024-metadati-fotografici-iptc-xmp-interoperabili.md) governano metadati e copie da condividere; il trasferimento **in ingresso** non conferisce automaticamente diritti d'uso né autorizza a importare campi EXIF come dati catalografici.
- [TECH-005](REPS-TECH-005-anteprime-media-grandi-archivi.md) riguarda prestazioni delle anteprime, non il trasporto.
- Nel filone [Gestione e UX, PR #14](https://github.com/Cesare-ml/Repertica-Studio/pull/14) esistono proposte su catalogazione seriale (UX-018), verifica doppioni (UX-055), opere digitali (UX-057), contenuti e diritti. **Non** introdurre in questo studio nuovi flussi di vendita, creazione opere, revisioni della scheda o associazioni a collezionisti; i relativi file non sono modificati.

**Escluso:** sincronizzazione bidirezionale, accesso al database da telefono, app mobile obbligatoria, cloud o relay esterno, scansione ricorsiva automatica della galleria del telefono, server sempre acceso, QR pubblico permanente, account, IA/riconoscimento visivo, invio di file ai contatti, scrittura da dispositivo remoto di opere, vendite, certificati o impostazioni.

## 3. Scenari d'uso e flussi UX descrittivi

**A. Trasferire tre fotografie per un'opera.** Artista sul desktop: «Media Center → Importa → Da smartphone (rete locale)», con testo che spiega che il computer diventa **temporaneamente raggiungibile sulla LAN**. Sceglie «Apri sessione» dopo una conferma. Una pagina locale da aprire sul telefono può essere raggiunta tramite QR e pairing di sessione **solo se il canale è stato progettato sicuro**. Sul telefono l'utente **sceglie esplicitamente** tre file mediante il selettore del browser; l'accesso alla fotocamera integrata è solo eventuale e dipende dal supporto del browser e dal contesto di sicurezza. Sul desktop vede nome, formato, dimensione e anteprima *solo per i file supportati*; seleziona «Importa nel Media Center». L'associazione successiva alla scheda dell'opera è **manuale** e usa i comandi già esistenti. «Chiudi sessione» revoca ogni accesso.

**B. Rete non disponibile.** PC e telefono su reti diverse, Wi-Fi con isolamento client, firewall aziendale, VPN, hotspot non compatibile, nessun Wi-Fi o permessi OS negati → messaggio esplicito; niente fallback verso servizi pubblici. Proporre trasferimento tramite USB/AirDrop/Quick Share/cartella condivisa e poi l'import multiplo già disponibile. Un'eventuale prova non deve richiedere accesso a Internet.

**C. File inatteso o upload interrotto.** HEIC non supportato, video, file eccessivo, falso MIME, filename ostile, disconnessione, telefono che cambia rete, disco pieno → rifiuto o fallimento della singola sorgente **prima** di creare un media definitivo; mostrare il numero di ricevuti, importati, scartati e ancora in attesa. I temporanei devono essere scaduti e rimossi in modo sicuro, senza cancellare file già importati o media storici. **Nessun ritentativo implicito** che rischi duplicati.

**D. Telefono non fidato o sessione dimenticata.** L'app non deve pubblicare nomi delle opere, elenco clienti, prezzi o immagini dell'archivio al telefono. Se non si può verificare identità/canale, l'app **non avvia** la sessione. Chiusura manuale, chiusura all'uscita del desktop, scadenza breve e stato visibile «Sessione di ricezione attiva»; non esiste una modalità permanente nascosta.

**Accessibilità:** sia sul desktop sia nell'eventuale pagina mobile, azioni tastiera/screen reader, progressi testuali, focus ripristinato, errore distinto per ogni file, annullamento esplicito e zoom senza troncamenti. La pagina di ricezione non deve rivelare anteprime dei file locali già presenti.

## 4. Specifica funzionale minima e ipotesi di API (non implementazioni)

**Principio:** il trasporto consegna **solo byte non fidati**. È il backend locale esistente a decidere se possono diventare un media. Nessuna funzione remota deve invocare liberamente `media_update`, `artwork_gallery_from_media`, comandi Tauri o query SQLite.

Modello descrittivo, da decidere solo in fase di fattibilità:

```text
LocalTransferSession {
  session_id, created_at, expires_at, status: awaiting_pair | receiving | closed,
  chosen_network_interface?, device_label_optional?,
  paired_device_fingerprint_or_ephemeral_public_key?, // solo se protocollo sicuro verificato
  max_files, max_total_bytes, per_file_limit,
  received_count, accepted_count, rejected_count
}
PendingPhoto {
  transfer_id, session_id, display_filename_sanitized, detected_type,
  size_bytes, sha256, temporary_staging_id, validation_state,
  error_code?, acknowledged_by_user: boolean
}
openLocalTransferSession(user_confirmation, interface_policy) -> SessionInvitation
receivePhotoChunkOrFile(session_id, paired_peer, bytes, declared_meta) -> PendingReceipt
inspectPendingTransfers(session_id) -> SanitizedPreview[]
confirmSelectedImports(pending_ids, expected_session_state) -> ImportSummary
closeLocalTransferSession(session_id) -> ClosedAndRevoked
```

Le API sono **solo ipotesi**; non costituiscono permesso a creare endpoint. `display_filename_sanitized` è **informativo** e non determina il percorso su disco. L'archiviazione definitiva deve usare UUID e percorsi relativi sotto `media/`; staging isolato e non indicizzabile fino a validazione e conferma. Testare atomicità, rollback e cancellazione dei temporanei considerando il mutex SQLite e le copie di backup concorrenti. Per duplicati identici proporre solo un avviso con hash e confronto, non cancellazione o collegamento automatico.

**Limiti iniziali ipotetici:** una sola sessione, un dispositivo per volta, solo foto JPEG/PNG/WebP fino ai limiti di import effettivamente definiti nel codice; tetto complessivo prudente e coda piccola; nessun RAW/video né accesso alla libreria dal telefono. Le soglie non sono approvate e devono essere misurate su hardware reale. **Non affermare che tutte le immagini smartphone, in particolare HEIC, siano supportate.**

## 5. Offline, sicurezza e privacy: condizioni bloccanti

La rete locale **non implica** un canale sicuro. Una sessione HTTP in chiaro su Wi-Fi condiviso può esporre fotografie e codici di pairing a intercettazione, manipolazione e spoofing. **Non** basta un token casuale nel link QR per garantire confidenzialità o autenticazione; token in query string possono finire in cronologie, screenshot, log o referrer. La decisione fra TLS con autenticazione/pinning verificabile, protocollo cifrato con pairing fuori banda o nessun server deve essere sottoposta a una **revisione delle minacce** prima di qualsiasi prototipo che trasporti immagini private. **Nessun invito a ignorare gli avvisi del browser per certificati non attendibili**. Evitare il modello «listener HTTP non autenticato sull'intera LAN», inclusi test distribuiti agli utenti.

Requisiti da dimostrare se proseguisse lo studio:

1. **Avvio solo su gesto esplicito, disattivato di default**, durata breve, revoca e stato visibile; binding su interfaccia LAN deliberata, niente WAN/port forwarding, UPnP, DNS pubblici, relay o telemetria; nessuna modifica silenziosa al firewall di Windows/macOS/Linux.
2. Identificazione della sessione e del peer con un **protocollo resistente a intercettazione e replay**, non solo PIN a sei cifre o QR contenente URL in chiaro; limiti di tentativi, dimensione, numero file e velocità; protezione da CSRF, DNS rebinding, richieste da pagine web ostili e upload cross-origin indesiderati.
3. Nessuna navigazione del filesystem, lettura di dati SQLite, foto precedenti, contatti, importi, token MCP o chiavi sul computer. Input sanitizzato (nome, MIME, dimensioni reali, magic bytes, limite pixel/decoder, zip bomb e file speciali); non fidarsi di `Content-Type` e delle estensioni.
4. I file in staging non devono essere eseguibili, aperti da parser permissivi o conservati senza limite. Se un file fallisce o la sessione scade, **nessuna sostituzione, cancellazione o nuova riga incompleta**. Crash/restart: nessun listener si riattiva automaticamente e nessun pairing precedente resta valido.
5. Distinguere «foto ricevuta», «foto validata» e «foto registrata». Solo le fotografie definitivamente importate entrano nel backup completo; i temporanei vanno segnalati separatamente. Le immagini possono contenere EXIF/GPS e persone identificabili: consenso sul **trasferimento**, nessuna estrazione o pubblicazione automatica.
6. Il server MCP STDIO rimane rigorosamente separato e read-only. Anche qualora il trasferimento LAN fosse fattibile, **non** introduce un server MCP HTTP né una API generica dell'archivio.

Il funzionamento locale significa **nessuna dipendenza da Internet**, ma può richiedere una rete LAN configurata dall'utente. Niente account Repertica né abbonamenti. L'app resta pienamente utilizzabile con la funzionalità disattivata.

## 6. Impostazioni, localizzazione, dati ed esportazioni

- **Impostazioni:** preferenza solo locale «Consenti sessioni di ricezione da smartphone» inizialmente non abilitata, eventualmente eliminabile del tutto; interfaccia di rete esplicita se più interfacce; scadenza sessione non prolungata in automatico; nessun dispositivo «sempre autorizzato» nella prima fase. Non memorizzare codici di pairing in SQLite o log.
- **Lingue:** lingua OS, fallback inglese e scelta manuale già prevista IT/EN/ES/FR/DE; testi relativi a sicurezza, consenso, sessione, QR, rifiuti, scadenza e dati incompleti tradotti. Titoli, filename e metadati originali non tradotti; codici d'errore stabili e messaggi comprensibili. Test con tastiera e lettori di schermo su desktop e mobile.
- **Modello dati:** evitare nuove tabelle persistenti per una funzione solo di trasporto; gli ID di sessione sono effimeri e le foto confermate diventano normali record del Media Center, secondo schema esistente. Nessuna modifica ai certificati storici.
- **Import/export:** il flusso si conclude nell'importazione locale esistente; i registri CSV restano metadati, non contengono i byte; per esportare/spostare l'archivio valgono backup e ripristino verificati in cartelle liberamente scelte. Trasferimento LAN **solo in ingresso**, non sincronizzazione e non sostituto di un backup.
- **Compatibilità:** valutare Windows/macOS/Linux, firewall, Wi-Fi ospite, VPN, hotspot, iOS Safari/Android Chrome e limiti dei file picker; l'accesso alla camera del browser può richiedere contesto sicuro HTTPS e permessi ulteriori. Non promettere la stessa UX su tutti i sistemi.

## 7. Alternative, dipendenze, rischi e complessità

| Alternativa | Valore | Costi e limiti |
| --- | --- | --- |
| **A. Solo guida al flusso esistente (prima scelta da testare)** | Nessuna rete aperta da Repertica; conservazione dei normali controlli import; massimo rispetto offline | Passaggio intermedio nel filesystem; utenti poco tecnici potrebbero trovare scomoda la condivisione OS |
| **B. Apertura cartella di ricezione locale scelta dall'utente** | Sfrutta trasferimenti USB/AirDrop/Quick Share già gestiti dall'OS, senza listener Repertica | La condivisione è configurata fuori dall'app e resta a carico dell'utente |
| **C. Sessione LAN temporanea con accoppiamento sicuro** | Potenziale rapidità da browser su telefoni senza installare un'app | **Alta complessità/superficie d'attacco:** cifratura e autenticazione, rete, permessi firewall, test cross-device, gestione errori e limiti, supporto continuo |
| **D. Servizio cloud di upload** | Può superare reti isolate | **Fuori perimetro:** infrastruttura, trasferimento dati, account e dipendenza operativa incompatibili con lo scopo di questa proposta |

Dipendenze **solo ipotetiche**: componenti standard Rust/Tauri per listener temporaneo, libreria TLS/pairing verificata e mantenuta, WebView/Browser mobile, eventuale discovery LAN; **nessuna dipendenza da aggiungere adesso**. Evitare mDNS automatico e discovery invasiva nella prima fase. Rischi principali: esfiltrazione in LAN, utilizzo come endpoint di file arbitrari, attacchi a decoder, privacy EXIF, blocco rete/firewall, UI poco comprensibile, codice difficile da mantenere su tre sistemi e un'esperienza solo apparentemente più semplice.

**Complessità stimata:** bassa per documentare i flussi OS; **alta** per un trasporto LAN realmente sicuro e mantenibile. Il vantaggio deve essere misurato, non dedotto.

## 8. Criteri di accettazione e test *per un'eventuale futura decisione*, non richieste di sviluppo

- **Scoperta del bisogno:** colloqui/prove con almeno 3–5 studi che usano iPhone/Android e Windows/macOS, confronto del tempo e degli errori con import multiplo dopo condivisione OS; se non c'è beneficio netto, **non sviluppare** il listener.
- **Sicurezza prima di una build con LAN:** threat model rivisto, prova MITM/replay/CSRF/DNS rebinding/hostile website/port scan, nessuna connessione in ingresso senza attivazione, peer estranei respinti, revoca immediata, test firewall e nessun TLS con avvisi ignorati.
- **Privacy:** telefono non può elencare media preesistenti, opere, clienti o vendite; nessun traffico verso Internet, nessun log di nomi/immagini/token; invio dei soli byte selezionati; gestione esplicita EXIF/GPS.
- **Integrità e resilienza:** 1/20 foto, duplicati byte-identici, file corrotti, JPEG/PNG/WebP con estensione falsa, HEIC e video respinti finché non supportati, upload parziale, disco pieno, crash, chiusura sessione, backup concorrente: il database e il Media Center restano coerenti, media precedenti invariati, temporanei identificabili ed eliminabili.
- **Portabilità e accessibilità:** stesso archivio ripristinabile senza il telefono e senza la LAN; nessuna nuova configurazione indispensabile; localizzazioni IT/EN/ES/FR/DE, uso tastiera e screen reader, messaggi di errore e stato leggibili.
- **Compatibilità:** test reali, non soltanto unitari, su Windows/macOS/Linux e iOS/Android nelle reti domestiche e reti con isolamento client; niente promessa di supporto su reti che impediscono il collegamento diretto.

## 9. Fasi possibili e domande aperte al proprietario

- **Fase 0 — solo ricerca:** raccogliere casi reali, documentare il flusso di import già disponibile e misurare se USB/condivisione OS risolve il problema; **decisione esplicita del proprietario** su eventuale prosecuzione.
- **Fase 1 — eventuale progettazione:** architettura di sicurezza e pairing, prototipo confinato a file sintetici su rete di prova **non distribuito**, analisi licenze/portabilità e revisione indipendente della superficie d'attacco. Nessuna modifica prevista da questa scheda.
- **Fase 2 — solo se approvata in futuro:** ricezione manuale e temporanea, file validati nel Media Center e chiusura verificabile; nessuna associazione automatica.
- **Fase 3 — opzionale:** comodità aggiuntive giustificate dai test (es. resume consapevole di file grandi) senza aggiungere cloud o accesso remoto all'archivio.

**Domande per revisione personale, tutte aperte:**

1. Quanto spesso il passaggio telefono → computer rallenta davvero la catalogazione rispetto ad AirDrop/Quick Share/USB?
2. Servono soltanto fotografie già scattate o l'acquisizione diretta dalla fotocamera del telefono? Quest'ultima richiede requisiti browser e sicurezza ulteriori.
3. È accettabile rendere il desktop temporaneamente raggiungibile in LAN o la semplicità e l'assenza di listener prevalgono?
4. Quale soglia di beneficio e quale verifica indipendente della sicurezza sarebbero necessarie per passare da P3 a P2?
5. Quali combinazioni OS, reti e telefoni hanno una domanda reale? Sono sufficienti JPEG/PNG/WebP finché TECH-027 resta non approvata?

**Conclusione non vincolante:** prima dimostrare che i flussi OS non bastano. Se una sessione LAN non può garantire trasporto cifrato, pairing robusto e isolamento dell'archivio, preferire **nessuna nuova funzione**. Questa scheda non autorizza implementazione, merge, modifica di dipendenze o esposizione di servizi in rete.
