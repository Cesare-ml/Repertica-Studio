# REPS-TECH-041 — Identificazione delle opere secondo la checklist Object ID, senza attestazioni automatiche

- **ID stabile:** REPS-TECH-041
- **Data ricognizione:** 2026-11-22
- **Stato:** **Proposta — da valutare**
- **Filone:** Strumenti e architettura, esclusivamente documentazione
- **Priorità proposta, non approvata:** **P2** soltanto se gallerie, archivi e artisti con esigenze di identificazione/consegna confermano il bisogno; **P3** per archivi personali semplici o quando una scheda manuale basta
- **Nessuna idea approvata, assegnata o autorizzata allo sviluppo.**

## 1. Problema, destinatari e riscontri verificati

Fotografie, misure, tecnica e segni caratteristici sono fondamentali quando bisogna **identificare una specifica opera fisica** dopo smarrimento, sottrazione, trasferimento o richiesta documentale. Sono informazioni diverse da un prezzo, un certificato di autenticità o un dossier commerciale. Se sono sparse fra scheda, immagini e note libere, chi prepara un fascicolo di identificazione deve ricomporle a mano e può omettere dettagli discriminanti.

Lo standard descrittivo **Object ID**, riconosciuto da ICOM e richiamato da INTERPOL, definisce una procedura di documentazione con fotografie, breve descrizione, custodia sicura e **nove categorie**: tipo di oggetto, materiali/tecniche, misure, iscrizioni/marchi, caratteristiche distintive, titolo, soggetto, data/periodo, autore/produttore. Fonti istituzionali verificate il 22/11/2026:
- [ICOM — Object ID](https://icom.museum/en/resources/standards-guidelines/objectid/)
- [ICOM — Checklist Object ID](https://icom.museum/wp-content/uploads/2020/12/ObjectID_english-1.pdf)
- [INTERPOL — Object ID](https://www.interpol.int/en/Crimes/Cultural-heritage-crime/Object-ID)

**Importante:** Object ID è una **checklist documentale**, non un formato JSON ufficiale universalmente interoperabile, né una procedura automatica di registrazione presso INTERPOL o di attribuzione di autenticità. In questa scheda «basato sulla checklist» **non** significa certificazione di conformità formale.

**Fatti dal `main` esaminato il 22/11/2026:**
- [`src-tauri/src/models.rs`](../../../src-tauri/src/models.rs) conserva su `Artwork` codice, titolo, anno libero, tecnica, supporto, tre dimensioni opzionali in cm, descrizione, ubicazione e immagine. `signed: bool` dice solo che l'opera è firmata: **non trascrive una firma, né ne indica la posizione**.
- [`src-tauri/src/media.rs`](../../../src-tauri/src/media.rs) e [README](../../../README.md) documentano Media Center, versioni e fotografie; [MVP](../../MVP.md) descrive tipi di foto aggiuntive (incluso dettaglio/firma), ma una fotografia di tipo «firma» **non prova** che l'iscrizione sia stata verbalizzata.
- [`docs/DATA-PORTABILITY.md`](../../DATA-PORTABILITY.md) prevede dodici registri CSV, con relazioni e percorsi, e backup integrali SQLite + media; i CSV **non includono automaticamente** i byte delle fotografie.
- [`docs/BROCHURES-OFFLINE.md`](../../BROCHURES-OFFLINE.md) descrive brochure editoriali; i certificati sono snapshot storici immutabili. Nelle superfici lette **non risulta un percorso dedicato** alla valutazione delle nove categorie Object ID. Questo è un limite di ricognizione statica, **non un bug dimostrato**, e non prova che gli utenti richiedano la funzione.

**Destinatari ipotetici:** archivi d'artista, piccole gallerie, collezionisti che documentano la consistenza della raccolta, professionisti che preparano una descrizione per assicuratore o autorità competente. Non assume che chi conserva dati abbia titolo giuridico per denunciare, reclamare o condividere l'opera.

## 2. Confini anti-duplicazione e requisiti certi

- **[UX-007 — Dossier e presentazioni](https://github.com/Cesare-ml/Repertica-Studio/blob/docs/idee-gestione-ux-20261121/docs/idee/gestione-e-ux/REPS-UX-007-dossier-presentazione-opere.md):** dossier **editoriale/commerciale** per selezioni di opere; qui si studia **completezza descrittiva e corrispondenza esplicita alle categorie Object ID**, per identificazione fisica, non impaginazione promozionale.
- **[UX-004 — Condizione](https://github.com/Cesare-ml/Repertica-Studio/blob/docs/idee-gestione-ux-20261121/docs/idee/gestione-e-ux/REPS-UX-004-rapporti-stato-conservazione.md):** rapporti datati di stato e conservazione; qui si riporterebbero soltanto segni distintivi **dichiarati** e riferimenti a fotografie, senza aprire pratiche di restauro.
- **[UX-016 — Coperture](https://github.com/Cesare-ml/Repertica-Studio/blob/docs/idee-gestione-ux-20261121/docs/idee/gestione-e-ux/REPS-UX-016-valutazioni-coperture-assicurative-scadenze.md):** valori e polizze; qui nessuna stima, premio, copertura, assicurazione o titolo di proprietà.
- **[UX-020 — Etichette](https://github.com/Cesare-ml/Repertica-Studio/blob/docs/idee-gestione-ux-20261121/docs/idee/gestione-e-ux/REPS-UX-020-etichette-inventariali-identificativi-scansionabili.md):** trovare fisicamente una scheda via codice/QR, non documentare i segni che permettono di identificare l'oggetto.
- **[UX-031 — Autorialità](https://github.com/Cesare-ml/Repertica-Studio/blob/docs/idee-gestione-ux-20261121/docs/idee/gestione-e-ux/REPS-UX-031-autorialita-opere-attribuzioni-collaborazioni.md):** eventuali attribuzioni strutturate; qui autore non noto o dubbio deve restare **non verificato**, mai inferito dal profilo dell'utente.
- **[TECH-001](REPS-TECH-001-pacchetto-dati-aperto.md), [TECH-013](REPS-TECH-013-copie-fotografiche-condivisibili-privacy-metadati.md), [TECH-024](REPS-TECH-024-metadati-fotografici-iptc-xmp-interoperabili.md):** interscambio dell'**intero archivio**, copie fotografiche privacy-aware, metadati **della fotografia**. Un fascicolo di una **singola opera fisica** deve semmai riutilizzare tali regole, senza inventare un secondo export integrale o trattare un copyright foto come proprietà dell'opera.

**Requisiti certi del progetto:** gratuità e codice aperto; Tauri/SQLite locale; offline-first; nessun account né cloud obbligatorio; dati sempre esportabili e backup a cartella scelta; lingua di sistema con fallback inglese; immutabilità di certificati storici, registri economici e file originali.

**Ipotesi da validare:** operatori realmente interessati alla checklist; campi mancanti abbastanza frequenti da richiedere persistenza; maggiore utilità di un riepilogo rispetto a un modello PDF compilato manualmente.

## 3. Casi d'uso e flussi UX proposti

**Scenario A — opera che parte per una mostra:** l'archivista apre `Opera → Strumenti → Identificazione (Object ID)`; vede le nove categorie come **presenti / parziali / mancanti / non applicabili dichiarato**, con indicazione esatta della fonte. Per esempio: titolo e misure presenti, «iscrizioni» non documentate benché `signed=true`. La UI **non trasforma** «firma presente» in una descrizione inventata.

**Scenario B — identificazione dopo smarrimento:** una persona incaricata prepara una copia documentale con foto panoramica e dettagli selezionati, titolo e segni annotati. Si distinguono fatti presenti nella scheda, dichiarazioni aggiunte e campi ignoti. Il programma **non produce una denuncia, non trasmette dati a INTERPOL, non verifica proprietà o autenticità** e non dichiara automaticamente l'opera rubata.

**Scenario C — comunicazione con terzi:** prima di esportare, anteprima e selezione esplicita dei dati, soprattutto fotografie di segni non pubblici, ubicazione, nome del possessore, riferimenti identificativi e metadati EXIF. Predefinito per uso personale, **nessuna condivisione pubblica implicita**. Distinguere «copia di lavoro riservata» da «copia per destinatario scelto»; quest'ultima omette per default ubicazione esatta, collezionista, prezzi, ricevute, note commerciali e dettagli sensibili.

**Scenario D — opera con dati insufficienti:** `year` è approssimativo, autore incerto, manca foto leggibile del retro. La checklist segnala le assenze, permette di tornare alla scheda e non impedisce un documento **chiaramente marcato come incompleto**; una data approssimativa non diventa un giorno preciso e un'opera firmata non viene attribuita automaticamente all'artista del profilo.

**Fase iniziale raccomandata solo come studio:** pannello **read-only di completezza e origine** che non crea nuove tabelle; eventuali campi specifici «iscrizioni», «segni distintivi», «soggetto», «tipo di oggetto» vanno prima testati con operatori e potrebbero restare una **bozza temporanea di esportazione**, non trasformarsi automaticamente in nuovi attributi obbligatori dell'opera.

**Esportazione eventuale:** scegliere foto e campi → mostrare anteprima con stato di completezza, fonte e destinatario previsto → conferma → scegliere cartella esterna liberamente → generare **una nuova sottocartella**, mai sovrascrivere → verificare tutti i file copiati → mostrare risultato e avviso su dati personali. Prima considerare un **PDF leggibile** con immagini; solo se i destinatari lo richiedono, aggiungere un **JSON aperto e documentato di Repertica**, etichettato *mappatura delle categorie Object ID*, **non «formato ufficiale Object ID»**.

## 4. Dati e ipotesi API — solamente descrittivi

Rappresentazione **effimera** di un rapporto di completezza, non schema deciso:

- `IdentificationField { category, value?, source_kind, source_record_id?, declared_by_user: boolean, completeness, uncertainty_note? }` con categorie stabili `object_type`, `materials_techniques`, `measurements`, `inscriptions_markings`, `distinguishing_features`, `title`, `subject`, `date_period`, `maker`.
- `IdentificationPhoto { media_id, revision_id?, purpose: overall | mark | detail, share_selected, rights_note?, content_sha256? }`; l'hash identifica **byte del file**, non la stessa opera o i suoi diritti.
- `IdentificationSnapshot { artwork_id, observed_at, source_revision?, fields[], selected_photos[], intended_audience, incomplete_categories[], format_version }` creato solo in seguito a conferma esplicita. Un rapporto di completezza **non** equivale a uno snapshot storico immutabile né a una prova.
- API concettuali: `inspectIdentificationCoverage(artworkId) → CoverageReport`, `prepareIdentificationExport(artworkId, selection) → Preview`, `exportIdentificationPacket(previewToken, destination) → Manifest`. Se scheda o media cambiano dopo l'anteprima, il piano va invalidato e riconfermato; nessuna esecuzione schedulata.
- Valori sconosciuti separati da «non applicabile» e «vuoto per scelta»; unità dichiarate; annotazioni di incertezza; titolo e descrizione nella lingua originale. Nessuna trasformazione di indizi in fatti.

Un eventuale formato supplementare usa UTF-8, versione, nomi di categorie documentati, percorsi relativi e manifest SHA-256 per le immagini effettivamente selezionate. I riferimenti devono essere risolvibili **senza** installare Repertica, ma non sono un feed automatico per archivi delle forze dell'ordine. Preservare UUID e storia dell'archivio senza pubblicare UUID come collegamenti pubblici.

## 5. Offline, privacy, sicurezza, impostazioni, lingue

- **Offline:** analisi sul DB e file locali; output nella cartella scelta, anche disco esterno/NAS montato, senza API remote, browser esterni, account o abbonamento. Il collegamento Internet non è necessario per leggere la checklist.
- **Privacy e sicurezza:** dettagli di riparazioni, marchi particolari, luogo di custodia e fotografie del retro possono aumentare l'esposizione a furto o falsa rivendicazione. Escludere per default informazioni personali/commerciali e GPS/EXIF non necessari; nessuna pubblicazione automatica o indicizzazione in MCP; preview prima della copia. Verificare permission e destinazioni, no symlink/traversal, no sovrascrittura, nessuna scrittura nell'archivio attivo per mera anteprima. Le copie esterne **non entrano automaticamente nei backup** dell'archivio.
- **Sicurezza dei media:** non alterare gli originali, leggere versioni verificabili, contenere dimensioni/numero immagini e consumi di memoria, non fidarsi di metadati file o estensioni, distinguere fallimenti del PDF da errori di scrittura; cancellare solo file temporanei che l'operazione ha creato e mai cartelle di terzi.
- **Impostazioni:** nessun nuovo interruttore obbligatorio. Eventuale preferenza locale «mostra avvisi di dati sensibili sempre» solo dopo prova UX, senza default permissivo. Il profilo dell'artista non compila automaticamente l'identità del creatore.
- **Multilingua e accessibilità:** messaggi e checklist localizzati nella lingua OS con fallback inglese, selezione manuale compatibile con IT/EN/ES/FR/DE; **il contenuto dichiarato dall'utente non viene tradotto automaticamente**. PDF eventuale nella lingua selezionata, font e caratteri Unicode da verificare (vedi [TECH-032](REPS-TECH-032-pdf-brochure-unicode-font-incorporati-offline.md)); navigazione solo tastiera, screen reader che annunci stato per campo, anteprime accessibili e stati non identificati soltanto dal colore.

## 6. Complessità, dipendenze, alternative, rischi

**Complessità orientativa:** bassa–media per semplice checklist read-only che mappa i dati; media per campi aggiuntivi di identificazione; medio–alta per pacchetto multiformato completo con immagini e impaginazione nativa. Stime **non misurate**.

**Dipendenze possibili, non assegnate:** `Artwork`, galleria fotografica e versioni media, renderer locale PDF eventualmente riutilizzabile, pipeline export sicuro e backup. Un eventuale nuovo schema richiederebbe migrazione **additiva**, export completo dei nuovi campi e prova di ripristino, con compatibilità dei backup precedenti; non cambiare snapshot e seriali dei certificati esistenti.

**Alternative reali:** (A) checklist ICOM compilata a mano e foto ordinate in una cartella: costo sviluppo zero, buona opzione iniziale; (B) sezione di note libere nelle schede senza modello nuovo: meno accurata ma semplice; (C) solo relazione di completezza letta dalle schede: lieve beneficio, rischio minimo; (D) dossier UX-007 adattato alla checklist **solo se** il percorso di verifica descrittiva resta distinto dal commerciale.

**Rischi:** confondere presenza di campi con attendibilità; divulgare fotografie e dettagli identificativi riservati; interpretare una firma come attribuzione; dichiarare falsamente «conforme Object ID»; creare dati duplicati rispetto ai report di condizione; esportare PDF privi di caratteri leggibili; conservare copie esterne non protette; aggiungere troppa complessità ad artisti con poche opere.

## 7. Fasi eventuali e criteri di accettazione da sottoporre al proprietario

**Fase 0 — validazione del bisogno:** esaminare con archivisti/gallerie 3–5 casi documentali reali e confrontare checklist manuale con la sola scheda esistente; identificare destinatari e dati realmente richiesti. Se basta un modello compilabile esterno, **non procedere**.

**Fase 1 — rapporto di copertura senza mutazioni:** mappare tutte e nove le categorie, mostrare fonte/incertezza/assenza, distinguere `signed=true` da iscrizione descritta; nessuna modifica al DB, media o certificati. Test con record vuoti, anno approssimativo, opere senza foto, opere con più foto, titolo Unicode.

**Fase 2 — export locale esplicito, se necessario:** PDF descrittivo con immagini selezionate, anteprima dei campi, privacy predefinita restrittiva, destinazione libera, non sovrascrittura, errori atomici, funzionamento senza rete. Verificare visualizzazione su Windows/macOS/Linux e su supporti esterni scollegabili.

**Fase 3 — solo se richiesta dagli utenti:** metadati specifici persistenti, eventuale JSON di mappatura aperto versionato, test di round-trip e documentazione di quali campi Object ID sono stati dichiarati o non compilati. Nessun import automatico da INTERPOL/ICOM.

**Accettazione, esempi concreti:** un'opera con `signed=true` e nessuna trascrizione produce «iscrizione non documentata»; nessuna foto viene esportata senza scelta; l'assenza del creatore non è riempita dal nome del proprietario; output esterno non contiene per default ubicazione privata, contatti, prezzi o ricevute; chi legge distingue **incompleto**, **non applicabile dichiarato**, **dato attestato dall'utente**; la revisione della scheda fra preview ed export non produce un pacchetto dichiarato coerente; un errore lascia intatti archivio e file preesistenti; nessun campo nuovo viene omesso dal backup integrale se in futuro ne fosse approvata la persistenza.

## 8. Questioni aperte — esclusiva decisione del proprietario

1. Quanti utenti reali richiedono una checklist Object ID invece di un dossier o PDF generico?
2. Preferire un semplice **controllo delle informazioni già presenti** o l'introduzione di campi facoltativi e persistenti (tipo/soggetto/iscrizioni/segni)?
3. Occorrono copie «riservata» e «da condividere» con regole indipendenti e quale fotografia selezionare per ciascuna?
4. È necessaria un'esportazione machine-readable oltre al PDF, e quale sistema esterno la importerebbe davvero?
5. Come distinguere identificazione descrittiva, certificazione d'autenticità, documentazione assicurativa e procedure legali nelle etichette UI?

**Esito della ricognizione:** una sola proposta, derivata da funzioni presenti e checklist istituzionale; **nessun codice, test, workflow, configurazione, dipendenza o documento dell'altra automazione modificato**. La valutazione non equivale ad approvazione o sviluppo.
