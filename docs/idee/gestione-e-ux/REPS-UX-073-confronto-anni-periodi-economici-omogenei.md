# REPS-UX-073 — Confronto tra anni su periodi economici omogenei

- **ID stabile:** REPS-UX-073
- **Data:** 2026-11-23
- **Stato:** **Proposta — da valutare**
- **Filone:** Gestione e UX
- **Priorità ipotetica:** **P2** soltanto se il confronto ricorrente risulta utile in prove con professionisti; **P3 o nessuno sviluppo** se i riepiloghi annuali attuali e LibreOffice Calc soddisfano il bisogno.
- **Decisione:** nessuna approvazione, nessuna assegnazione o autorizzazione allo sviluppo. Questa è documentazione ideativa, non una specifica implementativa approvata.

## 1. Riscontro sul prodotto, problema e destinatari

**Osservato sul branch `main` il 23 novembre 2026, senza prove desktop:** la sezione **Economia** (`src/app/app.component.html`, circa righe 219–280) presenta un **solo anno di riferimento** selezionato, totali di venduto lordo, netto vendite, spese registrate e margine, tabella di dodici mesi e istogrammi. Mostra separatamente gli **incassi dichiarati per mese**. `app.component.ts::loadFinanceYear` (circa righe 629–637) richiede `api.financeOverview(financeYear)` e `api.receiptYearOverview(financeYear)`, sempre per lo stesso anno mostrato.

`src-tauri/src/finance.rs::finance_overview_core` genera già dodici `MonthlyFinance` per l'anno selezionato; conta **vendite per data della vendita** e **spese per data di spesa**, applicando l'eventuale `treasury_start_date` separatamente alle rispettive righe. `docs/ECONOMY-I18N.md` definisce il margine come *netto vendite meno spese registrate*, non utile fiscale. `src-tauri/src/receipts.rs` e `docs/SALE-RECEIPTS.md` definiscono gli incassi per **data dichiarata di ricezione** e non per data vendita. Il riepilogo annuo degli incassi non risulta applicare lo stesso `treasury_start_date`: non assumere basi identiche senza verifica.

**Possibile bisogno, non ancora confermato:** chi dirige uno studio, una galleria o un piccolo archivio artistico vuole capire se nei **primi dieci mesi** dell'anno sono aumentate vendite registrate, spese o incassi rispetto agli **stessi dieci mesi** dell'anno precedente, senza riportare a mano dodici righe in un foglio. Il confronto tra un anno precedente completo e quello corrente incompleto può produrre interpretazioni errate. La soluzione deve essere prima di tutto **comprensibile**, non un nuovo motore di contabilità.

**Destinatari da verificare:** artisti con operazioni durante più anni, gallerie con picchi stagionali (fiere/mostre), gestori di studio che rivedono spese e quote. Per archivi piccoli o con pochi anni, il confronto potrebbe non avere valore.

## 2. Casi d'uso e flussi UX

1. **Confronto 2026/2025 a novembre 2026.** L'utente apre Economia → «Confronta con l'anno precedente»; il criterio iniziale è **gennaio–ottobre 2026 vs gennaio–ottobre 2025**, cioè **soli mesi civili completati**. Le intestazioni esplicitano i due intervalli e la data di aggiornamento; **novembre incompleto non è inserito silenziosamente**. È possibile scegliere un intervallo diverso di mesi interi, indicato con chiarezza.
2. **Confronto fra due anni completi.** L'utente sceglie il 2024 e il 2025, gennaio–dicembre, per confrontare stagionalità senza aggiungere previsioni. Vede due colonne per ogni mese e il delta in euro. I valori sono derivati dai registri attuali e possono cambiare se si corregge una registrazione storica.
3. **Anno precedente senza dati o senza storia completa.** Se nel 2025 non esistono righe, visualizzare «Nessuna registrazione nel periodo» invece di dichiarare che l'attività fosse nulla. Delta in centesimi leggibile, ma **nessuna crescita percentuale** quando la base è zero, negativa o non comparabile.
4. **Filtro tesoreria con inizio a metà anno.** Se `treasury_start_date` è 2026-06-15, i mesi antecedenti risultano esclusi dai riepiloghi vendite/spese 2026; la vista segnala **«periodi non confrontabili per inizio rilevazione»** e non espone una percentuale fuorviante. Non tratta i dati precedenti come zero economico effettivo. Gli incassi hanno una base separata da controllare.
5. **Vendita a dicembre e incasso a gennaio.** La vendita compare nel confronto del venduto di dicembre, l'incasso in quello di gennaio dell'anno successivo. **Non trasformare** gli incassi in vendite né sommarli al margine; una differenza fra i due non indica automaticamente insolvenza.
6. **Condivisione volontaria con consulente.** «Esporta confronto CSV aggregato» propone solo mesi, intervalli, valuta, totali/delta e metodologia, senza nomi di collezionisti, opere, note o fornitori; l'utente sceglie cartella e nome. Nessuna email o condivisione automatica.

