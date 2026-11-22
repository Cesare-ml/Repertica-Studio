# REPS-UX-033 — Entrate professionali non derivanti dalla vendita di opere

- **ID stabile:** REPS-UX-033
- **Data della ricognizione:** 2026-11-22
- **Stato:** Proposta — da valutare
- **Filone:** Gestione e UX
- **Priorità ipotetica:** P2; P1–P2 soltanto se artisti e studi confermano entrate extra-vendita ricorrenti
- **Natura:** ricerca e progettazione documentale. Nessun requisito qui è approvato, assegnato o autorizzato per l'implementazione.

## 1. Problema e riscontri sul prodotto

L'attività economica di un artista non coincide necessariamente con la cessione delle opere fisiche. Possono esserci **compensi per laboratori didattici, partecipazioni retribuite, prestazioni creative non collegate a un'opera catalogata e corrispettivi per licenze o riproduzioni**. Oggi un utente potrebbe tenere questi valori in un foglio esterno, oppure inserirli impropriamente come vendita di un'opera, producendo un archivio e statistiche fuorvianti.

**Riscontro limitato al codice e alle docs di `main` al 22/11/2026:** [README](../../../README.md), [Economia e localizzazione](../../ECONOMY-I18N.md), [`finance.rs`](../../../src-tauri/src/finance.rs), [`models.rs`](../../../src-tauri/src/models.rs), [portabilità CSV](../../DATA-PORTABILITY.md). `finance_overview_core` aggrega gli importi di `sales` (con `sale_finances`) e `expenses` per mese; le vendite richiedono un'opera e un collezionista. L'elenco dei 12 CSV non menziona un registro di entrate professionali extra-vendita. **Non emerge**, nei percorsi consultati, un registro generico di ricavi separati dalle vendite. Questo non dimostra un difetto runtime né la frequenza del bisogno. La [issue #13](https://github.com/Cesare-ml/Repertica-Studio/issues/13) parla di incassi/cashflow e acquisti; non va considerata chiusa, implementata integralmente o equivalente a un registro di nuove fonti di ricavo.

**Destinatari:** artista indipendente che alterna vendite, corsi e incarichi professionali; piccolo studio con entrate da workshop; galleria con compensi per servizi distinti dalle opere; archivio d'artista che annota corrispettivi di utilizzo delle immagini. Non è necessario per chi gestisce soltanto opere e vendite; il percorso ordinario deve rimanere identico.

## 2. Casi d'uso da verificare con professionisti

1. **Workshop:** in aprile un artista annota un compenso pattuito di 250,00 € per un laboratorio. La voce compare in «Altre entrate registrate» del mese, ma **non** incrementa il numero di opere vendute, il fatturato delle vendite di opere o le quote finanziarie già congelate.
2. **Licenza immagine:** vengono concordati 180,00 € per l'uso di una fotografia. La scheda può citare una autorizzazione **dichiarata** futura (UX-010), senza inferire automaticamente titolarità dei diritti, validità del contratto, pagamento o avvenuta pubblicazione.
3. **Prestazione senza opera:** un piccolo studio riceve un compenso per allestimento o consulenza creativa: la voce non richiede di inventare un'opera, un collezionista o una vendita fittizia.
4. **Accordo non ancora incassato:** l'utente registra 300,00 € come corrispettivo **concordato/registrato**, ma non dichiara alcun versamento. La UI evita di mostrare «300 € incassati» o di aumentare un saldo di cassa verificato.
5. **Entrata cancellata o contestata:** una registrazione errata viene corretta lasciando, dove giustificato, nota e collegamento di rettifica. Non si inserisce una seconda voce negativa generica che possa essere scambiata per una spesa.
6. **Archivio senza extra-entrate:** non viene creata nessuna voce, la Panoramica e i report delle vendite continuano a mostrare gli stessi dati di prima.

## 3. Esperienza utente proposta

**Economia → Altre entrate** (pannello facoltativo): elenco con data, categoria, descrizione, controparte opzionale, importo e indicazione esplicita del fatto economico: «corrispettivo registrato» non significa «denaro ricevuto». Filtri per anno/mese, categoria, controparte e stato documentale; visibile «nessuna entrata di altro tipo registrata» quando vuoto. Non introdurre nell'MVP un CRM o una contabilità completa.

**Aggiungi entrata:** form breve per data del fatto dichiarato, categoria (es. didattica, compenso professionale, riproduzioni/licenze, altro), descrizione, importo positivo in centesimi, valuta e note. L'utente può collegare *facoltativamente* un'opera o contatto già esistente per contesto, ma **non** trasforma il record in vendita o certificato. Riepilogo prima del salvataggio: «questa operazione non registra un incasso bancario e non emette una fattura». Le categorie possono essere testo personalizzato senza attribuire automaticamente natura fiscale.

**Opzione successiva, da decidere:** «Pagamento dichiarato» con data e importo, solo su esplicita registrazione manuale, tenuto separato dal corrispettivo; eventuali pagamenti parziali richiedono un dominio di incassi proprio o un'interfaccia condivisa con il futuro registro della issue #13, **non** duplicazione arbitraria di `receipts` legati a vendite. Per la prima fase è preferibile mostrare **solo importi registrati** ed evitare qualunque campo «pagato» ambiguo.

**Riepilogo mensile:** mantenere identificabili le metriche storiche delle vendite di opere: *vendite lorde*, *vendite nette*, *altre entrate registrate*, *spese registrate*. Un eventuale **totale gestionale combinato** è una **nuova** metrica chiaramente etichettata, calcolata solo quando l'utente l'ha richiesta; non sostituire silenziosamente i valori `gross_cents`, `net_cents`, `margin_cents` o la serie dei report già esistenti. Non presentare le nuove entrate come incassi verificati o utile fiscale. Indicare la base temporale scelta e separarla da eventuali date di pagamento.

**Accessibilità:** interfaccia in una colonna a schermo stretto, campi etichettati, importi leggibili da screen reader, focus ripristinato dopo il salvataggio, errori descritti accanto ai campi, differenze fra «vendite» e «altre entrate» testuali e non soltanto cromatiche. Nessuna grafica essenziale senza equivalente tabellare.

## 4. Regole economiche indispensabili

- **Divieto di doppio conteggio:** un corrispettivo già registrato come `sale` non va reinserito come altra entrata. L'app dovrebbe presentare una scelta esplicita («È la vendita di un'opera? Usa Vendite») e prevedere eventuale collegamento di riferimento, non importi duplicati.
- **Separazione tra maturato, registrato e incassato:** un compenso promesso, una prestazione effettuata, una registrazione gestionale e il denaro ricevuto sono fatti distinti. Le etichette non possono suggerire un principio contabile universalmente valido: il momento di riconoscimento della voce è una **data dichiarata dall'utente**, senza certificazione fiscale.
- **Ripartizioni percentuali:** le quote e i fondi storici delle vendite rimangono immutabili. Non applicare automaticamente la ripartizione artista/attività/risparmio alle entrate extra-vendita: serve una scelta del proprietario e una policy separata, se necessaria.
- **Spese e costi:** una entrata non è un rimborso di spesa per definizione; non compensare automaticamente le `expenses` esistenti. Un eventuale rimborso documentato richiede convenzioni specifiche e non deve cancellare la spesa originale.
- **Rettifiche:** preferire note di correzione e legame esplicito all'originale, senza modifiche retroattive silenziose a report esportati. Gli storni formali, note di credito e fiscalità restano fuori ambito.
- **Valuta:** non inventare un cambio per un compenso in valuta diversa da EUR. Se UX-027 fosse approvata, coordinare codici e modalità; altrimenti la prima fase potrebbe essere limitata a EUR con avvertenza e importazione di altre valute rinviata.
- **Date:** accettare solo date valide e mantenere distinta data del fatto, data di inserimento e, in un'eventuale fase successiva, data di incasso dichiarato.

