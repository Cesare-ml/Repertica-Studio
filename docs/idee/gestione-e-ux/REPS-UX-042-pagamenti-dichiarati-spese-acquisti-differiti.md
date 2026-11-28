# REPS-UX-042 — Acquisti con saldo differito: pagamenti dichiarati e scadenze, senza duplicare le spese

- **ID stabile:** REPS-UX-042
- **Data della ricognizione:** 2026-11-22
- **Stato:** Proposta — da valutare
- **Filone:** Gestione e UX
- **Priorità ipotetica:** P2 per studi con acquisti a saldo differito; P3 se bastano note e calendario; P1–P2 soltanto dopo riscontri diretti e frequenti
- **Natura:** analisi e documentazione soltanto; **nessuna implementazione, approvazione, assegnazione o modifica del comportamento attuale**.

## 1. Problema, destinatari e fatti verificati in `main`

Un artista può acquistare materiali, una cornice speciale o un'attrezzatura a **pagamento differito**, con acconto oggi e saldo dopo la consegna. Il costo di acquisto e la data del pagamento non sono la stessa cosa. Se il registro mostra soltanto una spesa di 1.200 €, non permette di distinguere tra costo registrato, 300 € dichiarati versati, 900 € da saldare e mancanza di qualunque informazione sui pagamenti. Questo complica il controllo di impegni verso fornitori senza giustificare una contabilità bancaria.

**Destinatari da validare:** artisti che acquistano attrezzature o materiali con acconti/saldi, studi di produzione, piccole gallerie e laboratori che commissionano cornici, allestimenti o servizi a fornitori esterni. Per chi paga tutto contestualmente e registra poche spese, il modulo potrebbe essere superfluo.

**Riscontro statico su `main` il 22 novembre 2026, non prova d'uso:** `src/app/models.ts` espone `Expense` con `expense_date`, `amount_cents`, `category`, `fund`, `supplier`, `purchase_url` e `notes`; `src-tauri/src/db.rs` conserva queste spese in SQLite; `docs/ECONOMY-I18N.md` dichiara espressamente che le statistiche mensili raggruppano i **costi registrati per data di spesa**, **senza attestare denaro pagato**, e che una futura evoluzione potrà distinguere pagamenti effettivi e debiti. Sullo stesso `main` esistono già **incassi dichiarati e piani rateali delle vendite** (`docs/MVP.md`), ma riguardano il lato cliente/acquirente, non il pagamento di un acquisto a fornitore. `docs/DATA-PORTABILITY.md` esporta `expenses` senza un registro di pagamenti riferiti alle singole spese. Non risulta dai percorsi consultati un registro equivalente per le **uscite dichiarate a saldo di spese già registrate**.

È una **lacuna funzionale possibile**, non un bug riprodotto, un obbligo fiscale o un bisogno già dimostrato presso utenti reali.

## 2. Confini e confronto anti-doppione

