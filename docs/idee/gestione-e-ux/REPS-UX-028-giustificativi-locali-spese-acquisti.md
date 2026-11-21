# REPS-UX-028 — Giustificativi locali delle spese e degli acquisti

- **ID stabile:** REPS-UX-028
- **Data:** 2026-11-21
- **Stato:** Proposta — da valutare
- **Filone:** Gestione e UX
- **Priorità proposta:** P1–P2, condizionata a interviste e prove con documenti reali
- **Natura:** esclusivamente ideativa e documentale; **non autorizza sviluppo, assegnazione o modifiche operative**

## 1. Problema, utenti e verifica dell'esistente

Un artista che acquista colori, cornici, strumenti o servizi di stampa può registrare importo e fornitore, ma deve poter ritrovare anche **la fotografia dello scontrino o il PDF della fattura** dopo mesi, senza dipendere dal link del negozio, dall'email o dal disco dove si trovava il file. Lo stesso serve a piccole gallerie e studi con più collaboratori, non per automatizzare la contabilità fiscale ma per ricostruire le proprie decisioni di spesa.

**Fatti verificati su `main` al 21 novembre 2026:**

- [`src-tauri/src/finance.rs`](../../../src-tauri/src/finance.rs) e [`src/app/models.ts`](../../../src/app/models.ts) espongono per `Expense` data, categoria, descrizione, importo in centesimi EUR, quota/fondo, fornitore, `purchase_url` e note: **non compare un allegato locale associato alla spesa**.
- [`docs/ECONOMY-I18N.md`](../../ECONOMY-I18N.md) descrive registrazione ed esportazione CSV delle spese come dati gestionali, non pagamenti bancari accertati.
- [`docs/RECEIPT-ATTACHMENTS.md`](../../RECEIPT-ATTACHMENTS.md) descrive già **allegati locali agli incassi delle vendite**, con copia nel media store, controllo del tipo, hash SHA-256, backup e download. **Non sono allegati delle spese**. Questa funzionalità è quindi un precedente UX/tecnico, non un motivo per duplicare i documenti degli incassi.

**Problema concreto:** il link di acquisto può scadere e il PDF conservato in una cartella esterna può essere spostato; ripetere l'inserimento manuale o perdere la prova documentale ostacola il controllo dei costi. Non è stato misurato quanto spesso succeda: l'entità del problema è un'**ipotesi da convalidare**.

**Destinatari:** artisti indipendenti che acquistano materiali, studi con spese ricorrenti, galleristi con fatture di trasporto/allestimento, archivisti che documentano costi di conservazione.

## 2. Confini e anti-doppioni

Questa proposta riguarda soltanto il **collegamento tra una spesa già registrata e i suoi documenti di supporto**, senza cambiare importo, categoria, data o statistiche di quella spesa.

