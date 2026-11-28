# REPS-TECH-032 — Brochure PDF offline: caratteri Unicode, font incorporati e impaginazione affidabile

- **ID stabile:** REPS-TECH-032
- **Data della ricognizione:** 2026-11-22
- **Stato:** **Proposta — da valutare**
- **Ambito:** strumenti tecnici e architettura; analisi e documentazione soltanto
- **Priorità proposta:** **P1–P2 condizionata** alla verifica dell'uso di nomi e testi non rappresentabili in WinAnsi; **P2** se il bisogno è limitato
- **Decisioni:** nessun renderer, font, dipendenza, implementazione o incarico approvato

## 1. Problema e prove disponibili

**Fatto verificato su `main`, 22 novembre 2026.** Repertica Studio offre già un editor di brochure con bozze SQLite (schema 15), cinque lingue del documento (IT/EN/ES/FR/DE), tre impaginazioni per le opere ed esportazione PDF A4 **locale**. La documentazione [Brochure offline](../../BROCHURES-OFFLINE.md) dichiara un limite: il PDF usa Helvetica/Helvetica-Bold con codifica WinAnsi; caratteri non rappresentabili possono essere sostituiti.

Nel [renderer Rust](../../../src-tauri/src/brochures.rs), la funzione `win_ansi` converte i caratteri fuori mappatura nel byte `?`; `hex_text` serializza tali byte come testo PDF, mentre `render_pdf` dichiara i font Type1 e `WinAnsiEncoding`. Il wrapping attuale separa i testi per **numero di caratteri**, non per larghezza reale del glifo. Ciò rende plausibili due problemi **distinti**: perdita di caratteri nei documenti e impaginazione imprecisa su titoli lunghi o testi con glifi di ampiezza diversa.

Per un artista che prepara una brochure destinata all'estero, nomi propri, titoli di opere, citazioni, tecniche e contatti devono poter essere riprodotti **senza traslitterazioni o sostituzioni silenziose**, anche quando la lingua dell'interfaccia è italiana. Ad esempio un nome in alfabeto greco o un titolo in cirillico non dovrebbe diventare una sequenza di punti interrogativi. Una brochure francese o tedesca con caratteri rappresentabili nel vecchio encoding può invece già funzionare: non va presentato un guasto universale.

**Non verificato:** frequenza di testi in alfabeti non latini fra gli utilizzatori, risultati di rendering sulle tre piattaforme, estrazione testuale da visualizzatori PDF diversi, difetti reali di layout su brochure prodotte dagli utenti. Nessun test eseguito in questa ricognizione.

## 2. Destinatari e casi d'uso

1. **Artista internazionale:** inserisce il titolo originale di un'opera in greco, cirillico, giapponese o in una lingua con segni combinanti, poi invia un PDF che conserva l'identità testuale.
2. **Galleria:** crea una brochure con autori, nomi e recapiti multilingue; vuole che la stampa e la consultazione offline non dipendano dal font installato sul computer del destinatario.
3. **Archivio professionale:** esporta ripetutamente lo stesso documento dopo un aggiornamento e desidera risultati visivi stabili, senza perdere caratteri e senza modificare le schede delle opere.
4. **Operatore con testi molto lunghi:** inserisce didascalie e descrizioni che richiedono a capo corretti, gestione dell'overflow e possibilità di identificare contenuti non esportabili **prima** di scrivere il file.

## 3. Confini e confronto anti-doppione

