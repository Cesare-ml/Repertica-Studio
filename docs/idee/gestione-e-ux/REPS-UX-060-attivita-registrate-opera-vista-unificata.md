# REPS-UX-060 — Attività già registrate per opera: vista unificata, senza inventare una storia completa

- **ID stabile:** REPS-UX-060
- **Data:** 2026-11-23
- **Stato:** Proposta — da valutare
- **Filone:** Gestione e UX (esclusivamente documentale)
- **Priorità proposta:** P2 **solo se** l'uso reale dimostra il costo di consultare sezioni separate; P3 o nessuno sviluppo se bastano i collegamenti attuali
- **Decisione:** nessuna approvazione, assegnazione, issue operativa o autorizzazione allo sviluppo

## 1. Motivazione e destinatari

**Esigenza da verificare.** Quando un artista o un archivista riceve una domanda su una specifica opera («Quando è stata registrata la vendita?», «Sono stati annotati incassi?», «Quale certificato risulta emesso?», «Quando è stata aggiunta la fotografia?»), deve passare fra Archivio, Vendite/incassi, Certificati e Galleria. Una vista di **attività che risultano attualmente registrate** potrebbe ridurre passaggi e confusioni, senza creare un nuovo registro gestionale né fingere una tracciabilità storica di ciò che non è stato conservato.

**Destinatari da intervistare:** artisti con decine/centinaia di opere e vendite; collaboratori di studio che rispondono a richieste operative; piccole gallerie e archivi che incrociano un'opera con documenti e rapporti economici. Per archivi piccoli o consultati raramente la complessità aggiuntiva potrebbe non valere.

**Evidenze statiche su `main`, lette il 23/11/2026:** `README.md`, `docs/MVP.md`, `docs/SALE-RECEIPTS.md`, `src/app/models.ts`, `src/app/app.component.ts`, `src/app/app.component.html` e `src-tauri/src/db.rs`. Sono presenti UUID e collegamenti per opere, vendite, incassi e certificati; la UI mostra registro vendite, incassi nel relativo dialogo, certificati e Galleria in punti separati. `Artwork` ha `created_at`/`updated_at`, `Sale` ha `sold_at`, `SaleReceipt` ha `received_at` e `created_at`, `Certificate` ha `issued_at`, e `GalleryImage` ha data di inserimento. **Non** è stato eseguito un collaudo desktop e **non** è dimostrato che questa navigazione sia un problema ricorrente. Non è stata verificata una cronologia immutabile generale delle operazioni: la proposta non deve sostenerne l'esistenza.

## 2. Confini e coordinamento per evitare doppioni

