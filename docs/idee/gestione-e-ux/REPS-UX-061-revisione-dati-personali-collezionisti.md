# REPS-UX-061 — Revisione dei dati personali dei collezionisti, senza cancellazioni storiche implicite

- **ID stabile:** REPS-UX-061
- **Data della ricognizione:** 2026-11-23
- **Stato:** Proposta — da valutare
- **Filone:** Gestione e UX, esclusivamente documentale
- **Priorità ipotetica:** P2 solo dopo validazione; P3 o nessuno sviluppo se sono sufficienti le procedure manuali
- **Decisione:** nessuna approvazione, assegnazione o autorizzazione all'implementazione

## 1. Problema professionale e destinatari

Un'artista o una piccola galleria può dover correggere un indirizzo, togliere dati non più necessari, rispondere a una richiesta di accesso ai dati personali oppure spiegare perché un nominativo continua a comparire nei documenti storici di una vendita. Oggi le informazioni possono trovarsi nella scheda del collezionista, nelle sue fotografie, nelle vendite e nei relativi incassi, nei certificati con contenuto storico congelato, nei CSV già esportati e in backup che restano sotto il controllo dell'utente.

Il bisogno non è introdurre una piattaforma di compliance, né promettere che un clic possa eliminare ogni copia. L'opportunità è **rendere comprensibile l'impatto delle scelte** e consentire una **revisione consapevole dei dati correnti** senza alterare la storia delle opere, inventare obblighi legali o compromettere l'archivio.

**Destinatari ipotetici:** artisti con rapporti continuativi con collezionisti, piccoli studi, gallerie e archivi indipendenti che gestiscono dati personali in locale. Non è una funzione di marketing, mailing list o gestione di pratiche giudiziarie.

## 2. Riscontri verificati su `main` al 23 novembre 2026

- `src/app/models.ts` e `src-tauri/src/collectors.rs`: il contatto possiede nome, recapiti, indirizzo, note, profili social e campi facoltativi di data/ora/luogo di nascita; ha fotografie di profilo e aggiuntive. È possibile modificare i campi. La cancellazione del collezionista **viene bloccata se esiste una vendita collegata**, mentre per un contatto senza vendite è già disponibile.
- `src-tauri/src/commands.rs`: una vendita usa il collezionista come relazione; il certificato mantiene uno `snapshot_json` storico non riscritto da normali modifiche alla scheda dell'opera.
- `docs/DATA-PORTABILITY.md`: il CSV integrale `collectors` include contatti, note, dati di nascita, riferimenti a fotografie; anche registri di vendita, certificati, rate, saldi e incassi possono contenere riferimenti personali. I CSV sono esportazioni unidirezionali, **non** uno strumento di cancellazione o ripristino.
- `README.md` e documentazione backup: le copie includono SQLite e media, sono verificabili e **in chiaro**; l'utente sceglie la cartella, anche esterna. Copie precedenti e file già distribuiti non cambiano quando si modifica il database attivo.

Sono riscontri di design del codice/documentazione, **non** prova di una violazione, di richieste reali ricevute o di obblighi legali specifici per un determinato archivio.

## 3. Confine rispetto alle schede esistenti

