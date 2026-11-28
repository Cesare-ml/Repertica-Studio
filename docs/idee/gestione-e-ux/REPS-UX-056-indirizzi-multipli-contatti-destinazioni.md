# REPS-UX-056 — Indirizzi multipli dei contatti, destinazioni dichiarate e recapiti storici senza automatismi

- **ID stabile:** REPS-UX-056
- **Data:** 2026-11-23
- **Stato:** Proposta — da valutare
- **Filone:** Gestione e UX
- **Priorità ipotetica:** P2 soltanto se artisti/gallerie usano frequentemente più indirizzi per lo stesso contatto; P3 o nessuno sviluppo se bastano note e conferma manuale al momento della consegna
- **Natura:** esclusivamente analisi documentale. Non approvata, non assegnata, non avviata; nessuna modifica proposta qui viene applicata al codice.

## 1. Problema e destinatari: fatti verificati, ipotesi e valore

**Riscontro nel branch `main`, 23/11/2026, via connettore GitHub (lettura statica, nessun test desktop):** [`src-tauri/src/collectors.rs`](../../../src-tauri/src/collectors.rs) definisce, nell'anagrafica `Collector` e nel suo input, **un solo** campo testuale facoltativo `address`, oltre a email, telefono, nome e note. [`src/app/models.ts`](../../../src/app/models.ts) espone l'indirizzo come singola stringa. [`src-tauri/src/export.rs`](../../../src-tauri/src/export.rs) e [`docs/DATA-PORTABILITY.md`](../../DATA-PORTABILITY.md) lo includono nel CSV `collectors`; i backup integrali verificati mantengono SQLite e media. Il [README](../../../README.md) descrive un'anagrafica locale per collezionisti, acquisti e contatti. **Questo accerta il limite di modellazione, non un malfunzionamento o una richiesta utente già provata.**

**Problema professionale ipotetico:** un collezionista può abitare in un luogo, ricevere corrispondenza presso uno studio professionale, preferire la consegna di un'opera in una casa diversa oppure cambiare residenza nel tempo. Sovrascrivere il campo unico fa perdere una destinazione ancora utile; accodare più indirizzi in `notes` non chiarisce a cosa servano, quale sia aggiornato e quale debba essere confermato per una spedizione. Le piccole gallerie possono avere recapito amministrativo e sede espositiva distinti. Un indirizzo in rubrica **non prova** domicilio legale, proprietà, consegna effettuata, consenso o recapito aggiornato.

**Destinatari da intervistare:** artisti che vendono direttamente a privati, studi con consegne ripetute, gallerie con referenti e sedi distinte. Per chi ha pochi clienti o consegna sempre a mano la funzione potrebbe essere sproporzionata.

**Requisiti certi del progetto (non requisiti già approvati della funzione):** desktop gratuito e open source, offline-first senza account/abbonamento; dati dell'utente locali, export portabile e backup con cartella scelta liberamente; lingua di sistema, fallback inglese. **Ipotesi da validare:** più indirizzi per soggetto apportano un vantaggio reale rispetto a un solo indirizzo e alla verifica manuale nelle note.

## 2. Confronto con proposte esistenti — confini per evitare doppioni

- **[REPS-UX-014 — Rubrica e relazioni professionali](REPS-UX-014-rubrica-relazioni-professionali.md)** studia soggetti, ruoli e interazioni trasversali. Questa scheda riguarda *solo la pluralità e l'uso dichiarato degli indirizzi* di un contatto già noto; non crea un CRM, un soggetto nuovo o una rubrica concorrente. Se UX-014 venisse approvata, la fonte autorevole dell'indirizzo andrebbe scelta **una sola volta**, evitando due copie divergenti fra `collectors` e future `party`.
- **[REPS-UX-046 — Consegne post-vendita](REPS-UX-046-consegne-spedizioni-post-vendita-checklist.md)** tratta *eventi logistici, destinatario, checklist e indirizzo confermato per una specifica consegna*. UX-056 conserva **recapiti riutilizzabili**; non attesta spedizioni o consegne, non genera etichette, non seleziona indirizzi automaticamente e non modifica una vendita. Una futura consegna dovrebbe acquisire un **valore copiato e confermato** (snapshot), non leggere sempre un indirizzo di rubrica che potrebbe cambiare.
- **[REPS-UX-035 — Preferenze di contatto](REPS-UX-035-preferenze-contatto-comunicazioni-individuali.md)** distingue finalità e consensi alle comunicazioni; un indirizzo etichettato «corrispondenza» **non** costituisce consenso a comunicazioni promozionali.
- **[REPS-UX-048 — Identificativi storici delle opere](REPS-UX-048-identificativi-storici-riferimenti-esterni-opere.md)** riguarda codici delle opere, non indirizzi delle persone. **[TECH-040 — vCard](../strumenti-e-architettura/REPS-TECH-040-scambio-contatti-vcard-offline.md)** riguarda interoperabilità dei contatti: esportare più indirizzi richiederà mappature e limiti espliciti, ma nessun file TECH viene modificato.
- **[TECH-001 — Portabilità](../strumenti-e-architettura/REPS-TECH-001-pacchetto-dati-aperto.md)** e **[TECH-016 — Allegati](../strumenti-e-architettura/REPS-TECH-016-allegati-documentali-locali-multi-entita.md)** restano studi infrastrutturali indipendenti. UX-056 non introduce allegati di documenti d'identità, scansioni di fatture o dati fiscali.
- Confrontati gli indici delle **55 schede Gestione/UX precedenti** e delle **51 schede tecniche** disponibili sui branch delle PR documentali. Non risulta una scheda dedicata alla selezione prudente e alla cronologia minima degli **indirizzi di contatto riutilizzabili**. La distinzione rimane concettuale: non si considera approvata nessuna dipendenza fra proposte.

