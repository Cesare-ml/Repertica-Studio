# REPS-TECH-024 — Metadati fotografici IPTC/XMP interoperabili, con importazione consapevole e originali intatti

- **ID stabile:** REPS-TECH-024
- **Data della ricognizione:** 2026-11-21
- **Stato:** **Proposta — da valutare**
- **Filone:** Strumenti e architettura
- **Priorità proposta:** **P2**, solo ipotetica e condizionata a riscontri di fotografi d'arte, artisti, archivi e gallerie
- **Natura:** ricerca e specifica funzionale *esclusivamente documentale*: non approva API, schema, librerie, dipendenze o implementazione

## 1. Problema, destinatari ed evidenze

Chi documenta opere spesso riceve JPEG/TIFF/RAW preparati con altri strumenti fotografici. Fotografo, credit line, titolarità dichiarata dei diritti, descrizione e altre informazioni possono essere già presenti nei metadati **IPTC/XMP** del file o in un sidecar. Ricopiarli a mano nel Media Center introduce attrito e ambiguità; esportare immagini destinate a un professionista senza poter scegliere un corredo interoperabile di metadati può obbligare a reimmissioni nel software destinatario.

**Distinzione essenziale:** il creatore della **fotografia**, il creatore dell'**opera raffigurata**, il titolare dei diritti sulla fotografia e quello dei diritti sull'opera sono ruoli diversi. I dati incorporati nel file sono **dichiarazioni della fonte**, non una verifica giuridica o di autenticità. I metadati possono essere errati, superati o sensibili.

**Evidenze verificate su `main` il 2026-11-21:**

- `src-tauri/src/media.rs` conserva `title`, `description`, `alt_text`, `credits`, `copyright_status`, tag e revisioni fotografiche. `MediaMetadataInput` riceve i valori dichiarati dall'utente; il percorso `import_core` importa file locali, senza mappatura IPTC/XMP evidente nei metodi esaminati.
- `src-tauri/src/imaging.rs` applica l'orientamento EXIF nel *decoding*, ma ciò **non equivale** a leggere e conservare l'intero profilo dei metadati fotografici. I derivati ricodificati con l'encoder immagini non dichiarano un trasferimento verificato dei campi IPTC/XMP.
- `src-tauri/src/raw_files.rs` registra un RAW per opera con dimensione e hash: non emerge nel percorso analizzato un'interfaccia per gestire sidecar XMP abbinati a quel RAW.
- `docs/DATA-PORTABILITY.md` descrive **dodici registri CSV**, con riferimenti ai media e non byte, e backup completi SQLite + `media/`; non documenta un interscambio dei metadati IPTC/XMP con editor fotografici.
- La ricerca di codice `xmp` sul branch principale non ha restituito una funzione dedicata; **questa ricognizione statica non prova** che tutti i formati o i programmi esterni siano incompatibili, né che esista già un problema lamentato dagli utenti.

**Destinatari:** fotografi d'arte che cedono immagini con crediti, artisti che ricevono foto da professionisti, archivi che riordinano nuclei fotografici, gallerie che preparano materiale per cataloghi. Per chi usa solo foto personali senza metadati, nessuna nuova interfaccia obbligatoria.

## 2. Confini e confronto con proposte esistenti

