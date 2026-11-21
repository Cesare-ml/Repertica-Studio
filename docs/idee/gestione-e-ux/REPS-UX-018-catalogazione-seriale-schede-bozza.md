# REPS-UX-018 — Catalogazione seriale con riuso controllato dei dati e schede da completare

- **ID stabile:** REPS-UX-018
- **Data della ricognizione:** 2026-11-21
- **Stato:** Proposta — da valutare
- **Filone:** Gestione e UX
- **Priorità ipotetica:** P1–P2 soltanto per utenti che registrano molte opere in sequenza; da verificare
- **Natura:** esclusivamente ricerca e progettazione documentale; nessuna approvazione, assegnazione o autorizzazione all'implementazione

## 1. Motivazione, destinatari e fatti verificati

**Problema.** Registrare venti o cento opere di uno stesso studio, inventariare un lascito o ricevere una selezione da una galleria implica ripetere tecnica, supporto, anno e talvolta ubicazione, mentre codice, titolo e misure devono restare distinti. L'inserimento ripetitivo favorisce errori di trascrizione; una scheda interrotta può perdere il lavoro non ancora salvato. Occorre un flusso **di catalogazione manuale assistita**, non un importatore opaco né una modifica indiscriminata di record già consolidati.

**Utenti:** artisti che censiscono nuove opere dello stesso ciclo; studi e gallerie che accolgono gruppi di lavori; piccoli archivi che ricostruiscono dati parziali da fotografie e inventari cartacei. Per chi inserisce un'opera ogni tanto, l'attuale modulo singolo deve restare semplice e prioritario.

