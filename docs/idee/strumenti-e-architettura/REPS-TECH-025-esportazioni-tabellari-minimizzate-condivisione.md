# REPS-TECH-025 — Esportazioni tabellari minimizzate per collaborazione senza divulgazione involontaria

- **ID stabile:** REPS-TECH-025
- **Data:** 2026-11-21
- **Stato:** **Proposta — da valutare**
- **Priorità proposta:** **P2**, da discutere dopo interviste con artisti, gallerie, archivi e consulenti; non approvata
- **Filone:** Strumenti e architettura, esclusivamente studio documentale
- **Decisione:** nessuna funzionalità approvata, nessuna implementazione autorizzata

## 1. Problema, evidenze e destinatari

Un professionista può voler consegnare a una galleria un elenco di opere, a un curatore una tabella da importare nel suo catalogo o a un collaboratore una sintesi di inventario **senza consegnargli l'archivio completo**. Un CSV apparentemente innocuo può invece contenere ubicazioni, nomi di acquirenti, importi, note interne, identificativi che consentono di incrociare altri registri e percorsi di file. La semplice esclusione delle colonne «telefono» e «email» non rende anonimo il dataset.

**Fatti osservati in `main` il 21/11/2026:**

- [`docs/DATA-PORTABILITY.md`](../../DATA-PORTABILITY.md) descrive **12 registri CSV integrali**, indipendenti dai filtri della schermata; quello `collectors` include dati anagrafici, contatti e note private; `artworks` include ubicazione e percorso relativo della copertina; `certificates` contiene lo `snapshot_json` storico completo; `installments`, `sales`, `balances`, `receipts` e altri registri possono contenere nomi, riferimenti commerciali e cifre riservate.
- [`src-tauri/src/export.rs`](../../../src-tauri/src/export.rs) definisce query e intestazioni fisse per ogni registro, con scrittura locale `create_new`, protezione dalle formule da foglio elettronico e divieto di salvataggio nella cartella dell'archivio. Non emerge un comando distinto per produrre **una proiezione di colonne e righe destinata a soggetti esterni**.
- I backup completi e i CSV integrali sono già funzioni importanti e **non vanno ridotti, mascherati né sostituiti**. Una copia minimizzata non è un backup, né un formato di ripristino.

Questa è **una opportunità di ridurre il rischio in una condivisione volontaria**, non una segnalazione di fuga di dati o incidente già verificato. Occorre verificare con professionisti quanto spesso scambino tabelle e con quali destinatari.

**Destinatari:** studi d'artista, archivi, gallerie, collaboratori incaricati di catalogazione, curatori e consulenti. Non è una funzione per pubblicare automaticamente dati su Internet.

## 2. Scenari d'uso

1. **Catalogo per mostra:** selezionare venti opere e ottenere una tabella con codice, titolo, tecnica, anno e dimensioni, senza ubicazione interna, prezzo di vendita, nomi dei collezionisti o percorsi dei file.
2. **Inventario per interlocutore esterno:** produrre una lista da verificare insieme a un restauratore o a una galleria, mostrando solo i campi concordati; gli stati commerciali restano opzionali e chiaramente identificati.
3. **Analisi economica per consulente:** se in futuro realmente necessaria, esportare somme aggregate per periodo **senza righe identificative di vendita** e senza definirle impropriamente anonime; aggregati con pochi casi potrebbero comunque rivelare transazioni.
4. **Caso negativo:** una descrizione libera contiene numero di telefono o un riferimento personale. La funzione non deve attestare che tale testo sia sicuro solo perché la colonna non è etichettata «privata».

## 3. Specifica funzionale e flussi UX proposti

### Flusso A — profilo conservativo per opere (prima fase)

- Dalla schermata Opere, comando **«Esporta tabella per condivisione»**, ben distinto da **«Esporta registro completo (CSV)»** e da **Backup**.
- Scegliere esplicitamente **tutte le opere** oppure **la selezione corrente**. Non usare silenziosamente i filtri correnti come se corrispondessero a una selezione verificata: visualizzare numero di righe, criteri e data di estrazione.
- Profilo iniziale a **lista consentita**: codice, titolo, anno, tecnica/supporto e misure, con eventuali campi aggiuntivi abilitati uno per uno. Per default escludere ubicazione, descrizioni libere, foto/RAW, percorsi, identificativi relazionali non necessari, contatti, vendite, incassi, certificati, costi e note.
- Anteprima leggibile delle **colonne effettive**, delle prime righe e del numero totale, con un avviso sui campi liberi/identificativi selezionati. Mostrare il nome del profilo, la sua versione e i dati esclusi. Non limitarsi a nascondere colonne nel frontend: il file deve essere generato sul backend usando una allowlist.
- Confermare e scegliere file e cartella con dialogo nativo (anche USB/NAS montato). Non inviare, caricare, spedire o aprire automaticamente. Non sovrascrivere file già presenti e non scrivere dentro l'archivio attivo.
- Messaggio di completamento: **«Tabella esportata; non è un backup»**. La sicurezza della divulgazione resta una scelta consapevole dell'utente.

