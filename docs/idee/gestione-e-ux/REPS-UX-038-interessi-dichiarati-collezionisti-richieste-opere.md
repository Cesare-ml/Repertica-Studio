# REPS-UX-038 — Interessi dichiarati dei collezionisti e richieste di opere simili

- **ID stabile:** REPS-UX-038
- **Data:** 2026-11-22
- **Stato:** Proposta — da valutare
- **Filone:** Gestione e UX
- **Priorità proposta:** P2 condizionata; P3 se bastano note ricercabili. Non approvata né assegnata.
- **Natura:** esclusivamente studio documentale; non autorizza implementazioni, invii, campagne, issue o decisioni di prodotto.

## 1. Evidenze nel repository e bisogno da convalidare

Nel [MVP su main](../../MVP.md), la scheda collezionista (schema v8) contiene recapiti, note, fotografie e collegamenti commerciali, ma dichiara esplicitamente che gli **«interessi» per le opere del Gestionale Opere non sono stati portati**. Il [modello frontend](../../../src/app/models.ts) espone Collector e CollectorInput senza entità strutturate per una richiesta o un interesse verso un'opera/categoria; le [vendite](../../../src-tauri/src/commands.rs) e gli [incassi](../../SALE-RECEIPTS.md) rappresentano fatti diversi. Questa è una **lacuna documentata di copertura**, non un bug verificato e non una richiesta utenti misurata.

Un collezionista può dire: «Mi piace questa opera, ma è già venduta; vorrei vedere lavori simili su carta»; oppure «Se realizzi una nuova opera di formato piccolo, fammi sapere». Conservare solo una nota generica può rendere difficile ritrovare una richiesta precisa mesi dopo. Viceversa, classificare un acquisto passato come preferenza attuale sarebbe un'inferenza non giustificata.

**Destinatari ipotetici:** artisti che vendono direttamente e hanno interlocutori ricorrenti, piccoli studi e gallerie. Per archivi puramente storico-documentali la funzione può essere superflua.

