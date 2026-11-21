# REPS-TECH-027 — Importazione offline delle fotografie HEIC/HEIF, con originali intatti e anteprime interoperabili

- **ID stabile:** REPS-TECH-027
- **Data:** 2026-11-22
- **Stato:** **Proposta — da valutare**
- **Filone:** Strumenti e architettura; ricerca e progettazione, **nessuna implementazione autorizzata**
- **Priorità suggerita:** **P2 condizionata** a un riscontro d'uso concreto; P3 se la conversione manuale già soddisfa gli utenti
- **Ambito:** ingresso di fotografie da smartphone e dispositivi che producono HEIC/HEIF; **non** un editor HEIC, **non** trattamento RAW, **non** supporto generale ai video

## 1. Problema, destinatari e fatti verificati su `main`

Fotografie delle opere, dell'allestimento o delle condizioni di conservazione possono arrivare da iPhone e altri dispositivi nel formato **HEIC** (immagini in contenitore HEIF). Al momento chi conserva il file originale HEIC deve convertirlo altrove prima di usarlo come immagine visibile nell'archivio. Il problema **non è stato misurato presso gli utenti Repertica**: è un limite di formato osservabile nel codice, la cui importanza va prima validata.

- [`src-tauri/src/imaging.rs`](../../../src-tauri/src/imaging.rs) accetta esplicitamente soltanto `ImageFormat::Jpeg | Png | WebP` per decodifica e generazione di anteprime; applica già l'orientamento EXIF e limita dimensioni/pixel.
- [`src-tauri/src/gallery.rs`](../../../src-tauri/src/gallery.rs) e [`src-tauri/src/commands.rs`](../../../src-tauri/src/commands.rs) riconoscono i byte JPEG/PNG/WebP, non HEIC. Il [Media Center](../../../src/app/media-center.component.ts) offre un filtro di importazione `jpg, jpeg, png, webp`.
- [`src-tauri/Cargo.toml`](../../../src-tauri/Cargo.toml) abilita per la libreria `image` i soli codec `jpeg`, `png`, `webp`. Il [MVP](../../MVP.md) indica **12 MiB per immagine** ordinaria; il file RAW (un originale per opera, con altro modello e altri limiti) è un **allegato distinto**, non una soluzione implicita per le fotografie HEIC.
- La libreria media mantiene versioni e originali non distruttivi; backup locali verificati e CSV esistono già. **Non** si propone di sostituirli né di aumentare i limiti senza test.

