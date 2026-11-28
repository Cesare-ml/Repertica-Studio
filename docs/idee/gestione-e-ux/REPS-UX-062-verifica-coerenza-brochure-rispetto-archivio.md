# REPS-UX-062 — Verifica delle modifiche alle opere prima di ristampare una brochure

- **ID stabile:** REPS-UX-062
- **Data della ricognizione:** 2026-11-23
- **Stato:** Proposta — da valutare
- **Filone:** Gestione e UX; esclusivamente analisi e documentazione
- **Priorità ipotetica:** P2 solo se le ristampe di brochure esistenti sono frequenti e le differenze creano errori; altrimenti P3 o nessuno sviluppo
- **Decisione:** nessuna approvazione, assegnazione o autorizzazione all'implementazione

## 1. Problema, destinatari e fatti verificati

Un'artista prepara una brochure per una galleria e la riapre alcuni mesi dopo, quando un'opera ha cambiato titolo, tecnica, misure, descrizione o fotografia principale. Desidera **sapere quali dati appariranno nella nuova copia**, senza supporre che sia identica alla precedente, e scegliere se accettare consapevolmente le informazioni aggiornate. Il bisogno è più rilevante per artisti con brochure riutilizzate, studi e gallerie che ristampano cataloghi a più riprese; meno rilevante per chi produce un PDF e non lo riapre mai.

**Riscontri statici su `main` al 23 novembre 2026 (connettore GitHub, non collaudo nativo):**

- [Brochure offline](../../BROCHURES-OFFLINE.md) documenta un editor già funzionante, con bozze JSON locali e PDF Rust. L'anteprima Angular è **illustrativa** e i PDF esportati si trovano nella cartella scelta, **non** vengono conservati come snapshot immutabili nel database.
- [Modello TS](../../../src/app/models.ts), [editor Angular](../../../src/app/brochures.component.ts) e [renderer Rust](../../../src-tauri/src/brochures.rs): ogni elemento della brochure conserva `artwork_id`, layout, eventuale `gallery_image_id`, `title_override` e `description_override`. Le sovrascritture vuote rinviano ai valori **correnti** dell'opera; codice, anno, tecnica, supporto e misure vengono letti dall'archivio al momento della generazione. La foto principale corrente è utilizzata se non è indicata una fotografia aggiuntiva specifica.
- La logica già rileva opere o foto aggiuntive non più esistenti e può impedire l'esportazione. Non emerge un **confronto esplicito con i contenuti dell'ultima brochure verificata/esportata** che distingua modifiche deliberate, cambiamenti dinamici e media mancanti.
- Ciò è una **lacuna potenziale di revisione editoriale**, non prova di PDF alterati in passato, errore di stampa o reclami reali. Non presumere che ogni modificazione all'archivio sia dannosa: l'aggiornamento dinamico può essere proprio ciò che l'utente desidera.

## 2. Confini con le proposte già presenti — evitare duplicazioni

- **[UX-007 — Dossier e schede condivisibili](REPS-UX-007-dossier-presentazione-opere.md):** studia *quali dati* presentare e come comporre un dossier. Qui si verifica una **brochure già salvata** rispetto ai dati che verrebbero usati per una **nuova esportazione**, senza creare un secondo editor o dossier.
- **[UX-041 — Titoli/descrizioni multilingua](REPS-UX-041-titoli-descrizioni-opere-multilingua.md):** eventuali testi editoriali per lingua. Questa proposta non traduce nulla e non modifica titoli canonici.
- **[UX-059 — Controllo pre-emissione certificati](REPS-UX-059-controllo-preemissione-certificati.md):** concerne un documento **numerato e irreversibile**, mentre una brochure è una bozza modificabile con PDF derivato. Nessun seriale né attestazione storica.
- **[TECH-032 — PDF Unicode](../strumenti-e-architettura/REPS-TECH-032-pdf-brochure-unicode-font-incorporati-offline.md)** e **[TECH-034 — Export PDF atomico](../strumenti-e-architettura/REPS-TECH-034-esportazione-brochure-pdf-atomica-senza-sovrascrittura.md):** affrontano rendering e sicurezza del file; qui soltanto la **decisione editoriale prima della ristampa**. **[TECH-010 — Revisioni metadati](../strumenti-e-architettura/REPS-TECH-010-storico-revisioni-metadati.md)** considera una cronologia generale delle opere: non deve essere un prerequisito, poiché bastano confronti tra due rappresentazioni dichiarate.
- **[TECH-054 — Catalogo web statico](../strumenti-e-architettura/REPS-TECH-054-pacchetto-catalogo-web-statico-offline.md):** diverso formato/destinazione; non si introduce alcuna pubblicazione web o integrazione cloud.
- Nessun file o processo dell'altra automazione viene modificato.

