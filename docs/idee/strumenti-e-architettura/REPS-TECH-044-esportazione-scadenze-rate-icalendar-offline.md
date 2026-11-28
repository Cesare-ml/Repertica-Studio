# REPS-TECH-044 — Esportazione manuale delle scadenze rateali in iCalendar, senza sincronizzazione obbligatoria

- **ID stabile:** REPS-TECH-044
- **Data della ricognizione:** 2026-11-22
- **Stato:** **Proposta — da valutare**
- **Filone:** Strumenti e architettura (esclusivamente studio e documentazione)
- **Priorità ipotetica:** **P2–P3**, subordinata alla verifica di casi d'uso reali; P3 o nessuno sviluppo se la consultazione interna e un promemoria manuale bastano.
- **Decisione:** nessuna funzionalità approvata, nessuna implementazione autorizzata, nessuna assegnazione o modifica al prodotto.

## 1. Motivazione, destinatari e riscontri

**Problema ipotetico da verificare:** un artista o una piccola galleria che vende opere a rate registra correttamente il piano in Repertica, ma utilizza anche un calendario di lavoro indipendente. Trascrivere manualmente le date nel calendario può creare omissioni; condividere l'intero CSV degli incassi con un collaboratore sarebbe spesso eccessivo. Un file locale **iCalendar `.ics`**, prodotto solo su comando, consentirebbe di trasferire **appuntamenti informativi** verso programmi di calendario, senza chiedere account, abbonamenti, accesso alla rubrica o collegamento a un servizio online.

**Fatti verificati nel `main` il 22/11/2026:**
- [`src-tauri/src/installments.rs`](../../../src-tauri/src/installments.rs) contiene già `sale_installments`, con `id`, `sale_id`, `position`, `due_date` nel formato `YYYY-MM-DD`, `amount_cents`, note e uno stato calcolato da `sale_receipts`; le rate sono **2–60** e gli incassi reali sono registrati separatamente.
- `list_with_status` restituisce `PAID`, `PARTIAL`, `OVERDUE`, `DUE`: sono **classificazioni gestionali ricavate da dati dichiarati**, non una conferma bancaria. `replace_plan` elimina le righe precedenti e le reinserisce con nuovi UUID quando si sostituisce l'intero piano: **un UUID di rata non è oggi un identificatore di calendario stabile dopo una revisione**.
- [`docs/MVP.md`](../../MVP.md), [`docs/SALE-RECEIPTS.md`](../../SALE-RECEIPTS.md) e [`docs/DATA-PORTABILITY.md`](../../DATA-PORTABILITY.md) descrivono rate, incassi, registri CSV e backup locale; non documentano un'esportazione iCalendar delle scadenze.
- La presenza di un'esportazione .ics **non è stata richiesta o convalidata da utenti intervistati**; nessuna perdita di scadenze è stata osservata, né è stato eseguito un test nativo.

**Destinatari da verificare:** artisti che vendono direttamente con pagamenti dilazionati, studi con incaricati amministrativi, gallerie che ricontrollano le scadenze dal calendario di lavoro. Utilità limitata per chi non usa piani rateali o consulta Repertica quotidianamente.

