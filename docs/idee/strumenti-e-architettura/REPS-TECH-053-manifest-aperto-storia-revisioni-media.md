# REPS-TECH-053 — Manifest aperto della storia delle revisioni fotografiche del Media Center

- **ID stabile:** REPS-TECH-053
- **Data della ricognizione:** 2026-11-23
- **Stato:** **Proposta — da valutare**
- **Filone:** Strumenti e architettura; esclusivamente ricerca e documentazione
- **Priorità ipotetica:** **P2–P3**, soltanto dopo verifica di richieste effettive di trasferimento della storia fotografica; **nessuno sviluppo** se bastano gli export già presenti
- **Decisione:** nessuna funzionalità approvata, nessuna assegnazione, nessuna modifica al formato di backup o all'applicazione

## 1. Problema, destinatari e riscontri sul codice

**Bisogno da verificare.** Un artista che corregge la prospettiva di una fotografia di un dipinto può dover consegnare a fotografo, archivista o galleria **l'originale, il risultato finale e la relazione fra le versioni**, anche senza Repertica installato. Salvare due JPEG senza indicare a quale elemento appartengano, quali file siano stati selezionati e quale sia la revisione corrente può creare equivoci. Questo è particolarmente importante se la fotografia è allegata a schede e se versioni precedenti compaiono in documenti storici.

**Destinatari possibili:** studi che producono molta documentazione fotografica, fotografi d'arte, gallerie che ricevono immagini corrette, archivi di conservazione; per l'artista che consegna una sola foto finale il beneficio potrebbe essere nullo.

**Verificato sul branch `main` il 23/11/2026**, tramite [README](../../../README.md), [MVP](../../MVP.md), [DATA-PORTABILITY](../../DATA-PORTABILITY.md), [media.rs](../../../src-tauri/src/media.rs), [imaging.rs](../../../src-tauri/src/imaging.rs) e [db.rs](../../../src-tauri/src/db.rs):

- Il Media Center conserva già **oggetti fotografici e revisioni non distruttive**. `media_revisions` registra UUID della revisione, numero, tipo `ORIGINAL`/`EDIT`, percorso, dimensioni, byte, SHA-256, `operations_json` e data; `media_items` indica `current_revision_id`. L'editor conserva i vecchi file e, su nuova modifica, produce un file nuovo.
- `export_core` esporta **una** fotografia `current`, `original` oppure `custom` in una destinazione scelta, senza sovrascrivere. Il suo contratto non è un export della **storia intera** della fotografia.
- I **12 registri CSV** documentati non comprendono un registro delle revisioni `media_revisions`, con collegamento ai byte. Il **backup completo** contiene SQLite e media, ma è rivolto alla conservazione e al ripristino dell'intero archivio, non a una consegna selettiva e descrittiva a un destinatario.
- `imaging.rs` definisce crop, prospettiva, rotazione, riflessione e regolazioni, ma `operations_json` è un contratto **interno** di Repertica. Non esiste evidenza che altri programmi sappiano ripetere esattamente quelle operazioni, o che il risultato resti bit-identico fra versioni di motore, encoder e profilo colore.

**Limite della ricognizione:** non sono state eseguite prove desktop, misurazioni su media reali né interviste. Non è dimostrata alcuna perdita di revisioni o falla del backup. Si studia una **possibilità di interoperabilità mirata**, non un bug confermato.

## 2. Perimetro e confronto anti-doppioni

