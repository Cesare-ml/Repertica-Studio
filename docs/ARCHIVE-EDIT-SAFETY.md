# Sicurezza della modifica di opere e collezionisti

Concorrenza ottimistica locale, senza migrazioni.

- Creazione: nessun token, UUID nuovo; i flussi create-full restano invariati.
- Modifica: il renderer conserva updated_at della scheda aperta e lo passa come expectedUpdatedAt ai comandi save_artwork/save_collector.
- Il backend verifica il token nel mutex SQLite e nella transazione; UPDATE include WHERE id=? AND updated_at=?, richiede una riga modificata e verifica la revisione letta prima del commit.
- Ogni salvataggio di scheda, registrazione/annullamento di vendita, promozione o sostituzione della copertina e ritorno a disponibile dopo una cancellazione a cascata usano lo stesso generatore e avanzano il timestamp RFC3339 anche quando l'orologio non avanza.
- In caso di conflitto la bozza rimane aperta, con messaggio localizzato e lista aggiornata; per modificare il dato corrente bisogna chiudere e riaprire la scheda. Nessun retry automatico.

Non è un registro revisioni o un sistema di sincronizzazione: non rileva scritture SQLite esterne che omettono updated_at, né garantisce revisioni globali per tutti gli altri comandi media/vendite o dopo il ripristino di un backup. Nessuna rete o account richiesti.
