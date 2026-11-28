# REPS-TECH-047 — OCR locale facoltativo dei documenti scansionati

- **ID stabile:** REPS-TECH-047
- **Data:** 2026-11-22
- **Stato:** **Proposta — da valutare**
- **Filone:** strumenti e architettura, solo analisi e documentazione.
- **Priorità proposta (non approvata):** P2–P3 condizionata a esigenze reali; nessuno sviluppo se basta una soluzione esterna.
- **Principi:** desktop Tauri, SQLite e file locali, offline-first, nessun account/abbonamento, dati esportabili, backup in cartella scelta, lingua di sistema e fallback inglese.

## 1. Problema, destinatari ed evidenze

Un artista conserva uno scontrino fotografato; una galleria conserva una ricevuta scansionata; un archivista deve rintracciare un codice scritto in un PDF costituito da immagini. L'estrazione del **testo già incorporato** nei PDF non può ricostruire le parole rappresentate soltanto da pixel. Si può valutare un **OCR locale avviato manualmente** per avere una trascrizione dichiaratamente fallibile, senza alterare il documento originale.

**Utenti da intervistare:** studi d'artista, piccole gallerie, archivi con incassi documentati e pochi documenti digitalizzati. Nessuna evidenza quantitativa è stata raccolta sull'effettiva frequenza di ricerca: non è un bug osservato o una funzione già approvata.

**Riscontro statico su main (22 novembre 2026):**

- [RECEIPT-ATTACHMENTS](../../RECEIPT-ATTACHMENTS.md) e [backend degli allegati](../../../src-tauri/src/receipt_attachments.rs): esiste già **un allegato per incasso**, PDF/JPEG/PNG/WebP fino a 12 MiB, copiato in media/, con MIME, SHA-256 e backup verificato. I PDF si esportano per essere aperti nel lettore del sistema, non nella WebView protetta da CSP.
- [MVP](../../MVP.md), [portabilità](../../DATA-PORTABILITY.md) e [Cargo.toml](../../../src-tauri/Cargo.toml): copie complete SQLite + media, esportazioni CSV senza documenti binari; nelle dipendenze e nei percorsi analizzati **non emerge un motore OCR**. Questa è verifica statica, non collaudo desktop.
- [MCP](../../MCP.md): server STDIO locale già esistente e read-only, con esclusione delle fotografie. L'OCR non deve ampliare automaticamente le informazioni ottenibili dagli assistenti IA.
- [REPS-TECH-028](REPS-TECH-028-ricerca-testo-pdf-allegati-offline.md) propone **ricerca del testo nativo** dei PDF allegati, esplicitamente **senza OCR**. Il presente studio riguarda esclusivamente il riconoscimento da immagini e può essere valutato autonomamente, senza inventare una ricerca già implementata.
- L'altra automazione propone [REPS-UX-028 — giustificativi di spesa](https://github.com/Cesare-ml/Repertica-Studio/blob/docs/idee-gestione-ux-20261121/docs/idee/gestione-e-ux/REPS-UX-028-giustificativi-locali-spese-acquisti.md), cioè **collegamenti documentali alle spese**, non riconoscimento del testo. Non è una funzione già presente e non viene incorporata qui.

**Anti-doppioni:** TECH-016 tratta la custodia e il riuso dei file; TECH-028 il testo PDF già presente; TECH-024 metadati IPTC/XMP; TECH-021 il colore; TECH-003 l'integrità; TECH-014 permessi MCP; TECH-002 automazioni locali. Nessuno di questi documenti va modificato. Nessun nuovo registro di spese, prezzi, dati fiscali, provenienza o rapporti con clienti è compreso nel perimetro.

## 2. Scenari e flussi UX ipotetici

**A. Documento fotografico già allegato a un incasso.** L'utente apre l'allegato e preme «Riconosci testo sul dispositivo…». Un dialogo indica nome, tipo, lingua di riconoscimento, limiti di elaborazione e avverte che il testo può essere errato. Default: «Usa solo questa volta», senza indicizzazione permanente. Dopo consenso esplicito, il programma mostra una trascrizione con etichetta **«OCR automatico — controllare sull'originale»**. È possibile selezionare e copiare manualmente una parte del testo. Annulla non modifica record o file.

