# REPS-UX-057 — Opere digitali: file costitutivi, versioni di presentazione e istruzioni di fruizione

- **ID stabile:** REPS-UX-057
- **Data della ricognizione:** 2026-11-23
- **Stato:** Proposta — da valutare
- **Filone:** Gestione e UX
- **Priorità ipotetica:** P3 per il pubblico generale; P2 **solo** se artisti digitali, gallerie o archivi confermano casi ricorrenti
- **Natura:** esclusivamente studio documentale. Non autorizza implementazioni, migrazioni, approvazioni, issue, assegnazioni o merge.

## 1. Problema, destinatari e riscontri dal repository

**Problema professionale.** Per una videoinstallazione, un'animazione generativa, un'opera sonora o una stampa nata da un file digitale, la fotografia di catalogo **non è necessariamente l'opera**. I professionisti devono poter distinguere il file *costitutivo* da un'anteprima, sapere quale versione è stata consegnata/esposta e conservare istruzioni su come ripresentarla, senza presumere che il semplice possesso dei byte certifichi autenticità, diritti o riproducibilità futura.

**Possibili destinatari:** artisti che creano opere digitali, piccole gallerie, studi di produzione, archivi di new media e curatori con installazioni audiovisive. **Non** è necessario complicare le schede dei pittori che archiviano soltanto opere fisiche.

**Fatti verificati su `main` nella ricognizione del 23/11/2026:** [`README.md`](../../../README.md), [`docs/MVP.md`](../../MVP.md), [`src/app/models.ts`](../../../src/app/models.ts) e [`src-tauri/src/models.rs`](../../../src-tauri/src/models.rs) modellano un'opera con metadati, fotografia principale e galleria fotografica; [`src-tauri/src/raw_files.rs`](../../../src-tauri/src/raw_files.rs) associa **un RAW fotografico per opera**, limitato a 1 GiB e a formati riconosciuti come RAW di fotocamera, con copia locale e SHA-256. La [portabilità corrente](../../DATA-PORTABILITY.md) esporta i metadati RAW e le immagini per registri CSV, mentre il backup completo include i file effettivi. **Questo non equivale a supporto per master audiovisivi, file eseguibili d'opera o versioni espositive**. Nessuna richiesta diretta di utilizzatori è stata verificata e non è stato eseguito un collaudo desktop: opportunità da testare, non bug.

### Differenze dalle schede esistenti, senza doppioni

