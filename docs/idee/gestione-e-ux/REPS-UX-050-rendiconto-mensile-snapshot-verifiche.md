# REPS-UX-050 — Rendiconto mensile datato con verifiche gestionali, senza chiusura contabile

- **ID stabile:** REPS-UX-050
- **Data della ricognizione:** 2026-11-22
- **Stato:** Proposta — da valutare
- **Filone:** Gestione e UX
- **Priorità ipotetica:** P2 solo se gli artisti preparano riepiloghi periodici per studio/collaboratori/consulenti; P3 o nessuno sviluppo se bastano CSV e Calc
- **Natura:** studio esclusivamente documentale; nessuna approvazione, assegnazione, issue operativa o autorizzazione a implementare

## 1. Problema e destinatari: cosa è verificato, cosa è ipotizzato

**Riscontro statico in `main` (22/11/2026, senza test desktop):** `src-tauri/src/finance.rs` calcola già un riepilogo annuale con dodici mesi di **vendite lorde**, imposte/spedizione registrate, **netto vendite**, **spese registrate**, **margine gestionale** e quote delle ripartizioni storiche congelate per vendita. La documentazione `docs/ECONOMY-I18N.md` esplicita che vendita e spesa sono collocate secondo le rispettive **date dichiarate** e non attestano movimenti bancari. `src-tauri/src/receipts.rs` e `docs/SALE-RECEIPTS.md` riportano **incassi dichiarati** per data di incasso, separati dal venduto e mai inventati per vendite storiche. Il registro spese permette modifica/eliminazione; anche gli incassi hanno un percorso di eliminazione vincolato agli allegati. Di conseguenza una vista ricalcolata oggi può differire da quanto l'utente aveva letto il mese scorso.

`docs/DATA-PORTABILITY.md` documenta **dodici esportazioni CSV integrali**, indipendenti dai filtri e dall'anno selezionato: sono registri utili, **non un rendiconto mensile deliberatamente riesaminato** e non un backup ripristinabile. `docs/MVP.md` indica limiti di collaudo: non presumere che il prodotto sia già pronto come unico archivio di produzione.

**Problema potenziale da validare:** un artista o piccolo studio può voler rivedere i numeri di un periodo, conservare *che cosa risultava all'epoca del controllo* e distinguere le modifiche successive. Trascrivere i valori in un foglio è possibile, ma il rischio è presentare come «incassato» il venduto, omettere una spesa tardiva o consegnare a un consulente dati personali non necessari. Questo non è un bug riscontrato, una riconciliazione bancaria, una dichiarazione fiscale, né un'esigenza già confermata da utenti.

**Destinatari da intervistare:** artisti con molte vendite e acquisti mensili; studi con una seconda persona che controlla le registrazioni; piccole gallerie che preparano riepiloghi periodici. Per artisti con poche operazioni il pannello attuale e un foglio locale potrebbero essere la soluzione migliore.

## 2. Confini precisi e assenza di duplicazioni