**Utenti:** artisti che fotografano opere con il telefono, piccoli studi senza flusso fotografico professionale, gallerie che ricevono foto HEIC, archivi che combinano immagini mobili e JPEG. Apple documenta che il formato di cattura «High Efficiency» può produrre HEIF, mentre «Most Compatible» privilegia JPEG; la conversione in ingresso dipende anche dal metodo di trasferimento, quindi non tutte le foto da iPhone richiedono questa funzione ([Apple Support](https://support.apple.com/en-jo/116944)).

## 2. Perimetro e prevenzione dei doppioni

Questa idea riguarda **riconoscere, conservare e rendere consultabile** una sorgente HEIC, non i colori o i metadati da soli:

- [TECH-021](REPS-TECH-021-gestione-colore-icc-derivati-fotografici.md) approfondisce **ICC, conversioni colore e attendibilità cromatica** dei derivati; qualunque decoder futuro deve rispettarne le cautele, non creare un secondo motore del colore.
- [TECH-024](REPS-TECH-024-metadati-fotografici-iptc-xmp-interoperabili.md) studia lettura e adozione consapevole di **IPTC/XMP**; in questa proposta EXIF, GPS, profondità e dati tecnici non vengono importati automaticamente in schede professionali.
- [TECH-013](REPS-TECH-013-copie-fotografiche-condivisibili-privacy-metadati.md) disciplina copie condivisibili **senza dati sensibili**; la semplice conversione HEIC → JPEG non garantisce anonimizzazione.
- [TECH-005](REPS-TECH-005-anteprime-media-grandi-archivi.md) riguarda **cache e prestazioni generali**; qui la conversione specifica da HEIC è il presupposto, non un nuovo sistema di miniature.
- [TECH-001](REPS-TECH-001-pacchetto-dati-aperto.md) è il formato aperto per l'**intero archivio**. L'originale HEIC, se accolto, deve essere portabile come byte, anche quando un altro programma non sa visualizzarlo.
- Le **31 schede Gestione e UX** della [PR #14](https://github.com/Cesare-ml/Repertica-Studio/pull/14) trattano catalogazione, foto e diritti come processi professionali; non propongono un decoder HEIC. Nessun file di quel filone deve cambiare.

**Esclusi:** video HEVC, Live Photos come coppia video+foto, panorami, sequenze temporali, HDR end-to-end garantito, editing RAW/HEIC avanzato, conversione automatica massiva dell'archivio esistente, invio delle immagini a servizi online.

## 3. Scenari e flussi UX proposti

**Scenario A — una fotografia di un'opera.** In «Nuova opera» o Media Center → «Importa immagini», l'utente sceglie un file HEIC. Se è supportato dalla configurazione locale, una **anteprima esplicita** indica formato sorgente, dimensioni, orientamento, immagine primaria selezionata e dimensione del file. Alla conferma l'**originale viene copiato senza alterarne i byte**; una copia **JPEG/PNG di sola consultazione** può essere creata localmente e rigenerata. Se l'utente vuole ritagliare, una nuova revisione viene derivata senza sovrascrivere la sorgente. La galleria deve continuare a riconoscere copertina, versione e snapshot del certificato.

**Scenario B — formato non disponibile.** Se il sistema non dispone di un decoder verificato o il file è incompatibile, mostra «HEIC non supportato su questo dispositivo» (non «file corrotto») con soluzione locale alternativa: convertire una **copia** con lo strumento OS, quindi importare JPEG/PNG. Non rinominare l'estensione e non archiviare falsamente un HEIC sotto `.jpg`.

**Scenario C — file complesso.** Un HEIF può contenere più immagini, miniature incorporate, canali ausiliari/profondità o informazioni HDR. Un'eventuale prima fase deve importare solo la **immagine primaria chiaramente identificata**, segnalando che le altre rappresentazioni non sono state elaborate. Se non è possibile sceglierla con sicurezza, bloccare l'import anziché generare una copertina sbagliata. Non presentare una conversione SDR come prova di resa cromatica fedele dell'originale.

**Scenario D — esportazione, cambio computer e backup.** Dalla libreria l'utente sceglie «Esporta originale HEIC» oppure «Esporta copia compatibile JPEG/PNG». Le due azioni hanno nomi e MIME reali e non cambiano il file di origine. Un ripristino su Windows/macOS/Linux deve preservare **byte, hash e associazioni dell'originale** anche se su quella macchina non è installato un decoder; al massimo l'anteprima risulta indisponibile con messaggio chiaro.

**Accessibilità UX:** messaggi e progressi percepibili da screen reader e tastiera; anteprima priva di sole icone o colore per identificare un errore; avviso comprensibile quando una conversione riduce la gamma dinamica o perde dettagli; annullamento senza scritture parziali.

## 4. Modello e API, **soltanto ipotesi descrittive**

La scelta minima è valutare un record sorgente e un derivato compatibile **senza** cambiare la semantica dei certificati storici né trattare la cache come sorgente autorevole:

```text
ForeignPhotoInspection {
  source_path_selected_by_user,
  declared_extension, detected_container, detected_codec,
  primary_image_index?, width?, height?, orientation?,
  source_bytes, source_sha256,
  decoder_status: supported | unavailable | invalid | unsafe,
  warnings: [stable_code]
}
PhotoIngestPlan {
  source_sha256,
  preserve_original: true,
  source_mime: "image/heic" | "image/heif",
  preview_kind: "jpeg" | "png" | "unavailable",
  preview_transform_disclosure?,
  destination_archive_id,
  source_expected_unchanged
}
PhotoIngestResult {
  media_id, original_revision_id, original_sha256,
  preview_rebuildable: true,
  optional_derived_revision_id?, warnings: [stable_code]
}
inspectForeignPhoto(selectedPath) -> Inspection
previewForeignPhotoImport(inspection) -> Plan
confirmForeignPhotoImport(plan) -> Result
exportOriginalOrDerivative(mediaId, selectedKind, chosenPath) -> Report
```

Non sono API approvate. `media_id`, `revision_id` e hash dovrebbero riferirsi al contenuto **effettivamente conservato**. La progettazione deve chiarire se l'anteprima è una **cache eliminabile** o una **revisione persistente**: non si può usare una cache rigenerabile come unica fotografia di un certificato storico. I riferimenti dei certificati già emessi restano immutabili.

La compatibilità del modello dati SQLite, dei 12 CSV e del backup `manifest.json` v1 richiede una matrice di campi e formati. Nessun HEIC viene accettato nell'archivio attuale soltanto cambiando il filtro del dialogo: le verifiche in Rust, le revisioni, i backup, gli export e la diagnostica devono concordare.

## 5. Offline, sicurezza, privacy e scelte di impostazione

- **Offline completo**, nessun upload, API di conversione remota, account, abbonamento, servizio di licenza o codec scaricato automaticamente. L'import avviene solo dopo selezione consapevole di file locali; output, esportazioni e backup nelle cartelle liberamente scelte dall'utente.
- Originale non modificabile, derivati identificabili, staging temporaneo e pubblicazione atomica; in caso di errore nessun media orfano deliberato e nessuna scheda salvata con una foto fantasma. SHA-256 su byte originali. Verificare file regolari, limiti dimensionali e niente symlink/path traversal; conservare e ripristinare anche gli HEIC originali.
- I container ISO Base Media File e i codec sono input **non fidati**. Richiedere sniffing dei brand e del codec, limiti su numero immagini, dimensioni dichiarate e decodificate, memoria, CPU e tempo, immagini annidate, metadata e contenuti malformati; evitare che una decompression bomb blocchi l'app. Valutare isolamento del processo di decodifica e manutenzione delle vulnerabilità **prima** di qualsiasi dipendenza.
- Non pubblicare né trasferire in automatico EXIF/GPS, dati di profondità, geolocalizzazione, identificatori di dispositivo o diritti dichiarati. L'originale mantiene quei dati e va trattato come potenzialmente sensibile; la copia per condivisione segue le cautele di TECH-013 e TECH-024. Non dedurre autenticità o titolarità dei diritti dalla foto.
- **Impostazioni:** nessuna conversione automatica in background. Se utile, opzione locale di preferenza per derivati JPEG o PNG e avvisi di conversione, mai eliminazione degli originali; un eventuale pannello «Compatibilità formati» può indicare la disponibilità effettiva del decoder.
- **Multilingua:** lingua di sistema, fallback inglese e scelta manuale come oggi; errori tradotti con codici stabili, nomi di formati standard non tradotti, contenuti originali e metadati senza traduzione automatica.

## 6. Alternative, dipendenze, licenze e rischi

| Alternativa | Vantaggio | Rischio/limite |
| --- | --- | --- |
| **A. Documentare conversione manuale locale in JPEG/PNG** | Nessuna nuova dipendenza, minimo costo e superficie d'attacco; disponibile subito come procedura | Doppio passaggio, possibile perdita di metadata/gamma dinamica; HEIC originale conservato solo esternamente se non allegato altrove |
| **B. Decodifica nativa per sistema operativo** | Potenziale riuso di funzionalità installate su macOS/Windows | Comportamenti e disponibilità divergenti tra OS e distribuzioni Linux; build e test più difficili |
| **C. Decoder open source incorporato facoltativamente** | Maggiore uniformità e indipendenza dal codec del sistema | Peso, manutenzione, vulnerabilità, packaging multipiattaforma, licenze di librerie/codec e **possibili profili brevettuali HEVC** da verificare prima della distribuzione |
| **D. Custodia HEIC come solo originale non visualizzabile** | Portabilità dei byte senza decodifica | Inutile per copertina, galleria e certificati se manca un derivato verificato; non va simulato supporto completo |

**Nessuna libreria è scelta.** Il progetto [libheif](https://github.com/strukturag/libheif) è un riferimento di studio per HEIF/HEIC e AVIF, ma supporta back-end diversi, ha licenza e componenti con condizioni proprie e richiede una valutazione aggiornata di sicurezza, licenze, brevetti, footprint e riproducibilità delle build. La compatibilità open source del software **non** prova la libertà da brevetti o vincoli di distribuzione dei codec. L'eventuale decoder dovrà essere disponibile senza rete nelle build rilasciate oppure la funzione dovrà dichiararsi non disponibile.

**Dipendenze progettuali:** modelli media/revisioni e pipeline di ingest, cache dei thumbnail, salvaguardia delle immagini storiche, controllo di integrità SQLite/media, backup completo, export esplicito, policy privacy. **Complessità stimata medio-alta/alta**, specie per Windows/macOS/Linux, HDR e packaging. Non è richiesto un nuovo servizio cloud, né un secondo catalogo media.

## 7. Criteri di accettazione e prove future (non eseguite)

1. Matrice Windows/macOS/Linux su **file campione con licenza di test**: HEIC 8/10 bit, orientamento EXIF, ICC/P3, un file HEIF multi-immagine, file malformato, file oltre i limiti, JPEG/PNG/WebP di regressione. Il comportamento supportato/non supportato deve essere dichiarato chiaramente per ciascun OS.
2. SHA-256 e dimensione **dell'originale importato** identici a quelli della sorgente; conversione e ritaglio producono revisioni distinte, senza mutare originali, copie storiche e riferimenti dei certificati.
3. Import di file illeggibile, codec non disponibile, disco pieno, annullamento e crash non creano opere con foto mancanti, file di staging abbandonati o backup incompleti.
4. Backup verificato e ripristino offline preservano byte HEIC e relazioni, anche su macchina incapace di decodificare il formato. Esportazione dell'originale **byte-identica**; esportazione del derivato identificata come nuova copia con eventuali limiti cromatici.
5. Nessuna scansione/upload di file fuori dall'archivio; eventuale GPS non emerge in copie condivisibili senza consenso. Profilo colore, EXIF e orientamento non vengono dichiarati «corretti» senza prove tecniche.
6. Costo CPU, memoria, dimensioni app e durata della conversione misurati su file realistici e hardware modesto; accessibilità dei messaggi e corretto fallback di lingua verificati.
7. Audit delle licenze e sicurezza delle dipendenze **prima** dell'eventuale adozione; nessuna regressione sulla possibilità di usare Repertica completamente offline o esportare tutto.

## 8. Priorità, fasi ipotetiche e decisioni per il proprietario

**Requisiti certi:** la libreria accetta attualmente JPEG/PNG/WebP e distingue i RAW; immagini e dati sono locali; archivio, backup, originali e certificati devono restare affidabili. **Ipotesi:** molti utenti vorrebbero importare HEIC senza conversione esterna; un decoder locale sarebbe sostenibile; un derivato riproducibile risolverebbe il problema. Non sono disponibili misurazioni della frequenza o dei costi di supporto.

**Fasi possibili, non assegnate:** (0) raccogliere file e casi reali e verificare prima se serve; (1) documentare il workaround locale e la matrice di compatibilità; (2) prototipo isolato/benchmark del decoder, audit licenze e sicurezza, **senza includerlo nelle release**; (3) se approvato, ingresso locale con originali intatti e derivati espliciti; (4) solo dopo verifiche, edit/export avanzato e integrazione con politiche colore/metadati già discusse.

**Questioni aperte da sottoporre al proprietario:** quanti artisti ricevono davvero HEIC? È sufficiente una conversione manuale descritta bene? Si vuole conservare sempre l'HEIC oltre al JPEG, pur aumentando backup e consumo di disco? Serve import di un'intera cartella di foto oppure solo file selezionati? Quali dispositivi/OS sono prioritari? Quali limiti di peso e HDR sono accettabili? Si preferisce rinunciare al supporto quando un decoder non è distribuibile in modo sicuro/giuridicamente sostenibile?

> Nessuna idea approvata, nessuna modifica di software, dipendenze, workflow o impostazioni; nessuna issue o PR di implementazione. Il proprietario può valutare questa proposta e decidere anche di **non** procedere.
