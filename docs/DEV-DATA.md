# Dati di sviluppo separati dall'archivio reale

`npm run tauri:dev` non apre mai l'archivio della Repertica Studio installata. Il 27 novembre 2026 una build di sviluppo con lo stesso identificatore della release (`it.repertica.studio`) aveva aperto l'archivio reale e lo aveva portato allo schema 19; la release installata (0.1.2, schema massimo 18) si rifiutava poi di avviarsi ("Archivio creato da una versione più recente"). Da allora lo sviluppo usa un identificatore suo, e quindi una cartella dati sua.

## Dove stanno i dati

| | Release installata | `npm run tauri:dev` |
|---|---|---|
| Identificatore | `it.repertica.studio` | `it.repertica.studio.dev` |
| Archivio (Windows) | `%LOCALAPPDATA%\it.repertica.studio` | `%LOCALAPPDATA%\it.repertica.studio.dev` |
| Archivio (macOS) | `~/Library/Application Support/it.repertica.studio` | `~/Library/Application Support/it.repertica.studio.dev` |
| Archivio (Linux) | `~/.local/share/it.repertica.studio` | `~/.local/share/it.repertica.studio.dev` |
| Dati della finestra (preferenze, guida iniziale) | dentro la cartella dell'archivio | dentro la cartella dell'archivio di sviluppo |
| Cartella di sicurezza di un ripristino interrotto (accanto all'archivio) | `.repertica-studio-restore-rollback` | `.repertica-studio-restore-rollback-dev` |
| Archivio conservato dopo un ripristino (accanto all'archivio) | `ReperticaStudio-before-restore-…` | `ReperticaStudio-before-restore-dev-…` |
| Password della posta nel portachiavi di sistema | servizio `repertica-studio-mail` | servizio `repertica-studio-mail-dev` |
| Voce MCP scritta negli assistenti (Claude, Codex, VS Code…) | `repertica-studio` | `repertica-studio-dev` |
| Server MCP senza `--data-dir` | legge l'archivio della release | legge l'archivio di sviluppo |
| Titolo della finestra | Repertica Studio | Repertica Studio (dev) |
| Aggiornamenti automatici | sì (build firmate) | mai |
| Backup automatico a orario | sì, se impostato | solo con `REPERTICA_DEV_AUTO_BACKUP=1` |

Il collegamento ad Auctions non lascia niente da condividere: il token resta solo in memoria e il rientro dal browser usa ogni volta una porta locale casuale (`http://127.0.0.1:<porta>/repertica/oauth/callback`).

Il backup automatico è spento nelle build di sviluppo perché un archivio ripristinato lì contiene ancora la cartella di backup della proprietaria e l'elenco delle sue copie: il timer scriverebbe una copia dell'archivio di prova tra quelle vere e, con "conserva solo le ultime N", ne cancellerebbe una. Per provarlo apposta: `REPERTICA_DEV_AUTO_BACKUP=1` prima di `npm run tauri:dev`, con una cartella di destinazione di prova.

## Come funziona

- `src-tauri/tauri.dev.conf.json` contiene solo `"identifier": "it.repertica.studio.dev"`; lo script `tauri:dev` lo passa con `--config`. `tauri:build` e la pipeline di release (`--config src-tauri/tauri.release.conf.json`) lasciano `it.repertica.studio`.
- `src-tauri/build.rs` incide nel programma l'identificatore effettivo (`tauri.conf.json` più l'eventuale `--config`), e `src-tauri/src/identity.rs` ne ricava i nomi della tabella qui sopra. La release tiene tutti i nomi di sempre; ogni altro identificatore riceve un suffisso (`-dev`, `-e2e`…).
- Una build di debug con l'identificatore della release **non parte**: `npx tauri dev` senza `--config` o `cargo run` in `src-tauri` si fermano con un messaggio che rimanda a `npm run tauri:dev`. All'avvio l'app controlla anche che l'identificatore inciso coincida con quello usato da Tauri per la cartella dati.
- Per prove automatiche con un archivio usa-e-getta basta un altro identificatore, per esempio un file con `"identifier": "it.repertica.studio.e2e"` passato con `--config` (e, se serve, un'altra porta in `build.devUrl` e `build.beforeDevCommand`). Alla fine si cancella `%LOCALAPPDATA%\it.repertica.studio.e2e`.
- `cargo test` compila senza `--config` (identificatore della release) ma i test usano solo cartelle temporanee: non leggono né scrivono l'archivio reale.
- `src-tauri/src/identity.rs` e `tests/dev-identity.test.mjs` verificano tutto questo.

## Ripartire da zero

Chiudi la finestra di sviluppo e cancella la sua cartella (mai quella senza `.dev`):

```powershell
Remove-Item -Recurse -Force "$env:LOCALAPPDATA\it.repertica.studio.dev"
```

Al prossimo `npm run tauri:dev` l'archivio di sviluppo riparte vuoto.

## Provare con una copia dell'archivio reale

Non copiare a mano i file di `%LOCALAPPDATA%\it.repertica.studio` e non far puntare una build di sviluppo a quella cartella (niente `--data-dir` o `REPERTICA_DATA_DIR` verso di essa). La copia passa dal backup dell'app, che ne fa una fotografia coerente e verificata:

1. Nella **Repertica Studio installata**: Impostazioni → Backup del tuo archivio → crea un backup in una cartella di prova (per esempio `Documenti\Repertica-prove`). Nasce una sottocartella `ReperticaStudio-backup-…`.
2. Avvia `npm run tauri:dev` e, nella finestra "Repertica Studio (dev)", Impostazioni → Ripristina → scegli quella sottocartella. Viene sostituito solo l'archivio di sviluppo; quello di sviluppo precedente resta in `%LOCALAPPDATA%\ReperticaStudio-before-restore-dev-…`. Se la build di sviluppo ha uno schema più recente, aggiorna la copia: l'archivio reale resta com'è.
3. Chiudi la finestra di sviluppo e rendi sicura la copia con un qualsiasi strumento SQLite (per esempio DB Browser for SQLite) su `%LOCALAPPDATA%\it.repertica.studio.dev\repertica.sqlite`:

   ```sql
   -- La copia punta ancora alla cartella di backup della proprietaria e all'elenco delle sue copie.
   UPDATE backup_schedule SET enabled = 0, destination = NULL, last_directory = NULL;
   DELETE FROM auto_backup_log;
   -- Caselle di posta e bozze (le password non sono nei backup e non sono nel portachiavi di sviluppo).
   DELETE FROM mail_drafts;
   DELETE FROM mail_accounts;
   ```

4. Se la copia deve uscire dal tuo computer, togli anche i contatti dei collezionisti:

   ```sql
   UPDATE collectors SET
     name = 'Collezionista ' || substr(id, 1, 8),
     first_name = NULL, last_name = NULL, email = NULL, phone = NULL, address = NULL, notes = NULL,
     birth_date = NULL, birth_time = NULL, birth_place = NULL,
     instagram = NULL, facebook = NULL, tiktok = NULL, website = NULL, linkedin = NULL;
   ```

   Le fotografie in `media/`, le note delle opere e i testi dei certificati restano: valuta se è il caso di condividerli.
5. Cancella la cartella di backup del punto 1 quando non serve più: i backup sono in chiaro.
