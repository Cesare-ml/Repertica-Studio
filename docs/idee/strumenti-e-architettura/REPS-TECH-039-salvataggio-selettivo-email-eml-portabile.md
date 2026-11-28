# REPS-TECH-039 — Esportazione selettiva della corrispondenza email in formato EML aperto

- **ID stabile:** REPS-TECH-039
- **Data della ricognizione:** 2026-11-22
- **Stato:** **Proposta — da valutare**
- **Filone:** Strumenti e architettura; solo studio, nessuna implementazione o approvazione.
- **Priorità ipotetica:** P2–P3, subordinata a riscontri presso utenti reali.
- **Complessità:** media per un export singolo robusto; medio-alta/alta per eventuale conservazione gestita, legami e verifica nel tempo.

## 1. Problema, utenti e riscontri nel repository

Artisti, gallerie e archivi possono ricevere per email accordi sulle condizioni di esposizione, conferme di acquisto o istruzioni su un'opera. La posta di un provider è un servizio esterno: un messaggio oggi consultabile via IMAP potrebbe non esserlo più dopo la chiusura dell'account, una cancellazione o un cambio di gestore. Repertica persegue l'indipendenza del **proprio archivio**, ma attualmente la consultazione della posta ricevuta resta legata al provider.

**Fatti verificati su main il 22/11/2026:**
- [Posta professionale](../../BUSINESS-MAIL-DESKTOP.md) è un client individuale IMAP/SMTP facoltativo; non richiede account Repertica, ma leggere una casella remota richiede connessione e credenziali del provider. Il documento dichiara esplicitamente che **non conserva i corpi dei messaggi ricevuti** e non scarica allegati ricevuti.
- [business_mail.rs](../../../src-tauri/src/business_mail.rs): mail_list_messages usa IMAP EXAMINE e restituisce le ultime 40 intestazioni; mail_read_message controlla la coppia cartella + UID + UIDVALIDITY, recupera il messaggio con BODY.PEEK[] fino a **1 MiB**, ne estrae una parte testuale e la restituisce all'interfaccia. Non espone l'originale MIME come file esportabile. Il corpo HTML viene convertito in testo non eseguibile; è una protezione da mantenere nella vista.
- [Backup](../../../src-tauri/src/backup.rs) protegge SQLite e media locali. Non equivale a un backup della casella IMAP; le password del portachiavi del sistema operativo non sono incluse.
- [Portabilità CSV](../../DATA-PORTABILITY.md) esporta registri dell'archivio, non i messaggi ricevuti.
- Non è emersa nelle funzioni lette un'esportazione selettiva di un messaggio completo in formato standard **.eml**. Questo è un limite funzionale deducibile dal codice, **non un incidente di perdita di corrispondenza riscontrato**.

**Destinatari:** professionisti che conservano corrispondenza importante a supporto della propria attività e desiderano poter aprire una copia con software indipendente dal provider; irrilevante per chi usa già un client con esportazione adeguata o non impiega la posta interna di Repertica.

**Requisiti certi di prodotto:** app Tauri gratuita, open source, database locale, scelta libera delle cartelle, export sempre possibile, nessun account/abbonamento Repertica, funzionamento offline per i dati *già acquisiti*, lingua di sistema e fallback inglese, nessun invio o scaricamento occulto.

**Ipotesi da convalidare:** frequenza di email professionali da conservare; necessità dell'intero originale con allegati rispetto al semplice PDF/testo; interesse per un export integrato anziché usare il client email già disponibile. Non sono stati condotti test IMAP reali né interviste.

## 2. Confini e verifica anti-doppione

