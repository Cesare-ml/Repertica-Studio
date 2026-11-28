# REPS-UX-027 — Vendite in valuta estera: importi originari, incassi e statistiche senza conversioni implicite

- **ID stabile:** REPS-UX-027
- **Data:** 2026-11-21
- **Stato:** Proposta — da valutare
- **Filone:** Gestione e UX
- **Priorità ipotetica:** P1–P2 solo per artisti e gallerie che operano realmente in più valute; P3 per studi esclusivamente EUR
- **Perimetro:** ricerca e specifica funzionale **non approvata**. Nessuna decisione di prodotto, incarico di sviluppo, implementazione, modifica del codice, issue operativa o merge.

## 1. Problema professionale, destinatari e verifica su `main`

Un artista italiano espone negli Stati Uniti e vende un'opera in USD; una galleria acquista materiali in GBP e registra costi in EUR. Scrivere semplicemente «2200» in un campo che il programma interpreta come **2200 EUR** altera il significato economico della vendita; un'equivalenza approssimativa fatta mentalmente perde invece la valuta concordata e la documentazione dell'eventuale cambio. **Valore di vendita, incasso dichiarato e valore convertito per un report sono tre fatti differenti.**

**Utenti potenziali:** artisti con clienti internazionali, gallerie attive in più paesi, studi che acquistano materiali esteri, archivi con rendiconti storici in valute diverse. Per chi lavora soltanto in EUR la funzionalità deve essere invisibile o disattivabile, senza aggravare i flussi attuali.

**Riscontri statici del 21/11/2026, branch `main`:**

- [`src-tauri/src/models.rs`](../../../src-tauri/src/models.rs) espone `Sale.currency`, ma `SaleInput` non contiene una scelta di valuta.
- [`src-tauri/src/commands.rs`](../../../src-tauri/src/commands.rs) inserisce le vendite con `currency='EUR'` in almeno due percorsi; `src-tauri/src/finance.rs` descrive gli importi come **centesimi EUR**, e i riepiloghi mensili sommano gli importi senza distinguere valute.
- [`docs/ECONOMY-I18N.md`](../../ECONOMY-I18N.md) documenta la tesoreria in EUR e separa vendite registrate da incassi bancari; [`docs/DATA-PORTABILITY.md`](../../DATA-PORTABILITY.md) descrive dodici CSV, con valuta sulle vendite/incassi/saldi ma non un contratto multivaluta per tutti i registri; le spese sono `amount_cents` senza valuta autonoma.
- [`docs/MVP.md`](../../MVP.md) segnala la **gestione multivaluta** fra le funzioni ancora mancanti. Qui si approfondiscono significato, errori da evitare e UX; **non** si dichiara un bug o un requisito già autorizzato.

**Domanda ancora da dimostrare:** quante vendite e spese reali degli utenti Repertica sono in valuta diversa da EUR? Nessuna intervista, benchmark o analisi di archivi d'uso è stata svolta in questa ricognizione. Potrebbe bastare, per alcuni, la registrazione dell'equivalente EUR nelle note; occorre verificare se così si perde informazione indispensabile.

## 2. Confini e anti-doppioni

