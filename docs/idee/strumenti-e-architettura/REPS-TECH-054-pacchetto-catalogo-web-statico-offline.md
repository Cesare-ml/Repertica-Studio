# REPS-TECH-054 — Pacchetto locale per cataloghi web statici, senza pubblicazione automatica

- **ID stabile:** REPS-TECH-054
- **Data ricognizione:** 2026-11-23
- **Stato:** **Proposta — da valutare**
- **Filone:** Strumenti e architettura — analisi e documentazione soltanto
- **Priorità proposta:** **P2 condizionata** a frequenti aggiornamenti manuali di cataloghi web da parte di artisti/gallerie; **P3 o nessuna nuova funzione** se bastano CSV, brochure e preparazione manuale.
- **Decisione del proprietario:** nessuna. Non approvata, non assegnata, non autorizza implementazioni o una specifica pubblica definitiva.

## 1. Problema, utenti e verifica del punto di partenza

Un artista può gestire opere e fotografie in locale e, separatamente, mantenere un sito vetrina creato con un generatore statico, un collaboratore oppure un assistente IA scelto liberamente. Copiare a mano titoli, tecniche, misure, testi e fotografie dal gestionale al sito a ogni aggiornamento genera doppio lavoro e può introdurre errori. Un'esportazione completa, invece, sarebbe pericolosa: il sito non deve ricevere clienti, vendite, indirizzi, note interne, ubicazioni o certificati riservati.

**Destinatari potenziali:** artisti con siti personali, gallerie con cataloghi pubblici, archivi d'artista e piccoli studi che consegnano dati a un webmaster. Non è un bisogno dimostrato da interviste: l'opportunità è da verificare.

**Evidenze lette su `main`, 23/11/2026:**
- [README](../../../README.md) e [MVP](../../MVP.md): Repertica Studio è Tauri/Angular/Rust/SQLite, gratuito, senza account, offline-first; **Sites non è un modulo del desktop MVP** e non ci sono API obbligatorie verso servizi online.
- [DATA-PORTABILITY](../../DATA-PORTABILITY.md): i **12 CSV** sono esportazioni **integrali**, anche quando l'interfaccia applica filtri; includono registri commerciali e dati personali. Non trasportano i byte delle fotografie.
- [BROCHURES-OFFLINE](../../BROCHURES-OFFLINE.md): è già presente un editor per brochure PDF, con opere e foto selezionate e testo personalizzabile. Il PDF è un prodotto editoriale, non una sorgente dati per ricostruire pagine web.
- [media.rs](../../../src-tauri/src/media.rs) espone per le fotografie titoli, didascalie/descrizioni, alt text, crediti e uno stato dichiarato di copyright, oltre a revisioni. Questi campi **non sono un'attestazione di diritti di pubblicazione**.
- Nei percorsi esaminati non risulta un **pacchetto locale di dati e immagini deliberatamente destinati a un catalogo web statico**. Questo non prova l'assenza di strumenti manuali esterni né costituisce un difetto accertato.

**Requisiti certi ereditati dal progetto:** dati locali, esportazione completa sempre disponibile senza limitazioni, backup in una cartella liberamente scelta, funzionamento offline, nessun login/abbonamento, nessun cloud o provider IA imposto, multilingua con lingua di sistema e fallback inglese. **Ipotesi:** uno scambio strutturato ridurrebbe tempi ed errori rispetto alle esportazioni esistenti.

## 2. Confini e non duplicazione

