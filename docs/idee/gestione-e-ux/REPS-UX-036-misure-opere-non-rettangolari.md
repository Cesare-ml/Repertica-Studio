# REPS-UX-036 — Misure di opere non rettangolari e dimensioni dichiarate senza stime fuorvianti

- **ID stabile:** REPS-UX-036
- **Data:** 2026-11-22
- **Stato:** **Proposta — da valutare**
- **Filone:** Gestione e UX — esclusivamente analisi documentale
- **Priorità proposta:** **P2 condizionata**; P3 se bastano le note già disponibili
- **Decisione:** nessuna approvazione, assegnazione, implementazione o modifica ai modelli esistenti

## 1. Problema concreto, destinatari e riscontri su `main`

La scheda di Repertica descrive già `width_cm`, `height_cm` e `depth_cm` come tre valori positivi facoltativi: ottimi per un dipinto rettangolare, ma meno precisi per una scultura circolare, un'opera irregolare o un'installazione con estensione variabile. Un museo può chiedere «diametro 62 cm», uno spazio espositivo «ingombro massimo 140 × 80 × 65 cm» oppure una dicitura «dimensioni variabili» che non significa tre misure conosciute. Salvare «62 × 62» come larghezza e altezza per un disco senza qualificarlo rischia di farlo apparire come una superficie rettangolare.