- **[UX-023 — Prezzi richiesti e listini](REPS-UX-023-prezzi-richiesti-listini-storico.md)** conserva il prezzo *proposto* per un'opera, distinto dalla vendita finale. REPS-UX-027 tratta la **denominazione monetaria degli importi registrati** in vendite, incassi, spese e report: non crea listini, non traduce automaticamente il prezzo suggerito del coefficiente artista e non cambia il prezzo richiesto. Un possibile listino multi-valuta resta decisione separata.
- **[UX-012 — Conto vendita](REPS-UX-012-affidamenti-conto-vendita-rendiconti.md)** riguarda termini e commissioni degli intermediari; qui nessuna nuova commissione o riconciliazione di mandati. **[UX-017 — Vendite multiopera](REPS-UX-017-vendita-multiopera-ripartizione-importi.md)** riguarda l'unità di un accordo con più opere: se in futuro esistono entrambi, scegliere una valuta esplicita per accordo, senza somme inter-valuta.
- **[UX-003 — Costi per opera](REPS-UX-003-costo-e-margine-per-opera.md)** attribuisce costi e margini; questa proposta non inventa nuovi costi, ma impedisce di sommare importi monetari di valute differenti. Le quote di ripartizione già presenti su `main` **non** sono tassi di cambio.
- **[TECH-001 — Pacchetto aperto](https://github.com/Cesare-ml/Repertica-Studio/blob/docs/idee-strumenti-architettura-20261121/docs/idee/strumenti-e-architettura/REPS-TECH-001-pacchetto-dati-aperto.md)**, **[TECH-018 — Esportazioni CSV](https://github.com/Cesare-ml/Repertica-Studio/blob/docs/idee-strumenti-architettura-20261121/docs/idee/strumenti-e-architettura/REPS-TECH-018-esportazioni-csv-scalabili-atomiche.md)** e **[TECH-019 — Importazione guidata](https://github.com/Cesare-ml/Repertica-Studio/blob/docs/idee-strumenti-architettura-20261121/docs/idee/strumenti-e-architettura/REPS-TECH-019-importazione-guidata-cataloghi-tabellari.md)** definiscono infrastruttura e compatibilità. Questa scheda definisce **regole di dominio e flussi dell'utente**, senza modificarne i file.
- Fuori ambito: conto bancario, acquisizione automatica dei cambi, speculazione valutaria, feed online obbligatori, invio di pagamenti, riconciliazione bancaria, fatture, imposte, IVA multigiurisdizione e conversione silenziosa di importi storici.

## 3. Casi d'uso da validare

1. **Vendita internazionale.** Galleria registra una vendita concordata a **USD 2.200,00**. La scheda mostra sempre l'originale, il cliente e la data. Nessun prezzo equivalente EUR viene inventato e la vendita non entra nel totale «€» come se fosse euro.
2. **Acconto e saldo nella stessa valuta.** Per quella vendita l'operatore dichiara due incassi rispettivamente di **USD 800,00** e **USD 1.400,00**. Il residuo passa da 2.200 a 1.400 e poi a zero **in USD**; ciò non attesta né verifica gli effettivi movimenti bancari.
3. **Acquisto materiale all'estero.** Spesa documentata in **GBP 145,50**. Il filtro mensile può mostrare un subtotale GBP distinto dal totale EUR, mai un margine aggregato senza conversione dichiarata.
4. **Archivio storico EUR.** Dopo un eventuale aggiornamento, tutte le vendite e spese precedenti restano importi EUR identici, senza ricalcolo, perdita di UUID, modifiche di certificati o nuove richieste di dati obbligatorie.
5. **Valuta senza centesimi.** Una spesa in JPY è inserita come valore intero (es. JPY 12.500). Non va interpretata come 125,00 né si devono mostrare due decimali fittizi.
6. **Rendiconto gestionale.** Il professionista sceglie se vedere un prospetto per valuta (sempre possibile offline) oppure un **equivalente stimato EUR** solo se ha dichiarato manualmente un cambio e la sua fonte; le statistiche originarie rimangono immutate.
7. **Incasso in valuta diversa da quella di vendita.** Caso da tenere **fuori dalla prima fase**: un cliente deve USD ma paga EUR con conversione del prestatore; senza valore effettivamente accreditato, cambio applicato e commissioni bancarie non è possibile dedurre automaticamente il saldo economico.

## 4. Flussi UX proposti

**Vendita → Registra vendita.** Il percorso ordinario resta in EUR e invariato per impostazione predefinita. Se la funzione viene attivata, una tendina «Valuta dell'accordo» propone EUR e un elenco di codici ISO 4217 supportati; la valuta è sempre visibile **accanto** al numero, anche nell'anteprima e nel dettaglio. Scelta facoltativa, mai inferita da lingua di sistema, paese del cliente o simbolo «$». Cambio di valuta in fase di digitazione *non converte* l'importo: azzera la bozza o richiede conferma esplicita e nuova immissione, senza dare l'impressione che 2.200 EUR siano diventati 2.200 USD in equivalenza.

**Incassi → Registra incasso.** In fase iniziale eredita la valuta **della vendita** e la mostra come testo non ambiguo. Convalida che la somma degli incassi dichiarati non superi il lordo nella medesima unità monetaria, senza arrotondamenti float. Un incasso non implica «pagamento accertato». La sequenza rateale, se presente, resta nella valuta della vendita. Ogni conferma mostra importo, codice, data e nuovo residuo.

**Spese → Nuova spesa.** Se e solo se la funzionalità viene estesa ai costi, scelta separata della valuta della spesa e subtotalizzazione dedicata. Non nascondere una spesa GBP in un riepilogo EUR; se non è possibile un confronto, mostrare «Non convertito», non zero. Non cambiare retroattivamente la categoria o il fondo della spesa.

**Economia → Riepiloghi mensili.** Vista predefinita: righe e totali *per valuta* con etichette esplicite. Totali su più valute non vengono sommati. Le quote percentuali delle vendite si calcolano **nella valuta di origine**; un totale globale per quota può essere mostrato solo separato per valuta o con conversione consapevole documentata. Se una UI non è ancora in grado di rendere statistiche multivaluta corrette, la fase iniziale deve limitarsi a una **scheda di studio/ricognizione**, non consentire vendite non-EUR in produzione.

**Conversione eventuale (non fase 1).** Azione «Aggiungi cambio dichiarato» con valore, data di riferimento, direzione del rapporto (es. EUR per 1 USD), provenienza («valore contrattuale», «rendiconto intermediario», «stima personale») e nota. Mostrare anteprima «importo originale» e «equivalente informativo». Eventuali cambi di riferimento non sono tassi garantiti né somme effettivamente accreditate; modifiche richiedono nuova revisione, non sovrascrivono la scelta usata in un report storico.

**Accessibilità.** Etichette leggibili dagli screen reader («Valuta USD, dollaro statunitense»), campi numerici con precisione corretta, feedback scritto se una valuta non è supportata; navigazione completa tastiera, messaggi senza dipendere dal colore, tabella con intestazioni di valuta ripetute, anteprima stampabile e zoom 200%. Evitare che il solo simbolo ¥, $ o kr identifichi una moneta.

## 5. Dati e regole di calcolo — ipotesi descrittive, non schema approvato

- **Importo monetario concettuale:** `{ amount_minor: integer, currency_code: ISO4217, minor_unit_scale: integer }`; usare l'unità minima **della valuta registrata** (non chiamarla genericamente `cents`), con scala e arrotondamento deterministici. Non utilizzare `f32`/`f64` per transazioni o cambi congelati. Preferire tabelle di scala versionate e validate localmente; conservare l'unità usata nello storico se uno standard cambia.
- **Vendita:** UUID esistente, `currency` esplicita, importo originale, data, opera e collezionista invariati. `sale_finances` e quote storiche devono usare stessa valuta per tutte le sottrazioni/allocazioni, senza trattare USD come EUR.
- **Incasso e rate:** legame stabile alla vendita; nella prima fase stessa valuta; calcolo del residuo sempre nella valuta della vendita. Confronto e limiti transazionali in unità minime intere. Vietata la compensazione fra valute senza una registrazione di cambio approvata.
- **Spesa:** valuta propria, importo originario e data, eventuali attribuzioni mantenute; un report misto non presume EUR.
- **Cambio dichiarato (fase futura):** UUID, valuta sorgente/destinazione, rappresentazione razionale o decimale esatta del rapporto, direzione, data di efficacia, fonte, data di registrazione, versione della regola di arrotondamento, valore convertito e riferimento all'operazione/report. **Non** sostituire il valore originale; non ricalcolare report precedenti a ogni avvio.
- **Visualizzazioni e export:** ogni importo conserva `currency_code` e scala espliciti; valori originari e «equivalenti informativi» in colonne diverse. Assenza di cambio = `NULL` e messaggio «Nessun equivalente calcolato», mai `0`.
- **Invarianti:** somma quote = netto **nella stessa valuta**; somma rate = lordo **nella stessa valuta**; incassi <= lordo nella stessa valuta; nessuna conversione implicita; nessuna deduzione su trasferimento bancario, tasse o commissioni. Le vendite storiche EUR non vengono mutuate in altre valute.

## 6. Impostazioni, offline, multilingua, privacy

- **Impostazioni:** abilita valute aggiuntive (facoltativo); valuta predefinita per nuove vendite (EUR per archivi attuali); lista locale delle valute consentite; eventuale valuta di *presentazione* dei report, distinta dalla valuta contrattuale. Nessuna preferenza di locale cambia un importo storico; tornare a modalità EUR non nasconde né modifica eventuali record non-EUR.
- **Offline-first assoluto:** codici, regole e conversioni dichiarate dall'utente sono nel DB locale, inclusi backup verificati a **cartella liberamente scelta**. Nessuna connessione a banca, provider di tassi, cloud, abbonamento o account. Un futuro recupero automatico di tassi, se mai considerato, richiederebbe una proposta diversa e consenso esplicito: non è autorizzato qui.
- **Lingue:** lingua del sistema, selezione IT/EN/ES/FR/DE e fallback inglese. Localizzare nomi di valuta, separatori decimali, errori, stampa e testi di avviso, **non** i codici ISO né i numeri storici. Non tradurre automaticamente note o fonti dell'utente. Distinguere formato visualizzato (locale) e memorizzazione (numeri interi + codice).
- **Privacy e condivisione:** clienti, importi, cambi e note contrattuali restano locali. Anteprima e selezione esplicita delle colonne prima di un prospetto esterno; non condividere automaticamente indirizzi, recapiti, note private o dati delle altre vendite. I backup attuali sono in chiaro: ricordarlo quando l'utente sceglie cartelle USB/NAS.
- **Compatibilità:** Windows, macOS, Linux; nessun controllo dipendente da connessione Internet o configurazioni regionali OS. Non assumere che ogni moneta abbia esattamente due cifre decimali.

## 7. Import/export, migrazione, dipendenze e alternative

**Export:** preservare gli attuali **12 registri CSV** e i formati esistenti per gli archivi EUR. Un eventuale formato di nuovo tipo/versione deve indicare esplicitamente `currency_code`, `amount_minor` e precisione, senza scrivere `amount_cents=220000` per una valuta con scala non 2 e fingere compatibilità con lettori che sommano euro. Per report di confronto includere in colonne distinte originale/cambio/equivalente con fonte; data ISO, UUID e separatore tabellare documentati. Non far dipendere l'esportazione integrale da filtri. L'importazione di valute e cambi richiederebbe anteprima, validazioni, scelta di mappatura e rollback, coordinata eventualmente con TECH-001/019; **non** si deduce automaticamente una valuta da simboli non univoci.

**Backup/ripristino:** i dati monetari e gli eventuali riferimenti restano in SQLite e nelle copie complete con manifest. Un vecchio backup EUR si apre con EUR invariato. Versioni precedenti dell'app incapaci di interpretare un archivio più nuovo devono fermarsi in sicurezza; mai conversione distruttiva dei record per forzare compatibilità.

**Dipendenze possibili:** schema SQLite e migrazioni versionate; economia/ripartizioni; incassi e rate, CSV, i18n e test numerici multipiattaforma. L'anagrafica commerciale UX-014 e i prezzi UX-023 non sono prerequisiti. Standard orientativo: [ISO 4217 — codici e unità minori](https://www.iso.org/iso-4217-currency-codes.html); i [tassi di riferimento BCE](https://www.ecb.europa.eu/stats/policy_and_exchange_rates/euro_reference_exchange_rates/html/index.en.html) sono **informativi**, non tassi automaticamente applicabili ai singoli pagamenti. Nessuna dipendenza remota.

**Alternative da valutare prima di progettare:** (A) rimanere EUR e annotare la valuta originaria nelle note (zero complessità, ma scarsa tracciabilità); (B) affiancare *solo un campo informativo* «prezzo originale in valuta» lasciando EUR autorevole (meno invasivo ma possibili divergenze); (C) supportare valuta contrattuale per singola vendita e riepiloghi separati (più corretto, più impegnativo); (D) supportare tutte le conversioni e pagamenti cross-currency subito (sconsigliato, alto rischio). **Preferire una validazione professionale dell'alternativa B prima di C.**

## 8. Rischi, complessità e criteri di accettazione

**Rischi:** confondere USD con EUR; errori di fattore 100 o di scala JPY/KWD; retroconversioni sui record preesistenti; report mensili apparentemente corretti ma matematicamente falsi; tassi non tracciati, percentuali su basi di valuta diversa; esportazioni lette erroneamente da vecchi fogli; aumentare i passaggi per chi lavora solo in EUR. Un cambio manuale non prova il costo o accredito applicato dall'intermediario.

**Complessità:** *alta* per un supporto multi-valuta pienamente coerente su vendite, rate, incassi, spese, quote, filtri, report, CSV, backup/migrazioni e retrocompatibilità; *bassa-media* per una prima ricognizione informativa dei prezzi in valuta con valore EUR distinto e dichiarato. Nessuna libreria, servizio o stima di durata approvati.

**Criteri verificabili (per un'eventuale futura decisione):**

1. In un archivio esistente ogni importo EUR, UUID, ripartizione e certificato rimane identico dopo migrazione/backup/ripristino; i flussi EUR non acquistano passaggi obbligatori.
2. USD 2.200 con due incassi USD 800 e 1.400 produce saldo USD 0; nessun totale mensile EUR aumenta di 2.200.
3. JPY 12.500 viene registrato e presentato come numero intero; test anche su valute a tre unità decimali e sui limiti di intero.
4. Quote, imposte e spese sono sommate **solo entro la medesima valuta**; report misti mostrano subtotali e avvisi, mai un totale inventato.
5. Nessun tasso di cambio viene richiesto o scaricato automaticamente; se assente, il report dice «Non convertito» e non mostra un importo EUR equivalente.
6. Ogni esportazione distingue unità, valuta, eventuale cambio dichiarato e fonte; file CSV storici EUR restano interpretabili dai consumer documentati o si offre un percorso esplicito di migrazione del formato.
7. Errore e annullamento non modificano record, saldi, rate, certificati o backup. Test numerici e desktop su Windows, macOS e Linux, con locale IT/EN/ES/FR/DE e fallback inglese.
8. Un archivio con vendite non-EUR non viene aperto e reinterpretato da una versione precedente; backup completo sempre ripristinabile da cartella scelta da una versione compatibile.

## 9. Fasi ipotetiche e domande al proprietario

- **Fase 0 — Validazione senza sviluppo:** interviste a 4–6 artisti/galleristi che fatturano o pagano all'estero; raccolta anonima di **casi** (non contratti o dati personali reali); mappatura dei report e dei CSV dipendenti dall'assunzione EUR. Domanda: è importante conoscere l'originale oppure la contabilità di studio può restare EUR?
- **Fase 1 — Opzione minima:** solo annotazione strutturata **informativa** della valuta/importo originario, se sufficiente. Nessuna seconda fonte di ricavi né conversione automatica.
- **Fase 2 — Solo previa approvazione esplicita:** valuta nativa per vendita, rate e incassi nella stessa valuta e riepiloghi per valuta con dati storici preservati.
- **Fase 3 — Solo se richiesta da professionisti:** spese multi-valuta, cambi manuali con fonte e storico, equivalenti informativi e integrazione selettiva nei report.
- **Fase 4 — Eventuale ricerca separata:** incassi in valuta diversa dall'accordo, commissioni intermediarie e cambi effettivi, tenendo distinta la contabilità fiscale.

**Questioni aperte per il proprietario (non risposte/approvate da questa scheda):** quale percentuale degli utenti vende fuori dall'eurozona? È prioritario registrare l'importo originario o analizzare redditività tra valute? È necessario supportare JPY/valute con tre decimali o basta EUR/USD/GBP/CHF? Serve una valuta di rendiconto predefinita? Quanto è tollerabile cambiare il formato CSV? Chi decide e documenta un cambio manuale? Quali vincoli legali/fiscali vanno esplicitamente lasciati fuori? Quale fase, eventualmente, meriterebbe un prototipo reversibile?

**Esito:** singola proposta descrittiva, **«Proposta — da valutare»**, senza implementazione, approvazione, assegnazione o modifica al prodotto.
