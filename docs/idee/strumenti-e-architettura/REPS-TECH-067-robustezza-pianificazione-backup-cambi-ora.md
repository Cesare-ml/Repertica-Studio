# REPS-TECH-067 — Backup programmati e cambi di ora, fuso e orologio di sistema

- **ID stabile:** REPS-TECH-067
- **Data:** 2026-11-23
- **Stato:** **Proposta — da valutare**
- **Filone:** strumenti e architettura, affidabilità dell'esecuzione locale
- **Priorità proposta:** **P2 solo se scenari reali riproducono omissioni o indicazioni fuorvianti;** P3 per sola documentazione/QA negli altri casi. Valutare P1 esclusivamente davanti a una perdita di protezione effettiva e riproducibile.
- **Natura:** analisi e specifica ipotetica, **non decisione, approvazione, implementazione o incarico**.

## 1. Problema, destinatari e riscontro diretto

**Problema da verificare.** La persona sceglie per i backup automatici un'ora locale e una cadenza giornaliera, settimanale o mensile. L'orologio del dispositivo, tuttavia, può saltare avanti o indietro: passaggio ora solare/legale, correzione NTP, cambio manuale della data, fuso diverso dopo un viaggio o ripristino su un nuovo PC. Un'ora locale può non esistere, comparire due volte oppure essere temporaneamente sbagliata. Per un archivio di opere e fotografie l'obiettivo non è l'esattezza al secondo ma **non lasciare credere che una copia periodica sia stata eseguita quando non lo è stata** e non creare duplicazioni inaspettate.

**Fatti letti sul branch `main`, il 23 novembre 2026:**

- [`src-tauri/src/auto_backup.rs`](../../../src-tauri/src/auto_backup.rs) offre già pianificazione **disattivata per impostazione predefinita**, frequenze `DAILY`, `WEEKLY`, `MONTHLY`, orario `HH:MM`, recupero di una scadenza mancata alla successiva occasione, riprova di un errore dopo 30 minuti, un solo tick al minuto mentre l'app è aperta. Il sistema esegue un backup completo già verificato; **non si propone di sostituirlo**.
- `Schedule::previous_occurrence` e `next_occurrence` calcolano date con `NaiveDateTime`, mentre `start`, `get_backup_schedule`, `save_backup_schedule` e `run_backup_now` passano `Local::now().naive_local()`. `last_success_at`, `last_attempt_at` ed `enabled_at` sono confrontati come date/ore locali senza informazione di offset; il formato `STAMP` è `%Y-%m-%dT%H:%M:%S`.
- `is_due` considera la massima fra data di abilitazione e ultimo successo e usa `now < last_failure + 30 minuti` per il retry. `record_outcome` impedisce già a un esito più vecchio di sovrascrivere uno successivo in base a timestamp e sequenza in-process. Queste protezioni vanno preservate.
- Il file contiene test per frequenze, inizio/riattivazione, ritardo, retry, concorrenza e retention. **Nel percorso letto non emerge una politica esplicita** per ore locali mancanti o ripetute, fuso cambiato e correzioni arbitrarie dell'orologio, né un test che simuli quei passaggi. Questo **non dimostra un errore in produzione**; non è stato eseguito il programma né provato un cambio di ora reale.
- [`docs/MVP.md`](../../MVP.md) chiarisce che l'app non è un servizio di sistema e che i backup automatici non avvengono da chiusa. Questa limitazione resta una premessa, non un difetto da correggere con un servizio obbligatorio.

**Destinatari e scenari:** artisti che lasciano acceso Repertica durante la notte; gallerie che usano un disco esterno per copie periodiche; professionisti che viaggiano e cambiano fuso; studi con orologio di sistema corretto dopo un avvio errato; persone che ripristinano l'archivio su un altro computer. Il bisogno resta **ipotetico** finché non emergono logiche effettivamente problematiche o casi osservati.

## 2. Confini e confronto anti-duplicazione

