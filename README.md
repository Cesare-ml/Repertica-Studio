# Repertica Studio

**Il tuo archivio. Le tue opere. Sempre tuoi.**

Gestionale artistico **gratuito, open source e locale**, realizzato con Angular 20, Tauri 2, Rust e SQLite. Nessun account, abbonamento o server necessari.

> Stato: MVP 0.1.0 in sviluppo. Non utilizzare ancora come unica copia di archivi reali. Le funzioni mancanti e le modalità di collaudo sono documentate in [docs/MVP.md](docs/MVP.md).

## Funzionalità MVP

- **Opere**: archivio con codice, titolo, tecnica, anno, misure, stato, ubicazione, descrizione e immagine principale. Ricerca anche per supporto, descrizione e nome dell'acquirente, filtri per stato e collezionista, ordinamento per titolo o anno.
- **Collezionisti**: anagrafica privata e note. Dalla lista si possono consultare le opere acquistate e aprirle nell'archivio; dalle opere vendute si può raggiungere il relativo collezionista.
- **Vendite**: registrazione importi in centesimi, cliente e opera associati; una vendita per opera nella prima versione.
- **Certificati**: registro immutabile con numero progressivo univoco, fotografia della scheda al momento dell'emissione e stampa/salvataggio PDF tramite dialogo di stampa.
- **Offline**: SQLite incorporato e fotografie nella cartella privata dell'applicazione.
- **Backup e ripristino**: selezione di qualsiasi cartella accessibile (anche disco esterno/NAS), backup SQLite e media con manifest SHA-256, verifica e ripristino con conservazione dell'archivio precedente.
- **Aggiornamenti firmati**: architettura predisposta; da abilitare solo quando verrà configurata una chiave di firma e pubblicate release compatibili.

Non include **Sites**, login, sincronizzazione cloud, fatturazione fiscale, aste o invio dati a terzi.

## Avvio sviluppo

Requisiti: Node.js 22+, Rust stabile, prerequisiti di sistema di [Tauri 2](https://v2.tauri.app/start/prerequisites/). Su Windows: Microsoft C++ Build Tools e WebView2.

```bash
npm install
npm run tauri:dev
```

Esegui `npm run test:unit` per i test locali di struttura e filtri dell'archivio (Node.js 22+).

Per verificare il solo frontend: `npm run build`. `npm start` da browser non può accedere all'archivio nativo e mostra intenzionalmente un messaggio di avvio Tauri.

Per creare l'app di produzione: `npm run tauri:build`. Sono previsti controlli CI su GitHub Actions.

## Archivio e privacy

Il database `repertica.sqlite` viene creato automaticamente nella cartella app-local-data di Repertica Studio. Le immagini vengono salvate nella sottocartella `media/`. È possibile visualizzare il percorso in **Impostazioni**. Le operazioni ordinarie non richiedono Internet.

**Backup**: da **Impostazioni → Backup del tuo archivio**, scegli una cartella locale, un'unità USB, un percorso NAS raggiungibile o persino una cartella che contiene altri archivi. L'app crea sempre una sottocartella nuova `ReperticaStudio-backup-...` con `repertica.sqlite`, `media/` e `manifest.json`. Il database è acquisito con `VACUUM INTO` per ottenere uno snapshot coerente anche con journaling WAL. Tutti i file vengono verificati con SHA-256; non serve copiare manualmente SQLite.

**Ripristino**: seleziona una singola sottocartella di backup. Prima di confermare, l'app verifica hash, integrità SQLite e riferimenti delle immagini. Durante il ripristino una copia integrale dell'archivio precedente viene conservata nella cartella padre di quella attiva. Il ripristino **sostituisce** l'archivio in uso, non fonde due archivi diversi.

**Privacy**: i backup sono in chiaro (non cifrati), possono contenere dati personali e vanno conservati su un dispositivo protetto. Non essendo ancora collaudate le build native Windows e macOS, non utilizzare questa MVP come unica copia di dati importanti.

## Architettura e provenienza

La logica è stata progettata prendendo come riferimento i domini del repository privato `Gestionale-Opere` (opere, clienti, vendite, certificati) e le pratiche native di `virgostars-desktop`. Questa implementazione è indipendente, non contiene dati personali o credenziali dei repository sorgenti e non richiede PostgreSQL.

- `src/`: interfaccia Angular standalone e comando IPC tipizzato.
- `src-tauri/src/db.rs`: schema SQLite e inizializzazione.
- `src-tauri/src/commands.rs`: operazioni native e controlli di integrità.
- `docs/MVP.md`: criteri, vincoli e roadmap.
- `.github/workflows/ci.yml`: verifica TypeScript e Rust.

## Licenza

**CPAL-1.0**, come VirgoStars Ephemeris; il testo integrale è in [LICENSE](LICENSE), incluse le condizioni di attribuzione a Repertica. I marchi non sono ceduti con la licenza del software.

Contributi e segnalazioni sono benvenuti tramite pull request e GitHub Issues.

## Build per macOS tramite GitHub Actions

Il workflow [macOS builds (Intel + Apple Silicon)](.github/workflows/build-macos.yml) genera file `.dmg` su runner GitHub macOS, senza richiedere un Mac locale.

1. Apri **Actions → macOS builds (Intel + Apple Silicon) → Run workflow**, selezionando `main`.
2. Al termine, apri l'esecuzione e scarica gli artifact `repertica-macos-Apple-Silicon-...` e/o `repertica-macos-Intel-...`.
3. Ogni artifact include il `.dmg` e il file SHA-256 per verificarlo.

Il workflow esegue anche compilazioni sul branch `main` quando il codice dell'app cambia. I DMG sono **build di prova con firma ad hoc**, non notarizzate da Apple. macOS Gatekeeper potrebbe impedire l'apertura o mostrare avvisi: non presentarle come release pubbliche verificate. Nessuna release GitHub o aggiornamento automatico viene pubblicato da questa Action. Per una distribuzione ufficiale senza avvisi occorreranno Developer ID, notarizzazione e test su dispositivi reali.
