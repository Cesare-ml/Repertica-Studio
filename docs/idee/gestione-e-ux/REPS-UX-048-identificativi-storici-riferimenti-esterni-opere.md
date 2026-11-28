# REPS-UX-048 — Identificativi storici e riferimenti esterni delle opere, senza sostituire il codice Repertica

- **ID stabile:** REPS-UX-048
- **Data della ricognizione:** 2026-11-22
- **Stato:** Proposta — da valutare
- **Filone:** Gestione e UX
- **Priorità ipotetica:** P2 per archivi, artisti con cataloghi legacy e gallerie; P3 o nessuno sviluppo per piccoli archivi in cui bastano le note. Nessuna priorità approvata.
- **Perimetro:** esclusivamente ideativo/documentale; nessuna implementazione, assegnazione, migrazione o modifica ai documenti storici autorizzata.

## 1. Problema, destinatari ed evidenze verificabili

**Problema professionale da convalidare.** La stessa opera può comparire con numeri diversi in un vecchio inventario cartaceo, in un database precedente, nell'elenco prestiti di una galleria o nell'archivio di un curatore. In una telefonata viene citato «INV-94/017», mentre Repertica conosce oggi la stessa scheda come «ART-0238». La ricerca per il codice attuale non trova il riferimento precedente: copiare tutti questi numeri nella descrizione rende difficile distinguerne origine, precisione e conflitti. Questo è particolarmente rilevante per archivi d'artista con decenni di documenti e gallerie che riconciliano inventari esterni.

**Riscontro limitato al `main` esaminato il 22/11/2026:**
- [README](../../../README.md) e [MVP](../../MVP.md) documentano un codice archivio assegnato automaticamente (con politica di prefisso e progressivo) e un UUID locale stabile per opera.
- [`models.rs`](../../../src-tauri/src/models.rs) espone per `Artwork` un solo `code`, oltre a UUID, titolo, descrizione e altri campi; [`archive_code.rs`](../../../src-tauri/src/archive_code.rs) gestisce numeri progressivi e unicità del **codice corrente**.
- [`archive-filters.ts`](../../../src/app/archive-filters.ts) ricerca titolo, codice corrente, tecnica, supporto, descrizione, anno e altri campi, ma non dispone nel modello di una raccolta di **codici storici qualificati per fonte**.
- [Portabilità CSV](../../DATA-PORTABILITY.md) esporta il codice corrente nel registro `artworks`, senza un registro separato di identificativi associati. Non emerge quindi un flusso dedicato per cercare «il numero usato dalla galleria X» e risalire, dopo verifica, alla scheda locale.

Si tratta di **opportunità derivata da modello e documentazione**, non di bug riprodotto, richiesta cliente verificata o garanzia di valore universale. Non è stata svolta una prova d'uso installata.

**Destinatari principali:** archivi e fondazioni d'artista, gallerie che gestiscono opere di provenienze amministrative differenti, artisti che migrano registri di studio precedenti. Per chi ha un unico inventario recente la soluzione può non servire.

## 2. Scenari d'uso concreti

1. **Archivio storico:** un archivista apre la scheda `ART-0238` e registra «INV-94/017», associandolo alla fonte «Registro di studio 1994, pag. 12», con stato «trascritto da fonte, identità ancora da verificare». Alla ricerca del vecchio numero compare la scheda **come possibile corrispondenza documentata**, senza attribuire automaticamente autenticità all'opera.
2. **Consegna a una galleria:** la galleria chiama una scultura «G-128» nel suo elenco. Il titolare annota il codice come riferimento **della galleria**, senza modificare il codice interno, creare una movimentazione, una vendita o un trasferimento di proprietà.
3. **Collisione reale:** due gallerie usano entrambe «17» per opere diverse. Una ricerca mostra contesto e fonte; non impone un'unicità globale falsamente restrittiva. Due fonti *della stessa galleria* possono comunque essere in disaccordo: l'app non decide quale sia giusta.
4. **Errore di trascrizione:** un riferimento «94/017» è stato attribuito all'opera sbagliata. Una rettifica conserva quale associazione era stata registrata e la sua motivazione, senza cambiare UUID, codici correnti o certificati emessi.
5. **Cataloghi che usano titoli diversi:** la presenza di un titolo alternativo in una pubblicazione resta di competenza del riferimento bibliografico [UX-019](REPS-UX-019-storico-esposizioni-bibliografia-documentata.md) o della presentazione multilingua [UX-041](REPS-UX-041-titoli-descrizioni-opere-multilingua.md), **non** viene memorizzata surrettiziamente come codice.

