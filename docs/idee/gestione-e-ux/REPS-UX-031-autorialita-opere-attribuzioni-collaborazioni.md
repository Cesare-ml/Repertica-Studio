# REPS-UX-031 — Autorialità delle opere: attribuzioni, collaborazioni e identità distinte

- **ID stabile:** REPS-UX-031
- **Data:** 2026-11-22
- **Stato:** Proposta — da valutare
- **Filone:** Gestione e UX
- **Priorità ipotetica:** P2; potenzialmente P1 solo se interviste a gallerie e archivi con più artisti confermano il bisogno
- **Natura:** esclusivamente analisi e documentazione. Non è un'implementazione, un'autorizzazione, un'approvazione né un incarico.

## 1. Motivazione, utenti e riscontri verificati

**Problema.** Un artista che archivia soltanto le proprie opere può ragionevolmente non compilare ogni volta il proprio nome. Una galleria che gestisce opere di più artisti, un archivio che censisce un lascito o uno studio che realizza lavori a più mani ha invece bisogno di distinguere **chi ha creato l'opera** da chi la possiede, custodisce, vende, fotografa o firma un documento. Una voce testuale «Autore» non equivale ad attribuzione verificata: in molti archivi l'identificazione è incerta, storica o controversa.

**Utenti:** artisti indipendenti e collaboratori (flusso minimo), piccole gallerie con più artisti, curatori, collezioni e archivi d'artista. La frequenza del bisogno nella base utenti Repertica non è stata misurata: **questa è un'opportunità di dominio, non un bug dimostrato**.

**Osservazioni su `main`, 22 novembre 2026** (ricognizione statica): `src/app/models.ts` e `src-tauri/src/models.rs` descrivono `Artwork`/`ArtworkInput` con titolo, tecnica, misure, foto e stato commerciale, **senza campo strutturato di autore/attribuzione**; `certificates.artist_name` è un testo inserito per il singolo certificato ed è parte del relativo snapshot storico. `Brochure.artist` è un testo del documento e non una relazione catalografica stabile. `studio_settings` e `finance.rs` espongono **un solo coefficiente artista globale**. Non si deve confondere questo assetto con una prova che il prodotto non serva alle gallerie. Riferimenti: [modelli Rust](../../../src-tauri/src/models.rs), [modelli Angular](../../../src/app/models.ts), [economia e localizzazione](../../ECONOMY-I18N.md), [MVP](../../MVP.md).

### Requisiti certi, ipotesi e vincoli di prodotto

- **Certi nel perimetro richiesto:** desktop open source gratuito, Tauri, nessun account/abbonamento, SQLite locale, funzionamento offline-first, dati completamente esportabili, backup verificati nella cartella scelta e lingua del sistema con fallback inglese. Conservare i record di vendita e i certificati già emessi.
- **Fatti del codice osservato:** identità UUID per le opere; un solo `artist_coefficient` nelle impostazioni; `artist_name` sul certificato; autore non presente nella scheda `Artwork`.
- **Ipotesi di prodotto da validare:** registrare una o più attribuzioni, con fonte e stato di certezza, può ridurre errori nei cataloghi che includono opere di artisti diversi. Per l'artista singolo il vantaggio potrebbe essere insufficiente.
- **Non sono certi:** validità delle attribuzioni storiche, titolarità dei diritti, quota di paternità, identità della persona oltre quanto dichiarato dalle fonti.

## 2. Confini e anti-doppione

