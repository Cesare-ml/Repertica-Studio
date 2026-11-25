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
- Backup e ripristino locali ora disponibili in impostazioni, con snapshot SQLite e media, verifica SHA-256, integrità SQLite e copia di sicurezza prima del ripristino. La suite Rust (oltre 80 test, backup e ripristino compresi) è stata eseguita con successo su Windows 11 il 2026-11-21 e ora gira anche nella CI `windows-latest`; in precedenza 13 test di backup fallivano su Windows perché `manifest.json` restava aperto durante la rinomina della cartella. Mancano ancora test di accettazione dell'app installata su Windows/macOS e su dischi esterni/NAS reali; restano da sviluppare versionamento di immagini, gestione multivaluta, IVA, transazioni multi-opera e fatture.
- Server MCP locale **in sola lettura** via STDIO (`Repertica Studio --mcp`), senza porte di rete e senza dati di contatto o fotografie: vedi [MCP.md](MCP.md). Resta da collaudare con assistenti reali (per esempio Claude Desktop) sui tre sistemi operativi.
- Gli aggiornamenti automatici sono implementati ma attivi solo nelle build di release con la chiave di firma del progetto (vedi [RELEASING.md](RELEASING.md)); il ciclo completo scarica → installa → riavvia va ancora provato con una release firmata reale.
- Non usare ancora come archivio di produzione unico.

## Criteri minimi di collaudo

1. Installazione senza account e creazione DB locale.
2. Creazione/modifica/ricerca opera, allegato foto e stato.
3. Creazione/modifica collezionista; email facoltativa.
4. Registrazione vendita collegata; operazioni duplicate o importi invalidi respinti; stato opera aggiornato.
5. Emissione certificato con seriale unico e dati congelati; stampa locale.
6. Resistenza agli errori: opere e clienti già legati a vendite/certificati si eliminano solo dopo una conferma che elenca cosa verrà eliminato a cascata.
7. Test Rust schema e vincoli, compilazione Angular, build Tauri per Windows/macOS/Linux.

## Prossimi lavori

