# REPS-TECH-035 — Configurazioni MCP di client esterni: conflitti, ripristino e scritture prudenti

- **ID stabile:** REPS-TECH-035
- **Data della ricognizione:** 2026-11-22
- **Stato:** **Proposta — da valutare**
- **Priorità proposta:** **P2 condizionata**; P3 se l'uso effettivo è raro e le istruzioni manuali bastano. **Nessuna approvazione, assegnazione o avvio di sviluppo.**
- **Ambito:** soltanto il collegamento/disconnessione **volontario** del server MCP **già presente** a file di configurazione *di software terzo*. Non riguarda l'autorizzazione alla lettura dei dati MCP, il contenuto dei tool, la gestione interna di opere o i profili utente.

## 1. Motivazione, destinatari ed evidenze

**Esigenza da validare.** Chi utilizza più assistenti sullo stesso PC può modificare i rispettivi file JSON/TOML con l'editor, un comando CLI o un'estensione mentre Repertica esegue «Collega», «Aggiorna» o «Scollega». È importante che l'operazione non cancelli **modifiche concorrenti** degli altri strumenti o un'impostazione inserita dall'utente dopo che Repertica ha letto il file. Un collegamento che rispetta l'indipendenza dell'utente deve poter fallire **senza danneggiare** il documento altrui e offrire un recupero comprensibile.

**Destinatari:** artisti, galleristi e archivisti che interrogano il catalogo tramite Codex, Claude Desktop, Cursor, Windsurf, Gemini CLI o VS Code; particolarmente rilevante per utenti che configurano più server MCP. Nessun assistente è obbligatorio per usare Repertica.

**Riscontri su `main`, letti il 22/11/2026:**

- [MCP.md](../../MCP.md) e [mcp_client.rs](../../../src-tauri/src/mcp_client.rs) descrivono e implementano **già** il collegamento con un clic e la rimozione della sola voce `repertica-studio` per **sei** client; JSON con altre chiavi e TOML con commenti/tabelle sono conservati **nell'elaborazione sequenziale ordinaria**. Non riproporre il connettore come funzione mancante.
- In `mcp_client.rs`, `configure_at` e `remove_at` leggono il documento, ne modificano la struttura in memoria, chiamano `back_up` e quindi `write_atomically`. La scrittura usa un percorso temporaneo derivato dal nome del file e `fs::rename`. **Nel percorso esaminato non compare una verifica finale che il contenuto del file di origine sia ancora uguale a quello letto**, né una gestione esplicita dei conflitti tra due writer diversi. Un `rename` non realizza da solo una transazione con confronto sul contenuto del file esterno.
- `back_up` conserva **una volta sola** `<file>.repertica-backup` quando il file esiste: buona salvaguardia dell'originale al primo collegamento, **non** necessariamente copia dell'ultimo stato prima di ogni operazione successiva. Non viene sovrascritto il backup precedente.
- Sono già presenti test per configurazioni valide e non valide, preservazione di altri server, BOM, commenti TOML, assenza di client e collegamenti obsoleti; **non sono emersi nei test consultati scenari con un'altra applicazione che modifica il file tra lettura e pubblicazione**.

**Precisazione:** la possibilità di *lost update* deriva dall'ordine delle operazioni, **non** da un incidente riprodotto. Non si afferma che le altre voci vengano normalmente eliminate, che il file sia corrotto o che la rinomina sia priva di protezioni su ogni OS/filesystem.

## 2. Distinzione dalle altre schede e dai bisogni Gestione/UX