## 3. Scenari d'uso e percorsi UX proposti

1. **Collezionista con due recapiti.** La persona ha un indirizzo per corrispondenza e un secondo luogo dove *potrebbe* ricevere opere. L'utente li distingue con etichette, vede quale è stato inserito più di recente e **verifica ogni volta** la destinazione; il programma non deduce quella corretta dalla natura della vendita.
2. **Cambio indirizzo.** Il collezionista comunica un trasferimento. L'artista modifica il recapito abituale e, soltanto se serve documentarlo, mantiene il precedente come **non attivo** con data/fonte dichiarate. Non ricostruire automaticamente spostamenti né conservare ogni battitura corretta nel form.
3. **Galleria con due sedi.** L'utente memorizza «sede amministrativa» e «sala espositiva». Nessuna delle due diventa sede fiscale o ubicazione fisica delle opere senza un'ulteriore dichiarazione.
4. **Consegna a terzi.** Un acquirente chiede una consegna a una persona diversa. La destinazione viene confermata nel *caso di consegna* eventuale UX-046 e non aggiunta obbligatoriamente ai recapiti permanenti del collezionista; evitare di trasformare un indirizzo occasionale in dato profilato persistente.
5. **Correzione/portabilità.** L'utente consulta l'elenco degli indirizzi di un soggetto, corregge un refuso senza creare una falsa cronologia, elimina un recapito non necessario e può esportare i dati che ha deciso di condividere.

**Flusso minimo (prima fase, se validata):** `Collezionisti → Dettaglio → Indirizzi (sezione comprimibile) → Aggiungi indirizzo`. Nome libero facoltativo («Studio», «Casa», «Sede A»), uso suggerito *generico / corrispondenza / possibile destinazione / altro*, testo dell'indirizzo e nota di verifica **manuale**, non un bollino «verificato» da Repertica. Mostrare il campo `address` attuale nel normale form; l'utente non deve compilare un pannello avanzato per continuare a usare l'app.

**Gestione preferenze:** eventualmente una preferenza distinta per ciascun uso, assegnata esplicitamente e reversibile. Per copia in un documento, presentare sempre **indirizzo completo in anteprima, uso e data dell'ultima conferma dichiarata**, con «Copia questo indirizzo» e «Annulla»; mai compilazione silenziosa né invio email/posta. Sui dati non confermati mostrare «Da verificare con il destinatario» invece di «Indirizzo valido».

**Accessibilità:** schede navigabili da tastiera; pulsanti con nomi propri «Modifica indirizzo Studio», non icone anonime; etichette e stati leggibili da screen reader; nessuna dipendenza dal colore o drag-and-drop; testo lungo a capo; navigazione stabile in finestre piccole e al 200% di zoom. Prima della rimozione, avviso chiaro sul dato effettivamente eliminato.

## 4. Dati e regole — solo ipotesi descrittive, NON schema/API approvati

La fase più economica sarebbe **nessuna nuova tabella**: conservare un solo indirizzo e, quando necessario, indicare l'uso nelle note. Se i test dimostrano un problema ripetuto, studiare un'entità locale concettuale `contact_address` con campi:

