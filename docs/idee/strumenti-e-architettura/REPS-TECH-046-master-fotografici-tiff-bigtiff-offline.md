# REPS-TECH-046 — Master fotografici TIFF e BigTIFF preservati offline, senza confonderli con le anteprime

- **ID stabile:** REPS-TECH-046
- **Data ricognizione:** 2026-11-22
- **Stato:** **Proposta — da valutare**
- **Priorità proposta (non approvata):** **P2 condizionata** a un bisogno verificato presso fotografi d'arte, gallerie e archivi; **P3 o nessuno sviluppo** quando JPEG/PNG/WebP, RAW e un catalogo esterno soddisfano già il caso d'uso.
- **Ambito:** interoperabilità di formati fotografici, conservazione locale di originali grandi e derivati di consultazione; soltanto studio. Nessuna funzione è approvata, assegnata o implementata da questa scheda.

## 1. Esigenza, destinatari ed evidenze su `main`

Una galleria, un fotografo professionale o un archivio di opere può ricevere una scansione o una fotografia master in **TIFF**, talvolta a 16 bit/canale o con profilo colore incorporato, che non intende ricomprimere in JPEG per conservarla. Vorrebbe **associare il file originale all'opera, ritrovarlo, conservarlo con backup verificato e ottenerne una copia identica**, continuando a usare miniature leggere per la navigazione. Il bisogno è plausibile per questo pubblico, **ma non è stato ancora validato da interviste o raccolte reali di file**.

**Verificato staticamente su `main` il 22 novembre 2026:**
- [`src-tauri/Cargo.toml`](../../../src-tauri/Cargo.toml) abilita attualmente nel crate `image` soltanto `jpeg`, `png`, `webp`; non abilita `tiff`.
- [`src-tauri/src/commands.rs`](../../../src-tauri/src/commands.rs) (`read_image_file`) accetta fotografie fino a **12 MiB**, legge i byte e `image_type` riconosce JPEG/PNG/WebP; [`src-tauri/src/media.rs`](../../../src-tauri/src/media.rs) registra foto nel Media Center, conserva revisioni e crea miniature/anteprime. [`src-tauri/src/imaging.rs`](../../../src-tauri/src/imaging.rs) decodifica quei tre formati, applica l'orientamento EXIF e genera derivati RGB.
- [`src-tauri/src/raw_files.rs`](../../../src-tauri/src/raw_files.rs) offre **già** un allegato RAW per opera, copiato in streaming e verificato con SHA-256 fino a 1 GiB. Un TIFF di scansione **non è automaticamente un file RAW camera**: non è corretto chiamare il supporto RAW esistente «supporto TIFF».
- [`README.md`](../../../README.md), [`docs/MVP.md`](../../MVP.md), [`docs/DATA-PORTABILITY.md`](../../DATA-PORTABILITY.md) documentano copie di backup SQLite + `media/` con manifest e verifica; i CSV sono metadati e **non** trasportano i byte delle immagini. Questa proposta non sostituisce tali garanzie.
- Non risulta da questi percorsi una registrazione **dedicata** di TIFF master di conservazione con associazione all'opera e anteprima dichiaratamente derivata. È un'osservazione sul codice letto, **non** la prova che tutti i workflow esterni o i file TIFF falliscano, né un bug riprodotto.