### Flusso B — revisione dei campi rischiosi (fase successiva, solo se utile)

Per un profilo avanzato, mostrare prima della conferma le categorie «identificativo riconducibile a un soggetto», «dato economico», «testo libero», «ubicazione», «riferimento a file» e «contenuto di terzi». L'inclusione di ciascuna categoria potenzialmente riservata deve richiedere un gesto separato, non un'impostazione globale nascosta.

Evitare una promessa di rilevamento automatico perfetto di dati personali nelle descrizioni. Possibile alternativa semplice: **escludere sempre i testi liberi** dai profili di condivisione predefiniti, lasciandoli ai registri integrali.

### Confine importante

Le **12 esportazioni integrali** continuano esattamente come oggi, per autonomia dell'utente e analisi esterna. Il nuovo flusso non modifica database, documenti, prezzi, stati, legami commerciali, impostazioni di backup, né produce dati sintetici che possano essere reimportati come autentici.

## 4. Dati, formati e API ipotetiche (solo descrittive)

La prima fase può essere **stateless**: un profilo incorporato nell'app, nessuna migrazione SQLite e nessuna cronologia di destinatari. Modello concettuale non vincolante:

```text
ExportProjectionV1 {
  profile_id: "artworks-share-minimal-v1",
  source_kind: "artworks",
  source_scope: { all: true } | { selected_artwork_ids: [...] },
  allowed_columns: ["artwork_code", "title", "year", "technique", "support",
                    "width_cm", "height_cm", "depth_cm"],
  output_format: "csv",
  purpose_label: "external-table",
  version: 1
}

previewShareExport(profile, scope) ->
  { row_count, column_names, warnings, sample_rows_redacted }

exportShareProjection(profile, scope, destination) ->
  { created_path, row_count, profile_version, warnings }
```

**Regole:** il backend Rust risolve solo ID appartenenti all'archivio attivo, verifica nuovamente la selezione al momento della generazione e applica una allowlist indipendente dal payload Angular. Non accettare nomi di tabella, espressioni SQL, percorsi relativi arbitrari o colonne inventate dal frontend. Per dataset mutabili, scegliere un unico snapshot SQLite coerente; la differenza tra anteprima e salvataggio va resa esplicita, eventualmente con conferma se il numero di righe cambia.

CSV standard UTF-8 e intestazioni stabili in inglese/codici macchina; delimitatori e quote compatibili con quanto già documentato, importi in unità esplicite se mai inclusi, difesa anti-formula identica o superiore ai CSV integrali. **Niente snapshot JSON, path dei media o byte di allegati** nell'esportazione minima. Un eventuale file descrittivo con schema e profilo sarebbe facoltativo, anch'esso privo di dati privati e non chiamato «certificato di anonimizzazione».

**Identificativi:** valutare se per il profilo esterno mantenere il codice d'archivio (utile per riconciliare le opere, ma potenzialmente riconoscibile) oppure emettere ID locali al file. UUID e chiavi collezionista/vendita vanno esclusi di default. Non promettere «anonimizzazione»: persino un titolo molto noto può identificare il proprietario di un'opera se incrociato con altre fonti.

## 5. Offline, sicurezza, privacy, interoperabilità

- **Sempre offline**, senza account, abbonamento, telemetria, cloud o contatto con destinatari. L'utente sceglie liberamente cartella e formato; il NAS è solo un filesystem montato, non un servizio integrato.
- **Minimizzazione per default**, nessuna raccolta di destinatari, nessun log con righe, nomi o valori. Il programma può conservare solo preferenze innocue e locali, se serve; nessun export pianificato senza autorizzazione.
- **Nessuna garanzia di anonimato**, di conformità giuridica automatica o di assenza di dati sensibili nei campi testuali. Avvisi proporzionati e traducibili, senza bloccare la libertà di esportare l'intero archivio.
- Trattare collisioni di nomi, symlink, cartelle condivise, permessi, fallimenti di scrittura, chiavette scollegate e file parziali come da valutare con **TECH-018**; non ripetere qui il disegno dello streaming.
- L'output deve aprirsi in LibreOffice, Excel e strumenti CSV indipendenti, senza Repertica. I valori vuoti e i decimali devono restare documentati. La proposta **non** è un nuovo protocollo di sincronizzazione o importazione.
- Eventuali pacchetti con immagini sono **fuori dalla prima fase**: richiederebbero la sanificazione fotografica specifica di **TECH-013**, e i permessi di riproduzione dichiarati in **UX-010**.

## 6. Impostazioni, multilingua e accessibilità

- **Lingua UI** dal sistema operativo, fallback in inglese, selezione manuale ove presente; messaggi e profili traducibili in IT/EN/ES/FR/DE senza tradurre codici colonna o testi liberi.
- Eventuale preferenza di ultimo formato o cartella solo locale e facoltativa; **mai** memorizzare come impostazione globale un profilo «mostra tutti i dati riservati». Ogni export sensibile richiede riconferma.
- Tastiera completa, etichette per screen reader, tabella anteprima navigabile, annunci del conteggio di righe e dei warning, focus coerente. L'esclusione/inclusione dei campi non deve dipendere dal solo colore.
- Chiarire in tutte le lingue la distinzione tra **«registro completo»**, **«tabella minimizzata»** e **«backup verificato»**.

