# REPS-UX-041 — Titoli e descrizioni delle opere in più lingue, senza traduzioni presunte

- **ID stabile:** REPS-UX-041
- **Data della ricognizione:** 2026-11-22
- **Stato:** Proposta — da valutare
- **Filone:** Gestione e UX
- **Priorità proposta, soltanto ipotetica:** P2 per artisti/gallerie che preparano regolarmente materiali internazionali; P3 se le personalizzazioni nelle brochure sono sufficienti; P1–P2 solo dopo riscontri d'uso
- **Natura:** esclusivamente analisi e documentazione. Nessuna approvazione, assegnazione, modifica tecnica o autorizzazione allo sviluppo.

## 1. Problema reale possibile, persone interessate e riscontri su `main`

Un artista o una galleria che presenta le stesse opere a curatori italiani, francesi e inglesi può dover riscrivere ogni volta la descrizione in una nuova brochure. Nella stessa scheda archivistica il **titolo originale** può essere parte dell'identità dell'opera, mentre un titolo espositivo tradotto è una variante editoriale: non sono intercambiabili. La dispersione di testi in documenti diversi espone a versioni non aggiornate, traduzioni discordanti e perdita dell'attribuzione della traduzione.

**Utenti interessati da verificare:** artisti che partecipano a mostre estere, gallerie con cataloghi multilingue, studi e archivi con schede bilingui, curatori che preparano materiali informativi. Chi usa soltanto la propria lingua non deve vedere una nuova complessità obbligatoria.

**Fatti letti sul branch `main` il 22/11/2026, non prove desktop:**

- [Modelli Angular](../../../src/app/models.ts) e [modelli Rust](../../../src-tauri/src/models.rs): la scheda `Artwork` contiene un singolo `title` e una singola `description`, senza varianti per lingua. Il titolo è obbligatorio e identifica la scheda; la descrizione è facoltativa.
- [Schema SQLite](../../../src-tauri/src/db.rs): la tabella `artworks` conserva questi campi testuali, non emerge una tabella di traduzioni editoriali per opera.
- [Brochure offline](../../BROCHURES-OFFLINE.md), [editor](../../../src/app/brochures.component.html) e [renderer Rust](../../../src-tauri/src/brochures.rs): la brochure ha una lingua `it/en/es/fr/de`, ma la lingua cambia oggi le etichette del documento; il titolo e la descrizione dell'opera usano l'originale oppure `title_override`/`description_override` **specifici di quella brochure**. Un override vuoto torna al testo corrente della scheda, non a una traduzione salvata per lingua. Il documento BROCHURES-OFFLINE menziona già fra le evoluzioni possibili «campi editoriali IT/EN per singolo elemento»: questa scheda approfondisce il **riuso archivistico controllato** di quei testi, non propone un secondo editor PDF.
- [Esportazioni](../../DATA-PORTABILITY.md): il CSV `artworks` contiene il titolo e la descrizione correnti, senza un registro autonomo delle varianti linguistiche. Il backup integrale SQLite/media resta il mezzo di recupero completo.
- [Localizzazione](../../ECONOMY-I18N.md): la **lingua dell'interfaccia** è scelta dal sistema (con override opzionale) e fallback inglese. Non equivale alla lingua **del contenuto scritto dall'utente**.

Questi elementi indicano una possibile lacuna di riuso, **non** un bug comprovato, né una domanda già validata da utenti. Non è stata eseguita alcuna modifica o prova runtime.

## 2. Obiettivo e limiti

Studiare una piccola sezione facoltativa **«Testi per presentazione»** associata alla singola opera, dove un autore umano può conservare un titolo espositivo e una descrizione in una o più lingue, mantenendo **intatto** il titolo canonico e la descrizione attuale. Non significa tradurre automaticamente, sostituire un titolo storico, certificare la qualità linguistica o generare un nuovo certificato.

**Distinzioni necessarie:**

- **Titolo originale/canonico:** il `title` già presente, mai sovrascritto perché l'utente cambia lingua UI, prepara una brochure o aggiunge una variante.
- **Titolo di presentazione in lingua X:** facoltativo; per molti artisti coincide deliberatamente con l'originale anche nei documenti stranieri. Deve essere possibile lasciare il titolo non tradotto e tradurre soltanto la descrizione.
- **Descrizione editoriale in lingua X:** testo manuale, facoltativo, indipendente dalle note private e dalle descrizioni storiche conservate nei certificati.
- **Testo pronto o bozza:** non presumere che una frase inserita o importata sia stata controllata da un traduttore. Lo stato «pronto per condivisione» è solo una conferma editoriale manuale, non una certificazione.

