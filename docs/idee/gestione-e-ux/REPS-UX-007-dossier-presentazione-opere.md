# REPS-UX-007 — Scheda di presentazione e dossier opere con condivisione controllata

- **ID stabile:** REPS-UX-007
- **Data:** 2026-11-21
- **Stato:** Proposta — da valutare
- **Filone:** Gestione e UX
- **Priorità ipotetica:** P1–P2 per gallerie, artisti e archivi che inviano schede a curatori o collezionisti; da confermare con il proprietario
- **Perimetro:** progettazione documentale, nessuna implementazione autorizzata

## Contesto, problema e confini

L'[archivio MVP](../../MVP.md) contiene opere, fotografie, tecnica, supporto, misure, descrizione e ubicazione. Un **certificato** è un documento storico numerato, con snapshot immutabile e finalità diversa. Le esportazioni [CSV](../../DATA-PORTABILITY.md) sono registri integrali per analisi e portabilità, non presentazioni selettive. [TECH-001](https://github.com/Cesare-ml/Repertica-Studio/blob/docs/idee-strumenti-architettura-20261121/docs/idee/strumenti-e-architettura/REPS-TECH-001-pacchetto-dati-aperto.md) progetta un pacchetto interoperabile, non un dossier per destinatari esterni. Nessuna di queste funzioni equivale a una **scheda curatoriale o commerciale pronta da condividere**, con campi esplicitamente inclusi dall'utente.

**Problema:** un artista invia la foto e una scheda a una galleria, curatore o potenziale acquirente. Copiare manualmente foto, titolo, misure e testo da un gestionale in un documento separato comporta errori, versioni obsolete e rischio di includere dati privati come nome del collezionista, ubicazione esatta, spese, margini o contatti.

**Obiettivo ipotetico:** generare localmente un dossier stampabile/PDF e una scheda per una o più opere, controllando i campi visibili. Non creare un sito pubblico, una mail automatica, un catalogo fiscale o un certificato di autenticità.

## Destinatari e casi d'uso

1. **Galleria:** seleziona sei opere disponibili per una proposta espositiva, crea un dossier con copertina facoltativa, immagini e schede uniformi, senza mostrare i margini interni.
2. **Artista:** prepara una pagina singola per un curatore, scegliendo biografia/descrizione espositiva e una foto di qualità; lascia i recapiti del collezionista fuori dal documento.
3. **Archivio privato:** produce un elenco per una mostra in cui una determinata ubicazione è **riservata**. La provenienza incerta è omessa, oppure chiaramente qualificata se scelta esplicitamente.
4. **Opera venduta:** il dossier può indicare «non disponibile» o non mostrare alcuna disponibilità, senza divulgare necessariamente nominativo e prezzo pagato.
5. **Edizione numerata futura:** se [UX-005](REPS-UX-005-edizioni-multipli-serie-numerate.md) viene approvata, l'utente potrà includere numero esemplare e tiratura dichiarata, non inventata; non è una dipendenza per il primo modello.

## Flusso UX proposto