**Evidenze su `main`, 21 novembre 2026:**
- [`src/app/app.component.ts`](../../../src/app/app.component.ts) espone un unico `artworkDraft`, un dialogo opera e un normale flusso di salvataggio; non risulta una sessione per inserimenti seriali o un elenco persistente di bozze.
- [`src-tauri/src/commands.rs`](../../../src-tauri/src/commands.rs) valida e salva **una scheda per chiamata**; codice e titolo sono obbligatori, misure positive se specificate, e lo stato «venduta» non può essere impostato senza la corrispondente vendita.
- [`src-tauri/src/db.rs`](../../../src-tauri/src/db.rs) conserva UUID stabili e `artworks.code` con unicità case-insensitive; la colonna `status` ha significato commerciale e **non** comprende `draft`. `year`, tecnica, supporto, misure, descrizione, ubicazione e foto sono opzionali; lo schema corrente è v5.
- Il [README](../../../README.md) e [MVP](../../MVP.md) descrivono ricerca, filtri, foto/galleria e backup locale; non documentano catalogazione sequenziale.
- Esiste una [PR #35](https://github.com/Cesare-ml/Repertica-Studio/pull/35) **aperta** sulle sovrascritture da moduli obsoleti: non presumere che il relativo controllo di concorrenza sia già disponibile su `main`.

**Bisogno da convalidare**, non dedotto come fatto: utenti con lotti di almeno 15–30 opere possono trarre beneficio da una modalità dedicata; senza osservazioni d'uso, il guadagno di tempo non è dimostrato.

## 2. Confini per evitare doppioni

- **[UX-005](REPS-UX-005-edizioni-multipli-serie-numerate.md)** progetta **esemplari di tirature** con relazioni proprie; questa proposta registra **opere già autonome** e non inferisce serie numerate né copia prezzi, titolarità o diritti.
- **[UX-011](REPS-UX-011-cicli-creativi-relazioni-opere.md)** gestisce relazioni artistiche fra opere; la ripetizione di campi non crea automaticamente un ciclo, un dittico o una parentela.
- **[UX-015](REPS-UX-015-ricognizioni-inventariali-discrepanze.md)** verifica la presenza **fisica di opere già catalogate**; qui si progetta l'acquisizione dei metadati in archivio, non una sessione di ricognizione.
- **[TECH-001](https://github.com/Cesare-ml/Repertica-Studio/blob/docs/idee-strumenti-architettura-20261121/docs/idee/strumenti-e-architettura/REPS-TECH-001-pacchetto-dati-aperto.md)** studia l'importazione/esportazione strutturata da file; la catalogazione seriale è **interattiva e manuale**, con un operatore che verifica ogni scheda prima di registrarla. Non include parser CSV o migrazioni di dati esterni.
- **[TECH-008](https://github.com/Cesare-ml/Repertica-Studio/blob/docs/idee-strumenti-architettura-20261121/docs/idee/strumenti-e-architettura/REPS-TECH-008-viste-salvate-filtri-ricerca-locale.md)** studia viste e filtri su opere già registrate; questa scheda riguarda **inserimento, verifica e ripresa di bozze**, non un nuovo motore di ricerca.
- **[TECH-010](https://github.com/Cesare-ml/Repertica-Studio/blob/docs/idee-strumenti-architettura-20261121/docs/idee/strumenti-e-architettura/REPS-TECH-010-storico-revisioni-metadati.md)** concerne la revisione storica di metadati consolidati; una bozza temporanea non è cronologia probatoria.
- Nessuna importazione dal Gestionale Opere ([issue #5](https://github.com/Cesare-ml/Repertica-Studio/issues/5)), modifica massiva delle vendite, certificazione, generazione automatica di opere da IA, OCR o servizio di riconoscimento fotografico. Nessun file del filone tecnico viene cambiato.

## 3. Scenari professionali

1. **Trenta tele prodotte nello stesso periodo.** L'artista sceglie di riutilizzare solo «tecnica: olio», «supporto: tela» e «ubicazione: studio». Inserisce per ogni opera codice, titolo e dimensioni individuali, con il comando **Salva e continua**. L'anno viene copiato solo se selezionato esplicitamente; le fotografie si collegano all'opera corretta.
2. **Catalogazione interrotta.** Un archivista comincia dieci schede, ma tre non hanno codice o titolo certo. Nella fase evoluta salva **tre bozze locali distinte**, riprende la sessione dopo il riavvio e finalizza solo quelle complete. Le bozze non risultano come opere disponibili, prenotabili o vendibili.
3. **Codice già utilizzato.** Una galleria propone `GAL-0018` per una nuova opera, ma esiste già `gal-0018`. Il controllo mostra entrambe le schede e chiede un identificativo diverso; non sovrascrive l'opera esistente, non salta silenziosamente il codice e non ricicla un UUID.
4. **Gruppo eterogeneo.** Un archivio inserisce fotografie, sculture e disegni. Il riuso è facoltativo e disattivabile campo per campo: la modalità sequenziale non assegna «olio su tela» a una scultura per inerzia.
5. **Revisione prima della conferma.** Prima di finalizzare un gruppo di nuove schede si vedono codice, titolo, misure, eventuale foto e campi ignoti. L'operatore può confermare singolarmente; un errore su una scheda non deve apparire come conferma di tutte le altre.

## 4. Flusso UX ipotetico

**A — Inizio.** Da **Opere → Nuova opera** mantenere il flusso attuale e aggiungere una scelta facoltativa **Inserisci più opere in sequenza**. Nessun passaggio preliminare o wizard obbligatorio per l'inserimento singolo.

**B — Campi ripetibili.** Mostrare un pannello compatto «Mantieni per la prossima opera» con selezioni individuali per anno, tecnica, supporto, ubicazione, descrizione (quest'ultima *disattivata per default*). Non riutilizzare mai automaticamente codice, titolo, misure, immagine, stato commerciale, prezzo, vendite, clienti, note private o certificati. Le proposte di valore sono visibili e modificabili prima di ciascun salvataggio. «Azzera campi riutilizzati» deve essere evidente.

**C — Codice.** Facoltativamente suggerire un prefisso e un progressivo **solo come anteprima**. Il codice rimane definito e confermato dall'operatore; effettuare verifica contro i codici esistenti con confronto case-insensitive e nuova validazione al momento del commit. Non garantire sequenze senza lacune e non «riservare» codici in modo invisibile. In mancanza di regole desiderate, l'utente scrive il codice manualmente.

**D — Salva e continua (prima fase).** Dopo la validazione della scheda, salvare un'opera e mostrare il suo codice/UUID e un riscontro esplicito. Aprire subito il modulo seguente mantenendo **solo** i campi scelti. Il comando «Annulla» non deve cancellare le opere già confermate. Raggiungibile da tastiera, con focus sul primo campo veramente variabile e pulsante standard «Salva e chiudi» sempre disponibile.

**E — Sessione di bozze (fase successiva, da approvare separatamente).** Lista locale di schede temporanee con stato **da completare / pronta alla conferma / errore da risolvere / confermata**, conteggio preciso, date di modifica e ricerca per codice provvisorio. I dati facoltativi o incerti restano tali: non si inventano titoli, date o dimensioni. L'operatore può rientrare in ciascuna bozza, eliminare una sola bozza con conferma o finalizzarla come nuova opera. Eventuali fotografie in attesa richiedono una politica separata di staging sicuro e pulizia non distruttiva.

**F — Riepilogo di sessione.** Mostrare esattamente quali righe sono state create e quali **non** lo sono; collegamento alle opere definitive. Nessun comando «applica a tutte» a campi di schede consolidate nella prima fase. Un'eventuale conferma multipla futura dovrebbe offrire anteprima per riga e garanzie transazionali esplicite.

**Accessibilità:** tasti dedicati non obbligatori; ordine tab logico, pulsanti con nomi accessibili, errori presso i campi, live region moderata per il successo, stato espresso in testo oltre al colore, navigazione della lista bozze senza drag-and-drop e form utilizzabile con zoom 200% e screen reader.

## 5. Dati necessari e vincoli (solo ipotesi descrittive)

- **Fase minima:** riusare `ArtworkInput` e l'operazione singola di creazione esistente; la selezione dei campi da riutilizzare è *stato di sessione* locale, non uno schema di catalogazione parallelo. Il codice resta obbligatorio per un'opera definitiva, e l'unicità `NOCASE` resta autoritativa.
- **Impostazione facoltativa:** `sequential_entry_preferences` con elenco dei *nomi* dei campi che possono rimanere precompilati, prefisso codice se scelto, nessun valore sensibile obbligatorio. Le preferenze sono per archivio locale, mai sincronizzate in remoto.
- **Eventuale fase bozze:** entità separata `cataloguing_draft` (UUID bozza, session UUID, posizione, payload di metadati parziali, timestamp, stato di convalida, riferimenti temporanei ai media, eventuale UUID opera finalizzata). **Non aggiungere `draft` a `artworks.status`**: lo stato attuale ha valore commerciale e può essere legato alle vendite.
- **Regole di integrità:** due bozze possono avere lo stesso codice provvisorio **solo finché non finalizzate**; la conferma deve rieseguire tutte le validazioni correnti, assegnare una sola volta l'UUID dell'opera e non produrre certificati, vendite, incassi o spese. Se una bozza è già finalizzata, un secondo clic non crea duplicati (operazione idempotente con riferimento finale).
- **Foto:** mai condividere automaticamente il percorso fisico di una foto fra più opere. Lo staging temporaneo, se introdotto, deve restare confinato nel supporto locale dell'archivio, essere recuperabile tramite backup e non cancellare file storici dei certificati o originali di altre opere. Prima fase senza staging complesso.
- **Modifiche successive:** non applicare massivamente campi a opere già esistenti. Se in futuro saranno previste, richiedono controllo dei conflitti di versione, anteprima dei cambiamenti e salvaguardie sulle opere vendute: non presumere che la [PR #35](https://github.com/Cesare-ml/Repertica-Studio/pull/35) sia già integrata.

## 6. Offline, impostazioni, lingue, privacy e compatibilità

- **Offline totale:** il riuso di campi, le convalide, le bozze e l'eventuale riepilogo lavorano sullo SQLite/media del dispositivo; non richiedono server, account, browser, abbonamento, IA, localizzazione GPS o connessioni esterne.
- **Impostazioni:** modalità seriale sempre opt-in; elenco campi riutilizzabili configurabile, funzione «non conservare impostazioni di sessione»; nessun prefisso obbligatorio. Distinguere preferenze salvate dalle **bozze contenenti dati**. Disattivare l'eventuale riapertura automatica di bozze personali su postazioni condivise.
- **Multilingua:** lingua di sistema e fallback inglese; override IT/EN/ES/FR/DE come già previsto in [ECONOMY-I18N](../../ECONOMY-I18N.md). Tradurre validazioni, stati, opzioni e istruzioni, senza tradurre codici, titoli, tecniche o note inserite dagli utenti. Etichette di unità e numeri seguono la locale; le misure sono conservate come numeri in cm con parser sicuro, non come stringhe ambigue.
- **Privacy:** ubicazioni, fotografie, titoli non pubblici e metadati di opere non ancora presentate restano dati locali. Una bozza può essere confidenziale quanto una scheda completa; non visualizzarla in esportazioni pubbliche, certificati o schede per terzi. I backup esistenti sono **in chiaro**, pertanto includere eventuali bozze significa richiamare il rischio di accesso ai dati su supporti condivisi.
- **Compatibilità:** la sola fase «Salva e continua» non richiede cambiare lo schema delle opere; le bozze persistenti richiederebbero migrazione **additiva**, schema versionato e verifica che backup/restore includano eventuali file di staging. Gli archivi senza bozze non ricevono dati artificiosi. Le versioni più vecchie possono non aprire schemi futuri: documentare e collaudare, non promettere retrocompatibilità in scrittura.
- **File e diritti:** non dedurre provenienza, autenticità, licenze o diritti da nomi foto/cartelle; l'utente resta responsabile della catalogazione e dell'eventuale condivisione.

## 7. Import, export e interoperabilità

- **Già presente:** i sette CSV di [DATA-PORTABILITY](../../DATA-PORTABILITY.md) sono registri integrali **in uscita**, non un formato di reimportazione, mentre il backup verificato include SQLite e media. Il flusso seriale non deve modificare né filtrare silenziosamente queste esportazioni.
- **Esportazione facoltativa:** riepilogo sessione in CSV UTF-8 di sole opere definitive con UUID/codici e esito; anteprima dei campi inclusi e disattivazione per default di ubicazioni e note private. È un report operativo, **non** un ottavo registro che sostituisce i CSV integrali.
- **Bozze:** se l'utente vuole esportarle, prevedere un JSON versionato, marcato `unfinished=true`, con identificativi distinti e valori opzionali; import **solo con anteprima**, verifica duplicati e conferma esplicita. Non considerare una bozza equivalente a un record di opera e non fabbricare code/UUID di opere vere durante import. Eventuali file fotografici devono essere espliciti in un pacchetto con manifest; nessun percorso assoluto non portabile.
- **Coordinamento TECH-001:** un eventuale pacchetto di interscambio generale di opere va progettato dal filone tecnico, non duplicato qui. Se un import massivo professionale è già disponibile, questa UX dovrebbe limitarsi a **revisione assistita degli elementi importati**, evitando doppie pipeline.
- **Ripristino:** backup completo su cartella liberamente scelta e verifica SHA-256 restano l'unico percorso di ripristino integrale. Un report di sessione non è un backup e non consente recupero automatico delle fotografie.

## 8. Complessità, dipendenze, rischi e alternative

**Complessità indicativa:** bassa-media per «Salva e continua» e riuso campo per campo; media per suggerimenti codice e anteprime duplicati; medio-alta per bozze persistenti, foto in staging, recupero da crash e import/export idempotente. Le stime sono **progettuali**, non una pianificazione assegnata.

**Dipendenze:** modelli di opera, validazione Rust e `UNIQUE COLLATE NOCASE`, UI Angular/Tauri, i18n, backup/restore. Nessuna dipendenza necessaria dalle altre proposte; eventuale integrazione TECH-001/008/010 e UX-005/011/015 solo dopo decisioni autonome.

**Rischi principali:** propagazione accidentale di tecnica/ubicazione errata a molte opere; collisioni di codice durante la revisione; interpretare bozze come opere disponibili; prematura assegnazione del campo «venduta»; salvataggi parziali nascosti; perdita di foto in staging; note riservate copiate per inerzia; ingombro dell'interfaccia per utenti occasionali.

**Mitigazioni:** campo di riuso scelto esplicitamente, intestazione persistente «stai riutilizzando», reset rapido, doppia verifica codice al commit, stato bozze separato, nessuna scrittura commerciale implicita, conferma per opera, report degli esiti, test crash/backup e modalità singola invariata.

**Alternative da comparare:**
1. Semplice pulsante **Duplica i campi** su una nuova scheda, con esclusione forzata di codice/titolo/foto: costo contenuto ma rischia di copiare troppo.
2. **Salva e continua** con soli 2–3 campi comuni e senza sessione persistente: probabile candidato di prima fase.
3. Foglio di calcolo/import assistito esterno: più adatto a migliaia di record già digitalizzati e attinente a TECH-001/issue #5; meno utile per l'inserimento manuale davanti alle opere.
4. Mantenere il modulo singolo se le interviste mostrano che i lotti sono rari.

## 9. Criteri di accettazione ipotetici (non test eseguiti)

1. Inserire consecutivamente 30 opere riutilizzando **solo** tecnica e supporto genera 30 UUID distinti e 30 codici univoci; titolo, misure, stato e foto non vengono ereditati.
2. Un tentativo con `GAL-018` quando esiste `gal-018` fallisce con messaggio localizzato e **zero modifiche** al record preesistente; l'operatore può correggere e proseguire.
3. Il pulsante «Annulla» su una nuova scheda non annulla le 5 opere precedentemente confermate; il riepilogo distingue 5 salvate, 1 abbandonata.
4. Nella fase bozze, chiudere e riaprire l'app recupera le schede non finalizzate senza trasformarle in `artworks`, vendite, certificati o righe delle statistiche.
5. Confermare due volte la stessa bozza produce **una sola opera**; un errore su una bozza non conferma silenziosamente altre; eventuali conferme parziali sono elencate correttamente.
6. Una scheda con codice/titolo mancanti può essere una bozza, **mai un'opera finale**. Nessun salvataggio seriale può impostare direttamente `sold` senza vendita valida.
7. Backup/ripristino su cartella scelta preservano opere confermate e, **se introdotte**, bozze e media temporanei; la verifica fallisce in modo chiaro quando manca una foto necessaria.
8. Tutti i passaggi funzionano senza rete, in IT/EN/ES/FR/DE con fallback inglese, con Unicode, percorsi non ASCII, tastiera e screen reader; controllare che zoom e schermate piccole non nascondano conferme.
9. Il tempo effettivo e la percentuale di errori su un lotto reale vengono confrontati con l'inserimento singolo prima di attribuire a questa UX un vantaggio misurato.

## 10. Fasi possibili (nessuna autorizzata)

- **Ricerca preliminare:** osservare 3–5 sessioni reali di inserimento, misurare opere per sessione, campi ripetitivi, errori e gestione di dati ignoti; verificare che il beneficio superi complessità e costi di manutenzione.
- **Fase A — minima:** modalità opt-in «Salva e continua» con riuso esplicito di pochi campi, schermata singola invariata e messaggi di conflitto codice chiari. Nessuno schema «bozze».
- **Fase B — soltanto se utile:** suggerimenti di codice non vincolanti e riepilogo delle opere appena create, controlli di completezza non bloccanti sui campi opzionali.
- **Fase C — eventuale:** bozze persistenti, ripresa sessioni, verifica pre-conferma e media in staging con test di backup/restore.
- **Fase D — eventuale:** export/import delle sessioni e interoperabilità descrittiva con TECH-001, senza incorporare importatori o automazioni esterne.

## 11. Requisiti certi, ipotesi e questioni aperte per il proprietario

**Certi dall'attuale progetto:** app desktop gratuita/open source, SQLite locale e offline-first, nessun account/abbonamento; opere con UUID/codice unico/titolo e campi opzionali; salvataggio singolo e stato commerciale vincolato; backup verificato in cartella scelta; OS language/fallback inglese; certificati immutabili; sette CSV tabulari unidirezionali.

**Ipotesi non provate:** esistono utenti che archiviano regolarmente 15–30 opere per sessione; pochi campi sono ripetuti quasi sempre; bozze persistenti valgono il costo aggiuntivo; i professionisti desiderano suggerimenti di codice piuttosto che codici completamente manuali.

**Domande da decidere personalmente:** la destinazione primaria è artista singolo, galleria in acquisizione o archivio di grandi dimensioni? È sufficiente «Salva e continua» senza bozze? Quali campi si possono riutilizzare con sicurezza e quali mai? Il codice progressivo deve avere schema configurabile o meglio solo anteprima manuale? Servono fotografie nelle bozze o possono essere aggiunte dopo? Quanto spesso l'inserimento viene interrotto? Il beneficio atteso giustifica una nuova schermata, rispetto a migliorare il modulo attuale?

**Stato finale: Proposta — da valutare. Non approvata, non assegnata, non implementata.**
