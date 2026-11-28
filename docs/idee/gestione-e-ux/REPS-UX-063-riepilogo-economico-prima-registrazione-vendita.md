# REPS-UX-063 — Riepilogo economico prima della registrazione di una vendita

- **Identificativo stabile:** REPS-UX-063
- **Data della proposta:** 2026-11-23
- **Stato:** **Proposta — da valutare**
- **Filone:** Gestione e UX
- **Priorità suggerita, non approvata:** **P2 condizionata** a prove con utenti; P3 o nessuna funzione se basta un riepilogo statico
- **Ambito:** controllo *prima* di registrare una vendita di una sola opera, non nuove regole economiche

## 1. Problema verificato nel prodotto e bisogno ancora da verificare

**Dato certo nel codice di `main` (23/11/2026):** il dialogo **Registra vendita** in `src/app/app.component.html` raccoglie opera, collezionista, importo, tasse, spese di spedizione, data, eventuali rate/incasso iniziale e note; il pulsante chiama direttamente `saveSale()`. Esiste inoltre la registrazione della vendita contestuale alla creazione di una nuova opera. Il motore Rust acquisisce transazionalmente vendita, dati economici, ripartizione corrente e piano di pagamento; l'opera passa a `sold`. Il piano delle quote configurato in Impostazioni è *congelato* sulla vendita: modificarlo in seguito non altera quella vendita (`src-tauri/src/allocations.rs`, `src-tauri/src/finance.rs`, `docs/ECONOMY-I18N.md`). In presenza di incassi registrati, la vendita non può essere eliminata direttamente; non va però descritta come irrevocabile in ogni circostanza.

**Osservazione circoscritta:** nel modulo di conferma non risulta un *riepilogo preventivo esplicito* del netto registrato e degli importi attribuiti alle singole quote che stanno per essere congelate. I campi e la configurazione delle percentuali sono disponibili in punti diversi dell'interfaccia. Non è stato svolto un test nativo né è stato segnalato da utenti un errore reale: **il bisogno è un'ipotesi da validare, non un bug confermato**.

**Utenti ipotetici:** artista autonomo che ripartisce il ricavo fra personale, attività e risparmio; atelier con più fondi; galleria che registra molte vendite (senza pretendere contabilità fiscale o commissioni di terzi non modellate). È più utile quando tasse/spedizione sono variabili o il piano percentuale è stato cambiato di recente.

## 2. Scenari professionali e risultati attesi

1. **Vendita da catalogo:** prima di salvare 1.500 €, con 100 € di tasse dichiarate e 50 € di spedizione, l'operatore vede **netto registrato 1.350 €** e l'eventuale ripartizione *effettiva* sulle quote attive.
2. **Vendita durante inserimento dell'opera:** la stessa verifica precede il salvataggio atomico di nuova opera + cliente eventuale + vendita; l'utente torna ai campi senza perdere fotografie e bozze se decide di correggere.
3. **Quote disattivate:** appare chiaramente “Nessuna ripartizione applicata a questa vendita”, senza mostrare 0 € come se fosse un fondo.
4. **Rate/acconto:** mostra a parte il piano rateale e l'incasso manualmente dichiarato, **senza trasformare le quote sul venduto in incassi realmente disponibili**.
5. **Piano modificato mentre il dialogo è aperto:** il riepilogo non deve rassicurare con percentuali vecchie; prima della registrazione va riletta e rivalidata la configurazione autorevole, con nuova conferma se il contenuto è cambiato.

## 3. Flusso UX proposto, in due forme alternative

### Variante A — minimo incremento di interfaccia (da provare prima)

Sotto i campi esistenti compare un pannello non modificabile “Riepilogo della vendita” con **opera/codice, collezionista, data, lordo, tasse, spedizione, netto registrato, percentuali attive e centesimi di ogni quota**. Se esiste un acconto, la sua indicazione è distinta e presenta la natura di registrazione manuale. Quando uno dei valori cambia, il pannello si aggiorna; anomalie e somme non valide sono spiegate accanto al campo e con un riepilogo accessibile. L'utente può premere il pulsante di registrazione già presente: nessuna finestra in più.

### Variante B — revisione prima della conferma (solo se A è insufficiente)

Il pulsante iniziale è “Rivedi vendita”; si apre un secondo passaggio con le stesse informazioni, **Modifica** per tornare ai dati e **Conferma registrazione** come unica azione che scrive. La fase di revisione non genera ID, non crea clienti, non riserva opere e non registra incassi. Prima della conferma si controlla di nuovo la validità e che piano quote/opera/collezionista non siano cambiati; se sono cambiati si torna al riepilogo aggiornato, mai una conferma automatica. Escape/Annulla mantengono la bozza finché il dialogo è aperto; la persistenza di bozze fra riavvii è **fuori ambito** (UX-018).

**Decisione ancora aperta:** un secondo passaggio può aggiungere frizione per chi registra molte vendite; verificare se basti il pannello A e una sola conferma contestuale. Evitare modali annidate.

## 4. Dati e regole da rappresentare (nessun nuovo schema richiesto nella fase minima)