Questa proposta riguarda **solo una vista di consultazione e navigazione di record già esistenti**, non la creazione di nuovi eventi storici:
- **[UX-001 — Provenienza, ubicazioni e movimentazioni](REPS-UX-001-provenienza-movimentazioni.md)**: eventuali trasferimenti fisici, possesso e fonti restano dominio proprio. La vendita **non** prova la consegna, e il campo ubicazione corrente non costituisce storico di spostamenti.
- **[UX-026 — Diario di produzione](REPS-UX-026-diario-lavorazione-opera-fasi-datate.md)**, **[UX-019 — Mostre e bibliografia](REPS-UX-019-storico-esposizioni-bibliografia-documentata.md)**, **[UX-045 — Rettifiche dei certificati](REPS-UX-045-rettifica-sostituzione-certificati-opere.md)**: queste proposte introducono registrazioni professionali distinte; la vista REPS-UX-060 non le sostituisce né anticipa.
- **[UX-050 — Rendiconto mensile](REPS-UX-050-rendiconto-mensile-snapshot-verifiche.md)**: aggrega importi per periodo, non fornisce navigazione cronologica per singola opera.
- **[TECH-010 — Revisioni dei metadati](https://github.com/Cesare-ml/Repertica-Studio/blob/docs/idee-strumenti-architettura-20261121/docs/idee/strumenti-e-architettura/REPS-TECH-010-storico-revisioni-metadati.md)**: storia di *valori modificati* e recupero selettivo, attualmente soltanto proposti. Un valore `updated_at` non spiega quali campi siano cambiati e **non** deve trasformarsi in una voce «modifica avvenuta» probante.
- **[TECH-053 — Storia revisioni dei media](https://github.com/Cesare-ml/Repertica-Studio/blob/docs/idee-strumenti-architettura-20261121/docs/idee/strumenti-e-architettura/REPS-TECH-053-manifest-aperto-storia-revisioni-media.md)**: riguarda la consegna portabile dei binari e delle revisioni fotografiche, non questa navigazione gestionale.
- **[UX-054 — Presentazione riservata](REPS-UX-054-vista-presentazione-riservata-in-studio.md)**: proteggere una presentazione dal mostrare numeri e clienti è un problema diverso. Questa vista è **interna**: non va aperta automaticamente durante una presentazione dell'opera.
- Il filone tecnico **[PR #16](https://github.com/Cesare-ml/Repertica-Studio/pull/16)** resta intatto. Nessuna tabella di eventi professionali duplicata, nessun audit log e nessun cambiamento alle sorgenti autorevoli.

## 3. Scenari d'uso e flussi UX

1. **Richiesta sul certificato.** Dall'opera «ART-...», l'utente apre `Attività registrate`, trova «Certificato n. ... emesso il ...» e apre il certificato **storico** con la sua snapshot; non si presume che i dati attuali dell'opera siano gli stessi.
2. **Vendita rateale.** Per una vendita del 10 ottobre e due incassi dichiarati il 15 ottobre e il 5 novembre, la vista indica **una vendita** e **due incassi registrati**, con date, importi e un'etichetta che chiarisce «incasso dichiarato, non verificato in banca». Non presenta una rata futura come incassata né deduce stato di consegna.
3. **Scheda catalogata dopo la produzione.** Un'opera prodotta nel 2018 ma registrata nell'app nel 2026 mostra «Scheda creata nell'archivio nel 2026», **non** «Opera creata nel 2026».
4. **Archivio incompleto o storico importato.** Se una vendita non ha ricevute registrate, la vista non dice «mai pagata»: mostra «Nessun incasso registrato» e rinvia al registro autorevole. Se un incasso è stato eliminato, la vista non ricostruisce falsamente un evento cancellato.
5. **Fotografia e data.** Per una foto di galleria, mostra «Fotografia aggiunta al catalogo» con la data di inserimento, **non** data dello scatto; la foto di copertina non deve essere inclusa con una data ipotizzata.
6. **Presentazione riservata.** L'utente può chiudere rapidamente la vista e tornare alla scheda; nessun importo e nessun nome cliente sono proiettati automaticamente nel visualizzatore a tutto schermo.

**Percorso minimo:** Archivio → seleziona opera → `Attività registrate` (pannello facoltativo nella scheda, non nuova voce globale) → elenco compatto con tipo, data, fonte, descrizione e azione `Apri registro` → scheda originale → `Torna all'opera`. L'azione di ritorno deve preservare ricerca, filtri e posizione di scorrimento.

**Presentazione:** predefinita dalla più recente, con opzione ordine cronologico; sezioni facoltative `Catalogazione`, `Vendita`, `Incassi`, `Certificati`, `Fotografie`. Due date restano separate: **data dichiarata dell'evento** e **data della registrazione** quando disponibile. L'ordine dei record con pari data è stabile e dichiarato. Non ordinare eventi per numero progressivo di certificato o posizione delle fotografie.

**Stati vuoti e anomalie:** «Nessuna registrazione in questa categoria»; «Data non disponibile»; «Dato registrato successivamente, data riferita all'evento»; «Record di origine non più disponibile» soltanto se esiste un riferimento realmente osservabile, non un'ipotetica tombstone. Filtri non devono creare l'illusione che non ci siano altri dati.

**Accessibilità:** elenco HTML semantico, etichette testuali oltre alle icone/colori, uso con tastiera e screen reader, focus ripristinato dopo il ritorno, lettura distinguibile di date e importi, zoom 200%, niente drag-and-drop, animazioni non necessarie, contrasto e stampa leggibili.

## 4. Dati, regole e confini di verità (solo ipotesi descrittive)

**Fase minima preferibile:** vista **calcolata in sola lettura** dai registri esistenti con i rispettivi UUID; nessuna nuova tabella di «storia». Un modello UI concettuale, **non schema/API approvata**, potrebbe essere:

```text
ArtworkActivityItem {
  stable_key: "<tipo-record>:<uuid-sorgente>",
  artwork_id: uuid,
  source_type: "artwork_catalog_entry" | "sale" | "sale_receipt"
             | "certificate" | "artwork_gallery_entry",
  source_id: uuid,
  occurred_on: date_or_timestamp | null,   # significato specifico del tipo
  registered_at: timestamp | null,         # solo se realmente disponibile
  date_kind: "catalogued_at" | "sold_at" | "received_at"
           | "issued_at" | "image_added_at",
  confidence_label: "registrazione_locale" | "data_dichiarata",
  private_summary: object,                # mai pubblicato per default
  source_navigation: source_type + source_id
}
```

Un eventuale comando `list_artwork_activity(artwork_id, filters, cursor)` è **solo un esempio di contratto** per una valutazione futura. Preferire una lettura SQLite coerente per evitare metà elenco prima e metà dopo una modifica concorrente; la lettura non deve scrivere su tabelle esistenti. Eventuali record di origine aggiunti da altre idee devono essere integrati **solo dopo approvazione autonoma del relativo modello**, senza popolare nulla retroattivamente mediante supposizioni.

**Regole imprescindibili:** la fonte della verità per importi, percentuali, rimanenze, intestatari e date rimane il registro di origine; niente calcoli alternativi nella timeline; la vendita non prova pagamento o consegna; l'incasso non prova l'accredito bancario; la certificazione non prova autenticità indipendente. Non esibire un «evento di modifica» dalla sola colonna `updated_at`. Le date retrodatate non sono date di inserimento; l'assenza di righe non costituisce prova che qualcosa non sia avvenuto. La vista è **fotografia dello stato corrente dell'archivio**, non una pista di audit completa.

## 5. Impostazioni, offline, lingue, privacy, import/export

- **Offline:** sola lettura da SQLite/media locali, senza rete, account, sincronizzazione, cloud, notifiche automatiche o interrogazioni a servizi esterni. Nessun cambio del percorso del backup a cartella scelta e nessuna dipendenza dall'MCP.
- **Preferenze:** apertura manuale; ricordare eventualmente *solo sul dispositivo* l'ultimo filtro e ordine, senza introdurre configurazione obbligatoria. Prima fase senza impostazioni nuove. Nessun refresh che riveli dati privati in una finestra di presentazione.
- **Multilingua:** lingua di sistema, fallback inglese, eventuale scelta manuale già disponibile; etichette e formati locali per IT/EN/ES/FR/DE. I codici dei tipi evento, UUID e importi in centesimi rimangono invarianti. Titoli e note scritti dall'utente **non** vengono tradotti automaticamente.
- **Import:** la vista non importa nulla, non modifica dati e non fonde archivi. I dati importati tramite futuri meccanismi autorizzati possono comparire solo se possiedono i normali record e identificativi validi del registro d'origine; nessun evento ricostruito per analogia.
- **Export, solo fase successiva se richiesto:** CSV UTF-8 limitato alla singola opera con `source_type`, `source_id`, `date_kind`, data ISO e descrizione minimale; alternativa JSON aperto versionato. Anteprima, selezione esplicita dei tipi, salvataggio in una cartella scelta dall'utente, nessun invio, nessuna sovrascrittura silenziosa; escludere per default nomi collezionisti, note, numeri di riferimento dei pagamenti e immagini. L'esportazione è **report**, non backup, cronologia completa o prova certificata.
- **Privacy:** la vista interna può contenere clienti, importi e informazioni di pagamento; visibilità soltanto dopo azione dedicata, mascheramento di dettagli sensibili nell'anteprima e nessuna stampa/condivisione predefinita. L'app non dispone di autenticazione per ruoli: un toggle non è una misura di controllo accessi. I backup attuali sono in chiaro: non sostenere che la cronologia aggiunga protezione.
- **Portabilità e compatibilità:** fase minima senza migrazione SQLite, senza cambiare gli ID, i CSV esistenti, i certificati immutabili e i backup. Archivi storici mostrano solo eventi effettivamente presenti. Se l'API nativa richiedesse nuove letture, i vincoli Rust resterebbero quelli attuali e nessuna ricalcolazione storica è consentita.

## 6. Complessità, dipendenze, rischi e alternative

**Complessità proposta:** bassa-media per un elenco di eventi già presenti, media per navigazione coerente e accessibile, più alta per export minimizzato o supporto a registri futuri. Non è una stima di effort validata.

**Dipendenze concettuali:** UUID e FK degli attuali registri; semantica delle date; permessi di accesso ai dati locali; traduzioni, navigazione Angular, coerenza letture SQLite. Nessuna dipendenza obbligatoria da cloud, TECH-010, UX-001, UX-026 o nuovi documenti. Se si introducono eventi da tali proposte, preservare i loro ID e le loro definizioni, senza un secondo modello autorevole.

**Rischi principali:** attribuire a una «timeline» completezza, autenticità o certezza; confondere data dichiarata/data di inserimento; mostrare dati privati durante una visita; confondere incassi con venduto; deterioramento prestazionale su archivi grandi; rottura del ritorno alle liste; cronologia apparentemente immutabile benché alcuni record possano essere eliminati. Mitigare con titolo prudente, etichette visibili, salti verso le fonti, dataset sintetici e limite agli eventi caricati per pagina se necessario.

**Alternative prima dello sviluppo:** (A) mantenere sezioni attuali e aggiungere soltanto collegamenti incrociati alle schede pertinenti; (B) una card riepilogativa con numeri e link (una vendita, X incassi, Y certificati) senza timeline; (C) consultazione tramite CSV esistenti in LibreOffice Calc. La via B può essere superiore se l'ordine temporale non è davvero utile.

## 7. Accettazione ipotetica e fasi

**Prima validazione senza codice:** con 4–6 professionisti, 6 scenari sintetici (venduto/non pagato, pagato a rate, certificato storico, foto aggiunta, record assente, dati retrodatati). Misurare passaggi e incomprensioni nella UI attuale rispetto a prototipo cartaceo o link simulati; verificare che capiscano «attività registrate» e non la interpretino come storia completa. Se il bisogno non ricorre, **non sviluppare**.

**Criteri di accettazione, solo se l'idea verrà approvata:**
1. Tutti gli elementi hanno record sorgente consultabile e chiave stabile; zero eventi dedotti da status, filename, `updated_at` o assenza di righe.
2. Vengono etichettate correttamente data dichiarata, inserimento e significato di ogni evento; contenuti retrodatati e date pari producono ordine deterministico.
3. Una vendita con due incassi mostra 1 vendita e 2 incassi, senza duplicare importi, rate pianificate o supposte consegne; i certificati aprono lo snapshot originale.
4. Un archivio che non registra incassi non viene qualificato come «mai pagato»; nessuna transazione o media cambia dopo la lettura.
5. La vista è navigabile da tastiera, leggibile con screen reader e a zoom 200%; dopo «Torna all'opera» si mantengono i filtri dell'elenco.
6. Nessun dato cliente o importo viene esportato o mostrato in una vista di presentazione senza selezione esplicita; nessuna comunicazione esterna.
7. Le prestazioni sono valutate con centinaia/migliaia di opere e i relativi record; paginazione se necessaria; backup, CSV e schema anteriori restano compatibili.

**Fasi ipotetiche:** 0. ricerca e alternativa B; 1. sola consultazione dei record autorevoli e collegamenti; 2. filtri e accessibilità affinati, solo se utili; 3. export selettivo privacy-by-default, solo se richiesto; 4. eventuale aggregazione di *nuovi* domini dopo decisioni indipendenti. Nessuna fase assegnata.

## 8. Requisiti certi, ipotesi e decisioni riservate al proprietario

**Requisiti certi (principi e caratteristiche lette in `main`):** software gratuito/open source Tauri, database locale/offline-first; niente account/abbonamenti obbligatori; backup verificabili nella cartella scelta; dati esportabili; lingua di sistema e fallback inglese. Gli attuali domini archivio/vendita/incassi/certificati/galleria sono distinti e collegabili tramite ID. La vista non può cambiare registri, numeri di certificato, importi, quote di guadagno o proprietà dell'opera.

**Ipotesi:** è abbastanza frequente cercare più registrazioni della *stessa opera* da giustificare una vista; il valore supera quello di collegamenti contestuali; l'aggregazione è comprensibile senza percezione ingannevole di uno storico completo.

**Domande per la revisione personale (nessuna risposta presunta):** preferire un semplice pannello con link o una lista cronologica? Gli importi degli incassi sono davvero necessari nella prima vista, oppure solo la presenza di registrazioni? In quali flussi concreti l'utente cerca contemporaneamente certificato, vendita e foto? È sufficiente evitare l'export? Quale dimensione di archivio rappresenta il pubblico principale?

---

*Documento di sola analisi. Stato permanente fino a decisione espressa: «Proposta — da valutare». Non modifica né autorizza modifiche a software, dati o automazioni.*