**Flusso minimo:** Economia → anno di riferimento già presente → azione discreta «Confronta periodo» → secondo anno selezionabile (per default il precedente) → intervallo di mesi interi **uguale in entrambi gli anni** → tre aree distinte **vendite/spese/margine**, **incassi dichiarati**, **note sui criteri** → possibilità di tornare alla vista singola senza cambiare dati o impostazioni dell'archivio. Preferire una tabella accessibile e numeri leggibili a grafici ridondanti. Se l'anno è il primo con dati, mostrare un messaggio utile, non una comparazione ingannevole.

**Controlli UI:** date e denominatori sempre visibili; scelta esplicita «12 mesi completi» per anni storici o «mesi completi fino all'ultimo mese chiuso» nell'anno in corso; eventuale scelta di includere il mese corrente **solo in una futura modalità con granularità giornaliera corretta**, non simulandola con gli aggregati mensili già disponibili. Tabelle con intestazioni anno/mese, segno del delta e descrizione testuale, non solo rosso/verde.

## 3. Dati, formule e impostazioni (ipotesi descrittive)

**Già disponibili:** `FinanceOverview(year, months[12])`, `MonthlyFinance(month, gross_cents, net_cents, expense_cents, margin_cents, ...)`, `ReceiptYearOverview(year, months[12], count, total_cents)`, importi in **centesimi interi**, date ISO, `StudioSettings.treasury_start_date`. Per la fase minima due letture di anni diversi e aggregazione degli **stessi mesi** potrebbero bastare, purché si garantisca coerenza del dataset in caso di modifiche concorrenti (confronto da rigenerare se i dati cambiano).

**Modello concettuale soltanto:** `ComparablePeriods { baseYear, comparisonYear, fromMonth, toMonth, completedMonthsOnly, currency, treasuryStartDate, sourceDefinitionVersion }`; `SeriesComparison { metric, baseCents, comparisonCents, deltaCents, percentChange?: number, comparable, reason? }`. Le percentuali sono solo *visualizzazioni calcolate da valori interi*, mai denaro float da memorizzare. Denominatori **zero, negativi o parziali** producono `percentChange = null` e spiegazione testuale. Non sommare o convertire importi in valute diverse se viene realizzata la futura proposta multivaluta.

**Definizioni immutate:**
- Venduto lordo: `sales` per `sold_at`, non incassi;
- Netto vendite: lordo meno tasse/spedizione dichiarate sulle vendite;
- Spese: `expenses` per `expense_date`, non uscite bancarie verificate;
- Margine: netto vendite meno spese del periodo, **non liquidità né utile fiscale**;
- Incassi: `sale_receipts` per `received_at`, senza dedurne la competenza temporale delle vendite;
- Quote dei fondi: non ricalcolare con il piano attuale gli importi delle vendite storiche; **non confrontare saldi teorici come denaro disponibile**.

**Impostazioni:** nessuna obbligatoria. Anni e mesi sono scelte della singola consultazione, inizialmente non persistite; la lingua dipende dal sistema con fallback inglese (override manuale già previsto), senza memorizzare numeri come stringhe localizzate. Attenersi al `treasury_start_date` esistente, **senza modificarlo**; mostrare esplicitamente quando filtra solo alcuni mesi/metriche. Nessuna soglia automatica di «buona crescita» né benchmark esterni.

## 4. Offline, dati personali, accessibilità, import/export e compatibilità

