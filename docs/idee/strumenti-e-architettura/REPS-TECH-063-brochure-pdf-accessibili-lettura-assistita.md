# REPS-TECH-063 — Brochure PDF accessibili: struttura semantica, lettura assistita e descrizioni delle immagini

- **ID stabile:** REPS-TECH-063
- **Data della ricognizione:** 2026-11-23
- **Stato:** **Proposta — da valutare**
- **Filone:** Strumenti tecnici e architettura; analisi documentale soltanto
- **Priorità proposta:** **P3**, elevabile a **P2** dopo necessità d'uso reale documentata e verifica di fattibilità; nessun requisito normativo o certificazione presunto
- **Esito decisionale:** nessuna approvazione, assegnazione, implementazione o scelta di dipendenze

## 1. Problema, motivazione e base verificata

**Fatti controllati sul branch `main`, 23/11/2026:**

- Repertica dispone già di un editor locale per brochure con bozze SQLite, fotografie di opere, titoli, descrizioni, contatti, tre layout e PDF A4 esportati in una cartella scelta. La lingua del documento è indipendente da quella dell'interfaccia. Si veda [Brochure offline](../../BROCHURES-OFFLINE.md).
- In `src-tauri/src/brochures.rs`, `render_pdf` costruisce un PDF 1.4 con catalogo, pagine, font Type1, testo in flussi grafici e immagini JPEG; nel percorso esaminato **non costruisce** un albero `/StructTreeRoot`, riferimenti di contenuto marcato, descrizioni alternative `/Alt`, né una lingua `/Lang` del documento. La sola presenza di testo disegnato e immagini non equivale a un PDF strutturato per lettori assistivi.
- Le pagine possono contenere immagini dell'opera e testi disposti in aree grafiche separate; l'ordine di lettura di un lettore assistivo non può essere assunto in base all'ordine visivo.
- [REPS-TECH-032](REPS-TECH-032-pdf-brochure-unicode-font-incorporati-offline.md) analizza già font Unicode, testo selezionabile e fedeltà tipografica. Quel documento **esclude esplicitamente** dall'ambito ordine di lettura, tag strutturali, testo alternativo e conformità PDF/UA: sono il perimetro specifico di questa nuova scheda.

**Problema potenziale:** una galleria invia una brochure visivamente leggibile, ma chi usa screen reader o navigazione tramite struttura del documento potrebbe non riuscire a individuare le opere, capire l'associazione fra immagine, titolo e dati tecnici, oppure passare da una sezione all'altra. La semplice correzione di Unicode non risolve la semantica.

**Non verificato:** segnalazioni di clienti, comportamento concreto con NVDA/VoiceOver/lettori PDF su tutte le piattaforme, esigenze contrattuali di PDF/UA, percentuale di brochure effettivamente distribuite in questo modo. Non classificare l'assenza di tag come incidente di accessibilità riportato dagli utenti né dichiarare una violazione legale senza analisi contestuale.

## 2. Destinatari e scenari d'uso

1. **Artista indipendente:** prepara un catalogo di 10 opere e vuole che una persona che ascolta il documento possa navigare fra i titoli e comprendere i metadati, non soltanto sentire una lista di stringhe fuori contesto.
2. **Galleria o fondazione:** consegna un PDF a un ente culturale che ha una richiesta documentata di lettura accessibile. Il personale deve poter revisionare le descrizioni alternative delle immagini senza cambiare le didascalie archivistiche.
3. **Archivista e curatore:** deve distinguere la fotografia dell'opera (contenuto significativo) da cornici e ornamenti grafici (decorativi), evitando che un lettore vocale legga pagine e intestazioni ripetute a ogni opera.
4. **Professionista multilingue:** produce una brochure in francese o tedesco e vuole indicare correttamente la lingua prevalente del documento, con eventuali frammenti in altre lingue da revisionare.

## 3. Requisiti certi, ipotesi e confini

**Vincoli certi del prodotto:** desktop gratuito e open source, database e file locali, uso offline senza account, backup esportabile, percorsi di output scelti dall'utente, nessuna IA o cloud obbligatori, lingue UI dal sistema con fallback inglese.

**Ipotesi da valutare**, non nuovi requisiti approvati:

- produrre, eventualmente con un **profilo esplicito**, un PDF con struttura logica di documento, intestazioni, paragrafi, figure e contenuti decorativi;
- rappresentare un ordine di lettura comprensibile in *tutti e tre* i layout, comprese le pagine aggiuntive delle descrizioni lunghe;
- chiedere una descrizione alternativa **redatta o confermata da una persona** per le fotografie significative: la tecnica, il titolo o la didascalia possono fornire contesto, ma non sono automaticamente una descrizione visiva attendibile;
- offrire un controllo prima dell'esportazione e un resoconto di problemi e limiti, senza indicare «accessibile» o «conforme PDF/UA» solo perché sono presenti alcuni tag.

