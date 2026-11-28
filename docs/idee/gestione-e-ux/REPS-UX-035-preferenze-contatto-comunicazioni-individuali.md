# REPS-UX-035 — Preferenze di contatto e cautele nelle comunicazioni individuali

- **ID stabile:** REPS-UX-035
- **Data:** 2026-11-22
- **Stato:** Proposta — da valutare
- **Filone:** Gestione e UX
- **Priorità proposta:** P2 condizionata; P3 se è sufficiente una nota visibile. Nessuna approvazione.
- **Natura:** soltanto ricerca e progettazione; non autorizza sviluppo, campagne, invii o decisioni giuridiche.

## Punto di partenza verificato al 22 novembre 2026

Su [main](../../../README.md) esiste una scheda [collezionista](../../../src/app/models.ts) con email, telefono e note. È stato inoltre documentato e realizzato un modulo separato di [Posta professionale IMAP/SMTP](../../BUSINESS-MAIL-DESKTOP.md): suggerisce gli indirizzi dei collezionisti locali e consente messaggi individuali con invio SMTP **solo dopo conferma**. La posta è facoltativa, richiede un account del provider scelto dall'utente e Internet per IMAP/SMTP; le bozze restano in SQLite e la configurazione credenziale risiede nel portachiavi OS. Non è una piattaforma di newsletter.

Nei modelli e nei percorsi documentati consultati non emerge una preferenza strutturata **per finalità e canale** associata al collezionista, né una verifica di questa preferenza durante la composizione. Questo **non dimostra un difetto legale, un invio indebito o una domanda utente già validata**: si propone di studiare un promemoria locale per rispettare istruzioni dichiarate dai contatti, senza inventare consensi.

## Problema, persone e utilità

Un'artista può avere recapiti di persone che hanno acquistato opere, chiesto un preventivo o partecipato a una mostra. Una persona può accettare messaggi riguardanti la propria opera, ma chiedere di **non ricevere inviti promozionali**; un'altra può preferire il telefono. Una nota generica è facile da perdere quando si compone un messaggio. In uno studio con molti contatti un promemoria contestuale potrebbe ridurre errori e comunicazioni indesiderate.

**Destinatari:** artisti indipendenti, piccoli studi, gallerie e archivi con relazioni continuative. **Non destinatari primari:** operatori di newsletter o marketing massivo. Non si assume che tutti gli utenti debbano gestire basi giuridiche, mailing list o una rubrica CRM completa in Repertica.

## Casi d'uso

1. **Richiesta di non ricevere inviti.** Un collezionista comunica «non inviarmi altri inviti alle mostre». L'artista annota canale email, finalità inviti/promozione, data, fonte dichiarata e stato «non contattare per questa finalità». La sua scheda d'acquisto e le vendite non cambiano.
2. **Conversazione su un ordine.** Lo stesso collezionista può dover essere contattato sul ritiro di un'opera venduta. La UI distingue «comunicazione relativa alla transazione» da «invito promozionale», ma **non decide automaticamente** quale base giuridica legittimi il messaggio: l'utente ne resta responsabile.
3. **Recapito senza informazioni sufficienti.** Una email storica senza preferenze resta **sconosciuta**, mai «consenso ottenuto» per default. In composizione si può mostrare un'indicazione «preferenza non registrata» senza affermare che l'invio sia permesso o vietato dalla legge.
4. **Rettifica e cambio preferenza.** La persona chiede di cambiare canale o ritirare una precedente disponibilità. La preferenza corrente viene aggiornata conservando, ove davvero necessario, una breve cronologia delle dichiarazioni, evitando di duplicare il messaggio email integrale.
5. **Invio a più indirizzi.** L'utente compila To/Cc/Bcc manualmente. Per indirizzi associati in modo **univoco** a un collezionista si mostrano avvisi pertinenti; indirizzi non riconosciuti, condivisi o ambigui restano marcati come non verificabili, senza associazioni automatiche.

## Flussi UX descrittivi (nessuna azione automatica)

