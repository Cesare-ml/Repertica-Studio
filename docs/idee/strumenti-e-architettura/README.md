# Idee future — Strumenti e architettura

**Catalogo esclusivamente ideativo — nessuna funzionalità approvata o assegnata allo sviluppo.**

Data ricognizione: **2026-11-21** (quinta ricognizione). Stato uniforme: **Proposta — da valutare**.

| ID stabile | Scheda | Priorità ipotetica | Stato |
| --- | --- | --- | --- |
| [REPS-TECH-001](REPS-TECH-001-pacchetto-dati-aperto.md) | Pacchetto di dati aperto, esportazione semantica e reimportazione verificabile | P1–P2 | Proposta — da valutare |
| [REPS-TECH-002](REPS-TECH-002-automazioni-locali-consapevoli.md) | Automazioni locali con anteprima, permessi e tracciabilità | P2 | Proposta — da valutare |
| [REPS-TECH-003](REPS-TECH-003-diagnostica-integrita-media.md) | Diagnostica dei media e recupero assistito negli archivi grandi | P2 | Proposta — da valutare |
| [REPS-TECH-004](REPS-TECH-004-ricerca-federata-archivi-locali.md) | Consultazione federata di più archivi locali senza fusione | P2 | Proposta — da valutare |
| [REPS-TECH-005](REPS-TECH-005-anteprime-media-grandi-archivi.md) | Anteprime derivate e cancellabili per raccolte fotografiche numerose | P2 | Proposta — da valutare |
| [REPS-TECH-006](REPS-TECH-006-backup-cifrati-portabili.md) | Backup cifrati facoltativi, con recupero su altri dispositivi e formato versionato | P1–P2 | Proposta — da valutare |
| [REPS-TECH-007](REPS-TECH-007-archivio-media-esterno-rilocalizzabile.md) | Archivio degli originali fotografici in cartella esterna opzionale, con migrazione verificata e backup sempre autosufficiente | P2 | Proposta — da valutare |
| [REPS-TECH-008](REPS-TECH-008-viste-salvate-filtri-ricerca-locale.md) | Viste nominative, filtri combinati e ricerca strutturata nell'archivio attivo (non federazione) | P1–P2 | Proposta — da valutare |

## Ricognizione e confini

Verificati sul branch `main` al **21 novembre 2026**: [README](../../../README.md), [MVP](../../MVP.md), [economia e localizzazione](../../ECONOMY-I18N.md), modelli dati e codice nativo di `db.rs`, `backup.rs`, `export.rs`, `commands.rs`.