**B. PDF che contiene soltanto scansioni.** La pagina viene rasterizzata in un ambiente isolato e riconosciuta con un modello linguistico già disponibile in locale. I risultati sono identificati per pagina, anche quando alcune non sono elaborabili. Se il PDF contiene testo nativo, preferire l'eventuale estrazione prevista da TECH-028, prima di duplicare il riconoscimento. Non si genera né si sostituisce automaticamente un PDF ricercabile.

**C. Numero, importo o nome riconosciuto male.** Il risultato resta testo *non verificato*: «€ 38,00» non diventa una spesa o un incasso, un codice non identifica automaticamente un'opera. Non si aggiorna nessuna scheda, documento legale, pagamento o certificato e non si presuppone che un valore estratto sia autentico.

**D. Lingue e scansioni difficili.** L'interfaccia segue la lingua del sistema con fallback inglese, ma **la lingua del documento è un'impostazione distinta**. Se il modello per la lingua desiderata non è installato, mostra un errore comprensibile senza aprire una connessione obbligatoria. Grafie a mano, timbri, scansioni storte e stampe sbiadite possono produrre omissioni; «nessun testo leggibile» non equivale a «documento vuoto».

**E. Eliminazione, sostituzione o cambio archivio.** Un eventuale indice OCR derivato deve essere scartabile e segregato per archivio. Alla rimozione dell'allegato, al mutamento del suo SHA-256 o al restore di un diverso archivio, i vecchi risultati non possono essere mostrati come correnti. Il backup rimane autosufficiente tramite **originali e relazioni**, senza dipendere dal testo OCR.

## 3. Specifica funzionale e limiti

1. **Consenso per allegato selezionato**, spento per impostazione predefinita: non cercare autonomamente in tutto l'archivio, nel disco, nelle email, nei backup o negli allegati di altri archivi. In prima fase soltanto i documenti incasso già acquisiti, non i futuri giustificativi di spesa.
2. **Identità dei byte e del documento:** validare percorso interno, file regolare, tipo dichiarato/riconosciuto, dimensione e SHA-256 prima di elaborare. I contenuti testuali del file non possono autorizzare accessi a altri percorsi.
3. **Originali intatti:** nessuna riscrittura del PDF o dell'immagine, nessuna modifica allo snapshot dei certificati, nessun ricalcolo di dati commerciali. Testo derivato in memoria per default; indice persistente soltanto come fase separata, opt-in e cancellabile.
4. **Risorse limitate e cancellazione:** massimi di pagine/pixel, memoria, tempo, processi concorrenti e dimensione della trascrizione; anteprima dei limiti e annullamento. Stati distinti: completato, parziale, non supportato, protetto, lingua mancante, errore e annullato.
5. **Input ostili:** immagini decompression bomb, PDF malformati, allegati e URI incorporati, payload che impegnano CPU/RAM, output Unicode/HTML malevoli. Preferire processo figlio limitato e privo di accesso alla rete, con validazione IPC; non inserire contenuto OCR come HTML eseguibile o come PDF viewer nella WebView. Non allentare la CSP esistente.
6. **Ricerca solo eventuale:** se adottata TECH-028 o una ricerca derivata, la consultazione delle trascrizioni richiede attivazione esplicita. Distinguerla da «PDF non analizzato», «nessun testo riconosciuto» e «nessun risultato». Nascondere anteprime testuali sensibili per default quando si condivide lo schermo. Qualsiasi risultato rimanda all'originale verificato.
7. **Confine IA/MCP:** il testo OCR non deve apparire automaticamente nei tool STDIO MCP, in server HTTP, nei log, nelle notifiche, in clipboard o in servizi remoti; una futura esposizione richiederebbe un mandato e controlli di consenso separati.
8. **PDF ricercabili:** eventuali PDF derivati e layer invisibili sono fuori dal primo rilascio ipotetico. Dovrebbero essere copie etichettate con versione, origine e checksum distinti e mai sostituire gli originali.