- [TECH-060](REPS-TECH-060-identita-destinazione-backup-programmati.md): *identità fisica della cartella/volume selezionato*; qui soltanto **identità temporale della scadenza** e comportamento quando cambia l'orologio.
- [TECH-066](REPS-TECH-066-compatibilita-filesystem-destinazioni-backup.md) e [TECH-022](REPS-TECH-022-preflight-spazio-operazioni-locali.md): capacità del filesystem/spazio, non pianificazione.
- [TECH-062](REPS-TECH-062-progresso-annullamento-sicuro-operazioni-locali.md): stato delle operazioni lunghe, non scelta del momento in cui devono iniziare.
- [TECH-009](REPS-TECH-009-catalogo-verifiche-conservazione-backup.md), [TECH-052](REPS-TECH-052-verifica-indipendente-backup-formato-v1.md), [TECH-058](REPS-TECH-058-ricognizione-post-ripristino-nuovo-computer.md): copie prodotte, verifica e recupero su un nuovo PC; nessuna tratta precisamente le **ore locali non univoche durante lo scheduler**.
- Nell'[indice Gestione e UX della PR #14](https://github.com/Cesare-ml/Repertica-Studio/blob/docs/idee-gestione-ux-20261121/docs/idee/gestione-e-ux/README.md) ci sono scadenze commerciali e flussi amministrativi, non una specifica del tempo civile dei backup. **Nessun documento di quell'altra automazione va modificato.**

**Esclusioni:** niente cloud, invio dati, notifiche remote, servizio che continua ad agire quando l'app è chiusa, sincronizzazione tra PC, nuova retention, nuovo formato di backup, cancellazione di copie, cambio automatico della cartella, o sincronizzazione dell'orologio del sistema.

## 3. Semantica proposta (da decidere, non comportamento esistente garantito)

La prima attività **senza sviluppare nuove funzioni** dovrebbe essere una tabella di casi e test di comportamento in ambiente controllato; solo dopo scegliere una politica. Una possibile convenzione prudente:

1. **Ora locale inesistente (salto primaverile):** se la scadenza scelta non esiste, tentare **una sola volta** non appena il dispositivo torna a un orario valido e l'app è in esecuzione; dichiarare in cronologia il motivo del ritardo. Alternativa da confrontare: saltare quella scadenza e segnalarla chiaramente. Non fingere un'esecuzione all'ora inesistente.
2. **Ora locale ripetuta (salto autunnale):** una scadenza logica giornaliera/settimanale/mensile dovrebbe produrre **al massimo una esecuzione dovuta per il giorno**, pur esistendo due istanti fisici. Un backup manuale aggiuntivo rimane sempre una nuova azione volontaria.
3. **Orologio spostato in avanti e poi all'indietro:** se un successo è registrato con data futura rispetto all'orario corretto, evitare sia di sopprimere silenziosamente copie per molti giorni sia di iniziare una raffica di backup. Evidenziare che la cronologia temporale è incoerente; in una fase successiva valutare una riconciliazione conservativa di singola scadenza, senza cancellare o correggere i timestamp storici.
4. **Retry dopo un errore:** i 30 minuti di attesa dovrebbero corrispondere, se tecnicamente possibile, a tempo effettivamente trascorso nella sessione, senza retry immediati dopo il salto avanti né attese indefinite dopo il salto indietro. Dopo riavvio, applicare una politica documentata con controlli conservativi, non un contatore monotono non persistibile.
5. **Cambio di fuso:** definire apertamente se «20:00» significhi **20:00 dell'orologio locale del computer, anche dopo il viaggio** (possibile prima scelta, fedele all'attuale UX), oppure un fuso fisso che l'utente conferma. Evitare che il ripristino su un altro PC riattivi implicitamente backup su un fuso non atteso.
6. **App chiusa/sospesa:** resta una singola esecuzione di recupero alla prossima occasione, se dovuta, senza accumulo di una copia per ciascuna scadenza saltata. Il backup «Esegui ora» deve rimanere disponibile e non essere confuso con una scadenza già soddisfatta.

La politica, una volta scelta, deve essere **deterministica e comprensibile all'utente**, non un generico «gestiamo automaticamente il DST».

## 4. Flussi UX ipotetici e accessibilità

**Configurazione:** «Ogni giorno alle 02:30 (ora locale del dispositivo)» con un breve chiarimento raggiungibile da tastiera: «Se l'app è chiusa o l'ora non esiste, la copia può avvenire più tardi». La schermata mostra **ultima copia riuscita, ultimo tentativo e prossima occasione stimata**, etichettando esplicitamente eventuali orari incerti e i backup ancora mancanti. Non annunciare «backup riuscito» per una sola pianificazione attiva.

**Anomalia di orologio:** avviso sobrio, non bloccante per le altre funzioni: «L'orario di questo computer è cambiato: verifica la prossima copia automatica». Azioni: «Controlla impostazioni», «Esegui ora» e «Chiudi avviso». Nessun intervento sull'orologio di sistema, nessuna modifica silenziosa a date, saldi, certificati o storico dell'archivio.

