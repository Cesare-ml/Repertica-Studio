# REPS-UX-015 — Ricognizioni inventariali e gestione delle discrepanze fisiche

- **ID stabile:** REPS-UX-015
- **Data:** 2026-11-21
- **Stato:** Proposta — da valutare
- **Filone:** Gestione e UX
- **Priorità proposta:** P1–P2 solo per archivi che effettuano controlli fisici periodici; da convalidare
- **Perimetro:** studio progettuale, senza autorizzazione a implementare o assegnare lavori.

## Problema, destinatari e situazione verificata

Il branch [main](../../../README.md) permette di catalogare ogni opera con UUID, codice, titolo, stato commerciale e campo di ubicazione testuale; offre ricerca/filtri e backup completi. [MVP](../../MVP.md) e [modelli](../../../src/app/models.ts) non descrivono però una **ricognizione fisica periodica** con elenco delle opere attese, esito del controllo di ogni singolo pezzo e confronto sistematico delle discrepanze.

**Destinatari:** artisti con studio e deposito, piccole gallerie, archivi privati e professionisti che controllano materialmente dove si trovano decine o centinaia di opere. Il processo è meno necessario a chi ha poche opere sempre nello stesso ambiente.

Una ricerca per «in studio» non prova che l'opera sia presente; viceversa una vendita registrata non prova che l'opera sia già uscita. Chi compila un inventario deve distinguere almeno: «verificata nel posto atteso», «trovata in altro posto», «cercata ma non rinvenuta», «non ancora controllata» e «presenza non identificata». Senza questa distinzione, un inventario incompleto rischia di essere scambiato per una lista di opere scomparse.

### Confini con schede e interventi già esistenti

