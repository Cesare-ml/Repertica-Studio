# REPS-UX-009 — Progetti espositivi: selezioni, scadenze e checklist

- **ID stabile:** REPS-UX-009
- **Data:** 2026-11-21
- **Stato:** Proposta — da valutare
- **Filone:** Gestione e UX
- **Priorità proposta:** P2, da validare per studi e piccole gallerie
- **Perimetro:** sola progettazione documentale, non autorizza sviluppo

## Punto di partenza verificato e confini con le idee precedenti

In [main](../../MVP.md), un'opera conserva **ubicazione** e **stato commerciale**, con fotografie e metadati, ma non esistono entità **mostra/progetto espositivo** o una pianificazione condivisa di opere, tappe e materiali. [UX-001](REPS-UX-001-provenienza-movimentazioni.md) propone **registrazioni di trasferimento fisico e custodia** (anche verso mostre), non la regia operativa dell'evento. [UX-007](REPS-UX-007-dossier-presentazione-opere.md) riguarda **documenti informativi da presentare a terzi**, non tempi, attività e verifica delle preparazioni. [UX-002](REPS-UX-002-trattative-prenotazioni.md) e [issue #13](https://github.com/Cesare-ml/Repertica-Studio/issues/13) riguardano trattative e finanza, non la preparazione di esposizioni. La [PR #16 tecnica](https://github.com/Cesare-ml/Repertica-Studio/pull/16) include automazioni e portabilità proposte, nessuna delle quali definisce un evento espositivo.

## Problema e destinatari

Una mostra, open studio o partecipazione a fiera comporta **selezionare opere, confermare disponibilità, preparare foto/didascalie, prenotare trasporto, controllare imballaggi e seguire scadenze**. Attualmente le informazioni rimarrebbero sparse fra note delle opere e file esterni. La proposta è una **checklist locale collegata alle opere**, non un software di biglietteria, project management generalista, calendario cloud o CRM.

## Tre scenari realistici

1. **Collettiva in galleria:** l'artista seleziona sei opere candidate, ne conferma quattro, annota allestimento e scadenza per consegna immagini. Una delle sei viene venduta: il progetto mostra **un avviso di potenziale conflitto**, senza modificare lo stato commerciale dell'opera.
2. **Open studio personale:** prima dell'apertura si spuntano assicurazione già verificata (se richiesta), etichette, luce, imballaggi e riprese fotografiche. Una checklist standard è modificabile e non presenta il completamento come conformità legale.
3. **Fiera con diverse date:** organizzatore, inizio e fine evento, deadline materiali e rientro ipotetico. Le opere candidate rimangono nello studio fino all'eventuale **movimento esplicito** (se UX-001 verrà approvata): un evento non dimostra che un'opera sia stata consegnata.

## Esperienza utente ipotetica

- **Progetti → Nuova mostra**: titolo, tipologia, sede descrittiva, organizzatore facoltativo, periodo previsto (con supporto date ancora non certe), note interne. Non serve registrare un account, una sede geolocalizzata o un soggetto esterno.
- **Opere candidate**: ricerca nell'archivio esistente, selezione multipla e stato di partecipazione `candidate`, `confirmed`, `withdrawn` oppure `returned` **solo per il progetto**. L'interfaccia mostra separatamente `artwork.status` reale: **la conferma della mostra non prenota né sposta l'opera**.
- **Controllo preliminare**: riepilogo per progetto di fotografie disponibili, dimensioni, descrizione, stato commerciale e eventuali sovrapposizioni temporali con altri progetti. Un overlap è un **segnale di verifica**, non un divieto automatico: un'opera può partecipare a mostre di date non sovrapposte o in forme digitali.
- **Checklist**: righe modificabili (fotografie inviate manualmente, imballaggio, etichette, trasporto, accordi, montaggio, rientro), con completamento sì/no, data limite opzionale, note e riferimento opera o progetto. Una «scadenza superata» appare all'apertura dell'app senza promettere notifiche a programma chiuso.
- **Linea temporale**: elenco attività «da preparare», «in corso», «completate» e milestone della mostra. Nessuna generazione di e-mail o invio a terzi; la registrazione di «inviato» è solo una **dichiarazione manuale**.
- **Vista riepilogativa**: tabella stampabile per coordinamento con terzi, con scelta esplicita dei dati da esportare e anteprima che **nasconde per default** costi, prezzi storici, clienti, note interne e ubicazioni private.
- **Accessibilità**: cambio stato con pulsanti/etichette (non drag-and-drop obbligatorio), focus logico, check leggibili da screen reader, stati con testo oltre al colore, test su layout compatti.

## Dati e relazioni proposti (solo descrizione, non API operative)

- `exhibition_project`: UUID, titolo, categoria, venue_name facoltativo, indirizzo **solo se necessario**, organizzatore testuale, date inizio/fine (certe o stimate), descrizione, fase `planning`/`active`/`closed`/`cancelled`, timestamp.
- `exhibition_artwork`: UUID, project UUID, artwork UUID, fase della **partecipazione** (non della vendita), titolo/didascalia espositiva facoltativa, ordine, note su installazione, date e legami logistici **opzionali**. Vincolo di unicità del collegamento nella stessa mostra, mantenendo cronologia per ritiri/cambiamenti.
- `exhibition_task`: UUID, project UUID, artwork UUID opzionale, tipo suggerito o testo libero, scadenza con indicatore di incertezza, stato, completata il, note, ordine. Non registrare autore utente/account inesistente nell'MVP.
- Gli elenchi di opere disponibili vengono da record correnti, **non** sono copie immutabili o una seconda fonte di vendita. Per eventuali schede stampa storiche il proprietario deve decidere tra **snapshot esplicito** e dati live, evitando una lista che cambia senza avviso.
- Non creare «trasferimenti», «prestiti», «prenotazioni», «certificati», «vendite», «spese», «incassi» o «assicurazione attiva» impliciti dalla check di una casella. I loro registri restano autoritativi nei rispettivi domini.
- Permettere che un progetto contenga solo note e date, senza opere, e che un'opera sia collegata a più progetti; le referenze UUID rimangono valide anche se il progetto viene chiuso.

## Impostazioni, privacy, offline e localizzazione

- **Impostazioni:** checklist iniziali facoltative per categorie di evento (mostra/fiera/open studio); colonna «scadenze» disattivabile; formato preferito data/ora secondo lingua del sistema. Nessun template forzato o integrazione remota.
- **Offline:** tutte le schede e i controlli di sovrapposizione sono elaborati localmente in SQLite, anche senza Internet. Niente inviti, sincronizzazione cloud, calendario di terzi o notifiche persistenti non autorizzate.
- **Multilingua:** lingua di sistema con override IT/EN/ES/FR/DE e fallback inglese, come [documentato](../../ECONOMY-I18N.md). Tradurre stati, messaggi, checklist iniziali e date; titoli/descrizioni inseriti dall'utente **non** vengono tradotti né sostituiti.
- **Privacy:** i nomi di organizzatori e recapiti facoltativi possono essere dati personali. Limitare il dato raccolto; l'export esterno non contiene di default indirizzi privati, nominativi di collezionisti, prezzo storico, margini, note interne o path delle foto. Nessun registro di «destinatario ha letto» senza infrastruttura e consenso (qui non previsti).
- **Backup:** se implementata in futuro, la pianificazione entra nel normale archivio locale e nella copia/ripristino verificati su cartella scelta; file stampati o salvati fuori dalla cartella app restano esterni e non possono essere garantiti dal backup.

## Import, export e interoperabilità

- **Esportazione operativa:** CSV di opere coinvolte e checklist con ID stabili, stati chiaramente distinti da disponibilità/vendita, date in ISO 8601, schede senza dati sensibili di default. Possibile stampa locale di un «foglio allestimento» solo dopo anteprima.
- **Round trip futuro:** JSON versionato contenente progetti, legami opera e attività, con idempotenza UUID, validazione periodi e precedenza a record già esistenti; coordinamento concettuale con [TECH-001](https://github.com/Cesare-ml/Repertica-Studio/pull/16), non una modifica di quel filone.
- **Import:** solo file scelto localmente, anteprima dei conflitti, controllo artwork UUID mancanti, scelta fra scartare, collegare o creare *progetto* in bozza; mai creare opere, contatti o vendite silenziosamente. Annullamento transazionale.
- **Nessun upload:** gli eventuali dossier [UX-007](REPS-UX-007-dossier-presentazione-opere.md) sono documenti distinti; il progetto può **riutilizzare** una selezione di opere se entrambe le funzionalità saranno approvate, ma non è dipendente da tale funzionalità.

## Dipendenze, complessità, rischi e alternative

- **Dipendenze:** IDs opere, ricerca/filtri, i18n, storage SQLite e backup. Non richiede UX-001 né UX-007, ma dovrebbe poter collegarsi a esse se mai approvate.
- **Complessità:** bassa-media per progetto/partecipanti/checklist; media per conflitti, date incerte e multiprogetto; medio-alta per esportazioni selettive, import relazionale e versioni storiche.
- **Rischi:** falsa disponibilità delle opere, confusione fra elenco esposizione e trasferimento reale, checklist interpretata come prova di assicurazione o documento legale, doppie fonti di date e ubicazioni, dati personali su esportazioni, progetto divenuto un gestionale eventi enorme.
- **Mitigazioni:** stati distinti, testi «da verificare», nessuna azione mutante su catalogo/vendite, limitare i campi obbligatori, anteprima privacy, no assegnazione operativa a utenti inesistenti.
- **Alternativa minima:** collezioni temporanee di opere con titolo, periodo e checklist libera, senza fasi o timeline complesse. **Alternativa esterna:** calendario e foglio di calcolo locali, con scarsa interoperabilità ma costo implementativo nullo.
- **Fuori ambito:** ticketing, pubblicazione web, spedizioni automatiche, contratti e assicurazioni digitali, newsletter, fatture, collaborazione multi-account.

## Criteri di accettazione ipotetici

1. Creare una mostra con quattro opere non cambia `artworks.status`, ubicazione, vendite o certificati; un'opera può apparire in due progetti diversi.
2. La vendita di una delle quattro opere aggiorna l'avviso **alla prossima consultazione**, senza modificare silenziosamente il progetto o inventare una consegna.
3. Checklist complete o in ritardo rimangono distinguibili senza rete, anche a programma riaperto; una attività può esistere senza opera collegata.
4. Cancellare una partecipazione solo se deliberato non elimina l'opera né le sue fotografie; progetti chiusi conservano l'elenco storico con distinzione tra dato live e snapshot.
5. Se UX-001 esiste in futuro, la registrazione di un movimento resta esplicita: l'evento «mostra iniziata» **non** sposta l'opera.
6. Anteprima export nasconde per default dati collezionisti, costi, margini e indirizzi privati; import ripetuto con gli stessi ID non duplica legami o attività.
7. Backup/ripristino conservano date, attività, stati e riferimenti; UX testata con tastiera, screen reader, Unicode e lingua OS/fallback inglese su Windows/macOS/Linux.

## Fasi esplorative, non autorizzate

- **A:** colloqui con artisti/gallerie, verifica che eventi ricorrenti giustifichino un modulo; semplice lista progetto + opere + checklist.
- **B:** calendario interno delle scadenze e avvisi di conflitto, checklist salvabili, stampa filtrata con tutela privacy.
- **C:** import/export versionato, collegamenti opzionali a movimentazioni/dossier, test multipiattaforma e grandi archivi.

## Dati certi, ipotesi e domande al proprietario

- **Certi dal repo:** archivio opere e collezionisti, posizione e stato commerciale, foto, date e certificati, backup locale, nessun modulo eventi e nessun account; le informazioni su movimento fisico e dossier sono solo **idee** nelle altre schede.
- **Ipotesi:** molti professionisti preparano mostre almeno alcune volte l'anno e vorrebbero l'elenco con checklist direttamente collegato alle opere; i template semplificano il lavoro.
- **Decisioni aperte:** è prioritario l'uso per artista singolo o piccola galleria? Serve rappresentare le opere «candidate» o basta una lista confermata? Le date incerte meritano supporto esplicito? Occorre un semplice elenco stampabile o un layout d'allestimento? Quanto spesso le mostre coinvolgono più sedi?

**Stato finale invariato: Proposta — da valutare. Nessuna approvazione o assegnazione allo sviluppo.**