- **[UX-014](REPS-UX-014-rubrica-relazioni-professionali.md):** modella persone e ruoli; qui si studia come *rivedere i dati personali già conservati* e capire le conseguenze delle azioni, senza introdurre una seconda rubrica.
- **[UX-035](REPS-UX-035-preferenze-contatto-comunicazioni-individuali.md):** istruzioni sulle comunicazioni; non decide quali altri dati possano essere rettificati o cancellati.
- **[UX-056](REPS-UX-056-indirizzi-multipli-contatti-destinazioni.md):** eventuali recapiti multipli; non comprende una revisione dell'insieme dei dati.
- **[TECH-025](https://github.com/Cesare-ml/Repertica-Studio/blob/docs/idee-strumenti-architettura-20261121/docs/idee/strumenti-e-architettura/REPS-TECH-025-esportazioni-tabellari-minimizzate-condivisione.md):** esportazioni selettive per collaboratori; qui la selezione riguarda *un singolo soggetto e l'operatività dell'utente*, senza definire un nuovo formato tecnico.
- **[TECH-040](https://github.com/Cesare-ml/Repertica-Studio/blob/docs/idee-strumenti-architettura-20261121/docs/idee/strumenti-e-architettura/REPS-TECH-040-scambio-contatti-vcard-offline.md):** rubrica vCard per interoperabilità, non risposta a richieste relative ai dati personali.
- **TECH-006/009/017/052:** cifratura, conservazione e verifica dei backup, da non duplicare né modificare. L'eventuale proposta UX deve soltanto segnalare la presenza e l'autonomia delle copie.

Nessun file dell'altra automazione va modificato.

## 4. Casi d'uso, senza presunzioni giuridiche

1. **Recapito non aggiornato.** Un collezionista segnala un vecchio indirizzo e chiede di eliminarlo. L'operatore individua il dato nella scheda attiva, lo corregge o svuota, vede che esportazioni e backup già prodotti non sono aggiornati. Non si assume che un certificato storico debba essere riscritto.
2. **Dati facoltativi superflui.** L'artista rivede data, ora e luogo di nascita, fotografie e note private e decide manualmente quali dati servono ancora. Il software non impone una durata di conservazione né deduce una base giuridica.
3. **Richiesta relativa a un acquirente.** Il contatto è collegato a una vendita e non può essere cancellato con il comando corrente. La vista mostra *quali registri sono coinvolti*, consente di modificare i dati correnti se lecito e invita alla valutazione umana per quelli storici e per eventuali obblighi di conservazione. **Non offre anonimizzazione automatica** o bypass delle relazioni.
4. **Richiesta di copia.** Prima di condividere qualsiasi file, l'operatore vede l'ambito dei dati collegati e il rischio di includere dati di terzi; l'esportazione integrale dell'archivio non deve essere presentata come copia del solo interessato.
5. **Ripristino successivo.** Dopo una correzione, un vecchio backup può reintrodurre campi precedenti. Il software non dichiara eliminati dati che sopravvivono in copie esterne: indica il limite e l'esigenza di verificare dopo il ripristino.

## 5. Flusso UX ipotetico

**Collezionisti → Apri scheda → Rivedi dati personali** (voce facoltativa; non nuovo permesso di accesso). Pannello locale, ordinato per: identità/recapiti; note e dati facoltativi; fotografie; riferimenti indiretti a vendite, incassi, certificati e relativi documenti; informazioni sulle copie esterne.

- **A. Inventario leggibile:** mostra campi effettivamente presenti e la loro posizione, senza classificazioni come «illegale», «obbligatorio» o «consenso acquisito» calcolate automaticamente. I riferimenti storici sono collegamenti ai registri autorevoli, non copie modificabili.
- **B. Scelta esplicita:** azioni distinte *Correggi dato corrente*, *Svuota campo facoltativo*, *Rimuovi fotografia non necessaria* (se consentito dai vincoli esistenti), *Elimina contatto senza vendite* (comando già presente), *Esamina manualmente dati storici*. Nessuna modifica in blocco di vendite o certificati. Anteprima prima di confermare interventi irreversibili.
- **C. Conflitti e limiti:** se una relazione impedisce la cancellazione, spiegare cosa rimane e offrire solo azioni ammesse, senza «sblocchi» automatici. Una decisione giuridica non può essere affidata a una spunta predefinita.
- **D. Riepilogo locale:** elenco di modifiche effettivamente confermate e di punti ancora da verificare (copie, documenti distribuiti, eventuali record storici). *Completato* significa «operazioni locali scelte eseguite», **non** «tutti i dati sono stati cancellati ovunque» o «conformità garantita».
- **E. Esportazione solo intenzionale:** eventuale prospetto per una singola persona con anteprima e omissis verificati manualmente, se ritenuto utile in una fase successiva. Mai invio automatico via email, MCP o rete.

Il flusso non sostituisce le normali funzionalità di modifica/cancellazione; se la revisione manuale con un semplice promemoria è sufficiente, non si costruisce un nuovo modulo.

## 6. Dati ipotetici, compatibilità, import/export

**Fase minima:** nessuna nuova tabella: riepilogo calcolato in lettura dai record esistenti, con conferma affidata ai comandi esistenti. Gli UUID non cambiano; niente riassegnazione di vendite e nessuna mutazione degli snapshot dei certificati.

**Solo se giustificato da casi reali:** una nota locale di revisione `data_review` con `id`, `collector_id`, `opened_at`, `closed_at`, `request_kind`, `decision_kind` e una descrizione minima **facoltativa**. Non memorizzare automaticamente copia della richiesta originale o documenti identificativi, né status giuridici come fatti accertati. Definire esplicitamente cosa accade se il contatto viene eliminato: nessuna scheda orfana che rivelerebbe nuovamente nome o recapiti. Schema e API qui sono soltanto ipotesi, senza migrazioni approvate.

- **Offline-first:** nessuna connessione, account Repertica o server; nessuna scansione di servizi cloud o cartelle esterne all'insaputa dell'utente.
- **Portabilità:** tutte le eventuali nuove note di revisione andrebbero incluse nel backup completo e nei futuri formati aperti **se realmente implementate**, mantenendo l'esportazione integrale disponibile. Non modificare i 12 CSV attuali per effetto di una vista selettiva; un eventuale export per soggetto sarebbe un *documento aggiuntivo*, non un backup.
- **Backup e copie:** le modifiche del database corrente non cancellano né riscrivono backup preesistenti, CSV, PDF o file già consegnati. Indicare chiaramente l'impatto delle politiche di conservazione delle copie e l'eventuale rischio di reintroduzione tramite restore, senza cancellare automaticamente backup scelti dall'utente.
- **Compatibilità:** documentare limiti dei database esistenti, certificati immutabili, referenze e archivi v1–vN; nessun requisito di migrazione nella prima fase, nessuna trasformazione dei nomi degli acquirenti in pseudonimi senza valutazione esplicita.

## 7. Privacy, sicurezza e responsabilità

I dati personali possono essere presenti anche nelle fotografie e in testo libero, non solo nei campi anagrafici. Una ricerca automatica non può garantire di trovare qualsiasi riferimento. Gli archivi e i backup in chiaro rimangono sotto responsabilità dell'operatore. Non aggiungere log dettagliati contenenti vecchi valori rimossi (rischio di duplicazione del dato), non caricare report a terzi e non esporre recapiti al server MCP, che oggi li esclude deliberatamente.

**Contesto normativo, non consulenza legale:** il GDPR prevede principi di minimizzazione e accuratezza e diritti di rettifica/cancellazione con condizioni ed eccezioni; non prescrive una cancellazione universale di qualunque dato al primo clic. Riferimenti ufficiali: [Regolamento (UE) 2016/679, artt. 5, 16 e 17](https://eur-lex.europa.eu/eli/reg/2016/679/oj/eng/) e [Commissione europea — Principi del GDPR](https://commission.europa.eu/law/law-topic/data-protection/information-business-and-organisations/principles-gdpr_en). La determinazione del ruolo del titolare, della base giuridica e dei tempi di conservazione richiede valutazione specifica dell'utilizzatore e, quando opportuno, assistenza professionale.

## 8. Impostazioni, lingue e accessibilità

Nessuna attivazione automatica o promessa di compliance. Eventuale preferenza per mostrare promemoria di revisione è **disattivata per impostazione predefinita**, senza cronologia di richieste imposta a tutti. UI in lingua di sistema (IT/EN/ES/FR/DE supportate), fallback EN per lingua non supportata e singole stringhe mancanti, come già previsto; contenuti del contatto e documenti storici non vengono tradotti automaticamente. Date mostrate secondo la localizzazione senza cambiare i valori persistiti.

Tastiera e screen reader: sezioni con titoli espliciti, etichette non basate su colori, focus che ritorna al punto di origine, annunci delle modifiche riuscite/fallite, nessuna finestra modale interminabile, nessuna esportazione o cancellazione attivabile involontariamente da un unico pulsante.

## 9. Complessità, dipendenze, rischi, alternative

- **Complessità:** bassa–media per il solo pannello riepilogativo; media–alta per un export attendibile del singolo soggetto e per tutte le relazioni storiche; molto alta e *non proposta ora* per un sistema di cancellazione/anonimizzazione end-to-end con backup e documenti immutabili.
- **Dipendenze possibili:** UI collezionisti e collegamenti già presenti; sicurezza dei comandi CRUD; future proposte di export minimizzato e interoperabilità solo se approvate separatamente. Nessuna dipendenza obbligatoria da servizi esterni.
- **Rischi:** falsa promessa di adempimento GDPR; perdita di prove e riferimenti nelle vendite; oscuramento indebito degli originali; esportazione accidentale di dati di terzi; reintroduzione tramite vecchi backup; eccesso di dati nei log della revisione; sovraccarico UX per artisti senza casi d'uso.
- **Alternativa senza sviluppo (prima scelta di prova):** scheda attuale + modifica/svuotamento manuale dei campi, cancellazione del contatto senza vendite, ispezione dei registri collegati e checklist scritta in ODT. Una guida nella documentazione potrebbe bastare, senza aggiungere nuove tabelle o comandi.
- **Fuori perimetro:** pareri legali, retention automatica, anonimizzazione irreversibile, modifica retroattiva di certificati, cancellazione di backup esterni, email a interessati, nuovi ruoli amministrativi e nuova rubrica.

## 10. Criteri di accettazione da verificare *solo se approvata in futuro*

1. Per un contatto con campi facoltativi e fotografie, la vista elenca correttamente i campi presenti senza rivelare il contenuto ad altri strumenti.
2. Su contatto privo di vendite il comando di cancellazione esistente continua a funzionare; su contatto con vendita la restrizione rimane, senza perdita di integrità relazionale.
3. Correggere email e indirizzo non cambia automaticamente vendite, ripartizioni, ricevute e snapshot di certificati.
4. I riferimenti ai documenti storici sono descritti come tali, con differenza fra dato attuale e dato congelato; nessun testo «cancellato ovunque».
5. Nessun backup esistente viene riscritto o eliminato; l'interfaccia segnala che il restore di una copia vecchia può ripristinare vecchi valori.
6. Nessuna modifica avviene aprendo il pannello o scorrendo il riepilogo; azioni distruttive richiedono conferma chiara.
7. Il percorso funziona interamente offline in tutte le lingue supportate con fallback inglese, focus e annunci accessibili.
8. Un eventuale prospetto condivisibile esclude per default altri soggetti, importi, note e documenti riservati; un revisore umano deve verificare l'anteprima.
9. I registri CSV completi, la possibilità di backup in cartella libera e i dati dell'archivio rimangono integri ed esportabili.
10. Prove con dati sintetici e professionisti dimostrano un vantaggio rispetto alla checklist manuale, prima di qualsiasi sviluppo.

## 11. Fasi ipotetiche e priorità

- **Fase 0 — ricerca, senza codice:** intervistare 4–6 artisti/galleristi con casi simulati di correzione e rimozione dati; osservare se le attuali schede e una guida bastano. Confrontare pratiche di conservazione realmente applicate, senza attribuire obblighi indistinti.
- **Fase 1 — eventualmente guida o vista solo lettura:** elencare campi e legami storici, con rimando alle azioni esistenti e avvisi non allarmistici. Nessuna modifica schema.
- **Fase 2 — solo se utile e approvata:** prospetto minimo per singolo soggetto e controlli di condivisione; verificare prima fattibilità e rischio dati terzi.
- **Fase 3 — non raccomandata senza requisiti legali specifici:** tracciamento facoltativo delle richieste o forme avanzate di trattamento, mantenendo intatti certificati e registri autorevoli.

**Priorità condizionale P2** quando le interviste dimostrano errori o lavoro ripetitivo; **P3 / nessun sviluppo** se le procedure già possibili e una guida soddisfano il bisogno.

## 12. Requisiti certi, ipotesi e domande per il proprietario

**Certi (principi di prodotto e codice consultato):** software Tauri gratuito/open source e offline-first, nessun account Repertica o abbonamento, SQLite e media locali, export completo, backup in cartella liberamente scelta, lingua sistema/fallback EN; modifica dei contatti e blocco cancellazione se collegati a una vendita; certificati con dati storici immutabili.

**Ipotesi non verificate:** richieste frequenti di rettifica/cancellazione, necessità di una UX più guidata rispetto alle schede esistenti, utilità di un prospetto per soggetto, importanza di una cronologia locale delle decisioni. Nessuna violazione riscontrata.

**Questioni aperte al proprietario (nessuna risposta presunta):**
1. Gli utenti ricevono davvero richieste di eliminazione o correzione? Su quali campi e con quale frequenza?
2. I campi opzionali di nascita sono necessari nell'uso professionale di Repertica? Possono essere semplicemente lasciati vuoti?
3. È sufficiente una guida pratica, o il rischio di modificare il contatto sbagliato giustifica una schermata dedicata?
4. Quali record/documenti storici devono poter essere consultati senza essere modificati, in funzione dei diversi utilizzi professionali?
5. Quali informazioni possono comparire in un eventuale prospetto per la persona interessata senza coinvolgere altri soggetti?
6. Come va spiegato il rapporto con copie di backup, vecchie esportazioni e documenti già consegnati, senza promettere cancellazioni che il programma non può eseguire?

_Questo documento non autorizza sviluppo, approvazioni o merge. Non è una dichiarazione di conformità giuridica._
