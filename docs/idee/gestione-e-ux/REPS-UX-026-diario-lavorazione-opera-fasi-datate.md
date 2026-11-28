# REPS-UX-026 — Diario di lavorazione delle opere: passaggi datati e fotografie di processo

- **ID stabile:** REPS-UX-026
- **Data:** 2026-11-21
- **Stato:** Proposta — da valutare
- **Filone:** Gestione e UX
- **Priorità ipotetica:** P2; possibile P1–P2 per studi che documentano regolarmente la produzione, **solo dopo interviste**
- **Perimetro:** esclusivamente analisi/documentazione. Nessuna approvazione, assegnazione, modifica al software o mandato di implementazione.

## 1. Problema, utenti ed evidenze

**Problema.** La scheda finale di un'opera conserva titolo, anno, tecnica, supporto, dimensioni, immagini e stato commerciale, ma non racconta **come e quando è stata prodotta**: preparazione del supporto, passaggi fotografati, modifiche di tecnica, tempi di asciugatura, firma, verniciatura, incorniciatura e conclusione dichiarata. Artisti, assistenti di studio e archivi possono aver bisogno di ricostruire la genesi di un'opera per una pubblicazione, per il proprio lavoro o per verifiche interne, senza affidarsi a nomi di file e note sparse.

**Destinatari:** artisti che lavorano a opere originali anche **senza commissione**; piccoli studi con assistenti; gallerie che raccolgono informazioni di produzione direttamente dall'artista; archivi d'autore. Un diario strutturato non deve gravare su chi cataloga soltanto opere già terminate.

**Riscontro su `main` (21/11/2026):** [README](../../../README.md), [modelli](../../../src/app/models.ts), [MVP](../../MVP.md) e [portabilità](../../DATA-PORTABILITY.md) documentano opere, galleria, media center, fotografie classificabili `WORK_IN_PROGRESS`, e campi correnti `signed`, `final_varnish`, `framed`. I modelli esaminati espongono `ArtworkStatus = available | reserved | sold | archived` con significato **commerciale**, non uno stato di avanzamento della lavorazione. La classificazione di una foto come `WORK_IN_PROGRESS` non costituisce una cronologia di azioni, e i tre booleani non registrano **quando** un'operazione sia stata compiuta. Non risulta nei file consultati un diario datato per opera. Questa è una **lacuna progettuale possibile**, non un bug verificato nell'app nativa.

La necessità di un diario, il suo utilizzo frequente e i benefici in tempo risparmiato **non sono ancora dimostrati**. Occorre valutare su casi reali se sia sufficiente la galleria con didascalie.

## 2. Confini espliciti e coordinamento anti-doppione

