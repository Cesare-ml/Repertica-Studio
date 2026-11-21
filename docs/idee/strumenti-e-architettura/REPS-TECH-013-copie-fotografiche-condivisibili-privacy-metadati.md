# REPS-TECH-013 — Copie fotografiche condivisibili con controllo dei metadati

- **ID stabile:** REPS-TECH-013
- **Data:** 2026-11-21
- **Stato:** **Proposta — da valutare**
- **Priorità suggerita:** P2 (da rivedere dopo interviste e prova con fotografie reali)
- **Ambito:** strumento tecnico locale e facoltativo; **nessuna implementazione autorizzata**.

## 1. Problema, motivazione e destinatari

Artisti, fotografi d'opera, archivi e gallerie devono spesso inviare fotografie ad acquirenti, curatori, cataloghi o tipografie. Una fotografia JPEG, PNG o WebP può contenere, oltre ai pixel, metadati EXIF/IPTC/XMP o campi testuali: geolocalizzazione, data originale, modello/identificativo del dispositivo, software, commenti, copyright, miniature e altri dettagli. **Non si afferma che le immagini degli utenti contengano effettivamente tali dati**; il rischio esiste quando le copie vengono condivise fuori dall'archivio.

La copia destinata al pubblico dovrebbe poter essere prodotta **senza alterare né sostituire l'originale locale**, evitando che la rimozione di dati privati distrugga anche attribuzioni volute, orientamento o resa cromatica. Il problema è tecnico (byte e contenitori), non il processo editoriale di preparazione di un dossier.

### Evidenza sul repository (lettura di `main`, 21/11/2026)

- `src-tauri/src/commands.rs`, `save_artwork_image_core`: dopo controllo di dimensione e firma JPEG/PNG/WebP, i byte sono salvati nella cartella `media/` senza una fase esplicita di analisi/rimozione dei metadati.
- `src-tauri/src/gallery.rs`, `add_artwork_gallery_image`: comportamento analogo per le fotografie aggiuntive; `artwork_gallery_image_data` restituisce i byte originali codificati per la visualizzazione locale.
- `docs/MVP.md`: originali locali, massimo 20 fotografie aggiuntive per opera e 12 MiB ciascuna; certificati con snapshot immutabile e percorso storico della copertina; backup integrali con fotografie.
- `docs/DATA-PORTABILITY.md`: i sette registri CSV sono **testuali**, non esportano immagini. Non risulta nel codice esaminato un comando dedicato a generare *copie fotografiche per condivisione con metadati controllati*. Questa assenza non equivale alla prova che l'app trasmetta o esponga dati sulla rete: il funzionamento corrente è locale.

**Requisito certo del progetto:** software gratuito/open source, desktop Tauri, offline-first, nessun account o abbonamento, dati locali ed esportabili, cartella liberamente scelta, lingua di sistema con fallback inglese. **Ipotesi da verificare:** la condivisione frequente di foto esterne e la presenza di metadati sensibili nei file tipici degli utenti.

## 2. Scenari d'uso

1. **Artista individuale:** prepara la foto principale da inviare a una galleria, eliminando eventuali coordinate GPS, identificatori del dispositivo e note private, ma mantenendo un credito copyright scelto consapevolmente.
2. **Archivio con molte opere:** produce un lotto di immagini pubblicabili da una selezione, con nome file controllato, senza sovrascritture e con un resoconto degli esiti.
3. **Restauratore/curatore:** conserva l'originale integrale nell'archivio e nel backup; genera separatamente una copia a minore risoluzione per una presentazione esterna.
4. **Caso negativo:** file corrotto, metadati non riconosciuti o libreria che non garantisce il risultato: l'app non dichiara la copia “ripulita”; interrompe l'operazione o segnala esplicitamente una verifica incompleta.

## 3. Specifica funzionale proposta e flussi UX (solo descrittivi)

### Flusso A — singola fotografia

Dalla scheda opera/galleria, l'utente sceglie **«Prepara copia per condivisione»** (azione distinta da sostituire, scaricare backup, modificare copertina o emettere certificato). Una finestra riepiloga: fonte selezionata; categoria dei metadati eventualmente rilevabili; profilo; eventuale ridimensionamento; orientamento; anteprima dell'immagine e delle informazioni che **potrebbero rimanere**. Nessuna credenziale o connessione richiesta.