- **[TECH-001](REPS-TECH-001-pacchetto-dati-aperto.md)** progetta la **portabilità semantica di tutto l'archivio** e un eventuale reimport: questo studio riguarda **un singolo elemento Media Center scelto** e un pacchetto **consultabile, senza import automatico**. In caso di futura convergenza riutilizzare nomenclatura e verifiche, non creare due formati incompatibili.
- **[TECH-024](REPS-TECH-024-metadati-fotografici-iptc-xmp-interoperabili.md)** studia IPTC/XMP, autori e diritti **incorporati o di accompagnamento alla fotografia**. Qui l'oggetto è il **rapporto verificabile tra byte di versioni diverse**; IPTC/XMP può essere opzionale, non un sostituto del manifest.
- **[TECH-021](REPS-TECH-021-gestione-colore-icc-derivati-fotografici.md)** è dedicata alla fedeltà cromatica; **[TECH-013](REPS-TECH-013-copie-fotografiche-condivisibili-privacy-metadati.md)** alla condivisione di copie a basso rischio; **[TECH-003](REPS-TECH-003-diagnostica-integrita-media.md)** alla diagnostica e al recupero nell'archivio attivo; **[TECH-052](REPS-TECH-052-verifica-indipendente-backup-formato-v1.md)** alla verifica di backup completi. Nessuna funzione viene duplicata o assunta approvata.
- Nel filone **[Gestione e UX — PR #14](https://github.com/Cesare-ml/Repertica-Studio/pull/14)**, UX-010 riguarda permessi e crediti, UX-026 il **processo creativo dell'opera**, UX-057 i **master costitutivi di opere digitali**. Questo documento non crea un diario di lavorazione, non gestisce proprietà o diritti e non cambia le schede d'opera.
- **Fuori ambito:** pubblicazione su web, AI o riconoscimento visivo, sincronizzazione, condivisione automatica, modifica dei documenti emessi, ricostruzione garantita da ricette, import delle revisioni da pacchetti terzi e nuovi flussi per vendite/clienti.

## 3. Scenari e flussi UX soltanto proposti

**A — Consegna a una galleria.** Nel Media Center l'artista seleziona una fotografia con originale, una correzione prospettica e un ritaglio. Una vista **«Prepara storia delle versioni»** mostra l'originale, la revisione corrente e le altre revisioni disponibili con miniatura, data e dimensioni. L'artista seleziona i file desiderati; un riepilogo esplicito dichiara cosa sarà esportato e che **non contiene dati gestionali**. Sceglie una cartella locale; si crea una **nuova sottocartella**, con immagini selezionate e manifest leggibile da altri programmi. Nessun invio a terzi.

**B — Conservazione fuori da Repertica.** L'archivista apre `manifest.json` e i file con normali strumenti, controlla SHA-256 e riconosce **quale file è originale, quale è derivato e quale era corrente al momento della preparazione**. Le operazioni annotate sono descrittive e non necessarie per **aprire i file già generati**.

**C — Rilievo di un file alterato.** Se un file sorgente manca o il suo SHA-256 reale non coincide con quello archiviato, l'app **non produce un pacchetto apparentemente integro**: distingue file mancante, byte modificati e versione non inclusa; presenta una decisione di annullamento o ripetizione, senza correggere arbitrariamente il database.

**Esperienza dettagliata ipotetica:** selezione di un elemento singolo → anteprima revisioni e relazioni → scelta delle versioni (default da valutare: originale e corrente, non necessariamente tutta la storia) → schermata dati da includere/escludere → cartella di destinazione → verifica e esportazione atomica in sottocartella nuova → rapporto con conteggi, hash verificati, errori e percorso apribile. Comandi accessibili da tastiera e lettore di schermo; nessuna esportazione massiva implicita.

## 4. Specifica descrittiva di dati e API (NON approvata)

**Formato candidato:** cartella autocontenuta, con `manifest.json` UTF-8 e `files/` che ospita i file binari selezionati **senza ricompressione né alterazione**. Evitare percorsi assoluti e nomi di filesystem provenienti da input non attendibili; i nomi nel pacchetto sono stabili e validati. Un formato testuale versionato è preferibile a un PDF come **unica** fonte della cronologia.

Schema **concettuale**, non schema approvato:

- `format`, `format_version`, `exported_at_utc`, `source_application_version` (quando verificabile);
- `item`: identificativo di pacchetto, titolo e descrizione **solo se scelti**, originale e revisione corrente al momento dell'export;
- `revisions[]`: ID locale o pseudonimo dichiarato, numero, tipo, `relative_file`, `bytes`, `sha256`, dimensioni pixel, `created_at`, `derived_from` **solo se ricavabile realmente**, indicatori `selected` e `current_at_export`;
- `processing_notes` opzionali: tipo e parametri delle operazioni annotate, versione del vocabolario interno e avvertenza che **non sono una ricetta interoperabile riproducibile**; non esportare identificativi di operazioni sconosciute fingendo di comprenderli;
- `verification`: algoritmi, esito della verifica dei byte nella preparazione ed eventuale versione del formato. Gli hash misurano identità dei byte, **non autenticità, data certa, integrità morale dell'opera o titolarità di diritti**.

**Attenzione alla semantica delle revisioni:** nel codice la nuova revisione viene generata a partire dalla **corrente in quel momento**; la numerazione cronologica non prova da sola una catena lineare di derivazione dopo eventuali ripristini di revisione. Non dedurre `derived_from = n-1` senza una relazione realmente persistita. In fase minima si può omettere la genealogia e dichiarare soltanto il tipo, l'ordine cronologico e lo stato corrente.

API puramente illustrative: `previewMediaHistoryPackage(mediaId, selectedRevisions, disclosureOptions) -> Plan`; `verifyMediaHistoryPlan(plan) -> Checks`; `exportMediaHistoryPackage(plan, chosenDirectory, cancellation) -> Report`. Il piano non deve autorizzare scritture nell'archivio, né inviare immagini a un processo MCP o a un servizio online.

## 5. Offline, sicurezza, privacy e prestazioni

- Tutte le operazioni avverrebbero localmente, senza account, abbonamenti, telemetria, cloud obbligatorio o aggiornamento di servizi esterni. Destinazione scelta liberamente, anche USB/NAS montato; il pacchetto non è un backup e **non** deve essere collocato automaticamente dentro `media/`.
- I file possono includere persone, firme, ubicazioni, targhette e metadati originali. Anteprima e conferma esplicita per ogni contenuto; **no export per default** di collezionisti, contatti, vendite, note private, percorsi assoluti, registro completo degli usi o riferimenti storici ai certificati. Rimuovere EXIF/IPTC, se voluto, **modifica i byte**: l'originale non va chiamato originale se trasformato. Per condivisione sicura rimandare a TECH-013.
- Percorsi relativi normalizzati, no symlink o traversal, file regolari verificati, limiti di memoria e dimensione, controllo degli hash su copia coerente; staging su destinazione e pubblicazione solo a esito positivo. In caso di annullamento o disconnessione del disco, non lasciare un manifest che dichiari integro un pacchetto incompleto.
- Leggere immagini e dati con un piano coerente: il Media Center potrebbe essere modificato mentre avviene l'export. Il piano deve rilevare revisioni eliminate o cambiate prima della finalizzazione; evitare lock di lunga durata che blocchino attività normali.
- Non promettere replay bit-identico di correzione prospettica o colori su software esterni. **I byte degli output esportati** sono il riferimento; eventuali profili ICC e la loro resa richiedono verifica separata (TECH-021).

## 6. Impostazioni, multilingua, accessibilità, import/export e compatibilità

- Eventuale impostazione **solo per questa esportazione**, non un nuovo account/profilo globale: selezione delle revisioni e disclosure di descrizioni/crediti, con default conservativi da validare. Non introdurre un automatismo periodico.
- Etichette nella lingua di sistema con fallback inglese, anche in assenza di traduzioni aggiuntive; tradurre interfaccia e messaggi, non alterare titoli/nomi digitati. Nel manifest le **chiavi restano stabili e neutrali rispetto alla lingua**, mentre l'eventuale README umano può essere localizzato.
- Supporto tastiera, ordine focus e annunci screen reader per numero versioni, scelta, avanzamento, esito, hash differenti; non fondare il riconoscimento della versione unicamente sul colore o sulle miniature.
- Compatibilità con Windows, macOS e Linux, file UTF-8 e nomi multipiattaforma. Un pacchetto deve rimanere **leggibile senza Repertica**, senza librerie o codec proprietari obbligatori *per interpretare il manifest*; leggere un particolare file immagine resta responsabilità del programma destinatario.
- **Non sostituisce** i CSV esistenti, il backup completo, l'esportazione corrente del singolo file, né introduce un import o una sincronizzazione. Un software terzo può ignorare `processing_notes` e limitarsi a leggere file, dimensioni e hash.

## 7. Dipendenze, alternative, rischi e complessità

**Dipendenze eventuali:** modello corrente Media Center e SHA-256; regole di export sicuro già presenti; possibile riuso futuro del formato aperto TECH-001. Non occorre un servizio web né si propone una dipendenza software oggi.

**Prima alternativa da testare:** usare **Export originale**, **Export corrente** e una breve nota Markdown manuale con hash calcolati con strumenti OS. Provare 4–6 consegne reali con fotografo, galleria e archivista. Se hanno bisogno solo dell'immagine finale, **nessuno sviluppo**. La storia interna è comunque conservata dal backup completo esistente.

**Rischi:** confondere la cronologia software con una prova di autenticità; esporre metadati sensibili o diritti non verificati; promettere la ripetibilità degli interventi pur senza informazioni di versione; aumentare inutilmente lo spazio con copie di revisioni; esportazioni incomplete su dischi instabili; nomi file incompatibili; una falsa ricostruzione della genealogia quando l'utente ha ripristinato una revisione.

**Complessità indicativa:** **media** per export selettivo di file e manifest con verifica coerente, **medio-alta** se si desiderano genealogia storica precisa, opzioni di privacy approfondite o lettura da software terzi. La genealogia non va inserita retroattivamente con supposizioni.

## 8. Test e criteri di accettazione ipotetici

Se e solo se approvata in futuro, considerare superata la prova quando:

1. Tre versioni dello stesso item, compresi originale, edit e una revisione ripristinata, sono distinguibili senza aprire Repertica; la revisione corrente al momento dell'export è marcata correttamente, senza inventare il parent.
2. L'hash reale di ciascun file esportato corrisponde al manifest; un file mancante/cambiato produce errore esplicito, mai successo falso; nessuna modifica a SQLite, revisioni, certificati e backup.
3. Export annullato, destinazione occupata, USB scollegata e spazio esaurito non sovrascrivono file esistenti e non pubblicano un manifest che attesti un'operazione incompleta.
4. Titoli con accenti, caratteri non latini e nomi potenzialmente pericolosi funzionano nelle tre piattaforme; il manifest resta validabile con parser JSON comuni.
5. Un lettore esterno può aprire manifest e immagini con strumenti normali. Eseguire test con immagini di grandi dimensioni e molte revisioni, senza uso di memoria illimitato.
6. Campi privati non selezionati non compaiono in manifest, percorsi o nomi dei file; informativa che la copia può contenere metadati incorporati anche senza campi gestionali.
7. Collaudo con tastiera/screen reader, lingua di sistema e fallback inglese. Test di interpretazione degli hash vs prova legale: l'interfaccia non descrive il pacchetto come autenticato.

Nessun test è stato eseguito durante la ricognizione documentale.

## 9. Priorità, fasi ipotetiche e domande per il proprietario

**P2 soltanto se le consegne di più versioni ricorrono realmente**, altrimenti **P3 / nessun intervento**. Sequenza eventuale: **Fase 0** interviste e pacchetti manuali per misurare il bisogno; **Fase 1** specifica e fixture sintetiche in documentazione; **Fase 2** eventuale export di originale + corrente con manifest hash; **Fase 3** scelta di revisioni aggiuntive e note operative, solo se richieste; **Fase 4** possibili strumenti di lettura esterni senza dipendenze imposte. Sono ipotesi, non attività assegnate.

**Questioni aperte per revisione personale:**

- I destinatari hanno bisogno davvero della **storia**, oppure basta il file finale e una copia dell'originale?
- È preferibile condividere identificativi UUID locali, oppure pseudonimi temporanei per ridurre possibilità di correlazione?
- Serve un `README.md` testuale nel pacchetto, o basta un JSON semplice con schema pubblico?
- Si vuole conservare soltanto il contenuto dei file o anche le annotazioni delle operazioni interne, chiaramente non portabili?
- In una selezione di revisioni non consecutive, come mostrare senza ambiguità ciò che **non** è stato consegnato?
- Quali licenze/consensi regolano la consegna a terzi? Nessun diritto può essere dedotto dai dati nel Media Center.

**Requisiti certi:** archivio locale, indipendenza, export completo e backup esistente, multilingua con fallback inglese, niente account/cloud obbligatorio. **Ipotesi da validare:** frequenza di consegne multi-revisione, utilità del manifest, formato finale e possibili opzioni di privacy. **Decisioni riservate al proprietario:** priorità, eventuale roadmap, scelta dell'implementazione o scelta di non intervenire.