**Scheda collezionista → Contatti e preferenze.** Sotto i recapiti esistenti, pannello compatto «Preferenze di comunicazione», inizialmente facoltativo e vuoto. Azione «Registra indicazione ricevuta»: scegliere canale (email/telefono/altro), tipo di comunicazione (informazioni su transazione, inviti/novità, altra finalità specificata), stato **non indicato / gradito dichiarato / non desiderato dichiarato**, data e fonte facoltativa (es. «detto di persona»). Linguaggio neutro: «gradito dichiarato» **non è un'autorizzazione giuridica verificata**. Mostrare vicino alla email un'icona testuale «Inviti email: non desiderati» soltanto quando esiste una relativa dichiarazione.

**Posta professionale → Nuovo messaggio.** L'utente scrive il messaggio come oggi. Potenziale fase successiva: piccolo selettore facoltativo «Motivo del messaggio» che **non precompila** o classifica legalmente il contenuto. Prima della conferma invio, confronto *locale* con gli indirizzi To/Cc/Bcc e le preferenze dichiarate: un avviso per esplicita opposizione, una nota per sconosciuto/ambiguo. Nessun invio in background, nessun blocco che impedisca comunicazioni necessarie senza offrire una decisione consapevole e registrata solo se richiesta. Per liste di destinatari, evitare di esporre To/Cc visibili ad altri; presentare Bcc distintamente e non convertirlo automaticamente.

**Accessibilità:** etichette «non desiderato», «non indicato», «dato ambiguo» insieme a icone; testo leggibile da screen reader, tab order, errori associati ai campi, focus dopo la conferma, layout mobile/finestre ridotte senza affidarsi al solo colore. Le preferenze non devono oscurare il pulsante di invio esistente o moltiplicare dialoghi.

## Dati e ipotesi di relazioni — non schema o API approvate

- Possibile record **contact_preference**: UUID, riferimento stabile a collector UUID, canale codificato, finalità codificata o descrizione specifica, stato dichiarato, data dichiarazione (anche approssimativa con precisione), fonte minima, nota limitata e privata, data registrazione/ultima rettifica.
- **Storico opzionale**: record evento con precedente/successivo e motivo di correzione; solo se la verifica del bisogno dimostra utilità. Non conservare contenuti di email, registrazioni di chiamate, indirizzi storici o documenti identificativi per default.
- Non associare una preferenza a un indirizzo digitato solo per coincidenza parziale. Il collegamento usa l'UUID del collezionista e controlla corrispondenza completa normalizzata (con cautela sui recapiti condivisi), distinguendo recapito **attuale** da precedenti recapiti; nessun nuovo contatto creato silenziosamente.
- Se in futuro esistesse la rubrica trasversale di [UX-014](REPS-UX-014-rubrica-relazioni-professionali.md), la preferenza dovrebbe poter fare riferimento al soggetto senza duplicare dati personali: la soluzione qui non richiede né approva quell'anagrafica.
- Le preferenze non sono fatture, incassi, autorizzazioni di diritti sull'opera, registrazioni SMTP di consegna o prova che il destinatario abbia letto un messaggio. Nessuna modifica a certificati, vendite, percentuali, coefficiente artista o stati di disponibilità.

## Impostazioni, offline, lingua, portabilità

