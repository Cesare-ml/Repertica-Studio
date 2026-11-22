# REPS-TECH-045 — Associazione guidata di fotografie già importate alle opere, senza abbinamenti silenziosi

- **ID stabile:** REPS-TECH-045
- **Data della ricognizione:** 2026-11-22
- **Stato:** **Proposta — da valutare**
- **Filone:** Strumenti e architettura; esclusivamente ricerca e progettazione.
- **Priorità proposta:** **P2 condizionata** per archivi fotografici con molti file da collegare; **P3 o nessuno sviluppo** per chi gestisce poche immagini. Nessuna approvazione né assegnazione.
- **Oggetto:** una *revisione assistita* degli abbinamenti tra fotografie **già presenti nel Media Center** e opere già censite. Non un nuovo sistema di import, non una modifica batch automatica alle schede e non riconoscimento di immagini tramite IA.

## 1. Problema, utenti e riscontri verificati

Un artista fotografa decine di opere e importa le immagini nel Media Center. La libreria consente già un'importazione multipla, ma per associare le immagini importate alle singole schede occorre poi aprire i contenuti e scegliere l'opera di destinazione, con rischio di scambiare fotografie di opere simili. Un archivista che riceve una cartella con file chiamati per esempio `ART-0042-fronte.jpg` e `ART-0042-dettaglio.jpg` desidera **controllare** la corrispondenza prima di collegarli alle opere, non lasciar decidere il programma.

**Fatti riscontrati su `main` il 22/11/2026:**
- [README](../../../README.md) e [MVP](../../MVP.md) descrivono Media Center, immagini principali e fino a 20 foto aggiuntive per opera (JPEG/PNG/WebP, massimo 12 MiB per immagine) e backup completi.
- [`src/app/media-center.component.ts`](../../../src/app/media-center.component.ts), metodo `importFiles`, permette già di selezionare **più file** con il dialogo nativo, esegue `mediaImport(paths)` e restituisce import riusciti e falliti: **non proporre nuovamente un import multiplo come lacuna**.
- [`src-tauri/src/media.rs`](../../../src-tauri/src/media.rs) registra singoli file come oggetti della libreria, mantiene versioni e utilizzi; `artwork_gallery_from_media` collega **un media a una galleria alla volta**, verifica esistenza opera, limite 20, stato attivo e impiego già esistente in gallerie. `artwork_cover_from_media` modifica esplicitamente una copertina. Sono già presenti controlli per impedire che la rimozione di un riferimento cancelli un file protetto o una foto storica di certificato.
- [`docs/DATA-PORTABILITY.md`](../../DATA-PORTABILITY.md) espone dodici esportazioni CSV, fra cui `gallery`, ma non prevede che gli UUID/percorsi CSV autorizzino modifiche automatiche alle schede. Il backup integrale di SQLite + media è l'unico ripristino completo.
- La UI esaminata consente selezione di un media e scelta manuale dell'opera per la galleria; **non emerge** un riepilogo *di gruppo* con suggerimenti di associazione da confermare uno per uno. Questa è una verifica statica del codice/documenti consultati, **non** una segnalazione d'errore riprodotta né un bisogno misurato presso utenti reali.

**Destinatari:** fotografi d'opera, artisti con cataloghi già codificati, archivi di atelier, piccoli musei e gallerie che ricevono molti file per opere già registrate. L'interfaccia ordinaria per uno o due scatti deve rimanere invariata.

## 2. Confini e anti-doppioni

