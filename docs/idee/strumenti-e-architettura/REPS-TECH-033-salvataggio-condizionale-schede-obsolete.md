# REPS-TECH-033 — Salvataggio condizionale delle schede: impedire sovrascritture da dati obsoleti

- **ID stabile:** REPS-TECH-033
- **Data:** 2026-11-22
- **Stato:** **Proposta — da valutare**
- **Priorità proposta:** **P1–P2 condizionata** alla frequenza e gravità dei conflitti reali; **non approvata**
- **Ambito:** integrità tecnica dei salvataggi di metadati già presenti in SQLite (prima opere e collezionisti), non cronologia completa, collaborazione online, sincronizzazione o gestione degli incarichi

## 1. Motivazione, utenti e riscontro verificabile

Un artista può tenere aperta una scheda d'opera mentre registra una vendita; una galleria può modificare il medesimo record in due finestre; dopo un'operazione in un altro pannello la maschera precedente può conservare valori non aggiornati. Il successivo «Salva» non dovrebbe sostituire senza avviso le modifiche apportate nel frattempo. È soprattutto una **prevenzione della perdita di dati**, prima di immaginare sistemi di versionamento o collaborazione.

**Destinatari:** artisti individuali che lavorano con più finestre o pannelli, studi con cataloghi estesi, gallerie, archivisti che correggono schede e professionisti che tengono aperta un'anagrafica durante altre operazioni. Il prodotto rimane **desktop gratuito e open source, offline-first, senza account o abbonamento**; questa esigenza non richiede utenti multipli con ruoli né un server.

**Fatti accertati mediante lettura di `main` il 22 novembre 2026:**

- [Sicurezza modifica archivio](../../ARCHIVE-EDIT-SAFETY.md) documenta esplicitamente la protezione **già implementata** contro il reinserimento di opere/collezionisti eliminati, ma dichiara come limite l'assenza del controllo di concorrenza ottimistica per record **ancora esistenti**.
- [commands.rs](../../../src-tauri/src/commands.rs), funzione `save_artwork_core`, verifica che l'UUID esista e i vincoli di vendita, poi esegue un `UPDATE artworks ... WHERE id=?1` senza condizionare la scrittura alla versione caricata nella maschera; `record_sale` e `delete_sale` possono cambiare lo stato dell'opera. Il mutex e la transazione serializzano le operazioni, **non** rilevano da soli l'obsolescenza del formulario.
- [collectors.rs](../../../src-tauri/src/collectors.rs), funzione `save_collector_core`, esegue un `UPDATE collectors ... WHERE id=?1` e verifica le righe aggiornate per distinguere il record eliminato. Non confronta il contenuto letto in precedenza con la revisione corrente.
- [models.ts](../../../src/app/models.ts) espone `updated_at` sulle schede ricevute, ma non un token di revisione negli input `ArtworkInput` e `CollectorInput`. In [app.component.ts](../../../src/app/app.component.ts), `editArtwork` e `editCollector` copiano i campi in una bozza; `saveArtwork` e `saveCollector` inviano la bozza senza token di versione.
- Esiste già un **precedente locale utile, non da duplicare**: [business_mail.rs](../../../src-tauri/src/business_mail.rs) usa un campo intero `revision` nelle bozze email e rifiuta il salvataggio obsoleto. Non significa che le schede d'opera abbiano già questa protezione.

**Cosa non è stato dimostrato:** nessun incidente di perdita dati osservato su un desktop installato, nessuna prova di finestre concorrenti realmente in uso, nessuna analisi completa di ogni mutazione trasversale (immagini, media, vendita, import). È una lacuna dedotta dal codice e dalla documentazione, non una segnalazione utente misurata.

## 2. Confini e confronto anti-doppione

