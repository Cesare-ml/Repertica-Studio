# REPS-UX-003 — Scheda economica per opera: costi attribuiti e margine stimato

- **Stato:** Proposta — da valutare
- **Data:** 2026-11-21
- **Filone:** Gestione e UX
- **Priorità proposta:** P2 (soggetta a revisione)
- **Perimetro:** esclusivamente documentale; non autorizza implementazioni

## Contesto e differenza dall'esistente

Il repository registra già [spese per categoria e fondo, ripartizione percentuale delle vendite, prezzo suggerito per dimensione e riepiloghi mensili](../../ECONOMY-I18N.md). La vendita e il costo generale sono eventi distinti; non esiste una relazione esplicita tra **costo di una specifica opera** e spese già registrate. Non si propone una nuova tesoreria, un altro coefficiente artista o un secondo registro spese. La [issue #13](https://github.com/Cesare-ml/Repertica-Studio/issues/13) copre incassi/documenti e miglioramenti della contabilità generale: questa scheda riguarda esclusivamente la **contabilità gestionale per opera**, senza duplicare i flussi monetari.

## Problema e utenti

L'artista deve capire se il prezzo suggerito e quello di vendita coprono tela, colori, cornice, stampa, packaging e lavoro. Le spese aggregate mostrano andamento dello studio, non costo di produzione di ciascuna opera. Reinserire una spesa nella scheda opera rischierebbe di sommarla due volte ai totali.

## Scenari

1. L'artista acquista materiali per 90 EUR già registrati come spesa attività e attribuisce una parte verificabile (per esempio 30 EUR) a una delle tre opere realizzate: il totale speso dallo studio **non aumenta**.
2. Una cornice acquistata esclusivamente per un dipinto è collegata a quel dipinto e riappare nel suo costo di produzione.
3. Prima della vendita si analizzano prezzo suggerito, costi attribuiti e margine **ipotetico**; dopo la vendita si mostra margine **gestionale** sul prezzo registrato, separandolo dalle quote personale/attività/risparmio.

## Flusso e modello UX

- Scheda **Opera → Costi e redditività**: costo documentato da spese, costi stimati non contabilizzati, ore di lavoro stimate e valore figurativo del lavoro **se scelto**; prezzo suggerito già disponibile, eventuale ricavo da vendita, margine gestionale e differenza tra reale e simulato.
- CTA **Attribuisci una spesa esistente**: trova voce del registro spese, propone importo disponibile da ripartire, conferma quota in centesimi. Mostra altre opere che condividono la stessa spesa e saldo attribuibile residuo.
- **Stima nuovi costi**: voce temporanea non registrata come uscita monetaria; un esplicito pulsante *Registra come spesa* può essere valutato in futuro, ma non deve creare uscite implicitamente.
- **Simula prezzo**: campo prezzo ipotetico editabile in EUR, confronto con suggerimento del coefficiente artista, margine e percentuale margine. L'anteprima non scrive su vendite o anagrafiche.
- **Vista trasversale** opzionale: opere con costi non coperti, opere senza dati costo, confronto ricavi e costi; senza classifiche normative o suggerimenti IA.
- Indicatori e formule accompagnati da tooltip comprensibili; codici visivi affiancati da etichette e supporto tastiera.

## Regole dati e formule (ipotesi da validare)

- Nuova entità di **attribuzione di costo**: UUID, `artwork_id`, `expense_id` opzionale, `amount_cents` intero, tipo `actual_linked` oppure `estimate`, descrizione, data, note, timestamp. Per costi reali già registrati riferirsi obbligatoriamente alla spesa originale; in caso di spesa esclusivamente per un'opera, attribuirla al 100%.
- Validazione `0 ≤ somma delle attribuzioni effettive alla stessa spesa ≤ expense.amount_cents`; impedire sovra-attribuzioni. Correzioni e riassegnazioni tracciabili. I costi stimati non si trasformano automaticamente in spese.
- Eventuale lavoro: ore/minuti × valore orario scelto, espresso in **centesimi**; esporre come **costo figurativo**, non spesa pagata né profitto fiscale. Valore orario modificabile senza riscrivere silenziosamente stime storiche: salvare snapshot dei parametri nella stima.
- **Margine gestionale opera (semplificato)** = ricavo di vendita registrato al netto di tasse/spedizione **se già registrate nella vendita** − costi attribuiti documentati. Mostrare separatamente eventuali costi stimati e figurativi. Non sottrarre due volte shipping e tasse se già nel netto.
- **Margine potenziale** = prezzo ipotetico − costi documentati; etichetta **simulazione**, non previsione garantita. Niente attribuzione automatica di costi generali (affitto, elettricità) salvo futura scelta esplicita di criterio.
- In assenza di prezzo/costo, mostrare “dato non disponibile” e non uno zero ingannevole. Il margine può essere negativo; rapporto percentuale non definito con ricavo zero.
- Le quote di destinazione personale/attività/risparmio rimangono quelle della vendita ([snapshot delle percentuali](../../ECONOMY-I18N.md)); non sono sinonimi di margine dell'opera.

## Impostazioni, offline, multilingua, import/export

- **Impostazioni:** costo orario suggerito opzionale, categorie costo suggerite, scelta della visualizzazione *solo documentato / includi stimato e figurativo*; la prima modalità è predefinita per evitare numeri poco fondati.
- **Offline e privacy:** tutti i dati nella SQLite locale e nel backup già esistente; nessun tracciamento ordini fornitori, conti bancari o cloud. I costi di produzione e i margini sono riservati e non compaiono per default in schede pubbliche o certificati.
- **Lingue:** etichette/formule/spiegazioni localizzate IT/EN/ES/FR/DE, lingua OS e fallback inglese; importi visualizzati secondo locale senza alterare il calcolo in centesimi EUR.
- **Esportazione:** CSV per analisi personale con colonne distinte per costi documentati, stimati e figurativi; JSON versionato delle attribuzioni con riferimenti a `artwork_id`/`expense_id`; import assistito che rispetta vincoli ed evita doppie attribuzioni. Non esportare per errore questi numeri nei certificati.
- **Compatibilità:** migrazione additiva SQLite; archivi preesistenti non ricevono costi dedotti inventati e continuano a funzionare anche senza attribuzioni. Nessuna modifica retroattiva alle spese.

## Complessità, rischi e alternative

- **Complessità:** media per attribuzioni, simulatore e aggregazioni; alta se si introduce magazzino materiali con scarico quantità automatico. **Il magazzino automatico è fuori ambito**.
- **Dipendenze:** `expenses`, `sales`, `sale_finances`, prezzo suggerito, importi interi, migrazione, i18n, backup.
- **Rischio principale:** doppio conteggio in tesoreria; **regola:** l'attribuzione modifica esclusivamente l'analisi per opera, non crea una seconda spesa. Vietato presentare i margini come risultato fiscale.
- **Altri rischi:** cifre stimate spacciate per costi effettivi, modifica di una spesa già ripartita, cancellazione di riferimenti, contabilità multi-valuta non supportata, sovra-allocazione degli sconti.
- **Mitigazioni:** legenda permanenti, riepilogo “attribuito / da attribuire”, validazioni bloccanti, conferma se modifica spesa con allocazioni, tracciabilità degli aggiustamenti.
- **Alternativa leggera:** semplici note di costo per opera, senza sommare alla tesoreria; meno utile per report e controlli, ma più rapida da progettare.

## Criteri di accettazione ipotetici

1. Ripartire una spesa di 90 EUR fra tre opere non fa crescere di un centesimo le spese mensili del registro finanziario.
2. Tentare attribuzioni totali per oltre 90 EUR viene respinto con errore localizzato; una spesa già attribuita non è cancellabile silenziosamente.
3. Prezzo suggerito, prezzo ipotetico, prezzo di vendita, costo documentato e costo figurativo sono sempre distinguibili.
4. Simulare un prezzo non modifica prezzo dell'opera o dati vendita; in assenza di vendita non appare “margine realizzato”.
5. Modificare il coefficiente artista non riscrive vendite pregresse né stime storicizzate.
6. Export/import di prova conserva i collegamenti UUID e non duplica spese/attribuzioni.
7. Funziona senza rete, in tutte le lingue supportate, con navigazione da tastiera e importi corretti al centesimo.

## Fasi suggerite

- **Fase A:** attribuzioni documentate dalle spese esistenti, dettaglio opera e guardrail anti doppio conteggio.
- **Fase B:** simulatore prezzi e stime separate; valore figurativo del lavoro opzionale.
- **Fase C:** confronto per serie/opere e export avanzato, mantenendo netta la separazione da incassi e contabilità fiscale.

## Requisiti certi, ipotesi e domande per il proprietario

- **Certo:** spese e ripartizioni mensili esistenti, coefficiente/prezzo per cm², valuta delle vendite EUR, architettura solo locale e backup; nessun sistema di incassi certificati.
- **Ipotesi:** utilità pratica dell'attribuzione analitica di materiali e ore di lavoro; disponibilità di dati sufficienti a inserirle.
- **Da decidere:** includere il tempo di lavoro? Quali spese vanno attribuite solo a un'opera e quali possono essere ripartite? Si desidera un costo per lotto/serie in futuro? Separare sempre margine operativo da quota personale/attività/risparmio?

**Decisione attuale:** nessuna approvazione o assegnazione allo sviluppo.
