# Idee future — Gestione e UX

**Catalogo esclusivamente ideativo**, da valutare personalmente. Le schede non autorizzano implementazioni, modifiche al prodotto o assegnazioni agli sviluppatori.

Aggiornato: **2026-11-21** (terza ricognizione). Stato delle proposte: **Proposta — da valutare**.

| ID stabile | Idea | Priorità ipotetica | Stato |
| --- | --- | --- | --- |
| [REPS-UX-001](REPS-UX-001-provenienza-movimentazioni.md) | Provenienza, ubicazioni e movimentazioni delle opere | P1 | Proposta — da valutare |
| [REPS-UX-002](REPS-UX-002-trattative-prenotazioni.md) | Trattative, proposte e prenotazioni prima della vendita | P1–P2 | Proposta — da valutare |
| [REPS-UX-003](REPS-UX-003-costo-e-margine-per-opera.md) | Scheda economica per opera e costi attribuiti | P2 | Proposta — da valutare |
| [REPS-UX-004](REPS-UX-004-rapporti-stato-conservazione.md) | Rapporti di condizione e conservazione, con confronti datati | P2 | Proposta — da valutare |
| [REPS-UX-005](REPS-UX-005-edizioni-multipli-serie-numerate.md) | Edizioni numerate, multipli e varianti con esemplari individuali | P2 | Proposta — da valutare |
| [REPS-UX-006](REPS-UX-006-incarichi-opere-su-richiesta.md) | Incarichi per opere su richiesta, brief versionati e avanzamento | P2 | Proposta — da valutare |
| [REPS-UX-007](REPS-UX-007-dossier-presentazione-opere.md) | Dossier e schede di presentazione con dati condivisi selettivamente | P1–P2 | Proposta — da valutare |

## Cosa è stato verificato per evitare duplicati

