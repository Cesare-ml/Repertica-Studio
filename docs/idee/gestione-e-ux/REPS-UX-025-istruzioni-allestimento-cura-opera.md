# REPS-UX-025 — Istruzioni di allestimento, movimentazione e cura per singola opera

- **ID stabile:** REPS-UX-025
- **Data:** 2026-11-21
- **Stato:** Proposta — da valutare
- **Filone:** Gestione e UX
- **Priorità ipotetica:** P2; P1–P2 soltanto per archivi con molte installazioni e opere tecnicamente complesse, dopo validazione con utenti
- **Natura:** esclusivamente analisi e documentazione; **non** è una funzionalità approvata, una prescrizione di conservazione, un incarico di sviluppo o una modifica al prodotto

## 1. Problema, destinatari e riscontro

Una scheda anagrafica può descrivere titolo, tecnica, misure e fotografie ma non spiegare **come l'opera debba essere installata, maneggiata, esposta, spenta, smontata, imballata e custodita** senza perdere informazioni determinanti. Per un quadro ordinario può bastare una nota; per una scultura modulare, un'opera con luce, suono, elettronica o parti fragili, istruzioni trasmesse a voce o sparse in email rischiano di andare perse o di essere applicate nella versione sbagliata.

**Destinatari:** artisti che consegnano opere a gallerie; assistenti di studio; allestitori; piccole gallerie; archivi e conservatori che consultano indicazioni originali dell'artista o del produttore. La domanda per un modulo strutturato in un gestionale per artisti **non è dimostrata**: convalidare frequenza, incidenti e costo delle informazioni mancanti con almeno alcuni casi reali.

**Verificato su `main`, 21/11/2026:** README, `docs/MVP.md`, `src/app/models.ts` e ricerca del repository documentano scheda opera, tecnica, misure, immagine principale e galleria, backup locale, certificati storici e media center, ma non una **scheda strutturata di prescrizioni di allestimento e movimentazione versionata per opera**. Si tratta di opportunità specifica, non di bug constatato. La lettura del codice non equivale a una verifica desktop completa.

