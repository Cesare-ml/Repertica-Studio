# REPS-UX-024 — Scadenziario degli incassi concordati e piani di pagamento per vendita

- **ID stabile:** REPS-UX-024
- **Data:** 2026-11-21
- **Stato:** Proposta — da valutare
- **Filone:** Gestione e UX
- **Priorità ipotetica:** P1–P2 per vendite con acconti, saldi differiti e rate; P3 per chi vende solo con pagamento contestuale. Da convalidare.
- **Perimetro:** soltanto studio e documentazione; nessuna approvazione, issue operativa o assegnazione allo sviluppo.

## Aggiornamento di ricognizione su `main` (2026-11-21) — evitare implementazioni duplicate

**La descrizione originale più sotto è una fotografia della situazione precedente, non dello stato attuale di `main`.** Il branch principale ora documenta e contiene già il nucleo del piano rateale (SQLite **v11**): modalità pagamento `INSTALLMENTS`, **2–60 rate** con scadenze, piano modificabile, ripartizione automatica degli incassi registrati in ordine di scadenza e stati pagata/parziale/scaduta/da incassare. Sono presenti il registro CSV `installments`, `src-tauri/src/installments.rs`, `src/app/payment-plan.component.ts` e comandi Tauri di gestione. La fonte è [docs/MVP.md](../../MVP.md) e il codice di `main`, verificati il 21/11/2026.

Di conseguenza **non riproporre come nuova funzione** la creazione delle rate, la scadenza, i saldi o il calcolo degli stati: questi elementi risultano già sviluppati. Rimane eventualmente una **domanda separata da validare**: servono una cronologia esplicita delle *revisioni degli accordi* con fonte e causale, oppure funziona già adeguatamente il piano modificabile esistente? Prima di considerare altro, verificarne l'UX reale e i test su desktop e interpellare utenti. **La presente scheda mantiene lo stato «Proposta — da valutare» per le sole ipotesi ancora aperte; non costituisce un'approvazione retroattiva del progetto ideativo né un incarico.**

## 1. Problema riscontrato e destinatari

Un artista vende un'opera oggi per 3.000 EUR, concordando 1.000 EUR subito e due rate da 1.000 EUR alle date successive. Il gestionale deve distinguere **quanto è stato venduto**, **quanto è previsto di incassare e quando**, **quanto l'utente dichiara già incassato** e **quanto rimane scoperto**. Senza date e condizioni concordate, un saldo residuo unico non basta a distinguere il credito non ancora esigibile da quello da verificare perché la scadenza è trascorsa.

Per piccoli studi, artisti che vendono direttamente e gallerie indipendenti, la scadenza dei pagamenti concordati può essere un'esigenza concreta di lavoro. Non tutte le vendite hanno un piano rateale: la funzione deve restare facoltativa, silenziosa e leggera.

**Riscontro nel repository al 2026-11-21:** in `main`, `src/app/models.ts` espone `Sale` con importo EUR e data, `SaleReceipt` con incassi registrati manualmente, `SaleBalance` con venduto/incassato/residuo. `docs/SALE-RECEIPTS.md` documenta il registro locale degli incassi parziali e le statistiche mensili basate sulla data dell'incasso dichiarato. Nei modelli, nella documentazione e nella ricerca nel codice non emerge un calendario strutturato di **scadenze concordate** collegato alla vendita. È un'opportunità da validare, non un difetto provato.

### Distinzioni e non duplicazione