**Riscontro esterno, non validazione Repertica:** [Art Galleria — Contacts & Relationships](https://www.artgalleria.com/product/contact-management) descrive interessi per artista, medium e fascia di prezzo; [ArtPlacer — Contact management](https://www.artplacer.com/contact-management/) lega contatti, note e interazioni commerciali. Queste esperienze indicano un caso d'uso di settore, non provano che sia necessario un CRM completo, IA, segmentazione automatica o servizi cloud per Repertica.

## 2. Confini e confronto anti-duplicazione

- **[UX-002 — Trattative e prenotazioni](REPS-UX-002-trattative-prenotazioni.md):** riguarda una **negoziazione concreta** su un'opera, con offerta/prenotazione e possibile passaggio a vendita. Un interesse generico o una richiesta «se in futuro crei qualcosa di simile» non è un'offerta né una riserva. Non si devono duplicare le trattative.
- **[UX-014 — Rubrica professionale](REPS-UX-014-rubrica-relazioni-professionali.md):** definisce identità, ruoli e cronologia dei contatti. Qui si approfondisce soltanto il **contenuto di una richiesta artistica dichiarata**, non una nuova anagrafica o timeline parallela; un eventuale collegamento futuro deve mantenere una singola fonte autorevole.
- **[UX-035 — Preferenze di contatto](REPS-UX-035-preferenze-contatto-comunicazioni-individuali.md):** «interessato a dipinti» **non equivale** a «acconsente a ricevere email promozionali». Nessun consenso è inferito, nessuna comunicazione è consentita automaticamente.
- **[UX-006 — Incarichi su richiesta](REPS-UX-006-incarichi-opere-su-richiesta.md):** un desiderio informale non è un incarico accettato, un preventivo, una commissione artistica o una promessa di consegna.
- **[UX-023 — Prezzi richiesti](REPS-UX-023-prezzi-richiesti-listini-storico.md):** un eventuale intervallo di spesa dichiarato dalla persona non determina il prezzo di listino o di vendita.
- **[TECH-030 — Vocabolari locali](https://github.com/Cesare-ml/Repertica-Studio/blob/docs/idee-strumenti-architettura-20261121/docs/idee/strumenti-e-architettura/REPS-TECH-030-vocabolari-locali-tecniche-supporti-interoperabili.md):** si occupa della semantica di tecniche/supporti, non della volontà espressa da un contatto. In fase iniziale bastano termini liberi; nessuna dipendenza obbligatoria dal vocabolario.
- **[TECH-001/019/025](https://github.com/Cesare-ml/Repertica-Studio/pull/16):** formati aperti, import e minimizzazione di esportazioni sono responsabilità trasversali del filone tecnico, che qui viene solo richiamato, non modificato.

## 3. Casi d'uso e regole UX

1. **Interesse per una specifica opera già venduta.** Da scheda opera o collezionista si annota «interesse dichiarato» con data, riferimento all'opera e frase sintetica. Lo stato «venduta» resta invariato; non si prenota né promette una replica.
2. **Desiderio per opere future.** Un collezionista indica «opere su carta, circa 20–40 cm» con eventuale intervallo di prezzo indicativo *nella valuta originale dichiarata*. Può essere conservato senza collegarlo a un'opera esistente, e senza registrare una trattativa.
3. **Richiesta cambiata o non più attuale.** Il contatto precisa che non cerca più una certa tipologia. L'utente sospende/chiude manualmente la richiesta, annotando la fonte; non deduce la perdita di interesse da mancata risposta.
4. **Nuova opera catalogata.** L'artista può consultare manualmente gli interessi ancora attivi e decidere se una richiesta sembra pertinente; nessuna graduatoria probabilistica, profilazione da acquisti, invio o suggerimento commerciale automatico.
5. **Dati ambigui o condivisi.** Due contatti con lo stesso nome non vengono uniti né collegati a un interesse senza scelta esplicita dell'utente; l'assenza di preferenza di contatto resta «non indicata».

### Flusso minimo consigliato

**Collezionista → Interessi dichiarati → Aggiungi:** tipo «opera specifica» oppure «caratteristiche desiderate», descrizione libera breve, data (eventualmente sconosciuta), **fonte dichiarata** (conversazione, email riferita manualmente, incontro, altro), eventuale opera di riferimento; etichetta visibile «annotazione dell'utente, non richiesta verificata». Nessun valore tecnico o budget obbligatorio.

**Consultazione:** elenco «attivi / da chiarire / chiusi» nella scheda collezionista; filtro globale **solo se utile** «interessi attivi per tecnica, supporto, parola chiave o opera». Il collegamento a un'opera venduta non altera le sue condizioni di vendita. Un eventuale pulsante «Apri trattativa» precompila soltanto il riferimento e richiede salvataggio/conferma separata.

**Rettifica:** modifica o chiusura con conferma; per cambi sostanziali, eventuale piccola cronologia con data e origine senza duplicare testi sensibili. Se l'opera referenziata viene eliminata nei limiti consentiti dall'archivio, il sistema non deve eliminare silenziosamente il testo della richiesta; mostrare un riferimento non più risolvibile o bloccare la cancellazione secondo politica da definire.

**Accessibilità:** form interamente navigabile da tastiera, etichette esplicite per «dichiarato», «annotato», «sconosciuto» e «chiuso», errori associati ai campi, focus preservato dopo salvataggio, filtri usabili con screen reader; nessuno stato comprensibile dal solo colore.

## 4. Dati necessari e interfacce descrittive — NON schema approvato

Entità ipotetica **collector_interest**:
- UUID, UUID collezionista e riferimento opera UUID **opzionale** (nessuna opera fittizia);
- descrizione testuale originale, eventuale tipo dichiarato (opera specifica/caratteristiche generali), tecnica/supporto testuali facoltativi, misure indicative non obbligatorie, intervallo economico **facoltativo e non normalizzato automaticamente** (valuta esplicita), stato «attivo / da chiarire / chiuso»;
- data della dichiarazione con precisione esplicita o assente, fonte minima e livello di certezza («riportato dall'utente / dichiarato dalla persona»), note private facoltative, data di inserimento/aggiornamento;
- eventuale data di chiusura e motivo, con storico minimo solo se necessario. Non conservare per impostazione predefinita messaggi integrali, dati sensibili, acquisti inferiti, punteggi o tracce di consultazione.

Possibili operazioni concettuali: elenca interessi per collezionista; aggiungi/modifica/chiudi con validazione dell'UUID; cerca *solo nei dati dichiarati* e filtra stati; esporta selettivamente. Nessuna API remota o modifica al server MCP proposta; per principio gli interessi **non** sarebbero esposti via MCP di default. I record delle vendite, dei certificati, delle offerte e delle spese restano fonti separate.

## 5. Impostazioni, offline, lingue, privacy e portabilità

- **Configurazione:** pannello disattivabile/nascosto, nessuna compilazione obbligatoria dei vecchi collezionisti, facoltativo promemoria «verificare se interesse ancora attuale» visualizzato *solo aprendo l'app*, non una notifica automatica. Non inferire preferenze dai dati già registrati.
- **Offline e piattaforme:** SQLite/archivio locale Windows/macOS/Linux, nessun account, canone, rete, sincronizzazione, controllo di disponibilità online, IA o import automatico dalla posta. Le comunicazioni restano sempre esterne e deliberatamente attivate dall'utente.
- **Multilingua:** lingua di sistema, override IT/EN/ES/FR/DE e fallback inglese già documentati in [ECONOMY-I18N](../../ECONOMY-I18N.md); codici dei tipi/stati indipendenti dalla traduzione; testo libero e descrizioni originali *mai tradotti automaticamente*. Date visualizzate secondo locale, esportate in forma stabile.
- **Privacy:** può trattarsi di dati personali su gusti, disponibilità di spesa e conversazioni. Minimizzazione e rettifica/cancellazione governata dalle relazioni; nessun ranking, segmentazione occulta o marketing; nessuna divulgazione in brochure, cataloghi, Media Center o esportazioni pubbliche. Un interesse **non** è consenso per campagne o inviti. L'anteprima dell'export mostra esattamente nomi, note e richieste da condividere; per default è escluso dai dossier esterni. I backup attuali **non sono cifrati**: usare prudenza con cartelle condivise.
- **Export:** CSV UTF-8 consultivo, con UUID del contatto, codice stato, riferimento opera, testo e date; opzione esplicita per omettere nomi, note personali o budget. Per *round trip* futuro, JSON versionato con ID stabili, null espliciti e relazioni verificate; nessuna trasformazione silenziosa di EUR o di termini locali.
- **Import:** solo file scelto dall'utente, anteprima, validazione contatti/opere, controllo duplicati per UUID, rollback e gestione dei riferimenti mancanti; nessun contatto, offerta o vendita creati automaticamente. Il backup/ripristino integrale a cartella liberamente scelta include i nuovi dati locali se mai implementati, senza mutare il formato attuale senza migrazione verificata.

## 6. Compatibilità, complessità, dipendenze, rischi e alternative

- **Compatibilità:** schema additivo con dati preesistenti invariati e vecchi interessi assenti (non falsamente «nessun interesse»). Collegamenti stabili ai collector UUID; qualora UX-014 introducesse una rubrica trasversale, documentare una migrazione esplicita senza doppio archivio persone. Gli snapshot dei certificati restano immutabili. Versioni vecchie dell'app potrebbero non leggere uno schema futuro: prevedere avvisi, rollback di migrazione e backup.
- **Dipendenze possibili:** anagrafiche già esistenti, UI locale/filtri, i18n, backup, esportazione; UX-002/014/035 e TECH-030 sono collegamenti concettuali **non prerequisiti**. Nessun SDK CRM, provider email o libreria IA.
- **Complessità:** bassa-media per registro locale e ricerca per parola/stato; media per importazione relazionale e cancellazione/rettifica coerente; alta e **sconsigliata** per deduzioni automatiche, segmentazione, campagne o scoring.
- **Rischi:** trattare un desiderio come prenotazione o contratto; ritenere attuali richieste obsolete; dedurre potere di spesa, profilazione indebita; inviare messaggi indesiderati; duplicare cronologie o preferenze; esportare informazioni personali; termini liberi non confrontabili.
- **Mitigazioni:** fonte e data facoltative ma visibili, stato «da chiarire», chiusura manuale, limiti chiari, dati non inferiti, separazione UX-035, esportazione minima, anteprima completa.
- **Alternativa preferita da validare:** campo note esistente con etichetta «Interessi dichiarati» e filtro locale sulle note, senza alcuna nuova tabella. Alternativa ancora più semplice: un foglio CSV personale separato. Il costo di un modulo dedicato è giustificabile solo se più richieste per persona, scadenze o ricerca trasversale emergono dall'uso reale.

## 7. Criteri di accettazione ipotetici

1. Registrare «vorrei un'opera simile» per un'opera venduta non crea vendita, riserva, commissione, incasso o comunicazione.
2. Una richiesta generale senza opera, budget, email o data può esistere come descrizione e stato «da chiarire»; non si inventano valori mancanti.
3. Il filtro sugli interessi usa soltanto voci dichiarate/manuali, non ricostruisce gusti dagli acquisti e distingue «non registrato» da «nessun interesse».
4. Chiudere un interesse non cancella o cambia il collezionista, l'opera, lo storico vendite o i certificati; eventuali collegamenti mancanti sono segnalati.
5. Nessun destinatario è contattato senza un'azione esplicita e distinta; «interessato» non viene trasformato in «autorizzato a ricevere promozioni».
6. Export consultivo minimizzato, import di prova senza duplicati, backup/ripristino in cartella scelta conservano record e collegamenti; offline completo senza account o rete.
7. Flussi leggibili con tastiera e screen reader, testo multilingue nella UI con fallback inglese, dati Unicode preservati su Windows/macOS/Linux.

## 8. Fasi di valutazione, non lavori approvati

- **Fase 0 — Validazione:** mostrare note + ricerca a 3–5 artisti o galleristi che hanno richieste ripetute; misurare quante richieste per collezionista e quante ricerche trasversali fanno realmente.
- **Fase A — Minimo eventuale:** interessi dichiarati testuali e stati, collegamento opzionale a opera esistente, nessuna automazione.
- **Fase B — Consultazione:** filtri, rettifiche e collegamenti **manuali** a trattative se UX-002 fosse approvata, senza automatismi.
- **Fase C — Portabilità:** export/import versionato e test su grandi archivi, con privacy e compatibilità verificate.

## 9. Requisiti certi, ipotesi e questioni riservate al proprietario

- **Certi dal repo/vincoli:** collezionisti e vendite locali, campi note, interessi non portati dal Gestionale Opere, SQLite, backup a cartella scelta, i18n di sistema e fallback inglese; nessun account necessario.
- **Ipotesi non validate:** richieste ricorrenti dei collezionisti, valore di uno stato di interesse autonomo, utilità della ricerca per tecnica/misura e del riferimento a una opera venduta.
- **Domande aperte:** bastano note ricercabili? L'interesse riguarda soprattutto un'opera precisa o un genere/tecnica? Serve associare interessi a persone che non hanno mai comprato (implicherebbe UX-014)? Che durata tipica hanno le richieste? Quali dati economici sono davvero indispensabili, e quale rischio privacy è accettabile? È utile una consultazione manuale trasversale o verrebbe percepita come marketing?

**Decisione:** nessuna approvazione, assegnazione o avvio dello sviluppo. La proposta resta «Proposta — da valutare».