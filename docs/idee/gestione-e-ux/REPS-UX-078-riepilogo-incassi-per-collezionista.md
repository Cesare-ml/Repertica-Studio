# REPS-UX-078 — Riepilogo privato delle vendite e degli incassi per collezionista

- **ID stabile:** REPS-UX-078
- **Data:** 2026-11-23
- **Stato:** Proposta — da valutare
- **Filone:** Gestione e UX
- **Priorità ipotetica:** P2 solo se il bisogno ricorrente è confermato; P3 o nessuna funzione se il registro attuale basta.
- **Perimetro:** studio, non specifica approvata né incarico di sviluppo.

## Problema, destinatari e riscontri

Artisti con acquirenti abituali, assistenti di studio e piccole gallerie possono dover verificare più vendite dello stesso contatto prima di rispondere a una richiesta sul saldo. Cercare ogni vendita separatamente richiede attenzione, specialmente con incassi parziali e anni diversi. Non si propone una classificazione della solvibilità del cliente.

**Riscontro statico su main, 23/11/2026:** [template Angular, righe 140–202](https://github.com/Cesare-ml/Repertica-Studio/blob/main/src/app/app.component.html#L140-L202): l'espansione del collezionista mostra le opere acquisite e le date, con rimando all'Archivio; il Registro vendite mostra importi, incassi e residuo per singola vendita. Non emerge un riepilogo degli importi di più vendite per lo stesso contatto. [refresh e indici](https://github.com/Cesare-ml/Repertica-Studio/blob/main/src/app/app.component.ts#L458-L489) caricano vendite e saldi e indicizzano le vendite per UUID di collezionista. [SaleBalance](https://github.com/Cesare-ml/Repertica-Studio/blob/main/src/app/models.ts#L284-L286) contiene già i tre importi. Attenzione: [balanceForSale](https://github.com/Cesare-ml/Repertica-Studio/blob/main/src/app/app.component.ts#L1472-L1477) usa un fallback a incassato zero se manca il saldo; un aggregato nuovo non dovrebbe trasformare questo fallback in un dato certo. [SALE-RECEIPTS](https://github.com/Cesare-ml/Repertica-Studio/blob/main/docs/SALE-RECEIPTS.md) distingue esplicitamente dichiarazioni di incasso e pagamenti verificati. Nessuna sessione di utilizzo, benchmark o errore reale osservati.

**Ipotesi:** un prospetto a richiesta ridurrebbe passaggi e rischio di confondere saldi di opere diverse. Verificare con utenti che abbiano più vendite per contatto; non aggiungere una schermata se il bisogno è raro.

## Confronto con le idee esistenti

Consultati gli indici di 77 UX e 73 TECH nelle PR #14 e #16. UX-014 tratta rubrica multi-ruolo; UX-024 tratta piani rateali già implementati e possibili revisioni, non un totale storico per acquirente; UX-060 riguarda attività della singola opera; UX-070 avvisi sugli incassi duplicati; UX-071 ritorno contestuale; UX-073 confronto annuale dello studio; UX-061 revisione privacy. TECH-008 filtri e viste salvate per opere, TECH-025 esportazioni selettive e TECH-055 snapshot dei CSV, TECH-072 coerenza MCP: non aggiungere filtri generici, export, nuovo MCP o una nuova contabilità. UX-017 eventuali vendite multi-opera richiederebbero in futuro conteggio per vendita, non per riga. La proposta circoscritta riguarda soltanto la consultazione economica privata per un contatto esistente. Nessuna modifica ai file tecnici.

## Scenari e flussi UX

- In Collezionisti → Dettagli di un contatto con vendite → azione esplicita «Riepilogo incassi». Non mostrare importi nella rubrica generale né nell'editor del contatto. Presentare una vista privata, con intestazione nominativa e avviso «Dati registrati nello studio, non verificati dalla banca».
- Esempio sintetico: due vendite di 800 e 1.200 EUR, incassi dichiarati rispettivamente 800 e 300 EUR; vendite registrate 2.000 EUR, incassi dichiarati 1.100 EUR, residuo registrato 900 EUR. Mostrare due righe con titolo, data, tre importi, più l'azione esistente «Gestisci incassi» per la vendita scelta. Nessuna scrittura prima di un salvataggio già autorizzato nel flusso corrente.
- Se non ci sono vendite, distinguere «Nessuna vendita registrata» dal saldo zero di vendite interamente incassate. Un residuo non prova ritardo, mancato pagamento o debito esigibile; non sommare rate e residuo, non mostrare semafori di affidabilità.
- Per omonimi usare solo l'UUID del contatto selezionato. Non fondere contatti, opere o acquisti di terzi. Chiudere il prospetto e tornare al contesto di partenza; chiudere anche al cambio/ripristino archivio.
- Per il primo perimetro: tutte le vendite registrate del contatto, indipendentemente dall'anno e dalla soglia di tesoreria di Economia, con etichetta esplicita. Niente filtri per data inizialmente: limitare le vendite per data non equivale a limitare gli incassi per data. Eventuale filtro successivo richiede distinzione esplicita tra coorte di vendite e movimenti del periodo.
- Se dati in caricamento, mancanti o non coerenti: indicare «Riepilogo non disponibile / aggiorna», senza presentare zero o totale parziale come certo. Rileggere dopo le modifiche; se il refresh indipendente di vendite e saldi non permette consistenza sufficiente, valutare solo successivamente una lettura aggregata locale coerente. Non risolvere il problema cambiando i dati.
- Accessibilità: tastiera, focus di ritorno, intestazioni semantiche, importi leggibili dal lettore schermo, stato testuale di caricamento/errore, contrasto e adattamento a finestre piccole. Nessun significato affidato unicamente al colore.

## Dati, impostazioni e API descrittive

Requisito certo: identificatori stabili e fonti esistenti (`Sale.collector_id`, `Sale.id`, `SaleBalance.sale_id`, importi interi in centesimi EUR). Nessuna tabella, migrazione o duplicazione di incassi nella fase minima. Derivazione effimera: elenco delle vendite del contatto, join per ID con saldo autorevole, conteggio, somme dei tre importi e stato completezza. Ogni vendita deve apparire una sola volta; verificare corrispondenza del lordo, non negatività, identità lordo = incassato + residuo e aritmetica sicura. Se non disponibile una vista coerente non esporre il totale. Una possibile API futura puramente descrittiva leggerebbe il prospetto per UUID in una sola snapshot locale: nessuna nuova API pubblica, nessuna scrittura e nessun allargamento dei dati MCP. Non inferire valuta da locale: EUR attuale; se mai multivaluta (UX-027), separare per valuta senza conversione implicita.

Nessuna impostazione obbligatoria; azione a richiesta, non persistente. UI nella lingua di sistema, fallback inglese; tradurre titoli, descrizioni, stati, messaggi, formattazione data/moneta e accessibilità. Nomi e titoli originali non vengono tradotti. Nessuna rete necessaria.

## Privacy, portabilità e compatibilità

Dati economici personali visibili solo nel prospetto scelto dall'utente, non in modalità presentazione, brochure, posta, notifiche o nuovi output MCP. Nessuna condivisione, sollecito o analisi creditizia automatica. Non affermare protezione da altri utenti del computer: il database locale resta soggetto ai permessi OS. Non conservare copie o log con nomi e importi. I CSV vendite, incassi e saldi integrali esistenti consentono già una verifica indipendente per UUID; nessun nuovo export/import nella fase minima. Eventuale prospetto CSV per singolo contatto richiederebbe autorizzazione distinta e anteprima dei campi (senza contatti o note superflui), destinazione scelta, etichetta del perimetro e non sovrascrittura; non sarebbe un backup. Backup completi e ripristino in cartelle scelte restano invariati, con avvertenze sulla mancata cifratura. Nessuna migrazione; compatibilità con archivi storici senza incassi, che rappresentano registrazioni mancanti, non mancati pagamenti accertati. Collaudo desktop Windows/macOS/Linux da prevedere, non eseguito.

## Complessità, dipendenze, alternative e rischi

Bassa per semplice lista contestuale con importi esistenti; bassa–media per aggregato con controlli di completezza, aggiornamento e accessibilità; media se necessaria lettura coerente nel backend. Nessuna libreria esterna, account o servizio. Dipende dalla semantica dei saldi attuali e dai flussi esistenti di incasso. Prima alternativa: Registro vendite attuale, oppure CSV saldi e filtro/tabella pivot locale in Calc; se sufficiente, nessuno sviluppo. Alternativa intermedia: sola lista delle vendite del contatto, senza aggregato. Rischi: somma di snapshot discordanti, zeri presunti, doppio conteggio, esposizione durante presentazioni, confusione tra residuo e scaduto, ulteriore ingombro della rubrica. Mitigazioni: completezza fail-closed, vista privata a richiesta, nessun giudizio sul cliente, nessun nuovo flusso di scrittura, test su dati sintetici.

## Criteri di accettazione ipotetici

1. Con due vendite sintetiche dell'esempio si ottengono esattamente 200000/110000/90000 centesimi; una vendita omonima di altro UUID è esclusa.
2. Vendita storica senza incassi dichiarati espone residuo registrato, senza affermare insolvenza; contatto senza vendite è distinto da contatto con residuo zero.
3. Saldo mancante, duplicato, incoerente o caricamento incompleto non produce un totale apparentemente valido; nessun utilizzo del fallback zero.
4. Registrare manualmente un incasso nel flusso esistente aggiorna la vista dopo refresh; aprire/chiudere il prospetto non cambia il DB, piani, certificati o documenti.
5. Dati di più anni inclusi esplicitamente; cambio anno in Economia non altera il prospetto; nessuna doppia somma delle rate.
6. Offline, cambio lingua e fallback inglese, tastiera e lettore schermo funzionano; nessuna trasmissione, nessuna nuova colonna CSV o inclusione nei documenti pubblici.
7. Ripristino/riapertura archivio invalida la vista e rilegge soltanto i dati correnti; backup ed export integrali rimangono utilizzabili.

## Priorità, fasi e domande al proprietario

- Fase 0: osservare 3–5 professionisti con acquirenti ricorrenti mentre verificano il saldo di più vendite; confrontare tempo/errori con registro e CSV; nessuna funzione se non emerge valore.
- Fase A eventuale: semplice elenco privato per contatto con importi e link agli incassi; valutare aggregato soltanto con completezza verificata. P2 condizionata.
- Fase B eventuale e separata: valutare snapshot locale coerente se necessaria, filtri semantici o export selettivo soltanto su domanda osservata. P3 condizionata.

**Requisiti certi:** offline-first, gratuità, nessun account obbligatorio, fonte autorevole esistente, export integrale preservato, privacy e revisione umana. **Ipotesi:** frequenza dei clienti ricorrenti, valore del totale storico, fattibilità senza nuova API. **Domande aperte:** basta il dettaglio delle singole vendite? Preferire un prospetto nella rubrica o un accesso al Registro vendite filtrato? È utile mostrare tutte le annualità o serve davvero una coorte? Quanto è importante mantenere gli importi nascosti durante l'uso in presenza del cliente? Serve mai un documento condivisibile, e con quali dati esplicitamente autorizzati? Nessuna risposta presunta e nessuna idea approvata.
