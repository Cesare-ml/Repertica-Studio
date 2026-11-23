# REPS-UX-065 — Anteprima dell'impatto di tariffa e coefficiente sui prezzi suggeriti del catalogo

- **Identificativo stabile:** REPS-UX-065
- **Data della proposta:** 2026-11-23
- **Stato:** **Proposta — da valutare**
- **Filone:** Gestione e UX
- **Priorità suggerita, non approvata:** **P2 condizionata** a una necessità ricorrente osservata; P3 o nessuna nuova funzione in caso contrario
- **Perimetro:** sola **simulazione locale, non distruttiva**, delle conseguenze della modifica di due impostazioni di prezzo; nessun nuovo listino, vendita, storico valutativo o aggiornamento massivo di schede

## 1. Problema, situazione verificata e destinatari

**Fatti verificati nel branch `main` il 23/11/2026:**

- `src/app/pricing.ts` e `src-tauri/src/finance.rs::price_cents` distinguono tre precedenze: **prezzo fisso** della singola opera, **coefficiente proprio** della singola opera, oppure **coefficiente artista** delle Impostazioni; quando non c'è prezzo fisso si usa la formula **larghezza × altezza × tariffa per cm² × coefficiente**. Senza dimensioni utilizzabili, il prezzo calcolato non è disponibile. L'importo resta espresso in centesimi, con arrotondamento secondo il motore corrente.
- La sezione **Impostazioni → Prezzi delle opere** permette già di modificare tariffa e coefficiente artista e presenta anteprime per **due formati standard (30 × 40 e 90 × 120 cm)**. La scheda singola presenta già il proprio prezzo calcolato. `priceInfo(a)` legge le impostazioni correnti; non è stato individuato nel percorso esaminato un confronto **sulle opere effettivamente archiviate** prima di salvare i nuovi parametri.
- Una variazione della **tariffa** può modificare i prezzi calcolati sia delle opere con coefficiente artista sia di quelle con coefficiente personalizzato. Una variazione del **coefficiente artista** interessa solo le opere che lo ereditano; il **prezzo fisso** prevale sempre. Non tutte le opere saranno quindi influenzate. Vendite e quote storiche non vengono ricalcolate dalla formula.

**Bisogno da validare, non malfunzionamento verificato:** un artista con molte opere disponibili può cambiare tariffa o coefficiente ritenendo che l'esempio standard rappresenti il proprio archivio; potrebbe scoprire solo dopo che alcune opere hanno suggerimenti diversi, altre restano invariate e altre non hanno misure sufficienti. Per una galleria o un archivista la stessa distinzione aiuta una revisione interna di listini *senza* attribuire automaticamente valore di mercato, assicurativo o fiscale.

**Destinatari ipotetici:** artista autonomo che aggiorna il metodo di prezzo, assistente di studio che prepara un confronto, piccola galleria che consulta opere con molte eccezioni. Per un archivio di poche opere, gli esempi già esistenti possono essere sufficienti.

## 2. Scenari professionali da osservare

1. **Variazione solo coefficiente artista:** da 1,0 a 1,3. Le opere senza override e con misure valide mostrano una differenza; quelle con coefficiente individuale o prezzo fisso non devono apparire falsamente modificate.
2. **Variazione solo tariffa/cm²:** da 0,16 € a 0,20 €. Cambiano i suggerimenti delle opere calcolate con coefficiente ereditato **e** individuale; nessuna variazione delle opere con prezzo fisso.
3. **Dimensioni mancanti:** opera senza altezza ma senza prezzo fisso = «Prezzo non calcolabile», non € 0, né voce da sommare alle differenze.
4. **Venduta, non disponibile o con prezzo fisso:** non cambiano l'importo di una vendita registrata, lo storico certificato, l'eventuale prezzo richiesto definito manualmente o i ricavi. Prevedere un filtro che escluda per default le opere non disponibili; le eccezioni commerciali vanno esplicitate, non dedotte dal prezzo.
5. **Archivio esteso:** cercare per codice o titolo una specifica opera interessata; capire quanti record sono calcolabili, invariati o esclusi senza blocchi e senza esportare dati privati.

## 3. Flusso UX proposto: dalla spiegazione minima a un confronto mirato

**Ingresso:** nella sezione prezzi già presente, dopo la modifica *non ancora salvata* di tariffa e/o coefficiente, si può scegliere «Vedi l'effetto sulle tue opere» (nome da provare con utenti). Nessuna simulazione deve essere richiesta per continuare a usare le Impostazioni.

