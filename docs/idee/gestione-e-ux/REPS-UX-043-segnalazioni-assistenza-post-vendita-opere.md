# REPS-UX-043 — Segnalazioni post-vendita e richieste di assistenza riferite alle opere

- **ID stabile:** REPS-UX-043
- **Data della ricognizione:** 2026-11-22
- **Stato:** Proposta — da valutare
- **Filone:** Gestione e UX
- **Priorità ipotetica:** P2 solo per studi/gallerie che ricevono richieste ricorrenti; **P3** se bastano note e collegamenti manuali. Nessuna priorità approvata.
- **Natura:** studio esclusivamente documentale; non autorizza implementazione, issue operativa, approvazione o assegnazione.

## 1. Problema, destinatari e riscontro sul prodotto esistente

Dopo la consegna di un'opera un collezionista può segnalare un difetto della cornice, richiedere istruzioni di manutenzione o chiedere una valutazione circa un intervento conservativo. **Ricevere una segnalazione non significa riconoscere un difetto, assumere una garanzia, accettare un reso o confermare uno stato di conservazione**. Per un artista indipendente può bastare una nota; per una piccola galleria o uno studio che segue più clienti nel tempo è utile distinguere richiesta, risposta promessa, valutazione ed esito effettivamente documentato.

**Riscontro statico su `main` il 22/11/2026, non test desktop:** `src/app/models.ts` distingue opere, collezionisti, vendite, certificati e incassi; `src/app/archive-filters.ts` offre ricerca locale e un filtro per collezionisti senza email/telefono. `docs/MVP.md` stabilisce una vendita per opera, certificati con snapshot immutabile e assenza di cloud obbligatorio. `docs/ECONOMY-I18N.md` distingue vendite da incassi; `docs/DATA-PORTABILITY.md` elenca i registri CSV attuali, senza un registro di richieste assistenza per opere. La presenza di client IMAP/SMTP professionali non costituisce una cronologia di casi collegati a opere o vendite.

**Destinatari da verificare:** artisti con collezionisti di lungo periodo, gallerie che gestiscono rapporti dopo la consegna, piccoli archivi con richieste di cura o verifica. Non è dimostrato che la frequenza delle richieste giustifichi un modulo: servono esempi reali da professionisti, non una stima inventata.

## 2. Confini anti-doppione: che cosa questa idea NON fa

