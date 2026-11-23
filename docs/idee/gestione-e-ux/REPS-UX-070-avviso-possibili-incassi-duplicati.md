# REPS-UX-070 — Avviso prudente prima di registrare un possibile incasso duplicato

- **ID stabile:** REPS-UX-070
- **Data:** 2026-11-23
- **Stato:** **Proposta — da valutare**
- **Filone:** Gestione e UX; analisi e documentazione esclusivamente
- **Priorità proposta:** **P2 solo se gli errori sono ricorrenti**, altrimenti P3 o nessuna nuova funzione
- **Decisione:** non approvata, non assegnata, non autorizzata all'implementazione

## 1. Problema professionale, utenti e riscontro verificabile

Artisti, piccoli studi e gallerie registrano gli incassi di una vendita manualmente, talvolta in più rate. Quando devono ricostruire pagamenti da ricevute, contanti, estratti conto o messaggi, possono **inserire due volte lo stesso incasso**. Se la vendita resta parzialmente insoluta, entrambi gli inserimenti possono essere formalmente validi: la somma non supera il lordo, ma **il saldo residuo e il report degli incassi dichiarati risultano inaccurati**. L'errore può in seguito confondere il rapporto con il collezionista.

**Verifiche sul branch `main` il 23/11/2026, non prova di errori reali:**

- `src-tauri/src/receipts.rs`: `record_core` convalida data, importo, metodo, vendita esistente e **somma non superiore all'importo della vendita**. Ogni salvataggio produce un nuovo UUID. Non emerge dal percorso verificato una verifica di uguaglianza con incassi già registrati della stessa vendita.
- `src/app/app.component.html`: «Gestisci incassi» mostra la tabella degli incassi della vendita (data, importo, metodo e riferimento) **sopra** la maschera «Nuovo incasso». La maschera permette di salvare con «Registra incasso» e mostra il saldo residuo.
- `src/app/app.component.ts`: `saveSaleReceipt()` verifica il limite del residuo e chiama `api.recordSaleReceipt(input)`; dopo il successo aggiorna la bozza in funzione del saldo. Dal flusso verificato non appare un avviso specifico di possibile duplicato.
- `docs/SALE-RECEIPTS.md` e `docs/RECEIPT-ATTACHMENTS.md`: gli incassi sono **dichiarazioni manuali, non verifiche bancarie**. Gli allegati probatori sono facoltativi e separati. `docs/DATA-PORTABILITY.md` descrive CSV di incassi e saldi.

**Problema ipotetico, non malfunzionamento attestato:** due pagamenti diversi possono legittimamente avere stesso importo, data e metodo (ad esempio due contanti o rate distinte). Un avviso deve pertanto essere **prudente, spiegabile, non bloccante e mai chiamato «duplicato certo»**.

## 2. Utenti e scenari

1. **Bonifico probabilmente già registrato.** Per la stessa vendita esiste un incasso di €350 del 4 novembre, con riferimento «TR-84». L'utente prova a registrare di nuovo €350 con il medesimo riferimento. Se il residuo lo consente, viene mostrato il record già presente e una richiesta di verifica; nessuna riga aggiunta senza ulteriore conferma.
2. **Due pagamenti legittimi uguali.** Due rate da €250 sono state pagate in contanti nello stesso giorno. L'operatore consulta i due giustificativi, dichiara che si tratta di versamenti distinti e può proseguire. **Mai** vietare automaticamente l'operazione.
3. **Riferimento riutilizzato o trascritto male.** Importo e metodo differiscono, ma il riferimento non vuoto coincide con quello di un incasso precedente della stessa vendita: mostrare un indizio debole, non un verdetto. Riferimenti mancanti non significano «uguali».
4. **Vendita pagata interamente.** Il sistema già non consente nuovi incassi quando il saldo è zero: non introdurre una seconda regola che pretenda di identificare un trasferimento bancario.
5. **Errore prima del salvataggio.** L'utente interrompe dopo l'avviso per verificare le proprie carte: il saldo e la cronologia restano invariati.
6. **Esito incerto dopo timeout o doppio clic.** Non reinviare alla cieca la stessa scrittura: prima ricaricare la lista e verificare se l'incasso appare già. Questo riguarda soprattutto il comportamento UI, senza promettere idempotenza transazionale non presente.
7. **Documento allegato.** Un PDF/ricevuta associato al record esistente può essere aperto secondo il flusso attuale, ma un documento non dimostra da solo che i pagamenti coincidano; non confrontare automaticamente contenuti, nomi di file o dati bancari.

