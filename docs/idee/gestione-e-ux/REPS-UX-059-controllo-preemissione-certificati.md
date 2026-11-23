# REPS-UX-059 — Controllo finale e anteprima prima dell'emissione irreversibile di un certificato

- **ID stabile:** REPS-UX-059
- **Data ricognizione:** 2026-11-23
- **Stato:** Proposta — da valutare
- **Filone:** Gestione e UX
- **Priorità ipotetica:** P1–P2 per la prevenzione degli errori nei documenti immutabili, **da confermare** con professionisti; P3/nessuna funzione se è sufficiente il controllo manuale attuale
- **Natura:** esclusivamente ricerca e documentazione. **Non** è una decisione del proprietario, una richiesta di implementazione né un'approvazione.

## 1. Evidenza sul prodotto, problema e destinatari

Un certificato d'autenticità numerato contiene una fotografia e metadati riferiti a una precisa opera, oltre a intestatario, emittente, lingua, data e altre dichiarazioni. Una volta emesso non deve essere trattato come una bozza modificabile: un refuso nelle misure o nella foto può restare nello snapshot storico, anche se in seguito la scheda dell'opera viene corretta. Il controllo **prima** dell'emissione ha quindi una natura diversa dalla rettifica **dopo** l'emissione.

**Ricognizione statica sul branch main, 23/11/2026:** [modelli Angular](../../../src/app/models.ts), [finestra di emissione](../../../src/app/app.component.html), [flusso Angular](../../../src/app/app.component.ts), [comandi Rust](../../../src-tauri/src/commands.rs), [MVP](../../MVP.md) e [portabilità](../../DATA-PORTABILITY.md). Il dialogo «Emetti certificato» mostra scelta opera, artista, lingua, luogo, intestatario e note, con un avviso testuale sull'immutabilità. Il pulsante «Emetti certificato» chiama **saveCertificate → captureCertificate → issueCertificate**; la successiva **previewCertificate** avviene **dopo** che il backend ha creato il certificato. Il backend acquisisce uno snapshot dell'opera, incrementa il contatore e inserisce il certificato nella **stessa transazione**. L'anteprima *post*-emissione già esiste: non va presentata come una funzione mancante. Nei file letti non emerge invece un controllo visuale dei **dati definitivi e della fotografia prima** dell'operazione irreversibile. Non è stato eseguito un test nativo, né sono stati esaminati casi di errore documentale reali: **lacuna potenziale**, non bug accertato.

**Utenti da intervistare:** artisti che emettono certificati, assistenti di studio, gallerie che li preparano per la firma, archivi che hanno storicità di documenti. Per chi emette un certificato l'anno, un percorso troppo lungo sarebbe controproducente.

**Bisogno da verificare:** vedere sullo stesso schermo, prima dell'emissione, **quali dati saranno fissati**, quali non sono compilati e se l'immagine selezionata è quella corretta, senza consumare un numero di certificato per un semplice controllo.

## 2. Non duplicare funzioni, documenti o proposte precedenti