- **[TECH-001](REPS-TECH-001-pacchetto-dati-aperto.md):** portabilità **integrale**, anche con reimport dei dati gestionali. Qui: **proiezione pubblicabile** di sole opere/foto autorizzate, in sola uscita e **non reimportabile** nel database.
- **[TECH-025](REPS-TECH-025-esportazioni-tabellari-minimizzate-condivisione.md):** CSV selettivi per analisi e condivisione, senza file immagine. Qui: un contratto JSON **orientato alla presentazione pubblica** con copie di foto selezionate, manifest e check prima della consegna. Riutilizzare eventuali allowlist comuni.
- **[TECH-013](REPS-TECH-013-copie-fotografiche-condivisibili-privacy-metadati.md) / [TECH-024](REPS-TECH-024-metadati-fotografici-iptc-xmp-interoperabili.md):** sicurezza dei byte/EXIF e metadati fotografici. Non duplicare il loro motore; una copia non analizzabile in sicurezza non va qualificata come «bonificata».
- **[TECH-029](REPS-TECH-029-scambio-selettivo-schede-opere-archivi-indipendenti.md):** consegna e possibile import in **altri archivi Repertica**. Qui nessun collegamento fra basi dati, import, merge, sincronizzazione o trasferimento di proprietà.
- **[TECH-051](REPS-TECH-051-manifest-iiif-presentazione-opere.md):** manifest istituzionali IIIF con requisiti HTTP(S). Qui: **sorgenti locali per qualunque sito statico**, senza dichiarare conformità IIIF né richiedere URL pubblici.
- **[Gestione e UX — PR #14](https://github.com/Cesare-ml/Repertica-Studio/pull/14):** UX-007 dossier/brochure, UX-010 diritti dichiarati, UX-041 testi di presentazione multilingua, UX-053 cartellini e UX-054 presentazione riservata **a schermo**. Questa scheda riguarda soltanto un **confine di interscambio tecnico**; non inventa titolarità, nuovi contenuti editoriali, licenze, workflow di vendita né editor visuale.
- **Repertica Sites eventuale:** può essere **un possibile consumatore**, non una dipendenza esclusiva. Il pacchetto dev'essere utilizzabile anche da siti artigianali, generatori statici, script propri o IA di libera scelta. Nessun editor di siti nel desktop e nessuna API remota introdotta di default.

## 3. Scenari d'uso reali da sottoporre a verifica

1. **Artista con sito vetrina:** seleziona 15 opere già catalogate e due fotografie per ciascuna, corregge prima della consegna i testi che saranno pubblici e salva una nuova cartella locale da consegnare al webmaster.
2. **Aggiornamento periodico:** esporta una seconda selezione mantenendo, se desiderato, identificativi pubblici stabili per opere già note al sito. Il pacchetto dichiara le modifiche rispetto al **precedente snapshot di pubblicazione scelto**, ma **non aggiorna** alcun sito e non presume che opere non incluse siano da eliminare.
3. **Galleria con incarichi a terzi:** riceve soltanto un catalogo pubblico leggibile senza Repertica. Nessuna informazione sugli acquirenti o sulle percentuali economiche viene consegnata.
4. **Caso critico — opera venduta o rimossa:** lo stato nell'archivio cambia dopo un export; il sito pubblicato altrove **non cambia automaticamente**. Repertica non può promettere revoca di fotografie o rimozione di vecchi file una volta che sono stati condivisi.

## 4. Specifica funzionale e flussi UX ipotetici

### Fase minima: esportazione manuale una tantum

1. **Entrata separata:** da «Opere → Esporta per catalogo web (bozza)», distinta visivamente da «Backup integrale», «CSV completo» e «Brochure PDF». Nessun invio automatico.
2. **Selezione esplicita:** scegliere opere singole o una raccolta, vedere numero effettivo e criteri. Non interpretare implicitamente tutti i filtri o tutte le opere come autorizzazione alla pubblicazione.
3. **Campi pubblici in allowlist:** codice/titolo/anno/tecnica/supporto/dimensioni; descrizione solo dopo revisione. Altri campi solo con gesto esplicito. **Default esclusi:** clienti, acquisti, vendite, importi di vendita, costo materiali, fondi, pagamenti, numeri di certificato, contatti, ubicazioni private, note libere non riviste, percorsi macchina, ID gestionali che espongano altri dati, file RAW e metadati EXIF originali.
4. **Immagini:** scegliere singolarmente copertina/revisioni/foto aggiuntive. Richiedere al produttore dei derivati una modalità che rimuova metadati privati e verifichi l'esito; dove non è dimostrabile, bloccare la qualifica «pronta da pubblicare» oppure offrire un export esplicitamente **non verificato**, non il silenzio. Alt text e crediti possono essere forniti o revisionati, mai inventati. Stato «copyright sconosciuto» richiede revisione, non genera automaticamente licenza.
5. **Anteprima prima della scrittura:** mostrare esattamente le opere, tutte le proprietà e immagini che entrerebbero nel file, segnalando testo libero, dati mancanti, possibili diritti non chiariti e numero di byte. Conferma separata del contenuto pubblico.
6. **Esito offline:** scelta della cartella (anche USB/NAS montato); nuova sottocartella, scrittura temporanea, verifiche su tutti i file, pubblicazione atomica dove supportata dal filesystem; se il supporto non garantisce atomicità, non dichiararla. Mai sovrascrivere un pacchetto o l'archivio attivo. Manifest leggibile e rapporto degli esclusi.
7. **Trasferimento volontario:** l'utente consegna i file manualmente, li dà a un generatore statico, a un collaboratore o a un assistente IA **fuori da Repertica**. Una scelta successiva di caricarli in rete è indipendente e richiede un'azione fuori dal flusso.
8. **Annullamento/errori:** eliminare soltanto i temporanei creati dall'operazione, mai contenuti già esistenti; nessun pacchetto parziale presentato come completo.

### Fase eventuale: confronto fra due snapshot di pubblicazione

Solo se utile a chi aggiorna spesso il sito: su **due cartelle già esportate e scelte dall'utente**, leggere manifest verificati e mostrare opere/immagini aggiunte, cambiate, non più incluse. «Non più inclusa» **non significa** eliminata, venduta, resa non pubblicabile o da rimuovere dal sito: richiede conferma separata sul sistema di pubblicazione. Non fare sync, watcher o pubblicazione in background.

## 5. Strutture dati / API *solo descrittive*

Esempio di file di **interscambio indipendente** (nome e versione da discutere, non contratto approvato):

```text
Repertica-web-catalog-<timestamp>-<random>/
  README.md                       # come leggere il pacchetto, limiti e licenze dei dati
  manifest.json                   # format_id, format_version, exported_at, file hashes, counts
  artworks.json                   # SOLO campi pubblici selezionati, ordinamento, riferimenti relativi
  media/
    <opaque-public-id>-1.jpg
    <opaque-public-id>-2.webp
```

**Schema concettuale di un record** (non JSON vincolante):

```text
PublicArtworkV1 {
  public_id: string,                 # opaco; stabile solo entro la politica scelta
  title: string,
  year?: string,                     # non inferire date esatte da anni incerti
  technique?: string,
  support?: string,
  dimensions_cm?: { width?: number, height?: number, depth?: number },
  description?: string,              # solo testo revisionato per uso pubblico
  images: [{
    relative_path: string,
    alt_text?: string,
    credit?: string,
    width_px: integer,
    height_px: integer,
    sha256: string,
    mime: string
  }]
}
```

**Regole per il contratto:** versione dell'**export** indipendente da `PRAGMA user_version`; JSON UTF-8, ordinamento deterministico, date/istanti espliciti, unità di misura e campi facoltativi definiti. Percorsi sempre relativi e canonici; nessuna URL remota obbligatoria, nome file originario, percorso assoluto, token o credenziale. Hash SHA-256 come verifica di integrità **non come firma o prova dei diritti**. Il sito consumatore deve trattare testi/JSON come **dati non attendibili**, con escaping HTML e senza eseguire contenuti o istruzioni incorporati (allineamento concettuale con TECH-049).

**Identificatori pubblici:** *non* rendere pubblici automaticamente UUID di clienti, vendite o archivi. Per le opere valutare ID opachi separati e una mappatura locale esplicita e facoltativa per mantenerli fra esportazioni; in fase stateless i link non sono necessariamente stabili. Questa è una decisione progettuale aperta, non un requisito già implementato.

**API solo illustrative:** `previewWebCatalog(selection, publicFields, mediaChoice) -> DisclosureReport`; `exportWebCatalog(selection, profile, destination) -> VerifiedManifest`; `comparePublicSnapshots(folderA, folderB) -> DiffReport`. Nessuna API per `publish`, `sync`, `deleteRemote` o `importToStudio`.

**Standard opzionali, non prerequisiti:** un consumer può trasformare questi dati in JSON-LD [Schema.org VisualArtwork](https://schema.org/VisualArtwork) (tipologia con proprietà quali `artMedium`, `artworkSurface` e misure); non sostenere che ogni dato Repertica abbia corrispondenza esatta o che il mapping garantisca SEO/indicizzazione. Un file JSON semplice rimane leggibile senza framework o servizio dedicato.

## 6. Offline, sicurezza, privacy, impostazioni e lingue

- **Offline garantito per creazione e verifica**: nessun DNS, HTTP, login, plugin proprietario, servizio Repertica Sites o provider IA necessario. Non aprire browser e non inviare richieste a URL presenti nel testo dell'utente.
- **Confine di divulgazione:** allowlist applicata dal backend; anteprima e file finale devono concordare. Mai serializzare l'intero oggetto SQLite/Angular e nascondere solo in pagina i campi privati. Cercare dati personali anche in didascalie, descrizioni e nomi file; nessun classificatore automatico può garantire l'assenza di dati sensibili.
- **Filesystem:** prevenire symlink, path traversal, nomi riservati/collisioni case-insensitive/Unicode, file interrotti, archivi troppo grandi e overwrite. Limitare ogni sorgente ai media effettivamente legati alle opere selezionate; non fidarsi di percorsi dal frontend. Verificare checksum dei derivati prodotti.
- **Diritti:** stato «owned/licensed/public domain» in Media Center è una *dichiarazione locale*, non un titolo giuridico verificato; nessuna licenza inventata, watermark automatico o pubblicazione implicita. Segnalare ove mancanti crediti e autorizzazioni.
- **Impostazioni:** prima fase senza nuovi parametri persistenti; eventuali profili locali di campi consentiti richiedono nome comprensibile, revisione per ogni export e nessuna casella «pubblica tutto» pre-selezionata. Cartella scelta liberamente, non riutilizzata per upload silenziosi.
- **Multilingua:** UI e messaggi nella lingua di sistema fra quelle supportate, inglese di fallback; etichette di schema stabili in inglese. La lingua dei testi pubblici dell'opera va **esplicitamente selezionata**; assenza di traduzione non autorizza traduzione automatica o interpretazione dei testi originali (eventuale UX-041 è separata). Alt text revisionato, anteprime e report accessibili con tastiera/screen reader.
- **Conservazione:** il pacchetto esportato **non è backup** e non è incluso automaticamente in backup successivi di Studio; il contenuto già pubblicato non può essere revocato cancellando l'export locale.

## 7. Dipendenze, alternative, rischi, complessità

**Dipendenze eventuali:** API di consultazione selettiva delle opere e immagini esistenti; derivati condivisibili da TECH-013 se approvati; gestione dei percorsi/manifest e verifica hash; modello di presentazione pubblica da decidere. **Nessuna dipendenza tecnica da Sites, da un CMS, da IA o da un server**. Possibile riuso di contratti sviluppati per TECH-001/025, senza nuovi formati incompatibili.

**Alternative da provare prima di sviluppare:**
1. CSV `artworks` già presente + foto esportate manualmente, **dopo rimozione delle colonne private**, con il rischio documentato di errore umano;
2. brochure PDF già disponibile quando basta una presentazione statica;
3. preparazione di un file JSON/CSV minimale con LibreOffice o uno script locale scelto dall'utente; per siti occasionali può essere sufficiente;
4. gestione di contenuti e foto direttamente nel generatore del sito, senza integrazione con Studio.

**Rischi principali:** fuga di dati nei testi liberi o EXIF; uso di fotografie senza diritti; snapshot obsoleti sul sito; confusione fra «esportato» e «pubblicato»; URL instabili; perdita di identità pubblica in esportazioni stateless; aumentare la complessità del desktop per un bisogno raro; discrepanza fra anteprima e byte finali; costi di manutenzione di un contratto per siti eterogenei.

**Complessità indicativa:** **M** per una cartella statica JSON + immagini già sicure; **L** se includere profili salvati, versionamento pubblico stabile e confronto snapshot, oltre a verifiche approfondite di privacy e accessibilità. La bonifica reale dei metadati fotografici può richiedere lavoro autonomo e non va sottostimata. Non stimare tempi prima di un prototipo e di casi concreti.

## 8. Test/criteri di accettazione *proposti*, non eseguiti

1. Su archivio sintetico con opere, clienti, prezzi, incassi, note private, ubicazioni e certificati, **nessuna stringa/ID riservata** compare in alcun file di output, nome file, manifest o report del pacchetto minimo.
2. Esportazione di **due opere e tre immagini scelte**: riferimenti relativi risolti, conteggi corretti, byte/hash validi, nessun media non selezionato. Nessun originale modificato.
3. Test di **privacy negativa**: testo libero contenente telefono/email e foto con GPS/EXIF; l'anteprima lo segnala, non assegna diritti e non dichiara una copia «ripulita» senza verifica dei byte.
4. Destinazione su cartelle con caratteri Unicode e supporti esterni: annullamento, collisione di nomi, spazio esaurito, symlink e scrittura interrotta non alterano archivio e file preesistenti.
5. Un consumer indipendente legge `artworks.json` e mostra opere/foto **senza avviare Repertica, senza Internet e senza specifiche proprietarie nascoste**; nessuna promessa di browser funzionante tramite doppio clic `file://` se il generatore prescelto richiede un server locale.
6. Schema versionato rifiuta versioni non supportate **senza interpretarle come dati corrotti**; hash mancante o non conforme viene segnalato.
7. Sul secondo snapshot, differenze e assenze sono indicate senza cancellare automaticamente file precedenti o dati nell'archivio; nessuna modifica di vendita, stato, certificato o accesso alla rete.
8. Tasti, focus, lettore schermo, lingue IT/EN/ES/FR/DE, fallback EN e Unicode nei titoli passano prove su Windows e macOS installati; supporto alla lingua del documento separato dalla lingua UI.

**Prova di utilità prima di approvare:** simulare 4–6 trasferimenti verso siti gestiti con strumenti diversi, misurando tempo, errori di campo/fotografia e necessità di aggiornamenti successivi. Se CSV+foto risolvono il problema, sospendere questa proposta.

## 9. Fasi ipotetiche e questioni aperte per il proprietario

- **F0 — Validazione senza codice:** interviste e prova manuale con artisti, webmaster e piccoli archivi; definire quali campi sono davvero pubblici; confrontare CSV e brochure già esistenti.
- **F1 — Solo se approvata successivamente:** una esportazione stateless manuale, senza database/migrazioni, con anteprima, campo ammesso e pacchetto verificato. Nessun collegamento automatico a Sites.
- **F2 — Facoltativa:** compatibilità esplicita con più generatori statici, mapping JSON-LD e fotografie derivate verificate; test reali di privacy e accessibilità.
- **F3 — Solo a domanda dimostrata:** identificatori pubblici stabili, profili locali revisionabili, confronto di due snapshot **senza sync**.

**Domande non risolte:** i destinatari chiedono JSON, CSV selettivo o basta una cartella di JPEG? Serve continuità del permalink tra export, e chi ne governa l'identità? Sono desiderabili prezzi di listino *pubblici* come scelta separata, distinguendoli rigorosamente dai prezzi di vendita? Quali crediti/diritti sono effettivamente accertati? Serve il supporto a più lingue del catalogo oppure un solo contenuto per esportazione? Si vuole davvero che un futuro Repertica Sites consumi il formato, o è preferibile mantenerlo generico?

**Esito della ricognizione:** sola proposta documentale, senza codice, dati reali, test, issue, approvazione, incarico di sviluppo, pubblicazione o merge. Lo stato resta **«Proposta — da valutare»**.