| Elemento già documentato | Differenza di questa proposta |
| --- | --- |
| [UX-021 — Acquisizioni da terzi](REPS-UX-021-acquisizioni-opere-terzi-titolo-documentato.md) | Documenta il **titolo dichiarato di ingresso** nel patrimonio; qui solo la **creazione/attribuzione** dell'opera, indipendente dall'acquisto. |
| [UX-014 — Rubrica professionale](REPS-UX-014-rubrica-relazioni-professionali.md) | Documenta **contatti e relazioni di lavoro**. L'autore catalografico potrebbe collegarsi a un soggetto solo dopo scelta esplicita, senza rendere obbligatorio un recapito né usare automaticamente «collezionista» come autore. |
| [UX-010 — Diritti di riproduzione](REPS-UX-010-diritti-riproduzione-crediti-autorizzazioni.md) | La paternità dichiarata dell'opera **non** dimostra possesso, licenza o trasferimento dei diritti; il fotografo dell'immagine rimane una persona/ruolo distinto. |
| [UX-011 — Cicli creativi](REPS-UX-011-cicli-creativi-relazioni-opere.md) | Raggruppa **opere**, non definisce gli autori. |
| [UX-012 — Conto vendita](REPS-UX-012-affidamenti-conto-vendita-rendiconti.md) | Le percentuali economiche pattuite e la parte che vende **non** sono quote di autorialità. |
| [UX-023 — Prezzi e listini](REPS-UX-023-prezzi-richiesti-listini-storico.md), [UX-003 — Margini](REPS-UX-003-costo-e-margine-per-opera.md) | Un eventuale coefficiente per artista è solo **parametro del prezzo suggerito**, non prova d'autore né ricalcolo dei prezzi/vendite già registrati. |
| [TECH-001 — Pacchetto aperto](https://github.com/Cesare-ml/Repertica-Studio/blob/docs/idee-strumenti-architettura-20261121/docs/idee/strumenti-e-architettura/REPS-TECH-001-pacchetto-dati-aperto.md), [TECH-010 — Storico metadati](https://github.com/Cesare-ml/Repertica-Studio/blob/docs/idee-strumenti-architettura-20261121/docs/idee/strumenti-e-architettura/REPS-TECH-010-storico-revisioni-metadati.md), [TECH-024 — IPTC/XMP](https://github.com/Cesare-ml/Repertica-Studio/blob/docs/idee-strumenti-architettura-20261121/docs/idee/strumenti-e-architettura/REPS-TECH-024-metadati-fotografici-iptc-xmp-interoperabili.md) | Infrastrutture di portabilità, revisioni e **crediti della fotografia**; non definiscono gli attributi di autorialità dell'oggetto artistico. Nessun loro file va modificato. |

**Fuori ambito:** riconoscimento automatico da foto/firma, verifica di autenticità, perizie, certificazioni legali, calcolo dei diritti d'autore, gestione pagamenti/royalty, CRM completo, sincronizzazione a repertori online o una nuova modalità multiutente obbligatoria.

## 3. Scenari e casi difficili

1. **Artista singolo.** Crea una tela come oggi: il software non obbliga a compilare un'anagrafica autore. Se in futuro viene configurato un «autore abituale» locale, il valore è *proposto* e sempre visibile, non attribuito retroattivamente alle opere esistenti.
2. **Galleria con autori diversi.** Registra dieci opere di tre artisti, ritrova ciascun gruppo con ricerca per autore, prepara una distinta senza mostrare i contatti privati, non confonde i coefficienti individuali con il coefficiente globale di studio.
3. **Collaborazione.** Un'installazione è opera dichiarata congiuntamente di due autori. L'ordine di visualizzazione e i ruoli sono espliciti; nessuna percentuale di paternità o guadagno viene inventata.
4. **Attribuzione incerta.** Un dipinto antico è catalogato come «attribuito a X» con fonte dichiarata. La UI non mostra «opera autentica di X»; una seconda ipotesi rimane consultabile, senza cancellare la prima.
5. **Rettifica di catalogo.** Un'opera assegnata a X viene successivamente attribuita a Y sulla base di nuove fonti. La scheda corrente può cambiare con rettifica tracciata; **certificati e documenti PDF storici restano immutabili**, e un nuovo documento deve avvertire della discrepanza.
6. **Stesso nome, persone diverse.** Due artisti omonimi rimangono identità separate con UUID distinti. Il nome simile propone soltanto una verifica, non una fusione automatica.

## 4. Esperienza utente ipotetica

**Fase minima — scheda leggibile.** In **Opera → Catalogazione** si potrebbe aggiungere un campo facoltativo «Autore / attribuzione» e una distinzione semplice tra «dichiarato» e «non documentato». Per chi registra solo proprie creazioni, il campo rimane opzionale e non aggiunge schermate obbligatorie. L'alternativa ancora più prudente è testare prima una sola etichetta testuale senza nuove entità.

**Fase strutturata — identità.** Un'azione facoltativa «Collega autore» consente di cercare per nome/alias in un elenco locale, scegliere la persona giusta o crearne una senza dati di contatto obbligatori. Mostrare sempre identificativo di archivio e eventuale nota di disambiguazione; nessuna deduzione dell'identità da firme, nomi sui certificati o credit fotografici.

**Più attribuzioni.** Un elenco per opera con campi *autore*, *coautore*, *attribuito a*, *bottega/ambito di* oppure *non determinato*, ciascuno con fonte, livello di certezza, data della registrazione e ordine di presentazione. Alcuni ruoli possono essere **alternativi**, non cumulativi: «attribuito a X» e «attribuito a Y» non significa automaticamente «coautori».

**Anteprima e documenti.** Prima di creare un nuovo dossier/brochure/certificato, mostrare il testo di autorialità scelto e le eventuali incertezze. Il nome autore inserito sul certificato resta una scelta esplicita e congelata nella fotografia storica del documento; nessun aggiornamento in massa di certificati già emessi. Una modifica alle attribuzioni non aggiorna vendite, destinazioni delle quote, proprietari o custodi.

**Ricerca e accessibilità.** Filtri per autore, attribuzione incerta e «autore da completare»; risultati con nome **e ruolo**, mai affidati al solo colore. Combobox e selezioni utilizzabili da tastiera, navigazione screen reader, focus corretto dopo collegamento e avvisi comprensibili su omonimi. Non inserire un CRM nel flusso Nuova opera.

## 5. Dati e invarianti: modello soltanto descrittivo

**Opzione A, minima:** `artwork_creator_label` nullable o metadato dichiarativo simile, eventualmente fonte libera e stato `unrecorded/declaration/uncertain`. Non usare un nome testuale come identificativo globale né presumere che abbia valore legale. Una semplice stringa è facile ma rende difficili disambiguazione e più autori.

**Opzione B, condizionata a domanda reale:**
- `creator`: `id` UUID, `display_name`, `name_variants` facoltativi, note di disambiguazione, stato `active/merged_reference` soltanto se definita una procedura di unione verificabile, timestamp. Nessun requisito di nascita, indirizzo o recapito.
- `artwork_creator`: UUID, `artwork_id`, `creator_id`, `role_code`, `statement_kind` (autorialità dichiarata, attribuzione possibile, alternativa), `certainty_code` senza punteggi pseudo-scientifici, `source_reference`, `source_date` anche incerta, `display_order`, `recorded_at`, `notes_private`, eventuale intervallo di validità della dichiarazione.
- `creator_price_profile` **solo in una fase ulteriormente approvata**: identificativo autore, parametro di coefficiente e tariffa, data di efficacia e fonte; l'anteprima del prezzo mostra i parametri effettivamente scelti. Non è parte del modello minimo e non deve scrivere su vendite pregresse.
- Possibile `party_id` facoltativo se UX-014 sarà mai approvata: il collegamento richiede disambiguazione e consenso operativo esplicito. Una persona che figura come collezionista o fornitore non viene automaticamente trasformata in autrice.

**Invarianti:**
- Un'opera può avere **nessun autore registrato**, uno o più autori e attribuzioni alternative; `non registrato` è diverso da `anonimo`, `sconosciuto` e `contestato`.
- Nessuna migrazione deduce l'autore dal proprietario dell'app, dal nome inserito su un certificato, dalla firma `signed`, dalla brochure o dall'autore fotografico.
- Autori omonimi possono coesistere; la rimozione di un'autrice collegata deve essere impedita o gestita con una *disattivazione* senza rompere le relazioni storiche.
- Catalogo, vendita, ricevute, spese, ripartizioni, coefficiente di studio, prezzi registrati e snapshot dei certificati rimangono fonti autonome, mai modificate implicitamente.
- Una rettifica di attribuzione è **nuova dichiarazione con fonte**, non riscrittura della verità storica. L'interfaccia non deve trasformare in autenticazione giudizi di terzi.

## 6. Offline, impostazioni, multilingua, privacy

- **Offline:** ricerca, inserimento, confronto e revisioni sul database locale; nessuna chiamata a VIAF, Wikidata, IA, social, cloud o registri pubblici in background. Un eventuale identificativo esterno va introdotto solo tramite immissione manuale o import deliberato, senza dipendenza di rete.
- **Impostazioni:** abilitazione facoltativa di catalogazione multi-autore per archivi che ne hanno bisogno; autore abituale suggerito ma non applicato a tutti i record storici; lingua di interfaccia e preferenza per mostrare «attribuito a» nella lista. Un archivio monografico può restare semplice. Un eventuale profilo prezzo personale non sostituisce automaticamente la tariffa/coefficiente globale.
- **Multilingua:** lingua di sistema con fallback inglese; etichette e stati IT/EN/ES/FR/DE secondo `docs/ECONOMY-I18N.md`. Ruoli salvati con codici neutri, nomi, alias, citazioni di fonte e dichiarazioni restano nel testo originale; non tradurre automaticamente nomi d'autore o qualifiche inserite dall'utente. Valutare traduzioni precise di *attribuito a*, *cerchia di*, *collaborazione*, *autore non registrato*.
- **Privacy:** minimizzazione per persone vive; no date di nascita o contatti necessari alla scheda; note di disambiguazione e fonti private escluse per default da copie condivise. Non usare immagini di firme o dati di contatto per verificare identità. Backup correnti in chiaro: avvertire quando si copia in cartelle condivise.
- **Compatibilità:** Windows/macOS/Linux; Unicode e accenti nei nomi, ordinamento locale senza cambiarne gli identificativi; archivi esistenti apribili senza una migrazione che inventi autori. I backup di formato attuale rimangono ripristinabili tramite la versione compatibile; una eventuale estensione di schema richiede gestione esplicita delle versioni, non retrocompatibilità presunta con app vecchie.

## 7. Import/export, portabilità e collegamenti

- **Esportazione di lavoro:** CSV UTF-8 con righe di attribuzione, UUID opera e autore, ruolo, testo e fonte, distinguiendo autore certo/dichiarato/incerto; in un CSV di opere il nome sintetico deve avere una politica esplicita per attribuzioni multiple o alternative, senza perdere i dettagli nel formato completo.
- **Pacchetto strutturato ipotetico:** JSON versionato con `creators`, `artwork_creators`, `source_references` opzionali e UUID stabili; coerente concettualmente con TECH-001, non in sostituzione dei dodici registri CSV esistenti o del backup.
- **Reimport:** selezione manuale di file, anteprima conflitti e omonimi, mapping UUID, verifica relazioni, rollback transazionale e nessuna fusione per solo nome. Le attribuzioni alternative non si sommano; una seconda importazione identica è idempotente.
- **Backup/ripristino:** tutti i dati e gli allegati eventualmente approvati entrano nel backup completo verificato con selezione libera della destinazione; ripristino totale, non merge silenzioso. I PDF/CSV esterni già esportati non vengono automaticamente richiamati o modificati.
- **Condivisione:** anteprima degli autori che compariranno in dossier e brochure e nota di incertezza; nessun accesso aggiuntivo a MCP o pubblicazione remota per il solo fatto di registrare un'autrice. Un futuro ampliamento dell'MCP richiederebbe un esame separato dei dati esposti.

## 8. Dipendenze, complessità, rischi e alternative

- **Dipendenze esistenti:** `artworks`, UUID, documenti certificato immutabili, brochure, backup, export CSV, i18n e validazione locale. **Dipendenze opzionali, non approvate:** UX-014 (rubrica), UX-010 (diritti), TECH-001 (interoperabilità), TECH-010 (revisioni). Non sono richiesti servizi SaaS o una libreria di autorità esterna.
- **Complessità indicativa:** *bassa* per campo testuale opzionale e testo d'avviso; *media* per directory artisti e ricerca; *medio-alta* per attribuzioni alternative, coautori, versioni, conflitti import, prezzi per autore e riallineamento dei documenti futuri.
- **Rischi:** falsa certezza dell'attribuzione, confusione autorialità/proprietà/diritti, omonimi fusi, certificati storici riscritti, presenza accidentale di note private nei dossier, dati pregressi attribuiti senza fonte, coefficienti prezzo usati per dedurre ripartizioni economiche, complessità inutile per artisti singoli.
- **Mitigazioni:** campo opzionale, «attribuito a» e «non documentato» espliciti, fonti dichiarate, nessun matching automatico, snapshot immutabili e selezione manuale prima di stampare/esportare.
- **Alternative:** lasciare semplici note autore nella descrizione; introdurre soltanto il testo libero (A); elenco autori condiviso senza ruoli; mantenere archivi distinti per artista tramite futura gestione multiarchivio TECH-011. Le alternative riducono funzioni e rischio, ma possono rendere più difficili catalogazione con collaborazioni e dossier trasversali.

## 9. Criteri di accettazione e prove **ipotetiche**

1. Un archivio monografico storico si apre e resta utilizzabile senza alcun nome d'autore dedotto o obbligatorio.
2. Galleria con due autori omonimi: record distinti, selezione basata su UUID e disambiguazione; un salvataggio non li fonde.
3. Opera con due coautori compare una sola volta nell'inventario e nella vendita, ma riporta entrambi nei documenti nuovi quando selezionati; non genera due ricavi.
4. «Attribuito a X» non viene stampato come «di X» senza un'azione esplicita; l'incertezza rimane anche nelle uscite strutturate.
5. Modifica da X a Y non altera `certificates.snapshot_json`, PDF passati, importo vendita, scadenze/incassi, percentuali di ripartizione e note originali.
6. Il coefficiente di studio e i prezzi storici restano invariati; una simulazione per autore non cambia importi senza conferma separata.
7. Export JSON e reimport conservano UUID, ruoli, alternative, fonti e ordinamento; reimport duplicato non moltiplica le relazioni né altera fotografie.
8. Backup/ripristino da cartella liberamente scelta conserva tutti i collegamenti, anche su disco esterno scollegato/ricollegato e percorsi Unicode.
9. Nessun dato d'autore, fonte privata o contatto è inviato in rete; un dossier condivisibile non include per default note confidenziali.
10. Tastiera e screen reader permettono cercare, distinguere omonimi, correggere e salvare; IT/EN/ES/FR/DE con lingua OS e fallback inglese non cambiano i codici persistiti.
11. Una migrazione da archivi senza autori non mostra «autore certificato» né riusa automaticamente i nomi digitati sui certificati storici.
12. Le prove su Windows/macOS/Linux e archivi con migliaia di opere distinguono tempo di ricerca per autori e correttezza delle stampe; finché non condotte, nessun beneficio prestazionale è affermato.

## 10. Fasi future **non autorizzate** e domande per il proprietario

- **Fase A — validazione del bisogno:** 3–5 interviste con artisti, gallerie e archivi; osservare esempi anonimizzati di cataloghi con una o più firme, attribuzioni controverse e dossier già utilizzati. Verificare se un campo testuale sia sufficiente.
- **Fase B — minima UX:** campo facoltativo per autore/qualifica, export leggibile e avvertenza sulle attribuzioni non documentate, senza archivio autorità.
- **Fase C — relazioni strutturate:** se realmente necessario, directory autore con UUID, più autori/attribuzioni e selezione per scheda; revisioni delle fonti e gestione dei conflitti.
- **Fase D — integrazioni eventuali:** filtri trasversali, dossier multi-autore, eventuali profili di coefficiente artistico e forme di interoperabilità più sofisticate. Ogni fase richiede decisione distinta.

**Domande aperte:** (1) Repertica deve rimanere principalmente monografico oppure servire gallerie con artisti diversi nello **stesso archivio**? (2) È sufficiente «Autore/i» in testo libero o serve disambiguare identità e attribuzioni storiche? (3) Le collaborazioni richiedono ruoli granulari o basta elencare nomi? (4) Coefficiente artista per autore: esigenza reale o complessità superflua rispetto al parametro globale? (5) Per i documenti nuovi, quale terminologia deve accompagnare attribuzioni incerte? (6) Chi può correggere una fonte e come si deve mostrare la rettifica senza rivendicare autorità probatoria?

**Conclusione:** un solo studio approfondito, senza implementazione, account, nuove dipendenze, assegnazioni, issue operative o cambiamenti ai dati esistenti. Ogni decisione è riservata al proprietario.
