# Repertica Studio

**Il tuo archivio. Le tue opere. Sempre tuoi.**

Gestionale artistico **gratuito, open source e locale**, realizzato con Angular 20, Tauri 2, Rust e SQLite. Nessun account, abbonamento o server necessari.

> Stato: MVP 0.1.0 in sviluppo. Non utilizzare ancora come unica copia di archivi reali. Le funzioni mancanti e le modalità di collaudo sono documentate in [docs/MVP.md](docs/MVP.md).

## Funzionalità MVP

- **Opere**: archivio con codice, titolo, tecnica, anno, misure, stato, ubicazione, descrizione e immagine principale.
- **Collezionisti**: anagrafica privata e note.
- **Vendite**: registrazione importi in centesimi, cliente e opera associati; una vendita per opera nella prima versione.
- **Certificati**: registro immutabile con numero progressivo univoco, fotografia della scheda al momento dell'emissione e stampa/salvataggio PDF tramite dialogo di stampa.
- **Offline**: SQLite incorporato e fotografie nella cartella privata dell'applicazione.
- **Aggiornamenti firmati**: architettura predisposta; da abilitare solo quando verrà configurata una chiave di firma e pubblicate release compatibili.

Non include **Sites**, login, sincronizzazione cloud, fatturazione fiscale, aste o invio dati a terzi.

## Avvio sviluppo

Requisiti: Node.js 22+, Rust stabile, prerequisiti di sistema di [Tauri 2](https://v2.tauri.app/start/prerequisites/). Su Windows: Microsoft C++ Build Tools e WebView2.

```bash
npm install
npm run tauri:dev
```

Per verificare il solo frontend: `npm run build`. `npm start` da browser non può accedere all'archivio nativo e mostra intenzionalmente un messaggio di avvio Tauri.

Per creare l'app di produzione: `npm run tauri:build`. Sono previsti controlli CI su GitHub Actions.

## Archivio e privacy

Il database `repertica.sqlite` viene creato automaticamente nella cartella app-local-data di Repertica Studio. Le immagini vengono salvate nella sottocartella `media/`. È possibile visualizzare il percorso in **Impostazioni**. Le operazioni ordinarie non richiedono Internet.

**Avvertenza sui backup**: il backup/ripristino completo e la migrazione dell'archivio storico non sono ancora implementati. Conservare copie esterne prima di prove con dati importanti. Non copiare soltanto il file SQLite mentre il programma è in esecuzione: è utilizzato il journaling WAL.

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
