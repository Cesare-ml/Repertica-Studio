# REPS-TECH-030 — Vocabolari locali facoltativi per tecniche e supporti, senza perdere i termini originali

- **ID stabile:** REPS-TECH-030
- **Data della ricognizione:** 2026-11-22
- **Stato:** **Proposta — da valutare**
- **Filone:** Strumenti e architettura
- **Priorità proposta:** **P2–P3, ipotetica**; P2 soltanto se utenti con cataloghi eterogenei confermano problemi di coerenza/interoperabilità
- **Natura:** studio esclusivamente documentale. Nessuna tassonomia, migrazione, API, dipendenza o nuova schermata è approvata o assegnata.

## 1. Problema, destinatari e riscontri verificati

Un archivio potrebbe registrare la stessa tecnica come «olio», «pittura a olio», «oil painting» o «olio su tela». Ma **«olio su tela» contiene anche un'informazione sul supporto**: non è corretto equiparare automaticamente le quattro espressioni. La grafia libera è utile agli artisti, mentre classificazioni incoerenti rendono meno affidabili raggruppamenti e scambi fra archivi.

**Riscontri sul `main` del 22/11/2026:** [schema SQLite](../../../src-tauri/src/db.rs) salva `artworks.technique` e `artworks.support` come `TEXT`; [modelli Angular](../../../src/app/models.ts) e [modelli Rust](../../../src-tauri/src/models.rs) li trattano come stringhe facoltative, non come riferimenti a concetti; [ricerca archivio](../../../src/app/archive-filters.ts) cerca nelle stringhe, normalizzando accenti e maiuscole. [Portabilità](../../DATA-PORTABILITY.md) esporta entrambi i campi originali nel CSV `artworks`, senza un identificatore semantico condiviso. Nel [Media Center](../../../src/app/api.ts) esistono già **tag per le immagini**: non sono, per ciò solo, un vocabolario di materiali o tecniche delle opere.

**Destinatari possibili:** gallerie con cataloghi redatti da più persone, archivi che importano elenchi multilingue, artisti con descrizioni storiche disomogenee. In archivi piccoli o deliberatamente non classificati, non serve alcuna funzione supplementare. Non è emersa una segnalazione di bug né è stata misurata la frequenza del bisogno.

### Requisiti certi, ipotesi, limiti

- **Certo dal prodotto/documentazione:** testi originali delle opere, ricerca locale, CSV e backup devono rimanere accessibili; sistema gratuito, offline-first, account non richiesti, lingua OS con fallback inglese; certificati storici e vendite non devono essere reinterpretati retroattivamente.
- **Ipotesi da verificare:** la differenza tra grafie è davvero frequente; una mappatura semantica facoltativa è più utile di una semplice lista di suggerimenti; gli utenti desiderano vocabolari condivisibili tra archivi.
- **Da non assumere:** due parole simili descrivono lo stesso materiale; ogni tecnica ha una traduzione univoca; un vocabolario esterno è autorevole, aggiornato o libero da vincoli di riuso.

## 2. Confini anti-doppione

