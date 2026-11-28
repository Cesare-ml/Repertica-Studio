# REPS-UX-055 — Revisione dei possibili doppioni di schede-opera senza fusioni automatiche

- **ID stabile:** REPS-UX-055
- **Data:** 2026-11-23
- **Stato:** Proposta — da valutare
- **Filone:** Gestione e UX
- **Priorità proposta:** P2 se il problema emerge in archivi con molte opere o import storici; P3 o nessuno sviluppo per cataloghi piccoli e ben curati
- **Natura:** sola ricerca e documentazione. Nessuna approvazione, assegnazione, implementazione o richiesta di modifica del software.

## 1. Problema, utenti e riscontri verificati

**Problema professionale da verificare.** Dopo anni di catalogazione, una stessa opera fisica potrebbe essere descritta da due schede diverse, con codici e UUID distinti: ad esempio una prima scheda inserita in fretta durante una mostra e un secondo inserimento con titolo appena modificato. In un archivio di centinaia di opere può sfuggire. Due schede per un oggetto possono gonfiare il numero delle opere apparentemente disponibili, generare presentazioni incoerenti o rendere complicata una ricerca. **La somiglianza, però, non dimostra che sia lo stesso oggetto**: titoli ricorrenti, studi preparatori, copie d'artista, dittici e tirature possono essere correttamente catalogati come opere differenti.

**Verificato nel branch `main` al 23/11/2026, tramite connettore GitHub:**

- [Modello Rust](../../../src-tauri/src/models.rs) e [Angular](../../../src/app/models.ts): ogni opera possiede UUID `id`, `code`, titolo, anno testuale, tecnica, supporto, dimensioni, eventuale immagine e stato commerciale. Il codice è distinto dall'identità fisica.
- [Schema SQLite](../../../src-tauri/src/db.rs): `artworks.code` è univoco con confronto `NOCASE`. Il vincolo impedisce un codice identico riutilizzato, **non** due record con codici diversi riferiti, per errore umano, allo stesso oggetto.
- [Ricerca archivio](../../../src/app/archive-filters.ts) e [README](../../../README.md): ricerca per parole/campi, filtri e ordinamenti già disponibili. Non emerge, dai percorsi verificati, una revisione dedicata di **coppie di schede-opera** basata su segnali spiegabili.
- [MVP](../../MVP.md): il **Media Center** trova file fotografici identici (SHA-256) o simili (dHash). È una funzione di **immagini**, non una decisione sull'identità delle opere: la stessa foto può rappresentare un'edizione o venire riutilizzata come riferimento, e una sola opera può avere fotografie differenti.
- Vendite e certificati esistenti possono essere legati a una delle due schede candidate: qualunque proposta che le accorpi senza revisione sarebbe inappropriata. Non è stata osservata una duplicazione reale, né svolto un test d'uso con artisti.

**Destinatari:** artisti che ricostruiscono archivi estesi, gallerie che hanno avuto più operatori, archivi che importano cataloghi storici. L'utente con poche opere potrebbe non aver bisogno di questa funzione.

**Lacuna ≠ malfunzionamento confermato:** lo schema rende possibile l'errore semantico, ma non è dimostrata una frequenza significativa. Il caso va validato prima di introdurre nuove schermate.

## 2. Confini con il catalogo delle idee — niente doppioni di proposta

