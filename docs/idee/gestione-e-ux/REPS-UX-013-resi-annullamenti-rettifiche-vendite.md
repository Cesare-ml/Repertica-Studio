# REPS-UX-013 — Resi, annullamenti e rettifiche post-vendita con cronologia

- **ID stabile:** REPS-UX-013
- **Data:** 2026-11-21
- **Stato:** Proposta — da valutare
- **Filone:** Gestione e UX
- **Priorità proposta:** P1 per affidabilità del registro vendite; ipotesi da convalidare
- **Perimetro:** sola proposta documentale, senza interventi sulle regole del software e senza documentazione fiscale automatica

## Stato di fatto e unicità della proposta

Su `main`, `record_sale` crea una vendita per opera e aggiorna l'opera a `sold`; `delete_sale` elimina la vendita e rimette l'opera a `available` se consentito. La migrazione SQLite v4 ha aggiunto incassi manuali parziali: non si può cancellare una vendita con incassi collegati. I certificati già emessi sono snapshot storici immutabili. La contabilità mensile distingue fatturato registrato da denaro incassato.

Questa proposta riguarda il **ciclo successivo alla vendita**: come registrare una cancellazione, un rimborso, un reso, una rettifica di dati o un'eventuale successiva rivendita senza cancellare la storia. Non ripete:
- [UX-002](REPS-UX-002-trattative-prenotazioni.md), che termina alla conversione in vendita;
- [UX-001](REPS-UX-001-provenienza-movimentazioni.md), che riguarda consegne e rientri **fisici**, non l'annullamento commerciale;
- [UX-003](REPS-UX-003-costo-e-margine-per-opera.md), che attribuisce costi di produzione;
- [issue #13](https://github.com/Cesare-ml/Repertica-Studio/issues/13), [docs/SALE-RECEIPTS.md](../../SALE-RECEIPTS.md) e la [PR #23](https://github.com/Cesare-ml/Repertica-Studio/pull/23), che gestiscono **incassi e loro export**, non una pratica di rettifica della vendita con storia conservata.

## Problema e destinatari

Un artista o gallerista può aver inserito una vendita sbagliata, concordato una cancellazione prima della consegna, ricevuto indietro un'opera o dover documentare un rimborso. Il semplice pulsante «elimina vendita» rischia di nascondere ciò che era stato dichiarato nei rendiconti; con incassi esistenti l'eliminazione è bloccata. Al tempo stesso, indicare un'opera come «disponibile» può essere falso se la restituzione fisica non è avvenuta. Serve una UX prudente con **rettifiche esplicite**, non un sistema fiscale o bancario.

## Scenari concreti

1. **Errore di inserimento, nessun incasso**: l'utente ha selezionato il collezionista sbagliato. Sceglie «Correggi dati» e vede quali informazioni possono essere aggiornate e quali richiedono una rettifica storicizzata, senza cancellare il fatto registrato.
2. **Vendita annullata dopo acconto**: l'opera risulta venduta e un incasso di 200 EUR è registrato. Un annullamento crea una pratica con motivazione; l'acconto rimane un fatto dichiarato. La pratica segnala «rimborso da documentare», non registra un rimborso automatico.
3. **Reso dell'opera dopo consegna**: l'utente distingue «accordo commerciale risolto», «opera restituita fisicamente» e «rimborso dichiarato», con relative date anche diverse. La disponibilità commerciale non torna automaticamente «available».
4. **Rivendita successiva**: la vecchia vendita resta consultabile e contrassegnata secondo la rettifica; un'eventuale nuova vendita non deve violare silenziosamente l'attuale unicità per opera o alterare vecchi certificati.

## Esperienza utente candidata

- **Vendite → Dettaglio → Gestisci variazione**: azioni etichettate *Correggi*, *Annulla*, *Registra reso*; wizard guidato che mostra impatti su stato opera, quote, report mensili, incassi dichiarati e certificati collegati prima di confermare.
- **Rettifica semplice**: distinguere errore materiale (data o nota) da evento reale avvenuto successivamente. Registrare versione precedente, valore nuovo, causale e data di efficacia; impedire riscrittura occulta di campi che determinano gli storici.
- **Pratica di annullamento/reso**: stato *aperta*, *da verificare*, *conclusa*; motivo facoltativo codificato e note libere; riepilogo visibile «vendita registrata», «acconti dichiarati», «restituzione fisica», «rimborsi registrati» con assenze marcate **non verificato**.
- **Scelta separata su disponibilità**: offrire un passaggio esplicito per rimettere a disposizione l'opera solo quando le condizioni commerciali lo consentono; non dedurre da qui il luogo né la prova della consegna.
- **Report mensili**: filtro tra vendite originarie, annullamenti/storni e totali rettificati; date «vendita originaria» e «rettifica» entrambe disponibili; etichette che spiegano il criterio di periodo selezionato. Non chiamare questi valori «saldo bancario».
- **Scheda opera e collezionista**: mostrare riferimento alla vendita storica e alla pratica in evidenza; nessuna perdita della relazione, nessuna cancellazione automatica di certificati emessi.
- **Accessibilità**: conferma con tastiera, focus sui rischi, testo oltre ai colori, riepiloghi leggibili da screen reader; blocco chiaro delle azioni incompatibili senza scorciatoie distruttive.

## Dati e invarianti (schema solo concettuale)

- **Pratica post-vendita**: UUID, `sale_id`, tipo (errore, annullamento, restituzione, modifica condizioni), data di registrazione, data di efficacia, stato, motivazione, documenti locali opzionali, note e riferimenti a rettifiche correlate.
- **Eventi append-only**: UUID, pratica, tipo, data, importi eventualmente coinvolti in **centesimi EUR**, riferimento a incasso/rimborso se rappresentato da una registrazione **autonoma e distinta**, utente locale descrittivo opzionale (senza account), precedente versione e nuova versione dove pertinenti.
- **Classificazione vendita**: distinguere evento storicamente registrato, validità commerciale attuale e importo incluso nei report, senza trasformare ogni cancellazione in un DELETE fisico. Eventuale seconda vendita della stessa opera richiede una **decisione architetturale separata**: vincolo attuale «massimo una vendita per opera» non si può aggirare con righe inventate o sovrascritture.
- **Incassi/rimborsi**: non alterare `sale_receipts` alla cieca. Un incasso documenta denaro che l'utente dice di avere ricevuto; un rimborso è un'altra dichiarazione e non equivale a «incasso negativo». Valutare un registro distinto solo in coordinamento con l'evoluzione di Finance 2; non inventare movimenti bancari.
- **Certificati**: snapshot e seriali storici rimangono immutabili. Un eventuale «non più attuale» deve essere **annotazione separata e datata**, non editing del certificato originale.
- **Compatibilità vendite precedenti**: nessun annullamento dedotto automaticamente da vendite senza note o da stati opera. Le quote storiche personali/attività/risparmio restano associate all'evento originario; regole di rettifica dei report da definire e spiegare esplicitamente.

## Impostazioni, offline, multilingua e privacy

- **Impostazioni**: motivo di rettifica suggerito ma facoltativo, criterio visualizzazione report (origine/evento di rettifica), conferma rafforzata per annullamento, senza preimpostare aliquote o trattamenti legali.
- **Offline**: registrazioni locali e documenti in archivio Tauri/SQLite; nessun collegamento a banca, cassa, fatturazione elettronica, email, calendario o provider terzi; nessun account/abbonamento.
- **Lingue**: lingua di sistema con override locale IT/EN/ES/FR/DE e fallback inglese; linguaggio differenziato per «annullata», «restituita», «rimborsata», «non consegnata» e per gli importi non verificati. Le causali in testo libero conservano la lingua inserita.
- **Privacy**: dati di clienti, motivi sensibili e documenti non esportati in modo indiscriminato. Anteprima con oscuramento di contatti e note private, esportazione esplicitamente autorizzata. Backup in chiaro da proteggere.

## Interoperabilità, compatibilità e dipendenze

- **CSV** con registro rettifiche, collegamenti UUID e colonne separate per importi originari/rettifiche/incassi/rimborsi dichiarati; mai una riga di vendita storica «sparita» senza spiegazione. Il CSV corrente delle vendite resta completo secondo la sua specifica finché non si decide esplicitamente un criterio per vendite annullate.
- **JSON** strutturato versionato per un eventuale round-trip, idempotenza degli UUID, preservazione della cronologia e degli snapshot; import con anteprima, controlli referenziali, rollback e nessun DELETE implicito. Allineamento concettuale con [TECH-001](https://github.com/Cesare-ml/Repertica-Studio/pull/16).
- **Backup/ripristino**: cartelle scelte liberamente, preservazione di pratiche, relazioni e allegati; compatibilità con archivi legacy v1–v5 tramite migrazione additiva e assenza di annullamenti fittizi. Vecchie versioni dell'app possono non leggere schemi futuri: comunicarlo prima di ripristinare.
- **Dipendenze**: invarianti Rust/SQLite su vendite e ricevute, calcoli di `sale_finances` e aggregati mensili, certificati append-only, stato opera, i18n, eventuale futuro registro movimenti UX-001.
- **Piattaforme**: identico comportamento in Windows, macOS e Linux, importi coerenti al centesimo, Unicode e test di ripristino su disco esterno.

## Complessità, rischi e opzioni

- **Complessità**: medio-alta per correzioni semplici senza incasso, alta per annullamento con incassi/rimborsi, molto alta se si autorizza rivendita nel medesimo archivio mantenendo storia e vincoli.
- **Rischi**: falsare report annuali cancellando vendite storiche; confondere rimborso con incasso negativo; certificati staccati dalla storia; liberare un'opera non restituita; produrre documenti che sembrino note di credito fiscali; migrazioni distruttive dell'unicità.
- **Mitigazioni**: append-only e rettifiche collegate, anteprima delle conseguenze, date distinte e semantica chiara, divieto di generare documenti fiscali, restrizioni transazionali senza cancellazioni implicite.
- **Alternativa minima**: «vendita contestata/da rettificare» con nota e data non modificabile; non cambia lo stato né il report finché l'utente non compie un'azione manuale. Alternativa esterna: annotazioni fuori dal gestionale e rettifica contabile manuale, meno sicure per la memoria storica.

## Criteri di accettazione ipotetici

1. Con vendita senza incassi l'utente può avviare una rettifica che mantiene l'ID e il riferimento storico, con anteprima delle conseguenze prima di confermare.
2. Con vendita di 1.000 EUR e acconto dichiarato di 200 EUR, l'annullamento **non cancella** automaticamente la vendita né l'incasso e non registra un rimborso inesistente.
3. Un reso commerciale non cambia automaticamente ubicazione né attesta restituzione fisica; un certificato storico mantiene numero e snapshot.
4. Le viste mensili distinguono importo originale, rettifiche e criterio temporale di aggregazione; i test escludono doppi conteggi.
5. Tentare una nuova vendita per un'opera già venduta/annullata non aggira il vincolo attuale: mostra una decisione di flusso esplicita o un errore leggibile finché il modello di rivendita non è definito.
6. Operazioni fallite o annullate non lasciano pratiche o totali parzialmente modificati; le variazioni mantengono UUID e data.
7. Export/import di prova e backup/ripristino preservano cronologia, allegati, riferimenti a incassi e certificati senza registrazioni inventate.
8. Navigazione tastiera, screen reader, messaggi localizzati e fallback inglese funzionano offline su desktop supportati.

## Suddivisione possibile, mai incarico di implementazione

- **A — Ricerca**: raccogliere casi reali, separare errori materiali, annullamenti, resi, rimborsi e rivendite; concordare semantica di report e privacy.
- **B — Minimo prudente**: marcatura «da rettificare», causale, cronologia, allegati, anteprima; nessuna mutazione fiscale o correzione automatica dei totali.
- **C — Rettifiche amministrative**: annullamento con regole esplicite per l'inclusione nei report, legami a incassi, segregazione dei rimborsi dichiarati e test contabili.
- **D — Rivendite**: esaminare revisione del vincolo una vendita per opera solo dopo conferma della domanda e modello di storicizzazione dei certificati.

## Certezze, ipotesi e quesiti per la revisione del proprietario

- **Certi nel repository**: vendita unica per opera, eliminazione che ripristina disponibilità se permessa, incassi parziali che impediscono l'eliminazione, dati in centesimi, certificati storici immutabili, archivi e backup locali.
- **Ipotesi**: il rischio di cancellare la storia commerciale è sufficiente a giustificare uno storico delle correzioni; i resi e le cancellazioni sono abbastanza comuni da chiedere una UX dedicata.
- **Domande**: serve davvero la rivendita dopo un reso nella prima release? Un errore materiale di nome cliente deve poter essere modificato senza pratica completa? Il report annuale deve esporre correzione nel mese originario o nel mese di registrazione della rettifica (o entrambe le viste)? Quando una vendita è annullata ma l'opera non è rientrata, quale etichetta di disponibilità è corretta? Come distingueremo un rimborso dichiarato da uno fiscalmente documentato?

**Tutto resta Proposta — da valutare. Nessuna approvazione o assegnazione allo sviluppo.**