## 7. Dipendenze, alternative, rischi e complessità

**Dipendenze concettuali:** tabelle e controlli di `export.rs`, API Tauri tipizzate, snapshot SQLite, selezioni UI e i18n; test nativi. Nessuna nuova dipendenza o libreria viene installata con questa proposta.

**Alternative valutabili:**

1. **Restare ai CSV integrali** e aggiungere solo un avviso privacy: costo minimo, ma chi riceve il file deve rimuovere manualmente i campi superflui; il rischio umano rimane.
2. **Esportare una sola lista essenziale delle opere**, senza profili personalizzati: soluzione probabilmente più adatta come prima fase e più semplice da verificare.
3. Usare un foglio elettronico esterno per creare una copia privata: interoperabile ma può lasciare dati nascosti, cronologie o collegamenti nel documento.
4. **UX-007 dossier** quando l'obiettivo è una presentazione impaginata: non serve un nuovo esportatore tecnico per quel caso.

**Rischi:** falsa fiducia nell'«anonimato»; dati sensibili inseriti in titolo/codice/testi liberi; ridentificazione tramite UUID, titoli, rare date/importi; profili predefiniti troppo ampi; traduzione che altera intestazioni; perdita di riferimenti fra tabelle; comparsa di dati nuovi dopo l'anteprima; crash e file parziali. **Complessità stimata:** bassa–media per una sola proiezione di opere; media–alta per selezioni, template persistenti, output multi-registro e statistiche con mitigazione del rischio di ridentificazione. Non introdurre questi ultimi senza domanda documentata.

## 8. Criteri di accettazione per un'eventuale implementazione futura

1. I 12 CSV integrali e il backup funzionano **come prima**, senza filtri impliciti, omissioni o cambi di schema.
2. Esportazione minima delle opere: in output solo le colonne allowlist, nell'ordine e con la codifica previsti; nessun campo privato nascosto in metadati, nomi o intestazioni.
3. Dataset di prova con contatti, ubicazioni, UUID, note e snapshot di certificati contenenti stringhe «sentinella»: **nessuna sentinella riservata** deve apparire nel file minimo.
4. Test specifici con titoli/codici contenenti nomi e dati personali: avviso sul **limite della minimizzazione**, senza falsa attestazione di anonimato.
5. Selezione di zero, una, molte e tutte le opere; concorrenza anteprima/esportazione; righe/colonne e ordine deterministici; corretta gestione di testo Unicode, delimitatori, multilinea e formule.
6. Ripetere le prove su Windows/macOS/Linux e cartelle locali/USB/NAS: no overwrite di file esistenti, comportamento comprensibile se volume non disponibile, nessuna modifica all'archivio.
7. Nessuna rete, scrittura di impostazioni sensibili, log contenente dati personali o pubblicazione automatica. Verifica con tastiera e screen reader.
8. Una copia minimizzata non può essere scambiata con un backup completo o un import canonico; naming e messaggi devono indicarlo.

## 9. Fasi ipotetiche, priorità e questioni per il proprietario

- **Fase 0 — Validazione:** interviste e 3–5 esempi reali di scambi tabellari; inventario dei campi che i professionisti devono effettivamente condividere. Senza bisogno confermato, **non sviluppare**.
- **Fase 1 — Minimo verificabile:** un solo profilo conservativo per tabella opere, preview e scelta cartella, senza configurazione persistente; test con sentinelle e casi di ridentificazione.
- **Fase 2 — Solo se richiesta:** altre proiezioni coerenti e salvabili, previa revisione della privacy e del contratto CSV; evitare un generatore SQL universale.
- **Fase 3 — Eventuale:** export aggregati non identificativi, se sono documentate esigenze di consulenti e regole di soglia; non promettere anonimato dei piccoli gruppi.

**Domande aperte al proprietario (non decisioni):** si scambiano davvero più spesso CSV che dossier PDF? Quali campi è indispensabile inviare a una galleria? Codice dell'opera pubblico o identificativo temporaneo? Serve includere descrizioni libere, accettando il rischio? La prima fase deve riguardare solo opere o anche un inventario economico aggregato? Sono accettabili profili non persistenti?

**Distinzione rispetto alle proposte esistenti:** **TECH-001** è la portabilità completa, reimportabile e senza lock-in; **TECH-018** è la robustezza delle esportazioni integrali; **TECH-019** importa tabelle esterne; **TECH-013** tratta i metadati delle *immagini*; **TECH-008** definisce viste e filtri interni; **UX-007** produce dossier impaginati, **UX-010** tratta diritti e permessi. Qui si studia esclusivamente un **output tabellare deliberatamente ridotto e controllabile**, a beneficio della collaborazione professionale. Nessuno di questi file viene modificato.

**Stato finale:** **Proposta — da valutare**. Non è una roadmap, non autorizza codice, issue, dipendenze, merge o incarichi. Revisione personale del proprietario nella PR documentale.