- **Funzioni presenti:** la vista Economia, i dodici mesi, gli incassi per mese e i CSV rimangono le fonti esistenti. **Non** crearne copie indipendenti, non alterare criteri di margine, importi, spese, vendite, saldi o quote.
- **[UX-030 — Quadratura bancaria](REPS-UX-030-quadratura-facoltativa-estratti-conto-locali.md):** confronta eventuali righe di un estratto conto importato esplicitamente. Qui **nessuna banca né riconciliazione**: solo controllo di coerenza delle registrazioni gestionali locali.
- **[UX-032 — Budget mensili](REPS-UX-032-budget-mensili-categorie-spesa-scostamenti.md)** e **[UX-047 — Budget mostre/fiere](REPS-UX-047-budget-consuntivo-mostre-fiere-spese-collegate.md):** confrontano preventivi e consuntivi. Qui **non c'è alcun budget**, né attribuzione delle spese a progetti.
- **[UX-039 — Ripartizione informativa degli incassi](REPS-UX-039-ripartizione-informativa-incassi-dichiarati.md):** propone quote teoriche sugli incassi, distinte da quote del venduto. Il rendiconto può mostrarle in futuro soltanto se quella semantica sarà prima definita; **non** deve ricalcolare nuove quote sulle somme ricevute.
- **[UX-024 — Rate](REPS-UX-024-scadenziario-incassi-piani-pagamento-vendite.md), [UX-027 — Valute](REPS-UX-027-valute-estere-vendite-incassi-costi.md), [UX-028 — Giustificativi](REPS-UX-028-giustificativi-locali-spese-acquisti.md):** scadenze, valute e documenti restano nei propri domini, non vengono creati dal riepilogo.
- **Filone tecnico [PR #16](https://github.com/Cesare-ml/Repertica-Studio/pull/16):** TECH-018/025 progettano robustezza e minimizzazione delle **esportazioni in generale**; TECH-001 studia un **pacchetto aperto ripristinabile**; TECH-010 tratta revisioni dei **metadati delle opere**; TECH-042 compatibilità delle versioni. Questa proposta definisce soltanto **significato e UX di un riepilogo economico datato**, riutilizzando eventualmente l'infrastruttura tecnica, senza modificare file dell'altro filone.

**Fuori ambito:** «chiusura» che impedisce di correggere il mese, blocchi contabili, prima nota fiscale, IVA deducibile, bilancio civilistico, dichiarazioni fiscali, saldo bancario certificato, fatture, notifiche, invio automatico al commercialista, collegamento cloud, identificazione dell'operatore mediante un account inesistente. Il termine preferito è **«Rendiconto del periodo»**, non «Chiusura definitiva».

## 3. Scenari verificabili

1. **Mese di vendite e incassi parziali.** Due vendite registrate nel mese: 1.200 € e 500 €; imposte/spedizione registrate complessivamente per 200 €, spese per 200 €, incassi dichiarati nel mese per 400 €. Il riepilogo mostra separatamente **venduto lordo 1.700 €**, **netto vendite 1.500 €**, **spese 200 €**, **margine gestionale 1.300 €**, **incassi dichiarati 400 €**. Il margine **non** è disponibilità bancaria; l'ultimo importo può riguardare anche vendite di altri mesi. Non sottrarre automaticamente i 400 € dai 1.700 € per una falsa «cassa mensile».
2. **Correzione retrodatata.** Dopo aver esportato il riepilogo di ottobre, l'utente modifica una spesa di ottobre da 80 a 100 €. La vista attuale si aggiorna come oggi; il vecchio documento resta esplicitamente **«fotografia generata il ...»**. Se esistono snapshot locali opzionali, mostrare «i dati correnti differiscono» e offrire un **nuovo** rendiconto, senza sovrascrivere o correggere silenziosamente il precedente.
3. **Finestra di controllo prima della condivisione.** La persona controlla mesi con importi negativi di margine, vendite con residuo e voci senza fornitore o categorie riconoscibili: il pannello le segnala solo come **«da verificare»** quando ha dati sufficienti. Campo fornitore assente **non è di per sé errore** e un residuo aperto **non equivale a insolvenza**.
4. **Dati storici senza incassi.** Un archivio migrato può contenere vendite e nessun incasso: mostrare «nessun incasso registrato» e mai «tutte le vendite non pagate» come fatto accertato, né 0 € come prova bancaria.
5. **Invio selettivo a un consulente.** L'utente crea un CSV riepilogativo **aggregato** per mese, senza automaticamente includere collezionisti, note, indirizzi, fornitori o titoli di opere. Una scelta più dettagliata è esplicita e revisionabile prima del salvataggio, mai un invio diretto.
6. **Periodo parziale.** Se `treasury_start_date` taglia un mese, mostrare **«periodo parziale dal ...»** e non confrontare il mese senza avviso con mesi interi. Un mese senza record va distinto da un periodo di dati incompleti.

## 4. Flussi UX suggeriti

**Ingresso facoltativo:** `Economia → Rendiconto del periodo`, collocato vicino ai riepiloghi **già presenti**, senza nuovo modulo principale. Selezione **mese e anno**; mostrare subito il perimetro di date, l'eventuale `treasury_start_date` e una spiegazione compatta: «I risultati derivano dalle registrazioni; non verificano i pagamenti».

**Vista in sola lettura:** tre blocchi chiaramente etichettati: (A) vendite e margine **per data vendita**; (B) spese **per data spesa**; (C) incassi dichiarati **per data incasso**. Totali, conteggi, mese, valuta e criteri; azione «Mostra registrazioni incluse» che apre i filtri sui registri **esistenti**, senza creare una seconda fonte contabile. Non rappresentare spese come addebiti bancari e non sottrarre incassi al margine.

**Piccola checklist non coercitiva:** ad esempio «ricontrollati i dati delle vendite», «ricontrollati incassi e residui», «ricontrollate le spese», con opzione «non verificato» e annotazione locale. Una spunta esprime **solo una dichiarazione dell'utente**, non prova dell'esattezza; non bloccare operazioni, non attribuire nome e firma verificata all'autore della spunta.

**Anteprima e output:** «Esporta riepilogo mensile» con schema/dizionario dei campi e distinzione fra **totali dichiarati** e **valori derivati**. Opzione minima: CSV compatto con etichette canoniche e importi in **centesimi**, formato locale presentato nella UI; eventualmente PDF leggibile/ODT solo se realmente richiesto. Il salvataggio su percorso scelto avviene dopo anteprima, senza sovrascrittura di file. L'export è una **copia statica**, non una sincronizzazione o un backup. Niente contenuti di email, recapiti o allegati per default; nessun invio.

**Revisione successiva (solo se serve):** elenco «Rendiconti salvati nell'archivio» distinto da file esportati all'esterno; confronto informativo dei totali salvati e di quelli attuali, avvisando quando sono cambiati. La parola «confermato» non deve impedire la correzione delle registrazioni originali. Una nuova revisione porta nuova data/ID; l'originale non viene modificato.

**Accessibilità:** tutte le operazioni da tastiera, etichette dei criteri di periodo lette da screen reader, tabelle semantiche (non solo grafici/colori), segni più/meno pronunciabili, focus persistente dopo anteprima/esportazione, formati numerici senza ambiguità e zoom/finestra ridotta su Windows/macOS/Linux.

## 5. Dati necessari, impostazioni e possibili API (solo descrittivi)

**Fase minima senza nuova tabella:** `period = YYYY-MM`, `range_start/range_end`, `treasury_start_date` già esistente, `currency = EUR` nella situazione attuale, viste `FinanceOverview/MonthlyFinance` e riepilogo incassi già disponibili, più conteggi derivati dai registri autorevoli. I calcoli devono prendere i dati da **una lettura coerente** dello stesso archivio: se una modifica interviene durante la costruzione, non pubblicare una combinazione di istanti diversi. I valori monetari restano interi in centesimi; percentuali in punti base quando necessarie. Non riapplicare il piano corrente alle vendite storiche.

**Solo in fase eventuale di snapshot locale**, concetto `PeriodReview`:
- `review_id` UUID, periodo ISO, `created_at_utc`, versione della definizione del rendiconto, versione schema/app, filtro tesoreria applicato, valuta, totali e conteggi etichettati con la **base temporale**;
- `checklist` facoltativa con `unchecked/declared_checked/not_applicable`, note sintetiche, senza finta identità dell'operatore;
- impronta/fingerprint dei **dati sorgente selezionati** o manifest locale versionato *solo* per individuare differenze successive, non per attestare autenticità o impedire manipolazione; valutare quanto di dati riservati e di cronologia è davvero necessario conservare;
- `supersedes_review_id` opzionale per collegare una revisione successiva; mai scrivere l'ID del documento come nuova vendita, spesa o incasso; evitare di salvare automaticamente path assoluti di esportazioni esterne.

Possibili operazioni **descrittive, non API implementate**: `previewPeriodReview(period, filters)`, `exportPeriodSummary(preview, fields, target)`, `savePeriodSnapshot(preview, notes)`, `compareCurrentWithSnapshot(id)`. Il backend dovrebbe ricontrollare che il dataset non sia cambiato fra anteprima e conferma; su conflitto chiedere di rigenerare, non pubblicare numeri obsoleti come aggiornati. Se servisse fedeltà alle **singole righe storiche** dopo modifiche o cancellazioni, un solo digest e totali non bastano: ciò richiederebbe versionamento dedicato, **non autorizzato** da questa idea.

**Impostazioni:** nessuna procedura di chiusura obbligatoria, nessun cambio ai mesi o al filtro `treasury_start_date`; eventualmente preferenza per formato di export e inclusione di note, **disattivata per default** per dati privati. Se verranno adottate valute multiple in futuro, non sommare valute diverse né convertire automaticamente con servizi di rete. Date e format UI dalla lingua di sistema, fallback inglese, con override già previsto IT/EN/ES/FR/DE; codici, date ISO e nomi di colonne tecniche **indipendenti dalla traduzione**. I dati testuali liberi rimangono nella lingua scritta dall'utente.

## 6. Offline, import/export, privacy, compatibilità e dipendenze

- **Offline:** i numeri derivano soltanto da SQLite locale. Nessuna credenziale, account, servizio remoto, telemetria, invio automatico o collegamento bancario. Cartelle per esportare e fare backup sempre liberamente scelte dall'utente.
- **Esportazione:** CSV riepilogativo *aggiuntivo* rispetto ai 12 registri integrali esistenti, con schema/versione espliciti, date ISO, centesimi e semantica dei campi; non modificare i CSV integrali. Non rappresentare il file come documento fiscale o backup. Le tabelle di dettaglio, se richieste, passano da selezione e anteprima privacy. Gli aggregati di un piccolo archivio possono essere riconducibili a singole persone: non promettere anonimato.
- **Import:** non necessario nella prima fase; per uno snapshot portabile futuro, un JSON versionato **solo descrittivo/di consultazione** può essere aperto in anteprima e confrontato senza importare vendite/incassi/spese né creare record economici. Il ripristino vero resta il backup completo SQLite+media con manifest e verifica, non CSV/JSON riassuntivi.
- **Backup e ripristino:** ogni eventuale `PeriodReview` memorizzato nell'archivio entra nel normale backup completo; file CSV/PDF salvati *fuori* dall'archivio non vengono automaticamente inclusi. Il ripristino non deve inventare revisioni successive né fondere rendiconti di backup differenti. I backup attuali **non sono cifrati**; avvisare su USB/NAS condivisi.
- **Compatibilità:** prima fase senza nuove migrazioni; fase snapshot con schema additivo e transazioni, mantenendo archivi storici senza alcun rendiconto generato. Backup di versioni più nuove possono non essere apribili con release precedenti: messaggi chiari. Nessun cambiamento a `sales`, `expenses`, `sale_receipts`, `sale_allocations`, certificati immutabili, identità UUID e vincoli di una vendita per opera nell'MVP. Non alterare la possibilità di correggere una spesa o incasso per motivi legittimi.
- **Privacy/sicurezza:** contenuti economici, fornitori, nomi clienti, note e link d'acquisto sono potenzialmente riservati; minimizzazione per default, anteprima dei campi, protezione anti-formula CSV, gestione di errori/salvataggi senza sovrascrivere documenti. Un hash o screenshot di un riepilogo non certifica assenza di modifiche malevole al database.
- **Dipendenze:** report già esistenti `finance.rs`/`receipts.rs`, SQLite/backup, dialoghi nativi e i18n. Eventuali esportazioni robuste/selective dovrebbero seguire i contratti del filone tecnico senza modificarli. **Nessuna dipendenza obbligatoria** da UX-030/032/039/047 o da server MCP, mail professionale, cloud e Sites.

## 7. Complessità, rischi, alternative e criteri di accettazione

**Complessità indicativa:** **bassa–media** per un'anteprima leggibile e CSV aggregato (riuso di report già presenti, da verificare); **media** per controlli coerenti e checklist; **medio-alta** per conservare e confrontare snapshot versionati e riconoscere correzioni retroattive senza ricostruire una falsa storia delle righe. Nessuna stima in giorni senza prova sul codice.

**Rischi:** confusione fra rendiconto e «chiusura», dati vecchi presentati come correnti, venduto chiamato incassato, margine scambiato per profitto fiscale, spese sommate due volte, contatti privati esportati, falsi errori per campi opzionali, falsa garanzia di integrità, nuovo menu eccessivo. Mitigazioni: etichette precise, basi temporali esplicite, alert su mesi parziali, export minimale, report datato, nessuna modifica agli originali.

**Alternative da valutare prima di sviluppare:** (A) esportare i registri attuali e comporre un modello **LibreOffice Calc offline** con legenda e filtri mensili; (B) aggiungere soltanto stampa/CSV del pannello mensile *senza conservare revisioni*; (C) snapshot locali solo dopo prove d'uso di discrepanze tra rendiconti successivi. Se (A) è sufficiente, **nessuno sviluppo**.

**Criteri di accettazione ipotetici:**
1. L'esempio numerico dello scenario 1 produce 1.700/1.500/200/1.300/400 € nei cinque indicatori, con etichette e date di competenza; l'incasso non entra come seconda vendita.
2. Una spesa retrodatata modifica il mese nella **vista corrente** ma non riscrive un documento già esportato o uno snapshot storico; il confronto non proclama *quale riga* sia cambiata se non dispone di cronologia sufficiente.
3. Mese senza registrazioni, vendite senza incassi e periodo tagliato da `treasury_start_date` restano casi distinti e correttamente etichettati.
4. Una percentuale di ripartizione cambiata oggi non ricalcola le quote congelate delle vendite passate; un resoconto non crea fondi o pagamenti.
5. Esportazione predefinita senza dati identificativi; condivisione avanzata solo dopo selezione e anteprima; nessuna sovrascrittura e nessuna formula CSV eseguibile.
6. Con archivi grandi il risultato deriva da una lettura coerente; la variazione durante la conferma mostra conflitto recuperabile, mai un documento ambiguo. Errori di I/O non lasciano file finali parziali.
7. Se in futuro memorizzati, gli snapshot sopravvivono a backup/ripristino locale verificato, senza necessità di rete o account; formati e riferimenti restano esportabili.
8. Tutto il flusso è operabile da tastiera e screen reader, senza affidarsi solo al colore, con lingua di sistema/fallback inglese e importi sempre leggibili su Windows/macOS/Linux.

## 8. Fasi decisionali (non lavori assegnati), requisiti certi, ipotesi e domande

**Fase 0 — validazione:** raccogliere 5–8 esempi *anonimizzati* di resoconti reali, chiedere come vengono condivisi, quali controlli richiedono e se la vista attuale/Calc bastano. Verificare con particolare attenzione il significato locale di «netto» e «margine».

**Fase A — se davvero utile:** anteprima mensile e CSV aggregato in sola lettura, senza nuovo database né blocco dei mesi; riutilizzare fonti esistenti.

**Fase B — condizionata:** checklist leggera e documento datato. Distinguere «controllato» da «pagamento accertato» e «dichiarazione verificata».

**Fase C — solo dopo prove concrete:** snapshot versionati nell'archivio, confronto con stato corrente, eventuale JSON consultabile; nessun audit fiscale implicito.

**Requisiti certi:** desktop gratuito/open source Tauri, SQLite e media locali, offline-first senza account/abbonamento, dati e backup esportabili su cartella scelta, i18n sistema con fallback EN, preservazione di vendite/incassi/spese/quote/certificati e loro significato attuale.

**Ipotesi:** utenti desiderano un riepilogo periodico riutilizzabile e una traccia delle differenze dopo correzioni; una UI aggiuntiva sarebbe preferibile a Calc; esiste un pubblico abbastanza ampio da giustificare nuove tabelle.

**Questioni aperte per il proprietario:** (1) c'è chi consegna ogni mese prospetti a un consulente o collaboratore? (2) basta l'export del mese oppure serve confrontare *due fotografie* successive? (3) preferisci un pannello nell'Economia esistente, senza voce autonoma? (4) il documento dovrebbe contenere solo aggregati o poter includere un dettaglio esplicitamente selezionato? (5) quali controlli minimi sono utili senza sconfinare nella contabilità fiscale?

**Decisione:** nessuna proposta approvata o assegnata allo sviluppo.