- **[UX-013 — Resi, annullamenti e rettifiche](REPS-UX-013-resi-annullamenti-rettifiche-vendite.md):** tratta il mutamento **commerciale** di una vendita e le sue conseguenze economiche. Qui si registra una **domanda di assistenza** e il relativo seguito, anche se non comporta alcun reso, correzione o rimborso. Una richiesta di reso eventuale rinvia alla pratica UX-013, senza duplicare movimenti o importi.
- **[UX-004 — Rapporti di condizione](REPS-UX-004-rapporti-stato-conservazione.md):** descrive un'osservazione professionale sulla conservazione, con data e fonte. Un messaggio «vedo un alone» resta **affermazione del segnalante**; non genera automaticamente un rapporto tecnico o una diagnosi.
- **[UX-025 — Istruzioni di cura](REPS-UX-025-istruzioni-allestimento-cura-opera.md):** conserva istruzioni associate all'opera. Qui si segue il rapporto con chi ha chiesto supporto; eventualmente si collega a istruzioni già approvate, senza riscriverle.
- **[UX-001 — Movimentazioni](REPS-UX-001-provenienza-movimentazioni.md)** registra consegna, presa in carico e rientro **fisici**. «Da ritirare per intervento» non significa «ritirata» e non sposta l'opera.
- **[UX-014 — Rubrica professionale](REPS-UX-014-rubrica-relazioni-professionali.md)** registra contatti/interazioni generali. Una richiesta assistenza ha invece un oggetto, una domanda, un responsabile dichiarativo e un esito; eventuali interazioni sono **riferimenti**, non una seconda rubrica.
- **[TECH-015](https://github.com/Cesare-ml/Repertica-Studio/blob/docs/idee-strumenti-architettura-20261121/docs/idee/strumenti-e-architettura/REPS-TECH-015-diagnostica-assistenza-locale-report-privato.md)** riguarda la diagnostica del software, non il servizio ai collezionisti. **TECH-016** studia il contenitore tecnico per allegati locali; questa scheda ne definisce solo l'uso eventuale in una pratica.
- Nessuna emissione di fatture, riconoscimento automatico di garanzia, responsabilità legale, preventivi vincolanti, pagamento, consegna, restauro, invio email o estrazione automatica di conversazioni.

## 3. Casi d'uso verificabili e limiti semantici

1. **Cornice danneggiata segnalata:** dopo una vendita l'acquirente telefona dicendo «la cornice presenta un graffio». L'artista annota la dichiarazione con la sua data e l'opera correlata, senza cambiare il campo `framed`, lo stato dell'opera o certificare un danno.
2. **Richiesta di manutenzione:** un collezionista chiede come pulire una scultura. L'utente collega eventuali istruzioni UX-025 o annota una risposta inviata manualmente, senza fingere che il cliente l'abbia ricevuta/letta.
3. **Verifica prima di decidere:** una galleria chiede fotografie dell'opera e concorda una valutazione. Lo stato è «in valutazione», non «riparazione approvata». Foto/indicazioni sono **dichiarazioni**, non perizie.
4. **Richiesta proveniente da persona diversa dall'acquirente originario:** un nuovo detentore di un'opera cita il codice archivio; nessuna inferenza che sia proprietario, acquirente o avente diritto. L'utente può tenere un contatto testuale separato o un riferimento esplicito da confermare.
5. **Assistenza senza intervento:** una richiesta si chiude con informazioni fornite, impossibilità di verificare o scelta di non procedere, conservando cronologia e motivazione. Non viene creato alcun reso o movimento.
6. **Due richieste sulla stessa opera:** nello stesso anno arrivano due segnalazioni indipendenti; non devono fondersi perché condividono opera, nome cliente o argomento.

## 4. UX candidata, volutamente leggera

**Accesso facoltativo:** `Opere → Dettaglio → Richieste e assistenza` e, se associata a una vendita, collegamento anche da `Vendite → Dettaglio`. Un filtro «richieste aperte» può comparire in una sezione discreta; nessuna nuova homepage complessa nella fase minima.

**Apertura richiesta:** «Registra segnalazione» → scegliere opera già esistente (obbligatoria nella prima fase), origine della dichiarazione (*collezionista*, *altra persona*, *osservazione dello studio*), riferimento facoltativo a vendita/collezionista, data in cui il problema è stato **riferito**, breve oggetto e descrizione. Prima del salvataggio, avviso: «Questa è una segnalazione dichiarata, non una constatazione di difetto o garanzia».

**Seguire la pratica:** timeline di eventi brevi **annotazione, risposta dichiarata, valutazione richiesta, appuntamento concordato, decisione, chiusura**. Stato *aperta*, *in valutazione*, *in attesa di risposta*, *conclusa*, *chiusa senza intervento*. Ogni evento mostra data dell'evento (riferita dall'utente) e data di registrazione distinte. La prossima azione con data è facoltativa; un termine trascorso non certifica inadempimento.

**Collegamenti senza automatismi:** da una richiesta, l'utente può **aprire** la scheda opera, la vendita o il collezionista originali. Se in futuro approvate UX-004/013/001, mostrare riferimenti espliciti alle relative pratiche; nessuna copia automatica dei record o variazione a stati commerciali, disponibilità, finanze o ubicazioni.

**Correzioni prudenti:** non cancellare silenziosamente una richiesta chiusa, specie se condivisa con altri. Correzione datata con nota e riferimento all'evento precedente; «chiusa» non implica «garanzia adempiuta». Se un contatto viene eliminato nel tempo, conservare un riferimento comprensibile senza riutilizzare il suo UUID.

**Accessibilità:** layout compatto con ordine di tab logico, scorciatoie opzionali, messaggi non basati sul solo colore, focus sul primo campo errato, lista e timeline utilizzabili da tastiera/screen reader, zoom 200%, testo di stato distinto dall'icona, date nelle lingue supportate. Nessuna notifica desktop o email senza consenso futuro esplicito.

## 5. Dati minimi e ipotesi d'interfaccia (NON migrazione o API approvate)

**Richiesta** (proposta descrittiva): `case_id` UUID, `artwork_id` UUID, `sale_id` UUID facoltativo, `collector_id` UUID facoltativo e verificato, `reporter_label` testo facoltativo per un soggetto non identificato, `subject`, `description`, `reported_on`, `opened_at`, `status`, `next_action_on` facoltativa, `closed_at` e `closure_reason`. **Mai inferire `collector_id` solo da nome/email** o imporre che chi segnala sia chi ha comprato.

