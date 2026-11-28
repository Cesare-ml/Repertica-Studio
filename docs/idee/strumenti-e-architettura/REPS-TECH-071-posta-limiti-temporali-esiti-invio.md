# REPS-TECH-071 — Posta: attese di rete limitate ed esiti di invio non ambigui per l'utente

- **Stato:** Proposta — da valutare
- **Data:** 2026-11-23
- **Priorità proposta:** P2 solo se i blocchi sono riproducibili; P1–P2 per eventuale rischio concreto di reinvio; P3/guida o nessun intervento altrimenti.
- **Complessità:** media per lettura IMAP; medio-alta per scadenze end-to-end e distinzione degli esiti SMTP/APPEND.
- **Nessuna approvazione, assegnazione o implementazione.**

## 1. Problema, riscontro e destinatari

Artisti e gallerie che usano la Posta integrata possono trovarsi in atelier, fiere o reti instabili. Un server che non risponde non dovrebbe rendere indefinitamente indisponibile la gestione locale delle bozze. Ancora più delicato: un messaggio accettato dal server SMTP ma con copia Inviati non confermata non va presentato come sicuramente non inviato.

**Riscontri statici sul main, non test runtime:**
- [BUSINESS-MAIL-DESKTOP.md](https://github.com/Cesare-ml/Repertica-Studio/blob/main/docs/BUSINESS-MAIL-DESKTOP.md) dichiara già l'assenza di un timeout applicativo complessivo per la connessione IMAP iniziale. La proposta approfondisce questa limitazione nota, non la presenta come scoperta di un bug.
- [business_mail.rs, righe 165–175](https://github.com/Cesare-ml/Repertica-Studio/blob/main/src-tauri/src/business_mail.rs#L165-L175): `with_imap` usa `imap::connect`, login, azione e logout. Nel wrapper non si vedono limiti espliciti per queste fasi; ignorare l'errore di logout non ne limita il tempo. Le eventuali scadenze interne delle librerie/OS non sono state misurate.
- [righe 207–221](https://github.com/Cesare-ml/Repertica-Studio/blob/main/src-tauri/src/business_mail.rs#L207-L221): operazioni su worker bloccanti, non sulla UI. [Frontend, righe 59–65 e 122–155](https://github.com/Cesare-ml/Repertica-Studio/blob/main/src/app/business-mail.component.ts#L59-L65): il flag comune `busy` resta attivo fino a risoluzione della promessa; Connetti esegue prima cartelle e poi lista messaggi.
- [business_mail.rs, righe 484–502](https://github.com/Cesare-ml/Repertica-Studio/blob/main/src-tauri/src/business_mail.rs#L484-L502): l'invio SMTP ha già un messaggio conservativo in caso di esito incerto e vieta il reinvio automatico; dopo accettazione può fare IMAP APPEND tramite lo stesso wrapper. L'esito finale distingue accettazione da copia Inviati. Un APPEND lento può quindi ritardare la visualizzazione dell'accettazione. Non si presume che lettre sia privo di timeout propri.

Non sono stati riprodotti blocchi, perdite, duplicazioni o incidenti reali. Nessuna connessione a caselle reali effettuata.

## 2. Scenari d'uso

1. A una fiera senza Internet, Test IMAP termina con un messaggio comprensibile; l'artista continua a catalogare e preparare bozze locali.
2. Il provider risponde al login ma non alla lista o al logout: distinguere risposta utile già ottenuta da chiusura della sessione incompleta, senza lasciare worker illimitati.
3. Un preventivo viene accettato via SMTP ma la copia IMAP tarda o fallisce: l'artista vede che l'invio è stato accettato, che il recapito non è garantito e che Inviati è un esito distinto. Non reinviare per riparare la copia.
4. La connessione cade mentre si attende la risposta SMTP finale: esito sconosciuto, verifica manuale col provider prima di qualsiasi nuovo invio esplicitamente confermato.

## 3. Requisiti certi e ipotesi di specifica

**Certi (vincoli del progetto e garanzie esistenti):** app gratuita/offline-first senza account Repertica, nessuna attività di rete non richiesta; IMAP e SMTP sono servizi esterni facoltativi. TLS validato, UIDVALIDITY, EXAMINE/PEEK, password solo nel portachiavi, limiti dei messaggi e conferma esplicita dell'invio rimangono. Nessuna modifica a opere, vendite, allegati contabili, backup o dati storici.

**Ipotesi da validare:** scadenze per connessione (inclusi risoluzione nomi e TLS), inattività in lettura/scrittura e operazione complessiva, misurate con clock monotono, non con ora di sistema. Un timeout del solo await/frontend non ferma necessariamente il worker o il socket; non spacciarlo per annullamento. Limitare worker simultanei e garantire liberazione delle risorse oppure dichiarare un limite della piattaforma. Non introdurre retry automatici, neppure di APPEND, nella prima fase. Evitare di disattivare le protezioni TLS per migliorare tempi.

Per prima cosa verificare con le versioni effettive dei crate se sono configurabili socket, TLS, logout e timeout SMTP, e se DNS è limitabile in modo affidabile su tutti i sistemi. Solo in caso negativo valutare alternative; non assumere che un timeout di socket da solo limiti l'intero flusso (risposte lente ma continue).

### Flusso UX ipotetico

- In Posta, azione manuale → stato leggibile della fase (connessione/lettura/chiusura); eventuale errore localizzato con consiglio concreto e possibilità di riprovare manualmente. Una lista incompleta non va mostrata come casella vuota. Un risultato tardivo non deve sostituire la vista di una casella differente.
- Le bozze e i moduli locali rimangono utilizzabili indipendentemente dalla rete, evitando conflitti con operazioni in corso. La cancellazione esplicita di una lettura è un'opzione successiva solo se il backend può garantirne l'effetto o descriverne onestamente i limiti.
- In invio, separare stato SMTP (non tentato, rifiuto certo se distinguibile, esito sconosciuto, accettato) da stato copia Inviati (non configurata, in corso, salvata, non confermata). Non affermare «non inviato» dopo un timeout ambiguo; non equiparare accettazione a recapito. Se serve un aggiornamento anticipato dell'accettazione, studiare il contratto IPC evitando che un evento perso faccia regredire uno stato già noto.
- Se si torna alla composizione dopo esito sconosciuto, proteggere dall'invio ripetuto involontario; preservare una bozza già salvata secondo il contratto esistente. Chiarire che controllare Inviati è solo un indizio, non prova negativa di mancata accettazione.

## 4. Dati e contratto puramente descrittivi

Nessuna nuova tabella nella fase iniziale. Possibile stato effimero in memoria: identificativo richiesta, identificativo locale account, tipo/fase, durata monotona, categoria di esito, certezza dell'esito SMTP e stato copia. Correlazione degli eventi solo all'operazione e alla vista correnti. I codici stabili sono separati dalle frasi tradotte. Nessun corpo, credenziale, indirizzo destinatario o percorso allegati nei dati diagnostici. Non proporre un registro persistente di invii, né un'outbox con consegna garantita.

Eventuale API concettuale: avvio delle richieste esistenti con contesto e budget temporale interno, stato consultabile o esito tipizzato; eventuale annullamento solo di letture. Non è una firma da implementare. Dopo crash, lo stato effimero non è una prova dell'esito remoto: la bozza non va scambiata per una coda di reinvio.

## 5. Offline, impostazioni, lingue, privacy, portabilità

- Offline: nessuna chiamata se non richiesta, nessuna sonda esterna; la diagnosi avviene nel tentativo esplicito. No cache obbligatoria della posta, telemetria, servizio residente o polling.
- Impostazioni: prima adottare limiti ragionevoli e misurati, senza nuovi campi; eventuale profilo connessione lenta avanzato e limitato soltanto dopo validazione. Non salvare credenziali o configurazioni di rete aggiuntive nei backup.
- IT/EN/ES/FR/DE, lingua di sistema e fallback inglese; stati leggibili da screen reader, non comunicati solo con colore o animazioni. Le categorie di errore non devono contenere testo grezzo del server, password, destinatari o host privati nei report condivisi.
- Import/export: nessuna variazione a CSV, schema o backup esistenti nella prima fase; non si promette esportazione della posta remota. Eventuali preferenze future seguono le convenzioni locali di backup con documentazione esplicita, senza segreti.
- Compatibilità: Windows/macOS/Linux, proxy/reti lente, IPv4/IPv6 e più indirizzi DNS, provider con diverse latenze. Nessuna modifica del protocollo MCP, del client SMTP del provider o delle impostazioni globali dell'OS.

## 6. Dipendenze, alternative, rischi

Dipendenze da verificare: stack IMAP/native-tls/TcpStream, lettre, gestione worker Tauri e comportamento DNS/OS. Preferire capacità esistenti, nessun nuovo crate o servizio presunto. Prima alternativa: guida che chiarisce i tempi variabili e suggerisce client di posta esterno; documentare accettazione SMTP e copia separata già presenti. Seconda: limitare solo la lettura IMAP e migliorare diagnosi; non modificare la pipeline di invio se non emerge bisogno. Terza, solo se necessaria: esiti anticipati dell'invio e del successivo APPEND.

Rischi: timeout troppo breve su reti lente, worker orfani, risposte tardive, accumulo di thread, falsi negativi SMTP e invii duplicati, falsi positivi sulla copia Inviati, errori di login confusi con assenza di rete. Una deadline applicativa non garantisce che il server non abbia già compiuto un'azione. Mai terminare arbitrariamente il processo durante SMTP per simulare annullamento. L'assenza della copia Inviati non dimostra mancata consegna.

## 7. Test e criteri di accettazione (proposti, non eseguiti)

Con server sintetici controllati e caselle di prova autorizzate, non con dati reali:
- Simulare DNS lento/irrisolvibile, TCP non raggiungibile, TLS bloccato o certificato invalido, login/list/fetch/logout senza risposta e flusso di byte lento. Misurare tempo alla risposta UI, durata worker e socket residui; definire soglie per fase e budget complessivo sulla base delle misure. Verificare 20 tentativi manuali sequenziali senza accumulo non limitato di risorse.
- Durante una lettura lenta, aprire archivio e gestire bozza locale senza perdere testo; cambiare casella e assicurarsi che il risultato tardivo non aggiorni la nuova vista. Nessuna chiamata in background.
- SMTP: rifiuto prima dell'accettazione, caduta prima/durante/dopo DATA, risposta di accettazione seguita da APPEND non responsivo o connessione caduta dopo APPEND. Verificare stato prudente e zero reinvii o APPEND duplicati automatici; nessuna promessa di recapito. Non dedurre certezza dalle sole stringhe di errore di libreria.
- Verificare TLS, privacy, UIDVALIDITY e PEEK invariati; niente credenziali, indirizzi o contenuti in log. Tastiera, screen reader e localizzazioni/fallback inglese. Regressione backup/export e uso locale a rete spenta.
- Se non si può garantire un arresto effettivo del worker su una piattaforma, dichiararlo e non offrire un comando che promette tale arresto. Una semplice scadenza grafica non soddisfa il criterio tecnico.

## 8. Fasi e decisione del proprietario

Fase 0: prove su reti simulate e 3–5 professionisti che usano Posta; censire limiti delle librerie e comportamenti esistenti, stabilire se la guida basta. Fase 1 eventuale: scadenze affidabili per lettura e diagnostica, senza cambiare SMTP. Fase 2 eventuale e separata: gestione esplicita della fase post-accettazione SMTP/APPEND e protezione contro reinvio involontario. Fase 3 facoltativa: profilo rete lenta o annullamento reale delle letture, soltanto con necessità dimostrata.

Domande: quali attese sono davvero problematiche? Si usa la Posta integrata in reti instabili? È prioritario rendere disponibile prima l'esito SMTP o è sufficiente la separazione esistente? Quale margine di tempo per reti lente senza esporre a worker indefiniti? Se non emerge alcun problema, chiudere la valutazione senza sviluppo.

## 9. Confini anti-doppioni e fonti

Confrontati README e [MVP](https://github.com/Cesare-ml/Repertica-Studio/blob/main/docs/MVP.md), documentazione Posta, backend e componente Angular su main; indici delle 70 proposte TECH e delle 75 [UX](https://github.com/Cesare-ml/Repertica-Studio/pull/14). TECH-062 concerne operazioni **locali** lunghe; qui la rete ha esiti remoti non reversibili. TECH-056 è autenticazione OAuth, TECH-031 allegati delle bozze, TECH-039 esportazione EML, TECH-015 report tecnico su richiesta, UX-035 preferenze di contatto e UX-067 modifiche non salvate: non riproporre questi strumenti. La limitazione IMAP già citata nelle docs è il punto di partenza, non un nuovo requisito approvato. Solo proposta documentale in questa cartella; nessun codice, test, configurazione o file dell'altro filone modificato.