Riferimenti metodologici, **non** standard che Repertica dichiari di rispettare: [Collections Trust — Object requirement information](https://collectionstrust.org.uk/resource/object-requirement-information/) distingue raccomandazioni di esposizione, ambiente, movimentazione, imballaggio, sicurezza e deposito; [Collections Trust — Installation note](https://collectionstrust.org.uk/resource/installation-note/) descrive il bisogno di istruzioni di installazione quando un oggetto è in prestito. Non copiare procedure tecniche protette: registrare dati originali dell'utente.

## 2. Confini anti-duplicazione

- **[UX-022 — Componenti fisici](REPS-UX-022-opere-composte-componenti-fisici.md):** definisce **cosa compone** un'opera e quantità attese; questa scheda documenta **come montarla, usarla e conservarla**. Nessuna riga di istruzione crea un componente o ne certifica la presenza.
- **[UX-004 — Rapporti di condizione](REPS-UX-004-rapporti-stato-conservazione.md):** registra **ciò che è stato osservato** in una certa data, e cita interventi conservativi come possibilità futura; qui si conservano **indicazioni e vincoli dichiarati per l'uso futuro**, non osservazioni, perizie, trattamenti effettuati o danni.
- **[UX-009 — Progetti espositivi](REPS-UX-009-progetti-espositivi-checklist.md):** gestisce scadenze e attività **di una mostra specifica**; questa scheda segue l'**opera** attraverso mostre diverse, con possibilità di associare una versione a un progetto senza inventare checklist eseguite.
- **[UX-001 — Movimentazioni](REPS-UX-001-provenienza-movimentazioni.md), [UX-015 — Ricognizioni](REPS-UX-015-ricognizioni-inventariali-discrepanze.md):** non sono registrazioni di spedizione, uscita, rientro o presenza fisica. Un'istruzione di imballaggio non registra che l'opera sia stata imballata.
- **[UX-007 — Dossier](REPS-UX-007-dossier-presentazione-opere.md):** impagina la presentazione generale; qui esiste un documento tecnico operativo, separato per riservatezza e scopo.
- **[TECH-016 — Allegati riutilizzabili](https://github.com/Cesare-ml/Repertica-Studio/blob/docs/idee-strumenti-architettura-20261121/docs/idee/strumenti-e-architettura/REPS-TECH-016-allegati-documentali-locali-multi-entita.md)** e **[TECH-001 — Portabilità](https://github.com/Cesare-ml/Repertica-Studio/blob/docs/idee-strumenti-architettura-20261121/docs/idee/strumenti-e-architettura/REPS-TECH-001-pacchetto-dati-aperto.md):** eventuali servizi trasversali; non modificati né presupposti già disponibili.
- **Economia e certificati:** nessuna istruzione crea spese, quote, vendita, incasso o un certificato. I certificati già emessi conservano il proprio snapshot immutabile.

## 3. Scenari concreti

1. **Installazione modulare.** Un'opera unica comprende quattro moduli e un supporto; l'artista registra sequenza di montaggio, orientamento, foto dei passaggi, materiali/accessori richiesti e operazioni inverse. La distinta dei componenti resta un registro separato (UX-022).
2. **Opera luminosa o sonora.** Lo studio dichiara condizioni richieste di alimentazione, funzionamento e spegnimento; una galleria stampa istruzioni per il personale. Il software **non certifica la sicurezza elettrica** o la compatibilità con gli impianti di sede.
3. **Dipinto con accorgimenti di manipolazione.** Sono annotati punti da non toccare, modalità di imballaggio e limiti dichiarati per esposizione/deposito. Se l'informazione proviene da un restauratore o è incerta, si registra autore/fonte; non si inventano valori ambientali.
4. **Riallesti successivi.** Nel 2027 una galleria utilizza la versione 1 delle istruzioni. Nel 2028 l'artista pubblica una revisione. Il fascicolo della mostra 2027 continua a dichiarare la **versione consultata allora**, senza essere riscritto retroattivamente.
5. **Informazioni discordanti.** Una vecchia scheda suggerisce un orientamento diverso: entrambe le fonti rimangono consultabili e marcate come da chiarire. Nessuna sostituzione automatica con istruzioni presunte più affidabili.

## 4. Flusso UX ipotetico

**Scheda opera → «Allestimento e cura»**: tab opzionale, inizialmente vuoto con messaggio «Nessuna istruzione registrata» (non «nessun requisito»). Per la maggior parte delle opere una nota libera basta: l'utente non deve compilare un questionario tecnico.

**Nuova istruzione**: selezione di sezioni facoltative — preparazione, montaggio, esposizione, ambiente, uso/spegnimento, movimentazione, imballaggio, deposito e smontaggio. Inserimento di passaggi ordinabili tramite pulsanti accessibili da tastiera, descrizione, eventuali foto/diagrammi locali, avvertenze testuali, ruolo professionale cui rivolgersi e informazioni mancanti. Separare **requisito dichiarato**, **raccomandazione**, **dato ancora da verificare** e **nota storica**: non codificare limiti tecnici numerici come se fossero soglie universali.

**Fonte e revisione**: scegliere autore dichiarato (artista, studio, conservatore, terzo, sconosciuto), data e precisione, eventuale titolo della fonte e versione. Anteprima completa prima di «Conferma versione». Dopo conferma, le rettifiche significative creano una **nuova revisione collegata** con motivazione; conservare le vecchie per contestualizzare allestimenti passati, non promettere firme legali.

**Consultazione e consegna**: pulsante «Anteprima istruzioni» con scelta delle sezioni, lingua delle intestazioni e media; esportazione/stampa locale ad hoc per l'allestitore, senza recapiti privati, costi, prezzi, ubicazione interna, login o note economiche. Opzione di includere esplicitamente fonte e qualifiche dell'autore, e avvertimento che l'utente deve **verificare le istruzioni con professionisti competenti** prima di compiere operazioni potenzialmente rischiose.

**Accessibilità:** editor non dipendente da drag-and-drop o sola codifica cromatica, sequenze numerate leggibili da screen reader, didascalie e testi alternativi, stato focus coerente, anteprima con ordine logico su schermi piccoli; consentire stampa in bianco e nero.

## 5. Dati necessari e API soltanto descrittive

**Ipotesi modellistica (NON migrazione approvata):**

- `artwork_care_guide`: UUID, `artwork_id`, titolo/etichetta, stato bozza/confermato/superato, versione intera e `supersedes_id` facoltativo, data redazione, autore/fonte dichiarati, precisione data, `language_tag` del testo, motivazione revisione e timestamp tecnico.
- `artwork_care_step`: UUID, `guide_id`, sezione stabile, ordine intero, testo, `instruction_kind` (requisito/raccomandazione/incerto), `source_note` facoltativa e avvertenza. Evitare marcatori automatici «sicuro» o «conforme».
- `artwork_care_media`: UUID, `step_id` o `guide_id`, percorso relativo gestito dall'app, tipo immagine/documento consentito, didascalia, testo alternativo e integrità byte ove disponibile; un file di terzi non diventa automaticamente attendibile per il solo hash.
- `care_guide_usage` **solo in fase successiva**, se UX-009 approvata: versione guida referenziata/snapshot associato a progetto; **nessuna** prova automatica di esecuzione degli step.
- Le raccomandazioni ambientali (ad es. temperatura/umidità) possono essere **testo libero** nella prima fase. Eventuali intervalli strutturati richiederebbero unità, validazione, fonte, contesto e gestione «sconosciuto» distinta da zero; mai suggerire valori di default per opere reali.

**API concettuali:** «lista versioni guida per opera», «salva bozza», «conferma nuova versione», «leggi versione storica», «anteprima esportazione selettiva», «esporta pacchetto portabile». Queste sono descrizioni di flussi, **non** nomi di endpoint, comandi Tauri o cambi di codice proposti come approvati.

**Compatibilità:** il modello resta additivo; le opere già presenti rimangono identiche e senza guide. UUID dell'opera e codici esistenti restano stabili; nessun cambio a vendita, certificato, stato commerciale, media center o schema di provenienza. Non dedurre automaticamente istruzioni dai metadati tecnici dell'opera.

## 6. Impostazioni, offline, lingue, privacy e interoperabilità

- **Impostazioni opzionali:** modello sintetico/dettagliato, sezioni preferite, formato stampa A4/Letter, intestazioni lingua documento, dati di firma/contatto pubblico **opt-in**, eventuale visibilità della versione corrente. Nessuna impostazione globale obbligatoria né soglie preimpostate presentate come linee guida conservative.
- **Offline:** SQLite + media locale. Funziona senza Internet, account, abbonamento, API esterne, webcam, sensori ambientali, scansione QR, notifiche e caricamenti automatici. Nessuna modifica a file esterni senza scelta esplicita.
- **Multilingua:** lingua iniziale del sistema operativo, fallback inglese e override secondo i18n esistente (IT/EN/ES/FR/DE). Categorie, etichette, avvisi e unità tradotti; **contenuti scritti dall'utente rimangono nella lingua originale**, con metadato lingua. La selezione della lingua del documento non fa traduzione automatica di prescrizioni tecniche: mostrare avviso chiaro se manca una versione linguistica.
- **Import/export:** (A) PDF/stampa locale per persone; (B) CSV riepilogativo senza contenuto privato per default; (C) JSON versionato con ID, provenienza, versioni e sezioni, più manifest e allegati in percorsi relativi per round-trip. Import solo da file scelti dall'utente, con anteprima, risoluzione esplicita di artwork UUID mancanti, conflitti di versione, deduplica e rollback integrale; vietare percorsi assoluti/parent traversal e contenuti eseguibili. Non assumere che i CSV esistenti siano reimportabili; un PDF non è il formato canonico di recupero.
- **Backup/ripristino:** includere nuove tabelle, versioni e allegati nella copia verificata su **cartella liberamente scelta**, anche rimovibile/NAS, senza rompere backup preesistenti. I PDF esportati **fuori** dall'archivio non entrano automaticamente nella copia.
- **Privacy:** le istruzioni possono includere nome del restauratore, foto di spazi privati, planimetrie, dettagli antifurto, contatti, eventuali codici/accessi tecnici. Dividere **istruzioni interne** e **materiale condivisibile** con preview deny-by-default: non esportare note di sicurezza, contatti privati, ubicazioni, credenziali, nomi di file locali o EXIF involontariamente. Non incoraggiare a conservare password nei campi liberi. I backup attuali sono in chiaro, da proteggere sul supporto scelto.

## 7. Complessità, dipendenze, rischi, alternative

- **Complessità:** bassa-media per guida testuale con sezioni e stampa; media per foto non distruttive, versionamento, uso per mostre e accessibilità; medio-alta per import/export completo e riconciliazione allegati. Non introdurre visualizzatori CAD, monitoraggio ambientale, certificazioni di sicurezza o diagnosi IA come prerequisiti.
- **Dipendenze possibili:** UUID opere e media locali già presenti, stampa locale, i18n e backup. Riutilizzo di TECH-016 (allegati) o UX-022 (componenti) **solo se approvati**. Il nucleo testuale non deve dipendere da queste idee.
- **Rischi:** una guida obsoleta spacciata per attuale; falsa autorevolezza di istruzioni non verificate; omissioni in fasi di montaggio; divulgazione di informazioni sensibili; allegati ingestibili in archivi grandi; differenze di traduzione su terminologia tecnica; responsabilità professionale e sicurezza elettrica/strutturale. Mitigazioni: fonte, data e versione visibili; avvertenza di validazione; nessun auto-compilato; visibilità selettiva; anteprima; conferme prima di sovrascrivere una bozza; limiti media e test desktop.
- **Alternativa minima:** una sola **nota tecnica lunga** per opera, stampabile, con data/fonte e zero allegati/versioni. È meno complessa e potrebbe bastare alla maggioranza; verificare prima l'adozione.
- **Alternativa fuori dal prodotto:** documenti PDF mantenuti dall'utente in cartelle scelte e soltanto collegati alla scheda; nessun lock-in, ma minore ricercabilità. Da confrontare con l'eventuale TECH-016.

## 8. Criteri di accettazione da discutere

1. Un'opera senza guida indica «non documentato»; un'opera con guida può avere istruzioni parziali e fonti incerte senza campi inventati.
2. È possibile creare una guida con due passaggi e stamparla senza rete, senza contatti, senza impostazioni iniziali obbligatorie.
3. Una revisione del 2028 non riscrive la versione del 2027; citando la guida in un progetto futuro si conserva quale versione sia stata effettivamente consultata, non uno stato di montaggio inventato.
4. Nessuna modifica delle guide cambia componenti, ubicazione, status di vendita, spese, certificati o incassi. Le vecchie opere e i loro backup restano consultabili.
5. L'export per un allestitore esclude per default note interne, ubicazioni private, credenziali, prezzi e dettagli sensibili; l'anteprima elenca i campi inclusi, con opt-in esplicito per eccezioni appropriate.
6. Round-trip JSON di prova ricostruisce UUID, successione delle versioni, fonti, ordine dei passaggi, note e media senza path assoluti né duplicati; l'import fallito non modifica l'archivio.
7. Backup verificato e ripristino in cartella scelta conservano guida e media; assenza di un file è segnalata senza generare contenuti fittizi.
8. Tutti i flussi sono utilizzabili da tastiera e screen reader; etichette localizzate e contenuti non tradotti automaticamente; verifica su Windows/macOS/Linux con layout compatti e stampa A4.
9. Eventuali indicazioni tecniche pericolose sono attribuite alla fonte dichiarata e **non presentate dal software come prescrizioni validate**.

## 9. Fasi ipotetiche (nessuna autorizzazione)

- **A — Verifica bisogno:** intervistare 3–5 operatori con opere modulari, scegliere 2 casi veri, valutare se basta una nota con data/fonte e stampa.
- **B — Eventuale MVP:** testo strutturato opzionale per opera, passaggi ordinati, fonte/data, anteprima e stampa locale, nessuna dipendenza tecnica nuova obbligatoria.
- **C — Se utile:** media e versioni storiche, allegati e integrazione leggera con componenti/progetti, export/import versionato completo e collaudo.
- **D — Solo domanda provata:** modelli avanzati per categorie di opere e requisiti ambientali tipizzati, senza certificazioni automatiche.

## 10. Requisiti certi, ipotesi e decisioni del proprietario

**Certi dal progetto:** Tauri/Rust/SQLite locali; gratuità/open source; nessun account o abbonamento; backup verificato nella cartella scelta; esportazione dati; i18n OS con fallback inglese; certificati immutabili e registro vendite indipendente.

**Ipotesi:** il personale di gallerie e studi necessita un documento riutilizzabile più spesso di una nota libera; serve confrontare revisioni successive; le foto di montaggio aggiungono utilità sufficiente al costo dei media.

**Domande per revisione personale (non richiedono risposta adesso):**
1. Utente prioritario: artista che scrive istruzioni, galleria che le riceve o archivista che le conserva?
2. Bastano testo/foto e PDF, oppure è davvero utile una sequenza di passaggi versionata?
3. Per quali tipologie d'opera occorrono campi dedicati (luce, audio/video, scultura modulare), e quali devono restare testo libero?
4. Quanto devono essere separati i dati interni da quelli esportabili? Servono modelli distinti per allestitore, trasportatore e collezionista?
5. È necessario conservare una copia esatta della guida consegnata a una specifica mostra o basta il riferimento alla versione storica?

**Stato finale invariato: Proposta — da valutare. Nessuna approvazione, incarico o sviluppo.**
