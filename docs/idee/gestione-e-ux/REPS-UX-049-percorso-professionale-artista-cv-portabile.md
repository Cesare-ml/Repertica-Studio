# REPS-UX-049 — Percorso professionale dell'artista: riconoscimenti, formazione e residenze per un CV portabile

- **ID stabile:** REPS-UX-049
- **Data:** 2026-11-22
- **Stato:** Proposta — da valutare
- **Filone:** Gestione e UX
- **Priorità proposta:** P2 condizionata all'uso ricorrente di CV e candidature; P3 o nessuno sviluppo se bastano documenti esterni.
- **Natura:** studio documentale, non approvazione, assegnazione o autorizzazione allo sviluppo.

## 1. Problema, utenti e riscontri

**Fatto verificato su `main` (22 novembre 2026; lettura statica, non prova UI):** `README.md`, `docs/MVP.md` e `docs/BROCHURES-OFFLINE.md` descrivono archivio opere, collezionisti, vendite e brochure PDF locali. La brochure salva copertina, nome artista, introduzione, contatti e opere, ma non è una fonte strutturata dei **riconoscimenti e della carriera**; la sua introduzione è testo editoriale specifico per il documento. `src/app/models.ts` definisce `Brochure` e `Artwork`, non un registro di premi, formazione o residenze. I CSV di `docs/DATA-PORTABILITY.md` non contengono una storia professionale dell'artista. Il progetto non dichiara un CV strutturato o documenti autobiografici generati automaticamente.

**Problema da verificare con artisti reali:** quando un artista deve candidarsi a una residenza, presentarsi a una galleria, rispondere a una call o inviare un curriculum aggiornato, ricopia formazione, premi, borse e residenze da documenti Word/LibreOffice e cartelle separate. Errori di data e versioni discordanti diventano probabili; inserire tutto nell'introduzione di ogni brochure crea copie divergenti. Non serve però trasformare Repertica in una piattaforma di candidature o in un CRM.

**Utenti:** artisti indipendenti con attività espositiva, atelier con assistenti, archivi e piccole gallerie che redigono profili per propri artisti. Per chi non usa CV o partecipa a rarissime call, un file `.odt` mantenuto manualmente potrebbe essere migliore.

