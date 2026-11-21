# REPS-TECH-023 — Certificati d'opera in PDF/A verificabile, generati da snapshot storici

- **ID stabile:** REPS-TECH-023
- **Data ricognizione:** 2026-11-21
- **Stato:** **Proposta — da valutare**
- **Filone:** strumenti e architettura; **sola analisi e documentazione**
- **Priorità ipotetica:** **P2**, da rivalutare con gallerie, archivi e artisti che conservano i certificati per molti anni
- **Decisioni:** nessuna soluzione, libreria, profilo PDF/A o implementazione approvati; nessun incarico di sviluppo

## 1. Motivazione, destinatari e fatti verificati

Un certificato di autenticità deve restare consultabile e stampabile anche dopo il cambio di computer, sistema operativo o software. Per archivi e gallerie è importante consegnare un documento riproducibile con un contenuto coerente con la **scheda al momento dell'emissione**, senza dipendere dalla stampa PDF di un browser specifico. L'obiettivo non è provare l'autenticità dell'opera: è migliorare **portabilità e conservazione del documento digitale**.

**Fatti su `main` consultato il 21 novembre 2026:**

1. [`docs/MVP.md`](../../MVP.md) indica esplicitamente che i certificati vengono stampati tramite il sistema/browser, e non esiste ancora un'esportazione PDF/A certificata. I certificati hanno seriale univoco, emittente, data, firma a mano e **snapshot immutabile** dell'opera.
2. [`src/app/app.component.ts`](../../../src/app/app.component.ts) espone `printCertificate`, che usa `window.print()`; [`src/app/app.component.css`](../../../src/app/app.component.css) contiene il layout di stampa A4. Il risultato dipende dal motore e dalle impostazioni di stampa del sistema: **non è corretto chiamarlo PDF/A** senza una generazione e validazione dedicate.
3. [`docs/DATA-PORTABILITY.md`](../../DATA-PORTABILITY.md) esporta un registro CSV `certificates` con `snapshot_json` originale e precisa che `current_artwork_title` è il **titolo attuale**, non necessariamente quello all'emissione. I backup completi conservano anche i riferimenti delle immagini storiche degli snapshot validi.
4. [`docs/RECEIPT-ATTACHMENTS.md`](../../RECEIPT-ATTACHMENTS.md) dimostra già una politica prudente di esportazione di file: percorso scelto dall'utente, no sovrascrittura, verifica dei dati e protezione della directory dell'archivio. È un riferimento di comportamento, **non** un generatore PDF.
5. La [PR #14 — Gestione e UX](https://github.com/Cesare-ml/Repertica-Studio/pull/14) comprende dossier di presentazione (UX-007), diritti/crediti (UX-010) e istruzioni tecniche (UX-025), ma non un contratto tecnico di **conformità PDF/A dei certificati immutabili**.

**Ipotesi da verificare:** artisti che consegnano molti certificati digitali e archivi con obblighi di deposito potrebbero beneficiare di un documento PDF/A validato. Per chi stampa e firma solo su carta, il flusso corrente potrebbe essere sufficiente. **Non è stato rilevato né misurato un difetto nei PDF prodotti dagli utenti.**

## 2. Confini e non duplicazione

- **TECH-001** progetta un *pacchetto aperto* di dati e la reimportazione dell'archivio; qui si studia il **singolo certificato leggibile autonomamente**, senza includere il database né promettere reimportazione.
- **TECH-018** studia CSV coerenti/atomici; qui il documento è PDF e deve superare criteri di conformità specifici. Possibile riuso futuro delle politiche di salvataggio, non sostituzione dell'esportatore.
- **TECH-020** studia tastiera e screen reader nell'app; qui serve valutare anche **accessibilità del PDF** (testo selezionabile, ordine di lettura), senza dichiarare automaticamente conformità PDF/UA.
- **TECH-013/021** trattano privacy dei derivati e gestione ICC delle fotografie; non equivalgono alla preparazione completa di un PDF/A. Le immagini del certificato richiederebbero comunque decisioni coordinate sui profili colore.
- **UX-007** riguarda il dossier commerciale selezionabile; un certificato **storico già emesso** non va rigenerato con prezzi, titoli, destinatari o fotografie correnti. **UX-010** riguarda autorizzazioni/crediti, non la conformità PDF/A.
- Non sono previsti firme elettroniche, identificazione di chi firma, marcature temporali, verifica legale dell'autenticità, autenticazione pubblica del seriale, invio email, stampa remota o registrazione fiscale. **PDF/A non equivale a firma né prova di autenticità.**

## 3. Destinatari e scenari d'uso

- **Artista:** stampa il certificato e, se necessario, salva un duplicato digitale per inviarlo a un acquirente; vuole verificare che la versione digitale corrisponda allo snapshot di emissione.
- **Galleria:** archivia copie elettroniche e le riapre su Windows/macOS/Linux dopo anni, senza che cambino i caratteri, le proporzioni o il numero seriale.
- **Archivio/studio:** deposita documenti di centinaia di opere presso un conservatore o in una cartella liberamente scelta; necessita di un'esportazione ispezionabile e di un esito di verifica, non soltanto di un'etichetta “PDF/A”.
- **Restauro/ricatalogazione:** l'opera cambia titolo, fotografia o ubicazione; il documento esportato **mantiene** il contenuto storicizzato nell'emissione, non i campi aggiornati della scheda.
- **Caso degradato:** la fotografia storica è mancante o lo `snapshot_json` è malformato; l'app deve **segnalare il problema**, non sostituirlo tacitamente con la fotografia corrente o attestare una conformità inesistente.

## 4. Specifica funzionale proposta e flussi UX

### Flusso A — esportazione singola su richiesta

1. L'utente seleziona un certificato già emesso e sceglie una funzione eventuale **“Esporta copia per conservazione (PDF/A)”**. L'azione storica **“Stampa”** continua a funzionare.
2. Un'anteprima mostra i dati ricavati **esclusivamente dallo snapshot immutabile**: numero, data, autore/emittente, titolo storico, tecnica, misure, eventuale destinatario, note e foto **solo se già incluse nello snapshot**. Campi non disponibili restano dichiarati “non presenti”; non si riempiono da `artworks` corrente.
3. Prima di salvare, un riepilogo specifica se i dati personali del destinatario, le note e l'immagine sono presenti. L'utente sceglie liberamente una cartella tramite dialogo nativo; nessuna esportazione automatica.
4. La generazione avviene localmente in file temporaneo sicuro. Una **validazione tecnica effettiva** confronta il risultato con il profilo PDF/A scelto, prima del salvataggio finale; su fallimento, nessun file viene pubblicato come “conforme”.
5. Il file viene pubblicato senza sovrascrivere documenti preesistenti; nome suggerito con seriale **sanificato** e suffisso univoco, non con dati personali. È possibile aprire il file nel lettore PDF del sistema o mostrare l'esito della verifica, senza richiedere servizi web.

### Flusso B — certificato già emesso e scheda modificata

- Anteprima confrontabile con lo snapshot storico; non consentire un nuovo “certificato identico” basato sui dati correnti.
- Se si desidera correggere un certificato legalmente/operativamente errato, rinviare a una **procedura gestionale separata**, da decidere: non alterare `snapshot_json`, numero seriale, vendite o copie già consegnate.

### Flusso C — possibili esportazioni multiple, soltanto in una fase successiva

- Se emerge una domanda concreta, selezione esplicita di certificati e cartella, con conteggio, anteprima delle informazioni sensibili e rapporto finale per elemento.
- Nessuna creazione automatica di un intero archivio pubblico; un fallimento parziale produce un rapporto chiaro e lascia i certificati originari invariati.
- I documenti esportati sono **derivati ricreabili**; non diventano automaticamente nuovi allegati permanenti o parti del backup, a meno di una distinta decisione sul loro versionamento.

## 5. Conformità proposta e scelte da validare

- Valutare anzitutto **PDF/A-2u** o un profilo altrettanto supportato per il testo Unicode e l'archiviazione: **nessuna scelta approvata**. Il solo livello `2b` può garantire riproducibilità visiva, ma non offre necessariamente tutte le proprietà di estraibilità Unicode/struttura richieste dal caso d'uso.
- Caratteri incorporati e legalmente redistribuibili; layout stabile, metadati XMP coerenti, assenza di risorse remote, profilo colore e trasformazioni immagini esplicite; verificare che non siano inseriti JavaScript o elementi attivi.
- Il documento dovrebbe conservare **testo selezionabile**, non una scansione raster dell'intera pagina. Per fruibilità da screen reader, valutare struttura e ordine di lettura e requisiti PDF/UA in un percorso separato: **superare PDF/A non prova accessibilità**.
- Definire un **validatore offline indipendente**, per esempio veraPDF come riferimento di collaudo, con risultati e versione del profilo riproducibili. Non basta che il file contenga nei metadati la parola `PDF/A`.
- **Mai** etichettare un PDF stampato dal sistema come PDF/A validato senza un esito esplicito. Le eventuali firme manoscritte sono spazi grafici e **non** costituiscono firma digitale.
- Riferimenti tecnici da approfondire nella revisione: [veraPDF — profili di convalida PDF/A e PDF/UA](https://docs.verapdf.org/validation/), [veraPDF CLI](https://docs.verapdf.org/cli/validation/), [PDF Association — PDF Declarations](https://pdfa.org/resource/pdf-declarations/). Un'autodichiarazione di conformità non basta.

## 6. Ipotesi di dati e API — descrittive, NON implementative

| Elemento concettuale | Dati possibili | Regole da chiarire |
| --- | --- | --- |
| `CertificateHistoricalView` | `certificate_id`, `serial`, `issued_at`, `snapshot_json` decodificato, `image_ref?`, `field_warnings[]` | Solo valori storici; nessun refresh da opera/collezionista corrente |
| `PdfArchiveExportOptions` | `target_profile?`, `include_historical_image`, `locale`, `destination_path` | Profilo e inclusione foto solo secondo politica deliberata; non proporre omissioni che falsino il contenuto del certificato |
| `PdfValidationResult` | `profile_claimed`, `profile_checked`, `is_compliant`, `rule_failures[]`, `validator_id?`, `validated_at?` | Un esito sconosciuto o errore non è “conforme” |
| `CertificateExportReceipt` | `certificate_id`, `output_sha256`, `output_size_bytes`, `validation_result`, `file_name` | Rapporto **facoltativo**, locale; nessuna dichiarazione di autenticità |
 
API *puramente ipotetiche*: `previewHistoricalCertificate(id) -> HistoricalView`; `renderArchivePdf(historicalView, options) -> TemporaryFile`; `validateArchivePdf(tempFile, profile) -> ValidationResult`; `publishVerifiedPdf(tempFile, destination) -> ExportReceipt`.

**Evitare una nuova tabella SQLite nel primo perimetro:** `certificates` resta la fonte autorevole e immutabile. Se una futura cronologia delle esportazioni è realmente necessaria, deciderne retention, backup, compatibilità e privacy prima di definirla. L'hash del PDF è una traccia di copia, **non** una firma o una prova di chi abbia emesso il certificato.

## 7. Offline-first, sicurezza, privacy, impostazioni e lingue

- **Offline:** generazione, font, immagini, validazione e lettura funzionano senza account, cloud, licenze attivate online, server di stampa o dipendenza dal PDF di una piattaforma proprietaria. Cartella di destinazione scelta liberamente, anche USB/NAS già montato dall'OS.
- **Sicurezza:** sorgenti media lette dal percorso confinato dell'archivio, contro traversal e symlink; limiti a dimensioni e decompressione delle immagini; escaping dei campi testuali; file temporanei con permessi restrittivi e cleanup prudente; output finale solo dopo validazione e senza sovrascrittura. Gestire interruzione/disconnessione, spazio insufficiente e nomi illegali su tre sistemi.
- **Privacy:** nome di acquirente, note e immagini possono essere sensibili; anteprima chiara, non esportare automaticamente interi registri, non usare validatori web, non scrivere contenuti privati in log o rapporti telemetrici. Il PDF/A è **in chiaro** e può essere copiato liberamente da chi riceve il file.
- **Impostazioni:** nessuna configurazione obbligatoria. Eventuale default del profilo deciso dal progetto, con motivazione e linguaggio semplice; possibilità di aprire la cartella selezionata. No setting nascosto che trasformi automaticamente ogni stampa in export certificato.
- **Multilingua:** lingua del sistema come predefinita e **fallback inglese**; scelta manuale tra lingue supportate IT/EN/ES/FR/DE. Localizzare UI, errori ed esiti, ma **non tradurre automaticamente il testo storico** di un certificato emesso. Date e separatori leggibili; metadati e normalizzazione Unicode stabili.
- **Compatibilità:** Windows/macOS/Linux, archivi SQLite e backup esistenti **inalterati**, certificati storici con e senza immagine, stampanti correnti non toccate. Metadati e font devono essere portabili; il PDF visualizzato non deve cambiare con i font disponibili sul computer ricevente.

## 8. Dipendenze, alternative, rischi, complessità

**Dipendenze da valutare, non aggiungere ora:** generatore PDF locale con supporto reale a PDF/A e Unicode, caratteri incorporabili con licenza compatibile col software, gestione del colore, validatore indipendente offline, test corpus e packaging multipiattaforma. Esaminare le licenze, l'ingombro e la manutenzione prima di introdurre una libreria; **veraPDF è un possibile strumento di verifica, non una dipendenza automaticamente scelta per l'app**.

**Alternative:**

1. **Lasciare la sola stampa di sistema:** costo nullo e UX già nota; non consente di promettere PDF/A verificato.
2. **Guida all'esportazione manuale con software esterno:** minore costo tecnico, ma introduce passaggi e possibili incompatibilità e richiede verifica della conformità caso per caso.
3. **Export PDF semplice, dichiaratamente non PDF/A:** può migliorare determinismo ma non risolve la conservazione certificata; valutare solo se PDF/A è prematuro.
4. **Documento HTML statico + font/immagini:** più semplice ma fragilità e sicurezza delle risorse quando viene trasferito; non sostituisce necessariamente il PDF per uso professionale.

**Rischi:** certificati falsamente etichettati come conformi; differenze di resa visiva rispetto alle stampe storiche; font/licenze; gestione colore; regressioni di immagini storiche, payload JSON malformati, privacy di destinatari; PDF apparentemente “immutabile” ma alterabile dall'esterno; conflitti con firme digitali future. **Complessità stimata M–A**, maggiore se sono richiesti simultaneamente PDF/A, PDF/UA, firma digitale e supporto affidabile dei font per tutte le lingue. Le ultime due capacità non sono in perimetro.

## 9. Criteri di accettazione e verifiche suggerite (nessun test implementato ora)

- Esportazione locale da **certificato storico**, senza modificare UUID, seriale, snapshot, vendite, database o foto originali; confronto dei campi con `snapshot_json`, anche dopo aggiornamento dell'opera.
- Una copia valida supera un **validatore indipendente offline** per il profilo dichiarato; falso positivo deliberato (metadati “PDF/A” su PDF non conforme) deve essere respinto.
- Caratteri accentati e non ASCII, nome lungo, impaginazione su più pagine, dati assenti, immagini con orientamento/colore non standard; nessun carattere sostituito silenziosamente.
- Confronto visuale e controllo di testo selezionabile su Windows, macOS e Linux; prova di apertura **senza** i font installati sul computer d'origine. Non proclamare PDF/UA senza un collaudo separato.
- Snapshot malformato o foto mancante: avviso e **nessuna sostituzione tacita** con valori correnti; esito di non conformità non pubblicato come conforme.
- Cartella già occupata, path non scrivibile, disco pieno o smontato, symlink/traversal e file di grandi dimensioni: nessuna sovrascrittura o corruzione di copie/archivio; temporanei gestiti con prudenza.
- Controllo privacy (nessun contenuto inviato su rete), backup preesistenti ripristinabili, esperienza con solo tastiera e screen reader per selezione/avviso/risultato.
- Verifica che i PDF già prodotti tramite stampa non vengano rietichettati retroattivamente e che il flusso “Stampa” rimanga disponibile.

## 10. Fasi ipotetiche, priorità e questioni aperte

1. **Fase di ricerca P2:** interviste brevi con artisti, gallerie e archivisti; raccogliere *esempi fittizi* di certificati e requisiti effettivi di deposito. Decidere se il documento corrente è sufficiente; definire profilo, font e test corpus senza sviluppo.
2. **Fase eventuale di prototipo separato:** sperimentazione locale del solo rendering/validazione su snapshot di test, senza toccare certificati reali o UI definitiva; misurare conformità e portabilità.
3. **Fase eventuale d'integrazione:** scelta esplicita del proprietario, contratto UX con salvataggio sicuro e reporting; collaudo multipiattaforma.
4. **Fase eventuale successiva:** export multiplo con rapporto, valutando necessità e costi; firme digitali e verifica di autenticità restano fuori perimetro.

**Questioni da sottoporre al proprietario (non decisioni):**

- Quali utenti richiedono davvero **PDF/A** invece di normale PDF o stampa cartacea? Quale profilo è richiesto da eventuali istituti destinatari?
- Il certificato digitale deve essere soltanto **copia fedele** di quello storico o occorre distinguere un ulteriore formato consultivo? Cosa fare per certificati con immagini storiche irrimediabilmente assenti?
- È necessaria una firma **manoscritta su carta**, una scansione o una firma digitale? Sono processi con significati giuridici diversi e non devono essere confusi.
- In quali lingue viene **emesso** il certificato? L'interfaccia può cambiare lingua senza mutare il contenuto storico?
- Quanti certificati vengono esportati ogni anno, e quanto pesa per l'utente verificare una copia alla volta?
- Quali standard di accessibilità PDF sono effettivamente richiesti? Una futura verifica PDF/UA merita una proposta distinta?
- Qual è il costo sostenibile di packaging e manutenzione delle librerie, e basta inizialmente una guida per esportazione esterna?

**Conclusione:** opportunità delimitata e verificabile, ma soltanto **Proposta — da valutare**. Nessun profilo approvato, nessun codice/test/configurazione/issue/PR implementativa, nessun merge.