- **Già presente**: database locale SQLite, backup completo a cartella scelta con manifest e hash SHA-256, ispezione prima del ripristino e conservazione della copia precedente. Le schede non ripropongono questo sistema.
- **Già presente**: cinque CSV unidirezionali per **opere, collezionisti, certificati, vendite e spese**, con ID stabili e scelta esplicita della destinazione ([docs/DATA-PORTABILITY.md](../../DATA-PORTABILITY.md)). La proposta TECH-001 riguarda invece un **formato di interscambio semantico** per tutti i domini e la verifica di reimportabilità, che non sostituisce i backup.
- **Già pianificato**: [issue #5](https://github.com/Cesare-ml/Repertica-Studio/issues/5) per migrazione assistita **da Gestionale Opere**; TECH-001 è indipendente dalla fonte, non realizza l'esportatore privato e propone prima importazione in archivio vuoto.
- **Già pianificato**: [issue #3](https://github.com/Cesare-ml/Repertica-Studio/issues/3) per MCP **STDIO read-only**. TECH-002 non è un server MCP: descrive un eventuale sistema di regole **interne** all'app, attivato su esplicita autorizzazione.
- **Già pianificato**: [issue #4](https://github.com/Cesare-ml/Repertica-Studio/issues/4) per aggiornamenti firmati. Nessuna proposta li riguarda.
- **Già in corso in altro filone**: [PR #14](https://github.com/Cesare-ml/Repertica-Studio/pull/14), schede UX-001 provenienza/movimenti, UX-002 trattative/prenotazioni, UX-003 costi attribuiti per opera, UX-004 rapporti di condizione e UX-005 edizioni numerate. Le proposte qui non definiscono quei processi, ma semmai strumenti trasversali utilizzabili in futuro.
- **Già presente**: una foto principale e fino a 20 immagini aggiuntive JPEG/PNG/WebP per opera (limite 12 MiB ciascuna), dopo il merge della [PR #17](https://github.com/Cesare-ml/Repertica-Studio/pull/17). La verifica SHA-256 dei media è prevista nel **backup**. TECH-003 distingue verifica dell'archivio attivo e ricerca **assistita** di file mancanti, senza cancellazione automatica.

Non è stata creata né considerata approvata una roadmap di sviluppo. Le priorità P1/P2 sono soltanto suggerimenti.

## Seconda ricognizione — 21 novembre 2026

- **REPS-TECH-004:** ricerca read-only su più backup verificati, senza modificare, importare, sincronizzare o fondere archivi. Non ripete la ricerca nel singolo archivio già presente né la proposta TECH-001 sulla portabilità semantica.
- **REPS-TECH-005:** anteprime derivate, temporanee e ricostruibili per migliorare prestazioni con molte fotografie. Non ripete la diagnostica TECH-003 né la [PR #17](https://github.com/Cesare-ml/Repertica-Studio/pull/17) sulla galleria, ancora aperta.
- **Aggiornamento `main`:** esportazione CSV di opere, collezionisti e certificati aggiunta dopo la precedente ricognizione (PR #18 integrata), oltre a vendite e spese; nessuno dei cinque CSV è un formato di reimportazione o backup completo.
- **Coordinamento:** [PR #14](https://github.com/Cesare-ml/Repertica-Studio/pull/14) contiene UX-001…UX-005; nessun file dell'altro filone viene modificato o considerato approvato.
- **Prudenza:** entrambe le nuove proposte sono P2 ipotetiche; richiedono convalida dei casi d'uso, benchmark o interviste prima di ogni decisione.

## Terza ricognizione — 21 novembre 2026

- **REPS-TECH-006:** i backup attuali sono completi, verificati e ripristinabili ma **in chiaro**. La nuova scheda valuta backup **facoltativamente cifrati**, con segreto portabile tra dispositivi, contenitore autenticato, verifica integrale e ripristino isolato prima della sostituzione dell'archivio.
- **Confini:** non modifica né sostituisce il backup ordinario; non propone la cifratura del database attivo; non duplica la portabilità semantica TECH-001, l'integrità dei media TECH-003, la ricerca federata TECH-004 o la proposta di dossier UX-007 della [PR #14](https://github.com/Cesare-ml/Repertica-Studio/pull/14).
- **Rischi e decisioni:** se la frase segreta va perduta, la copia cifrata potrebbe essere irrecuperabile; la soluzione richiede analisi del rischio, confronto con BitLocker/FileVault/LUKS e revisione crittografica indipendente **prima** di qualunque approvazione.
- **Stato:** *Proposta — da valutare*. Nessuna modifica software, approvazione, incarico agli sviluppatori o merge.

## Quarta ricognizione — 21 novembre 2026

- **REPS-TECH-007:** studiata la scelta facoltativa della cartella degli **originali fotografici**, distinta dal backup e dalla cache delle anteprime. La proposta richiede migrazione verificata, gestione sicura dei volumi scollegati e **backup sempre completi e indipendenti dal percorso esterno**; priorità soltanto ipotetica P2. È un progetto diverso da TECH-003 (diagnostica media), TECH-005 (miniature), TECH-006 (cifratura delle copie) e dalle funzioni gestionali UX-001…009 della [PR #14](https://github.com/Cesare-ml/Repertica-Studio/pull/14).
- **Aggiornamento del riferimento a `main`:** la galleria di immagini aggiuntive della [PR #17](https://github.com/Cesare-ml/Repertica-Studio/pull/17) è stata effettivamente integrata; `gallery.rs` e lo schema SQLite fino a v4 ora sono presenti nel branch principale. La documentazione ideativa non deve confondere funzioni implementate con PR ancora aperte.
- **Vincolo di sicurezza:** prima di approvare l'idea occorrono convalida dei casi d'uso, collaudo dei backup su USB/NAS ([issue #7](https://github.com/Cesare-ml/Repertica-Studio/issues/7)) e progettazione robusta della recovery di una migrazione cross-volume. Non sono stati modificati file operativi.

## Quinta ricognizione — 21 novembre 2026

- **[REPS-TECH-008](REPS-TECH-008-viste-salvate-filtri-ricerca-locale.md):** approfondite viste salvate e filtri strutturati per il **solo archivio attivo**, distinti da TECH-004 (ricerca federata nei backup), TECH-001 (interscambio), UX-007 (dossier) e dalle funzionalità già presenti su `main`.
- **Riscontro tecnico verificato:** `src/app/archive-filters.ts` combina testo libero, stato, collezionista e ordinamento **in memoria** su tutte le opere restituite da `list_artworks`; non offre viste persistenti, predicati tipizzati su campi/assenze o interrogazioni combinabili. `docs/MVP.md` menziona genericamente ricerche avanzate: questa scheda ne chiarisce la possibile UX, la privacy dei criteri salvati, il comportamento con dati incompleti e l'eventuale bisogno di ottimizzare query su archivi molto grandi.
- **Garanzie:** i cinque CSV esistenti restano sempre esportazioni **integrali** e non filtrate, le viste non creano duplicati dei record e non sono da intendere come report storici immutabili. Nessun SQL arbitrario, cloud, account, server MCP o modifica all'altro filone.
- **Convalide richieste:** esempi di query realmente ripetute, benchmark su archivi di varie dimensioni e selezione dei campi; priorità P1–P2 solo ipotetica, stato invariato **Proposta — da valutare**. Nessuna implementazione né merge.

## Regole per le prossime ricognizioni

1. Consultare `main`, issue e PR aperte; confrontare le schede di entrambi i filoni prima di aggiungerne altre. Se un problema è già risolto, aggiornare la scheda indicando il cambiamento, non produrre un doppione.
2. Assegnare ID successivi `REPS-TECH-009`, `010`, ecc., **mai rinumerare** schede citate altrove. Mantenere titolo, data, stato, requisiti verificati/ipotesi/decisioni e collegamenti ai limiti correnti.
3. Valutare sempre dati personali, assenza di lock-in, formati aperti, multilingua (lingua OS, fallback inglese), funzionamento offline-first, assenza di account o abbonamenti e scelta libera della cartella.
4. Modificare **solo Markdown dentro questa cartella**, usando un branch e una PR esclusivamente documentali. Non toccare `docs/idee/gestione-e-ux/`; niente merge automatici, modifiche al codice, issue operative o assegnazioni implicite.
5. Se emergono trade-off inconciliabili, registrarli come domanda al proprietario: non decidere in sua vece.
