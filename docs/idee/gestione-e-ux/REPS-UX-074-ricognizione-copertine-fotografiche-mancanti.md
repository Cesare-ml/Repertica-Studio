# REPS-UX-074 — Ricognizione delle opere prive di fotografia di copertina

- **ID stabile:** REPS-UX-074
- **Data:** 2026-11-23
- **Stato:** Proposta — da valutare
- **Filone:** Gestione e UX; analisi e documentazione soltanto
- **Priorità proposta:** P2 **solo dopo** conferma di un bisogno ricorrente in archivi con molte opere; P3 o nessuno sviluppo se il controllo manuale corrente basta
- **Decisione:** nessuna approvazione, assegnazione, issue operativa o autorizzazione all'implementazione

## 1. Problema, destinatari e evidenze

**Problema da verificare.** Un artista o un archivista ha catalogato numerose opere, alcune senza una fotografia principale. Prima di preparare una brochure, un catalogo o un dossier deve individuare rapidamente **quali schede non hanno ancora una copertina collegata**, senza aprire ogni opera. Una mancanza può essere intenzionale: non tutte le opere sono fotograficamente documentabili e le fotografie aggiuntive non sono necessariamente adatte a rappresentare il fronte dell'opera.

**Destinatari:** studi con molti dipinti, gallerie che raccolgono un inventario prima di una mostra, archivisti che ricevono schede parziali e fotografi incaricati di completarne la documentazione. Per pochi record può essere più efficiente la normale lista con miniature.

**Riscontri statici sul branch `main`, connettore GitHub, 23/11/2026:**
- [Modello delle opere](../../../src/app/models.ts): `Artwork.image_path` è facoltativo; le immagini di galleria sono record separati `GalleryImage`, con `kind` (dettaglio, firma, ambientata, ecc.) e `sort_order`.
- [Archivio](../../../src/app/app.component.html) e [filtri](../../../src/app/archive-filters.ts): sono già presenti ricerca, stato, collezionista e ordinamento. Nel percorso consultato **non risulta un filtro esplicito per l'assenza di copertina**. La lista corrente usa miniature dove disponibili; non occorre inventare una nuova galleria fotografica.
- [MVP e diagnostica media](../../MVP.md): esiste già un controllo di **integrità dei riferimenti fotografici**, che segnala file mancanti, percorsi non validi e fotografie storiche dei certificati. Non identifica come errore la scelta legittima di **non aver associato alcuna copertina**.
- Le fotografie si possono già aggiungere o scegliere dal Media Center nella scheda opera. Non si propone una nuova funzione di acquisizione o associazione.

Questi sono **riscontri di codice e documentazione**, non risultati di test UX, segnalazioni di utenti né statistiche sulla frequenza delle copertine mancanti. Non è stato effettuato collaudo su Windows/macOS/Linux.

## 2. Requisiti certi, ipotesi e confini anti-doppioni

**Vincoli certi del progetto:** desktop Tauri open source e gratuito; archivio SQLite locale, nessun account/abbonamento obbligatorio; modalità offline; dati esportabili e backup in cartella scelta; lingua di sistema con fallback inglese e override manuale; vendite, certificati e fotografie storiche non devono essere alterati da una consultazione.

**Ipotesi da convalidare:** un filtro `Senza copertina` riduce il tempo necessario a preparare una sessione fotografica; può essere sufficiente senza creare liste di attività, stati `da fotografare`, scadenze, tag obbligatori o un motore di workflow.

