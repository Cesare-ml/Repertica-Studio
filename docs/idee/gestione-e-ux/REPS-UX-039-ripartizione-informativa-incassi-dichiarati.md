# REPS-UX-039 — Ripartizione informativa degli incassi dichiarati per fondo

- **ID stabile:** REPS-UX-039
- **Data della ricognizione:** 2026-11-22
- **Stato:** Proposta — da valutare
- **Filone:** Gestione e UX
- **Priorità ipotetica:** P1–P2 per chi utilizza quote e vendite a rate; P3 se bastano i registri esistenti
- **Natura:** sola analisi e documentazione. Nessuna approvazione, assegnazione o autorizzazione allo sviluppo.

## 1. Problema professionale, utenti e fatti osservati

Un artista imposta una ripartizione del ricavato di una vendita fra «Personale», «Attività» e «Risparmio» (oppure parti personalizzate). Se vende un'opera per 1.000 EUR con incasso di soli 250 EUR, il riepilogo per fondo costruito sul *netto della vendita registrata* può far apparire quote riferite anche ai 750 EUR non ancora incassati. L'artista ha bisogno di distinguere **quote del venduto** da **quote teoriche riferibili agli incassi che ha dichiarato di aver ricevuto**. Non è una richiesta di trasferire fondi o di certificare denaro realmente disponibile.

**Destinatari:** artisti e piccoli studi che dividono i proventi per finalità, soprattutto con acconti, rate o vendite saldate dopo alcuni mesi; assistenti amministrativi che preparano un controllo mensile. Per chi vende sempre con pagamento contestuale o non usa le ripartizioni, non aggiungere complessità alla vista corrente.

**Riscontri statici su main, 22/11/2026:**
- [Economia e localizzazione](../../ECONOMY-I18N.md) distingue esplicitamente venduto, netto, spese e incassi: le quote vengono congelate **quando è registrata la vendita**, e il riepilogo mensile raggruppa per data vendita.
- [allocations.rs](../../../src-tauri/src/allocations.rs) memorizza quote per parti identificabili, incluse quelle successivamente disattivate; ogni nuova vendita ne conserva lo snapshot. Le quote del piano attuale non devono essere applicate retroattivamente.
- [finance.rs](../../../src-tauri/src/finance.rs) calcola le quote mensili dalle vendite per data di vendita, sottraendo imposte e spedizione registrate per ottenere il netto. Le spese sono raggruppate per data di spesa.
- [SALE-RECEIPTS.md](../../SALE-RECEIPTS.md) e [receipts.rs](../../../src-tauri/src/receipts.rs) descrivono incassi **inseriti manualmente**, anche parziali, con data propria, senza prova bancaria. Esiste già una vista mensile degli incassi e il CSV dei saldi.
- [DATA-PORTABILITY.md](../../DATA-PORTABILITY.md) documenta 12 registri CSV completi, tra cui vendite, allocazioni, incassi e saldi. Il backup SQLite+media è la copia completa e verificata.
- **Non risulta, nei percorsi esaminati, una lettura delle quote ripartite per la data dei singoli incassi dichiarati.** È una lacuna di lettura gestionale ipotizzata, **non** un bug runtime dimostrato. Nessun archivio reale, intervista utente o prova desktop è stato eseguito.

## 2. Distinzioni rispetto alle idee già presenti