- Riferimenti esistenti: UUID dell'opera e del collezionista, codici e nomi leggibili, `sold_at`; `amount_cents`, `tax_cents`, `shipping_cents`; piano corrente `allocation_settings`/`allocation_parts`; pagamento e rate già previsti dal form.
- Equazione informativa: `net_cents = gross_cents - tax_cents - shipping_cents`, **secondo le convenzioni gestionali correnti**, non profitto fiscale. Quote in basis point (10000 = 100%), centesimi interi; la quota finale assorbe l'eventuale resto di arrotondamento, esattamente come il motore Rust.
- La ripartizione deve riflettere la stessa logica che verrà congelata nella registrazione; **non creare una seconda formula JS divergente**. È un vincolo concettuale sul design, non un incarico tecnico.
- Se la ripartizione è disattivata, nessuna quota prospettata. Se il piano è invalido, il salvataggio resta impedito secondo la validazione backend già esistente; non sanarlo silenziosamente.
- Distinguere **prezzo suggerito da coefficiente artista**, **importo concordato registrato**, **netto gestionale** e **incasso dichiarato**: non derivare automaticamente sconti, margini o disponibilità bancaria. Prezzi suggeriti e prezzi richiesti rimangono in UX-023.
- Gli eventuali dati mostrati nel pannello sono temporanei, non un nuovo libro contabile. Se in futuro si desidera conservare il “visto” dell'operatore, servono una motivazione verificata e una scelta separata.

### Contratto di lettura ipotetico, soltanto descrittivo

Una futura preview read-only potrebbe accettare i campi del form ed esporre un valore tipizzato: `gross_cents`, `tax_cents`, `shipping_cents`, `net_cents`, `allocation_enabled`, `parts[{id,label,share_bp,amount_cents}]`, `warnings[]`, `plan_fingerprint`. **Non è una richiesta di endpoint/API né un vincolo di implementazione**. Anche con una preview tale risposta non autorizza la scrittura: il commit deve rivalidare quanto effettivamente salvato nella transazione.

## 5. Impostazioni, offline, multilingua e accessibilità

- Nessun account, abbonamento, servizio remoto, sincronizzazione, email inviata o automazione di pagamento.
- L'eventuale riepilogo usa solo SQLite locale e valori presenti nel dialogo; coerente con Windows/macOS e Tauri. La lingua segue le impostazioni già esistenti: lingua del sistema quando in automatico, selezione manuale possibile, **inglese di fallback**. Tradurre etichette e errori, non identificatori dei fondi rinominati dall'utente; formattare valuta/data per locale senza variare i centesimi archiviati.
- Tastiera e screen reader: ordine logico dei campi, titoli del riepilogo associati, errori annunciati con misura, focus preservato al ricalcolo, niente conferma basata solo su colore o icone. Verificare zoom elevato, dialogo ristretto, importi lunghi e lettura dei segni meno.
- Nessun aumento obbligatorio del numero di click se la variante A funziona; niente imposizione di percentuali a chi disattiva il riparto.

## 6. Privacy e portabilità

Il riepilogo contiene potenzialmente il nome del cliente e importi riservati: visibile **solo all'operatore**, senza copia negli appunti, notifiche di sistema, log, analytics o allegati. Non è una vista di presentazione pubblica (UX-054). Non esportare automaticamente un “preventivo” con dati del collezionista.

Gli archivi e i documenti esistenti non cambiano per il solo calcolo del riepilogo. La vendita confermata continua nei registri ordinari, nei **CSV integrali** e nei backup SQLite+media già disponibili, creati nella cartella liberamente scelta; eventuali esportazioni minimizzate sono trattate da TECH-025. Questa proposta **non introduce un nuovo formato** né sostituisce i backup verificati. In esportazioni future i valori devono mantenere unità centesimi/basis point e una distinzione netta fra previsioni non salvate e dati registrati.

## 7. Compatibilità, dipendenze, complessità e rischi

**Complessità ipotetica:** bassa–media per la variante A, media per la B; cresce se serve una preview autorevole condivisa fra backend e frontend e un controllo di coerenza al commit. Dipendenze concettuali: salvataggio vendite `commands.rs`, ripartizione `allocations.rs`/`finance.rs`, inserimento rapido opera `create_artwork_full`, UI dei piani rateali, i18n. **Nessun pacchetto o dipendenza esterna necessari come requisito.**

**Rischi:** differenze di arrotondamento fra anteprima e persistenza; quote cambiate durante la revisione; falsa interpretazione di “netto” come liquidità o utile fiscale; extra-click; presenza di clienti in condivisione schermo; grafica che nasconde un errore di tasse/spedizione; data di vendita in un fuso non pertinente; regressioni nell'inserimento combinato opera/vendita. Per mitigare, etichette rigorose, assert di equivalenza tra preview e snapshot finale, comportamento conservativo, nessuna contabilizzazione prima del commit.

**Alternative prima di qualsiasi sviluppo:** (i) procedura manuale: esportare le quote da Impostazioni e verificare importi con Calc; (ii) un testo informativo statico sulle quote congelate; (iii) pannello A senza seconda schermata. Raccogliere 4–6 prove con artisti/gallerie e dati inventati; misurare errori, tempo, comprensione di netto e incassi. **Se basta la sola comunicazione, nessuna funzione nuova.**