- **Impostazioni:** funzione disattivabile; categorie finalità predefinite minimali e localizzate; vista riepilogativa «non desidera inviti» opzionale; nessuna compilazione iniziale obbligatoria. Nessun servizio remoto, profilo, consenso all'uso del programma, account Repertica o abbonamento.
- **Offline:** registrazione, ricerca, correzione e avvisi sulle preferenze funzionano con SQLite locale senza rete. Solo le attività IMAP/SMTP già esistenti, effettuate su richiesta, richiedono il provider di posta. Nessun invio, sincronizzazione o lettura automatica della casella in questo progetto.
- **Multilingua:** lingua di sistema con scelta locale IT/EN/ES/FR/DE e fallback EN, come [ECONOMY-I18N](../../ECONOMY-I18N.md). Codici interni indipendenti dalla lingua; nomi, note e dichiarazioni non tradotti automaticamente. Date formattate secondo locale ma serializzate in ISO 8601 con precisione dichiarata.
- **Import/export:** eventuale CSV UTF-8 per audit locale delle preferenze con UUID, canale, finalità, stato, data e fonte minimizzata; scegliere esplicitamente se includere nome e recapiti. Nel backup **completo** rientrano tutte le righe e lo storico eventualmente scelto, insieme al database e ai media, nella cartella liberamente selezionata. CSV integrali esistenti e backup attuali non vengono riscritti. Un futuro import ha anteprima e rollback, rifiuta riferimenti a collezionisti inesistenti e conflitti, non muta vendite né invia email.
- **Privacy:** finalità e opposizioni sono dati personali contestuali; accesso locale, no telemetria, no server Repertica, no esposizione ai tool MCP di default, no allegati sensibili o invio di elenchi a terzi. I backup esistenti **sono in chiaro**: ricordarne la protezione su dischi esterni/NAS; la scheda non li presenta come cifrati. Una esportazione pubblica di catalogo non include mai queste preferenze.

## Compatibilità, dipendenze, complessità

- **Compatibilità:** schema SQLite additivo (solo se mai approvato), compatibilità conservativa dei collector UUID e delle vendite, salvataggio/ripristino completo e CSV precedenti invariati; versioni più vecchie potrebbero non leggere schemi futuri e devono essere avvertite. Windows/macOS/Linux; possibili indirizzi Unicode e case-folding limitato senza deduzioni di identità.
- **Dipendenze già presenti:** Collector locale, Tauri, SQLite, i18n, backup, client di posta manuale. Non richiede rubriche cloud, SDK di campagne, CRM, OAuth o accesso alle email ricevute.
- **Complessità stimata:** bassa-media per campo/registro locale e UI; media-alta per verifica destinatari multipli, casi di recapiti condivisi, rettifiche, import e per evitare affermazioni normative improprie.
- **Rischi:** scambiare una preferenza per una base legale; presentare lo stato «sconosciuto» come autorizzato; trattare un rifiuto promozionale come divieto assoluto di rispondere su una vendita; conflitti tra indirizzi e persone; usare elenchi privati per invii massivi; attrito eccessivo durante l'invio individuale.
- **Mitigazioni:** campi dichiarativi e fonte, stato sconosciuto esplicito, distinzione delle finalità, conferma umana, nessun invio automatico, collegamenti univoci e informazioni limitate. Revisione legale esterna appropriata **prima** di presentare il prodotto come supporto alla conformità GDPR; nessuna garanzia di compliance fornita da una semplice preferenza.

## Alternative da confrontare prima di qualsiasi sviluppo

1. **Zero nuove funzioni:** istruzione testuale in note del collezionista; supporta pochi contatti ma è poco visibile durante la composizione.
2. **Alternativa minima consigliata per validare:** singolo campo «Non inviare inviti promozionali» + breve nota/data sulla scheda, senza analisi destinatari o tabelle complesse.
3. **Proposta completa condizionata:** più canali/finalità, storico dichiarazioni, avvisi contestuali pre-invio e export selettivo. Evitare gestione campagne, profilazione o automazione delle risposte.

## Criteri di accettazione ipotetici