**Riferimento aperto:** [RFC 5545 — Internet Calendaring and Scheduling Core Object Specification](https://www.rfc-editor.org/rfc/rfc5545), formato iCalendar. L'importazione di un file è un'istantanea, **non** un abbonamento o una sincronizzazione: [documentazione Microsoft per Outlook](https://support.microsoft.com/en-us/outlook/import-or-subscribe-to-a-calendar-in-outlook-com-or-outlook-on-the-web). Non promettere compatibilità universale senza prove su programmi reali.

## 2. Confini e confronto anti-doppioni

- **[UX-024 — Scadenziario degli incassi](https://github.com/Cesare-ml/Repertica-Studio/blob/docs/idee-gestione-ux-20261121/docs/idee/gestione-e-ux/REPS-UX-024-scadenziario-incassi-piani-pagamento-vendite.md):** la gestione delle rate e del saldo è **già nel prodotto**; UX-024 studia eventualmente la storia degli accordi. **Questa scheda non crea scadenze né le modifica:** si limita all'interoperabilità del dato esistente, una copia per il calendario.
- **UX-037** (spese ricorrenti), **UX-009** (mostre/fiere), **UX-046** (consegne): riguardano scadenze **potenziali di altri domini** e non devono essere materializzate o esportate come se già esistessero nel database. La prima fase legge **soltanto le rate delle vendite già registrate**.
- **TECH-001** (pacchetto aperto importabile), **TECH-018** (CSV atomici), **TECH-025** (esportazione tabellare con privacy) e **TECH-040** (vCard per contatti) coprono **altri formati o problemi**. Il file .ics è un calendario di date, **non un backup, un CSV, una rubrica o un pacchetto di interoperabilità completo**.
- **TECH-002** (automazioni locali) studia un motore generico autorizzato; qui **nessuno scheduler, notifiche automatiche, invio di email, evento di sistema o servizio residente**.
- **TECH-014/035/036** (MCP) non entrano nel flusso: nessuna lettura di scadenze da parte di assistenti esterni per default.
- Confrontati gli indici di **43 schede tecniche precedenti** e **48 proposte Gestione e UX** nelle PR [#16](https://github.com/Cesare-ml/Repertica-Studio/pull/16) e [#14](https://github.com/Cesare-ml/Repertica-Studio/pull/14). Non sono state trovate schede già dedicate all'**esportazione manuale iCalendar delle rate**. Non si modifica nessun file dell'altro filone.

## 3. Scenari d'uso, comportamento e flussi UX

1. **Un promemoria personale senza dati commerciali.** L'artista seleziona le rate ancora da verificare per i prossimi 90 giorni e salva `.ics` in una cartella scelta. Nel calendario appare «Verifica scadenza Repertica» alla data prevista, **senza nome del cliente, titolo dell'opera, importo, note o percorso di file**.
2. **Calendarizzazione informativa per una galleria.** Dopo aver verificato il contenuto e solo con scelta esplicita, il professionista può includere un riferimento interno leggibile (es. codice dell'opera) o l'importo, consapevole che i calendari di terzi possono sincronizzarsi online.
3. **Rata già incassata.** Per default la copia include solo voci con stato corrente `DUE` o `PARTIAL` e data futura/odierna; può includere `OVERDUE` soltanto con scelta chiara. `PAID` è esclusa di default. Un evento in un calendario **non diventa automaticamente pagato** e una rata saldata in seguito **non scompare automaticamente dal calendario**.
4. **Piano aggiornato dopo l'esportazione.** L'utente cambia date o importi in Repertica. Il file già importato rimane **obsoleto**: non esiste sincronizzazione bidirezionale. Una nuova importazione può creare duplicati; l'interfaccia deve avvisare e suggerire, se possibile, un **calendario separato dedicato** sostituibile dall'utente, senza presumere supporto nativo alla sostituzione.
5. **USB/disco scollegato.** Il salvataggio fallisce senza alterare SQLite né sovrascrivere file già esistenti; si può riprovare in un'altra cartella.

**Flusso suggerito, solo descrittivo:** Vendite → Rate → «Esporta scadenze per calendario (.ics)» → intervallo e anteprima conteggio → campo selezione dati esportabili (predefinito: **testo generico**; importo/codice solo opt-in) → avviso su snapshot, possibili duplicati e calendar cloud → scelta della cartella attraverso dialogo nativo → conferma → file .ics creato localmente, senza aprire o inviare automaticamente.

**Prima fase deliberatamente minima:** nessun nuovo pannello calendario interno, nessun import .ics, nessuna configurazione dell'account calendario, nessun invito ad attendee, nessuna sottoscrizione `webcal`, nessun popup obbligatorio o timer. I record rimangono visibili e modificabili solo tramite gli strumenti di gestione rate già esistenti.

## 4. Specifica funzionale e contratto dati descrittivo

**Origine autorevole:** `sale_installments` e `sale_receipts` nel SQLite locale. Un export usa uno snapshot di lettura coerente; se l'anteprima è diventata obsoleta al momento della scrittura, si valuta una conferma per differenze sostanziali. Non creare nuove rate, incassi o movimenti economici. Validare date, ordinamento, stato derivato e intervallo; gestire un elenco vuoto in modo esplicito.

**Contratto concettuale (non API implementata):**

```text
IcalExportRequestV1 {
  date_from: YYYY-MM-DD,
  date_to: YYYY-MM-DD,
  states: [DUE, PARTIAL, OVERDUE],  // selezione esplicita
  include_amount: false,            // default prudente
  include_artwork_code: false,      // default prudente
  destination: user_selected_file
}
IcalPreviewV1 { count, period, states, exposed_field_categories, warnings }
IcalExportResultV1 { count, saved_path, snapshot_created_at }
```

**Forma `.ics` da valutare:** `VCALENDAR` con `VERSION:2.0`, `PRODID` stabile, `CALSCALE:GREGORIAN` e un `VEVENT` per ciascuna scadenza selezionata. La data è **un giorno civile**, non un'ora di pagamento stabilita: usare `DTSTART;VALUE=DATE:YYYYMMDD`; nessuna conversione con `TZID` o offset a mezzanotte che sposti il giorno sui calendari in altro fuso. Il `DTSTAMP` è in UTC; `UID` è univoco; `SUMMARY` è neutro; `CLASS:PRIVATE` e `TRANSP:TRANSPARENT` sono indicazioni al client, **non** un meccanismo di cifratura o una garanzia di riservatezza. Nessun `ORGANIZER`, `ATTENDEE`, `METHOD:REQUEST`, `URL` esterno o `VALARM` nella fase minima. Se un calendario non accetta VEVENT di un solo giorno, valutare `DTEND;VALUE=DATE` del giorno seguente, secondo RFC, con test di interoperabilità.

**Serializzazione:** UTF-8, righe CRLF, escape corretti per backslash/virgole/punti e virgola/nuove righe nei campi testuali, folding a 75 ottetti senza spezzare sequenze UTF-8, limiti prudenti sulle stringhe. Testare titoli con caratteri non ASCII, inclusi accenti e emoji, anche se il profilo minimo non li include. Il nome file suggerito deve essere neutro e non divulgare cliente o opera.

**Identità / re-export (questione critica):** `replace_plan` reinserisce le rate e cambia i loro UUID. Non è corretto promettere UID persistenti basandosi su `sale_installments.id`, né generare il medesimo UID dal solo ordine della rata: il piano può essere riordinato o sostituito. **Prima fase:** esportazione di un'istantanea autonoma con UID globalmente univoci; spiegare la possibilità di duplicati quando l'utente reimporta. **Solo dopo bisogno validato:** studiare identità durable delle occorrenze e semantica di revisioni/cancellazioni, compatibilità di calendari diversi e migrazioni del database, senza trattarle come parte implicita di questo export.

**Atomicità e percorso:** scrittura in file nuovo, verifica della scrittura e pubblicazione finale sicura, controlli su permessi, collegamenti simbolici e destinazioni inattese, con gestione conservativa dei temporanei (coordinare con TECH-018/034). Vietato salvare nel media store/archivio attivo o modificare file di calendario esistenti senza conferma esplicita; prediligere `create_new` e nessuna sovrascrittura.

## 5. Offline, privacy, impostazioni, multilingua, accessibilità

- **Offline-first:** l'esportazione è locale e funziona senza Internet, account, servizio Repertica, provider calendario o abbonamenti. L'utente decide *se* importare il file in un programma diverso; quel programma può eventualmente sincronizzarlo su Internet per scelta o configurazione sua.
- **Privacy minimizzata:** niente nomi di collezionisti, recapiti, importi, descrizioni, note private, ID interni, numero di vendita o titolo dell'opera per default. Avviso se si includono importo/codice: in un file `.ics` non cifrato, su NAS condiviso o in un calendario cloud, tali informazioni possono essere divulgate. `CLASS:PRIVATE` da sola **non protegge** i dati.
- **Confidenzialità:** niente invio automatico, log contenenti titoli/clienti/importi o telemetria. Nessuna pretesa di conformità legale, certificazione dei pagamenti o notifica affidabile da parte dei client esterni.
- **Impostazioni:** preferire nessuna impostazione persistente iniziale. L'eventuale ultima cartella/periodo è locale e facoltativa; non memorizzare il consenso a includere campi riservati in esportazioni successive.
- **Multilingua:** pulsanti, avvertenze e `SUMMARY` generici localizzati secondo la lingua di sistema e fallback inglese (override manuale se previsto). `UID`, proprietà iCalendar e date ISO restano standard/stabili e non tradotti; i campi degli utenti non vanno tradotti automaticamente.
- **Accessibilità:** anteprima con conteggio e categorie dati leggibile da screen reader, dialogo navigabile da tastiera, annunci di errori e conclusione, focus prevedibile, nessun colore come unica indicazione.
- **Esportabilità e backup:** CSV integrali, export completo e backup verificato su cartella liberamente scelta rimangono invariati. Il file .ics creato **fuori dall'archivio non entra automaticamente nei backup di Repertica**; non è ripristinabile come sorgente delle rate e non va presentato come copia del database.
- **Compatibilità:** macOS/Windows/Linux; testare sia lettori calendario locali sia importatori facoltativamente online. L'assenza di client calendario non deve bloccare il salvataggio.

## 6. Dipendenze, alternative, rischi, complessità

**Dipendenze progettuali eventuali:** lettura di `installments.rs` e `sale_receipts`, dialogo nativo di scelta file, serializzazione RFC 5545, i18n e controlli di salvataggio. **Nessuna dipendenza aggiunta ora.** La serializzazione potrebbe essere fatta senza librerie esterne, ma va confrontata con l'affidabilità/maintenance di un parser/serializer maturo in un'eventuale decisione tecnica futura.

**Alternative da confrontare prima di sviluppare:** (A) mantenere la consultazione delle rate dentro Repertica e usare l'esistente CSV `installments` con elaborazione manuale; (B) modello .ics esterno compilato manualmente per pochi eventi; (C) un solo export puntuale per una specifica vendita, meno rischio privacy e minore complessità; (D) calendario dedicato con sincronizzazione solo su esplicita richiesta futura, molto più complesso e fuori ambito. **Preferenza di ricerca:** iniziare con A/C e dati d'uso reali, non costruire una piattaforma calendar.

**Rischi:** aspettative errate di sincronizzazione, notifiche o pagamento automatici; duplicati all'importazione ripetuta; esposizione del calendario a servizi terzi; fusi orari e giorni slittati; stati/solleciti obsoleti; UIDs instabili dopo revisione del piano; formati .ics diversamente accettati; file parziali o sovrascritture; calendari importati che appaiono pubblici nonostante `CLASS:PRIVATE`.

**Complessità stimata:** **bassa–media** per esportazione puntuale di soli eventi giornalieri anonimi; **media** per selezioni, privacy con anteprima e interoperabilità robusta; **alta** per aggiornamento incrementale, rimozione eventi o sincronizzazione durevole. Non accorpare il terzo caso al primo.

## 7. Test e criteri di accettazione per eventuale sviluppo

1. Dati di prova: rate future, scadute, parziali e pagate, incassi parziali, 2–60 rate, importi in centesimi, febbraio bisestile, fine mese, giorni a cavallo del cambio ora; nessun incasso o stato modificato dall'export.
2. File `.ics` conforme a RFC 5545; validazione strutturale con parser indipendente e importazione manuale in almeno due calendari realmente disponibili, incluso un programma capace di lavorare senza cloud.
3. `DTSTART;VALUE=DATE` mantiene il giorno civile tra fusi; nessun orario inventato, allarme non voluto, attendee, invito o sincronizzazione di default.
4. Il profilo minimo **non contiene nessuna sentinella** di dati riservati (nominativi, email, importi, descrizioni, note, percorsi, UUID/codici non autorizzati), nemmeno in `UID`, intestazioni e nome file.
5. Caso `replace_plan`: esportazione precedente resta intatta; una nuova esportazione è chiaramente identificata come nuova istantanea e non promette update automatico né assenza di duplicati.
6. Se l'utente include importo/codice, solo i campi selezionati compaiono e l'avvertenza di condivisione appare prima del salvataggio.
7. Test con testo Unicode, CRLF, caratteri speciali, lunghezza/folding, zero eventi selezionati, eventi omonimi, due rate con medesima data, file esistente, destinazione non scrivibile o USB scollegata.
8. Nessuna sovrascrittura o file finale parziale, nessuna scrittura all'archivio, nessuna modifica a CSV, backup, media, credenziali, workflow o MCP.
9. Test Windows/macOS/Linux di accessibilità da tastiera/screen reader e localizzazione IT/EN/ES/FR/DE, fallback inglese.
10. Un utente non tecnico distingue in UI **«data concordata»**, **«incasso dichiarato»**, **«file calendario statico»**, **«backup»** e **«notifica non garantita»**.

## 8. Fasi possibili e questioni aperte per il proprietario

- **Fase 0 — conferma dell'esigenza:** chiedere a 3–5 professionisti con rate reali se consultano un calendario esterno, quali programmi usano, se preferiscono file `.ics` o restare ai CSV/alla UI; confrontare con esempi manuali. **Se la domanda è rara, nessun sviluppo.**
- **Fase 1 — semplificazione:** solo export manuale di scadenze di una vendita, con dati neutri, validazione standard e avviso «istantanea senza sincronizzazione».
- **Fase 2 — solo se utile:** più vendite e filtri per periodo/stato; selezione di singoli campi sensibili con anteprima; compatibilità dei maggiori importatori desktop.
- **Fase 3 — separata e da riapprovare:** valutare identità duratura e revisioni, gestione dei duplicati e possibilità di sincronizzazione locale esplicitamente autorizzata. Non è un requisito dell'idea attuale.

**Domande senza risposta:** Quanti utenti usano davvero pagamenti rateali e calendari esterni? È sufficiente una singola esportazione per vendita? È desiderabile mostrare un codice opera o solo una data generica? Come si gestirebbero modifiche e cancellazioni dopo l'import in modo indipendente dal client? Serve davvero un file multi-evento? Quali calendar app e sistemi operativi vale la pena collaudare? Il beneficio supera il rischio di duplicati e di divulazione di metadati?

**Esito della ricognizione:** **Proposta — da valutare**. Tutte le scelte tecniche e le priorità restano ipotesi sottoposte alla revisione del proprietario. Nessuna implementazione, modifica di codice, issue o PR operativa è autorizzata da questa scheda.