- **REPS-TECH-008** progetta filtri/viste salvate, **non** l'identità dei termini; **REPS-TECH-019** studia import CSV/TSV con mapping di *colonne*, non la scelta motivata dei significati dei valori; **REPS-TECH-001** tratta portabilità dell'intero archivio, non un modello terminologico locale; **REPS-TECH-024** legge IPTC/XMP delle *fotografie*, non tecnica e supporto delle *opere*.
- **REPS-UX-034** nella [PR Gestione e UX #14](https://github.com/Cesare-ml/Repertica-Studio/pull/14) descrive *l'analisi delle vendite per tecnica/supporto*, che oggi può raggruppare testi letterali. Questa scheda è **solo l'eventuale infrastruttura di corrispondenze semantiche**, utilizzabile anche da ricerca/esportazione; non propone una seconda dashboard, metriche economiche o modifica di transazioni. La necessità può essere valutata con UX-034 senza modificare i suoi file.
- **REPS-UX-018** riguarda creazione di molte schede e riuso di campi; non si devono confondere i suggerimenti in compilazione con la revisione consapevole delle classificazioni storiche. **REPS-UX-031** distingue autorialità e attribuzione; non si intende normalizzare nomi di artisti o dichiarazioni di autenticità.
- Nessun catalogo remoto, abbonamento, IA, OCR, classificazione automatica delle immagini, arricchimento da terzi o conversione silenziosa dei dati esistenti.

## 3. Scenari d'uso da validare

1. **Due denominazioni effettivamente equivalenti:** «acquerello» e «watercolour» vengono proposte come possibili termini collegabili allo stesso concetto *solo dopo scelta dell'utente*. Entrambi i testi originali restano visibili nelle rispettive schede e nel CSV attuale.
2. **Espressione composita:** «olio su tela» non deve diventare automaticamente `tecnica=olio` e `supporto=tela`. La UI può segnalare un possibile campo composito e offrire *Mantieni originale*, *Collega soltanto una categoria*, *Correggi manualmente i campi*; nessuna azione implicita.
3. **Termini incerti o locali:** «tecnica mista», «sperimentale», nomi di materiali regionali possono rimanere non mappati; «non classificato» non equivale a «sconosciuto» né a `NULL`. L'utente può creare termini personali.
4. **Consegna a un archivio estero:** un'esportazione *aggiuntiva* può includere testo originale, identificativo locale del concetto, etichetta in lingua scelta ed eventuale URI esterno dichiarato; chi riceve non deve confondere l'UUID locale con uno standard internazionale.
5. **Cambio di lingua:** un'opera con valore originale «pittura a olio» non cambia campo quando il sistema passa da italiano a inglese. Si traduce la UI e, se disponibile, l'etichetta di navigazione; altrimenti mostrare il testo originale, mai inventare una traduzione.
6. **Vendite e certificati storici:** una nuova corrispondenza influenza eventualmente una **vista aggregata esplicitamente etichettata come classificazione corrente**. Non riscrive gli snapshot dei certificati, importi o descrizioni storiche.

## 4. Flussi UX proposti

**Percorso minimo, da preferire se sufficiente:** in una vista secondaria «Qualità dati», l'utente vede un elenco dei valori distinti per `technique` e `support`, numero di opere che li usano e presenza di grafie molto simili. Può consultare le schede originali, ma il sistema **non** applica né suggerisce un'unificazione automatica irreversibile. Una semplice esportazione CSV di questo riepilogo potrebbe esaurire il bisogno.

**Solo se confermato, collegamento facoltativo:**
1. `Impostazioni → Catalogo → Termini e corrispondenze`: separare **Tecniche** e **Supporti**, mostrare sempre valore originario e opere coinvolte.
2. Selezionare un termine locale o crearne uno. L'interfaccia propone collegamenti **possibili**, senza affermarne l'equivalenza; l'utente confronta valore, campo, esempi e quantità di schede.
3. Anteprima «Collega senza modificare i testi» con elenco delle opere interessate; annullamento e conferma. Per modificare *i testi*, occorre un flusso separato con conferma e salvaguardie sulle opere con documenti storici.
4. Ricerca avanzata opzionale: «Cerca anche varianti collegate» visibilmente distinto dalla ricerca testuale ordinaria; risultati accompagnati dall'indicazione «corrispondenza tramite vocabolario».
5. Esportazione facoltativa di tabella dei termini e associazioni, con anteprima e scelta di percorso. L'export CSV integrale già presente non cambia senza una decisione di versione del contratto.
6. Rimozione di un collegamento: mantiene i testi delle opere, non elimina immagini, vendite o certificati; conferma differenza tra **scollegare** e **cancellare il termine inutilizzato**.

**Accessibilità:** controllo completo da tastiera, confronti non basati solo sul colore, differenze leggibili da screen reader, filtri per «non mappato», focus preservato e conferme annullabili; etichette comprensibili al 200% di zoom. Una anteprima affiancata deve poter essere letta anche in sequenza lineare.

## 5. Specifica descrittiva e ipotesi di dati/API

**Opzione A (senza modificare le schede):** registro locale di corrispondenze, applicato *solo in lettura* ad analisi e ricerche. È l'opzione da valutare per prima.

Schema puramente concettuale, non SQL approvato:

```text
VocabularyTerm {
  id: UUID locale, kind: "TECHNIQUE" | "SUPPORT",
  canonical_label: string, labels_by_locale?: map<BCP47,string>,
  external_uri?: string, external_source?: string,
  provenance_note?: string, created_at, updated_at
}
ValueMapping {
  id: UUID locale, kind: "TECHNIQUE" | "SUPPORT",
  exact_source_value: string, term_id: UUID,
  confirmed_by_user_at: datetime, note?: string
}
```

La chiave `kind + exact_source_value` distingue grafie e contesti; collazioni per suggerimento non sono vincoli di equivalenza. Il `term_id` locale non prova un'identità fra archivi. Non collegare automaticamente campi diversi. Al limite, una mappatura per *singola opera* potrebbe essere necessaria per omonimie: va validata prima di fissare lo schema.

API **solo indicative**: `list_distinct_artwork_values({kind})`, `preview_mapping({kind, values, term_id})`, `confirm_mapping({preview_id})`, `remove_mapping({id})`, `export_vocabulary({target_path})`. Il backend dovrebbe applicare controlli su tipo, dimensione, versioni e cambi concorrenti; la previsualizzazione scade se l'archivio cambia. Nessuna query SQL o comando file arbitrario esposto a client MCP. Nessuna API è implementata da questo documento.

**Compatibilità:** preservare per default `artworks.technique/support` come testo originale; `NULL` e stringhe vuote non vanno convertiti tra loro senza consenso; mantenere i 12 CSV correnti, i riferimenti UUID e i backup pieni. Se in futuro il modello acquisisse una FK al termine, occorrerebbero migrazione non distruttiva, rollback collaudato e uno schema di esportazione **versionato**. Report statistici devono indicare se stanno raggruppando **testo grezzo** o **termine collegato**, e non fondere due gruppi in modo opaco.

## 6. Offline, sicurezza, privacy, preferenze e multilingua

Tutto su SQLite locale e file scelti dall'utente, anche senza Internet. **Nessuna interrogazione automatica di Getty AAT, Wikidata o altri dizionari**: un URI può essere soltanto una citazione descrittiva, non verifica di correttezza. Un eventuale pacchetto di termini importato dall'utente deve essere validato per dimensioni, schema, profondità, codifica, caratteri di controllo e URI non sicuri; non deve contenere codice eseguibile né fetch remoti. Verificare licenze e obblighi di attribuzione prima di redistribuire dati di terzi.

Le corrispondenze possono rivelare vocabolario o pratiche di uno studio: per default sono **private nell'archivio**. Niente telemetria, sincronizzazione, autenticazione, ricerca su servizi cloud o trasferimento verso client MCP. Evitare che descrizioni/libere etichette siano interpretate come istruzioni o contenuti HTML; proteggere da formula injection anche l'eventuale CSV aggiuntivo. Import/export con anteprima, nessuna sovrascrittura silenziosa dei file, avviso che un pacchetto condiviso può rivelare nomi e terminologia proprietaria.

**Impostazioni ipotetiche:** modalità solo testo (predefinita), abilita suggerimenti locali, visualizza raggruppamenti collegati, lingua di visualizzazione e reset delle sole corrispondenze. La lingua UI segue il sistema, fallback **inglese**, override manuale coerente con l'app. Le etichette libere di catalogazione non si traducono in automatico; per una traduzione mancante mostrare il termine originario e un indicatore discreto. Gli identificatori di esportazione/API rimangono stabili e non localizzati.

## 7. Dipendenze, alternative, rischi e complessità

**Dipendenze concettuali:** definizione della semantica di «tecnica» e «supporto», dataset di prova realistici e autorizzati, eventuale schema SQLite aggiuntivo e confronto con il formato portabile TECH-001. Nessuna dipendenza software viene scelta o modificata qui.

**Alternative:**
- Non intervenire: testo libero, ricerca attuale, CSV e filtri esistenti.
- Aggiungere soltanto suggerimenti dai valori già presenti, **senza** mappatura semantica persistente.
- Esportare i valori distinti e normalizzarli manualmente in Calc/Excel; costo di sviluppo minimo, ma risultati esterni all'archivio.
- Usare un vocabolario esterno imposto: **sconsigliabile senza verifica** di copertura artistica, lingua, licenza e capacità di annotare termini personali.

**Rischi:** appiattimento delle differenze artistiche; classificazioni sbagliate con etichette simili; eccessiva complessità per il singolo artista; divergenza tra testo originale e vista raggruppata; traduzioni improprie; chiavi locali erroneamente considerate globali; migrazioni di schema e compatibilità export; prestazioni su cataloghi grandi; costi di manutenzione dei dizionari. **Complessità bassa** per un semplice rapporto di valori distinti, **media** per mappature opt-in solo in lettura, **alta** per vocabolari condivisi multilingue e import/export semantico rigoroso.

## 8. Criteri di accettazione e prove future (non implementate)

1. Dati sintetici con «olio», «olio su tela», «oil painting», supporto «tela» e valori `NULL`: nessun raggruppamento avviene senza scelta dichiarata, e i campi originali restano byte-per-byte uguali nel round-trip già supportato.
2. Il collegamento `acquerello` ↔ `watercolour` produce un raggruppamento **solo nella modalità facoltativa**; con modalità disabilitata, risultati e CSV attuali rimangono immutati.
3. Distingue etichette ripetute in Tecnica/Supporto, collisioni in lingua e voce inesistente; nessun `NULL` convertito automaticamente.
4. Dopo annullamento, errore, disco pieno, crash o ripristino, non rimangono mappature parziali; con backup completo e ripristino si ottengono le stesse associazioni **se e solo se** deliberate nel formato compatibile.
5. Certificati emessi e rendiconti di vendite/incassi restano invariati. Le analisi future dichiarano «dati correnti» e modalità di raggruppamento; nessuna conclusione causale o autenticazione implicita.
6. JSON/CSV di esempio con caratteri Unicode, separatori, etichette lunghe, formule da spreadsheet, URI sbagliati e codifiche invalide: validazione, rifiuto sicuro e report locale leggibile.
7. Test manuali Windows/macOS/Linux, offline e su archivio numeroso; lingua sistema IT/EN/ES/FR/DE, fallback EN e screen reader/tastiera; nessuna chiamata di rete.
8. Migrazione da database senza vocaboli: tutte le opere già esistenti restano leggibili; rollback, aggiornamenti futuri e cancellazione delle mappature non cancellano i testi.

## 9. Fasi possibili e priorità suggerita

- **Fase 0 — validazione:** raccogliere esempi anonimizzati di 20–50 valori realmente conflittuali da almeno due cataloghi; capire se sono varianti linguistiche, concetti diversi o casi compositi. Valutare se bastano filtri e CSV.
- **Fase 1 — solo analisi:** elenco locale di valori distinti con frequenze e link alle opere, nessuna riscrittura/migrazione.
- **Fase 2 — solo se richiesta:** termini locali e associazioni esplicite, reversible, con modalità disabilitabile e viste chiaramente etichettate.
- **Fase 3 — eventuale:** esportazione/importazione di un vocabolario *aperto e versionato*, profili multilingua e URI bibliografici verificati, previa revisione di licenze e interoperabilità.

**Priorità P2–P3 solo ipotetica**. Per un archivio piccolo, P3 o nessuna implementazione è un esito valido.

## 10. Questioni per la revisione del proprietario

1. Esistono cataloghi reali con grafie che impediscono oggi analisi o ricerche utili? Quante opere e quanti collaboratori coinvolgono?
2. Serve davvero il concetto condiviso, oppure è sufficiente mostrare suggerimenti tratti dai valori già registrati?
3. In presenza di «olio su tela», bisogna mantenere un unico campo composito o favorire correzioni manuali distinte di tecnica/supporto?
4. Le traduzioni delle etichette devono essere redatte e controllate dall'utente, o è preferibile mantenere solo i termini originali?
5. Ha valore portare le associazioni tra archivi senza imporre identificativi globali? Quali licenze di vocabolari esterni sarebbero accettabili?
6. Il livello di complessità è proporzionato rispetto ai bisogni delle gallerie e alla semplicità desiderata per l'artista singolo?

**Esito documentale:** **Proposta — da valutare**. Nessuna decisione, test, codice, API, dipendenza, impostazione o workflow modificati. Nessun documento `docs/idee/gestione-e-ux/` toccato. PR tecnica [#16](https://github.com/Cesare-ml/Repertica-Studio/pull/16) da mantenere in bozza e senza merge.