1. Collaudo integrato su Windows/macOS/Linux, test su USB/NAS reali (il disco pieno è ora riconosciuto e segnalato con un messaggio chiaro, ma non è stato provato su un'unità realmente piena) (formato a cartelle versionate con manifest, non ZIP).
2. Importazione opzionale e assistita dal Gestionale Opere, solo da export esplicito e con dati dell'utente.
3. Migliorare certificati PDF, stampa e firma, ordinamento/gestione avanzata della galleria e ricerche avanzate.
4. Prova end-to-end degli aggiornamenti firmati con una release reale (chiavi e workflow sono pronti).
5. Accessibilità, localizzazione completa e test UI desktop.

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

## Consolidamento (novembre 2026)

- **Errori comprensibili:** vincoli SQLite, trigger, disco pieno, archivio occupato e permessi negati sono tradotti in messaggi d'azione (`src-tauri/src/util.rs`); gli errori sconosciuti restano invariati.
- **Disco pieno:** un backup o un ripristino interrotto per spazio insufficiente rimuove la cartella parziale e non tocca l'archivio attivo; il messaggio indica di liberare spazio. Il controllo preventivo dello spazio libero non è implementato.
- **Percorsi:** l'interfaccia non mostra più il prefisso `\\?\` di Windows.
- **Certificati:** data di emissione e anno del seriale seguono il calendario locale, non UTC.
- **Fotografie:** copertina e galleria si scelgono con il dialogo di sistema e il file è letto dal programma dopo averne controllato dimensione e tipo; non transita più come base64 nell'interfaccia.
- **Guida di primo avvio:** pannello di benvenuto nella Panoramica (cinque passi, apribile di nuovo da Impostazioni) e suggerimenti nelle liste vuote. Lo stato «guida nascosta» è salvato nel browser incorporato dell'app, non nell'archivio, quindi non entra nei backup.

## Evoluzione del prodotto (novembre 2026)

- **Nuova opera come nel Gestionale Opere:** il codice archivio è assegnato automaticamente (tabelle `archive_code_policy` e `archive_code_sequences`, schema v7). Prefisso (1–12 lettere/cifre), numerazione progressiva (`ART-0001`) o annuale (`ML-2026-001`, con l'anno dell'opera se indicato), cifre (2–8) e separatore (`-`, `.`, `_`) si scelgono in Impostazioni. I numeri non vengono mai riutilizzati, nemmeno dopo un'eliminazione, e un salvataggio fallito non ne consuma. L'anteprima del prossimo codice non lo riserva. L'ubicazione è stata tolta dall'interfaccia: il campo resta nel database e nelle esportazioni, quindi i dati esistenti non vanno perduti.
- **Creazione in un solo passaggio:** `create_artwork_full` salva scheda, copertina, galleria (fino a 20 fotografie) ed eventuale vendita (anche con un nuovo collezionista) in una transazione. Le immagini sono validate prima di scrivere qualsiasi file; in caso di errore non restano record, file o numeri consumati.
- **Collezionisti (schema v8):** nome e cognome, telefono, email, indirizzo, Instagram/Facebook/TikTok/sito/LinkedIn, dati di nascita facoltativi, note; una foto profilo (sostituibile) e fino a 20 altre fotografie, incluse nei backup, nella verifica dei media e nell'eliminazione del collezionista. I collezionisti esistenti conservano il loro nome (nel campo «nome»). Non sono stati portati gli «interessi» per le opere del Gestionale Opere.
- **Ripartizione delle vendite (schema v9):** parti con nome, percentuale (somma esatta 100%) e opzione «copre le spese»; tre parti predefinite (Personale, Attività, Risparmio) con le quote correnti; interruttore generale. La quota applicata è congelata su ogni vendita (`sale_allocations`), quindi modificare il piano non riscrive lo storico. Una parte già usata viene disattivata invece che eliminata. Le spese possono essere imputate a una qualsiasi parte che copre le spese, o a nessuna. Registro CSV `allocations`; il ripristino rifiuta vendite con quote diverse dal 100%.
- **Backup automatico (schema v10):** disattivato di default; frequenza, ora locale, giorno e cartella a scelta. Funziona mentre l'app è aperta; una copia mancata viene eseguita alla prima occasione, un errore (per esempio unità scollegata) è mostrato e ritentato dopo 30 minuti. La conservazione delle ultime N copie elimina soltanto cartelle create da questa funzione e registrate nel log interno, dopo controlli su nome, posizione e manifest. Le copie restano non cifrate. Il programma non può eseguire backup quando è chiuso: non è un servizio di sistema.
- **MCP senza terminale:** collegamento a Claude Desktop con un clic da Impostazioni (vedi [MCP.md](MCP.md)).
- **Interfaccia:** rimosse le frasi promozionali dalle pagine; resta l'attribuzione «Powered by Repertica» richiesta dalla licenza.

Da collaudare su desktop reale: creazione di un'opera con più fotografie grandi, collegamento a Claude Desktop installato, esecuzione del backup automatico all'ora impostata e dopo un riavvio, comportamento della ripartizione con piani modificati a metà anno.

## Media center, file RAW e pagamenti rateali (schema v11–v13)

- **Media center (v13):** catalogo di tutte le immagini dell'archivio (copertine, gallerie, foto dei collezionisti e immagini importate) con ricerca senza accenti, metadati (titolo, descrizione, testo alternativo, crediti, diritti), tag, raccolte, cestino con eliminazione definitiva, ricerca di file identici (SHA-256) e di immagini simili (impronta percettiva dHash a 64 bit; soglia 6 bit). Le immagini già presenti vengono registrate automaticamente alla prima apertura della libreria. Non sono stati portati gli alias di duplicato, il flusso di revisione dei duplicati e gli utilizzi sui siti del Gestionale Opere.
- **Strumenti immagine (locali):** ritaglio con proporzioni predefinite, correzione prospettica a quattro punti con proporzioni del risultato, rotazione, specchio, esposizione, contrasto e saturazione, con anteprima del risultato. Ogni modifica crea una **nuova versione**: la versione 1 è sempre l'originale intatto e qualsiasi versione si può ripristinare, ovunque l'immagine sia usata. I file delle versioni non vengono mai sovrascritti né eliminati dalla rimozione di un'opera o di una foto (restano nella libreria fino all'eliminazione definitiva dal cestino, rifiutata se una versione è conservata in un certificato emesso). Le miniature sono in una cache (`thumbs/`) esclusa dai backup. L'output resta PNG per i PNG; JPEG e WebP vengono salvati come JPEG (qualità 92). Immagini oltre 120 megapixel sono rifiutate.
- **Modifica in fase di inserimento:** nella finestra «Nuova opera» ogni fotografia scelta da file si può ritagliare, correggere e regolare prima del salvataggio; vengono conservati sia l'originale sia la versione modificata. Le immagini già in archivio si modificano dal media center (pulsante «Modifica» nella scheda opera e nella galleria) e si possono riusare dalla libreria come copertina o fotografia di galleria.
- **File RAW (v12):** un file RAW per opera (CR2/CR3, NEF, ARW, DNG, ORF, RAF, RW2, PEF e altri formati comuni), controllato per estensione e intestazione, copiato in streaming con SHA-256 (fino a 1 GiB), sostituibile, rimovibile e salvabile in una copia. Fa parte dei backup: i RAW possono renderli molto più grandi.
- **Pagamenti rateali (v11):** una vendita può essere non incassata, saldata, parzialmente incassata o **rateale** (2–60 rate con scadenze mensili e quote che sommano esattamente al prezzo, con resto sulle prime rate; acconto facoltativo). Il piano è modificabile dopo la vendita. Lo stato di ogni rata (pagata, parziale, scaduta, da incassare) deriva dagli incassi registrati applicati in ordine di scadenza; il piano non è un incasso. Il ripristino rifiuta piani con totale diverso dal prezzo. Registro CSV `installments`.