- **[UX-018 — Inserimenti seriali](REPS-UX-018-catalogazione-seriale-schede-bozza.md)**: supporta l'inserimento manuale ripetuto e il controllo di *codici* già assegnati; qui si studia la **revisione di record esistenti con codici diversi**, senza obbligare un nuovo flusso di catalogazione.
- **[UX-005 — Edizioni e varianti](REPS-UX-005-edizioni-multipli-serie-numerate.md)** e **[UX-011 — Cicli e studi](REPS-UX-011-cicli-creativi-relazioni-opere.md)**: opere intenzionalmente simili non vanno collassate; se approvate, queste relazioni potrebbero spiegare somiglianze, non forzare una fusione.
- **[UX-048 — Identificativi storici](REPS-UX-048-identificativi-storici-riferimenti-esterni-opere.md)**: studia riferimenti alternativi documentati **della stessa scheda**; qui non si attribuisce automaticamente un codice di un record a un altro.
- **[UX-015 — Ricognizioni inventariali](REPS-UX-015-ricognizioni-inventariali-discrepanze.md)**: verifica presenza fisica; qui si confrontano **descrizioni nel database**, senza accertare presenza o provenienza.
- **[TECH-019 — Import tabellari](../strumenti-e-architettura/REPS-TECH-019-importazione-guidata-cataloghi-tabellari.md)** e **[TECH-001 — Pacchetto aperto](../strumenti-e-architettura/REPS-TECH-001-pacchetto-dati-aperto.md)**: controllano l'identità e i conflitti **durante un import esplicito**. La presente proposta riguarda la revisione volontaria di un **archivio già esistente** e non crea un secondo importatore.
- **[TECH-008 — Filtri e viste salvate](../strumenti-e-architettura/REPS-TECH-008-viste-salvate-filtri-ricerca-locale.md)**: eventuale infrastruttura riutilizzabile; la presente scheda studia il **giudizio umano, la presentazione delle prove e i limiti delle conclusioni**, non un motore di ricerca generale.
- **[TECH-010 — Cronologia metadati](../strumenti-e-architettura/REPS-TECH-010-storico-revisioni-metadati.md)** e **[TECH-030 — Vocabolari](../strumenti-e-architettura/REPS-TECH-030-vocabolari-locali-tecniche-supporti-interoperabili.md)**: possibili integrazioni non obbligatorie. Nessun file dell'altro filone viene modificato.

**Esclusioni rigorose:** deduplicazione automatica; fusione degli UUID; cancellazione in serie; ricollegamento di vendite, certificati, fotografie o clienti; uso obbligatorio di IA; attribuzioni di autenticità; import automatico; correzione automatica delle descrizioni; modifica delle statistiche economiche per semplice somiglianza.

## 3. Scenari e flussi UX ipotetici

1. **Due schede della stessa tela, per errore.** `ART-0041` («La finestra», 2021, 50 × 70) e `ART-0330` («Finestra», 2021, 50 × 70) sembrano riferirsi alla stessa tela. L'operatore vede le due foto, descrizioni e cronologie, indica «Da verificare» e decide al di fuori di questo flusso come correggere le schede. Il sistema non ne sceglie una come «originale».
2. **Titoli comuni, opere diverse.** Dieci dipinti si chiamano «Senza titolo». Non devono produrre un elenco ingestibile di allarmi «duplicato certo»: senza dimensioni, anno o altri segnali concordanti si segnala, al più, una debole somiglianza da ignorare.
3. **Tiratura fotografica.** Due esemplari 3/25 e 4/25 possono condividere titolo, misure e immagine digitale. Sono oggetti individuali con identità distinte. La schermata consente di contrassegnare «Distinte — edizione / multiplo» **soltanto come giudizio dichiarato**, non come relazione di tiratura già implementata.
4. **Vendita o certificato già collegato.** Una candidata risulta venduta e possiede un certificato immutabile; l'altra è disponibile. Evidenziare il contesto e l'avvertenza **«Non modificare automaticamente i legami storici»**. Non suggerire un pulsante «Unisci».
5. **Import storico incompleto.** Due record hanno foto assente e `year = "circa 1990"`. Non inventare una data esatta, né confrontare centimetri nulli come zeri. L'operatore può aprire entrambe le schede e continuare a raccogliere documentazione.
6. **Riesame periodico.** In un archivio ampio l'utente avvia, solo quando vuole, una revisione, completa alcune coppie e può interrompere senza alterare opere o fatturato.