- `id` UUID stabile, `collector_id` UUID esistente (oppure futuro soggetto trasversale **solo se** UX-014 fosse approvata), `label` libero, `purpose` codificato facoltativo, `address_text` Unicode preservato.
- Facoltativi: `country_code` ISO 3166-1 alpha-2 **dichiarato**, `postal_code`, `city`, `region`, `contact_name`, `note`. Non obbligare a scomporre paesi con formati postali differenti; la stringa libera resta autorevole per la visualizzazione.
- `active`/archiviato, `preferred_for_purpose` soltanto per decisione manuale, `confirmed_by_user_at` e `confirmation_source_note` (fonte **dichiarata**, non convalida dell'indirizzo). `created_at` e `updated_at` per audit minimo, non prova dell'ultima comunicazione ricevuta.
- Eventuale `delivery_address_snapshot` in UX-046 **solo se in futuro approvata**: testo copiato al momento della conferma, indipendente da modifiche successive alla rubrica; non una foreign key al valore «corrente» usata come documento storico.

**Invarianti:** `collectors.id`, vendite, incassi, snapshot certificati e spese non cambiano; nessun indirizzo modifica automaticamente la proprietà o ubicazione dell'opera. Un contatto può avere zero indirizzi; non inferire indirizzi da email, IP, luogo di nascita o campi liberi. Nessuna sincronizzazione di sistemi indirizzi esterni e nessuna correzione geocodificata obbligatoria. Le future API, solo illustrative, potrebbero essere `list_contact_addresses`, `save_contact_address`, `archive_contact_address`; non sono richieste di implementazione.

## 5. Impostazioni, offline, lingue, privacy, import/export

**Impostazioni:** nessuna configurazione iniziale; se utile soltanto etichette suggerite traducibili, scelta consapevole dell'eventuale indirizzo predefinito per uso, possibilità di disattivare la funzione nell'interfaccia senza perdita di dati. No account, provider cartografico, corriere o geocoding.

**Offline-first:** tutti i dati rimangono nel database SQLite locale; nessun controllo di recapiti via rete, rubrica OS, suggerimenti di indirizzo online o servizio SaaS. L'utente sceglie liberamente la cartella del backup integrale; copie verificabili includono le eventuali nuove righe. Il backup in chiaro è sensibile quanto il resto dell'anagrafica.

**Multilingua:** lingua del sistema, fallback inglese, selezione manuale già prevista in Repertica (IT/EN/ES/FR/DE). Tradurre **solo** etichette di uso, errori e azioni; non tradurre indirizzi e nomi propri, non riordinare parti dell'indirizzo assumendo formato italiano. Unicode, diacritici, apostrofi e caratteri non latini devono restare integri nel salvataggio, nell'anteprima e nell'export; date interne ISO, visualizzazione locale.

**Privacy e sicurezza:** indirizzi privati esclusi per default da brochure, certificati generici, PDF presentazioni e output MCP; nessun nuovo campo esposto agli assistenti. Un'etichetta «possibile consegna» non giustifica un utilizzo promozionale né conserva automaticamente dati di terzi. Prevedere rimozione di recapiti superflui e avviso prima di esportazioni contenenti dati personali; non presentare il programma come strumento di verifica del domicilio. Le copie di backup non subiscono cancellazione retroattiva quando si elimina un recapito dall'archivio attivo.

**Portabilità integrale:** se esisteranno indirizzi multipli, esportarli in **CSV dedicato** con `address_id;collector_id;label;purpose;address_text;active;...` oltre al CSV `collectors` legacy, dichiarando la relazione e preservando UUID e valori esatti. Mai comprimere indirizzi multipli in una colonna ambigua o cambiare silenziosamente il significato del vecchio `address`. Un eventuale formato strutturato versionato per round-trip dovrà mantenere relazioni, note e preferenze; import solo con anteprima, validazione, gestione conflitti e nessuna sovrascrittura senza conferma. vCard (TECH-040) solo come **trasferimento parziale dei recapiti**, senza garanzia di conservare note/etichette o cronologia.

**Compatibilità:** mantenere aperti gli archivi con il solo `collectors.address` e mostrarlo come indirizzo principale già esistente; migrazione **additiva e reversibile ove possibile**, senza duplicarlo due volte o cancellarlo nel CSV attuale. Se in futuro si decidesse di normalizzare gli indirizzi, specificare un periodo di compatibilità e verificare il ripristino delle copie pre-migrazione. Compatibilità Windows/macOS, tastiera e screen reader; nessuna dipendenza dal formato locale dei separatori.

## 6. Complessità, dipendenze, rischi e alternative

- **Complessità stimata:** bassa per una nota strutturata in UI senza nuovo modello; media per entità indirizzi multipli con preferenze, archiviazione e CSV; alta se si estende a una rubrica unificata e a snapshot di consegne/documenti. **Dipendenze eventuali**, non richieste: un unico modello di contatto (UX-014), consegne (UX-046), portabilità interoperabile (TECH-001/040).
- **Rischi principali:** scelta di un recapito obsoleto come destinazione; raddoppio incoerente dei dati collezionista/contatto; divulgazione di domicilio in un documento pubblico; falsa affidabilità dovuta a «indirizzo verificato»; eccesso di raccolta di indirizzi di terzi; migrazioni non round-trip; costo UX superiore al beneficio.
- **Mitigazioni concettuali:** selezione e conferma esplicite per ogni uso pratico, data/fonte soltanto dichiarate, nessun invio automatico, dati privati per default, legami stabili per UUID, backup prima di eventuali migrazioni e nessuna modifica silenziosa di documenti storici.
- **Alternative prima dello sviluppo:** (A) campo indirizzo attuale + note, con istruzione contestuale «conferma sempre l'indirizzo di consegna»; (B) due campi testuali opzionali fissi «corrispondenza»/«possibile consegna» se gli utenti usano quasi sempre esattamente due recapiti; (C) modello multi-indirizzo solo dopo casi documentati. Preferire A o B se soddisfano gli utenti, per minimizzare raccolta e interfaccia.

## 7. Criteri di accettazione e prove ipotetiche — NON test implementati

1. Su archivio legacy con un solo `collectors.address`, il dato è visibile, modificabile ed esportato **esattamente come prima**; vendite e certificati esistenti non mutano.
2. Due indirizzi distinti dello stesso contatto possono essere salvati/visualizzati senza confondere uso o preferenza; la scelta per una consegna richiede conferma esplicita, mai inferenza.
3. Modificare/archiviare il recapito corrente **non** modifica un eventuale snapshot storico di consegna già confermata, né il testo di documenti già emessi.
4. Con contatto privo di indirizzo, form, ricerca e backup continuano a funzionare; dati incompleti non generano errori bloccanti né false «verifiche».
5. I recapiti privati non compaiono in output di presentazione, MCP o esportazioni selettive salvo autorizzazione esplicita; l'export integrale documenta chiaramente i dati personali inclusi.
6. CSV e backup di prova con città non italiane, nomi Unicode, CAP alfanumerici e indirizzi multilinea conservano i caratteri e la relazione al contatto anche dopo ripristino/import controllato.
7. Flussi aggiungi/modifica/seleziona/rimuovi leggibili da screen reader, completi da tastiera e utilizzabili con zoom 200%; traduzioni UI mancanti tornano in inglese.
8. Un test con **almeno tre contesti di utenti** (studio con spedizioni, piccola galleria, artista senza spedizioni frequenti) dimostra che il beneficio supera la complessità rispetto a note o due campi.

## 8. Fasi facoltative e domande aperte al proprietario

**Fase 0 — Validazione, nessuno sviluppo:** raccogliere 5–10 casi anonimizzati di recapiti multipli; chiedere quante destinazioni servono davvero, chi le conferma e con che frequenza cambiano. Verificare se note già bastano.

**Fase 1 — Solo se necessaria:** UI leggera per visualizzare etichetta e recapiti senza automatismi. Se due campi fissi soddisfano i casi, rinunciare al modello generico.

**Fase 2 — Condizionata:** indirizzi multipli portabili con eventuali preferenze dichiarate, compatibilità legacy e test di backup/CSV.

**Fase 3 — Solo con ulteriori decisioni separate:** collegamento esplicito a UX-046 per copia confermata della destinazione, o a UX-014 per rubrica unica; mai introdurre per trascinamento della presente scheda.

**Questioni per revisione personale (nessuna risposta presunta):**
- Quanti utilizzatori reali hanno **più di un recapito stabile** per collezionista? Quante volte una spedizione va a terzi?
- È utile un campo aggiuntivo semplice oppure serve davvero un elenco aperto di indirizzi?
- Dove deve risiedere la fonte autorevole se UX-014 sarà eventualmente approvata?
- Serve conservare recapiti precedenti, o la minimizzazione suggerisce di eliminarli quando non servono?
- Quali indirizzi, se mai, possono apparire in documenti condivisi? Con quale conferma specifica?

**Decisione proposta, non approvata:** priorità **P2 solo dopo validazione con utenti**; altrimenti **P3 o nessuno sviluppo**. La scheda non abilita alcuna implementazione o assegnazione.