- **[UX-006 — Incarichi su richiesta](REPS-UX-006-incarichi-opere-su-richiesta.md):** gestisce *rapporti con un committente, brief, revisioni, consegne e tappe concordate*. Il diario qui proposto registra *lavoro effettivamente eseguito su un'opera esistente*, inclusa quella iniziata per scelta autonoma, senza brief, cliente o accordo. Una scadenza dell'incarico non prova che un passaggio sia avvenuto e un passaggio non conferma una tappa contrattuale.
- **[UX-018 — Catalogazione seriale e bozze](REPS-UX-018-catalogazione-seriale-schede-bozza.md):** crea o completa *schede anagrafiche* in sequenza. Il diario non inventa opere, non attribuisce codici provvisori, non implementa bozze di catalogazione; richiede un `artwork_id` già registrato.
- **[UX-003 — Costi per opera](REPS-UX-003-costo-e-margine-per-opera.md)** e **[UX-008 — Scorte e consumi](REPS-UX-008-scorte-materiali-consumi.md):** sono le fonti eventuali di *valori monetari* e *quantità di materiali*. Una nota «usati tre tubetti» non è automaticamente uno scarico di magazzino né una spesa.
- **[UX-011 — Cicli creativi](REPS-UX-011-cicli-creativi-relazioni-opere.md):** relazioni **tra** opere, non storia **dentro** una singola opera; lo stesso diario può descrivere un bozzetto autonomo senza crearne uno nuovo.
- **[UX-025 — Istruzioni di allestimento e cura](REPS-UX-025-istruzioni-allestimento-cura-opera.md):** spiega **come intervenire in futuro**; il diario descrive **cosa è stato fatto in passato**, non contiene prescrizioni di sicurezza.
- **[UX-004 — Condizione](REPS-UX-004-rapporti-stato-conservazione.md):** registra *rilievi dello stato fisico e, in prospettiva, interventi conservativi*: riparazione/restauro **dopo** la realizzazione appartiene a quel perimetro, non deve essere duplicata da questo diario di produzione originaria.
- **[TECH-010 — Revisioni metadati](https://github.com/Cesare-ml/Repertica-Studio/blob/docs/idee-strumenti-architettura-20261121/docs/idee/strumenti-e-architettura/REPS-TECH-010-storico-revisioni-metadati.md):** lo storico delle modifiche a *titolo/tecnica/descrizione* non sostituisce il diario. **[TECH-016 — Allegati](https://github.com/Cesare-ml/Repertica-Studio/blob/docs/idee-strumenti-architettura-20261121/docs/idee/strumenti-e-architettura/REPS-TECH-016-allegati-documentali-locali-multi-entita.md)** potrebbe fornire file riutilizzabili solo se approvato; nessuna modifica ai suoi documenti.
- Non sostituisce una scheda tecnica scientifica, un sistema di time tracking per dipendenti, un registro di sicurezza del lavoro o un certificato d'autenticità. **Non** integra nuove funzioni nel Media Center.

## 3. Scenari d'uso concreti

1. **Dipinto realizzato senza committente.** L'artista registra «preparato il supporto» (10 marzo), «prima stesura» (13 marzo) e «firma» (aprile, giorno non ricordato). Associa a ogni passaggio una foto esistente o nessuna foto. Non deve compilare un preventivo o simulare una vendita.
2. **Opera rielaborata più volte.** Un artista registra una seconda stesura e una modifica del soggetto, con motivazione e fotografia. Il titolo attuale rimane nella scheda opera; i passaggi precedenti non vengono cancellati né usati per dichiarare la tecnica definitiva senza conferma.
3. **Archivio retrospettivo.** Una fotografia è etichettata come «work in progress» ma la data reale è ignota. L'archivista la collega a un passaggio con *data non verificata* e fonte «testimonianza dell'artista», senza assumere che la data EXIF coincida con la produzione.
4. **Studio con più opere in lavorazione.** Da una vista facoltativa si vedono le opere con un evento recente o con una **prossima azione annotata**, senza confondere l'assenza di eventi con «lavoro concluso».
5. **Condivisione selettiva.** Per un catalogo l'artista esporta tre fotografie e alcune note editoriali sulla genesi, ma non tempi privati, bozze scartate, dati di clienti o ubicazione.
6. **Opera venduta.** La cronologia di produzione resta consultabile e non modifica prezzo, stato di vendita, certificati emessi o incassi. Si possono aggiungere memorie storiche con data dell'evento distinta da data di registrazione.

## 4. Esperienza utente e flussi proposti

**Opera → Processo creativo** come sezione opzionale, vuota per default con testo «Nessun passaggio documentato». L'utente non deve attivare un modulo, compilare una checklist né scegliere un «metodo di lavoro» per creare o vendere un'opera.

**Registra passaggio:** form breve con descrizione obbligatoria, data dell'evento *facoltativa* e precisione dichiarata (giorno, mese, anno, periodo impreciso o sconosciuto), categoria opzionale (preparazione, lavorazione, finitura, documentazione, altro), fonte/operatore *dichiarato* e una o più immagini locali già associate all'opera, oppure una nuova copia gestita dal media store. La data di inserimento è acquisita separatamente. Anteprima e **Salva**; nessuna conclusione automatica. Se l'utente vuole annotare soltanto due righe, non deve caricare foto o usare categorie.

**Pianificato ≠ eseguito:** una prossima azione («attendere asciugatura», «verificare cornice») può essere una nota futura distinta e visibilmente **non eseguita**. Nel primo esperimento preferire **solo eventi avvenuti**, lasciando promemoria e pianificazione a una fase separata; non trasformare note future in eventi compiuti per trascorrere della data.

**Timeline:** ordinamento per data dell'evento e, per date parziali, ordinamento stabile esplicito con indicatore «periodo indicativo»; possibilità di mostrare in ordine di registrazione. Ogni scheda presenta testo, autore/fonte dichiarati, foto e indicazione del livello di certezza. Correzioni sostanziali dei passaggi già consolidati producono una **rettifica collegata**, non una riscrittura invisibile. Evitare la pretesa di firma/autorialità verificata quando l'app non prevede account.

**Vista trasversale facoltativa:** «Lavorazioni» filtra opere con eventi recenti, passaggi marcati da rivedere e ultime attività. Non si assegna automaticamente `status='available'` alla conclusione del lavoro, né `status='archived'` alla sospensione; gli stati commerciali dell'MVP restano intatti.

**Riepilogo condivisibile:** selezione consapevole di singoli eventi/immagini, ordine e intestazioni; anteprima in modalità **pubblicabile** con default che omette ore, stime, note riservate, ubicazione e soggetti terzi. Il testo dell'utente non viene tradotto né riscritto automaticamente.

**Accessibilità:** navigazione completa tastiera, immagini con didascalia/alternativa testuale, sequenza logica senza drag-and-drop obbligatorio, filtri annunciati agli screen reader, date in testo e non soltanto colore, zoom al 200%, layout compatto e stampa ad alto contrasto.

## 5. Dati necessari e contratti concettuali (NON schema approvato)

- `artwork_process_entry`: UUID, `artwork_id` (FK), descrizione, `event_kind` opzionale con codice stabile, `occurred_on`/periodo e `date_precision` (possono essere null), `date_source`/certezza, `recorded_at`, `recorded_by_label` opzionale, `source_note`, visibilità suggerita (interno/condivisibile) **non considerata una misura di access control**, `supersedes_entry_id` per rettifiche, timestamps tecnici.
- `artwork_process_media`: UUID, `entry_id`, `media_id` o percorso relativo sicuro, ordine, descrizione dell'immagine e riferimento alla foto già presente nell'opera ove applicabile. Niente duplicazione obbligatoria dei file; non eliminare una fotografia condivisa quando si rettifica un evento.
- **Opzione futura, non MVP:** `artwork_process_plan` separato, con testo, data obiettivo facoltativa, stato esplicito e chiusura manuale. **Mai** derivare eventi di esecuzione dal semplice superamento di una data.
- **Durata di lavoro** facoltativa in minuti interi soltanto se l'utente la richiede; non dedurla dall'intervallo tra due eventi. Eventuale tariffa oraria e costo figurativo appartengono a UX-003, non al diario.
- **Nessun effetto collaterale:** eventi, categorie e foto non cambiano `artworks.status`, `signed`, `final_varnish`, `framed`, prezzi, quote, `expenses`, `sales`, `receipts`, certificati o scorte. Le spunte nella scheda descrivono condizioni correnti, ma non generano retroattivamente un «evento di firma» con data inventata.
- **API puramente descrittive:** «elenca passaggi dell'opera», «registra evento», «rettifica evento», «collega immagine selezionata», «anteprima selettiva», «esporta/leggi pacchetto versionato». Non sono comandi Tauri da creare in questa attività.

## 6. Offline, impostazioni, multilingua e privacy

- **Offline-first:** dati e foto nel solo archivio SQLite/media locale; nessun account, abbonamento, cloud, OCR, IA obbligatoria, sincronizzazione, indicizzazione remota, caricamento o notifica verso terzi. File esterni non vengono toccati o aperti senza selezione.
- **Impostazioni facoltative:** preferenza mostra/nascondi sezione, elenco ridotto di categorie favorite e ordinamento timeline; non imporre workflow, fase iniziale o giorni standard di essiccazione. Niente cronometro in background senza consenso esplicito.
- **Multilingua:** lingua sistema operativo, fallback inglese e override IT/EN/ES/FR/DE conformemente a [ECONOMY-I18N](../../ECONOMY-I18N.md). Tradurre etichette, date, stati, categorie ed errori; testo libero e titolo opera **restano nella lingua originale** con eventuale tag lingua dichiarato. Date persistite in forma ISO + precisione, non ricostruite da una stringa formattata localmente.
- **Privacy:** il processo creativo può contenere opere inedite, figure riconoscibili, studio privato, collaboratori, fotografie di committenti, formule di lavorazione e informazioni commercialmente riservate. Le note sono **interne di default**, anche se una foto della galleria è marcata `WORK_IN_PROGRESS`; non dedurre dal tag l'autorizzazione alla pubblicazione. Anteprima dei file/metadati e scelta consapevole dei contenuti prima di esportare/stampare. I backup correnti sono **in chiaro**: evitare destinazioni pubbliche non protette e non dichiararli cifrati.

## 7. Import, export, compatibilità e dipendenze

- **Esportazione leggibile:** stampa/PDF selettivo del processo di una sola opera, con data/fonte e foto scelte; CSV UTF-8 per tabella interna di eventi (UUID, opera, tipo, data/precisione, testo scelto, fonte) senza allegati binari e con avviso sui dati riservati.
- **Round-trip futuro:** JSON **versionato** con UUID dell'opera e degli eventi, precisione date, rettifiche, media con manifest/hash e percorsi relativi. Import da file selezionato dall'utente, anteprima, ricostruzione delle dipendenze, deduplica per UUID, scelta esplicita dei conflitti, staging e rollback atomico. **Non** spacciar gli attuali CSV (12 registri in uscita) per formato importabile.
- **Compatibilità:** dati additivi, nessun diario fittizio per opere storiche. La scheda opera e i file esistenti rimangono validi; database futuri più nuovi potrebbero non essere aperti da vecchie build, senza promessa di downgrade. Certificati emessi, copie fotografiche originali e valori delle vendite non cambiano. I backup verificati su **cartella liberamente scelta** devono includere eventi, rettifiche e allegati; restore completo non deve lasciare riferimenti orfani.
- **Piattaforme:** Windows/macOS/Linux, nomi Unicode, file su disco interno/USB/NAS già montato, nessuna dipendenza da un servizio web.
- **Dipendenze minime se autorizzato:** UUID opera, database, immagini e gestione media già presenti, localizzazione, backup. Integrabilità futura con UX-003/006/008/018 e TECH-001/010/016 **solo se approvate**; non sono prerequisiti per un registro testuale datato.

## 8. Rischi, complessità, alternative

- **Complessità:** bassa-media per timeline testuale minimale; media per rettifiche e navigazione/filtri; medio-alta per media multipli, condivisione selettiva e import/export di recupero con integrità.
- **Rischi:** date EXIF erroneamente trattate come fatti storici; fotografie pubblicate senza consenso; cronologia che sembra prova notarile di paternità/diritti; foto eliminate perché condivise da più eventi; duplicazione di materiale e costi; uso dei tre booleani esistenti come «milestone automatiche»; eccessivo carico di inserimento.
- **Mitigazioni:** distinguere data evento/data inserimento/precisione/fonte; rettifiche e null espliciti; anteprima privacy; gestione referenziale media; nessun cambio di registri economici, inventariali o commerciale; attività interamente opzionale.
- **Alternativa più leggera:** utilizzare le foto `WORK_IN_PROGRESS` già disponibili con didascalie curate e una nota libera nella descrizione dell'opera. **Prima di un nuovo modello** verificare con utenti reali cosa perdono con questa soluzione (date, cronologia consultabile, ricerca, provenienza delle affermazioni).
- **Alternativa esterna:** diario personale in Markdown/foglio di calcolo locale, con link manuali a codice opera e fotografie; buona indipendenza ma legami meno affidabili. Un'integrazione con l'app non deve rendere obbligatoria l'importazione.

## 9. Criteri di accettazione ipotetici (solo per revisione)

1. Un'opera esistente riceve due eventi datati e uno con data imprecisa; la timeline distingue quando sono **accaduti** da quando sono stati **registrati**, senza ordinamento temporale fittiziamente preciso.
2. Aggiungere «opera firmata» **non** imposta automaticamente `signed` e spuntare `signed` nella scheda non crea un evento retrodatato.
3. Un evento senza foto è legittimo; collegare una foto `WORK_IN_PROGRESS` non modifica i pixel, i media originali o i certificati.
4. Correggere una data registrata male conserva la versione precedente e la motivazione; archiviare la scheda non elimina la storia.
5. Non vengono create spese, ore tariffate, scarichi di scorta, incassi o vendite per i passaggi; il registro economico mantiene esattamente gli stessi totali.
6. Una prova di esportazione esterna include solo eventi e media esplicitamente scelti; nomi, ubicazioni, note riservate, EXIF e contatti non vengono divulgati per default.
7. Un'importazione ripetuta di prova mantiene UUID, date parziali, riferimenti e rettifiche senza moltiplicare eventi; errore durante import non lascia modifiche parziali.
8. Backup e ripristino in cartella liberamente scelta ricostruiscono il diario e tutti gli allegati; funzionamento senza Internet e senza account su piattaforme supportate.
9. Navigazione e stampa accessibili da tastiera/screen reader, layout adattivo, interfaccia localizzata dal sistema con fallback inglese; descrizioni originali non alterate.
10. Un archivio senza passaggi mostra «non documentato», **non** «mai lavorato» o «completato».

## 10. Fasi possibili, senza roadmap approvata

- **A — Validazione:** osservare 3–5 artisti/archivisti; raccogliere 2–3 esempi di processo reale; confrontare le didascalie `WORK_IN_PROGRESS` esistenti con un semplice diario; misurare tempo aggiuntivo richiesto.
- **B — Eventuale MVP:** eventi testuali e data con precisione esplicita, fonte, timeline per opera, senza pianificatore, spese, nuovi stati o automatismi.
- **C — Solo se utile:** collegamento non distruttivo a fotografie già gestite, rettifiche, esportazione selettiva e test di backup/round-trip.
- **D — Futuro subordinato a bisogno:** vista trasversale, note pianificate separate, collegamenti volontari a incarichi, materiali e analisi economica senza conti paralleli.

## 11. Requisiti certi, ipotesi e domande al proprietario

**Requisiti certi dal prodotto:** desktop gratuito/open source, Tauri/Rust/SQLite, archivi locali, nessun login o abbonamento, backup completo e ripristino nella cartella scelta, esportabilità, lingua OS con fallback inglese, stati commerciali e vendite separati dai dati descrittivi; galleria con categoria `WORK_IN_PROGRESS` già disponibile.

**Ipotesi:** l'artista desidera ricostruire passaggi e decisioni nel tempo; una semplice didascalia non basta; gli archivi ricevono testimonianze e foto con date incomplete; una vista di lavorazioni attive riduce dispersione di informazioni.

**Questioni da valutare personalmente (senza richiesta di risposta immediata):**
1. L'utente prioritario è l'artista in produzione o l'archivista che ricostruisce opere storiche?
2. La prima versione deve essere **solo storia passata** o serve davvero anche un promemoria «da fare» distinto?
3. Bastano eventi testuali e collegamenti alle foto già presenti, evitando un sistema autonomo di allegati?
4. Quali categorie di processo sono realmente condivise tra pittura, scultura, fotografia e installazioni, e quante devono essere libere?
5. È utile indicare durate di lavorazione o sarebbe un onere eccessivo, meglio lasciato a UX-003?
6. Il processo creativo va pubblicato raramente (con selezione manuale) oppure è una componente frequente dei dossier?

**Decisione attuale:** «Proposta — da valutare». Nessuna funzionalità approvata, assegnata o avviata.
