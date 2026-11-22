# Posta — client desktop IMAP / SMTP

## Provenienza e confine

L'implementazione deriva dai principi e dai contratti di
Cesare-ml/Gestionale-Opere/backend/modules/business-mail/
(in particolare personal-imap.js, personal-smtp.js, personal-drafts.js,
personal-preferences.js, personal-contacts.js, secret-vault.js e
docs/implementation/BUSINESS_MAIL_FOUNDATION_2026-11-03.md).

Lì il modulo è in Node/Express, basato su un account specifico e su DPAPI in ambiente Windows; Repertica Studio è invece un'app Tauri distribuita a chiunque. Perciò **non viene copiato il server e non vengono inserite credenziali o host personali**, ma i contratti vengono trasposti in Rust, con configurazione per più account e portachiavi nativo multipiattaforma.

Questa funzione è **posta individuale**, non il prodotto Email Platform per campagne, marketing o newsletter.

## Implementato

- Voce Posta nel menu desktop, impostazioni caselle con account multipli.
- Configurazione esplicita IMAP TLS porta 993; SMTP con TLS diretto 465 o STARTTLS obbligatorio 587.
- Password IMAP e password SMTP facoltativamente separata nel portachiavi di sistema. Configurazione senza segreti in SQLite; i backup non contengono password. Se il portachiavi non è disponibile, le operazioni sulle credenziali falliscono senza fallback in chiaro.
- Test IMAP e discovery cartelle; lettura di 40 intestazioni più recenti per cartella in modalità EXAMINE / PEEK. Non modifica lo stato di lettura.
- Identità di messaggi controllata con coppia UIDVALIDITY + UID + cartella: un UIDVALIDITY cambiato blocca la lettura stale.
- Lettura sicura della prima parte MIME text/plain; quando è presente solo HTML, questo viene convertito in testo semplice senza eseguire script né caricare risorse remote. I messaggi superiori a 1 MiB sono rifiutati in questa fase.
- Composizione semplice con To, Cc, Bcc, oggetto e corpo; reply/forward tramite citazione semplice.
- Allegati in uscita scelti esplicitamente da file locali, massimo cinque per messaggio, massimo 5 MiB ciascuno e 15 MiB complessivi. I file vengono letti soltanto al momento dell'invio; le bozze salvano esclusivamente i riferimenti ai percorsi, non una copia dei contenuti.
- Invio SMTP reale **solo dopo conferma esplicita dell'utente**. Se l'esito non è certo, si mostra un errore conservativo: non inviare automaticamente di nuovo.
- Copia Inviati via IMAP APPEND solo se il nome cartella viene configurato. Il risultato distingue messaggio accettato dal server SMTP e copia salvata; non promette recapito alla casella destinataria.
- Bozze offline in SQLite con revisioni ottimistiche, segnalazione conflitto, salvataggio/riapertura/rimozione. Bozze incluse nel backup del database locale.
- Firma in testo semplice per ogni account, inserita all'apertura di un nuovo messaggio o risposta.
- Suggerimenti di contatti basati sui collezionisti con indirizzo email già presenti nell'archivio locale; nessun accesso esterno ai contatti.
- Interfaccia localizzata IT/EN/ES/FR/DE, fallback in inglese.

## Sicurezza e privacy

La password passa al backend Tauri soltanto alla configurazione. Il backend usa il gestore credenziali di Windows, il Portachiavi macOS e Secret Service Linux. Non è inclusa nell'oggetto MailAccount restituito all'interfaccia, in tabella mail_accounts, nelle migrazioni né nei backup. Le bozze **contengono dati personali in chiaro nel database locale**, nello stesso modello di archiviazione locale di Repertica Studio: il backup del database le include e va protetto dall'utente.

Le connessioni IMAP/SMTP sono autenticazioni reali e quindi richiedono una connessione Internet. Non richiedono account Repertica, servizi di Repertica Online, endpoint HTTP propri o un abbonamento. Il client non avvia sincronizzazioni in background. L'utente deve autorizzare le proprie credenziali, usare password per app quando richiesto dal provider e verificare la configurazione IMAP/SMTP dal provider.

**Nessun upload automatico**, tracciamento, importazione remota di contenuti, newsletter o agente IA collegato alla casella. L'invio non fa parte delle automazioni.

## Limiti MVP

- Nessun supporto OAuth2/XOAUTH2: provider che rifiutano password e password per app non sono ancora compatibili.
- Niente rendering HTML (solo conversione in testo), download degli allegati ricevuti, ricerca remota, paginazione oltre gli ultimi 40 messaggi, download di messaggi superiori a 1 MiB, sincronizzazione offline della posta ricevuta o contatti esterni.
- Le bozze non sono ancora sincronizzate con la cartella IMAP Drafts; solo in archivio Studio.
- La firma è plain text. La copia Inviati è separata dall'accettazione SMTP e può fallire.
- L'invio non implementa ancora ricevute di consegna; l'accettazione SMTP non garantisce che il messaggio venga consegnato. Gli allegati in uscita sono supportati entro limiti prudenti, ma i riferimenti nelle bozze non sopravvivono allo spostamento dei file locali.
- La connessione IMAP iniziale non implementa ancora un timeout applicativo complessivo; in caso di server irraggiungibile i tempi dipendono dalle impostazioni della rete.
- Credenziali protette dal sistema: il backup su altro computer ripristina l'account, ma richiede di reinserire le password nel nuovo sistema.

## Verifica

1. Avviare npm run tauri:dev dopo npm install; aggiungere una casella da Posta.
2. Inserire server IMAP, SMTP, username e password del provider, quindi salvare.
3. Cliccare Test IMAP; usare Connetti per elencare le cartelle e recuperare le ultime 40 intestazioni.
4. Aprire un messaggio con parte text/plain e verificare che non cambi il flag letto.
5. Creare e salvare una bozza; riaprire dopo il riavvio, controllare la persistenza.
6. Usare una casella test, confermare l'invio SMTP esplicitamente, verificare risultati nella cartella del provider.
7. Confermare il comportamento in caso di password errata, rete interrotta e UIDVALIDITY cambiato.
8. Verificare che un backup contenga la configurazione ma non password, e che sul nuovo computer il portachiavi vada riconfigurato.

**Attenzione:** i test automatizzati verificano i contratti offline e la compilazione, ma non possono provare un login IMAP/SMTP vero senza credenziali e casella test autorizzata.

## Sviluppi consigliati

1. Download degli allegati ricevuti, anteprima MIME sicura e gestione dell'archiviazione degli allegati delle bozze.
2. Ricerca remota e paginazione IMAP, gestione \Seen/\Flagged/move, cartelle speciali più robuste.
3. Caching locale opzionale cifrato di messaggi e allegati, con sincronizzazione UIDVALIDITY/QRESYNC e gestione conservativa dei conflitti.
4. OAuth2 per provider compatibili; gestione invii e bozze IMAP sul server, senza sovrascritture automatiche.
5. Editor ricco sicuro, template, firme multiple, rubrica locale e collegamenti espliciti con i collezionisti.
