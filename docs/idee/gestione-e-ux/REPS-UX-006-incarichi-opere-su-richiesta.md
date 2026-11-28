# REPS-UX-006 — Incarichi per opere su richiesta: brief, revisioni e avanzamento

- **ID stabile:** REPS-UX-006
- **Data:** 2026-11-21
- **Stato:** Proposta — da valutare
- **Filone:** Gestione e UX
- **Priorità ipotetica:** P2; P1 soltanto se gli artisti su incarico risultano un pubblico centrale dopo verifica con il proprietario
- **Ambito:** studio di funzionalità future, nessuna implementazione o assegnazione approvata

## Problema, destinatari e distinzione

Il [MVP](../../MVP.md) crea una scheda per un'opera esistente e collega, eventualmente, una singola vendita al relativo collezionista. La proposta [UX-002](REPS-UX-002-trattative-prenotazioni.md) riguarda **interessamento e offerte per opere già identificabili**, non il ciclo di realizzazione di un'opera nuova su specifiche concordate. La [UX-003](REPS-UX-003-costo-e-margine-per-opera.md) propone attribuzione di costi, non gestione delle consegne di un incarico. La [issue #13](https://github.com/Cesare-ml/Repertica-Studio/issues/13) riguarda **commissioni come costi commerciali, incassi e cashflow**, non l'incarico creativo. Non duplicare quei processi.

**Destinatari:** pittori ritrattisti, scultori, fotografi, illustratori e piccoli laboratori che ricevono lavori personalizzati con revisioni, prove e scadenze. Un elenco di opere archiviate non rappresenta chiaramente «richiesto ma non avviato», «bozza approvata», «in lavorazione», «pronto ma non consegnato».

## Scenari professionali

1. Una persona richiede un ritratto: l'artista annota misure, tecnica, riferimento fornito volontariamente, scadenza indicativa e ciò che **non** è incluso. Conferma il brief solo dopo revisione con il committente; nessun obbligo di emettere fattura.
2. Il committente modifica dimensioni dopo l'approvazione iniziale: richiesta di variante, confronto prima/dopo e conferma esplicita; il brief iniziale resta consultabile.
3. L'artista lavora a più incarichi: una vista mostra stati, prossima tappa e scadenze, senza interpretare scadenze ipotetiche come accordi legalmente vincolanti.
4. A conclusione, l'artista associa l'incarico a un'opera reale nell'archivio. La scheda può poi seguire la **normale** vendita e il certificato; nessun automatismo che dichiari incasso o trasferimento di proprietà.
5. Il committente recede; l'artista archivia lo stato e i file già ricevuti, con opzioni di conservazione/cancellazione nel rispetto dei vincoli dei documenti eventualmente già emessi.

## Esperienza utente suggerita

- **Home/Archivio → Incarichi (opzionale):** elenco separato con titolo breve, committente, fase, prossima azione, scadenza proposta, alert per date trascorse non confermate e collegamento all'opera quando esiste. Nessun nuovo modulo obbligatorio per chi cataloga soltanto opere finite.
- **Nuovo incarico:** scelta del contatto esistente o inserimento come bozza locale; nome progetto, brief libero, tecnica/misure previste, materiali, massimale e date *facoltativi*. Non richiedere anticipo o prezzo.
- **Rivedi brief:** confronto di versioni con campi cambiati, autore della modifica dichiarato dall'utente (nessuna identità digitale attestata), data e note; «proposto» → «confermato dall'utente» con azione consapevole, non firma elettronica o accettazione giuridica.
- **Fasi:** richiesta, da definire, concordato, in lavorazione, in revisione, completato, annullato; passaggio manuale con eventuale nota. Le fasi descrivono **avanzamento creativo**, non stato finanziario.
- **Revisioni:** lista di richieste, risposta e allegati di riferimento opzionali; distinzione «richiesto», «accettato», «non accettato». Nessuna modifica a una versione approvata senza nuova versione.
- **Collega opera realizzata:** scegli un record esistente o crea una bozza di opera tramite il normale modulo, ma mai una vendita automatica. Verifica dell'eventuale collegamento già esistente.
- **Accessibilità:** cronologia testuale ordinata anche senza colori, indicatori con etichetta e data, navigazione tastiera completa, riepilogo delle modifiche leggibile con screen reader; nessun drag-and-drop necessario.

## Ipotesi dati e regole (solo descrittive)

- Entità **artwork_request**: UUID, riferimento opzionale a collector/contact UUID, nome, descrizione brief, fase, data richiesta, scadenza stimata, versione corrente, eventuale artwork UUID, timestamps e note private.
- **request_revision**: UUID, richiesta UUID, numero di versione monotono, tipo (brief, dimensioni, materiali, tempi), dati prima/dopo *minimi*, data, stato e nota di conferma manuale; versioni storiche in sola lettura.
- **request_milestone**: UUID, richiesta UUID, descrizione, data prevista opzionale, data completamento effettiva opzionale, stato. Non trasformare la pianificazione in notifiche automatiche.
- **request_attachment**: UUID, richiesta UUID, file nel media store locale, tipo, descrizione, data. Allegati originali preservati e inclusi nel backup; limiti di formato e spazio da decidere. Evitare i percorsi assoluti sensibili o file link esterni come unica fonte.
- **Identità e collegamenti:** l'incarico può esistere senza artwork e senza vendita. Un incarico può riferirsi a una sola opera nella proposta semplice; una relazione molti-a-molti per installazioni o set è alternativa futura da valutare. Il vincolo dell'MVP «una vendita per opera» resta intatto.
- **Valori economici:** eventuale prezzo *indicativo* in centesimi interi EUR, con versione e dicitura «non contabilizzato». Nessun acconto reale, IVA, fattura, commissione di intermediazione o incasso: sono fuori ambito e dipendono da altre decisioni.
- **Compatibilità e cancellazioni:** migrazione additiva solo se un giorno autorizzata; nessun record pregresso associato automaticamente, nessuna sovrascrittura di vendite, certificati o storico. Preferire archiviazione/annullamento per incarichi con versioni confermate; cancellazione dei dati di riferimento con controllo dei legami.

## Impostazioni, offline e multilingua

- Impostazioni **facoltative**: fasi mostrate, scadenza predefinita *solo suggerita*, unità di misura (salvataggio canonico in cm), nomi di tappe; non introdurre template contrattuali obbligatori.
- Tutto funziona con SQLite e media locali, senza account, abbonamenti, sincronizzazione, mail, chat, IA, caricamenti esterni o accesso remoto. I riferimenti fotografici forniti dal committente possono essere particolarmente sensibili e non devono apparire nelle schede pubbliche.
- Lingua del sistema con fallback inglese, override locale IT/EN/ES/FR/DE già previsto. Tradurre etichette, fasi predefinite, conferme ed errori; non alterare brief e nomi propri scritti dall'utente. Date localizzate a video, persistite in formato non ambiguo.

## Portabilità, backup, sicurezza e privacy

- CSV minimale per il riepilogo incarichi (UUID, fase, tappe e date, **senza** brief, riferimenti e contatti per default); esportazione dettagliata solo con scelta esplicita e anteprima del contenuto sensibile.
- Formato strutturato versionato per un eventuale round trip con UUID, versioni, legami e media relativi; coordinarsi con [TECH-001 — pacchetto dati aperto](https://github.com/Cesare-ml/Repertica-Studio/blob/docs/idee-strumenti-architettura-20261121/docs/idee/strumenti-e-architettura/REPS-TECH-001-pacchetto-dati-aperto.md), **senza** implementarlo in questo filone. Riconciliare contatti esistenti e prevenire duplicati.
- Backup completo nella cartella scelta dall'utente deve includere cronologia e file **solo se la funzionalità verrà realizzata**; eventuale ripristino deve preservare versioni e allegati, senza inventare approvazioni.
- Privacy per impostazione predefinita: dati di contatto, ritratti, immagini di riferimento, condizioni e budget non trasmessi; anteprima prima di stampare/esportare e possibilità di rimuovere materiale non più necessario senza cancellare per errore un originale protetto.

## Complessità, rischi e alternative

- **Complessità:** media per fasi/brief e collegamento all'archivio; alta per versioni, allegati, riduzione dei dati personali e compatibilità import/export.
- **Dipendenze:** collector UUID (o successivo contatto generico), artwork UUID, galleria/media e backup, i18n. **Non** dipende dall'adozione di UX-002 o di un sistema contabile.
- **Rischi:** doppia creazione di opere, interpretazione «brief approvato» come firma valida, promesse sulle scadenze, file di terzi conservati indefinitamente, confusione tra prezzo e incasso. Mostrare chiaramente stati descrittivi e provenienza manuale.
- **Alternativa minima:** una scheda incarico con brief, fase, data e link opera, senza versionamento o allegati. Ideale per testare la domanda prima di modellare un workflow complesso.
- **Alternativa esterna:** usare il normale registro opere con note; costo di sviluppo quasi nullo ma inadeguato per opere non ancora esistenti e per cronologia delle revisioni.

## Criteri di accettazione ipotetici

1. Un incarico senza opera o vendita è salvabile e ricercabile; non entra nei ricavi e non modifica la disponibilità degli artworks.
2. La modifica a un brief già confermato crea una **nuova revisione consultabile**, senza riscrivere la precedente.
3. Completare un incarico non registra automaticamente vendite, incassi, certificati, consegna o trasferimento di proprietà.
4. Collegando un'opera già esistente, il sistema segnala eventuali duplicati e permette di annullare senza conseguenze; la cronologia resta.
5. Un'esportazione per terzi esclude di default foto private, budget e recapiti; un round trip locale autorizzato conserva revisioni e allegati.
6. Backup e ripristino includono le relazioni e non lasciano file orfani; la normale app resta pienamente utilizzabile offline, in lingue supportate, da tastiera.
7. Date e importi assenti restano «non indicati», non si inventano scadenze, prezzo, pagamento o obblighi contrattuali.

## Fasi esplorative, non autorizzate

- **A:** conferma di utilità tramite casi reali, scheda incarico minimale e collegamento manuale ad artwork.
- **B:** tappe, brief versionati e registro revisioni; preferenze per stati.
- **C:** allegati locali ed esportazione selettiva con controlli privacy, convalida su grandi archivi e restore.

## Certezze, ipotesi e decisioni aperte

- **Certi dal repository:** opera, collezionista, vendita, certificato, foto e backup locali esistono; una vendita per record; i ricavi registrati non sono incassi; i dati monetari sono in centesimi EUR; la lingua è configurata con fallback inglese.
- **Ipotesi da validare:** gli utenti ricevono abbastanza incarichi su misura da richiedere un registro distinto dalle opere; le versioni di brief contano più di semplici note.
- **Questioni al proprietario:** sono centrali artisti con ordini personalizzati? Servono incarichi con più opere o solamente uno-a-uno? Il brief dovrebbe includere testo di approvazione non vincolante? È desiderata una vista settimanale delle tappe o basta l'elenco? Quale politica di eliminazione applicare agli allegati forniti da terzi?

**Questa scheda non autorizza sviluppo, non approva alcun flusso e non modifica la roadmap.**
