# REPS-UX-016 — Valutazioni documentate e coperture assicurative dichiarate delle opere

- **ID stabile:** REPS-UX-016
- **Data:** 2026-11-21
- **Stato:** Proposta — da valutare
- **Filone:** Gestione e UX
- **Priorità proposta:** P2, da rivalutare soltanto se prestiti, fiere e opere di valore elevato risultano frequenti tra gli utenti
- **Ambito:** studio ideativo/documentale; nessuna approvazione, implementazione o assegnazione
- **Avvertenza:** il gestionale non verifica la validità giuridica delle polizze, non eroga assicurazioni, non stima autonomamente il valore delle opere e non sostituisce consulenti, periti o assicuratori.

## 1. Problema e destinatari

Artisti, gallerie, archivi d'artista e collezioni private devono talvolta preparare un elenco di opere con **valori dichiarati**, documenti di stima, riferimenti di copertura assicurativa e date di validità, specialmente prima di una mostra, del trasporto o del rinnovo di una polizza. Oggi tali informazioni rischiano di rimanere in fogli separati, email o PDF esterni. Una polizza collettiva può coprire più opere, ma il suo massimale complessivo **non** corrisponde necessariamente a un importo garantito per ogni opera.

**Riscontro nel branch `main` al 21/11/2026:** `Artwork` contiene codice, titolo, misure, tecnica, ubicazione e stato, ma non storico di valutazioni, polizze o ambiti di copertura. `StudioSettings` comprende coefficiente d'artista e tariffa per cm²: la formula genera un **prezzo suggerito**, non una stima peritale o assicurativa ([economia](../../ECONOMY-I18N.md)). Vendite e incassi sono registri differenti dal valore attribuito a un'opera; un importo venduto non prova il valore assicurato. I backup sono completi e verificati, ma in chiaro; sei CSV correnti esportano registri differenti ([portabilità](../../DATA-PORTABILITY.md)).

**Bisogno da verificare, non certezza:** la domanda potrebbe essere sporadica per artisti con pochi lavori custoditi nello studio. Evitare quindi un modulo assicurativo complesso senza casi reali e valutare una semplice scheda per opera come prima alternativa.

## 2. Perimetro e confronto con idee già registrate

