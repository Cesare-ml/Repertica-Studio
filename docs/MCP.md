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

**Da Impostazioni, senza terminale.** In **Impostazioni → Assistenti IA locali (MCP)** l'app riconosce Claude Desktop e mostra lo stato del collegamento (non collegato, collegato, da aggiornare). Il pulsante **Collega a Claude Desktop** mostra il file che verrà modificato, chiede conferma, aggiunge soltanto la voce `repertica-studio` e conserva una copia del file originale (`claude_desktop_config.json.repertica-backup`). **Scollega** rimuove solo quella voce. Dopo ogni modifica va riavviato Claude Desktop.

Il collegamento non tocca mai le altre impostazioni o gli altri server MCP del file; un file che non è JSON valido viene lasciato intatto con un messaggio di errore. Se l'app viene spostata, lo stato diventa «da aggiornare» e basta un clic su **Aggiorna il collegamento**. Supporta la posizione classica (`%APPDATA%\Claude`) e quella della versione Microsoft Store di Claude Desktop.

**Altri assistenti.** Nella stessa sezione, **Configurazione manuale per altri assistenti** mostra il JSON pronto con il percorso corretto dell'eseguibile e dell'archivio e un pulsante per copiarlo (esempio Windows):

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
| `search_artworks` | Ricerca per parole, senza distinzione di maiuscole e accenti, in codice, titolo, anno, tecnica, supporto, descrizione e acquirente. `"frasi tra virgolette"` = parole consecutive. Filtri `status` e `limit` (max 50). |
| `get_artwork` | Scheda completa per `id` o `code`, con vendita, saldo, certificati e numero di fotografie. |
| `list_collectors` | Nomi con numero e totale degli acquisti; filtro `query`, `limit` (max 200). Nessun contatto. |
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