Profili da valutare, con etichette traducibili:
- **Riservato (predefinito per l'export):** re-encoding locale con elenco chiuso dei metadati consentiti; nessun GPS, identificativo di dispositivo, nota privata, thumbnail incorporata o data di scatto. Conservare informazioni cromatiche indispensabili solo se validate.
- **Professionale controllato:** permette di mantenere o immettere **esplicitamente** credito/autore/copyright e una descrizione pubblica; non copiarli implicitamente dal file. L'app non verifica la titolarità dei diritti.
- Un profilo **«originale invariato»** non dovrebbe usare il verbo «ripulire»: un eventuale export byte-identico deve avvertire che i metadati restano e non può essere il default pubblico.

Il dialogo nativo consente di scegliere qualsiasi cartella scrivibile, anche esterna, proponendo un nome neutro basato sul codice opera o un nome immesso dall'utente; nessuna esposizione automatica di nomi clienti, percorsi originali o note. Anteprima delle collisioni e **divieto di sovrascrivere** file preesistenti. Completamento con controllo del file risultante e messaggio chiaro su ciò che è stato verificato. Si apre la cartella solo su azione dell'utente.

### Flusso B — lotto (fase successiva)

Selezione esplicita di più foto/opere, riepilogo quantità e spazio stimato, anteprima di 2–3 esempi, scelta di una cartella, avanzamento e annullamento. Ogni file ha esito autonomo; un errore non fa sparire gli originali e non produce un lotto apparentemente completo. Risultato consultabile localmente con conteggi e motivi generici degli errori, senza log dei metadati rimossi. Un eventuale rapporto macchina leggibile è **facoltativo**, salvato nella medesima cartella solo con consenso, e privo di dati sensibili.

### Cosa NON fare automaticamente

Nessun upload, invio mail, pubblicazione, integrazione social, correzione della fotografia originale, modifica delle didascalie, variazione del DB, conversione dei certificati storici, cancellazione di file, sostituzione dei backup o applicazione retroattiva a materiali già condivisi. Un avviso che dice «metadati non individuati» non certifica assenza assoluta di informazioni nascoste, né rende automaticamente pubblicabile l'immagine sul piano dei diritti.

## 4. Strutture dati e ipotesi API non vincolanti

La prima versione può produrre **solo file derivati** senza migrazione SQLite. Ipotesi puramente descrittiva di comando locale:

```text
prepare_shareable_images(
  media_refs: [media_id | artwork_cover_ref],
  destination: selected_directory,
  profile: "private" | "professional",
  public_credit?: explicit_text,
  max_pixel_edge?: positive_integer,
  naming: "neutral_code" | "manual",
  optional_report: boolean
) -> {
  successes: [{ source_ref, output_name, dimensions, metadata_check: "passed" }],
  failures: [{ source_ref, reason_code }],
  cancelled: boolean
}
```

I `media_refs` sono identificativi già associati all'archivio attivo, **non percorsi arbitrari forniti dal frontend**. L'API non deve esporre i byte né il contenuto dei metadati non pubblici nei risultati o nei log. Il controllo dei file con symlink/path traversal, permessi e destinazioni appartiene al backend Rust; non si assume che il semplice dialogo nativo basti a prevenire corse o sostituzioni.

Un profilo salvabile in impostazioni, se mai utile, dovrebbe contenere solo preferenze innocue (es. lato massimo), **non dati EXIF, destinatari o indirizzi**. Eventuale schema/versione del rapporto esportato: identificatore di formato e versione, numero file, esito, avviso sulla limitatezza del controllo; nessun `snapshot_json` commerciale o informazioni su collezionisti.

## 5. Offline, protezione degli originali, sicurezza e privacy

- Tutta la decodifica, ricodifica, anteprima e verifica avvengono **sul dispositivo**, senza cloud, account, abbonamento, rete o telemetria; nessun provider esterno è necessario.
- Gli originali restano byte-identici, referenziati da schede, galleria e snapshot dei certificati. I backup completi continuano a conservare **gli originali integrali**: un export “riservato” **non** è un backup e non sana i file già condivisi.
- Preferire una procedura *fail closed*: decodificare e ricodificare i pixel con una allowlist rigorosa di chunk/segmenti, poi riesaminare il file di output. La sola cancellazione di una stringa EXIF non è sufficiente: JPEG può contenere EXIF, IPTC, XMP, profili ICC, miniature e commenti; PNG chunk testuali; WebP chunk EXIF/XMP/ICCP. L'effettiva copertura va dimostrata con fixture, non presunta.
- L'orientamento EXIF deve essere applicato ai pixel prima della rimozione, se necessario: il file finale non deve apparire ruotato. Il profilo colore merita gestione consapevole per non cambiare sensibilmente i colori dell'opera; un profilo ICC può a sua volta contenere metadati. Trattare incompatibilità e file malformati come fallimenti espliciti.
- Limiti di pixel, memoria, durata e dimensione di output contro decompression bomb. Evitare anteprime che leggano metadati pericolosi in un motore non confinato; valutare librerie Rust mature e manutenzione delle dipendenze, **non aggiungerle ora**.
- Scrittura in file temporaneo nella destinazione scelta, sincronizzazione e pubblicazione atomica quando il filesystem lo permette, gestione interruzioni/disk-full, rifiuto di overwrite e symlink; rimuovere solo i **temporanei generati** dall'operazione. Non garantire atomicità cross-volume o di NAS senza verifiche.
- Una foto può mostrare visivamente targhe, indirizzi, documenti o persone: togliere metadati **non** rende anonimi i pixel. L'app deve dirlo chiaramente.

## 6. Impostazioni, multilingua e accessibilità

Rispettare la lingua di sistema e la preferenza dell'app, con fallback completo in inglese. Localizzare pulsanti, messaggi di rischio, categorie di metadati e stati di errore, evitando tecnicismi non spiegati; supportare anche italiano, e per le altre lingue segnalare eventuali traduzioni parziali senza inventare una copertura già completa. Terminologia coerente tra «originale», «copia», «metadati», «credito pubblico» e «backup».

Flusso interamente da tastiera, etichette per screen reader, focus prevedibile, testo non dipendente dal solo colore, conteggi leggibili, scelta profilo non comunicata solo da icona, anteprima con testo alternativo significativo. La scelta del profilo di privacy non deve essere una casella già selezionata con permessi eccessivi.

## 7. Import/export e compatibilità

Non è un nuovo **importatore** e non modifica gli originali importati; sono copiati/re-encodati in file JPEG/PNG/WebP standard, apribili fuori da Repertica. Le copie create non acquisiscono un nuovo ID opera e non vengono aggiunte al catalogo per default. Il naming deve essere portabile tra Windows/macOS/Linux (caratteri vietati, maiuscole, normalizzazione Unicode, lunghezze, collisioni); scegliere se preservare o convertire il formato **solo dopo** verifiche sui colori e sulla trasparenza.

L'import/export semantico TECH-001 rimane diverso: riguarda record e relazioni, non la riservatezza di foto preparate per la pubblicazione. Gli export CSV integrali e i backup verificati rimangono invariati.

## 8. Dipendenze, alternative, rischio e complessità

**Dipendenze concettuali:** accesso sicuro ai media esistenti e dialogo nativo; libreria di decodifica/ricodifica sottoposta a valutazione di sicurezza e license compliance; test nativi su file reali. Nessuna dipendenza installata da questa scheda.

**Alternative da confrontare:**
1. Guida all'uso di software esterno offline e affidabile per eliminare metadati: costo di implementazione minore, UX frammentata.
2. Funzione locale minimale di esportazione di **una** immagine senza ridimensionare: perimetro più testabile, meno prestazioni sul batch.
3. Rimuovere i metadati **all'importazione**: sconsigliato come default perché distruggerebbe informazioni originarie utili al professionista e potrebbe compromettere elementi storici.

**Rischi principali:** falsa garanzia di anonimizzazione; perdita di attribuzione voluta; rotazione o colori alterati; JPEG ricompresso e perdita di qualità; bypass da chunk non supportati; file malevoli; scritture parziali o collisioni su NAS; spazio extra per i derivati; esposizione di dati nel nome del file o rapporto. L'output non è una prova di autenticità, titolarità o licenza.

**Complessità indicativa:** media per una sola immagine con profilo rigido; medio-alta per batch multi-formato, ICC, crash recovery e verifica metadati robusta. **Priorità suggerita P2**, subordinata a evidenza di condivisione esterna frequente e rischi reali.

## 9. Criteri di accettazione e test **ipotetici**, non eseguiti

1. Su fixture **sintetiche** JPEG/PNG/WebP contenenti GPS, camera serial, timestamp, commenti, XMP/IPTC e miniature incorporati, il file di output del profilo Riservato non contiene alcuno dei campi proibiti; prova condotta con *lettore indipendente* dei metadati. Se un formato/contenitore non è supportato, fallisce chiaramente invece di restituire un finto “successo”.
2. SHA-256 degli originali prima/dopo identico, incluso il caso di foto copertina storica usata da un certificato. Nessuna riga di opere, galleria, vendite o certificati cambia; backup dell'archivio continua a includere gli originali.
3. Rotazioni 90/180/270° e profili cromatici tipici resi correttamente; differenze di pixel e colore misurate e riportate, non promessa di identità bit-per-bit del derivato.
4. Test su file troncati, enormi dimensioni dichiarate, chunk non canonici, Unicode nei nomi, percorso selezionato non scrivibile, symlink, cartella scomparsa, disco pieno, due copie con stesso nome, annullamento batch: **nessuna sovrascrittura** e nessun successo falso.
5. Anteprima e risultati accessibili da tastiera e screen reader; lingua OS/fallback inglese; nessuna chiamata HTTP, telemetria o connessione obbligatoria.
6. Un utente può esportare e aprire fuori dall'app un JPEG/PNG/WebP compatibile senza installare Repertica; il nome/rapporto non contiene note su clienti o percorsi privati.
7. Confronto di tempi/memoria su raccolte di 100–1000 fotografie, con soglie da decidere **dopo** benchmarking e caratteristiche reali degli archivi.

## 10. Fasi suggerite (non roadmap autorizzata)

- **Fase 0 — Ricerca:** campionare in forma volontaria foto *sintetiche o autorizzate*, intervistare fotografi, artisti e gallerie; stabilire la matrice dei metadati per formato e i vincoli di conservazione colore.
- **Fase 1 — Prototipo isolato valutabile:** esportazione di una foto, profilo Riservato, verifica indipendente, scelta cartella e nessuna modifica SQLite.
- **Fase 2 — Affidabilità e UX:** test con ICC/orientamento, error handling e accessibilità; eventualmente credito pubblico esplicito.
- **Fase 3 — Lotto facoltativo:** selezione multipla, cancellazione sicura dei soli temporanei, report privo di dati sensibili e benchmark.
- **Fase 4 — Collegamenti opzionali:** se approvate separatamente, integrazione con UX-007 (dossier) e UX-010 (crediti/diritti); nessuna pubblicazione automatica.

## 11. Confini e coordinamento con idee esistenti

- **TECH-003:** verifica di presenza/integrità locale dei media; **qui** preparazione di copie verso terzi e revisione dei metadati.
- **TECH-005:** miniature e cache interna per performance; **qui** file derivato destinato a essere usato *fuori* dall'app, con contratto di privacy e compatibilità.
- **TECH-001:** pacchetto semantico di interoperabilità; **qui** immagini derivate e privacy, senza reimportazione del catalogo.
- **TECH-006/007/009/012:** sicurezza, collocazione, catalogo dei backup e migrazioni, che devono **sempre** preservare originali e metadata di archivio.
- **UX-007:** chi progetta il contenuto condivisibile e il dossier decide *quali opere e quali dati* mostrare; questa scheda riguarda il trattamento sicuro dei **byte delle foto**.
- **UX-010:** annotazioni di diritti, crediti e autorizzazioni; **qui** metadati tecnici su copie di lavoro. Nessuna deduzione o concessione automatica di diritti.

Nessuna scheda del filone gestione e UX viene modificata. Le altre proposte rimangono indipendenti e non approvate.

## 12. Questioni per la revisione del proprietario

1. Con quale frequenza gli utenti inviano realmente originali fotografici a clienti, redazioni o gallerie? Un tool esterno offline sarebbe sufficiente?
2. È preferibile un solo profilo riservato rigoroso o una scelta esplicita tra copia riservata e credito pubblico?
3. Occorre conservare profili ICC, tag di copyright o dati tecnici per tipografie professionali? Come evitare perdite o informazioni personali nel profilo?
4. Ha valore un export in batch nella prima versione o soltanto dopo un caso d'uso documentato?
5. Quale standard di qualità/risoluzione accettare per i derivati? Quando è più sicuro non procedere?
6. Chi valida il linguaggio dell'avviso «metadati rimossi» per impedire false garanzie di anonimato e di diritti?

**Decisione richiesta:** soltanto se e quando il proprietario valuterà questa scheda. **Stato invariato: Proposta — da valutare.** Nessun incarico, approvazione o modifica software.