- [REPS-UX-001 — Provenienza e movimenti](REPS-UX-001-provenienza-movimentazioni.md): documenta **dove** si trova l'opera e gli eventi di custodia; qui si registra soltanto ciò che una fonte **dichiara in materia di valore/copertura**. Un trasferimento non attiva né termina automaticamente una polizza.
- [REPS-UX-004 — Rapporti di condizione](REPS-UX-004-rapporti-stato-conservazione.md): descrive lo **stato fisico** prima/dopo un evento; non determina risarcimenti, rischio assicurativo o autenticità.
- [REPS-UX-009 — Mostre e checklist](REPS-UX-009-progetti-espositivi-checklist.md): pianifica progetti; una casella «assicurazione verificata» non è una copertura effettiva. Eventuale collegamento futuro **facoltativo e solo informativo**.
- [REPS-UX-012 — Conto vendita](REPS-UX-012-affidamenti-conto-vendita-rendiconti.md): definisce condizioni commerciali e commissioni; non determina chi assicuri l'opera. Nessuna clausola viene dedotta dall'affidamento.
- [REPS-UX-003 — Costi per opera](REPS-UX-003-costo-e-margine-per-opera.md), [issue #13](https://github.com/Cesare-ml/Repertica-Studio/issues/13) e `finance.rs`: costi, premi eventualmente pagati e commissioni sono **movimenti economici** separati. Qui non si creano spese, sinistri, incassi, fondi o rate.
- [REPS-TECH-001 — Pacchetto dati aperto](https://github.com/Cesare-ml/Repertica-Studio/blob/docs/idee-strumenti-architettura-20261121/docs/idee/strumenti-e-architettura/REPS-TECH-001-pacchetto-dati-aperto.md) riguarda l'interoperabilità generale; [TECH-009 — Catalogo backup](https://github.com/Cesare-ml/Repertica-Studio/blob/docs/idee-strumenti-architettura-20261121/docs/idee/strumenti-e-architettura/REPS-TECH-009-catalogo-verifiche-conservazione-backup.md) riguarda le copie locali. Non ridefinire questi strumenti né modificare la cartella dell'altra automazione.

**Esclusioni intenzionali:** preventivi o confronto tra compagnie, assicurazioni sottoscrivibili dall'app, interpretazione automatica di clausole, gestione di sinistri e liquidazioni, perizie IA, aggiornamento automatico del mercato dell'arte, scadenziario di pagamento dei premi, invio email, monitoraggio di polizze online e consulenza legale/assicurativa.

## 3. Scenari d'uso

1. **Prestito per mostra:** un pittore seleziona tre opere e registra un valore dichiarato per ciascuna con fonte e data; annota un riferimento di polizza della galleria e il periodo indicato dal documento. La vista mostra **«copertura dichiarata — condizioni non verificate»**, mai «assicurata» senza riserve.
2. **Polizza di studio con massimale aggregato:** una galleria associa un documento a venti opere. L'interfaccia conserva il massimale generale **separato** dai valori delle singole opere: non somma limiti condivisi né ripartisce automaticamente il massimale.
3. **Rivalutazione:** un'opera precedentemente dichiarata a € 3.000 ha una nuova valutazione documentata a € 4.500. Entrambe restano consultabili con la data di riferimento e l'origine; nessuna modifica automatica al prezzo di listino, alla vendita o al coefficiente artista.
4. **Scadenza durante trasporto:** il riferimento di polizza ha una data di fine anteriore al rientro *previsto* da una mostra. Compare un avviso **«verificare i documenti»** basato sulle date annotate, non un giudizio di inesistenza della copertura.
5. **Vendita e conservazione storica:** l'opera viene venduta; lo storico delle valutazioni e delle attestazioni precedentemente inserite rimane, ma il software non presume che la polizza sia stata trasferita, cessata o prorogata.

## 4. Flussi UX ipotetici

**A — Opera → Valori e coperture.** Due schede visivamente distinte: «Valutazioni e fonti» e «Documenti di copertura». Intestazione contestuale con ultimo valore **documentato**, valuta, data e fonte; se manca la fonte, mostrare «valore dichiarato dall'utente, non perizia». Una terza riga «situazione assicurativa: non verificata» non deve avere un verde rassicurante per default.

**B — Aggiungi valutazione.** Scelta manuale di importo, valuta ISO 4217, data di riferimento, tipologia dichiarata (stima personale, richiesta d'assicurazione, perizia di terzi, altro), fonte e relativo documento opzionale. L'utente vede che il valore non modifica prezzi, margini, vendite o quote. Un precedente valore non si sovrascrive: si aggiunge una nuova dichiarazione o una rettifica motivata. Date storiche parziali richiederebbero precisione dichiarata, non date inventate.

**C — Registra riferimento di copertura.** Nome descrittivo, eventuale numero/riferimento, soggetto assicurato dichiarato, controparte facoltativa, intervallo di date dichiarato, ambito testuale (studio, deposito, transito, mostra), eventuale massimale aggregato, franchigia/esclusioni **come testo riportato**, file di riferimento. Un documento può essere associato esplicitamente a più opere; per ciascun legame si registra la fonte della selezione e l'eventuale valore indicato in polizza. Non dedurre la copertura dalla sola localizzazione o dal periodo.

**D — Riesamina e scadenze.** Vista riepilogativa «Da verificare» con documenti prossimi alla data fine, opere con valori vecchi secondo una soglia locale opzionale e prestiti con date potenzialmente non allineate **soltanto se UX-001/009 saranno approvate**. Nessun push di sistema, messaggio a compagnie o modifica automatica delle scadenze; avvisi interni visibili aprendo l'app. Se non è registrata una data fine, usare «non indicata», mai «illimitata».

**E — Condividi una distinta.** Selezione consapevole di opere, campi e destinatario d'uso («interno» / «distinta per organizzatore»). Anteprima riga per riga dei dati inclusi; per default nascondere numeri di polizza completi, indirizzi di deposito, recapiti, riferimenti bancari, prezzi di vendita, margini e allegati riservati. Stampa PDF locale o CSV; niente invio automatico.

**Accessibilità:** moduli navigabili da tastiera, label esplicite per valuta e tipo di importo, errori in testo, focus prevedibile, riepiloghi compatibili con screen reader, avvisi non espressi soltanto con colori, layout utilizzabile con zoom elevato e date lette nel formato locale.

## 5. Dati e relazioni proposti — ipotesi descrittive, NON schema approvato

- **`artwork_valuation`**: UUID, `artwork_id`, importo minore intero positivo/zero se esplicitamente ammesso, codice valuta, data di riferimento e precisione, categoria dichiarata, fonte/testimone facoltativo, data documento, riferimento a media locale opzionale, nota, autore **dichiarato** facoltativo, creato il e rettifica di UUID precedente. Non chiamare «perizia certificata» un record non verificato.
- **`coverage_document`**: UUID, etichetta, riferimenti privati, controparte/soggetto dichiarati, inizio/fine **riportati dal documento**, precisione date, ambiti testuali, massimale aggregato **facoltativo**, valuta, franchigia/esclusioni descritte, allegato locale opzionale, note, data di registrazione e indicatore di verifica manuale con autore dichiarato.
- **`coverage_artwork`**: collegamento univoco per documento/opera/versione, UUID opera, tipo di inclusione dichiarata, valore specificamente riportato **se presente**, valuta e fonte; assenza di importo per opera **non significa zero** né diritto a quota del massimale generale.
- **`coverage_revision`** (eventuale seconda fase): correzione documentata con riferimento alla registrazione precedente, motivazione e data. Conservarla distinta dai rapporti di conservazione e dalla cronologia editoriale [TECH-010](https://github.com/Cesare-ml/Repertica-Studio/blob/docs/idee-strumenti-architettura-20261121/docs/idee/strumenti-e-architettura/REPS-TECH-010-storico-revisioni-metadati.md).
- **Invarianti:** un record può dichiarare una copertura o un valore, non provarli; non calcolare cambi valuta con tassi inventati; valori storici mai convertiti silenziosamente; nessuna scrittura implicita su `Sale`, `Expense`, `Receipt`, `Certificate`, `Artwork.status` o `Artwork.location`. I certificati rimangono immutabili. Vincoli referenziali e gestione degli orfani da definire prima di un eventuale sviluppo.

## 6. Impostazioni, offline, multilingua, privacy

- **Impostazioni facoltative:** valuta predefinita (non conversione), intervallo locale di avviso scadenze (es. 30/60/90 giorni), soglia di revisione delle valutazioni, nascondi riferimenti privati nelle stampe; tutte modificabili senza account. Non impostare la copertura assicurativa come obbligatoria per catalogare o vendere un'opera.
- **Offline:** tutto in SQLite e allegati locali, senza connessioni a compagnie, servizi di quotazione, IA, telemetria o cloud. L'utente sceglie liberamente la cartella dei backup, incluso USB/NAS montato. Ogni documento/allegato gestito internamente deve essere incluso nei backup completi e verificati. I file esterni solo collegati non sono garantiti: preferire copia gestita o avviso esplicito.
- **Lingue:** lingua del sistema (IT/EN/ES/FR/DE ove supportate), override manuale e fallback inglese come [documentato](../../ECONOMY-I18N.md). Tradurre tutte le label, gli avvisi e i testi di stampa; descrizioni e clausole inserite dall'utente restano nella lingua originale. Codici tipo/valuta e date ISO nello scambio, formati locali in UI senza cambiare importi.
- **Privacy:** dati su opere di valore, ubicazioni e numeri delle polizze sono sensibili dal punto di vista della sicurezza patrimoniale anche quando non sono dati personali. Minimizzare dati di assicuratori, periti e soggetti assicurati; nessun tracking o ricerca online automatica; anteprima con omissioni predefinite nell'export per terzi. I backup attuali sono **in chiaro**, quindi la presenza di documenti riservati richiede avvertenza e archiviazione protetta; l'eventuale cifratura appartiene a TECH-006 e non è un prerequisito già disponibile.

## 7. Importazione, esportazione e compatibilità

- **Export:** CSV UTF-8 con record e UUID stabili per valutazioni/collegamenti; PDF stampabile con data di generazione, legenda **«dati dichiarati, da verificare sul documento originale»**, nessuna promessa di copertura. JSON versionato separato per eventuale round trip, con manifest degli allegati e percorsi relativi; il file non deve rivelare i path assoluti dell'archivio locale.
- **Import futuro:** file selezionato dal dispositivo, anteprima con rilevamento di valuta, UUID e duplicati, collegamento manuale a opera esistente, distinguere aggiornamento da nuovo documento, non creare polizze «attive» per deduzione. Import atomico con annullamento su conflitto o allegato mancante; non sostituire senza conferma dati esistenti o certificati. Per interoperabilità semantica coordinarsi concettualmente con TECH-001 **solo se approvata**.
- **Compatibilità:** aggiunta eventuale di entità senza alterare record e importi già esistenti; archivi antecedenti mostrano «nessun dato documentato», non «non assicurato». Backup precedenti restano apribili con migrazione sicura; se una versione vecchia dell'app non capisce il nuovo schema, avvertire anziché degradare i dati. Gestire correttamente importi in centesimi/minor units, limiti interi e codici di valuta; non usare `f64` per denaro.

## 8. Dipendenze, alternative, rischi e complessità

**Dipendenze certe nel prodotto attuale:** UUID delle opere, SQLite/migrazioni, gestione degli allegati, backup/ripristino verificati, stampa locale, localizzazione. **Possibili integrazioni non autorizzate:** UX-001 (movimenti), UX-009 (mostre), UX-014 (rubrica di professionisti), TECH-001 (formato aperto), TECH-006 (backup cifrati). Nessuna deve essere assunta già operativa.

**Alternativa minima:** solo campi «ultimo valore dichiarato», «fonte», «data», «riferimento documento», «scadenza da verificare» nella scheda opera, senza polizze collettive. Contro: duplicazione di una polizza comune e perdita dello storico; pro: UX e migrazioni meno onerose. Alternativa esterna: modello CSV stampabile conservato dall'utente, se casi d'uso troppo rari.

**Rischi principali:** falso senso di copertura; clausole interpretate dal programma; massimali aggregati sommati impropriamente; valori di mercato presentati come ufficiali; documenti personali esposti in un PDF; dati riservati nei backup non cifrati; complicazioni con valuta e polizze multilocali; schede diventate un sistema assicurativo specialistico. **Mitigazioni:** terminologia «dichiarato», fonti visibili e campi non verificati espliciti, nessun badge «copertura garantita», separazione fra valore e polizza, previsualizzazione export, documenti locali, nessuna decisione automatica.

**Complessità indicativa:** bassa-media per registrare valutazioni datate; media per polizze condivise, allegati, filtri e scadenze; medio-alta per revisioni/round trip e combinazione con movimenti/mostre. Non stimare tempi o assegnare lavoro prima dell'approvazione.

## 9. Criteri di accettazione ipotetici e prove

1. Una scheda opera senza dati visualizza «valore non documentato» e «copertura non documentata/da verificare», mai «valore €0» o «non assicurata» per inferenza.
2. Due valutazioni datate rimangono distinguibili; la più recente non sovrascrive la precedente né il prezzo suggerito dal coefficiente artista, la vendita o la tesoreria.
3. Una polizza registrata per venti opere conserva **un solo** massimale generale e gli eventuali singoli valori soltanto quando esplicitamente documentati; nessuna ripartizione implicita.
4. Una data di scadenza passata o mancante produce un **promemoria di verifica** oppure «data non disponibile», non un verdetto sulla copertura reale. Una vendita o un movimento non cambiano automaticamente documenti o stati assicurativi.
5. La distinta per terzi omette per default identificativi completi di polizza, indirizzi, note private, clienti e prezzi di vendita; anteprima e inclusioni manuali sono accessibili da tastiera.
6. Backup/ripristino su cartella scelta mantengono valutazioni, relazioni e documenti; verificare media mancanti, file danneggiati, nomi Unicode, volumi rimovibili e snapshot storici.
7. Export e successivo import di prova conservano UUID, valuta, importi esatti, date, fonti, relazioni e allegati; l'import duplicato non moltiplica record o sovrascrive certificati.
8. Prove senza rete, IT/EN/ES/FR/DE con lingua di sistema e fallback inglese, screen reader/zoom, archivi grandi e migrazione da uno schema precedente non compromettono i moduli correnti.

## 10. Fasi possibili — esclusivamente ipotesi di valutazione

- **Fase A — convalida del bisogno:** interviste a 3–5 artisti, galleristi e archivisti con opere in prestito; raccogliere esempi di **distinte anonimizzate** e criteri davvero utilizzati. Valutare se basti il modello CSV o una scheda minima.
- **Fase B — valutazioni datate:** solo fonti, valori dichiarati, documenti opzionali e vista per opera; nessun motore assicurativo. Sottoporre la UX al proprietario prima di estendere lo schema.
- **Fase C — riferimenti di polizze collettive:** collegamenti espliciti, periodo dichiarato, elenco da verificare, distinta con redazione dei dati; senza verifiche giuridiche o automazioni.
- **Fase D — interoperabilità e integrazioni opzionali:** round trip dati, collegamenti a mostre/movimenti solo se i rispettivi filoni saranno prima approvati, e avvisi nell'app aperta.

## 11. Requisiti certi, ipotesi e questioni per il proprietario

**Requisiti certi del progetto:** applicazione Tauri desktop gratuita/open source, locale/offline-first, senza account né abbonamento, dati sempre esportabili, backup in cartella liberamente scelta, multilingua con lingua OS e fallback inglese; prezzi suggeriti, vendite, spese e certificati esistenti rimangono distinti e integri.

**Ipotesi non validate:** frequenza di richieste di distinte assicurative; utilità di gestire polizze condivise; disponibilità dell'utente a conservare documenti sensibili; necessità di storicizzare valori dichiarati e di vedere scadenze durante la preparazione di mostre.

**Domande riservate alla revisione del proprietario:** (1) la prima utenza target è artista individuale o galleria/archivio con prestiti frequenti? (2) serve lo storico delle valutazioni oppure basta una data e una fonte? (3) si desidera memorizzare il documento di polizza o soltanto un riferimento testuale, considerato che i backup sono in chiaro? (4) esistono casi verificati di polizze collettive e valori per singola opera? (5) quali campi mostrare in una distinta per terzi senza esporre luoghi o rapporti economici? (6) la presenza di una polizza dovrebbe essere totalmente separata dai workflow di mostra e movimento anche se questi verranno approvati?

**Decisione:** **Proposta — da valutare**. Nessuna idea è approvata né assegnata allo sviluppo.
