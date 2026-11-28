# REPS-UX-045 — Certificati d'opera: rettifiche documentate e sostituzioni senza cancellare lo storico

- **ID stabile:** REPS-UX-045
- **Data della ricognizione:** 2026-11-22
- **Stato:** Proposta — da valutare
- **Filone:** Gestione e UX
- **Priorità proposta:** P1–P2 **solo se** si riscontrano certificati errati già consegnati; altrimenti P2–P3. Priorità ipotetica, non deliberata.
- **Ambito:** esclusivamente ideazione e documentazione; non approva né assegna alcuna implementazione.

## 1. Problema, destinatari, fatti e ipotesi

Un certificato numerato può essere emesso con un errore materiale (intestatario, misura, tecnica, testo, fotografia riferita alla scheda), mentre l'opera e la sua scheda possono essere corrette successivamente. Per un artista, una galleria o un archivista è importante poter indicare **quale documento è da riesaminare** e, se necessario, **quale nuovo certificato è stato emesso al suo posto**, senza distruggere il documento originale e senza presumere di aver recuperato copie già consegnate.

**Verificato su main il 22 novembre 2026:**
- [Schema SQLite](../../../src-tauri/src/db.rs): i certificati hanno UUID, seriale unico, data, emittente, intestatario, note e snapshot JSON; i trigger impediscono UPDATE e DELETE anche fuori dall'interfaccia. Esiste un contatore progressivo per i seriali.
- [Comandi Rust](../../../src-tauri/src/commands.rs): sono disponibili elenco ed emissione di certificati. Ogni emissione acquisisce uno snapshot dell'opera e un nuovo seriale. Non risulta un registro separato di contestazioni o relazioni fra emissioni.
- [Interfaccia](../../../src/app/app.component.html): l'elenco mostra seriale, *titolo attuale dell'opera*, artista, data e «Stampa / PDF». La finestra di emissione avverte dell'immutabilità. Il titolo mostrato nell'elenco non è necessariamente il titolo dello snapshot originario.
- [MVP](../../MVP.md) e [portabilità dati](../../DATA-PORTABILITY.md): certificati numerati, storico congelato, export CSV del contenuto originale e backup locale con media storici protetti. Il documento generato non certifica automaticamente l'autenticità e deve essere verificato/firmato dall'emittente.

**Problema non accertato sul campo:** non sono state esaminate richieste reali di rettifica né documenti già distribuiti. Non è un bug dimostrato o una necessità approvata; prima occorre capire quanto spesso accadano errori **dopo** l'emissione, soprattutto quando il certificato è già firmato o consegnato.

**Destinatari principali:** artisti che emettono certificati con il proprio nome; gallerie che ne conservano copia; archivi che ricostruiscono un fascicolo dopo variazioni di attribuzione o errori di registrazione. Per chi non emette mai certificati, questa funzione non deve aggiungere complessità all'archivio.

## 2. Differenze rispetto a idee e funzioni esistenti

- **UX-013 — Resi/annullamenti/rettifiche vendite:** regola eventi commerciali, incassi e stato di vendita. Lì è solo richiamato che un certificato storico non va alterato; **qui** il soggetto è il ciclo documentale del certificato, che può essere rettificato anche senza vendita. Non si modificano operazioni commerciali né si interpreta una rettifica del certificato come reso.
- **UX-004 — Conservazione e rapporti di condizione; UX-019 — Esposizioni e bibliografia; UX-031 — Autorialità:** possono fornire elementi da verificare, ma non prendono decisioni automatiche sui certificati già emessi. Un cambio nell'attribuzione artistica non prova che un certificato sia falso.
- **UX-001 — Movimentazioni:** una nuova emissione non dimostra spostamenti fisici, riconsegne o titolarità.
- **TECH-023 — PDF/A verificabile:** studia il **formato archivistico** delle copie digitali da snapshot immutabili, non un flusso gestionale per segnalare errori e collegare una sostituzione. Il PDF/A non è revoca legale né firma.
- **TECH-001/016 — Portabilità e allegati:** concernono infrastruttura di scambio e file; qui non si costruisce un formato generale né un repository documentale alternativo.
- L'intera [PR tecnica #16](https://github.com/Cesare-ml/Repertica-Studio/pull/16), con 40 proposte, è stata confrontata solo per evitare sovrapposizioni; **nessun suo file è modificato**.

**Fuori ambito:** dichiarazioni automatiche di autenticità, invalidità giuridica, cancellazione/riciclo di seriali, firma elettronica, invio di notifiche, verifica remota di certificati, blockchain, riscrittura PDF già consegnati, sostituzione automatica di intestatari in base alle vendite e produzione di fatture o documenti fiscali.

## 3. Scenari d'uso distinti