## Fotografie sempre dalla libreria

- Ogni fotografia di una nuova opera (copertina e galleria) o di una scheda esistente viene importata nella libreria immagini e poi collegata: così si può **ritagliare e correggere subito**. Dopo la scelta della copertina l'editor si apre da solo (si può saltare: l'originale resta nella libreria); con «Ritaglia e correggi» si riapre quando serve. Le modifiche creano nuove versioni, l'originale non viene mai alterato.
- Se la finestra «Nuova opera» viene chiusa senza salvare, le immagini importate durante quella sessione e rimaste inutilizzate sono spostate nel **cestino** della libreria (ripristinabili da lì). Quelle scelte dalla libreria non vengono toccate.
- La sezione fotografie ha una copertina grande (anteprima completa, senza tagli), miniature per la galleria con modifica e rimozione, e il file RAW.
- **Assistenti IA:** il collegamento con un clic ora vale per Claude Desktop, Codex, Cursor, Windsurf, Gemini CLI e Visual Studio Code (vedi [MCP.md](MCP.md)).

## Caratteristiche dell'opera, tipo di fotografia e aggiornamenti (schema v14)

- **Caratteristiche:** tre caselle sulla scheda dell'opera, sia in creazione sia in modifica: **Opera firmata**, **Vernice finale applicata**, **Incorniciata**. Sono salvate come interi 0/1 con vincolo, partono da «no» per le opere esistenti, compaiono nel registro CSV `artworks`, nello snapshot dei certificati emessi dopo l'aggiornamento e nello strumento MCP `get_artwork`.
- **Tipo di fotografia:** ogni fotografia aggiuntiva ha un tipo scelto tra Dettaglio, Firma, Mockup digitale, Opera ambientata, Opera incorniciata, Work in progress e Altro (predefinito). Si sceglie al momento dell'inserimento (nella finestra «Nuova opera» su ogni miniatura, nella galleria prima di aggiungere) e si cambia in qualsiasi momento dalla galleria. Il tipo è memorizzato come codice stabile (vincolato dal database, esportato nel registro `gallery`); se una fotografia diventa copertina, la vecchia copertina entra in galleria come «Altro».
- **Aggiornamenti automatici:** controllo all'avvio e ogni 6 ore (disattivabile), avviso, download con avanzamento, verifica della firma, backup preventivo se il backup automatico è attivo, installazione e riavvio, sempre dopo conferma. Dettagli e configurazione delle chiavi in [RELEASING.md](RELEASING.md).