**Assenza di rischio osservato:** se i test dimostrano che la semantica attuale è già sufficiente, **preferire una spiegazione in documentazione e un messaggio più preciso**, senza nuova UI, dati o impostazioni.

**Accessibilità e lingue:** stato comprensibile senza colore; focus e annunci accessibili; localizzazione automatica dalla lingua del sistema e fallback inglese, scelta manuale prevista; testi «ora locale», «tentativo», «riuscito», «mancato» e «incerto» traducibili nelle lingue supportate; mostrare timestamp localizzati senza perdere l'istante non ambiguo nelle informazioni tecniche.

## 5. Strutture dati e API illustrative, solo descrittive

È possibile valutare un modello **separato dalla struttura del backup**, senza introdurre alcuna tabella o migrazione finché non serva davvero:

```text
SchedulePolicy {
  cadence: daily | weekly | monthly
  local_time: HH:MM
  time_basis: device_local | fixed_zone_confirmed
  nonexistent_local_time: run_at_next_valid_opportunity | skip_with_notice
  repeated_local_time: once_per_logical_slot
}
ScheduledSlot {
  policy_revision: opaque_id
  local_calendar_date: YYYY-MM-DD
  requested_local_time: HH:MM
  zone_context: optional zone_or_offset_if_available
  chosen_utc_instant: optional timestamp
}
AttemptRecord {
  slot_id: optional ScheduledSlot
  began_at_utc: timestamp
  local_time_observed: string
  outcome: success | failure | unknown
  monotonic_retry_marker: optional session_only_value
}
ClockAnomaly {
  kind: ambiguous_hour | missing_hour | backward_jump | forward_jump | changed_zone | unknown
  confidence: observed | suspected
}
```

Possibili contratti **non esistenti**: `explainNextBackup(schedule, clockContext, lastAttempts) -> ScheduleExplanation`; `evaluateDueSlot(schedule, now, previousState) -> DueDecision`; `getClockAnomalyStatus() -> Status`. Le API devono separare **identità della scadenza**, **inizio tentativo**, **esito verificato** e **tempo civile mostrato**. L'uso di UTC per ordinare tentativi **non sostituisce** una regola per il giorno/ora locale scelti.

**Compatibilità:** i valori persistiti nel database attuale sono timestamp locali senza offset: non possono essere convertiti retroattivamente in istanti UTC certi. Eventuale evoluzione non deve inventare offset storici; mantenere i valori legacy e identificarli come ambigui, senza alterare log, copia di backup o importi. I `next_run_at` preesistenti restano mostrabili fino alla decisione su una nuova versione del contratto. Il formato del backup completo deve restare autosufficiente e portabile.

## 6. Offline, sicurezza, privacy, impostazioni e dipendenze

- **Funzionamento:** tutto offline, tramite l'orologio e i servizi locali del sistema; nessuna verifica NTP forzata, richiesta online, login, account o abbonamento. Una destinazione NAS resta soltanto una cartella già montata dall'OS.
- **Privacy:** cronologia e segnali di anomalia temporale salvati solo localmente; potrebbero rivelare abitudini e orari di attività. Nessuna telemetria, nessun export automatico, nessun log di file, percorsi o nominativi verso terzi.
- **Impostazioni:** mantenere frequenze e orario correnti, backup disabilitato per default e possibilità di qualunque cartella; evitare opzioni difficili da spiegare. Se una preferenza sul fuso fosse necessaria, è **opt-in e da confermare**.
- **Sicurezza dei dati:** una correzione dell'orologio non può autorizzare eliminazioni, ripristini o «recuperi» distruttivi; le salvaguardie del backup verificato, la retention conservativa e la gestione della concorrenza restano intatte.
- **Dipendenze:** prima analizzare gli strumenti già presenti (Rust, `chrono`, SQLite, timer Tauri). Una libreria IANA cross-platform o API temporali del sistema sono soltanto **alternative da valutare** dopo prove su Windows, macOS e Linux; nessuna dipendenza o configurazione viene introdotta da questa scheda.
- **Complessità:** bassa per matrice di test e documentazione; media per semantica esplicita e codici di stato; medio-alta per nuove persistenze e compatibilità delle cronologie legacy fra zone e migrazioni.

## 7. Alternative e rischi

