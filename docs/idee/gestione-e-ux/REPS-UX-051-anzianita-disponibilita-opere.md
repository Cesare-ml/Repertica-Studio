# REPS-UX-051 — Opere disponibili da tempo: revisione del catalogo senza falsi tempi di giacenza

- **ID stabile:** REPS-UX-051
- **Data:** 2026-11-22
- **Stato:** Proposta — da valutare
- **Filone:** Gestione e UX
- **Priorità proposta:** P2 solo se il bisogno è verificato; P3 o nessuno sviluppo se bastano filtri ed esportazioni esistenti
- **Natura:** esclusivamente analisi e documentazione; non approvata, non assegnata, nessun mandato di implementazione

## 1. Problema, utenti e riscontri

**Problema.** Artisti, studi e piccole gallerie con decine o centinaia di opere devono rivedere periodicamente quali lavori sono ancora disponibili, quali non sono stati presentati di recente e quali meritano una verifica manuale del listino, delle fotografie o della collocazione. L'obiettivo non è suggerire sconti automatici o dichiarare che un'opera «non si vende»: è ottenere una **lista di revisione operativa** e, solo quando esiste un dato affidabile, capire da quanto tempo la disponibilità commerciale è stata **dichiarata**.

**Riscontro sul branch `main` il 22/11/2026:** [README](../../../README.md), [MVP](../../MVP.md), [modelli Rust](../../../src-tauri/src/models.rs), [modelli Angular](../../../src/app/models.ts), [filtri di archivio](../../../src/app/archive-filters.ts) e [portabilità](../../DATA-PORTABILITY.md). Ogni opera espone `status` (`available | reserved | sold | archived`), `year` testuale, `created_at`, `updated_at` e informazioni della scheda. I filtri correnti comprendono stato, acquirente, ricerca e ordinamento per titolo o anno; i CSV esportano le registrazioni. **Non emerge una data distinta e attendibile di effettiva immissione in vendita.** `created_at` è la data di **registrazione nel software**, non la data di produzione, esposizione, presa in carico o inizio della vendita; `updated_at` può cambiare per ragioni estranee alla disponibilità. Una vendita usa `sold_at`, che non certifica incasso, consegna o durata dell'offerta.

**Destinatari:** artista con archivio ampio, studio che effettua revisioni stagionali, galleria che prepara una selezione da aggiornare, archivista che distingue opere disponibili e opere soltanto registrate. Per uno studio con dieci lavori può bastare la ricerca già presente.

**Fatto verificato ≠ domanda verificata:** le proprietà del modello sono osservabili nel repository; l'esigenza della lista di revisione, la frequenza e il suo valore economico sono ancora **ipotesi**, da verificare con utilizzatori reali.

## 2. Confini e anti-doppioni