- [UX-001 — Provenienza e movimenti](REPS-UX-001-provenienza-movimentazioni.md) riguarda la **storia dei trasferimenti e la custodia**, non il controllo di presenza a una determinata data. Una differenza inventariale non crea un evento di movimento e non riscrive l'ubicazione.
- [UX-004 — Condizione e conservazione](REPS-UX-004-rapporti-stato-conservazione.md) riguarda **danni e conservazione**, non l'esito del censimento.
- [UX-009 — Progetti espositivi](REPS-UX-009-progetti-espositivi-checklist.md) riguarda mostre e checklist di preparazione: non costituisce una ricognizione generale del deposito.
- [UX-005 — Edizioni](REPS-UX-005-edizioni-multipli-serie-numerate.md) riguarda esemplari con identità distinta; una verifica futura deve poter distinguere tali esemplari e non sommare genericamente «copie».
- [UX-008 — Scorte materiali](REPS-UX-008-scorte-materiali-consumi.md) controlla quantità di colori, tele e materiali, non **opere individuali**.
- Le idee [TECH-008 — Viste salvate](https://github.com/Cesare-ml/Repertica-Studio/pull/16) e TECH-010 — Revisioni metadati non conservano dichiarazioni di controllo fisico. Le sei esportazioni CSV attuali e il backup locale non sostituiscono questa funzione. Nessun file dell'altro filone deve essere modificato.

## Scenari d'uso

1. **Studio con 65 dipinti.** Il catalogo ne indica 50 «in studio». Durante la verifica l'artista conferma 45 presenze, trova 2 opere in un altro locale, ne cerca 1 senza trovarla e non controlla le restanti 2. Il riepilogo non deve equiparare «non controllata» a «non rinvenuta».
2. **Piccola galleria con opere vendute non ritirate.** L'operatore osserva fisicamente un quadro già venduto ma ancora in deposito: la ricognizione non cambia vendita, certificato, incasso né stato commerciale.
3. **Controllo a più riprese.** Una sessione iniziata lunedì viene completata venerdì senza rete; gli esiti conservano la data in cui ogni opera è stata osservata, non presumono una singola fotografia temporale del deposito.
4. **Codice duplicato.** L'utente trova un'opera con etichetta ambigua: il sistema propone risultati possibili con UUID e metadati, ma non la assegna automaticamente al primo titolo simile.
5. **Opera estranea alla lista.** Si rinviene un oggetto non compreso nella selezione: annotazione «da identificare» senza creazione automatica di una scheda, vendita o provenienza.

## Flussi UX proposti (non approvati)

**A. Apri ricognizione.** Da «Inventario → Nuova ricognizione», titolo, data iniziale, eventuale luogo e ambito: tutte le opere, quelle con una determinata ubicazione dichiarata o una selezione esplicita. Mostrare prima di confermare **numero e criteri** dei record inclusi; lo stato «sold» non equivale a esclusione dal luogo. Nessun account o configurazione preliminare. Una sessione può essere sospesa e ripresa.

**B. Elenco di riferimento.** Al momento dell'apertura conservare uno **snapshot minimo** di UUID, codice, titolo, ubicazione e stato commerciale usati per costruire l'elenco «atteso». Gli oggetti non iniziano «presenti», «assenti» o «mancanti»: iniziano **non controllata**. Se cambiano titolo, ubicazione o status nel catalogo durante il controllo, mostrare «valore alla partenza» e «valore corrente» senza riscrivere lo snapshot.

**C. Rilievo per singola opera.** Ricerca per codice, titolo o identificativo; visualizzazione facoltativa della foto locale; pulsanti accessibili per esito:
- «Presente nel luogo atteso»;
- «Presente altrove» (luogo osservato facoltativo);
- «Non rinvenuta dopo il controllo» (non equivale a prova di smarrimento);
- «Da verificare / identificazione dubbia»;
- «Non controllata» per le righe ancora prive di rilievi.

Aggiungere data osservazione, nota sintetica, eventuale operatore **dichiarato** e foto solo se in futuro approvata e inclusa nel backup. L'inserimento di un rilievo deve essere rapido, con focus stabile e messaggi chiari; non richiedere trascinamento o lettore QR.

**D. Eccezioni e oggetti non associati.** Una presenza inattesa può essere registrata come osservazione libera (descrizione e foto opzionale), senza un UUID opera attribuito per somiglianza. Prima di collegarla a una scheda, conferma esplicita e indicazione dell'incertezza. Eventuali etichette duplicate devono sempre offrire disambiguazione manuale.

**E. Differenze e chiusura.** Riepilogo con conteggi **separati**: verificate, ancora da controllare, non rinvenute dopo verifica, presenti altrove, rinvenimenti non identificati, dati catalogo cambiati. Permettere risoluzione manuale con motivazione, oppure chiusura della sessione **con discrepanze aperte**. Mostrare «inventario incompleto» se restano record non controllati. Una nuova sessione non cancella quella precedente.

**F. Collegamento eventuale alla correzione.** Una differenza è una **osservazione**, non un evento di possesso o un aggiornamento automatico. In un prodotto che abbia approvato separatamente UX-001, un pulsante potrebbe aprire il suo flusso per registrare un trasferimento; in assenza di UX-001, la modifica del campo posizione rimane separata e intenzionale. Mai «correggi automaticamente tutte le posizioni», aggiorna vendite, emetti certificati o modifica incassi per effetto dei rilievi.

**Accessibilità:** tastiera completa, focus dopo ogni marcatura, compatibilità screen reader, stati espressi anche testualmente, zoom e contrasto; riepiloghi in tabella fruibile e filtri navigabili, evitando la sola codifica per colore.

## Dati necessari — ipotesi descrittive, non API operative

- **Sessione:** UUID, titolo, descrizione luogo/ambito, criteri versione/codice, data apertura, eventuale chiusura, stato bozza/attiva/chiusa, note, timestamp; operatore testuale facoltativo e non autenticato.
- **Elementi attesi:** session UUID, artwork UUID, codice/titolo/ubicazione/status **catturati all'apertura**, data baseline, eventuale riferimento futuro a esemplare; coppia sessione/opera univoca. La presenza nel baseline non dimostra presenza fisica.
- **Osservazione:** UUID, session UUID, artwork UUID opzionale per oggetto non identificato, esito tipizzato, luogo rilevato facoltativo, data rilevazione e data registrazione distinte, dichiarazione operatore facoltativa, nota, riferimento a osservazione rettificata. Nessuna geolocalizzazione obbligatoria.
- **Risoluzione:** UUID, osservazione e sessione, esito (da verificare/confermata/rettificata/chiusa motivatamente), data, motivazione, eventuale futuro riferimento al **vero movimento** UX-001; nessuna modifica automatica di altri domini.
- **Invarianti:** i rilievi importanti non scompaiono con un overwrite silenzioso; le correzioni devono conservare storia. Gli UUID delle opere non cambiano. Una scheda eliminabile deve ricevere protezione referenziale o chiara strategia di conservazione storica; dati di riferimento mancanti devono restare riconoscibili.
- **Limiti:** nessun utente remoto, multi-postazione concorrente o identità verificata. La prima fase può rinunciare a una entità di risoluzione distinta se conserva comunque la tracciabilità minima dei cambi di esito.

## Impostazioni, offline, lingue, privacy

- **Preferenze opzionali:** luoghi suggeriti, ordinamento per codice o ubicazione, layout compatto, campo «operatore» manuale, modelli di checklist e stampa senza indirizzi completi per default. Nessun inventario periodico forzato, nessuna notifica di sistema o scanner necessario.
- **Offline:** tutte le operazioni in SQLite dell'app e cartelle locali; nessun servizio web, account, telemetria, riconoscimento visivo remoto o sincronizzazione. I backup continuano a essere completi e nella **cartella scelta liberamente**, anche USB/NAS montati; nessuna esportazione al posto del backup.
- **Multilingua:** lingua del sistema IT/EN/ES/FR/DE secondo [ECONOMY-I18N](../../ECONOMY-I18N.md), override manuale e fallback inglese anche per stringhe mancanti. Tradurre stati, avvisi, stampa e messaggi; mantenere valori dati come codici non localizzati, note e nomi propri nella lingua inserita. Date ISO nello scambio e formattate per la locale nell'interfaccia.
- **Riservatezza:** ubicazioni di opere di valore e nominativi di custodi possono avere implicazioni di sicurezza personale. Non mostrare per default indirizzi privati, contatti collezionisti, note riservate, prezzi o dati finanziari nei verbali da condividere. Nessun controllo equivale a certificazione assicurativa o giudiziale. I backup attuali sono in chiaro: avviso coerente.

## Import, export, interoperabilità e compatibilità

- **Esportazione consultiva:** CSV UTF-8 con UUID, baseline, esito dichiarato, luogo osservato (se scelto), data e stato differenza; CSV di sessione **separato** dai sei registri integrali oggi presenti. Prima di stampa/CSV, anteprima e scelta esplicita dei dati sensibili. Verbale stampabile non fiscale, indicato come «ricognizione interna» con data, criteri, incompleti e differenze.
- **Round-trip futuro:** JSON versionato con ID delle sessioni, snapshot e osservazioni, riferimenti UUID ed eventuali allegati relativi; import solo da file scelto, anteprima di conflitti e referenze mancanti, deduplicazione/idempotenza e annullamento transazionale; non creare opere né riscriverne l'ubicazione durante import. Allineamento concettuale a TECH-001, senza modificare la proposta tecnica.
- **Compatibilità:** futura migrazione SQLite **additiva**; archivi preesistenti senza sessioni restano validi. I documenti originali/certificati immutabili e le quote di vendita non cambiano. I backup devono includere sessioni e allegati, non solo riferimenti esterni; versione del database più recente può non essere apribile con vecchie versioni dell'app: avvisare e collaudare restore su Windows/macOS/Linux. File esterni prodotti dall'esportazione non sono automaticamente inclusi nel backup.
- **Scalabilità:** valutare paginazione e recupero incrementale degli esiti per migliaia di opere, salvataggio a ogni operazione e ripresa sicura dopo crash; non promettere inventario simultaneo da più computer.

## Complessità, dipendenze, rischi e alternative

- **Complessità:** media per sessione/marcature/report; medio-alta per snapshot immutabile, rettifiche, conflitti e import/export idempotente; alta per scanner mobile o collaborazione, fuori dalla fase iniziale.
- **Dipendenze:** UUID e campi catalogo correnti, database/migrazioni, backup, i18n e navigazione accessibile. Non dipende necessariamente da UX-001, UX-004 o TECH-008: possibili integrazioni solo se approvate autonomamente.
- **Rischi:** «mai controllata» confusa con «scomparsa»; titolo/codice omonimo collegato all'opera sbagliata; correzioni massive errate; dati di posizione esposti; una lista chiusa interpretata come inventario certificato; sessioni molto lunghe su dati cambiati.
- **Mitigazioni:** stati non ambigui, identificativi stabili, confronto baseline/live, correzioni solo esplicite, verbale con limiti, anteprima privacy e test ripristino.
- **Alternativa a costo zero:** stampare la lista delle opere e spuntarla a mano. È adeguata per archivi piccoli, ma perde confronto e tracciabilità.
- **Alternativa minima interna:** checklist temporanea di opere selezionate senza persistenza dei confronti. Da preferire se le interviste non dimostrano bisogno di ricognizioni ripetute.

## Criteri di accettazione ipotetici

1. In sessione da 100 opere, dopo 60 controlli di cui 2 «non rinvenute», il riepilogo mostra **40 non controllate**, non «42 mancanti».
2. Verificare una opera venduta ma ancora in deposito non cambia vendita, stato commerciale, certificato, incassi o statistiche mensili.
3. Trovare una opera altrove crea differenza visibile ma non sovrascrive il campo ubicazione; eventuale cambio successivo nel catalogo appare come valore corrente diverso dal baseline.
4. Codici duplicati richiedono disambiguazione manuale; rinvenimenti non identificati non producono nuove schede in automatico.
5. Correzioni e ulteriori osservazioni conservano traccia; una chiusura con rilievi non risolti mantiene visibile l'incompletezza.
6. Crash/ripresa, schede modificate durante sessione e grandi inventari non producono duplicati né perdite di rilievi.
7. Anteprima esportazione protegge dati di clienti e ubicazioni; import ripetuto non duplica sessioni o osservazioni; backup/ripristino locale recupera sessioni complete.
8. Funziona completamente offline, con lingua di sistema/fallback inglese, nomi Unicode, tastiera e screen reader su Windows/macOS/Linux.

## Fasi esplorative, non autorizzate

- **A — Validazione:** osservare 3–5 controlli fisici reali presso artisti/gallerie e capire frequenza, volumi, luoghi e codici utilizzati.
- **B — Nucleo semplice:** sessione locale, elenco snapshot, marcature e conteggio certo delle **non controllate**, report stampabile minimo.
- **C — Differenze consapevoli:** rettifiche, identificazioni ambigue, confronto baseline/live e collegamento **facoltativo** agli eventi di movimento se UX-001 sarà approvata.
- **D — Interoperabilità:** export/import versionato, prove su grandi archivi e file esterni; valutazione di scanner USB solo dopo necessità dimostrata.

## Distinzione fra certezze, ipotesi e decisioni del proprietario

- **Certi da main al 21/11/2026:** UUID/codice/titolo/ubicazione/stato delle opere, archivio SQLite, backup completi a cartella scelta, sei esportazioni CSV, IT/EN/ES/FR/DE con fallback inglese; nessun modulo di ricognizione fisica descritto.
- **Ipotesi:** inventari periodici sufficientemente frequenti e grandi giustificano una sessione dedicata; gli operatori preferiscono esiti in app rispetto a fogli cartacei.
- **Questioni aperte per il proprietario:** utenti principali artisti singoli o anche gallerie e depositi? Sessione sempre riferita a un luogo o anche selezione trasversale? Quali dati identificano gli esemplari più spesso (codice, fotografia, eventuale QR)? Serve verificare in più giorni con operatori differenti, pur senza account? Quanto è importante conservare lo storico delle sessioni? È sufficiente un verbale interno non firmato o serve solo CSV?

**Stato conclusivo: Proposta — da valutare. Nessuna approvazione o assegnazione allo sviluppo.**