**Evento**: `event_id`, `case_id`, tipo, `event_on` (data dichiarata), `recorded_at` (timestamp locale di registrazione in formato stabile), testo breve, origine (*segnalante* / *operatore*), riferimento a rettifica/evento precedente. Nessuna attestazione esterna di ricezione/avvenuta consegna. Eventuali riferimenti a un documento/foto locale gestiti come link stabili **solo se** esiste un contenitore di allegati adatto; non richiede nuove foto nel MVP.

**Invarianti:** più richieste per opera e vendita; nessuna pratica altera righe `sales`, `sale_receipts`, `sale_finances`, `expenses`, `certificates` o `artworks.status/location`; seriali e snapshot certificati rimangono invariati. La chiusura non cancella la cronologia. Cancellazioni delle entità referenziate richiedono protezione, disassociazione deliberata o conservazione di un riferimento storico leggibile, **mai riassegnazioni tacite**.

**Operazioni concettuali:** «elenca richieste dell'opera», «apri richiesta», «aggiungi evento/precisa dichiarazione», «chiudi con motivo» e «esporta selezione». Sono descrizioni di UX, **non comandi Tauri o API da implementare**.

## 6. Impostazioni, offline, lingua, privacy e portabilità

- **Impostazioni:** modulo facoltativo/nascosto se non usato, categorie di segnalazione minime e localizzate, ordinamento per data/stato. Nessuna scelta obbligatoria di SLA, garanzie legali, responsabile account o gestione ticket multiutente. Se utile in futuro, reminder soltanto locale e opt-in, preferibilmente visibile all'apertura.
- **Offline-first e indipendenza:** tutto su SQLite locale, con allegati eventuali nel media store; nessun account, abbonamento, cloud, telemetria, classificazione IA o consultazione delle email. La posta IMAP/SMTP esistente resta un flusso separato e su iniziativa dell'utente; non raccogliere automaticamente contenuti di caselle o allegati.
- **Multilingua:** rilevamento lingua del sistema, IT/EN/ES/FR/DE con override locale e **fallback inglese** per lingue non supportate o chiavi mancanti; codici di evento/stato invarianti in DB/export, date ISO 8601, nomi e note nella lingua originale (nessuna traduzione automatica).
- **Privacy:** eventuali fotografie di interni, recapiti, dati sul domicilio e dettagli su controversie sono sensibili nel contesto; richiedere il minimo necessario e non includerli in brochure, cataloghi pubblici, pagine Sites, email, MCP o export generale per default. Nessun invio a terzi implicito. I backup correnti sono **in chiaro** e possono contenere questi dati: avviso su cartelle USB/NAS condivise.
- **Esportazione:** un eventuale CSV UTF-8 dedicato delle richieste con UUID, data, codice opera, stato e oggetto (senza contatti/note per default); JSON versionato per richieste/eventi/connessi, selezionabile e con anteprima privacy, evitando di chiamare CSV «backup». L'export completo di tutte le proprie informazioni mantiene UUID, collegamenti e cronologia attraverso il **backup integrale SQLite + media** in cartella liberamente scelta.
- **Importazione:** solo se successivamente motivata da casi reali, tramite anteprima e risoluzione manuale di UUID di opere/vendite/collezionisti, idempotenza, rollback integrale in caso di errore e nessuna creazione silenziosa di contatti. Non alterare i 12 registri CSV esistenti né presumere che siano reimportabili.
- **Compatibilità:** estensione additiva di schema **solo se deliberata**; archivi storici privi di richieste rimangono privi di richieste e validi, senza casi generati a partire dalle note. Valutare avvisi per vecchie versioni dell'app che non leggono schemi futuri; preservare vendite uniche per opera, certificati immutabili, filtri correnti, Unicode, backup/ripristino e comportamento Windows/macOS/Linux.

## 7. Dipendenze, complessità, rischi, alternative

**Dipendenze esistenti:** catalogo opere e UUID, vendite e collezionisti, Tauri/Rust/SQLite, i18n, esportazione e backup locale. UX-013, UX-001, UX-004, UX-014 e UX-025 **non sono prerequisiti per una prima versione**: sono solo possibili link futuri. TECH-016 è rilevante soltanto se si desiderano allegati gestiti.