**A — Preferita inizialmente:** non sviluppare nulla, verificare i casi e documentare il comportamento già esistente; adottare solo testi più precisi se bastano. **B:** mantenere `NaiveDateTime` ma aggiungere regole esplicite di anomalia e robustezza del retry senza migrazione del database. **C:** identità persistente delle scadenze e tempi assoluti, con eventuale fuso IANA e migrazione compatibile: scegliere solo se la B non garantisce risultati verificabili. **D:** imporre solo orari diurni «sicuri»: facile, ma limiterebbe ingiustificatamente la libertà d'uso e non risolverebbe l'orologio errato.

**Rischi:** duplicare copie occupando spazio e causando retention inattesa; saltare backup senza avviso; mostrare la «prossima esecuzione» a un'ora civile non esistente; confondere cambi di fuso volontari con orologi difettosi; rinviare retry per ore o giorni dopo salto all'indietro; falsi positivi che rendono allarmante l'app. Le mitigazioni richiedono codici di stato verificabili, limiti di un recupero per occasione e **nessuna cancellazione di backup fondata su timestamp ambiguo**.

## 8. Matrice di prove e criteri di accettazione proposti

Verifiche suggerite (non eseguite e non autorizzate come sviluppo):

1. Simulazione su fuso **Europe/Rome** di **02:30 non esistente** al salto primaverile e **02:30 ripetuta** al salto autunnale; aspettarsi secondo la politica scelta una singola decisione esplicita e mai due esecuzioni automatiche per lo stesso slot logico.
2. Simulare correzioni di orologio di **+48 ore e poi −48 ore**, fuso locale cambiato, ripresa da sospensione e riapertura dell'app; nessun rinvio silenzioso di giorni presentato come normale riuscita.
3. Errore su disco scollegato prima e durante cambio di ora, retry dopo 30 minuti effettivi quando misurabili; nessun ciclo rapido e nessuna attesa apparentemente infinita.
4. Due tentativi sovrapposti (manuale e automatico) che terminano in ordine inverso; non degradare le protezioni già presenti in `record_outcome`.
5. Cronologia legacy priva di offset, ripristino su altro PC/fuso, aggiornamento/downgrade della app: niente UTC inventato, sovrascritture storiche o perdita di backup.
6. Prove native Windows/macOS/Linux con orologio simulato/isolato, dati sintetici, unità locale e rimovibile; nessuna operazione sui backup reali e nessun cambiamento all'orologio del computer dell'utente.
7. Messaggi e orari comprensibili da screen reader, nelle lingue disponibili, con fallback inglese e possibilità di avviare comunque una copia manuale.

**Accettazione minima:** il numero di copie dovute, tentate e riuscite è spiegabile; anomalie sono visibili e verificabili; per ogni scadenza logica non si hanno duplicati involontari; una mancata esecuzione non è spacciata per successo; dati e backup esistenti restano integri. **La verifica delle copie, non il solo scheduler, resta l'autorità sull'effettiva riuscita.**

## 9. Fasi possibili e decisioni riservate al proprietario

- **Fase 0 — sola ricerca:** ricostruire le decisioni attuali, creare una matrice di scenari temporali sintetici e chiedere a 3–5 professionisti se usano davvero backup a orari notturni, in viaggio o su dispositivi con data incerta. Se nessun bisogno o rischio reale emerge, archiviare l'idea.
- **Fase 1 — eventuale specifica minima:** formalizzare la politica delle ore ripetute/mancanti e del retry e proporre QA/documentazione. Nessuna approvazione implicita a modificare codice.
- **Fase 2 — soltanto se necessario:** confrontare miglioramenti limitati con un modello di slot persistenti e compatibilità del vecchio storico. Separare valutazione dell'interfaccia da eventuali decisioni sul database.
- **Fase 3 — collaudo su dispositivi reali:** validare sospensione, cambio fuso, dischi esterni e ripristino, prima di considerare la proposta pronta per una decisione.

**Domande aperte per revisione personale:**

1. La cadenza deve seguire sempre l'ora corrente del dispositivo o una zona scelta dall'utente?
2. Per le ore mancanti è preferibile recuperare subito alla prima ora valida o saltare con avviso?
3. Quando un orologio viene corretto all'indietro di giorni, quale recupero prudente è comprensibile senza produrre raffiche?
4. Che livello di cronologia dei tentativi sarebbe utile senza creare un registro invasivo?
5. Bastano QA e una spiegazione chiara oppure esiste un problema ripetuto che giustifichi modifiche al scheduler?

**Esito della ricognizione:** proposta **non approvata**, non assegnata, da confrontare con bisogni e collaudi reali. Non modifica il calendario commerciale, le copie esistenti, i file dell'altra automazione o il codice applicativo.
