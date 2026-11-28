# REPS-UX-068 — Verifica prudente di un collezionista già presente prima del nuovo inserimento

- **ID stabile:** REPS-UX-068
- **Data:** 2026-11-23
- **Stato:** Proposta — da valutare
- **Filone:** Gestione e UX (solo ricerca e documentazione)
- **Priorità proposta:** P2, *soltanto* se errori di inserimento o casi ricorrenti vengono confermati; P3/nessuna funzione se bastano ricerca e istruzioni
- **Decisione:** nessuna approvazione o assegnazione allo sviluppo; nessuna implementazione autorizzata

## 1. Problema, utenti e evidenza nel repository

**Possibile problema professionale:** durante l'inserimento rapido di una vendita e di una nuova opera, l'operatore può creare «Mario Rossi» anche se un «Mario Rossi» è già nell'archivio. Le vendite diventano collegate a UUID differenti e la vista delle opere acquistate per collezionista risulta frammentata. All'opposto, due persone possono davvero condividere lo stesso nome: bloccare nomi omonimi sarebbe sbagliato e potrebbe attribuire una vendita a un estraneo.

**Riscontri verificati su `main` il 23/11/2026 tramite connettore GitHub:**

- `src-tauri/src/collectors.rs`: `save_collector_core` assegna un nuovo UUID a un nuovo contatto, inserendolo senza un controllo semantico di corrispondenza per nome/recapiti. Nome obbligatorio, email facoltativa; l'email è validata nella forma, ma non usata come chiave univoca.
- `src-tauri/src/commands.rs`: `create_artwork_full` consente di registrare **una nuova opera e una vendita** con `new_collector_name`; il nuovo collezionista nasce nella stessa transazione. Non emerge da questo percorso un avviso di possibili omonimi prima del commit.
- `src/app/app.component.html`: nella creazione opera+vendita si può selezionare un collezionista esistente oppure «Nuovo collezionista…», con campo nome del nuovo. `src/app/app.component.ts` gestisce il form e la lista dei collezionisti.
- `src-tauri/src/db.rs`: la tabella `collectors` ha UUID e campi anagrafici; le vendite hanno un riferimento al collezionista con vincoli. Il database non impone unicità del nome, opportunamente perché l'omonimia non determina identità.
- `src/app/archive-filters.ts`: esiste già ricerca sui collezionisti; la proposta non intende sostituirla.

Questi sono **rischi plausibili del flusso**, non la prova di duplicati effettivamente creati dagli utenti o di un malfunzionamento. Destinatari da interpellare: artisti che registrano vendite mentre catalogano, studi con clienti ricorrenti e piccole gallerie con rubrica in crescita.

## 2. Scenari concreti e confini

1. **Nome già presente:** mentre crea opera+vendita, l'artista digita «Maria Bianchi»; prima di scrivere compare un suggerimento leggibile «Possibile contatto già presente», con sufficienti indizi per scegliere consapevolmente l'UUID esistente oppure proseguire con un nuovo contatto.
2. **Omonimia autentica:** due «Maria Bianchi» diverse; l'operatore sceglie «Crea comunque nuovo». Mai selezione automatica e mai blocco basato soltanto su somiglianza.
3. **Nuovo collezionista con nome invertito o accenti:** «Bianchi Maria» o varianti di spazi/accenti potrebbero essere mostrate come indizio *debole*; niente certezza statistica, niente deduzione da foto o dati sensibili. Misurare prima se questi casi meritino la complessità.
4. **Contatti con vendita già registrata:** l'avviso mostra, senza dettagli economici se non necessari, che una scheda esistente ha opere associate. Evitare di generare una nuova scheda e separare ulteriormente uno storico reale.
5. **Collezionista aggiunto dalla sezione Contatti:** la stessa regola UX può essere offerta anche lì, ma la prima prova si concentra sul percorso rapido opera+vendita, dove è più facile non consultare la rubrica.

