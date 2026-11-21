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
   - *Variable* `REPERTICA_UPDATER_PUBLIC_KEY`: il contenuto del file `.key.pub` (la chiave pubblica non è segreta).
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
