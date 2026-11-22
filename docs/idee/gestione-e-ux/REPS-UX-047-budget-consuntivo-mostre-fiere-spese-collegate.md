# REPS-UX-047 — Budget e consuntivo di mostre e fiere senza duplicare le spese

- **ID stabile:** REPS-UX-047
- **Data:** 2026-11-22
- **Stato:** Proposta — da valutare
- **Filone:** Gestione e UX (solo analisi documentale)
- **Priorità ipotetica:** P2 **condizionata** a un riscontro di utilizzo frequente presso artisti che partecipano a fiere o gallerie; P3 o nessuno sviluppo se basta un foglio locale
- **Approvazione/assegnazione:** nessuna; la presente scheda non autorizza implementazioni, issue operative o modifiche al prodotto

## 1. Problema professionale, destinatari e fatti riscontrati

Un artista partecipa a una fiera e sostiene quota dello stand, trasporto, stampe, imballaggi e ospitalità. Prima della manifestazione vuole sapere **quanto prevede di spendere**; dopo vuole confrontare il preventivo con le **spese che ha registrato**, capire quali non ha ancora classificato e predisporre un consuntivo per sé o per un partner. Non può usare correttamente il semplice totale delle spese del mese: una mostra può coprire due mesi, condividere spese con altri eventi e includere acquisti non esclusivi.

**Destinatari:** artista indipendente che partecipa a fiere, piccola galleria che gestisce più mostre, studio con progetti espositivi, archivio che rendiconta un'iniziativa circoscritta. Non è pensato come contabilità fiscale o sistema per organizzatori di fiere.

**Verifica di `main` il 22/11/2026:** [README](../../../README.md), [MVP](../../MVP.md), [Economia e i18n](../../ECONOMY-I18N.md), [portabilità](../../DATA-PORTABILITY.md), [modelli Rust](../../../src-tauri/src/models.rs) e [comandi](../../../src-tauri/src/commands.rs). Repertica Studio è desktop Tauri/Angular/Rust/SQLite; registra `expenses` con data, categoria, fornitore, importo in centesimi e fondo opzionale; espone statistiche mensili, vendite distinte dagli incassi, ripartizioni e CSV. Le spese registrate sono **registrazioni gestionali**, non attestazioni di pagamento bancario. Non emerge un'associazione autorevole di **una spesa a un'iniziativa espositiva** né un budget per evento. Il prodotto rimane MVP, non ancora unico archivio di produzione raccomandato.

**Natura dell'osservazione:** lacuna funzionale *potenziale*, non bug e non richiesta confermata da professionisti. Nessun numero o beneficio economico viene presunto.

## 2. Confini anti-doppione, anche con l'altro filone