## 5. Dati e relazioni: ipotesi descrittive, NON schema approvato

Un'eventuale entità `other_income` con UUID immutabile, `record_date` (ISO `YYYY-MM-DD`), `category_code` o etichetta controllata, `description`, `amount_cents` intero positivo, `currency` (fase iniziale eventualmente `EUR`), `counterparty_name` facoltativo, `artwork_id` / `contact_id` opzionali, `notes`, `source_reference` manuale, `created_at`, `updated_at`, `corrects_id` facoltativo e stato di rettifica esplicito. Non attribuire a una controparte il ruolo di `collector` in automatico.

**Esempio di confine fra operazioni, non API da creare ora:** «registra altra entrata» deve essere separata da «registra vendita opera» e da «registra incasso vendita». Le consultazioni potrebbero restituire `other_income_monthly_total` come campo **nuovo**, senza riscrivere il significato di `gross_cents` esistente. Eventuali integrazioni future devono riferirsi agli UUID, non a descrizioni, nomi o importi come identità.

**Cancellazione:** richiedere conferma; eventuali giustificativi o rettifiche collegati impongono una politica di conservazione, non eliminazioni silenziose. Un record di natura economica non deve diventare un certificato di autenticità o un documento fiscale.

## 6. Impostazioni, offline, multilingua e privacy