## 3. Scenari concreti

1. **Titolo corretto:** una brochure conservata da mesi usa il titolo dell'archivio e non un override; l'artista aggiorna il titolo della scheda. Alla nuova esportazione il sistema indica «titolo diverso dall'ultima revisione confermata», mostra *prima/dopo*, senza correggere automaticamente alcun valore.
2. **Titolo personalizzato nella brochure:** un `title_override` editoriale non vuoto è già indipendente dal titolo dell'opera. Non segnalarlo erroneamente come «cambiamento stampato» quando varia solo il titolo sottostante. Eventualmente indicare che la brochure mantiene una variante scelta deliberatamente.
3. **Copertina sostituita:** l'elemento che usa la fotografia principale potrebbe usare la nuova immagine; una fotografia aggiuntiva fissata tramite ID non cambia selezione solo perché cambia copertina. Se il contenuto dei byte selezionati varia, distinguerlo da un cambio di ID solo se è disponibile una prova affidabile (hash).
4. **Fotografia mancante:** una fotografia aggiuntiva eliminata o un'opera rimossa deve mostrare un problema che impedisce la nuova stampa, senza fingere di recuperare il PDF precedente; la gestione dell'errore esiste già e non deve regredire.
5. **Modifiche legittime e brochure datata:** la brochure è stata creata per la mostra 2025, ma l'opera adesso ha nuove dimensioni dichiarate. L'utente può mantenere il documento storico esterno e scegliere esplicitamente una nuova versione: **nessuna riscrittura silenziosa dei PDF già distribuiti**.
6. **Prima apertura di una vecchia bozza:** se non è presente un riferimento precedente confrontabile, la schermata dice «nessuna base storica per il confronto» e offre un controllo dei valori correnti. Non afferma «nessuna modifica».

## 4. Flusso UX proposto (minimo, senza nuovo editor)

**Ingresso:** Brochure → apri bozza salvata → «Controlla prima dell'esportazione». In un eventuale futuro, il controllo può apparire come passaggio leggero prima di «Esporta PDF», non come blocco permanente alla creazione.

**Pannello di confronto:** elenco delle opere della brochure con miniature, codice identificativo, campi che incidono realmente sull'output (titolo e descrizione *effettivi*, anno, tecnica, supporto, misure, fotografia selezionata). Stati spiegati nel testo: «invariato rispetto alla revisione confermata», «cambiato», «non confrontabile», «origine mancante», «personalizzato nella brochure». Filtri *solo differenze* e *tutte*. Il confronto considera i valori effettivamente renderizzati, non campi nascosti.

**Decisione esplicita:** «Continua con valori correnti», «Torna alla bozza e personalizza», «Annulla». La modalità minima può limitarsi a revisione + conferma globale; **nessun pulsante “Ripristina scheda opera”**, nessun aggiornamento dei valori canonici, nessuna sostituzione forzata di override. Se non esiste baseline, la persona conferma solo una nuova stampa con dati attuali.

**Feedback:** data/ora dell'ultima revisione confermata (se disponibile), data della nuova esportazione realmente riuscita, numero di opere ricontrollate. Non chiamare «PDF identico» una semplice verifica dei campi: impaginazione, motore PDF e font possono essere cambiati. Per il PDF esistente richiedere sempre un file esterno e non affermare che sia riproducibile byte-per-byte.

