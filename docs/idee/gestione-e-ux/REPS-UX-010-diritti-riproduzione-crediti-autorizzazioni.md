# REPS-UX-010 — Diritti di riproduzione, crediti e autorizzazioni d'uso

- **ID stabile:** REPS-UX-010
- **Data:** 2026-11-21
- **Stato:** Proposta — da valutare
- **Filone:** Gestione e UX
- **Priorità proposta:** P2 (da verificare con artisti, gallerie e archivi)
- **Perimetro:** esclusivamente analisi; nessuna implementazione, approvazione o assegnazione

## Problema professionale, utenti e confini verificati

Il branch `main` registra opere, fotografie (immagine primaria e galleria), collezionisti, vendite e certificati con snapshot immutabili; non contiene un registro strutturato delle **autorizzazioni alla riproduzione e pubblicazione delle immagini**. Un artista che consegna una fotografia a un curatore, una galleria che prepara un catalogo, o un archivio che conserva foto realizzate da terzi può dover distinguere: **titolarità dichiarata dell'opera**, eventuali diritti sulla fotografia, permesso per un uso specifico, limitazioni temporali, crediti da attribuire e prove documentali disponibili. Il software non può dedurre questi diritti da proprietà fisica, vendita o certificato.

La proposta **non** duplica [UX-007](REPS-UX-007-dossier-presentazione-opere.md), che definisce **cosa mostrare e come impaginare un dossier**, né [UX-009](REPS-UX-009-progetti-espositivi-checklist.md), che organizza eventi e opere, né [UX-005](REPS-UX-005-edizioni-multipli-serie-numerate.md), che distingue esemplari fisici in tiratura. Anche [TECH-001](https://github.com/Cesare-ml/Repertica-Studio/pull/16) riguarda **portabilità dei dati**, non autorizzazioni. Qui si ipotizza esclusivamente un **registro manuale di fatti e attestazioni dichiarati**, senza consulenza legale, licenze automatiche o verifica giuridica.

## Scenari e destinatari

1. **Catalogo di una mostra:** una galleria chiede la fotografia di tre opere per un catalogo cartaceo e un post social. L'operatore registra due usi distinti, ambito, periodo, eventuale citazione obbligatoria e fonte del consenso; un permesso per la stampa **non viene esteso** automaticamente al web.
2. **Fotografo terzo:** un artista ha la titolarità dichiarata dell'opera, ma la fotografia è stata eseguita da un professionista. La scheda immagini indica il **credito fotografico**, la fonte dell'autorizzazione dichiarata e gli usi consentiti/non accertati, senza presumere che siano identici ai diritti sull'opera.
3. **Materiale storico o incerto:** l'archivio possiede un file, ma non trova il documento di autorizzazione. L'interfaccia mostra **«Non documentato»**, non «Libero uso», anche quando il file è nell'archivio locale.
4. **Richiesta con termine:** per un uso espositivo con scadenza, l'utente può consultare una lista delle autorizzazioni da rivedere all'apertura dell'app. Nessun promemoria inviato a terzi o controllo in background.

## Flussi UX proposti

- **Scheda opera → Diritti e crediti:** panoramica di dichiarazioni sull'opera (se presenti), note di attribuzione e riferimenti; stato di conoscenza distinto da uno stato autorizzativo. I campi non compilati sono esplicitamente **non verificati**.
- **Galleria immagini → Crediti e permessi:** per ciascuna foto scegliere il fotografo/ente dichiarato, il testo di credito e il riferimento a un documento; stessa opera può avere immagini con permessi differenti. Il cambio di copertina della galleria (oggi disponibile su `main`) non deve cambiare il permesso associato **all'identità della foto**.
- **Nuova autorizzazione registrata:** selezionare opera e/o singole fotografie, destinatario (testo libero o riferimento opzionale), usi dichiarati (catalogo, stampa, sito, social, altro), territorio eventuale, date inizio/fine, eventuale esclusività **dichiarata**, condizioni, fonte e stato documentale. Prima di salvare mostrare un riepilogo leggibile e l'eventuale mancanza di prova.
- **Vista di consultazione:** filtri per opera, immagine, destinatario, ambito d'uso e prossime scadenze; etichette testuali `documentato`, `dichiarato`, `non documentato`, `revocato/da verificare`, senza certificare efficacia legale.
- **Preparazione di un dossier (solo se UX-007 sarà approvata):** eventuale **avviso non bloccante** sulle immagini con stato non documentato, ma nessun controllo può dichiarare automaticamente «pubblicazione lecita». L'export resta un gesto esplicito separato e con selezione dei dati pubblicabili.
- **Accessibilità:** stati espressi in testo e non solo in colore; moduli lineari utilizzabili da tastiera, etichette per screen reader e messaggi di attenzione collegati ai rispettivi campi.

## Dati proposti (esclusivamente descrittivi)

- **Dichiarazione sull'opera:** UUID, `artwork_id`, nominativo/ente dichiarato, ruolo (autore, titolare dichiarato, rappresentante, sconosciuto), fonte e data di acquisizione della dichiarazione, affidabilità **non automatica**, note interne; nessuna identità anagrafica certificata dal software.
- **Crediti delle immagini:** `image_id` ove esiste; per la copertina attuale, serve un identificatore persistente del file o un collegamento esplicito non basato sulla sola posizione nel layout; fotografo/ente, formula del credito, riferimento documento e note.
- **Autorizzazione registrata:** UUID; collegamenti opzionali a opera e a una o più fotografie; ambito, canale, destinatario, intervallo di validità con eventuale incertezza, territorio, condizioni, fonte, consenso dichiarato, riferimento ad allegato locale, date di registrazione/rettifica, indicazione di revoca o scadenza. **Un permesso generico non si applica implicitamente a tutte le foto**.
- **Relazioni e storia:** le modifiche materiali a durata, uso o revoca devono lasciare traccia di rettifica; non cancellare per effetto collaterale una registrazione collegata a un dossier già generato. Un allegato può essere un contratto scansito, ma **il software non lo interpreta né lo valida**.
- **Fonte di verità:** `artworks.status` resta commerciale; `sales` registra una vendita (non un trasferimento automatico di diritti); `certificates` rimane intatto. Non creare prezzi, royalty o ricavi dalle autorizzazioni: sono domini giuridici/economici ulteriori e non decisi.

## Impostazioni, offline e lingue

- Nessun account, cloud, firma remota, marketplace, scraping o verifica di licenze online. Consultazione, registrazione e ricerca operano su SQLite e media locali, anche senza connessione.
- **Impostazioni facoltative:** dicitura di credito suggerita, tipi di uso ricorrenti come lista modificabile, avvisi scadenze solo all'apertura dell'app; nessun permesso preconfermato per le opere nuove o vecchie. Preset mai equiparati ad autorizzazioni.
- Lingua di sistema con selezione manuale IT/EN/ES/FR/DE e fallback inglese secondo [ECONOMY-I18N](../../ECONOMY-I18N.md). Tradurre categorie e avvertenze; **non tradurre** automaticamente contratti, condizioni o formule di credito immesse dall'utente. Conservare codici strutturati indipendenti dalla lingua.

## Portabilità, compatibilità e privacy

- Backup/ripristino verificati dell'archivio devono includere record e allegati **solo se** questa proposta fosse approvata e realizzata; i documenti salvati fuori dalla cartella gestita rimangono esterni. Non cambiare il formato di backup in questo incarico.
- Export **interno completo** futuro: CSV delle autorizzazioni con UUID, riferimenti, date/stati; JSON versionato per relazioni e prove, allegati nel pacchetto opzionale. Import manuale con anteprima, convalida riferimenti/duplicati, idempotenza per UUID e rollback. La compatibilità con [TECH-001](https://github.com/Cesare-ml/Repertica-Studio/pull/16) è un vincolo concettuale, non una dipendenza operativa approvata.
- Export **verso terzi**: versione minimizzata con opera, titolo, soli crediti pubblicabili e ambito selezionato; **mai** allegare automaticamente contratti privati, recapiti, nomi di collezionisti, prezzi, note interne o prove complete. Mostrare anteprima e chiedere salvataggio in cartella scelta, senza upload automatici.
- Compatibilità archivi pregressi: assenza di una registrazione equivale a **dato sconosciuto**, mai a permesso implicito. Migrazioni eventuali solo additive e versionate; backup più recenti non devono essere dichiarati apribili con versioni precedenti.

## Complessità, rischi, alternative

- **Complessità:** media per registro e filtri; medio-alta per collegamento robusto a immagini e rettifiche storiche; alta per eventuale interpretazione giuridica (esclusa) e per trasferibilità semantica senza esporre dati riservati.
- **Dipendenze:** UUID opere, identità affidabile dei media, storage locale e allegati protetti, i18n, backup. Integrazione con UX-007/009 **opzionale** e sottoposta a ulteriori decisioni.
- **Rischi:** falsa garanzia di titolarità, ambiti di uso confusi, dati sensibili divulgati, «revocato» interpretato come decisione legale, documenti scaduti ma non aggiornati, trasformare il software in un sistema di contrattualistica o consulenza.
- **Alternative:** A) soli campi liberi «credito» e «note diritti» nelle immagini/opere (costo basso ma poco ricercabili); B) gestire accordi in cartelle di documenti esterne, con minore controllo contestuale; C) modello strutturato con verifica **umana** e nessuna conclusione automatica (questa proposta).