**Evidenze lette il 22/11/2026:** [modelli Angular](../../../src/app/models.ts), [modelli Rust](../../../src-tauri/src/models.rs), [schema SQLite](../../../src-tauri/src/db.rs), [maschera dell'opera](../../../src/app/app.component.html), [anteprima di prezzo](../../../src-tauri/src/finance.rs), [dati portabili](../../DATA-PORTABILITY.md), [convenzioni economiche e linguistiche](../../ECONOMY-I18N.md). La maschera usa Altezza/Larghezza/Profondità **in centimetri**; l'anteprima orientativa del prezzo moltiplica **larghezza × altezza × tariffa/cm² × coefficiente artista**. Le esportazioni delle opere trasmettono le tre colonne numeriche. Nei percorsi esaminati non emerge un campo distinto per *diametro*, *ingombro*, *misura indicativa* o *dimensioni variabili*. Non è stata provata una richiesta degli utenti né svolto un test nativo: è un'opportunità, non un bug accertato.

**Utenti:** scultori, artisti di installazioni, gallerie che compilano schede per terzi, archivi e curatori che devono distinguere dimensioni fisiche da ingombri di installazione. L'interesse di chi cataloga soltanto quadri rettangolari può essere nullo.

## 2. Perimetro rispetto alle idee esistenti: evitare doppioni

- [UX-022 — Componenti fisici](REPS-UX-022-opere-composte-componenti-fisici.md) registra **parti di una singola opera** e relativi rapporti, non la *semantica delle misure complessive*. Questa proposta non inventaria parti, quantità, varianti o esemplari.
- [UX-005 — Edizioni e multipli](REPS-UX-005-edizioni-multipli-serie-numerate.md) distingue esemplari; le misure non diventano un identificativo di tiratura.
- [UX-023 — Listini e prezzi](REPS-UX-023-prezzi-richiesti-listini-storico.md) distingue richiesto e venduto; qui si evita di usare ingombri non confrontabili nella **stima orientativa esistente**, senza proporre nuove politiche di prezzo.
- [UX-025 — Allestimento e cura](REPS-UX-025-istruzioni-allestimento-cura-opera.md) conserva prescrizioni operative; un ingombro dichiarato non verifica che l'installazione sia stata allestita secondo tali istruzioni.
- [UX-034 — Analisi per caratteristiche](REPS-UX-034-analisi-vendite-caratteristiche-opere.md) aggrega vendite per tecnica/supporto: questa scheda **non** produce statistiche di vendita o coefficienti.
- [TECH-001 — Portabilità](https://github.com/Cesare-ml/Repertica-Studio/blob/docs/idee-strumenti-architettura-20261121/docs/idee/strumenti-e-architettura/REPS-TECH-001-pacchetto-dati-aperto.md) e [TECH-030 — Vocabolari locali](https://github.com/Cesare-ml/Repertica-Studio/blob/docs/idee-strumenti-architettura-20261121/docs/idee/strumenti-e-architettura/REPS-TECH-030-vocabolari-locali-tecniche-supporti-interoperabili.md) trattano formati/termini, non la rappresentazione delle misure. **Nessun file del filone tecnico va modificato.**

## 3. Casi d'uso verificabili

1. **Scultura a disco:** l'archivista registra «diametro 62 cm; spessore 4 cm», senza convertire il diametro in una larghezza fittizia e senza prezzo automatico pari a 62 × 62 cm². Può conservare separatamente l'ingombro rettangolare se effettivamente misurato.
2. **Opera a forma libera:** vengono indicate misure massime di ingombro «140 × 80 × 65 cm, circa» con fonte «misurazione in studio»; il report segnala che non sono dimensioni esatte di una superficie pittorica.
3. **Installazione variabile:** la scheda riporta «dimensioni variabili in funzione dell'allestimento», eventualmente con intervallo stimato documentato. La casella priva di numero non deve essere interpretata come zero.
4. **Consegna a una galleria straniera:** l'utente esporta una scheda che usa l'unità preferita dal destinatario (ad esempio pollici), con un valore convertito ed etichettato e la misura originale conservata. La conversione non pretende di aumentare la precisione del dato.
5. **Archivio retrospettivo:** «Ø 45 cm, fonte catalogo 1989, non misurato personalmente» rimane un'affermazione documentale; la successiva misurazione non sovrascrive in modo invisibile la fonte precedente.
6. **Opera composta:** le dimensioni complessive restano riferite all'opera intera; eventuali dimensioni dei singoli componenti appartengono a UX-022 e non sono sommate automaticamente.

## 4. Esperienza utente e interazioni candidate

**Scheda opera → Misure**: mantenere l'attuale inserimento semplice **Altezza / Larghezza / Profondità (cm)** per la maggioranza degli utenti. Inserire un link non invasivo «Misure particolari o indicative» con campo testuale **facoltativo** e breve esempio, senza imporre una nuova tassonomia. Empty state: «Nessuna misura particolare registrata», non «misure mancanti» se esistono le tre misure standard.

**Versione minima raccomandata:** una nota dedicata alle misure particolari, visibile nella scheda e nelle stampe selezionate, con indicazione facoltativa della fonte e «approssimativa / da verificare». Nessun parser automatico del testo, nessuna geometria dedotta e nessun nuovo coefficiente obbligatorio.

**Eventuale versione strutturata da valutare:** scelta esplicita *misura standard / diametro / ingombro massimo / dimensioni variabili / descrizione libera*. I codici sono stabili, ma la scelta deve consentire sempre testo libero, misure mancanti, data incerta e fonte dichiarata. Una misura «ingombro» non è equiparata a larghezza/altezza della superficie. Un valore numerico convertito conserva l'unità originale e la precisione dichiarata; evitare conversioni numeriche ripetute che degradano il dato.

**Prezzo indicativo:** l'anteprima basata su larghezza × altezza non deve partire implicitamente da diametri, ingombri o intervalli. Se coesistono misure standard e particolari, mostrare la **base esatta** utilizzata per l'anteprima e una dicitura «Stima da rettangolo larghezza × altezza; non considera forma o volume». L'utente può decidere il prezzo commercialmente, ma non si propone nessun nuovo automatismo di vendita o modifica del coefficiente artista. Non riapplicare retroattivamente calcoli a vendite o certificati.

**Consultazione ed export:** mostrare misure sintetiche leggibili («Ø 62 cm» oppure «Ingombro massimo: circa …»), dettagli/fonte solo all'apertura. Stampa e dossier con anteprima della dicitura esatta, unità, carattere approssimativo e fonti selezionate. Le esportazioni per terzi **non** includono automaticamente note interne, contatti o ubicazioni.

**Accessibilità:** campo testo con etichetta, suggerimenti ed errori associati; nessuna icona Ø come unico significato (affiancare «diametro»), input utilizzabili da tastiera, riepilogo pronunciabile da screen reader, supporto zoom 200% e stampa senza dipendenza dal colore.

## 5. Dati necessari e contratti descrittivi (NON schema approvato)

**Fase minima possibile:** `artwork_measurement_note` (testo Unicode facoltativo), `measurement_note_source` (fonte testuale facoltativa), `measurement_note_approximate` (indicazione facoltativa) collegati tramite UUID opera. Si può anche sperimentare usando inizialmente una sezione di testo esistente **senza nuovo schema**, se non serve ricerca o export dedicato: da verificare prima di qualsiasi progettazione tecnica.

**Fase successiva, solo se dimostrata necessaria:** `measurement_record` (UUID, `artwork_id`, `kind` fra codici stabili, `original_value` decimale o null, `original_unit` [cm/mm/in], `value_text` originale, `qualifier` [exact/approximate/unknown], `scope` [physical/envelope/installation], `source_note`, data dichiarata e precisione, `recorded_at`, eventuale `supersedes_id`). Non utilizzare campi nullable come zeri fittizi, né trasformare un intervallo in un numero singolo.

**Vincoli di interoperabilità:** i vecchi campi `width_cm/height_cm/depth_cm` restano immutati e autorevoli **per il loro significato attuale**, inclusa l'anteprima rettangolare laddove appropriata. Qualunque nuovo tipo di misura deve dichiarare *cosa* misura. Identità UUID, date, unità e note viaggiano in rappresentazione aperta; non esiste chiamata automatica a un servizio di conversione. API soltanto concettuali: «leggi misure dichiarate», «salva nota qualificata», «genera riepilogo con unità richiesta», «esporta dati supplementari», «anteprima prezzo con formula dichiarata». Non sono endpoint o comandi da implementare in questa attività.

## 6. Offline, impostazioni, lingue, portabilità e privacy

- **Offline-first:** SQLite e copie locali, senza account, abbonamento, servizi esterni, telemetria o conversione cloud. L'utente conserva proprietà e accesso alle informazioni; backup e ripristino continuano a funzionare in qualsiasi cartella accessibile scelta dall'utente.
- **Lingue:** lingua OS con fallback inglese, override manuale IT/EN/ES/FR/DE come già documentato; traducibili solo etichette, tipo di misura, qualificatori, aiuti e messaggi. **Mai tradurre automaticamente le note dell'utente**. Non dedurre la forma da una parola tradotta.
- **Impostazioni candidate:** preferenza visiva cm / cm+pollici / pollici per la sola *visualizzazione*; lingua dei titoli dei PDF; nessun cambio forzato degli standard del database. Il sistema metrico resta la sorgente delle tre colonne numeriche storiche. Le conversioni usano precisione/arrotondamento dichiarati e non alterano il valore persistito.
- **Esportazione integrale:** qualsiasi dato aggiuntivo approvato dovrà comparire in un nuovo CSV/JSON versionato o in colonne additive documentate e nel backup verificato. **Non cambiare l'ordine, le intestazioni o il significato dei 12 CSV esistenti senza una decisione esplicita di compatibilità**. Le esportazioni selettive per terzi mantengono un'anteprima; gli export di archivio non devono perdere le note per opere non rettangolari.
- **Importazione futura:** distinguere la dicitura originale, l'unità sorgente e il valore numerico dichiarato; anteprima e risoluzione conflitti manuali, senza dedurre diametri da larghezza=altezza, né calcolare prezzi o modificare vecchi certificati.
- **Privacy:** note/fonte possono riportare nomi o indirizzi di professionisti e luoghi di custodia; trattarle come dati privati nel backup **oggi non cifrato**, nelle esportazioni e in eventuali assistenti MCP. Nessuna pubblicazione automatica; evitare di esporre note a terzi per default.
- **Compatibilità:** vecchi archivi senza nota continuano a funzionare; migrazione eventualmente additiva e transazionale; vecchi certificati immutabili; nessuna riscrittura delle vendite, dei prezzi congelati, delle statistiche, dei riferimenti alle foto o del coefficiente artista. Su Windows/macOS/Linux gli input localizzati devono produrre numeri equivalenti senza ambiguità tra virgola e punto.

## 7. Dipendenze, rischi, alternative, complessità

- **Dipendenze certe:** modelli e validazioni attuali delle opere, SQLite, backup/portabilità, sistema locale di traduzioni, anteprima prezzo. **Non** dipende dalla possibile gestione delle parti (UX-022), dai pacchetti TECH-001 o da qualunque provider online.
- **Rischio principale:** interpretare Ø, dimensioni massime o allestimenti variabili come superficie rettangolare e presentare un prezzo apparentemente preciso. Mitigare con tipo esplicito, formula visibile, comportamento prudente del pulsante di stima e assenza di conversione automatica fra figure geometriche.
- **Altri rischi:** proliferazione di campi per casistiche rare; misure discordanti; falsa precisione nelle conversioni inch/cm; font PDF senza il simbolo Ø; terminologia inconsistente fra lingue; esportazioni che perdono l'informazione qualificativa. Mitigare con testo originale, etichette complete, pochi tipi stabili e un percorso semplice.
- **Complessità:** **bassa–media** per nota facoltativa e visualizzazione; **media** per ricerca/export dedicati e unità di visualizzazione; **medio-alta** per una struttura di misure multiple con revisioni e conflitti. Nessuna modellazione tridimensionale, scansione, riconoscimento immagini o calcolo di superfici/volumi automatici.
- **Alternativa da testare per prima:** usare la descrizione dell'opera e le tre misure correnti con una convenzione editoriale chiara, producendo schede statiche; se questa soluzione soddisfa gallerie e scultori, **non realizzare un nuovo modulo**.

## 8. Criteri di accettazione ipotetici

1. Una scultura descritta «diametro 62 cm» non viene trasformata implicitamente in `width_cm=62, height_cm=62` e non ottiene automaticamente una superficie o stima di vendita.
2. Un utente di dipinti tradizionali compila gli stessi tre campi di prima senza passaggi aggiuntivi.
3. «Dimensioni variabili» ed «approssimative» rimangono distinguibili da misure esatte e dall'assenza di dati, anche in esportazione.
4. Una conversione visiva cm→inch mostra l'unità e non modifica i numeri originali; ripetere il cambio di visualizzazione non accumula errore.
5. L'anteprima prezzo indica la formula e le dimensioni realmente utilizzate, non produce prezzi da una nota in testo libero.
6. Restore e export completo conservano testo, qualificatori e fonte, con archivi storici e certificati già emessi invariati.
7. Ricerca, lettura e inserimento sono utilizzabili offline su Windows/macOS/Linux, da tastiera e con screen reader, con lingua sistema/fallback inglese.
8. L'esportazione selettiva consente di escludere fonte/note private e mostra il riepilogo prima di produrre file; i vecchi CSV restano compatibili.

## 9. Suddivisione in fasi e decisioni aperte

- **Fase 0 — validazione, senza sviluppo:** chiedere a pochi scultori/galleristi esempi *anonimizzati* di etichette reali (disco, forma libera, installazione variabile). Stabilire quanti oggi usano `description` e se ciò costituisce davvero un problema.
- **Fase A — eventuale minimo:** nota misure distinta e qualificata, facoltativa, con visualizzazione e stampa; avvertenza sulla stima se utilizzati dati di ingombro non rettangolari.
- **Fase B — soltanto se necessaria:** tipi essenziali, unità di presentazione, validazioni/export, ricerca locale dei tipi e controlli di compatibilità.
- **Fase C — opzionale e separata:** misure multiple con fonti/rettifiche e selezione per ciascun contesto di esportazione; valutare costi/benefici prima di estendere lo schema.

**Requisiti certi:** gratuità/open source; nessun account o abbonamento; SQLite locale; dati integralmente esportabili e backup con cartella scelta; lingua di sistema e fallback inglese; numeri standard in cm e anteprima `width × height` già esistenti; nessuna modifica automatica a vendite/certificati.

**Ipotesi, non fatti provati:** un numero sufficiente di utenti gestisce sculture/forme irregolari; una nota dedicata aumenta chiarezza e riduce errori; conversione inch utile per esportazioni internazionali; un modello strutturato potrebbe essere giustificato.

**Domande riservate al proprietario:** è prioritario servire scultori e installazioni già nella prima release? Basta una dicitura libera separata o servono diametro/ingombro come attributi filtrabili? È preferibile bloccare la stima per tutte le misure non standard oppure mantenerla manuale, con forte avvertenza? Le unità non metriche servono nella compilazione o soltanto in stampa/esportazione? Il livello di fonte/precisione merita una cronologia o basta una nota?

**Esito:** solo proposta documentale. Nessuna funzionalità approvata o assegnata allo sviluppo.
