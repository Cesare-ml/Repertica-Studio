# REPS-UX-071 — Tornare al punto di consultazione senza perdere ricerca e filtri

- **ID stabile:** REPS-UX-071
- **Data ricognizione:** 2026-11-23
- **Stato:** **Proposta — da valutare**
- **Filone:** Gestione e UX; documento di analisi, **non approvato, non assegnato e non autorizzato allo sviluppo**
- **Priorità ipotetica:** **P2 soltanto se la perdita del contesto è osservata nei test d'uso**; altrimenti P3, guida o nessuna nuova funzione.

## 1. Problema concreto e persone interessate

Un artista con molte opere filtra il catalogo, apre un'opera venduta, passa alla scheda del collezionista e desidera riprendere dalla stessa selezione. Un assistente di galleria consulta una scheda, risponde a una domanda e torna alla ricerca iniziale senza ricordare tutte le condizioni impostate. Per piccoli archivi il problema può essere trascurabile; è **un'ipotesi di costo operativo**, non una segnalazione verificata degli utenti.

**Riscontro statico sul branch `main` al 23/11/2026:** in `src/app/app.component.ts`, `navigate(section)` cambia sezione e azzera sempre `search`, e quando entra in `artworks` ripristina `artworkStatusFilter='all'`, `artworkCollectorFilter='all'` e `artworkOrder='recent'`. L'interfaccia contiene collegamenti contestuali `showCollectorArtworks(collectorId)` e `openCollectorFromArtwork(artworkId)`; `app.component.html` offre ricerca e filtri delle opere, ricerca dei collezionisti e accesso al visualizzatore. La navigazione ordinaria usa lo stato `section`, non è dimostrata una cronologia di browser per il ritorno. **Conseguenza verificabile nel codice:** l'entrata in una sezione cancella i filtri precedenti; **non è dimostrato** quanto questo disturbi utenti reali. `archive-filters.ts` già offre ricerca e filtri locali utilizzabili: non sono una funzionalità da reinventare.

**Destinatari da consultare:** artisti con cataloghi grandi, addetti di galleria, archivisti, collaboratori che verificano acquirenti e storico commerciale durante una consultazione.

## 2. Confini e confronto con le proposte esistenti