## 3. Casi d'uso concreti

1. **Mostra a Parigi:** opera `REP-0042`, titolo originale «Senza titolo». L'artista mantiene il titolo come tale e crea una descrizione in francese, marcata «pronta». Nella brochure francese sceglie quel testo senza riscriverlo; il certificato già emesso resta immutabile.
2. **Catalogo bilingue:** galleria prepara prima brochure in italiano e poi in inglese con le stesse dodici opere. L'interfaccia indica 9 descrizioni inglesi pronte, 2 in bozza e 1 assente. L'utente decide consapevolmente se utilizzare il testo originale per quelle mancanti o tornare a completarle.
3. **Traduzione corretta a posteriori:** una descrizione inglese viene rettificata. La scheda conserva la nuova versione editoriale; le brochure **già esportate in PDF** fuori dall'archivio non vengono modificate. Una bozza brochure riaperta segnala, se possibile, che usa contenuti attuali differenti da una precedente esportazione, senza fingere uno storico che oggi non esiste.
4. **Titolo deliberatamente intraducibile:** una scultura ha un titolo poetico originale. Nessuna lingua aggiuntiva può sovrascriverlo o obbligare una traduzione. La stampa mostra il titolo canonico e un eventuale campo esplicativo distinto.
5. **Archivio migrato o monolingue:** per tutte le opere storiche non sono note le lingue dei testi. La sezione resta vuota; nessun testo viene etichettato automaticamente «traduzione inglese» sulla base della lingua del sistema.

## 4. Flussi UX e accessibilità (ipotesi progettuali)

**Fase minima, da verificare:** `Opere → Scheda → Testi per presentazione`. Per le schede senza varianti, un semplice link «Aggiungi una lingua», non cinque campi visibili per default. Selezione lingua del testo, campi indipendenti «Titolo per presentazione» e «Descrizione», indicazione «Titolo originale invariato», stato `bozza/pronto` per lingua e anteprima affiancata al testo canonico. `Salva` conserva soltanto la variante; `Elimina variante` mostra una conferma, non cancella l'opera.

**Uso dalla brochure, eventuale fase successiva:** la scelta `language` del documento resta distinta dalla lingua UI. Per ciascuna opera mostra «titolo originale / testo disponibile in lingua scelta / bozza non confermata / traduzione assente». Ordine editoriale **esplicito** da valutare: 1) override specifico della brochure se non vuoto; 2) variante in lingua scelta marcata pronta e selezionata; 3) testo canonico solo dopo indicazione di fallback e conferma per la pubblicazione. Una bozza di traduzione **non** viene promossa automaticamente come testo pronto. Il campo `include_description` esistente continua a governare se la descrizione appare. Cambiare lingua del documento non deve sovrascrivere gli override che l'utente aveva scritto.

**Controllo prima dell'esportazione:** riepilogo per opera della **lingua effettivamente usata** e di eventuali fallback; azione per correggere la scheda o usare espressamente il testo originale. Se una brochure salvata riferisce testi live, una nuova esportazione può essere diversa dalla precedente: rendere visibile questa caratteristica e valutare uno snapshot editoriale solo in una fase distinta.

**Accessibilità:** form interamente da tastiera, ordine del focus prevedibile, stato testuale oltre al colore, messaggi accessibili su varianti assenti/in bozza, campo linguistico identificabile dagli screen reader, lunghezze e overflow visibili senza perdere il testo; test al 200% di zoom e in finestre strette. Nessuna bandiera nazionale usata come unico indicatore della lingua.

## 5. Modello informativo e contratti descrittivi — NON implementati

Ipotesi minima di entità `artwork_presentation_text`:

| Campo | Scopo e vincoli ipotetici |
| --- | --- |
| `id` UUID, `artwork_id` UUID | Identità stabile; collegamento all'opera attiva, senza nuova opera |
| `language_tag` | Codice lingua stabile, per esempio `it`, `en`, `fr`, `es`, `de`; validare sintassi e normalizzare il codice senza tradurre contenuto |
| `display_title` opzionale | Titolo editoriale, anche vuoto per mantenere il titolo originale |
| `display_description` opzionale | Descrizione editoriale che l'utente scrive/approva; non aggiungere contenuti automaticamente |
| `readiness` | `draft` o `ready`; stato dichiarato dall'utente, non qualità linguistica verificata |
| `translator_credit` opzionale | Fonte/attribuzione testuale per uso interno o pubblico soltanto se scelta, senza raccolta obbligatoria di dati personali |
| `updated_at` | Data dell'ultima modifica locale; non attestazione di autenticità |

