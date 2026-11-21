# REPS-UX-023 — Prezzi richiesti delle opere, listino operativo e storico delle revisioni

- **ID stabile:** REPS-UX-023
- **Data:** 2026-11-21
- **Stato:** Proposta — da valutare
- **Filone:** Gestione e UX
- **Priorità ipotetica:** P1–P2 per artisti e gallerie che espongono opere in vendita; P3 se i listini sono gestiti raramente. Da verificare con utenti, non approvata.
- **Natura:** esclusivamente ricerca e progettazione. Nessuna implementazione, assegnazione, issue o decisione di prodotto.

## 1. Problema professionale e destinatari

Un artista può avere un **prezzo suggerito** dalla formula interna, un **prezzo richiesto** effettivamente comunicato a una galleria, un'offerta negoziata con un singolo acquirente e infine un **prezzo di vendita** registrato. Questi sono quattro dati differenti. Se l'artista rivede un listino dopo una mostra, deve poter rispondere a domande concrete: «A quanto era proposta quest'opera a marzo?», «Qual è il listino attuale delle opere disponibili?», «Quale prezzo posso inserire nella distinta che porto in galleria?».

Destinatari: artisti che vendono direttamente, studi che preparano listini per mostre e fiere, piccole gallerie con opere in gestione. Un archivio senza attività commerciale può lasciare il pannello disattivato. L'app non deve diventare una piattaforma di quotazioni di mercato.

**Verifica del repository, 21/11/2026:** il branch main (README, docs/MVP.md, docs/ECONOMY-I18N.md, src/app/models.ts) documenta il coefficiente e il prezzo orientativo derivante da superficie × tariffa × coefficiente, la vendita EUR registrata con importo in centesimi, gli incassi e le statistiche; nel modello Artwork non appare un campo per **prezzo richiesto/listino né uno storico delle sue modifiche**. Si tratta di una **lacuna funzionale ipotetica**, non di un bug o di una richiesta già approvata.