**Anteprima:** mostra in modo leggibile i valori **attualmente salvati** e quelli **proposti**; accanto, i conteggi distinti «prezzo suggerito variato», «invariato per prezzo fisso», «invariato per coefficiente individuale» (solo quando cambia il coefficiente globale), «non calcolabile» e «non incluso per filtro». Le categorie devono essere **mutuamente esclusive** secondo una precedenza documentata; evitare sommare due volte la stessa opera. La tariffa cambiata può modificare anche il caso con coefficiente individuale: non presentarlo come escluso in quella situazione.

**Confronto per opera:** tabella con codice, titolo, disponibilità, metodo di prezzo, **prima → dopo**, differenza in centesimi/euro o «non calcolabile». Filtri iniziali prudenti: opere disponibili, poi facoltativamente tutte; ricerca locale; visualizzazione di sole righe variate. Anche opere con differenza esatta di zero per arrotondamento restano «invariate», senza inventare un aumento. I dati storici di vendita non compaiono come prezzi simulati.

**Uscita:** «Torna alle impostazioni» conserva la bozza di tariffa/coefficiente; «Annulla» non salva nulla. «Salva impostazioni» resta un'azione **separata e consapevole**, attraverso la validazione backend già esistente. Prima della conferma, se archivio o parametri sono cambiati rispetto alla lettura dell'anteprima, il confronto viene invalidato e ricalcolato o esplicitamente marcato «non aggiornato»: **mai presentare una preview obsoleta come garanzia del salvataggio**. Il salvataggio modifica solo le impostazioni previste, senza trasformare i prezzi suggeriti in prezzi fissi e senza scrivere nuove revisioni dei singoli record.

**Variante leggera da provare prima:** sotto gli esempi standard, un breve testo statico che distingua **tariffa globale**, **coefficiente ereditato**, **override per opera** e **prezzo fisso**; eventualmente una piccola lista di 3–5 opere reali selezionate manualmente. Se soddisfa il bisogno, evitare la tabella completa.

**Accessibilità:** ordine tastiera e focus conservato, confronto leggibile senza colore, colonne prima/dopo con intestazioni esplicite, messaggio per screen reader solo a fine ricalcolo (no raffiche aria-live a ogni tasto), zoom elevato e importi lunghi senza troncature. Etichette comprensibili anche a chi non distingue coefficiente e tariffa.

## 4. Semantica e dati necessari — ipotesi, non implementazione

**Dati già presenti:** `Artwork.id`, `code`, `title`, `status`, `width_cm`, `height_cm`, `artist_coefficient_override`, `fixed_price_cents`; `StudioSettings.artist_coefficient`, `base_price_cm2_cents`. Non servono vendite, clienti, note o fotografie per questo confronto. Un valore `fixed_price_cents = 0` va distinto da `null`: un prezzo fisso esplicito non è «prezzo mancante» nella logica attuale. Non presumere categorie di stato oltre i codici realmente esistenti, da mantenere coerenti col catalogo.

**Ipotesi di vista temporanea:** `price_impact_preview` (non tabella persistente): `artwork_id`, `price_mode` (fixed/own/artist), `before_cents: integer|null`, `after_cents: integer|null`, `delta_cents: integer|null`, `reason` codificato, `included_in_filter`, `settings_fingerprint` e `archive_revision` solo se necessari a invalidare preview obsolete. Gli importi sono in centesimi **interi** e i formati localizzati solo in UI. `null` non è convertito in zero. Identificativi e modalità non si traducono nel database.

**API puramente descrittive, non richieste operative:** «ottieni anteprima locale dell'impatto per una coppia di impostazioni», «elenca righe filtrate della preview», «invalida preview quando cambia il suo snapshot». Il calcolo deve coincidere col comportamento effettivo del **motore Rust**: non introdurre una formula alternativa JS che diverga per casi limite, valori invalidi o arrotondamenti. Nessuna scrittura alla simulazione; per archivi grandi considerare paginazione e limiti di memoria come requisito di esperienza da misurare, non introdurre dipendenze a priori.

**Nessuna somma di portafoglio per default:** sommare prezzi suggeriti di opere non equivale a valore di inventario, ricavo potenziale, utile, assicurazione, liquidità o prezzo richiesto. Se mai si valutasse un totale, sarebbe uno studio diverso con denominazione e condizioni esplicite; *non è parte della presente proposta*.

## 5. Impostazioni, offline, privacy e multilingua