- **REPS-UX-003** attribuisce costi esistenti alle singole opere; il giustificativo rimane associato alla **spesa sorgente**, eventualmente raggiungibile dalla scheda dell'opera, senza creare altre uscite.
- **REPS-UX-008** propone scorte, quantità e consumi; qui non si conteggiano pezzi né movimenti di magazzino.
- **REPS-UX-027** studia le valute; qui non si convertono importi né si deducono valute da documenti.
- **REPS-TECH-001/003/006/018** (filone [tecnico #16](https://github.com/Cesare-ml/Repertica-Studio/pull/16)) studiano portabilità, integrità dei media, cifratura e CSV trasversalmente; non definiscono l'interazione gestionale per il giustificativo di spesa.
- **Allegati agli incassi già presenti:** riutilizzarne concettualmente le cautele di sicurezza; non proporre un nuovo archivio documentale universale, una duplicazione del Media Center o un'estensione degli incassi.

**Esplicitamente fuori ambito:** OCR obbligatorio; estrazione automatica degli importi; scansione tramite servizio cloud; fatturazione elettronica; dichiarazioni fiscali; recupero diretto da email o banca; corrispondenza fiscale certificata; pagamento della spesa; verifica dell'autenticità legale del documento.

## 3. Scenari d'uso e flussi UX

1. **Acquisto materiali.** Dopo aver registrato una spesa di 47,80 EUR, l'artista apre **Spese → Dettaglio → Documenti → Aggiungi documento**. Sceglie una foto JPEG dello scontrino. Un riepilogo mostra nome, formato e dimensione; la conferma importa una copia locale. La spesa resta di 47,80 EUR, indipendentemente dall'allegato.
2. **Fattura di servizio.** La galleria collega un PDF del trasporto a una spesa. In seguito può esportare **solo quel documento** nella cartella scelta e aprirlo con il programma PDF di sistema. Non serve più il percorso originale e non viene trasmesso a terzi.
3. **Documento non disponibile.** È possibile registrare una spesa senza allegati: il pannello mostra «Nessun documento associato» e **non** «Spesa non valida». Un filtro facoltativo «Senza documento» permette una ricognizione manuale, senza solleciti esterni.
4. **Fattura riferita a più spese.** Un'unica fattura comprende tele, colori e trasporto registrati come righe distinte. **Nella fase iniziale**, non deduplicare né ripartire automaticamente il totale: indicare il limite e consentire all'utente di conservare la fattura in una singola voce riepilogativa. La relazione uno-a-molti/ molti-a-molti con lo stesso file originale è un'opzione da esaminare in una fase successiva, con importi attribuiti espliciti.
5. **Correzione e cancellazione.** Modificare l'importo della spesa non modifica il PDF né lo interpreta; può comparire un avviso non bloccante «Verifica che il documento corrisponda». Eliminare un legame richiede conferma; l'eventuale eliminazione fisica non deve distruggere un file ancora utilizzato.
6. **Backup e trasferimento.** Su un altro computer, un backup completo verificato ripristina spesa, allegato e collegamento. Un CSV delle sole spese rimane invece una tabella di dati e **non contiene il PDF**.

### Interazione minima

Nella riga di una spesa, indicatore testuale accessibile «1 documento» oppure «Nessun documento»; nel dettaglio, elenco con nome, formato, dimensione, azioni **Aggiungi / Esporta copia / Scollega**, con conferma esplicita prima di rimuovere. L'utente non deve riaprire il registro degli incassi né navigare il Media Center. La selezione e l'esportazione usano dialoghi nativi; trascinamento file e anteprima immagini sono possibili miglioramenti successivi, **non prerequisiti**. Per il PDF, preferire un salvataggio locale e l'apertura col lettore OS invece di indebolire la CSP della WebView.

## 4. Dati necessari e invarianti (ipotesi descrittive, nessuno schema approvato)

- **`expense_document`**: `id` UUID, `original_filename` informativo, `stored_media_path` interno relativo, `mime_type`, `size_bytes`, `sha256`, `created_at`, eventuale `document_date`, `document_reference` e nota facoltativa. Non salvare il percorso di origine come dipendenza funzionale. Le informazioni identificative nel nome file sono considerate potenzialmente personali.
- **`expense_document_link`**: `expense_id`, `document_id`, data del collegamento; cardinalità iniziale consigliata **un documento per spesa** per un MVP semplice, ma valutare se non bloccare strutturalmente un successivo modello molti-a-molti. Un UUID file non equivale a una nuova spesa.
- **Invarianti:** una spesa senza allegato resta valida; documento e spesa hanno identità distinte; allegare, scollegare, esportare o sostituire il file **non** aggiorna `amount_cents`, `fund`, totale mensile, allocazioni o vendita; non inferire automaticamente prezzo o IVA dal documento.
- **Metadati facoltativi:** data e numero del documento vengono dichiarati dall'utente, distinti dalla data della spesa; se assenti, non inventare valori. Un importo indicato nel documento può comprendere più voci e non deve essere presentato come importo della singola spesa.
- **Integrità:** prima di pubblicare il collegamento, copia in staging e convalida tipo reale/estensione, regolarità del file, dimensione e hash; dopo la scrittura, commit coerente. Un fallimento lascia la spesa intatta e non pubblica file parziali. Gli hash servono a rilevare alterazioni, **non** a certificare l'origine fiscale.

**Limiti iniziali possibili, da confermare:** PDF/JPEG/PNG/WebP e limite coerente con i documenti degli incassi (oggi 12 MiB), visualizzando le ragioni di eventuali rifiuti. File multipagina, scansioni e immagini con dati di contatto o bancari devono essere considerati dati sensibili per chi li conserva.

## 5. Impostazioni, offline, lingue e accessibilità

- **Impostazioni:** funzione disponibile senza attivazione complessa; nessun cloud; eventuale limite importazione configurabile *solo* se esiste un caso d'uso concreto, altrimenti mantenere un limite prudente documentato. Possibile filtro predefinito «mostra anche spese senza documento», disattivabile.
- **Offline e indipendenza:** allegato copiato nella cartella dati locale di Tauri; nessuna richiesta Internet, account o abbonamento. Si possono scegliere documenti da USB/NAS montati, ma dopo la copia l'archivio non dipende dall'unità di origine. Nessuna automazione di lettura dei file senza azione esplicita.
- **Lingue:** locale di sistema con fallback inglese, override IT/EN/ES/FR/DE. Localizzare pulsanti, errori, indicazioni di dimensione, date e avvisi, **non tradurre** nomi originali dei file e contenuto dei documenti; nessuna modifica ai valori registrati.
- **Accessibilità:** tutti i comandi da tastiera, focus restituito alla riga di origine dopo chiusura, avvisi di errore leggibili per screen reader, conferme non espresse soltanto tramite colore, nome file leggibile e troncamento con nome completo consultabile; layout usabile a zoom elevato e finestra ridotta.

## 6. Import/export, privacy e compatibilità

- **Import:** copia esplicita di un file scelto, non importazione di un registro fiscale o di un URL. MIME, contenuto reale ed estensione devono essere coerenti; rifiuto di link simbolici e percorsi non sicuri. Se il file è già collegato altrove, non creare associazioni silenziose: proporre soltanto in futuro un riuso consapevole.
- **Export di un documento:** l'utente sceglie la destinazione; verifica SHA-256 prima della copia; rifiuto di sovrascrivere senza decisione esplicita; niente salvataggio nella directory interna dei dati dell'app. Restituzione di un errore chiaro se la copia non è leggibile.
- **Export dei metadati:** mantenere le **dodici esportazioni CSV attuali**; valutare un registro aggiuntivo `expense_documents` con ID di spesa/documento, hash, MIME, dimensione e nome, marcato esplicitamente «metadati, nessun file allegato». Un export completo reimportabile richiede un futuro formato aperto (TECH-001); il **backup verificato** resta l'unico pacchetto completo oggi.
- **Backup:** includere ogni allegato in `media/` con il manifest esistente e verificare integrità + collegamenti **prima** di pubblicare o ripristinare una copia. Una copia con allegato mancante non deve sovrascrivere l'archivio buono. I backup esistenti **non sono cifrati**: segnalare chiaramente il rischio se si condividono cartelle di rete o chiavette.
- **Privacy:** nessuna anteprima pubblica, nessun invio al MCP locale, a un sito o a un provider esterno; esportazione separata e consapevole. Documenti possono contenere IBAN, indirizzi e riferimenti di terzi. L'apertura in un'app di sistema è una scelta dell'utente, con implicazioni per la cronologia del lettore PDF.
- **Compatibilità:** migrazione SQLite solo additiva; tutte le spese esistenti conservano identità, importi e andamento mensile, con zero allegati iniziali **senza documento fittizio**. Versioni precedenti dell'app potrebbero non comprendere il nuovo schema: occorre documentare il vincolo e preservare ripristini affidabili. Funzionamento Windows/macOS/Linux con path non ASCII e unità esterne.

## 7. Dipendenze, complessità, rischi, alternative

**Dipendenze concettuali:** registro `expenses`, media store locale, routine sicure di copia/verifica e backup, UI Angular di gestione spesa, dialoghi Tauri, i18n. Valutare il **riuso architetturale** dei controlli esistenti in `receipt_attachments.rs`, senza assumere che il codice attuale sia già generico. Nessuna dipendenza SaaS.

**Complessità:** **media** con un documento per spesa e supporto a copia/consultazione/esportazione; **medio-alta** per link condivisi molti-a-molti, revisioni e ricostruzione documenti, casi non autorizzati in questa fase.

**Rischi e mitigazioni:**
- Documento che include più spese → somma errata/doppio conteggio: i file non generano mai movimenti e l'eventuale attribuzione richiede importi espliciti.
- Eliminazione della spesa o del file condiviso → perdita storica: avvisi e conteggio referenze; nessuna cancellazione in cascata non dichiarata; verifiche in backup.
- PDF/immagini dannosi o file grandissimi → controlli dei byte, limiti, nessun renderer PDF privilegiato, apertura volontaria con software esterno.
- Errori su unità rimovibili e file con nomi particolari → copia atomica, errori localizzati, conservazione dell'originale e nessuna dipendenza dal percorso sorgente.
- Falsa percezione di conformità fiscale → etichettare «documento allegato dall'utente, non verificato»; non dichiarare la spesa fiscalmente deducibile.
- Riservatezza su esportazioni e backup non cifrati → conferma destinazione, messaggi privacy e assenza di sincronizzazione automatica.

**Alternative:** (a) tenere soltanto `purchase_url`, minimale ma fragile con link scaduti; (b) file in cartelle esterne con percorso salvato, evita copie ma espone a perdita di collegamenti e backup incompleti; (c) archiviare tutti i documenti nel Media Center generico, più riusabile ma richiede distinguere fotografie artistiche da giustificativi finanziari e permessi: non appare necessario per una prima fase.

## 8. Criteri di accettazione ipotetici

1. Importare uno scontrino di una spesa EUR 47,80 non crea righe spesa, vendite o incassi e **non cambia di un centesimo** il riepilogo mensile.
2. Dopo il collegamento, scollegare il dispositivo sorgente non impedisce di consultare/esportare il documento dall'archivio locale.
3. Un file non valido, link simbolico, troppo grande o di formato ingannevole è respinto con errore comprensibile; nessun record/file parziale resta pubblicato.
4. Il ripristino da un backup completo verificato conserva file, hash e `expense_id` senza duplicare né modificare spese.
5. Un backup con documento mancante o alterato non sostituisce l'archivio attivo; una spesa preesistente senza allegati continua a funzionare normalmente.
6. Scollegare o rimuovere un documento usato altrove non distrugge il file in modo silenzioso; la cancellazione di una spesa richiede una scelta comprensibile sul relativo allegato.
7. Esportare un PDF produce byte con SHA-256 corrispondente; non sovrascrive un file di destinazione preesistente né scrive nella directory privata dell'app.
8. Il CSV spese resta interoperabile e **non** viene presentato come backup dei documenti; se compare un CSV metadati, è dichiarato esplicitamente non comprensivo di binari.
9. Prove native su Windows/macOS/Linux, con file Unicode, USB/NAS, tastiera, screen reader e lingue IT/EN/ES/FR/DE con fallback inglese.

## 9. Fasi possibili — nessuna autorizzata

- **A — Verifica dei bisogni:** raccogliere esempi di spese, tipi di documenti, dimensioni e casi con una fattura per più voci; definire la conservazione minima utile.
- **B — MVP eventuale:** un documento PDF/immagine per singola spesa, copia locale, consultazione/esportazione, rimozione sicura, backup verificato; nessuna modifica ai conti.
- **C — Estensioni solo se motivate:** più documenti per spesa, riuso esplicito del medesimo giustificativo, filtro spese senza documenti, import/export descrittivi completi.
- **D — Solo con richiesta separata:** versioni del documento e controllo delle incongruenze dichiarate, senza OCR obbligatorio né automatismi fiscali.

## 10. Requisiti certi, ipotesi e domande al proprietario

**Certi dal repository:** spese e CSV esistono; il modello `Expense` non espone allegati locali; allegati agli **incassi** e meccanismi SHA-256/backup esistono; app gratuita locale, lingua OS/fallback inglese, backup a cartella scelta. **Certo come principio di progetto:** una copia documento non deve creare o modificare movimenti finanziari senza conferma esplicita.

**Ipotesi da convalidare:** esigenza frequente di conservare fatture/scontrini direttamente in Repertica, utilità del filtro «Senza documento», adeguatezza di 12 MiB e di un solo file nella prima fase.

**Domande per revisione del proprietario, senza presumere risposta:** conviene permettere subito più documenti per spesa? Quali formati documentali servono davvero (PDF/immagini sono sufficienti)? È preferibile far riusare una fattura tra varie spese o mantenere un unico record spesa per documento? Chi deve poter vedere gli allegati quando il prodotto evolverà verso collaborazione locale? Serve una politica di conservazione distinta per documenti fiscali rispetto alle semplici note d'acquisto?

**Decisione:** proposta non approvata, non assegnata e non avviata. Rimane per revisione personale nella PR documentale del filone UX.
