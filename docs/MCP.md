# Server MCP locale (sola lettura)

Repertica Studio include un server [Model Context Protocol](https://modelcontextprotocol.io) per permettere a un assistente IA **installato sul tuo computer** di consultare l'archivio. È pensato per domande come «quali opere ad acquerello sono disponibili?» o «riassumi le vendite del 2026».

## Garanzie

- **Solo STDIO.** Nessuna porta di rete, nessun server HTTP, nessuna connessione in uscita: l'assistente avvia il processo e dialoga con standard input/output.
- **Sola lettura.** L'archivio SQLite è aperto con `SQLITE_OPEN_READ_ONLY` e `PRAGMA query_only`; nessuno strumento scrive, e ogni scrittura verrebbe rifiutata dal database stesso.
- **Dati privati esclusi.** Non vengono esposti email, telefoni, indirizzi, note dei collezionisti né le fotografie (nemmeno i percorsi dei file). Sono esposte le note descrittive delle opere (troncate negli elenchi), i nomi dei collezionisti, gli importi e i certificati.
- **Nessuna scrittura su disco** e nessun log: i messaggi diagnostici vanno solo su standard error.
- Funziona mentre l'app è aperta (il database usa WAL), senza bloccarla.

> **Attenzione alla privacy.** Repertica non invia nulla online, ma ciò che l'assistente legge può essere inviato al fornitore del modello (per esempio nella conversazione). Usa soltanto assistenti di cui ti fidi e ricorda che nomi, importi e descrizioni sono dati personali e commerciali.

## Configurazione

**Da Impostazioni, senza terminale.** In **Impostazioni → Assistenti IA locali (MCP)** l'app riconosce gli assistenti installati e mostra per ciascuno lo stato del collegamento (non trovato, non collegato, collegato, da aggiornare). Il pulsante **Collega** mostra il file che verrà modificato, chiede conferma, aggiunge soltanto la voce `repertica-studio` e conserva una copia del file originale (`<file>.repertica-backup`). **Scollega** rimuove solo quella voce. Dopo ogni modifica va riavviato l'assistente.

| Assistente | File modificato | Formato |
|---|---|---|
| Claude Desktop | `%APPDATA%\Claude\claude_desktop_config.json` (anche la versione Microsoft Store) | JSON, sezione `mcpServers` |
| Codex (OpenAI) | `~/.codex/config.toml` (o `$CODEX_HOME`) | TOML, tabella `[mcp_servers.repertica-studio]` |
| Cursor | `~/.cursor/mcp.json` | JSON, `mcpServers` |
| Windsurf | `~/.codeium/windsurf/mcp_config.json` | JSON, `mcpServers` |
| Gemini CLI | `~/.gemini/settings.json` | JSON, `mcpServers` |
| Visual Studio Code | `%APPDATA%\Code\User\mcp.json` | JSON, sezione `servers` con `"type": "stdio"` |

Il collegamento non tocca mai le altre impostazioni o gli altri server MCP: nei file JSON l'ordine delle chiavi è conservato, in `config.toml` anche commenti, formattazione e le altre tabelle. Un file che non si può leggere (JSON o TOML non valido, sezione di tipo inatteso) viene lasciato intatto e l'errore è mostrato accanto all'assistente. Se l'app viene spostata, lo stato diventa «da aggiornare» e basta un clic su **Aggiorna il collegamento**.

**ChatGPT.** L'app e il sito di ChatGPT si collegano soltanto a server MCP online (HTTP) e non possono usare un server locale su STDIO: per i prodotti OpenAI si usa Codex. Claude Code non è configurato automaticamente (il suo file contiene molte altre impostazioni): si collega con il comando mostrato nella configurazione manuale.

**Altri assistenti.** Nella stessa sezione, **Configurazione manuale per altri assistenti** mostra, con il percorso corretto dell'eseguibile e dell'archivio e un pulsante per copiare, il JSON, la tabella TOML per Codex e i comandi `codex mcp add` / `claude mcp add`. Esempio JSON (Windows):

```json
{
  "mcpServers": {
    "repertica-studio": {
      "command": "C:\\Program Files\\Repertica Studio\\Repertica Studio.exe",
      "args": ["--mcp", "--data-dir", "C:\\Users\\Anna\\AppData\\Local\\it.repertica.studio"]
    }
  }
}
```

Dalla riga di comando (uso avanzato): `"Repertica Studio" --mcp [--data-dir <cartella>]`. Senza `--data-dir` si usa la variabile `REPERTICA_DATA_DIR` e, in mancanza, la cartella standard dell'app (`%LOCALAPPDATA%\it.repertica.studio`, `~/Library/Application Support/it.repertica.studio`, `~/.local/share/it.repertica.studio`).

Se l'archivio non esiste o ha uno schema diverso da quello dell'app (apri prima l'app per aggiornarlo), il processo termina con un messaggio su standard error e codice di uscita 1.

## Strumenti

| Strumento | Descrizione |
|---|---|
| `archive_overview` | Conteggi (opere per stato, collezionisti, vendite, certificati) e totali venduto, incassato e da incassare. |
| `search_artworks` | Ricerca per parole, senza distinzione di maiuscole e accenti, in codice, titolo, anno, tecnica, supporto, descrizione e acquirente. `"frasi tra virgolette"` = parole consecutive (anche virgolette tipografiche; una frase finale non chiusa rimane un termine di ricerca). Filtri `status` e `limit` (max 50). |
| `get_artwork` | Scheda completa per `id` o `code`, con vendita, saldo, certificati, numero di fotografie e caratteristiche (`signed`, `final_varnish_applied`, `framed`). |
| `list_collectors` | Nomi con numero e totale degli acquisti; `query` cerca anche codice, titolo, tecnica, supporto, anno, descrizione, ubicazione e dimensioni delle **sole opere acquistate** dal collezionista; `limit` (max 200). Non restituisce contatti né descrizioni delle opere. |
| `list_sales` | Vendite dalla più recente con incassato e residuo; filtro `year`, `limit` (max 200). |
| `list_certificates` | Certificati emessi (seriale, data, opera, artista). |

Gli importi sono in centesimi (`*_cents`, interi esatti) e in testo (`*_eur`, per esempio `1500.50`). Tutti gli strumenti dichiarano `readOnlyHint: true`.

Versioni di protocollo supportate: 2025-06-18, 2025-03-26 e 2024-11-05 (JSON-RPC 2.0, un messaggio per riga; i batch non sono supportati).

## Verifica

`cargo test` esegue test di protocollo, ricerca, privacy dei contatti, sola lettura a livello di database e due test end-to-end che avviano l'eseguibile vero con `--mcp` e dialogano tramite pipe.

## Limiti

- Lo schema supportato è quello corrente (quello scritto dalla versione corrente dell'app); un archivio più vecchio va prima aperto con l'app.
- Il server non aggiorna i dati in tempo reale: ogni richiesta legge lo stato corrente dell'archivio.
- Nessuna autenticazione: chi può avviare il programma sul tuo computer, con il tuo utente, può leggere l'archivio, come già con il file SQLite.
