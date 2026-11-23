# REPS-TECH-074 — Orientamento coerente delle fotografie nelle brochure PDF

- **Stato:** Proposta — da valutare
- **Data:** 2026-11-23
- **Priorità ipotetica:** P2 se la discrepanza è riprodotta con immagini valide; P1–P2 solo se documenti professionali essenziali ne risultano concretamente compromessi; altrimenti P3/QA o nessuno sviluppo.
- **Complessità:** bassa–media per un eventuale allineamento circoscritto; da stimare dopo prove native.
- **Perimetro:** studio documentale, nessuna implementazione o approvazione.

## Motivazione e riscontro sul repository

Un artista fotografa un quadro verticale con uno smartphone o una fotocamera. Il JPEG può memorizzare i pixel in orizzontale e dichiarare l'orientamento tramite EXIF. Se una brochure contiene quella foto ruotata o specchiata rispetto all'anteprima, il destinatario vede una rappresentazione errata dell'opera. Il problema è rilevante anche per immagini con iscrizioni e firme; non riguarda solo la scelta dell'impaginazione.

Riscontri di lettura su `main` del 23 novembre:

- [imaging.rs, righe 76–99](https://github.com/Cesare-ml/Repertica-Studio/blob/main/src-tauri/src/imaging.rs#L76-L99), blob `9df90d0303570ce553153030d45b114b00706a20`: `dimensions` tiene conto dei casi di scambio larghezza/altezza; `decode` legge `decoder.orientation()` e applica la trasformazione con `apply_orientation` prima di produrre RGB. Anteprime, miniature, elaborazione ed export del Media Center usano questo decoder.
- [brochures.rs, righe 134–165](https://github.com/Cesare-ml/Repertica-Studio/blob/main/src-tauri/src/brochures.rs#L134-L165), blob `79e37f710275280f2c0435abbebfd6793004d287`: `load_photo` verifica il percorso, file regolare, dimensione massima 32 MiB e limite 36 MP; poi usa separatamente `image::open`, thumbnail 2400×2400, conversione RGB e JPEG qualità 86. Non si osserva in questo percorso una chiamata esplicita all'applicazione dell'orientamento. Le dimensioni del derivato governano la geometria nel PDF (riga 228).
- [brochures.component.ts, righe 78–94](https://github.com/Cesare-ml/Repertica-Studio/blob/main/src/app/brochures.component.ts#L78-L94) carica le foto della galleria o copertina per l'anteprima; il template usa elementi img e dichiara che l'impaginazione a schermo è indicativa.
- [BROCHURES-OFFLINE.md](../../BROCHURES-OFFLINE.md) descrive export nativo, controlli sulle immagini e prova di anteprima. [MVP.md](../../MVP.md) documenta il Media Center con originali e revisioni non distruttive.

**Fatto:** i due percorsi Rust sono diversi. **Ipotesi da riprodurre:** per foto con orientamento EXIF diverso da 1, il PDF potrebbe usare i pixel non orientati mentre il Media Center o la WebView mostrano la foto orientata. La sola lettura del sorgente non prova un difetto su ogni codec o WebView: non sono stati eseguiti test, build o export e non è noto alcun incidente reale. Verificare anzitutto la semantica della versione effettiva di image 0.25 e il comportamento dei lettori.

## Destinatari e scenari

- Artista che invia un catalogo con foto di smartphone, senza ritoccare ogni immagine.
- Galleria che riceve fotografie da più fotografi e vuole un PDF coerente con le foto concordate.
- Archivista che prepara una selezione di immagini storiche o di dettaglio (firma, retro, iscrizioni), per cui uno specchiamento improprio è fuorviante.
- Professionista che alterna un originale con orientamento incorporato a una revisione già normalizzata: l'orientamento non deve essere applicato due volte.

Esempio sintetico: una matrice rettangolare con quattro angoli marcati A, B, C, D, e una freccia verso l'alto. Una copia con orientamento EXIF 6 e una con orientamento 2 permettono di distinguere rotazione e riflessione da un semplice scambio di larghezza/altezza. Nessuna fotografia reale è necessaria.

## Confini e anti-doppioni

Riletti gli indici di 73 schede TECH e 78 UX (PR #14, non modificata). TECH-013 tratta la preparazione di copie condivisibili e la rimozione dei metadati; TECH-021 studia ICC e già riconosce l'orientamento in imaging.rs; TECH-024 tratta l'interscambio IPTC/XMP; TECH-068 la memoria del renderer e menziona l'orientamento tra le regressioni; UX-062 la coerenza storica di una brochure rispetto all'archivio. Questa proposta circoscrive un percorso **già presente** che potrebbe non normalizzare l'orientamento, indipendentemente da colore, prestazioni, privacy di export e cambiamenti successivi delle schede. Se confermata, inserirla nei criteri di regressione dei filoni già esistenti, non costruire un secondo editor o generatore PDF. Nessuna nuova proposta sulla rotazione manuale, già disponibile.

## Specifica funzionale e flusso UX ipotetici

1. L'utente seleziona copertina o fotografia aggiuntiva nella brochure esistente; non gli si chiede di interpretare EXIF.
2. Il derivato di stampa interpreta la stessa sorgente e revisione nel medesimo sistema di orientamento dell'editor. Applica una sola volta le trasformazioni supportate prima del ridimensionamento e dell'impaginazione; i quattro angoli devono corrispondere all'anteprima orientata. Rispettare il ritaglio intenzionale già previsto dai template; non promettere identità cromatica o geometria identica alla preview indicativa.
3. Nessuna nuova impostazione per ruotare automaticamente le opere o modificare gli originali. Un eventuale errore di lettura non deve fingere un orientamento certo; distinguere assenza di tag da impossibilità di interpretarlo dove il decoder lo consente. Valutare una diagnostica traducibile se emerge un caso non interpretabile.
4. L'utente esporta come oggi in una cartella scelta esplicitamente. Il PDF già esportato non cambia; per sostituire una consegna occorre una nuova esportazione deliberata. Non alterare le brochure salvate, le foto, gli snapshot dei certificati o gli altri registri.

## Struttura dati e ipotesi di API (solo descrittive)

Nessuna nuova tabella, colonna, comando IPC pubblico o migrazione per la fase minima. La foto è selezionata dagli UUID e percorsi esistenti; i risultati di decodifica sono temporanei. Una possibile interfaccia interna, non codice eseguibile:

- Input: sorgente già convalidata, politica di limiti del chiamante (byte, dimensioni, pixel), lato massimo.
- Output: pixel RGB orientati o derivato JPEG, larghezza e altezza effettive, esito di interpretazione (se distinguibile).
- Il chiamante brochure conserva i limiti specifici 32 MiB / 36 MP e lato 2400; non eredita automaticamente i limiti più larghi dell'editor. Usare dimensioni e byte della medesima sorgente verificata; considerare errori e limiti di allocazione prima della decodifica.
- Il decoder condiviso è una possibilità, non un requisito di refactoring. Anche un adattamento strettamente locale a load_photo potrebbe essere adeguato. Non cambiare i formati ammessi senza una decisione indipendente.

## Offline, privacy, sicurezza e compatibilità

- Requisiti certi: desktop Tauri gratuito e open source, SQLite e media locali, nessun account, abbonamento, upload, telemetria o servizio esterno; backup integrali in cartella scelta dall'utente ed export completi invariati.
- Non conservare nuovi metadati EXIF né divulgarli nei PDF o nei log diagnostici. I byte originali e i relativi SHA-256 devono restare intatti. Non riaprire i file storici in scrittura e non rimuovere metadati dagli originali.
- Preservare le protezioni su percorsi relativi, file regolari, symlink, limiti di dimensione, tipi e destinazioni. Nessuna regressione nella gestione di file malevoli o corrotti; nessuna promessa di interpretare ogni metadato di ogni formato.
- Non aumentare la memoria utilizzata senza misure; conservare le politiche attuali per errori e output, e coordinare eventuali futuri cambiamenti di pubblicazione sicura con TECH-034. Nessuna nuova azione automatica.
- Compatibilità: archivi attuali e precedenti, immagini senza tag e revisioni già orientate continuano ad avere lo stesso significato. JPEG, PNG e WebP nei limiti e nel supporto effettivo; RAW/TIFF/HEIC fuori perimetro. PDF già consegnati intatti. Import/export CSV e backup non cambiano.
- Multilingua: lingua di sistema, fallback inglese e override esistente; nessun nuovo settaggio obbligatorio. Eventuali diagnostiche in IT/EN/ES/FR/DE, codici tecnici stabili, titoli e dati originali non tradotti. Avvisi accessibili senza dipendere dal colore; nessuna nuova interfaccia se non necessaria.

## Dipendenze, alternative e rischi

La crate image 0.25 (JPEG/PNG/WebP) è già dichiarata in Cargo.toml e usata da imaging.rs. Prima verificare se basta riutilizzare le primitive già disponibili; nessuna nuova dipendenza prevista. Non aggiornare crate o renderer per questa sola ipotesi senza motivazione separata.

- A: nessuna modifica se la matrice dimostra parità. Documentare il risultato e chiudere la proposta.
- B: guida temporanea a esportare una revisione normalizzata con gli strumenti già presenti, senza modificare l'originale; accettabile se il problema è raro, non ideale se ogni brochure richiede lavoro manuale.
- C: normalizzazione limitata a load_photo, preservando i limiti e il comportamento del renderer.
- D: contratto interno condiviso di decodifica orientata con limiti del chiamante, solo se riduce differenze senza ampliare il perimetro. Non riscrivere l'intero motore.

Rischi: doppia rotazione dei derivati, specchiamenti sbagliati, nuova alterazione della qualità o del crop, limiti di allocazione indeboliti, modifiche non intenzionali a hash e percorsi, comportamento divergente dei decoder. La trasformazione deve essere applicata esattamente una volta a ciò che dichiara il file, non inferita dal soggetto. Separare orientamento da ICC e trasparenza, non risolvere questi temi implicitamente.

## Criteri di accettazione e prove proposte (non eseguite)

1. Creare fixture JPEG asimmetriche con orientamenti EXIF 1–8, compresi i casi specchiati, e una senza tag; usare riferimenti visivi indipendenti per gli angoli e le dimensioni attese, senza usare lo stesso decoder come unico oracolo.
2. Confrontare Media Center, anteprima brochure e immagini rasterizzate/estratte dal PDF su almeno Windows/macOS e Linux se supportato. Controllare geometria e contenuto dei quattro angoli, non solo dimensioni; tollerare artefatti della compressione JPEG.
3. Ripetere con una foto di galleria, una copertina e una revisione già normalizzata. Testare tutti e tre i layout (classico, a tutta pagina, immagine e testo): nessuna doppia trasformazione o ritaglio imprevisto dovuto al rapporto d'aspetto.
4. PNG/WebP con e senza metadati, nei limiti del decoder effettivo; tag mancante, anomalo o non supportato, file troncato, dimensioni eccessive e path/symlink respinti. Nessuna regressione dei limiti specifici della brochure o uso incontrollato di RAM.
5. Confrontare prima/dopo gli SHA-256 degli originali e delle revisioni, i record SQLite e gli snapshot storici: invariati; nessuna modifica a PDF preesistenti o file fuori dalla destinazione scelta.
6. Verificare un'installazione senza rete, lingua di sistema supportata e non supportata (fallback EN), nessuna necessità di account; eventuali errori traducibili e comprensibili.
7. Se nessuna discrepanza è riprodotta con fixture valide, registrare il risultato come QA: non introdurre nuova UI, schema o refactoring.

## Fasi ipotetiche

- Fase 0: riproduzione isolata su copie sintetiche e revisione con il proprietario (nessuna azione in questa ricognizione).
- Fase 1: soltanto dopo approvazione e riproduzione, allineamento minimo e prove incrociate; nessuna migrazione.
- Fase 2: eventuale consolidamento interno dei decoder se i vantaggi sono dimostrati, coordinato con TECH-021/068; non necessario per la prima correzione.

## Questioni aperte per il proprietario

- Sono già stati osservati PDF con foto ruotate o specchiate rispetto alla preview? Quali formati e provenienze fotografiche sono frequenti?
- Basta un allineamento invisibile dopo prova, oppure è utile un avviso soltanto per metadata non interpretabili?
- Quali piattaforme e layout hanno priorità nelle prove native?
- Se il rischio non è riproducibile, archiviare la scheda come studio senza sviluppo?

**Nessuna decisione presa a nome del proprietario. Stato: Proposta — da valutare.**