## Nuova opera, visualizzatore, certificato ed esportazioni (schema v17)

- **Nuova opera:** il modulo è diviso in sezioni (Identificazione, Tecnica e misure, Caratteristiche, Descrizione). **Tecnica** e **Supporto** sono menu a tendina con le voci del Gestionale Opere e «Altro…» per un testo libero (i valori già salvati che non sono in elenco compaiono sotto «Altro»). Il **prezzo indicativo** si aggiorna da solo, circa 0,3 secondi dopo l'ultima modifica di altezza o larghezza; non c'è più il pulsante «Calcola stima» e un errore di calcolo non interrompe mai il modulo. Firmata / vernice finale / incorniciata sono interruttori con spunta ben visibile.
- **Visualizzatore a schermo intero:** nell'archivio opere un clic sulla miniatura o sul titolo apre l'opera a schermo intero, come nel Gestionale Opere: ← → scorrono le opere (nell'ordine e con i filtri dell'elenco), ↑ ↓ le fotografie (copertina e galleria, con il tipo e la didascalia), Esc chiude; lente d'ingrandimento sul passaggio del mouse; scheda con codice, stato, anno, tecnica e supporto, misure, caratteristiche, acquirente e descrizione, e scorciatoie a «Modifica opera» e alle fotografie. Le immagini si caricano solo per l'opera mostrata (al massimo 6 in memoria).
- **Certificato:** stessa grafica del Gestionale Opere (A4, nome dell'artista e titolo in alto, immagine grande, dati dell'opera, dichiarazione, luogo e data, riga della firma, riquadro «Verifica»). Si sceglie la lingua (italiano, inglese o bilingue) e il luogo di emissione. L'immagine è esattamente quella registrata nello snapshot al momento dell'emissione, anche se la copertina è stata cambiata in seguito. Il riquadro di verifica riporta il numero e l'impronta SHA-256 (numero + snapshot): non c'è un codice QR né una pagina pubblica di verifica, perché l'archivio è solo locale. Si può vedere l'anteprima e poi stampare o salvare in PDF dalla finestra di stampa del sistema.
- **Nome dell'artista:** si imposta una volta in **Impostazioni → Artista e certificati** (al primo avvio la Panoramica lo chiede) e compare su ogni certificato; l'emissione è bloccata finché manca. I certificati già emessi mantengono il nome registrato. Il luogo scelto viene ricordato per l'emissione successiva.
- **Esportazioni:** i pulsanti CSV non sono più nelle pagine dell'archivio: sono tutti in **Impostazioni → Esportazioni**, raggruppati per Archivio opere, Collezionisti, Vendite e incassi, Certificati e Spese. Il registro `certificates` ha ora anche le colonne `language` e `issued_place`.
- **Schema v17:** `studio_settings.artist_name`, `certificates.language` (it/en/bilingual, predefinito it) e `certificates.issued_place`. I certificati restano immodificabili.

## Brochure, stampa del certificato, posta e icona

