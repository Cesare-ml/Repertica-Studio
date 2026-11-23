# REPS-UX-058 — Distinta di imballaggio per trasferimenti temporanei di più opere

- **ID stabile:** REPS-UX-058
- **Data:** 2026-11-23
- **Stato:** Proposta — da valutare
- **Filone:** Gestione e UX
- **Priorità ipotetica:** P2 per studi/gallerie che preparano spedizioni collettive frequenti; P3 o nessuno sviluppo per archivi piccoli
- **Natura:** esclusivamente documentale. Nessuna approvazione del proprietario, richiesta di implementazione o assegnazione.

## 1. Problema reale da verificare, utenti e fonti di partenza

Un artista che porta otto opere a una fiera può distribuirle in tre casse, inserire più dipinti in uno stesso collo o preparare un imballaggio temporaneo prima del ritiro. Durante la preparazione o lo scarico deve poter rispondere a **quali opere erano previste in quale collo**, che cosa è stato effettivamente controllato e su quale documento si basava la lista. Una selezione di opere per una mostra non è una distinta d'imballaggio; una vendita non equivale a una spedizione. L'elenco scritto a mano resta spesso più semplice del gestionale: va verificato se i professionisti ripetono abbastanza spesso questa operazione da giustificare una funzione specifica.

**Destinatari:** artisti che partecipano a mostre e fiere, assistenti di studio, piccole gallerie che preparano ritiri multiopera, archivi e registrar che coordinano consegne a sedi temporanee. **Non** è prevista una gestione dei corrieri o del magazzino logistico.

**Riscontro statico del 23/11/2026 su `main`:** [README](../../../README.md), [MVP](../../MVP.md), [modelli Angular](../../../src/app/models.ts), [documentazione brochure](../../BROCHURES-OFFLINE.md) e [portabilità](../../DATA-PORTABILITY.md) descrivono opere con UUID/codice, fotografia, dimensioni, ubicazione testuale, stato commerciale, brochure e CSV. Il CSV delle opere è integrale, non un documento di imballaggio; nel modello consultato non emerge un'entità per **un gruppo di colli e il suo contenuto dichiarato**. Non si è verificato il comportamento nativo: la lacuna è un'ipotesi progettuale, non un bug accertato.

## 2. Confini e coordinamento anti-doppione