- **Impostazioni:** sezione disattivabile/nascosta per chi usa solo l'archivio opere; categorie personali facoltative; unità monetaria visibile. Nessuna configurazione, account o partita IVA obbligatoria. Evitare opzioni che cambino retroattivamente la semantica dei report.
- **Offline-first:** SQLite locale e, solo se autorizzati in fase futura, allegati nel media store locale; nessun SaaS, fatturazione online, accesso bancario, cloud, IA remota, apertura automatica di URL o comunicazione con controparti.
- **Lingue:** lingua di sistema con possibilità di override IT/EN/ES/FR/DE e **fallback inglese**, come definito in [ECONOMY-I18N](../../ECONOMY-I18N.md). Etichette, messaggi, categorie predefinite e legenda dei report tradotti; descrizioni immesse dall'utente restano nella loro lingua. Codici dati non localizzati, date ISO nei formati di interscambio e importi interi in centesimi.
- **Riservatezza:** compensi, tariffe e controparti sono dati commerciali/personali; lista e anteprima mostrano solo dati pertinenti, non li inviano a servizi. I backup completi sono **attualmente in chiaro**, quindi ricordare di proteggere il supporto di destinazione. Non esporre questi dati a MCP senza nuovo consenso specifico.
- **Compatibilità:** l'introduzione, *solo se autorizzata*, sarebbe additiva. Archivi attuali senza altre entrate restano identici; nessuna migrazione deve inferire entrate dalle vendite o riconvertire i registri esistenti. Database con schema futuro potrebbero non essere aperti dalle versioni vecchie: richiedere backup verificato prima di migrare.

## 7. Import/export e interoperabilità senza lock-in

- **Esportazione dedicata:** CSV UTF-8 di tutte le altre entrate, con UUID, data ISO, categoria stabile, importi in centesimi, valuta e riferimenti espliciti; distinguere sempre dal CSV `sales` e dal CSV `receipts`. Qualsiasi esportazione selettiva verso terzi deve avere anteprima e campi privati **esclusi di default**; il CSV completo resta disponibile per proprietà dei dati, con avviso di riservatezza.
- **Portabilità completa:** includere tutte le nuove entità, referenze e futuri allegati nel backup a cartella scelta (SQLite + media + manifest verificato), senza compromettere ripristino ed esportazione dei 12 registri CSV attuali. Un CSV senza media non è un backup completo.
- **Import successivo e facoltativo:** anteprima di righe, precisione centesimi, validazione ISO/date/valute, riferimento a categoria e UUID, scelta su controparti mancanti, deduplica per UUID, riconoscimento dei conflitti e rollback integrale all'annullamento. Non trasformare automaticamente vecchi CSV spese/vendite in altre entrate; protezione contro formule da spreadsheet come per gli export già esistenti.
- **Interoperabilità concettuale:** coordinare eventuali formati con TECH-001 (pacchetto aperto) e TECH-025 (esportazione minimizzata), **senza modificare le loro schede** o presumere che siano già implementate.