- **Brochure:** la copertina è solo tipografica (nessuna immagine): cornice a doppio filetto, nome dell'artista in corpo grande con grazie (Times), titolo in corsivo e sottotitolo tra filetti dorati. Le brochure non nominano mai Repertica Studio: l'intestazione delle pagine interne riporta il nome dell'artista (o il titolo, se manca) e le pagine di copertina e contatti si chiudono con un piccolo filetto; l'ultima pagina («Contatti») è centrata verticalmente con etichette e valori per email, sito e Instagram, e omette i contatti vuoti. I caratteri sono quelli standard del PDF (Helvetica e Times): le larghezze usate per centrare e andare a capo vengono dalle metriche Adobe. L'**anteprima** nell'editor ora riproduce la stessa geometria del PDF per copertina, introduzione, le tre impaginazioni delle opere (classica, a tutta pagina, immagine e testo) e pagina dei contatti, con numerazione.
- **Apri cartella:** dopo l'esportazione il messaggio «PDF salvato in …» ha il pulsante **Apri cartella**, che mostra il file in Esplora risorse (Finder, o il gestore file su Linux). Il comando nativo `reveal_in_folder` accetta solo percorsi assoluti esistenti, senza virgolette né caratteri di controllo, e non passa da una shell.
- **Stampa del certificato:** il foglio ha margine di pagina 0, quindi la finestra di stampa non aggiunge più il bordo bianco né la riga con data, titolo e indirizzo; lo sfondo e i colori sono forzati nella stampa. Nella finestra di stampa del sistema conviene lasciare scala 100% e carta A4.
- **Posta:** la voce del menu si chiama ora soltanto **Posta** (Mail, Correo, Courrier, E-Mail).
- **Icona:** la sorgente `branding/icon.png` era di 32×32 pixel e veniva ingrandita; ora è 1024×1024 (stesso disegno: quadrato verde con la «R» in Georgia), da cui `tauri icon` ricava tutte le dimensioni dell'.exe, dell'installatore e del .ico (16–256 px).

## Prezzo per opera e profilo dell'artista (schema v18)

- **Regole del prezzo** (le stesse del Gestionale Opere): un **prezzo fisso** impostato a mano vince su tutto; altrimenti il prezzo è larghezza × altezza × tariffa al cm² × coefficiente, dove il coefficiente è quello **proprio dell'opera** se presente, altrimenti quello dell'artista nelle Impostazioni. Nella scheda dell'opera (nuova e modifica) la sezione **Prezzo** offre tre modi: coefficiente dell'artista, coefficiente per quest'opera, prezzo fisso (che parte dal prezzo calcolato arrotondato all'euro). Il prezzo si ricalcola subito mentre si digitano misure, coefficiente o prezzo: il calcolo è nell'interfaccia (`pricing.ts`) e coincide con `finance::price_cents`, usato anche dallo strumento MCP `get_artwork` (che ora riporta `price_eur`, `price_source` e `artist_coefficient_override`). Il comando `price_preview` non esiste più.
- **Dove si vede:** colonna **Prezzo** nell'archivio (con «Prezzo fisso» o «Coefficiente x» sotto la cifra), scheda del visualizzatore a schermo intero e importo proposto quando si registra la vendita insieme a una nuova opera. Il prezzo calcolato non modifica mai le vendite già registrate. Il registro CSV `artworks` ha le colonne `artist_coefficient_override` e `fixed_price_cents`.
- **Primo avvio:** la Panoramica chiede **nome e cognome**, **nome d'artista** (facoltativo) e **coefficiente artista**, mostrando in tempo reale quanto verrebbe un'opera **30 × 40** e una **90 × 120** con la tariffa in uso. Le stesse voci si cambiano in Impostazioni (profilo in «Artista e certificati», coefficiente e tariffa in «Prezzi delle opere», con gli stessi esempi).
- **Nome sui documenti:** il nome d'artista, se c'è, compare come autore nei certificati (intestazione e riga «Artista») e viene proposto come artista di una nuova brochure; la dichiarazione del certificato è firmata col nome e cognome (registrato su ogni certificato come `declarant_name`; i certificati emessi prima restano invariati).

## Posta: grafica del Gestionale, sincronizzazione automatica e notifiche (schema v19)

