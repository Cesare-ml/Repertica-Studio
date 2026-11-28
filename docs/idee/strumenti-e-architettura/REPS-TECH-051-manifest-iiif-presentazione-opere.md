# REPS-TECH-051 — Esportazione IIIF Presentation per scambio con archivi e musei, senza hosting obbligatorio

- **ID stabile:** REPS-TECH-051
- **Data della ricognizione:** 2026-11-23
- **Stato:** **Proposta — da valutare**
- **Filone:** strumenti e architettura; esclusivamente analisi e documentazione
- **Priorità ipotetica:** **P3**; elevabile a **P2 solo dopo richiesta verificata** da archivi, gallerie o istituzioni che usino effettivamente IIIF
- **Decisione del proprietario:** nessuna. Non autorizza sviluppo, modifica di codice, pubblicazione web, esposizione di dati o dipendenze nuove.

## Problema concreto e destinatari

Per uno scambio con musei e biblioteche digitali, un CSV Repertica o una brochure PDF possono non essere la rappresentazione richiesta dal sistema destinatario. IIIF Presentation è un formato aperto di **presentazione di oggetti digitali e immagini**, utilizzato da viewer e infrastrutture di collezioni digitali. Un curatore potrebbe chiedere un *Manifest* IIIF per mostrare diverse riprese della stessa opera in un viewer compatibile, pur conservando il catalogo originale interamente su Repertica.

**Destinatari potenziali:** archivi d'artista che collaborano con istituzioni, fondazioni, conservatori e gallerie con un catalogo pubblico. Per uno studio con poche opere o senza partner IIIF, il bisogno non è dimostrato: **non proporre un nuovo menu per tutti**.

**Verificato su `main` il 2026-11-23:** `src-tauri/src/media.rs` conserva immagini originali e revisioni non distruttive con identificativi, dimensioni, hash SHA-256, crediti e stato di copyright dichiarato; `src-tauri/src/gallery.rs` gestisce più immagini per opera; `docs/DATA-PORTABILITY.md` documenta esportazioni CSV dei registri e backup con SQLite e media. Non risulta un esportatore IIIF in questi percorsi né una voce IIIF nella ricognizione delle proposte TECH/UX. Non è stata ricevuta una richiesta reale da un museo: questa è **un'opportunità da validare**, non una lacuna critica o un bug.

## Natura del formato: limite fondamentale da non nascondere