- **Offline e indipendenza:** tutto calcolato localmente sull'archivio SQLite aperto, senza account, SaaS, abbonamento, internet, analisi IA, telemetria o collegamento a banche. L'azione non altera vendite, spese, incassi, ripartizioni, certificati o media.
- **Privacy:** tabelle aggregate in app; nessuna visualizzazione predefinita di identità, indirizzi, note, titoli di opere o dati bancari. Ricordare che piccole aggregazioni possono comunque essere sensibili. Eventuali esportazioni solo dopo anteprima e salvataggio locale esplicito, file non sovrascritti e neutralizzazione di campi testuali potenzialmente interpretati come formule CSV.
- **Import/export:** nessuna importazione richiesta nella fase minima. Eventuale CSV aperto **aggiuntivo** con colonne stabili, periodo ISO, anno base/di confronto, codice metrica, valuta, centesimi, differenza e nota sul filtro di tesoreria; non cambiare gli export completi esistenti. Un confronto esportato **non è backup**, non è importabile per creare registrazioni e non attesta valori bancari. Per l'archivio completo resta il backup verificato SQLite+media su cartella scelta.
- **Compatibilità:** in forma minima, nessuna nuova tabella/schema; compatibile con archivi migrati che abbiano dati parziali. Il riuso di API annue esistenti non deve produrre un confronto incoerente se fra una richiesta e l'altra l'archivio cambia: valutare snapshot di lettura consistente o avviso/refresh. Non promettere ripristino dei CSV come database. Multivaluta eventuale: serie per valuta senza conversioni di rete.
- **Multilingua e accessibilità:** lingua di sistema IT/EN/ES/FR/DE quando supportata, inglese in fallback; mesi e decimali secondo lingua UI, **colonne tecniche esportate invarianti**. Navigazione completa da tastiera, focus leggibile, etichette di tutte le celle, annunci per intervallo selezionato, testo per «non confrontabile», differenze anche con +/− e non solo colori. Dimensioni ridotte su Windows/macOS/Linux senza nascondere gli anni a confronto.

## 5. Complessità, dipendenze, rischi, alternative

- **Complessità:** bassa–media per mostrare due serie mensili nella UI riutilizzando le risposte già disponibili; **media** per lettura coerente, segnalazione dei mesi incompleti, compatibilità del filtro tesoreria ed export comparativo; alta e qui esclusa per storico immutabile, fiscalità, sincronizzazione o previsioni.
- **Dipendenze concettuali:** solo report economici e incassi già presenti, traduzioni UI, export locale e verifica della coerenza di lettura. **Nessun requisito** di sviluppare prima budget, contabilità avanzata, riconciliazione bancaria, cloud o nuove librerie di grafici.
- **Rischi:** confronti fra annualità disomogenee, dati inseriti retroattivamente, `treasury_start_date` che tronca solo le serie vendite/spese, inizio archivio non noto, percentuali su basi zero/negative, marginalità spacciata per utile, incassi spacciati per venduto, interpretazione di «nessuna registrazione» come prova di zero attività, export di dati riservati.
- **Mitigazioni:** stesse finestre mensili, criteri leggibili, niente percentuali se incomparabili, nessuna conclusione automatica, refresh su dati cambiati, legenda separata per vendite/spese/incassi e avvisi per date di inizio raccolta. Richiedere verifiche con dati sintetici prima di affidare qualunque scelta gestionale ai confronti.
- **Alternativa A — prima dello sviluppo:** esportare CSV integrali e fornire **solo un esempio di foglio LibreOffice Calc locale** con periodi omogenei e definizioni; nessuna nuova funzione.
- **Alternativa B:** affiancare due schede annuali in sola lettura senza delta percentuali o export: utile se l'unico problema è alternare il selettore dell'anno.
- **Alternativa C:** confronto a gruppi di mesi con delta e solo dopo eventuale CSV aggregato. Nessuna memoria permanente, snapshot o «rendiconto ufficiale».

## 6. Requisiti certi, ipotesi, domande al proprietario

**Requisiti certi (vincoli di prodotto):** desktop open source gratuito, Tauri, offline-first e nessun account obbligatorio; database locale, backup su cartella scelta, esportabilità dei dati; lingua di sistema con fallback inglese; distinzione fra vendite, incassi e spese; nessuna modifica automatica dei record storici.

**Riscontri nel codice:** già esistono le dodici mensilità annuali, il selettore di un anno per volta, `treasury_start_date` per vendite/spese, un riepilogo degli incassi per data `received_at` e registri CSV. **Non** è verificata l'usabilità su desktop installato, né la necessità effettiva del nuovo confronto.

**Ipotesi da testare:** gli utenti consultano almeno due annualità più volte al mese; confrontare gli **stessi mesi** riduce errori e tempo; la distinzione dei tre domini economici è compresa; aggiungere un confronto non appesantisce la pagina per piccoli archivi.