- **[REPS-TECH-008](https://github.com/Cesare-ml/Repertica-Studio/blob/docs/idee-strumenti-architettura-20261121/docs/idee/strumenti-e-architettura/REPS-TECH-008-viste-salvate-filtri-ricerca-locale.md)** studia **viste denominate e criteri persistenti** e nuovi filtri avanzati. Qui si valuta soltanto il **ritorno contestuale temporaneo** alla lista da cui l'utente è partito, senza viste salvate, nuove query, nuovo motore di ricerca o nuove impostazioni obbligatorie.
- **[REPS-TECH-020](https://github.com/Cesare-ml/Repertica-Studio/blob/docs/idee-strumenti-architettura-20261121/docs/idee/strumenti-e-architettura/REPS-TECH-020-accessibilita-flussi-critici-tastiera-screen-reader.md)** definisce un contratto generale per il focus. Qui il recupero del focus è applicato al **percorso lista → collegamento → ritorno**, con vincoli professionali di contesto, senza riscriverne la specifica tecnica.
- **[REPS-UX-060](REPS-UX-060-attivita-registrate-opera-vista-unificata.md)** studia una vista per le attività di una singola opera e menziona il ritorno da quella vista: **non** esiste ancora su `main` ed è un flusso specifico. Questa scheda riguarda la consultazione **già esistente** fra elenco opere, visualizzatore, collezionista e opere acquisite; non introduce alcun registro attività.
- **[REPS-UX-067](REPS-UX-067-protezione-modifiche-non-salvate-moduli.md)** riguarda la protezione di bozze non salvate. Questa scheda **non** autorizza ad abbandonare editor con modifiche non salvate né propone autosave; la precedenza della protezione dalle perdite resta invariata.
- Non vengono duplicati filtri, report, vendite, incassi, certificati, movimenti fisici, schede cliente o log di operazioni. Nessun documento dell'altro filone viene modificato.

## 3. Casi d'uso e flussi UX ipotetici

1. **Dalle opere al collezionista e ritorno.** L'utente cerca `acquerello`, seleziona stato `Vendute`, ordina per titolo, scorre fino a un'opera e apre il collezionista associato. Dalla scheda collezionista usa l'azione esplicita **«Torna ai risultati delle opere»**. Ritrova query, stato, ordine e, se possibile, la riga di partenza. Il ritorno **non** assume che il collezionista coincida con ogni opera mostrata.
2. **Dal collezionista alle sue opere e ritorno.** Una persona apre un contatto trovato con un filtro, consulta le opere acquistate e poi sceglie **«Torna ai contatti»**. Riappare la ricerca del contatto precedente; l'eventuale filtro `collectorId` applicato alle opere nel passaggio intermedio non contamina la ricerca generale.
3. **Entrata intenzionale dal menu principale.** Fare clic direttamente su **Opere** o **Collezionisti** nel menu resta, almeno in fase iniziale, un'apertura *neutra* come oggi: non riattiva una vecchia ricerca non richiesta. La conservazione del contesto è riservata a un **percorso di ritorno esplicito**; valutare con utenti se serva invece un comando distinto «Riprendi ricerca».
4. **Dati cambiati mentre si consulta.** Se l'opera è stata modificata o cancellata, rieseguire i filtri sulle **informazioni correnti**; non presentare snapshot obsoleti come catalogo valido. Se la riga-ancora non esiste più, tornare alla lista con i criteri originali e un messaggio sobrio; nessun salto verso un'altra opera con lo stesso titolo.
5. **Abbandono intenzionale / nuova attività.** Il menu, `Azzera filtri` e un nuovo percorso di lavoro non devono sembrare un «indietro» e non devono far riapparire accidentalmente query personali o uno stato di un altro archivio.
6. **Tastiera e lettori assistivi.** Il ritorno deve avere un'etichetta contestuale esplicita; il focus si riporta alla riga (o al titolo della lista se la riga manca), con annuncio discreto del numero aggiornato di risultati. Non far affidamento solo sulla posizione dello scroll, sul colore o sul pulsante Back del browser.

**UX minima:** nessuna barra di navigazione complessa; un solo collegamento contestuale `Torna ai risultati` quando esiste un'origine valida. Se manca l'origine, semplice collegamento `Vai all'archivio` che non promette recupero. Preferire il comportamento attuale alla restaurazione silenziosa di un contesto vecchio.

## 4. Requisiti certi, ipotesi e dati descrittivi

**Vincoli certi del prodotto:** app Tauri gratuita/open source, database SQLite locale, nessun account/abbonamento/cloud necessari, archivi locali ed esportazione completa, backup/ripristino su cartella scelta, lingua di sistema con fallback inglese. Queste proprietà non possono essere degradate dalla proposta.

**Fatti verificati sul codice:** esistono filtri e ordinamento in memoria; il metodo `navigate` li azzera; i collegamenti collezionista/opera sono presenti; le entità usano identificativi locali. Non è stato svolto un test con utenti o una verifica della gestione del focus sul desktop installato.

**Ipotesi di modello UI, non schema approvato:**

```text
ReturnContext {
  source_section: "artworks" | "collectors",
  destination_section: "collectors" | "artworks",
  source_query: string,
  source_filters: { status?, collector_id?, collector_kind?, order? },
  anchor_record_id?: UUID,
  anchor_offset_hint?: number,
  source_archive_identity?: opaque_local_id
}
```

Il contesto conterrebbe **solo criteri di visualizzazione e un riferimento stabile alla riga**, non copie di opere, fotografie, vendite o profili. Idealmente **effimero e in memoria**, con profondità limitata (per esempio un solo ritorno contestuale; niente pila infinita); non è proposta una tabella SQLite. Se l'identità dell'archivio non è disponibile in modo affidabile, invalidare il contesto a ogni riapertura/ripristino/cambio archivio invece di indovinare.

**Interfacce ipotetiche, solo descrittive:** `openRelatedRecord(target, optionalOriginContext)`; `returnToOrigin(context) -> {restored, reason?}`; `discardReturnContext()`. Nessuna nuova API nativa, sincronizzazione, tracciamento eventi o autorizzazione a sviluppare.

## 5. Impostazioni, offline, multilingua, import/export, privacy

- **Impostazioni:** preferibilmente nessuna nuova opzione nella prima fase. Se la persistenza delle ricerche tra sessioni viene desiderata, trattarla come decisione distinta da verificare anche contro TECH-008.
- **Offline:** tutte le transizioni sono locali; nessuna rete, utente esterno, cookie di ricerca remoto o dipendenza da servizi.
- **Lingue:** etichette e avvisi in italiano/inglese/spagnolo/francese/tedesco dove supportate dall'app; la lingua del sistema è quella iniziale e una lingua non supportata ricade sull'inglese. I valori dei campi, i titoli originali delle opere e i nomi dei clienti **non vengono tradotti**.
- **Import/export:** nessun nuovo tipo di dato dell'archivio. I CSV e i backup completi già disponibili restano invariati; un contesto **solo transitorio** non è una registrazione gestionale da esportare o importare. Se in futuro si decidesse di salvare viste permanenti, la portabilità farebbe parte di TECH-008, non di questa scheda.
- **Privacy e sicurezza:** ricerche e UUID possono rivelare nomi o interessi; non salvare query in log, telemetria, URL condivisibili, clipboard o notifiche. All'uscita dall'app il contesto sparisce. In caso di cambiamento archivio, nessun ritorno verso record di un'altra base dati; mai interpretare criteri come istruzioni SQL.
- **Compatibilità:** nessuna migrazione schema e nessuna modifica delle relazioni autorevoli fra opere, clienti e vendite. Verifica su Windows/macOS/Linux e finestre strette/zoom; la navigazione interna oggi basata su `section` non deve presupporre che il sistema operativo esponga una cronologia browser utile.

## 6. Alternative, dipendenze, complessità e rischi

- **Alternativa zero sviluppo:** rientrare in Opere e ripetere ricerca e filtri; provare in sessioni reali se il costo giustifica una nuova affordance.
- **Alternativa minima:** conservare temporaneamente query/stato/ordine solo dopo un collegamento opera↔collezionista e aggiungere un pulsante di ritorno, senza memorizzare scroll o creare una navigazione generale.
- **Alternativa più ampia, non richiesta:** navigazione gerarchica con breadcrumb o cronologia interna globale. È probabilmente eccessiva per un'app MVP; rischia interazioni inattese con dialoghi, dati non salvati e altre sezioni.
- **Dipendenze:** stato attuale Angular e identificativi stabili esistenti; coordinare in futuro le regole di focus con TECH-020, senza modificarne i file. Nessun servizio, libreria o pacchetto esterno necessari **per la sola fase concettuale**.
- **Complessità stimata:** **bassa–media** per un ritorno contestuale limitato; **media–alta** se si promettono scroll esatto, stack profondo, focus robusto e ripresa dopo ripristino.
- **Rischi:** ricerche personali mostrate per errore, filtro stale che sembra nascondere opere, ancora UUID cancellata, ritorni circolari, passaggio non intenzionale fra archivi, interferenza con la conferma di modifiche non salvate, aspettative di cronologia universale. Mitigazioni: pulsante esplicito, contesto limitato, invalidazione rigorosa, risultati sempre ricalcolati, messaggi senza dettagli privati.

## 7. Criteri di accettazione e prova con professionisti (solo proposta)

1. Da un insieme sintetico di almeno 100 opere, cercare parola+stato+collezionista+ordine, aprire un contatto collegato e usare il ritorno: **stessi criteri**, elenco ricalcolato, focus accessibile sull'opera-ancora o fallback spiegato.
2. Da un elenco di collezionisti filtrato, aprire le opere acquistate e tornare: la ricerca contatti originaria ricompare, senza essere sostituita dalla query opere.
3. Aprire direttamente `Opere` dal menu: nessun ripristino di una ricerca precedente senza segnale chiaro; `Azzera filtri` continua a essere prevedibile.
4. Rimuovere/modificare durante il percorso l'opera-ancora (in dati di test) e ritornare: nessuna selezione su un altro UUID, risultati aggiornati e messaggio proporzionato.
5. Simulare nuovo avvio, cambio archivio e ripristino: contesto transitorio invalidato; nessun nome cliente nei log o in un export.
6. Verificare ritorno con tastiera, screen reader, lingue IT/EN/ES/FR/DE, zoom elevato e finestra ridotta su desktop nativo. Non dichiarare accessibilità conforme senza prove.
7. **Validazione prodotto prima di implementare:** osservare 4–6 professionisti mentre effettuano 5–10 passaggi opere↔clienti con l'interfaccia attuale; registrare numero di ricerche rifatte, tempo e confusione, poi confrontare un prototipo documentale. Se il problema non emerge, chiudere l'ipotesi senza sviluppo.

## 8. Fasi ipotetiche e domande riservate al proprietario

- **Fase 0 — verifica del bisogno:** sessioni osservate e prova con la UI corrente, senza modificare prodotto.
- **Fase 1 — eventuale MVP:** ritorno contestuale singolo opere↔collezionisti, recupero di query/filtri/ordine e selezione tramite UUID; nessun dato persistente.
- **Fase 2 — eventuale affinamento:** posizione di elenco e focus robusti, gestione stati anomali, supporto a ulteriori percorsi *solo* se frequenti.
- **Fase 3 — da non presumere:** preferenze persistenti, ricerche nominate o stack universale: valutazione separata con TECH-008.

**Domande non risolte, senza richiesta di risposta immediata:** i professionisti preferiscono che il menu laterale azzeri sempre i filtri? Quanto spesso passano da opere a contatti e ritorno? È sufficiente conservare i criteri senza scroll preciso? Una sola azione di ritorno basta? Devono essere inclusi vendita/certificato o prima è meglio limitarla a opere↔contatti? Come invalidare con certezza lo stato dopo il ripristino di un archivio?

**Decisione:** nessuna idea approvata, nessuna assegnazione, nessuna attività di sviluppo avviata.
