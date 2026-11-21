# REPS-UX-022 — Opere composte e distinta dei componenti fisici

- **ID stabile:** REPS-UX-022
- **Data:** 2026-11-21
- **Stato:** Proposta — da valutare
- **Filone:** Gestione e UX
- **Priorità ipotetica:** P2, eventualmente P1–P2 per gallerie e archivi che gestiscono installazioni e opere multipartite; da convalidare con utenti.
- **Natura:** studio esclusivamente documentale. Nessuna approvazione, assegnazione o modifica al software.

## 1. Problema professionale e destinatari

Un'installazione di dodici elementi, un'opera con pannelli smontabili o un complesso con struttura, supporti, alimentatore e contenitore può costituire **una sola opera catalografica e commerciale**, ma avere parti fisiche distinguibili e smarribili. Una scheda con una sola ubicazione, una fotografia principale e una descrizione libera non consente di controllare con affidabilità *quali componenti* devono accompagnare l'opera quando viene preparata, esposta, consegnata o riconsegnata.

Gli utenti più interessati sarebbero artisti che realizzano installazioni, scultori, gallerie con opere da montare e archivi che custodiscono pezzi separati. Per il pittore che tratta solo tele uniche la funzione dovrebbe essere invisibile o facoltativa.

**Riscontro verificato su `main` il 21/11/2026:** lo schema SQLite v6 e `src/app/models.ts` descrivono `artworks` con UUID, codice unico, titolo, dimensioni, ubicazione testuale, stato commerciale, fotografia principale e galleria; `sales.artwork_id` è unico e i certificati conservano uno snapshot immutabile. Non è presente una distinta strutturata dei **componenti fisici appartenenti a un'unica opera**. Il sistema di backup conserva DB e media, ma non può inventare le informazioni non censite. È una lacuna di modellazione per un sottoinsieme di utenti, non un errore accertato.