## 3. Flusso UX ipotetico: minimo e reversibile

**Scheda Opera → «Codici e riferimenti» (pannello facoltativo).** Il codice corrente Repertica resta evidenziato e non modificato da questo flusso. Sotto, una lista di riferimenti dichiarati: `codice esatto`, `ambito / soggetto che lo ha usato`, fonte, stato («dichiarato», «fonte consultata», «contestato», «rettificato»), data o periodo se noto. Se vuota, «Nessun riferimento esterno registrato», **non** «nessun riferimento esterno esistente».

**Aggiungi riferimento.** Campi minimi: codice **come scritto nella fonte** e descrizione dell'ambito (es. inventario studio 1994, galleria X). Opzionali: tipo (inventario, catalogo interno terzi, archivio precedente, altro), data con precisione anno/mese/giorno, pagina o localizzatore nella fonte, note, verifica dichiarata. Una «fonte consultata» significa soltanto che qualcuno l'ha letta: non certifica che la descrizione si riferisca davvero all'opera. Mostrare un piccolo riepilogo prima di salvare. Non tentare lookup online.

**Ricerca:** il campo di ricerca opere può includere gli identificativi associati, ma il risultato deve essere etichettato «Trovato in: riferimento storico [ambito]», distinguendolo da «Codice archivio corrente». In caso di più schede candidate, presentarle **tutte** con titolo, fotografia locale se disponibile, codice attuale e ambito; mai aprire o unire automaticamente un record. Per una ricerca con sole parole, non mescolare senza etichette codici storici e titoli. Prevedere ricerca esatta per identificativi punteggiati (barra, punto, trattino) e ricerca testuale normale.

**Rettifica e rimozione visiva:** se il riferimento era errato, possibilità di marcarlo come «attribuzione contestata/rettificata», specificando nuova evidenza. La vista ordinaria può nasconderlo, ma lo storico resta consultabile; se si permette una cancellazione esplicita per privacy/errore di dati personali, la relativa politica va progettata separatamente e non deve toccare l'opera o i certificati. Nessuna fusione automatica di opere omonime o con codici uguali.

**Accessibilità e qualità:** input e liste raggiungibili con tastiera; etichette per stato non basate soltanto su colore; risultati ambigui annunciati da screen reader; zoom 200%, testi lunghi, stampa in bianco/nero, contrasto e focus prevedibile. Anteprima di ogni copia condivisibile con distinzione fra codice interno e riferimento di terzi.

## 4. Dati necessari — ipotesi descrittive, NON schema approvato

