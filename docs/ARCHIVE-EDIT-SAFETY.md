# Sicurezza della modifica di opere e collezionisti

Incremento locale con **concorrenza ottimistica** (schema SQLite v5 invariato).

I comandi Tauri `save_artwork` e `save_collector` distinguono creazione e modifica:

- Creazione: `id` assente, UUID nuovo. Non è richiesto un token di revisione.
- Modifica: `id` presente, record già esistente. La scheda invia `expected_updated_at`, il valore ricevuto quando è stata aperta.
- Il backend confronta `expected_updated_at` con `updated_at` corrente tenendo il mutex SQLite; l'UPDATE include anche `WHERE updated_at = ?` e rifiuta risultati senza righe aggiornate.
- Se un'altra operazione ha modificato o eliminato il record, il salvataggio viene rifiutato. Non viene creato né sovrascritto alcun record.
- L'interfaccia mantiene aperta la bozza non salvata e aggiorna l'elenco in secondo piano. Per applicare una nuova modifica bisogna chiudere la scheda, verificare i dati attuali e riaprirla.
- L'UUID e le relazioni con vendite, certificati e galleria rimangono invariati.

I test Rust verificano record eliminati, aggiornamenti intervenuti dopo l'apertura, token mancanti e un successivo aggiornamento riuscito con la revisione corretta.

Questa difesa è **per singola scheda**, non uno storico revisioni. Il campo `updated_at` funziona come token leggero: programmi SQLite esterni che scrivono senza aggiornare questo campo non possono essere rilevati. Database e dati restano completamente locali ed esportabili.