**Accessibilità:** elenco navigabile con tastiera, intestazioni per opera, annunci screen reader per differenze e pulsanti, assenza di sole codifiche cromatiche; vista compatta a finestra ridotta, zoom 200%, testo lungo non troncato senza modo di espanderlo.

## 5. Dati e contratti soltanto descrittivi

- Possibile `brochure_review_baseline` **facoltativa**, associata a `brochure.id`: revisione/versione del formato, data di conferma, per ogni `artwork_id` i soli **valori effettivamente stampabili** selezionati, origine (corrente o override), `gallery_image_id` se presente e hash immagine se rilevabile. Alternative: incorporare un piccolo snapshot nel JSON già salvato anziché una nuova tabella. Nessuna scelta approvata.
- Una baseline **non** prova quando l'opera è stata modificata, da chi o se il PDF storico è stato effettivamente consegnato. È un confronto tra valori noti, **non un audit trail**.
- Per una bozza senza baseline: stato `unknown`, non `unchanged`. Un riferimento diventa autorevole per le future verifiche soltanto dopo conferma esplicita e, se scelta questa semantica, dopo export riuscito; errore, annullamento o destinazione non valida non avanzano la baseline.
- Se si sceglie di conservare hash, distinguere hash dei byte della **fotografia** e hash del **PDF esportato**: sono informazioni differenti. La fase minima può non includere hash né archivio PDF. Testi personalizzati e note private non devono essere copiati fuori dal documento originale senza necessità.
- Non introdurre obbligatoriamente il server MCP, revisioni generaliste, servizi online o sottoscrizioni.

## 6. Offline, impostazioni, lingue, import/export, privacy e compatibilità

- **Offline:** confronto su JSON brochure e SQLite/media dell'archivio locale, senza rete; nessun account, riconoscimento immagini IA o upload. Non aprire URL/contatti automaticamente.
- **Impostazioni:** controllo facoltativo all'esportazione (valutare *attivo per bozze con baseline* come default prudente), possibilità di avviarlo manualmente in ogni momento; nessun monitoraggio continuo o notifica dell'OS non richiesta. L'utente deve poter uscire senza modifiche.
- **Multilingua:** lingua del sistema IT/EN/ES/FR/DE e fallback inglese, con override manuale esistente. Codici di stato stabili internamente; etichette/testi di confronto tradotti, contenuto libero dell'opera non tradotto. La lingua del PDF brochure può essere diversa dalla lingua UI e non deve modificare il confronto dei valori.
- **Export/import:** il PDF continua ad essere esportato solo per scelta dell'utente in un percorso arbitrario. Un riepilogo di differenze in Markdown/CSV **solo eventuale** e con anteprima, non una seconda copia automatica di tutto l'archivio. Non esiste oggi un import di brochure da PDF: non prometterlo. La portabilità completa rimane tramite backup verificato SQLite + media; un eventuale formato aperto di interscambio segue il filone TECH-001 se in futuro approvato.
- **Privacy:** testi di opere, fotografie e recapiti sulla pagina contatti possono essere destinati a terzi; il controllo non deve aprire i dati personali dei clienti o includere fatturato, vendite, margini e ubicazioni. PDF esterni, vecchi backup e brochure già inviate non sono modificati. Ricordare che i backup attuali sono in chiaro.
- **Compatibilità:** schemi e JSON delle brochure preesistenti restano validi, `baseline` assente è trattata correttamente; supporto di backup/ripristino integrale della sola eventuale nuova metainformazione. Non cambiare significato degli override vuoti, ID opere o foto, seriali certificati, né comportamento di esportazione delle bozze legacy. Un aggiornamento di versione non deve convertire automaticamente “nessuna base” in “nessuna differenza”.

## 7. Dipendenze, complessità, rischi e alternative