**Esperienza minima proposta:** da **Opere → Strumenti archivio → Verifica possibili doppioni**, azione esplicita e non intrusiva. Dopo un'analisi tutta locale appare una lista di coppie candidate, ciascuna con **perché è stata segnalata**: titolo simile, anno compatibile, dimensioni vicine, tecnica/supporto coincidenti, eventuale foto riutilizzata. Ogni scheda mantiene **codice e UUID distinti**, foto, stato commerciale e collegamenti documentali visibili solo nell'interfaccia interna. Prima di proporre correzioni, pulsanti «Apri scheda A», «Apri scheda B», «Ignora per ora» ed eventualmente «Rivedi più tardi». **Nessun comando di fusione**.

**Riduzione del rumore:** mostrare inizialmente soltanto coppie con almeno due segnali indipendenti e dati sufficienti, e collocare i casi deboli in sezione separata/disattivata. «Senza titolo» e dimensioni mancanti non bastano. L'utente può scegliere di eseguire una ricerca manuale anche quando non emerge alcuna coppia; «nessun candidato trovato» **non significa** archivio certamente privo di doppioni.

**Accessibilità:** tabella/risultati con motivazioni testuali, navigazione completa da tastiera, confronto affiancato che degrada a pannelli sequenziali su schermi piccoli, zoom 200%, indicatori non solo cromatici, immagini con descrizione alternativa se presente, focus/annunci leggibili da screen reader. Nessun carosello visivo obbligatorio.

## 4. Dati e regole concettuali (NON API o database approvati)

**Fase di studio più leggera: zero nuove tabelle.** Leggere dati già presenti nell'archivio attivo e costruire in memoria candidati `{artwork_a_id, artwork_b_id, signals[], missing_fields[]}`; conservare sempre il **motivo** di ogni somiglianza. Nessun `confidence_score` percentuale che sembri un giudizio di identità. La ricerca può partire da campi catalografici e, solo in futuro, dalla fotografia già indicizzata nel Media Center, senza decodificare indiscriminatamente migliaia di RAW.

**Regole candidate, da provare su dati reali sintetici:**

- Priorità ai titoli normalizzati **senza perdere l'originale Unicode** (accenti, apostrofi, spazi); evitare equivalenze invasive fra titoli artistici distinti.
- Confrontare dimensioni soltanto se entrambe presenti e con unità certe; tolleranza millimetrica o in centimetri **da validare**, non assumere che piccole differenze provino identità. Non permutare larghezza/altezza senza indicarlo.
- Anno testuale approssimativo o assente = dato incerto, non evidenza concordante. Tecnica/supporto possono essere descritti con sinonimi o terminologie differenti: prima usare confronto letterale spiegabile.
- Un file identico o un'immagine simile è soltanto indizio fotografico, mai prova dell'unicità dell'opera; i due record possono condividere un media intenzionalmente.
- La mancanza di una coppia di indizi solidi non deve generare un blocco al salvataggio. Ogni candidato resta **possibile corrispondenza**, non «doppione trovato».

**Se servisse ricordare le verifiche (solo fase successiva):** metadati facoltativi `ReviewFinding {id, artwork_a_id, artwork_b_id, assessment: "unreviewed"|"distinct"|"needs_evidence", reason_note?, reviewed_at?}` con coppia ordinata stabilmente e possibile data ultima modifica delle due schede. Quando un record cambia, la valutazione è marcata **da ricontrollare**; il giudizio «distinte» resta dichiarazione dell'operatore, non certificazione. L'eventuale persistenza richiede migrazione additiva e inclusione nel backup, senza aggiungere un nuovo stato commerciale e senza alterare `sales`/`certificates`.

**Invarianti:** un UUID resta l'identità del record. Un `artworks.code` univoco è vincolo di codice, non di oggetto. Nessuna modifica indiretta a `status`, ubicazione, ripartizione guadagni, coefficienti o incassi. Nessuna conclusione di deduplicazione viene trasferita automaticamente ad archivi separati.

## 5. Impostazioni, offline, lingue, privacy e portabilità

