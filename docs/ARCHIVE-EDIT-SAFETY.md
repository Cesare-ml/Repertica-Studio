# Sicurezza della modifica di opere e collezionisti

Incremento backend locale (schema SQLite v5 invariato).

I comandi Tauri `save_artwork` e `save_collector` distinguono esplicitamente **creazione** e **modifica**:

- `id` assente: si crea un nuovo UUID e si inserisce un nuovo record, con le validazioni esistenti.
- `id` presente: l'identificativo deve già esistere in SQLite; se è stato eliminato dopo l'apertura della finestra di modifica, il salvataggio viene rifiutato senza creare un nuovo record.
- La modifica di un record esistente mantiene lo stesso UUID. Non cambiano i vincoli di relazione con vendite, certificati e gallerie.

Il controllo è effettuato mentre l'app possiede il mutex del database, immediatamente prima della scrittura. È utile anche se in futuro più finestre o importazioni renderanno una scheda obsoleta.

Due test Rust esercitano ciascuno: inserimento, modifica con ID stabile, eliminazione e tentativo di salvataggio della scheda obsoleta. Non sono state introdotte migrazioni né comunicazioni esterne.

## Limite

Questa protezione evita le **ricreazioni accidentali**. Non è ancora un controllo di concorrenza ottimistica sulle modifiche di un record tuttora esistente (che richiederebbe, per esempio, confrontare `updated_at` con quello caricato dalla finestra).