- **Archivio → Seleziona opere → Prepara dossier** oppure **Scheda opera → Scheda da condividere**. Mostrare numero opere, foto selezionata e completezza dei dati prima di procedere.
- **Scegli formato:** «scheda singola», «dossier più opere» o «elenco compatto». In prima fase basta un modello pulito stampabile da sistema; nessun editor grafico complesso necessario.
- **Scegli dati e visibilità:** sezioni selezionabili titolo, codice, anno, tecnica/supporto, dimensioni, testo descrittivo, immagini, autore e contatto pubblico **facoltativi**. **Esclusi di default**: contatti dei collezionisti, note private, ubicazione esatta, dati di vendita, prezzi storici, costi e percentuali, date e documenti sensibili. Se si vuole mostrare prezzo richiesto o disponibilità, selezione esplicita con etichetta distinta da ricavi/incassi.
- **Personalizza solo la presentazione:** titolo dossier, destinatario testuale opzionale, breve testo di introduzione, ordine opere e didascalie; non scrivere nuovamente sugli originali. La presenza di campi mancanti deve essere visibile come avviso, senza riempimenti fittizi.
- **Anteprima obbligatoria:** una schermata riepiloga i campi effettivamente inclusi e segnala rischi («contiene un contatto personale», «foto mancante», «dati dichiarati dall'utente»). Un comando **Conferma ed esporta** porta al dialogo nativo di stampa/salvataggio; mai inoltro automatico via rete.
- **Visualizzazione accessibile:** ordine coerente titoli/dati, alternative testuali alle foto quando inserite, contrasto elevato, nessuna informazione solo cromatica; anteprima navigabile da tastiera, linguaggio leggibile; controllo del layout con testi lunghi.
- **Post-export:** feedback di file salvato e percorso scelto, senza aprire altre app o inviare documenti. Un eventuale registro dei dossier esportati è opzionale e non deve diventare tracking dei destinatari.

## Ipotesi di dati (non API implementate)

- **dossier_preset**: UUID, nome locale, versione modello, ordine sezioni, campi visibili, lingua documento, formato pagina e nota «uso interno / uso esterno»; predefiniti conservativi. Non contenere credenziali o token.
- **dossier_draft** (facoltativa): UUID, titolo, elenco ordinato di artwork UUID, scelta immagine per ogni opera, testo di apertura, data, preset UUID, elenco campi selezionati. Le bozze sono sempre locali, modificabili e indipendenti dai certificati.
- **dossier_export_record** (facoltativo): UUID, data, formato, numero opere, versione delle schede/filtro campi e digest del file **se desiderato**, senza salvare automaticamente indirizzo o nome destinatario. **Non è prova di consegna, firma, pubblicazione o lettura**.
- **Origine dati:** i dati dell'opera sono letti senza modificarli. Per riprodurre un dossier del passato, decidere tra (A) rigenerazione dai dati correnti, chiaramente marcata come *nuova versione*, o (B) snapshot locale del contenuto esportato, con costo in spazio e conseguenze privacy. Mai sovrascrivere snapshot dei certificati.
- **Provenienza e qualità:** se [UX-001](REPS-UX-001-provenienza-movimentazioni.md) venisse approvata, includere solo elementi selezionati e qualificati come dichiarati/verificati dall'utente; non generare attestazioni autonome. Se [UX-004](REPS-UX-004-rapporti-stato-conservazione.md) venisse approvata, non inserirne di default contenuti potenzialmente delicati.
- **Compatibilità:** niente nuovo campo obbligatorio sulle opere pregresse; documenti prodotti dal sistema di stampa rimangono indipendenti dallo schema SQLite. I modelli eventualmente esportabili devono essere versionati.

## Offline, impostazioni e multilingua

- Generazione, anteprima, stampa ed esportazione sono **locali**; nessun account, cloud, newsletter, invio mail o telemetria. Se una foto non è disponibile in archivio, mostrarlo, senza recuperarne copie da Internet.
- Impostazioni utente: dati di presentazione pubblicabili (nome artista/studio e contatto **opzionali**), modello pagina e formato (A4/Letter), logo solo se fornito dall'utente, scelta lingua documento, campo «mostra prezzi» disattivato di default.
- Multilingua: UI con lingua di sistema, fallback inglese e override IT/EN/ES/FR/DE. Lingua di output esplicita e distinta dalla lingua interfaccia, se desiderata; intestazioni tradotte, **testi liberi delle opere non tradotti automaticamente**. Data, misure e importi formattati per il locale del documento; importi salvati in centesimi EUR, senza conversione valutaria inventata.

## Sicurezza, privacy, import/export

- Regola **deny-by-default** per campi privati: la selezione dei dati usa una **lista consentita** di campi di presentazione, non un generico export JSON dell'opera con campi nascosti via CSS. Verificare anche testo incorporato, metadati delle immagini e nome del file; preferire immagini derivate depurate dei metadati EXIF **solo dopo verifica tecnica**, evitando promesse di anonimizzazione assoluta.
- Un export destinato all'esterno non deve includere nel contenuto o nei metadati percorsi locali, UUID dei contatti, coordinate precise, note interne, cronologia trattative, foto non scelte, costo/margine, o l'archivio intero.
- **Esportazione:** PDF via stampa locale (coerente con l'MVP), possibile file HTML statico/immagini come **alternativa da valutare**; contenuti da salvare nella cartella scelta dall'utente, senza sovrascrivere file esistenti salvo conferma esplicita. PDF/A e accessibilità PDF non si dichiarano ottenuti senza verifiche specifiche.
- **Import di preset:** opzionale JSON locale versionato, con anteprima e senza script o sorgenti esterne. Il dossier **non è** un formato di reimportazione dell'archivio; rimane il backup verificato e il futuro eventuale pacchetto [TECH-001](https://github.com/Cesare-ml/Repertica-Studio/pull/16) per la portabilità completa.
- **Backup/ripristino:** eventuali preset e bozze salvate rientrano nel backup SQLite/media e nei ripristini locali; i PDF che l'utente ha salvato altrove non devono essere considerati implicitamente inclusi. Nessun invio automatico a destinatari.
- **Protezione dati:** incoraggiare verifica della destinazione scelta (USB o cartella condivisa possono avere accessi più ampi) senza impedire la scelta libera dell'utente.

## Dipendenze, complessità, alternative e rischi

- **Dipendenze:** archivio opere, immagini già disponibili, schema SQLite, traduzioni, print nativo. **Nessuna** dipendenza da MCP, server, funzionalità fiscali, UX-001/004/005 o portabilità semantica.
- **Complessità:** bassa-media per una scheda singola stampabile; media per selezione multi-opera, preset e anteprima con redazione; alta per PDF pienamente accessibile, layout lungo/multilingua, immagini grandi e versioni storiche.
- **Rischi UX:** spuntare involontariamente prezzo o ubicazione; presentare descrizioni non confermate come provenienza certificata; tagliare testi o immagini; esportare metadati EXIF; attribuire validità legale al dossier; differenze tra render e stampa native.
- **Contromisure:** preset conservativi, schermata di anteprima/controllo privacy, watermark testuale facoltativo «Documento informativo», selezione immagine visibile, avvisi su campi mancanti e test di stampa su più OS.
- **Alternativa minima:** generare una scheda singola pulita tramite stampa di sistema con soli metadati non sensibili; non salvare preset né tracciare export. Utile come esperimento prima di introdurre dossier multi-opera.
- **Alternativa fuori app:** esportare dati CSV e impaginarli manualmente; evita sviluppo ma richiede competenze tecniche e aumenta rischio di divulgare campi privati.

## Criteri di accettazione ipotetici

1. Un dossier di cinque opere usa ordine, immagini e campi scelti; non modifica record, vendite o certificati.
2. Nessuna scelta predefinita può rivelare contatti collezionisti, indirizzi privati, costi/margini o prezzi storici; anche i dati nascosti **non** sono presenti nel file esportato o nei suoi metadati controllabili.
3. Foto non disponibili e campi assenti producono avvisi chiari, non dati inventati; un'opera venduta non diventa automaticamente disponibile.
4. L'utente può interrompere il flusso dall'anteprima senza creare file o cambiare stato dell'archivio; il percorso di salvataggio è liberamente scelto.
5. Dossier e certificati restano visivamente e concettualmente diversi; la rigenerazione di un dossier esistente indica esplicitamente se usa dati correnti o uno snapshot.
6. Prova offline, con titoli lunghi, Unicode e archivi grandi su Windows/macOS/Linux; lingua OS o fallback inglese, date e misure corrette; navigazione tastiera e screen reader almeno nella creazione e anteprima.
7. Backup e restore, se conservano preset o bozze, ne ripristinano esattamente struttura e scelte; PDF esterni non sono falsamente inclusi.

## Fasi di esplorazione, NON roadmap autorizzata

- **A:** verifica casi d'uso e prototipo solo documentato di scheda singola minimale, senza persistenza di preset.
- **B:** multi-selezione opere, anteprima privacy e opzioni di presentazione; convalida stampa Windows/macOS/Linux.
- **C:** preset e bozze locali, eventuale snapshot di dossier e test approfonditi di accessibilità/esportazione strutturata.

## Certezze, ipotesi e domande al proprietario

- **Certi dal repo:** opere e media locali, dati personali dei contatti, vendita e certificato, stampa PDF nativa dei certificati, CSV integrali di archivio, backup con cartelle scelte, i18n sistema/fallback inglese.
- **Ipotesi:** artisti e gallerie inviano schede con sufficiente frequenza da giustificare un flusso dedicato; una lista consentita di campi limita errori rispetto all'esportazione manuale.
- **Da decidere:** deve essere prioritario il dossier per curatore o la scheda commerciale per collezionista? È indispensabile il prezzo richiesto? Logo e identità visiva? PDF soltanto o anche HTML? Conservare lo snapshot delle versioni distribuite o solo i preset? Quanto spazio e quale qualità delle immagini devono essere disponibili? Serve esplicitare diritti di riproduzione/credito fotografico senza una licenza imposta?

**Proposta da sottoporre al proprietario. Nessuna implementazione, approvazione o assegnazione implicita.**