**Complessità:** **bassa** per una nota strutturata e un filtro locale; **media** per registro con timeline/stati, cancellazioni referenziali e export privacy-aware; **medio-alta** per allegati, migrazioni, ricongiunzioni con altri moduli o import selettivi. Non promettere implementazione economica senza prototipi.

**Rischi:** sembrare una procedura di garanzia legale; confondere dichiarazioni e diagnosi; aprire troppe voci nel menu; duplicare la rubrica, i rapporti di conservazione o i resi; memorizzare dati personali non necessari; falsi promemoria di risposta; classificazione erronea di chi possiede l'opera; esporre segnalazioni private tramite export o assistenti IA.

**Alternative, dalla meno onerosa:** (A) usare una nota del collezionista o della scheda opera con data, senza alcun nuovo modulo; (B) aggiungere una lista leggera di «richieste aperte» con oggetto e prossimo passo, senza stati complessi; (C) registro completo con timeline solo se prove d'uso dimostrano richieste ricorrenti e perdite informative. Strumenti esterni restano opzionali, senza sincronizzazioni obbligatorie.

## 8. Criteri di accettazione ipotetici

1. Una richiesta «cornice graffiata» collegata a un'opera venduta rimane **solo segnalazione**: non modifica vendita, incassi, quote, stato, ubicazione, certificato o registro conservativo.
2. Due richieste sulla stessa opera rimangono distinte, con UUID diversi e cronologie separabili.
3. Un segnalante che non coincide con l'acquirente può essere registrato come **non verificato**, senza attribuzione automatica di proprietà o consenso a comunicazioni.
4. Un appuntamento previsto non diventa movimento/consegna avvenuta; una risposta annotata non diventa email inviata/ricevuta.
5. Chiusura «nessun intervento» conserva motivazione, eventi e date; nessun credito, rimborso, debito o garanzia è dedotto.
6. Una scheda storica priva di pratiche non genera casi sintetici e non cambia il comportamento dell'archivio.
7. Un export destinato a presentazione dell'opera non include richieste o dati privati. Esportazioni dedicate richiedono anteprima, scelta esplicita e gestione sicura delle formule CSV.
8. Backup/ripristino completo nella cartella scelta ricostruisce i legami e la timeline; import/restore falliti non lasciano richieste parziali.
9. Tutti i flussi della fase scelta funzionano offline, su Windows/macOS/Linux, con tastiera/screen reader, IT/EN/ES/FR/DE e fallback inglese; nessun accesso MCP/email automatico ai contenuti.

## 9. Fasi e decisioni riservate al proprietario

- **A — Ricerca, senza sviluppo:** 4–6 colloqui con artisti e gallerie sui casi reali degli ultimi 12 mesi; contare richieste e canali, distinguere richieste di cura da resi e segnalazioni informali; verificare se bastano note con data.
- **B — Eventuale minimo:** registro locale di segnalazione con oggetto, opera, data, descrizione e stato semplice; nessun allegato, email o logica di garanzia.
- **C — Eventuale estensione:** timeline, prossima azione facoltativa e collegamenti espliciti ai moduli approvati senza cambiare i loro dati.
- **D — Solo su necessità dimostrata:** esportazione selettiva, allegati e modelli di comunicazione manuale; nessuna automazione di risposta.

**Requisiti certi dal repository:** archivio SQLite e media locali, UUID, collegamenti opera-vendita-collezionista, certificati immutabili, vendite e incassi distinti, lingua di sistema/fallback EN, backup completo in cartella scelta; modulo post-vendita specifico non emerso dai percorsi consultati.

**Ipotesi non verificate:** la richiesta post-vendita si ripete abbastanza da giustificare un registro; gli utenti distinguerebbero utilmente «ricevuto», «in valutazione» e «concluso»; una vista separata ridurrebbe messaggi smarriti.

**Domande per revisione personale:** con quale frequenza i professionisti seguono interventi dopo una vendita? Preferisci una scheda leggera sulla singola opera o un elenco traversale? È opportuno riferirsi anche a opere non vendute o a detentori non verificati? Serve conservare traccia della comunicazione inviata manualmente? Basta un promemoria locale visibile all'apertura? Quali dati personali sono davvero indispensabili?

**Stato finale: Proposta — da valutare. Nessuna approvazione o assegnazione allo sviluppo.**