**Anti-doppioni:**
- [UX-055](REPS-UX-055-revisione-possibili-doppioni-schede-opera.md) studia una **ricognizione di opere già catalogate**, non l'identificazione prudente di una persona al **momento della creazione**. Il pattern di confronto umano può essere riutilizzato, non duplicato in logiche autonome.
- [UX-014](REPS-UX-014-rubrica-relazioni-professionali.md) riguarda ruoli e contatti professionali trasversali: qui nessuna nuova rubrica o nuova entità.
- [UX-018](REPS-UX-018-catalogazione-seriale-schede-bozza.md) riguarda schede opera; questa verifica non cambia il salvataggio delle opere o delle bozze.
- [UX-061](REPS-UX-061-revisione-dati-personali-collezionisti.md) riguarda revisione/rettifica dei dati attuali e copie storiche, non l'evitare un nuovo record all'ingresso.
- [TECH-019](../strumenti-e-architettura/REPS-TECH-019-importazione-guidata-cataloghi-tabellari.md) e [TECH-040](../strumenti-e-architettura/REPS-TECH-040-scambio-contatti-vcard-offline.md) affrontano import tabellari/vCard e relativi conflitti, non il flusso manuale di registrazione della vendita.
- [TECH-033](../strumenti-e-architettura/REPS-TECH-033-salvataggio-condizionale-schede-obsolete.md) studia protezione da scritture di schede obsolete; nessuna sua modifica o approvazione è prerequisito per valutare questa esperienza.
- Questa scheda **non propone unione, riallocazione delle vendite, riscrittura di certificati, deduplicazione automatica o cancellazione**. Un eventuale processo di risoluzione di duplicati storici richiederebbe decisione distinta.

## 3. Specifica funzionale e flusso UX ipotetici

**Variante minima da verificare prima (nessun algoritmo complesso):**