- **[UX-045 — Rettifica e sostituzione dei certificati](REPS-UX-045-rettifica-sostituzione-certificati-opere.md):** riguarda segnalazioni, errori e possibili relazioni **dopo** l'emissione; questa scheda studia solo una barriera **prima** della prima emissione, per ridurre la necessità di rettifiche. Nessun registro parallelo di contestazioni e nessun seriale di sostituzione.
- **[UX-007 — Dossier](REPS-UX-007-dossier-presentazione-opere.md)** e **[UX-053 — Cartellini](REPS-UX-053-didascalie-sala-cartellini-espositivi.md):** producono materiali di presentazione condivisibili, non documenti storici numerati e immutabili.
- **[UX-018 — Catalogazione e bozze](REPS-UX-018-catalogazione-seriale-schede-bozza.md):** riguarda creazione di schede opera; questo percorso può soltanto rimandare all'editor dell'opera e poi ripetere il controllo. Non introdurre una seconda scheda.
- **[TECH-041 — Checklist Object ID](https://github.com/Cesare-ml/Repertica-Studio/blob/docs/idee-strumenti-architettura-20261121/docs/idee/strumenti-e-architettura/REPS-TECH-041-checklist-identificazione-object-id-offline.md):** verifica completezza descrittiva per identificazione fisica; qui si controlla **esattamente il sottoinsieme di informazioni che entrerebbe nel certificato**. Non richiedere nuove categorie Object ID né attestare conformità.
- **[TECH-023 — PDF/A](https://github.com/Cesare-ml/Repertica-Studio/blob/docs/idee-strumenti-architettura-20261121/docs/idee/strumenti-e-architettura/REPS-TECH-023-certificati-pdf-a-conservazione-verificabile.md):** studia il formato di conservazione del file già generato; non decide l'emissione né la correttezza delle dichiarazioni. **[TECH-020 — Accessibilità](https://github.com/Cesare-ml/Repertica-Studio/blob/docs/idee-strumenti-architettura-20261121/docs/idee/strumenti-e-architettura/REPS-TECH-020-accessibilita-flussi-critici-tastiera-screen-reader.md)** dà principi comuni, non il flusso professionale.
- **Esistente da preservare:** emissione transazionale, seriale unico, snapshot immutabile, fotografia storica, anteprima e stampa *dopo* l'emissione, backup integrale. Nessun file dell'altro filone viene cambiato.

**Fuori ambito:** autenticazione dell'opera, perizia, firma digitale, invio email, validità legale automatica, revoca di certificati, conversione PDF/A, certificati in serie, revisione obbligatoria da terzi, importazioni di dati estranei, numeri progressivi riservati nelle anteprime.

## 3. Scenari d'uso e flussi UX ipotetici

**A — Errore nelle misure.** L'artista seleziona un'opera con larghezza di 50 cm anziché 60 cm. Dalla schermata di conferma vede **50 cm** accanto a titolo, tecnica, fotografia e codice. Sceglie «Torna alla scheda opera»; corregge attraverso il normale editor, torna al processo di emissione e riesamina i dati aggiornati. **Non** è stato creato un certificato e **non** è stato consumato un seriale per la sola verifica.

**B — Fotografia non disponibile.** La copertina referenziata non può essere caricata. L'anteprima mostra «Immagine non disponibile; verifica necessaria» in testo, non un riquadro vuoto scambiabile per fotografia valida. Nella fase minima l'utente può interrompere e correggere; decidere dopo collaudi se sia consentita emissione deliberata senza fotografia, dato che il backend attuale può costruire snapshot di opere senza immagine.

**C — Nome dell'intestatario.** Il campo facoltativo è rimasto vuoto anche se esiste una vendita. L'interfaccia evidenzia «Intestatario: non indicato» e non riempie automaticamente dal collezionista collegato, perché acquirente, intestatario e possessore possono essere diversi. Il nome, se inserito, va confermato manualmente.

**D — Documento bilingue.** Un'artista seleziona italiano/inglese e controlla le etichette e il testo visualizzabile; i nomi propri e i campi di catalogo non sono tradotti automaticamente. L'anteprima non deve simulare un risultato tipografico garantito se il renderer di stampa non è identico.

**E — Scheda modificata altrove.** Dopo la revisione dei dati l'opera o i dati dell'emittente sono cambiati. Il sistema non deve emettere silenziosamente un certificato diverso da quello confermato: segnala «Dati cambiati, rileggi il riepilogo». Non promettere la protezione dal conflitto senza una verifica nel backend al momento dell'emissione.

**Percorso minimo proposto:** «Certificati → Nuovo → Compila → Riepilogo definitivo / controllo dati → Conferma emissione → Anteprima del certificato emesso (già presente)». Nel riepilogo: miniatura leggibile, codice e titolo, anno, tecnica/supporto, dimensioni, artista e dichiarante, intestatario, luogo, lingua, note, avvisi di assenza/ambiguità. Distinguere **campi realmente stampati** da eventuali metadati solo gestionali: non promettere sul certificato informazioni che il template attuale non mostra.

**Interazione prudente:** due azioni chiaramente diverse, «Rivedi dati» e «Emetti certificato irreversibile». La conferma è esplicita; nessuna casella legalistica «autenticità verificata». Se si esce, restano intatte opera e certificati precedenti. Evitare una conferma rituale doppia se la prova con utenti dimostra sufficiente una singola schermata ben progettata.

## 4. Dati necessari, invarianti e ipotesi API puramente descrittive

**Fase A, senza nuova persistenza:** usare la scheda opera corrente, l'identità dell'emittente nelle impostazioni e il *draft* di emissione già in memoria. Rappresentazione concettuale temporanea: **CertificateReview** con artwork UUID, revisione/fingerprint di dati rilevanti, dati da congelare, campi editabili del draft, fotografia/riferimento, esito del caricamento, avvisi, momento della lettura e lingua. Il riepilogo non è un certificato, non ha seriale e non viene memorizzato come emissione.

**Controllo sulla data:** il seriale e la data reale d'emissione restano attribuiti **all'emissione effettiva** con calendario locale, non all'anteprima; in caso di passaggio di mezzanotte o cambio d'anno il riepilogo va aggiornato e non deve esporre un «seriale previsto» spacciandolo per definitivo.

**Concorrenza:** se mai approvato, il backend deve verificare che i dati effettivamente congelati coincidano con quelli confermati, **all'interno della transazione di emissione**. Un confronto solo lato Angular è insufficiente: il campo updated_at dell'opera non coprirebbe necessariamente cambi di artista/alias, fotografia o note del draft. Un eventuale token di revisione/fingerprint deve comprendere i campi realmente rilevanti, avere ambito solo locale, e scadere dopo una modifica. Su conflitto: non inserire certificato né consumare numero, far ripetere la revisione. Non introdurre un sistema di blocchi su tutte le opere.

**Invarianti:** snapshot storico e seriale immutabili; un'immagine non caricata non viene etichettata «verificata»; nessuna nuova vendita/incasso/assegnazione percentuali; nessuna variazione automatica dei dati dell'opera, nome del collezionista o data storica. Emissioni ripetute per una stessa opera non sono automaticamente illecite: nessun blocco presuntivo; eventuale avviso informativo con seriali preesistenti.

**Operazioni concettuali, NON comandi da implementare:** «prepara riepilogo», «controlla disponibilità foto», «confronta versione dei dati», «conferma emissione se i dati coincidono». Si tratta di descrizioni di comportamento, non di nuove API approvate né di schema SQLite.

## 5. Impostazioni, offline, multilingua, privacy e accessibilità

- **Impostazioni:** nessun modulo aggiuntivo obbligatorio; valutare un'impostazione soltanto dopo test sull'uso frequente. La lingua del documento rimane IT/EN/bilingue, separata dalla lingua dell'interfaccia. Non permettere un'opzione che ometta la verifica indispensabile su un atto irreversibile senza una decisione specifica.
- **Offline e backup:** tutte le letture e le eventuali verifiche avvengono sullo SQLite/media locali. Zero account, licenze, rete, IA, cloud, notifiche o abbonamenti. Il draft temporaneo non è un documento emesso e non richiede backup. Certificati emessi restano negli attuali backup completi su cartella scelta; PDF esterni non entrano nel backup solo perché stampati.
- **Localizzazione:** lingua di sistema con fallback inglese; tradurre avvisi, etichette, conferme e stati nelle lingue supportate IT/EN/ES/FR/DE. Il documento ha soltanto le scelte effettivamente supportate. Numeri, unità e date mostrati secondo locale; il loro valore non cambia per effetto della traduzione. Titoli, nominativi, note, tecniche libere e intestatari non sono tradotti automaticamente.
- **Privacy:** il riepilogo può contenere nominativi e dati di opere riservate; niente immagini inviate a servizi esterni, log di contenuto o anteprime salvate fuori dall'archivio senza scelta. Schermate condivise e stampa sono decisioni dell'utente: un'anteprima non offre una barriera di accesso.
- **Accessibilità:** sequenza tastiera prevedibile, ritorno al campo da correggere, messaggi associati al campo, focus sul primo problema, testo oltre al colore, navigabilità con screen reader e zoom 200%, foto con descrizione testuale del codice opera. Conferma accessibile senza pressioni temporali né scorciatoie irreversibili involontarie.

## 6. Import/export, compatibilità, alternative e dipendenze

**Import/export:** nessun formato nuovo è necessario per la fase minima. I dati delle opere e dei certificati continuano nei CSV integrali attuali e nel backup completo SQLite + media; i CSV restano **esportazioni**, non import di certificati. Eventuale stampa di una bozza per revisione fisica deve recare «BOZZA — NON EMESSO», **nessun numero seriale valido**, e passare attraverso un'esportazione locale esplicitamente richiesta. Mai salvare un PDF provvisorio come certificato storico.

**Compatibilità:** conservare struttura dei certificati, seriali e snapshot attuali; vecchi archivi restano leggibili senza nuove colonne. La fase iniziale potrebbe evitare migrazioni SQLite. Qualora una successiva strategia di confronto richieda metadati, studiare migrazione additiva e vecchi backup senza inventare certificati mancanti. Funzionamento su Windows/macOS/Linux, percorsi Unicode e immagini archiviate in media locale.

**Dipendenze ipotetiche:** editor opere, dati artista e alias, renderer di certificati, gestione fotografie storiche, emissione transazionale e traduzioni già esistenti; per un vero controllo di coerenza serve verifica backend (non solo UI). Nessuna dipendenza dal progetto Object ID, PDF/A, dal server MCP o da un servizio remoto.

**Alternative da provare prima di sviluppare:** (1) lasciare l'attuale avviso e un controllo manuale dell'opera; (2) aggiungere una breve schermata di riepilogo senza una replica della pagina A4; (3) preparare una lista di controllo stampabile da usare in studio. Preferire la soluzione più leggera che riduce errori reali.

## 7. Rischi, complessità e criteri di accettazione ipotetici

**Complessità:** bassa-media per riepilogo in sola lettura e usabilità; media per anteprima visiva fedele, gestione delle foto e rientro nell'editor; medio-alta per garantire che i dati validati corrispondano alla transazione finale e ai cambi di calendario. Evitare una duplicazione autonoma del motore di stampa.

**Rischi:** illusione che la revisione garantisca autenticità o conformità giuridica; anteprima diversa dalla stampa; doppio clic o UI obsoleta; seriale consumato durante una bozza; fotografia non trovata; foto modificata fra revisione ed emissione; blocco inutilmente rigido a causa di un campo non essenziale; ulteriori passaggi che rallentano artisti che emettono documenti di rado. Mitigazioni: etichette chiare, dati esatti da snapshot, controllo lato backend prima del commit, test su casi reali, possibilità di tornare indietro, avvisi classificati per gravità senza inventare obblighi legali.

**Criteri di accettazione proposti soltanto per valutazione futura:**

1. Aprire e chiudere cento riepiloghi di pre-emissione **non** crea certificati né incrementa il contatore.
2. Con misura, titolo o foto errati, l'utente può rivedere l'opera e ripetere il controllo senza seriali consumati; la correzione non riscrive i certificati preesistenti.
3. Intestatario vuoto resta **esplicitamente vuoto**, anche se l'opera è venduta: nessuna deduzione dall'acquirente.
4. Se la fotografia è mancante/corrotta, il problema è riconoscibile anche senza immagine e senza affidarsi al colore.
5. Se cambiano i dati usati per il riepilogo, la conferma non deve congelare una versione diversa in silenzio; il tentativo conflittuale non consuma un seriale.
6. La data d'emissione e l'anno del seriale seguono l'istante effettivo in ora locale, anche a mezzanotte.
7. Dopo l'emissione, resta il flusso corrente di visualizzazione e stampa da snapshot, senza alterare la numerazione storica.
8. Il flusso funziona senza rete, con lingue e fallback previsti, da tastiera/screen reader e su finestra ridotta.
9. Se un utente decide di non procedere, nessun dato economico, stato commerciale o file media viene modificato.

## 8. Fasi suggerite e domande per revisione del proprietario

- **Fase 0 — Ricerca senza software:** far provare a 4–6 artisti/galleristi l'emissione esistente su dati di prova, osservare tempi ed eventuali errori; raccogliere 3–5 casi anonimi di correzioni realmente richieste. Se gli errori non sono ricorrenti e l'avviso basta, **nessuno sviluppo**.
- **Fase A — Studio del flusso minimo:** riepilogo testuale/immagine prima della conferma, senza seriale né salvataggio e senza secondo editor.
- **Fase B — Solo se necessaria:** controllo di coerenza atomico tra informazioni revisionate ed emissione, e gestione dei dati cambiati.
- **Fase C — Solo con evidenza:** anteprima della pagina A4 prima dell'emissione, se il problema riguarda **l'impaginazione** più dei metadati; niente secondo renderer non verificato.

**Requisiti certi:** applicazione locale, gratuita e open source; lingua OS/fallback inglese; backup a destinazione scelta; certificati numerati e immutabili; anteprima *post* emissione presente; clic del pulsante corrente avvia l'emissione e il backend congela uno snapshot in transazione. **Ipotesi:** il controllo visivo prima del commit riduce errori significativi ed è desiderato dai professionisti; una miniatura basti a controllare l'immagine; la domanda riguardi più i dati che l'impaginazione. **Questioni aperte per il proprietario:** serve una conferma unica o due azioni? Che cosa rendere bloccante rispetto a un semplice avviso? Un certificato senza foto va ammesso deliberatamente? Come trattare emittente/pseudonimo e nominativi? È preferibile un riepilogo semplice oppure la pagina A4 fedele? Quanto è frequente emettere certificati?

**Decisione:** nessuna approvazione, assegnazione o implementazione. La scheda documenta soltanto un'opportunità da confrontare con l'uso reale.
