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
- Sono supportate immagini principali e fino a 20 immagini aggiuntive per opera, nei formati PNG, JPEG o WebP, massimo 12 MiB per immagine. Le immagini aggiuntive hanno didascalia facoltativa e sono caricate solo su richiesta in anteprima.
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
3. Migliorare certificati PDF, stampa e firma, ordinamento/gestione avanzata della galleria e ricerche avanzate.
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

## Galleria fotografica (schema SQLite v3, ordinamento v5)

- L'immagine principale di un'opera resta nel campo storico artworks.image_path ed è indipendente dalle immagini aggiuntive.
- La tabella artwork_images memorizza UUID, opera collegata (FK con cancellazione a cascata), percorso relativo nel media store, didascalia e data di inserimento. La migrazione automatica da schema 1/2 conserva l'immagine primaria e le relazioni esistenti.
- Il modulo Rust gallery.rs controlla presenza dell'opera, numero massimo di immagini aggiuntive, formato riconosciuto dai byte, limite 12 MiB e percorso media. Nessun upload verso server e nessun percorso esterno accettato come immagine.
- La finestra Galleria nelle opere consente aggiunta, visualizzazione su richiesta, modifica/azzeramento della didascalia ed eliminazione delle foto aggiuntive. Una foto aggiuntiva può diventare la copertina: se esiste una copertina precedente, i riferimenti ai file vengono scambiati in una transazione SQLite e il vecchio file diventa una foto della galleria senza didascalia; se la copertina era assente, l'immagine viene rimossa dalla tabella galleria e diventa la foto principale. Il file non viene duplicato o eliminato, la didascalia della foto promossa non viene trasferita, i certificati storici non vengono modificati e non occorre una nuova migrazione. La modifica della didascalia conserva UUID, file originale e data di inserimento; il limite è 300 caratteri Unicode e non richiede migrazioni. L'eliminazione di un'opera senza vendite/certificati elimina anche le sue fotografie locali.
- I backup versionati conservano il formato manifest 1 e accettano gli schemi SQLite 1–5; sugli schemi 3–5 verificano che ogni fotografia in artwork_images sia inclusa e protetta da SHA-256. Un backup richiede una versione dell'app che supporti il suo schema SQLite: versioni più vecchie non potranno aprirlo.
- Lo schema v5 aggiunge `sort_order` alle immagini della galleria: la migrazione conserva l'ordine storico (data e UUID) per ciascuna opera. Con i comandi **Sposta prima / Sposta dopo** si riordina senza modificare file, identificativi, didascalie, date o certificati. Il backup SQLite+media continua a verificare gli hash e supporta i database v1–v5, senza alterare il formato manifest. Le versioni vecchie dell'app non possono aprire database v5.
- Le immagini principali sostituite e le fotografie rimosse dalla galleria vengono eliminate dal disco soltanto se non sono referenziate dal campo `image_path` di uno snapshot di certificato storico. In caso di snapshot JSON non leggibile, il vecchio file viene conservato per prudenza. La fotografia storica così conservata resta inclusa nei backup completi del media store; questo non può recuperare immagini eliminate da versioni precedenti. La stampa del certificato non include ancora automaticamente la fotografia: il riferimento storico viene preservato per successive funzioni di visualizzazione/stampa.
- I certificati già emessi restano snapshot immutabili e non acquisiscono automaticamente nuove fotografie.
- Da collaudare su desktop reale: caricamenti multipli di immagini grandi, eliminazione/restore, consumo memoria e compatibilità Windows/macOS/Linux.

## Diagnostica locale fotografie (lettura soltanto)

- In **Impostazioni → Diagnostica fotografie dell’archivio**, il controllo esamina i riferimenti alle copertine, alle fotografie di galleria e alle fotografie storiche conservate negli snapshot dei certificati. Anche la fotografia di un certificato che non è più la copertina corrente viene controllata.
- Verifica percorsi relativi sicuri dentro `media/`, file mancanti, elementi non regolari o collegamenti simbolici, dimensioni ammesse e intestazioni JPEG/PNG/WebP. Un risultato «leggibile» non equivale alla decodifica integrale del file.
- La verifica mantiene il mutex dell'archivio, non corregge record, non elimina immagini e non considera automaticamente eliminabili i media senza riferimenti: potrebbero essere necessari a documenti storici.
- I problemi sono contati integralmente; l'interfaccia mostra al massimo i primi **100** e indica se il report è troncato. Non espone file a server.
- Gli snapshot storici dei certificati con JSON malformato, struttura diversa da un oggetto o `image_path` di tipo errato sono segnalati come **Snapshot del certificato non valido** (identificativo del certificato e campo `snapshot_json`, mai il contenuto JSON). Un certificato senza fotografia, con `image_path` assente o `null`, è valido per questa verifica e non genera un falso allarme. Il conteggio include ogni riferimento fotografico e ogni snapshot storico non interpretabile; una foto usata da più certificati può essere verificata più volte. La diagnostica rimane di sola lettura e non ripara né riscrive documenti storici.
- **Limite importante:** non esistono hash storici registrati per ogni fotografia dell'archivio attivo, quindi questo controllo non può garantire che i byte non siano cambiati. Il backup completo dispone invece di un manifest SHA-256 e della relativa verifica.
- Se il controllo segnala un problema, non cancellare immagini o sovrascrivere backup: recuperare i file da una copia verificata e collaudare il ripristino separatamente.