1. Nel percorso «Nuova opera → Registra una vendita → Nuovo collezionista» il nome inserito viene confrontato solo con le schede locali esistenti dopo il normale controllo dei campi. La lettura non scrive dati.
2. Se esistono uno o pochi candidati, una riga non bloccante indica «Esiste già un collezionista con questo nome» e un comando «Confronta». Non mostra dati personali aggiuntivi finché l'utente non apre il confronto.
3. Il confronto affiancato mostra **nome, cognome quando presente, email/telefono se già presenti e soltanto se utili, UUID/codice tecnico in dettaglio**, e l'indicazione «scheda con N opere acquistate» senza importi; provenienza anagrafica solo se documentata. Evidenzia quali campi mancano; mai «stessa persona» come verdetto.
4. Due azioni esplicite: **«Usa collezionista esistente»** (seleziona l'UUID già presente e mantiene la bozza opera/vendita) e **«Crea un nuovo collezionista»** (procedimento attuale). Un terzo comando «Torna ai dati» lascia tutto invariato.
5. Il controllo finale deve essere coerente con il salvataggio autorevole: se un altro percorso ha creato o modificato un candidato fra avviso e conferma, mostra «Verifica di nuovo», senza selezionare un'identità di propria iniziativa. Se l'utente sceglie nuovo non si blocca per omonimia.
6. Il modulo «Nuovo collezionista» indipendente può proporre identico avviso, senza aggiungere un altro wizard o imporre controlli a chi crea numerosi contatti distinti.

**Alternativa ancora più leggera:** aggiungere, accanto all'opzione «Nuovo collezionista…», un link «Cerca nella rubrica» che mostra risultati secondo il filtro già esistente. Solo prove d'uso possono indicare se serva un avviso dinamico.

**Accessibilità:** avviso testuale non solo cromatico; il lettore di schermo annuncia la presenza dei candidati *senza ripetere a ogni battuta*, ordine del focus e ritorno alla bozza mantenuti; Enter non conferma automaticamente una corrispondenza; Escape chiude solo il confronto, non distrugge la vendita; zoom e finestra stretta devono restare utilizzabili.

## 4. Dati minimi, regole e impostazioni (nessuno schema approvato)

La prima fase può leggere gli attuali `collectors.id`, `name`, `first_name`, `last_name`, eventuali `email` e `phone`, e conteggi delle vendite indicizzati per `collector_id`. Nessuna nuova informazione obbligatoria, nessun identificativo biometrico, nessuna data di nascita usata per dedurre identità. Le informazioni mancanti non costituiscono un mancato match.

**Corrispondenze:** iniziare solo da nome normalizzato per spazi e maiuscole/minuscole; valutare accenti e altre varianti soltanto dopo falsi positivi/negativi misurati. L'email non è identità legale (può essere condivisa), il numero può cambiare e omonimie sono normali. Non mostrare una «percentuale di identità». Se non emerge nessun candidato, mantenere il flusso odierno senza un passaggio inutile.

**Impostazioni ipotetiche:** per la versione minima nessuna preferenza necessaria; se le segnalazioni fossero intrusive, valutare in futuro «Mostra suggerimenti sui contatti» disattivabile, senza impostazioni invasive per punteggi/soglie. Nessun nuovo pannello di monitoraggio obbligatorio.

**Import/export e portabilità:** la semplice verifica non produce nuovi dati: i CSV collezionisti/vendite e i backup completi continuano a includere UUID e relazioni già esistenti. Un eventuale report locale di revisione sarebbe un CSV *separato*, esplicitamente richiesto, con dati minimi, non destinato alla condivisione per default. Non introdurre un formato privato o una seconda anagrafica; l'export completo e il ripristino su cartella liberamente scelta restano indipendenti da questa idea.

## 5. Offline, privacy, compatibilità e localizzazione

Funzionamento integralmente su SQLite locale nel desktop Tauri, senza richieste internet, account, provider IA, cloud o sincronizzazioni. Non leggere rubriche OS, posta IMAP, account social, immagini dei clienti o altri archivi senza autorizzazione separata. Non registrare nomi/email cercati in log, notifiche, telemetria o cronologie persistenti di ricerca. I dati comparativi possono apparire su schermo soltanto durante un confronto volontario; attenzione alla condivisione dello schermo.

Traduzioni IT/EN/ES/FR/DE delle etichette e degli errori; riconoscimento della lingua di sistema con fallback inglese, e rispetto dell'override locale già previsto. I nomi propri non sono tradotti; le normalizzazioni di confronto non modificano grafia e diacritici conservati. Conteggi e date formattati secondo locale, UUID invariati.

Compatibilità con vecchi archivi e versioni di backup: nella variante minima è una verifica informativa, senza migrazione SQL, modifiche a dati storici, vendite, certificati, assegnazione delle quote o tabelle nuove. Il flusso attuale «crea comunque» resta disponibile per omonimi reali. Gli indici e i filtri esistenti dovrebbero essere sufficienti per un confronto elementare, da misurare.

## 6. Dipendenze, complessità, rischi e alternative

**Complessità ipotetica:** bassa–media per un richiamo di ricerca e confronto prima del salvataggio nei due moduli; media se servono corrispondenze robuste a nomi invertiti, molte migliaia di persone o controllo del cambiamento concorrente. Non richiede nuove dipendenze, API esterne o algoritmi IA. L'operazione critica resta la transazione di creazione/vendita esistente; il controllo non deve introdurre creazioni parziali né sostituire le validazioni native.

**Rischi:** associare per errore una vendita a un omonimo (più grave del doppione); mostrare dati di contatto non necessari; suggerimenti frequenti per nomi comuni, famiglie o organizzazioni; falsi negativi con grafie diverse; rallentamento del flusso rapido; filtri di confronto non accessibili; concorrente modifica del record; affidarsi a un'unicità email non prevista. Protezioni: nessuna selezione automatica; nuova scheda sempre consentita; confronto esplicito; indicare motivi e incertezza; consultazione minima dei recapiti; conferma effettiva prima del commit.

**Alternative prima di sviluppare:** ricerca manuale nella sezione Collezionisti prima della nuova vendita; elenco alfabetico esistente; semplice microtesto «Verifica che il cliente non sia già in elenco» nel dialogo; foglio CSV locale. Se il promemoria basta, non introdurre l'avviso automatico.

## 7. Criteri di accettazione proposti (non test eseguiti)

1. Due schede omonime di persone differenti rimangono entrambe creabili, senza collegamenti o fusioni automatiche; la vendita va esclusivamente all'UUID scelto dall'operatore.
2. In un archivio con «Maria Bianchi» già presente, selezionarla durante creazione opera+vendita conserva i dati della bozza, non crea un secondo collezionista e produce una sola vendita collegata all'UUID selezionato.
3. Scegliere «Crea nuovo» preserva il comportamento attuale anche se un nome coincide, e non consuma ID di vendita/opera prima del commit; il caso di errore continua a effettuare rollback atomico.
4. Un nome comune con 10 omonimi mostra candidati limitati e filtrabili senza un verdetto, ordinando in modo stabile e senza rivelare importi o note private per default.
5. Match assenti, email vuota, cognome assente, diacritici, spazi ripetuti e nome invertito sono provati; le varianti non verificate non sono classificate come identità certa.
6. Nessun controllo online o accesso a rubriche esterne, nessun log di nomi/contatti; UI da tastiera/screen reader e a zoom elevato in cinque lingue, con fallback inglese.
7. Nessuna modifica retroattiva a vendite, incassi, certificati, fotografie, backup e CSV; su 1.000–5.000 contatti sintetici le interazioni restano utilizzabili (misurare tempo/risorse invece di promettere soglie).
8. Riapertura della bozza o modifica concorrente di un candidato non determina una selezione silenziosa; quando l'identità selezionata non è più valida, il commit rifiuta con messaggio chiaro.

## 8. Fasi possibili, non assegnate

- **Fase 0 — ricerca**: osservare 4–6 sessioni simulate con artisti/gallerie, nomi omonimi e ricorrenti. Verificare se il problema si presenta davvero durante inserimento opera+vendita, quanto tempo richiede una ricerca, quanto spesso il suggerimento sarebbe falso e se basta un link alla rubrica.
- **Fase 1 — solo se motivata:** avviso minimo su corrispondenza esatta nel flusso rapido, scelta esplicita fra contatto esistente e nuovo; nessuna persistenza nuova.
- **Fase 2 — eventuale:** stessa esperienza nel modulo Contatti e miglioramenti accessibili di confronto basati su feedback.
- **Fase 3 — soltanto dopo prove:** ampliare il criterio di confronto con indizi deboli, senza automatismi, e senza includere un progetto di fusione dei record.

## 9. Requisiti certi, ipotesi e questioni per il proprietario

**Certi (codice/docs):** UUID separati per collezionista, vendita vincolata a un ID, creazione rapida opera+vendita anche con un nuovo nominativo, ricerca collezionisti, dati locali, backup completo verificato, CSV e multilingua. **Certi (vincoli di prodotto):** gratuito/open source, nessun account o abbonamento richiesto, offline-first, dati integralmente esportabili, cartella backup a scelta, lingua di sistema/fallback inglese.

**Ipotesi da verificare:** errori di ricreazione del medesimo cliente sono abbastanza frequenti da giustificare un avviso; un semplice confronto minimizza le duplicazioni senza aumentare le associazioni errate; il rischio cresce con l'archivio.

**Domande riservate al proprietario:** (1) ci sono casi osservati di vendite distribuite su due schede dello stesso collezionista? (2) vale più l'avviso sul nome esatto o il collegamento rapido alla ricerca? (3) è importante distinguere persone/organizzazioni fin da questa fase o solo nella futura UX-014? (4) quante persone omonime sono comuni nei suoi archivi? (5) la revisione deve apparire solo in opera+vendita oppure anche nel modulo Contatti? (6) quali campi personali sono davvero necessari per la scelta senza esporre informazioni riservate?

**Esito:** proposta esclusivamente documentale, **Proposta — da valutare**; nessuna approvazione, issue di sviluppo, PR di implementazione o merge.
