# REPS-UX-005 — Edizioni numerate, multipli e varianti di opere

- **Data di proposta:** 2026-11-21
- **Stato:** Proposta — da valutare
- **Filone:** Gestione e UX
- **Priorità ipotetica:** P2 per artisti di stampe, fotografia e scultura in tiratura; da rivedere con il proprietario.
- **Perimetro:** solo analisi e documentazione. Nessuna creazione di codice, migrazione, catalogo o modifica ai certificati autorizzata.

## Problema professionale e confini

Il [MVP](../../MVP.md) cataloga opere con codice univoco, dimensioni, tecnica, stato e foto; registra **una sola vendita per ciascun record opera**, con certificato e snapshot immutabile. Questo è adeguato per un originale unico, ma un fotografo, incisore o scultore può avere una **edizione di 25 esemplari distinti**, alcuni disponibili, altri riservati, venduti o destinati a prova d'artista. Trattare l'edizione come un'unica opera impedisce vendite distinte; clonare 25 schede senza una relazione produce codice/tiratura incoerenti.

**Nuovo dominio proposto:** collegamento fra un **progetto/edizione** e i suoi **esemplari identificabili**. Non si reinventa il catalogo opere esistente, né la cronologia delle ubicazioni di [UX-001](REPS-UX-001-provenienza-movimentazioni.md), le trattative di [UX-002](REPS-UX-002-trattative-prenotazioni.md), i costi attribuiti di [UX-003](REPS-UX-003-costo-e-margine-per-opera.md) o il formato di esportazione [TECH-001](https://github.com/Cesare-ml/Repertica-Studio/blob/docs/idee-strumenti-architettura-20261121/docs/idee/strumenti-e-architettura/REPS-TECH-001-pacchetto-dati-aperto.md). **Non** certifica automaticamente che la tiratura dichiarata sia effettivamente esaurita o autentica.

## Utenti e scenari

1. **Fotografia in tiratura limitata:** definizione «Edizione 2026, 25 esemplari», generazione *esplicita* dei record 1/25…25/25 con codici unici; il numero 3/25 può essere venduto a un collezionista e il numero 4/25 rimane disponibile.
2. **Scultura fusa:** esemplari con finiture, misure o fotografie individuali diverse, ma collegati a uno stesso progetto, distinguendo caratteristiche comuni e peculiarità dell'esemplare.
3. **Prove d'artista e fuori commercio:** gestione di categorie come AP/EA, HC, prototipi o tirature non numerate, mantenendo inventari separati e regole di numerazione dichiarate dall'artista, non dedotte dal software.
4. **Riedizione o variante:** una nuova stampa su supporto diverso è una **nuova edizione** o versione esplicitamente collegata al progetto; non modifica il totale e le condizioni dell'edizione precedentemente documentata.
5. **Catalogo storico:** import di record singoli già venduti con numerazione descritta in note, con procedura assistita di *riconciliazione*, senza riscrittura delle vendite o dei certificati già emessi.

## Flussi UX suggeriti

- **Opera → Crea edizione:** scegliere prima «opera unica» oppure «progetto con esemplari»; form del progetto con titolo, tecnica, supporto, anno, dimensioni comuni, immagine rappresentativa, tiratura dichiarata opzionale e note. La dicitura «tiratura limitata» va mostrata solo se configurata esplicitamente.
- **Scheda edizione:** riepilogo «esemplari registrati / tiratura dichiarata / disponibili / riservati / venduti / fuori commercio» con filtri e lista di **codice e numero del singolo esemplare**. Distingue sempre *creati nel gestionale* da *materialmente prodotti*: una scheda programmata non equivale a oggetto realizzato.
- **Genera esemplari (solo conferma esplicita):** anteprima dei numeri e codici, controllo duplicati su tutto l'archivio, scelta fra creare schede previste oppure effettivamente prodotte, conferma quantità e possibilità di operare anche un record alla volta. Nessuna vendita o certificato creati automaticamente.
- **Scheda esemplare:** conserva singolo stato commerciale, ubicazione, immagini, note, prezzo scelto, identificatore fisico e futuro collegamento a movimentazioni; pulsanti di vendita/certificato dell'MVP operano **solo sull'esemplare**, non sull'edizione intera.
- **Varianti:** scegliere se una caratteristica è *comune all'edizione* o *specifica di un esemplare*; le modifiche agli attributi comuni dopo vendite o emissioni di certificati richiedono riepilogo impatto e non alterano snapshot storici.
- **Accessibilità:** vista tabellare e vista gruppo, intestazioni comprensibili, navigazione tastiera e screen reader; non basarsi solo sui colori di disponibilità.

## Ipotesi di modello dati e vincoli

- `edition`: UUID stabile, titolo/progetto, gruppo e variante opzionali, `declared_size` (intero positivo **opzionale**), `numbering_scheme` dichiarato, data di creazione, caratteristiche condivise e note. Nessuna deduzione automatica di autenticità o dei diritti di riproduzione.
- `edition_instance`: UUID, `edition_id`, `artwork_id` **univoco** che punta a un record opera individuale, categoria (`standard`, `artist_proof`, `hors_commerce`, `prototype`, `unnumbered`), numero intero opzionale, denominatore dichiarato/snapshot, fase produttiva (`planned`, `made`) e note. Unicità vincolata per `(edition_id, category, edition_number)` quando il numero è presente; le categorie hanno contatori e politiche esplicite.
- `artworks.code` resta globalmente univoco; per esempio codice **ST-2026-003**, edizione **3/25** come dato distinto. Mai usare solo «3/25» come identità globale.
- **Vendita e certificato:** restano legati al singolo `artwork_id` dell'esemplare; ogni esemplare massimo una vendita; numeri di certificato univoci secondo l'MVP. Al momento dell'emissione un eventuale snapshot di categoria/numero/denominatore deve essere **congelato**.
- **Invarianti:** tiratura dichiarata, quantità pianificata, quantità effettivamente prodotta e quantità venduta sono grandezze **diverse**. Impedire una numerazione duplicata; non consentire riduzione del limite sotto i numeri già assegnati, né cancellazione/riassegnazione silenziosa di esemplari venduti o certificati; prevedere una **rettifica documentata** della dichiarazione quando necessario. Vietato cancellare un'edizione che possiede esemplari referenziati da vendite.
- **Certezze retroattive:** un archivio preesistente contiene **opere singole**, non «edizioni da dedurre»: nessuna migrazione che raggruppi automaticamente per titolo, anno o immagine, nessuna inferenza dal testo di note.
- **Prezzo:** il suggerimento con coefficiente artista esistente rimane indicativo. È ipotizzabile un prezzo base per edizione e override per esemplare, ma nessun prezzo è ricalcolato retroattivamente su vendite registrate.

## Impostazioni, localizzazione, offline e import/export

- **Impostazioni:** prefisso codici suggerito, convenzione di visualizzazione dei numeri, categorie di prova attivabili e separazione «previsti / prodotti». Predefiniti semplici; mai forzare la creazione di una tiratura per opere uniche.
- **Offline:** esclusivamente SQLite e media locali, senza marketplace, stampa remota, licenze cloud, account, tracking, richieste di rete o sincronizzazione. Eventuale supporto a barcode/etichette è una proposta separata, non presupposta.
- **Multilingua:** usare lingua di sistema con fallback inglese (IT/EN/ES/FR/DE). Tradurre «edizione», «prova d'artista», «esemplare», «fuori commercio», «pezzo programmato» senza tradurre i codici di categoria persistiti; note e titoli rimangono come inseriti.
- **Esportazione:** CSV per elenco delle edizioni ed esemplari, contenente identificativi stabili, categoria, numerazione, quantità e venduto/disponibile; JSON con schema versionato e relazioni `edition → instance → artwork → sale/certificate` per round trip; immagini relative nel pacchetto semantico **solo se la proposta TECH-001 verrà adottata**. Anteprima, mappatura e deduplica per UUID e codici, rollback; esportazioni per terzi devono escludere di default nomi, indirizzi e prezzi privati.
- **Backup/compatibilità:** backup attuale deve continuare a funzionare per archivi senza edizioni e includere le nuove relazioni solo dopo eventuale approvazione e collaudo; nessuna riscrittura di backup o certificati storici. In caso di file di un'app nuova su versione precedente, rispettare il blocco schema più recente; non fingere che sia pienamente retrocompatibile.

## Complessità, rischi e alternative

- **Complessità:** media-alta per relazioni e filtri; alta per retrocompatibilità dei certificati, correzioni di tiratura, duplicati importati e gestione più di un'edizione di una stessa composizione.
- **Dipendenze:** `artworks`, `sales`, `certificates`, backup e UUID; eventuale galleria fotografica della [PR #17](https://github.com/Cesare-ml/Repertica-Studio/pull/17) è un miglioramento indipendente, **non** obbligatorio.
- **Rischi:** conteggio esemplari inesatto; numero duplicato o non assegnato; cancellazioni di pezzi venduti; false attestazioni di esclusività, autenticità o diritti; copie prodotte ma non registrate. Chiarire «dichiarato dall'utente», mantenere audit/cronologia per correzioni, blocchi di unicità e avvisi sulle discrepanze.
- **Rischio economico:** sommare il valore nominale dell'intera tiratura ai ricavi effettivi o conteggiare più volte il costo comune di produzione. Le vendite esistenti restano la **sola** fonte dei ricavi registrati; costi condivisi, se necessari, sono da coordinare con [UX-003](REPS-UX-003-costo-e-margine-per-opera.md) e **non** duplicare nelle uscite.
- **Alternativa piccola:** aggiungere a ogni singola opera i campi «ID serie / numero / totale» con filtri, senza entità progetto e senza creazione in massa. È più semplice e adatta a pochi multipli, ma meno affidabile per tirature estese e varianti.

## Criteri di accettazione ipotetici

1. In un'edizione 25 pezzi, 3/25 venduto non modifica disponibilità e certificati di 4/25. La vista totale indica **1 venduto**, non «25 venduti».
2. Non sono possibili due esemplari numerati **3/25** nella stessa edizione/categoria, anche con import duplicati; il sistema impedisce anche collisioni del codice opera globale.
3. Una prova d'artista **AP 1/3** non riduce implicitamente i 25 numeri standard, e le due denominazioni restano separate.
4. La modifica successiva della tiratura o del numero non altera vendite e snapshot certificati già emessi; una rettifica è tracciabile e segnalata.
5. Opere uniche e archivi pregressi non vengono forzati in un gruppo; import testuale non deduce appartenenza da titolo uguale.
6. Una selezione di esemplari può essere esportata/importata localmente conservando UUID, riferimenti, numeri e stato, senza duplicare vendite, e senza dati cliente nelle viste pubbliche salvo autorizzazione.
7. I flussi sono utilizzabili senza rete, tastiera e screen reader, con locale OS/fallback inglese e importi in centesimi se si mostra un prezzo.

## Fasi progettuali eventuali, non approvate

- **A:** associazione manuale fra edizione e esemplari già catalogati, numerazione e filtro, nessuna generazione massiva.
- **B:** anteprima creazione esemplari, conteggi produzione/disponibilità, validazioni e rettifiche tracciate.
- **C:** varianti multiple e confronto fra edizioni, export/import semantico e coordinamento con eventuali movimentazioni/costi.

## Requisiti certi, ipotesi, questioni aperte

- **Certi:** un record opera = una vendita al massimo nell'MVP; codici unici, certificati immutabili, architettura locale e lingua OS/fallback inglese.
- **Ipotesi:** necessità di rappresentare edizioni e multipli con identità fisiche separate; convenzioni di tiratura sufficientemente frequenti tra utenti del progetto.
- **Da decidere:** Repertica deve servire soprattutto originali unici o anche stampa/fotografia/scultura in tiratura? Si devono creare record **solo dopo produzione**, oppure anche pianificati? Come trattare AP/HC e varianti con numeri uguali? Una tiratura può essere aumentata dopo la prima vendita con rettifica o deve essere bloccata? È necessaria una genealogia edizione/progetto o bastano campi comuni?

**Questa è una proposta da valutare, non un'autorizzazione allo sviluppo.**