- [TECH-013](REPS-TECH-013-copie-fotografiche-condivisibili-privacy-metadati.md) affronta soprattutto **rimozione/controllo di metadati sensibili nelle copie da condividere**; questa scheda riguarda **lettura, provenienza, mappatura consapevole e uscita interoperabile** dei soli campi selezionati. In un futuro prodotto le due funzioni dovrebbero usare una **medesima politica di export**, non produrre due pulsanti contraddittori.
- [TECH-021](REPS-TECH-021-gestione-colore-icc-derivati-fotografici.md) riguarda profili colore e fedeltà cromatica; **ICC non è IPTC/XMP**. Qui non si propongono conversioni cromatiche.
- [TECH-001](REPS-TECH-001-pacchetto-dati-aperto.md) progetta un **pacchetto dell'archivio**; [TECH-019](REPS-TECH-019-importazione-guidata-cataloghi-tabellari.md) importa **cataloghi CSV/TSV**; questa scheda scambia **metadati incorporati o sidecar di singole fotografie**, senza creare opere o importare interi cataloghi.
- [TECH-016](REPS-TECH-016-allegati-documentali-locali-multi-entita.md) progetta documenti collegabili: un sidecar di metadati **non** diventa un allegato generico né una nuova voce contabile.
- [UX-010 nella PR documentale #14](https://github.com/Cesare-ml/Repertica-Studio/blob/docs/idee-gestione-ux-20261121/docs/idee/gestione-e-ux/REPS-UX-010-diritti-riproduzione-crediti-autorizzazioni.md) studia il **processo professionale** di autorizzazione alla riproduzione. Un campo XMP non crea licenze, contratti o autorizzazioni.
- L'eventuale [PR #58](https://github.com/Cesare-ml/Repertica-Studio/pull/58) sui **registri CSV delle versioni media** è un altro formato d'uscita: verificare il suo stato prima di ogni futura decisione per evitare sovrapposizioni di colonne e semantiche.

## 3. Scenari reali da verificare

1. **Fotografo esterno:** un JPEG contiene fotografo = «Studio Alfa», credit line = «© Studio Alfa», descrizione della ripresa e copyright dichiarato. L'artista importa la foto e può **vedere** tali informazioni prima di adottarle nel Media Center. Nessun campo già compilato viene sovrascritto per default; non viene dedotto chi possiede i diritti dell'opera.
2. **Contrasto fra dati:** la scheda media indica un credito diverso dall'XMP incorporato. La UI mostra affiancati «valore in archivio» e «valore nel file», con fonte, e offre **Mantieni**, **Sostituisci esplicitamente**, **Ignora**. Nessuna risoluzione automatica.
3. **Catalogo consegnato a una galleria:** l'utente esporta *una copia* con un insieme minimo di campi fotografici autorizzati oppure un sidecar accompagnatorio; vede una **anteprima esatta** dei valori trasferiti e un avviso sulla possibile perdita di metadati nel software ricevente. La condivisione non avviene in automatico.
4. **Fotografia scattata nello studio:** il file contiene coordinate GPS, seriale dell'apparecchio o dati personali. Questi restano **esclusi dai metadati proposti per la condivisione**; una richiesta di conservarli deve essere separata dalla richiesta di pubblicarli.
5. **RAW con sidecar:** l'utente porta un RAW e un XMP associato. Un'eventuale fase avanzata deve verificare associazione esplicita, collisioni di nomi, disconnessione del disco e cambi del sidecar; non assumere che nomi uguali in cartelle diverse identifichino lo stesso scatto.

## 4. Flussi UX proposti, senza aggravare il flusso corrente

**Fase minima — ispezione read-only**

Media Center → fotografia → pannello facoltativo **«Metadati del file»** → campi riconosciuti, categoria e provenienza (`XMP`, `IPTC-IIM`, `EXIF`, eventuale sidecar) → sezione «Dati sensibili non proposti» → chiusura senza modifiche. Se il file non contiene campi supportati, messaggio chiaro senza falsi valori precompilati. Avviso distinguibile da un errore se un blocco non è leggibile.

**Fase successiva — adozione selettiva**

«Confronta con i dati della libreria» → tabella campo per campo → selezione esplicita (nessuna selezione per default quando il campo interno è già valorizzato) → anteprima delle modifiche → conferma → aggiornamento della sola scheda media. Conservare identità del file/revisione usati per l'anteprima e invalidare l'operazione se nel frattempo cambiano.

**Fase opzionale — esportazione interoperabile**

Media Center → **Esporta copia** → scelta profilo **«Solo immagine»** / **«Crediti essenziali verificati dall'utente»** / **«Sidecar XMP selezionato»** (nomi illustrativi) → anteprima dei valori e del nome dei file → cartella scelta dall'utente → creazione senza sovrascrittura. Separare un sidecar leggibile dal risultato grafico: nessun effetto implicito sul JPEG, sul RAW o su derivati esistenti.

**Accessibilità:** etichette testuali della provenienza e del conflitto, tabella percorribile da tastiera, focus restituito dopo chiusura, anteprima leggibile da screen reader; testo e stato non affidati soltanto a colore/icona. Non far apparire dieci campi tecnici nel modulo principale di una nuova opera.

## 5. Ipotesi strutturali e API *solo descrittive*

Non è proposta una nuova tabella come scelta approvata. Possibile struttura concettuale, da confrontare con lettura al volo senza persistenza:

```text
PhotoMetadataObservation {
  media_id: UUID,
  media_revision_id: UUID,
  source_kind: "embedded_xmp" | "embedded_iptc" | "embedded_exif" | "user_selected_sidecar",
  source_sha256: string,            # identità della sorgente realmente letta
  observed_at_utc: timestamp,
  fields: [{ namespace, property, raw_value, normalized_value?, language?, warning? }],
  diagnostics: [warning_code]       # non incollare XML non fidato nell'interfaccia
}

PhotoMetadataAdoptionPreview {
  media_id, expected_media_revision_id,
  selected_mappings: [{ target_field, proposed_value, source_property }],
  conflicts: [{ target_field, current_value, proposed_value }],
  privacy_warnings: [warning_code]
}
```

**Mappatura iniziale da valutare:** `dc:creator` / IPTC Creator → **autore della fotografia** (non creatore dell'opera); `photoshop:Credit` → `credits` solo su conferma; `dc:description` → descrizione fotografia solo su conferma; `dc:rights` → avviso/nota di copyright della foto, **non** attribuzione automatica a `copyright_status=OWNED`. L'attuale schema del Media Center potrebbe non bastare per distinguere tutti questi ruoli: alternativa prudente è **sola visualizzazione** senza nuovi campi. La descrizione fotografica non è automaticamente `alt_text` accessibile.

**API descrittive, non un contratto:**
- `inspectPhotoMetadata(mediaId, revisionId) -> ObservationsAndWarnings`
- `previewPhotoMetadataAdoption(mediaId, expectedRevisionId, selections) -> ProposedDiff`
- `confirmPhotoMetadataAdoption(previewToken) -> MediaDetail`
- `exportSelectedPhotoMetadata(mediaId, profile, destination) -> ExportReport`

Preferire identificazione stabile tramite UUID e hash; non fare affidamento su percorso sorgente esterno o nomi di file. Se si conserva una rappresentazione originale XMP, decidere con cura limiti dimensionali, conservazione e modalità di cancellazione; in una fase minima può essere sufficiente **non memorizzarla**. Un eventuale sidecar generato dev'essere JSON/XML conforme allo standard scelto, mai un contenitore proprietario che prometta compatibilità inesistente.

## 6. Offline, privacy, sicurezza e integrità

- Tutto **offline**; nessun servizio IA, OCR, indicizzazione sul cloud, telemetria, account o abbonamento. La lettura avviene solo su file esplicitamente importati o selezionati; i sidecar su cartelle esterne **non** vanno scanditi automaticamente.
- Non modificare originali, RAW o snapshot storici dei certificati. Le eventuali revisioni esportate sono copie; conservarne l'identità e rendere evidente quale revisione è stata scelta.
- Trattare blocchi XMP/XML/IPTC come input **non fidato**: parser con limiti di dimensione/profondità, niente entità esterne/DTD attive, nessun accesso alla rete, rifiuto di percorsi `../`, di link simbolici imprevisti e di sidecar incongruenti; escapare il testo in UI e CSV.
- **Confidenzialità:** XMP/EXIF possono includere GPS, dati del dispositivo, persone, note riservate, numeri seriali. Anteprima dei campi divulgati, minimizzazione predefinita e niente inclusione automatica in dossier, MCP, export generale o copie condivisibili. La scritta di copyright dentro l'immagine **non** costituisce una licenza verificata.
- Le copie di backup verificate e le esportazioni complete devono restare autosufficienti: qualunque metadato adottato **nell'archivio** deve accompagnarlo; un sidecar conservato soltanto su una cartella esterna non può diventare indispensabile al restore. Se un nuovo file sidecar dovesse entrare in `media/`, aggiornare *in un futuro progetto separato* l'inventario/manifests e i controlli SHA-256, senza indebolire il ripristino.
- Prestazioni: ispezione su richiesta, preferire lettura dei soli segmenti metadata anziché decodifica multipla di RAW grandi; limiti e timeout; impossibilità di leggere un campo non deve bloccare consultazione e backup delle immagini.

## 7. Impostazioni, lingue, import/export e compatibilità

**Impostazioni ipotetiche:** politica predefinita «non importare né pubblicare metadati automaticamente»; profilo esportazione selezionabile *per ogni operazione*; preferenza locale per ricordare un profilo solo dopo opt-in; nessuna scelta nascosta di «scrivi i crediti nei file». I metadati originali non sono considerati dati approvati.

**Lingue:** lingua di sistema, fallback **inglese**, scelta esplicita disponibile; etichette e messaggi localizzabili in italiano, inglese, spagnolo, francese e tedesco. I namespace tecnici (`dc:`, `photoshop:`), codici, UUID e valori storici non si traducono. Le stringhe IPTC/XMP in altre lingue mantengono il proprio tag `xml:lang`, ove presente, e non vengono tradotte automaticamente.

**Interoperabilità:** applicare i significati definiti da [IPTC Photo Metadata Standard 2025.1](https://www.iptc.org/std/photometadata/specification/IPTC-PhotoMetadata-2025.1.html) e dalla [documentazione XMP Adobe](https://developer.adobe.com/xmp/docs/xmp-specifications/), distinguendo XMP incorporato e sidecar. Non promettere round-trip perfetto né supporto automatico a ogni tag IPTC-IIM/EXIF, TIFF o RAW; **prima fase JPEG/PNG/WebP supportati**, e matrice reale per formato/programma esterno.

- **Import:** mai creazione automatica di opere, vendita o diritti per il solo fatto che i tag esistano; se i campi sono in conflitto, niente aggiornamento silenzioso. Nessuna dipendenza da cartelle cloud sincronizzate.
- **Export:** solo campi scelti; filename sicuro, creazione senza sovrascrittura, report di campi riusciti/esclusi; esportazioni preesistenti, dodici CSV e backup continui a funzionare come oggi. Un sidecar prodotto esternamente non deve entrare nel backup a meno che l'utente non lo importi e il formato sia esplicitamente supportato.
- **Compatibilità:** archivi precedenti restano leggibili; vecchie fotografie prive di metadata/sidecar non ricevono attribuzioni inventate. Una eventuale migrazione deve mantenere UUID, revisioni, certificati immutabili e policy di rollback; nessun ri-encoding degli originali per migrare metadati.

## 8. Dipendenze, alternative, rischi, complessità

- **Dipendenze concettuali:** modelli Media Center e revisioni; lettura sicura dei formati; policy condivisa con TECH-013 per i campi divulgabili; eventuale integrazione al manifesto backup; librerie/parser da **valutare, non approvate**. Nessun nuovo servizio locale obbligatorio.
- **Alternative:** (A) nessuna funzione, crediti manuali correnti; (B) solo visualizzatore metadata — più sicuro e poco costoso; (C) importazione selettiva nel DB; (D) sidecar XMP esportabile su richiesta; (E) incorporazione nei derivati, più rischiosa e non necessaria nell'MVP. L'alternativa B potrebbe già soddisfare molti casi.
- **Rischi:** confusione fra diritti dell'opera/foto, introduzione involontaria di GPS in file condivisi, tag malformati, collisioni XMP/IIM, campo non supportato trasformato, perdita di hash dell'originale dopo un'ipotetica scrittura, inconsistenza tra metadati modificati e sidecar preesistente, formati RAW diversi, crescita/fragilità del backup.
- **Complessità qualitativa:** ispezione read-only **media**; importazione selettiva **medio-alta**; scrittura XMP interoperabile per più formati e gestione sidecar **alta**. Nessuna stima di tempi o libreria vincolante.

## 9. Criteri di accettazione *per un'eventuale futura decisione*

1. Su fixture IPTC/XMP ufficiali, mostra correttamente creator fotografico, credit line, descrizione e fonte; campi sconosciuti e malformati non provocano crash o modifiche.
2. Originali e RAW mantengono **identico SHA-256** dopo ispezione, anteprima, adozione e export della copia; anche le foto storiche dei certificati restano inalterate.
3. Con valore corrente già presente, un metadato esterno diverso **non** lo sovrascrive senza conferma; concorrenza tra anteprima e salvataggio invalida il confronto.
4. GPS e dati tecnici sensibili sono esclusi da ogni profilo condivisibile di default; un report dell'export rende verificabile l'insieme dei campi effettivamente inclusi.
5. Parser testato con XML/segmenti enormi, DTD/entità esterne, Unicode, lingua alternativa, percorsi malevoli, link simbolici e sidecar non corrispondenti: nessun file fuori archivio o accesso di rete.
6. Export e reimport presso **almeno due programmi esterni realmente usati** confrontati campo per campo su Windows/macOS/Linux (quando disponibile), con matrice dei limiti documentata; niente dichiarazioni generiche di compatibilità.
7. Archivio vecchio, backup e restore di prova dopo l'eventuale aggiunta di dati: nessuna perdita, duplicazione di fotografie o dipendenza da sidecar rimasti sul computer sorgente.
8. Navigazione da tastiera, zoom, testo di conflitto e annunci accessibili; traduzioni e fallback inglese testati senza tradurre i valori dei metadati sorgente.

## 10. Fasi ipotetiche e questioni per il proprietario

**Fase 0 — ricerca:** parlare con 3–5 fotografi/gallerie/archivi, raccogliere immagini e sidecar non sensibili con diritti d'uso per test; verificare se i campi già manuali sono sufficienti e quali strumenti fotografici usano realmente.

**Fase 1 — possibile MVP:** ispezione read-only dei metadati nelle immagini già supportate, senza schema aggiuntivo, nessun write-back.

**Fase 2 — solo previa approvazione distinta:** mappatura esplicita verso campi foto selezionati, confronti/consensi e compatibilità backup/export aperto.

**Fase 3 — eventuale, non necessaria:** sidecar XMP o scrittura nelle *copie* secondo standard, con test di round-trip e privacy rigorosi. Non includere automaticamente il supporto RAW completo.

**Domande da sottoporre alla revisione personale:**

1. Quanti utenti ricevono fotografie già corredate di IPTC/XMP e da quali software?
2. È più utile **leggere** i crediti esistenti o **produrre** sidecar da inviare a un fotografo/galleria?
3. Quali campi fotografici vanno tenuti separati dai dati sull'autore/titolare dei diritti **dell'opera**?
4. Vuole un profilo export privacy di default solo con nome fotografo e crediti, o preferisce partire da un viewer senza export?
5. È opportuno supportare i sidecar RAW ora, quando l'archivio prevede **un RAW per opera**, oppure rimandare?
6. Ci sono fixture fotografiche e strumenti esterni con cui dimostrare il bisogno? Senza di esse, preferire non avviare lo sviluppo.

**Esito della ricognizione:** una proposta tecnica facoltativa, da discutere e validare. **Nessuna decisione approvata o incarico di implementazione.**
