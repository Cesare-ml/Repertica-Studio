# REPS-UX-044 — Attrezzature riutilizzabili dello studio: schede, verifiche e manutenzioni dichiarate

- **ID stabile:** REPS-UX-044
- **Data della ricognizione:** 2026-11-22
- **Stato:** **Proposta — da valutare**
- **Filone:** Gestione e UX
- **Priorità ipotetica:** **P2** per studi/laboratori con apparecchiature costose o soggette a manutenzione; **P3** per artisti che usano poche attrezzature o già le seguono agevolmente altrove
- **Natura:** **solo analisi documentale**. Non approva funzionalità, nuove tabelle, test, workflow o attività di sviluppo.

## 1. Bisogno professionale, destinatari e verifica sul repository

Un artista o laboratorio può registrare come spesa l'acquisto di un forno per ceramica, una fotocamera, una stampante fine-art o un compressore, ma l'importo non dice **quale esemplare fisico** sia in uso, dove si trovi, quale manuale o riferimento di assistenza riguardi e **quando sia stata effettivamente compiuta una verifica**. Questi beni durevoli non sono opere in catalogo e non si consumano come tele o pigmenti. La perdita di date e informazioni operative tra fogli, email e cartelle può essere costosa e ostacolare la continuità dello studio.

**Destinatari da intervistare:** artisti con un laboratorio attrezzato, studi di stampa/fotografia/ceramica, assistenti di produzione e piccole gallerie con propria attrezzatura di allestimento. Per un artista che usa solo strumenti ordinari e pochi beni numerati è ragionevole **non introdurre un altro modulo**.

**Fatti verificati staticamente su `main` il 22/11/2026:** il [README](../../../README.md) descrive registrazione di spese e categorie; il [modello Expense](../../../src/app/models.ts), [finance.rs](../../../src-tauri/src/finance.rs) e [schema SQLite](../../../src-tauri/src/db.rs) conservano data, categoria, descrizione, importo, fondo, fornitore, link e note di una **spesa**, senza scheda autonoma di bene strumentale o storico di manutenzione. La categoria frontend `equipment`/«Attrezzatura» è disponibile, ma **una categoria di spesa non è un inventario fisico**. [ECONOMY-I18N](../../ECONOMY-I18N.md) specifica che le statistiche sono gestionali, non contabilità fiscale. [DATA-PORTABILITY](../../DATA-PORTABILITY.md) documenta l'export delle spese ma non un registro beni strumentali. Nessun utilizzo reale o incidente è stato osservato, né è stata eseguita una prova desktop.

## 2. Confini e confronto con le proposte già esistenti