Una possibile entità **`artwork_external_reference`** con:
- `id` UUID stabile; `artwork_id` FK a un'opera esistente; `literal_identifier` testo originale non vuoto, Unicode, senza riscrittura o riformattazione implicita; `source_scope` nome/etichetta dell'inventario o istituzione che usa il codice (testo esplicito), `source_kind` codice stabile opzionale.
- `source_citation` testo facoltativo (documento, pagina, inventario), `source_date` e `date_precision` facoltativi, `verification_state` dichiarativo (mai «autentico/verificato dal software»), `notes`, `created_at`, `updated_at`; eventuale `superseded_by` o evento di rettifica per associazioni sbagliate.
- La **chiave dell'opera rimane soltanto `artworks.id`**, mentre `artworks.code` rimane il codice corrente governato dalla politica di assegnazione. Un numero esterno è un *riferimento attribuito*, **non** un secondo UUID, una prova di possesso o un nuovo numero di inventario Repertica.
- Un valore `reference_normalized` per ricerca sarebbe derivato in memoria o in un indice locale, **non** sostituisce il testo di fonte. «G-001» e «G001» possono essere codici distinti: non eliminarne arbitrariamente punteggiatura o zeri. Lo stesso identificativo in due ambiti **non è un duplicato certo**.
- L'unicità **non va imposta globalmente**: anche nella stessa fonte possono esistere trascrizioni errate o casi controversi. Mostrare collisioni per `source_scope` e `literal_identifier` come avvisi da esaminare; eventuali vincoli più forti richiedono validazione reale. Riferimenti a documenti presenti solo come stringhe nella fase minima, senza creare un secondo archivio media.
- Nessuna scrittura su `sales`, `collectors`, `certificates`, `expenses`, `archive_code_sequences` o stato commerciale. Consultazione e salvataggio sarebbero azioni locali, non endpoint remoti o mutazioni MCP.

## 5. Confini anti-doppioni e coordinamento