1. Un collezionista storico senza preferenze appare «non indicato»; non viene considerato disponibile a campagne o contatti di massa.
2. Una dichiarazione «niente inviti email» non modifica email, vendite, certificati e incassi; è visibile localmente nel pannello contatto.
3. Un nuovo messaggio individuale a quel recapito, se mai aggiunta la verifica, produce avviso leggibile **prima** della conferma SMTP quando la finalità è promozionale; non invia nulla da solo.
4. Per un recapito non associato o condiviso tra due soggetti, l'interfaccia esplicita «preferenza non verificabile» e non attribuisce una dichiarazione a persone diverse.
5. Le comunicazioni sul ritiro di un'opera non sono etichettate automaticamente come marketing; il prodotto non certifica alcuna liceità dell'invio.
6. Backup e ripristino nella cartella scelta ricostruiscono le preferenze con UUID e date; CSV/archivi esistenti restano leggibili e invariati.
7. Esportazione per presentazione delle opere non trasporta preferenze o recapiti; export dedicato e completo richiedono scelta consapevole.
8. La UI rimane usabile offline con mouse, tastiera e screen reader in IT/EN/ES/FR/DE e fallback inglese su Windows/macOS/Linux.
9. Nessun dato viene inoltrato al server MCP o al provider di posta a causa del solo salvataggio di una preferenza; SMTP conserva conferma e comportamento già documentati.

## Fasi opzionali, nessuna autorizzata

- **A — Validazione:** 4–6 interviste con artisti/gallerie su casi reali di preferenze e comunicazioni; testare il costo cognitivo di una semplice nota/etichetta e verificare se la posta Repertica viene usata.
- **B — MVP minimo eventuale:** singola preferenza «non inviare inviti», fonte/data facoltative, ricerca locale e backup, senza aggancio all'editor.
- **C — Estensione eventuale:** finalità/canali differenziati e avviso contestuale all'invio individuale, con gestione degli indirizzi ambigui.
- **D — Portabilità e audit:** export dedicato minimizzato, import assistito solo su richiesta e storico essenziale, se richiesti da esigenze dimostrate.

## Confini e anti-doppioni

- [UX-014](REPS-UX-014-rubrica-relazioni-professionali.md) studia **identità, ruoli e cronologia generale dei rapporti**, cita solo preferenze dichiarate e rinvia un'eventuale registrazione di autorizzazione; qui si isola la **UX concreta per non ignorare un'indicazione di contatto**, collegata alla posta individuale già implementata, senza proporre una seconda rubrica.
- [UX-002](REPS-UX-002-trattative-prenotazioni.md) riguarda attività di trattativa; [UX-024](REPS-UX-024-scadenziario-incassi-piani-pagamento-vendite.md) scadenze di pagamento, non la volontà di ricevere inviti.
- La [PR tecnica #16](https://github.com/Cesare-ml/Repertica-Studio/pull/16) contiene TECH-014 sui **permessi del server MCP**, TECH-017 sulla cifratura dell'archivio e TECH-025 sulle esportazioni minimizzate: sono problemi di controllo tecnico dei dati, distinti dalle preferenze dichiarate dei destinatari. Nessun file tecnico modificato.
- [BUSINESS-MAIL-DESKTOP](../../BUSINESS-MAIL-DESKTOP.md) descrive già invio personale con conferma; questa scheda non propone di reimplementare email, autorizzare mail marketing, tracciare aperture o creare campagna newsletter.

## Certezze, ipotesi e domande riservate al proprietario

- **Certo dalla documentazione e dai file di main esaminati:** schede collezionista con email/telefono/note; IMAP/SMTP opzionale, bozze locali, conferma di invio, suggerimenti dei contatti; SQLite locale e backup in cartella scelta, i18n con fallback inglese. Nei percorsi letti non risulta un registro strutturato di preferenze.
- **Ipotesi non validate:** frequenza degli inviti personali; bisogno di distinguere più finalità; utilità reale del promemoria mentre si compone una mail; disponibilità degli utenti a registrare la fonte di una dichiarazione.
- **Domande da decidere:** (1) la posta integrata verrà usata per comunicazioni ricorrenti ai collezionisti? (2) basta una etichetta visibile nel contatto? (3) registrare preferenze in una futura rubrica condivisa o solo nei collezionisti? (4) quali finalità servono davvero e come comunicarne i limiti senza qualificazioni legali? (5) l'avviso pre-invio va introdotto solo dopo una sperimentazione? (6) quali dati delle dichiarazioni devono poter essere esportati?

**Esito:** Proposta — da valutare. Nessuna idea è approvata o assegnata; nessun invio, implementazione, modifica a codice o automazione.
