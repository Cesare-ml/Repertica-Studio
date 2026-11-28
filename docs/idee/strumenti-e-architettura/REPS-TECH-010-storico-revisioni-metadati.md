# REPS-TECH-010 — Storico locale delle revisioni e recupero mirato dei metadati

- **ID stabile:** REPS-TECH-010
- **Data:** 2026-11-21
- **Stato:** **Proposta — da valutare**
- **Priorità proposta:** **P2**, da confermare con casi d'uso; non è un'attività autorizzata
- **Ambito:** affidabilità e tracciabilità delle modifiche ai metadati del catalogo **nell'archivio attivo**, non backup, non controllo dei movimenti fisici e non contabilità fiscale

## Motivazione, destinatari e lacuna verificata

Una scheda d'opera può essere corretta molte volte durante inventariazione, autenticazione, preparazione di mostre e passaggio fra professionisti. La sovrascrittura accidentale di titolo, tecnica, dimensioni, descrizione o ubicazione può privare l'archivio della formulazione precedente senza che un backup recente sia disponibile o senza voler ripristinare **tutto** il database. Serve valutare una cronologia consultabile e un recupero selettivo dei **metadati** della singola scheda.

**Destinatari:** artista che corregge descrizioni, galleria che aggiorna cataloghi, archivista che ricontrolla attribuzioni, studio che gestisce centinaia/migliaia di opere; soltanto in una fase successiva e con tutele dedicate, anagrafiche private di clienti o altri registri.

**Riscontro su `main` (21/11/2026):**
- `src-tauri/src/commands.rs`: `save_artwork` sostituisce i campi della riga con un `UPDATE` e aggiorna `updated_at`; `save_collector` fa analogamente. Non è visibile una tabella o un'API per la cronologia delle precedenti versioni.
- `src-tauri/src/db.rs`: schema SQLite fino a `user_version=5`, con `created_at` e `updated_at` ma senza registro delle revisioni delle schede. I certificati, invece, conservano `snapshot_json` al momento dell'emissione e sono immutabili: **non** devono essere resi modificabili.
- I backup verificati sono **istantanee integrali** dell'archivio con media; il ripristino sostituisce tutto l'archivio corrente, salvando quello precedente. Non consentono la stessa UX di confronto/ripristino di singoli campi senza un'operazione manuale complessa.