**Indizi esterni di uso professionale, non verifica della domanda Repertica:** [ITGallery — Artwork prices](https://support.itgalleryapp.com/article/766-artwork-prices) descrive prezzi diversi per tipo e valuta sulla stessa opera; [WrkLst — works](https://wrklst.art/product/works) descrive la preparazione di listini da selezioni di opere. Non si propongono le loro architetture cloud, costi, sistemi proprietari o automatismi.

## 2. Confini anti-duplicazione

- **Funzioni già in main:** coefficiente artista e tariffa per cm² forniscono un *suggerimento calcolato*, non un prezzo di listino accettato dall'artista; la vendita registra l'importo *conclusivo*, non il prezzo richiesto prima della negoziazione; gli incassi effettivi sono un ulteriore registro separato.
- **[UX-002 — Trattative](REPS-UX-002-trattative-prenotazioni.md):** l'offerta o controfferta è **specifica a una trattativa e a un collezionista**. Il listino è una decisione commerciale generale per l'opera, anche senza acquirenti. Un'offerta non modifica mai automaticamente il listino.
- **[UX-003 — Costi e margine per opera](REPS-UX-003-costo-e-margine-per-opera.md):** calcola attribuzioni e simulazioni reddituali, ma non pubblica né storicizza il prezzo richiesto. Una simulazione non diventa listino senza conferma separata.
- **[UX-016 — Valutazioni e assicurazioni](REPS-UX-016-valutazioni-coperture-assicurative-scadenze.md):** stima, valore assicurato e listino hanno significati diversi. Nessuna copia o riconciliazione automatica tra tali importi.
- **[UX-012 — Conto vendita](REPS-UX-012-affidamenti-conto-vendita-rendiconti.md):** condizioni e percentuali dell'accordo con un intermediario non sono il prezzo pubblico della singola opera. Prezzi concordati per canale sono una fase eventuale, da coordinare con l'accordo, non una seconda contabilità.
- **[UX-007 — Dossier](REPS-UX-007-dossier-presentazione-opere.md):** definisce presentazione, impaginazione e selezione dei dati condivisi. Qui si definisce **quale prezzo è valido e divulgabile**; un futuro dossier potrà leggerlo, senza duplicare un motore di documenti.
- **Filone tecnico:** [TECH-001 — Interscambio aperto](https://github.com/Cesare-ml/Repertica-Studio/blob/docs/idee-strumenti-architettura-20261121/docs/idee/strumenti-e-architettura/REPS-TECH-001-pacchetto-dati-aperto.md), [TECH-008 — Filtri](https://github.com/Cesare-ml/Repertica-Studio/blob/docs/idee-strumenti-architettura-20261121/docs/idee/strumenti-e-architettura/REPS-TECH-008-viste-salvate-filtri-ricerca-locale.md) e [TECH-018 — CSV](https://github.com/Cesare-ml/Repertica-Studio/blob/docs/idee-strumenti-architettura-20261121/docs/idee/strumenti-e-architettura/REPS-TECH-018-esportazioni-csv-scalabili-atomiche.md) riguardano l'infrastruttura, **non** il significato commerciale dei listini. Le loro schede non vengono modificate.

**Fuori ambito:** quotazioni automatiche IA, raccolta prezzi da aste, algoritmo di mercato, fatturazione fiscale, sconti obbligatori, cambio valutario online, vendita sul web, sincronizzazione dei prezzi con terzi, modifica massiva senza anteprima, assegnazione automatica dei prezzi alle opere non prezzate.

## 3. Scenari d'uso realistici

1. **Prima mostra.** L'artista cataloga quindici opere, imposta manualmente il prezzo richiesto di dieci e lascia cinque come **prezzo su richiesta**. Stampa o esporta una distinta locale soltanto delle dieci con importo esposto e, se scelto, delle cinque marcate come non prezzate.
2. **Aggiornamento di listino.** Un prezzo passa da 1.800,00 EUR a 2.100,00 EUR con data e motivo «nuovo listino primavera». Il vecchio importo resta leggibile nello storico; il coefficiente artista non viene cambiato.
3. **Trattativa e vendita.** Un'opera proposta a 2.100,00 EUR riceve un'offerta di 1.950,00 EUR e viene venduta a 2.000,00 EUR. Il sistema mostra *richiesto*, *offerto* (solo se UX-002 fosse approvata) e *venduto* in sezioni distinte, senza riscrivere alcuno dei tre numeri.
4. **Opere non in vendita.** Per un'opera archiviata, donata o riservata alla collezione personale, non viene inventato un prezzo pari a zero. L'assenza di listino resta uno stato comprensibile.
5. **Due interlocutori commerciali.** Una galleria può richiedere un listino specifico per fiera o canale. Questa esigenza è da valutare in una **fase successiva**, senza autorizzare due prezzi «attuali» ambigui nella prima fase.

## 4. Flussi UX e regole di comportamento

**Scheda opera → Prezzo richiesto (facoltativo).** La sezione mostra distintamente: prezzo orientativo calcolato in base alle impostazioni **al momento della consultazione**, prezzo richiesto approvato manualmente e, se esiste, importo della vendita. Se il campo non è compilato: «Nessun prezzo richiesto registrato», non «€ 0». Una modalità esplicita «Su richiesta» non implica prezzo assente o gratuito.

**Imposta / aggiorna.** L'utente sceglie «Imposta prezzo richiesto», digita un importo in EUR, seleziona visibilità («interno» / «divulgabile in listino») e registra data di efficacia e motivazione facoltativa. Una conferma mostra *prima → dopo* e comunica che certificati, importi di vendita e statistiche non cambieranno. In caso di annullamento niente scritture. Nuova cifra e data producono una nuova revisione storica, non la sovrascrittura silenziosa della precedente.

**Elenco opere → Listino locale.** Filtro per disponibilità, opere con prezzo, «su richiesta», eventuale esposizione/sele­zione manuale. Tabella con codice, titolo, tecnica/misure essenziali, prezzo richiesto attuale e data di efficacia; valori interni non divulgabili nascosti per default nelle esportazioni esterne. Prima di salvare CSV o stampare/PDF: anteprima del numero di opere, colonne e dati effettivamente condivisi. Nessun invio online e nessun caricamento automatico su siti.

**Venduta / non disponibile.** Lo storico dei prezzi rimane consultabile; il listino delle opere **disponibili** esclude per default le vendute. Il prezzo richiesto storico non diventa ricavo, non crea incassi, non modifica lo stato commerciale, non genera certificati e non prova che l'opera sia stata offerta a terzi. L'eventuale scelta di pubblicare anche opere vendute deve recare un'etichetta chiara.

**Revisioni e date.** Una revisione con decorrenza futura è proposta solo per un'eventuale fase 2 e non va considerata «attuale» prima della data. Per la fase 1 preferire decorrenza immediata, con data locale esplicita. Non permettere che due revisioni si presentino entrambe come listino attuale della stessa opera. Correzione di errori materiale con rettifica motivata, non cancellazione della storia.

**Accessibilità.** Input monetario con formato locale e lettura del valore in parole/testo, errori in linea comprensibili da screen reader; navigazione e scelta opere da tastiera; focus evidente; niente differenze espresse soltanto dal colore. Anteprima stampabile e scalabile, ad alto contrasto, importi allineati e senza troncamenti.

## 5. Dati necessari e API concettuali — solo ipotesi

- **artwork_asking_price_revision:** UUID revisione; UUID opera; valore in **centesimi interi EUR** oppure *stato* esplicito «su richiesta»/«non prezzato»; data e ora di registrazione; decorrenza; motivo facoltativo; visibilità («interno» / «divulgabile»); legame opzionale alla revisione rettificata; eventuale fonte dichiarata e note riservate. Non usare importi float, stringhe localizzate o valori fittizi zero per rappresentare assenze.
- **Stato attuale derivato** dalle revisioni valide per opera; una sola scelta corrente per l'MVP. L'indicazione di attualità non presume disponibilità commerciale: questa è determinata separatamente dal campo status dell'opera. Conservare un puntatore esplicito è un'alternativa, da valutarne consistenza e atomicità.
- **price_list_selection** solo opzionale in fase successiva: UUID, nome descrittivo, opera e revisioni *congelate* se si desidera riprodurre esattamente una distinta storica. Senza snapshot una ristampa futura mostrerebbe importi aggiornati, quindi **non** va etichettata «listino originale».
- **Canale e valuta** solo eventuali fasi successive: canale codificato (studio, fiera, galleria), EUR nella prima fase; futura valuta ISO 4217 senza conversione implicita o tassi inventati. Per ciascuna combinazione da definire unicità delle revisioni attive e precedenza dei prezzi, dopo verifica dei casi reali.
- **API descrittive, non implementazioni:** «leggi prezzo corrente e storico dell'opera», «crea revisione dopo conferma», «genera anteprima listino da selezione», «esporta selezione con politiche privacy». Non riutilizzare comandi della vendita per registrare un listino.
- **Vincoli di integrità:** UUID stabili, FK su opere, rettifiche non distruttive, amount > 0 solo quando stato = «prezzo numerico», controllo overflow, ordinamento deterministico; nessun effetto su Sale, SaleReceipt, Expense, Certificate, StudioSettings o quote di guadagno.
- **Legame con la formula:** se cambia il coefficiente artista, il suggerimento ricalcolato cambia, **ma le revisioni di prezzo richiesto restano immutate**. Non clonare nella revisione l'intero ambiente del calcolo se non necessario; la provenienza del suggerimento originario è una decisione da valutare.

## 6. Impostazioni, offline, multilingua e privacy

- **Impostazioni facoltative:** mostra/nascondi listino nel catalogo; stato iniziale di visibilità (prudenzialmente **interno**); colonne preferite nella stampa; nessun prezzo obbligatorio; nessuna configurazione richiesta per usare le funzioni attuali.
- **Offline-first e indipendenza:** SQLite locale e stampa/salvataggio nativi, senza account, telemetria, abbonamento, API di quotazione, sincronizzazioni o cloud necessario. Tutto consultabile e modificabile senza Internet. Cartella destinazione export e backup scelta dall'utente; supporti USB/NAS montati solo come normali filesystem.
- **Multilingua:** rilevamento lingua di sistema, override locale IT/EN/ES/FR/DE e fallback inglese anche per stringhe mancanti, secondo docs/ECONOMY-I18N.md. Valori memorizzati in centesimi e codici non tradotti; formati e separatori visualizzati secondo lingua/locale senza cambiare valore. Titoli e motivazioni libere non sono tradotti automaticamente.
- **Privacy:** prezzo di riserva interno, motivazioni e condizioni commerciali possono essere riservati. La modalità «divulgabile» riguarda *solo* l'importo selezionato, non note, recapiti, ubicazioni o margini. Anteprima con omissioni per default e avviso prima di esportazioni esterne; non etichettare un file locale come protetto da password. I backup correnti non sono cifrati; spetta all'utente custodirli su supporti appropriati.
- **Sicurezza documentale:** un listino condiviso è un documento commerciale preparato dall'utente, non una promessa contrattuale, una fattura, un certificato di autenticità né una prova che un acquirente l'abbia ricevuto. Esplicitare data di generazione e natura del prezzo.

## 7. Import, export, backup e compatibilità

- **CSV UTF-8:** elenco con ID/codice opera, titolo, disponibilità informativa, stato prezzo, importo in centesimi, valuta EUR, data efficacia e visibilità; eventuale CSV **separato per revisioni storiche** destinato all'archivio privato. Evitare che l'export pubblico includa note interne o prezzi di riserva. Adottare le convenzioni esistenti per escaping CSV e protezione dalle formule del foglio di calcolo; non ridefinire l'infrastruttura di TECH-018.
- **JSON versionato proposto:** per round-trip completo di UUID, relazioni, stati e revisioni, coerente con TECH-001. L'export tabellare di una selezione **non** equivale al backup completo; mantenere distinti «condividi listino» e «porta via tutti i dati». Il backup integrale dell'app deve comprendere **anche prezzi non pubblici e storico**, senza omissioni nascoste.
- **Import assistito eventuale:** anteprima di righe, valute, duplicati di UUID/codici, stati «su richiesta», data di efficacia, collisioni con revisioni esistenti; conferma e transazione con rollback. Mai sovrascrivere vendite storiche o generare incassi. Il reimport di un export già importato deve essere idempotente.
- **Archivi precedenti:** migrazione soltanto additiva e nessun prezzo inventato per le opere esistenti; conservare vendite, certificati e UUID. Una vecchia versione potrebbe non aprire il nuovo schema: segnalarlo e garantire backup precedente verificato. Percorsi Windows/macOS/Linux e Unicode nelle esportazioni, importi senza perdita del centesimo.
- **Interoperabilità:** nessun vincolo con un modello commerciale terzo; il nome del canale o del listino futuro deve poter essere esportato con lo stesso valore semantico. I CSV esistenti restano integrali e inalterati.

## 8. Complessità, rischi, dipendenze e alternative

- **Complessità:** medio-bassa per una sola cifra corrente per opera; media per storico coerente + anteprima export; medio-alta per listini congelati, canali multipli, decorrenze future e import round-trip.
- **Dipendenze eventuali:** UUID delle opere, stato commerciale, SQLite versionata, i18n, dialoghi export/backup, conferme UI. UX-002/007/012 e TECH-001/018 sono **integrazioni opzionali** e richiederebbero approvazione autonoma, non sono prerequisiti da anticipare.
- **Rischio principale:** confondere prezzo suggerito, richiesto, offerta, valore stimato, vendita e incasso. Mitigazione: lessico stabile, etichette persistenti, nessuna scrittura incrociata automatica e prove di comprensione con utenti.
- **Altri rischi:** esposizione di prezzi interni, perdita di storico modificando una cifra, stampa di un listino successivamente non riproducibile, confusione fra valuta e imposte, prezzi divergenti per canali, import duplicati, illusioni di valutazioni oggettive.
- **Alternativa minima:** aggiungere solo «Prezzo richiesto EUR» facoltativo alla scheda opera senza storico; costa meno, ma dopo una revisione non consente ricostruire un listino precedente. Seconda alternativa: esportare le opere in CSV e mantenere prezzi in un foglio di calcolo esterno; nessuno sviluppo, ma più lavoro manuale e rischio disallineamento.
- **Non proporre per la prima fase:** reprice automatico in massa, calcolo automatico sconti, prezzi per ciascun collezionista, conversioni valutarie, promozioni e listini su siti esterni.

## 9. Criteri di accettazione ipotetici

1. L'utente inserisce un prezzo richiesto di 1.800,00 EUR per un'opera; l'interfaccia mantiene distinti suggerito, richiesto e realizzato e non crea vendite, incassi, spese, costi o certificati.
2. Una revisione confermata a 2.100,00 EUR lascia consultabile il precedente 1.800,00 EUR con data e motivazione; cambiare il coefficiente non altera entrambi.
3. Un'opera «su richiesta» non appare con prezzo zero; un'opera mai prezzata è distinguibile da «su richiesta».
4. Un'opera venduta resta nello storico, ma non entra nel listino predefinito delle disponibili; la vendita e i suoi importi rimangono invariati.
5. Il CSV condivisibile include solo campi scelti e prezzi marcati divulgabili. Note private, indirizzi, margini e prezzi interni non compaiono; prima della scrittura si vede anteprima dettagliata.
6. Tutti i record, inclusi i prezzi interni, tornano correttamente da un backup verificato su un altro computer, in cartella scelta; round-trip JSON eventuale conserva revisioni e UUID senza duplicazioni.
7. I controlli in centesimi gestiscono correttamente 0,01 EUR, valori molto grandi, Unicode, date, casi di input invalido; nessuna precisione monetaria persa.
8. Interfaccia e documento esportato sono utilizzabili da tastiera/screen reader, con alto contrasto e fallback inglese quando mancano traduzioni; nessuna rete richiesta.
9. Non è possibile produrre due revisioni concorrenti presentate entrambe come «correnti» per la stessa opera nella fase iniziale.

## 10. Fasi proposte (non programmate)

- **A — Validazione del bisogno e soluzione minima:** interviste a 3–5 artisti/galleristi con listini reali; prototipo cartaceo della distinzione suggerito/richiesto/venduto. Se basta un foglio esterno, sospendere la proposta.
- **B — Solo prezzo richiesto locale e storico:** una condizione corrente per opera, revisione esplicita, pannello opera e filtro listino, senza listini per canale o integrazioni commerciali.
- **C — Esportazione e selezioni:** CSV/PDF/stampa locale con anteprima dei campi; eventuale fotografia immutabile del listino solo se viene richiesta la ristampa identica di documenti storici.
- **D — Opzioni avanzate condizionate:** listini per canale, periodi di efficacia e valuta, solo dopo casi reali e risoluzione delle collisioni; nessun cambio di valuta automatico.

## 11. Requisiti verificati, ipotesi e domande per revisione personale

**Requisiti certi:** Repertica è desktop Tauri/Angular/Rust, gratuito/open source, archivio SQLite locale; la formula del coefficiente è già prevista e usata per il suggerimento; vendite e incassi sono registrati separatamente; backup/ripristino ed esportazione locali; nessun account obbligatorio; lingua OS e fallback inglese. La presenza del bisogno *non* è un requisito già deliberato.

**Ipotesi non validate:** utenti con listini cambiati nel tempo; necessità di preparare una distinta per fiere; interesse per la cronologia; valore del prezzo per canali; eventuale necessità di riprodurre un listino identico dopo mesi.

**Domande al proprietario, senza assumerne la risposta:** (1) Vuoi un solo prezzo richiesto per opera o listini diversi per galleria/fiera? (2) Il prezzo «su richiesta» deve essere distinto da «non prezzato»? (3) Lo storico serve dalla prima release della funzione o è sufficiente in seguito? (4) Importo lordo di eventuali imposte e commissioni oppure base commerciale non fiscale? (5) Le opere riservate devono entrare nel listino predefinito? (6) Serve un documento di listino **congelato** per poterlo ristampare identico?

**Esito:** soltanto studio. **Stato: Proposta — da valutare**. Nessuna funzionalità approvata, issue operativa aperta, sviluppo o merge.