## 8. Criteri di accettazione proposti, non test eseguiti

1. Con 1.500 € lordo, 100 € tasse e 50 € spedizione, il riepilogo indica 1.350 € netti gestionali e la somma delle quote è **esattamente 135.000 centesimi**.
2. Con importi che generano resti di un centesimo e da 1 a 12 quote, somma e ordine di attribuzione coincidono con i record realmente salvati; nessun arrotondamento a virgola mobile presentato come affidabile.
3. Ripartizione disattivata, fondo storico inattivo, piano invalido e piano aggiornato mentre si rivede la vendita non producono conferme false.
4. Vendita diretta, nuova opera+vendita, cliente nuovo contestuale, piano rateale, acconto e vendita senza rate sono coperti con gli stessi concetti; l'anteprima non modifica SQLite né file.
5. Opera non più vendibile o collezionista rimosso: nessuna conferma di esito positivo, errore comprensibile e nessuna registrazione parziale. Backend ancora autorevole.
6. Dopo conferma, lo snapshot delle quote resta invariato anche se cambiano le impostazioni; non viene alterato il comportamento delle vendite storiche, dei certificati o degli incassi.
7. In tutte le cinque lingue previste, lingua di sistema/fallback inglese, tastiera, screen reader, zoom e formati numerici i valori mantengono stesso significato e centesimi.
8. Nessun dato personale o importo viene inviato a servizi esterni o scritto in log/telemetria; export e backup esistenti rimangono compatibili.
9. Prova osservativa: in 4–6 sessioni su scenari sintetici l'operatore comprende **prima della scrittura** cosa viene congelato; se la variante B non supera A, non adottarla.

## 9. Fasi possibili, senza assegnazioni

- **Fase 0 — ricerca:** interviste e prova cartacea/HTML statica sui due flussi; documentare casi in cui le quote non sono comprese. Nessuna modifica al prodotto.
- **Fase 1 — UX minima, solo se confermata:** variante A e terminologia economica coerente nelle due vie di registrazione; proposta futura separata da approvare.
- **Fase 2 — solo se necessaria:** conferma esplicita B e ricalcolo autorevole con gestione del cambiamento in corsa, senza nuove scritture o fondi.
- **Fase 3 — eventuale:** perfezionamenti accessibilità e casi grandi dopo prova nativa, mai obbligo di cloud.

## 10. Requisiti certi, ipotesi e domande al proprietario

**Certi (repo):** Tauri/SQLite offline-first, export CSV e backup completi, quote opzionali congelate per vendita, data e tasse/spedizione nel dialogo, rate e incassi manuali; nessuna fatturazione fiscale. **Certi (vincoli di prodotto):** gratuità/open source, nessun account/abbonamento richiesto, dati locali/esportabili, cartella backup scelta, lingua sistema/fallback inglese.

**Ipotesi non validate:** operatori commettono errori economici perché non vedono la scomposizione subito; il riepilogo riduce gli errori; hanno bisogno di un passaggio ulteriore anziché una riga esplicativa.

**Da valutare personalmente:** quanto spesso tasse e spedizione variano? Serve mostrare le quote *per fondo* o solo il netto? Il riepilogo deve comparire sempre o solo quando la ripartizione è attiva? Per chi crea opere e vendite insieme è accettabile una revisione in più? Basta un pannello immediato o si desidera una conferma distinta? Non presumere risposte o approvazione.

## 11. Anti-doppioni e fonti esaminate

- **UX-023** prezzi richiesti/listini: non tratta la scomposizione *della vendita che si sta registrando*.
- **UX-024** scadenze e rate e **UX-039** quota informativa sugli *incassi* dichiarati: non sono preview delle quote sul *venduto* prima del commit.
- **UX-017** vendita multiopera: progettazione di più opere in una vendita; qui solo una vendita di **una** opera, invariato il modello.
- **UX-059** controllo prima dell'emissione *di certificati*: evento e dati diversi; la vendita non è un certificato né sempre irrevocabile.
- **UX-003/050** costo per opera e rendiconto mensile: analisi post-registrazione, non controllo preventivo della transazione.
- **TECH-018/055/057** affidabilità e contratto degli export CSV, **TECH-033** salvataggio condizionale di schede obsolete, **TECH-020** accessibilità dei flussi critici: iniziative complementari, non cambiare i loro documenti.

Fonti consultate su **`main`** il 23/11/2026: `README.md`, `docs/ECONOMY-I18N.md`, `src/app/app.component.html`, `src/app/app.component.ts`, `src-tauri/src/commands.rs`, `src-tauri/src/allocations.rs`. Confrontati gli indici dei **62 studi Gestione/UX** e dei **58 studi tecnici** sui rispettivi branch documentali, senza modifica al secondo. Nessuna prova d'uso, test nativo o implementazione effettuata.

> **Decisione:** soltanto proposta di studio. Stato **Proposta — da valutare**, non approvata e non assegnata allo sviluppo.