- **[UX-037 — Scadenze ricorrenti](REPS-UX-037-scadenze-spese-ricorrenti-senza-registrazioni-presunte.md):** pianifica un *possibile futuro costo periodico* (canone, affitto), anche **prima** che esista una spesa. Questa proposta segue **il saldo di una singola spesa già registrata**, anche non ricorrente. Una scadenza ricorrente non diventa un debito o pagamento, e non servono due calendari divergenti.
- **[UX-040 — Rimborsi e rettifiche](REPS-UX-040-rimborsi-rettifiche-spese-acquisti.md):** gestisce eventi che **rettificano il valore dell'acquisto** o restituiscono denaro; qui si dichiarano **versamenti effettuati verso il fornitore** a fronte dell'acquisto, non note di credito. Se entrambe le idee fossero approvate, il calcolo del saldo dovrebbe distinguere esplicitamente costo rettificato, promessa di rimborso, rimborso ricevuto e pagamento fornitore, senza eventi duplicati.
- **[UX-030 — Quadratura bancaria locale](REPS-UX-030-quadratura-facoltativa-estratti-conto-locali.md):** confronta manualmente righe di un *CSV bancario* con registrazioni esistenti; **non costituisce la fonte dei pagamenti qui dichiarati**. Un eventuale collegamento sarebbe un riscontro opzionale, mai creazione automatica o conferma bancaria implicita.
- **[UX-024 — Rate dei clienti](REPS-UX-024-scadenziario-incassi-piani-pagamento-vendite.md):** la quota rateale delle **vendite** è già in parte realizzata su `main`; non duplicare modelli, totali o comandi delle vendite per i fornitori. La semantica di una spesa può differire da quella di un prezzo di vendita.
- **[UX-003 — Costi per opera](REPS-UX-003-costo-e-margine-per-opera.md), [UX-032 — Budget](REPS-UX-032-budget-mensili-categorie-spesa-scostamenti.md):** attribuzione dei **costi** ad opere e budget per categoria non sono prova di pagamento; questa proposta non riscrive costi né budget.
- **[TECH-001/018/019/025 — Portabilità](https://github.com/Cesare-ml/Repertica-Studio/pull/16):** eventuali formati e import/export devono coordinarsi, ma il filone tecnico rimane separato e non è stato modificato.

**Divieti funzionali essenziali:** nessun conto bancario integrato, PSD2, bonifico, invio a fornitore, attestazione di quietanza, fatturazione fiscale, scadenzario obbligatorio, creazione automatica di spese, riconciliazione silenziosa, doppio inserimento di costi o modifica delle quote storiche delle vendite.

## 3. Scenari concreti e casi-limite

1. **Macchinario per 1.200 €.** Spesa registrata il 20 novembre 2026 per 1.200 €, con acconto **dichiarato** di 300 € il 20 novembre e saldo concordato di 900 € per il 20 gennaio 2027. La vista mostra: costo registrato 1.200 €, pagamenti manualmente dichiarati 300 €, residuo **rispetto alle dichiarazioni** 900 €, scadenza **prevista** 20 gennaio. La statistica delle spese di novembre resta **1.200 €**; un eventuale grafico delle uscite dichiarate mostra **300 €** a novembre, non 1.500 €.
2. **Acquisto apparentemente saldato, ma senza informazioni.** Un artista aveva già pagato tutto quando registrò una vecchia spesa di 80 €: il sistema **non** deve mostrarla come «80 € insoluti». Il messaggio corretto è «Stato dei pagamenti non documentato». L'utente potrebbe annotare retrospettivamente un pagamento dichiarato, ma non è obbligato a riscrivere lo storico.
3. **Due pagamenti nello stesso giorno.** Due versamenti identici al medesimo fornitore sono possibili: data+importo non sono chiave unica e non autorizzano eliminazioni automatiche.
4. **Saldo rinviato.** Il fornitore sposta da gennaio a febbraio la scadenza: cambiare la **previsione** non crea, elimina né retrodata pagamenti già dichiarati.
5. **Rettifica o rimborso.** Una restituzione di materiale modifica eventualmente il costo gestionale in UX-040, non è un «pagamento negativo». Se il costo dovuto scende sotto i pagamenti già dichiarati, mostrare **situazione da verificare**, non un saldo positivo inventato o un rimborso presunto.
6. **Fornitore pagato con carta aziendale o in contanti.** «Pagato» resta dichiarazione dell'utente e **non** prova di regolamento bancario; non acquisire automaticamente movimenti o credenziali.
7. **Spesa ripartita su un fondo disattivato.** Conservare il fondo storico sulla spesa; un pagamento non deve duplicare il costo in un altro fondo o riscrivere quote pregresse.
8. **Archivio importato, con dati incompleti.** Nuove colonne vuote **non equivalgono a zero pagato**; distinguere «non monitorato», «monitorato senza pagamenti dichiarati» e «con pagamenti dichiarati».

## 4. Flussi UX proposti

**Ingresso facoltativo:** `Economia → Acquisti e spese → Dettaglio spesa → Pagamenti e scadenze`. La scheda preesistente mostra i valori di costo già noti. Il pannello è collassato o assente quando non utilizzato; non si forza ogni registrazione a passare per un nuovo wizard.

**Attiva monitoraggio di questa spesa:** breve spiegazione «Il costo resta nella statistica delle spese. Qui annoti soltanto pagamenti e date concordate». Se la spesa ha uno storico preesistente, non chiede di dichiarare che fosse insoluta. L'utente può scegliere «Non monitorare» senza cancellare alcun record economico.

**Registra pagamento dichiarato:** importo positivo, data effettiva **secondo l'utente**, metodo facoltativo (contanti/bonifico/carta/altro), riferimento e nota; anteprima del nuovo totale dichiarato e conferma. Se è disponibile un giustificativo locale (eventuale UX-028), il collegamento è esplicito, **non** un pagamento generato dal documento. Nessun avvio automatico di transazioni o comunicazioni.

**Pianifica saldo (fase eventuale):** un'unica scadenza concordata con importo opzionale o un piano con più scadenze che sommano **esplicitamente** all'importo da saldare; distingue «previsto», «scaduto rispetto alla data dichiarata», «pagamenti dichiarati» e «da verificare». La data passata non certifica insolvenza. Modifica del piano con motivo e data; evitare di riutilizzare direttamente gli stati dei piani rateali delle vendite.

**Riepilogo e filtri:** nella lista spese l'utente può attivare un filtro «con pagamento da verificare», «scadenza indicata trascorsa» o «con acconti», senza cambiare il totale economico mensile. In un'eventuale vista delle uscite dichiarate, ogni cifra espone *periodo per data di pagamento dichiarata* e avvertenza «Non è estratto conto». **Nessun totale di liquidità disponibile** derivato da queste sole registrazioni.

**Correzioni:** correzione con motivazione o annullamento di una dichiarazione errata mantenendo traccia, almeno nei casi già inclusi in un riepilogo condiviso. Non eliminare silenziosamente pagamenti collegati quando si modifica/elimina la spesa; prima mostrare riferimenti e richiedere decisione esplicita. Non sovrascrivere automaticamente riferimenti allegati.

**Accessibilità:** testi «costo», «pagamento dichiarato», «scadenza concordata», «saldo informativo» leggibili senza colore/icone; focus ed errori annunciati, pulsanti e tabelle da tastiera, totale esposto in testo per screen reader, zoom 200%, date localizzate, layout adatto a finestre piccole.

## 5. Dati proposti e regole (solo descrizione, NON schema approvato)

Una possibile entità **`expense_payment_declaration`** potrebbe contenere: `id` UUID, `expense_id` UUID, `paid_on` (data dichiarata), `amount_cents` positivo in centesimi interi, `currency` (EUR nel modello attuale), `method` facoltativo, `reference` facoltativo, `note`, `recorded_at`, `correction_of_id` facoltativo e stato di correzione. Non inferire il mezzo da testo libero, numero di conto o nome fornitore.

Una possibile entità **`expense_due_plan`** opzionale, *solo fase B*, potrebbe contenere UUID spesa, scadenze e importi previsti in centesimi, fonte/motivo del piano e revisioni datate. Le scadenze sono **previsioni**, non scritture di pagamento. Evitare la creazione del piano da una semplice spesa se l'utente non lo richiede.

Un eventuale flag di **monitoraggio volontario** e un indicatore `payment_record_coverage` distinguerebbero una spesa senza cronaca di pagamenti da una spesa monitorata con zero dichiarazioni. **Non migrare tutti gli acquisti storici a «non pagati».**

**Invarianti ipotetici:**
- `expense.amount_cents` e `expense.expense_date` rimangono la fonte del **costo registrato** e dei grafici oggi esistenti; i pagamenti non scrivono seconde righe `expenses` e non modificano margini, fondo o quote.
- Il totale dei pagamenti dichiarati non deve superare il valore dovuto **senza una situazione esplicita da correggere**. Per l'MVP monovaluta preferire il blocco di nuovi versamenti eccedenti; non alterare retroattivamente dichiarazioni storiche quando una rettifica cambia il valore della spesa.
- Il saldo «spesa registrata meno pagamenti dichiarati» è **informativo e condizionato alla copertura dei dati**, non una posizione debitoria certificata. Non esporlo come «debito totale dello studio» quando alcune spese non sono monitorate.
- Un pagamento non dimostra di per sé consegna del materiale, disponibilità di magazzino o regolarità fiscale. Un allegato SHA-256 dimostra integrità dei byte, non l'avvenuto pagamento.
- Nessuna unione con la contabilità delle vendite; i record `sale_receipts` restano entrate dai clienti, non pagamenti a fornitori.

**API puramente concettuali:** «leggi pagamenti dichiarati per spesa», «registra/annulla dichiarazione», «leggi scadenze concordate», «calcola riepilogo informativo». Nessun comando Tauri o migrazione è qui autorizzato.

## 6. Impostazioni, offline, lingue, privacy, esportazioni e compatibilità

- **Impostazioni:** monitoraggio opzionale per singola spesa; nessun default che dichiari pagata una spesa esistente. Formati di data e importi rispettano il locale; metodi di pagamento in vocabolario stabile. Un futuro promemoria, se deliberato, deve restare locale e con scelta esplicita, senza scheduler nascosto.
- **Offline:** SQLite locale, allegati eventualmente nel media store privato; nessuna chiamata esterna, conto bancario, email o notifica remota; tutti i flussi disponibili senza Internet e senza account.
- **Lingue:** lingua di sistema per l'interfaccia, override manuale IT/EN/ES/FR/DE e fallback inglese. Tradurre etichette, stati, avvisi ed errori; note/fonte e nomi del fornitore restano nella lingua originale, nessuna traduzione automatica. Date ISO e codici stabili in export.
- **Esportazione:** CSV UTF-8 con una riga per **pagamento dichiarato** e UUID spesa, importo in centesimi, valuta, data, metodo, riferimento e rettifiche; CSV separato delle **scadenze previste**, senza sommarle ai pagamenti. JSON locale versionato per un eventuale round trip verificabile con riferimenti UUID e null espliciti; import con anteprima, verifica conflitti, deduplica per UUID e rollback, **mai** reimportazione cieca o generazione di spese da un pagamento orfano. Backup integrale SQLite+media in cartella liberamente scelta rimane la via di recupero completo. Non confondere CSV con backup.
- **Privacy:** nomi di fornitori, causali, riferimenti bancari e note possono essere personali/sensibili; di default non condividere riferimenti/annotazioni, mostrare anteprima e chiedere conferma per export esterno. Non richiedere IBAN o informazioni bancarie. Backup attuali **in chiaro**, da proteggere a livello di dispositivo/cartella.
- **Compatibilità:** modifica eventualmente **additiva** e reversibile per database storici; nessuna conversione arbitraria di date/costi preesistenti né riscrittura del CSV `expenses` senza compatibilità valutata. Tutti i nuovi riferimenti devono sopravvivere a backup/ripristino verificato e al cambio fra Windows/macOS/Linux; nomi e percorsi Unicode consentiti. Se in futuro UX-027 prevede altre valute, confronti/saldi si fanno soltanto per valuta omogenea salvo conversioni deliberate e documentate.

## 7. Complessità, rischi, alternative e dipendenze

**Complessità proposta:** media per dichiarazioni manuali e riepilogo; medio-alta per piano di scadenze, rettifiche, cancellazione protetta e collegamento con rimborsi; alta e fuori ambito per contabilità bancaria/fiscale, automazioni di pagamento e integrazione bancaria.

**Dipendenze effettive:** identità stabile delle spese, calcoli in centesimi, SQLite, backup e portabilità, i18n e dialoghi di conferma. **Dipendenze soltanto eventuali:** UX-028 giustificativi, UX-037 impegni ricorrenti, UX-040 rimborsi, UX-030 quadratura. Non imporre la realizzazione delle altre schede per studiare la fase minima.

**Rischi:** scambiare «dichiarato» per «verificato»; attribuire debiti inesistenti agli archivi storici; doppio conteggio di spese; confondere rimborsi con versamenti; modifica di un acquisto che rende incoerenti i pagamenti; cronologia incompleta ma totale di saldo presentato come certo. Mitigare con terminologia, modalità non monitorata, vincoli, segnalazioni di incoerenza, conferme, prove su archivi migrati e riepiloghi separati.

**Alternative da confrontare prima di sviluppare:**
1. **Minima:** due campi facoltativi «acconto dichiarato / data saldo previsto» in una nota, senza calcoli né nuovi registri, per chi ha pochissimi acquisti.
2. **Intermedia (da verificare per prima):** elenco manuale di pagamenti dichiarati solo sulle spese monitorate, senza piano rateale.
3. **Completa:** calendario delle scadenze concordate, storico delle rettifiche e vista distinta dei pagamenti dichiarati, **solo se l'uso lo giustifica**.
4. Usare un calendario esterno + la scheda spesa corrente; nessun lock-in né trasferimento obbligatorio di dati.

## 8. Criteri di accettazione ipotetici, da usare soltanto dopo decisione

1. Con costo di 1.200 € in novembre e un pagamento dichiarato di 300 € in novembre, il totale delle **spese registrate** del mese resta 1.200 €; la vista separata delle **uscite dichiarate** mostra 300 €, e una previsione di 900 € in gennaio **non** diventa pagamento.
2. Due pagamenti dichiarati di 300 € e 900 € riportano un residuo informativo di 0 €, senza modificare costi, fondi, vendite o quote storiche. Superare il costo con un nuovo pagamento viene bloccato o segnalato come incongruenza da risolvere, **mai** sommato tacitamente.
3. Un archivio con spese storiche e **nessun** pagamento registrato non le mostra come «tutte insolute»: stato visibile «non monitorato», senza alterazione automatica.
4. Modificare una scadenza non crea spese/pagamenti; registrare una correzione preserva riferimenti e motivazione; una spesa con pagamenti collegati non viene cancellata senza avviso e percorso sicuro.
5. Se una rettifica del costo rende il saldo ambiguo, la vista mostra «da verificare» e non genera rimborsi, pagamenti negativi o movimenti bancari.
6. Export/import di prova conservano UUID, centesimi, valuta, date, distinzione previsto/dichiarato e riferimenti; backup/ripristino da cartella arbitraria conservano dati e allegati.
7. Flussi completabili da tastiera e screen reader su Windows/macOS, testi IT/EN/ES/FR/DE e fallback inglese; nessuna rete, account o invio non autorizzato.

## 9. Fasi suggerite, non roadmap approvata

- **Fase 0 — Validazione dei bisogni:** raccogliere 3–5 esempi reali di acquisti saldati in più volte e distinguere casi già gestibili con note/giustificativi.
- **Fase A — Separazione informativa minima:** monitoraggio volontario e pagamenti manualmente dichiarati associati a una spesa, senza modificare alcuna statistica preesistente.
- **Fase B — Scadenze concordate:** una o più scadenze, revisioni e filtri, senza contabilizzazione automatica.
- **Fase C — Interoperabilità prudente:** export/import e integrazioni solo se approvate; confronto eventuale con UX-030/040, conservando un'unica fonte autorevole per ogni tipo di evento.

## 10. Requisiti certi, ipotesi, domande al proprietario

**Requisiti certi:** desktop gratuito e open source, offline-first senza account/abbonamento, SQLite locale, backup completo verificato in cartella scelta, dati sempre esportabili, lingua sistema/fallback inglese; oggi le spese contano per data del costo, non sono prova di pagamento; esistono già rate/incassi lato **vendita**.

**Ipotesi:** gli utenti possiedono acquisti a saldo differito abbastanza frequenti da meritare un pannello; un registro di pagamenti manuali sarebbe più utile delle sole note; le date di saldo concordate possono essere ricostruite; gli utenti capiscono la differenza fra costo e uscita dichiarata.

**Domande riservate al proprietario:** (1) Quante spese a rate/acconti ci sono normalmente in un anno? (2) È sufficiente annotare acconto e data saldo o serve il registro dei singoli versamenti? (3) Il monitoraggio è per spesa, per fornitore o per entrambe le viste? (4) Si desidera un pannello delle scadenze *solo al momento dell'apertura* o preferire nessun promemoria? (5) Quale terminologia evita di chiamare «debito reale» un semplice residuo ricavato da dichiarazioni incomplete? (6) Quanto vale la pena integrare con UX-040 prima di verificare i casi reali?

**Decisione attuale: nessuna approvazione, nessuna assegnazione e nessun avvio di sviluppo.**
