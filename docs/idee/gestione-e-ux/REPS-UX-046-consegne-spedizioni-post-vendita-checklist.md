# REPS-UX-046 — Consegne e spedizioni dopo la vendita: preparazione, affidamento e conferme dichiarate

- **ID stabile:** REPS-UX-046
- **Data della ricognizione:** 2026-11-22
- **Stato:** Proposta — da valutare
- **Filone:** Gestione e UX
- **Priorità ipotetica:** P2 soltanto per artisti e gallerie con consegne ricorrenti; P3 se bastano note e checklist esterne. Nessuna priorità approvata.
- **Perimetro:** studio esclusivamente ideativo/documentale. Non autorizza sviluppo, modifiche al prodotto, issue operative, assegnazioni o invii a terzi.

## 1. Problema, destinatari e riscontri verificabili

Per un artista che vende di persona o spedisce opere, il lavoro non termina quando registra la vendita: occorre proteggere l'opera, preparare documenti, concordare il ritiro, affidarla al trasportatore, documentare ciò che **si sa realmente** dell'esito e gestire eventuali eccezioni. Una vendita può essere saldata ma ancora in studio; una spedizione può risultare affidata al corriere ma non consegnata; un numero di tracking non prova il ricevimento. Senza una vista operativa, queste informazioni restano in messaggi, note e fogli separati.

**Destinatari potenziali, da verificare sul campo:** artisti che vendono direttamente, studi che imballano e inviano opere, piccole gallerie con ritiri su appuntamento; utilità inferiore per chi non gestisce direttamente la logistica.

**Riscontro statico sul branch `main` (22/11/2026; non test desktop):**
- `src-tauri/src/models.rs` e `src/app/models.ts` contengono vendite con `sale_id`, `artwork_id`, `collector_id`, data, importo e note; `SaleInput` contiene `shipping_cents` per il conteggio economico. **Il costo di spedizione registrato non è un evento logistico**.
- `docs/MVP.md` distingue stato commerciale dell'opera, vendita e incassi; resta una vendita per opera nell'MVP e non esiste un ciclo documentato di preparazione/affidamento/ritiro/consegna.
- `docs/SALE-RECEIPTS.md` e `docs/ECONOMY-I18N.md` distinguono importi venduti, ricevuti e spese registrate: nessuna di queste righe può attestare una consegna.
- `docs/DATA-PORTABILITY.md` elenca i registri CSV esistenti; non compare un registro delle consegne. Il backup verificato include SQLite e media; il CSV non è un backup ripristinabile.

**Grado di certezza:** presenza dei modelli e dei campi sopra = verificata nei file; necessità di un nuovo modulo e frequenza degli errori = **ipotesi da convalidare**. Non è stato osservato un bug né intervistato un utente.

## 2. Confini anti-doppione