**Riferimenti esterni utili per progettazione, non obblighi automatici di conformità:**
- [Library of Congress — Recommended Formats Statement, Still Image Works](https://www.loc.gov/preservation/resources/rfs/stillimg.html): TIFF figura tra i formati preferiti per fotografie digitali; attenzione a risoluzione, profondità e profilo colore.
- [LibTIFF — BigTIFF Design](https://libtiff.gitlab.io/libtiff/specification/bigtiff.html): il TIFF classico usa offset a 32 bit e ha il limite strutturale di circa 4 GiB; BigTIFF usa offset a 64 bit. **BigTIFF non è sinonimo di TIFF standard** e il suo supporto non va promesso senza decoder e prove dedicate.

## 2. Confini e confronto con idee già presenti

- [TECH-027](REPS-TECH-027-importazione-fotografie-heic-heif-offline.md) studia **HEIC/HEIF da smartphone**, con scelte e licenze codec diverse. Qui l'oggetto è un **master di scansione/fotografia TIFF** di uso professionale.
- [TECH-021](REPS-TECH-021-gestione-colore-icc-derivati-fotografici.md) è l'autorità eventuale per **profili ICC e conversioni cromatiche**, anche nel caso di un TIFF; non duplicare il motore di gestione colore.
- [TECH-005](REPS-TECH-005-anteprime-media-grandi-archivi.md) tratta **cache di anteprime** indipendentemente dal formato; [TECH-003](REPS-TECH-003-diagnostica-integrita-media.md) integrità e file mancanti; [TECH-007](REPS-TECH-007-archivio-media-esterno-rilocalizzabile.md) posizione esterna dei media; [TECH-026](REPS-TECH-026-backup-compatti-snapshot-verificabili.md) copie compatte. Restano separati.
- [TECH-024](REPS-TECH-024-metadati-fotografici-iptc-xmp-interoperabili.md) regola eventuale lettura/scambio dei metadati; [TECH-016](REPS-TECH-016-allegati-documentali-locali-multi-entita.md) studia allegati **generici**, mentre qui occorrono **semantica master/derivato, rischi decoder e identificazione fotografica**.
- [TECH-029](REPS-TECH-029-scambio-selettivo-schede-opere-archivi-indipendenti.md) riguarda lo scambio fra archivi; qui un file master può essere parte di un export selettivo **solo se** quella proposta sarà adottata.
- Nel filone [Gestione e UX — PR #14](https://github.com/Cesare-ml/Repertica-Studio/pull/14), **UX-007** riguarda dossier e presentazioni, **UX-010** crediti/diritti di riproduzione, **UX-019** documentazione storica, **UX-025** istruzioni di cura e allestimento. La presente scheda **non** introduce processi gestionali né attribuisce diritti alle fotografie.

Non creare un nuovo Media Center, non alterare il campo dell'immagine principale e non assimilare un TIFF a un certificato di autenticità. Nessuna decodifica massiva automatica di archivi esistenti.

## 3. Scenari e flussi UX proposti

### Caso A — conservazione di un master, minimo valore utile

1. Nella scheda opera, l'utente sceglie facoltativamente **«Collega master fotografico TIFF»** e seleziona un file tramite dialogo nativo.
2. L'app mostra **nome, formato dichiarato, dimensione su disco, eventuali informazioni tecniche leggibili e spazio necessario**, con avviso che l'anteprima potrebbe non essere disponibile; chiede conferma esplicita prima della **copia** nell'archivio privato.
3. Una eventuale operazione locale in streaming copia i byte senza riscriverli, registra SHA-256, esito e riferimento alla scheda. L'originale nel percorso sorgente resta intatto; un'operazione fallita non produce un collegamento valido e non cancella file preesistenti.
4. L'opera indica **«Master originale conservato»**, dimensione, stato di integrità eventualmente ricalcolato su richiesta e le azioni **«Esporta copia identica»** / **«Mostra dettagli»**. Il record non dichiara automaticamente se la fotografia sia aggiornata, autorizzata o utilizzabile commercialmente.
5. Miniatura dell'opera e copertina corrente **non cambiano da sole**: possono rimanere JPEG/PNG/WebP già associate.

### Caso B — consultazione e derivato (fase successiva)

Se l'utente non dispone già di una fotografia leggera, può **richiedere una copia di consultazione**, con preview del progetto di conversione e consenso. Il motore deve dichiarare il supporto reale della variante TIFF (compressione, profondità, numero di pagine, spazio colore) e **rifiutare in modo trasparente** quelle non compatibili; mai prendere la prima pagina o ridurre a 8 bit in modo tacito. Un derivato JPEG/PNG è etichettato come **derivato**, con hash/relazione alla sorgente e opzioni coerenti con TECH-021 e TECH-024. Non sovrascrivere il master; non far diventare automaticamente il derivato la copertina.

### Caso C — ripristino e condivisione

Il master salvato fa parte del **backup integrale** in una cartella liberamente scelta, USB o NAS montato; il ripristino deve verificare tutti i byte prima di sostituire l'archivio attivo. Un destinatario che riceve una copia manuale può verificare l'hash con un comune strumento offline. Nessuna pubblicazione web, invio email, sincronizzazione o apertura di link esterni automatica.

**Accessibilità:** stato testuale di copia/verifica, avanzamento comprensibile con screen reader, focus stabile, annullamento distinguibile da errore, nessuna dipendenza dal solo colore delle immagini.

## 4. Contratto dati e ipotesi di API (descrittivi, NON implementati)

Esempio di **modello concettuale** separato dai RAW camera, dalle revisioni JPEG/PNG/WebP e dai certificati:

```text
ArtworkArchivalMaster {
  id: UUID,
  artwork_id: UUID,                   // non altera l'identità dell'opera
  original_filename: string,
  media_relative_path: string,       // solo sotto il media store autorizzato
  declared_format: "tiff" | "bigtiff",
  inspect_status: "supported" | "limited" | "uninspected" | "invalid",
  file_size_bytes: uint64,
  sha256: string,                     // sui byte originali, non sul derivato
  width?: uint64, height?: uint64,
  bit_depth?: uint16, page_count?: uint32,
  color_profile_status?: string,
  created_at: timestamp,
  optional_note?: string
}
MasterDerivativeLink {
  master_id: UUID,
  derivative_media_id: UUID,
  master_sha256_at_derivation: string,
  processing_recipe_version?: string,
  declared_purpose: "preview" | "presentation"
}
inspectArchivalMaster(path_selected_by_user) -> InspectionReport
attachArchivalMaster(artwork_id, confirmed_selection) -> CopyReceipt
verifyArchivalMaster(master_id) -> IntegrityReport
exportArchivalMaster(master_id, user_selected_destination) -> CopyReceipt
requestPreviewDerivative(master_id, explicit_options) -> PreviewOrUnsupported
```

Sono **nomi esemplificativi**: non definiscono tabelle, metodi Rust/Tauri, endpoints o migrazioni approvati. Valutare prima se, per l'MVP di conservazione, basti riutilizzare in modo **esplicito** un futuro sistema di allegati generici (TECH-016), aggiungendo tipo e hash senza moltiplicare tabelle. Non occupare senza decisione lo slot RAW esistente (**un RAW per opera**). La possibilità di **più master per opera** (scansione fronte/retro, versioni, luce radente) è una scelta aperta: non imporre arbitrariamente uno oppure infiniti.

**Regola di identità:** il file master conserva hash e byte originali; qualsiasi derivato ha **identità distinta**. Link semantici non implicano licenza, titolarità o autenticità.

## 5. Offline, sicurezza, privacy, impostazioni, lingue

- **Offline-first incondizionato:** nessun account, abbonamento, codec cloud, download di servizi, rete o telemetry obbligatori. L'eventuale decoder deve essere disponibile e utilizzabile localmente dopo installazione normale, con licenze e distribuzione verificabili.
- **Ingestion sicura:** verificare file regolare, percorso selezionato, limite di dimensione configurabile secondo prove, spazio, sorgente stabile durante copia, streaming con hash, staging, rollback prudente; rifiutare symlink/path traversal e non fidarsi dell'estensione `.tif`.
- **Parsing non attendibile:** TIFF è un contenitore complesso (IFD multipli, tile/strip, compressioni, metadati, canali, profondità elevate); limiti rigorosi per megapixel, dimensioni, memoria, tempo, pagine e struttura ricorsiva. Valutare decoder con isolamento dal processo principale ove necessario. **Memorizzare byte senza decodificare** può essere molto meno rischioso della produzione di preview.
- **No conversioni silenziose:** non abbassare bit depth, non reinterpretare CMYK/ICC e non alterare le immagini storiche di certificati già emessi. Non dichiarare «fedeltà del colore garantita» in assenza di un percorso di gestione colore validato.
- **Privacy:** originali TIFF possono contenere informazioni su fotografo, committente, posizione, volti o tag privati. Nessuna lettura automatica da parte di MCP, invio alla rete o esportazione inclusiva per default. L'hash non va usato come certificato di proprietà/autenticità; nei report anonimi non includere percorsi assoluti.
- **Impostazioni da valutare:** consentire/non consentire master pesanti, limite ragionevole per singolo file (da misurare), policy di anteprime derivate e spazio indicativo. Senza modifiche alle preferenze generali, archivi preesistenti continuano a funzionare.
- **Multilingua:** lingua del sistema con fallback inglese e scelta manuale esistente; messaggi chiari e localizzati per «originale», «derivato», «non visualizzabile», «dimensione troppo grande», «backup include il file», errori di codec/restore; nomi file e metadati dell'utente mai tradotti o alterati.

## 6. Import/export, backup, compatibilità e dipendenze

- **Import:** copia locale di byte sorgente **identici**, con hash e validazioni; nessun import di URL o download remoto. Non considerare una copia da un disco esterno soltanto referenziata tramite percorso un file effettivamente acquisito.
- **Export:** il master originale si esporta **byte-per-byte** in una cartella scelta dall'utente, senza sovrascrittura di file esistenti, con eventuale ricevuta SHA-256 leggibile offline. Un CSV separato, se utile, può riportare UUID opera/master, nome, formato, dimensione e hash, ma **non sostituisce il backup**.
- **Backup:** rispettare formato di manifest versionato, controlli SHA-256, snapshot SQLite coerente e presenza di tutti i media referenziati. Non perdere file che il manifest tratta prudenzialmente come storici. Il backup può crescere molto: presentare dimensioni indicative e verificare limiti del filesystem di destinazione (per esempio FAT32 e file singoli grandi), senza duplicare TECH-022.
- **Ripristino:** archivio e file master recuperati insieme, nessun riferimento assoluto all'hardware di origine; l'app che non supporta il nuovo eventuale schema deve rifiutare il ripristino incompatibile, non ignorare i master.
- **Compatibilità:** Windows/macOS/Linux, file e nomi Unicode; NAS/USB come filesystem OS montati. Una versione vecchia può non conoscere nuovi record e non deve fingere di aver ripristinato tutti gli allegati. Le funzioni odierne di Media Center, allegati RAW, certificati e CSV restano invariate in assenza di adozione esplicita.
- **Dipendenze potenziali, solo da studiare:** parser/intestazioni TIFF in sola lettura; decoder locale per preview opzionale; streaming/checksum e protocollo di staged copy già usati per RAW; storage/media e backup esistenti; contratto colore TECH-021 e cache TECH-005. Non aggiungere librerie, binari, tool, workflow o configurazioni finché il proprietario non decide.

## 7. Alternative, rischi e complessità

| Alternativa | Vantaggi | Limiti |
| --- | --- | --- |
| **A. Nessuna funzione nuova**: conservare TIFF fuori da Repertica con cartella/backup proprio e associarli tramite note | Zero nuove dipendenze | Il backup Repertica non comprende gli originali esterni; rischio di collegamenti persi |
| **B. Allegato TIFF in sola conservazione** (fase minima ipotetica) | Integrità e recupero, senza obbligo di decodifica | Non mostra il master direttamente; richiede gestione chiara della distinzione |
| **C. TIFF classico + derivato leggero opzionale** | Consultazione pratica | Maggiori superfici di decoder, rischio ICC/bit-depth/multipagina |
| **D. BigTIFF e varianti professionali ampie** | Archivi di scansioni molto grandi | Costi elevati di parsing, streaming, spazio, test multipiattaforma; **non** fase iniziale predefinita |

Rischi: file molto grandi su FAT32/NAS, copia incompleta, file sostituito durante acquisizione, percezione che l'hash provi autenticità, archivio gonfiato, decoder con input malevoli, metadati sensibili, falsa fedeltà cromatica, master multi-pagina non coperto da una singola preview, migrazione che perde legami, deduplicazione distruttiva. Mitigazioni: validazioni e limiti, copia atomica/staging, hash post-copia, distinzione originale/derivato, nessuna cancellazione automatica, backup verificato e test reali.

**Complessità stimata:** bassa-media per una **valutazione/guida**; media per conservazione di un allegato senza preview con test di backup; alta per decoder/derivati/colorimetria; **molto alta** per supporto esteso BigTIFF e set multi-pagina su più filesystem. Valutare costi di manutenzione e superficie di sicurezza rispetto a pochi utenti potenziali.

## 8. Prove e criteri di accettazione ipotetici (NON eseguiti)

1. Raccogliere, con permesso e senza conservarli nel repository, **5–10 esempi reali** TIFF di professionisti (dimensioni, bit depth, profili, compressioni, pagine) e contare chi ne richiede davvero l'archiviazione dentro Repertica. Se nessuno, chiudere l'idea.
2. Un master accettato rimane byte-identico: hash SHA-256 alla fonte, nella copia archiviata, nel backup ripristinato e nel file esportato **coincidono**, inclusa una prova con interruzione volontaria.
3. Un TIFF sconosciuto/senza decoder non viene classificato come «foto corrotta» se la funzione in uso è **solo conservazione**; il software distingue «formato non supportato per preview» da «copia o hash errati».
4. Il dataset comprende: TIFF little/big endian, LZW/Deflate/non compresso, 8/16 bit, ICC valido/mancante, CMYK, canali alfa, multipagina, file volutamente malformati, offset illegali, bombe di decompressione e **BigTIFF** solo per verificarne il riconoscimento/rifiuto esplicito quando non supportato.
5. La UI mantiene copertina e certificati immutabili, non modifica la versione Media Center corrente e non genera duplicazioni di record dopo annullamento o retry.
6. Backup e ripristino (con spazio insufficiente, filesystem senza supporto a file grandi, file mancante, USB rimossa, errore a metà) non sostituiscono mai l'archivio valido con una copia incompleta.
7. Nessun invio/rete/account, nessuna scansione automatica di cartelle esterne, nessuna lettura MCP di master o metadati sensibili; import/export manuali con avvisi e dialoghi navigabili solo da tastiera.
8. Test desktop reali su Windows/macOS/Linux e almeno un'unità USB/NAS; prima di definire i limiti, misurare tempi, memoria, crescita del backup e tempi di ripristino per file grandi.
9. Le schermate e le condizioni di errore sono tradotte in tutte le lingue supportate, rispettano la lingua OS/fallback EN e non confondono «originale» con «anteprima».

Questi criteri sono **proposte per un eventuale futuro sviluppo**, non test eseguiti né suite modificate in questa PR.

## 9. Fasi da sottoporre al proprietario

- **Fase 0 — validare il bisogno e la soluzione più piccola:** campioni reali, frequenza di uso, strumenti esterni già adottati; decidere se una guida a conservazione TIFF esterna + backup autonomo è sufficiente.
- **Fase 1 — conservazione non distruttiva solo se richiesta:** collegamento master ↔ opera, copia streaming, hash, export originale, backup completo, nessuna preview necessaria.
- **Fase 2 — anteprime su richiesta:** subset TIFF misurato e dichiarato, decoder verificato, derivati, relazione esplicita, coordinamento TECH-021/005/024, senza conversioni tacite.
- **Fase 3 — BigTIFF e archivi molto grandi:** soltanto con esigenza documentata, minacce/codec/OS testati e costo di supporto accettabile.

## 10. Questioni aperte per revisione personale

1. Quanti utenti target mantengono master `.tif/.tiff` che **vogliono realmente dentro Repertica**, e quanto pesano?
2. Basta un **allegato master verificato** con export integro, oppure è indispensabile mostrarlo e derivarne anteprime nell'app?
3. Un'opera deve poter avere **più master** (fronte/retro, risoluzioni, acquisizioni, versioni), e con quale nomenclatura non ambigua?
4. Quali compressioni, profondità di bit, profili e tipi di TIFF sono effettivamente in uso? L'intera compatibilità «TIFF» sarebbe troppo vaga.
5. Quali limiti di spazio, singolo file e durata backup sono accettabili, inclusi dispositivi USB e NAS?
6. Meglio integrare l'eventuale tipo di allegato nel modulo trasversale TECH-016 anziché introdurre un'altra relazione nel database?
7. Il proprietario desidera **non sviluppare questa funzione** finché un archivio pilota non la richiede? Questa è una scelta pienamente valida.

**Esito della ricognizione:** opportunità documentata, non malfunzionamento verificato. **Proposta — da valutare**; nessuna modifica a codice, test, build, workflow, configurazioni, dipendenze o file del filone Gestione e UX; nessuna implementazione, approvazione o assegnazione.