**Confini e duplicati evitati:** [TECH-009](REPS-TECH-009-catalogo-verifiche-conservazione-backup.md) cataloga e verifica **copie complete**; [TECH-001](REPS-TECH-001-pacchetto-dati-aperto.md) descrive **interscambio**; [TECH-003](REPS-TECH-003-diagnostica-integrita-media.md) controlla **file digitali**; [TECH-008](REPS-TECH-008-viste-salvate-filtri-ricerca-locale.md) crea **filtri**, non revisioni. Nella [PR #14](https://github.com/Cesare-ml/Repertica-Studio/pull/14), **UX-001** riguarda provenienza/ubicazione e movimentazioni fisiche, **UX-004** documenta rilievi conservativi e **UX-013** riguarda resi e rettifiche di vendita: la presente proposta non sostituisce tali **eventi professionali**. Una modifica alla casella “ubicazione” non costituisce prova di una movimentazione avvenuta.

## Scenari d'uso e flussi UX ipotetici

1. **Correzione semplice:** l'utente apre un'opera, aggiorna la tecnica e salva. Una nuova revisione registra il valore precedente e quello nuovo, una data/ora e l'origine `ui`; la scheda corrente resta leggibile come oggi. Salvataggio identico ai valori già presenti non crea una revisione inutile.
2. **Verifica storica:** dalla scheda si apre «Cronologia modifiche». Una timeline in ordine temporale mostra le date, i campi cambiati (non una pagina piena di dati invariati), e un confronto fra due versioni; le note lunghe dispongono di diff testuale accessibile. Non viene mostrato un inesistente nome di operatore: senza account non c'è identità affidabile del redattore.
3. **Recupero selettivo:** l'utente sceglie una versione precedente e specifica i singoli campi di testo da **riproporre**. Una schermata mostra anteprima e avviso dei valori correnti. Confermando, si crea una **nuova** revisione; non si cancella la cronologia né si riscrive il passato.
4. **Situazioni vincolate:** se l'opera è venduta, il recupero non può riportare lo stato a `available` né violare le regole di `sales`/certificati; i campi non recuperabili sono chiaramente marcati. La modifica non aggiorna retroattivamente snapshot dei certificati, ricavi, incassi, stock o movimenti.
5. **Dati eliminati/archivi ripristinati:** una scheda definitivamente cancellata senza vincoli non è recuperabile tramite questa funzione nella prima fase; il recupero di file o record cancellati resta competenza del backup. Dopo il ripristino di un archivio intero la cronologia torna allo stato contenuto in quella copia, senza “unire” automaticamente revisioni successive.

**Flusso minimo proposto:** Scheda opera → **Cronologia** → lista revisioni → confronto → «Ripristina alcuni campi» → anteprima → conferma esplicita → nuova revisione. Senza conferma, nessuna scrittura. Possibile modalità iniziale **sola lettura**, utile per misurare valore prima del ripristino selettivo.

## Requisiti certi, ipotesi e modello dati descrittivo

### Vincoli certi dal prodotto

Archivio SQLite locale, UUID stabili delle opere, validazioni Rust, `updated_at` già presente; certificati immutabili e dati economici con regole proprie; backup completo esterno su cartella scelta, offline-first, nessun login o abbonamento. I valori della storia non possono essere presentati come prova certificata di autenticità, autore fisico o data effettiva di una movimentazione.

### Ipotesi, NON schema/API approvate

```text
MetadataRevision {
  revision_id: uuid,
  entity_type: "artwork",            # prima fase; "collector" eventuale fase separata
  entity_id: uuid,
  sequence: integer,                # progressivo dell'entità nell'archivio
  committed_at_utc: timestamp,
  origin: "ui" | "import" | "system", # fonte tecnica, NON identità di persona
  schema_version: integer,
  changed_fields: string[],
  before: object?, after: object?,  # campi consentiti, canonici e versionati
  reason?: string                   # nota manuale facoltativa; valutare privacy
}
```

Alternative per l'archiviazione: **snapshot completi** dei metadati per ogni revisione (lettura/ricostruzione semplice, occupazione maggiore) oppure **differenze** con checkpoint periodici (minore spazio potenziale, recupero fragile se una differenza si corrompe). Non scegliere senza benchmark. Il modello non va esteso implicitamente alle fotografie binarie, ai segreti o alle vendite.

API **soltanto concettuali**:
- `listMetadataRevisions(entityId, cursor, limit) -> RevisionSummary[]`
- `compareMetadataRevisions(entityId, revisionA, revisionB) -> FieldDiff[]`
- `previewRestoreFields(entityId, fromRevision, selectedFields, expectedCurrentRevision) -> RestorePreview`
- `commitRestoreFields(previewToken) -> NewRevision`

Il `previewToken` dovrebbe essere breve, locale e invalidato da cambi alla scheda; non rappresenta un'autorizzazione persistente. Campo di versione/numero revisione confrontato al salvataggio per evitare che una finestra vecchia sovrascriva modifiche più recenti.

**Atomicità:** registrazione del cambio e aggiornamento della scheda nella **stessa transazione SQLite**; fallimento dell'una annulla l'altra. Altrimenti una cronologia incompleta farebbe più danni che non averla. Valutare anche modifiche da altri ingressi (import eventuale, azioni MCP future) con un unico servizio nativo di scrittura; nessuna garanzia di tracciabilità se il proprietario modifica direttamente SQLite fuori dall'app. Il database locale non offre da solo una prova anti-manomissione.

**Immagini/galleria:** in prima fase registrare eventualmente solo metadati (caption, ordine, riferimento) *senza promettere ripristino dei byte*. Per file cancellati e immagini originali rimandare a TECH-003 e ai backup. Se in futuro si versionano anche file, servirebbe una progettazione separata di storage, quote, sicurezza e cancellazione.

## Offline, esportabilità, compatibilità, lingua e impostazioni

- Completamente offline, nessun server, porta di rete, cloud, account, abbonamento o telemetria; dati della cronologia nel database locale e inclusi nelle **copie complete** una volta eventualmente implementati.
- Scegliere se includere le revisioni in un futuro formato aperto versionato [TECH-001](REPS-TECH-001-pacchetto-dati-aperto.md), dichiarandone campo, versioni e compatibilità. Gli **attuali sei CSV integrali** non diventano automaticamente una storia delle modifiche. Un eventuale export separato JSONL/CSV della cronologia deve essere esplicito e documentare perdita di semantica in formati tabellari; importare solo in contesto controllato, mai fondere storie arbitrarie con stessi UUID.
- **Compatibilità:** non cambiare i certificati già emessi; vecchi database senza revisioni non devono ricevere una cronologia inventata. Versione iniziale della scheda potrebbe essere marcata «stato al momento dell'attivazione», non «dato originale». Migrazioni transazionali, apertura di vecchi backup, rollback su errori e rifiuto esplicito di schema più recente.
- Lingua di sistema, fallback **inglese**, override locale disponibile; UI nelle lingue supportate (it/en/es/fr/de), date/ore e numeri formattati secondo locale. Nomi tecnici delle chiavi e identificativi non tradotti; contenuti immessi dall'utente mantenuti nella lingua originale.
- **Impostazioni ipotetiche:** abilitazione o pausa della registrazione (da valutare: lasciare buchi dichiarati nella storia), campo facoltativo «motivo della modifica», profondità massima di visualizzazione per pagina. Evitare cancellazione implicita della storia per risparmiare spazio; eventuale policy di conservazione richiede decisione separata e segnalazione esplicita.

## Sicurezza, privacy, accessibilità

La cronologia può conservare **dati precedentemente cancellati**. Questo è particolarmente delicato per indirizzi e recapiti: **prima fase limitata ai metadati delle opere**, non ai contatti personali dei collezionisti. Un'estensione a `Collector` va discussa a parte con scelte esplicite su retention, rimozione definitiva e protezione dei dati di terzi. Anche descrizioni di opere possono contenere nomi e informazioni private; avviso nella UI quando si attiva lo storico, nessun invio esterno, nessuna visualizzazione di valori storici negli export generali senza scelta informata.

Il ripristino dei campi non è un «annulla tutto»: preserva ID, vincoli, stato vendita, contatori e documenti storici. Proteggere il servizio contro payload eccessivi, campi sconosciuti, JSON non valido, injection nella rappresentazione dei diff e perdita di dati per conflitti fra finestre. Le copie SQLite possono essere manomesse da chi ha accesso al filesystem: non presentare la cronologia come **audit forense, firma o prova legale**.

Timeline e diff usabili con tastiera e screen reader, focus stabile, indicatori non basati solo su colore, testo alternativo «prima/dopo», scorciatoie non obbligatorie, conferma leggibile, annuncio accessibile di errori o conflitti. Non utilizzare indicatori che suggeriscano falsamente identità/autorizzazione dell'autore.

## Dipendenze, alternative, rischi e complessità

**Dipendenze progettuali:** chiarire quali campi delle opere siano versionabili, contratto stabile delle validazioni native, migrazione schema e test transazionali, separazione di status/vendite da campi editoriali; verificare su backup/restore reali (issue [#7](https://github.com/Cesare-ml/Repertica-Studio/issues/7)). Nessuna nuova dipendenza tecnica o scelta di libreria è autorizzata.

**Alternative:**
1. **Solo backup frequenti + ripristino intero:** già disponibile; basso costo aggiuntivo, ma rischio di perdere modifiche recenti e impossibilità di confronto puntuale.
2. **«Annulla» per la sola sessione di modifica:** UX più semplice e minore conservazione dei dati, non copre errori scoperti a distanza.
3. **Registro descrittivo senza recupero:** offre tracciabilità ma evita operazioni rischiose; possibile prima fase.
4. **Snapshot immutabili per revisione** vs **delta + checkpoint**: trade-off fra spazio, verificabilità e resilienza, da misurare.
5. **Cronologia completa di tutte le entità:** fuori dal primo perimetro; rischi elevati su anagrafiche private, finanza, eliminazioni e rapporti con certificati.

**Rischi:** accumulo illimitato di revisioni; diffusione involontaria di valori riservati; falsa attribuzione d'autore; storie incomplete in presenza di import o modifica diretta SQLite; differenze binarie/galleria non recuperabili; conflitto con scheda aperta da tempo; aumento dei tempi di salvataggio; versioni e formati futuri.

**Complessità indicativa:** **media** per cronologia dei soli metadati delle opere in lettura, **medio-alta** per ripristino selettivo sicuro, **alta** per estensione a clienti, vendita/media e strumenti di collaborazione. Priorità **P2 proposta**, subordinata a riscontri professionali, non assegnata a nessuno.

## Collaudi e criteri di accettazione ipotetici

1. Salvare tre versioni di un'opera con Unicode, descrizioni lunghe e campi null: timeline ordinata, diff fedele, data UTC e nessuna versione spuria per salvataggio identico.
2. Dopo un errore SQLite, **né scheda né revisione** cambiano; senza nuova revisione l'edit non è confermato. Simulare chiusura improvvisa durante il salvataggio e riavvio.
3. Modifica concorrente da seconda finestra fra anteprima e conferma: il ripristino è bloccato con avviso e nuovo confronto, non sovrascrive il lavoro recente.
4. Un ripristino di titolo/descrizione crea **una nuova** revisione senza alterare le precedenti, i certificati emessi, le vendite, gli incassi e le percentuali economiche.
5. Ogni tentativo di riapplicare `status='available'` su opera venduta è bloccato dalle stesse regole dell'archivio attivo. Nessuna operazione imposta involontariamente `image_path` su un file non più presente.
6. Vecchio DB senza storia: etichetta «precedenti non disponibili», nessuna creazione di revisioni retrodatate. Backup attuale, ripristino e nuove revisioni verificati insieme.
7. Su 100, 1.000 e 10.000 revisioni (dati sintetici, numeri **di test** e non previsioni d'uso), misurare tempi, spazio e memoria; soglie prestazionali da decidere su misure, con paginazione stabile.
8. I dati sensibili non finiscono nei log, nei rapporti di errore o in esportazioni generiche; nessuna connessione di rete; localizzazione e accessibilità verificate in tutti i sistemi supportati.

## Fasi proposte, nessuna implementazione autorizzata

- **0 — Ricerca:** esempi anonimi di errori di catalogazione, numero modifiche e necessità di recupero; verificare che il bisogno non sia soddisfatto da backup + semplice annulla.
- **1 — Specifica e cronologia read-only:** lista campi ammessi, politica sulla conservazione, modellazione dati, visualizzazione dei cambi senza potere di ripristino.
- **2 — Recupero mirato:** anteprima, selezione di soli campi ammissibili, controllo della revisione corrente, nuova revisione atomica e test di conflitto.
- **3 — Solo dopo nuova valutazione:** eventuali metadati galleria, esportazione storia, schema aperto, estensione ad altre entità e criteri di cancellazione definitiva.

## Questioni aperte per revisione del proprietario

1. I professionisti correggono abbastanza spesso **schede esistenti** da giustificare una cronologia per record, anziché solo un «annulla»?
2. Servono solo descrizioni e attribuzioni oppure anche misure e ubicazione? Meglio **escludere sempre lo stato commerciale** dal recupero?
3. È utile la motivazione manuale dei cambiamenti? Come evitare di insinuare che si conosca l'identità di chi ha modificato i dati senza account?
4. Storico sempre attivo o opzionale per archivio? In caso di disattivazione, come dichiarare i periodi mancanti senza generare false certezze?
5. Come affrontare la cancellazione completa di dati personali se in futuro si estende la funzione ai collezionisti?
6. Se un backup vecchio viene ripristinato, interessa confrontarlo con revisioni presenti in un **altro** archivio o è preferibile non fondere mai storie?
7. È sufficiente la consultazione, oppure il recupero selettivo deve far parte del valore iniziale? Quale impatto massimo su dimensioni e velocità di salvataggio è accettabile?

---

**Esito: Proposta — da valutare.** Questo documento non approva una roadmap, non modifica il software e non dà mandato di sviluppo.