**Fuori ambito:** creazione automatica di descrizioni fotografiche, scansione di tutti gli archivi, PDF/A dei certificati, firma dei documenti, nuovo editor visuale, hosting, sincronizzazione o modifica automatica di schede, certificati e dati di vendita. Nessuna conformità PDF/UA-1 o PDF/UA-2 è promessa senza validazione completa del profilo scelto.

## 4. Specifica funzionale e flussi UX ipotetici

**Flusso A — controllo preliminare, prima di un nuovo renderer.** Dalla bozza attuale l'utente sceglie *Verifica accessibilità del documento* (nome da validare). Vengono presentati, senza modifiche automatiche, la lingua della brochure, le opere selezionate, la presenza di una descrizione per ogni immagine significativa, i testi troppo lunghi/illeggibili già segnalabili e gli aspetti che il PDF attuale non può garantire. Il report ha collegamenti contestuali alla bozza, non una pagella colorata.

**Flusso B — export strutturato, eventuale seconda fase.** L'utente revisiona l'ordine delle sezioni (copertina → introduzione → opere nell'ordine scelto → contatti), la gerarchia dei titoli, le immagini con descrizione, le intestazioni decorative e la lingua. Solo dopo conferma esplicita sceglie cartella e nome del PDF, mantenendo l'esportazione tradizionale compatibile. Il rapporto finale distingue *controlli tecnici superati*, *controlli manuali ancora necessari* e *limitazioni note*. Un risultato privo di errori macchina non equivale all'approvazione di un lettore assistivo.

**Flusso C — assenza di descrizione.** Se manca la descrizione di un'immagine, indicare l'opera e consentire **scrivere, lasciare non risolto o annullare**. Non ricavare un'immagine descritta dal solo titolo e non trasformare automaticamente ogni foto in decorazione. Per piccoli strumenti grafici e ornamenti effettivamente privi di valore informativo, valutare marcatura da *artefatto*.

**Flusso D — consegna prudente.** Come oggi il PDF esportato è un derivato esterno alla cartella dell'archivio: avviso sul fatto che non è automaticamente incluso nel backup e che l'esportazione può contenere contatti scelti dall'autore. Nessun invio automatico a collezionisti o istituzioni.

Ogni schermata proposta deve usare etichette testuali, controlli da tastiera e messaggi annunciabili ai lettori assistivi; l'accessibilità del controllo preparatorio non deve essere inferiore a quella del PDF che si vuole generare.

## 5. Dati e contratti descrittivi, **non API da implementare**

Nessuna modifica a SQLite è un requisito già accettato. Possibile struttura **puramente concettuale**, eventualmente conservabile nella bozza brochure se e solo se necessaria:

| Dato ipotetico | Funzione e cautela |
| --- | --- |
| `document_language` | Codice linguistico della brochure già selezionabile, da mappare al metadato PDF; non derivare dalla lingua del PC |
| `image_alt_by_item` | Descrizione umana specifica della **foto scelta in questa brochure**; non modificare la didascalia originale né presumere un diritto di pubblicazione |
| `image_role` | Foto significativa o artefatto decorativo, scelta esplicita e verificabile |
| `semantic_sections` | Sequenza logica di titoli, introduzione, opere, metadati e contatti, possibilmente **derivabile** dagli elementi già esistenti |
| `accessibility_review` | Stato della verifica, limitazioni e data, **non** una certificazione permanente; non includere informazioni personali nel log |
| `pdf_export_profile` | Profilo interno dichiarato e versionato; distinguere PDF tradizionale, PDF con tag parziali ed eventuale profilo standard validato |

**Ipotesi di API interne, solo come vocabolario:** `reviewBrochureAccessibility(draft) -> findings[]`, `renderStructuredPdf(draft, reviewedDescriptions, profile) -> artifact`, `validateAccessiblePdf(artifact, profile) -> checks+limitations`. Non sono richieste firme, moduli, tabelle o librerie.

L'associazione foto/descrizione va agganciata all'identità della **foto scelta**, non soltanto all'opera: cambiando fotografia, l'alternativa precedente non deve essere riutilizzata silenziosamente come se fosse valida.

## 6. Offline, sicurezza, privacy e interoperabilità

- **Offline integrale:** generazione, struttura, convalida e revisione non richiedono connessioni, modelli IA, servizi terzi o font scaricati. L'app rimane usabile senza adottare la funzione.
- **Privacy:** nessun testo della brochure, descrizione delle immagini o dato di contatto deve comparire in telemetria, log diagnostici di supporto, prompt esterni o file temporanei persistenti. La descrizione alternativa può rivelare più informazioni del necessario: controllo esplicito dell'autore prima dell'export.
- **Sicurezza e integrità:** mantenere i limiti sui percorsi media, sui symlink, sulle dimensioni delle immagini e sulle destinazioni già esistenti; valutare sicurezza di eventuali nuovi generatori/validatori PDF. Il PDF non deve essere scritto come risultato «completo» se la generazione fallisce; non modificare DB di opere, vendite, certificati o originali fotografici.
- **Interoperabilità:** il PDF deve aprirsi senza software proprietario e mantenere un ordine di lettura verificabile; l'eventuale conformità a un profilo PDF/UA richiede identificare una versione, i prerequisiti tecnici e lo strumento di validazione. Anche un validator positivo richiede test pratici con tecnologie assistive e revisione editoriale.
- **Compatibilità:** vecchie brochure si aprono senza migrazioni distruttive; le descrizioni opzionali non devono essere imposte per la normale funzione di brochure, e gli export storici non vengono riscritti. Windows, macOS e Linux restano supportati; i programmi di lettura PDF possono comportarsi diversamente.

## 7. Impostazioni e multilingua

- Lingua dell'interfaccia = lingua del sistema operativo con **fallback inglese**, salvo override locale già previsto; tutte le diagnosi e le istruzioni devono essere localizzabili.
- Lingua **del documento** distinta da quella dell'interfaccia. Se il testo contiene più lingue, evitare etichette false e proporre una verifica mirata: scegliere una lingua principale non rende automaticamente corretti i passaggi bilingui.
- **Nessuna preferenza obbligatoria** per utenti che vogliono semplicemente stampare una brochure. Un profilo di esportazione accessibile, se giustificato, può essere selezionato ad hoc, con limiti dichiarati.
- Niente colori come unica indicazione dell'esito: errori, avvisi e operazioni devono avere testo leggibile e focus coerente.

## 8. Alternative da verificare prima di impegnarsi in un nuovo motore

1. **Guida e checklist per documenti esterni:** consegnare oggi un PDF accompagnato da descrizioni e testo in un formato aperto già accessibile (es. HTML/ODT creato manualmente) quando il destinatario lo accetta. Basso costo, ma non rende accessibile il PDF esistente.
2. **Esportazione HTML locale strutturata**, con immagini scelte e descrizioni umane, da aprire offline senza pubblicazione. Potrebbe risultare più economica e controllabile per alcune consegne, ma aggiungerebbe un formato da mantenere e richiederebbe le stesse cautele privacy.
3. **Renderer PDF esterno o libreria open source** capace di tagging e validazione: maggiore manutenzione e superficie di dipendenze; fattibilità licenze CPAL e compatibilità Win/Mac/Linux ancora da verificare.
4. **Migrazione incrementale del renderer attuale** per aggiungere tag/ordine senza cambiare layout. Evita un nuovo motore, ma il renderer artigianale potrebbe non essere un fondamento ragionevole per una conformità certificabile.
5. **Nessuna nuova funzione** se la domanda effettiva è rara e un file testuale accessibile separato risolve i casi osservati.

## 9. Dipendenze, rischi e complessità

| Dimensione | Valutazione ipotetica |
| --- | --- |
| Diagnostica/checklist | Bassa–media, soprattutto disciplina editoriale e UX |
| PDF con struttura robusta | **Alta**: tagging, immagini, riferimenti, ordine logico, lingue e test multipiattaforma |
| Eventuale conformità PDF/UA | **Alta** e non garantita; obbligatorio stabilire profilo corretto, standard, validator e revisione umana |
| Dipendenze | Interazione con TECH-032 (Unicode e mappatura testo), renderer PDF, eventuali validatori indipendenti e screen reader |
| Rischi | Senso falso di conformità, descrizioni inadeguate, ordine errato in layout a colonne, immagini ornamentali lette come contenuto, regressioni visive, file più pesanti, licenze e compatibilità |
| Mitigazione | Nessuna etichetta «conforme» senza prove; casi campione revisionati da persone con disabilità, profili espliciti e fallback conservativo |

## 10. Test e criteri di accettazione **proposti**, non eseguiti

1. Un campione sintetico con copertina, introduzione, tre layout, due opere, descrizioni lunghe, contatti e pagine continuative produce l'ordine **logico** previsto; prova reale con almeno due combinazioni diverse lettore PDF/screen reader.
2. I titoli hanno gerarchia consistente; fotografie informative sono raggiungibili come figure con alternative deliberate, mentre ornamenti e intestazioni ripetute non interferiscono con la lettura. Nessuna descrizione inventata dal software.
3. Testo, descrizione, alt e lingua sono estraibili da un tool indipendente quando il profilo li prevede; testo visivo e contenuto parlato non contraddittori.
4. Il PDF conserva immagini e layout attuali senza contenuti sovrapposti, troncati o illeggibili; coordinare con TECH-032 per caratteri non WinAnsi e con le prove di overflow.
5. Una bozza storica si apre ancora; l'assenza di campi di accessibilità **non impedisce** la stampa/esportazione tradizionale. La modifica della foto selezionata segnala che l'eventuale alt preesistente va riesaminato.
6. Nessun file viene spedito o pubblicato; non rimangono copie temporanee sensibili. Interruzione della generazione o destinazione non disponibile non deve alterare l'archivio.
7. Solo **se** si decide di dichiarare uno standard: validator documentato su file reale e verifica manuale dell'esito con persona esperta; non basta riconoscere `/StructTreeRoot` con una ricerca testuale.

## 11. Fasi e priorità per eventuale revisione del proprietario

- **Fase 0 — senza sviluppo:** contattare 3–5 artisti/gallerie/istituzioni, raccogliere almeno due requisiti concreti di consegna, osservare un PDF campione con lettori assistivi; confrontare il costo di HTML/ODT accessibile e PDF.
- **Fase 1 — solo se utile:** checklist editoriale e prove guidate su una brochure campione, eventualmente fuori dall'app.
- **Fase 2 — subordinata:** specifica verificabile dell'ordine semantico e descrizioni per immagine, con test su layout e lingue; prima definire rapporto con font Unicode di TECH-032.
- **Fase 3 — soltanto su autorizzazione futura:** renderer strutturato, verificatore, prove utenti e possibile profilo standard dichiarato; nessuna partenza automatica dopo questa scheda.

**Priorità iniziale P3.** Diventa **P2** soltanto dopo evidenza che una brochure PDF accessibile sia necessaria più spesso di quanto possano risolvere formati alternativi e che una soluzione locale mantenibile sia realistica.

## 12. Anti-doppioni e coordinamento

- **TECH-032**: resa Unicode/font/metriche e selezione del testo; qui **semantica, alternative delle immagini e lettura assistita**. Necessaria coordinazione futura, non duplicazione.
- **TECH-020**: accessibilità dei **dialoghi nell'app**; qui accessibilità del **file esportato letto da un terzo**.
- **TECH-023**: eventuale PDF/A dei **certificati**; non implica accessibilità delle brochure, né questa scheda modifica certificati o attestazioni.
- **TECH-034**: scrittura non distruttiva del PDF; qui struttura logica del documento, non il protocollo filesystem.
- **UX-007/010/041/053/054/062** della [PR Gestione e UX #14](https://github.com/Cesare-ml/Repertica-Studio/pull/14): contenuti, diritti, titoli multilingua, cartellini, presentazione e coerenza delle brochure, non tagged PDF. Nessun file di quella cartella è da cambiare.
- Il modulo brochure esistente mantiene l'autorevolezza di dati e immagini dell'archivio; le alternative accessibili sarebbero **derivati** deliberati, mai attestazioni di provenienza, consenso o diritti.

## 13. Questioni aperte, riservate al proprietario

1. Esistono destinatari reali che richiedono un **PDF** accessibile, oppure accettano un file HTML/ODT strutturato insieme al PDF visivo?
2. La foto dell'opera deve avere una descrizione visiva autonoma, e chi è responsabile della sua redazione/revisione?
3. Serve solo fruibilità verificata con screen reader, oppure è necessario un determinato profilo PDF/UA e quale?
4. Quali lettori e sistemi operativi sono effettivamente usati dai destinatari? Che cosa significa «accessibile abbastanza» nei casi reali senza promettere una conformità non misurata?
5. È accettabile richiedere un controllo manuale per ogni immagine significativa, anche quando le brochure hanno molte opere?
6. Il costo di manutenzione è giustificato rispetto a un'eventuale esportazione HTML accessibile più semplice?

## 14. Fonti tecniche consultate il 23 novembre 2026

- [Repertica Studio — Brochure offline](../../BROCHURES-OFFLINE.md), [renderer Rust `brochures.rs`](../../../src-tauri/src/brochures.rs), [TECH-032](REPS-TECH-032-pdf-brochure-unicode-font-incorporati-offline.md).
- [W3C WAI, PDF3: ordine di lettura e navigazione nei PDF](https://www.w3.org/WAI/WCAG22/Techniques/pdf/PDF3.html).
- [W3C WAI, PDF9: titoli strutturali nei PDF](https://www.w3.org/WAI/WCAG21/Techniques/pdf/PDF9.html).
- [W3C, elenco delle tecniche PDF (PDF1 alternative, PDF4 artefatti, PDF9 titoli)](https://www.w3.org/WAI/WCAG21/Techniques/).
- [PDF Association, Accessible PDF e riferimento a PDF/UA-2 (ISO 14289-2)](https://pdfa.org/accessibility).

**Conclusione:** studio di fattibilità da sottoporre a revisione; non autorizza modifiche al codice, workflow, librerie, documenti dell'altro filone o stato dell'app.