**Riferimento esterno, non certificazione di conformità:** le [Visual Art CV Guidelines della College Art Association](https://www.collegeart.org/standards-and-guidelines/guidelines/visual-art-cv) distinguono master CV e versione selezionata, raccomandano dati documentabili, ordine cronologico e sezioni adattate alla carriera. Sono indicazioni editoriali, non una norma tecnica obbligatoria né prova che gli utenti Repertica chiedano il modulo.

## 2. Confini e controllo anti-doppioni

- **[UX-019 — Storico espositivo e bibliografia *per opera*](REPS-UX-019-storico-esposizioni-bibliografia-documentata.md):** una pubblicazione o mostra relativa a una specifica opera appartiene lì. La proposta attuale riguarda **l'identità professionale dell'artista**: formazione, premi/borse, residenze e altre attività personali. Se in futuro entrambi gli studi saranno approvati, una voce di mostra o bibliografia deve essere **referenziata** da UX-019, non ricreata in una seconda tabella o ricavata dall'esistenza di un progetto.
- **[UX-009 — Progetti espositivi](REPS-UX-009-progetti-espositivi-checklist.md):** progetti pianificati e attività non dimostrano automaticamente mostre effettuate o premi ricevuti. Nessuna conversione da `planned` a «partecipazione professionale» senza dichiarazione.
- **[UX-007 — Dossier opere](REPS-UX-007-dossier-presentazione-opere.md)** e **`docs/BROCHURES-OFFLINE.md` su `main`:** compongono *documenti*, con titoli e introduzioni adattabili. La nuova idea, se mai approvata, è un **registro fonte facoltativo**, non un nuovo editor PDF né una copia della brochure.
- **[UX-014 — Rubrica relazionale](REPS-UX-014-rubrica-relazioni-professionali.md):** interlocutori/contatti non sono titoli accademici, borse o riconoscimenti. L'ente organizzatore può restare semplice testo; mai creare un cliente/contatto automaticamente.
- **[UX-006 — Incarichi su richiesta](REPS-UX-006-incarichi-opere-su-richiesta.md):** una commissione artistica assegnata non è di per sé un premio, e il suo processo economico rimane separato.
- **[TECH-001 — Pacchetto dati aperto](https://github.com/Cesare-ml/Repertica-Studio/blob/docs/idee-strumenti-architettura-20261121/docs/idee/strumenti-e-architettura/REPS-TECH-001-pacchetto-dati-aperto.md), [TECH-016 — Allegati locali](https://github.com/Cesare-ml/Repertica-Studio/blob/docs/idee-strumenti-architettura-20261121/docs/idee/strumenti-e-architettura/REPS-TECH-016-allegati-documentali-locali-multi-entita.md), [TECH-025 — Export selettivi](https://github.com/Cesare-ml/Repertica-Studio/blob/docs/idee-strumenti-architettura-20261121/docs/idee/strumenti-e-architettura/REPS-TECH-025-esportazioni-tabellari-minimizzate-condivisione.md):** possibili servizi trasversali, **non prerequisiti** approvati; qui si definiscono significato dei dati e UX. Nessun file dell'altro filone viene modificato.
- **Escluso:** Repertica Sites, pubblicazione online, scraping dei premi, invio candidature/email, attestazione di titoli, classifiche di prestigio, graduatorie automatiche, inferenze da vendite o collezionisti, dichiarazioni di appartenenza a collezioni pubbliche in base a una sola vendita.

## 3. Scenari professionali

1. **Candidatura a residenza:** un'artista necessita di CV di due pagine. Seleziona alcune voci di formazione, due residenze realmente concluse e tre riconoscimenti documentati. Esporta il risultato localmente; la selezione non cancella il master.
2. **Riconoscimento da verificare:** è riportata una «menzione speciale» in una fonte indiretta, senza documento ufficiale. La registra come *dichiarato / fonte da verificare*; non viene presentata come premio certificato.
3. **Cronologia incompleta:** un periodo di formazione è noto soltanto per anno, una residenza solo per mese. Date parziali e «non noto» rimangono tali; non inventare il primo gennaio.
4. **Brochure per una galleria:** l'utente desidera un profilo breve. Potrebbe scegliere un sottoinsieme di voci da integrare editorialmente in futuro, **senza cambiare** le descrizioni delle opere o creare una seconda fonte dei fatti.
5. **Archivio pluriennale:** un assistente corregge un nome di ente o una data in una voce già citata in un CV. Il registro conserva la rettifica e segnala che i PDF già esportati sono istantanee da aggiornare manualmente.
6. **Istituzione in una lista CV:** il nome di un museo come sede di evento **non prova** che possieda un'opera; eventuali «collezioni che ospitano opere» devono essere espressamente dichiarate, documentate e solo se necessarie.

## 4. UX candidata — minima e deliberatamente non invasiva

**Primo passo (validazione senza codice):** confrontare almeno 5 CV anonymizzati, 2 richieste reali di candidatura e un file `.odt` modello. Chiedere quali sezioni vengono aggiornate ripetutamente e se una semplice introduzione della brochure o un documento esterno sono sufficienti.

**Archivio → Profilo professionale (sezione facoltativa)**, senza obbligo di compilazione al primo avvio. Riepilogo «Eventi professionali registrati» e pulsante «Aggiungi voce», categorie minime *Formazione*, *Premio o riconoscimento*, *Borsa*, *Residenza*. Non chiamare «verificato» quanto è solo stato scritto dall'utente. Per portafogli di gallerie, l'identità dell'artista collegata alle voci richiede decisione esplicita: l'MVP non ha account e non deve fingere un'autenticazione personale.

**Aggiunta voce:** categoria; denominazione e organizzatore/sede testuali; anno o periodo con precisione dichiarata; stato *previsto* / *in corso* / *concluso* / *riportato senza conferma*; fonte descritta facoltativamente, breve nota interna e indicazione di riservatezza. «Premio candidato» non è «premio ricevuto». Campi minimi per una nota breve e mai compilazione obbligatoria dell'intero CV.

**Elenco e correzione:** raggruppato per categoria e ordine cronologico con spareggio stabile; ricerca locale e vista «da verificare». Rettifica collegata per cambi significativi a voci già pubblicate; nessuna finzione di audit con utente firmatario. Una voce può essere esclusa dal CV senza eliminarla dall'archivio.

**Componi CV (fase eventuale separata):** scegli formato *completo* o *selezione breve*, sezioni, ordinamento e singole voci. Mostrare **anteprima esatta** di ciò che sarà condiviso: intestazioni e date localizzate, testo originale invariato; voci non confermate escluse di default o rese esplicitamente qualificabili. Avvisare «Nessuna mostra/bibliografia importata da altre fonti» invece di simulare completezza. Nessuna selezione automatica «migliori premi».

**Accessibilità:** operazioni senza trascinamento obbligatorio, etichette/stati non basati su colore, tab e ritorno del focus prevedibili, descrizioni per screen reader, zoom 200%, stampa bianco-nero e anteprime leggibili. Layout responsivo della finestra Tauri.

## 5. Dati e interfacce puramente descrittive — NON schema da implementare

- `artist_profile` **solo se realmente necessario**: UUID, nome pubblico dichiarato, denominazione/alias facoltativi, breve testo professionale facoltativo, recapito **pubblicabile scelto esplicitamente**. Un singolo profilo locale può bastare nella fase iniziale; supportare più artisti andrebbe progettato solo dopo decisione di prodotto, senza derivare l'identità dal login inesistente.
- `career_entry`: UUID stabile; `profile_id`; categoria codificata (education/award/grant/residency/other); titolo; istituzione/ente e località come testo libero; `start_date/end_date` e `date_precision` con possibilità `null`; stato dichiarativo; fonte originale e attendibilità dichiarata; `recorded_at/updated_at`; nota interna; `supersedes_id` opzionale per rettifiche documentali.
- `career_source` **solo se indispensabile**: riferimento a documento/allegato locale, descrizione breve, eventuale URI **non richiamato automaticamente**; non assumere autenticità dall'hash del file. Evitare importi di borse e dati fiscali: quelli appartengono all'economia e non sono ricavi automaticamente registrati.
- `cv_selection` **fase successiva facoltativa**: UUID, profilo, lingua intestazioni, ID voci scelte in ordine, titoli di sezione e eventuali righe editoriali **distinte dai fatti registrati**; identità del documento e data di generazione. Un PDF esportato rimane un derivato, non prova di invio, lettura o accettazione.
- Collegamenti futuri a UX-019 tramite UUID del record **originale**, senza copie: se non disponibile, non generare una lista di mostre dal nulla. Nessuna relazione automatica con `Artwork`, `Sale`, `Collector`, `Expense`, quote o `Certificate`.
- Operazioni concettuali: «lista voci», «salva/ret­tifica», «scegli selezione», «anteprima», «esporta». Non sono API Tauri da creare ora.

## 6. Impostazioni, offline, multilingua, privacy

- **Impostazioni:** sezione visibile/nascosta, ordinamento, massimo numero di voci in CV breve solo se richiesto; campi pubblicabili e lingua del documento separati dalle preferenze UI. Nessun profilo online, identità obbligatoria, abbonamento, login o form iniziale imposto.
- **Offline e piattaforme:** eventuali record soltanto in SQLite locale e allegati sotto media gestiti, indipendenti da connessione e dagli account; UI Windows/macOS/Linux senza upload, telemetria, API esterne, ricerca automatica di enti, accesso al browser o invii. Nessun cronoprogramma eseguito a programma chiuso.
- **Multilingua:** rileva lingua OS, supporto IT/EN/ES/FR/DE con scelta manuale e fallback inglese; chiavi di categoria/stato stabili nell'archivio. Tradurre interfaccia, intestazioni e messaggi, **mai** trasformare automaticamente nomi di istituzioni, titoli, attestazioni, testi o qualifiche. Date incomplete formattate secondo locale senza aggiungere falsa precisione; nessuna equivalenza automatica di titoli di studio.
- **Privacy:** recapiti privati, data/luogo di nascita, organizzazioni e membri della giuria non entrano nel CV per default; dati personali di terzi minimizzati. Una lista di collezionisti o una vendita non dimostra una collezione pubblica. Anteprima con indicatori dei campi inclusi; export in chiaro da custodire con attenzione. I backup integrali attuali **non sono cifrati** e non devono essere descritti come sicuri su cartelle condivise per il solo fatto di essere verificati.
- **Documenti esterni:** PDF/`.odt` salvati fuori dall'archivio attivo non fanno automaticamente parte del backup Repertica; avviso esplicito prima del salvataggio.

## 7. Import/export, compatibilità e alternative

- **Export minimo:** CSV UTF-8 con UUID, categoria, titolo, ente, periodo+precisione, stato e riferimento fonte, con esclusione opzionale di note/fonti private; non confondere questo CSV con un backup completo. Un file di CV leggibile e modificabile (`.odt` o HTML statico) è da valutare prima di sviluppare renderer nuovi; PDF tramite modalità locale già disponibile **solo se applicabile**, senza presumere riuso immediato dell'attuale renderer brochure. Nessuna conformità PDF/A dichiarata.
- **Round trip eventuale:** JSON versionato e con ID stabili per voci, fonti e selezioni; import soltanto guidato con anteprima, deduplica prudente, scelta dei conflitti e rollback transazionale. Non creare profili, eventi né premi da righe ambigue senza conferma. I CSV integrali esistenti non diventano automaticamente reimportabili.
- **Compatibilità:** nessuna nuova colonna obbligatoria sulle opere, schede collezionisti, vendite o brochure storiche; archivi precedenti validi anche senza CV. Backup/ripristino su cartella liberamente scelta devono preservare eventuali nuove entità/allegati senza cambiare il formato attuale in modo silenzioso; vecchie versioni dell'app non sono necessariamente compatibili con futuri schemi, da comunicare prima di un eventuale rilascio.
- **Dipendenze certe oggi:** UUID, i18n, archivio locale, backup, esportazione. **Opzionali e non approvate:** UX-019 per mostre/bibliografia, TECH-016 per allegati, TECH-001 per pacchetti interoperabili, componenti brochure per impaginazione. Repertica Sites non è richiesto.
- **Alternative più economiche:** campo note professionali + modello `.odt` fornito come **documentazione**, oppure tabella CSV compilata manualmente e poi importata in LibreOffice; tenere il master CV fuori da Repertica se gli utenti non vogliono un nuovo dominio.
- **Complessità:** bassa-media per voci testuali e filtri; media per fonti/rettifiche/periodi parziali e export; medio-alta per aggregazione senza doppioni da UX-019, selezioni storiche e impaginazione robusta.
- **Rischi:** presentare candidature o mostre pianificate come concluse; attribuire meriti non verificati; far comparire nome di collezionista; rendere divergenti CV, brochure e storico opere; internazionalizzazione che altera titoli originali; aumentare struttura e oneri per chi usa pochi record. Mitigare tramite etichette, fonte unica per ogni dominio, anteprima, selezione manuale e fase minima senza editor ricco.

## 8. Test / criteri di accettazione ipotetici

1. Una voce `award` *candidatura* non compare come *premio ricevuto* e una residenza prevista non viene convertita in conclusa dal solo trascorrere della data.
2. Due eventi con date solo annuali restano rappresentati come tali, anche dopo backup, CSV/JSON e rendering in una UI con lingua diversa.
3. Correggere una voce non cambia opere, collezionisti, vendite, spese, quote, incassi, certificati storici o una brochure già esportata.
4. Una mostra di UX-019, se approvata, è **referenziata una volta**, non duplicata e non attribuita a tutti gli artisti presenti nell'archivio.
5. Una versione breve scelta manualmente include solo le voci approvate per quell'export, senza dati privati in testo nascosto, metadati o file accessori; l'utente vede l'anteprima esatta prima del file.
6. Il funzionamento è interamente offline su Windows/macOS/Linux, con lettura tastiera/screen reader e UI in lingua di sistema, override e fallback inglese.
7. Un archivio preesistente senza voci si apre e non genera carriera sintetica, premi o istituzioni fittizie; export/import di prova idempotente non moltiplica UUID; un restore da cartella scelta preserva tutte le voci e le fonti.
8. Se si mantiene solo la soluzione `.odt` esterna, la documentazione avverte chiaramente che la copia non è inclusa automaticamente nel backup dell'app.

## 9. Fasi possibili, senza assegnazioni

- **0 — Validazione:** 5–10 CV anonimizzati di artisti indipendenti, richieste reali di gallery/call; confronto con semplice modello `.odt`, decisione se vale la pena procedere.
- **A — Registro ridotto:** voci di formazione, riconoscimenti e residenze testuali, fonte/stato, filtri e CSV locale; **nessun CV generato necessariamente**.
- **B — CV selettivo:** export accessibile e controllato con intestazioni localizzate, selezioni brevi/lunghe e anteprima, senza sincronizzazioni.
- **C — Riuso dei dati:** riferimento non duplicante a UX-019, eventuale integrazione facoltativa nelle brochure e allegati, solo dopo verifica di compatibilità e reale domanda.

## 10. Requisiti certi, ipotesi e domande per revisione personale

**Requisiti certi dal progetto:** app gratuita/open source Tauri/Rust/SQLite; database e media locali; nessun account o abbonamento; backup verificato su cartella scelta; lingue di sistema con fallback inglese; brochure già implementata; certificati storici immutabili; esportazioni CSV non equivalenti a backup.

**Ipotesi:** gli artisti mantengono abitualmente più CV; esiste valore nel registrare meriti formativi e riconoscimenti in Repertica anziché in LibreOffice; un registro minimale riduce duplicazioni di testo.

**Domande aperte al proprietario:** Repertica deve contenere **un solo artista per archivio** o diversi? Sono frequenti candidature e brochure con CV breve? Per voci poco verificabili è preferibile escluderle o consentire la dicitura «dichiarato»? I riconoscimenti richiedono allegati reali? È più utile esportare `.odt`, testo/HTML o solo PDF? È giusto **non sviluppare** se i documenti esistenti risultano più semplici?

**Esito:** proposta esclusivamente ideativa, **da valutare personalmente**. Nessun codice, test, workflow, configurazione, issue, assegnazione, merge o sviluppo autorizzati.