- **Impostazioni:** analisi avviata manualmente, per default conservativa; eventuale filtro per anni/ubicazione e quantità massima di coppie per pagina. Non attivare un controllo bloccante su ogni salvataggio, notifiche invasive, scadenze ricorrenti o categorizzazioni AI senza una decisione futura. Eventuali soglie dovrebbero essere esplicitabili in linguaggio semplice e ripristinabili.
- **Offline-first:** confronti solo su SQLite e media locali dell'archivio attivo, senza rete, account, abbonamenti, backend remoto, telemetria, OCR obbligatorio o invio di fotografie. Non esegue contatti con servizi di immagini.
- **Multilingua:** stessa lingua OS con override locale e fallback inglese (IT/EN/ES/FR/DE), includendo motivi di corrispondenza, messaggi e stati del riesame. Titoli, tecniche, note e codici restano come registrati; eventuali valori tipizzati persistiti sono indipendenti dalla lingua.
- **Import:** fase minima **nessun nuovo import**. Le schede importate da strumenti esterni rimangono integre e, se presenti, possono semplicemente essere incluse nella revisione. In futuro un report delle valutazioni potrà essere reimportato **solo se** esiste un formato versionato con idempotenza, controllo UUID e conflitti, mai come autorizzazione a unire record.
- **Export:** report locale CSV/JSON di coppie *candidate* con UUID/codici, motivi e stato revisione, **senza** esportare fotografie, indirizzi dei collezionisti, prezzi o note per default; anteprima dei campi quando il report viene condiviso. Il report è diagnostico, **non** il formato canonico dell'archivio né una lista di «opere identiche» accertate.
- **Backup/ripristino:** la fase in memoria non genera file permanenti. Se un giorno fossero salvate valutazioni, queste devono entrare nello snapshot completo SQLite+media con manifest verificato, destinazione liberamente scelta e ripristino testato. I backup esistenti restano leggibili dalle versioni compatibili. Non confondere CSV con backup.
- **Privacy:** il confronto interno può mostrare nomi degli acquirenti, ubicazioni, fotografie di opere in collezioni private e riferimenti a certificati: non inviarli automaticamente, non inserirli nei report condivisibili di default. I backup attuali sono in chiaro, da proteggere sul dispositivo.

## 6. Compatibilità, dipendenze, complessità, alternative e rischi

**Complessità indicativa:** bassa–media per un controllo manuale a regole semplici con poche candidate; media per una revisione scalabile di migliaia di opere con paginazione e indici; alta per persistente storico delle valutazioni, riconoscimento fotografico evoluto o una vera fusione delle schede. **La fusione è esclusa** e richiederebbe una decisione e una progettazione distinte.

**Dipendenze:** ID/codici delle opere, ricerca corrente e eventuali legami a vendite/certificati. La funzione minima potrebbe non richiedere migrazioni, nuove librerie, tecnologie IA, né l'approvazione di TECH-008/019 o UX-005. L'eventuale confronto immagini può sfruttare diagnostica Media Center già esistente, con attenzione alla differenza file/opera.

**Rischi:** troppi falsi positivi per titoli generici; perdita di tempo degli utenti; somiglianza scambiata per prova di duplicazione; fotografie identiche di esemplari differenti; falsi negativi con titoli cambiati; carico di CPU/memoria su grandi archivi; divulgazione di dati in export; suggerimenti di fusione che corromperebbero riferimenti a vendite e certificati.

**Alternative da testare prima di progettare UI:** usare la ricerca attuale per titolo, tecnica, dimensioni e anno; esportare il CSV integrale delle opere e fare una ricognizione manuale in LibreOffice Calc, confrontando codici e fotografie con l'archivio. Se per i partecipanti è sufficiente, **nessuno sviluppo**. Non è necessario uno strumento dedicato per 10–20 schede.

## 7. Criteri di accettazione ipotetici — solo dopo eventuale approvazione