- **Dipendenze certe:** funzionalità brochure già esistente, lettura locale delle schede opere e media, i18n, feedback di export. **Opzionali:** hash fotografici già riutilizzabili, storico dei metadati TECH-010, migliore preview PDF e futura esportazione aperta.
- **Complessità stimata:** **bassa–media** per revisione manuale dei dati correnti senza baseline (solo checklist); **media** per baseline facoltativa, differenze campo-per-campo e compatibilità; **medio-alta** per confronti fotografici di byte, revisione versionata e snapshot PDF. Nessuna stima di tempi di sviluppo approvata.
- **Rischi:** allarmi falsi per modifiche che l'override nasconde; mancato avviso su foto mutate; baseline sovrascritta prima di esportazione riuscita; messaggi ambigui circa identità dei PDF; falsa sicurezza della preview Angular; crescita dati o complicazioni della bozza.
- **Mitigazioni:** confrontare **valori effettivamente usati**, differenziare incertezza ed errore, catturare solo i campi necessari, non modificare l'archivio, rispettare i controlli esistenti su foto mancanti, collaudare con dati sintetici e impaginazioni diverse.
- **Alternative minime senza codice:** esportare un PDF campione dopo ogni variazione, confrontare a mano con la versione precedente e annotare la data di revisione in un nome file; usare una semplice checklist per le brochure riutilizzate. Se questa procedura basta, **nessuna funzione aggiuntiva**.

## 8. Criteri di accettazione ipotetici

1. Modificare tecnica, misure o anno in una scheda dopo una conferma produce una differenza leggibile nella brochure che usa i valori correnti; nessuna scrittura sull'opera.
2. Il cambio di un titolo dell'opera **non** genera falso avviso di testo stampato cambiato se la brochure usa un `title_override` non vuoto.
3. Copertina cambiata vs foto aggiuntiva selezionata sono trattate distintamente; se non è verificabile l'identità dei byte, la UI non afferma che l'immagine sia immutata.
4. Una bozza legacy senza baseline indica chiaramente «non confrontabile», resta salvabile ed esportabile secondo le regole precedenti.
5. Una esportazione fallita/annullata non aggiorna il riferimento alla precedente revisione; nessun PDF storico è modificato o cancellato.
6. Confronto, annullamento e conferma funzionano offline, da tastiera e con screen reader, nelle lingue supportate; fallback inglese per stringhe mancanti.
7. Dopo backup/ripristino, bozza e (se introdotta) baseline conservano il significato; un PDF esterno non è descritto come automaticamente incluso nel backup.
8. Una selezione di molte opere permette individuare differenze e dati mancanti senza forzare un editor generale né esporre contatti e importi riservati.

## 9. Fasi possibili, NON roadmap approvata

- **Validazione prima del codice:** mostrare a 4–6 professionisti due PDF della stessa brochure, con cambio di tecnica, titolo e foto; raccogliere numero di ristampe mensili, errori e utilità del controllo rispetto a una checklist. Non inventare risultati.
- **Fase A (eventuale):** pannello di controllo sui valori correnti e sorgenti mancanti prima dell'export, senza dati persistenti aggiuntivi.
- **Fase B (solo se giustificata):** confronto con baseline versionata salvata dopo conferma/esportazione riuscita.
- **Fase C (facoltativa):** hash e confronto media, eventuale identificazione di copie editoriali con report portabile; NON generare automaticamente un archivio immutabile o un servizio cloud.

## 10. Requisiti certi, ipotesi e domande per il proprietario

**Certi dal progetto:** Repertica è locale/open source; le brochure sono bozze persistenti con dati tecnici letti dall'archivio e testi editoriali opzionali; il PDF è una copia esterna; lingua di sistema con fallback inglese; backup verificato dell'archivio locale.

**Ipotesi da verificare:** la stessa brochure viene riutilizzata abbastanza spesso da creare problemi; gli utenti vogliono essere avvisati dei dati aggiornati anziché preferire sempre valori live; è utile una baseline persistente rispetto a un semplice riepilogo.

**Domande per revisione personale:** conviene partire con la sola checklist senza baseline? Una ristampa della stessa brochure dovrebbe mantenere una versione storica dei dati oppure usare normalmente sempre il catalogo aggiornato? In quali documenti andrebbe mostrata una data/etichetta di revisione? Le brochure vengono realmente ristampate da gallerie/artistə in più momenti?

**Conclusione:** **Proposta — da valutare**, nessuna approvazione, nessuna assegnazione e nessun avvio dello sviluppo.