- [TECH-031](REPS-TECH-031-allegati-bozze-posta-ripristinabili.md) riguarda **file locali collegati a bozze in uscita**, non la conservazione selettiva di messaggi già ricevuti dal server.
- [TECH-016](REPS-TECH-016-allegati-documentali-locali-multi-entita.md) studia una custodia documentale trasversale: qui il nucleo minimo è **esportare byte MIME autentici in un file indipendente**, senza introdurre una seconda gestione documentale. Un eventuale collegamento a opere rimane fuori dalla prima fase e dipenderebbe da quella valutazione.
- [TECH-001](REPS-TECH-001-pacchetto-dati-aperto.md) tratta l'interscambio dell'archivio SQLite/media; non è un backup di caselle email.
- [TECH-013](REPS-TECH-013-copie-fotografiche-condivisibili-privacy-metadati.md) riguarda la privacy delle fotografie, non la conservazione della corrispondenza.
- Le proposte della [PR Gestione e UX #14](https://github.com/Cesare-ml/Repertica-Studio/pull/14), incluse UX-014 (relazioni professionali), UX-035 (preferenze di contatto) e UX-043 (segnalazioni post-vendita), descrivono **processi professionali**, non un export interoperabile delle email. Nessun file del loro filone va modificato.
- Il documento [Posta professionale](../../BUSINESS-MAIL-DESKTOP.md) menziona in futuro cache offline, ricerca, paginazione, download di allegati e OAuth2: **questa proposta non è una sincronizzazione della casella, una cache generalizzata, una nuova autenticazione né la gestione di campagne**.

## 3. Scenari professionali da osservare

1. **Accordo ricevuto via email:** l'artista sceglie un messaggio e salva una copia EML in una propria cartella; in seguito può aprire il file anche senza server IMAP, senza cambiare il contenuto della casella o lo stato letto/non letto.
2. **Messaggio con allegati:** la copia portabile deve includere i byte effettivamente consegnati dal server, incluse le parti MIME, non solo il testo che oggi l'interfaccia mostra. Se non è possibile recuperare tutto, l'operazione fallisce: niente file etichettato «completo» se è parziale.
3. **Cambio provider:** l'utente apre la copia con un lettore EML compatibile e non dipende da password o server originali. Il file non garantisce autenticità del mittente, firma valida, recapito o valore probatorio: conserva solo il contenuto ricevuto.
4. **Cartella USB/NAS scollegata:** destinazione indisponibile o disco pieno produce un errore, senza sovrascrivere documenti precedenti né lasciare un file apparentemente definitivo ma incompleto.
5. **Messaggio spostato/cancellato o UIDVALIDITY cambiato:** fermarsi, mostrare quale messaggio non è più identificabile con certezza, invitare a riselezionare; **mai** sostituire tacitamente con un altro messaggio che ha lo stesso UID.
6. **Email riservata:** il professionista sceglie se conservare e dove; nessuna esportazione alla cartella di Repertica o al cloud avviene per il semplice fatto di aprire il messaggio.

## 4. Specifica funzionale e flusso UX proposti (non implementati)

**Fase minima da valutare — un solo messaggio, su azione esplicita:**

1. Dalla vista di una email ricevuta, azione **«Salva copia completa (.eml)»**. Mostrare chiaramente **casella, cartella, mittente, oggetto, data, dimensione indicata dal server**, spiegando che il formato può includere allegati, indirizzi e dati personali. Oggetto e mittente vanno trattati come dati non affidabili.
2. Aprire il dialogo di sistema nella cartella scelta dall'utente. Proporre nome prudente (data + identificativo neutro, caratteri ammessi, estensione .eml), consentendo modifica; mai costruire percorsi da intestazioni remote senza sanificazione. **Non sovrascrivere** un file occupato, nemmeno in caso di due esportazioni simultanee.
3. Prima di acquisire i byte, riverificare UIDVALIDITY e l'identità del messaggio; leggere con BODY.PEEK[] senza modificare i flag IMAP. Per contenuti più grandi dell'attuale limite di **1 MiB** della *vista testo*, valutare un **limite separato esplicito** per l'export, con avviso di dimensioni e streaming/memoria limitata. Non aggirare i limiti senza verifica di fattibilità e sicurezza.
4. Scrivere i **byte originali** EML, senza conversione HTML, normalizzazione di righe, ricodifica o estrazione/trasformazione di allegati. Preflight della dimensione, file temporaneo su volume adeguato e pubblicazione finale non distruttiva; se il server chiude la connessione o la lunghezza non coincide con quella attesa, conservare al massimo il temporaneo in stato chiaramente non completato, senza dichiarare successo.
5. Confermare soltanto dopo chiusura e controlli necessari: **«Copia locale salvata in: …»**, con possibilità di aprire la cartella. Spiegare: **«Il file scelto è esterno all'archivio Repertica e NON entra automaticamente nei suoi backup»**.
6. Progressi, annullamento prudente e informazioni testuali accessibili da tastiera e screen reader; nessuna nuova schermata permanente se il bisogno è raro.

**Fase separata e solo eventuale:** scelta **consapevole** di importare l'EML come documento privato gestito nell'archivio, con riferimento UUID e hash, backup/restauro e relativi costi di spazio. Prima della sua valutazione occorre decidere TECH-016; non proporre collegamenti silenziosi a opere, collezionisti o vendite. La cancellazione del messaggio sul server o della copia esterna resta sempre un'azione indipendente.

**Esclusioni:** nessun monitoraggio continuo di nuove email, sincronizzazione bidirezionale, invio automatico, consultazione di caselle senza conferma, import massivo dell'intero account, OCR, rendering attivo HTML, classificazione IA, trasmissione a MCP, attribuzione automatica della proprietà o autenticità dell'opera.

## 5. Struttura dati e possibili API — solo descrittive

La fase minima **non richiede tabelle SQLite o migrazioni**: esporta un singolo file nel filesystem scelto e restituisce uno stato, senza conservarne automaticamente il contenuto.

Contratto concettuale non eseguibile:

    EmailSelection { account_id, folder, uid, uid_validity, expected_size_bytes? }
    ExportPreview { display_subject, display_sender, size_bytes?, privacy_warning, suggested_filename }
    ExportResult { destination_local, bytes_written, sha256?, completed: true }
    exportSelectedEml(selection, destination, size_budget, consent) -> ExportResult

- Non includere password, token o corpo del messaggio in oggetti serializzati verso la UI; il salvataggio dei byte dovrebbe essere gestito dal lato Rust, non tramite base64 in Angular.
- L'impronta SHA-256, se calcolata, descrive l'**integrità della copia in quell'istante**, non prova autenticità o completezza storica di una conversazione.
- Il nome file, il percorso e il corpo non vanno nei log ordinari né nel server MCP. Token/consenso non deve dare accesso a un percorso arbitrario esterno selezionato da un assistente IA.
- Se si valutasse in seguito custodia gestita, un record ipotetico sarebbe: `EmailArchiveItem { id, account_label?, original_folder?, uid_validity?, uid?, mime_path_relative, sha256, size_bytes, acquired_at, linked_entity_ids? }`. Questa struttura richiederebbe regole di redazione dati, verifica, cancellazione e backup **distinte** dalla semplice esportazione.

## 6. Offline, riservatezza, sicurezza e lingue

- **Offline:** il download originale da IMAP **richiede rete** e una casella già configurata. L'EML salvato è invece indipendente dalla rete e apribile con software terzo. Non promettere di esportare un messaggio mai acquisito mentre la casella è offline.
- **Privacy:** gli EML possono includere destinatari, Bcc, messaggi inoltrati, IP nelle intestazioni, firme, link, allegati personali e contenuti riservati. Mostrare avviso prima della scelta della destinazione, soprattutto su dischi condivisi. Nessuna telemetria, cloud, import in Repertica, indexing o dati a host IA per impostazione predefinita. I backup esistenti non coprono i file esportati fuori dall'archivio.
- **Sicurezza MIME:** non renderizzare HTML attivo, non aprire allegati in automatico e non interpretare messaggi come comandi. Il formato EML è un involucro di dati non affidabili; il lettore esterno scelto dall'utente può avere rischi propri.
- **Sicurezza filesystem:** file regolari, percorsi risolti e consentiti fuori dalla root interna dell'app per evitare riscritture accidentali di SQLite/media, controllo symlink/junction e conflitti di nomi, nessuna sovrascrittura (create-new), pubblicazione su volume uguale quando necessario, pulizia conservativa di temporanei. Verificare comportamento reale di rename, fsync e permessi su NTFS/APFS/exFAT/NAS.
- **Impostazioni:** nessuna nuova impostazione obbligatoria. Solo eventuali preferenze locali esplicite per dimensione massima ammessa e ultima cartella usata (senza salvare un percorso personale nei report di assistenza). Non memorizzare messaggi in background.
- **Multilingua/accessibilità:** seguire lingua di sistema, fallback inglese e lingue supportate IT/EN/ES/FR/DE. Tradurre stati e avvisi, **mai** modificare o tradurre contenuto del MIME o nomi originali in esso. Prova con zoom, contrasto, tastiera, lettore schermo, nomi e oggetti Unicode; dimensione e date localizzate senza toccare byte esportati.

## 7. Dipendenze, alternative, rischi, complessità

**Dipendenze da studiare, non da installare ora:** attuale crate IMAP e API per recuperare messaggi completi senza flag di lettura; limiti di flusso e timeout dell'operazione; salvatori di file nativi Tauri e API Rust di creazione non distruttiva; fixtures MIME lecite con allegati e intestazioni non ASCII; compatibilità di lettori EML indipendenti. Nessuna nuova libreria è stata scelta.

**Alternative prima di sviluppare:** (A) documentare come esportare le email con Thunderbird o altro client standard, senza aggiungere codice; (B) salvare solo una trascrizione testuale, più semplice ma **non** una copia MIME fedele; (C) caching offline completo, molto più complesso e fuori ambito; (D) collegamento tramite TECH-016, utile solo se la custodia delle email è realmente richiesta.

**Rischi:** aumentare la superficie di trattamento di dati personali, introdurre falsi attestati di «originalità», file incompleti interpretati come completi, archivi colmi per allegati grandi, apertura di contenuto malevolo da software terzi, conferire ai backup una copertura inesistente, caricare messaggi in RAM senza limiti, differenze di semantica tra provider. L'esportazione singola è di complessità **media**; custodia gestita e conservazione storica sono **medio-alte/alte** e non vanno aggiunte come pretesto per la prima.

## 8. Criteri di accettazione e prove proposte

1. Su casella IMAP di prova, esportare un messaggio semplice e uno multipart con allegati: byte completi identici a quelli restituiti dal server nella medesima acquisizione (senza riscrivere line ending o codifiche).
2. I flag \\Seen e \\Flagged non cambiano; controllo UIDVALIDITY impedisce di salvare un messaggio diverso con UID riutilizzato.
3. Messaggi assenti, UIDVALIDITY cambiato, connessione interrotta, timeout, lunghezza inattesa e superamento limite portano a fallimento dichiarato **senza EML finale**; nessun retry automatico non autorizzato.
4. Il percorso è selezionato esplicitamente, mai costruito ciecamente da oggetto o mittente; nessun file preesistente viene sovrascritto, anche dopo collisioni/concorrenza; dati di Repertica non sono bersaglio di esportazione.
5. Con disco pieno, USB scollegata o NAS non disponibile: nessun successo parziale e nessuna alterazione della casella o del DB; handling conservativo dei temporanei e dei permessi.
6. Un file salvato resta apribile offline da almeno un lettore EML indipendente; l'importazione in Repertica e l'apertura automatica degli allegati non avvengono.
7. Backup/ripristino dell'archivio originale invariati e avviso corretto che un export esterno **non** entra nel backup.
8. Nessuna credenziale, corpo o intestazione personale entra in log, diagnostica o MCP; tutti i flussi sono accessibili e traducibili, con fallback inglese.
9. Prima della scelta di un limite superiore a 1 MiB, prove controllate con messaggi grandi e allegati su Windows/macOS/Linux, includendo memoria usata e possibilità di interruzione.

## 9. Fasi e questioni riservate al proprietario

- **Fase 0 — verificare il bisogno:** intervistare artisti e gallerie sull'uso delle email per accordi e sull'esportazione attuale con client esterni; raccogliere soltanto esempi sintetici e dati aggregati volontari. Se il bisogno non emerge, **non implementare**.
- **Fase 1 — eventuale export su richiesta:** un EML alla volta, nessuna migrazione o copia interna, UX breve e comportamento sicuro; solo dopo test reali MIME/IMAP e filesystem.
- **Fase 2 — soltanto se richiesta:** associazione esplicita di EML all'archivio come documento gestito tramite eventuale TECH-016, backup integrale verificato, esportazione e rimozione prudente.
- **Fase 3 — non presupposta:** valutare ricerca locale o ulteriori strumenti di importazione **solo** dopo consenso e misurazione dei volumi, senza confondere questa proposta con cache/sync della casella.

**Domande da valutare personalmente:** gli utenti desiderano davvero EML originale? È sufficiente indicare un client esterno? Che volume massimo di messaggi serve? Devono essere custoditi nel backup di Repertica o in una cartella scelta? Serve un legame manuale a una scheda opera o basta un file autonomo? Quali avvisi privacy sono comprensibili? Come documentare la differenza fra integrità del file e autenticità del contenuto?

**Decisione:** nessuna. Priorità **P2–P3 puramente ipotetica**, stato permanente **Proposta — da valutare** finché il proprietario non si pronuncia. Non è autorizzato alcuno sviluppo, issue, approvazione o merge.