| Ambito già presente/proposto | Differenza precisa |
| --- | --- |
| [UX-009 — Progetti espositivi](REPS-UX-009-progetti-espositivi-checklist.md) | Prevede **selezioni di opere, attività, scadenze e checklist organizzative**; qui l'oggetto è soltanto **pianificazione e attribuzione economica**, non il programma della mostra. Se UX-009 fosse adottata, riutilizzarne lo **stesso identificativo di progetto**, senza creare una seconda anagrafica eventi. |
| [UX-032 — Budget mensili](REPS-UX-032-budget-mensili-categorie-spesa-scostamenti.md) | Target per **mese e categoria**; una fiera attraversa periodi diversi e può impiegare solo parte di una spesa di un mese. Sono due *viste analitiche* dello stesso registro di spese, **mai due spese**. |
| [UX-003 — Costi per opera](REPS-UX-003-costo-e-margine-per-opera.md) | Attribuzione a una **singola opera**. Un costo di trasporto può comparire nella vista mostra e, con criterio dichiarato, nell'analisi di una singola opera, ma **non dev'essere sommato due volte** nella tesoreria. |
| [UX-008 — Scorte](REPS-UX-008-scorte-materiali-consumi.md), [UX-028 — Giustificativi](REPS-UX-028-giustificativi-locali-spese-acquisti.md), [UX-042 — Pagamenti a fornitori](REPS-UX-042-pagamenti-dichiarati-spese-acquisti-differiti.md) | Quantità di materiali, allegati e pagamenti sono rispettivamente le **fonti dei propri domini**; non si ricreano magazzino, ricevute o movimenti da un budget. |
| [UX-012 — Conto vendita](REPS-UX-012-affidamenti-conto-vendita-rendiconti.md), [UX-017 — Vendite multiopera](REPS-UX-017-vendita-multiopera-ripartizione-importi.md) | Gli accordi con intermediari e i registri vendita rimangono separati. Una vendita avvenuta durante la fiera **non viene automaticamente attribuita** alla fiera. |
| [TECH-001 — Portabilità aperta](https://github.com/Cesare-ml/Repertica-Studio/blob/docs/idee-strumenti-architettura-20261121/docs/idee/strumenti-e-architettura/REPS-TECH-001-pacchetto-dati-aperto.md), [TECH-025 — Esportazioni selettive](https://github.com/Cesare-ml/Repertica-Studio/blob/docs/idee-strumenti-architettura-20261121/docs/idee/strumenti-e-architettura/REPS-TECH-025-esportazioni-tabellari-minimizzate-condivisione.md) | Definiscono infrastruttura e privacy di scambio; qui solo **significato contabile e percorso UX**. Non modificare né duplicare quelle schede. |

**Esclusioni:** emissione fatture, IVA/detrazioni, conto corrente, pagamenti presunti, ricavi attesi spacciati per incassi, sistemi POS, prenotazione stand, calendari online, preventivi inviati automaticamente, collegamento a gestionali cloud, tariffazione degli incarichi e nuove commissioni di vendita.

## 3. Scenari d'uso testabili

1. **Artista a una fiera.** Prevede 1.000 €: stand 600, trasporto 200, materiale promozionale 200. In Economia registra separatamente tre spese **effettive** (nel senso di *registrate*) per 600 + 160 + 40 = **800 €**. Nel progetto: «obiettivo 1.000 € / spese attribuite 800 € / scostamento −200 €». Il totale della tesoreria resta 800 € e non diventa 1.600 €.
2. **Costo condiviso.** Una spesa di trasporto da 300 € serve due mostre: 180 € attribuiti all'evento A, 120 € all'evento B. È vietato attribuirne 200 + 200 senza conferma di una correzione della ripartizione, perché supererebbe 300 €.
3. **Fiera a cavallo di due mesi.** Acconto per lo stand registrato a ottobre, trasporto a novembre: la vista fiera li include entrambi se attribuiti, mentre i riepiloghi mensili conservano le proprie date e importi.
4. **Nessuna spesa attribuita.** Un evento con budget compilato ma privo di collegamenti mostra «nessuna spesa collegata», **non** «evento costato zero». Sono distinguibili budget assente e budget esplicitamente pari a zero.
5. **Vendita in mostra.** L'artista vende un'opera durante i giorni dell'evento; la vendita e gli eventuali incassi rimangono nel registro autorevole. Non dedurre la relazione dal solo intervallo di date né proclamare «utile netto della mostra». Solo un'eventuale fase successiva potrebbe consentire un collegamento **manuale** a vendite già registrate, e distinguere venduto, incassato dichiarato e spese attribuite.

## 4. Flussi UX candidati

**A — Vista evento → «Budget e spese» (solo se UX-009 sarà adottata).** Mostrare «Preventivato», «Attribuito dal registro spese», «Non ancora attribuito / da verificare», «Scostamento». In un eventuale MVP indipendente, usare un **progetto economico minimo**, con nome e periodo, dichiarando che **non è** un secondo gestore di mostre. Se un domani nasce UX-009, prevedere il riuso/consolidamento esplicito dell'identità, non un doppio record nascosto.

**B — «Prepara budget»**: scegliere evento/periodo, aggiungere voci libere (stand, trasporto, allestimento…), importi in centesimi nella valuta selezionata e note facoltative; nessuna categoria, importo o fornitore obbligatorio. Mostrare revisione e data. Salvare il preventivo **non** crea spese, ordini o promemoria pagamenti. Un target pari a zero è intenzionale e distinto da campo mai valorizzato.

**C — «Collega spesa esistente»**: ricerca locale nel registro delle spese, per data/fornitore/categoria/testo; anteprima di importo totale, quantità già attribuita a eventi e residuo; inserire l'importo da imputare a questo evento e confermare. Le spese fuori periodo non sono nascoste: richiedono un avviso e possono essere corrette (anticipo/consuntivo). L'utente può togliere il collegamento senza eliminare la spesa.

**D — Correzioni trasparenti.** Se la spesa originale viene ridotta, cancellata o cambiata di data, mostrare un conflitto da risolvere; non ridurre di nascosto quote già attribuite, non duplicare spese e non mantenere un consuntivo falsamente «verificato». L'eventuale collegamento viene sospeso/annotato in modo conservativo finché l'utente decide. Revisione del budget con confronto «prima/dopo», senza cancellare retroattivamente una stampa storica.

**E — Consultazione, accessibilità e condivisione.** Evidenziare voci di budget senza spesa associata, spese collegate fuori periodo e importi condivisi. Non usare soltanto colori: etichette testuali e totale in formato locale; focus evidente, navigazione completa da tastiera, righe leggibili da screen reader, zoom desktop e stampa leggibile. Un'anteprima distingue «riepilogo interno» da «copia condivisibile» e nasconde **di default** fornitori, note private, margini e nominativi.

## 5. Dati e API ipotetiche — **non sono uno schema approvato**

- `event_budget`: UUID, riferimento **facoltativo** a progetto UX-009 se esisterà (in alternativa un `economic_event_id` unico e non ambiguo), nome, periodo dichiarato e precisione, valuta ISO 4217, stato (bozza/confermato), data registrazione, note riservate e versione.
- `event_budget_line`: UUID, budget UUID, categoria/libera descrizione, `planned_cents` intero **≥ 0**, valuta comune, ordinamento e revisione. Assenza della riga ≠ riga con importo zero.
- `event_expense_allocation`: UUID, evento UUID, **UUID della `expense` autorevole**, importo attribuito in centesimi > 0, data e motivazione facoltativa, stato del collegamento, timestamps; nessuna nuova spesa. Vincolo centrale: per ogni spesa la **somma delle quote collegate a eventi non deve superare `expenses.amount_cents`**; somme e sottrazioni solo con interi, senza float.
- La stessa spesa può avere **attribuzioni analitiche in dimensioni diverse** (evento, opera, budget mensile): non si sommano tali dimensioni per calcolare il totale dello studio. Le attribuzioni a eventi invece condividono un unico limite di capienza nella dimensione «eventi». Eventuali spese negative, rimborsi o valute diverse richiedono una scelta semantica separata: evitare collegamento automatico fino a regole verificate.
- **Viste derivate** (ipotesi concettuali): `event_budget_overview(event_id)` → target, somme attribuite, copertura, scostamento e riferimenti alle spese sorgente; `preview_event_expense_allocation(expense_id, event_id, amount)` → disponibilità e conflitti; `list_event_cost_sources(event_id)`. Nessuna nuova API implementata, nessun comando eseguito.
- La differenza `spese attribuite − budget` va etichettata come **scostamento**; valori positivi significano sopra preventivo, negativi sotto. Percentuale solo con budget > 0; con budget=0 mostrare «sopra preventivo di X», senza divisione per zero.
- **Nessuna stima automatica di utile/ROI:** eventualmente, con esplicita attribuzione delle vendite in fase successiva, vista separata `venduto registrato − spese attribuite` come *indicatore gestionale limitato*, non incasso, liquidità, margine fiscale o profitto di un progetto.

## 6. Impostazioni, offline e multilingua

Funzione **facoltativa** e invisibile finché non richiesta; senza onboarding obbligatorio né categoria standard imposta. Preferenza locale per colonne visibili e modalità interna/condivisibile; eventuale impostazione per valuta **solo dopo** una decisione su UX-027 (nessuna conversione automatica tra valute). Non imporre soglie, notifiche, credenziali o scansione di altre cartelle.

Tutti i record in SQLite locale; stessi backup completi verificati, destinazione scelta liberamente (USB/NAS montato incluso), nessun server o account/abbonamento. Un dispositivo offline ha esattamente le stesse funzioni. Nessun cloud implicito, nessun inoltro alle integrazioni MCP/IA, nessuna sincronizzazione di costi o fornitori.

Lingua dell'interfaccia rilevata dal sistema con override locale, **fallback inglese**; IT/EN/ES/FR/DE secondo il prodotto. Etichette, avvisi e unità tradotte; **nomi di eventi, note e categorie inserite dall'utente non vengono tradotti**. Date e importi mostrati nel locale scelto, ma importi memorizzati in centesimi e dati esportati in valori non ambigui. Fuso orario e giorni di confine spiegati nelle viste periodo.

## 7. Portabilità, compatibilità e privacy

- **Esportazione:** CSV/JSON versionato e documentato **del solo budget e delle associazioni**, con UUID evento/riga/spesa, valuta, quantità in centesimi e date ISO, distinguendo importi previsti da spese registrate. Secondo export umano opzionale con anteprima delle colonne; mai sostituire o alterare i CSV integrali esistenti.
- **Importazione eventuale, non in MVP:** scelta manuale di file, controllo schema/versione, mapping dei riferimenti a spese che esistono davvero nell'archivio locale, anteprima per duplicati/UUID estranei, rifiuto di attribuzioni oltre la capienza, transazione tutto-o-niente o righe esplicitamente escluse; *non creare una spesa da una riga di budget*. Evitare merge taciti.
- **Compatibilità:** futura migrazione SQLite puramente additiva e reversibilità documentale mediante export, archivi precedenti senza budget restano perfettamente interpretabili; nessun backfill dai movimenti di vendita o da spese in date di fiera. Gli snapshot storici di certificati, le vendite, gli incassi e le quote congelate restano invariati.
- **Backup:** dati e collegamenti vanno inclusi nella copia integrale dell'archivio; il CSV esportato fuori dall'archivio non è incluso automaticamente nel backup. Usare cartelle scelte dall'utente e nessuna sovrascrittura di file esterni. Nessuna dipendenza da endpoint remoti.
- **Privacy:** costi, fornitori e strategia commerciale sono dati riservati; non inserirli in dossier pubblici, certificati, report condivisi, MCP o PDF per terzi per default. File esterni in chiaro esplicitamente segnalati. Dati dei collezionisti non necessari per la prima fase e non acquisiti.

## 8. Dipendenze, alternative, complessità e rischi

**Dipendenze reali:** registro spese già presente; UX-009 opzionale ma, se realizzata, deve diventare **unica fonte** dei progetti. UX-032 (budget mensili) e UX-003 (costi per opera) sono iniziative distinte, **non prerequisiti**; TECH-001/025 possono fornire successivamente infrastruttura di scambio, senza doverne approvare i progetti.

**Complessità ipotetica:** bassa per un modello di foglio CSV esemplificativo e guida; **media** per budget per evento, link manuali e vincolo di capienza; **alta** se si includono più valute, vendite attribuite, commissioni, rimborsi, revisioni storiche e interoperabilità tra archivi. La fase minima *non* deve partire dai casi più complessi.

**Rischi:** confondere consuntivo con pagamenti verificati; spese non attribuite che rendono il dato incompleto; quote doppie; modifica o eliminazione di spese collegate; date sovrapposte; target rivisti dopo il fatto; falsi risultati di redditività; rivelazione di informazioni riservate; dashboard complesse per artisti che lavorano da soli. Mitigazioni: linguaggio non contabile, copertura esplicita, capienza controllata, link alla riga originale, conferme locali, anteprima con privacy by default e test con tastiera.

**Alternative da testare prima dello sviluppo:** foglio LibreOffice Calc offline con registro spese CSV esportato; singola nota di «costo previsto» nel futuro progetto UX-009; colonna `progetto` libera nel registro spese invece di modulo budget. Preferire quest'ultima solo se si dimostra sufficiente e non compromette analisi e portabilità. Se pochi utenti hanno più di una fiera/anno e non chiedono raffronti, non sviluppare.

## 9. Criteri di accettazione proposti (non test eseguiti)

1. Evento con target 1.000 € e spese collegate per 600 + 160 + 40 € mostra **800 € attribuiti**, **−200 € sotto budget**; nella pagina Economia il totale spese registrate resta **800 €**, invariato prima/dopo il collegamento.
2. Una spesa da 300 € attribuita per 180 € all'evento A consente al massimo 120 € all'evento B; tentativo di altri 121 € viene rifiutato con messaggio localizzato e nessuna scrittura parziale.
3. Una spesa fuori periodo può essere collegata solo con avviso leggibile; non è nascosta o attribuita automaticamente.
4. Budget assente, budget esplicito 0 € e budget positivo producono stati distinti e nessuna percentuale indefinita.
5. Riduzione di spesa già collegata genera conflitto da risolvere **senza** riscrivere la cifra originaria o moltiplicare le spese e senza presentare un totale inesatto come definitivo.
6. Una vendita in data compresa nella manifestazione **non** appare come ricavo della fiera senza successiva attribuzione manuale esplicita; ricevute e rate restano registri indipendenti.
7. Esportare e reimportare in un ambiente di prova preserva UUID, importi interi, versioni e legami, non duplica costi; una referenza mancante viene esplicitata prima di qualsiasi scrittura.
8. Backup verificato e ripristino preservano progetti e associazioni; archivi precedenti senza progetti restano leggibili; nessun CSV o altra integrazione riceve dati senza scelta.
9. UI navigabile solo con tastiera, zoom e screen reader, importi localizzati IT/EN/ES/FR/DE, locale non supportato → inglese; messaggi «previsto», «registrato» e «attribuito» distinguibili senza affidarsi al colore.
10. Nessuna chiamata di rete, login, canone, invio automatico o modifica a statistiche, certificati, ripartizioni di vendita e archivi esterni.

## 10. Fasi possibili — non sono lavori assegnati

- **Fase 0, scoperta:** chiedere a 5–8 artisti/gallerie una copia *anonimizzata* del proprio consuntivo di mostra/fiera; misurare frequenza, costo delle attribuzioni manuali, bisogno di confrontare preventivi. Provare una scheda Calc o semplice CSV senza modificare codice.
- **Fase 1, soltanto se validata:** budget minimo locale + collegamenti a spese esistenti con capienza, vista per evento e CSV scelto dall'utente; eventuale integrazione a UX-009 con una sola identità di progetto, mai nuovo modulo separato obbligatorio.
- **Fase 2, su nuova approvazione:** revisione storica dei budget, ripartizione più comoda di spese comuni, stampe condivisibili con occultamento dei dati sensibili.
- **Fase 3, solo con prova di necessità:** collegamenti manuali a vendite registrate e indicatore gestionale esplicitamente limitato; non introdurre automaticamente contabilità, imposte o dati bancari.

## 11. Requisiti certi, ipotesi e domande per la revisione personale

**Requisiti certi del progetto:** software desktop open source gratuito e Tauri, dati locali, offline-first, nessun account/abbonamento, backup su cartella libera, completa esportabilità, multilingua lingua OS con fallback inglese. In `main`: spese, vendite, incassi, ripartizioni e statistiche mensili sono distinti e già rappresentati. Le schede UX-009/032/003 e TECH-001/025 sono **proposte**, non funzioni approvate o necessariamente implementate.

**Ipotesi da validare:** le mostre e le fiere generano spese abbastanza numerose e trasversali ai mesi da giustificare il progetto; gli utenti accettano collegamenti manuali; i dati delle spese sono sufficientemente completi per un consuntivo; una vista economica per evento è più utile di un foglio di calcolo; l'identità di un progetto espositivo può essere riutilizzata senza sovrapporre scopi.

**Domande al proprietario, senza decisioni implicite:** privilegiare artista singolo o piccola galleria? Quanto spesso serve ripartire una spesa fra eventi? È indispensabile uno storico del budget modificato? La prima versione deve limitarsi ai **costi registrati** o servono anche **ricavi associati manualmente**? Come presentare un evento senza spese collegate? Sarebbe sufficiente un template Calc e un esempio nel manuale, senza nuova UI?

**Conclusione di ricerca:** proposta P2 condizionata a esigenze documentate. È ammissibile concludere **«non implementare»** se l'alternativa minima copre i casi d'uso. Nessuna approvazione o assegnazione è attribuita al proprietario.