1. Un test con due record della **stessa opera** ma codici diversi produce una coppia motivata senza cambiare i record e senza dedurne identità legale o autenticità.
2. Una serie di dieci opere diverse intitolate «Senza titolo» non genera automaticamente decine di allarmi forti; casi ambiguamente simili restano spiegati come deboli.
3. Due esemplari distinti che condividono fotografia e dimensioni **non** vengono uniti, cancellati, rinumerati o trasformati in una singola vendita.
4. Due schede candidate con vendite, certificati e foto storiche preservano tutti gli ID e gli snapshot dopo analisi, uscita o riapertura dell'app.
5. Tutti i motivi della segnalazione sono verificabili; dati assenti/anno incerto non vengono convertiti in evidenze. Nessuna percentuale di certezza priva di validazione.
6. Nessun dato viene scritto o trasmesso dall'analisi base; un export facoltativo esclude di default dati personali ed economici e non sovrascrive file preesistenti.
7. Su 500–2.000 schede sintetiche, il confronto resta navigabile e interrompibile; misurare tempo, memoria, falsi positivi e falsi negativi prima di definire soglie o carichi ammessi.
8. UI Windows/macOS/Linux navigabile con tastiera/screen reader e zoom; messaggi in IT/EN/ES/FR/DE, fallback inglese; codice e UUID distinguibili.
9. Backup/ripristino, CSV correnti, vendite e certificati restano invariati; se la valutazione viene persistita in futuro, è conservata e ripristinabile senza riassociare schede per somiglianza.

## 8. Fasi di valutazione (non roadmap assegnata)

- **Fase 0 — Ricerca senza codice:** chiedere a 3–5 archivisti/artisti con almeno 100 opere se hanno incontrato schede doppie; chiedere esempi **anonimizzati** e misurare una revisione manuale con ricerca/CSV/Calc. Quantificare il costo dei falsi positivi.
- **Fase 1 — Eventuale controllo minimo:** comando volontario, indizi catalografici spiegabili, confronto affiancato, apertura delle schede. Nessun salvataggio di valutazioni, fusione o cambi a metadati.
- **Fase 2 — Solo se utile:** ricordare «distinte / da verificare» e motivazioni, aggiornare indizi dopo modifiche, report CSV consultivo e filtri migliori. Prima esaminare necessità di persistenza e compatibilità.
- **Fase 3 — Facoltativa:** integrazione controllata con esemplari/varianti (UX-005), riferimenti storici (UX-048) e importazione tabellare (TECH-019), **solo se tali filoni sono approvati separatamente**. Nessun merge di record compreso.

**Priorità ipotetica:** P2 per archivi importati o gestiti da più operatori; P3 oppure nessuna funzionalità nuova se la ricerca e il CSV esistenti bastano. Nessuna idea approvata.

## 9. Requisiti certi, ipotesi e domande per il proprietario

**Certi dal repository:** UUID e codice per opera; unicità del codice; modelli vendita/certificato con legami alle opere; Media Center già capace di cercare media simili; archivio locale, backup libero, multilingua, assenza di account obbligatorio.

**Ipotesi da verificare:** più schede possono descrivere involontariamente lo stesso oggetto; il rischio aumenta con dati importati, operatori multipli e cataloghi ampi; gli utenti vorrebbero poter **revisionare** possibili doppioni senza modificarli. Nessun dato reale o frequenza è stata raccolta.

**Questioni aperte:**

1. Sono stati riscontrati casi concreti di due schede per la stessa opera, oppure bastano i codici unici e la ricerca attuale?
2. Si desidera un controllo al momento dell'inserimento, una ricognizione avviata manualmente, o entrambe? La proposta minima favorisce **solo il riesame volontario**.
3. È preferibile mostrare pochi candidati con alta precisione, perdendo qualche caso dubbio, o un elenco più ampio con filtri?
4. Quali dati distinguono davvero opere vicine (foto del retro, firma, dimensioni rilevate, riferimenti storici) senza imporre campi obbligatori?
5. L'utente vuole registrare soltanto «distinte / da verificare», o preferisce non conservare alcuna decisione nel database?
6. È utile una prova su archivio sintetico prima di analizzare fotografie o metadati di un archivio reale?

**Esito:** proposta solo documentale, non approvata, non assegnata. Le altre idee restano indipendenti.