- Nessuna nuova impostazione obbligatoria. Preferibile accesso su richiesta; filtri non persistenti e default «opere disponibili» salvo verifica dell'effettivo lessico di stato. Nessun salvataggio automatico di scenari e nessuna modifica al coefficiente per singola opera.
- Tutti i dati sono già su **SQLite locale**; simulazione funzionante senza rete, account, abbonamento, login o telemetria. Tauri/Windows/macOS (ed eventuale Linux) non devono dipendere da servizi esterni. Archivio di dimensioni elevate: nessun caricamento di fotografie, contatti o documenti non necessari.
- **Lingua:** segue quella di sistema con fallback inglese, possibilità di override IT/EN/ES/FR/DE già presente; tradurre etichette, motivi di esclusione, istruzioni e messaggi, non titoli delle opere. Le cifre sono formattate secondo locale senza cambiare importi né ricalcolarli in float per la sola presentazione.
- **Privacy:** un'anteprima di prezzi è un'informazione commerciale interna, non una pagina pubblica. Non includere nominativi, note, ubicazioni riservate, foto o dati fiscali. Nessun log dei suggerimenti e nessuna copia negli appunti implicita. Evitare schermate che si presumano sicure quando condivise.
- **Import/export:** nessun import necessario. Fase minima senza export della simulazione. Solo se gli utenti lo richiedono, CSV **esplicito, selettivo, senza campi privati** con codici delle opere e valori prima/dopo, distinguendo testata «simulazione non salvata» da prezzi effettivi; cartella scelta dall'utente, non sostitutivo dei CSV completi né del backup SQLite+media. Il salvataggio delle impostazioni mantiene il consueto export completo dei dati e il backup/ripristino in cartella liberamente selezionata; nulla deve vincolare i dati a questa vista.

## 6. Dipendenze, compatibilità, complessità e rischi

**Dipendenze concettuali:** `pricing.ts`, `finance.rs::price_cents`, `app.component.ts/html`, `Artwork` e `StudioSettings`, traduzioni locali; il catalogo mantiene la propria origine dati. Una eventuale preview per grandi archivi richiede calcolo autorevole e invalidazione della bozza, ma **nessun provider esterno, licenza terza o modifica allo schema è necessario per un prototipo di sola lettura**. Fase avanzata potrebbe avere requisiti differenti da valutare, non approvati.

**Compatibilità:** schema e backup attuali non devono cambiare per la sola anteprima; nessuna mutazione dei record opera, dei certificati (snapshot immutabili), delle vendite, degli incassi, delle ripartizioni congelate o delle spese. Un eventuale `asking price` con revisioni appartiene a UX-023 e non va sincronizzato in automatico. Su schede vendute la vista storica non deve confondere un suggerimento *attuale* con la somma realmente pattuita.

**Rischi:** interpretare il «prezzo suggerito» come listino o stima certificata; cambiare globalmente un parametro senza accorgersi dell'effetto sulle opere con override; confronto con dati incompleti o obsoleti; arrotondamenti discrepanti Rust/TypeScript; filtri che escludono troppe opere; tabella ingestibile su archivio grande; accessibilità insufficiente; esportare involontariamente valori interni. Mitigazioni: titolo «simulazione», motivi di ogni caso, conteggi di esclusione, nessun totale di valore, aggiornamento condizionale, prove con casi limite, export solo futuro e opt-in.

**Complessità ipotetica:** **bassa** per chiarimento testuale e piccoli esempi; **media** per confronto read-only paginato su tutte le schede; **medio-alta** se servono garanzie forti di coerenza tra lettura del catalogo e salvataggio in presenza di altri processi. Questa non è una stima di sviluppo approvata.

**Alternative:** (A) mantenere gli esempi standard e aggiungere spiegazione accurata; (B) confrontare manualmente un campione di opere nel catalogo e annotare le differenze; (C) esportare i CSV esistenti e calcolare offline con un foglio di calcolo, spiegando che le stesse regole e arrotondamenti devono essere rispettati; (D) non sviluppare alcuna vista se la ricerca con utenti non rileva errori o tempi significativi.

## 7. Criteri di accettazione ipotetici e verifiche