L'unicità `(artwork_id, language_tag)` eviterebbe duplicati involontari **nella fase semplice**; una cronologia di versioni potrebbe essere una fase futura e non va confusa con [TECH-010](https://github.com/Cesare-ml/Repertica-Studio/blob/docs/idee-strumenti-architettura-20261121/docs/idee/strumenti-e-architettura/REPS-TECH-010-storico-revisioni-metadati.md). Valutare se il valore vuoto significa «non definito» oppure «volutamente non mostrare», senza inferire silenziosamente l'intenzione. Con una traduzione importata in stato `draft`, non creare mai automaticamente `ready`.

**Operazioni ipotetiche, soltanto descrittive:** `list_presentation_texts(artwork_id)`, `save_presentation_text(artwork_id, language_tag, ...)`, `delete_presentation_text(artwork_id, language_tag)`, `preview_brochure_language_coverage(brochure_id)`. Non definiscono API approvate né richiedono strumenti nuovi. Nessuna API di traduzione esterna, nessuna scrittura tramite MCP e nessuna modifica ai contratti esistenti finché la proposta non venga eventualmente approvata.

## 6. Offline, multilingua, impostazioni, riservatezza

- **Offline e indipendenza:** storage locale SQLite, nessun account, login, abbonamento, servizio di traduzione, cloud o sincronizzazione richiesto. Nessun invio delle descrizioni a modelli IA o traduttori remoti; eventuale traduzione manuale viene incollata volontariamente dall'utente.
- **Lingua UI:** lingua di sistema con fallback **inglese** come già previsto; override utente preservato. Gli stati, gli errori e i comandi UI vanno localizzati almeno nelle lingue esistenti IT/EN/ES/FR/DE.
- **Lingua del testo:** scelta esplicitamente per ciascuna variante; la lingua dell'interfaccia **non** determina quali dati dell'opera siano riscritti o tradotti. Se manca una versione in una lingua, la UI deve dichiarare «testo non disponibile in [lingua]», non confondere il fallback inglese dell'interfaccia con una traduzione.
- **Impostazioni opzionali:** preferenza iniziale per la lingua dei nuovi documenti, distinta da quella UI; avviso «mostra testo originale quando manca una traduzione» eventualmente memorizzato come preferenza di anteprima, **non autorizzazione permanente a pubblicare un fallback**. Nessuna configurazione obbligatoria.
- **Privacy:** descrizioni e crediti dei traduttori possono contenere nomi, informazioni private, ubicazioni o riferimenti personali. L'esportazione verso terzi non include note private, clienti, prezzi, margini, indirizzi o percorsi locali; credito e testo vengono condivisi soltanto nell'output selezionato. Qualsiasi copia fuori dall'archivio è responsabilità dell'utente; non asserire che un PDF esportato sia un dato cifrato.

## 7. Import/export, backup, compatibilità e documenti storici