- **Media Center già attivo:** importazione multipla, editor non distruttivo, tag, raccolte, deduplicazione e associazione individuale **esistono**. Questa scheda valuta **solo la fase intermedia di revisione e collegamento di più immagini già in libreria**, senza riscrivere tali funzionalità.
- **[REPS-TECH-019](REPS-TECH-019-importazione-guidata-cataloghi-tabellari.md):** importa record di opere/contatti da tabelle esterne; qui **nessuna creazione o modifica di metadati delle opere** dal nome di una fotografia.
- **[REPS-TECH-024](REPS-TECH-024-metadati-fotografici-iptc-xmp-interoperabili.md)** e **[REPS-TECH-027](REPS-TECH-027-importazione-fotografie-heic-heif-offline.md):** interpretazione dei metadati e nuovi codec sono temi separati. Non dedurre autore, copyright, località, data dell'opera o contenuti della scheda da EXIF/XMP/nome file.
- **[REPS-TECH-003](REPS-TECH-003-diagnostica-integrita-media.md)** e **[REPS-TECH-038](REPS-TECH-038-staging-media-backup-concorrenza-snapshot.md):** controlli e coerenza dei file sono già un'altra linea; questa proposta **riusa** e non duplica i requisiti di sicurezza.
- **[REPS-TECH-029](REPS-TECH-029-scambio-selettivo-schede-opere-archivi-indipendenti.md):** scambio fra due archivi con pacchetti è distinto dal collegare fotografie nell'**archivio attivo**.
- **[REPS-UX-018](https://github.com/Cesare-ml/Repertica-Studio/blob/docs/idee-gestione-ux-20261121/docs/idee/gestione-e-ux/REPS-UX-018-catalogazione-seriale-schede-bozza.md)** tratta la creazione manuale in sequenza di **nuove opere**. Qui le opere **esistono già**. **[UX-048](https://github.com/Cesare-ml/Repertica-Studio/blob/docs/idee-gestione-ux-20261121/docs/idee/gestione-e-ux/REPS-UX-048-identificativi-storici-riferimenti-esterni-opere.md)** propone codici esterni qualificati, che **non** vanno usati per abbinare automaticamente fotografie.
- Non sono inclusi OCR, riconoscimento visivo, correlazione IA, lettura automatica di intere cartelle, upload cloud, sincronizzazione, creazione di opere, vendite, certificati, cambi di proprietà o diritti. Nessun file UX viene modificato.

## 3. Scenari e flussi UX proposti

1. **Archivio con nomi coerenti.** L'artista ha opere `ART-0042` e `ART-0043`, e media già importati `ART-0042-front.jpg`, `ART-0042-detail.jpg`, `ART-0043-front.jpg`. In Media Center seleziona questi file e apre «Associa a opere…» → una tabella *di anteprima* propone il codice **solo quando esiste un'identità univoca e completa nel catalogo attuale**, non sulla base del titolo, della somiglianza fotografica o di frammenti ambiguamente corrispondenti. Nessun collegamento è scritto durante l'anteprima.
2. **Codice incerto.** Due titoli o nomi file simili, codici annidati (es. `ART-1` e `ART-10`), file senza codice o corrispondenze multiple → stato «Da scegliere». Un operatore cerca e seleziona esplicitamente l'opera oppure marca «Non associare». L'avviso non suggerisce autenticità o proprietà.
3. **Fotografia in uso.** La libreria rivela che un media è già utilizzato come copertina, fotografia di altra galleria o foto di un collezionista → non duplicarlo né spostarlo tacitamente. Mostrare contesto d'uso e richiedere una **operazione separata**; la prima fase lascia la riga non eseguibile.
4. **Limite e ordinamento.** Un'opera ha già 19 fotografie aggiuntive e il gruppo propone tre immagini: soltanto una può essere confermata per quella galleria, con riepilogo del limite 20. Nessuna eliminazione di immagini esistenti per far posto. L'ordine proposto può essere revisionato manualmente senza riclassificare quelle già presenti.
5. **Cartella rimovibile già usata per import.** L'app non deve dipendere dal percorso originale sul disco USB: il collegamento usa l'**UUID del media esistente nel database** e la relativa revisione corrente. Non conservare come collegamento vivo il percorso assoluto `E:\...` o `/Volumes/...`.
6. **Interruzione dell'operazione.** Se durante la conferma arriva un errore, l'utente riceve un resoconto preciso di righe collegate e righe non collegate; nulla viene interpretato come «tutto completato» e nessuna riga già riuscita viene ripetuta silenziosamente al riavvio.

**Flusso accessibile minimo:** «Media Center → Seleziona immagini → Associa a opere» (azione facoltativa), vista con miniature già disponibili, nome file, utilizzi correnti, suggerimento esplicitamente qualificato, campo di ricerca opera, destinazione «Galleria» di default, didascalia e categoria (facoltative), stato della riga, selettore «Conferma». L'utente attiva «Verifica selezione», legge il riepilogo e poi «Conferma associazioni». Pulsante «Annulla» sempre disponibile prima del salvataggio; focus e conteggi di esito annunciati anche allo screen reader. **Nessuna sostituzione della copertina in questa prima fase:** sarebbe un percorso distinto con anteprima della copertina precedente, controllo dei certificati e conferma specifica.

## 4. Regole tecniche e ipotesi di API (solo descrittive)

**Fase minima:** preferire una **vista temporanea** senza nuova tabella: selezione di media già importati (UUID, revisione, stato e utilizzi attuali) più opera di destinazione (UUID; codice mostrato), ruolo `ARTWORK_GALLERY`, categoria della foto e didascalia opzionali. Non conservare un presunto «codice dell'opera» derivato dal filename come identità: può essere una semplice proposta di match con `confidence = manual_review_required` anche se l'algoritmo trova una sola corrispondenza.

Esempio descrittivo non vincolante:
- `inspectMediaAssignments(media_ids, archive_snapshot) -> { rows: [{ media_id, suggested_artwork_id?, filename, current_usages, reason, conflicts }], snapshot_token }`
- `previewMediaAssignments(rows, snapshot_token) -> { valid_rows, rejected_rows, proposed_changes, per_artwork_remaining_slots }`
- `commitConfirmedMediaAssignments(confirmed_rows, expected_current_state) -> { linked, skipped, failed }`

**Preflight e commit:** verificare ogni UUID e l'uso attuale della **revisione corrente** (che può cambiare tra anteprima e conferma); le appartenenze non devono essere inferite soltanto da stringhe di percorso. La fonte autorevole resta SQLite con le regole già adottate da `artwork_gallery_from_media`. Rivalidare al momento del commit vincoli su opera presente, media `ACTIVE`, disponibilità, limite 20 e ordine/didascalia/categoria. Non aggiornare vendite, `artworks.status`, proprietà, codici, prezzi, `media_revisions` o snapshot di certificati. Non eludere il divieto di impiego doppio nella galleria passando un altro percorso del medesimo file. Non sostituire l'algoritmo di deduplica Media Center con matching per filename.

**Politica errori da decidere:** una transazione unica per l'intero gruppo è più facile da comprendere come «tutto o niente», ma può bloccare lotti grandi; commit per opera con esito idempotente richiede identificazione robusta delle righe riuscite. La specifica preferisce **prima** lotti piccoli con rifiuto completo in caso di un conflitto sopraggiunto, salvo esigenze reali documentate. In futuro: avanzamento e ripresa di lotti grandi dopo misure e test; non promettere un'atomicità di file+SQLite non dimostrata.

**Persistenza eventuale:** solo se necessari dopo ricerche d'uso, `assignment_session_id`, `source_media_id`, `target_artwork_id`, `role`, `decision`, `base_revision_id`, `status` e timestamp, salvati **localmente** in formato versionato. La sessione non è una prova di attribuzione o di autenticità; non creare record di storia permanente solo per una proposta non approvata.

## 5. Offline, sicurezza e privacy

- Tutto avviene nel database e nella libreria media locali, **senza account, Internet, cloud, servizi IA o telemetria**. L'eventuale selezione iniziale di file da disco usa soltanto l'import multiplo **già presente**.
- Non seguire link simbolici né percorsi fuori dalla cartella media; controllare che le sorgenti siano file regolari già validati dal Media Center. Non accettare input che scelgano arbitrariamente un path come destinazione della galleria. Non leggere o caricare in memoria una seconda volta grandi batch di immagini se la libreria fornisce già le miniature.
- **Privacy:** nomi file e miniature possono rivelare nomi di clienti, firme, luoghi, opere non pubblicate e informazioni di laboratorio. Anteprima solo in locale; niente auto-export, cache esterne non dichiarate, eventi di rete o log con path assoluti. I dati di copyright del Media Center non diventano automaticamente autorizzazione di riproduzione.
- **Backup e versionamento:** continuare a usare il backup esistente SQLite + media con manifest e SHA-256, scegliendo liberamente la cartella. Una lista di assegnazioni in CSV/JSON è eventualmente solo report, **mai backup integrale** e mai sostituto di un ripristino. Vietata qualsiasi pulizia automatica di file storici o immagini non collegate.

## 6. Impostazioni, lingua e interoperabilità

- **Impostazioni:** funzione facoltativa disattivabile, nessuna scansione periodica di cartelle. Se in futuro esistessero regole di denominazione, sarebbero **esplicitamente impostate dall'utente per archivio**, anteprima sempre richiesta, nessuna applicazione tacita. Non riutilizzare automaticamente la scelta di destinazione al lotto successivo.
- **Multilingua:** usare la lingua del sistema con fallback inglese e selezione manuale già prevista (IT/EN/ES/FR/DE). Tradurre «corrispondenza suggerita», «ambigua», «in uso», «limite raggiunto», «non associata», avvisi e risultati; non tradurre nomi dei file, codici, titoli o didascalie inseriti da persone. Supportare tastiera, navigazione con zoom 200%, nomi accessibili e stati espressi anche con testo.
- **Import/export:** la proposta non inventa un formato proprietario di immagini né un nuovo parser di CSV. Mantiene i CSV `artworks` e `gallery` già esistenti (UUID, media, tipo, ordine, didascalie) e il backup completo per trasferimento effettivo dei file. Un eventuale report dei **soli collegamenti confermati** può essere un CSV UTF-8 scelto manualmente, con UUID e codici delle opere e ID dei media, senza fotografie/indirizzi/contatti e senza reimport automatica. Un report non deve creare l'illusione di una sincronizzazione bidirezionale.
- **Compatibilità:** prima fase possibilmente senza variazione dello schema SQLite, usando le associazioni esistenti; riconoscere che app più vecchie possono non aprire archivi con futuri schemi. Nessuna dipendenza nuova obbligatoria; per il matching del codice utilizzare regole deterministiche strettamente testate, senza modelli IA o servizi terzi.

## 7. Dipendenze, alternative, rischi e complessità

**Dipendenze già disponibili:** `media_import`, `media_list`, `media_detail`, `artwork_gallery_from_media`, ricerca opere, codici univoci Repertica, validazioni Rust, UI Angular/Tauri, SQLite, i18n, backup e storico Media Center. **Solo se valutata** servirebbe una specifica di applicazione batch con revalidazione e transazione; non è una richiesta di implementarla.

**Alternative da testare prima di sviluppare:**
1. Nessuna novità: cercare la foto e associare manualmente una per volta nell'interfaccia attuale.
2. Un comando minimale «Collega alla stessa opera la prossima fotografia selezionata» per 2–5 immagini: più economico, ma meno adatto a lotti misti.
3. Usare raccolte Media Center per organizzare i file e poi assegnarli individualmente; potrebbe risolvere quasi tutti i casi senza nuovo wizard.
4. Se gli utenti usano soprattutto file HEIC o CSV di cataloghi preesistenti, valutare prima le proposte TECH-027 o TECH-019; questa funzione non risolve quei problemi.

**Rischi:** false corrispondenze da prefissi e nomi, esposizione di media riservati, collisioni e reimport ripetute, modifica concorrente delle gallerie, accumulo di foto non collegate, problemi di cancellazione dei file originali, aspettative errate su diritti/autenticità, interfaccia sovraccarica. Mitigazione prioritaria: **nessun collegamento suggerito viene applicato senza una scelta esplicita per riga**, ricontrollo vincoli nel commit e riepilogo verificabile.

**Complessità stimata:** bassa per un riepilogo locale e suggerimenti di match deterministici; **media** per UI con selezione/revisione, preflight e commit protetto; **medio-alta** per lotti grandi, recovery/idempotenza e test di concorrenza, da motivare solo se l'uso reale lo richiede. Nessuna data di consegna assegnata.

## 8. Criteri di accettazione e prove da valutare

1. Import multiplo esistente resta disponibile e produce gli stessi risultati; la nuova vista inizia **solo** su richiesta dell'utente.
2. La proposta di match non scrive nulla in SQLite e non cambia byte, versioni o percorsi dei media.
3. `ART-1` non viene confuso con `ART-10`; per nomi senza confini chiari viene chiesta scelta manuale.
4. Due media candidati o due opere plausibili non generano collegamenti automatici.
5. L'opera assente, venduta o non disponibile non viene reinterpretata come nuova opera; nessuna scheda viene creata.
6. Foto già collegate, in cestino o in uso presso un collezionista sono segnalate e non spostate.
7. Una galleria con 19 foto accetta al massimo un ulteriore collegamento; con 20 non ne accetta alcuno.
8. Cambio della copertina non disponibile nella prima fase; certificati e fotografie storiche restano invariati.
9. Didascalie con Unicode, categorie consentite e ordine corretto persistono; i campi non scelti non ricevono valori inventati.
10. Due sessioni concorrenti, modifica della revisione del media o cancellazione di un'opera tra anteprima e commit producono rifiuto chiaro, non stato parziale silenzioso.
11. Errori su disco pieno, database occupato, annullamento, chiusura e riavvio non cancellano originali né alterano il backup precedente.
12. Backup/ripristino completo conserva le nuove relazioni; CSV galleria conferma esattamente i link effettivi.
13. Applicazione funziona completamente offline su Windows/macOS/Linux; dialoghi accessibili da tastiera e con screen reader, zoom 200%; lingua del sistema con fallback inglese.
14. Su piccoli lotti (2–5 immagini), la soluzione non deve essere più complessa del flusso singolo esistente: se lo è, rinviare questa idea.
15. Test con archivio reale anonimizzato, almeno 20–30 media destinati a più opere, e misurazione del tempo e degli errori rispetto al collegamento manuale; **nessuna efficacia presunta senza prova**.

## 9. Possibili fasi e domande al proprietario

**Fase 0 — ricerca senza sviluppo:** osservare almeno 3–5 sessioni reali di archiviazione e raccogliere esempi anonimizzati di nomi file; verificare se la selezione multipla attuale, le raccolte e i link individuali bastano.

**Fase 1 — solo se motivata e autorizzata in seguito:** pannello di anteprima/revisione su immagini già nel Media Center; esclusivamente associazioni confermate alla galleria, piccole serie, senza schema nuovo.

**Fase 2 — distinta e facoltativa:** lotti grandi con report, ripresa sicura e controllo delle modifiche concorrenti; valutare solo dopo misure di prestazioni e casi di interruzione.

**Questioni aperte per la revisione del proprietario:**
- Quanti media per sessione vengono oggi importati, e quante associazioni errate capitano?
- Le foto hanno davvero codici d'archivio nei filename o sono prevalentemente nominate da fotocamere/telefoni?
- Basterebbe un'operazione «assegna le 3 selezionate alla stessa opera» senza sistema di suggerimenti?
- È opportuno **escludere per sempre** l'assegnazione della copertina dai batch oppure prevederla come flusso separato?
- In caso di una sola riga non più valida, preferire annullare l'intero gruppo o confermare soltanto le altre, con rapporto di esito?
- La necessità è abbastanza frequente da giustificare interfaccia, controlli e test, oppure conviene mantenere il collegamento manuale?

**Esito della ricognizione:** documentazione soltanto. Stato **Proposta — da valutare**; nessun codice o workflow da cambiare, nessuna funzione approvata, nessun avvio sviluppo o merge.
