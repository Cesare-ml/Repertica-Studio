# Idee future — Gestione e UX

**Catalogo esclusivamente ideativo**, da valutare personalmente. Le schede non autorizzano implementazioni, modifiche al prodotto o assegnazioni agli sviluppatori.

Aggiornato: **2026-11-21**. Stato delle proposte: **Proposta — da valutare**.

| ID stabile | Idea | Priorità ipotetica | Stato |
| --- | --- | --- | --- |
| [REPS-UX-001](REPS-UX-001-provenienza-movimentazioni.md) | Provenienza, ubicazioni e movimentazioni delle opere | P1 | Proposta — da valutare |
| [REPS-UX-002](REPS-UX-002-trattative-prenotazioni.md) | Trattative, proposte e prenotazioni prima della vendita | P1–P2 | Proposta — da valutare |
| [REPS-UX-003](REPS-UX-003-costo-e-margine-per-opera.md) | Scheda economica per opera e costi attribuiti | P2 | Proposta — da valutare |

## Cosa è stato verificato per evitare duplicati

- [README principale](../../../README.md), [MVP](../../MVP.md), [economia e lingue](../../ECONOMY-I18N.md): catalogazione, ricerca, anagrafica collezionisti, vendite, certificati immutabili, backup in cartelle scelte, ripristino, suggerimento prezzo con coefficiente e statistiche mensili esistono già come funzionalità o sviluppi documentati.
- [Issue #13](https://github.com/Cesare-ml/Repertica-Studio/issues/13): incassi reali, documenti di acquisto, spese ricorrenti ed export CSV finanziario sono proposte già presenti; nessuna di queste tre idee le ripropone come novità.
- [Issue #5](https://github.com/Cesare-ml/Repertica-Studio/issues/5): import da Gestionale Opere, non duplicato.
- [Issue #3](https://github.com/Cesare-ml/Repertica-Studio/issues/3) e [#4](https://github.com/Cesare-ml/Repertica-Studio/issues/4): MCP locale e updater firmati appartengono al filone tecnico.
- Al momento della ricognizione sul branch `main` non esisteva ancora `docs/idee/`; nessuna PR aperta nel repository. I nuovi file sono quindi i primi del filone **gestione e UX**.

## Criteri editoriali

1. Ogni idea deve aggiungere un **processo professionale distinto**, non rinominare una funzione già presente. Chi propone cita la differenza rispetto all'MVP e alle issue.
2. Ogni scheda descrive problema, utenti, scenari, flussi, dati, offline, lingue, privacy, export/import, complessità, dipendenze, rischi, alternative, criteri e fasi; separa **certezze**, **ipotesi** e **decisioni da richiedere al proprietario**.
3. Non affermare autenticità della provenienza, saldo bancario reale, incasso o profitto fiscale senza prove; non trasformare un'ipotesi in una promessa del software.
4. Tutto resta **gratuito, open source, offline-first**, senza account, abbonamenti, sincronizzazione cloud o invio dei dati a servizi terzi; lingua del sistema con fallback inglese.
5. Prima di creare una nuova scheda verificare altre schede, issue e PR aperte, comprese quelle di `docs/idee/tecniche-e-strumenti/` se aggiunte dall'altro filone. Non modificare i file dell'altro filone. Se un'idea interseca altre schede, linkarle e definire confini.
6. Ogni ulteriore proposta deve avere ID successivo `REPS-UX-004`, ecc., mantenendo gli ID già assegnati stabili; gli sviluppatori non devono implementarla senza revisione esplicita del proprietario.
7. Lavorare solo su file Markdown di questa cartella e tramite PR **documentale**; non effettuare merge automatici.

## Sequenza di valutazione suggerita (non autorizzazione)

Prima validare con l'utente la gestione dei **movimenti e della custodia** (001), poi il significato operativo di **prenotazione commerciale** (002), infine quali costi e stime associare alle singole opere (003). La sequenza e le priorità sono opinioni progettuali, **non un piano approvato**.