## 8. Differenza dalle proposte e dai lavori esistenti

- [UX-006](REPS-UX-006-incarichi-opere-su-richiesta.md): brief e avanzamento di **opere commissionate**; la nuova idea tratta **compensi economici per attività anche senza opera** e non un gestore di incarichi.
- [UX-010](REPS-UX-010-diritti-riproduzione-crediti-autorizzazioni.md): documenta **permessi e diritti dichiarati**, non crea automaticamente compensi, incassi o contratti; eventuale collegamento solo manuale.
- [UX-023](REPS-UX-023-prezzi-richiesti-listini-storico.md): prezzi richiesti di opere, non entrate da servizi.
- [UX-024](REPS-UX-024-scadenziario-incassi-piani-pagamento-vendite.md) e [UX-030](REPS-UX-030-quadratura-facoltativa-estratti-conto-locali.md): scadenze/incassi e confronti su movimenti **già registrati**, non fonti autonome di compenso.
- [UX-027](REPS-UX-027-valute-estere-vendite-incassi-costi.md): valuta dei registri economici, non un nuovo tipo di corrispettivo.
- [UX-032](REPS-UX-032-budget-mensili-categorie-spesa-scostamenti.md): obiettivi di **spesa**, non compensi registrati.
- [TECH-001](https://github.com/Cesare-ml/Repertica-Studio/blob/docs/idee-strumenti-architettura-20261121/docs/idee/strumenti-e-architettura/REPS-TECH-001-pacchetto-dati-aperto.md), [TECH-018](https://github.com/Cesare-ml/Repertica-Studio/blob/docs/idee-strumenti-architettura-20261121/docs/idee/strumenti-e-architettura/REPS-TECH-018-esportazioni-csv-scalabili-atomiche.md), [TECH-025](https://github.com/Cesare-ml/Repertica-Studio/blob/docs/idee-strumenti-architettura-20261121/docs/idee/strumenti-e-architettura/REPS-TECH-025-esportazioni-tabellari-minimizzate-condivisione.md): strumenti trasversali per portabilità, robustezza e privacy, non un registro professionale extra-vendita.
- [Issue #13](https://github.com/Cesare-ml/Repertica-Studio/issues/13): da ricontrollare durante una futura valutazione per evitare doppioni sui **pagamenti**. Nessuna modifica o issue operativa aperta.

## 9. Rischi, dipendenze, complessità e alternative

- **Rischio principale:** doppio conteggio o finto «fatturato complessivo», confusione tra importo pattuito, compenso documentato, pagamento e utile. **Mitigazione:** registri separati, etichette esplicite, esempi e test di regressione dei report.
- **Altri rischi:** dati economici sensibili nei CSV/backup, classificazioni fiscali errate, duplicazione di compensi legati a diritti, arrotondamenti, mesi sbagliati, archivio in vecchio formato dopo upgrade, cancellazione di prove storiche. Nessuna consulenza fiscale, emissione di fatture o riconciliazione bancaria implicita.
- **Dipendenze tecniche future, solo da valutare:** migrazione SQLite sicura, controlli dei tipi e calcoli in centesimi, UX economia e i18n, pipeline export e manifest backup; allegati e soggetti professionali sono **facoltativi** e dipendono da future decisioni. Nessuna integrazione cloud o nuova dipendenza di rete.
- **Complessità stimata:** medio-bassa per registro manuale e CSV; **media-alta** per unire correttamente report e incassi/allocazioni. Non sviluppare la fase avanzata senza definire prima regole contabili e casi reali.
- **Alternativa minima:** nessuna nuova entità; una semplice nota locale «entrate extra non conteggiate» con report manuale esterno. Riduce superficie dati, ma non risolve la vista gestionale dei compensi.
- **Alternativa intermedia preferibile se il bisogno è confermato:** registro separato con soli importo, data, categoria, descrizione e CSV, senza pagamenti, rate, fondi o link automatici. È più affidabile di un secondo modulo contabile generalista.
- **Alternativa da evitare senza evidenza:** fatturazione fiscale, scadenze tributarie, collegamenti bancari, generazione di note di credito e motore automatico che classifichi entrate da ricevute.

## 10. Criteri di accettazione ipotetici

1. Una vendita opera da 1.000,00 €, un compenso workshop da 250,00 € e una spesa di 100,00 € restano tre fatti distinti; l'attuale report **vendite** resta a 1.000,00 € e il nuovo registro extra-entrate mostra 250,00 €, senza duplicazioni.
2. Un compenso registrato senza prova di pagamento **non** incrementa «incassi confermati», disponibilità di cassa o `receipts` di una vendita.
3. Creare, modificare e rettificare una voce non cambia UUID, importi, quote congelate o certificati delle vendite pregresse.
4. Percentuali personalizzate impostate per vendite nuove non sono attribuite implicitamente alle altre entrate; un'eventuale scelta futura è distinta e prospettica.
5. Record extra-entrata senza opera o collezionista si possono registrare e ritrovare; una voce descrittiva non crea un cliente duplicato.
6. Date, input negativi/zeri, overflow, valute non supportate e importi non interi producono errori chiari senza scritture parziali.
7. Un archivio precedente, senza questo registro, presenta le stesse metriche economiche e conserva piena compatibilità con i normali backup attuali dopo eventuale migrazione sicura.
8. Esportazione CSV completa e reimportazione di prova (se la seconda fosse approvata) non duplicano ID, non alterano centesimi, date o categorie; la copia completa trasferisce anche eventuali allegati.
9. L'intero flusso funziona offline, su Windows/macOS/Linux, in IT/EN/ES/FR/DE con fallback inglese, tastiera e screen reader; la visualizzazione mensile rimane interpretabile senza grafici né colore.
10. Nessuna funzione crea fattura fiscale, valuta status legale del diritto d'immagine o dichiara automaticamente ricevuto un pagamento.

## 11. Fasi eventuali, priorità e domande al proprietario

- **Fase A — validazione senza sviluppo:** intervistare 3–5 artisti/studi con fonti di entrata differenti; rilevare frequenza dei compensi extra-vendita, categorie davvero necessarie, registri esterni usati e costo del doppio inserimento.
- **Fase B — MVP soltanto dopo approvazione espressa:** registro locale manuale EUR con pochi campi, filtri e CSV dedicato, separato dal totale vendite. Nessuna gestione di pagamenti o fondi.
- **Fase C — opzionale:** nuovo riepilogo per fonti di ricavo senza cambiare KPI preesistenti; eventuali collegamenti manuali a opere/diritti/contatti.
- **Fase D — eventuale e subordinata a decisioni contabili:** registrazioni di pagamenti dichiarati e ripartizioni specifiche, integrate con regole condivise e senza doppioni con incassi delle vendite.

**Certo:** Repertica è gratuito, open source, desktop Tauri, offline-first; archivio e media locali con backup a cartella scelta; registri delle vendite/spese e 12 esportazioni CSV, lingue di sistema con fallback inglese; non è un programma fiscale. **Ipotesi da verificare:** esistenza di compensi extra-vendita frequenti presso la platea, utilità di un registro interno, categorie adatte, necessità di pagamenti collegati. **Domande per il proprietario:** (1) Repertica vuole gestire solo il mercato delle opere o anche l'economia complessiva dello studio? (2) Quali entrate professionali ricorrono realmente? (3) Serve un semplice registro di corrispettivi o anche distinzione fra accordi, prestazioni e denaro ricevuto? (4) Come devono essere trattati ricavi non-opera rispetto alle percentuali di guadagno? (5) Il registro deve restare solo EUR finché la gestione multivaluta non sia stata definita?

**Decisione:** nessuna idea approvata; la priorità indicata non costituisce roadmap, obbligo di implementazione o assegnazione allo sviluppo.