## 4. Dati e API ipotetici, solo descrittivi

Possibili strutture concettuali, **non tabelle né API approvate**:

~~~text
OcrJobPreview:
  archive_context, receipt_attachment_id, original_sha256,
  mime_type, page_estimate, language_codes, engine_id,
  model_versions, resource_limits, mode: temporary | opt_in_index
OcrPageResult:
  page_number, status, recognized_text?, warnings[],
  provenance: machine_recognition_not_verified
OcrReport:
  original_sha256, attachment_id, engine_version,
  model_versions, completed_at, pages[], was_partial
previewSelectedOcr(attachment_id, options) -> OcrJobPreview
recognizeAfterConfirmation(preview) -> OcrReport
forgetLocalOcrIndex(archive_context) -> RemovalReport
~~~

Se esistesse un indice derivato, invalidarlo usando **identità dell'archivio effettivo + UUID allegato + SHA-256 originale + versioni parser/engine/modelli**, non solo UUID; differenziare gli archivi dopo restore. Evitare di duplicare i documenti binari nell'indice. L'indice non diventa parte obbligatoria dello schema SQLite o del backup; il testo di riconoscimento non modifica i  CSV e non è assunto come dato finanziario, identificativo o prova fiscale.

**Import/export:** restano validi gli export degli allegati e il backup completo in cartella scelta. Si potrebbe offrire solo su richiesta un file **TXT UTF-8** o sidecar aperto con provenienza e stato di incertezza, chiedendo percorso e avvertendo della privacy. Nessuna importazione di trascrizioni verso campi autorevoli senza successiva scelta manuale documentata; nessuna sovrascrittura di export già esistenti.

## 5. Privacy, offline, lingue, impostazioni e accessibilità

- **Offline al 100%:** nessun account, abbonamento, cloud, download obbligatorio o telemetria; modelli linguistici forniti localmente o installati con operazione volontaria verificabile. Assenza dei modelli non rende inutilizzabile Repertica. Non interpretare una cartella modelli come permesso a leggere tutte le altre cartelle.
- **Dati personali:** fatture, ricevute e scansioni possono contenere indirizzi, dati fiscali, riferimenti bancari e firme. Il testo OCR persistente è una **nuova copia di dati potenzialmente riservati**: default nessuna conservazione; se in futuro opt-in, visualizzare dimensioni, posizione, cancellazione, limitazioni su SSD/swap e rischi in presenza di backup. Non promettere eliminazione forense.
- **Impostazioni proposte:** «OCR su documenti selezionati» disattivato, lingua del documento, modelli presenti e loro fonte, elaborazione temporanea predefinita, indice opt-in solo eventuale, limite lavoro, «Cancella dati OCR locali», disattivazione immediata. Ogni pulsante deve indicare la conseguenza, senza attivazioni invisibili.
- **Localizzazione:** lingua UI di sistema, fallback inglese, eventuale scelta manuale; il riconoscimento non traduce il testo. Errori localizzati e distinti per «modello mancante», «nessuna parola leggibile» e «elaborazione incompleta».
- **Accessibilità:** navigazione da tastiera, focus non perso durante il job, stati annunciabili da screen reader senza spam, testo selezionabile e copia esplicita, non usare solo il colore per indicare testi incerti, annullamento accessibile.

## 6. Alternative, dipendenze e rischi

| Alternativa | Vantaggio | Costo/limite |
| --- | --- | --- |
| A. Usare un OCR esterno già installato, offline | Nessuna dipendenza o nuovo dato in Repertica | Più passaggi manuali e privacy da gestire nel programma esterno |
| B. OCR di un allegato su richiesta, senza indice | Valore immediato senza persistenza predefinita | Packaging e sicurezza del parser ancora necessari |
| C. Indice OCR locale cancellabile e opt-in | Utile per archivi con molte scansioni | Duplicazione dati privati, invalidazione e prestazioni |
| D. OCR cloud obbligatorio | Nessuno compatibile con i principi stabiliti | **Da escludere** |