1. **Refuso prima della consegna.** Emesso RS-2026-000041 con dimensione errata nello snapshot: l'utente registra «errore materiale segnalato», corregge la *scheda corrente* tramite il normale editor, controlla l'anteprima e decide esplicitamente se emettere RS-2026-000042 come sostituto. L'originale resta intatto e consultabile. La segnalazione non implica automaticamente che sia stato consegnato o ritirato.
2. **Certificato già firmato e consegnato.** Una galleria comunica un errore nel nominativo. L'archivista può registrare «copia consegnata, segnalazione aperta», poi «nuova copia comunicata il giorno X» **solo se dichiarato**. La voce «ritiro confermato» richiede registrazione esplicita; il software non lo deduce dall'emissione sostitutiva.
3. **Errore contestato ma non accertato.** Un cliente segnala una discrepanza sull'anno dell'opera. L'utente conserva lo stato «da verificare» e la fonte della segnalazione, senza attribuire automaticamente torto all'emittente.
4. **Sostituzione successiva della sostituzione.** Un secondo errore richiede un terzo certificato. Si mantiene una catena cronologica aciclica, non più documenti contemporaneamente «sostitutivi correnti» senza una decisione esplicita.
5. **Nuova edizione legittima per la stessa opera.** Un artista emette un altro certificato per ragioni indipendenti. Il sistema **non** deduce che il primo sia stato annullato: due certificati della stessa opera possono essere autonomi finché non esiste un collegamento dichiarato.

## 4. Flussi UX: conservazione e chiarezza prima di nuovi automatismi

**A — Elenco certificati:** mantenere «Stampa / PDF» e introdurre, solo se la proposta fosse approvata, un'etichetta informativa distinta: «Nessuna segnalazione», «Da verificare», «Rettifica dichiarata», «Sostituito da [seriale]». Evitare «valido/non valido» come giudizio legale. Affiancare **titolo all'emissione** a eventuale titolo corrente per non nascondere differenze: entrambi identificati con etichetta, mai sovrapposti.