**Domande per la revisione personale (non bloccanti):**
1. È più utile il confronto anno-su-anno di **mesi completi**, o soltanto una tabella con due annualità affiancate?
2. Per attività appena avviate, preferisci non mostrare alcuna percentuale, o consentire numeri assoluti con avviso evidente sui dati incompleti?
3. Il confronto degli incassi dichiarati deve stare in un pannello separato, senza tentare un saldo di cassa?
4. L'esportazione CSV sintetica avrebbe reale utilità rispetto ai registri completi e al foglio Calc?
5. Sono prioritari confronti con l'anno precedente oppure una scelta manuale di due anni distanti?

## 7. Fasi di valutazione e criteri di accettazione (futuri, non eseguiti)

**Fase 0 — validazione:** osservare 4–6 artisti/galleristi che hanno almeno due anni di dati (anche anonimizzati/sintetici), chiedendo di confrontare gennaio–ottobre di due annualità con gli strumenti attuali. Misurare tempo, errori di periodo e fraintendimenti fra venduto/incassato. Se basta Calc, **non implementare**.

**Fase 1 — prototipo UX:** confronto in sola lettura con tabella 12 mesi e due colonne per anno; verifica manuale di leggibilità, trasparenza dei criteri e focus/tastiera. Nessuna persistenza.

**Fase 2 — solo previa approvazione separata:** definire dati coerenti e gestione delle eccezioni (filtro tesoreria, valute future, archivio incompleto); valutare un semplice export comparativo soltanto se gli utenti lo richiedono.

**Criteri di accettazione proposti, non test eseguiti:**
1. Il 23 novembre 2026 il default 2026 vs 2025 confronta **gennaio–ottobre con gennaio–ottobre**; novembre e dicembre non sono scambiati per mesi completi. Per due anni terminati si possono confrontare gennaio–dicembre.
2. Una vendita del 2025 incassata nel 2026 resta nel venduto 2025 e negli incassi dichiarati 2026; non è conteggiata due volte nelle stesse metriche.
3. Con 2025 = 0 € e 2026 = 100 €, il delta è +100 €, **non «+∞%»**, e c'è un messaggio adeguato; con margine negativo non appare una percentuale ingannevole.
4. Con `treasury_start_date = 2026-06-15`, vendite/spese dell'anno corrente parziali sono riconosciute e la vista evita una comparazione fittiziamente omogenea; la serie degli incassi non eredita quel filtro senza un contratto esplicito e verificato.
5. Cambiare coefficiente artista o piano corrente delle percentuali **non** ricalcola retroattivamente vendite/quote; modificare legittimamente una spesa ricalcola la **vista corrente** senza alterare documenti storici già esportati.
6. Tabelle e criteri sono completi da tastiera/screen reader, in IT/EN e fallback inglese, leggibili a finestra ridotta.
7. Il confronto usa dati del medesimo archivio in modo coerente; in caso di cambio archivio o modifiche concorrenti non mostra serie miste come se fossero una fotografia affidabile.
8. L'eventuale CSV ha date ISO, importi in centesimi, definizioni e valuta, nessuna identità cliente per default e non modifica i registri o il backup.

## 8. Anti-duplicazioni e confini con l'altro filone

- **REPS-UX-050:** riguarda il **rendiconto mensile datato e le revisioni nel tempo**. Qui soltanto **confronto dinamico di stessi mesi di due anni**, senza snapshot, attestazioni, checklist o chiusura.
- **REPS-UX-032 / UX-047:** budget vs consuntivo preventivato; qui solo **periodi storici effettivamente registrati**, nessun budget.
- **REPS-UX-034:** analisi delle vendite per attributi delle opere; qui solo serie **temporali economiche aggregate**, senza tecnica/supporto o segmentazione clienti.
- **REPS-UX-039:** quote teoriche sugli incassi; qui incassi come metrica **separata**, senza distribuire quote.
- **REPS-UX-027 / UX-030:** multivaluta e riconciliazione bancaria sono fuori ambito.
- **Filone tecnico PR #16:** TECH-008 riguarda ricerca/filtri avanzati, TECH-018/025/055/057/altre proposte di interoperabilità riguardano l'infrastruttura degli export; qui si definiscono soltanto **semantica comparativa e flusso UX**, senza modificare file dell'altro filone.

**Stato finale:** Proposta — da valutare. Nessuna funzionalità approvata o assegnata, nessuna modifica a codice, test, build, workflow, impostazioni o dipendenze, nessuna issue o PR implementativa.