- **[REPS-TECH-023](REPS-TECH-023-certificati-pdf-a-conservazione-verificabile.md)** riguarda conservazione e **validazione PDF/A dei certificati storici**; questa scheda riguarda **fedeltà del testo e del layout nelle brochure modificabili**. Font incorporati non significano PDF/A e non certificano autenticità.
- **[REPS-TECH-020](REPS-TECH-020-accessibilita-flussi-critici-tastiera-screen-reader.md)** riguarda principalmente interazioni native della UI; qui si valuta selezionabilità/copiabilità del testo del PDF. Non si promette automaticamente conformità PDF/UA o accessibilità completa.
- **[REPS-TECH-018](REPS-TECH-018-esportazioni-csv-scalabili-atomiche.md)** studia CSV, non text shaping o font PDF; le regole di scrittura non distruttiva sono utili come possibile criterio, non equivalgono a questa idea.
- **Gestione e UX [PR #14](https://github.com/Cesare-ml/Repertica-Studio/pull/14)**: UX-007 studia contenuto/condivisione dei dossier, UX-019 bibliografia e UX-031 attribuzioni; nessuna definisce l'encoding tipografico del **PDF di brochure già implementate**. Non modificare né duplicare le loro schede.
- Il file [Brochure offline](../../BROCHURES-OFFLINE.md) **menziona già** «font incorporati Unicode, controllo DPI/overflow» fra le evoluzioni future: questa scheda **approfondisce una lacuna riconosciuta**, non rivendica una scoperta né approva la soluzione tecnica.
- Esclusi: nuovo editor visuale, nuove lingue obbligatorie dell'interfaccia, servizi esterni, OCR, traduzione automatica, firma digitale, PDF/A, modifiche ai certificati, prezzi/listini, nuovi template e salvataggio automatico di PDF nell'archivio.

## 4. Comportamento proposto e flussi UX

**Percorso minimo da valutare.** Nell'editor brochure, chi ha scelto una bozza mantiene dati e lingua documento attuali. Al comando **Esporta PDF**, il motore controlla che i caratteri utilizzati siano rappresentabili nella configurazione tipografica disponibile. Se sì, esporta in una cartella scelta liberamente, come oggi. Se no, propone un messaggio localizzato con la posizione del testo problematico e un esito esplicito: **non sostituire caratteri senza avviso**. In una prima fase questa sola diagnosi potrebbe essere sufficiente.

**Percorso Unicode successivo, solo se giustificato.** Il PDF incorpora font di cui è consentito il bundling, con glifi effettivamente usati, codifica e mappatura Unicode corrette; viene inoltre calcolato l'ingombro del testo in base alle metriche dei glifi, non a un numero fisso di lettere. Prima dell'export mostra una verifica di eventuali glifi mancanti, testi eccedenti e pagine aggiuntive; l'utente può correggere la bozza o annullare. Nessuna correzione automatica delle anagrafiche o della descrizione originale dell'opera.

**Comportamenti da preferire:** anteprima esplicitamente indicata come *illustrativa* finché non viene validata la corrispondenza con il PDF; nessuna promessa di identità pixel-per-pixel con Angular. Non creare il file finale su export fallito; non sovrascrivere un PDF esistente senza decisione esplicita, coordinandosi con le regole di esportazione già presenti.

**Complessità delle scritture:** testo da sinistra a destra, diacritici composti e combinanti, legature, direzione da destra a sinistra, script che richiedono shaping e testo misto non sono casi equivalenti. Se la prima versione supportasse soltanto un sottoinsieme, deve dichiararlo esattamente e segnalare ciò che non può rappresentare, anziché attestare genericamente «Unicode completo».

## 5. Ipotesi dati e API — esclusivamente descrittive

Nessuna nuova tabella è un requisito certo. La bozza può mantenere gli attuali campi testuali Unicode e la lingua del documento: **non aggiungere conversioni distruttive o rimpiazzi di caratteri ai dati persistenti**.

Eventuale contratto interno (non una firma di funzione da implementare):

| Elemento descrittivo | Significato |
| --- | --- |
| `PdfTextDiagnostics` | elenco di glifi mancanti, campi interessati, overflow e gravità |
| `FontCapability` | identificativo del font incorporabile, copertura scritture, licenza e provenienza |
| `PdfExportProfile` | profilo scelto per layout/font; metadato di generazione, non nuova preferenza obbligatoria |
| `PdfExportResult` | percorso scelto, numero pagine, avvisi, versione del motore; hash eventuale se richiesto da un futuro flusso di conservazione |

L'eventuale mappatura PDF `ToUnicode` dovrebbe rendere verificabile l'estrazione dei caratteri, senza confonderla con la struttura semantica del PDF. La normalizzazione Unicode non deve cancellare differenze significative nei titoli originali; eventuale NFC deve essere scelta e documentata, non applicata tacitamente alle schede archivistiche.

## 6. Offline, sicurezza e privacy

- **Offline senza eccezioni** per composizione, diagnosi ed esportazione: nessun font scaricato in background, CDN, API di traduzione, telemetria o upload.
- **Privacy:** il testo della brochure può contenere nomi, recapiti e prezzi futuri. Nessun contenuto va in log diagnostici, crash report, cache esterne o prompt IA. Il PDF finale è una copia condivisibile **solo per scelta di chi esporta**.
- **Font e dipendenze:** esaminare licenze di ridistribuzione, obblighi di attribuzione, incorporamento in PDF, dimensione dei pacchetti e sicurezza delle librerie di parsing/rasterizzazione. Non prelevare automaticamente i font di sistema né incorporare font commerciali senza permesso. Non presupporre che un font copra tutti gli alfabeti.
- **Integrità:** testo e fotografie originali immutati, nessuna modifica al database di opere o vendite. Gli errori di font/memoria/capacità disco non devono corrompere bozza, SQLite o destinazioni esistenti.
- **Ambiente:** produzione su Windows, macOS e Linux con risultati coerenti, senza richiedere un browser headless, un'applicazione PDF a pagamento o una connessione.
- **Backup e portabilità:** le bozze restano incluse nel backup locale esistente e migrano con l'archivio. I PDF esportati fuori dall'archivio **non sono automaticamente inclusi** nel backup: deve essere comunicato, senza fingere che un file condiviso sia ripristinabile.

## 7. Impostazioni, lingue e accessibilità

- Lingua dell'interfaccia rilevata dal sistema con **fallback inglese**, scelta manuale come oggi; lingua della brochure **indipendente** (IT/EN/ES/FR/DE). Il rendering di nomi non latini **non deve essere vincolato** alla lingua selezionata.
- Diagnostica localizzata e utilizzabile da tastiera e screen reader: riferimenti al campo preciso e avvisi testuali, non solo cromatici. La scelta di font deve preferire impostazioni automatiche semplici; nessun pannello tipografico complesso necessario nell'MVP.
- PDF con testo visibile, selezionabile e copiabile è un obiettivo misurabile; ordine di lettura, tag strutturali, testo alternativo delle immagini e conformità PDF/UA richiederebbero un'analisi distinta, **non garantita da font Unicode**.

## 8. Alternative e scelte non assunte

1. **Solo avviso sui caratteri non rappresentabili:** minimo costo/risparmio di complessità, ma non risolve il problema di esportare la brochure.
2. **Font incorporati con copertura europea estesa:** progresso incrementale per nomi e diacritici, ma non supporta tutte le scritture; elenco preciso dei casi esclusi.
3. **Motore PDF locale con embedding, subset, shaping e metriche reali:** più capacità, ma maggiori dimensioni, dipendenze, rischio rendering, costi di verifica.
4. **Stampa da WebView:** potrebbe riusare il layout dell'anteprima, ma rischia differenze OS/font, dipendenze dal browser e minor determinismo; da confrontare, non scegliere per principio.
5. **Esportazione di contenuti testuali separati (PDF attuale + testo UTF-8):** workaround solo per consultazione, non sostituisce il PDF leggibile.

Nessuna alternativa è approvata. Non va abbandonata la semplicità dell'export A4 nativo senza evidenza di beneficio.

## 9. Rischi, dipendenze, complessità

| Fattore | Valutazione ipotetica |
| --- | --- |
| Complessità | **Bassa–media** per diagnostica esplicita; **medio–alta** per embedding e metriche; **alta** per shaping e bidirezionalità complete |
| Dipendenze | font redistribuibili, biblioteca PDF/font/shaping da valutare solo dopo decisione, strumenti locali di ispezione/estrazione PDF |
| Rischi | sostituzioni silenziose, font senza licenza adeguata, file grandi, font fallback discordanti, righe tagliate, PDF apparentemente valido ma non ricercabile |
| Compatibilità | bozze SQLite esistenti, layout A4, foto selezionate, PDF apribili offline; niente modifiche retroattive a schede e documenti già esportati |

L'incorporamento font può aumentare il PDF e la distribuzione desktop; il sottoinsieme di glifi può ridurre le dimensioni ma richiede una gestione corretta di fallback e mappatura caratteri. La pagina «anteprima» non prova da sola la qualità del documento esportato.

## 10. Criteri di accettazione da proporre, non test eseguiti

1. PDF con «Élodie», «Müller», «Łódź», testo greco e cirillico: nessun carattere sostituito; se non supportato, avviso **prima** del salvataggio e nessun file fuorviante.
2. Copia/incolla ed estrazione del testo mantengono i caratteri previsti, inclusi diacritici combinanti e apostrofi tipografici; differenze di normalizzazione chiaramente documentate.
3. Font incorporati effettivi verificabili da un lettore PDF indipendente; riapertura su sistemi senza quel font installato.
4. Descrizioni, codici opera, titoli lunghi, interruzioni di pagina e testi misti: nessun ritaglio non segnalato, nessuna sovrapposizione con immagini o contatti. Confronto visivo dei layout **Classica**, **A tutta pagina**, **Immagine e testo**.
5. Bozze create prima della modifica apribili senza conversione dei dati; esportazioni di base mantengono foto e ordine delle opere.
6. Errori di parsing, spazio esaurito, glifo non disponibile e file di destinazione già esistente non causano cancellazione/sovrascrittura implicita di file o dati.
7. Esecuzione offline su Windows/macOS/Linux; verifica con lettori differenti, senza dipendenza da font installati di terze parti.
8. Messaggi localizzati (IT/EN/ES/FR/DE) con fallback inglese; percorso di correzione navigabile da tastiera e leggibile da screen reader.
9. Verifica di licenze dei font e delle eventuali dipendenze prima di ogni integrazione; valutazione chiara della dimensione del programma.

## 11. Fasi suggerite, tutte subordinatamente a revisione

- **Fase 0 — misura:** raccogliere esempi **anonimizzati o sintetici** di titoli, lingue, caratteri non rappresentabili e brochure lunghe; confrontare rendering corrente e requisiti effettivi. Nessuna modifica necessaria.
- **Fase 1 — diagnostica:** rendere visibili caratteri non rappresentabili e overflow, con UX essenziale; verificare che tale protezione basti.
- **Fase 2 — resa Unicode mirata:** font redistribuibile e incorporato, mapping e metriche reali, con regressioni PDF e dimensioni del pacchetto accettabili.
- **Fase 3 — scritture complesse, solo se richieste:** shaping, bidi e fallback per script ulteriori; valutare separatamente accessibilità PDF completa e armonizzazione preview/export.

## 12. Domande riservate al proprietario

1. Ci sono già brochure destinate a nomi d'artista, titoli o mercati in alfabeti non latini? In quali scritture?
2. È preferibile un **avviso affidabile** prima dell'export o si considera irrinunciabile una vera resa Unicode offline fin dalla prima evoluzione?
3. Quale dimensione aggiuntiva del pacchetto desktop sarebbe accettabile per font incorporabili multipli?
4. È più importante mantenere l'attuale renderer Rust nativo oppure investire, dopo confronto tecnico, in un renderer locale con layout migliore?
5. Serve soltanto un PDF visivamente fedele, oppure anche testo facilmente ricercabile/copiabile e, in futuro, PDF accessibili?

**Conclusione:** una lacuna tecnica già riconosciuta e documentata, con conseguenze potenziali sulla fedeltà dei nomi, merita una valutazione puntuale. **Nessuna scelta implementativa o priorità è approvata.** La scheda non autorizza lavori sul codice.