- **Portabilità completa:** nuove varianti e relativi UUID devono rientrare nell'archivio locale e nel backup/ripristino verificati con cartella liberamente scelta. Il ripristino non può convertire automaticamente bozze in testi pronti.
- **Export dati:** mantenere senza cambiamenti semantici il CSV `artworks` dei dodici registri esistenti (titolo e descrizione canonici). Valutare un **registro CSV separato** per varianti con UUID opera, tag lingua, testi, stato e credito, oppure JSON UTF-8 versionato per un futuro round trip. Se si decide un CSV, neutralizzazione formule, Unicode, semicoloni/CRLF e distinzione bozza/pronto devono essere verificati. L'importazione non è attualmente disponibile per questi registri: non promettere il round trip come funzione già presente.
- **Import eventuale:** file locale selezionato dall'utente, anteprima e mappatura per UUID opera; se l'UUID manca, mostrare una corrispondenza proposta per codice ma **non** applicarla automaticamente; scelta conservativa in caso di collisione lingua/record, nessuna sostituzione dell'originale, rollback all'annullamento. Non eseguire import da un file remoto.
- **Brochure e compatibilità:** gli override già salvati nelle brochure devono conservare il significato e prevalere sulle varianti riutilizzabili. I PDF storici e gli snapshot dei certificati restano immutati; la scelta della lingua della brochure non altera record di vendita o certificati. Un vecchio archivio privo di varianti continua ad aprirsi mostrando soltanto i testi canonici.
- **Qualità della stampa:** il renderer PDF corrente ha limiti **WinAnsi/Helvetica e overflow** documentati in [BROCHURES-OFFLINE](../../BROCHURES-OFFLINE.md). Una variante in altre lingue non garantisce quindi PDF leggibili: segnalare preventivamente glifi non rappresentabili e coordinare concettualmente la valutazione con [TECH-032](https://github.com/Cesare-ml/Repertica-Studio/blob/docs/idee-strumenti-architettura-20261121/docs/idee/strumenti-e-architettura/REPS-TECH-032-pdf-brochure-unicode-font-incorporati-offline.md), **senza modificarne i documenti**.
- **Conservazione output:** il PDF esportato nella cartella scelta rimane esterno al backup; le bozze brochure interne all'archivio sono coperte dal backup attuale. Non retrodatare una variante con la data di creazione dell'opera.

## 8. Confini anti-doppione con le 40 UX e le 36 proposte tecniche già presenti

- [UX-007](REPS-UX-007-dossier-presentazione-opere.md): selezione dei **campi** di un dossier e privacy. Qui si propone la **gestione editoriale riutilizzabile dei testi linguistici** nella scheda, anche senza creare dossier.
- [UX-018](REPS-UX-018-catalogazione-seriale-schede-bozza.md): inserimento in serie di opere, non traduzioni di opere esistenti.
- [UX-023](REPS-UX-023-prezzi-richiesti-listini-storico.md): listini e valori commerciali, esplicitamente **esclusi** dalle traduzioni.
- [UX-010](REPS-UX-010-diritti-riproduzione-crediti-autorizzazioni.md): diritti e permessi per contenuti, non attribuzione linguistica di testi editoriali; nessun diritto dedotto da un credito di traduzione.
- [TECH-030](https://github.com/Cesare-ml/Repertica-Studio/blob/docs/idee-strumenti-architettura-20261121/docs/idee/strumenti-e-architettura/REPS-TECH-030-vocabolari-locali-tecniche-supporti-interoperabili.md): equivalenze **controllate di tecniche e supporti**, non descrizioni libere tradotte.
- [TECH-032](https://github.com/Cesare-ml/Repertica-Studio/blob/docs/idee-strumenti-architettura-20261121/docs/idee/strumenti-e-architettura/REPS-TECH-032-pdf-brochure-unicode-font-incorporati-offline.md): resa **tipografica** dei contenuti nel PDF, non loro creazione o riuso.
- [TECH-001](https://github.com/Cesare-ml/Repertica-Studio/blob/docs/idee-strumenti-architettura-20261121/docs/idee/strumenti-e-architettura/REPS-TECH-001-pacchetto-dati-aperto.md): portabilità globale, che potrebbe trasportare anche questi testi se entrambe le idee fossero approvate. Non modificarne file o contratti.
- La funzione **brochure esiste già**: non si propone di ricostruirla né di duplicare `title_override` per un singolo documento. Non sono inclusi traduzione automatica/IA, editor grafico, workflow collaborativi, fatturazione o pubblicazione online.

## 9. Complessità, dipendenze, rischi e alternative

**Dipendenze plausibili:** identificativi UUID delle opere, SQLite e migrazioni additive, backup verificati, i18n attuale, anteprima brochure esistente; nessun servizio esterno. Per uso con documenti che contengono alfabeti diversi da WinAnsi, la resa tipografica è una **dipendenza funzionale da verificare**, non motivo per assumere TECH-032 implementata.

**Complessità:** bassa-media per campi editoriali aggiuntivi senza integrazione; media per varianti archivistiche, import/export e anteprima copertura; medio-alta per congelamento editoriale dei testi delle brochure, gestione collisioni, prove di resa Unicode e PDF. Evitare editor di traduzione a più utenti e approvazioni formalizzate nella prima fase.

**Rischi:** un titolo di traduzione sostituisce per errore l'originale; descrizioni provvisorie vengono pubblicate; documento in lingua selezionata contiene testi nella lingua sorgente senza avviso; brochure vecchie mutano contenuto alla rigenerazione; esportazione di note private; fallimento di font/impaginazione; sovrapposizione funzionale con override già sufficienti.

**Mitigazioni:** separazione canonico/editoriale, bozza non pubblicata per default, riepilogo lingua prima dell'export, priorità conservativa degli override, PDF già esportati immutati, preview e niente condivisione automatica, test con Unicode/RTL dove supportati e dichiarazione esplicita delle limitazioni.

**Alternative da preferire prima di implementare:** (A) continuare a usare gli override dentro ogni brochure; (B) aggiungere soltanto una **piccola libreria locale di testi di presentazione** da copiare manualmente, senza entità di traduzione per opera; (C) una descrizione bilingue nello stesso campo libero, ma con minore ricercabilità e maggior rischio di confusione. Se l'uso è sporadico, A è probabilmente sufficiente e questa proposta può essere scartata.

## 10. Criteri di accettazione ipotetici e fasi

1. Creazione di una variante `fr` su un'opera già esistente, con titolo canonico invariato e descrizione francese persistente dopo riavvio offline; nessuna variante artificiale per archivi precedenti.
2. Possono esistere descrizioni in due lingue con titolo originale uguale. Inserire una versione vuota non deve eliminare il testo originale né creare una «traduzione approvata».
3. Una variante `draft` non entra automaticamente in una brochure; una variante `ready` è selezionabile soltanto se la lingua del documento coincide o l'utente ne sceglie esplicitamente l'uso.
4. In brochure un `title_override` o `description_override` manuale prevale sulla variante dell'archivio; assenza di variante viene **segnalata**, non nascosta da un falso fallback «inglese».
5. Due brochure in lingue diverse riutilizzano la stessa scheda senza cambiare codice, titolo canonico, disponibilità, vendita, certificato o foto.
6. PDF già esportato non cambia dopo una rettifica; ristampa/rigenerazione da bozza mostra avviso dei contenuti mutati o congela contenuti con scelta esplicita, secondo fase approvata.
7. Nuovi testi/crediti compaiono nel backup e si ripristinano anche su disco esterno; export semantico eventuale preserva lingua, stato, UTF-8, UUID e nessuna colonna privata implicita.
8. Nessuna rete, invio a provider o traduzione automatica. UI in italiano, inglese, spagnolo, francese e tedesco con lingua OS e fallback inglese; il contenuto dell'opera non viene tradotto automaticamente. Test tastiera, lettore schermo, zoom e testi lunghi.
9. Se il renderer non può rappresentare una lingua/glifo, la pubblicazione deve avvertire chiaramente e non produrre un PDF che sostituisce silenziosamente caratteri con `?` spacciandolo per corretto.

**Fasi soltanto prospettate:** F0 verificare con artisti/gallerie quanto spesso ripetono le traduzioni e se bastano gli override → F1 campi editoriali facoltativi per opera, senza cambio export → F2 scelta consapevole delle varianti nelle brochure e avvisi di copertura → F3 export/import versionati e eventuali snapshot editoriali. Ogni fase richiede una **decisione autonoma del proprietario**.

## 11. Certezze, ipotesi e domande per la revisione personale

**Requisiti certi del prodotto:** gratuito e open source, Tauri, offline-first, nessun account/abbonamento richiesto, dati SQLite locali e sempre esportabili, backup/ripristino su cartella scelta, lingua UI di sistema con fallback inglese. Le brochure correnti hanno già lingua e override propri; certificati e vendite storiche non vanno alterati.

**Ipotesi non validate:** artisti con più brochure internazionali preferirebbero traduzioni riusabili rispetto alle personalizzazioni per documento; è sufficiente uno stato `bozza/pronto`; la manutenzione delle traduzioni non supera il beneficio del riuso. Nessuna intervista utenti, benchmark o prova desktop è stata svolta in questa ricognizione.

**Questioni da sottoporre al proprietario, senza presumere risposte:**

1. Quante brochure bilingui/multilingui vengono davvero preparate da un artista medio, e quante descrizioni identiche vengono riscritte?
2. Il **titolo tradotto** deve essere quasi sempre disattivato per preservare l'identità originale? È utile distinguere «titolo di esposizione» da «traduzione»?
3. Ha senso limitarsi inizialmente a IT/EN/ES/FR/DE oppure consentire tag linguistici aggiuntivi senza prometterne la resa PDF?
4. All'atto della ristampa, meglio riutilizzare testo **live** con avviso o congelare il contenuto editoriale per riproducibilità?
5. L'utente vuole riutilizzare i testi anche fuori dalle brochure (schede per gallerie, siti futuri) oppure gli override per singolo documento sono sufficienti?

**Nessun esito approvativo:** questa è soltanto una proposta da valutare. Non costituisce via libera allo sviluppo né modifica i documenti del filone tecnico.
