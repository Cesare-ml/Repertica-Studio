# REPS-UX-077 — Ritrovare acquisti per fornitore e distinguere il perimetro delle spese

- **ID stabile:** REPS-UX-077
- **Data:** 2026-11-23
- **Stato:** Proposta — da valutare
- **Filone:** Gestione e UX
- **Priorità ipotetica:** P2 se la consultazione ripetitiva o l'ambiguità di perimetro sono confermate; P3/sola etichetta/nessuna funzione altrimenti.
- **Natura:** analisi documentale; nessuna approvazione, assegnazione o autorizzazione allo sviluppo.

## Problema verificato e destinatari

Ricognizione statica di main (base osservata dalla PR #14: b64f6bc7b5e4672d926b89834ca7f98fc374e35a). In [finance.rs, righe 147–164 e 343–347](../../../src-tauri/src/finance.rs), le spese hanno già UUID, data dichiarata, descrizione, categoria, fornitore testuale facoltativo, fondo, importo in centesimi, URL acquisto e note. `list_expenses` restituisce tutte le righe ordinate per data e creazione decrescenti. Il riepilogo economico invece limita le spese all'anno richiesto e alla eventuale `treasury_start_date`. In [app.component.ts](../../../src/app/app.component.ts), righe 463–470 e 629–636, il cambio anno aggiorna i riepiloghi, non la lista delle spese; [app.component.html](../../../src/app/app.component.html), righe 222–294, mostra il selettore anno sopra i riepiloghi e la tabella integrale «Acquisti e spese» sotto, senza filtro o indicazione esplicita di perimetro nella tabella. [ECONOMY-I18N](../../ECONOMY-I18N.md) conferma che la data iniziale non elimina dati passati; [DATA-PORTABILITY](../../DATA-PORTABILITY.md) conferma che i CSV sono integrali.

**Ipotesi da validare:** l'artista che deve riacquistare una tela, confrontare il costo storico di una cornice o trovare un fornitore può scorrere molte righe o esportare un CSV. La vicinanza tra selettore anno e lista potrebbe indurre a credere che le righe appartengano tutte all'anno visualizzato; non è stato osservato alcun errore reale e non è stata eseguita l'app. Utenti: artisti, assistenti di studio, piccoli laboratori/gallerie con acquisti ricorrenti. Per pochi acquisti può bastare l'elenco.

## Confini e confronto con idee precedenti

Confrontate le 76 schede Gestione e UX e le 72 tecniche, nonché le issue aperte. La proposta non estende la contabilità (anche la issue aperta «Finance 2: receipts, purchase documents and cashflow separation» resta fuori dal perimetro).
- [UX-069](REPS-UX-069-ripetizione-manuale-spesa-precedente.md) riusa dati in una **nuova registrazione**; qui si ritrovano registrazioni esistenti senza crearne altre.
- [UX-072](REPS-UX-072-categorie-spesa-personalizzabili-storico.md) modifica la gestione delle categorie; qui si usano solo codici già presenti. [UX-014](REPS-UX-014-rubrica-relazioni-professionali.md) studia una rubrica; il fornitore rimane testo, non un soggetto identificato.
- [UX-050](REPS-UX-050-rendiconto-mensile-snapshot-verifiche.md) prevede tra l'altro il rinvio alle registrazioni incluse: questa piccola vista indipendente potrebbe essere riusata lì, senza duplicare rendiconti, verifiche o snapshot. [UX-073](REPS-UX-073-confronto-anni-periodi-economici-omogenei.md) confronta statistiche aggregate: qui si consultano singole voci.
- TECH-008 (PR [#16](https://github.com/Cesare-ml/Repertica-Studio/pull/16)) propone filtri strutturati e viste salvate con scope opere; riusare eventualmente la normalizzazione testuale, non introdurre motore generico o viste persistenti. TECH-025 studia export minimizzati: non aggiungere un nuovo esportatore in questa fase.
- Nessuna scorta (UX-008), nuova spesa, riclassificazione, riconciliazione, pagamento, budget, variazione delle percentuali o nuova anagrafica.

## Scenari e flussi UX

1. **Cercare un acquisto precedente.** In Economia → Acquisti e spese, una ricerca locale «tela lino» confronta descrizione e fornitore; i risultati mostrano data, categoria, descrizione e importo già presenti. Un artista può aprire la registrazione esistente per leggere i dettagli tramite il flusso corrente; il filtro non salva nulla. Nessun URL aperto automaticamente.
2. **Ritrovare un fornitore storico.** «Cornici Rossi» trova le righe corrispondenti; «Rossi srl» non implica che si tratti dello stesso soggetto. Selezione eventualmente per valore testuale esatto, non deduplicazione o consolidamento. Le grafie originali rimangono intatte.
3. **Capire l'ambito.** La tabella reca «Tutte le spese registrate (tutti gli anni)» come stato iniziale; il selettore del riepilogo resta indipendente. Un comando esplicito «Spese incluse nel riepilogo 2026» applica anno e data iniziale della tesoreria, mostrando entrambe le condizioni e lasciando ripristinare l'elenco integrale. Prima di proporre questo comando, provare se basta l'etichetta.
4. **Esempio sintetico:** spese il 20/12/2025 di 80 €, il 10/01/2026 di 50 €, il 12/03/2026 di 100 €. Con anno 2026 e data iniziale tesoreria 01/02/2026, la lista integrale mostra tre voci; il perimetro del riepilogo ne include una (100 €). Le altre non sono cancellate e non diventano «non pagate». L'eventuale subtotale della selezione testuale non va etichettato come spese annuali.
5. **Dopo modifica:** se una voce non corrisponde più alla ricerca, aggiornare risultati e contatore, annunciare in modo discreto che la voce non corrisponde più, mantenere filtri e offrire «Azzera». Nessuna azione massiva. Le conferme di eliminazione attuali restano separate.

## Specifica minima, dati e impostazioni

- Fase minima: etichetta di ambito permanente; ricerca testuale facoltativa, conta delle righe visibili e totali, comando azzera. Filtri temporanei, non salvati. Ricerca su descrizione/fornitore, confronto senza distinzione maiuscole e accenti (normalizzazione coerente con [archive-filters.ts](../../../src/app/archive-filters.ts), ma non mutare le stringhe). Nessuna ricerca in note private o URL nella prima fase.
- Eventuale seconda fase: categoria per codice esistente, fondo per UUID o nessun fondo, e scelta esplicita di periodo «Tutte / Nel riepilogo [anno]». Categorie/fondi inattivi, non noti o non assegnati rimangono consultabili, senza riclassificazione. Per dati legacy non validi, non presumere date né includere in un subtotale senza segnalazione.
- Non aggiungere tabelle o campi: fonte `Expense`, impostazioni `treasury_start_date`, anno UI e piano fondi attuale. Stato effimero della vista: query, eventuale scope, categoria/fondo; nessuna modifica a `list_expenses` o ai calcoli Rust. Interfaccia puramente concettuale: filtro locale → righe e conteggio; l'eventuale perimetro del riepilogo deve condividere la semantica effettiva del backend e rifiutare equivalenze non verificate. Subtotali solo se utili, calcolati esattamente sui centesimi delle righe visibili, con etichetta «Totale delle righe mostrate», non saldo bancario. Nessun nuovo dato permanente.
- Accessibilità: etichette persistenti, tastiera, focus stabile, messaggio distinto «nessuna spesa» / «nessuna corrispondenza», conteggio accessibile senza annunci a ogni tasto, nessun affidamento al solo colore. Data e importi con formattazione locale; ricerca non dipendente dalla lingua delle etichette. Lingua di sistema, fallback inglese, override esistente; traduzioni almeno IT/EN e verifica ES/FR/DE come nel prodotto. I nomi dei fornitori non si traducono.

## Offline, privacy, import/export, compatibilità e backup

Interamente locale in Angular/Tauri e SQLite; niente account, abbonamento, telemetria o ricerca web. Query, fornitori e risultati non sono registrati o trasmessi, né esposti a MCP. Gli URL restano dati: mai fetch, preview remota o apertura implicita. Nessun nuovo permesso. Non alterare il comportamento dei comandi CRUD esistenti.

I dodici CSV correnti rimangono integrali, indipendenti da anno e filtri; ribadirlo vicino al registro e nella sezione Esportazioni. Per esigenze di confronto esterno usare il CSV spese con Calc locale; non produrre automaticamente una copia con dati del fornitore o URL. Un eventuale CSV filtrato esplicito è una decisione distinta coordinata con TECH-025. Nessun import nuovo; i CSV non sono backup. Il backup completo su cartella scelta e il ripristino rimangono invariati: la vista temporanea non richiede migrazione né backup delle preferenze e non modifica SQLite o media. Nessun problema di compatibilità dello schema atteso; verificare comunque su copie sintetiche e tre sistemi desktop.

## Alternative, complessità, dipendenze e rischi

- **Prima alternativa:** aggiungere solo una chiara indicazione che il registro contiene tutte le annualità (oppure chiarire la guida); CSV/Calc per le ricerche occasionali. Se il bisogno non emerge, nessun intervento.
- Ricerca locale minima: complessità bassa; periodo con equivalenza semantica verificata, localizzazione e test di accessibilità: bassa-media. Dipendenze solo da dati/lista già presenti e componenti UI. Nessuna libreria nuova necessaria. Per liste grandi misurare prima eventuali ricalcoli (TECH-070 concerne prestazioni della tabella opere; riusare principi senza duplicarne l'architettura).
- Rischi: confusione tra totale filtrato e annuale, perdita apparente delle spese pre-tesoreria, omonimie fra fornitori, ricerca accent-insensitive troppo ampia, importo fuori limite, reset inatteso dopo modifica, CSV integrale scambiato per filtrato. Mitigare con etichette, comportamento di default invariato, nessuna aggregazione per identità e prove su dati sintetici.

## Criteri di accettazione proposti (non eseguiti)

1. Senza filtri si vedono tutte le righe, incluse quelle di anni precedenti e pre-tesoreria, con perimetro leggibile. Cambiare l'anno non nasconde spese in questa modalità.
2. Una ricerca per fornitore/descrizione trova accenti e maiuscole equivalenti, non modifica valori e gestisce fornitore vuoto, grafie diverse e zero risultati. Il reset restituisce tutte le righe.
3. Se prevista la modalità riepilogo, l'esempio sintetico sopra coincide con il backend, inclusi data esattamente uguale alla soglia, anno passato e futuro, nessuna soglia, anno bisestile e zero righe. La soglia non si applica di nascosto alla lista integrale.
4. Importi in centesimi coerenti, eventuale subtotale non confuso con margine/quote/incassi. Modifica da un flusso già autorizzato aggiorna la vista; nessun'altra scrittura deriva dalla ricerca. L'esportazione integrale mantiene righe e schema; backup/restore invariati.
5. Sola tastiera, screen reader, finestra stretta, etichette IT/EN, fallback inglese per locale non supportato, date/importi e nomi Unicode. Nessuna rete necessaria, nessun logging query né accesso MCP aggiunto.
6. Verificare manualmente su Windows/macOS/Linux; misurare ricerca e scroll su 100, 1.000 e 10.000 spese sintetiche prima di introdurre infrastruttura nuova. Nessuna promessa prestazionale senza misure.

## Fasi e decisioni del proprietario

- Fase 0: osservare 3–5 professionisti che cercano un acquisto precedente o confrontano tabella e riepilogo; rilevare frequenza, tempo e fraintendimenti, anche con CSV/Calc.
- Fase 1: se utile, sola etichetta di perimetro e ricerca temporanea. Fase 2: solo dopo ulteriore evidenza, filtro periodo/categoria/fondo con verifica backend e collegamento eventuale a UX-050; nessun nuovo motore di report.
- **Requisiti certi:** offline-first, dati locali ed esportabili, backup libero, nessun account obbligatorio, rispetto di registri integrali e calcoli correnti; la differenza di ambito è riscontrata staticamente.
- **Ipotesi:** ricerca frequente per fornitore, ambiguità effettiva del selettore anno, valore di un subtotale; non verificate.
- **Domande riservate al proprietario:** basta un'etichetta? Quali campi sono davvero cercati? Il filtro «nel riepilogo» è più utile di un semplice anno di spesa? Serve un filtro fondo/categoria o sovraccarica l'interfaccia? Le ricerche sono così frequenti da giustificare una UI anziché CSV? La proposta rimane da valutare, senza assegnazioni.
