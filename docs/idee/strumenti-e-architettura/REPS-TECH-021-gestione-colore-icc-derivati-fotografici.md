# REPS-TECH-021 — Gestione colore ICC e tracciabilità cromatica dei derivati fotografici

- **ID stabile:** REPS-TECH-021
- **Data ricognizione:** 2026-11-21
- **Stato:** **Proposta — da valutare**
- **Filone:** Strumenti e architettura, esclusivamente analisi e documentazione
- **Priorità suggerita:** **P2**, elevabile solo dopo prove con fotografie professionali e un bisogno confermato
- **Approvazione/assegnazione:** nessuna. Nessuna dipendenza, migrazione o implementazione è autorizzata da questa scheda.

## 1. Problema e destinatari

Una fotografia d'opera può essere acquisita con uno spazio colore incorporato diverso da sRGB (per esempio Display P3 o Adobe RGB). L'artista può conservare l'originale corretto ma ottenere da un ritaglio, una correzione prospettica o un'esportazione un file che **sembra** cromaticamente diverso su altri dispositivi, se la trasformazione o l'etichettatura del colore non è esplicita. Per gallerie, fotografi di opere, archivi e professionisti della stampa, la differenza fra «originale conservato» e «derivato cromaticamente interpretabile» può essere importante.

**Evidenza verificata su `main` il 21 novembre 2026:**
- [`src-tauri/src/imaging.rs`](../../../src-tauri/src/imaging.rs) applica già l'orientamento EXIF, decodifica le immagini in `RgbImage` tramite `to_rgb8()` e scrive derivati JPEG/PNG tramite encoder RGB. Nei passaggi esaminati non è visibile un contratto esplicito che legga, validi e trasformi profili ICC o che dichiari il profilo di output nei derivati.
- [`src-tauri/src/media.rs`](../../../src-tauri/src/media.rs) conserva **l'originale** come prima revisione e le modifiche come ulteriori revisioni non distruttive; gestisce inoltre metadati dell'immagine e SHA-256. **Non** si propone di sostituire questi meccanismi.
- Il [README](../../../README.md) descrive strumenti di ritaglio, prospettiva, luce/colore ed esportazione nel Media Center; il [documento di portabilità](../../DATA-PORTABILITY.md) elenca **dodici** registri CSV e chiarisce che i CSV non includono i byte delle immagini.
- La ricerca nel codice di `color_profile`/`colorspace` non ha individuato un modulo dedicato; questa è **solo una verifica statica parziale**, non dimostra che il sistema operativo o WebView ignorino i profili nei normali percorsi di visualizzazione.

**Non è stato osservato né misurato alcun bug cromatico reale.** È necessario riprodurre differenze su file e dispositivi di prova; non si presume che ogni JPEG contenga un profilo ICC o che la resa di tutti i monitor sia affidabile.