## Criteri di accettazione ipotetici

1. Per una medesima opera, una foto autorizzata per catalogo e una con permesso non documentato restano distinguibili anche quando cambia la copertina.
2. Una vendita o l'emissione di un certificato non trasforma uno stato «non documentato» in autorizzazione all'uso.
3. Inserire/scadere/revocare un permesso non cambia retroattivamente vendita, certificato o dossier già stampato; rettifiche e fonti restano consultabili.
4. Un'esportazione pubblica non contiene contatti privati, contratti, note interne o metadati di percorso; nessun documento è inviato a servizi esterni.
5. Archivio pregresso senza record diritti mostra «non documentato», non presunzioni positive; import ripetuto con stessi UUID non duplica autorizzazioni.
6. Backup/ripristino di prova conservano collegamenti alle foto, allegati e rettifiche; il programma lavora senza Internet su Windows/macOS/Linux.
7. Flussi principali navigabili a tastiera e leggibili con screen reader; lingua OS oppure fallback inglese; la traduzione UI non altera il testo dei documenti.

## Fasi di studio (NON roadmap approvata)

- **A:** verificare con utilizzatori reali il bisogno; bozza UX del semplice credito per foto e stato «documentato/non documentato».
- **B:** registro autorizzazioni per ambito, periodo, destinatario, fonte e rettifiche, consultabile offline.
- **C:** export selettivo, collegamento facoltativo ai dossier e prove di accessibilità/interoperabilità su archivi grandi.

## Requisiti certi, ipotesi, domande al proprietario

- **Certi dal repo:** vendita, certificati, foto e schede opere sono già locali; nessun modulo diritti; foto primarie/promozione da galleria esistenti; account, cloud e invii non richiesti.
- **Ipotesi:** esiste una frequenza sufficiente di richieste di pubblicazione, cataloghi e foto di terzi da giustificare un registro strutturato; le distinzioni per singola foto riducono errori pratici.
- **Da decidere:** il pubblico primario è artista singolo o galleria/archivio? È utile una scheda permessi per foto già nell'MVP o basterebbe un campo crediti? Conservare copie dei documenti o soli riferimenti? Come trattare liberatorie di persone ritratte, opere in pubblico e diritti di terzi? Serve tracciare usi **effettivamente** avvenuti, separatamente dalle autorizzazioni?

**Nessuna autorizzazione allo sviluppo. Stato invariato: Proposta — da valutare.**
