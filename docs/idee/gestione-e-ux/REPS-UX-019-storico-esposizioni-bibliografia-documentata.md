# REPS-UX-019 — Storico espositivo e bibliografia documentata delle opere

- **ID stabile:** REPS-UX-019
- **Data:** 2026-11-21
- **Stato:** Proposta — da valutare
- **Filone:** Gestione e UX
- **Priorità ipotetica:** P2 (elevabile soltanto se artisti, archivi e gallerie documentano consultazioni ricorrenti)
- **Natura:** sola ricerca e specifica concettuale; nessuna approvazione, assegnazione o implementazione

## 1. Problema, utenti e riscontro concreto in `main`

Artisti, gallerie, archivi d'artista e curatori devono spesso rispondere a domande precise: *in quali mostre è stata effettivamente presentata quest'opera? In quale catalogo, articolo o pubblicazione compare? Da dove proviene questa informazione?* Oggi tali dati finiscono in descrizioni libere, cartelle separate o fogli di calcolo, perdendo struttura, riutilizzo e livello di affidabilità della fonte.

**Verificato il 21 novembre 2026:** [`README.md`](../../../README.md), [`docs/MVP.md`](../../MVP.md), [`src-tauri/src/models.rs`](../../../src-tauri/src/models.rs), [`src-tauri/src/db.rs`](../../../src-tauri/src/db.rs), [`src/app/models.ts`](../../../src/app/models.ts) e [`docs/DATA-PORTABILITY.md`](../../DATA-PORTABILITY.md). Il modello `Artwork` contiene testo descrittivo, anno, tecnica, dimensioni, ubicazione e stato; lo schema SQLite corrente arriva a v6 e non contiene entità strutturate per partecipazioni espositive **storiche accertate o dichiarate**, né riferimenti bibliografici con pagine/tavole e collegamenti molti-a-molti alle opere. La ricerca locale e i **sette** CSV integrali non equivalgono a un repertorio delle fonti. È una **lacuna funzionale rilevata**, non un malfunzionamento del software.

**Destinatari prioritari:** archivio d'artista che prepara un catalogo ragionato; artista con una carriera di esposizioni; galleria che produce schede per curatori; fondazione che riconcilia citazioni discordanti. **Destinatari secondari:** piccolo studio che vuole semplicemente ricordare mostre pregresse. Il beneficio va verificato: per archivi di poche opere potrebbe bastare una nota testuale.

## 2. Confini contro duplicazioni

