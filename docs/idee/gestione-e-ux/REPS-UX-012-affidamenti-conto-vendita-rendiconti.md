# REPS-UX-012 — Affidamenti in conto vendita e rendiconti agli intermediari

- **ID stabile:** REPS-UX-012
- **Data:** 2026-11-21
- **Stato:** Proposta — da valutare
- **Filone:** Gestione e UX
- **Priorità proposta:** P1–P2 per gallerie e artisti con intermediari; da verificare con utenti reali
- **Perimetro:** solo analisi; nessuna autorizzazione a sviluppare, creare contratti vincolanti o emettere documenti fiscali

## Riscontro nel repository e confini

In `main` le opere hanno un'ubicazione descrittiva e uno stato commerciale; le vendite associano una sola opera e un collezionista, con importo EUR in centesimi. Esistono quote personali/attività/risparmio salvate per vendita e incassi **dichiarati manualmente**, ma non una struttura specifica per un **mandato di conto vendita** con controparte, termini, durata, eventuali percentuali e rendiconto. Non dedurre che il software sappia chi possiede legalmente l'opera.

Questa proposta **non duplica**:
- [UX-001](REPS-UX-001-provenienza-movimentazioni.md): registra **dove** e presso **chi** si trova fisicamente un'opera. Qui si registrerebbe **a quali condizioni commerciali** è affidata a un intermediario. Un affidamento non dimostra una consegna né la crea.
- [UX-002](REPS-UX-002-trattative-prenotazioni.md): tratta offerte e prenotazioni rivolte a un potenziale acquirente. Il mandato regola il rapporto continuativo fra artista/proprietario e galleria/intermediario.
- [UX-003](REPS-UX-003-costo-e-margine-per-opera.md): imputa costi di produzione, non accordi di distribuzione/commissioni dovute a terzi.
- [Issue #13](https://github.com/Cesare-ml/Repertica-Studio/issues/13): affronta **commissioni e contabilità generale**. Questo documento propone il *processo e la UX di affidamento/rendiconto*; ogni effettiva scrittura finanziaria dovrebbe passare, dopo futura decisione esplicita, dal registro economico autorevole, mai da una seconda cassa.
- [UX-009](REPS-UX-009-progetti-espositivi-checklist.md): coordina mostre e attività, non il conto vendita.

## Problema e persone coinvolte

Un artista lascia opere a una galleria in conto vendita oppure una galleria custodisce opere appartenenti a più artisti. Senza un registro degli accordi, **prezzo concordato, percentuale di commissione, durata, condizioni di restituzione, riduzioni consentite e rendiconti** rimangono in documenti separati. È facile confondere il valore di listino, quanto dovrebbe spettare all'artista e quanto è stato effettivamente dichiarato come incassato. Il bisogno è particolarmente concreto per studi, associazioni, gallerie indipendenti e intermediari con molte opere in affidamento; meno frequente per chi vende esclusivamente in proprio.

## Scenari d'uso verificabili con interviste

1. **Artista che affida tre opere a una galleria.** Registra data, controparte, opere, prezzi richiesti e commissione proposta del 30%. Stampa un riepilogo non contrattuale da controllare con l'accordo reale; lo stato commerciale delle opere non cambia.
2. **Vendita segnalata dalla galleria.** Collega manualmente la comunicazione all'opera, distingue prezzo al pubblico, commissione teorica e importi effettivamente documentati. Non segna un incasso senza registrazione separata.
3. **Rientro di un'opera invenduta.** Chiude l'affidamento con data e motivo; la cronologia commerciale resta, mentre il ritorno fisico è eventualmente registrato con UX-001, se mai approvata.
4. **Galleria con opere di artisti diversi.** Consulta elenco degli affidamenti da rendicontare per controparte, senza trasformare i proprietari in «collezionisti acquirenti» e senza inviare comunicazioni automatiche.

## Flussi UX proposti

- **Opera → Rapporti commerciali → Nuovo affidamento**: selezionare controparte esistente o inserirne una essenziale; dichiarare il *ruolo* (proprietario/artista, galleria mandataria, intermediario) con testi chiari e modificabili; data avvio, termine facoltativo, tipo accordo e note.
- **Scheda accordo**: numero di riferimento locale, elenco opere collegate, prezzo di listino in EUR, base commissionale dichiarata (lordo o altra base concordata), percentuale o importo fisso, eventuali costi accessori **informativi**, data e riferimento del documento sorgente. Se la base non è definita, visualizzare «calcolo non disponibile».
- **Anteprima del rendiconto**: griglia «opera / prezzo concordato / vendita registrata / commissione contrattuale stimata / quota teorica destinatario / incassi dichiarati separati / importi da verificare». La quota teorica non è il saldo bancario, né un'imposta o una spesa automaticamente pagata.
- **Chiusura/cessazione**: registrare scadenza, rinnovo o rientro con nuova voce cronologica; chiedere conferma e non alterare inventario, ubicazione, prenotazioni o vendite in automatico.
- **Vista Affidamenti**: filtri per controparte, attivi/scaduti/chiusi, opere senza rendiconto, condizioni da chiarire. Riepiloghi salvabili come documento locale non fiscale, dopo anteprima dati.
- **Accessibilità**: wizard opzionale ma non obbligatorio; ogni passaggio raggiungibile da tastiera, campi con etichette esplicite, errori di formula espressi anche in testo e totali leggibili da screen reader.

## Regole di dominio e dati necessari (ipotesi, NON schema approvato)

- **Controparte**: UUID, nominativo/denominazione, ruolo dichiarato, contatti opzionali, note e riferimento documento. L'anagrafica **non deve sovraccaricare** il significato di `collectors`, oggi collegati alle vendite.
- **Accordo di affidamento**: UUID, riferimento, tipo (conto vendita, promozione/intermediazione, deposito commerciale), direzione (opera affidata a terzi / ricevuta da terzi), controparte UUID, inizio/fine, stato, condizioni testuali, eventuale allegato locale, timestamp e rettifiche.
- **Riga accordo–opera**: UUID, accordo, opera UUID, prezzo richiesto in centesimi EUR, base di calcolo della commissione esplicitata, percentuale in punti base o importo fisso, limiti sconti opzionali, storico delle variazioni. Lo stesso accordo può coprire più opere; un'opera può avere accordi successivi.
- **Riga rendiconto**: UUID, periodo, accordo, vendita UUID facoltativa, data del documento ricevuto, importo riferito dall'intermediario, commissione teorica e valore riconciliato/da verificare **distinti**, descrizione e fonte. Una vendita non va registrata due volte; gli importi non provati non vanno sommati a incassi/spese.
- **Invarianti**: no duplicazione dei record `sales`; eventuali restrizioni su più affidamenti commerciali attivi contemporaneamente sono avvisi con eccezione esplicita (agenti non esclusivi), non verità legali. Nessun passaggio da affidamento a «venduto» senza registrazione vendita. Saldi e formule basati su centesimi interi; arrotondamento dichiarato.
- **Storico**: rettificare una percentuale con una variazione datata, senza modificare retroattivamente rendiconti già finalizzati; conservare il documento sorgente e lo stato di verifica.

## Impostazioni, offline, privacy e lingue

- **Impostazioni**: modelli opzionali di condizioni, percentuale suggerita **non imposta**, aliquote mai presunte, campo per regola di arrotondamento da decidere; nessun account richiesto e nessuna categoria di galleria obbligatoria.
- **Offline**: tutte le operazioni su SQLite e allegati locali; nessun invio di rendiconti, firme, email, sincronizzazione o accesso a gestionali terzi. Collegamenti esterni restano testo, senza apertura automatica.
- **Lingue**: lingua del sistema, preferenza manuale IT/EN/ES/FR/DE e fallback inglese; tradurre ruoli, basi di calcolo, etichette «teorico/dichiarato/verificato», avvisi. I testi delle clausole personali non vengono tradotti automaticamente.
- **Privacy**: possibili contratti, dati finanziari, indirizzi e note riservate. Anteprima prima di qualunque stampa/export; per default non includere recapiti privati, prezzi di acquisto, costi di produzione né dati di altri clienti. Ricordare che i backup correnti non sono cifrati.

## Import/export e compatibilità

- **Export consultivo**: CSV UTF-8 con UUID, date ISO, EUR in centesimi e basis points, una riga per affidamento/partecipazione/rendiconto; separare esplicitamente valori teorici, vendite e incassi dichiarati.
- **Interoperabilità futura**: JSON versionato con riferimenti UUID, allegati locali in pacchetto esplicito e manifest, senza imporre un formato proprietario. Import solo con anteprima/validazione degli ID e rollback; attenzione ai cambi di termini e ai duplicati. Possibile integrazione futura con [TECH-001](https://github.com/Cesare-ml/Repertica-Studio/pull/16), non sua riscrittura.
- **Compatibilità**: nuove relazioni additive; archivi storici senza accordi restano validi, senza dedurre proprietari/commissioni dai campi `location`, `status` o dalle note. Backup completi e ripristino a cartella scelta devono comprendere le nuove informazioni. Nessuna alterazione di certificati emessi.
- **Piattaforme**: Windows, macOS e Linux; percorsi relativi e nomi non ASCII; assenza di dipendenze cloud, login o abbonamento.

## Complessità, rischi e alternative

- **Complessità**: media per accordi/lista e cronologia; alta per rendiconti multipli, cambi di condizioni e riconciliazione con vendite/incassi.
- **Dipendenze**: UUID opere/vendite, migrazioni SQLite, backup/media, economia in centesimi e i18n; eventuale UX-001 o Finance 2 solo come **integrazione facoltativa**, da progettare senza sovrapporre le fonti.
- **Rischi**: usare un «rendiconto stimato» come prova di pagamento; confondere possesso/mandato/proprietà; conteggiare una vendita due volte; commissioni diverse per base (lordo, netto tasse, sconti); produrre pseudo-contratti validi giuridicamente; esporre accordi riservati tramite dossier.
- **Mitigazioni**: termini contrassegnati come «dichiarati dall'utente», modello di calcolo visibile, nessun automatismo fiscale, anteprima privacy, cronologia e avvisi di conflitto, collegamenti manuali a vendite autorevoli.
- **Alternativa minima**: sola scheda accordo con controparte, opere, allegato e scadenza; niente calcolo commissioni e rendiconti prima di convalidare la frequenza d'uso. **Alternativa esterna**: foglio di calcolo offline legato a un archivio di contratti, senza integrazione ma senza costo di sviluppo.

## Criteri di accettazione ipotetici

1. Un affidamento per tre opere non cambia automaticamente stato, luogo, proprietario dichiarato, importi delle vendite o certificati.
2. Una percentuale del 30% su base dichiarata di 1.000,00 EUR mostra commissione **teorica** di 300,00 EUR e quota **teorica** di 700,00 EUR, senza scrivere entrate, uscite, incassi o fatture.
3. Due accordi per la stessa opera con periodi sovrapposti mostrano un avviso interpretabile, ma non inventano incompatibilità di legge.
4. Modificare una condizione registra la versione; un rendiconto precedente continua a mostrare la base/percentuale usata all'epoca.
5. Una vendita collegata una volta resta una sola vendita; un rendiconto non aumenta fatturato né «incassi effettivi» dichiarati.
6. Chiusura di un accordo e rientro fisico sono azioni distinte; annullare una procedura non altera l'archivio.
7. Export/import di prova conserva legami, percentuali e date senza duplicare rendiconti; backup/ripristino conserva allegati e storico.
8. Navigazione da tastiera e lettura screen reader funzionano offline in lingua OS, con fallback inglese, su piattaforme desktop supportate.

## Fasi di valutazione e sviluppo *ipotetico*, non autorizzato

- **A — Ricerca**: interviste a 3–5 artisti/gallerie; mappa dei modelli reali di conto vendita e identificazione del minimo indispensabile.
- **B — Essenziale**: anagrafica controparti leggere, accordi per opera, date, scadenze, file originali ed elenco; senza rendicontazione automatica.
- **C — Rendiconti**: calcolo esplicito commissione teorica con regole concordate, cronologia variazioni, PDF/CSV non fiscali e convalida manuale.
- **D — Portabilità**: import strutturato, casi multi-opera/multi-intermediario, compatibilità economica e test grandi archivi.

## Requisiti certi, ipotesi e domande al proprietario

- **Certi (da `main`)**: Tauri/SQLite offline, UUID stabili, registro vendita per opera, EUR in centesimi, incassi manuali separati, certificati storici immutabili, backup locale e lingue OS/fallback inglese.
- **Ipotesi da validare**: il conto vendita è frequente abbastanza da giustificare un modulo; gli utenti necessitano percentuali in app invece che in documenti; una parte dei casi ha mandati esclusivi.
- **Decisioni aperte**: il prodotto è rivolto anche a **gallerie che gestiscono opere di terzi**? Servono controparti «proprietario» distinte da collezionista? Come trattare base commissionale e IVA senza fingere conformità fiscale? I mandati non esclusivi sono comuni? È sufficiente una registrazione testuale nella prima versione?

**Nessuna idea approvata, nessuna implementazione o assegnazione.**