## 3. Flusso UX ipotetico, minimo e accessibile

**Ingresso:** `Vendite → Gestisci incassi → Nuovo incasso`; nessuna sezione nuova. La tabella degli incassi già presente è la sorgente primaria da rendere più visibile per la verifica.

**Soluzione di fase minima da valutare:** dopo che l'utente preme «Registra incasso», confrontare **solo gli incassi della vendita corrente** con il nuovo input, utilizzando campi già presenti. La proposta non richiede una scansione di tutti i clienti o un algoritmo di identità economica.

- **Indizio forte, ma non prova:** stessa vendita, stesso importo in centesimi, stesso giorno, stesso metodo; oppure stesso **riferimento non vuoto** con stesso importo per la stessa vendita, anche se la data differisce (da motivare in test). Nessuna normalizzazione aggressiva che cancelli zeri, trattini o caratteri identificativi; per il riferimento limitarsi inizialmente a spazi esterni e, dopo prove, eventuale confronto case-insensitive.
- **Indizio debole:** stesso riferimento non vuoto, ma importo o data diversi. Potrebbe essere una causale generica o un versamento parziale: avviso differenziato o **nessun avviso** se troppi falsi positivi nelle prove.
- **Non confrontare:** altri collezionisti/vendite, nomi, note libere, allegati, similarità fotografica, IBAN o estratti conto; un riferimento vuoto non fa scattare corrispondenze solo perché entrambi i campi sono vuoti.
- **Prima decisione:** se non emerge alcun indizio, il normale salvataggio prosegue con le verifiche esistenti. Se emerge, mantenere intatti i campi della bozza e mostrare un pannello chiaro: «Potresti aver già registrato questo incasso» + data, importo, metodo, riferimento dell'operazione o delle poche operazioni confrontabili.
- **Azioni:** «Torna a verificare» (predefinita, nessun salvataggio), «Vedi incasso esistente» (sposta focus sulla riga, senza chiudere arbitrariamente il form) e «Registra comunque: è un incasso distinto» (salva **solo su decisione esplicita**; nessuna casella preselezionata).
- **Risultato:** solo la conferma seguita da scrittura Rust riuscita produce un nuovo UUID; messaggio di conferma dopo lettura dell'esito, aggiornamento del saldo e della lista. Il programma non crea, unisce, cancella, corregge o riassegna incassi autonomamente.

**Comportamento in presenza di dati cambiati:** se tra confronto e conferma il registro è stato modificato (seconda finestra/processo), rileggere ove fattibile il riepilogo prima della richiesta di salvataggio e rispettare i vincoli transazionali di Rust; **un avviso lato UI non è una garanzia di assenza di duplicati**. Non trasformare una coincidenza di campi in un vincolo di unicità SQLite.

**Accessibilità:** spiegazione testuale non solo cromatica; apertura con focus sul titolo, tab order coerente e ritorno al campo originario, `aria-describedby` per il confronto, nessun salvataggio mediante Enter accidentale nel pannello, pulsanti distinti, screen reader e zoom 200%. Mostrare soltanto pochi candidati, con comando accessibile per vedere gli altri e senza scorrimento forzato di un dialogo troppo alto.

## 4. Dati, regole e impostazioni

**Dati certi già esistenti:** `SaleReceipt` contiene `id`, `sale_id`, `received_at` ISO, `amount_cents`, `method`, `reference?`, `notes?`, `created_at`; `SaleBalance` contiene `sale_cents`, `received_cents` e `outstanding_cents`. `record_sale_receipt` conserva già l'invariante `somma incassi <= lordo`. I report mensili sommano incassi per la data dichiarata, non sono saldi di banca.

**Ipotesi di sola presentazione:** confronto deterministico fra record della **stessa vendita** in memoria o nuova lettura locale; nessuna nuova colonna, indice di matching, punteggio opaco, campo «duplicato», log contenente dati bancari o nuova API remota. La severità della corrispondenza è uno strumento di orientamento UX, **non un'attestazione del versamento**.

**Impostazioni:** nessuna preferenza obbligatoria, nessun algoritmo regolabile in fase iniziale. Se i falsi positivi risultassero davvero frequenti, valutare dopo test una preferenza locale per disattivare i soli avvisi deboli, mantenendo comunque leggibile la cronologia; non introdurre opzioni senza dati.

