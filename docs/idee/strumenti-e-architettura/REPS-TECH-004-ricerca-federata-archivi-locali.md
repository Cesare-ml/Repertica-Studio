# REPS-TECH-004 — Consultazione federata di più archivi locali, senza fonderli

- **Data ricognizione:** 2026-11-21
- **Stato:** **Proposta — da valutare**
- **Priorità proposta:** **P2**, da validare con utenti che possiedono più archivi; **non** un incarico di sviluppo.
- **Categoria:** ricerca/consultazione trasversale offline, **non** sincronizzazione e **non** gestione multiutente.

## Problema osservato, destinatari e confini

Il `main` contiene un database SQLite attivo e ricerca/filtri su opere e collezionisti del **singolo archivio** ([README](../../../README.md), [archive-filters.ts](../../../src/app/archive-filters.ts), [db.rs](../../../src-tauri/src/db.rs)). Il backup corrente crea cartelle indipendenti contenenti snapshot SQLite e media verificati; il ripristino **sostituisce** l'archivio attivo, non ne combina i contenuti. Chi conserva cataloghi separati per artista, studio, periodo o incarico non ha una consultazione trasversale senza cambiare archivio o importare dati.

**Persone interessate:** artisti con archivi storici e correnti, studi che gestiscono progetti separati, archiviatori e gallerie che devono individuare un'opera fra raccolte distinte. L'esigenza è **ipotizzata**: va prima confermata con esempi reali e volumi di dati; non introdurre multiarchivio per chi usa una sola raccolta.