**Non duplicare:**
- [UX-018 — Catalogazione seriale](REPS-UX-018-catalogazione-seriale-schede-bozza.md) riguarda **nuove** schede e bozze; questa ricognizione legge **opere già esistenti** senza modificarle.
- [UX-064 — Revisione guidata dei metadati](REPS-UX-064-revisione-guidata-metadati-opere-esistenti.md) riguarda correzioni di tecnica/supporto; qui non si cambiano metadati in massa.
- [UX-007 — Dossier](REPS-UX-007-dossier-presentazione-opere.md) e [UX-062 — Controllo brochure](REPS-UX-062-verifica-coerenza-brochure-rispetto-archivio.md) riguardano i **documenti di presentazione**; qui si individua una lacuna di documentazione fotografica **prima** di prepararli, senza nuovi documenti.
- [TECH-003 — Diagnostica integrità media](https://github.com/Cesare-ml/Repertica-Studio/blob/docs/idee-strumenti-architettura-20261121/docs/idee/strumenti-e-architettura/REPS-TECH-003-diagnostica-integrita-media.md): una copertina nulla non è un file danneggiato; nessuna duplicazione degli hash o della diagnostica.
- [TECH-045 — Associazione guidata delle foto importate](https://github.com/Cesare-ml/Repertica-Studio/blob/docs/idee-strumenti-architettura-20261121/docs/idee/strumenti-e-architettura/REPS-TECH-045-associazione-guidata-foto-importate-opere.md) progetta **come collegare foto già importate**; qui solo **quali opere richiedono attenzione**. L'eventuale collegamento usa esclusivamente i comandi esistenti, uno per volta.
- [TECH-008 — Viste salvate](https://github.com/Cesare-ml/Repertica-Studio/blob/docs/idee-strumenti-architettura-20261121/docs/idee/strumenti-e-architettura/REPS-TECH-008-viste-salvate-filtri-ricerca-locale.md): nessun salvataggio persistente di filtri in questa proposta.
- Il filone tecnico [PR #16](https://github.com/Cesare-ml/Repertica-Studio/pull/16) resta intatto.

## 3. Casi d'uso concreti e flusso UX

1. **Preparare la sessione fotografica.** Un'artista con 180 opere apre **Opere → Filtri → Copertina: assente** e trova soltanto le schede con `image_path = null`, con codice e titolo. Può restringere alle opere **disponibili** e cercare un ciclo per titolo/tecnica. Il risultato è una selezione di lavoro, non un giudizio di qualità.
2. **Una galleria ha già fotografie di dettaglio.** Un'opera possiede tre `GalleryImage` ma nessuna copertina. Rimane nell'elenco `Senza copertina`; l'interfaccia può indicare «3 foto aggiuntive» **solo se quel conteggio è stato caricato correttamente**, senza fingere che una foto di dettaglio rappresenti il fronte. L'utente apre la scheda e decide manualmente se promuovere una foto.
3. **Opera senza foto per scelta.** Una scultura temporaneamente non accessibile compare nel filtro: l'utente può ignorarla. Nessuna bandiera di errore, notifica obbligatoria, scadenza o eliminazione dal catalogo.
4. **Foto collegata durante la consultazione.** Dopo l'inserimento riuscito della copertina, il conteggio e l'elenco si aggiornano a partire dai dati correnti. Se la scheda è stata modificata in un'altra finestra/processo, evitare azioni sulla base di una lista obsoleta.
5. **File riferito ma fisicamente mancante.** Se `image_path` è valorizzato e il file non esiste, **non** classificarlo come «copertina mai associata». Mostrare eventualmente un collegamento distinto alla diagnostica media già disponibile, che tratta integrità e recupero.

**Flusso minimo proposto:** Archivio → filtro discreto `Fotografia di copertina: tutte / presente / assente`, combinabile con ricerca, stato e collezionista → contatore «N opere senza copertina tra quelle visualizzate» → apri scheda → usa l'azione attuale di aggiunta/scelta foto → ritorna al risultato filtrato, conservando il contesto. Preferire una nuova opzione di filtro all'introduzione di un'altra pagina, di una dashboard «qualità» o di una lista persistente di compiti.

**Stati vuoti e chiarezza:** «Nessuna opera corrisponde ai filtri», non «Archivio fotografico completo»; «Copertina assente» non «Foto mancante o danneggiata». La modifica manuale resta possibile anche fuori dalla vista. Non inserire automaticamente immagini, non promuovere foto aggiuntive, non cambiare `status`, prezzo, vendite o certificati.

## 4. Dati necessari e ipotesi API soltanto descrittive

**Fase minima preferita:** nessuna nuova tabella e nessuna migrazione. Riutilizzare `Artwork.id`, `code`, `title`, `status`, `image_path`; il criterio è `image_path == null` (o assenza semanticamente equivalente verificata dal backend), **non** un tentativo di decodifica del file o un conteggio di foto Media Center non collegate. Il conteggio deve riguardare i risultati **dopo** gli altri filtri: dichiarare chiaramente se si mostra anche un totale sull'intero archivio.

Solo per una fase eventuale, se emerge domanda reale: riassunto in sola lettura `ArtworkPhotoCoverage { artworkId, hasCover, galleryCount?, kindsPresent? }` calcolato dai collegamenti realmente presenti in SQLite, non dai tag generici del Media Center. `GalleryImage.kind` è una **classificazione dichiarata dall'utente**, non prova visiva che l'immagine riprenda davvero la firma o sia utilizzabile. Le voci `fronte`, `retro`, `firma` non diventano obblighi universali: andrebbero definite per tipologia d'opera e approvate separatamente.

Possibile pseudoflusso **non implementativo**: `filtered = existingArchiveSearchAndStatusFilter(artworks).filter(a => filter==='all' || Boolean(a.image_path)===(filter==='present'))`. Nessun nuovo comando Tauri è richiesto concettualmente per la sola copertina; prestazioni e sincronizzazione andrebbero validate. Evitare di salvare un campo derivato `incomplete` che potrebbe restare non aggiornato.

**Impostazioni:** nessuna nuova impostazione globale, soglia o attività automatica. Il filtro può resettarsi alla riapertura del programma. Eventuali preferenze persistenti appartengono alla proposta TECH-008, non a questa.

## 5. Offline, privacy, export/import, compatibilità e accessibilità

- **Offline e privacy:** sola lettura locale per la ricerca, senza rete, IA, telemetria, riconoscimento di immagini o consultazione di filesystem esterni. I titoli e le ubicazioni delle opere possono essere sensibili: non includere **collezionisti, indirizzi, prezzi o note** in un elenco di lavoro condivisibile per default.
- **Esportabilità:** per la fase minima resta disponibile l'export completo già esistente e il backup SQLite+media verificato, in una cartella liberamente scelta. Se i professionisti chiedono una lista da consegnare al fotografo, valutare in fase successiva un **CSV selettivo** con UUID/codice/titolo/stato e indicatore `has_cover`, anteprima obbligatoria dei campi e salvataggio esplicito; CSV UTF-8 conforme alla convenzione dell'app, protezione dalle formule di fogli di calcolo e nessuna sovrascrittura implicita. L'export della lista non include immagini e **non è** un backup.
- **Import:** nessun import nuovo; immagini collegate esclusivamente tramite le funzioni esistenti, con selezione e conferma umana. Non importare CSV per modificare stati o associare foto automaticamente.
- **Compatibilità:** il filtro deve rispettare il modello `image_path` già presente negli archivi migrati; copertina di certificato storico e copertina corrente sono entità diverse. Non toccare UUID, sort order della galleria, limiti immagini, quote, CSV completi o formati backup. Un'esportazione opzionale non cambierebbe gli schemi degli export attuali. Verificare dataset con grandi numeri di opere per evitare di caricare e decodificare tutte le foto solo per filtrare.
- **Multilingua:** tutte le diciture tradotte nelle lingue disponibili; lingua di sistema e fallback in inglese, con override già previsto. Non localizzare UUID/valori tecnici nel CSV; tradurre solo etichette e messaggi. Evitare dipendenza dal fuso orario o dalle convenzioni monetarie.
- **Accessibilità:** `<label>` associata al filtro, stati rappresentati con testo e non solo con colore o icona, contatore annunciabile alle tecnologie assistive senza notifiche intrusive, focus preservato dopo ritorno all'elenco, tastiera e zoom 200%, risultati senza immagini comunque comprensibili. Non usare la miniatura come unica indicazione della presenza/assenza di copertina.

## 6. Alternative, rischi e complessità

**Prima alternativa a costo nullo:** usare elenco opere e miniature già disponibili, ordinare/consultare manualmente e tenere eventuale lista di codici in LibreOffice. Può essere sufficiente per archivi piccoli. Alternativa minima se richiesta: un solo filtro `Senza copertina` senza contatori, CSV o nuovi pannelli.

**Rischi:** far passare `assenza foto` per difetto obbligatorio; confondere copertina non collegata con media corrotto; contare foto di un'altra opera come collegate; mostrare record non autorizzati in un CSV; accrescere i filtri dell'Archivio fino a renderli ingombranti; peggiorare prestazioni caricando molte miniature; creare aspettative di controllo della **qualità professionale** della fotografia. La proposta non affronta risoluzione, esposizione, diritti o idoneità alla stampa.

**Complessità stimata, non impegno:** **bassa** per filtro locale sul campo già esistente; **bassa–media** per conteggi galleria affidabili; **media** per un export selettivo con anteprima, privacy e test cross-platform. Nessuna dipendenza esterna, cloud o servizio commerciale obbligatorio; possibile riuso di API foto e filtri già presenti. Non autorizzare codifica o migrazione.

## 7. Criteri di accettazione da usare solo dopo un'eventuale approvazione

1. Opera A senza copertina, B con copertina, C senza copertina ma con 3 foto di galleria: il filtro `assente` mostra **A e C**, non B.
2. Ricerca per codice, stato `available` e collezionista continuano a combinarsi correttamente con il filtro, con ordinamento stabile e conteggi coerenti.
3. Dopo una copertina aggiunta manualmente a C, C non compare più nei risultati `assente`; nessuna modifica a vendite, prezzi, certificati o didascalie.
4. Una copertina con percorso memorizzato ma file irraggiungibile non viene spacciata per `mai collegata`; la diagnostica media rimane la fonte dei problemi di integrità.
5. Su archivio grande, il filtro non induce caricamento o decodifica di tutte le immagini; il tempo percepito resta paragonabile ai filtri già presenti, con misura su dataset campione prima di fissare una soglia.
6. Tastiera, screen reader, zoom e dimensione minima di finestra consentono di leggere e annullare il filtro senza perdere il contesto.
7. Inglese fallback e tutte le lingue supportate mantengono diciture coerenti; il filtro non altera né produce dati persistenti incompatibili.
8. Se approvato il CSV opzionale: sono esportati **solo** i record selezionati in un'anteprima comprensibile, senza dati personali nascosti né sovrascrittura, e il backup completo continua a funzionare inalterato.

## 8. Fasi di decisione e domande riservate al proprietario

- **Fase 0 — Validazione senza sviluppo:** osservare 3–5 professionisti mentre preparano una sessione fotografica; contare quanti record consultano e in quali casi una foto mancante è scelta deliberata. Se il problema è raro, **non sviluppare**.
- **Fase A — Solo dopo approvazione autonoma:** valutare un filtro non persistente per copertina presente/assente dentro Archivio. Nessuna pagina nuova.
- **Fase B — Solo se richiesto:** conteggio delle fotografie realmente collegate per opera e categorie fotografiche esplicitamente dichiarate. Non imporre categorie obbligatorie.
- **Fase C — Solo se richiesto:** esportazione selettiva di una lista fotografica, senza immagini o dati privati di clienti.
- **Domande al proprietario (aperte, non bloccanti):** prevalgono artisti con 20 opere o archivi con centinaia? Serve un elenco stampabile per un fotografo esterno? Il criterio `senza copertina` è sufficiente oppure interessa distinguere intenzionalmente `non fotografabile`, evitando un nuovo stato obbligatorio? L'attuale archivio con miniature basta già nella pratica? È preferibile evitare del tutto contatori e nuova interfaccia?

**Esito della ricognizione:** proposta circoscritta e reversibile da sottoporre a revisione, non segnalazione di bug e non richiesta di implementazione. **Proposta — da valutare.**
