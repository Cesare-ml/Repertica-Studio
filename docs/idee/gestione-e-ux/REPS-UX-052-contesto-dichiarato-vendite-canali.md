# REPS-UX-052 — Contesto dichiarato delle vendite: luogo/modalità di conclusione e origine della richiesta

- **ID stabile:** REPS-UX-052
- **Data:** 2026-11-22
- **Stato:** Proposta — da valutare
- **Filone:** Gestione e UX; **solo analisi e documentazione**
- **Priorità ipotetica:** P2 se professionisti confermano il bisogno; P3 o nessuno sviluppo se note e CSV bastano
- **Decisione:** nessuna approvazione, assegnazione, implementazione, issue o PR di sviluppo.

## 1. Problema, professionisti e riscontri verificati

Un artista partecipa a una fiera, espone opere presso una galleria, riceve richieste dal sito o vende direttamente in studio. Dopo alcuni mesi vorrebbe distinguere **dove/con quale modalità è stata conclusa ciascuna vendita** da **dove è nata la prima richiesta**: sono fatti diversi. Può servire a decidere quali attività ripetere, ma un conteggio scorretto può fare attribuire vendite a una fiera solo perché avvenute nello stesso mese, oppure presentare come efficacia del social una vendita conclusa dopo più contatti.

**Destinatari possibili:** artisti con più canali di vendita, piccole gallerie, studi che partecipano a fiere/mostre. Un archivio non commerciale o un artista con poche vendite probabilmente non ne ha bisogno. **Nessun bisogno è stato verificato con interviste o test d'uso**.

**Evidenze consultate su main (22/11/2026):**
- [README](../../../README.md), [MVP](../../MVP.md), [ECONOMY-I18N](../../ECONOMY-I18N.md), [DATA-PORTABILITY](../../DATA-PORTABILITY.md).
- [Modelli Rust](../../../src-tauri/src/models.rs): la vendita contiene ID opera/collezionista, importo, valuta, data e note, senza campo tipizzato per modalità o provenienza della richiesta.
- [Comandi vendite](../../../src-tauri/src/commands.rs): la creazione registra la vendita e le quote storicizzate, non una fonte commerciale.
- [Export vendite](../../../src-tauri/src/export.rs): il CSV esporta importi, riferimenti, imposte/spedizione e note, ma non una modalità di conclusione.
- Esistono già statistiche mensili, incassi dichiarati distinti dal venduto e ripartizioni percentuali. Le note libere e il CSV sono un'alternativa effettiva da verificare prima di introdurre campi o schermate.