**Continuità dati:** non riscrivere incassi storici, riferimenti, note, allegati, rate, saldi già calcolati, quote congelate, vendite o certificati; mantenere il blocco sull'eccedenza del totale. L'esito manuale «è distinto» non costituisce informazione probatoria e non richiede un campo persistente.

## 5. Offline, lingua, privacy, import/export, compatibilità

- **Offline-first:** solo SQLite locale e la lista già disponibile; niente servizi bancari, telemetria, account, cloud, abbonamento o connessione. Deve funzionare senza rete e anche con provider esterni assenti.
- **Lingue:** lingua del sistema da Tauri, scelta utente nelle impostazioni quando prevista, IT/EN/ES/FR/DE con **fallback inglese**. Testare la traduzione completa di avvisi e pulsanti, pluralizzazione delle corrispondenze, date nella lingua UI; salvare sempre date ISO e importi in **centesimi interi**, senza inferenze dalle stringhe formattate.
- **Privacy:** confrontare solo gli incassi della vendita corrente; non esporre nome o dati del collezionista se non necessari; non inviare riferimenti a servizi esterni né copiarli negli appunti. L'eventuale consultazione del documento mantiene tutte le tutele già previste per gli allegati.
- **Esportazione:** le righe registrate continuano nei CSV integrali `receipts` e `balances` e nei backup completi; gli avvisi transienti non vengono esportati come fatti, non modificano colonne o formati. CSV non equivale a banca; un backup non esegue deduplicazione.
- **Import:** nessun nuovo import previsto; la proposta non duplica la futura quadratura CSV né richiede connettori bancari. Gli archivi storici restano leggibili e non vengono riscritti al primo avvio.
- **Compatibilità:** nessuna migrazione ipotizzata per il minimo, stessi vincoli SQLite e API Rust esistenti, Windows/macOS/Linux, tastiera e WebView desktop. Verificare con report mensili, piani rateali e allegati incasso v6; possibili modalità di pagamento diverse vanno trattate con semantica attuale e senza conversioni automatiche.

## 6. Confronto con le idee già presenti, dipendenze e rischi