- [TECH-014](REPS-TECH-014-bridge-mcp-locale-lettura-permessi.md) studia **quali dati il processo MCP può leggere e quali consensi siano effettivamente applicabili**, non la protezione del file JSON/TOML di un client esterno durante «Collega/Scollega».
- [TECH-033](REPS-TECH-033-salvataggio-condizionale-schede-obsolete.md) riguarda **revisioni di record SQLite delle opere/collezionisti**; qui il proprietario della configurazione è una **applicazione terza**, senza transazioni SQLite condivise. La somiglianza concettuale è soltanto «non sovrascrivere modifiche sopravvenute».
- [TECH-012](REPS-TECH-012-migrazioni-archivio-preflight-copia-sicurezza.md) tutela l'evoluzione dello schema dell'archivio Repertica; [TECH-015](REPS-TECH-015-diagnostica-assistenza-locale-report-privato.md) riguarda report tecnici privati. Non introduciamo nuovi servizi MCP.
- Il filone [Gestione e UX — PR #14](https://github.com/Cesare-ml/Repertica-Studio/pull/14) tratta catalogazione, vendite, collezionisti e processi professionali; questa scheda riguarda solo **l'integrità delle impostazioni di software esterni**. Nessun suo file viene modificato.

## 3. Scenari reali da discutere

1. **Modifica contemporanea.** Repertica legge `config.toml` di Codex; prima che la nuova voce sia pubblicata, Codex (o un utente) cambia un'altra impostazione. Repertica deve riconoscere il conflitto o ridurre al minimo la finestra di rischio: **mai proclamare che ogni modifica altrui sia preservata se il documento è cambiato**.
2. **Doppio clic/operazioni ravvicinate.** Due comandi su «Collega» e «Scollega» dello stesso client non devono contendere il medesimo temporaneo o lasciare un risultato ambiguo; il pulsante può essere disabilitato per l'operazione in corso, ma è necessaria una garanzia nel backend.
3. **File creato nel frattempo.** Al primo collegamento il file non esiste; un altro client lo crea prima della pubblicazione. Repertica non deve sostituire il documento nuovo con una configurazione minimale.
4. **Client aggiorna schema/percorso.** Una versione nuova di un'app modifica il formato di configurazione: la procedura riconosce un contenuto non valido/inatteso e **si ferma**, senza indovinare correzioni né svuotare configurazioni sconosciute.
5. **Interruzione o disco esterno.** File temporaneo incompleto, permessi negati o errore I/O: nessun messaggio ingannevole «collegato»; recupero verificabile senza sovrascrittura automatica di impostazioni recenti.
6. **Scollegamento dopo modifica manuale.** L'utente ha personalizzato la propria voce `repertica-studio`; mostrare che verrà rimossa **quella voce specifica**, senza presumere che Repertica possa rimuovere tutte le configurazioni MCP a livello di sistema.

## 4. Specifica funzionale ipotetica e flussi UX

**Vincoli certi:** integrazione opzionale, file appartenente a un client esterno, dati dell'archivio Repertica invariati, niente account/cloud obbligatorio, conferma esplicita per collegamento o scollegamento, conservazione delle altre voci, distinzione fra stato «connesso», «da aggiornare», «conflitto» e «non trovato».

**Flusso minimo candidato:** Impostazioni → Assistenti IA locali → seleziona client → «Collega» → mostra il file destinazione e la sola voce che cambierebbe → conferma → verifica che il documento non sia cambiato durante l'operazione → **successo** oppure **conflitto, nessun file sostituito**. In caso di conflitto: «Rileggi la configurazione» e riproponi l'anteprima; non applicare automaticamente una seconda scrittura.

**Scollega:** mostra la voce attualmente presente e chiede conferma; dopo rimozione, leggere nuovamente lo stato. Non chiamare questo gesto «revoca» dell'accesso a SQLite: il processo MCP resta avviabile con i normali permessi OS, come chiarito da TECH-014.

**Gradazione delle opzioni (nessuna scelta approvata):**

- **A — minima, preferibile da valutare prima:** serializzare le azioni avviate dalla stessa istanza; presentare la configurazione che verrà toccata e un avviso di modifica esterna; effettuare un confronto conservativo prima della pubblicazione; conflitto → annullamento senza scrivere, aggiornamento dell'anteprima.
- **B — robustezza aggiuntiva:** applicare modifica e pubblicazione attraverso una procedura di confronto/esclusione che riduca la finestra TOCTOU. Il solo hash prima di `rename` **non elimina** la possibilità di un writer concorrente che agisca subito dopo il confronto: documentare chiaramente il limite quando il programma esterno non coopera a un lock comune.
- **C — recupero assistito opzionale:** copia datata **della versione immediatamente precedente alla modifica** e scelta consapevole fra «non fare nulla», «apri la cartella» e «esporta una copia». **Non** ripristinare automaticamente un backup vecchio sull'ultimo stato del client.

**Accessibilità:** descrizione del file e differenze in testo selezionabile, messaggi con focus e annuncio screen reader, navigazione completa da tastiera; colore mai unico segnale di conflitto.

## 5. Dati e API **soltanto descrittivi**

Nessuna tabella SQLite obbligatoria e nessun nuovo server. Un possibile risultato temporaneo di lettura può essere:

```text
ExternalConfigSnapshot {
  client_id: enum,
  path: local_path,
  exists: bool,
  content_digest: SHA256,
  parsed_format: JSON | TOML,
  target_entry_before: optional<entry>,
  proposed_change: Add | Update | Remove
}

previewClientConfigChange(client_id, action)
  -> Preview(path, entry_before, entry_after, snapshot_token)
   | InvalidFormat | NotInstalled

applyClientConfigChange(client_id, action, snapshot_token, confirmed)
  -> Applied(new_status)
   | ChangedExternally(reload_required)
   | InvalidFormat | PermissionDenied | IoFailure
```

Il token resta locale, a vita breve e legato a client/percorso/versione del file; non è un token di autenticazione. Qualsiasi contratto dovrà rifiutare input arbitrari che permettano di scegliere un file diverso dalla configurazione del client riconosciuto. Non memorizzare password o valori segreti del JSON/TOML in log o messaggi di errore.

**Integrità proposta da valutare:** confrontare i byte effettivi e lo stato di esistenza, non soltanto data di modifica o dimensione; evitare destinazioni tramite symlink inattesi e permessi allargati; minimizzare l'esposizione dei contenuti di configurazioni terze. Per garanzie più forti occorre studiare la semantica di sostituzione/cambio nomi di file su Windows, macOS, Linux e filesystem condivisi. Non etichettare qualunque sequenza `read→hash→rename` come «atomica rispetto agli altri writer».

## 6. Offline, privacy, impostazioni, lingua, esportazione e compatibilità

- **Offline:** il collegamento riscrive un file locale solo su richiesta; non accede a Internet, non scarica o installa client e non richiede servizi Cloud. Il successivo uso dell'assistente **può** inviare dati al suo provider: l'integrazione non garantisce la riservatezza end-to-end fuori da Repertica.
- **Privacy:** i file terzi possono contenere percorsi privati, argomenti e perfino credenziali di altri MCP. Analizzarli in memoria, non trasferirli a server, non includerli nei report di assistenza, non registrarli automaticamente nel backup di **Repertica**. Una copia di recupero esterna conserva i **permessi** appropriati e non va condivisa involontariamente.
- **Impostazioni:** nessun nuovo interruttore di rete o account; eventuale preferenza «mantieni una copia locale prima delle modifiche a client esterni», inizialmente prudente. La selezione del client è sempre esplicita. Non modificare config di altri programmi fuori dalla loro specifica cartella.
- **Lingua:** etichette, conferme, differenze e messaggi tradotti dalla lingua del sistema; fallback inglese e nomi tecnici del file invariati. Unicode nei percorsi, nomi di programma e input dei client.
- **Import/export:** non si importa alcun record nel catalogo; nessun nuovo CSV. Se necessario, il solo *snippet di configurazione* può essere copiato/esportato su richiesta per inserimento manuale. Il backup completo SQLite+media in cartella liberamente scelta **non deve inglobare tacitamente** configurazioni o segreti di terzi; l'utente resta libero di gestirle separatamente. Lo scollegamento non cancella i dati già letti da un assistente.
- **Compatibilità:** file JSON (Claude Desktop, Cursor, Windsurf, Gemini CLI, VS Code) e TOML (Codex), comprese versioni/strutture riconosciute dalla versione di Repertica; preservare le altre chiavi, i commenti TOML, l'ordine delle chiavi dove previsto. File assente vs file non valido vs client non installato sono casi distinti. Nessun cambio allo schema dell'archivio e nessuna dipendenza obbligatoria approvata.

## 7. Dipendenze, alternative, rischi e complessità

**Dipendenze attuali note:** lettura/parse JSON e `toml_edit` già utilizzati in `mcp_client.rs`; strumenti standard per file locali. Qualunque protocollo di locking, snapshot, sostituzione o permessi multi-OS richiede analisi propria: non si assume disponibile o affidabile senza prove.

**Alternativa zero:** mantenere il collegamento attuale, ma indicare chiaramente di non modificare la configurazione dell'assistente contemporaneamente. **Alternativa senza write automatiche:** mostrare solo snippet manuali da copiare, con rischio di errore spostato sull'utente. **Alternativa intermedia:** verifica del digest + avviso; costo basso, ma non garanzia contro tutti i race. Un lock proprietario non rispettato da editor/client esterni non dà una garanzia completa.

**Rischi da esplorare:** perdita di aggiornamenti esterni, errori di permesso, file di app terze in uso, symlink/sostituzioni dei percorsi, temporanei omonimi, reset dei permessi, copie con credenziali, recupero di un file vecchio sopra uno recente, falsi successi di aggiornamento, divergenze fra formati. Complessità **bassa-media** per UX/controllo conservativo, **medio-alta** per garanzie di concorrenza e I/O cross-platform con applicazioni che non cooperano.

## 8. Criteri di accettazione e test **ipotetici, non eseguiti**

1. Su ciascuno dei sei client, il collegamento aggiunge/aggiorna **solo** `repertica-studio`; altre impostazioni, server e commenti TOML restano semanticamente invariati.
2. JSON o TOML non valido, cartella non installata, permessi negati e percorso anomalo producono errore leggibile **senza** sostituire il contenuto.
3. Se il file cambia fra anteprima e conferma, l'operazione non usa lo snapshot vecchio: richiede nuova lettura e nuova conferma.
4. Simulare due writer che cambiano una chiave indipendente e `repertica-studio`; verificare che nessun caso di conflitto noto venga presentato come successo senza avviso. Specificare nei risultati **quali race residui non sono esclusi**.
5. In interruzione/I/O failure, file originale recuperabile, temporanei individuabili e pulizia sicura senza eliminare configurazioni nuove; recupero manuale non sovrascrive.
6. Configurazioni con percorsi Unicode, spazi, BOM, newline, commenti TOML e client aggiornati restano interpretabili; test nativi Windows/macOS/Linux, includendo file condivisi solo se rilevanti per l'uso reale.
7. Nessun log contiene contenuti di configurazione o credenziali; backup Repertica ed export CSV restano invariati.
8. Tastiera, screen reader, lingua di sistema e fallback inglese coprono anteprima/conflitto/errore; nessun invio online viene causato da «Collega/Scollega».

## 9. Fasi candidate e domande **riservate al proprietario**

**Fase 0 — validazione:** determinare quanti utenti adottano gli assistenti e se aggiornano manualmente/automaticamente le configurazioni; riprodurre su desktop una modifica concorrente e classificare conseguenze, senza assumere che il problema sia frequente.

**Fase 1 eventuale — minima:** UX con anteprima, stato di conflitto e ripetizione manuale; test di preservazione delle altre configurazioni. Verificare prima se bastano istruzioni «non modificare il file contemporaneamente».

**Fase 2 eventuale — robustezza:** progettare e collaudare un protocollo di scrittura prudente e copie di recupero, limitando espressamente le garanzie su altri writer non cooperativi e filesystem remoti.

**Questioni aperte:** è preferibile mantenere il pulsante «Collega» o proporre lo snippet manuale a utenti avanzati? Quanto è comune avere più programmi che modificano le stesse impostazioni? Quando il valore `repertica-studio` è stato personalizzato dall'utente, una richiesta di «Aggiorna» deve fermarsi e chiedere conferma specifica? Le copie datate di file terzi sono desiderabili o introducono troppi rischi di privacy? Quale promessa prudente di rollback è sostenibile su Windows/macOS/Linux?

**Decisione finale:** soltanto del proprietario. Questo documento è **Proposta — da valutare**; non costituisce autorizzazione a implementare, cambiare librerie, modificare client o avviare sviluppo.
