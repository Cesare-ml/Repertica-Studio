# REPS-UX-030 — Quadratura facoltativa degli incassi e delle spese con estratti conto locali

- **ID stabile:** REPS-UX-030
- **Data:** 2026-11-21
- **Stato:** Proposta — da valutare
- **Filone:** Gestione e UX
- **Priorità ipotetica:** P2 (P3 se il bisogno reale è limitato); nessuna approvazione
- **Ambito:** analisi di una possibile UX futura, **non** requisito assegnato, implementazione o proposta di collegamento bancario online

## 1. Evidenza nel repository, problema e destinatari

**Riscontro statico su `main` (21/11/2026).** In [SALE-RECEIPTS](../../SALE-RECEIPTS.md), gli incassi di una vendita sono **dichiarati manualmente** e distinti da vendite, piani rateali e saldi. `src-tauri/src/receipts.rs` conserva per ogni incasso UUID, vendita, data, importo, metodo, riferimento e note; evita di superare il lordo venduto. [RECEIPT-ATTACHMENTS](../../RECEIPT-ATTACHMENTS.md) permette già un documento locale per incasso. `src-tauri/src/finance.rs` gestisce spese con data, categoria, fondo, fornitore e importo, ma **la data di spesa non attesta la data di addebito bancario**. [ECONOMY-I18N](../../ECONOMY-I18N.md) esclude al momento collegamenti bancari e riconciliazione. I [12 registri CSV](../../DATA-PORTABILITY.md) esportano registrazioni Repertica, **non** importano movimenti bancari.

**Problema da verificare con utenti.** Artisti con più acconti, pagamenti differiti e uscite possono trascrivere due volte lo stesso bonifico, omettere una rata realmente ricevuta o confondere il risultato economico mensile con il denaro effettivamente transitato sul conto. Attualmente possono controllare manualmente in due applicazioni: non è stato osservato un errore concreto né una richiesta documentata di collegare una banca.

**Destinatari:** artista con numerose vendite dilazionate, studio con spese ricorrenti, piccola galleria e amministrazione di archivio. Per un artista con poche operazioni, la complessità potrebbe superare il beneficio: **il flusso attuale deve rimanere il default**.

## 2. Confini e anti-doppioni