- **Non è un nuovo registro pagamenti.** `sale_receipts` e `list_sale_balances` rimangono fonte autorevole delle somme **dichiarate come ricevute**. La scheda non aggiunge un secondo incasso né modifica i vincoli che impediscono sovraincassi.
- **Non è la [issue #13](https://github.com/Cesare-ml/Repertica-Studio/issues/13).** Finance 2 già contempla incassi/eventi di pagamento, documenti e separazione del cashflow; questa proposta si limita all'**impegno temporale dichiarato** e alla sua UX, coordinandosi con i moduli esistenti. La scadenza non è un movimento bancario.
- **Non è [UX-002](REPS-UX-002-trattative-prenotazioni.md).** La trattativa precede la vendita; qui un piano viene associato soltanto a una vendita registrata. Eventuali condizioni dell'offerta possono essere copiate solo con conferma, mai trasformate in incasso.
- **Non è [UX-012](REPS-UX-012-affidamenti-conto-vendita-rendiconti.md)**: i rendiconti fra galleria e artista/intermediario non sono il calendario dell'acquirente finale.
- **Non è [UX-013](REPS-UX-013-resi-annullamenti-rettifiche-vendite.md)**: la gestione di annullamenti/rimborsi è un altro processo. Se in futuro esisterà, dovrà ridefinire il piano esplicitamente, non alterare lo storico in modo silenzioso.
- **Non è [UX-017](REPS-UX-017-vendita-multiopera-ripartizione-importi.md)**: il piano si riferisce alla vendita nel suo complesso, indipendentemente dalle righe opera.
- **Non è TECH-001/002/008/010/019** della [PR tecnica #16](https://github.com/Cesare-ml/Repertica-Studio/pull/16): rispettivamente portabilità, automazioni locali, filtri, storico trasversale e import tabellare. Qui si definiscono il processo commerciale, la terminologia e le decisioni dell'utente; nessun file tecnico viene modificato.

## 2. Utenti e scenari verificabili

1. **Artista — acconto e saldo.** Vendita di 2.400 EUR; impegno di 400 EUR alla firma e 2.000 EUR alla consegna entro una data concordata. Il primo versamento, se realmente ricevuto, si inserisce nel registro incassi già disponibile, non nel piano.
2. **Galleria — tre rate concordate.** Vendita di 3.000 EUR; rate da 1.000 EUR alle date 15/12/2026, 15/01/2027 e 15/02/2027. Le somme pianificate non aumentano l'incassato mensile. Quando l'utente registra 500 EUR, la UI mostra un totale residuo di 2.500 EUR e un confronto con le scadenze.
3. **Accordo ridefinito.** Cliente e artista spostano una scadenza; il piano conserva la data originaria e il motivo della nuova condizione. Un cambio di piano non modifica l'importo della vendita o gli incassi.
4. **Vendita senza condizioni note.** Il programma mostra «Scadenze non definite», non «Scaduto» e non «Da incassare oggi». L'importo residuo continua a derivare dagli incassi esistenti.
5. **Vendita già saldata.** Un piano eventualmente aperto non deve generare falsi arretrati: l'interfaccia evidenzia che il residuo registrato è zero e conserva il piano solo per documentazione.

## 3. Flussi UX proposti

- **Vendite → Dettaglio vendita → Condizioni di pagamento (facoltative):** visualizzare prezzo registrato, totale incassi dichiarati e saldo già esistenti. Mostrare un'azione «Aggiungi una scadenza» oppure «Definisci un piano», senza forzare chi non ne ha bisogno.
- **Creazione piano:** modalità «Data unica» o «Più scadenze»; righe con data locale ISO, importo EUR in centesimi, descrizione opzionale («acconto», «saldo»), fonte dell'accordo (nota/riferimento documento) e stato `bozza` o `confermato dall'utente`. Anteprima con somma delle righe, prezzo vendita e differenza. Un piano confermato copre **esattamente** l'importo della vendita; una bozza incompleta è esplicitamente etichettata «non completa» e non dà origine ad arretrati.
- **Riepilogo scadenze:** tabella locale con vendita/opera, acquirente (visibile nell'archivio, escluso per default da condivisioni), prossima data, importo originariamente concordato, incassi registrati, importo residuo e indicatori «da verificare», «non ancora scaduto», «scadenza trascorsa»; il testo non deve affermare un inadempimento legale. Filtri per finestra temporale e vendita; nessuna agenda cloud.
- **Collegamento agli incassi:** CTA «Registra incasso» apre **il flusso esistente** per `sale_receipts` con riferimento alla vendita, senza pre-registrare rate come incassi. Dopo salvataggio, il riepilogo legge di nuovo il saldo autorevole.
- **Attribuzione delle somme alle rate:** punto delicato da non nascondere. **Opzione base consigliata solo per valutazione:** vista illustrativa che imputa gli incassi registrati a partire dalla scadenza più vecchia, esplicitando «attribuzione convenzionale per il prospetto, non dichiarazione di accordo o causale bancaria»; la somma incassata totale resta quella di `sale_receipts`. **Alternativa:** collegamento manuale incasso–rata, più accurato ma molto più complesso; nessuna imputazione automatica invisibile. Se si preferisce evitare questa decisione in fase A, mostrare soltanto saldo totale e date concordate, **senza** importi di singole rate classificati come saldati.
- **Revisioni:** «Riprogramma» richiede conferma, motivo facoltativo, anteprima prima/dopo e conserva versioni precedenti. L'azione non aggiorna `sales`, non elimina ricevute, non modifica certificati e non comunica con l'acquirente.
- **Accessibilità e UX:** tastiera completa, etichette del tipo di dato e della fonte, tabella leggibile dagli screen reader, nessun colore come unica indicazione di scadenza, focus prevedibile dopo modifica, formato data rispettoso del locale e conferme reversibili dove possibile. Vista compatta per chi non usa rate.

## 4. Dati necessari — modello soltanto concettuale

- `payment_plan`: UUID, `sale_id` UUID, stato (`draft`/`confirmed`/`superseded`), versione, data accordo facoltativa, note e fonte dichiarata, creato/aggiornato, nessuna valuta diversa da quella della vendita attuale (EUR).
- `payment_due`: UUID, piano UUID, posizione, `due_on` (data calendario senza ora/fuso), `amount_cents` intero positivo, descrizione facoltativa, `reference` testuale, ordinamento deterministico.
- `payment_plan_revision`: UUID, piano/versione precedente, data della rettifica, motivo dichiarato e snapshot dei termini usati prima della modifica. Una versione passata è consultabile, non contabilizzata.
- **Eventuale** `receipt_due_allocation` solo se approvata una fase avanzata: legame a UUID di incasso già esistente e a UUID della riga di piano, centesimi attribuiti, data/causale e regole di riassegnazione. Da non introdurre per inerzia nel MVP.
- **Invarianti:** una sola versione `confirmed` corrente per vendita; importi delle rate confermate = `sales.amount_cents`; nessun totale negativo, zero o eccedente; date reali validate; residuo complessivo **sempre** = `sales.amount_cents - SUM(sale_receipts.amount_cents)`, mai dedotto dal piano. Il piano è una dichiarazione dell'utente, **non prova di incasso, credito esigibile o titolo esecutivo**.
- **Restauro e vendite storiche:** vecchie vendite senza piano rimangono prive di scadenze. Nessuna migrazione che inventi «immediato», «30 giorni» o rate già pagate. Non generare nuovi record per deduzione dai vecchi incassi.

## 5. Offline, impostazioni, multilingua

- **Offline-first:** SQLite locale; calendario calcolato senza servizi terzi, email, notifiche, sincronizzazione o connessione. Eventuali promemoria restano consultabili all'apertura; quando il computer è spento o l'app è chiusa non si promettono avvisi. Nessuna automazione di sollecito o contatto al cliente.
- **Impostazioni:** modulo facoltativo; modello di scadenza suggerito solo se richiesto (es. 30 giorni), **mai attivato per default**; preferenza per nascondere dettagli sensibili nelle viste condivisibili; nessuna aliquota, interesse moratorio o penale preimpostata.
- **Lingue:** lingua del sistema operativo con fallback inglese, opzione utente nelle lingue già supportate (IT/EN/ES/FR/DE). Tradurre «concordato», «registrato come incassato», «scadenza trascorsa» e «piano incompleto» in modo non ambiguo; testi e clausole inseriti dall'utente non sono tradotti automaticamente. Date visualizzate secondo locale, archiviate come ISO `YYYY-MM-DD`.
- **Compatibilità desktop:** Windows, macOS, Linux; evitare `DateTime` con conversioni UTC che trasformino una scadenza in giorno precedente/successivo. I dati già salvati e i backup precedenti si aprono senza piani inventati.

## 6. Privacy, export/import, backup

- **Dati riservati:** nominativi, scadenze, termini negoziali e note su pagamenti possono essere sensibili. Nessuna telemetria, tracking, invio automatico o apertura di URL di pagamento. La dashboard privata può mostrare la controparte, ma la **stampa/esportazione** richiede anteprima esplicita e selezione delle colonne; omettere per default recapiti, note private e riferimenti di pagamento.
- **CSV per analisi:** UUID di vendita/piano/rata, versione corrente, data, importo in centesimi EUR, stato di piano e indicatore separato `reported_received_cents` a livello **vendita**. Non presentare righe come transazioni né come valori di cashflow realizzato; niente doppia somma con l'export incassi.
- **Portabilità:** JSON aperto e versionato con UUID e relazioni per un futuro interscambio completo (coordinabile, senza riscriverlo, con TECH-001). Import locale solo con preflight, mapping delle vendite, avvisi di UUID duplicati, preview e rollback, senza generare incassi o retrodatare accordi.
- **Backup/ripristino:** piani, versioni e riferimenti documentali inclusi nei backup integrali già previsti, salvabili/ripristinabili scegliendo liberamente la cartella. Esplicitare che il backup attuale non è cifrato; esportazioni e copie possono essere consultate da chi accede alla cartella.
- **Compatibilità con futuri allegati TECH-016:** riferimento opzionale a un documento locale **solo se** il sistema allegati verrà approvato; un file non deve essere imposto come prova dell'accordo.

## 7. Complessità, dipendenze, alternative e rischi

- **Complessità:** bassa–media per sola data unica; media per tabella rate, versioni ed export; alta per imputazione manuale, rettifiche, resi, multiopera e import coerente con scadenze pregresse.
- **Dipendenze già presenti:** UUID vendita e collezionista, importi in centesimi EUR, registro manuale incassi parziali, backup SQLite/media, UI i18n. **Nessuna dipendenza SaaS, banca, gateway di pagamento, fatturazione o account.** Dipendenza concettuale facoltativa con UX-002/012/013/017 e TECH-001/016/002, senza richiederne l'approvazione.
- **Rischi principali:** confondere previsto con incassato; usare il termine «insolvente» o «credito scaduto» senza base contrattuale; falsa precisione nell'attribuzione alle rate; corrompere saldi quando una rata viene spostata; duplicare vendite o incassi; rivelare negoziazioni riservate in CSV/dossier; problemi di data a cavallo di fuso; far sembrare una semplice scheda una fattura legale.
- **Mitigazioni:** tre cifre sempre separate (venduto, pianificato, dichiarato incassato), terminologia prudente, indicazione della fonte, versioni storiche, nessuna scrittura implicita ai registri finanziari, permessi minimi per export, zero invii automatici, test sui mesi a cavallo dell'anno e sui casi con ricevute precedenti.
- **Alternativa semplice:** solo `payment_due_on` facoltativo per vendita e campo note, senza rate, cronologia o dashboard globale. **Alternativa esterna:** calendario/CSV locale gestito dall'utente, sacrificando collegamento ai saldi ma senza costo di sviluppo.

## 8. Criteri di accettazione proposti (non test implementati)

1. Vendita di 3.000,00 EUR con tre rate 1.000,00 EUR: piano confermato totalizza esattamente 300.000 centesimi; fatturato e incassato restano invariati alla creazione.
2. Registrando un incasso reale dichiarato di 500,00 EUR tramite il flusso esistente, il saldo complessivo risulta 2.500,00 EUR. Il prospetto **non** crea un secondo incasso né dichiara per certo quale rata bancaria sia stata pagata.
3. Un piano confermato con righe per 3.001,00 EUR su vendita 3.000,00 EUR è rifiutato; una bozza incompleta non genera scadenze erroneamente conteggiate.
4. Vendite storiche prive di piano rimangono con «scadenza non definita», anche quando il residuo è positivo; mai classificarle retroattivamente come scadute.
5. Modificare 15/01/2027 in 31/01/2027 conserva la scadenza precedente consultabile; non cambia `sale_receipts`, certificati, `sales.amount_cents` o statistiche mensili degli incassi.
6. Un incasso parziale precedente alla creazione del piano non viene duplicato; il piano deve rendere visibile il residuo senza inventare causali.
7. Date al confine di mese/anno, fuso orario di sistema e ora legale non spostano la giornata di scadenza.
8. CSV/JSON e backup/ripristino preservano esattamente importi, date, UUID, versioni e riferimenti, senza duplicati; nessuna comunicazione parte verso contatti o rete.
9. Navigazione completa da tastiera e screen reader, etichette di stato testuali e fallback inglese dove manca una traduzione.
10. Un piano senza documento non viene segnalato come accordo verificato; terminologia e schermata distinguono sempre «termini dichiarati» da evidenza di un contratto.

## 9. Fasi possibili, tutte subordinate ad approvazione futura

- **Fase A — Verifica del bisogno:** interviste a 3–5 artisti/galleristi con casi reali di rate; confronto tra sola scadenza e piano multiplo; test di comprensione tra «incassato» e «promesso». Nessuna implementazione automatica.
- **Fase B — Minimo utile:** data di saldo facoltativa collegata alla vendita e vista locale dei saldi con/senza scadenza, senza imputazione alle rate.
- **Fase C — Piani multipli:** rate con somma verificata, storico delle revisioni, anteprima export e indicatori prudenti di date trascorse.
- **Fase D — Solo se necessaria:** attribuzione esplicita incasso–rata, interscambio versionato e casi di rettifiche/resi coerenti con le altre proposte approvate.

## 10. Requisiti certi, ipotesi e domande al proprietario

**Requisiti certi del progetto:** desktop Tauri open source gratuito, offline-first, senza login/abbonamento; dati SQLite locali e sempre esportabili; backup e ripristino in cartella liberamente scelta; lingua di sistema e fallback inglese; un incasso è registrato manualmente ed è distinto dalla vendita.

**Ipotesi da validare:** la frequenza dei pagamenti dilazionati è sufficiente da giustificare uno scadenziario; serve una vista globale; gli utenti desiderano registrare i termini anche quando gli importi sono già incassati; le rate devono sempre coprire il lordo della vendita e non soltanto il saldo ancora da incassare.

**Domande per il proprietario:**
1. I casi tipici sono pagamento immediato, acconto+saldo oppure 3–12 rate? Quanti casi all'anno?
2. Preferire una sola data «saldo atteso» o più scadenze? È utile uno storico delle revisioni?
3. Serve conoscere il valore **di ogni singola rata ancora aperta**, accettando imputazioni esplicite, oppure basta mostrare saldo totale e prossime scadenze senza attribuzioni?
4. Con consegne e certificati, si vuole solo un avviso informativo «pagamento non completo» o una regola gestionale? **Non** inferire diritti, obblighi di consegna o effetti legali.
5. Lo scadenziario deve comprendere in futuro accordi con gallerie/intermediari (UX-012), oppure rimanere rigorosamente confinato alle vendite dirette ai collezionisti?

## Riferimenti per la valutazione

- Repertica [MVP](../../MVP.md), [incassi delle vendite](../../SALE-RECEIPTS.md), [economia e localizzazione](../../ECONOMY-I18N.md), `src/app/models.ts`, `src-tauri/src/receipts.rs` in `main` (consultati il 2026-11-21).
- [Artlogic Pay — depositi, pagamenti parziali e flessibili](https://artlogic.net/pay) e [ITGallery — pagamenti parziali collegati alla vendita](https://www.itgalleryapp.com/en/art-gallery/sales.html): esempi di bisogni del settore. **Non** sono modelli di licenza, architettura o pagamenti da replicare; Repertica dovrebbe conservare autonomia, offline e nessuna obbligazione online.

**Decisione attuale:** Proposta — da valutare. Nessuna funzionalità autorizzata allo sviluppo.