- [UX-008 — Scorte materiali](REPS-UX-008-scorte-materiali-consumi.md): segue **quantità disponibili e consumi** di materiali fungibili; qui si identifica un **singolo strumento durevole**, senza stock, giacenze o scarichi di materiale.
- [UX-003 — Costo per opera](REPS-UX-003-costo-e-margine-per-opera.md): attribuisce costi a specifiche opere; questa scheda **non calcola ammortamenti né ripartisce il prezzo di un macchinario** sui prodotti.
- [UX-037 — Scadenze ricorrenti](REPS-UX-037-scadenze-spese-ricorrenti-senza-registrazioni-presunte.md) e [UX-042 — Pagamenti acquisti](REPS-UX-042-pagamenti-dichiarati-spese-acquisti-differiti.md): la prima segue possibili spese **periodiche**, la seconda i **pagamenti dichiarati di un costo**. Una verifica periodica di funzionamento non crea scadenze di pagamento e un intervento effettuato non prova un addebito. Collegamenti eventuali devono restare espliciti e facoltativi.
- [UX-004 — Condizione delle opere](REPS-UX-004-rapporti-stato-conservazione.md), [UX-025 — Cura delle opere](REPS-UX-025-istruzioni-allestimento-cura-opera.md) e [UX-043 — Post-vendita](REPS-UX-043-segnalazioni-assistenza-post-vendita-opere.md): riguardano **opere d'arte o richieste dei collezionisti**, non dispositivi di proprietà/uso dello studio.
- [TECH-016 — Allegati documentali locali](https://github.com/Cesare-ml/Repertica-Studio/blob/docs/idee-strumenti-architettura-20261121/docs/idee/strumenti-e-architettura/REPS-TECH-016-allegati-documentali-locali-multi-entita.md): eventuale infrastruttura per manuali, ricevute o rapporti; **non** definisce il flusso professionale né è approvata. [TECH-001](https://github.com/Cesare-ml/Repertica-Studio/pull/16) tratta portabilità generale. Nessun file tecnico viene modificato.
- **Esclusi:** ERP, inventario fiscale dei cespiti, ammortamenti, registri di sicurezza regolamentati, certificazioni, telemetria/IoT, invio automatico al tecnico, manutenzione delle opere, controllo remoto dei macchinari, prestiti/trasferimenti di proprietà impliciti.

## 3. Scenari d'uso

1. **Forno ceramico:** uno studio annota marca, modello, numero seriale e collocazione interna; registra una verifica **dichiarata** del 15 novembre, il tecnico indicato e il documento di manutenzione. Un controllo successivo è una **data prevista**, non una verifica completata né una certificazione di sicurezza.
2. **Fotocamera per documentare opere:** un'artista conserva data di acquisto e riferimento al manuale locale. Quando sostituisce l'obiettivo, annota l'evento senza creare una nuova spesa, cambiare le fotografie del catalogo o dichiarare automaticamente il valore residuo.
3. **Stampante fine-art:** un laboratorio vuole vedere su quali apparecchiature manca una verifica registrata nell'ultimo periodo. L'app mostra «nessuna verifica documentata», **non** «apparecchiatura guasta» o «manutenzione obbligatoria scaduta» senza una regola dichiarata dall'utente.
4. **Strumento ceduto o dismesso:** lo stato passa a «non più in uso» con data e motivo. Si conservano seriale, storia ed eventuali documenti, senza eliminare spese originarie e senza dedurre automaticamente una vendita o un ricavo.
5. **Beni non propri:** un dispositivo in comodato/noleggio può avere un proprietario o custode **dichiarato**, ma registrarlo non certifica un titolo giuridico. L'eventuale reso non altera movimenti delle opere.

## 4. Flussi UX: prima una versione leggera

**Accesso facoltativo** da «Studio → Attrezzature» o «Economia → Attrezzature» (decisione aperta): lista di schede sintetiche con nome, tipo, collocazione, stato **dichiarato**, ultima verifica registrata e prossimo promemoria, se presente. Lo stato iniziale di un archivio preesistente è **«nessuna attrezzatura registrata»**, non «nessuna attrezzatura posseduta». Ricerca per nome, modello e identificativo; filtri per ubicazione, uso e verifiche da rivedere, senza inferire stato tecnico.

**Registra attrezzatura**: nome obbligatorio; tipologia libera o suggerita, marca/modello, seriale facoltativo, identificativo locale, ubicazione descrittiva, uso/proprietà dichiarati, data acquisizione eventualmente ignota, note. È possibile creare una scheda **senza spesa**: un macchinario regalato, già posseduto o preso in prestito non deve generare un costo.

**Dettaglio → Registra evento**: scegliere «verifica eseguita», «manutenzione riferita», «riparazione», «sostituzione componente» o «annotazione»; data effettiva o incerta, descrizione, soggetto/fonte della dichiarazione, eventuale documento scelto localmente, esito **testuale** e prossima verifica **ipotizzata separatamente**. Una voce può essere segnata «riferita» o «documentata» dall'utente; non usare etichette «sicura» o «certificata» come esiti generici. Correzioni sostanziali con rettifica datata, non sovrascrittura silenziosa.

**Programma una verifica (solo se utile)**: data prevista o frequenza manuale, senza notifiche a programma chiuso. Un pulsante «Registra verifica effettuata» apre una **nuova registrazione da confermare**; non completa la verifica al solo trascorrere della data. Distinguere «promemoria passato», «verifica registrata» e «nessun dato». Le eventuali notifiche del sistema operativo sono fuori prima fase e richiederebbero consenso specifico.

**Documenti e condivisione**: consultare manuali e attestazioni già salvati come file locali, non lanciare/eseguire file non attendibili automaticamente. Esportazione selettiva dopo anteprima; togliere per default seriali, ubicazioni interne, recapiti, cifre e note di sicurezza. Preservare disponibilità di stampa anche senza un nuovo modulo di documenti.

**Accessibilità:** azioni raggiungibili da tastiera; campi con etichette, feedback vicino all'errore e focus coerente; filtri e scadenze con **testo**, non solo colore; leggibilità con zoom 200%, screen reader e finestre piccole; data incerta non resa come zero; campi seriali copiabili senza dover leggere QR.

## 5. Dati e ipotesi di API — descrittivi, NON migrazione approvata

- **`studio_equipment` (ipotetica):** UUID, etichetta e ID locale opzionale, tipo testuale, marca/modello/seriale facoltativi, ubicazione descrittiva, proprietà/uso **dichiarati** (`owned`, `borrowed`, `rented`, `unknown`), stato operativo **dichiarato** (`in_use`, `out_of_use`, `under_review`, `unknown`), data acquisizione/precisione, note, `created_at`/`updated_at`, eventuale `archived_at`. Non obbligare un valore di acquisto o un seriale.
- **`equipment_event`:** UUID, equipment UUID, tipo stabile, data/precisione, testo, autore o tecnico **dichiarato**, fonte, livello di verifica della fonte, evento rettificato (UUID facoltativo), timestamp. La parola «manutenzione» nel tipo evento non prova che sia stata svolta correttamente.
- **`equipment_check_plan`** solo in fase successiva: equipment UUID, prossima data dichiarata o regola scelta, sospensione, nota; evitare serie generate che sembrino eventi realmente compiuti. Una previsione non è una spesa.
- **Collegamenti opzionali:** `expense_id` a una spesa esistente per acquisto/riparazione **solo su selezione esplicita**, con nessuna duplicazione di `amount_cents`. In assenza di collegamento il costo resta sconosciuto; collegare un costo non dimostra proprietà né pagamento.
- **Allegati:** riferimenti relativi a file copiati nel media store con metadati, checksum eventuale, fonte e visibilità. Studiarne l'eventuale riuso con TECH-016 **solo se approvata**; non inventare un'infrastruttura già esistente.
- **API concettuali (NON comandi Tauri):** «elenca/filtra attrezzature», «salva scheda», «registra rettifica evento», «mostra verifiche previste», «collega una spesa esistente», «anteprima esportazione». Nessun server, endpoint pubblico o mutazione MCP proposta.

## 6. Impostazioni, offline, lingue, backup, import/export e privacy

- **Preferenze opzionali:** modulo visibile/nascosto, categorie suggerite non obbligatorie, raggruppamento per luogo, eventuale finestra di avviso scelta dall'utente, convenzione data. Nessuna impostazione cambia inventario o spese automaticamente.
- **Offline e autonomia:** SQLite locale, media gestiti in `media/`, nessun login, cloud, abbonamento, rete, lettura sensori, invio email o telemetria. Un link a manuale esterno è solo testo; non promettere che il contenuto remoto sia disponibile offline.
- **Multilingua:** lingua iniziale del sistema, override IT/EN/ES/FR/DE e fallback inglese già definiti da [ECONOMY-I18N](../../ECONOMY-I18N.md). Tradurre messaggi e categorie di sistema; marchi, seriali, descrizioni e note restano **testi originali dell'utente**. Esportare codici stabili indipendenti dalla lingua; date ISO nei file, localizzate in UI.
- **Portabilità:** CSV UTF-8 con una riga per attrezzatura e CSV eventi separato, UUID/ID di collegamento, codici tipo stabili, campo `unknown` o vuoto distinto da «non disponibile»; **nessun** ricalcolo di euro o depreciamento. JSON versionato e manifest per round-trip documentale con allegati scelti in percorsi relativi; import da file selezionato, anteprima per UUID/seriali duplicati, risoluzione manuale conflitti e rollback intero. Il CSV attuale delle spese **non è un formato di importazione** e non deve essere alterato. Nessuna importazione automatica di attrezzature dalle descrizioni delle vecchie spese.
- **Backup/ripristino:** se mai implementata, l'informazione entra nel **backup completo verificato** SQLite + media su cartella liberamente scelta, anche disco/NAS, preservando compatibilità con gli archivi vecchi. Documenti esportati **fuori** dall'archivio non vengono magicamente inclusi. Non cancellare allegati storici non collegati senza decisione.
- **Privacy/sicurezza:** seriali, localizzazione interna e nomi di tecnici possono rivelare beni di valore, procedure e contatti. Vista export «solo dati condivisibili» con anteprima e selezione; nessuna pubblicazione o accesso MCP impliciti. Evitare di memorizzare password, codici allarme o credenziali IoT nel campo note; file importati trattati come non fidati, percorsi relativi verificati, limiti dimensione e validazione tipo. I backup esistenti sono in chiaro: avvertire della protezione del supporto fisico.

## 7. Dipendenze, complessità, rischi e alternative

- **Dipendenze minime:** UUID, SQLite e migrazione additiva, convenzioni i18n, backup/export locali, interfaccia opere/economia solo come riferimento. **Nessuna** dipendenza necessaria da cloud, TECH-016, UX-008 o UX-042. Gli allegati sono facoltativi e possono essere rinviati.
- **Complessità:** **bassa-media** per registro semplice e note; **media** per eventi/rettifiche, scadenze e collegamento manuale alle spese; **medio-alta** per allegati, export/import relazionale affidabile e test di compatibilità. Registro fiscale o integrazione con controlli industriali: **fuori ambito e alto rischio**.
- **Rischi:** dare l'impressione di attestare sicurezza, conformità o validità della garanzia; rendere un promemoria una manutenzione «eseguita»; trasformare una spesa in valore contabile del bene; esposizione di inventario sensibile; complicare il prodotto agli artisti che non ne hanno bisogno; duplicare scadenze in più pannelli.
- **Mitigazioni:** vocaboli espliciti («dichiarato», «da verificare», «registrato»), nessuna scadenza automatica di obblighi normativi, pannello facoltativo, fonte/precisione, minimo dato obbligatorio, collegamenti non mutanti, privacy-by-default.
- **Alternativa minima preferibile da verificare prima:** campo «Attrezzatura/identificativo» nelle note di spesa + documento in cartella locale e un CSV volontario. L'alternativa evita migrazioni ma non offre storico del singolo esemplare. Altra alternativa: calendario personale offline per verifiche; costo sviluppo nullo ma senza collegamento al bene.

## 8. Criteri di accettazione ipotetici e prove da richiedere **solo se approvato in futuro**

1. Un'apparecchiatura può esistere **senza** spesa collegata; la creazione non cambia costi, ripartizioni, statistiche mensili, coefficiente artista, vendite o certificati.
2. Due dispositivi dello stesso modello hanno UUID distinti anche se il seriale è assente; seriali uguali/errati producono **avviso**, non fusione o cancellazione automatica.
3. Un controllo programmato per domani resta previsto anche se oggi è passato l'ultimo controllo; non diventa mai «effettuato» senza registrazione e conferma.
4. Rettificare una manutenzione conserva la versione precedente e il motivo, senza qualificare il risultato come certificazione tecnica.
5. Collegare una spesa di 600 € a un dispositivo non crea una seconda spesa e non altera il totale delle statistiche del mese; cancellare/modificare la spesa segnala il collegamento da verificare, senza ricrearla.
6. L'esportazione selettiva non include per default seriali, ubicazioni sensibili, contatti o note private; un'esportazione integrale richiede scelta consapevole.
7. Backup/ripristino includono dati e file, conservano UUID e rettifiche; l'archivio storico si apre senza beni strumentali presunti. Round-trip JSON di prova non duplica dispositivi/eventi; input malevolo non evade dalla cartella selezionata.
8. Interfaccia leggibile da tastiera/screen reader, test zoom e almeno le cinque lingue (fallback inglese); selezione di data e stato distingue «ignoto» da «non presente».

## 9. Fasi possibili, **non assegnate**

- **Fase 0 — Validazione prima del software:** interviste a tre categorie di studi con esempi concreti di 5–20 attrezzature; misurare frequenza delle verifiche, perdita di tempo e sufficienza di semplici note. Se non emerge bisogno, **archiviare l'idea senza svilupparla**.
- **Fase A — Registro leggero:** schede fisiche con identificativi facoltativi, filtri, archivio storico e testo libero. Nessun costo o scadenza derivati.
- **Fase B — Eventi e promemoria dichiarati:** storico rettificabile e verifiche previste **solo dentro l'app**, collegamenti manuali e non distruttivi a spese esistenti.
- **Fase C — Documenti e scambio:** allegati locali, anteprima di condivisione, JSON versionato con import controllato, accessibilità e collaudi multipiattaforma per archivi grandi.

## 10. Requisiti certi, ipotesi e domande riservate al proprietario

**Certi dal repository/vincoli di prodotto:** Repertica è desktop open source e gratuito, Tauri/SQLite locale, senza account né abbonamento; gestisce spese, inclusa categoria attrezzatura, ma non un inventario di esemplari verificato nei file letti; backup in cartella liberamente scelta, CSV unidirezionali delle spese, multilingua da sistema con fallback inglese. Le proposte UX e TECH restano **non approvate**.

**Ipotesi da verificare:** studi di ceramica, stampa e fotografia hanno un numero di dispositivi e verifiche sufficiente a giustificare un nuovo registro; per alcuni basterebbe una rubrica con note; gli allegati possono arrivare molto dopo il registro minimo. Nessun problema d'uso è stato riprodotto né esistono interviste documentate in questa ricognizione.

**Domande per il proprietario, senza risposta presunta:** Repertica deve coprire anche i beni dello studio non destinati alla vendita, oppure rimanere focalizzata su opere e costi? Qual è il primo settore reale da servire (pittura, ceramica, stampa, galleria)? È desiderata una sola nota di manutenzione o uno storico con promemoria? Conviene lasciare tutto in uno strumento esterno e limitarsi a un link/nota? Come evitare duplicazioni se un domani esiste UX-037?

**Decisione:** nessuna approvazione, assegnazione o autorizzazione allo sviluppo. Scheda da revisione personale.
