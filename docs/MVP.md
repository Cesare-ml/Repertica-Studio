# Repertica Studio — MVP 0.1.0

## Ambito autorizzato

Quattro moduli locali: **opere**, **collezionisti**, **vendite**, **certificati**.
Nessun Repertica Sites, account online, servizi SaaS, telemetria, pagamento, sincronizzazione o accesso remoto.

## Decisioni

- Tauri 2 + Angular; Core e validazioni in Rust.
- SQLite per singolo archivio locale, journaling WAL, foreign keys e migrazione tramite `PRAGMA user_version`.
- Importi monetari in centesimi interi, mai float.
- Identità stabili UUID. Nessuna importazione di dati o password dal Gestionale Opere privato.
- Vendita collegata obbligatoriamente a opera e collezionista, massimo una per opera; registrazione vendita aggiorna stato.
- Certificato con numero non riutilizzabile, snapshot immutabile dell'opera, autore che emette, data e campo firma a mano.
- Gli originali delle immagini restano locali e non attraversano la rete.
- Nessuna API verso Sites, provider IA o altri endpoint.

## Limitazioni consapevoli 0.1.0

- Certificato esportabile come PDF tramite stampa di sistema (o stampante PDF), non ancora PDF/A firmato digitalmente.
- Sono supportate solo immagini principali PNG, JPEG o WebP; massimo 12 MiB per immagine.
- La migrazione da PostgreSQL del Gestionale Opere non è ancora automatizzata.
- Backup e ripristino locali ora disponibili in impostazioni, con snapshot SQLite e media, verifica SHA-256, integrità SQLite e copia di sicurezza prima del ripristino. Mancano ancora test di accettazione su Windows/macOS e su dischi esterni/NAS reali; restano da sviluppare versionamento di immagini, gestione multivaluta, IVA, transazioni multi-opera e fatture.
- Nessun server MCP nella build MVP iniziale: rimane obiettivo successivo, da includere come processo locale read-only con permessi rigorosi.
- Gli aggiornamenti firmati via GitHub Releases non vengono abilitati prima di release, generazione chiavi e test su tutte le piattaforme.
- Non usare ancora come archivio di produzione unico.

## Criteri minimi di collaudo

1. Installazione senza account e creazione DB locale.
2. Creazione/modifica/ricerca opera, allegato foto e stato.
3. Creazione/modifica collezionista; email facoltativa.
4. Registrazione vendita collegata; operazioni duplicate o importi invalidi respinti; stato opera aggiornato.
5. Emissione certificato con seriale unico e dati congelati; stampa locale.
6. Resistenza agli errori: niente eliminazione di opere o clienti già legati a vendite/certificati.
7. Test Rust schema e vincoli, compilazione Angular, build Tauri per Windows/macOS/Linux.

## Prossimi lavori

1. Collaudo integrato su Windows/macOS/Linux, test su USB/NAS e gestione degli errori di disco pieno per backup e ripristino (formato a cartelle versionate con manifest, non ZIP).
2. Importazione opzionale e assistita dal Gestionale Opere, solo da export esplicito e con dati dell'utente.
3. Migliorare certificati PDF, stampa e firma, immagini multiple e ricerche avanzate.
4. Collegamento MCP locale via STDIO senza aprire porte di rete.
5. Aggiornamenti Tauri firmati via GitHub Release, solo dopo test end-to-end.
6. Accessibilità, localizzazione e test UI desktop.

## Backup e ripristino (incremento 0.1.x)

- La destinazione si sceglie con il dialogo nativo di sistema. Può essere una cartella locale, USB, unità rimovibile o rete già montata, con permessi di scrittura.
- Ogni esecuzione crea una **nuova sottocartella** `ReperticaStudio-backup-...`, senza sovrascrivere cartelle esistenti. I backup possono coesistere con altri archivi e file nella stessa cartella.
- Snapshot del database con `VACUUM INTO` e copie dei file media mentre un mutex impedisce modifiche concorrenti nell'app.
- Manifest versionato con SHA-256 e dimensione di ogni file. Prima della pubblicazione del backup vengono controllati hash, `PRAGMA integrity_check`, `foreign_key_check`, schema e immagini referenziate.
- Per ripristinare occorre selezionare **la sottocartella del singolo backup**, non genericamente la cartella che raccoglie più copie. Il ripristino è una sostituzione (non un merge).
- I file importati vengono verificati in staging; l'archivio precedente resta come copia di sicurezza con nome `ReperticaStudio-before-restore-...`. Un rollback interrompibile è rilevato al riavvio.
- Nessuna rete utilizzata dall'app. Le unità remote funzionano come filesystem già montati dall'OS; non vengono gestite credenziali, SMB o provider cloud dall'app.
- I backup **non sono cifrati**; possono includere nomi, email, indirizzi, importi e fotografie. Non salvarli in cartelle pubbliche o condivise senza adeguate misure.
- Un backup integro può comunque andare perso insieme al disco: conservare almeno una copia esterna e provarne periodicamente il ripristino.

Non è ancora disponibile la gestione di più archivi **simultaneamente** nella stessa installazione. Il backup di un archivio diverso può essere ripristinato in quello attivo, ma la procedura sostituisce integralmente i dati presenti.