- **UX-024:** pianifica **scadenze/rate previste** (in parte già implementate su `main`); qui si confrontano **movimenti estratti da file scelti dall'utente** con registrazioni manuali già esistenti. Una rata dovuta **non** è una transazione avvenuta.
- **UX-027:** studia la corretta rappresentazione delle **valute**; la quadratura deve rifiutare corrispondenze fra valute diverse senza esplicito controvalore verificabile, non implementare i cambi.
- **UX-028:** documenta **giustificativi delle spese**, non il riscontro di un addebito; **UX-013** studia resi/rettifiche, non si possono dedurre automaticamente storni e rimborsi.
- **UX-003/012/017:** costi per opera, affidamenti e vendita multiopera restano domini diversi. **TECH-019** nella [PR tecnica #16](https://github.com/Cesare-ml/Repertica-Studio/pull/16) riguarda **importazione di cataloghi tabellari**, non l'interpretazione amministrativa di entrate/uscite; **TECH-025** riguarda **esportazioni minimizzate**. Nessuna modifica ai documenti tecnici.
- La proposta **non** crea un conto corrente integrato, un feed PSD2/Open Banking, una prima nota fiscale, riconciliazione automatica irrevocabile, fatturazione, certificazioni di pagamento o saldo bancario ufficiale. Non sostituisce il registro vendite/incassi/spese né il loro CSV integrale.

## 3. Scenari e casi limite

1. **Acconto già registrato.** Vendita di 1.200 EUR con incasso manuale di 400 EUR e saldo 800 EUR; in un CSV bancario compare un accredito di 400 EUR. L'utente collega le due righe: il saldo **rimane 800 EUR**, nessun secondo incasso, nessuna nuova vendita.
2. **Acconto non registrato.** Compare un accredito di 300 EUR senza incasso Repertica corrispondente. Il software lo mostra come **«da esaminare»**, non dichiara pagata alcuna vendita. Solo l'azione separata e confermata «Registra incasso per questa vendita» potrebbe usare i valori proposti, sottostando ai vincoli già esistenti; anche dopo la creazione la quadratura richiede conferma del collegamento.
3. **Spesa già registrata.** Materiali 48 EUR il 12/11, carta addebitata il 14/11. Collegare il movimento non cambia la data economica della spesa né incrementa il totale mensile. Una data distinta di addebito resta **dichiarata dal file**, non certificata.
4. **Estratto sovrapposto.** L'utente importa due volte un periodo che include il medesimo movimento: compare un potenziale duplicato con motivazione, senza duplicare i match confermati. Non usare data+importo da soli come identificatore certo: due bonifici da 500 EUR nello stesso giorno sono possibili.
5. **Una riga bancaria / più registrazioni.** Un accredito di 900 EUR può coprire 2 rate o vendite diverse, o essere netto di commissioni. In fase iniziale compare come **«non rappresentabile in corrispondenza 1:1»**; le ripartizioni n:m richiedono progetto separato, con somma verificabile e autorizzazione.
6. **Addebiti non pertinenti e storni.** Spese personali, commissioni bancarie, rimborsi e addebiti di clienti **non** diventano spese, ricavi o vendite senza azione esplicita. Un addebito successivo non cancella automaticamente un incasso storico.

## 4. Flussi UX descrittivi

**A — Economia → Controlla movimenti (facoltativo).** Spiegazione iniziale chiara: «I dati importati servono solo al confronto; non aggiornano incassi e spese». Selezione nativa di un **CSV locale** dell'utente; nessuna ricerca in cartelle private, connessione a banca, sincronizzazione o credenziali. Si può scegliere di non conservare il file origine nell'archivio.

**B — Anteprima import.** Selezionare intestazioni per data contabile, importo, segno dare/avere o colonne separate, valuta, identificativo della banca (se presente), descrizione. Anteprima di 20 righe con **data interpretata**, segno, valuta, separatore decimale; errori riga per riga. Non accettare ambiguità tipo `1.234,56` senza formato confermato. Imporre una scelta manuale del perimetro (periodo/account logico anonimo) e conferma del numero di righe. La prima fase evita automaticamente l'archiviazione di nomi/IBAN/causali completi.

**C — Vista affiancata.** Colonna sinistra «Movimenti da confrontare», destra «Incassi Repertica» o «Spese Repertica». Suggerimenti **solo non vincolanti** per stesso importo/valuta e finestra data configurabile; mostrare i criteri di suggerimento e segnalare collisioni. Testo visibile `Suggerito / Confermato manualmente / Non corrispondente / Da verificare`; mai «pagamento verificato dalla banca» come attestazione dell'app.

**D — Conferma e annullamento.** L'utente seleziona 1 movimento e 1 registrazione, rilegge importo e identificativi e conferma. Un match è revocabile con motivo; la revoca **non cancella** né modifica incasso/spesa. Un match già presente non può essere associato silenziosamente ad altro movimento. Se l'utente desidera inserire un incasso mancante, apre il form esistente con valori **solo precompilati**, conferma distinta e controllo del residuo. Nessuna creazione diretta dai suggerimenti.

**E — Rapporto di quadratura.** Periodo, numero di movimenti, confrontati/non confrontati, incongruenze (es. differenza di importo, date o valute) e lista in sola lettura. Il totale mostrato è **somma dei movimenti selezionati**, mai saldo conto ufficiale; nessuna fusione con il margine gestionale. Stampa/esporta solo dopo anteprima dei dati sensibili.

**Accessibilità:** completa interazione da tastiera, tabella semantica, selezioni testuali oltre al colore, annunci degli errori e dei collegamenti, mantenimento focus dopo conferme, zoom elevato e navigazione riga per riga senza drag-and-drop.

## 5. Dati minimi ipotetici (solo descrittivi, non migrazione richiesta)

- **`statement_import_session`:** UUID, data import, periodo dichiarato, etichetta privata dell'origine (non IBAN completo), valuta, numero righe importate/scartate, formato interpretato, fingerprint del file opzionale, stato `preview/confirmed/archived`. Conservazione del file sorgente **non obbligatoria**; se abilitata, copia interna protetta con limiti e backup.
- **`statement_entry`:** UUID interno, sessione, identificativo esterno se disponibile, data contabilizzazione, importo **intero in unità minime**, valuta ISO 4217, direzione accredito/addebito, descrizione **ridotta e opt-in**, digest stabile con versione e insieme dei campi usati, importato il; conservare il segno senza invertirlo implicitamente.
- **`reconciliation_link`:** UUID, `statement_entry_id`, tipo destinazione `sale_receipt` o `expense`, relativo UUID esistente, importo associato, stato confermato/revocato, date, nota/causa della rettifica. In prima fase **solo 1:1**: prevenire match doppi. Non creare copie dei dati autorevoli finanziari.
- Regole descrittive: import e linking atomici o annullabili; referenze valide, date ISO non ambigue, centesimi senza float, controlli sulle corrispondenze, rifiuto di valute differenti, nessun collegamento a rate future come prova di incasso. Se viene modificato/eliminato un incasso o una spesa già collegati, l'interfaccia segnala la quadratura da rivedere **prima** dell'azione; la cronologia dei legami non può presentarsi come tuttora valida.
- I movimenti importati sono **dichiarazioni contenute in un file fornito dall'utente**, potenzialmente manipolabile: la quadratura non certifica l'origine o l'effettività del pagamento.

## 6. Impostazioni, offline, privacy, sicurezza e lingue

- **Impostazioni opzionali:** abilita la sezione; finestra suggerimenti (es. ±3 giorni, configurabile); etichetta origine; politica minima di conservazione delle righe importate e cancellazione dei dati sensibili; scelta se memorizzare il documento originale. Mai chiavi bancarie, autorizzazione permanente, token, account o servizio obbligatorio.
- **Offline sempre:** parsing, confronto e conferma solo sul dispositivo; dialoghi nativi per file, nessuna rete, polling bancario, notifiche a terzi o analisi IA remota.
- **Privacy by default:** estratti conto possono includere **IBAN, nominativi, causali private, altri clienti e operazioni personali**. Anteprima con possibilità di escludere righe; minimizzazione al bisogno, no valori in log/errori, niente esposizione al server MCP per impostazione predefinita, niente preview in telemetria. Esportazione esterna con campi sensibili esclusi di default, anteprima e conferma. Eliminazione di sessioni non collegate e pseudonimizzazione da studiare senza compromettere gli audit.
- **Lingue:** lingua OS con override già supportato (IT/EN/ES/FR/DE), fallback inglese; tradurre etichette, avvisi, regole import e stati. **Le causali/nomi dell'utente non vengono tradotti.** Memorizzare importi/date in formato canonico e separare locale di visualizzazione dal locale di parsing del CSV, dichiarato nell'anteprima.
- **Compatibilità:** eventuale migrazione **additiva** SQLite, vuota per gli archivi storici; l'attuale UX di incassi/spese, i dodici CSV integrali e i backup verificati devono funzionare senza usare questa funzione. Restore di archivi più recenti richiede una versione compatibile come già previsto. Non archiviare riferimenti esterni che diventino necessari per aprire i dati.

## 7. Import/export e collaborazione senza lock-in

- **Import:** partire da CSV bancario manualmente esportato; *non esiste un CSV bancario universale*. Profili di colonne solo locali e facoltativi, valida prima **tutte** le righe, segnala duplicati per file/fingerprint e, ove presente, identificativo transazione; permette annullare senza scritture parziali.
- **Output di quadratura:** CSV UTF-8 e/o JSON aperto versionato con UUID interni, valuta, data, importi, stato e collegamenti; prima dell'esportazione scegliere se includere causali/campi di controparte. L'output non deve far passare un semplice collegamento manuale per ricevuta bancaria autenticata.
- **Backup/restore:** sessioni e match conservati in SQLite/media locale, inclusi nel backup verificato in qualunque cartella scelta. Import ripetuto o restore non moltiplicano i collegamenti. File originali esterni non indispensabili; se conservati come allegato, devono essere inclusi nella copia verificata.
- **Evoluzione da valutare, non promessa:** parser **camt.053 XML (ISO 20022)** in fase successiva, solo se arrivano estratti campione reali e si può validare versione/profilo XML; evitare parser XML permissivi, entità esterne e attese di omogeneità fra banche. Riferimenti generali: [ISO 20022](https://www.iso20022.org/) e [descrizione di camt.053](https://www.iotafinance.com/en/SWIFT-ISO20022-Message-camt-053-001-Bank-To-Customer-Statement.html).

## 8. Complessità, dipendenze, rischi, alternative

- **Complessità:** media-alta per CSV eterogenei, duplicati e interazione guidata; **alta** per matching n:m, importi multi-valuta, storni, rettifiche e conservazione responsabile di documenti bancari.
- **Dipendenze possibili:** UUID `sale_receipts`/`expenses`, vincoli e saldi esistenti, SQLite, backup, UX localizzata, dialogo nativo; TECH-019 solo come possibile riuso futuro del componente di mappatura CSV, **senza dipendenza obbligatoria** o richiesta di lavoro a quell'altro filone.
- **Rischi:** falso senso di «pagato/verificato», doppio conteggio degli incassi, match errati di importi uguali, leak di informazioni bancarie, confusione fra competenza e cassa, formati CSV non standard, reversali/commissioni, ricalcolo errato in valute diverse, grandi estratti duplicati.
- **Alternative:** (A) semplice vista con elenco incassi e spese e spunta manuale `controllato sul conto il…` **senza importare alcun estratto**; minor rischio privacy ma nessun confronto riga per riga; (B) mantenere il flusso esistente con riferimento e allegato incasso; (C) procedura manuale con CSV `receipts` esportato e confronto in foglio di calcolo locale. Scegliere in base al volume reale delle operazioni, non introdurre un modulo pesante per tutti.

## 9. Criteri di accettazione ipotetici

1. Importando un accredito di **400 EUR** e collegandolo all'incasso già registrato di 400 EUR per una vendita da 1.200 EUR, il saldo resta **800 EUR**; il numero degli incassi e le statistiche non cambiano.
2. Un nuovo accredito non collegato **non** produce un incasso o una spesa; qualsiasi registrazione richiede il relativo form e conferma separata.
3. Il doppio import dello stesso file segnala i possibili duplicati e non crea match aggiuntivi; **due accrediti identici nello stesso giorno** non vengono forzati a uno.
4. File con separatori, codifiche, formato numerico o segni ambigui chiedono chiarimento; una riga in USD non viene abbinata a una voce EUR.
5. Una spesa registrata il giorno 12 e addebitata il 14 conserva data, importo, fondo e statistiche mensili originari.
6. Modificando o eliminando un record collegato, i match non rimangono falsamente «validi»; la rimozione del collegamento non cancella il record economico.
7. File bancari non sono inviati a servizi esterni, non finiscono nei log né nel server MCP di default; backup/ripristino e export locale consentono portabilità senza file sorgente originale.
8. UX leggibile e interamente utilizzabile con tastiera/screen reader, lingua OS/fallback inglese, errore esplicito per import incompleto; test desktop reali su Windows/macOS/Linux.
9. Un rendiconto può elencare **voci confrontate/non confrontate** ma non etichetta come «certificati dalla banca» i match effettuati localmente.

## 10. Fasi eventuali e decisioni riservate al proprietario

- **Fase 0 — verifica bisogno:** interviste a 3–5 studi/gallerie; raccogliere CSV campione **anonimizzati** da almeno due banche/strumenti, con consenso; misurare numero di righe, errori effettivi e tempo del controllo manuale. Possibile esito legittimo: **non sviluppare**.
- **Fase A — soluzione minima:** vista di controllo manuale delle registrazioni esistenti, senza file bancari; confrontare utilità e onere privacy.
- **Fase B — solo se giustificata:** import CSV locale in anteprima, collegamenti espliciti 1:1, report e minimizzazione.
- **Fase C — su nuova valutazione:** raggruppamenti n:m, XML camt.053, report avanzati e regole di conservazione, senza cloud bancario.

**Requisiti certi:** prodotto gratuito/open source, Tauri offline-first, dati locali sempre esportabili, backup a cartella scelta, lingua sistema/fallback inglese; vendite, incassi e spese esistenti non si riscrivono in automatico.

**Ipotesi:** un campione di artisti/gallerie riconcilia abbastanza operazioni da apprezzare un secondo livello di controllo; i CSV bancari sono disponibili e non troppo disomogenei.

**Domande per il proprietario:** (1) Sono frequenti gli acconti tramite bonifico? (2) Vuoi gestire solo incassi vendita oppure anche gli addebiti delle spese? (3) Quanti movimenti annui giustificano l'import CSV? (4) È preferibile la minima spunta «controllato» senza memorizzare dati bancari? (5) Devono essere escluse in ogni caso le causali integrali e i dati di conto?

**Decisione attuale:** **Proposta — da valutare**. Nessuna idea approvata o assegnata allo sviluppo.