**Motore solo candidato:** [Tesseract OCR](https://github.com/tesseract-ocr/tesseract) ha codice Apache-2.0 e modelli linguistici da valutare separatamente; riconosce immagini, ma [**non legge direttamente PDF**](https://github.com/tesseract-ocr/tessdoc/blob/main/InputFormats.md). Servirebbe un rasterizzatore PDF isolato; [OCRmyPDF](https://ocrmypdf.readthedocs.io/en/stable/) è un riferimento tecnico per workflow di scansioni PDF, non una dipendenza autorizzata. Verificare licenze *di ogni componente*, modelli, dimensioni di distribuzione, firme e aggiornamenti, compatibilità nativa Windows/macOS/Linux, capacità di funzionare senza rete e costo di manutenzione. **Nessuna dipendenza è stata aggiunta.**

**Complessità:** medio-alta per singolo documento, alta per PDF multipagina e indice. Rischi: interpretazione errata dei numeri, crash del parser, blocchi di memoria/CPU, scansioni poco leggibili, modello non installato, condivisione involontaria di dati, aspettative di accuratezza e costo delle build. Prima di adottare, valutare un campione reale autorizzato.

## 7. Test e criteri di accettazione proposti

1. Con OCR spento e senza modelli, apertura archivio, incassi, allegati, backup e ripristino continuano a funzionare normalmente e senza chiamate di rete.
2. JPEG/PNG/WebP e PDF scansito autorizzati producono testo etichettato come **non verificato** con provenienza per pagina; confronto umano necessario, anche quando i numeri paiono corretti. OCR di testi nativi PDF non sostituisce silenziosamente una procedura di estrazione già prevista.
3. SHA-256 degli originali identico prima/dopo; nessuna mutazione di ricevute, vendite, importi, certificati, file originali o allegati storici.
4. Input vuoto, protetto, corrotto, multi-pagina, misto testo/immagine, enorme o ostile: limite risorse, errori leggibili, annullamento, nessun file parziale autorevole.
5. Se previsto indice, eliminazione/sostituzione dell'allegato, restore e cambio contesto non mostrano risultati obsoleti. Eliminare l'indice non elimina gli originali.
6. Windows/macOS/Linux reali: verificare installazione offline, peso motori/modelli, CPU, RAM, timeout, disco pieno, cartelle non accessibili e comportamento con file temporanei.
7. Nessun testo OCR finisce nei log, nel MCP, nelle notifiche o in invii di rete; un export volontario non sovrascrive file. Test da tastiera/screen reader, lingua sistema/fallback inglese e scelta OCR indipendente.

## 8. Fasi e questioni aperte per revisione del proprietario

**Fase 0 — Validazione senza sviluppo:** osservare 5–10 utenti e raccogliere esempi di scansioni con consenso e anonimizzazione; misurare bisogno di ricerca, accuratezza, tempi e risultati di un **OCR esterno offline**. Sospendere l'idea se basta.

**Fase 1, solo su approvazione futura:** OCR **singolo e temporaneo** per allegato già presente, con scelta lingua, controlli di risorse e testo non verificato.

**Fase 2, solo con bisogno dimostrato:** indice cancellabile opt-in, coordinato con TECH-028 senza creare due motori di ricerca separati.

**Fase 3, solo con nuovo mandato:** eventuali altri domini documentali se adottati e validati, esportazioni derivate manuali, mai cambio degli originali.

**Domande non risolte:** quanti documenti scansiti vengono cercati davvero? Basta copia manuale del testo ottenuto da OCR esterno? Quali lingue e dispositivi servono? È accettabile un pacchetto locale opzionale di modelli? È necessaria persistenza dell'indice? Quanto pesa nella distribuzione desktop? Quale sandbox è praticabile sui tre OS? Come informare gli utenti sui falsi negativi?

**Conclusione:** ipotesi senza incarico operativo, nessuna approvazione o assegnazione. **Stato: Proposta — da valutare.**