**B — Segnala una discrepanza:** dalla scheda del certificato → «Annota problema del documento». Mostrarne prima seriale, data, emittente e dati storici da snapshot **in sola lettura**. Form minimo: categoria (refuso / dati dell'opera / intestatario / fotografia / altro), data della segnalazione, descrizione, fonte e stato «da verificare». Le informazioni non documentate restano vuote. Conferma esplicita «Questa annotazione non modifica il certificato originale».

**C — Valuta e rettifica:** visualizzare confronto **originale vs scheda attuale**, evidenziando solo cambi reali, senza riempire automaticamente valori storici mancanti. Opzioni separate: chiudere la segnalazione come infondata; aggiungere una nota di rettifica senza nuova emissione; procedere alla normale emissione di un certificato nuovo *dopo controllo esplicito di ogni campo*. L'utente può interrompere il flusso senza perdere il documento originario e senza consumare numeri.

**D — Collega la nuova emissione:** dopo l'emissione effettiva, associare il nuovo seriale all'originale mediante riferimento UUID. La schermata deve ribadire: «Il documento precedente resta archiviato. La sua eventuale consegna o sostituzione presso terzi non è verificata da Repertica». Non inviare email, non generare movimenti fisici, non attestare recuperi o firme. Se una seconda operazione concorrente tenta un collegamento incompatibile, richiedere riconciliazione manuale.

**E — Consultazione e stampa:** l'originale deve continuare a essere apribile e riproducibile **esattamente da snapshot storico**, con un avviso UI contestuale quando è segnalato/sostituito. Eventuale documento **separato** di accompagnamento «annotazione dell'archivio» è una fase successiva: non alterare il contenuto del certificato emesso e non stampare un timbro di invalidità al suo interno come se fosse parte dell'originale.

**F — Visibilità e accessibilità:** navigazione da tastiera, stato annunciato con testo, differenze confrontabili riga per riga, dialogo con focus corretto, nessuna informazione affidata al colore, zoom 200%, etichette lunghe leggibili e conferme non distruttive. Il modulo esistente di emissione rimane semplice per gli utenti ordinari.

## 5. Dati concettuali e regole (NON schema approvato)

**Fase minima:** una voce separata e locale «annotazione-certificato», con UUID, certificate_id, stato di verifica, categoria, nota, data segnalazione (con distinzione dalla data registrazione), fonte dichiarata facoltativa, timestamp di creazione e aggiornamenti append-only. La correzione di una nota critica produce nuovo evento o rettifica collegata, mai una riscrittura silenziosa.

**Eventuale relazione di sostituzione:** collegamento esplicito {original_certificate_id, replacement_certificate_id, causa, dichiarato_il, operatore_locale_facoltativo}, con ID esistenti, stessa opera come regola generale e unicità/acyclicità della catena. Consentire **più annotazioni** su un certificato, ma definire quale sia la relazione sostitutiva principale. Non creare un «nuovo seriale» dal solo collegamento: il progressivo si attribuisce soltanto nel normale flusso di emissione.

**Stati separati da non confondere:**
- **Istruttoria interna:** da verificare / chiarita senza rettifica / rettifica documentata;
- **Relazione documentale:** nessuna sostituzione / sostituito da certificato X (solo se registrato);
- **Circolazione di copie:** consegna riferita / comunicazione dichiarata / ritiro dichiarato / ignoto. Mai assumere recapito, ricezione o distruzione fisica.

**Invarianti fondamentali:** mantenere intatti i trigger di immutabilità sulla tabella certificati, seriali non riutilizzabili, UUID e snapshot originali, fotografie storiche referenziate e conservate, eventuali firme manoscritte fuori dal controllo software. Nessuna modifica a vendite, incassi, quote di guadagno, coefficiente artista, stato commerciale o ubicazione. Una segnalazione su un certificato non deve modificare né ritirare automaticamente gli altri della stessa opera.

**API esemplificative solo descrittive:** «leggi fascicolo certificato», «aggiungi evento di segnalazione», «collega certificato già emesso come sostituto», «elenca catena delle sostituzioni». Non sono richieste operative né comandi progettualmente deliberati.

## 6. Impostazioni, funzionamento offline e lingue

- **Offline:** tutti gli eventi e i collegamenti restano nello SQLite locale; eventuali prove e allegati sono file locali scelti dall'utente, con riferimenti verificabili e media soggetti al backup. Nessun account, firma online, SaaS, sincronizzazione, email o API obbligatori.
- **Impostazioni facoltative:** avviso aggiuntivo prima dell'emissione; eventualmente «mostra sempre segnalazioni in elenco», senza obbligare a gestire casi. Nessuna procedura di rettifica imposta agli archivi precedenti.
- **Lingue:** sistema operativo come impostazione predefinita, fallback inglese e possibile override IT/EN/ES/FR/DE. Tradurre accuratamente «segnalato», «rettificato», «sostituito», «notificato secondo dichiarazione», senza equipararli a «revocato legalmente». Seriali e contenuti inseriti dall'utente **non si traducono**; date/numeri localizzati solo in presentazione.
- **Privacy:** possono emergere nomi di intestatari, errori di attribuzione, recapiti e documenti confidenziali. L'elenco locale è visibile solo sul dispositivo; nessuna annotazione esportata pubblicamente per impostazione predefinita. Backup attuali non cifrati: mostrare il rischio quando si conservano documenti sensibili su supporti condivisi.

## 7. Import/export, backup e compatibilità

- **Export integrale:** un futuro registro CSV degli eventi con ID stabili e riferimenti ai seriali dovrebbe essere disponibile insieme ai registri esistenti; il CSV «certificates» resta integralmente disponibile con l'originale snapshot immutabile e **non deve essere trasformato** silenziosamente in una vista dei soli documenti correnti. Se si aggiunge una colonna di stato derivato, versionare e documentare la variazione.
- **Export selettivo:** fascicolo in JSON aperto, versionato e con collegamenti, su richiesta; eventuale stampa separata del riepilogo, **senza attribuirgli valore di certificato**. Anteprima dei campi sensibili e nessuna esportazione automatica di note private, indirizzi o allegati. Il PDF storico resta quello dell'emissione originaria e non viene modificato.
- **Import eventuale:** non previsto nella fase minima. Solo con anteprima, UUID/seriale, verifica dell'esistenza dell'opera e dei certificati, rilevamento catene impossibili, idempotenza, conferma esplicita e rollback. Un file esterno non autorizza la modifica degli snapshot: l'eventuale round-trip va coordinato con TECH-001.
- **Backup:** include eventi, collegamenti, eventuali allegati e immagini storiche in una copia verificata SQLite+media nella cartella liberamente scelta. Ripristino dell'archivio intero, non «ripristino di un certificato» da CSV. I backup preesistenti devono continuare a essere leggibili senza eventi sintetici, mentre versioni app vecchie potrebbero non aprire un futuro schema migrato.
- **Compatibilità:** migrazione solo additiva, certificati legacy senza annotazioni mostrati come «nessuna segnalazione registrata» **non** come «validità certificata». Il titolo ricavato dalla scheda attuale resta distinguibile dal titolo storico dello snapshot; documenti già stampati o consegnati non possono essere corretti dal software.

## 8. Complessità, dipendenze, rischi e alternative

**Complessità:** bassa-media per una nota di discrepanza locale e avviso nell'elenco; media per collegamenti e timeline append-only; medio-alta per una sostituzione guidata, controlli di concorrenza, compatibilità documentale ed eventuali allegati/report. Sono valutazioni preliminari, non stime di sviluppo concordate.

**Dipendenze tecniche eventuali:** schema SQLite con trigger immutabilità, comando di emissione seriale, modelli Angular/Tauri, UI localizzata, export/backup attuali. Il formato PDF/A di TECH-023 **non** è prerequisito: riguarda la resa e la conservazione, non la semantica della rettifica.

**Rischi:** un avviso UI scambiato per revoca giuridica; falsa certezza sulla consegna del sostituto; reimpiego di seriali; perdita di fotografie storiche; creazione accidentale di un nuovo certificato; catene circolari; note che espongono contatti; stampa di una copia storica senza avviso nell'interfaccia; confusione tra correzione dell'opera e correzione retroattiva del certificato.

**Alternative:**
1. **Nessun nuovo modulo:** conservare una nota esterna e, se necessario, emettere un nuovo certificato manualmente (meno complessità, ma nessun collegamento archivistico).
2. **Prima fase minima:** nota separata per seriale, stato «da verificare» e avviso in elenco; nessun wizard di sostituzione.
3. **Fase intermedia:** collegamento manuale tra certificati già emessi, con avvisi e controllo della catena.
4. **Fase evoluta:** assistenza all'emissione sostitutiva, confronto dati e riepilogo documentale separato, soltanto dopo validazione con casi reali.

## 9. Criteri di accettazione ipotetici (nessun test eseguito)

1. Un certificato RS-2026-000041 con errore rimane con **stessi UUID, seriale, snapshot, data e file fotografici** dopo annotazione e sostituzione; UPDATE e DELETE della tabella originale continuano a fallire.
2. Un certificato nuovo RS-2026-000042 si collega al precedente solo **dopo emissione confermata**; un annullamento del dialogo non consuma il seriale e non produce una relazione fittizia.
3. Il sistema distingue «segnalato», «sostituito» e «copia ritirata»: nessuno stato implica automaticamente consegna, firma, recupero o valore giuridico.
4. La correzione del titolo o della tecnica dell'opera corrente non cambia lo snapshot del certificato; nell'elenco le due versioni sono chiaramente etichettate.
5. Una catena A→B→C rimane consultabile; non è possibile creare C→A o indicare contemporaneamente due sostituti correnti incompatibili senza risoluzione.
6. Le annotazioni non alterano vendite, rate, incassi, quote congelate, coefficiente artista, disponibilità né movimenti fisici.
7. Export CSV/JSON richiesti includono riferimenti stabili ma non espongono per default dati personali superflui; backup e restore su cartella scelta preservano originali, annotazioni, collegamenti e media senza perdere file storici.
8. Archivi precedenti mostrano «nessuna segnalazione registrata» senza produrre record falsi; stampa storica disponibile con avviso contestuale nella UI ma **nessuna modifica del contenuto originale**.
9. Flussi completamente offline e accessibili a tastiera, screen reader e zoom 200%, in IT/EN/ES/FR/DE con lingua sistema e fallback inglese.

## 10. Fasi possibili e decisioni del proprietario

- **Ricerca preliminare:** chiedere a 3–5 artisti/galleristi quanto spesso correggono certificati *già emessi*; distinguere errori prima della stampa, dopo firma e dopo consegna. Raccogliere esempi anonimi, non documenti con dati personali.
- **A — minima:** avviso in elenco e annotazioni descrittive separate, senza cambiare la procedura di emissione o il formato dei certificati.
- **B — condizionata:** legame esplicito fra certificato precedente e sostituto già emesso, con timeline e controlli di duplicazione.
- **C — eventuale:** confronto dei dati prima di una nuova emissione; fascicolo locale portabile, senza automatizzare comunicazioni né firme.
- **D — eventuale:** interoperabilità col formato tecnico di conservazione e con eventuali documenti/prove locali, senza dipendere da servizi online.

**Requisiti certi:** archivio gratuito/open source offline-first Tauri/SQLite, indipendenza senza account/abbonamenti, certificati immutabili con progressivo univoco, export locali e backup completi in cartella libera, lingua OS con fallback inglese.

**Ipotesi da verificare:** frequenza di errori dopo consegna; valore di una relazione tra emissioni; disponibilità degli utenti a distinguere istruttoria interna da comunicazioni a terzi; utilità di una segnalazione visibile accanto a ogni certificato.

**Domande per revisione personale:** quale definizione organizzativa preferire per il termine «rettificato»? Serve solo una nota di attenzione, oppure è necessario collegare due numeri seriali? Chi usa Repertica deve poter registrare anche una dichiarazione di avvenuta notifica, senza che il programma la attesti? Quali campi sono più frequentemente errati? È opportuno mostrare le annotazioni in un eventuale documento condivisibile oppure soltanto internamente? Quando basta un nuovo certificato manuale senza procedura dedicata?

**Conclusione:** **Proposta — da valutare**. Nessuna approvazione, assegnazione, implementazione o mutazione di certificati autorizzata da questa scheda.