- **[UX-015 — Ricognizioni inventariali](REPS-UX-015-ricognizioni-inventariali-discrepanze.md):** verifica la **presenza fisica**; questa proposta non attesta che un'opera sia presente, non produce ricognizioni, non cambia l'ubicazione e non registra ritrovamenti.
- **[UX-001 — Provenienza e movimenti](REPS-UX-001-provenienza-movimentazioni.md)** e **[UX-046 — Consegne](REPS-UX-046-consegne-spedizioni-post-vendita-checklist.md):** riguardano custodia, trasferimenti e consegne; nessun evento fisico viene dedotto dallo stato commerciale.
- **[UX-023 — Prezzo richiesto e listini](REPS-UX-023-prezzi-richiesti-listini-storico.md):** unica eventuale fonte dei prezzi deliberati; la vista di revisione non deve calcolare «prezzo da ribassare», riscrivere coefficienti o creare uno storico listini parallelo.
- **[UX-034 — Analisi vendite](REPS-UX-034-analisi-vendite-caratteristiche-opere.md):** aggrega **vendite registrate** per caratteristiche; questa scheda riguarda **opere attualmente disponibili** e una possibile data di disponibilità dichiarata, non la bontà di un genere o una previsione di vendita. UX-034 osserva già che `created_at` non consente di misurare il tempo sul mercato: questa proposta assume esplicitamente tale vincolo.
- **[UX-002 — Trattative](REPS-UX-002-trattative-prenotazioni.md):** segue le offerte e prenotazioni; «nessuna vendita» non significa «nessun interesse», e `reserved` non va considerato automaticamente disponibile.
- **[TECH-008 — Viste salvate e filtri](https://github.com/Cesare-ml/Repertica-Studio/blob/docs/idee-strumenti-architettura-20261121/docs/idee/strumenti-e-architettura/REPS-TECH-008-viste-salvate-filtri-ricerca-locale.md):** infrastruttura per filtri, riutilizzabile solo se eventualmente approvata; questa scheda descrive **semantica, decisioni UX e limiti del dato commerciale**, non una seconda infrastruttura di ricerca. **[TECH-025 — Export minimizzati](https://github.com/Cesare-ml/Repertica-Studio/blob/docs/idee-strumenti-architettura-20261121/docs/idee/strumenti-e-architettura/REPS-TECH-025-esportazioni-tabellari-minimizzate-condivisione.md)** resta distinto.
- Nessuna modifica alla cartella dell'altra automazione. **Fuori ambito:** IA per raccomandazioni, statistiche predittive, scoring dell'artista o dei clienti, automatismi di sconto, pubblicazione online, stime di valore assicurativo o di rendimento fiscale.

## 3. Casi d'uso professionali

1. **Preparazione di una mostra.** L'artista visualizza le opere con `status=available`, controlla foto, scheda e prezzi da rivedere e seleziona dieci lavori; la selezione non prova la loro presenza fisica né prenota automaticamente le opere.
2. **Importazione di archivio storico.** Un dipinto realizzato nel 2010 viene catalogato nel 2026. Il sistema mostra «Registrato nel 2026» ma **non** «Invenduto da 16 anni» o «Disponibile dal 2026» senza una dichiarazione specifica.
3. **Disponibilità dichiarata.** L'utente annota «Offerto dal 15/03/2025» con fonte «listino galleria». In una revisione del 22/11/2026 è possibile calcolare il **tempo trascorso dalla data dichiarata**, esplicitando che non rappresenta giorni di esposizione effettiva né continuità di disponibilità.
4. **Prenotazione o ritiro temporaneo dal mercato.** Un'opera passa da `available` a `reserved` o `archived`. La lista corrente la esclude e **non** converte automaticamente quel cambio in una storia completa di intervalli commerciali.
5. **Anno incerto.** `year = "circa 1998"` o vuoto non determina una falsa data di inizio disponibilità. La vista consente di trovare e correggere schede incomplete senza inventare una durata.
6. **Riesame privato.** Una galleria esporta un elenco di sole opere selezionate con codice, titolo, stato e data dichiarata, **senza** note private, recapiti dei collezionisti, dati economici o fotografie se non esplicitamente scelti.

## 4. Flussi di esperienza utente proposti (facoltativi)

**Entrata leggera, prima dei nuovi dati.** Da **Opere → Revisione disponibilità** (oppure un filtro più piccolo, da validare) vengono mostrate soltanto le opere con stato commerciale `available`. Tabella leggibile: codice, titolo, anno dichiarato, tecnica, luogo **dichiarato** se utile e una colonna «Disponibile dal» che inizialmente mostra **«Non indicato»**. Filtri per anno, tecnica e ricerca già esistenti vanno riutilizzati dove possibile; niente nuova dashboard obbligatoria o KPI finanziario.

**Data opzionale, non dedotta.** Eventuale azione sulla scheda «Indica quando l'opera è stata messa a disposizione» con data, precisione (giorno/mese/anno o non nota) e fonte/nota facoltativa. Nessuna precompilazione con `created_at`, `updated_at`, `year` o data della prima fotografia. Un anno o mese incerto resta un periodo, non un giorno artificiale: le durate puntuali sono mostrate **solo** quando la data è completa e credibile.

**Riesame, non automatismo.** La tabella può evidenziare «Da verificare» per data assente, foto principali mancanti, titolo incompleto o prezzo deliberato non disponibile **soltanto in presenza di relativa fonte dati già realizzata**. Le etichette sono consigli di verifica, non errori bloccanti. Facoltativa una selezione multiopera per una **lista da controllare**, senza modifica massiva di prezzo, stato, ubicazione o descrizione.

**Stati distinti.** Gli stati `reserved`, `sold`, `archived` sono visibili come categorie separate, mai inclusi nel conteggio dei «disponibili»; l'eventuale prezzo fisso non equivale al prezzo richiesto di UX-023. Un'opera venduta torna a essere conteggiata solo se il suo stato è stato corregto con i normali controlli del dominio, non per decorso del tempo. **Nessun rapporto di rotazione** viene pubblicato senza basi comparabili e dati storici verificati.

**Accessibilità.** Tabella con etichette semantiche, indicatori testuali oltre al colore, tastiera e screen reader, tabulazione e focus coerenti, zoom 200%, contrasto elevato, finestre ridotte, date parziali pronunciate correttamente; ordinamento alternativo ai grafici. Numeri e date localizzati, semantica dei dati invariata.

## 5. Dati necessari e ipotesi descrittive, non schema approvato

**Fase minima:** nessun nuovo dato obbligatorio; proiezione dei `artworks` esistenti con `id`, `code`, `status`, `year`, `technique`, `support`, `created_at`. `created_at` resta visibile *solo* come «data di inserimento nell'archivio», non parte di una formula di giacenza. Nessuna tabella di statistiche persistenti.

**Fase eventualmente utile:** attributo locale per opera (o entità separata se serve conservare rettifiche) `commercial_availability` con `artwork_id`, `available_from` oppure periodo, `precision`, `source_note`, `recorded_at`, `updated_at`. La data può essere nulla, pregressa o incerta. Per correzioni significative prevedere origine e rettifica, senza falsificare l'evento storico. Nessuna FK verso vendite o spese necessaria per il primo esperimento.

Una **proiezione concettuale** (non API da implementare) sarebbe `availability_review({status:"available", as_of_date, filters}) → {items:[{artwork_id, available_from, precision, age_days_or_null, needs_review_reasons}], data_coverage, definitions}`. `age_days_or_null` esiste solo per date esatte, passate o uguali ad `as_of_date`, confermate dall'utente; viene **etichettata «tempo dalla prima data dichiarata»** e non «giorni effettivamente in vendita». Nessuna durata negativa per date future; input futuro segnalato e da correggere.

**Questione critica:** lo stato commerciale attuale non registra automaticamente periodi storici di disponibilità. Se in futuro servisse davvero «numero di giorni sul mercato», occorrerebbe una **cronologia di intervalli** con inizio/fine, sospensioni e rettifiche (complessità alta, possibile dipendenza da UX-002/012), non una sottrazione fra due timestamp arbitrari. **Non inclusa nella fase minima.**

## 6. Impostazioni, localizzazione, offline, import/export e compatibilità

- **Impostazioni:** nessuna nuova soglia per default; eventuale filtro salvato o soglie di revisione (es. 6/12 mesi) **solo su richiesta esplicita** e disattivabili, senza notifiche o invii automatici. Non usare una soglia unica per misurare qualità delle opere o valore artistico.
- **Offline-first:** lettura e annotazioni unicamente nel database SQLite locale; zero servizi esterni, login, abbonamenti, telemetria, email o sincronizzazione. Non servono geolocalizzazione, IA, autorizzazioni di rete né lettura automatica dei file originali.
- **Lingua:** rilevamento della lingua del sistema, selezione manuale prevista e **fallback inglese**. Etichette e messaggi localizzabili IT/EN/ES/FR/DE; testi delle fonti, titoli e tecniche restano come inseriti dall'utente; date e numeri secondo locale senza alterare i valori di archivio. Esportazioni con chiavi/codici stabili e data ISO.
- **Import/export:** l'attuale CSV completo `artworks` non va sostituito né filtrato silenziosamente. L'elenco di revisione sarebbe una **esportazione separata e volontaria**, CSV UTF-8 e, solo se utile, JSON versionato con UUID e precisione della data; avviso sui campi sensibili e anteprima delle colonne. Eventuale import della nuova data **assistito con anteprima**, match per UUID, rigetto dei riferimenti mancanti/ambigui, nessuna sovrascrittura automatica delle date originali. Il CSV oggi disponibile non è un reimportatore transazionale.
- **Backup e ripristino:** gli eventuali nuovi dati entrano nel normale snapshot SQLite e nei backup in cartelle liberamente scelte; nessun nuovo formato cloud, nessuna promessa di cifratura (backup attuali in chiaro). Per un export condivisibile non includere per default collezionisti, indirizzi, note commerciali, fotografie o date di altri rapporti.
- **Compatibilità:** migrazione SQLite additiva e facoltativa solo per fase con dati nuovi; archivi preesistenti devono mostrare «Non indicato» e non inventare `available_from = created_at`. Gli ID stabili delle opere restano invariati; vendite, quote, scadenze, certificati e backup storici non cambiano. Versioni precedenti del software possono non leggere un database migrato: rispettare le precauzioni di compatibilità/recovery studiate in TECH-042.

## 7. Complessità, rischi, dipendenze e alternative

- **Complessità:** bassa per filtro/lista basati su `status`; media per data dichiarata, precisione, provenienza, export e validazioni; alta per cronologia delle finestre di vendita o statistiche serie di rotazione, che **non** si propongono ora.
- **Dipendenze:** modello Artwork, query/filtri esistenti, SQLite/migrazioni e convenzioni i18n; soltanto se approvati in futuro UX-023 per listini, UX-002 per prenotazioni, TECH-008 per viste salvate, TECH-025 per export selettivi.
- **Rischio 1 — metrica ingannevole:** classificare un'opera come «invenduta da N anni» perché realizzata anni prima o registrata di recente. Mitigazione: nessuna inferenza temporale dai campi correnti, indicatore «data non nota», distinzione fra data di registrazione e disponibilità.
- **Rischio 2 — pressione commerciale:** ordinamenti suggeriscono «opere peggiori» o sconti automatici. Mitigazione: linguaggio neutro «da verificare», nessun ranking qualitativo, nessuna decisione commerciale automatica.
- **Rischio 3 — falsi rapporti storici:** stato odierno `available` non prova disponibilità senza interruzioni. Mitigazione: non chiamare l'indicatore «giorni di esposizione» né «giorni effettivi sul mercato».
- **Rischio 4 — dati sensibili, falsi contatori e prestazioni:** esportare note/galleria/clienti senza volerlo; aggregare migliaia di schede caricando foto; confondere stati vendita con incassi. Mitigazioni: export minimizzato, tabelle paginabili, proiezioni senza media, cache non necessaria, test dei conteggi.
- **Alternative senza sviluppo:** filtri per stato e ordinamento per anno già disponibili; esportare `artworks.csv`, costruire una tabella di controllo in LibreOffice Calc, compilare manualmente una colonna `disponibile_dal` esterna. Questa è l'alternativa **da testare per prima**. Se risolve il bisogno, non proporre la nuova funzione.
- **Alternativa di interfaccia:** aggiungere soltanto un filtro «schede disponibili da rivedere» senza salvare date o durate; progettazione molto più semplice.

## 8. Verifiche e criteri di accettazione ipotetici

1. Con un'opera del 2010 catalogata oggi e senza data dichiarata, la vista mostra **«Disponibile dal: non indicato»**, mai «16 anni sul mercato».
2. Le opere `reserved`, `sold` e `archived` non sono incluse nella vista predefinita delle `available`; filtri esplicitati in chiaro, stati dell'archivio non modificati.
3. Una data dichiarata esatta calcola esclusivamente il tempo dalla data dichiarata alla data di osservazione; una data parziale non viene trasformata in un numero puntuale di giorni.
4. Una data futura o non valida non produce età negativa; una data assente non equivale a zero.
5. La revisione non crea vendite, offerte, incassi, spese, listini, movimenti fisici, backup o certificati e non modifica gli originali delle fotografie.
6. Cambiare anno, foto, coefficiente artista o ultima modifica non cambia tacitamente la data dichiarata.
7. Export di elenco selezionato non contiene per default nomi/acquirenti, email, indirizzi, note private o margini; il backup completo include comunque tutti gli eventuali nuovi dati.
8. Il flusso funziona senza rete, con lingua di sistema/fallback EN, tastiera e screen reader; test con almeno 1.000 opere e molte date mancanti.
9. Il valore di una successiva fase avanzata viene misurato su **5–8 professionisti**: numero di revisioni svolte, tempo risparmiato rispetto a CSV+Calc e errori di interpretazione della durata.

## 9. Fasi eventuali e priorità da discutere

- **Fase 0 — Validazione senza sviluppo (prima scelta):** raccogliere 5–8 casi di rassegna catalogo, incluso almeno un archivio importato con opere storiche. Provare filtri attuali + CSV + Calc e registrare se il problema permane. Nessuna accettazione automatica della proposta.
- **Fase 1 — Eventuale lista di revisione minimale (P2 solo se confermata):** sola lettura, selezione e note/criteri comprensibili; nessun nuovo campo obbligatorio e nessuna durata calcolata.
- **Fase 2 — Eventuale data di disponibilità dichiarata:** se richiesta ripetutamente, aggiungere data/precisione/fonte opzionali e relativi export/backup, senza inferenze dal catalogo.
- **Fase 3 — Solo se necessario:** storie di sospensioni e rientri, esposizioni commerciali o confronti quantitativi, coordinati con i domini approvati; possibile rinuncia completa se il costo supera il valore.
- **Priorità proposta:** P2 condizionata a casi d'uso documentati; P3 o **nessuno sviluppo** in assenza di evidenze. Il valore artistico non deve essere misurato dall'anzianità della disponibilità.

## 10. Requisiti certi, ipotesi e domande al proprietario

**Requisiti certi (dalla documentazione e dal codice esaminati):** app desktop Tauri/SQLite offline-first; nessun account/abbonamento; status commerciali, codice/UUID, anno testuale, timestamp di archiviazione e ricerca per stato; export e backup in cartella scelta; lingua OS e fallback inglese; distinzione fra vendita registrata, incasso e consegna.

**Ipotesi da validare:** interesse dei professionisti per un riesame periodico delle opere disponibili; disponibilità e attendibilità della data di inizio offerta; vantaggio di una vista rispetto a un foglio Calc; uso appropriato di soglie selezionate dall'utente.

**Domande per revisione personale (non bloccano questa scheda):**
- Il bisogno prevalente è **rivedere le schede** o davvero misurare l'intervallo commerciale? Quali decisioni viene aiutato a prendere?
- Quali studi/gallerie conoscono il giorno esatto di prima offerta e quali solo mese/anno?
- Una data «prima offerta» è corretta se l'opera viene tolta e rimessa sul mercato? Occorre mai il totale dei periodi effettivi?
- Bastano un filtro `available`, l'anno e un foglio di lavoro? Quante opere costituiscono il caso tipo?
- Quali campi sono sicuri da mostrare nell'eventuale lista condivisa?

**Decisione attuale:** `Proposta — da valutare`. Nessuna approvazione, assegnazione, implementazione, issue operativa o merge.