- **[UX-001 — Provenienza e movimentazioni](REPS-UX-001-provenienza-movimentazioni.md):** deve restare l'eventuale fonte autorevole per **trasferimenti fisici effettivamente dichiarati, custodia e ubicazione**. UX-046 riguarda invece **la preparazione e il seguito operativo di una consegna collegata a una vendita**: checklist, appuntamento, imballaggi e stato dichiarativo. Una spunta «consegnato al corriere» non deve duplicare automaticamente un movimento; semmai futuro *riferimento* al record UX-001, se entrambe le proposte saranno approvate.
- **[UX-025 — Istruzioni per singola opera](REPS-UX-025-istruzioni-allestimento-cura-opera.md):** descrive **come** un'opera va imballata/maneggiata in generale, con versioni riutilizzabili. Qui si annota **se e quando** una specifica spedizione o un ritiro è stato preparato, senza presumere che l'istruzione sia stata eseguita.
- **[UX-009 — Progetti espositivi](REPS-UX-009-progetti-espositivi-checklist.md):** coordina scadenze di mostre e fiere, non l'evasione di una vendita a un acquirente.
- **[UX-012 — Conto vendita](REPS-UX-012-affidamenti-conto-vendita-rendiconti.md):** documenta condizioni commerciali con intermediari, non la consegna finale al destinatario.
- **[UX-013 — Resi](REPS-UX-013-resi-annullamenti-rettifiche-vendite.md)** e **[UX-043 — Assistenza post-vendita](REPS-UX-043-segnalazioni-assistenza-post-vendita-opere.md):** una consegna non riuscita o un reclamo sono *segnali* da gestire, non un reso, annullamento, riconoscimento di difetto o garanzia automaticamente accettati.
- **[UX-017 — Vendita multiopera](REPS-UX-017-vendita-multiopera-ripartizione-importi.md):** tratta la struttura commerciale di un accordo con più opere. Un pacco può contenere più opere e una vendita può richiedere più colli: l'eventuale modello logistico non deve presumere corrispondenza 1:1 fra vendita, opera e collo.
- **[UX-004 — Rapporti di condizione](REPS-UX-004-rapporti-stato-conservazione.md)** e **[UX-016 — Coperture assicurative](REPS-UX-016-valutazioni-coperture-assicurative-scadenze.md):** un controllo dell'imballaggio non è una perizia; la presenza di un riferimento a polizza non prova copertura.
- **[TECH-016 — Allegati multi-entità](https://github.com/Cesare-ml/Repertica-Studio/blob/docs/idee-strumenti-architettura-20261121/docs/idee/strumenti-e-architettura/REPS-TECH-016-allegati-documentali-locali-multi-entita.md)** e **[TECH-001 — Portabilità](https://github.com/Cesare-ml/Repertica-Studio/blob/docs/idee-strumenti-architettura-20261121/docs/idee/strumenti-e-architettura/REPS-TECH-001-pacchetto-dati-aperto.md)** riguardano l'infrastruttura, non il processo di consegna; restano indipendenti.
- Confrontati gli indici completi delle **45 schede UX** e delle **41 schede tecniche** della [PR #16](https://github.com/Cesare-ml/Repertica-Studio/pull/16). Nessun file tecnico viene modificato.

**Esclusioni:** acquisto etichette dai corrieri, stima tariffe, monitoraggio automatico via API, prenotazioni logistiche online, documenti fiscali, autenticazione, geolocalizzazione, firma digitale, verifica giuridica della consegna, assicurazione automatica o modifica dello stato commerciale dell'opera.

## 3. Scenari d'uso e microflussi UX

1. **Ritiro personale dopo vendita.** La vendita viene registrata; l'opera rimane nello studio. Da **Vendita → Consegna** l'artista sceglie «Ritiro concordato», data/ora locale indicativa e checklist essenziale (opera controllata, certificato preparato, imballo pronto). Dopo il ritiro registra «Consegna dichiarata dall'operatore» e, se lo desidera, riferimento a una ricevuta sottoscritta **senza dichiararne autenticità**. Se il cliente non arriva, la data passa a «da riprogrammare» senza cambiare vendita o incassi.
2. **Spedizione con corriere.** L'artista seleziona «Spedizione», inserisce vettore e riferimento di tracking **come testo facoltativo**, data di affidamento effettivamente dichiarata, eventuali colli e documento di trasporto riferito. L'interfaccia dice «Affidato al corriere, esito non confermato», non «Consegnato». Quando riceve una comunicazione, può registrare «Consegna comunicata dal vettore» con fonte e data, oppure «Esito da verificare».
3. **Destinatario diverso dall'acquirente.** Un gallerista compra per un collezionista e indica un restauratore come recapito di ritiro. La schermata mostra chiaramente «Acquirente della vendita» e «Destinatario di questa consegna» **distinti**. Non riscrive la scheda collezionista e non deduce titolarità.
4. **Collo rifiutato o tornato indietro.** L'utente annota l'evento «Tentativo fallito» o «Rientro comunicato», conserva date/fonti e decide se creare un nuovo tentativo. Non genera un reso commerciale, rimborso, nuovo costo o movimentazione verificata.
5. **Archivio storico.** Una vendita precedente non ha dati di consegna: mostra «Nessuna consegna registrata», **mai** «Da consegnare» o «Consegnata», salvo un'annotazione esplicita.

**Flusso minimo proposto:** `Vendite → Dettaglio → Preparazione/consegna (facoltativo) → Modalità ritiro/spedizione → Checklist modificabile → Registra evento dichiarato → Riepilogo`. Lista opzionale «Consegne da verificare», con filtro per modalità, scadenza e ultimo evento. Completamento di una checklist **non** provoca transizioni di stato. Nessuna richiesta di compilare questo pannello quando si registra una vendita.

**Accessibilità:** navigazione e riordino delle checklist disponibili da tastiera senza trascinamento obbligatorio, step e stati nominati in testo, data/ora con etichette e fuso visibile, avvisi leggibili da screen reader, focus su campo in errore, layout mobile-friendly anche nelle finestre desktop strette, stampa contrastata.

## 4. Dati descrittivi, vincoli e ipotesi di API (nessuna implementazione)

**Prima ipotesi leggera:** `delivery_case` (UUID, riferimento `sale_id`, `artwork_id` solo per consultazione, modalità `pickup|carrier|other`, data prevista facoltativa, destinatario libero **solo se necessario**, indirizzo confermato dall'utente, note operative, creato/aggiornato). Per la sola preparazione si potrebbe iniziare con flag/checklist e annotazioni, senza stati formali.

**Se servono eventi multipli:** `delivery_event` (UUID, caso UUID, `event_type`, `occurred_at` oppure data senza ora, grado di certezza `planned|declared|reported_by_third_party|unverified`, fonte testuale, riferimento documento o allegato locale facoltativo, `created_at`, note). Esempi distinti: `packed_declared`, `handed_to_carrier_declared`, `pickup_declared`, `delivery_reported`, `attempt_failed`, `returned_reported`. Nessun booleano `is_delivered` dedotto da cronologia incompleta; il riepilogo è descrittivo e segnala incongruenze.

**Solo se giustificato:** `delivery_package` (UUID, caso UUID, identificativo locale, numero collo, peso/dimensioni *dichiarati*, vettore/riferimento testo, nessun costo contabilizzato). Non anticipare una gestione spedizioni multiopera/multicollo in fase A. Se si introdurranno vendite multiopera, progettare correlazioni esplicite a righe vendita e pacchi, **senza cambiare UUID storici o seriali certificati**.

**Invarianti:** una vendita resta fonte di importi commerciali; `sale_receipts` resta fonte degli incassi manuali; `sale_finances.shipping_cents` è voce finanziaria già registrata **non** il prezzo del corriere da registrare di nuovo. Un evento logistico non crea spese, certificati, resi, modifiche allo stato `artworks.status`, né trasferimenti UX-001. Cancellazioni e rettifiche devono conservare le comunicazioni storiche significative e non produrre orfani. Qualunque API futura dovrebbe esplicitare operazioni separate `list_delivery_cases`, `add_delivery_event`, `export_delivery_report` (nomi illustrativi, **non** contratti approvati).

## 5. Impostazioni, offline, multilingua, privacy, import/export

- **Impostazioni:** percorso di creazione come pannello non obbligatorio; checklist suggerite modificabili per ritiro o spedizione (es. imballo, foto, certificato, recapito confermato), eventuale soglia per mostrare «da verificare». Niente vettore preferito, tracking attivo o configurazione esterna necessari. Il costo in `shipping_cents` non va copiato automaticamente fra registri.
- **Offline-first:** pianificazione, eventi e ricerca restano nel database SQLite locale; allegati eventualmente acquisiti nel media store e inclusi nei backup completi. Il codice tracking è una stringa: niente interrogazione corrieri o geocoding, neppure al semplice avvio. Con app chiusa nessuna promessa di notifica o controllo automatico.
- **Lingue:** lingua del sistema operativo tramite meccanismo già esistente, scelta manuale eventuale IT/EN/ES/FR/DE, **fallback inglese** per lingue/stringhe non supportate. Tradurre stati e differenza tra «pianificato», «dichiarato», «comunicato dal vettore», «verificare». Indirizzi, annotazioni, vettori e testi scritti dall'utente non vengono tradotti. Date ISO internamente, visualizzazione locale con precisazione di data/ora non certa.
- **Privacy:** indirizzo di consegna, telefono del destinatario e riferimenti di tracking possono identificare persone e abitazioni. Proporre **minimizzazione**: non copiare automaticamente indirizzo/contatti del collezionista in documenti stampati o esportati; quando serve usare un recapito, richiedere conferma e permettere destinatario diverso senza sovrascrivere la scheda principale. Escludere note private e recapiti dagli export di default; anteprima prima della condivisione. Copie di documenti sono potenzialmente riservate. Backup attuali **in chiaro**: non presentare come cifrati.
- **Import/export:** fase iniziale CSV UTF-8 e semantica esplicita (`delivery_cases.csv` / `delivery_events.csv`) con UUID, stato dichiarativo, data ISO e campi sensibili esclusi di default; scelta cartella, conferma e **nessuna sovrascrittura**. L'export senza recapiti è distinto dall'esportazione completa dei dati dell'archivio. Round-trip soltanto futuro: JSON versionato e allegati con hash/percorso relativo nel perimetro di un pacchetto portabile validato, anteprima conflitti, nessun evento duplicato o prova inventata. I CSV attuali sono **solo esportazioni**, non procedure di import. Un file salvato fuori dall'archivio non entra automaticamente nel backup; l'archivio principale sì.
- **Compatibilità:** iniziare da vendite esistenti senza migrare lo stato di consegna: per tutte **«nessun dato registrato»**. Nessuna modifica di `sales`, `collectors`, `sale_finances`, ricevute o snapshot certificati. Eventuali nuove tabelle richiederebbero migrazione additiva e versione SQLite aggiornata, da valutare, non sono approvate. Il flusso operativo non dipende dall'eventuale adozione di UX-001, UX-017 o TECH-016.

## 6. Alternative, rischi, complessità e dipendenze

**Alternativa preferibile se il bisogno è occasionale:** una **nota datata sulla vendita** e una piccola checklist stampabile, senza nuove tabelle, stati o campi di spedizione. Prima di progettare un modulo ampio raccogliere almeno esempi di consegne saltate, richieste di prova, ripetizioni e tempi persi.

**Rischi principali:** presentare tracking come prova di recapito; spuntare «imballato» ma non aver preparato il pacco; far coincidere acquirente e destinatario; propagare dati personali a documenti condivisi; calcolare due volte la spedizione; duplicare un registro di movimentazioni se UX-001 verrà implementata; stati troppo rigidi in caso di ritiro, spedizione parziale, colli multipli o rientro. Mitigazioni: fonti esplicite, nessuna conferma automatica, interfaccia semantica prudente, riferimenti anziché copie di eventi autorevoli, esportazioni minimizzate, casi sconosciuti sempre ammessi.

**Complessità indicativa:** bassa–media per promemoria/checklist; **media** per eventi, riferimenti e allegati locali; **alta** per multi-collo/multiopera, integrazione con movimenti e casi di spedizione internazionale. Dipendenze logiche: vendite/collezionisti esistenti, SQLite/backup/i18n; allegati strutturati e storico movimenti sono solo **possibili** punti d'integrazione con proposte non approvate, non prerequisiti.

## 7. Criteri di accettazione ipotetici e fasi

1. Una vendita **non** crea un evento di consegna; una vendita storica senza dati logistici resta «non documentata».
2. Una consegna pianificata o una checklist completata **non** aggiorna ubicazione, proprietà, stato commerciale, certificati, spese, importi venduti, incassi o quote.
3. Un tracking inserito a mano non attiva richieste di rete, non viene validato come consegna e rimane modificabile con storia della correzione.
4. Il destinatario può differire dall'acquirente senza alterare la rubrica; la stampa/condivisione mostra soltanto campi scelti e una chiara fonte del dato.
5. Tentativi falliti, rinvii e dichiarazioni contraddittorie conservano il loro contesto senza trasformarsi automaticamente in reso o rimborso.
6. Simulazioni su un archivio storico e su backup/ripristino conservano tutti i dati preesistenti, con gli stessi UUID, seriali e importi.
7. CSV/JSON (solo se introdotto) distinguono date di pianificazione, eventi riferiti da terzi e fatti manualmente dichiarati; import prova in sandbox senza duplicati né scritture non richieste.
8. Interfaccia e messaggi funzionano da tastiera/screen reader, con lingua OS e fallback inglese; senza rete, account o abbonamento.

**Fase 0 — Validazione reale:** 3–5 interviste mirate a chi vende e consegna di persona o spedisce; frequenza/conseguenze; confronto con note già esistenti e modulo cartaceo. Possibile esito: **non implementare**.

**Fase A — Solo se confermata:** checklist leggera opzionale in dettaglio vendita con stato puramente informativo e nota datata; nessuna automatizzazione di movimenti.

**Fase B — Solo se emergono più consegne e incidenti:** cronologia di eventi con fonte e vista «da verificare», recapiti minimizzati, export locale. Valutare collegamento **solo referenziale** a UX-001 quando disponibile.

**Fase C — Facoltativa e non presunta:** multi-collo, documenti allegati e rapporti riutilizzabili; affrontare multiopera UX-017 soltanto con casi d'uso e vincoli chiariti. Nessuna integrazione corrieri prevista automaticamente.

## 8. Requisiti certi, ipotesi e domande per revisione del proprietario

- **Certi, dal repository:** desktop Tauri/SQLite locale, nessun account/abbonamento obbligatorio, backup verificati in cartella scelta, vendite e incassi separati, costo di spedizione economico distinto, UI multilingua con fallback inglese, vendita singola per opera al momento della ricognizione.
- **Ipotesi da convalidare:** esiste una frequenza sufficiente di spedizioni/ritiri complessi; uno scadenziario di consegna in Repertica sarebbe preferito rispetto a note/calendario; utenti desiderano conservare documenti e tracking nel proprio archivio; la gestione operativa vale il costo di complessità.
- **Domande aperte per il proprietario (nessuna risposta richiesta in questa esecuzione):** Repertica deve avere solo un promemoria post-vendita oppure uno storico formale per consegne multiple? Occorre distinguere consegna all'acquirente e a un destinatario terzo? Quale documento viene usato davvero come prova nella pratica corrente? È prioritario evitare un altro pannello per chi non spedisce? In caso di approvazione futura di UX-001, chi è la fonte unica del movimento fisico? È necessario supportare più colli o una semplice nota è sufficiente?

**Decisione attuale:** **Proposta — da valutare**. Nessuna funzionalità approvata, assegnata o avviata allo sviluppo.