- [TECH-010 — Storico revisioni metadati](REPS-TECH-010-storico-revisioni-metadati.md) studia **conservazione, confronto e recupero del passato** e cita un possibile token per il ripristino. Qui lo scopo è **impedire una scrittura stantia prima che avvenga**, anche **senza** memorizzare una cronologia: progettazione e validazione possono essere indipendenti. Se TECH-010 fosse scelta, riutilizzare lo **stesso** identificatore di revisione, non introdurne due.
- [TECH-012](REPS-TECH-012-migrazioni-archivio-preflight-copia-sicurezza.md) protegge **aggiornamenti dello schema**; [TECH-009](REPS-TECH-009-catalogo-verifiche-conservazione-backup.md) verifica **backup**; [TECH-001](REPS-TECH-001-pacchetto-dati-aperto.md) tratta **interoperabilità**. Nessuno sostituisce la condizione atomica su una singola scrittura.
- La [PR Gestione e UX #14](https://github.com/Cesare-ml/Repertica-Studio/pull/14) propone con **UX-018** catalogazione seriale e bozze di nuove schede, con **UX-014** una rubrica professionale e con **UX-013** rettifiche post-vendita; non tratta il contratto transazionale di concorrenza su modifiche già aperte. Nessun loro documento viene modificato.
- Non si propongono blocchi pessimistici duraturi, editing simultaneo condiviso, CRDT, sincronizzazione, salvataggi automatici, audit legale o obbligo di identificare altri utenti.

## 3. Scenari d'uso e flussi UX descrittivi

**A — Titolo modificato altrove.** Si apre «Opera A» con versione V7; un altro percorso salva il nuovo titolo come V8. La prima maschera, ancora su V7, prova a salvare la tecnica: **nessun campo viene scritto**, resta visibile il contenuto inserito e un messaggio spiega che la scheda è cambiata. Azioni: «Confronta» (eventuale fase successiva), «Ricarica dati aggiornati» dopo avviso su modifiche locali e «Continua a modificare questa bozza». Non prevedere «Sovrascrivi comunque» come default.

**B — Vendita registrata mentre la scheda è aperta.** Una maschera precedente mostra «disponibile»; una vendita valida aggiorna lo stato a «venduta». Un successivo salvataggio di descrizione dalla maschera obsoleta non deve riportare implicitamente la scheda a «disponibile», né riscrivere quote, incassi o certificati. Le regole Rust sulle vendite restano autorevoli; l'eventuale errore di conflitto è distinto da un errore di validazione della vendita.

**C — Collezionista con dati sensibili.** Cambia l'indirizzo nella scheda corrente mentre una finestra precedente mostra il vecchio. Al salvataggio, la seconda finestra conserva la bozza senza sovrascrivere e, prima del confronto, non esporta automaticamente nomi, indirizzi, email o note. Nessuna notifica o email viene inviata.

**D — Record eliminato/ripristino.** Se l'UUID non esiste più, mantenere l'errore «Scheda non trovata»: il dato non viene ricreato. Dopo un ripristino integrale, ogni bozza aperta nel precedente contesto deve essere invalidata anche se un contatore locale coincide; il cambio archivio (se introdotto) invalida sempre il token.

**E — Modifiche non conflittuali.** Se il record corrente è ancora V7, l'utente salva normalmente, ottiene V8 e l'editor aggiorna la propria versione. Un clic doppio, una richiesta ritrasmessa o una finestra con V7 non devono ripetere un UPDATE incontrollato.

**UX minimale (prima fase):** Apri scheda → memorizza revisione caricata → modifica → «Salva» → *salvata* **oppure** *scheda cambiata, bozza conservata* → scelta manuale di come proseguire. La fase di confronto mostra differenze tra «quando hai aperto», «attuale» e «tua bozza» con etichette accessibili; **nessuna fusione automatica** di campi testuali, importi o stati.

## 4. Specifica funzionale e invarianti (ipotesi, non requisiti approvati)

**Requisiti certi del prodotto:** UUID invarianti, vincoli economici/snapshot dei certificati, validazioni Rust, transazioni SQLite, archivio locale e backup completi verificati a cartella scelta. I dati restano esportabili; non dipendono da account né cloud.

**Opzione tecnica preferibile da valutare:** versione intera **monotona per record**, distinta dal timestamp visualizzato. Un confronto del solo `updated_at` è più leggero ma può essere ambiguo se più scritture hanno timestamp uguali, se dati sono importati o se non tutte le operazioni aggiornano correttamente l'orologio. Non approvare comunque una migrazione senza esame di tutti i percorsi di scrittura.

Ipotesi di dati e API **soltanto descrittive**:

~~~text
Artwork { id, ..., revision: integer_non_negative, updated_at }
Collector { id, ..., revision: integer_non_negative, updated_at }

saveArtworkIfCurrent(
  input: ArtworkInput,
  expectedRevision: integer,
  archiveContext?: opaqueLocalGeneration
) -> SavedArtwork(newRevision) | Conflict(latestMetadata) | NotFound | ValidationError

saveCollectorIfCurrent(input, expectedRevision, archiveContext?)
  -> SavedCollector(newRevision) | Conflict(latestMetadata) | NotFound | ValidationError
~~~

- Il backend verifica versione, identità, appartenenza all'archivio attivo, vincoli e **cambio del record nella medesima transazione**. Esempio di semantica (non patch SQL): `UPDATE ... WHERE id = :id AND revision = :expected` con incremento atomico; se zero righe, distinguere *eliminato* da *cambiato*. **Non** limitarsi a un GET lato UI seguito da UPDATE incondizionato.
- Ogni percorso **autorizzato** che modifichi campi monitorati deve incrementare la stessa versione: modifica ordinaria, cambio di stato collegato a vendita/resi, operazioni accessorie che alterano quei campi. Mappare prima i percorsi esistenti: un solo writer che non aggiorna la versione annulla la garanzia. Non cambiare la semantica delle emissioni già congelate.
- Distinguere *revision dell'entità* da *versione schema*, *timestamp*, *snapshot certificato*, *revisione media*, *revision della bozza email* e *generazione del contesto archivio*. Una numerazione uguale dopo un ripristino non è prova che la bozza provenga dallo stesso archivio: valutare invalidazione in memoria/identificatore di istanza.
- Record esistenti: un'eventuale migrazione potrebbe inizializzare il contatore a 1 come **versione all'adozione del meccanismo**, senza inventare una storia. Per nuove schede versioni definite e documentate. Non assumere riscrivibili backup di schemi più recenti o versioni di documenti esterni.
- Nessun merge tra finestre: in una fase opzionale l'utente può applicare **solo campi selezionati** dopo confronto e una **nuova** verifica della versione corrente, non salvare un intero record vecchio. Se la comparazione riguarda nomi e contatti, limitarla alla singola UI locale.

**Requisiti da valutare con cautela:** se un cambiamento delle sole fotografie, didascalie, rate o bozze di brochure debba invalidare l'editor metadati. Meglio definire una matrice per campo e per transazione anziché invalidare ogni modifica irrilevante. La protezione deve essere **coerente** prima di essere granulare.

## 5. Offline, privacy, sicurezza, impostazioni, multilingua e interoperabilità

- Totalmente locale: nessun endpoint, server di rete, account, telemetria o dialogo con IA. Il server MCP STDIO **attuale è in sola lettura**; non dedurre dal progetto un accesso multiutente in scrittura. La bozza resta nella memoria dell'interfaccia finché la finestra è aperta, non viene pubblicata né inviata fuori dal computer.
- Il messaggio di conflitto deve rivelare solo il minimo necessario, con eventuali dettagli sensibili disponibili **nella stessa scheda locale e dopo azione esplicita**, non nei log. Non introdurre per errore copie di contatti nelle diagnosi, nei CSV condivisi o nell'export filtrato.
- Impostazioni: **nessuna opzione da configurare nella prima fase**; l'integrità dei salvataggi non va disattivata con un toggle. Se utile, preferenza accessibile per conservare una bozza locale dopo conflitto, con cancellazione esplicita e nessun cloud. Non chiedere permessi di rete.
- Lingua: messaggi, pulsanti, errori e annunci per screen reader nella lingua del sistema (it/en/es/fr/de dove supportato), **fallback inglese**, eventuale override già disponibile; codici errore stabili non tradotti e orari formattati secondo locale. Supporto tastiera, focus coerente sul messaggio, nessuna chiusura della maschera né perdita di testo sul conflitto.
- Backup/ripristino: un eventuale `revision` nello schema SQLite va incluso nelle copie integrali e mantenuto nel ripristino, senza rigenerare versioni o applicare modifiche automatiche. Vecchi archivi migrati in modo transazionale e verificato; file CSV correnti restano utilizzabili (documentare eventuali nuove colonne/formati solo dopo scelta), dati e allegati originali sempre esportabili. Un export aperto futuro può riportare contatori come metadati tecnici, **non** come prova notarile.
- Compatibilità Windows/macOS/Linux; non servono componenti online o librerie di locking esterne. La strategia deve considerare richieste IPC duplicate, finestre concorrenti, dispositivi lenti e tentativi di scrittura esterni a SQLite (che non sono controllabili completamente dall'app).

## 6. Alternative, dipendenze e complessità

1. **Solo avviso UI se il valore è cambiato prima del Salva:** costo basso, ma vulnerabile alla race fra controllo e scrittura. Non è sufficiente come garanzia.
2. **Confronto condizionale su `updated_at`:** intervento più piccolo; richiede audit dei writer, serializzazione esatta dei timestamp e verifica delle collisioni/clock. Può essere una misura transitoria prudente.
3. **Contatore monotono per riga più condizione atomica:** più robusto e coerente con il precedente delle bozze email; richiede migrazione additiva, contratti Rust/Angular e revisione dei percorsi di scrittura. Non scegliere senza esaminare compatibilità e test.
4. **Storico completo (TECH-010) o lock persistenti:** costo e responsabilità nettamente superiori; **non indispensabili** per impedire il primo tipo di sovrascrittura.

**Dipendenze da analizzare:** mappa delle scritture in `commands.rs`, `collectors.rs`, vendite, Media Center e future importazioni; contratto di serializzazione IPC; schema/migrazioni SQLite e backup; messaggi i18n; flussi delle due maschere. Nessuna nuova dipendenza proposta.

**Complessità:** **M** per protezione delle due maschere se la matrice dei writer è piccola; **M–L** se molte operazioni trasversali aggiornano la stessa scheda o se si richiede differenza e fusione campo per campo. Rischi: falsi conflitti, writer dimenticati, bozze perse sulla UI, vecchi archivi non migrabili, concorrenza con restore/cambio archivio. Evitare «forza sovrascrittura» e conflitti silenziosi.

## 7. Criteri di accettazione e test da valutare, senza eseguirli ora

1. Due snapshot Vn della stessa opera: il primo salva come Vn+1; il secondo ottiene **conflitto** e il database conserva **tutti** i valori del primo, senza parziali. Niente successi doppi con Vn.
2. Stesso test con collezionista (email, indirizzo e note); nessun dato sensibile nei messaggi diagnostici.
3. Opera eliminata mentre la maschera è aperta: **NotFound**, mai INSERT implicito. Opera venduta nel frattempo: nessuna sovrascrittura dello stato e nessuna modifica a rate, quote, certificati o incassi.
4. Richieste concorrenti e ritentativi IPC: un solo vincitore, revisione incrementata una volta per scrittura accettata; rollback atomico se fallisce una validazione.
5. Percorsi alternativi di modifica dei campi (vendita e operazioni media dove rilevanti) passano dal controllo previsto e invalidano correttamente le vecchie bozze; nessun writer silenzioso.
6. Il messaggio di conflitto mantiene aperto l'editor, conserva il testo inserito e offre scelta senza sovrascrittura automatica; tastiera e lettore di schermo gestiscono dialogo e focus.
7. Migrazione da archivi storici, backup → ripristino → riapertura, riavvio della UI e cambio archivio eventuale: token e contesto corretti; nessun finto storico creato.
8. Verifiche native Windows/macOS/Linux e, dove applicabile, archivio su unità esterna/NAS montato; la protezione non presume che un file system sia un database concorrente distribuito.

**Nessun test è stato eseguito in questa ricognizione** e questi criteri non sono un incarico d'implementazione.

## 8. Possibili fasi (non approvate)

- **Fase 0 — convalida del problema:** audit dei writer e casi di concorrenza; capire quante finestre o maschere possono essere stantie. Confrontare il solo avviso UX con un controllo atomico.
- **Fase 1 — protezione minima:** versionamento condizionale delle modifiche metadati di opere e collezionisti, errori distinti, conservazione della bozza in UI; test di concorrenza e regressione.
- **Fase 2 — confronto assistito:** differenze fra bozza, base ed entità corrente con scelta manuale dei campi e nuovo controllo atomico; solo se utenti lo richiedono.
- **Fase 3 — riuso futuro:** il medesimo token potrebbe servire TECH-010 e importazioni assistite, senza introdurre nuovi motori duplicati.

## 9. Questioni aperte riservate alla revisione del proprietario

1. Gli utenti tengono concretamente più finestre o maschere aperte, oppure il problema nasce soprattutto da operazioni interne (vendite, fotografie, ricambio archivio)?
2. Preferire **immediato conflitto anche per modifiche a campi diversi** o una fase successiva con fusione manuale campo per campo?
3. In prima fase proteggere solo opere/collezionisti o anche spese, brochure e altre entità modificabili? Quali hanno davvero più percorsi di scrittura?
4. L'invalidazione della bozza dopo backup/ripristino deve chiedere solo di ricaricare oppure consentire recupero **manuale** dei testi non sensibili?
5. La priorità va al piccolo controllo preventivo oppure a TECH-010, se un vero storico revisioni risultasse più richiesto?

**Esito:** studio documentale soltanto. **Nessuna funzionalità approvata, implementata o assegnata**; nessun codice, build, workflow, impostazione, dipendenza o file Gestione e UX modificato.