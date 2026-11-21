# REPS-UX-001 — Provenienza, ubicazioni e movimentazioni delle opere

- **Stato:** Proposta — da valutare
- **Data:** 2026-11-21
- **Filone:** Gestione e UX
- **Priorità proposta:** P1 (soggetta a decisione del proprietario)
- **Perimetro:** esclusivamente documentale; non autorizza implementazioni

## Contesto verificato e differenza rispetto all'esistente

Nel [MVP](../../MVP.md) un'opera ha un campo `location`, uno `status` (`available`, `reserved`, `sold`, `archived`) e può essere collegata a una vendita e a certificati immutabili. L'archivio permette ricerca e relazione con il collezionista, ma il modello attuale non rappresenta una **cronologia verificabile di trasferimenti fisici** né la **provenienza dichiarata**. Non si propone di rifare l'archivio, il backup o i certificati.

## Problema e utenti

Un artista, uno studio o una piccola galleria deve sapere dove si trova un'opera, chi la custodisce, quando rientrerà da una mostra, se è in conto visione, prestito o deposito, e quali passaggi documentati ne attestano la storia. Una posizione testuale aggiornata a mano perde le informazioni precedenti; una vendita non prova da sola la consegna fisica. La ricostruzione manuale è fragile e causa smarrimenti, omissioni e dichiarazioni non verificabili.

## Scenari concreti

1. **Mostra temporanea:** cinque opere escono dallo studio; l'utente registra destinazione, data di uscita, responsabile, data prevista di rientro, eventuale stato conservativo e allegato verbale. Al rientro registra un nuovo evento, senza cancellare la storia.
2. **Opera venduta ma non consegnata:** la vendita esiste, ma l'ubicazione rimane lo studio finché non viene registrata una consegna. I due stati sono mostrati distintamente.
3. **Provenienza pregressa incompleta:** per un'opera acquisita o gestita per terzi l'utente può annotare passaggi precedenti con fonte e grado di certezza, senza presentarli come automaticamente autenticati.

## Esperienza utente proposta

- Scheda opera → pannello **Storia e ubicazioni**: luogo corrente, detentore/custode, eventuale rientro previsto e timeline cronologica con etichette **registrato**, **dichiarato** o **da verificare**.
- Azione **Registra movimento**: tipologia (ingresso, uscita, trasferimento, prestito, conto visione, deposito, consegna, rientro), luogo di origine e destinazione, data, controparte facoltativa, nota, condizioni osservate, riferimento a documento.
- Selezione multipla di opere per una medesima mostra, con conferma per ciascuna e riepilogo degli errori; in prima fase è accettabile la registrazione singola.
- Vista **Opere fuori sede** con filtri per luogo, responsabile, tipologia e rientri scaduti. Nessun messaggio automatico o invio a terzi.
- **Provenienza** come sezione diversa dai movimenti del proprio studio: fonti e qualifiche dei fatti, con rettifiche tracciate; nessun badge implicito di autenticità.
- Accessibilità: form navigabili da tastiera, errori associati ai campi, timeline leggibile senza dipendere dai colori, stampa ad alto contrasto.

## Dati minimi proposti (non schema approvato)

- Evento: UUID, `artwork_id`, tipo, data/ora o data parziale con precisione esplicita, luogo origine/destinazione, controparte descrittiva o collegamento opzionale al collezionista, custode, rientro previsto, note, riferimento allegato locale, creato/modificato, autore locale opzionale.
- Evento di provenienza: UUID, opera, descrizione del passaggio, periodo/eventuale intervallo, soggetto/luogo, riferimento fonte, natura della fonte (documentale, testimonianza, non verificata), indicatore di incertezza, note e rettifiche.
- Gli allegati sono copie locali sotto il percorso media del singolo archivio; i link esterni, se conservati, sono solo metadati e **non vengono contattati automaticamente**.
- Distinguere `status` commerciale esistente da **ubicazione fisica** e **custodia**, evitando transizioni implicite fra i due. Una vendita non sposta fisicamente l'opera. Proteggere record correlati da eliminazioni orfane.
- Le correzioni a eventi storici significativi dovrebbero generare un record di rettifica collegato, non una riscrittura silenziosa. I certificati già emessi rimangono intatti.