- [UX-022 — Componenti fisici](REPS-UX-022-opere-composte-componenti-fisici.md): inventaria pezzi **materiali** di un'unica opera (struttura, alimentatore, pannelli). Qui si studiano *ruoli, versioni e condizioni di uso dei file costitutivi*; un'installazione mista potrebbe collegare i due registri in futuro, senza duplicarli.
- [UX-025 — Istruzioni per un'opera](REPS-UX-025-istruzioni-allestimento-cura-opera.md): istruzioni generali di montaggio/cura, anche versionate. Qui si isola il problema specifico **master digitale ↔ copia espositiva ↔ requisiti di riproduzione**, non si riprogetta il modulo universale di istruzioni.
- [UX-005 — Edizioni e multipli](REPS-UX-005-edizioni-multipli-serie-numerate.md): esemplari commercialmente distinti; **una revisione del file master non crea un nuovo esemplare, un'edizione o una vendita**.
- [UX-010 — Diritti di riproduzione](REPS-UX-010-diritti-riproduzione-crediti-autorizzazioni.md): titolarità e autorizzazioni dichiarate; la presenza di un file non concede alcuna licenza e non prova il diritto a esporre o duplicare.
- [UX-004 — Conservazione](REPS-UX-004-rapporti-stato-conservazione.md): rilievi e interventi documentati; un checksum identico non misura la qualità artistica né la corretta esecuzione audiovisiva.
- [TECH-016 — Allegati multi-entità](https://github.com/Cesare-ml/Repertica-Studio/blob/docs/idee-strumenti-architettura-20261121/docs/idee/strumenti-e-architettura/REPS-TECH-016-allegati-documentali-locali-multi-entita.md), [TECH-007 — Media esterni](https://github.com/Cesare-ml/Repertica-Studio/blob/docs/idee-strumenti-architettura-20261121/docs/idee/strumenti-e-architettura/REPS-TECH-007-archivio-media-esterno-rilocalizzabile.md), [TECH-046 — Master TIFF](https://github.com/Cesare-ml/Repertica-Studio/blob/docs/idee-strumenti-architettura-20261121/docs/idee/strumenti-e-architettura/REPS-TECH-046-master-fotografici-tiff-bigtiff-offline.md) e [TECH-001 — Pacchetto aperto](https://github.com/Cesare-ml/Repertica-Studio/blob/docs/idee-strumenti-architettura-20261121/docs/idee/strumenti-e-architettura/REPS-TECH-001-pacchetto-dati-aperto.md) riguardano meccanismi trasversali di file e interscambio. **Questa scheda definisce esclusivamente il significato professionale e il flusso UX**; nessuna specifica di storage già proposta viene riscritta, nessun file TECH modificato.

## 2. Scenari reali da verificare con professionisti

1. **Videoarte:** l'autrice cataloga una videoinstallazione. Associa il master originale, una versione compressa per esposizione e una foto rappresentativa per la lista delle opere. Registra durata, rapporto d'aspetto, necessità di riproduzione in loop e il fatto che le specifiche siano *dichiarate*, non collaudate dall'app. Consegnare una copia non sostituisce né altera il master.
2. **Installazione audio-visiva:** un museo deve predisporre due schermi, diffusori e un player compatibile. L'archivista conserva una checklist di presentazione e la versione del file effettivamente concordata. L'hardware fisico può appartenere a UX-022; la checklist tecnica **non certifica** che un museo l'abbia eseguita.
3. **Opera generativa:** l'opera richiede file sorgenti, ambiente software e font/plugin specifici. L'utente dichiara questi requisiti, senza caricare o avviare codice dentro Repertica, senza includere credenziali o sostenere che un checksum garantisca l'eseguibilità negli anni.
4. **Ristampa da file digitale:** master e derivato per stampa sono distinti; colore, dimensioni e formato annotati manualmente sono un'indicazione di lavoro, non profilo ICC controllato né licenza di riproduzione. La tiratura fisica, se presente, appartiene a UX-005.
5. **Archivio storico incompleto:** esiste solo una fotografia o una copia di visione, non il master. Il sistema deve dire **«master non registrato / sconosciuto»**, non «opera completa» né «file perso».

## 3. Flussi UX candidati

**Ingresso discreto.** Da `Opera → Materiale costitutivo digitale` (sezione facoltativa, nascosta per default nelle opere tradizionali): una breve scelta «Questa opera comprende file necessari alla sua esistenza o presentazione?» con spiegazione chiara della differenza fra *fotografie documentali*, *file dell'opera*, *documenti allegati* e *RAW fotografici*. Nessuna conversione automatica delle vecchie fotografie in master.

**Registrazione delle versioni.** `Aggiungi file o riferimento` → scegliere *master dichiarato*, *copia di esposizione*, *sorgente*, *materiale necessario* o *altro*; descrivere il ruolo in testo accessibile, con data/fonte e versione. Se l'archiviazione gestita è supportata in una fase futura, selezionare un file locale tramite dialogo OS, mostrare nome, dimensione, estensione e verifica d'integrità. Altrimenti **solo metadati** con avvertenza esplicita «il file non è nell'archivio e non entrerà nel backup».

**Istruzioni di presentazione.** Scheda corta: durata e loop facoltativi, numero e disposizione dei display/audio (testo), formato/risoluzione *dichiarati*, requisiti hardware/software, fonte, data e note. Etichettare ogni campo «dichiarato / verificato manualmente in data / da verificare». Nessuna analisi automatica dei codec, riproduzione di file, esecuzione di progetti, installazione di plugin o conversione imposta.

**Pacchetto di preparazione alla consegna, soltanto eventuale.** Scelta deliberata della versione e dei testi da condividere; anteprima con **foto rappresentativa vs file effettivi** e avvisi per file mancanti, link esterni o licenze/diritti non verificati. Lista di consegna tecnica esportabile offline; non sostituisce atto di vendita, certificato, ricevuta, licenza o accordo di prestito. La condivisione dei byte richiede un comando esplicito diverso da «salva scheda».

**Revisione e rettifica.** Nuovo master o nuova copia espositiva produce una **nuova versione identificabile** collegata alla precedente; non sovrascrivere l'unico originale, non aggiornare certificati già emessi, vendite o dichiarazioni storiche. L'utente decide quale versione indicare come «corrente per presentazione» senza cancellare la storia.

**Accessibilità:** ogni file è identificabile con nome + ruolo + versione + disponibilità, mai con colore o icona soltanto; ordine tastiera e screen reader, focus su errori, etichette leggibili al 200% di zoom, contrasto e supporto finestre strette, nessuna timeline drag-only.

## 4. Dati ipotetici (soltanto descrittivi, non schema/API approvati)

- `artwork_digital_asset`: UUID stabile, `artwork_id` UUID, ruolo codificato, `version_label` libera, descrizione, data e precisione, provenienza/fonte dichiarata, stato `managed_copy|external_reference|metadata_only|missing`, nome originario, formato/MIME **se noto e dichiarato**, byte e SHA-256 **solo se calcolati sul file reale**, percorso relativo sicuro **solo se gestito**, riferimento esterno non eseguibile **solo se scelto**. Referenze esterne e descrizioni non sono prove di presenza.
- `digital_asset_relation`: UUID, derivato → master/sorgente con tipo `derived_from|alternative_of|replaces`, data e motivazione; impedire cicli semplici senza attribuire equivalenza di contenuto.
- `digital_presentation_profile`: UUID, opera, versione del profilo, campi facoltativi su durata, loop, aspect ratio, schermi/audio, requisiti software/hardware, note di calibrazione, fonte, data della verifica manuale ed eventuale link alla versione file suggerita. La scheda descrive **requisiti dichiarati**, non capacità testate dall'app.
- `digital_handover_note` **solo se validata**: UUID, opera, data, elenco degli asset/versioni **selezionati esplicitamente**, testo condiviso e fonte; non conservare per default recapiti sensibili, chiavi, password, numero di licenze o termini giuridici non verificati.
- Nomi di API puramente illustrativi: «elenca versioni digitali di un'opera», «confronta schede/versioni», «prepara anteprima di consegna». Non sono endpoint operativi, non abilitano accesso remoto.
- **Invarianti:** la scheda opera conserva lo stesso UUID e lo stesso stato commerciale; un master non crea vendita, certificato, quota finanziaria o componente fisico; un hash dimostra soltanto corrispondenza dei byte misurati, non autenticità artistica o fruibilità. In caso di file sconosciuto/mancante visualizzare stato incerto, mai conferma implicita.

## 5. Impostazioni, offline, lingue, privacy e portabilità

- **Impostazioni:** attivazione facoltativa del pannello digitale per archivio o opera, ruolo preferito non obbligatorio, eventuale scelta tra copia gestita e *solo metadati* con avvisi non disattivabili sulla portabilità; **nessun limite di spazio o codec inventato** prima di verificare requisiti tecnici e volumi reali. Non aggiungere onboarding obbligatorio.
- **Offline:** tutta la catalogazione e le checklist in SQLite locale, Windows/macOS/Linux, senza account, abbonamenti, telemetria, server, player web, DRM, streaming, cloud o software di terze parti necessario per aprire la scheda. La riproduzione effettiva resta esterna e facoltativa. Nessun file eseguibile viene avviato automaticamente.
- **Multilingua:** lingua di sistema (IT/EN/ES/FR/DE se supportata), selezione manuale già prevista, fallback inglese per altre lingue e testi mancanti. Ruoli e stati sono codici invarianti; nomi file, titoli, descrizioni, licenze e istruzioni dell'utente rimangono **nel testo originale**, mai tradotti o normalizzati distruttivamente.
- **Privacy e sicurezza:** opere inedite, file sorgenti, specifiche installative e licenze possono essere riservati. Non includerli negli export promozionali di default, né renderli leggibili tramite MCP senza valutazione di permessi separata. I riferimenti esterni non vengono aperti/contattati senza gesto dell'utente; file e progetti sconosciuti sono **dati non attendibili**, non codice da eseguire. Avviso che i backup correnti sono in chiaro, specialmente su NAS e cartelle condivise.
- **Backup completo:** **se un master è dichiarato «gestito»**, deve essere compreso nella copia verificata e ripristinabile su altro computer con hash e percorsi relativi, o il backup va dichiarato incompleto/fallito: non promettere il ripristino di dati non salvati. Per un asset di tipo `external_reference` il backup conserva **solo il riferimento/metadato**, non il file: mostrare questa limitazione in modo prominente prima dell'export. La cartella destinazione resta liberamente scelta. Se non è possibile assicurare autosufficienza, mantenere la proposta alla fase metadati senza fingere portabilità.
- **Import/export:** scheda descrittiva UTF-8/CSV con UUID, ruolo, versione, stato di custodia, fonte e specifiche dichiarate; pacchetto futuro **solo su scelta esplicita**, con JSON versionato e manifest dei byte effettivamente inclusi, checksum ricalcolato, nomi non eseguibili e percorsi relativi. Import con anteprima, mapping UUID, rilevazione di duplicati e versioni, validazione dei manifest e rollback; non sovrascrivere master o certificati, non recuperare automaticamente URL remoti. Un CSV da solo non è backup.
- **Compatibilità:** archivi storici senza asset digitali continuano a funzionare; nessuna migrazione attribuisce significato artistico a `image_path` o a `artwork_raw_files`. Eventuale estensione additiva richiede versione schema e collaudo del backup; app vecchie potrebbero non aprire schemi futuri. Le esportazioni CSV esistenti, UUID vendita/opera e snapshot dei certificati restano inalterati.

## 6. Dipendenze, complessità, rischi e alternative

**Dipendenze concettuali:** opere UUID, i18n, backup e media locale esistenti. Un eventuale deposito robusto di file eterogenei/grandi richiederebbe una **valutazione tecnica separata** con l'altro filone (TECH-016/007/001), senza modificarlo o presumere che sia implementato. UX-022/025/005/010 sono collegamenti possibili, non prerequisiti di una scheda minima.

**Complessità:** bassa-media per una scheda testuale con ruoli/istruzioni; media per versioni e controllo dei riferimenti; alta per immagazzinamento di gigabyte, verifica atomica di grandi file, backup multi-volume, import verificabile e compatibilità dei formati; **molto alta e fuori ambito** per player, emulazione, esecuzione di opere o preservazione software garantita.

**Rischi e mitigazioni:** un fotografo potrebbe chiamare «master» la foto di catalogo → guida semantica; checksum confuso con autenticità → etichette esplicite; file grande non incluso nel backup → preflight e fallimento/avviso, non falso successo; sostituzione accidentale del file originale → versioni non distruttive; diffusione di sorgenti o condizioni di licenza → preview e esclusione per default; uso di file eseguibili malevoli → nessuna esecuzione/preview attiva; eccessivo carico UX per artisti tradizionali → funzione opt-in.

**Alternative preferibili da verificare prima dello sviluppo:** cartella locale per opera con master e documento `.md`/ODT di requisiti, più codice opera nella nomenclatura; oppure semplice campo descrittivo «materiale digitale e istruzioni» nell'attuale scheda con file esterni **esplicitamente non gestiti**. Un archivio di file di grandi dimensioni potrebbe essere mantenuto in un filesystem già amministrato dall'istituzione, usando Repertica soltanto come indice: questa scelta non deve essere venduta come backup autosufficiente.

## 7. Criteri di accettazione ipotetici

1. Una scheda d'arte tradizionale resta invariata e non richiede campi digitali. Una videoopera distingue **foto di catalogo**, master, versione espositiva e versioni precedenti.
2. Registrare un file o una checklist non cambia vendite, prezzo, disponibilità, quota di guadagni, certificati o numero di edizione.
3. Dove un master è assente, lo stato mostrato è «non registrato/sconosciuto», non «conservato»; dove esiste solo un riferimento esterno, il backup **non** ne dichiara inclusi i byte.
4. Una versione nuova non sovrascrive il master precedente né alterazioni storiche della scheda; fonti e verifiche manuali sono distinguibili da dati automatici.
5. Esportazioni condivisibili mostrano in anteprima i nomi e le istruzioni, escludendo per default codice sorgente, file costitutivi e note/recapiti privati; nessun avvio/upload automatico.
6. Se la fase file gestiti è approvata: backup su cartella liberamente scelta e ripristino offline su altra installazione conservano hash, byte, ruoli e versioni; errori di spazio/interruzioni **non** producono copie dichiarate complete.
7. Import ripetuto non duplica UUID, modifica certificati o esegue file; verifiche e testi accessibili in IT/EN/ES/FR/DE con fallback inglese e per tastiera/screen reader.
8. Un file integro ma non più riproducibile viene descritto come **integro per byte ma non verificato per fruizione**; nessuna garanzia automatica di longevità.

## 8. Fasi esplorative, nessuna autorizzata

- **0 — Validazione:** raccogliere 4–6 casi anonimizzati (video, installazione, opera software, digitale per stampa) da professionisti; chiedere che cosa chiamano «originale», cosa devono consegnare e quali file realmente includono nei backup. Confrontare la cartella locale + ODT/Markdown come alternativa.
- **A — Solo metadati, eventuale:** ruoli master/derivato e requisiti di presentazione dichiarati, avviso «file non in backup»; nessuna nuova libreria file, playback o upload.
- **B — Versioni e checklist, solo se utili:** relazioni fra versioni, scheda di consegna e filtri sui dati mancanti; privacy e accessibilità verificate.
- **C — File gestiti, solo con requisito provato e progetto tecnico indipendente:** copia locale, checksum, backup/ripristino integrale, pacchetto d'interscambio versionato; non è compresa nella semplice approvazione della scheda UX.

## 9. Certi, ipotesi e domande al proprietario

- **Requisiti certi del prodotto:** desktop open source gratuito, offline-first, nessun account o abbonamento, dati locali e sempre esportabili, backup/ripristino in cartella scelta, lingue di sistema con fallback inglese, conservazione dei documenti storici. **Fatti del codice:** foto/opere e un RAW fotografico opzionale, non modello specifico di master audiovisivo.
- **Ipotesi non validate:** esiste un gruppo significativo di utenti che cataloga opere *born-digital* e preferisce una scheda integrata invece di cartelle e documenti separati; le versioni espositive vengono cambiate abbastanza spesso da giustificare il registro.
- **Domande da decidere:** Repertica intende servire artisti digitali/videoarte già nel prodotto generale? Basta *descrivere* master e istruzioni o ci si aspetta di *custodire i byte*? Qual è la dimensione tipica di un master (GB/TB)? È accettabile che alcuni asset restino esterni con avviso di backup parziale, oppure è inderogabile una copia gestita? Quali informazioni servono al destinatario senza divulgare sorgenti e licenze? Le opere software sono nel perimetro oppure vanno escluse da questa fase?

**Esito invariato: Proposta — da valutare. Nessuna approvazione o assegnazione allo sviluppo.**
