# REPS-UX-008 — Scorte dei materiali e consumi di laboratorio

- **ID stabile:** REPS-UX-008
- **Data:** 2026-11-21
- **Stato:** Proposta — da valutare
- **Filone:** Gestione e UX
- **Priorità proposta:** P2 (da verificare con artisti che gestiscono materiali ricorrenti)
- **Perimetro:** sola analisi documentale; nessuna funzione approvata o assegnata

## Evidenza nel repository e confini

In [main](../../ECONOMY-I18N.md) sono già presenti le **spese** (categoria, fornitore, importo, fondo e data), nonché acquisti di materiali descritti come **uscite monetarie**; [README](../../../README.md) e [modelli](../../../src/app/models.ts) non presentano un registro di quantità disponibili, prelievi o soglie di riordino. La [issue #13](https://github.com/Cesare-ml/Repertica-Studio/issues/13) tratta acquisti ricorrenti, ricevute e flussi di cassa: **non** è un sistema di magazzino. [REPS-UX-003](REPS-UX-003-costo-e-margine-per-opera.md) ipotizza l'attribuzione di importi spesi alle opere; questa scheda parla di **unità fisiche, disponibilità e consumo**. Eventuali valori economici dei materiali devono riconciliarsi senza creare nuove spese o margini fittizi. Il sistema non è già implementato.

## Problema e destinatari

Artisti, laboratori di stampa e piccoli studi comprano tele, pigmenti, cornici, carta e imballaggi in lotti. Una riga «acquisto materiali 80 €» non dice **quante unità** sono rimaste o se una tela è già stata impiegata. La mancanza di tracciabilità provoca ordini doppi, interruzioni nella produzione e stime di consumo inaffidabili. L'obiettivo è un **inventario operativo leggero**, non un ERP, un sistema fiscale o un controllo seriale obbligatorio.

## Casi d'uso significativi

1. L'artista acquista 10 tele formato 40×50, registra **entrata 10 pezzi**, ne usa 3 in laboratorio e visualizza **7 disponibili**, senza registrare altre spese.
2. La galleria prepara 12 imballaggi ma ne trova 10: effettua una **rettifica inventariale motivata**, lasciando traccia della differenza; non forza la creazione di un acquisto o di una vendita.
3. Un laboratorio utilizza 250 ml di medium da un flacone da 1 litro: il prelievo si esprime in una unità coerente e il saldo risulta 750 ml. La conversione viene dichiarata esplicitamente; niente conversione automatica fra peso e volume.
4. Prima di iniziare un'opera, si consultano materiali disponibili e soglie minime. Un prelievo può essere collegato a un'opera, ma **non** genera automaticamente il costo in EUR della [UX-003](REPS-UX-003-costo-e-margine-per-opera.md).

## Flussi UX candidati

- **Studio → Materiali**: elenco compatto con codice/nome, unità, scorta attuale, soglia facoltativa, ubicazione e filtro «da riordinare». Vista consultabile offline anche con centinaia di articoli; distinguere zero, sconosciuto e scorta negativa contestata.
- **Nuovo materiale**: nome e unità obbligatori; eventuali categoria, marca, formato, fornitore abituale e posizione fisica. Preimpostazioni modificabili; niente obbligo di fornitori anagrafici.
- **Registra entrata**: quantità, data e fonte opzionale (spesa già registrata, inventario iniziale, restituzione). Visualizzare «registrazione quantità, non nuova spesa». Collegare una spesa esistente **non** cambia i totali mensili.
- **Preleva / Consuma**: quantità positiva, uso generico o opera opzionale, data e nota. Prima di confermare mostrare saldo previsto e avviso quando la richiesta supera il disponibile. La gestione di eventuali eccezioni di saldo negativo va decisa dal proprietario.
- **Conta e rettifica**: confronta saldo teorico e conteggio fisico, richiede motivo e mostra delta; corregge tramite movimento datato, senza cancellare movimenti precedenti.
- **Articolo → Cronologia**: entrate, consumi, resi e rettifiche filtrabili per periodo e opera, con collegamento alla spesa solo se disponibile. **Lista di riordino** stampabile o esportabile manualmente, senza invio ordini automatico, acquisti e notifiche in background.
- **Accessibilità**: operazioni complete da tastiera, etichette di unità ripetute vicino al campo, errori annunciati, nessun significato affidato al solo rosso/verde, ordinamento leggibile e testi localizzati.

## Dati minimi ipotetici (non schema o API autorizzati)

- `material_item`: UUID stabile, codice locale opzionale, nome, categoria, unità canonica (`piece`, `ml`, `g`, `m` ecc.), precisione ammessa, posizione descrittiva, soglia minima facoltativa, archiviato sì/no, date di creazione/modifica. Nomi e descrizioni sono liberi.
- `material_movement`: UUID, item UUID, tipo (`opening`, `purchase_receipt`, `use`, `return`, `adjustment`), quantità **decimale a precisione dichiarata** o intero di unità minime, data, motivo, artwork UUID opzionale, expense UUID opzionale, timestamp e rettifica del movimento se necessaria.
- Giacenza = somma algebrica dei movimenti **nella stessa unità canonica**, non valore di una cella modificata direttamente. Una rettifica è un nuovo evento. Per `piece` evitare frazioni; per quantità frazionabili definire scale fisse (es. ml interi), mai float binari per calcoli inventariali.
- Unità di confezione e unità canonica sono distinte: «1 barattolo da 500 ml» registra **500 ml** solo quando formato e conversione sono stati inseriti. Nessuna equivalenza automatica fra litri e kg, dimensioni di carta o confezioni eterogenee.
- Collegamento a `expenses` facoltativo e soltanto referenziale: una spesa può alimentare **più** entrate documentate; non usare l'importo dell'intera fattura come costo di ogni singolo prelievo. Se in futuro si desidera valutazione economica per lotti, definire un criterio esplicito, senza confondere disponibilità e profitti.
- I riferimenti `artwork_id` servono per tracciabilità di consumo, non per decrementare la scorta automaticamente quando cambia lo stato di un'opera. Le vendite e i certificati restano invariati.
- Prevedere rettifiche con motivazione obbligatoria e recupero di cancellazioni erronee; l'eliminazione di un articolo con movimenti richiede archiviazione, non distruzione della storia.

## Impostazioni, offline, lingue e interoperabilità

- **Impostazioni:** unità e categorie suggerite, possibilità di disattivare soglie e collegamenti alle opere, scelta facoltativa se impedire saldi negativi; nessuna configurazione o rete necessaria per iniziare.
- **Offline/privacy:** movimenti e nominativi dei fornitori rimangono nella SQLite locale; nessuna telemetria, scansione di cataloghi remoti, cloud o acquisto automatico. Stesse cartelle di backup liberamente scelte già previste. Non registrare identificativi di operatori inesistenti: l'MVP non ha account.
- **Multilingua:** UI IT/EN/ES/FR/DE con lingua sistema e fallback inglese; etichette/unità con codici stabili e plurali/localizzazione corretti. Nessuna traduzione automatica delle note libere; numeri nel locale UI ma quantità serializzate in forma non ambigua.
- **Export:** CSV UTF-8 distinto per catalogo articoli e movimenti con ID, unità, quantità normalizzate, data, riferimenti e motivi; export della lista riordino senza prezzi o dati privati di default. JSON versionato per un futuro round trip con riferimenti UUID, in linea con [TECH-001](https://github.com/Cesare-ml/Repertica-Studio/pull/16); **non** è un formato già realizzato. Import soltanto dopo anteprima, controllo unità/precisione/duplicati, transazione con rollback e riconciliazione delle spese, senza generare uscite.
- **Backup/ripristino:** includere tutti gli articoli/movimenti nell'archivio locale se un giorno implementati; la verifica del restore deve ricostruire gli stessi saldi. Nessuna modifica proposta al sistema tecnico di backup o ai file del filone TECH.

## Dipendenze, complessità, rischi e alternative

- **Dipendenze:** UUID locali, registro spese, eventuale riferimento a opere, migrazione SQLite, backup integrale e i18n. UX-003 è **facoltativa**, utile soltanto per collegare quantità e costi in una fase separata.
- **Complessità:** media per articoli/movimenti/filtri; medio-alta per unità convertibili, import idempotente, saldi controversi e valorizzazione di lotti.
- **Rischi:** doppio conteggio contabile, uso di unità incompatibili, falsa precisione di scorte non contate, cancellazioni che riscrivono il passato, scorte negative incomprensibili, performance su storici numerosi.
- **Alternativa minima:** articoli con sola unità «pezzo», entrate/prelievi/rettifiche, soglia facoltativa; niente prezzi, lotti, scanner, ordini o integrazioni. Da confrontare con l'uso di semplici fogli di calcolo locali.
- **Fuori perimetro:** gestione IVA, fornitori ERP, barcode obbligatori, ordini automatici, lotti regolamentati, stock delle **opere** (già oggetti distinti), previsioni IA.

## Criteri di accettazione ipotetici

1. Entrata 10 pezzi, prelievo 3 e rettifica motivata −1 producono **6 pezzi**, con tre movimenti leggibili e senza alcun mutamento al registro spese.
2. Richiedere 0, quantità negativa non prevista o 0,5 «pezzi» genera errore comprensibile, non un movimento fantasma; 250 ml è valido se l'articolo è misurato in ml.
3. Collegare un movimento a una spesa non cambia il suo importo, il fondo, le statistiche mensili né il margine dell'opera; la storia resta consultabile dopo la chiusura dell'opera.
4. Importare due volte gli stessi ID non duplica giacenze; unità diverse non vengono sommate senza conversione esplicita.
5. Un conteggio fisico diverso crea una rettifica con data e motivo e permette di ricostruire il saldo alla data precedente.
6. Backup/ripristino in cartella scelta riproducono movimenti, riferimenti e saldi; CSV/JSON non contengono contatti privati non scelti.
7. Tutti i percorsi principali funzionano senza rete su Windows/macOS/Linux, accessibili da tastiera e localizzati con fallback inglese.

## Fasi di eventuale studio, non roadmap approvata

- **A:** validazione dell'effettiva frequenza di consumo; prova con articoli, unità canonica, entrate e prelievi.
- **B:** rettifiche, scorte minime, filtri e lista da riordinare; test su grandi cronologie e recovery.
- **C:** collegamenti controllati a spese/opere, import strutturato, valutazione **separata** dell'utilità di lotti e valorizzazioni.

## Certezze, ipotesi, domande per il proprietario

- **Certezze dal codice e dalle docs:** esistono acquisti/spese, opere, coefficienti di prezzo, fondi, statistiche per mese, SQLite e backup locali; non è presente un modello di giacenze; le vendite non equivalgono a incassi.
- **Ipotesi:** una parte significativa degli artisti usa scorte ripetitive e preferirebbe il controllo nell'app rispetto a un foglio esterno; i volumi/pezzi bastano senza lotti.
- **Da decidere:** chi è il destinatario principale (artista singolo, studio, galleria)? Servono solo pezzi o anche ml/g/m? Consentire saldo negativo con avviso? L'utente vuole collegare materiali e costi delle opere già dalla prima fase? È utile un inventario separato per più sedi?

**Nessuna approvazione, assegnazione, implementazione o merge impliciti.**
