# REPS-UX-020 — Etichette inventariali e identificativi scansionabili, senza collegamenti online

- **ID stabile:** REPS-UX-020
- **Data della ricognizione:** 2026-11-21
- **Stato:** Proposta — da valutare
- **Filone:** Gestione e UX
- **Priorità proposta:** P1–P2 per studi, depositi e piccole gallerie con molte opere; P3 per archivi piccoli. Priorità **non approvata**.
- **Natura:** esclusivamente ricerca, analisi e specifica concettuale. Non è una richiesta di implementazione né un incarico di sviluppo.

## 1. Esigenza reale, destinatari e base verificata

Un professionista che conserva decine o centinaia di opere deve ritrovare la **scheda corretta davanti all'oggetto fisico**, anche quando titoli simili, studi preparatori o imballaggi impediscono di identificarlo a colpo d'occhio. Scrivere a mano un codice sulla scatola e poi cercarlo in archivio è possibile, ma può comportare errori di trascrizione, ristampe incoerenti e confusione fra opere simili. Un'etichetta inventariale leggibile da persone e scanner può collegare fisicamente un'opera alla sua **identità di catalogo**, senza rivelare dati personali.

**Verifiche del 21/11/2026 sul branch `main`:** [README](../../../README.md), [MVP](../../MVP.md), [modelli TypeScript](../../../src/app/models.ts), [schema SQLite](../../../src-tauri/src/db.rs) e [portabilità](../../DATA-PORTABILITY.md). Ogni opera possiede già UUID stabile, codice e titolo; l'app offre ricerca/filtri e una cartella dati locale. Nei flussi e nei modelli consultati **non è documentata** una funzione per creare lotti di etichette inventariali o risolvere un codice scansionato verso la scheda opera. Questo è un **possibile miglioramento del flusso**, non un errore di Repertica. I codici visibili sono modificabili, mentre l'UUID è l'identità del record: la distinzione è decisiva.

**Utenti principali:** artista con deposito e tele imballate; galleria con scaffali, casse e opere affidate; archivio che deve eseguire censimenti. **Utenti secondari:** conservatori e assistenti che effettuano verifiche fisiche senza accesso a una connessione. Per chi gestisce dieci opere, un campo codice e la normale ricerca possono bastare.

## 2. Confini e coordinamento: cosa NON viene duplicato