1. Per un'opera **30×40 cm, tariffa 16 cent/cm², coefficiente 1**, anteprima = **19.200 cent**. Con solo coefficiente globale 1,5 = **28.800 cent**; un override individuale 2 la mantiene a **38.400 cent** in entrambi i casi; un prezzo fisso resta identico.
2. Con tariffa da 16 a 20 cent/cm², la stessa opera con override individuale 2 passa da **38.400 a 48.000 cent**; **non** va classificata fra gli esclusi per override.
3. Con misure mancanti e nessun fisso, `before_cents`/`after_cents` sono `null`, non zero. Prezzo fisso 0 (valido nel codice corrente) resta 0 e distinto da assenza; valori enormi/non validi non devono produrre importi fittizi, overflow o `NaN` mostrati.
4. La lista è ordinabile/filtrabile e i conteggi riconciliano il numero totale di opere effettivamente considerate; nessun doppio conteggio e nessuna omissione silenziosa.
5. Preview annullata = **nessuna scrittura**; salvataggio impostazioni = esclusivamente configurazione prevista e successivo ricalcolo dei suggerimenti correnti. Prezzi fissi, coefficienti per opera, vendite, incassi, certificati e ripartizioni storiche invariati.
6. Se i parametri o le opere cambiano dopo il calcolo, la preview si aggiorna oppure segnala esplicitamente che non è più valida prima del salvataggio. Errori del backend non vengono presentati come esito positivo.
7. Prova offline in lingua di sistema e fallback inglese; 5 lingue, tastiera, screen reader, zoom 200%, cifre negative e grandi, testo non troncato, nessuna comunicazione esterna o log riservati.
8. Su archivio sintetico con molte opere, misurare tempo di risposta, uso memoria e blocchi dell'interfaccia; rifiutare design che richieda caricare immagini/documenti.
9. Test **osservativi, non ancora effettuati**: 4–6 artisti/galleristi confrontano una tariffa e un coefficiente modificati usando prima gli esempi esistenti e poi un mockup della vista. Misurare comprensione di «fisso», «override», «venduto» e tempo/errori. Se la spiegazione leggera basta, non approvare la tabella completa.

## 8. Fasi potenziali — nessuna attività assegnata

- **Fase 0 — validazione:** interviste brevi, prototipo statico con dati inventati, confronto con gli esempi già presenti; definire se serve davvero una vista di archivio.
- **Fase 1 — chiarimento minimo se utile:** sola terminologia ed esempi, senza nuove scritture né esportazioni.
- **Fase 2 — solo con evidenza:** anteprima read-only dei record reali, motivi di esclusione, filtri e confronto prima/dopo, preservando prestazioni e accessibilità.
- **Fase 3 — eventuale e distinta:** esportazione selettiva della sola simulazione, se richiesta e dopo verifica privacy. Nessuna modifica massiva dei prezzi.

## 9. Requisiti certi, ipotesi e domande per il proprietario

**Certi e verificati nel repository:** indipendenza, offline-first, SQLite e backup scelto dall'utente; formula e priorità fisso/override/artista; due esempi di dimensione in Impostazioni; prezzi correnti legati ai parametri; prezzi di vendita non modificati dai cambi di configurazione; cinque lingue con fallback inglese.

**Ipotesi non dimostrate:** utenti cambiano i parametri frequentemente; confondono anteprime standard e prezzi delle proprie opere; hanno un numero di opere sufficiente a giustificare tabella e filtri; preferiscono una preview che preceda il salvataggio. **Nessun incidente, intervista o test nativo è stato accertato in questa ricognizione.**

**Questioni aperte da decidere personalmente:**
1. Nella pratica, ogni quanto cambi tariffa al cm² e coefficiente artista? Gli esempi standard ti sono già sufficienti?
2. Vuoi inizialmente vedere **solo opere disponibili**, oppure un controllo informativo anche delle vendute con forte distinzione dagli importi storici?
3. Preferisci un confronto per 5–10 opere scelte dall'artista o sull'intero archivio filtrabile?
4. Ha senso esportare l'anteprima, o questo rischia di circolare come se fosse un listino ufficiale?
5. Quale terminologia distingue meglio «prezzo suggerito» da «prezzo richiesto» (UX-023) e dal «venduto» (UX-063)?

## 10. Confronto con idee esistenti e confini

- **UX-023** valuta la memorizzazione di *prezzi richiesti deliberati e relativo storico*: questa proposta non crea, aggiorna o pubblica un listino e non propone un prezzo commerciale per conto dell'artista.
- **UX-003** riguarda costi e margini *di una singola opera*; **UX-050** rendiconto mensile di registrazioni; **UX-051** anzianità disponibilità; **UX-063** la conferma di una vendita; **UX-064** correggere manualmente *metadati* di più opere. Qui non ci sono costi, ricavi, mutazioni massive o registrazioni: è una preview di una **regola globale già presente**.
- **TECH-008** viste salvate/filtri, **TECH-010** storico metadati, **TECH-033** salvataggi condizionali, **TECH-057** schema dei CSV: sono studi tecnici con perimetro differente. Il filone tecnico [PR #16](https://github.com/Cesare-ml/Repertica-Studio/pull/16) è consultato ma **non modificato**.

**Esito:** studio **esclusivamente documentale**, non approvazione, piano operativo né richiesta di sviluppo. Tutte le idee restano **«Proposta — da valutare»**.