## Offline, lingue, impostazioni, import/export e privacy

- **Offline-first:** tutto su SQLite e cartella media esistenti; nessuna geolocalizzazione, cloud, email, API, telemetria, accesso remoto o firma online.
- **Lingue:** applicare la selezione automatica lingua del sistema e fallback inglese definiti in [Economy & i18n](../../ECONOMY-I18N.md); termini inventariali e livelli di certezza localizzati in italiano/inglese/spagnolo/francese/tedesco. Esportazioni con codici dati stabili, indipendenti dalla lingua UI.
- **Impostazioni:** tipi di movimento e luoghi ricorrenti facoltativi; una preferenza per la visualizzazione di date incomplete; nessuna configurazione obbligatoria.
- **Esportazione:** CSV UTF-8 per registro movimenti (una riga per evento); JSON versionato per timeline/provenienza con UUID e riferimenti; allegati mediante pacchetto locale esplicito con manifest e percorsi relativi. Import **in anteprima**, deduplica per UUID, gestione conflitti e rollback; non sostituire il formato di backup esistente.
- **Privacy:** nomi di custodi, recapiti, indirizzi privati e allegati sensibili possono comparire nelle esportazioni; chiedere conferma e permettere versione senza contatti. Backup in chiaro già documentati: non considerarli cifrati.

## Complessità, dipendenze, rischi, alternative

- **Stima:** media-alta; maggiori costi in storia append-only, allegati e importazione affidabile.
- **Dipendenze:** stabilità ID opere/collezionisti, versionamento schema SQLite, gestione file e backup esistenti, convenzioni i18n.
- **Rischi:** apparente certificazione della provenienza, confusione fra possesso e proprietà legale, incongruenze di tempi/luoghi, duplicati da import, dati personali esportati.
- **Mitigazioni:** attribuzione fonti e livelli di certezza; distinzioni terminologiche; controlli su movimenti che pretendono più ubicazioni correnti; segnalazioni senza blocco per storie incomplete; nessuna attestazione giuridica.
- **Alternativa leggera:** solo registro trasferimenti e vista fuori sede, rimandando la provenienza dettagliata; evitare geolocalizzazione o firma digitale nella prima fase.

## Criteri di accettazione ipotetici

1. Vendere un'opera non modifica luogo/custode; registrare consegna aggiorna la vista della sua ubicazione, non il certificato storico.
2. Uscita e rientro rimangono entrambi consultabili e ordinabili; un evento incompleto mostra chiaramente dati mancanti.
3. Un'opera in due prestiti fisicamente incompatibili provoca avviso e chiede correzione, non crea due ubicazioni correnti.
4. La vista **fuori sede** funziona offline anche su archivi numerosi e filtra i rientri scaduti.
5. Backup/ripristino conservano storia e allegati senza alterare il funzionamento dei backup attuali.
6. Esportazione e reimportazione di prova conservano ID, ordinamento e indicatori di certezza; import duplicato non moltiplica eventi.
7. La navigazione da tastiera permette registrare e consultare tutti gli eventi; tutti gli stati restano comprensibili senza colore.

## Fasi proposte

- **Fase A:** eventi di movimento, ubicazione corrente derivata, registrazione e vista fuori sede.
- **Fase B:** allegati, movimenti multipli e gestione scadenze locali.
- **Fase C:** provenienza con fonti/rettifiche ed export/import strutturato.

## Cosa è certo, cosa è ipotesi, decisioni da prendere

- **Requisiti certi:** software gratuito/open source, locale senza account; conservazione della privacy; certificati immutabili; SQLite e backup locali; lingue del sistema con fallback inglese.
- **Ipotesi progettuali:** necessità di un registro append-only; interesse per prestiti, mostre e provenienze; possibile utilizzo da gallerie.
- **Domande al proprietario:** servono conti visione e prestiti come casi principali o basta il registro luoghi? Quali prove documentali e quali date incomplete meritano supporto? È desiderabile una ricevuta stampabile di consegna **non fiscale**?

**Decisione attuale:** nessuna approvazione o assegnazione allo sviluppo.