Riferimento tecnico indipendente: [International Color Consortium — specifiche ICC](https://www.color.org/icc_specs2/). I profili ICC permettono interpretazione e conversioni tra codifiche colore; non sostituiscono la calibrazione di monitor e dispositivi di stampa.

## 2. Ambito e anti-duplicazione

- [TECH-013](REPS-TECH-013-copie-fotografiche-condivisibili-privacy-metadati.md) progetta **copie da divulgare** con controllo di EXIF/IPTC/XMP sensibili, GPS, crediti e selezione della cartella. Qui il problema è **l'interpretazione cromatica durante anteprima, modifica, generazione e riutilizzo dei derivati anche soltanto interni**, indipendentemente dalla condivisione. Se entrambe fossero approvate, TECH-013 potrebbe richiamare un singolo contratto di output colore, senza duplicare UI o pipeline.
- [TECH-005](REPS-TECH-005-anteprime-media-grandi-archivi.md) studia miniature e prestazioni, non la fedeltà dei colori.
- [TECH-003](REPS-TECH-003-diagnostica-integrita-media.md) verifica esistenza/integrità dei file, non la corrispondenza colorimetrica.
- [TECH-001](REPS-TECH-001-pacchetto-dati-aperto.md) riguarda portabilità completa dei dati; questa proposta aggiungerebbe soltanto descrittori aperti dei derivati.
- [REPS-UX-010 — Diritti e crediti](https://github.com/Cesare-ml/Repertica-Studio/blob/docs/idee-gestione-ux-20261121/docs/idee/gestione-e-ux/REPS-UX-010-diritti-riproduzione-crediti-autorizzazioni.md) registra autorizzazioni **dichiarate**, non converte colori né attribuisce diritti sulla fotografia. [UX-007](https://github.com/Cesare-ml/Repertica-Studio/blob/docs/idee-gestione-ux-20261121/docs/idee/gestione-e-ux/REPS-UX-007-dossier-presentazione-opere.md) impagina dossier, senza trasformare il motore grafico.
- Non è una proposta di stampa certificata, PDF/A, calibrazione del monitor, profili CMYK obbligatori o elaborazione RAW professionale.

## 3. Scenari reali da validare

1. **Artista con file incorporato Adobe RGB/P3:** carica una fotografia e ne conserva l'originale; dopo un ritaglio verifica che la copia generata sia correttamente etichettata oppure sia convertita in un profilo di destinazione dichiarato.
2. **Galleria con schede di catalogo:** confronta l'anteprima del Media Center e il file esportato con un visualizzatore color-managed esterno. Se non sono comparabili, l'app indica chiaramente la limitazione, anziché promettere «colori fedeli».
3. **Archivio misto:** incontra file sRGB, untagged e con ICC incorporato non leggibile. Il software distingue «profilo riconosciuto», «nessun profilo dichiarato» e «profilo non valido/non supportato»; non attribuisce silenziosamente un profilo a un'immagine incerta.
4. **Foto di un certificato storico:** una revisione o conversione nuova non modifica i byte originali né immagini congelate in certificati precedenti.

## 4. Specifica proposta e flussi UX

### Flusso A — ispezione non distruttiva, prima fase

Nella scheda di dettaglio della fotografia del Media Center, aggiungere eventualmente un riepilogo discreto **«Informazioni sul colore»**: profilo sorgente rilevato (nome normalizzato solo se affidabile), presenza del tag ICC, validità, orientamento già gestito e messaggio sui limiti della resa locale. Il pulsante «Dettagli tecnici» espone origine dei dati ed esito dell'ispezione. Nessun profilo viene modificato e nessuna foto convertita durante questa fase.

Per file senza profilo, mostrare **«Spazio colore non dichiarato»** anziché «sRGB verificato». Per file non validi, errore leggibile e scelta di conservare l'originale; non eseguire una conversione inventata.

### Flusso B — revisione fotografica e anteprima, fase eventuale

Quando l'utente apre l'editor locale, la pipeline deve distinguere: byte originali; orientamento EXIF; spazio colore di interpretazione; spazio di elaborazione; profilo del risultato; rendering del display/WebView. Prima di renderizzare, mostrare in modo non invasivo se la fedeltà rispetto alla sorgente è **verificata con test**, **limitata** o **non determinabile**. Non dichiarare mai che un monitor non calibrato visualizzi i colori fisici dell'opera in modo oggettivo.

Un'eventuale opzione avanzata, mai predefinita senza test, potrebbe permettere «Mantieni un profilo incorporato valido» oppure «Crea derivato sRGB mediante conversione esplicita». **Rietichettare** pixel senza trasformazione non equivale a **convertirli**. Le revisioni restano reversibili e non sovrascrivono originali, copertine storiche o certificati.

### Flusso C — produzione di un derivato, fase eventuale

L'utente sceglie un originale/revisione e una destinazione locale tramite dialogo nativo. Prima della conferma, un riepilogo dichiara: file sorgente, eventuale profilo riconosciuto, profilo di output effettivo, dimensioni, formato, applicazione dell'orientamento, possibile limite del visualizzatore e cartella scelta. È vietata la sovrascrittura silenziosa; un esito incompleto non deve essere pubblicato come file valido.

Per copie **da condividere**, le decisioni su metadata privacy e crediti resterebbero responsabilità dell'eventuale TECH-013: un profilo ICC non autorizza a divulgare GPS, dati di scatto o informazioni sui clienti.

**Accessibilità:** tutte le opzioni sono raggiungibili da tastiera e screen reader; stati espressi in parole e non soltanto con campioni colorati; nessuna istruzione dipendente esclusivamente dalla percezione delle differenze cromatiche.

## 5. Strutture dati e ipotesi di API — NON implementate

Una prima fase potrebbe evitare qualsiasi migrazione SQLite: leggere metadati dai byte conservati, eventualmente memorizzare un risultato **temporaneo e ricostruibile**. Possibili tipi descrittivi:

```text
ImageColorInspection {
  media_id, revision_id, source_sha256,
  profile_status: "valid" | "absent" | "invalid" | "unsupported",
  profile_name?: string, profile_sha256?: string,
  orientation_applied: boolean, warnings: [LocalizedCode]
}
DerivativeColorPlan {
  source_revision_id, source_sha256,
  requested_policy: "preserve_valid" | "convert_to_srgb",
  output_profile_id, transformation_status, destination,
  non_destructive: true
}
DerivativeColorReceipt {
  output_sha256, output_profile_id, source_revision_id,
  conversion_method_version, result: "verified" | "limited" | "failed"
}
inspectImageColor(media_id, revision_id) -> ImageColorInspection
previewColorPlan(media_id, revision_id, policy) -> DerivativeColorPlan
createColorManagedDerivative(plan, user_confirmation) -> DerivativeColorReceipt
```

Campi, nomi e funzioni sono **solo ipotesi**, non API pubbliche né nuovi comandi autorizzati. Il collegamento a `media_revisions` è preferibile a un percorso fisico che può cambiare; ogni ricevuta farebbe riferimento a SHA-256 sorgente/risultato. Se utili, descrittori interoperabili in JSON/CSV potrebbero accompagnare **solo su richiesta** copie esterne o il futuro pacchetto aperto TECH-001, senza incorporare obbligatoriamente profili proprietari. Nessuna classificazione automatica di copyright o valore dell'opera.

## 6. Offline, privacy, impostazioni e multilingua

- Elaborazione **100% locale**, senza account, abbonamento, API web, telemetria, invii o ricerca online di profili. Chi desidera lavorare senza rete deve poterlo fare interamente.
- Originali e revisioni rimangono in SQLite/media locali e nei backup completi già previsti. Mai riscrivere i byte sorgente, né ricalcolare certificati immutabili.
- Un eventuale cache di anteprime resta cancellabile e non è l'unica copia dei dati; un backup verificato deve includere tutti i **derivati storici referenziati**, non solo i profili temporanei.
- Evitare di registrare in log percorsi assoluti, nome del fotografo, metadati EXIF o nomi clienti. L'utente sceglie liberamente la destinazione degli export e dei backup, anche USB/NAS montati.
- **Preferenze opzionali** da discutere: visualizzazione delle informazioni colore; criterio esplicito di produzione dei derivati; nessuna pretesa di calibrazione automatica o monitoraggio hardware.
- **Multilingua:** interfaccia in lingua di sistema, selezione manuale secondo le lingue supportate e fallback inglese; codici tecnici stabili, messaggi traducibili, termini `ICC`, `sRGB` e nomi standard non alterati. I metadati originali in lingua libera non vengono tradotti automaticamente.

## 7. Sicurezza e compatibilità

Profili ICC e container multimediali provengono da file potenzialmente non attendibili. Un'eventuale libreria di parsing/CMM deve imporre limiti di dimensione, memoria e complessità, rifiutare strutture malformate, non risolvere riferimenti esterni e non eseguire contenuto del file. Nessun profilo può diventare un percorso arbitrario di lettura del disco. Verificare file validi/invalidi, compatibilità Windows/macOS/Linux, Tauri WebView e monitor con differenti profili OS.

Le alternative richiedono valutazione tecnica e delle licenze: usare solo le capacità native già disponibili dove sufficienti; CMM locale open source con supporto ICC v2/v4; delegare l'export color-managed a un editor esterno con avviso trasparente. **Nessuna nuova dipendenza è selezionata qui.** Conservare file e backup esistenti; non imporre conversioni a foto storiche né peggiorare la portabilità.

Un profilo colore descrive una codifica, **non** prova che la ripresa fotografica sia fedele al quadro fisico. HDR, canali particolari, proofing CMYK e calibrazione di stampa vanno esclusi inizialmente per evitare false garanzie.

## 8. Rischi e alternative

- **Alto rischio di aspettative eccessive:** la parola «fedeltà» può sembrare certificazione fotografica. Usare «profilo identificato» e «conversione verificata in laboratorio» soltanto se dimostrati.
- **Costo tecnico elevato:** differenze tra decoder, WebView, OS e CMM; una prova a vista non basta. Profili malformati possono esporre bug di sicurezza del parser.
- **Rischio di compatibilità:** derivati già prodotti non devono essere ritrasformati automaticamente o reinterpretati senza consenso.
- **Alternativa più leggera:** documentare chiaramente che le modifiche del Media Center non garantiscono gestione colore professionale, e invitare a usare un editor esterno per immagini destinate alla stampa. È una valida conclusione se il problema non emerge dai test.
- **Alternative di scope:** sola diagnostica colore; diagnostica più conversione dichiarata; contratto completo di gestione colore end-to-end. Scegliere solo dopo test d'uso e compatibilità.

**Complessità indicativa:** ispezione informativa M; pipeline con conversione/anteprime coerenti L–XL; copertura cross-platform, regressioni e sicurezza S/M aggiuntive. Tali stime non costituiscono un piano autorizzato.

## 9. Criteri di accettazione da discutere (test futuri, non eseguiti)

1. Matrìce di file campione **con permesso d'uso**: JPEG/PNG/WebP con sRGB, Display P3, Adobe RGB, profili v2/v4, senza profilo, profilo corrotto, orientamenti EXIF; verifiche con hash e visualizzatori indipendenti.
2. Importare ed elaborare un file non deve alterarne **mai** i byte originali o hash SHA-256; le revisioni e i certificati storici restano riferibili.
3. Dove prevista una conversione, verificare trasformazioni numeriche con immagini di riferimento e strumenti indipendenti; distinguere esplicitamente errore di conversione, differenze di preview e limiti del monitor. Soglie colorimetriche eventuali da concordare, **non inventate**.
4. Un file senza profilo non viene automaticamente dichiarato sRGB; un ICC non valido/non supportato non produce una falsa conferma di riuscita.
5. I file esportati contengono un profilo dichiarato coerente con la trasformazione; file incompleti non compaiono come risultati validi; niente sovrascrittura di file preesistenti.
6. Prove native su **Windows, macOS e Linux**: WebView, diverse impostazioni del display, zoom e tecnologie assistive; anche memoria e tempo su immagini grandi.
7. Backup/ripristino continuano a conservare **originali e revisioni**; i vecchi archivi e gli esportatori CSV rimangono compatibili.
8. Nessuna lettura di rete, fuga di metadati, percorso esterno non autorizzato o trasmissione automatica.

## 10. Fasi ipotetiche e questioni per il proprietario

- **Fase 0 — Evidenza:** raccogliere 5–10 immagini campione legittimamente utilizzabili da artisti/fotografi, osservare un workflow reale di Media Center, confrontare originali, anteprime e derivati su dispositivi differenti.
- **Fase 1 — Specifica minima:** se il problema è confermato, solo informazioni di rilevamento profilo e messaggi di limite; nessuna conversione obbligatoria.
- **Fase 2 — Prototipo isolato e confronto:** valutare almeno due CMM/approcci locali e la gestione corretta di originali, revisioni e output; test e revisione sicurezza/licenze.
- **Fase 3 — Eventuale integrazione, solo previa approvazione:** migliorare preview/modifica/export, documentare compromessi e coordinare il formato dei derivati con TECH-013/TECH-001.

**Domande aperte riservate al proprietario:**
1. Quanti utenti scattano e ritoccano con profili Adobe RGB/P3, oppure destinano le immagini alla stampa?
2. È sufficiente un'avvertenza con rinvio all'editor esterno, o si desidera una gestione colore locale completa?
3. Il principale obiettivo sarebbe coerenza **tra preview ed export**, accuratezza di stampa o semplice diagnosi del profilo?
4. È accettabile mantenere la conversione **opt-in** e un messaggio «non determinabile» per i file senza profilo?
5. Vale la pena introdurre un CMM multipiattaforma, con il costo di distribuzione e manutenzione conseguente?

**Conclusione:** opportunità tecnica plausibile ma non dimostrata; **Proposta — da valutare**, senza sviluppo, approvazione o assegnazioni.