**Riferimento metodologico esterno, non requisito di conformità:** Collections Trust distingue la scheda di un oggetto o gruppo e l'identificazione dei pezzi separati; la scelta del livello di catalogazione dipende dalla politica dell'archivio. Si vedano [Numbering](https://collectionstrust.org.uk/resource/numbering/), [Cataloguing — suggested procedure](https://collectionstrust.org.uk/resource/cataloguing-suggested-procedure/) e [Cataloguing — the Spectrum standard](https://collectionstrust.org.uk/resource/cataloguing-the-spectrum-standard/). Repertica non diventerebbe per questo certificato Spectrum.

## 2. Confini per evitare duplicazioni

- **[UX-011 — Cicli creativi](REPS-UX-011-cicli-creativi-relazioni-opere.md):** raggruppa **opere autonome** anche vendibili separatamente (es. un dittico di due quadri indipendenti). Qui una singola opera **contiene parti non autonome**; la relazione è fisica e di completezza, non creativa. Se due pannelli sono due opere commerciali, usare UX-011, non questa distinta.
- **[UX-005 — Edizioni](REPS-UX-005-edizioni-multipli-serie-numerate.md):** collega esemplari prodotti in serie e vendibili singolarmente; qui non si moltiplicano copie o tirature.
- **[UX-015 — Ricognizioni](REPS-UX-015-ricognizioni-inventariali-discrepanze.md)** e **[UX-020 — Etichette](REPS-UX-020-etichette-inventariali-identificativi-scansionabili.md):** la distinta definisce **cosa ci si aspetta** dall'insieme; ricognizione e scansione potranno in futuro verificare o identificare **ciò che è stato visto**, senza sostituire il catalogo dei componenti.
- **[UX-001 — Provenienza e movimenti](REPS-UX-001-provenienza-movimentazioni.md)** e **[UX-004 — Rapporti di condizione](REPS-UX-004-rapporti-stato-conservazione.md):** spostamenti, custodia, interventi e condizioni datate restano processi propri; la distinta non registra trasferimenti né perizie.
- **[UX-017 — Vendite multiopera](REPS-UX-017-vendita-multiopera-ripartizione-importi.md):** vende più **opere** in un accordo; qui i componenti di un'unica opera **non** producono righe di ricavo, incasso o nuove vendite.
- **[TECH-016 — Allegati documentali](https://github.com/Cesare-ml/Repertica-Studio/blob/docs/idee-strumenti-architettura-20261121/docs/idee/strumenti-e-architettura/REPS-TECH-016-allegati-documentali-locali-multi-entita.md)** e **[TECH-001 — Portabilità semantica](https://github.com/Cesare-ml/Repertica-Studio/blob/docs/idee-strumenti-architettura-20261121/docs/idee/strumenti-e-architettura/REPS-TECH-001-pacchetto-dati-aperto.md):** possibili servizi trasversali, non da rispecificare o implementare in questa scheda.

**Regola di scelta da comunicare chiaramente:** «Il pezzo può avere vendita e certificato propri?» Se sì è probabilmente **un'opera autonoma** (con eventuali collegamenti UX-011); se no, può essere **componente** di un'opera. Una definizione materiale non prova da sola proprietà, autenticità o completezza effettiva.

## 3. Scenari d'uso

1. **Installazione con 12 parti.** L'artista cataloga una sola opera `INST-026`, aggiunge i dodici componenti con identificativi interni, nome, quantità e fotografia di riferimento. Prima dell'imballaggio legge la distinta ordinata e stampa una checklist locale; non viene creata una vendita per ciascuna parte.
2. **Scultura con elementi separabili.** Base, corpo, elemento luminoso e custodia sono previsti nella distinta. La base è temporaneamente assente: il catalogo specifica che il componente è *previsto*, ma non dichiara automaticamente il resto «smarrito» o «danneggiato». Un futuro controllo UX-015 potrà registrare l'osservazione.
3. **Restauro e riconsegna.** Un archivista confronta la distinta originale con ciò che è arrivato. Può annotare una discrepanza senza cambiare automaticamente ubicazione, proprietà, prezzo, certificato o status commerciale.
4. **Opera già venduta.** Un record storico contiene una vendita e un certificato. L'utente può documentare oggi la composizione con data/fonte dichiarata, ma l'app non riscrive i documenti immutabili e non presume che la stessa distinta fosse valida alla data della vendita.
5. **Due tele indipendenti in un dittico.** L'utente riceve un suggerimento per UX-011 anziché essere indotto a ridurre due opere commerciabili a componenti non vendibili.

## 4. Esperienza utente e flussi proposti

**Ingresso discreto:** nella scheda opera, pannello espandibile «Parti e componenti» con azione «Definisci distinta». Le opere correnti mantengono l'interfaccia normale; nessuna conversione automatica. Una breve guida mostra la differenza tra «opera composta», «edizione» e «ciclo di opere».

**Definizione:** inserimento manuale di righe ordinate con denominazione, identificativo breve/suffisso suggerito (modificabile), tipo (parte artistica, supporto, accessorio, imballo, altro), quantità prevista positiva, unità `pezzi`, note di identificazione, misure facoltative e una o più immagini già locali quando supportate. Per parti insostituibili e pezzi ripetuti distinguere **una riga con quantità** da **identità individuali**: l'utente sceglie la granularità; non nascondere pezzi separabili che necessitano di un codice proprio.

**Vista riassuntiva:** «12 parti previste» accanto a un elenco ordinabile e stampabile, ma **mai** «12/12 presenti» se non si è svolta una verifica. Evidenziare «distinta non verificata» o «rilevata il [data]» solo se esiste un'osservazione esplicita. Tasti «sposta su/giù», ricerca e navigazione da tastiera; i colori non sono l'unico segnale di anomalia.

**Checklist di preparazione:** possibile secondo passo con selezione, anteprima stampabile/PDF locale e campi da spuntare a mano. Un documento preparato non è prova di spedizione, di ricezione o di trasferimento della custodia. L'eventuale riconciliazione digitale va progettata **insieme a UX-015/UX-001**, non con una cronologia parallela di movimenti.

**Avvisi non distruttivi:** se manca un identificativo o se una riga è stata rimossa dopo emissione di un certificato, mostrare informazioni sulla storia e chiedere conferma. Il sistema non impedisce di vendere o catalogare un'opera storicamente incompleta senza che sia stata concordata una politica; non altera gli importi.

## 5. Ipotesi di dati e interfacce (solo descrittive)

- `artwork_component`: UUID stabile, FK `artwork_id`, `part_code` unico **all'interno dell'opera** (normalizzazione da precisare), `name`, `kind` codificato, `expected_quantity` intero positivo, `sort_order`, `description`, `dimensions` facoltative, `created_at/updated_at`.
- `component_source` (eventuale fase 2): riferimento/documento, data o intervallo dichiarato, autore della dichiarazione **se noto** e nota di incertezza; non trasformare congetture in fatti.
- `component_media_link` (facoltativo): riferimento a media custoditi nell'archivio; nessun percorso esterno arbitrario e nessuna duplicazione opaca di byte. Compatibilità concettuale con la proposta TECH-016, senza dipendenza obbligatoria.
- Regole: un componente appartiene a una sola opera; non possiede un proprio `artwork_id` vendibile, non è un `sale`, non emette un certificato indipendente; niente gerarchie ricorsive nella prima fase. `artworks.id/code` continuano a identificare **l'opera intera**. Un identificativo fisico tipo `INST-026-B` è utile ma non sostituisce l'UUID del componente.
- Modifica/rimozione: proposta di rettifica esplicita o segnalazione dell'impatto sui documenti storici; una futura cronologia delle revisioni andrebbe coordinata con TECH-010, senza inventare audit trail già presenti.
- API ipotetiche, **non contratti da implementare**: «elenca componenti di un'opera», «aggiungi o correggi riga con verifica di duplicati», «produci distinta a stampa», «seleziona componente per un'osservazione inventariale futura».
- Statistiche: conteggiare **una sola opera** per inventario commerciale e ricavi. «Numero di componenti» può essere un dato informativo separato, non un moltiplicatore delle opere in portafoglio. Il coefficiente artista e il suggerimento prezzo continuano ad agire sul record dell'opera; non sommatoria automatica dei pezzi.

## 6. Offline, multilingua, impostazioni e accessibilità

- **Offline-first:** DB SQLite e media interamente locali, funzionamento senza Internet, nessun account, cloud, API terze, telemetria o abbonamento. Backup/ripristino autosufficienti verso cartella scelta dall'utente restano obbligatori come principio.
- **Impostazioni eventuali:** mostra/nascondi pannello, convenzione per i suffissi suggeriti (senza rinominare record storici), modalità di stampa e granularità preferita per i pezzi ripetuti. Nessun obbligo di impostazioni prima della catalogazione ordinaria.
- **Lingue:** lingua di sistema con selezione locale facoltativa; IT/EN/ES/FR/DE e fallback inglese per lingue e stringhe non tradotte, secondo `docs/ECONOMY-I18N.md`. Tipi di componente memorizzati con codici stabili non tradotti; nomi e note libere restano nella lingua dell'utente. Misure e date presentate secondo locale senza conversioni o arrotondamenti distruttivi.
- **Accessibilità:** form associati a etichette, ordine di tabulazione e focus visibile, convalide spiegate in testo, tabella leggibile dagli screen reader, reordering anche tramite pulsanti e stampa ad alto contrasto. Niente interazione esclusivamente drag-and-drop o basata sui colori.

## 7. Importazione, esportazione, privacy e compatibilità

- **Export**: CSV UTF-8 per distinta con ID opera e ID componente, codice, nome, tipo, quantità, ordine e campi dichiarati; JSON versionato per relazioni e importazione *se* si adotterà uno scambio semantico come TECH-001. Le attuali esportazioni CSV di domini esistenti restano integrali e inalterate.
- **Import**: scelta esplicita del file; anteprima e mappatura delle opere, controllo codici/UUID duplicati e quantità invalide, confronto dei componenti già esistenti, conferma prima delle modifiche, transazione con rollback. Mai dedurre parti da titoli, fotografie o suffissi di codici, né sovrascrivere una distinta storica senza revisione.
- **Privacy**: la distinta può contenere ubicazioni, dettagli di sicurezza degli imballaggi, nominativi di prestatori o inventari di beni di alto valore. In una versione condivisibile escludere per default dati di contatto, indirizzi, ubicazioni e note riservate; mostrare anteprima dei dati effettivamente esposti. Non inviare nulla automaticamente.
- **Compatibilità**: migrazione eventualmente additiva su schema SQLite versionato; vecchi archivi senza componenti devono aprirsi senza conversioni o righe fittizie. Versioni meno recenti dell'app non devono promettere apertura di schemi più nuovi. I certificati emessi restano immutabili; se un futuro certificato contiene la distinta, usarne uno **snapshot** datato, non un link dinamico.
- **Backup**: ogni componente e relativo riferimento ai media va incluso nelle copie complete e verificabili; ripristino su altro computer anche senza il percorso originario. Prove con Unicode, numerosi componenti e supporti rimovibili.

## 8. Dipendenze, alternative, rischi e complessità

**Dipendenze necessarie solo in caso di futura approvazione:** modelli opere e UUID, convalide Rust/SQLite, UI locale i18n, esportazioni e backup. UX-015/020 e TECH-016 possono arricchire la distinta ma **non** sono prerequisiti per il censimento elementare.

**Alternative:** (A) descrizione libera e fotografie, costo minimo ma nessuna verifica strutturata; (B) trattare ogni pezzo come un'opera indipendente, utile **solo quando lo è davvero**, altrimenti falsifica numero opere, disponibilità e vendite; (C) foglio CSV esterno allegato manualmente, portabile ma soggetto a disallineamenti. Raccomandazione **solo da sottoporre al proprietario**: iniziare, se ritenuto utile, con una distinta semplice e non con un completo workflow logistico.

**Rischi:** confondere componenti con opere autonome, associare automaticamente la proprietà dell'insieme a ogni singolo pezzo, suggerire falsi stati di completezza, generare codici fisici ambigui, introdurre gerarchie e liste enormi, rendere instabili documenti storici, duplicare movimenti o inventario. Possibile compromesso: consentire righe di componenti senza obbligare a censire ogni vite, lasciando la granularità alla politica archivistica.

**Complessità stimata:** media per elenco strutturato, UX e migrazione; medio-alta per importazione bidirezionale, storico, documenti immutabili, collegamenti a inventario/movimenti e grandi installazioni. Nessuna stima temporale affidabile senza prototipo e casi reali.

## 9. Criteri di accettazione suggeriti, non autorizzazioni

1. Una tela normale resta catalogabile senza aprire la sezione componenti; il numero di opere, ricavi e certificati non cambia.
2. Una installazione di 12 parti genera **una** opera e dodici componenti ordinati; codici interni duplicati o quantità pari a zero vengono rifiutati senza perdere dati.
3. Elenco e stampa mostrano correttamente **previsti**, non dichiarano **presenti/verificati** senza una osservazione esplicita; nessuna spunta crea una vendita o un movimento.
4. Dopo la registrazione della vendita e l'emissione di un certificato, correzioni della distinta non modificano snapshots già emessi e non generano nuovi importi.
5. Backup su cartella esterna, restore su altro PC e round trip di export/import conservano UUID, quantità, ordine, codici e legami, senza duplicare componenti dopo reimportazione.
6. Un archivio storico senza componenti resta leggibile dopo un'eventuale migrazione; un archivio nuovo non viene aperto silenziosamente da versioni incompatibili.
7. Interfaccia fruibile da tastiera/screen reader, con fallback inglese; nessuna condivisione non richiesta di indirizzi o note.
8. Prova con almeno un caso di opera incompleta e uno di dittico di opere autonome: l'utente capisce la distinzione e non può vendere un componente come opera senza una scelta esplicita di nuova catalogazione.

## 10. Fasi possibili (tutte subordinate a decisione del proprietario)

- **Fase 0 — Validazione:** intervistare artisti che realizzano installazioni, una galleria e un archivio; raccogliere 5 distinte reali anonimizzate e misurare se i fogli esterni causano errori.
- **Fase 1 — MVP facoltativo:** distinta manuale, codici/quantità, ordine, vista e stampa locale; nessuna movimentazione, scansione, verifica automatica o impatto sui documenti commerciali.
- **Fase 2 — Scambio e prova fisica:** esportazione/importazione verificata, foto/riferimenti di parte e checklist con esiti datati **solo dopo** raccordo progettuale con UX-015/001.
- **Fase 3 — Professionale:** eventualmente storia delle rettifiche e documenti di allestimento/restituzione, separando sempre fatti osservati, istruzioni e dichiarazioni.

## 11. Certezze, ipotesi e domande da sottoporre al proprietario

**Requisiti certi del progetto:** gratuità/open source, desktop Tauri, dati SQLite locali, offline-first, nessun account/abbonamento obbligatorio, esportabilità completa, backup/ripristino in cartella scelta, lingua di sistema con fallback inglese, integrità di vendite e certificati storici. **Fatto di codice verificato:** allo stato di `main` non esiste una tabella `artwork_component`.

**Ipotesi da validare:** la distinzione componenti/opere ricorre abbastanza da meritare una UI; le quantità aggregate sono sufficienti per molti utenti; la stampa di una distinta riduce errori di allestimento; la priorità P2 è adeguata. Non sono state svolte interviste e non è stato osservato alcun malfunzionamento.

**Domande aperte:**
1. Il pubblico principale registra spesso installazioni, polittici **inscindibili** o oggetti con accessori? Quanti componenti per opera e quante movimentazioni separate?
2. Quando due pannelli hanno mercati e certificati propri, devono essere sempre due opere indipendenti collegate (UX-011)?
3. Si vuole una distinta puramente descrittiva o anche un verbale datato firmabile per consegna/ritiro? Quest'ultimo richiede regole separate e raccordo con UX-001/015.
4. Serve identificare individualmente ogni elemento intercambiabile o bastano quantità per gruppo? Quale livello di storico documentale è necessario?
5. Nelle esportazioni verso terzi quali dati di componenti e imballaggi devono essere esclusi come riservati?

**Decisione:** nessuna. Stato invariato **Proposta — da valutare**. Nessuno sviluppo può essere dedotto da questa scheda.