- **[UX-024](REPS-UX-024-scadenziario-incassi-piani-pagamento-vendite.md):** i piani rateali e l'attribuzione degli incassi alle rate sono **già implementati nel nucleo su main**. Non vengono riprogettati. Qui interessa esclusivamente la **lettura delle quote sui proventi dichiarati incassati** per periodo.
- **[UX-003](REPS-UX-003-costo-e-margine-per-opera.md):** attribuzione dei costi a una specifica opera e margine, non distribuzione per fondo delle somme ricevute.
- **[UX-030](REPS-UX-030-quadratura-facoltativa-estratti-conto-locali.md):** confronto facoltativo con dati bancari; questa proposta **non** verifica né importa movimenti bancari.
- **[UX-012](REPS-UX-012-affidamenti-conto-vendita-rendiconti.md):** rendiconti fra artista e intermediario; qui si lavora solo sulle ripartizioni già congelate sulle vendite di Repertica.
- **[UX-032](REPS-UX-032-budget-mensili-categorie-spesa-scostamenti.md), [UX-033](REPS-UX-033-entrate-professionali-non-vendita-opere.md), [UX-034](REPS-UX-034-analisi-vendite-caratteristiche-opere.md), [UX-037](REPS-UX-037-scadenze-spese-ricorrenti-senza-registrazioni-presunte.md):** budget, entrate diverse dalle vendite, analisi per caratteristiche e obblighi futuri; nessuno sostituisce la ripartizione per data d'incasso.
- Nel filone tecnico [TECH-018](https://github.com/Cesare-ml/Repertica-Studio/blob/docs/idee-strumenti-architettura-20261121/docs/idee/strumenti-e-architettura/REPS-TECH-018-esportazioni-csv-scalabili-atomiche.md) e [TECH-025](https://github.com/Cesare-ml/Repertica-Studio/blob/docs/idee-strumenti-architettura-20261121/docs/idee/strumenti-e-architettura/REPS-TECH-025-esportazioni-tabellari-minimizzate-condivisione.md) studiano rispettivamente robustezza e selezione degli export: non si propone un nuovo motore tecnico di esportazione. Nessun loro file viene modificato.

## 3. Scenari d'uso verificabili

1. **Acconto.** Vendita da 1.000 EUR lordi, con 100 EUR di tasse e 100 EUR di spedizione già *annotate* nella vendita: netto gestionale 800 EUR. Con quote congelate al 60% e 40%, un incasso manuale da 250 EUR genera **200 EUR di quota netta teoricamente riferibile all'incasso**: 120 EUR e 80 EUR. Restano 750 EUR lordi non dichiarati incassati; la quota residua del netto è 600 EUR (360+240). Non è una dichiarazione che tasse/spedizione siano già state pagate o accantonate.
2. **Saldo in un altro anno.** Vendita del novembre 2026, saldo dichiarato incassato nel gennaio 2027: la ripartizione del venduto rimane nel 2026; la nuova **vista per incassi** mostra la quota del saldo nel gennaio 2027 e non riscrive il 2026.
3. **Piano quote cambiato.** Nel dicembre 2026 l'artista modifica le percentuali per le vendite future; un incasso del 2027 relativo a una vendita di ottobre 2026 usa **le quote congelate su quella vendita**, anche se una parte è oggi disattivata.
4. **Vendita senza incassi dichiarati.** Il venduto e le sue quote storiche restano visibili; per la vista per incassi si mostra zero con etichetta «Nessun incasso registrato», mai «non pagato in banca».
5. **Ripartizione disattivata su una vendita.** La vendita può avere incassi, ma non si inventano percentuali retroattive né si applica il piano corrente: la vista la classifica separatamente come «senza ripartizione storica».
6. **Spese del fondo.** Spese annotate a carico di «Attività» sono consultabili a parte; non vengono sottratte dal presunto contante disponibile perché le spese registrate **non sono necessariamente pagamenti verificati**.

## 4. Flusso UX ipotetico

**Economia → Ripartizioni:** conservare la tabella attuale «Quote del venduto (data vendita)» e offrire una seconda vista facoltativa **«Quote sugli incassi dichiarati (data incasso)»**. La scelta non cambia l'archivio e, per prudenza, non sostituisce i totali esistenti. Periodo/anno e parte sono filtri di sola lettura, sempre con definizione della metrica mostrata vicino al totale, non nascosta in un tooltip.

**Dettaglio:** per ciascuna parte e mese: totale attribuito in base agli incassi manuali, numero di incassi, e link verso le vendite sorgente. Una riga per vendita permette di confrontare importo lordo venduto, netto registrato, totale incassi dichiarati, netto pro-quota teorico, differenza residua e distribuzione per parte. Distinguere anche vendite senza snapshot quote.

**Riconciliazione visiva:** in corrispondenza di ogni vendita presentare la formula e un esempio numerico; avvertire che «incassato dichiarato» non equivale a «bonifico verificato», «denaro sul conto» o «utile disponibile». Nessuna CTA «preleva», nessun trasferimento e nessuna registrazione automatica di spese, imposte o accantonamenti.

**Accessibilità:** tabella leggibile da tastiera e screen reader, intestazioni persistenti, differenza fra base «vendita» e base «incasso» espressa sempre in testo, focus dopo cambio vista, esportazione con anteprima, layout utilizzabile a zoom 200%; niente colori come unica semantica.

## 5. Dati necessari e ipotesi di calcolo/API — soltanto descrittivi

**Fonte unica:** riusare le vendite, le finanze di vendita (importi delle detrazioni già annotati), le quote storiche in punti base e gli incassi manuali esistenti, con UUID e date originali. **La prima fase non richiede un secondo registro di quote né nuove operazioni contabili persistenti.** Non creare una nuova tabella di «fondi effettivamente versati».

**Esempio di algoritmo deterministico da discutere:**
- Per ciascuna vendita: lordo G maggiore di zero, imposte registrate T e spedizione registrata S; netto gestionale N = G − T − S. Non presumere che T e S siano effettivamente pagati.
- Ordinare gli incassi della vendita per data civile di ricezione, poi data di registrazione e UUID per risolvere parità. Per ogni incasso k calcolare il cumulato lordo dichiarato R(k), già vincolato a G.
- Calcolare la *quota netta cumulata illustrativa* N(k) = floor(N × R(k) / G) in **aritmetica intera a precisione estesa**; al saldo completo imporre esattamente N. Il contributo dell'incasso k è la differenza N(k) − N(k−1), non un arrotondamento indipendente per ogni rata.
- Applicare a N(k) le **percentuali congelate per quella specifica vendita** con regola di arrotondamento deterministica e residuo sull'ultima parte (come la logica esistente); ricavare per differenza le quote dell'evento k. A saldo completo la somma per parte deve coincidere con la ripartizione del netto della vendita, senza centesimi persi o duplicati.
- Se una vendita non ha snapshot di ripartizione, non inventare quote. Se un record è incoerente, riportare un errore esplicito, non percentuali ricostruite dai parametri correnti. Per report storici, note successive sugli incassi o rettifiche non ancora supportate richiedono avviso e revisione del modello.
- Il mese della nuova vista è quello della **data dell'incasso dichiarata**, non della vendita. Una variazione del piano rateale non produce nuovi incassi né nuove quote.

**Nomi concettuali, non endpoint approvati:** «calcola ripartizione dichiarata per periodo», «elenca contributi per incasso e vendita», «spiega formula e arrotondamenti», «prepara report esportabile». Per la prima fase valutare una vista **derivata read-only** senza cambiare schema SQLite, mantenendo i saldi autorevoli nel registro incassi. Non introdurre doppio conteggio fra anno di vendita e anno di incasso.

## 6. Impostazioni, offline, multilingua, privacy e compatibilità

- **Impostazioni:** vista facoltativa non attivata al posto di quella storica; possibile preferenza locale per l'ultima vista scelta, senza forzare nuovi valori sul piano percentuale. Disattivando l'opzione non si cancellano dati. Nessuna tariffa, aliquota o «cassa iniziale» richiesta.
- **Offline:** calcoli e filtri su SQLite locale in Tauri; niente conto bancario, API, credenziali, Repertica Online, accesso cloud obbligatorio, IA o abbonamento. Tutto disponibile senza rete.
- **Multilingua:** lingua OS con fallback inglese; override IT/EN/ES/FR/DE già previsto. Tradurre rigorosamente «venduto», «netto», «incassato dichiarato», «quota pro-quota teorica», «spesa registrata» e «non verificato»; mostrare date e importi secondo locale ma calcolare su centesimi EUR e codici stabili.
- **Privacy:** la dashboard privata può collegare l'importo a un collezionista. Un futuro report destinato a terzi **non** deve includere per default nominativi, recapiti, causali e dettagli dei fondi. I backup attuali non sono cifrati; conservare con cura le copie in cartelle scelte liberamente.
- **Compatibilità:** non riscrivere vendite, rate, incassi, spese, quote congelate o certificati. Non cambiare la semantica delle statistiche storiche né i 12 CSV attuali. Archivi vecchi con ripartizioni assenti o inattive restano apribili. Se il calcolo è soltanto derivato, il backup SQLite+media esistente già conserva tutte le fonti; il ripristino su cartella scelta deve dare risultati identici a parità di dati.
- **Import/export:** usare i CSV integrali già disponibili per la verifica dei calcoli; un export **aggiuntivo e facoltativo** può elencare per UUID evento/vendita/parte i centesimi e il metodo di arrotondamento, con periodo e base «incasso manuale». Non sostituisce i registri integrali e non è un formato di ripristino. Se un futuro pacchetto aperto reimporta le fonti, gli UUID e le quote storiche vanno validati prima di ricalcolare; nessuna importazione automatica di estratti bancari.

## 7. Complessità, rischi, dipendenze e alternative

- **Complessità:** bassa–media per consultazione annuale senza persistenza; media per attribuzione precisa a ciascun incasso, report e transizioni fra anni; più alta solo con rimborsi, resi o valuta multipla (non inclusi).
- **Dipendenze già disponibili:** registri sale_receipts, sale_finances, sale_allocations, parts attive/disattivate, date e valori in centesimi, backup, localizzazione. Nessuna nuova dipendenza software prescritta; il collegamento a UX-027 (multi-valuta) o UX-013 (resi) sarebbe un tema separato.
- **Rischi:** chiamare le quote «liquidità disponibile»; ritenere pagate imposte/spedizioni soltanto perché detratte dal netto; arrotondamenti duplicati su più rate; quote attuali applicate a vendite pregresse; report a cavallo di anni; due viste sovrapposte sommate per errore; nomi di collezionisti esportati senza necessità.
- **Mitigazioni:** etichetta informativa permanente, formule verificabili, esempi con centesimi non divisibili, vista storica separata, nessun movimento creato, lettura delle quote per UUID storico, anteprima privacy prima di esportare.
- **Alternativa minima:** accanto alle quote già presenti mostrare un semplice **avviso «calcolate sul venduto, non sull'incassato»**, con link ai saldi esistenti. Alternativa intermedia: totali di incassato senza attribuzione ai fondi. Alternative esterne: confronto manuale dei CSV con foglio di calcolo offline.

## 8. Criteri di accettazione ipotetici

1. Una vendita di 1.000 EUR, con 100+100 EUR di detrazioni registrate, ripartizione congelata 60/40 e incasso di 250 EUR, mostra **120/80 EUR** nella nuova vista e **480/320 EUR** come quote del *netto venduto*. Non dichiara alcun pagamento delle detrazioni.
2. Ulteriore incasso di 750 EUR: la somma di tutte le quote attribuite agli incassi è esattamente 800 EUR, divisa 480/320; nessun centesimo perso o contato due volte.
3. Con importi in centesimi non divisibili, tre o più incassi e quote personalizzate, tutte le differenze cumulative restano coerenti; uguaglianza esatta al saldo completo e indipendenza dall'ordine di calcolo.
4. Incasso dell'anno successivo compare soltanto nella **vista per incassi** dell'anno successivo; il totale venduto dell'anno precedente non cambia.
5. Cambiare o disattivare le parti non modifica le quote storiche; una vendita senza snapshot viene segnalata, non completata con quote inventate.
6. Senza incassi registrati la nuova vista presenta zero **registrato**, non denaro bancario verificato o pagamento in ritardo; eventuali rate non vengono trasformate in incassi.
7. Le spese restano nel registro esistente; i totali mensili storici, i CSV integrali, i certificati e il backup non cambiano.
8. Windows/macOS/Linux, test di data civile locale, lingue IT/EN/ES/FR/DE + fallback inglese, sola tastiera e screen reader; nessuna chiamata HTTP.
9. Report facoltativo: l'utente può confrontare i conteggi con i registri locali, scegliere le informazioni condivise e ottenere un file senza alterare i dati.

## 9. Fasi possibili — non autorizzate

- **Fase 0 — Verifica del bisogno:** raccogliere tre esempi di studio con acconti/saldi, confrontarli con il riepilogo già esistente; stabilire se basta l'avviso sulla base del calcolo.
- **Fase 1 — Eventuale vista minima read-only:** confronto contestuale «quote del venduto / quote su incassi dichiarati», formule e tooltip esplicativi, nessun nuovo record.
- **Fase 2 — Eventuale dettaglio ed export:** drill-down per vendita/incasso/parte, CSV specifico e test su arrotondamenti e anni diversi.
- **Fase 3 — Solo se richiesto:** comparazione, **separata e non equiparata alla disponibilità di cassa**, con spese registrate; successive rettifiche/valute soltanto dopo progettazione esplicita.

## 10. Requisiti certi, ipotesi e domande al proprietario

**Requisiti certi del prodotto:** software open source gratuito e offline-first senza account obbligatorio, Tauri/SQLite locale, registri ed esportazione dati, backup e ripristino con cartella liberamente scelta, UI multilingue di sistema con fallback inglese, snapshot storici delle quote, incassi manuali separati dalle vendite.

**Ipotesi, non fatti dimostrati:** esiste una confusione diffusa fra quote sul venduto e quote attribuibili agli incassi; serve una seconda vista per decisioni mensili; il pro-rata del netto è una convenzione utile, ma **non** una regola contrattuale/fiscale universale.

**Domande da riservare alla revisione del proprietario:** gli utenti vogliono realmente confrontare le due basi oppure basta una nota? Il criterio pro-rata sulle detrazioni è accettabile, o occorre mantenere sempre il lordo incassato e le trattenute previste in righe separate? Le quote per fondo devono essere viste per data incasso o solo cumulate per vendita? Quanto dettaglio dei collezionisti deve apparire nella vista privata? Conviene evitare qualsiasi termine simile a «disponibile»?

**Esito:** esclusivamente **Proposta — da valutare**. Nessuna approvazione o assegnazione allo sviluppo.