| Funzionalità o proposta | Confine di questa scheda |
| --- | --- |
| [UX-001 — Provenienza e movimenti](REPS-UX-001-provenienza-movimentazioni.md) | Un'etichetta **identifica** il record; la sua stampa o scansione non prova né registra spostamenti, consegne, custodia o provenienza. |
| [UX-015 — Ricognizioni inventariali](REPS-UX-015-ricognizioni-inventariali-discrepanze.md) | UX-015 progetta **sessioni di verifica e relative dichiarazioni**. Qui si approfondiscono **supporto fisico, leggibilità e percorso di ricerca**. Un futuro collegamento alla ricognizione sarà possibile solo su approvazione separata e mai con esito «presente» automatico. |
| [UX-009 — Mostre](REPS-UX-009-progetti-espositivi-checklist.md) e [UX-007 — Dossier](REPS-UX-007-dossier-presentazione-opere.md) | La targhetta **interna inventariale** non è una didascalia per il pubblico, un catalogo di mostra o un dossier da condividere. |
| [UX-005 — Edizioni e multipli](REPS-UX-005-edizioni-multipli-serie-numerate.md) | In caso di futura gestione di esemplari, ogni **oggetto fisico** richiederebbe identità distinta. Non presumere che un codice di edizione identifichi una copia singola. |
| [TECH-008 — Filtri](https://github.com/Cesare-ml/Repertica-Studio/pull/16), [TECH-001 — Interscambio](https://github.com/Cesare-ml/Repertica-Studio/pull/16), [TECH-011 — Multiarchivio](https://github.com/Cesare-ml/Repertica-Studio/pull/16) | La scansione effettua una **ricerca esatta nell'archivio attivo**, non crea una nuova ricerca federata, un protocollo di interoperabilità né una scelta automatica dell'archivio. Nessun file del filone tecnico è modificato. |

**Esclusioni:** RFID/NFC, geolocalizzazione, sensori, tracciamento in tempo reale, etichette antifalsificazione, certificati di autenticità, firme elettroniche, applicazione automatica di movimenti, acquisti, vendite, stati commerciali o esiti di inventario. Un QR identificativo **non autentica** un'opera: può essere copiato o spostato su un altro oggetto.

## 3. Scenari d'uso e benefici da verificare

1. **Deposito di 120 opere.** L'artista seleziona le opere da un elenco, visualizza un'anteprima delle etichette prima della stampa, stampa su A4 o salva in PDF, applica i cartellini su custodie e scaffali. Il foglio contiene identificativi distinti anche in presenza di titoli uguali.
2. **Cambio del titolo o del codice descrittivo.** La scheda viene aggiornata; la vecchia etichetta conserva un QR che punta allo stesso **UUID locale**. La lettura mostra il titolo/codice corrente e, se diverso da quello stampato, avvisa che l'etichetta potrebbe essere datata e offre una ristampa manuale.
3. **Verifica fisica.** Un assistente usa uno scanner USB che digita il testo come tastiera, oppure inserisce il codice a mano. La scheda corretta appare, **senza registrare automaticamente** una presenza o una movimentazione. Se in futuro UX-015 venisse approvata, l'utente dovrebbe comunque scegliere un esito e confermarlo.
4. **Cartellino illeggibile o errato.** L'utente può cercare il codice stampato; se il contenuto è sconosciuto, malformato o riguarda un altro archivio, Repertica mostra «nessuna corrispondenza verificata», non crea un'opera né seleziona un risultato somigliante.
5. **Consegna a una galleria.** Il professionista può usare un layout minimo che mostri solo codice interno e identificativo, senza nome del collezionista, prezzo, indirizzo dello studio o ubicazione riservata. Il destinatario esterno **non** riceve accesso ai dati per il solo possesso dell'etichetta.

## 4. Flussi UX proposti

### A. Preparazione e stampa

- Da **Opere → Seleziona → Etichette inventariali**, scegliere una o più opere. Un riepilogo indica numero di opere, codici mancanti/duplicati apparenti, fogli stimati e possibili collisioni grafiche. Non confondere la selezione con l'esportazione CSV completa.
- **Anteprima obbligatoria** con opzioni esplicite: codice leggibile, QR facoltativo, titolo opzionale, eventualmente anno. **Predefinito prudente:** codice e identificativo scansionabile, *senza* titoli, prezzi, contatti, indirizzi, ubicazioni, costi o note private.
- Formati iniziali: pagina A4 e stampa/PDF attraverso il **dialogo nativo o di sistema**, con misure in millimetri, margini e verifica dell'area realmente stampabile; non inviare job automaticamente alla stampante. Layout adatto anche al bianco e nero. La carta adesiva pretagliata richiede modelli e calibrazione specifici, quindi **non va promessa** nella prima fase.
- Stampare un solo esemplare di prova, con indicazioni per verificare leggibilità del codice a distanza e dopo la stampa; per una ristampa scegliere consapevolmente se mostrare il nuovo codice/titolo attuale. Anteprima visibile prima di ogni operazione esterna.
- Non stampare su, né incollare direttamente a, superfici delicate o originali: l'etichetta è destinata a **custodie, supporti reversibili o cartellini scelti dal professionista**, senza suggerire procedure conservative.

### B. Identificazione tramite scanner o tastiera

- **Opere → Identifica etichetta**: campo esplicito «Scansiona o incolla identificativo» con focus controllabile; funzionamento anche senza hardware. Uno scanner USB HID può emulare una tastiera, ma non assumere che tutti gli scanner siano compatibili senza test.
- Accettare solo un formato strutturato conosciuto, ad esempio `RS1:<uuid>` (schema **puramente illustrativo, da definire prima di implementare**) oppure un codice umano **solo quando la corrispondenza è univoca**; validare lunghezza, caratteri e versione, senza interpretare l'input come URL, percorso file, SQL o comando.
- Mostrare un **confronto visivo**: immagine locale se disponibile, codice, titolo, stato della scheda, ultima modifica. La ricerca usa l'**archivio attualmente aperto**; una mancata corrispondenza non cerca su Internet e non cambia archivio.
- Se l'identificativo corrisponde ma il codice scritto sulla vecchia etichetta differisce dal codice corrente, mostrare **«verifica l'etichetta»**. Se più archivi copiati condividono lo stesso UUID, ciò **non** prova che sia la stessa opera fisica: non offrire una conferma di autenticità.
- L'azione standard è **Apri scheda in sola consultazione**. Qualunque operazione successiva di vendita, spostamento, inventario o aggiornamento richiede le consuete schermate e conferme; nessuna scorciatoia di scansione può aggirarle.

### C. Accessibilità e qualità

Operazioni interamente utilizzabili da tastiera e con screen reader: elenco opere selezionate, dimensioni dell'etichetta, messaggi di errore, numero di pagine e risoluzione della scansione. Un codice umano sempre leggibile accompagna il QR, che non deve essere l'unico mezzo di identificazione. Non usare solo il colore per conflitti o avvisi. Anteprima con zoom accessibile e testo non tagliato, comportamento prevedibile alla perdita di focus e con font grandi.

## 5. Dati necessari e interfacce concettuali (NON implementazione)

**Senza nuova entità di catalogo per la fase minima:**
- `artworks.id` (UUID già esistente) come riferimento machine-readable immutabile a livello di record; `artworks.code` per lettura umana; `title` e `year` solo su richiesta dell'utente. Le foto sono opzionali e **non necessarie** all'etichetta.
- Un modello grafico locale (orientamento, larghezza/altezza in mm, margini, font, campi inclusi). Valutare se salvarlo nelle impostazioni del dispositivo oppure se mantenerlo soltanto per la sessione; **non** richiedere migrazione SQLite nella prima fase.
- Un eventuale `LabelPayload { version: 1, artwork_id: UUID }` descrive soltanto i dati codificati. **No** dati identificativi di persone, indirizzi, importi, credenziali, percorsi, nomi di file, dominio web o endpoint di Repertica.
- Un eventuale `resolveLabel(payload)` è una ricerca **locale e read-only** per UUID sull'archivio attivo, non un'API di rete, un'app registrata come protocol handler o una funzione che certifica autenticità.
- Non è necessario tenere un registro obbligatorio di ogni stampa/scansione: aggiungerebbe dati e complessità senza beneficio dimostrato. Un registro volontario di ristampe con data può essere valutato solo dopo riscontri reali.

**Questione di versionamento:** un UUID può essere identico in due copie ripristinate di un archivio. Se un domani TECH-011 abiliterà più archivi, occorrerà definire la semantica del **contesto archivio** prima di pretendere identificazione universale; non inserire ora un fingerprint occulto o un identificativo proprietario che impedisca la portabilità.

## 6. Offline, lingue, impostazioni, esportazione e privacy

- **Offline-first e indipendenza:** generazione locale, stampa tramite sistema operativo, identificazione via tastiera o hardware esterno solo se disponibile; nessun account, abbonamento, servizio di QR remoto, cloud, telemetria, sito pubblico o smartphone obbligatorio. Un QR non deve indirizzare a un URL remoto.
- **Multilingua:** lingua del sistema operativo, eventuale override locale IT/EN/ES/FR/DE già documentato in [ECONOMY-I18N](../../ECONOMY-I18N.md) e fallback inglese per locale sconosciuto o stringa non tradotta. Testi, unità e messaggi vengono tradotti; **UUID e contenuto del codice rimangono invarianti** indipendentemente dalla lingua.
- **Impostazioni facoltative:** modello carta/pagina, misure in mm, mostra titolo sì/no, quantità copie, margini/calibrazione, pagina iniziale. Nessun wizard obbligatorio, nessuna stampante predefinita imposta dall'app. Le preferenze non devono contenere dati personali.
- **Export:** documento di stampa o PDF tramite sistema, con cartella scelta liberamente dall'utente; eventuale CSV UTF-8 della sola selezione (UUID, codice, titolo se autorizzato) per stampatori esterni, con anteprima e avviso sulla divulgazione. La stampa **non sostituisce** i sette registri CSV integrali o il backup completo verificato. Salvare un PDF è una condivisione deliberata, non un backup.
- **Import:** non sono importate opere da un foglio etichette. La scansione **legge un identificativo**, non crea, modifica o sincronizza dati. Se si considerano in futuro modelli importabili JSON, limitarli a dati dichiarativi validati, senza script o riferimenti esterni, e fornire export dello stesso formato.
- **Privacy:** per default nessun prezzo, margine, percentuale, recapito, ubicazione, nome del compratore o nota. L'UUID non è un segreto e il QR non è una protezione d'accesso. Stampa e PDF possono finire negli spooler del sistema o in cartelle accessibili ad altri: informare l'utente prima di stampare dati aggiuntivi. I backup attuali, in chiaro, restano regolati dalle avvertenze già presenti.
- **Compatibilità:** Windows/macOS/Linux Tauri con stampa gestita dal sistema; verificare differenze fra WebView, ridimensionamento, dialogo PDF e stampanti. Nessun driver proprietario o scanner specifico obbligatorio; usare in alternativa ricerca manuale.

## 7. Complessità, dipendenze, rischi e alternative

**Complessità ipotetica:** bassa–media per selezione, layout e stampa con codice umano; media per QR, verifica scanner, impaginazione robusta e ristampe; alta se si pretendesse supporto a ogni formato di etichette pretagliate. Non si propone quella promessa.

**Dipendenze descrittive:** identità UUID/codice opere, stampa/PDF del sistema, componenti UI accessibili, i18n esistente, backup delle preferenze **solo se** persistenti. La codifica QR richiederebbe valutazione di una soluzione locale open source e delle licenze; **nessuna dipendenza viene aggiunta in questa attività**. TECH-011 (eventuali profili archivio) richiederebbe una regola esplicita per scansioni in contesti diversi.

**Rischi:**
1. QR copiabile o applicato all'opera sbagliata, e falsa percezione di autenticazione. **Mitigazione:** foto di confronto, testo di avvertenza, nessuna «verifica autenticità».
2. Codice visibile obsoleto dopo modifica scheda. **Mitigazione:** identificazione per UUID, avviso di mismatch e ristampa manuale.
3. Scansione che sembra confermare presenza fisica. **Mitigazione:** nessun evento di movimentazione/inventario generato automaticamente.
4. Esposizione involontaria di informazioni su etichette o file PDF. **Mitigazione:** allowlist di campi pubblicabili e anteprima.
5. Etichette tagliate o QR illeggibili. **Mitigazione:** prova di stampa, limiti di dimensione, quiet zone, contrasto, test su più stampanti; fallback al codice umano.
6. Collegamenti erronei fra archivi clonati. **Mitigazione:** ricerca solo nell'archivio attivo, nessun trasferimento/lookup automatico inter-archivio.
7. Danneggiamento delle opere per applicazioni adesive inappropriate. **Mitigazione:** indicazioni di utilizzo su custodie e supporti adatti, non sulle superfici dell'opera.

**Alternativa a minore costo:** stampare soltanto codice umano + titolo facoltativo, senza QR e senza nuova schermata di scansione. **Alternativa esterna:** usare un programma locale di stampa etichette alimentato da un CSV selettivo, senza sviluppare editor proprietari. Entrambe possono soddisfare molti studi; occorre confrontarle con il percorso integrato tramite test d'uso.

## 8. Criteri di accettazione e verifiche suggerite

1. Da 30 opere, comprese due omonime e codici simili, si genera una selezione con esattamente 30 etichette distinguibili, senza cambiare alcun record.
2. **Tutti** i campi privati rimangono assenti dal layout predefinito e dal payload scansionabile; test su QR decodificato e PDF risultante.
3. Per una scheda che cambia titolo/codice, il vecchio payload UUID apre ancora quella scheda e segnala l'eventuale incongruenza visibile. Il certificato storico, la vendita e l'ubicazione non cambiano.
4. Una scansione malformata, sconosciuta o dal contesto sbagliato non modifica database o file, non raggiunge Internet e non apre URL.
5. La ricerca funziona offline, con scanner HID **quando compatibile** e con inserimento manuale; nessuna azione automatica di inventario.
6. Una prova di stampa e una di stampa su PDF su Windows/macOS/Linux conservano contrasto, dimensioni e codice leggibile, senza affidamento a hardware proprietario.
7. Tastiera e screen reader consentono selezione, anteprima, stampa e consultazione dell'esito; nessuna informazione solo cromatica.
8. Le impostazioni dei modelli, se persistenti, sopravvivono al riavvio e sono recuperabili senza compromettere la portabilità; nessuna esportazione altera i sette CSV integrali.
9. Su archivio di almeno 1.000 opere, selezione e anteprima devono restare reattive con paginazione/virtualizzazione se necessaria; la soglia di prestazione va **misurata**, non indovinata.

## 9. Fasi possibili, priorità, requisiti e decisioni

- **Fase A — valida il bisogno:** prove osservative con 3–5 professionisti su archivi di dimensioni diverse; confrontare il flusso con semplice stampa di codici e ricerca manuale. **Nessuna implementazione autorizzata.**
- **Fase B — etichette base:** selezione e stampa/PDF con codice umano univoco e anteprima privacy; stampante e scanner non obbligatori.
- **Fase C — identificativo scansionabile:** payload locale versionato, QR, ricerca esatta con input tastiera/scanner, controlli anti-equivoco.
- **Fase D — integrazione facoltativa:** ingresso guidato nelle ricognizioni UX-015 o nei movimenti UX-001 **soltanto se quelle proposte saranno a loro volta approvate**, e sempre con dichiarazione e conferma esplicite.

**Requisiti certi:** il prodotto è gratuito/open source, Tauri + SQLite locale, senza account/abbonamento; i record opera hanno UUID e codice; esistono già export CSV e backup autonomi; lingua di sistema con fallback inglese; nessuna modifica a certificati immutabili, vendite o importi per effetto di una stampa/scansione.

**Ipotesi da validare:** la frequenza reale di ricognizioni e ristampe; convenienza di QR rispetto a sola stampa di codici; compatibilità pratica degli scanner USB con le WebView; interesse per modelli A4 anziché stampanti termiche; utilità di titoli stampati nonostante la privacy.

**Domande da sottoporre al proprietario (non decisioni prese):** preferire etichette per *custodie* o anche cartellini da esposizione? Accettare solo formato A4/PDF all'inizio? Serve realmente un QR oppure basta un codice leggibile? Deve essere possibile stampare etichette per opere non ancora complete (UX-018)? Quale comportamento è desiderato dopo il restore in un archivio duplicato?

**Stato finale:** **Proposta — da valutare**. Nessuna idea approvata, nessuna issue o PR di implementazione, nessuna assegnazione allo sviluppo.
