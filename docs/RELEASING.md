# Pubblicare una versione e aggiornamenti automatici

Repertica Studio controlla da solo se esiste una versione più recente tra le **release di GitHub** (`Cesare-ml/Repertica-Studio`), mostra un avviso, scarica il pacchetto, ne verifica la firma, lo installa e si riavvia.

## Come funziona per chi usa l'app

- Pochi secondi dopo l'avvio, e poi ogni 6 ore mentre l'app resta aperta, l'app legge `latest.json` dall'ultima release. Il controllo si può disattivare in **Impostazioni → Aggiornamenti**, dove c'è anche **Cerca aggiornamenti**.
- Se c'è una versione nuova compare un avviso in alto con **Scarica e installa**. Non viene installato nulla senza questo clic.
- Se il backup automatico è attivo, prima dell'installazione se ne esegue uno. Se fallisce (per esempio l'unità esterna non è collegata) l'installazione si ferma e si può scegliere **Installa senza backup**.
- Il download mostra l'avanzamento; a fine installazione l'app si riavvia (su Windows lo fa l'installatore).
- L'indirizzo del feed è fissato nel programma, il pacchetto deve essere in HTTPS dalle release di questo repository e deve avere una firma valida: altrimenti viene scartato. Le versioni di prova (`1.0.0-beta.1`) non vengono proposte.
- Una build **senza chiave pubblica** (per esempio `tauri dev` o una build locale) non controlla né installa nulla e lo dichiara in Impostazioni.

I dati dell'archivio non vengono toccati dall'installazione. Se la nuova versione cambia lo schema dell'archivio, la migrazione avviene alla prima apertura ed è solo in avanti: per questo conviene tenere attivo il backup automatico.

## Configurazione una tantum (chi pubblica le release)

1. Genera la coppia di chiavi di firma, **fuori dal repository**:

   ```powershell
   npx tauri signer generate -w $HOME\.tauri\repertica-studio.key
   ```

   Scegli una password. Il comando scrive la chiave privata nel file indicato e stampa la chiave pubblica (file `.key.pub`).
2. Su GitHub, in **Settings → Secrets and variables → Actions**:
   - *Secret* `TAURI_SIGNING_PRIVATE_KEY`: il contenuto del file `.key` (la chiave privata);
   - *Secret* `TAURI_SIGNING_PRIVATE_KEY_PASSWORD`: la password scelta;
   - *Variable* `REPERTICA_UPDATER_PUBLIC_KEY`: il valore pubblico generato da Tauri (base64 del testo minisign, oppure il testo `.key.pub` con intestazione). Il workflow normalizza e valida il formato prima della compilazione. La chiave pubblica non è segreta.
3. Conserva la chiave privata in un luogo sicuro: **chi la perde non può più pubblicare aggiornamenti** che le installazioni esistenti accettino, e chi la ottiene può firmare aggiornamenti falsi. Non va mai inserita nel repository.

## Pubblicare una versione

1. Aumenta la versione, uguale, in `package.json`, `src-tauri/Cargo.toml` e `src-tauri/tauri.conf.json` (per esempio `0.2.0`).
2. Esegui il commit e crea il tag con la lettera `v`:

   ```powershell
   git tag v0.2.0
   git push origin v0.2.0
   ```

3. Il workflow **Release** controlla che tag e versioni coincidano e che chiavi e variabili esistano, poi compila per Windows, firma il pacchetto di aggiornamento e pubblica la release con l'installatore e `latest.json`.
4. Le installazioni esistenti trovano la nuova versione al controllo successivo.

Note: la release deve essere pubblicata (non bozza né pre-release) perché l'indirizzo `releases/latest/download/latest.json` la veda. Per ora le release sono solo per Windows: l'installatore non è firmato con un certificato di code signing, quindi Windows SmartScreen può mostrare un avviso al primo avvio («Altre informazioni → Esegui comunque»). Il pacchetto di aggiornamento è invece firmato con la chiave del progetto e verificato dall'app. macOS resta fuori finché non ci sono Developer ID e notarizzazione.

## Cosa non è stato verificato

La logica dell'aggiornamento è coperta da test (indirizzi consentiti, versioni di prova, messaggi), ma l'intero ciclo scarica → installa → riavvia richiede una release firmata reale: va provato una volta pubblicando una versione di prova e installando quella precedente.


## Recupero di una release fallita senza cambiare tag (21/11/2026)

Il workflow per `v0.1.0` ha compilato **entrambi gli installer Windows** (`.msi` e `.exe`), ma si è fermato alla firma degli aggiornamenti:

```text
failed to decode pubkey: Missing comment in public key
```

Questo significa che i file creati durante quella esecuzione **non sono diventati release pubbliche**. Il problema era il formato della variabile `REPERTICA_UPDATER_PUBLIC_KEY` trasferito senza validazione a Tauri. Il formato atteso dall'updater è il base64 dell'intero file minisign, comprensivo di `untrusted comment:` e della riga `RW...`: non soltanto la riga `RW...` o la parte binaria della chiave.

Il nuovo file `scripts/normalize-updater-public-key.mjs` accetta in sicurezza tre rappresentazioni della **stessa** chiave pubblica (formato prodotto dal Tauri CLI, contenuto minisign su due righe, sola riga base64 minisign). Verifica la presenza della riga di commento, il payload Ed25519 da 42 byte e produce il valore codificato completo senza generare o sostituire la coppia crittografica. La verifica avviene *prima* della lunga compilazione Rust. Nessuna chiave privata viene memorizzata nel repository.

Per ricreare una **release dello stesso tag** senza spostare `v0.1.0`:

1. Apri **GitHub → Actions → Release → Run workflow**, selezionando `main`.
2. Inserisci `v0.1.0` nel campo `release_tag`.
3. Il workflow usa il codice esatto di `v0.1.0`, ma applica gli strumenti di firma corretti dal branch `main`.
4. Verifica la creazione della release con installer MSI/EXE firmati per l'updater, le firme e `latest.json`. Controlla anche la funzionalità dell'installer su Windows prima di promuoverla pubblicamente.

Il comando automatico per le future versioni rimane il push di un tag `vX.Y.Z` corrispondente alle versioni presenti nei file.

**Non trasformare una build `v0.1.0` in `v0.0.1` rinominando i file.** La versione Tauri fa parte dei metadati e il feed di aggiornamento deve corrispondere a quella effettivamente compilata.

Se la fase di firma si interrompe ancora, **non** pubblicare installer senza firma dell'updater. Verificare che il secret GitHub `TAURI_SIGNING_PRIVATE_KEY` sia completo e corrisponda alla chiave pubblica configurata, senza stampare il contenuto della chiave privata nei log.