| Ambito già progettato | Perché questa scheda non lo ripete |
| --- | --- |
| [UX-009 — Progetti espositivi](REPS-UX-009-progetti-espositivi-checklist.md) | UX-009 **pianifica** selezioni, attività, date e scadenze. Qui si conservano **riferimenti storici e testimonianze** della partecipazione effettiva o dichiarata, anche per eventi passati di decenni. La chiusura di un progetto non prova la presenza dell'opera: nessun trasferimento automatico dal progetto allo storico. |
| [UX-001 — Provenienza e movimentazioni](REPS-UX-001-provenienza-movimentazioni.md) | Una spedizione/uscita dallo studio non prova l'esposizione pubblica. I movimenti indicano **custodia e luoghi fisici**, non citazioni editoriali. |
| [UX-007 — Dossier di presentazione](REPS-UX-007-dossier-presentazione-opere.md) | Il dossier impagina **dati selezionati per condivisione**; questa proposta registra le fonti strutturate e il loro grado di verifica. Un dossier potrà eventualmente consumarle, senza essere prerequisito. |
| [UX-010 — Diritti/crediti](REPS-UX-010-diritti-riproduzione-crediti-autorizzazioni.md) | Avere una pubblicazione citata o una sua scansione locale **non concede** il diritto di riprodurne immagini o testi. |
| [UX-011 — Cicli creativi](REPS-UX-011-cicli-creativi-relazioni-opere.md) | Relazioni fra opere artistiche, non riferimenti bibliografici o documentazione di esposizioni storiche. |
| [UX-016 — Valutazioni](REPS-UX-016-valutazioni-coperture-assicurative-scadenze.md) | Una pubblicazione non determina autenticità, stima di mercato o copertura assicurativa. |
| [TECH-001 — Pacchetto dati aperto](https://github.com/Cesare-ml/Repertica-Studio/blob/docs/idee-strumenti-architettura-20261121/docs/idee/strumenti-e-architettura/REPS-TECH-001-pacchetto-dati-aperto.md) | Definisce la **portabilità complessiva** dei dati; qui si definisce il significato del dominio curatoriale e l'esperienza dell'utente. Nessun file tecnico viene modificato. |

**Esclusioni esplicite:** catalogo ragionato certificato, autenticazione automatica, ricerca bibliografica online, scraping, acquisizione da banche dati o biblioteche, DOI/ISBN risolti via rete, OCR, riconoscimento delle opere su immagini, generazione automatica di titolarità o diritti, pubblicazione di un sito, archiviazione di libri interi, emissione di certificati di autenticità o di documenti fiscali.

## 3. Scenari verificabili

1. **Artista e retrospettiva:** registra una mostra del 2014 presso una galleria, collegando cinque opere tramite codici e una fotografia di installazione. Per due opere esiste un catalogo con numero di pagina; per le altre la partecipazione è *dichiarata dall'artista*, non verificata documentalmente. Lo storico deve distinguere i due casi.
2. **Archivio e rivista:** una stessa pubblicazione cita tre opere, con pagine o figure differenti. Il bibliografico viene registrato **una sola volta** e ogni opera porta la propria citazione puntuale (pagina/tavola, tipo di menzione, nota e fonte).
3. **Informazioni contrastanti:** una didascalia storica attribuisce erroneamente un titolo. L'operatore conserva il titolo corrente della scheda opera, registra la variante **così come attestata** nella fonte e annota la discordanza senza correggere silenziosamente né la pubblicazione né il certificato.
4. **Catalogo mai verificato:** un cliente riferisce che l'opera è riprodotta in un libro non ancora consultato. L'utente inserisce un riferimento provvisorio con etichetta **«riferito, non verificato»**; nessuna evidenza o validità viene inventata.
5. **Preparazione di un prestito o dossier:** il curatore seleziona «Mostre e bibliografia» per una singola opera ed esporta una cronologia sintetica con fonti e incertezze, **senza** recapiti, margini, ubicazioni private né interi file dei documenti.

## 4. Flussi UX candidati

**A — Scheda opera → Mostre e bibliografia.** Due sezioni distinte: «Esposizioni storiche» e «Pubblicazioni e fonti». Ogni voce mostra titolo, data/periodo (con livello di precisione), riferimento puntuale e **stato della prova**. Evidenziare «dato non documentato» in testo, non con solo colore. Se vuoto, mostrare «Nessuna fonte registrata», mai «Opera mai esposta» o «Mai pubblicata».

**B — Collega mostra pregressa.** Ricerca in un elenco di eventi già inseriti o «Nuova esposizione storica». Campi minimali: titolo della mostra, luogo/sede testuali, data/periodo dichiarato (anno, mese o giorno), eventuale organizzatore, fonte e note. Poi selezione delle opere e indicazione **per ogni collegamento** di testimonianza (catalogo, fotografia, elenco prestiti, dichiarazione, altra), riferimento e verifica manuale. La presenza in un progetto UX-009 non crea automaticamente una partecipazione storica: serve conferma esplicita.

**C — Aggiungi riferimento bibliografico.** Maschera compatta con tipo (catalogo, monografia, periodico, articolo, pagina web, altro), titolo, autore/curatore facoltativo, editore/periodico, anno o data, identificativo ISBN/ISSN/DOI **facoltativo e non verificato automaticamente**, URL come testo, data ultima consultazione se pertinente. Riepilogo anti-doppione su titolo/autore/anno senza blocchi basati su uguaglianze incerte. «Collega opera» conserva pagina, tavola/figura, citazione breve **inserita dall'utente**, tipo di presenza (menzione, riproduzione, scheda critica), eventuale titolo storico, nota di verifica.

**D — Consultazione e correzione.** Vista «Fonti e mostre» con ricerca locale per autore, titolo, sede, anno e stato; accesso dalle opere collegate. Correzioni materiali devono conservare **almeno origine della rettifica, data e nota** per dati già citati in documenti pubblicati. Nessuna cancellazione a cascata di un'opera o di un certificato per eliminare un collegamento errato. Una fonte può essere segnata «non più attendibile» senza cancellare la storia dell'errore.

**E — Esporta/ stampa cronologia.** Selezione di opere, ordinamento delle mostre e citazioni, anteprima del contenuto e scelta «uso interno» o «per terzi». L'elenco non presume esaustività («Fonti registrate al [data]»), mostra l'incertezza e distingue *esposta* da *menzionata in catalogo*. Una citazione bibliografica non autorizza a esportare la scansione o la fotografia sottostante.

**Accessibilità obbligatoria in eventuale sperimentazione:** tabella semantica con intestazioni e lista leggibili da screen reader; controlli interamente da tastiera; focus dopo aggiunta/errore; descrizioni testuali di prova e conflitto; ricerca con etichette accessibili; date parziali non forzate a `01/01`; zoom 200%, tastiere internazionali e testi lunghi su Windows/macOS/Linux.

## 5. Modello dati concettuale — ipotesi, nessuna migrazione autorizzata

- **`historic_exhibition`**: UUID, titolo, sede/testo libero, città/paese testuali opzionali, organizzatore facoltativo, intervallo dichiarato e precisione di ogni data, note, fonte generale e timestamp. Non rappresenta un progetto pianificato né un movimento fisico.
- **`exhibition_artwork_evidence`**: UUID, `exhibition_id`, `artwork_id`, qualificatore partecipazione `documented` / `reported` / `uncertain` / `disputed` (codici **non traducibili nel database**), fonte e riferimento puntuale, titolo storico facoltativo, note, data della verifica manuale facoltativa. La data di verifica indica che una persona ha consultato una fonte, **non** che Repertica abbia accertato il fatto.
- **`bibliographic_reference`**: UUID, tipologia, autori/curatori in stringhe inizialmente (da normalizzare solo con motivazione), titolo, anno/data con precisione, editore/periodico, volume/fascicolo, identificativi esterni opzionali, URL come stringa **non aperta/sincronizzata automaticamente**, luogo di pubblicazione se noto, lingua originale facoltativa, note e timestamp.
- **`reference_artwork_citation`**: UUID, `reference_id`, `artwork_id`, pagina/tavola/figura come stringhe (es. `12–13`, `tav. VIII`), tipo menzione, eventuale titolo riportato, affidabilità dichiarata, fonte della verifica, nota. Non dedurre unicità della citazione da numero pagina o titolo; più menzioni della stessa opera sono possibili.
- **`source_attachment`** *solo se giustificato*: UUID, riferimento alla fonte/partecipazione, percorso relativo sicuro in media gestiti, tipo file, descrizione, provenienza dichiarata, eventuale restrizione di condivisione. Preferire come MVP **riferimenti testuali** a documenti esterni e un piccolo numero di immagini/documenti esplicitamente importati, non replicare automaticamente intere biblioteche.
- **`source_correction`** *seconda fase*: UUID, record collegato, natura della rettifica, valore precedente/nuovo se compatibile con privacy, motivazione, data registrazione. Possibile coordinamento futuro con TECH-010 senza presumerne l'implementazione.

**Invarianti:** UUID stabili e FK; opere preesistenti non alterate; mostra o pubblicazione **non** cambia disponibilità, vendita, ricavi, custodia, prezzo, certificati immutabili o diritti; evento non equivale ad autenticità dell'opera; assenza di fonte = informazione non inserita, non prova negativa. Evitare duplicati con suggerimenti non distruttivi. Per testi bibliografici esistenti mantenere grafia e accenti; la UI può offrire una versione di consultazione normalizzata, mai sovrascrivere l'originale.

## 6. Impostazioni, offline, privacy e lingue

- **Offline:** SQLite e allegati gestiti interamente in locale, nessun account, abbonamento, cloud, telemetria, ricerca in rete, notifica esterna né scraping. ISBN/DOI/URL sono **identificatori o testo**, non inviti automatici a servizi esterni. Si possono consultare citazioni senza rete e senza documenti allegati.
- **Impostazioni facoltative:** stile di visualizzazione della citazione (breve o estesa, **non** generatore universale APA/Chicago), ordine predefinito per anno/titolo, esportazione di stati incerti inclusi/esclusi con scelta esplicita, uso interno/esterno. Nessuna fonte obbligatoria per creare un'opera, nessun catalogo pubblico preconfigurato.
- **Multilingua:** lingua di sistema con override IT/EN/ES/FR/DE e fallback inglese secondo [ECONOMY-I18N](../../ECONOMY-I18N.md). Tradurre etichette, stati di prova, errori e intestazioni export; **non tradurre automaticamente** titoli originali, nomi propri o citazioni testuali. Date visualizzate secondo locale, precisione storica conservata in dati strutturati; nessuna «traduzione» dei codici persistenti.
- **Privacy:** alcune fonti possono rivelare proprietari privati, prestatori, indirizzi di deposito, fotografie d'installazione con persone o corrispondenze non pubbliche. Per terzi, campo consentito minimo e anteprima obbligatoria; note interne, recapiti, allegati, URL locali, indicazioni private sulla custodia, costi, margini e dati dei collezionisti esclusi per default. Nessuna attribuzione del copyright o licenza presunta. I backup attuali sono in chiaro: avviso quando si importano documenti riservati.
- **Integrità e allegati:** file e metadati sotto l'archivio gestito devono entrare in **backup e ripristino verificati** in cartella scelta liberamente (USB/NAS montato inclusi). Documenti collegati **solo con percorso esterno** non sono protetti automaticamente; segnalarlo. File PDF/immagini potenzialmente malevoli non devono poter eseguire codice né usare percorsi di attraversamento.

## 7. Import/export e compatibilità

**Esportazione consultiva:** CSV UTF-8 (compatibile con convenzioni [DATA-PORTABILITY](../../DATA-PORTABILITY.md)) separati per `historic_exhibitions`, `exhibition_links`, `bibliographic_references`, `reference_artwork_citations`, con UUID, date ISO più precisione, codici stabili e riferimenti senza path assoluti. Conservare i **sette CSV esistenti** nel loro significato originario; aggiungere registri senza sostituirli. Una distinta stampata non va chiamata certificato né prova ufficiale della partecipazione.

**Scambio integrale futuro:** JSON versionato con entità, relazioni, codici, livelli di prova e manifest di eventuali allegati **solo se** approvato; coordinamento concettuale con TECH-001, senza modificarne file o inventare interoperabilità già disponibile. Un export bibliografico RIS/BibTeX è **alternativa successiva**, solo per campi mappabili e con avvertenze su perdita di collegamenti opera/fonte; non è indispensabile alla prima fase.

**Import da file scelto dall'utente:** anteprima e mappatura di colonne; distinzione fra citazione nuova e fonte già presente; conflitti su UUID, titolo e anno **presentati**, non risolti automaticamente; collegamenti consentiti soltanto a opere esistenti o mappate esplicitamente; transazione atomica; round-trip idempotente con allegati mancanti segnalati. Nessuna importazione web a sorpresa, nessuna cancellazione silenziosa, nessuna riscrittura di certificati.

**Compatibilità:** migrazione additiva se in futuro approvata; archivi vecchi mantengono tutte le opere e mostrano «Nessuna fonte registrata» senza creare testimonianze fittizie. Backup di schema più nuovo non vanno dichiarati apribili con versioni precedenti; test di ripristino/rollback e percorsi Windows/macOS/Linux. Esportazioni contenenti opere non più presenti devono trattare riferimenti orfani con errore, non inventare record.

## 8. Complessità, dipendenze, alternative e rischi

- **Dipendenze certe già presenti:** identificativi UUID delle opere, schema SQLite e migrazioni, ricerca locale, convenzioni i18n, backup/ripristino e CSV, UI Angular/Tauri. Non serve internet e non deve dipendere dall'approvazione di UX-009 o TECH-001.
- **Dipendenze ipotetiche:** gestione allegati documentali generici, bibliografia condivisa fra molte opere, meccanismo di rettifica; vanno giustificati dall'uso prima dell'adozione. Nessuna nuova dipendenza software è richiesta da questo incarico documentale.
- **Complessità stimata:** media per due registri e associazioni; medio-alta per date parziali, deduplicazione assistita, evidenze, filtri; alta per allegati diversi dalle immagini, import interoperabili e rettifiche storiche dettagliate.
- **Rischi:** attribuzioni false presentate come verificate; creare duplicati di mostre/citazioni; pubblicazioni confuse con esposizione fisica; violazione dei diritti di riproduzione; divulgazione di proprietari o sedi; falsa esattezza di date incerte; import da file non attendibili; crescita incontrollata dell'archivio; trasformare Repertica in un sistema bibliotecario complesso.
- **Mitigazioni:** campo fonte e tipo di attestazione, «non verificato» in chiaro, identificatori stabili, ricerca duplicati solo suggerita, anteprima privacy, approccio per fasi, nessun download automatico, limite/controllo allegati e fonti minime anziché record bibliografici enciclopedici.
- **Alternativa minima:** due elenchi di testo strutturato per opera senza tabelle condivise: meno complessi, ma ripetono fonti comuni e rendono difficile la deduplica. **Alternativa esterna:** LibreOffice/CSV o gestori bibliografici autonomi, con costo di implementazione nullo ma senza legami stabili ai UUID delle opere.

## 9. Criteri di accettazione ipotetici e prove

1. Tre opere possono collegarsi alla stessa mostra storica e alla stessa pubblicazione con **riferimenti di pagina diversi**, senza duplicare la fonte e senza cambiare schede, stati, vendite o certificati.
2. Mostra futura pianificata in UX-009, se esistente, **non compare** automaticamente come esposizione effettiva; il collegamento richiede conferma consapevole e fonte.
3. Una fonte solo riferita ma non verificata mostra l'etichetta testuale «non verificata» in scheda, ricerca e stampa; l'assenza di record non appare come affermazione «mai esposta».
4. Intervalli e date imprecisi (solo 1999, oppure circa 2000) non vengono convertiti a giorno fittizio e mantengono precisione nel backup/export/import.
5. Un titolo divergente riportato dalla fonte resta distinto dal titolo corrente; una correzione non riscrive certificati storici né cancella l'evidenza originaria senza traccia.
6. Export esterno non include note, contatti privati, costi, ubicazioni sensibili, allegati o scansioni non selezionati; nessuna chiamata di rete viene invocata dai metadati bibliografici.
7. Export/import ripetuto non crea duplicati, conserva UUID e collegamenti; il backup/ripristino verificato conserva dati e allegati inclusi.
8. Tutti i flussi sono utilizzabili offline con tastiera/screen reader, contrasto e zoom 200%, lingua del sistema o inglese di fallback, su Windows/macOS/Linux.
9. Su archivio con migliaia di opere e centinaia di fonti le ricerche restano utilizzabili senza caricare tutti i file allegati in memoria; la soglia quantitativa/tempi va stabilita dopo benchmark reali.

## 10. Fasi soltanto esplorative

- **Fase 0 — Ricerca utente:** osservare 3–5 repertori reali (artista, galleria, archivio), distinguere esigenze frequenti e casi eccezionali; provare l'alternativa delle note e verificare se il riuso di fonti riduce errori.
- **Fase A — MVP concettuale:** storico mostre e bibliografia come registri manuali con link multipli, fonti dichiarate, date parziali, consultazione dalla scheda opera. Nessun allegato obbligatorio.
- **Fase B — Qualità della fonte e condivisione:** stati espliciti di verifica, correzioni datate, filtri, stampa/CSV minimizzati, controlli privacy e accessibilità.
- **Fase C — Interoperabilità ed evidenze opzionali:** import/versionamento, allegati gestiti con backup integrale, eventuale coordinamento con dossier UX-007 e pacchetto TECH-001; solo dopo approvazione separata e prove su archivi reali.

## 11. Requisiti certi, ipotesi, decisioni riservate al proprietario

**Certi dal prodotto/documentazione corrente:** desktop Tauri gratuito e open source; database SQLite e file locali; nessun account/abbonamento; backup integrale verificato con scelta cartella; certificati immutabili; sette registri CSV; lingua di sistema IT/EN/ES/FR/DE ove presente e fallback inglese; ricerca locale di opere. Il presente documento non estende funzionalità attuali.

**Ipotesi da validare:** esiste una domanda significativa di bibliografia e mostre storiche; le fonti sono condivise spesso fra più opere; gli utenti vogliono attestazioni esplicite e date parziali; una cronologia strutturata apporta più valore delle note libere; P2 è una priorità appropriata.

**Questioni aperte per revisione personale:**
1. Utente iniziale: artista individuale, galleria con molte mostre o archivio storico? La stessa UX sarebbe troppo complessa per i primi?
2. Sono necessari fin dall'inizio due registri (mostre e bibliografia) o conviene iniziare da uno?
3. Per una fonte cartacea serve conservare la scansione, un semplice riferimento o un link a un file esterno (non garantito nei backup)?
4. Come esprimere «fonte consultata da me» vs «fatto storicamente verificato» per non sovrastimare certezza?
5. Quanto supporto per date incerte, varianti di titolo e lingue originali serve davvero?
6. Per la scheda pubblica i riferimenti bibliografici completi devono essere opt-in o inclusi dopo un controllo dei diritti?
7. Quali archivisti/artisti possono fornire esempi reali, e quale misura di tempo/errore giustifica il costo della funzione?

**Decisione:** **Proposta — da valutare**. Nessun mandato operativo, nessuna approvazione e nessuna modifica al software.