- **Grafica:** la sezione Posta riprende il disegno della posta del Gestionale Opere: testata con ricerca nei messaggi caricati, cartelle a sinistra con pulsante **Scrivi**, posta in arrivo divisa in **Da leggere** e **Tutto il resto** (sezioni comprimibili, riga in grassetto per i non letti), lettore a tutta larghezza e **finestra di scrittura flottante** (riducibile, a schermo intero, con allegati, bozza locale e scarto). Le icone sono SVG incorporate.
- **Cartelle con nomi normali:** niente più `INBOX` o `INBOX.Sent`: le cartelle sono riconosciute dai flag del server (`\Sent`, `\Trash`, `\Junk`, `\Archive`…) e poi dal nome, e mostrate come **Posta in arrivo**, **Inviata**, **Archivio**, **Spam**, **Cestino**; le altre mostrano il proprio nome senza prefissi e con la codifica IMAP decodificata (sotto-cartelle come «Clienti / Roma», sotto «Altre cartelle»). Le bozze sono le **Bozze locali** dell'archivio; l'eventuale cartella Bozze del server compare sotto «Bozze sul server».
- **Sincronizzazione automatica, senza pulsanti:** aprendo Posta, cartelle e messaggi si caricano da soli; la cartella aperta si aggiorna ogni minuto e la posta in arrivo si aggiorna appena il controllo in background segnala novità. Sono spariti «Connetti / aggiorna IMAP» e «Aggiorna»; resta l'indicatore «Aggiornamento automatico» con l'ora dell'ultimo controllo (o «In attesa del server di posta» se il server non risponde).
- **Letto / non letto:** aprire un messaggio lo segna come **letto** sul server (e quindi anche negli altri programmi di posta); nel lettore c'è «Segna come non letto». È l'unica modifica che l'app fa sulla casella: la lettura in sé continua a non alterare nulla.
- **Numerino nel menu e notifiche:** mentre l'app è aperta, ogni minuto la posta in arrivo di ogni casella viene controllata in background (primo controllo pochi secondi dopo l'avvio). Il totale dei messaggi **da leggere** compare in un cerchietto rosso sopra l'icona di **Posta** nel menu a sinistra (99+ oltre i 99). Per la posta arrivata dopo l'ultimo controllo l'app mostra una **notifica di sistema** (mittente e oggetto, oppure «N nuove email» con i mittenti): è attiva per impostazione predefinita e si disattiva in Posta → Impostazioni della posta. Il primo controllo di una sessione registra solo lo stato della casella e non notifica la posta già presente.
- **Come funziona il controllo:** una connessione TLS per casella al minuto, solo sulla posta in arrivo (`UIDNEXT` e messaggi non letti); le password restano nel portachiavi di sistema e non finiscono mai negli eventi o nelle notifiche. Se `UIDVALIDITY` cambia, la posta non viene mai notificata come nuova.
- **Una sola casella e aspetto coerente:** si può collegare **una sola casella di posta** (il pulsante «Aggiungi account» sparisce finché ce n'è una; il salvataggio di una seconda è rifiutato anche dal backend, e per cambiarla si rimuove quella attuale). La grafica mantiene l'impianto del Gestionale ma usa colori, pannelli, tipografia e pulsanti di Repertica Studio (verde, titoli in Georgia, stessa selezione e stessi pulsanti del resto dell'app).

## Impostazioni riorganizzate

La pagina **Impostazioni** non è più un unico elenco: a sinistra c'è l'elenco delle categorie e a destra le schede della categoria scelta, ognuna in un riquadro.

| Categoria | Contenuto |
|---|---|
| **Profilo** | Nome e cognome / nome d'artista, lingua dell'app, guida di primo avvio |
| **Prezzi e vendite** | Tariffa e coefficiente (con esempi 30×40 e 90×120), ripartizione delle vendite, tasse e tesoreria |
| **Archivio** | Cartella dei dati, codice archivio, diagnostica delle fotografie |
| **Backup** | Backup automatico, backup manuale, ripristino |
| **Esportazioni** | Registri CSV |
| **Assistenti IA** | Collegamento MCP con Claude Desktop, Codex e altri |
| **Aggiornamenti** | Aggiornamenti automatici (con un puntino rosso quando ce n'è uno disponibile) e informazioni sul software |

Altre schermate aprono direttamente la categoria giusta: la guida di primo avvio apre **Backup**, la finestra dei certificati apre **Profilo**, l'avviso di aggiornamento apre **Aggiornamenti**. Le impostazioni della posta restano nella sezione **Posta** (ingranaggio). Nessuna impostazione è stata tolta o cambiata: sono solo raggruppate.

## Posta più veloce

Il tempo se ne andava soprattutto nel collegarsi al server: ogni operazione (cartelle, elenco, lettura, segno «letto», controllo in background) apriva una nuova connessione TLS e rifaceva il login, che con molti provider richiede alcuni secondi. Ora:

- **Una connessione tenuta aperta per casella:** dopo il primo accesso la sessione IMAP resta collegata e viene riusata dai comandi successivi e dal controllo in background (che la mantiene viva ogni minuto); se il server l'ha chiusa, il comando viene ripetuto una volta su una nuova connessione. Una sessione inutilizzata da più di 4 minuti viene scartata, e quando si cambiano password o server (o si rimuove la casella) la sessione si chiude.
- **Timeout:** 15 s per collegarsi e 30 s di silenzio al massimo su una connessione: un server che non risponde non blocca più la sezione indefinitamente.
- **Un solo comando per cartelle e messaggi:** `mail_overview` carica cartelle, contatori e messaggi della cartella aperta con una sola connessione, al posto di due comandi in fila.
- **Si vede subito quello che c'era:** riaprendo Posta (o una cartella già vista) l'elenco precedente compare subito, con «Aggiornamento in corso…», e viene sostituito appena arriva la risposta. Si conservano in memoria soltanto le intestazioni, mai su disco né il testo dei messaggi.
- **Lettore immediato:** cliccando un messaggio il lettore si apre subito con mittente e oggetto, e il testo arriva dopo («Caricamento…»).

## Eliminazione di certificati e a cascata

- **Certificati:** ogni certificato si può eliminare dalla tabella dei certificati (conferma richiesta). L'opera e la sua vendita non cambiano; il numero non viene riutilizzato. Un certificato emesso resta non modificabile: il trigger che lo bloccava in eliminazione è rimosso all'apertura dell'archivio (nessuna migrazione dello schema), quello contro le modifiche resta.
- **Opera:** prima di eliminarla, l'app mostra cosa dipende da lei — i certificati (con numero), la vendita con importo e data, gli incassi registrati e i documenti allegati — e chiede conferma. Con la conferma tutto viene eliminato in un'unica transazione SQLite (allegati, incassi, vendita, certificati, opera) e i file senza altri riferimenti vengono rimossi dal disco. Senza dipendenze basta la conferma semplice; senza conferma sulle dipendenze il comando backend rifiuta l'eliminazione.
- **Collezionista:** se ha vendite, la conferma avvisa che «verranno eliminate anche le vendite annesse» (con incassi e documenti). Le opere di quelle vendite tornano **disponibili**, i certificati già emessi restano. La stessa transazione elimina vendite, incassi, allegati e collezionista.
- Comandi: `artwork_deletion_impact`, `collector_deletion_impact`, `delete_certificate`; `delete_artwork` e `delete_collector` accettano `cascade` (di default no).
- **Eliminazione multipla:** nelle tabelle di opere, collezionisti e certificati ogni riga ha una casella e l'intestazione seleziona tutte le righe visibili. Con una selezione compare la barra «Elimina selezionati»: una sola conferma riepiloga quanti e quali record (i primi otto, poi «e altri N») e, sommati, certificati, vendite, incassi e documenti che verranno eliminati a cascata. Contano solo le righe visibili con i filtri attivi, così non si eliminano per errore record nascosti. L'eliminazione procede record per record; se uno fallisce l'app si ferma, ricarica l'elenco e indica quanti sono stati eliminati.