**Confini rispetto ad altre proposte:**
- [TECH-001](REPS-TECH-001-pacchetto-dati-aperto.md) riguarda **esportazione e reimportazione** di un pacchetto semantico; qui si consulta senza modificare/importare.
- [TECH-003](REPS-TECH-003-diagnostica-integrita-media.md) esamina **integrità del media store attivo**; qui non si riparano file.
- [UX-001 nella PR #14](https://github.com/Cesare-ml/Repertica-Studio/pull/14) riguarda provenienza e movimenti **dentro un archivio**, non federazione.
- Il filtro locale già presente rimane il comportamento predefinito; l'indice federato **non** autorizza accesso ai contatti altrui, condivisione o invio in rete.

## Scenari e specifica UX illustrativa

1. **Aggiungi fonte:** in una futura sezione «Archivi da consultare», l'utente sceglie esplicitamente la cartella di un **backup verificato** oppure (solo in fase successiva) un altro archivio locale supportato. La prima versione accetta soltanto backup integri, per non aprire database attivi in scrittura o leggere file WAL instabili. Nessuna ricerca automatica nelle cartelle del computer.
2. **Anteprima fonte:** mostra nome locale, collocazione, data del backup, versione schema, opere/clienti/certificati presenti, esito delle verifiche, dimensione stimata e possibile presenza di dati personali. Chiede una conferma separata per indicizzare anche i nomi dei collezionisti; impostazione iniziale **no**.
3. **Ricerca trasversale:** selettore «Solo archivio aperto» (predefinito) / «Archivi scelti»; testo e filtri compatibili con quelli esistenti. Ogni risultato porta chiaramente **archivio di provenienza**, **data dello snapshot** e stato «consultazione, non modifica»; filtri per archivio e presenza della foto.
4. **Dettaglio:** vista read-only dei dati consentiti e anteprima immagine caricata solo su richiesta. Azioni «Apri fonte in Esplora file» o «Copia riferimento locale»; non «Vendi», «Modifica» o «Emetti certificato» su una fonte esterna. Non copiare automaticamente record nell'archivio corrente.
5. **Fonte rimossa o disco scollegato:** risultato «Fonte non disponibile», senza inventare dati aggiornati né cancellare il collegamento; pulsante di riconnessione manuale con nuova verifica. Se il backup viene sostituito, l'indice precedente è invalidato; nessuna lettura della cache come se fosse attuale.
6. **Possibili omonimie:** due opere con stesso titolo/codice o lo stesso UUID su archivi duplicati sono **due occorrenze di fonte**, non una sola opera deduplicata né prova che esista una relazione commerciale.

## Dati e API concettuali — NON codice né contratto approvato

Possibile registro **locale** delle fonti: `source_id` UUID interno, `display_name`, `source_kind=verified_backup`, `local_locator` (non esportato per impostazione predefinita), `backup_manifest_digest`, `schema_version`, `last_verified_at`, `index_status`, `include_private_collectors`, `index_generation`; per ogni risultato, chiave composta `source_id + entity_kind + entity_id`. I path macchina-specifici **non** diventano identità globali dell'opera.

**Indice derivato:** soltanto i campi strettamente necessari (titolo, codice, tecnica, periodo e stato); i dati dei collezionisti sono esclusi senza opt-in. Preferire indice effimero o eliminabile integralmente, non duplicazione integrale di foto o note personali. Una revoca elimina indici e riferimenti derivati posseduti dall'app. Data e hash della fonte definiscono la **freschezza**: nessun aggiornamento in background da cartelle non più autorizzate.

API *descrittive*: `inspectSource(folder) -> ReadOnlyReport`; `approveSource(report, scope) -> SourceRef`; `rebuildIndex(source) -> IndexSummary`; `searchAcrossSources(query, source_ids, fields) -> ResultsWithProvenance`; `forgetSource(source_id) -> CleanupReport`. Tutti i confini restano nel processo locale; **non** endpoint MCP e **non** API di rete.

## Offline, sicurezza, privacy, impostazioni e lingue

- Funzionamento completamente offline, senza login, cloud, telemetria o abbonamento. Consentire percorsi scelti liberamente su dischi locali, USB e NAS **già montati**; non effettuare discovery LAN, montaggi automatici o sincronizzazione.
- Leggere copie di backup **verificate** con limiti di file/dimensioni/tempo, senza scritture nella fonte; verificare SHA-256/manifest, schema supportato, symlink, path traversal e riferimenti media. Aprire il DB in sola lettura senza applicare migrazioni, estensioni o SQL fornito dall'utente; evitare di usare direttamente DB attivi con WAL.
- Non indicizzare segreti, note clienti o dati di contatto per default. Opzione esplicita, per fonte, per includere la ricerca anagrafica; consentire revoca e eliminazione delle copie derivate. Non mostrare risultati privati nell'anteprima senza autorizzazione.
- In impostazioni, vedere fonti, cartelle, ambiti concessi, ultima verifica, occupazione cache, comando «Dimentica fonte» e spiegazione che **non cancella i backup originali**. La cronologia delle ricerche può rimanere solo in memoria nella prima fase.
- Interfaccia in lingua di sistema con fallback inglese, override locale, date/numeri formattati secondo locale senza alterare UUID e nomi di campo. Navigabile da tastiera, annunci dei risultati per screen reader, etichette testuali per stato e provenienza, nessuna dipendenza dai colori.
- **Portabilità:** l'indice è ricostruibile e **non** parte dei dati archivistici essenziali. Esportazione dei **soli riferimenti alle fonti** eventualmente possibile, con avviso che i percorsi assoluti sono macchina-specifici e possono rivelare informazioni: non includerli di default nei pacchetti TECH-001. I dati originali restano nei backup e nelle esportazioni standard; niente lock-in.

## Dipendenze, alternative, rischi, complessità

**Dipendenze ipotetiche:** lettore di backup già verificati, compatibilità esplicita con schema SQLite v1/v2 e successive migrazioni, indice derivato read-only, gestione permessi filesystem Tauri, isolamento delle query e caricamento media on demand. La [PR #17](https://github.com/Cesare-ml/Repertica-Studio/pull/17) per fotografie multiple è aperta, non ancora parte di `main`: la proposta deve trattarla come fonte facoltativa futura.

**Alternative da discutere:** (A) riaprire/ripristinare manualmente ogni backup: zero nuova complessità ma esperienza lenta e rischiosa se ripristino frequente; (B) esportare CSV da ciascuno e cercarli in strumenti esterni: già possibile per i cinque registri, ma foto e contesto non integrati; (C) importare tutto in un DB unico: può rompere separazione tra clienti e storici e introduce collisioni. La federazione è sensata solo dopo aver verificato che il bisogno sia frequente.

**Rischi:** esposizione di dati clienti di archivi diversi, cache stantie, perdita del contesto del certificato, confusione fra copie e originali, carico elevato su NAS o dischi lenti, manomissione di backup, path non più raggiungibili. Mitigazioni: ambiti stretti, etichette di provenienza, indici cancellabili, validazione e timeout, niente merge automatico, limite esplicito alle fonti/risultati.

**Complessità stimata:** **medio-alta** per sole opere con 2–3 backup verificati; **alta** con indici grandi, allegati multipli e supporto agli archivi attivi. Nessun obbligo di adottarla.

## Criteri di accettazione per eventuale futura implementazione

1. Con due backup verificati e un archivio attivo, la ricerca mostra i risultati delle **fonti selezionate**, completi di etichetta di provenienza e data snapshot; default mostra soltanto l'archivio attivo.
2. Stesso UUID/titolo in due fonti non diventa record unico. I dati non vengono modificati, fusi o importati. I comandi di scrittura non sono esposti sulle schede esterne.
3. Backup non valido, symlink, percorso malevolo, disconnessione USB/NAS, schema futuro e file modificato producono errore leggibile senza danneggiare nessun archivio.
4. Senza consenso per i clienti, l'indice non contiene né cerca nominativi o contatti; revoca dell'accesso elimina la cache derivata. Non vengono aperte porte di rete.
5. Prova con molte opere e media, Unicode, più lingue, tastiera/screen reader e rilancio dell'app: ricerca responsiva con progressi/cancellazione, nessuna scansione automatica delle cartelle.
6. Una reinstallazione può ricostruire l'indice dalle fonti scelte; la cache non è necessaria al recupero dei dati.

## Requisiti accertati, ipotesi e domande al proprietario

**Accertato nel repository al 2026-11-21:** SQLite e archivio locali; filtri in un archivio; backup verificati come cartelle con manifest; ripristino sostitutivo; CSV separati per opere, clienti, certificati, vendite e spese. Non si osserva un catalogo federato in `main`.

**Ipotesi:** domanda reale di più archivi, UI «Archivi da consultare», indice derivato e sorgente iniziale costituita da backup verificati. L'assenza di una funzionalità non significa che sia obbligatorio aggiungerla.

**Domande:** esistono utenti con **più archivi distinti** o basta una ricerca dentro uno solo? Ricerca anche i clienti oppure solo le opere? È accettabile conservare una cache di titoli sul disco locale? È davvero utile collegare backup storici o serve distinguere cataloghi correnti? Quanti archivi e quanti record sono realistici?

## Fasi di valutazione, non piano approvato

1. Interviste a 3–5 utilizzatori e prototipo statico; misurare necessità reale rispetto a ricerca singolo archivio.
2. Specificare provenienza, privacy e lettura sicura di backup; valutare scalabilità con dataset sintetici.
3. Solo se approvato: ipotesi MVP con fonti verificate e ricerca opere read-only; test su USB/NAS e dati ostili.
4. Eventuale estensione per anagrafiche esplicitamente autorizzate e immagini multiple, senza sincronizzazione.

> Nessuna modifica operativa o implementazione è autorizzata da questa proposta.