La specifica [IIIF Presentation API 3.0](https://iiif.io/api/presentation/3.0/) richiede identificativi `id` **HTTP(S)** per le risorse definite dalla specifica; i Canvas hanno un loro URI senza frammento. Le immagini accessibili ai viewer devono avere URL effettivamente servibili, non percorsi locali privati `file://`. Un JSON con `id` fittizi e JPEG in una cartella **non è da presentare come pubblicazione IIIF funzionante**. Il servizio [IIIF Image API](https://iiif.io/api/image/3.0/) è **facoltativo** nel progetto iniziale: non promettere zoom a tasselli, ricerca, autenticazione o un server immagini.

Il possibile risultato è un **pacchetto di prepubblicazione creato interamente offline**, con manifest JSON, immagini derivate per la pubblicazione, elenco dei percorsi/URL attesi e validazione statica. Sarà il **destinatario, su sua infrastruttura scelta liberamente**, a pubblicare i file e verificarne gli URL. In assenza di un dominio/hosting definito, il pacchetto è una bozza con stato **“Non pubblicato / URL non verificati”**, non un Manifest operativo in rete. Nessun servizio Repertica o canone necessario.

## Confini e anti-duplicazione

- [TECH-001](REPS-TECH-001-pacchetto-dati-aperto.md): esportazione **completa** e reimportazione semantica dell'archivio. IIIF è una **proiezione pubblicabile, selettiva, solo in uscita**, non un backup né un import.
- [TECH-013](REPS-TECH-013-copie-fotografiche-condivisibili-privacy-metadati.md), [TECH-021](REPS-TECH-021-gestione-colore-icc-derivati-fotografici.md) e [TECH-024](REPS-TECH-024-metadati-fotografici-iptc-xmp-interoperabili.md): privacy delle copie, colore e metadata *dentro le fotografie*. Questa proposta riguarda la **struttura dei Manifest/Canvas**, riutilizzando eventuali verifiche comuni anziché duplicarle.
- [TECH-029](REPS-TECH-029-scambio-selettivo-schede-opere-archivi-indipendenti.md): scambio di schede fra archivi Repertica indipendenti, non interscambio con viewer IIIF.
- [UX-007](https://github.com/Cesare-ml/Repertica-Studio/blob/docs/idee-gestione-ux-20261121/docs/idee/gestione-e-ux/REPS-UX-007-dossier-presentazione-opere.md), [UX-010](https://github.com/Cesare-ml/Repertica-Studio/blob/docs/idee-gestione-ux-20261121/docs/idee/gestione-e-ux/REPS-UX-010-diritti-riproduzione-crediti-autorizzazioni.md), [UX-041](https://github.com/Cesare-ml/Repertica-Studio/blob/docs/idee-gestione-ux-20261121/docs/idee/gestione-e-ux/REPS-UX-041-titoli-descrizioni-opere-multilingua.md): presentazione editoriale, autorizzazioni dichiarate e contenuti multilingua. **Nessuna di queste schede è approvata**. La proposta tecnica non attribuisce diritti e non risolve gli aspetti gestionali.
- Eventuale Repertica Sites è un prodotto separato; questa scheda **non** lo implementa, non crea siti, non propone un account e non collega il desktop a una piattaforma online.

## Scenari d'uso e flussi UX (ipotetici)

1. **Richiesta di un museo.** L'operatore sceglie manualmente 1–10 opere e per ciascuna le riprese destinate all'istituzione, non l'intera libreria. Prima della generazione, conferma titolo, descrizione, misure, fotografo e crediti *pubblicabili*. Un campo assente rimane omesso, non inventato.
2. **Preparazione offline.** Da “Esporta per istituzioni → IIIF (sperimentale)” seleziona le immagini e vede una lista precisa di ciò che uscirà. Il programma produce copie derivate in una **nuova** cartella liberamente scelta; mai pubblicazione o upload. Il riepilogo informa se non si conosce lo stato dei diritti o se restano metadata sensibili da esaminare. Per default: niente dati di collezionisti, acquirenti, trattative, email, indirizzi, note interne, prezzi, file RAW, seriali privati o contabilità.
3. **URL e destinazione.** Se il partner fornisce un URL base HTTPS e una convenzione dei percorsi, l'utente li inserisce *esplicitamente* e ottiene un Manifest con `id` e `body.id` coerenti con i **futuri** URL. Senza URL, si produce solo un **piano di pubblicazione** con i dati selezionati e un modello non validabile come Manifest completo: non fabbricare un `example.com` spacciandolo per URL reale.
4. **Controlli ed esito.** Anteprima leggibile dei titoli, ordine Canvas, immagini, crediti, campi ignorati e indirizzi che il destinatario dovrà ospitare. Verifica JSON locale e hash dei derivati, quindi “Esporta bozza”. Conferma che la verifica offline **non** prova raggiungibilità HTTP, CORS o compatibilità del viewer del museo.
5. **Pubblicazione successiva non automatizzata.** L'operatore consegna il pacchetto al partner che gestisce l'hosting, oppure lo ospita sul servizio statico di propria scelta. Una prova eventuale da browser è **separata e avviata manualmente**, non requisito del desktop offline.
6. **Test con opere non pubblicabili.** Se una ripresa è ad uso interno o un'autorizzazione è sconosciuta, il sistema non sceglie la licenza al posto dell'utente. Se il proprietario non intende esporla, la esclude dalla selezione; non promettere una verifica giuridica automatica.

## Dati e API concettuali — non approvate

Una possibile **proiezione senza nuovo database** (prima alternativa), alimentata dai dati correnti e dagli override temporanei inseriti nella schermata di export:

```text
IiifExportSelection {
  artwork_ids: UUID[],
  canvas_order: [{ artwork_id, media_revision_id, selected_for_publication }],
  public_fields: { title?, description?, year?, technique?, support?, dimensions? },
  credit_override?: string,
  rights_statement_uri?: string,   # soltanto scelta esplicita e compatibile
  language: BCP47,
  base_https_url?: string,
  destination_folder: user_chosen_path
}
IiifExportPreview {
  manifest_id?, canvas_ids[], image_urls[], omitted_fields[],
  rights_unknown[], generated_files[], warnings[], publicable: boolean
}
```

- **Output ipotetico:** `manifest.json` per opera, eventuale `collection.json` per una selezione, `images/` con **copie** scelte/derivate, `publication-map.json` con destinazioni e SHA-256 dei file, `README.md` con istruzioni e limitazioni. Il JSON tecnico usa chiavi standard stabili; non duplicare tutti i registri di Repertica.
- **Proiezione IIIF:** `Manifest` → `items` di `Canvas` nell'ordine scelto; ogni Canvas con `AnnotationPage` e `Annotation` con `motivation: "painting"` e `body` di tipo `Image`, oltre a `label`, `summary`, `metadata` solo se selezionati. La lingua nei `language maps` è distinta dalla lingua dell'interfaccia; usare `none` quando la lingua del testo è ignota, mai tradurlo automaticamente. Dimensioni del Canvas coerenti con la proporzione delle immagini pubblicate e non spacciate per dimensioni fisiche dell'opera.
- **Identità:** costruire gli URI pubblici dai UUID stabili dell'opera e dalle revisioni fotografiche selezionate, *non* da nomi, titoli o posizioni; mantenere URI stabili fra esportazioni verso la **stessa** destinazione, non assumere che lo stesso UUID autorizzi una pubblicazione su due domini. Se l'utente cambia URL base, mostrare le rotture di riferimenti attese.
- **Diritti:** un `copyright_status` di Media Center è una dichiarazione incompleta; un `rights` IIIF non va inferito automaticamente. Usare soltanto un URI di licenza/rights statement confermato dall'utente; altrimenti omettere il campo e mostrare avviso. Credito dell'artista e del fotografo restano ruoli distinti.
- **Contratti ipotetici:** `previewIiifExport(selection) -> Preview`, `validateIiifPackage(localFolder, expectedUrls) -> Report`, `exportIiifDraft(selection, folder) -> PackageResult`. Nessuna API di rete, scrittura su database o modifica delle fotografie originali è approvata.

## Offline, privacy, sicurezza, impostazioni, accessibilità e multilingua

- Tutta la selezione, produzione di JSON e generazione di copie avviene **offline**, senza account, API, login o cloud. La cartella può essere su disco/USB/NAS **già montato** dall'OS. Nessuna sincronizzazione o aggiornamento automatico della pubblicazione.
- Un pacchetto potenzialmente pubblico **non è un backup** e non deve contenere SQLite, dati personali dei clienti o l'intera libreria. Anteprima dei campi ed esplicita conferma di esportazione; impostazione “includi metadati personali” **non** globale. Gli URL non devono rivelare percorsi locali, token o nome utente del computer.
- File creati solo in una nuova sottocartella senza sovrascrivere dati di terzi; validare nomi, path traversal, symlink e collisioni Unicode/case, file fuori limite e interruzioni su USB/NAS. Ispezionare i metadata incorporati nei derivati (GPS/EXIF) secondo TECH-013; evitare script e HTML non fidati nei testi destinati a viewer, mantenendo escaping JSON corretto. Chi consegna all'istituzione è responsabile dell'autorizzazione alla pubblicazione.
- Impostazioni eventuali **locali e facoltative**: ultimo percorso di export, lingua dei testi pubblici, policy esplicita per i derivati, ultimo URL base per la stessa destinazione; nessun dominio, licenza, diritto o “pubblica automaticamente” predefinito. Non salvare credenziali.
- Lingua UI rilevata dal sistema, selezione manuale, **fallback inglese**. Distinguere UI IT/EN/ES/FR/DE dai titoli e descrizioni dell'opera; data/numero/misure leggibili nella preview, valori JSON secondo standard. Navigazione da tastiera, ordine focus, errori annunciati a screen reader, preview testuale equivalente alle immagini, nessuna informazione comunicata soltanto con colori.
- Per dimensioni grandi: generazione per file in streaming dove possibile, limiti memoria/tempo I/O, progresso/annullamento, stato “parziale” non scambiato per pacchetto completo. Non modificare o cancellare originali e loro revisioni.

## Dipendenze, alternative, rischi, complessità

**Dipendenze logiche:** dati opera e galleria attuali; veridicità dei collegamenti media, protezione delle esportazioni, controllo metadata e colore. Verificare compatibilità col [IIIF Presentation API 3.0](https://iiif.io/api/presentation/3.0/) e non confondere questa API con [IIIF Image API 3.0](https://iiif.io/api/image/3.0/). Il requisito di un HTTP(S) `id` significa che il **pacchetto offline da solo non basta** a soddisfare tutti i viewer; nessuna nuova libreria è decisa.

**Alternative reali da testare prima:**
1. Consegnare immagini scelte, CSV e brochure già esistenti; il museo effettua autonomamente la conversione nel proprio catalogo: **zero sviluppo Repertica**.
2. Usare un convertitore IIIF esterno o script locale ripetibile, eseguito dal partner, se le richieste sono rare.
3. Esportatore minimo di Manifest con asset statici, **solo se** più partner richiedono regolarmente dati IIIF; non costruire un server, un viewer, una piattaforma di publishing o un Image API.

**Complessità:** media per mappatura minima e validazioni strutturali, medio-alta per gestione robusta di pacchetti, privacy, immagini, identità e compatibilità multipiattaforma. Alta se si introducesse distribuzione di immagini, hosting o API dei tasselli: **fuori ambito**.

**Rischi:** manifest sintatticamente valido ma inutilizzabile perché URL non pubblicati, immagini pubblicate senza diritti, esposizione di metadata sensibili, perdita del legame fra revisioni e Canvas, dipendenza implicita da URL instabili, caching/indicizzazione di risorse pubblicate fuori dal controllo locale, falsa promessa di interoperabilità con ogni viewer. La convalida strutturale non garantisce supporto da parte di una specifica istituzione.

## Collaudi e criteri di accettazione futuri (solo ipotetici)

1. Archivi sintetici: un'opera con più foto, due opere in Collection, Unicode, testi in lingue diverse, diritti ignoti e campi assenti → output deterministico, solo campi selezionati e Canvas nell'ordine concordato.
2. Pacchetto senza URL base: **non** identificato come manifest online utilizzabile; nessuna rete avviata. Con URL HTTPS fornito manualmente: URI `id`, `Canvas`, `AnnotationPage`, `Annotation`, `Image` coerenti, nessun `file://` o placeholder occulto.
3. Validator indipendente conforme alla specifica IIIF 3.0 e prova con almeno **due viewer compatibili** dopo pubblicazione su un host di prova esplicitamente autorizzato; riportare separatamente esito **offline** ed esito **HTTP**.
4. Copie foto: hash degli originali invariati, niente RAW/certificati/note/contatti/prezzi estranei, GPS non inserito nelle copie pubblicabili secondo policy verificata. Assenza di dati diritti → nessuna licenza presunta.
5. Interruzione, cartella già esistente, unità scollegata, nomi Unicode, symlink e tentativi di path traversal: nessuna sovrascrittura né esportazione fuori destinazione. Su 1000 immagini sintetiche, memoria e durata misurate su Win/macOS/Linux senza fissare soglie arbitrarie.
6. UI con solo tastiera/screen reader, lingua non supportata → inglese, titoli originali non alterati. Backup e ripristino Repertica esistenti restano indipendenti.

## Fasi proposte, tutte da autorizzare separatamente

- **Fase 0, senza codice:** chiedere a 2–3 istituzioni partner se richiedono davvero IIIF Presentation 3, esempi di manifest attesi e condizioni di hosting/diritti; confrontare il costo con CSV+foto.
- **Fase 1 eventuale:** matrice campi interni → manifest IIIF e prototipo manuale **fuori dall'app** con dati fittizi, convalida e prova di viewer da parte del partner.
- **Fase 2 eventuale:** solo generazione offline di pacchetto ristretto, preview forte e test di privacy/compatibilità, senza pubblicazione automatica.
- **Fase 3 da nuova decisione:** Collection, aggiornamenti di URI e profili di esportazione per partner; nessun server IIIF Repertica per default.

## Questioni aperte al proprietario

1. Esistono musei, fondazioni, archivi o gallerie utenti che abbiano **chiesto IIIF** oppure basta una brochure/CSV? Quali client e versione della specifica usano?
2. Il partner gestirà dominio e immagini? Si vuole **solo preparare** un pacchetto offline o anche una specifica per chi lo pubblica autonomamente?
3. Quali campi dell'opera sono davvero pubblicabili senza revisioni manuali? Come distinguere diritti della fotografia da quelli dell'opera?
4. È opportuno mantenere gli URL pubblici fra esportazioni successive? Chi risponde di un'immagine rimossa dopo pubblicazione?
5. Meglio **nessuno sviluppo** finché un partner reale non fornisce un esempio e requisiti concreti?

> **Esito:** proposta esplorativa P3, non approvata né assegnata. Nessun codice o servizio online richiesto o avviato.