- [REPS-UX-024](REPS-UX-024-scadenziario-incassi-piani-pagamento-vendite.md) riguarda **scadenze delle rate**, non l'eventuale doppio inserimento di una registrazione **già effettuata**.
- [REPS-UX-030](REPS-UX-030-quadratura-facoltativa-estratti-conto-locali.md) studia il confronto volontario con **CSV bancari importati**: qui nessuna banca o import, solo i record manuali di una vendita.
- [REPS-UX-039](REPS-UX-039-ripartizione-informativa-incassi-dichiarati.md) descrive ripartizioni economiche dei **dati già registrati**, non verifiche di input.
- [REPS-UX-063](REPS-UX-063-riepilogo-economico-prima-registrazione-vendita.md) è una revisione **prima della creazione della vendita**, non prima di un suo singolo incasso.
- [REPS-UX-068](REPS-UX-068-verifica-collezionista-esistente-prima-inserimento.md) verifica omonimie fra **persone**; il pattern di avviso prudente può essere condiviso ma non si riutilizza l'algoritmo d'identità.
- [REPS-TECH-033](https://github.com/Cesare-ml/Repertica-Studio/blob/docs/idee-strumenti-architettura-20261121/docs/idee/strumenti-e-architettura/REPS-TECH-033-salvataggio-condizionale-schede-obsolete.md) esamina **modifiche concorrenti di schede**, non corrispondenze fra incassi; [TECH-055](https://github.com/Cesare-ml/Repertica-Studio/blob/docs/idee-strumenti-architettura-20261121/docs/idee/strumenti-e-architettura/REPS-TECH-055-esportazione-coerente-registri-csv.md) riguarda esportazioni. Non modificare file tecnici.
- **Dipendenze minime:** registro e form incassi esistenti, i18n già presente e metodo di lettura/salvataggio correnti. Nessuna libreria, OCR, AI, telemetria o API di pagamento proposta.

**Rischi:** falsi positivi su due contanti identici; falso senso di sicurezza quando non appare un avviso; degradazione UX se gli avvisi sono troppo frequenti; conferma irriflessa; rallentamento su registri lunghi; doppio invio dopo timeout; confusione tra duplicato *possibile* e operazione bancaria fraudolenta. Mitigazioni: scopo per vendita, matching semplice spiegabile, avviso non bloccante, nessuna pretesa di verificare movimenti reali, testo prudente, decisione esplicita, prova preventiva sui falsi positivi.

**Alternative prima dello sviluppo:** rendere più visibile o ordinabile il piccolo elenco degli incassi già presente nel dialogo e mostrare un promemoria statico «Controlla le registrazioni precedenti»; formare l'utente a inserire un riferimento distinguibile; usare la semplice esportazione CSV esistente quando deve riconciliare manualmente. Se questo basta, **non sviluppare** rilevamento.

**Complessità qualitativa:** **bassa–medio-bassa** per avviso puro su incassi già caricati, **media** se robustezza del caso ambiguo «salvataggio riuscito ma risposta persa», refresh e accessibilità diventano requisiti. Il backend e il formato di backup non dovrebbero cambiare nel minimo. Non è stimata una durata né approvata una soluzione tecnica.

## 7. Criteri di accettazione ipotetici e prove da fare, non eseguite

1. Una vendita con saldo residuo sufficiente e un incasso identico nel giorno/importo/metodo produce un **avviso prima della nuova scrittura**, non una seconda riga immediata.
2. «Torna a verificare», Escape e chiusura del pannello conservano i dati del modulo e non cambiano saldo, report, allegati né incassi.
3. «Registra comunque» dopo conferma volontaria crea **una sola nuova riga** solo se il backend accetta la somma; sono preservati entrambi i UUID e tutti i documenti.
4. Due incassi realmente distinti con stesso giorno/importo/metodo, e residuo sufficiente, rimangono registrabili.
5. Riferimenti vuoti non generano un match «per riferimento»; un riferimento ripetuto con importi diversi non produce certezza di identità.
6. Incasso eccedente il residuo continua a essere respinto dal backend, anche dopo una conferma di «distinto».
7. Due click e risposta incerta non devono portare a messaggi falsi di successo; controllare la lista prima di un eventuale reinvio, senza dedurre un esito dalla sola assenza di risposta.
8. Test con 4–6 professionisti e scenari sintetici misurano **falsi positivi, falsi negativi e tempo di registrazione**. Se gli avvisi sono disturbanti si rinuncia alla funzione.
9. Test accessibilità (tastiera, screen reader, focus, zoom 200%), IT/EN/ES/FR/DE, fallback inglese, date locali, Unicode e valori esatti in centesimi.
10. Prove desktop Windows/macOS/Linux, archivio con molte vendite e incassi, eventuali modifiche intervenute prima della conferma; CSV, backup e statistiche non devono subire regressioni. Nessun test è stato creato o eseguito nell'attuale attività documentale.

## 8. Priorità, fasi, requisiti certi / ipotesi / domande

**Requisiti certi / vincoli del progetto:** offline gratuito e open source, niente account o abbonamento, data locale ed esportabilità completa, backup su cartella scelta, lingua sistema con fallback inglese; incassi manuali e non bancariamente verificati; limite di somma incassi non superiore al lordo già presente.

**Ipotesi da validare:** frequenza di doppi inserimenti reali, qualità degli indizi data/importo/metodo/riferimento, vantaggio di un avviso rispetto alla sola lista visibile, priorità P2, costo cognitivo del confronto e fattibilità UX con dialogo lungo.

**Fase 0 — senza sviluppo:** valutare con 4–6 artisti/galleristi alcune vendite a più incassi e osservare se consultano già la tabella; verificare 10–20 casi reali anonimizzati o sintetici di «due pagamenti simili», senza raccogliere dati bancari veri.

**Fase 1 — eventuale minimo:** confronto limitato a stessa vendita con indizi espliciti; avviso accessibile prima del commit, alternative «Verifica / Registra comunque», nessun dato nuovo persistito.

**Fase 2 — solo se supportata dai test:** semantica differenziata dei riferimenti ripetuti, verifica dopo esito IPC incerto, comportamento con elenchi lunghi e aggiornamenti concorrenti; non introdurre vincoli di unicità che blocchino pagamenti legittimi.

**Domande riservate al proprietario:** (a) esistono casi concreti di doppi incassi registrati in Repertica o nei gestionali precedenti? (b) quanto spesso due pagamenti *distinti* condividono data e importo? (c) è utile evidenziare il solo riferimento identico oppure genera troppi falsi avvisi? (d) preferire un pannello nella maschera o un piccolo dialogo di conferma? (e) basta una tabella precedente più leggibile? (f) vale la priorità P2 in confronto a collaudi, backup e precisione dei registri?

**Nessuna decisione approvata:** il documento è una proposta da valutare personalmente, non una specifica vincolante, un bug confermato o un incarico di sviluppo.
