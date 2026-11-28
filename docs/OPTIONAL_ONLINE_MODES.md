# Visione: Studio locale, Studio Web e sincronizzazione opzionale

**Documento di progettazione, non requisito dell'MVP 0.1.0.** Le funzioni descritte non sono ancora implementate. Non modificare il comportamento offline-first esistente.

## Decisioni

- Repertica Studio desktop deve continuare a offrire TUTTE le funzioni locali senza account, connessione Internet, server o abbonamento.
- Alcuni artisti possiedono solo smartphone/tablet: progettare una versione **Studio Web** completa, responsive e accessibile con browser, account e archivio privato in Repertica Online.
- Gli utenti desktop possono attivare **facoltativamente** il backup/sync cloud per consultare e modificare l'archivio anche da telefono o browser, mantenendo una copia locale completa.
- Le tre modalità sono reversibili: locale, online, ibrida. Chi usa la modalità online deve poter esportare l'intero archivio in formato aperto e ripristinarlo in Studio desktop.
- Ogni artefatto pubblico per Sites/Auctions proviene da una **proiezione scelta**, non dal database privato replicato. Nessuna foto/raw, dato fiscale o dato collezionista va pubblicato automaticamente.
- **Nessun editor visuale dei siti e nessuna IA obbligatoria.** Repertica Sites fornisce la guida/SDK e la pubblicazione di bundle statici creati con qualunque IA; una chat integrata può essere studiata solo in futuro se sostenibile.

## Percorso di sviluppo raccomandato

1. Normalizzare contratti, UUID, migrazioni e modelli condivisi. Riutilizzare Angular e test di dominio senza appiattire il backend Tauri/Rust e quello Web in un unico runtime.
2. Implementare un archivio cloud personale Repertica Online (separato dalla proiezione pubblica Sites): dati strutturati in database privato, media in object storage privato, autenticazione con scoping account/device, cifratura in transito e a riposo, backup, export, quote.
3. Realizzare l'interfaccia Studio Web mobile-first con creazione, modifica, ricerca e gestione foto da smartphone, e poi parità progressiva con opere, collezionisti, vendite, certificati, economia, brochure, posta, ecc. Nessun finto bottone per moduli non implementati.
4. Offrire dal desktop un onboarding esplicito alla sincronizzazione. L'archivio locale rimane utilizzabile senza rete. Non upload dell'intero database senza consenso, preview iniziale dei contenuti, possibilità di revoca e disconnessione.
5. Sync bidirezionale con UUID per entità, change-log, revisioni optimistic, idempotency keys, queue offline, tombstones, hash per foto, retry e conflitti da risolvere senza perdere dati. Non fare sync del file SQLite in blocco.
6. Valutare dopo un'app nativa iOS/Android per accesso migliore a fotocamera e archivio locale persistente. Una PWA responsive può iniziare prima, ma la sua cache non è un backup affidabile.
7. Testare i casi concorrenti: modifica su desktop e cellulare offline, cancellazioni e ripristini, foto duplicate, device smarrito, revoca accesso, vendite/certificati che richiedono vincoli transazionali.

## Sicurezza e indipendenza

- L'archivio privato online non deve essere accessibile con token di pubblicazione di Sites, né con URL di media pubblici.
- Un account cloud comporta fiducia nel gestore della piattaforma: crittografia at-rest e HTTPS non equivalgono a end-to-end encryption. E2EE opzionale richiede progetto separato (gestione chiavi, ricerca web, recupero).
- Per utenti solo-mobile, backup esportabili completi (SQLite o manifest+JSON e media) e scaricabili verso una destinazione indipendente; avvisi e test periodici di ripristino.
- Il cloud non è un'alternativa ai backup: salvataggi indipendenti e procedure di fine servizio devono essere previsti da subito.
- Vedere `Cesare-ml/Repertica-Sites/docs/ECOSYSTEM_MODES.md` per i confini tra hosting statico, account e dati privati.

## Ambito protetto

L'MVP 0.1.0 di Repertica Studio **non** include un backend online. Questo documento non autorizza l'aggiunta di telemetria, upload automatici, login obbligatorio o modifiche distruttive al database desktop.

## Criterio di successo

L'artista che vuole indipendenza non perde alcuna capacità offline. L'artista che possiede solo uno smartphone può gestire l'intero lavoro. L'artista che usa entrambi i dispositivi può attivare la sync mantenendo la propria copia completa e un export autonomo.