- [UX-001 — Provenienza, ubicazioni e movimentazioni](REPS-UX-001-provenienza-movimentazioni.md): fonte futura per **uscita, trasferimento, custodia e rientro effettivamente registrati**; qui si preparerebbe soltanto la **distinta materiale di preparazione**, senza cambiare luogo né creare movimenti. Un collo segnato «pronto» non è partito.
- [UX-009 — Mostre e checklist](REPS-UX-009-progetti-espositivi-checklist.md): quali opere partecipano a un progetto e relative attività; una stessa selezione può alimentare una distinta, ma la distinta fotografa un **lotto concreto di imballaggio**, indipendente dalla mostra.
- [UX-046 — Consegne post-vendita](REPS-UX-046-consegne-spedizioni-post-vendita-checklist.md): logistica di una **vendita a un acquirente**, con eventuale nozione di collo. Qui sono centrali **trasferimenti collettivi non commerciali** a mostre, fiere e depositi temporanei, anche senza vendite.
- [UX-022 — Componenti fisici](REPS-UX-022-opere-composte-componenti-fisici.md): rappresenta ciò che costituisce una singola opera; il collo contiene oggetti per il trasporto, non stabilisce la struttura dell'opera. Una scultura scomposta richiederebbe un collegamento futuro esplicito fra componenti e colli, **non** nuove opere fittizie.
- [UX-025 — Istruzioni di cura e allestimento](REPS-UX-025-istruzioni-allestimento-cura-opera.md): prescrizioni generali dell'opera, non dichiarazioni sull'imballaggio eseguito per un viaggio specifico.
- [UX-004 — Rapporti di condizione](REPS-UX-004-rapporti-stato-conservazione.md) e [UX-016 — Assicurazione](REPS-UX-016-valutazioni-coperture-assicurative-scadenze.md): controlli e coperture restano fonti separate; una casella spuntata non costituisce perizia o prova assicurativa.
- [TECH-016 — Allegati locali](https://github.com/Cesare-ml/Repertica-Studio/blob/docs/idee-strumenti-architettura-20261121/docs/idee/strumenti-e-architettura/REPS-TECH-016-allegati-documentali-locali-multi-entita.md) e [TECH-001 — Dati aperti](https://github.com/Cesare-ml/Repertica-Studio/blob/docs/idee-strumenti-architettura-20261121/docs/idee/strumenti-e-architettura/REPS-TECH-001-pacchetto-dati-aperto.md): eventuali infrastrutture riusabili, senza modificare né presupporre approvate le proposte tecniche.

**Fuori ambito:** acquistare etichette dai corrieri, tracking remoto, booking spedizione, prenotazione opere, geolocalizzazione, inventario fisico certificato, documenti fiscali/di trasporto conformi per legge, firme probatorie, assicurazione, invio email e pubblicazione web.

## 3. Scenari d'uso e flussi UX ipotetici

1. **Fiera, otto opere in tre casse.** Dalla lista opere si selezionano otto UUID e si crea una bozza «Fiera — trasporto di andata». Si nominano Cassa A/B/C, si assegnano opere e si controlla il riepilogo «8 previste, 7 assegnate, 1 senza collo». L'utente decide se risolvere prima di stampare; nessuna assegnazione automatica.
2. **Ritiro da terzi.** Si produce una distinta con codice, titolo breve e numero di collo. Al momento dell'affidamento si annota «controllato da me», data e nota; la controparte potrebbe non aver confermato la ricezione. Nel resoconto compare «verifica dichiarata dall'operatore», non «consegna provata».
3. **Rientro parziale.** Sei opere rientrano e due rimangono in esposizione. Si crea, se necessario, un *nuovo lotto di ritorno* con selezione diversa e legame informativo al lotto di andata. Non si marca automaticamente alcun rientro, né si deduce un'ubicazione.
4. **Rettifica di un imballaggio.** La Cassa B contiene un'opera diversa rispetto alla prima distinta salvata. La bozza si modifica; dopo emissione/stampa l'eventuale nuova revisione è datata e identificabile, così le due copie non risultano indistinguibili.

**Percorso minimo:** `Opere → Seleziona → Prepara distinta → Crea/nomina colli → Assegna opere → Controllo delle omissioni → Anteprima condivisibile → Salva PDF o stampa`. Pannello facoltativo di controllo a due stati «previsto / verificato manualmente» con nota e data. Il salvataggio della bozza non modifica `artworks.status`, `artworks.location`, vendite, certificati, movimenti o spese.

**Microinterazioni/accessibilità:** assegnazione per menu e pulsanti raggiungibili da tastiera (nessun drag-and-drop obbligatorio); anteprima tabellare con numero collo e codice opera leggibili anche senza colori; indicatori in testo per opere non assegnate, ripetute o non più disponibili. Focus sul primo errore, nomi accessibili a screen reader, zoom e stampa ad alto contrasto. Non nascondere warning lunghi in tooltip.

## 4. Dati necessari e regole di coerenza (solo descrittive)

**Ipotesi minima:** `packing_batch` (UUID, titolo locale, finalità descrittiva, riferimento facoltativo a mostra, data prevista con precisione, destinazione *non obbligatoria*, fase `draft|issued|superseded`, revisione, `created_at`). `packing_container` (UUID, batch UUID, identificativo leggibile univoco **nel lotto**, ordine, descrizione facoltativa, dimensioni/peso *dichiarati* e unità, note). `packing_assignment` (batch UUID, container UUID o «non assegnato», artwork UUID, ordine, note facoltative). Non inventare UUID di componenti o numeri seriali delle opere.

**Storicizzazione:** l'emissione salva uno snapshot minimo di codici/titoli e associazioni al momento della stampa, con data/revisione; la scheda opera corrente resta autorevole per i dati correnti. Eventuali correzioni successive producono revisione esplicita, non riscrivono PDF precedentemente consegnati. La distinta non dichiara un inventario fisico verificato salvo annotazione umana.

**Controlli:** un'opera assegnata a due colli nello *stesso lotto* presenta conflitto da risolvere (tranne futura gestione esplicita dei componenti); uno stesso codice in lotti differenti è ammesso e non prova doppia presenza fisica. Lotti separati possono riferirsi a periodi sovrapposti: segnalare, non bloccare, perché le date sono dichiarate e i trasferimenti reali risiedono altrove. Un'opera eliminata o rinominata dopo emissione non deve cambiare lo snapshot storico.

**Ipotesi descrittive di operazioni, non API autorizzate:** `preview_packing_batch`, `issue_packing_list`, `export_packing_list`, `compare_packing_revisions`. Nessuna funzione del server MCP, IPC, schema SQLite o codice è modificata.

## 5. Impostazioni, offline e localizzazione

- **Impostazioni:** prefisso locale facoltativo delle casse (A/B oppure 01/02), colonne mostrate in distinta, formato pagina A4/Letter, esclusione predefinita di indirizzi e valori economici. Niente preferenze obbligatorie, account o servizi di logistica.
- **Offline-first:** metadati di bozza/revisione, se mai sviluppati, sarebbero conservati in SQLite locale; foto/allegati eventualmente scelti nel media store. Calcoli e stampa locali, senza invio a organizzatori o vettori. Con l'app chiusa nessuna notifica né sincronizzazione promessa.
- **Multilingua:** rilevamento della lingua di sistema e fallback inglese, con override IT/EN/ES/FR/DE secondo [Economy & i18n](../../ECONOMY-I18N.md). Codici `draft/issued`, identificativi e unità stabili; etichette, avvisi e date tradotti/localizzati. Titoli d'opera e note libere restano nella lingua originale, mai tradotti automaticamente.
- **Privacy:** default per PDF condivisibile: codice, titolo, numero collo e quantità; indirizzo privato di ritiro, nominativi, recapiti, prezzi, margini, descrizioni riservate e percorso interno dei file **esclusi salvo scelta esplicita**. Anteprima completa e conferma prima dell'export. PDF all'esterno del database non incluso nei backup per il solo fatto di essere stato esportato; i backup correnti in chiaro meritano cartelle protette.

## 6. Import/export, compatibilità e alternative

- **Export minimo:** stampa/PDF locale e CSV UTF-8 della distinta selezionata, con `batch_id`, `revision`, `container_code`, `artwork_id`, `artwork_code_snapshot`, `artwork_title_snapshot`, stato di verifica *dichiarato*, date ISO, senza dati personali di default. Un PDF non è un DDT o un contratto.
- **Portabilità completa eventuale:** JSON versionato con lotti/colli/assegnazioni e riferimenti UUID, più eventuali allegati selezionati e manifest; coordinamento con TECH-001, non duplicazione del backup.
- **Import futuro solo se necessario:** file locale scelto, anteprima delle opere mancanti e dei conflitti/revisioni, mapping manuale e operazione annullabile; mai creare opere, movimentazioni, vendite o dati di contatto automaticamente. In fase A può non esistere import.
- **Compatibilità:** schema soltanto additivo nel caso di futura approvazione; archivi vecchi restano consultabili, nessuna distinta inventata retroattivamente dal campo `location`. Backup/ripristino su qualunque cartella scelta includerebbero record e allegati *gestiti dall'app*, non PDF esterni. Windows/macOS/Linux e percorsi Unicode.
- **Alternativa senza sviluppo:** esportare le opere già disponibili in CSV, costruire in LibreOffice Calc una colonna «collo» e stampare il foglio; testare prima questa procedura con 3–5 invii reali. Altra opzione: introdurre un semplice modello Markdown/CSV documentato, senza modulo UI.

## 7. Rischi, complessità e dipendenze

**Complessità ipotetica:** bassa-media per bozza, selezione opere e PDF tabellare; media per versioni, controlli dichiarativi, collegamenti a progetti e import; alta se si introducono componenti fisici ripartiti, logistica multi-vettore o scambi fra archivi (da **escludere** finché non validati).

**Dipendenze eventuali:** UUID e codici opera, ricerca locale, snapshot, sistema di stampa/export, i18n, backup/restore, sicurezza dei file locali. UX-001/009/022/046 restano indipendenti e non sono prerequisiti. Le proposte TECH-001/016 sono coordinate concettualmente, non modificate.

**Rischi principali:** confondere «previsto» con «imballato/arrivato», usare la distinta come ricevuta legale, identificare un oggetto con il solo titolo, duplicare le informazioni dei movimenti, elenchi non aggiornati, pubblicazione di prezzi o indirizzi in PDF, difficoltà con opere composte. Mitigazioni: UUID/codice, revisioni datate, stato verificato solo manualmente, separazione dai registri autorevoli, scelta privacy per campo, modalità minima senza componenti.

## 8. Criteri di accettazione proposti per un'eventuale futura valutazione

1. Preparare otto opere in tre casse segnala esattamente un'assegnazione mancante e nessuna «spedizione avvenuta».
2. La distinta emessa mantiene il titolo/codice fotografato nello snapshot anche dopo una modifica della scheda opera; una rettifica genera revisione distinguibile.
3. Due copie della stessa opera nello stesso lotto sono segnalate, senza inventare che esistano due esemplari; un lotto successivo resta indipendente.
4. La stampa non espone per default collezionisti, ubicazioni private, costi, prezzi né nomi di file locali. Un export annullato non modifica l'archivio.
5. La spunta «verificato manualmente» non crea movimentazioni, vendite, incassi, spese, certificati, rapporti di condizione o garanzie.
6. CSV/JSON di prova conservano codici, UUID, numeri di colli, revisioni e date; reimport futuro non duplica gli identificativi e segnala opere mancanti.
7. Backup/ripristino su cartella scelta conservano dati gestiti e legami; PDF salvati fuori dall'app sono dichiarati esterni.
8. Tutto è utilizzabile senza rete con sola tastiera e screen reader, date nella lingua del sistema e fallback inglese; test su Windows/macOS/Linux con caratteri Unicode.

## 9. Fasi e domande per il proprietario

- **Fase di ricerca (senza sviluppo):** raccogliere 3–5 distinte reali anonimizzate, frequenza, numero di colli e gestione delle opere composte; provare CSV + Calc. Se il foglio basta, **non implementare**.
- **Fase A eventuale:** selezione opere, colli, assegnazioni, anteprima, stampa/PDF senza spedizioni o verifiche.
- **Fase B eventuale:** revisioni/snapshot, verifica manuale, CSV leggibile, filtri e collegamento informativo a progetti.
- **Fase C solo su domanda verificata:** JSON portabile e import assistito; integrazione consultiva con movimenti e componenti.

**Requisiti certi:** app desktop open source gratuita, archivio locale SQLite senza account, backup liberamente posizionabili, esportabilità, lingua del sistema/fallback inglese; esistono opere con codici/UUID e CSV, non esiste nelle fonti consultate un registro dei colli. **Ipotesi:** la gestione dei colli è frequente per gli utenti attuali, serve una cronologia revisionale, occorre collegamento con mostre o movimenti. **Questioni aperte al proprietario:** le distinte di fiere e mostre sono un'esigenza prioritaria reale? Basta un CSV con caselle di controllo o serve una stampa PDF? I casi con opere composte giustificano eccezioni già nella prima versione? Conviene evitare qualsiasi archivio storico di distinte?

**Decisione attuale:** **Proposta — da valutare**. Nessuna implementazione, approvazione, ticket o incarico.