**Contesto di settore (non prova della domanda per Repertica):** [Artlogic — Sales & CRM](https://artlogic.net/products/gallery/sales) distingue richieste originate da sito/app/fiera e percorso della trattativa; [Artwork Archive — Artwork Insights](https://www.artworkarchive.com/artists/artwork-insights) offre analisi di inventario e vendite. Sono riferimenti per osservare problemi reali, **non** un invito a replicare CRM, cloud, profilazione o abbonamenti.

## 2. Confini anti-doppione (51 schede UX e 47 TECH esaminate)

- **[UX-034](REPS-UX-034-analisi-vendite-caratteristiche-opere.md):** aggrega vendite secondo **tecnica/supporto/anno dell'opera**, non il **contesto dichiarato della singola vendita**. Se entrambe fossero adottate, riutilizzare uno stesso motore di aggregazione, senza duplicare prospetti.
- **[UX-002](REPS-UX-002-trattative-prenotazioni.md)** e **[UX-038](REPS-UX-038-interessi-dichiarati-collezionisti-richieste-opere.md):** seguono rispettivamente negoziazioni e richieste, non attribuiscono la vendita a un canale. Un eventuale collegamento futuro deve essere **manuale**, non generare una nuova trattativa.
- **[UX-009](REPS-UX-009-progetti-espositivi-checklist.md)** e **[UX-047](REPS-UX-047-budget-consuntivo-mostre-fiere-spese-collegate.md):** progetto e costi di una mostra/fiera. Una vendita **non** diventa automaticamente vendita «della mostra» per coincidenza di data/ubicazione. Solo eventuale collegamento esplicito al medesimo ID evento, senza duplicare progetti o spese.
- **[UX-012](REPS-UX-012-affidamenti-conto-vendita-rendiconti.md):** contratti, commissioni e rendiconti a intermediari; qui una modalità «galleria» è solo etichetta descrittiva e **non prova un accordo o una commissione**.
- **[UX-014](REPS-UX-014-rubrica-relazioni-professionali.md)** e **[UX-035](REPS-UX-035-preferenze-contatto-comunicazioni-individuali.md):** identità/contatti e preferenze di comunicazione. Nessuna profilazione o consenso può essere inferito dal contesto di una vendita.
- **[UX-017](REPS-UX-017-vendita-multiopera-ripartizione-importi.md)** e **[UX-013](REPS-UX-013-resi-annullamenti-rettifiche-vendite.md):** eventuali gruppi di vendita o resi restano registri autorevoli, da non creare/alterare.
- **[TECH-008](https://github.com/Cesare-ml/Repertica-Studio/blob/docs/idee-strumenti-architettura-20261121/docs/idee/strumenti-e-architettura/REPS-TECH-008-viste-salvate-filtri-ricerca-locale.md)** riguarda filtri, **[TECH-001](https://github.com/Cesare-ml/Repertica-Studio/blob/docs/idee-strumenti-architettura-20261121/docs/idee/strumenti-e-architettura/REPS-TECH-001-pacchetto-dati-aperto.md)** interoperabilità, **[TECH-025](https://github.com/Cesare-ml/Repertica-Studio/blob/docs/idee-strumenti-architettura-20261121/docs/idee/strumenti-e-architettura/REPS-TECH-025-esportazioni-tabellari-minimizzate-condivisione.md)** minimizzazione: questa scheda propone solo **semantica, decisioni e UX**, senza modificarne contenuti o introdurre infrastruttura parallela.

**Esclusioni:** tracciamento web, pixel, geolocalizzazione, riconoscimento automatico delle email, misurazione di conversioni o ritorni pubblicitari, attribuzione causale, invio di dati alle piattaforme, CRM obbligatorio, registri contabili paralleli, inventario eventi, fatturazione, pagamenti online o classificazioni IA.

## 3. Casi d'uso che evitano inferenze scorrette

1. **Fiera, vendita successiva in studio.** Una persona vede un'opera a una fiera, scrive dopo due settimane e conclude in studio. La **modalità di conclusione** è «studio»; l'**origine dichiarata del primo contatto** può essere «fiera» solo se l'artista ne ha evidenza. Se il dato manca, resta «non indicato». Non attribuire ricavi automaticamente alla fiera.
2. **Vendita tramite galleria.** L'artista registra «intermediario/galleria» come modalità dichiarata e, se utile, una nota sul referente. Non si genera un conto vendita, una percentuale né un incasso a partire dall'etichetta. Questi restano nei registri pertinenti.
3. **Vendita direttamente da contatto online.** L'acquirente conclude privatamente dopo una richiesta dal sito personale; modalità «diretta a distanza» oppure «altro» e origine «sito» dichiarata. Non servono login, dati del sito, analytics o numeri di visualizzazioni.
4. **Storico non qualificabile.** Per cento vendite passate senza origine certa, mostrare «non registrato»; **non** ricalcolare in blocco l'origine dalle note, da una località, dal nome del cliente, dall'anno o dalla data.
5. **Confronto descrittivo.** In un periodo ci sono tre vendite di importo lordo 400 €, 600 €, 1.000 €: «studio: 2 / 1.000 €; fiera: 1 / 1.000 €». È **venduto registrato**, non incassato, profitto o rendimento della fiera; non dedurre che un canale abbia prestazioni superiori senza denominatori (opere esposte, costi, durata, selezione).
6. **Vendita associata a più opere, se UX-017 fosse adottata.** L'utente decide se assegnare un unico contesto all'accordo o correggere singole righe; nessuna duplicazione del totale dell'ordine nei rapporti. La regola va decisa **prima** dello sviluppo.

## 4. Flussi UX candidati, dal minimo al possibile approfondimento

**Fase minima, registrazione ordinaria invariata:** in «Nuova vendita» mantenere i campi attuali e un controllo opzionale e non intrusivo «Contesto della vendita» che, solo se richiesto, consenta di registrare la modalità **di conclusione**. Valori suggeriti e modificabili senza imporre una tassonomia universale: *studio/diretto*, *mostra/fiera in presenza*, *tramite galleria/intermediario*, *diretta a distanza*, *altro*, *non indicato*. Se la persona vuole solo un testo libero, non costringerla a scegliere. Il codice del valore è stabile e tradotto; la località e il nome della galleria non sono obbligatori.

**Origine del primo contatto (solo eventuale fase successiva):** campo separato e chiaramente intitolato «Come è arrivata la richiesta, secondo quanto sai», con *non so* esplicito, provenienza annotata manualmente, data/fonte facoltativa. Non confondere con il canale dove si è concluso l'accordo. Un'opera vista su più canali non si può attribuire causalmente a uno solo: permettere «non determinabile» senza penalità.

**Consultazione:** nella scheda vendita e nel registro, mostrare modalità dichiarata e, solo se compilata, origine del contatto. Eventuale filtro in «Vendite» con conteggio e somma lorda su intervallo **data di vendita**; conservare una riga «non indicato». I totali devono tornare con l'elenco delle vendite incluse, esplicitando se il filtro esclude record. L'eventuale ripartizione sul **netto commerciale** deve riusare i dati esistenti e non essere descritta come utile fiscale.

**Revisione/correzione:** la modifica del contesto non riscrive importo, quota, incasso, saldo o certificato. Se si aggiorna una classificazione storica, la UI chiede conferma e può conservare una nota «classificazione corretta il…» senza modificare una vendita pregressa. Non attribuire una fonte «verificata» se è soltanto ricordata dall'utente.

**Accessibilità:** comando tastiera e screen reader, focus evidente, valori accompagnati da testo anche senza icone/colori, etichette chiare per «non registrato» e «non determinabile», funzionamento in finestre ridotte e zoom 200%; eventuale tabella prima dei grafici, con intestazioni e totali leggibili.

## 5. Dati e interfacce descrittive, non schema approvato

- **Alternativa zero:** compilare il campo **note di vendita già disponibile**, esportare CSV e classificare offline in LibreOffice. È la prima opzione da mettere alla prova e potrebbe evitare sviluppo.
- **Alternativa leggera futura:** campo **opzionale** della vendita oppure tabella satellite 'sale_context' riferita al UUID stabile di 'sales', con 'closing_channel_code' nullable, 'closing_channel_label' facoltativa e nota privata breve; dati di creazione/rettifica. Il valore assente è **«non registrato»**, distinto dalla scelta consapevole «origine non determinabile».
- **Solo se giustificato:** record distinto 'sale_enquiry_origin' con 'sale_id', 'source_kind', 'declared_at' e 'source_basis' (es. comunicazione riferita/annotazione di studio). Non importare log web, messaggi o profili social. La provenienza può essere incerta, multipla o sconosciuta; evitarne una gerarchia artificiale.
- **Se UX-009 fosse esistente:** un UUID evento facoltativo referenziato **esplicitamente**, non creato qui. L'etichetta «fiera» non aggiunge in automatico costi o vendite alla UX-047; qualunque report congiunto necessita riconciliazione deliberata.
- **Interfaccia descrittiva:** 'get_sales_by_declared_context(period, dimension)' restituisce periodo, dimensione, valuta, conteggio, somma lorda in centesimi, numero non classificato e gruppi. **Non è un'API da implementare**, né autorizza endpoint remoti. Se il futuro UX-027 introduce valute diverse, non sommare denaro di valute differenti senza tassi e regole esplicitamente scelti.
- **Vincoli:** una sola fonte autorevole per UUID vendita; nessuna modifica a 'sale_finances', ai conteggi della tesoreria, alle ricevute o ai PDF immutabili; i gruppi devono essere disgiunti per la metrica selezionata e l'importo cumulato deve coincidere con le vendite filtrate.

## 6. Offline, impostazioni, lingue, privacy e portabilità

- **Offline-first:** SQLite locale su Windows/macOS/Linux, senza account Repertica, abbonamento, server, tracking, sincronizzazione o rete. Eventuali integrazioni con Repertica Sites/posta/aste rimangono **escluse**, non presupposte.
- **Impostazioni:** campo nascosto o disattivabile per utenti senza esigenza; elenco piccolo di categorie localizzate, «altro» libero, nessun wizard obbligatorio. Non memorizzare canale predefinito che classifiche involontariamente tutte le nuove vendite. Origine del primo contatto sempre facoltativa.
- **Multilingua:** lingua di sistema IT/EN/ES/FR/DE, override locale e fallback inglese come [ECONOMY-I18N](../../ECONOMY-I18N.md). Codici stabili indipendenti dalle traduzioni. Nomi propri di eventi, canali personalizzati, note e citazioni restano come scritti; formattazione numerica locale a schermo, valori in centesimi negli export.
- **Privacy:** l'origine della richiesta può riferirsi a una persona identificabile e rivelare abitudini di contatto. Mai esportare automaticamente nome, recapiti, note o origine su documenti destinati ai collezionisti. I rapporti condivisibili mostrano **per default soltanto aggregati**, preferibilmente senza celle identificabili quando il gruppo conta una sola vendita; l'utente vede anteprima e decide i dettagli da includere. Nessun punteggio o profilazione degli acquirenti; la categorizzazione non vale come consenso marketing. Backup correnti **in chiaro**: avvertire quando salvati in percorsi condivisi.
- **Backup:** ogni dato eventualmente approvato entra nel backup completo verificato SQLite + media dentro una cartella liberamente scelta dall'utente. Non interrompere i backup, non imporre cloud e non modificare snapshot storici.
- **Export:** mantenere gli export CSV correnti e la loro semantica; se il campo sarà introdotto, documentare la versione del formato o un registro supplementare con UUID vendita, codice contesto, etichetta e fonte, UTF-8 e protezione dalle formule. Copie aggregate e copie complete sono distinte. **Import** solo eventuale, con anteprima, controllo UUID/valori, gestione dei conflitti e rollback: un import di contesto non può creare, annullare o modificare vendite/incassi.
- **Compatibilità:** eventuale migrazione SQLite additiva e tollerante ai record storici vuoti; nessun riempimento retroattivo. I software più vecchi non devono promettere lettura di uno schema futuro; verificare compatibilità backup/ripristino e aggiornamenti su più versioni. Nessuna modifica alle quote storicizzate, coefficiente artista, documenti o registri contabili.

## 7. Complessità, dipendenze e rischi

- **Complessità stimata:** bassa per campo e filtro locale opzionale, media per report aggregato corretto, medio-alta per provenienza del primo contatto, cronologia, eventi e compatibilità inter-versione. Sono stime qualitative, non tempi né impegni.
- **Dipendenze:** dati/registri vendite esistenti; eventuale rapporto con UX-009/047 e motore di aggregazione UX-034 solo se tali idee saranno approvate. Nessuna dipendenza obbligatoria da TECH-008/001/025, ma coerenza concettuale sui formati e sugli export. Nessuna nuova libreria richiesta nella fase di analisi.
- **Rischio semantico centrale:** «vendita in fiera» e «interesse nato in fiera» differiscono. Una classificazione incompleta non misura conversioni, quota di mercato, costi, numero di visitatori, visibilità o causalità. Evitare classifiche «canale migliore» su campioni piccoli.
- **Rischi operativi:** campi aggiuntivi rallentano il salvataggio; categorie troppo rigide non descrivono percorsi misti; correzioni retroattive falsano analisi confrontate nel tempo; esportazioni possono rivelare identità e abitudini. Alternative: note esistenti + Calc, oppure solo un filtro facoltativo.
- **Rischio di sovrapposizione:** se UX-002, UX-009, UX-012, UX-034 o UX-047 verranno approvate, occorre ridefinire una singola fonte per accordo/evento/canale e non creare due modalità concorrenti di attribuzione.

## 8. Verifiche, criteri di accettazione e fasi solo ipotetiche

**Fase 0 — validazione prima del prodotto:** osservare 5–8 artisti/gallerie con almeno 20 vendite ciascuno, chiedendo se annotano già il luogo di conclusione e l'origine del contatto, e provare **note + CSV + Calc**. Se la classificazione non è ripetuta o utile, **nessuno sviluppo**.

**Fase 1 eventuale — metadato minimo:** campo di modalità facoltativa su vendita, nessuna analisi obbligatoria, nessun riempimento automatico dello storico. Criteri: (1) vendita senza contesto funziona esattamente come oggi; (2) vendita classificata non modifica importi, quote, incassi, saldi o certificati; (3) valori «non registrato» e «non determinabile» sono distinti; (4) salvataggio/correzione e backup-ripristino preservano codici e testo; (5) tastiera, screen reader e lingue testati in Windows/macOS/Linux.

**Fase 2 eventuale — analisi non fiscale:** filtro e tabella sul periodo di vendita. Criteri: (6) somma/numero dei gruppi, inclusi sconosciuti, coincidono con le vendite selezionate, senza contare rate o incassi; (7) importi in centesimi senza conversioni arbitrarie; (8) export senza nominativi per default, con anteprima dei dettagli; (9) risultati dichiarano i dati non classificati; (10) CSV/backup storici rimangono leggibili e non vengono riscritti.

**Fase 3 soltanto dopo nuova valutazione:** fonte dichiarata del primo contatto e collegamento intenzionale a progetti espositivi, con verifica dei casi ambigui, della privacy e delle differenze fra canale e origine. Nessuna automazione, rete o CRM necessario.

## 9. Requisiti certi, ipotesi e domande al proprietario

**Certi nel progetto attuale:** gratuità e open source, Tauri con archivio SQLite locale e backup su cartella scelta, nessun account obbligatorio, CSV già presenti, locale OS con fallback inglese, vendita/quote/incassi separati.

**Osservazione verificata sul codice consultato:** il modello della vendita su main **non espone un campo strutturato per il contesto commerciale**. **Non** è un difetto dimostrato né una richiesta clienti accertata.

**Ipotesi da validare:** che classificare in modo facoltativo le vendite porti vantaggi rispetto alle note; che sia utile distinguere modalità di conclusione e origine dichiarata; che gli artisti abbiano campioni numerosi e affidabili per leggere un rapporto per canale.

**Domande per revisione personale, senza assumere risposte:** 
1. Per il tuo lavoro e per gli utenti che conosci, è più utile sapere **dove si è conclusa la vendita**, **da dove è nata la richiesta**, oppure nessuno dei due?
2. Esistono esempi reali di vendite passate documentabili senza attribuzioni arbitrarie?
3. Bastano **note + esportazione CSV** o vale la pena introdurre una categoria strutturata?
4. Se una vendita coinvolge una fiera e una galleria, preferisci una categoria principale manuale, più riferimenti dichiarati o «non determinabile»?
5. Vuoi che i rapporti futuri restino strettamente **privati** oppure che esista una copia aggregata minimizzata per collaboratori?

**Decisione finale non presa:** mantenere **«Proposta — da valutare»**. Nessuna idea è approvata, prioritaria per lo sviluppo o assegnata.