- [UX-020 — Etichette inventariali](REPS-UX-020-etichette-inventariali-identificativi-scansionabili.md) identifica **un record nel proprio archivio tramite il codice corrente/UUID**, per stampa e scansione. Questa proposta collega **vecchie o altrui sigle con fonte**, senza emettere etichette, scanner, QR o prova d'identità.
- [UX-019 — Esposizioni e bibliografia](REPS-UX-019-storico-esposizioni-bibliografia-documentata.md) conserva **citazioni a mostre/pubblicazioni e titoli attestati**; qui solo numeri di riferimento d'inventario/catalogo usati per rintracciare un'opera. Evitare di duplicare la bibliografia estesa: eventualmente usare il suo riferimento autorevole **solo se entrambe le idee saranno approvate**.
- [UX-018 — Catalogazione seriale](REPS-UX-018-catalogazione-seriale-schede-bozza.md) velocizza l'inserimento, non ricostruisce riferimenti passati. [UX-005 — Edizioni](REPS-UX-005-edizioni-multipli-serie-numerate.md) identifica **esemplari fisici**: un codice legacy non genera una nuova edizione o un esemplare.
- [UX-001 — Provenienza e movimentazioni](REPS-UX-001-provenienza-movimentazioni.md) ha contenuto storico su proprietà/custodia e luoghi; un vecchio numero stampato da una galleria **non dimostra** che l'opera fosse di sua proprietà o fisicamente lì.
- Nel filone tecnico [TECH-019 — Import CSV esterni](https://github.com/Cesare-ml/Repertica-Studio/blob/docs/idee-strumenti-architettura-20261121/docs/idee/strumenti-e-architettura/REPS-TECH-019-importazione-guidata-cataloghi-tabellari.md) studia mappatura e convalida *del formato*; [TECH-029 — Scambio fra archivi](https://github.com/Cesare-ml/Repertica-Studio/blob/docs/idee-strumenti-architettura-20261121/docs/idee/strumenti-e-architettura/REPS-TECH-029-scambio-selettivo-schede-opere-archivi-indipendenti.md) valuta il trasporto di schede senza identità condivisa; [TECH-001](https://github.com/Cesare-ml/Repertica-Studio/blob/docs/idee-strumenti-architettura-20261121/docs/idee/strumenti-e-architettura/REPS-TECH-001-pacchetto-dati-aperto.md) documenta portabilità. Qui si studia **l'esperienza curatoriale e il significato del riferimento**, senza proporre un nuovo formato standard o toccare quei file.
- [TECH-010 — Revisioni dei metadati](https://github.com/Cesare-ml/Repertica-Studio/blob/docs/idee-strumenti-architettura-20261121/docs/idee/strumenti-e-architettura/REPS-TECH-010-storico-revisioni-metadati.md) potrebbe servire per rettifiche generali, ma un codice **storico deliberatamente conservato** è diverso dal log generico delle modifiche del campo `code`.

## 6. Offline, impostazioni, lingue, sicurezza e privacy

**Solo locale:** SQLite e backup completo già previsti, senza account, abbonamento, API, registro pubblico centrale, sincronizzazione, IA o cloud obbligatorio. I dati di riferimento restano consultabili anche senza rete. Nessuna modifica automatica di codici, vendite, prezzo, coefficiente artista, quote di guadagno, certificati o rapporti di provenienza.

**Impostazioni facoltative:** pannello mostra/nascondi, ambiti suggeriti per archivi frequenti e visualizzazione di codici contestati nei risultati; nessun insieme di prefissi obbligatorio. Se già esistesse in futuro una rubrica professionale UX-014, l'ambito potrebbe riferirsi alla controparte, ma **mai** richiedere l'approvazione o la creazione di tale modulo.

**Multilingua:** lingua di sistema con fallback inglese e override IT/EN/ES/FR/DE conformemente a [ECONOMY-I18N](../../ECONOMY-I18N.md). Tradurre etichette e avvisi, non valori di codice, nomi di istituzioni, fonti o note. Codici di stato invarienti nel database; ISO 8601 e precisione delle date nell'export, presentazione locale in UI. Cercare anche sequenze Unicode senza perdere il valore originale.

**Privacy e sicurezza:** codici di inventario, nomi di gallerie e collocazioni delle fonti possono rivelare beni preziosi, rapporti commerciali o localizzazioni; non pubblicarli in brochure/certificati/QR, non esporli via MCP e non renderli esportabili per terzi senza anteprima e scelta esplicita. Validare lunghezza del testo e trattarlo sempre come **dato**, non come URL/comando/percorso eseguibile. I backup completi correnti sono in chiaro: proteggere cartelle, USB e NAS. Le esportazioni esterne **non entrano automaticamente** in un backup dell'archivio attivo.

## 7. Import/export, compatibilità, alternative e rischi

**Portabilità integrale proposta:** CSV UTF-8 distinto per i riferimenti, colonne `reference_id`, `artwork_id`, `literal_identifier`, `source_scope`, tipologia, data/precisione, stato e fonte/nota; importi e contatti non pertinenti esclusi. Il registro `artworks` esistente rimane invariato e continua a esportare il codice corrente. Per round-trip completo, possibile JSON versionato con UUID e collegamenti ad opere; non considerare una singola esportazione CSV un backup ripristinabile. Eventuale pacchetto completo include queste entità nel backup locale verificato SQLite+media su **cartella scelta liberamente dall'utente**, senza ricostruirle da nomi o titoli.

**Import opzionale successivo:** selezione manuale CSV esterno, anteprima di righe e ambiti, mappatura delle colonne, scelta esplicita dell'opera di destinazione anche in caso di codice legacy omonimo, confronto con riferimenti già presenti, conflitti, annullamento/rollback atomico. Un file esterno non dimostra quale UUID locale corrisponda a un'opera: **mai unire, creare o alterare schede automaticamente** per uguaglianza di titolo/codice. Nessuna elaborazione da Internet.

**Compatibilità:** nuova informazione additiva soltanto se autorizzata; per archivi vecchi l'elenco riferimenti è vuoto (dato sconosciuto), senza reinterpretare descrizioni o codici già assegnati e senza riscrivere snapshot dei certificati. Versioni vecchie possono non aprire uno schema SQLite nuovo: backup verificato prima di eventuali migrazioni e istruzioni di recupero, coerenti con TECH-042. Nessuna mutazione a CSV/registri economici correnti.

**Complessità:** bassa se si parte da un campo note esplicito; media per registro multi-riferimento, ricerca qualificata, conflitti, CSV e migrazione; medio-alta per import controllato e cronologia di rettifica. **Dipendenze:** opera/UUID, codice corrente, ricerca locale, SQLite, localizzazione, backup/export; TECH-019/029 e UX-019 sono integrazioni **facoltative**, non prerequisiti approvati.

**Rischi e mitigazioni:** (a) falsa identificazione con codici duplicati → ambito e anteprima dei candidati; (b) perdita della forma storica → conservare testo letterale e fonte; (c) dubbia affidabilità storica → stati dichiarativi, date incerte e rettifiche; (d) esposizione commerciale involontaria → export minimizzato e preview; (e) rallentamento ricerca su molti riferimenti → misurare con archivi grandi prima di sceglierne l'indicizzazione; (f) sovrapposizione ai moduli bibliografia/import → una sola fonte di verità e nessuna automazione di merge.

**Alternativa minima da sperimentare per prima:** annotare in una sola sezione testuale dell'opera «Codici precedenti e fonte», esportarla con il CSV attuale e usare la ricerca per descrizione. Raccogliere 5–10 esempi anonimi di inventari che usano codici multipli; se l'utente trova sempre il record così, non progettare nuove tabelle o UI.

## 8. Criteri di accettazione ipotetici

1. Un'opera con codice corrente `ART-0238` viene trovata cercando `INV-94/017`; nel risultato sono mostrati codice corrente **e** fonte del riferimento, senza cambiare UUID o codice.
2. Due opere possono condividere la stringa `17` in ambiti differenti; la ricerca mostra entrambe, senza fusione, errore occulto o apertura automatica. Collisioni nello stesso ambito vengono segnalate come **ambigue**, non corrette autonomamente.
3. Modificare il codice attuale secondo le regole esistenti non cancella un riferimento storico; aggiungerne uno non consuma il progressivo e non cambia un certificato immutabile.
4. Un riferimento contestato resta consultabile con stato e fonte; non è presentato come «verificato» solo perché presente in un foglio importato.
5. Export/round-trip di prova conservano Unicode, trattini, zeri iniziali, UUID, ambiti e stati senza generare opere o transazioni finanziarie.
6. Archivio preesistente senza riferimenti continua a funzionare; backup/ripristino locale nella cartella scelta conserva i nuovi legami, con assenza distinta da codice vuoto.
7. Operazioni raggiungibili da tastiera, esiti accessibili a screen reader, etichette in cinque lingue (fallback inglese), funzionamento completamente offline; nessuna rete o trasmissione MCP.
8. Una condivisione per terzi **non include** riferimenti storici, nomi di galleria e note per default; serve selezione esplicita e anteprima dei dati realmente inclusi.

## 9. Fasi possibili e decisioni aperte al proprietario

- **Fase A — Validazione senza nuovo codice:** 5–10 documenti legacy anonimi, test dell'alternativa «nota + ricerca», misurazione dei casi non risolvibili.
- **Fase B — Se giustificato:** uno o più identificativi per opera con ambito e fonte, ricerca qualificata, esportazione locale; nessun import massivo, cambio identità o rete.
- **Fase C — Solo se necessario:** collisioni e rettifiche con cronologia, mappatura CSV/import assistito, filtri trasversali per fonte, coordinamento con eventuale biblioteca bibliografica o interscambio fra archivi.

**Certo dal progetto:** codice corrente univoco, UUID, ricerca locale, CSV opere, backup verificato a destinazione scelta e piattaforma offline-first multilingua, senza account. **Ipotesi:** gli utenti utilizzano abitualmente più numerazioni per le stesse opere; serve cercarle dentro Repertica e non soltanto tenerle nelle note. **Da decidere dal proprietario:** si vuole anche ricercare un codice *soppresso* che in passato era il codice corrente? Quanta struttura per la fonte e la sua attendibilità? Consentire più ambiti omonimi? Meglio una singola nota facoltativa nel breve periodo? Chi ha bisogno di importazione CSV multipla e con quale frequenza? Nessuna risposta è presunta.

**Decisione attuale:** nessuna approvazione, nessuna assegnazione e nessun avvio di sviluppo.