- [README principale](../../../README.md), [MVP](../../MVP.md), [economia e lingue](../../ECONOMY-I18N.md): catalogazione, ricerca, anagrafica collezionisti, vendite, certificati immutabili, backup in cartelle scelte, ripristino, suggerimento prezzo con coefficiente e statistiche mensili esistono già come funzionalità o sviluppi documentati.
- [Issue #13](https://github.com/Cesare-ml/Repertica-Studio/issues/13): incassi reali, documenti di acquisto, spese ricorrenti ed export CSV finanziario sono proposte già presenti; nessuna delle sette idee le ripropone come novità.
- [Issue #5](https://github.com/Cesare-ml/Repertica-Studio/issues/5): import da Gestionale Opere, non duplicato.
- [Issue #3](https://github.com/Cesare-ml/Repertica-Studio/issues/3) e [#4](https://github.com/Cesare-ml/Repertica-Studio/issues/4): MCP locale e updater firmati appartengono al filone tecnico.
- Alla **prima** ricognizione sul branch `main` non esisteva ancora `docs/idee/`; la PR #14 ha introdotto il primo catalogo di gestione e UX. Le successive ricognizioni lavorano sulla medesima PR documentale in bozza.

## Seconda ricognizione (21 novembre 2026)

Le nuove proposte completano due lacune non coperte dalle prime schede o dalle issue aperte:

- **REPS-UX-004** documenta rilievi dello stato fisico e confronti conservativi. Si distingue da UX-001 (movimenti/provenienza) e da [TECH-003, PR #16](https://github.com/Cesare-ml/Repertica-Studio/pull/16) (integrità dei file digitali). La [PR #17](https://github.com/Cesare-ml/Repertica-Studio/pull/17) propone una galleria fotografica locale, ancora distinta dal registro dei rilievi e non assunta come presente su `main`.
- **REPS-UX-005** affronta la produzione e catalogazione professionale di edizioni numerate, copie fisiche e prove d'artista, non la sola immagine multipla di un'opera. Le regole correnti «una vendita per record opera» e «certificati immutabili» restano vincoli di progettazione.
- L'altra automazione lavora esclusivamente sotto `docs/idee/strumenti-e-architettura/` e ha già proposto portabilità semantica, automazioni locali e diagnostica media. Nessun file del suo filone è stato modificato.
- [Issue #13](https://github.com/Cesare-ml/Repertica-Studio/issues/13) comprende commissioni, incassi, documenti e cashflow: non viene duplicata in queste schede.
- Nessuna priorità proposta costituisce approvazione o incarico di implementazione.

## Terza ricognizione (21 novembre 2026)

Dopo aver riletto il branch `main` (README, MVP, ECONOMY-I18N, DATA-PORTABILITY e modelli UI) e le schede aggiornate nelle PR documentali [#14](https://github.com/Cesare-ml/Repertica-Studio/pull/14) e [#16](https://github.com/Cesare-ml/Repertica-Studio/pull/16), si approfondiscono due **processi differenti** senza riproporre funzionalità esistenti:

- **REPS-UX-006 — Incarichi per opere su richiesta:** brief e revisioni su opere **non ancora realizzate**, con fasi creative e conversione solo manuale in una scheda opera. Non è UX-002 (trattative/offerte su opere esistenti), non è UX-003 (attribuzione spese), né [issue #13](https://github.com/Cesare-ml/Repertica-Studio/issues/13) (commissioni economiche, incassi e documenti). Nessuna registrazione automatica di ricavi, acconti o certificati.
- **REPS-UX-007 — Dossier di presentazione e schede condivisibili:** documenti narrativi/visivi locali con scelta esplicita dei **soli dati divulgabili**, distinti dai certificati numerati immutabili già presenti, dai cinque CSV integrali e da TECH-001 (portabilità semantica). Non crea un sito, un canale di invio o una piattaforma marketing.
- **Filone tecnico rispettato:** TECH-001…005 riguardano portabilità, automazioni, diagnostica, ricerca federata e anteprime. Le nuove schede parlano **solo di UX e processi d'uso** e possono, in futuro e se approvate, interoperare con tali strumenti senza modificarne definizioni o file.
- **Validazione ancora necessaria:** l'effettiva domanda degli artisti che lavorano su incarico e la frequenza di condivisione dossier sono ipotesi; le priorità restano provvisorie. Tutte le sette schede sono **Proposta — da valutare**.

## Criteri editoriali

1. Ogni idea deve aggiungere un **processo professionale distinto**, non rinominare una funzione già presente. Chi propone cita la differenza rispetto all'MVP e alle issue.
2. Ogni scheda descrive problema, utenti, scenari, flussi, dati, offline, lingue, privacy, export/import, complessità, dipendenze, rischi, alternative, criteri e fasi; separa **certezze**, **ipotesi** e **decisioni da richiedere al proprietario**.
3. Non affermare autenticità della provenienza, saldo bancario reale, incasso o profitto fiscale senza prove; non trasformare un'ipotesi in una promessa del software.
4. Tutto resta **gratuito, open source, offline-first**, senza account, abbonamenti, sincronizzazione cloud o invio dei dati a servizi terzi; lingua del sistema con fallback inglese.
5. Prima di creare una nuova scheda verificare altre schede, issue e PR aperte, comprese quelle di `docs/idee/strumenti-e-architettura/` se aggiunte dall'altro filone. Non modificare i file dell'altro filone. Se un'idea interseca altre schede, linkarle e definire confini.
6. Ogni ulteriore proposta deve avere ID successivo `REPS-UX-006`, ecc., mantenendo gli ID già assegnati stabili; gli sviluppatori non devono implementarla senza revisione esplicita del proprietario.
7. Lavorare solo su file Markdown di questa cartella e tramite PR **documentale**; non effettuare merge automatici.

## Sequenza di valutazione suggerita (non autorizzazione)

Discutere per prime le esigenze davvero ricorrenti: **movimenti e custodia** (001), **trattative e prenotazioni** (002), **costi analitici** (003). Valutare separatamente l'interesse per **rilievi conservativi** (004), **edizioni e multipli** (005) e **incarichi su richiesta** (006), processi potenzialmente specialistici. Esaminare **dossier di presentazione** (007) quando è chiaro quali dati gli utenti desiderano condividere e quali debbano rimanere privati. La sequenza e le priorità sono opinioni progettuali, **non un piano approvato**.
