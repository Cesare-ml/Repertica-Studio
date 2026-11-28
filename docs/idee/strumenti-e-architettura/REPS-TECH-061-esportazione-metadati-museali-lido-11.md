# REPS-TECH-061 — Esportazione selettiva dei metadati delle opere in LIDO 1.1 per interlocutori museali

- **ID stabile:** REPS-TECH-061
- **Data ricognizione:** 2026-11-23
- **Stato:** **Proposta — da valutare**
- **Filone:** strumenti e architettura; studio **esclusivamente documentale**.
- **Priorità ipotetica:** **P3**, rivedibile a **P2 solo in presenza di almeno un partner istituzionale e di un profilo di consegna verificati**. Nessuna approvazione, pianificazione o incarico allo sviluppo.
- **Principi invarianti:** Repertica Studio desktop open source e gratuita, Tauri/Rust/SQLite, offline-first, nessun account, abbonamento o cloud obbligatorio, esportazione completa e backup libero **separati** da qualunque invio selettivo a terzi; UI multilingue secondo lingua di sistema, fallback inglese.

## 1. Motivazione, destinatari e fatto verificato

Un archivio d'artista o una galleria potrebbe dover **consegnare a un museo un piccolo insieme di descrizioni di opere in un formato riconosciuto dal suo catalogo**. La brochure PDF presenta le opere a persone; i dodici registri CSV di Repertica e il backup trasportano dati del gestionale. Nessuno dei due è automaticamente un record di interscambio museale.

**LIDO (Lightweight Information Describing Objects) 1.1** è uno schema **XML** per lo scambio di metadati di beni culturali, sviluppato dal gruppo ICOM-CIDOC, con supporto a informazioni descrittive, eventi, identificatori e risorse. È un formato di **consegna** di metadati, non un database, un certificato di autenticità, un formato di backup o un sistema di hosting. La versione 1.1 risulta pubblicata il 20 dicembre 2021 secondo la [specifica ufficiale](https://www.lido-schema.org/schema/v1.1/lido-v1.1.html); guida: [LIDO Primer](https://lido-schema.org/documents/primer/latest/lido-primer.html). **Non tutti i musei accettano un LIDO generico**: il destinatario può imporre il proprio profilo, vocabolari, identificatori e ulteriori regole.

**Riscontri su `main` letti il 2026-11-23:** `README.md`, `docs/DATA-PORTABILITY.md`, `docs/MVP.md`, `src-tauri/src/models.rs`, `src-tauri/src/db.rs` e `src-tauri/src/export.rs` descrivono opere con UUID/codice, titolo, anno, tecnica, supporto, misure, descrizione, stato, ubicazione e immagini locali; esistono dodici CSV integrali, backup con SQLite/media e certificati con snapshot storico. Non risulta, nei percorsi esaminati, un esportatore LIDO o una relazione evento-museale già normalizzata. **Questo non prova una richiesta reale né giustifica da solo la funzione.** Altre relazioni, attribuzioni e diritti sono oggetto di **proposte non approvate**, non dati che si possono presupporre già disponibili.

**Destinatari potenziali:** archivi d'artista, fondazioni, gallerie che consegnano schede a musei e uffici catalogo, conservatori esterni con un partner che specifica LIDO. Per artista singolo senza simili richieste, nessuna nuova interfaccia è necessaria.

## 2. Ambito distinto, confronto anti-doppioni

- [TECH-001](REPS-TECH-001-pacchetto-dati-aperto.md): **portabilità completa** e possibile reimportazione dell'archivio, con vendite, collezionisti, media e relazioni. LIDO sarebbe una **proiezione selettiva e asimmetrica** di **sole descrizioni condivisibili**, non un'altra copia completa né un import LIDO.
- [TECH-051](REPS-TECH-051-manifest-iiif-presentazione-opere.md): presentazione di immagini e Canvas a viewer IIIF, che normalmente richiede URL HTTP(S) pubblicati. LIDO comunica **metadati di oggetti** a un catalogo; i due standard potrebbero coesistere, ma nessuno deve forzare l'altro né generare URI falsi.
- [TECH-057](REPS-TECH-057-contratto-schemi-csv-dizionario-dati.md) e TECH-055: dizionario degli attuali CSV e snapshot coerenti dei registri interni; non sono un mapping istituzionale XML.
- TECH-024/030/041: IPTC/XMP dentro le fotografie, vocabolari locali e checklist Object ID; riutilizzabili come fonti **solo se realmente disponibili e con significato noto**, non come conversioni automatiche né sinonimi di LIDO.
- Filone [Gestione e UX — PR #14](https://github.com/Cesare-ml/Repertica-Studio/pull/14): UX-001 (provenienza), UX-010 (diritti), UX-019 (esposizioni), UX-031 (autorialità), UX-041 (testi multilingui), UX-048 (codici esterni), UX-057 (opere digitali) sono **proposte da valutare**, non prerequisiti implementati. Questa scheda definisce **soltanto un eventuale ponte tecnico in uscita** e non aggiunge schermate gestionali per eventi, attribuzioni o diritti.
- **Non** riguarda Repertica Sites, aste online, pubblicazione cloud, API remota, creazione di cataloghi pubblici o import/sync bidirezionale. Nessuna modifica ai file dell'altro filone.

## 3. Scenari realistici e flussi UX proposti

**A. Valutazione senza sviluppo.** Un archivista riceve dal museo un esempio LIDO valido, le regole locali e l'elenco campi obbligatori. Si tenta dapprima una **mappatura manuale** dei CSV/documenti Repertica, con 3–5 opere sintetiche. Se tale consegna è occasionale e gestibile, **non sviluppare** alcun esportatore.

**B. Esportazione mirata, soltanto se il bisogno viene verificato.** L'utente sceglie manualmente alcune opere dall'archivio attivo (non tutte per default) e seleziona un **profilo di destinazione** dichiarato: LIDO 1.1 generico o profilo fornito dall'istituzione, solo se supportato e verificato. Il flusso presenta una matrice **campo sorgente → campo LIDO → valore esportato → disponibilità → confidenza/fonte**, con differenze tra dato certo, non disponibile e inserito manualmente per questa consegna. Niente auto-pubblicazione.

**C. Campi non mappabili.** Se un campo obbligatorio richiede tipologia dell'oggetto, autore attribuito, ruolo o fonte autorevole non desumibile dal gestionale, il programma **non inventa né deduce** tali dati da titolo, tecnica, stato vendita, impostazione globale dell'artista o testo libero. Mostra «dato mancante / necessita revisione». L'utente può annotare un valore **soltanto per la bozza di consegna**, distinguendolo dal record interno; il sistema può rifiutare l'export qualificato come «conforme» se la validazione non passa.

**D. Consegna senza perdita di controllo.** L'utente vede quali titoli, descrizioni, eventuali immagini/URL e identificativi verranno inclusi, esclude ogni voce non pubblicabile, sceglie una cartella locale (USB/NAS anche offline) e conferma. Si genera **un nuovo file XML** più un rapporto di validazione leggibile; nessun invio remoto, nessuna modifica delle schede, nessun PDF/certificato riscritto. Se l'istituzione chiede l'XML via email, l'invio avviene **separatamente e con conferma esplicita**, senza integrazione automatica del modulo Posta.

**E. Riapertura di una consegna.** Su nuova richiesta, una nuova esportazione ricontrolla i dati correnti; eventuali annotazioni del profilo/bozza non devono diventare automaticamente una storia attendibile di eventi, provenienza o proprietà. Un hash del file e l'elenco delle opere esportate consentono confronto, **non** autenticano giuridicamente la scheda.

## 4. Ipotesi di mappatura e contratto dati — non implementati

Il tracciato esatto deve essere costruito dalla **specifica e dall'XSD LIDO 1.1 ufficiali**, non da un inventario artigianale di tag. In particolare, la presenza di un campo Repertica **non implica** che soddisfi i vincoli semantici o di cardinalità LIDO.

| Sorgente Repertica attuale | Possibile destinazione concettuale | Cautela obbligatoria |
| --- | --- | --- |
| UUID opera, codice catalogo | Identificatori del record/oggetto | **Distinguere** identificatore oggetto, identificatore record LIDO e fonte emittente; evitare ID non univoci tra archivi |
| Titolo, eventuale descrizione | Titoli e descrizione pubblicabile | Non assumere lingua del testo dalla lingua UI, non tradurre automaticamente, preservare Unicode |
| Tecnica e supporto | Descrizione materiali e tecniche | Non inventare URI controllati; terminologia libera solo dove il profilo la consente |
| Larghezza, altezza, profondità in cm | Misure dell'oggetto | Specificare unità e tipo di misura; non confondere cm con pixel della fotografia |
| Anno di realizzazione riportato | Dato cronologico, eventualmente evento | Anno testo ≠ data certa dell'evento; non inventare giorno, luogo, intervallo o agente |
| Nome dell'artista nelle impostazioni | **Non** collegamento automatico a creatore/attribuzione di ogni opera | Identità di configurazione non dimostra autorialità, ruolo o certezza dell'attribuzione |
| Percorsi foto locali, galleria, media | Eventuale riferimento a risorsa digitale | Percorsi privati `media/...` **non sono URL pubblici**; niente risorse pubblicate senza scelta e URL reale |
| Stato, ubicazione, vendite, collezionista, prezzi, certificati | **Esclusi dal primo profilo** | Non equivalgono a provenienza accertata o custodia; alto rischio privacy/commerciale |

**Ipotesi strutturale descrittiva**, da non scambiare per XML LIDO già valido:

```text
LidoExportRequest {
  selected_artwork_ids: UUID[],
  profile: {schema: "LIDO-1.1", recipient_profile_id?, profile_revision?},
  publisher_identifier: user_confirmed_local_identifier,
  field_overrides_for_this_export: {artwork_id, field, text, declared_language?, source_note?}[],
  include_resource_links: false_by_default,
  target_directory: user_selected_path
}
LidoExportReport {
  eligible_records: UUID[], blocked_records: [{id, missing_fields, why}],
  omitted_fields: [{id, field, privacy_or_semantic_reason}],
  xsd_validation: pass|fail|not_run,
  recipient_profile_validation: pass|fail|not_available,
  files: [{relative_path, sha256, size_bytes}],
  warnings: string[]
}
```

**API solo ipotetiche:** `inspectLidoMapping(selection, profile) -> Report`; `previewLidoExport(selection, draft) -> Preview`; `validateLidoDraft(draft, schemaBundle) -> Report`; `exportLidoDraft(draft, destination) -> Files`. Non definite né autorizzate; dati lato Rust in sola lettura, e invalidazione dell'anteprima se la selezione, la bozza o i record sorgenti cambiano. L'identificativo stabile del record di scambio deve avere una **politica di namespace esplicita** per archivio/emittente, non usare il codice leggibile come identificatore mondiale.

**Formato di uscita ipotetico:** XML UTF-8 LIDO per i record selezionati, eventualmente accompagnato da `report.json` e `README.md` descrittivi della versione schema, del profilo partner, della copertura dei campi e dei file; **nessun media incluso di default**. XML valido secondo XSD ≠ corretto semanticamente ≠ accettato dal museo: controlli distinti.

## 5. Offline, sicurezza, privacy, impostazioni e multilingua

- **Offline:** dati SQLite e media locali; schemi XSD, eventuali Schematron/profili di controllo disponibili **in copia versionata locale** con licenze e provenienza verificate. La validazione non scarica DTD, XSD, font o vocabolari al momento della consegna; nessuna connessione necessaria. Nessun account Repertica né endpoint remoto.
- **Sicurezza parser/export:** costruire XML con escaping corretto dei caratteri e serializzatore sicuro, vietare risoluzione di **entità esterne**, DTD remote e XInclude negli input/profili; limiti per file, profondità, stringhe e record; niente esecuzione di template o codice del partner. Validare i profili prima dell'uso, senza trattare file arbitrari come attendibili.
- **Privacy minimizzata:** escludere per default **collezionisti, recapiti, indirizzi, note interne, vendite, quote, importi, spese, dati di nascita, percorsi del computer, credenziali e RAW**. Gli eventuali identificativi delle opere possono essere informazione non pubblica; riepilogo esplicito di ogni campo prima di salvare. File XML in chiaro: avvertire in cartelle condivise; mai telemetria o upload implicito. Cessione dei diritti fotografici **non** deducibile dall'esistenza di immagini in archivio.
- **Impostazioni:** nessuna opzione globale attiva di default; profilo di esportazione scelto per operazione, preferenza di cartella facoltativa solo locale, reset disponibile. Se si consente di salvare una bozza, va inclusa nei backup solo con scelta progettuale esplicita e senza segreti; nessuna licenza Creative Commons impostata automaticamente.
- **Multilingua:** tutte le istruzioni, i messaggi di controllo e gli errori seguono la lingua OS con fallback inglese; `xml:lang` dei **contenuti** riflette la lingua dichiarata o verificata **del testo**, non il locale di sistema. Chiavi/tag XML LIDO e codici di controllo invarianti, indipendenti dalla lingua dell'interfaccia.
- **Compatibilità:** Windows/macOS/Linux; UTF-8, file esterni non sovrascritti, errori/dischi pieni non devono produrre file presentati come completi. L'archivio originale e i backup attuali restano immutati; eventuali nuove versioni di schema/profilo devono essere dichiarate e testate separatamente.

## 6. Dipendenze progettuali, alternative e rischi

**Dipendenze necessarie solo in caso di approvazione futura:** contratto attuale degli UUID e delle opere, esportatore Rust, i18n Angular, XSD ufficiale LIDO 1.1 con dipendenze locali e licenza documentata; un **partner che espliciti il proprio profilo**. Alcune semantiche (autorialità, diritti, provenienza, vocabolari controllati) potrebbero richiedere dati che il modello attuale non possiede: **non implementarle surrettiziamente** sotto forma di export. Prima verificare se siano evitabili, se possano essere fornite temporaneamente in bozza, o se la consegna non sia praticabile.

**Alternative preferibili da provare prima:** (1) CSV esistenti + tabella di mappatura manuale; (2) consegna di una brochure o di schede individuali in PDF se il museo le accetta; (3) convertitore **esterno e libero** gestito dall'istituzione partendo da un sottoinsieme di CSV; (4) semplice modello di raccolta metadati concordato con il museo. TECH-001 rimane la via per portabilità integrale. **Non adottare LIDO soltanto perché è uno standard noto.**

**Rischi:** record tecnicamente valido ma falso (creatore e produzione dedotti), URI e codici duplicati, omissioni silenziose, diritti fotografici non verificati, fuga di recapiti/prezzi, schema XSD valido ma profilo specifico rifiutato, aggiornamenti incompatibili, XML parser con entità esterne, eccessiva complessità per studi piccoli. **Complessità alta** per mapping semantico e validazione/qualità dei dati; **media** per una prima prova manuale senza sviluppo.

## 7. Criteri di accettazione, test e fasi eventuali

**Prima del software (fase 0):** almeno un archivio/galleria e un'istituzione consegnano **specifica di scambio reale**, esempi validi, campi obbligatori e criteri di accettazione. Verificare se i CSV/una mappatura manuale bastino; se sì, archiviare la proposta senza sviluppo. Il proprietario approva o respinge personalmente.

**Fase 1 (solo dopo decisione):** definire una matrice di corrispondenza verificabile su 3–5 opere **sintetiche**: opera semplice, metadati mancanti, titolo Unicode multilingue, dati non pubblicabili, attribuzione incerta. Validare manualmente XSD ufficiale e profilo destinatario; registrare lacune e scelte, senza modificare dati reali.

**Fase 2 ipotetica:** anteprima di esportazione, selezione esplicita di campi, XML generato offline con validazione, rapporto e directory scelta liberamente; file esistenti intatti, nessun invio. Valutare un profilo minimale **soltanto se il destinatario lo accetta**.

**Fase 3 ipotetica:** eventualmente più profili/versioni museali con mapping dichiarati, senza diventare un catalogatore universale. Import LIDO, IIIF automatico, pubblicazione OAI-PMH e sincronizzazione **fuori ambito**.

**Criteri testabili per un'eventuale implementazione:**
1. Validazione **XSD 1.1 offline** (qui significa schema *LIDO v1.1*, non XSD language 1.1) e validazione specifica del profilo partner quando disponibile; XML non validato mai mostrato come conforme.
2. Titoli Unicode, diacritici, note con caratteri XML speciali, testi vuoti, anno ambiguo, misure non presenti, codici simili e collisioni di namespace gestiti senza inventare campi o alterare i record.
3. Nessun contatto privato, importo, path assoluto o dato non selezionato in XML, rapporto, metadati del file o cartella di output; verifica anche di file di prova avversari.
4. Se mancano campi obbligatori l'export qualificato come conforme è bloccato **prima** di creare un output definitivo; l'utente può produrre soltanto un **rapporto di lacune**.
5. Esportazione ripetuta con stessi dati/profilo produce valori semantici stabili, con data di export separata; nessuna sovrascrittura di file e comportamento prudente su disco pieno/USB/NAS.
6. L'app e i backup continuano a funzionare **senza rete**, in italiano, inglese e con locale non supportato (fallback inglese); cambiamento della lingua UI non altera automaticamente lingua/certificazione dei dati.
7. Un destinatario reale riesce a **importare** (o convalidare secondo il proprio profilo) almeno un pacchetto di prova; la sola validità sintattica non basta.

## 8. Requisiti certi, ipotesi e questioni aperte al proprietario

**Certo dal repository:** archiviazione e backup locali, dodici CSV integrali non reimportabili, schede opera con metadati e fotografie, UUID e codici, contatti e registri economici privati; nessun esportatore LIDO individuato nei file esaminati. **Certo dalla fonte esterna:** esiste LIDO 1.1 con XSD e guida ufficiali; non segue automaticamente che sia un'esigenza della clientela Repertica.

**Ipotesi da validare:** esistenza di clienti istituzionali, selezione di un sottoinsieme di campi, bozza temporanea con override, rapporto di copertura, supporto a uno specifico profilo museale. Le API e i formati complementari descritti qui sono puramente propositivi.

**Domande per la revisione del proprietario:**
1. È arrivata una richiesta concreta da museo/fondazione? Quale **versione/profilo**, e quali campi non negoziabili?
2. Il partner accetterebbe **CSV e una mappatura esterna**, evitando una funzione permanente dentro Repertica?
3. Quali campi descrittivi autorizzare **di default** alla consegna, e chi controlla il consenso/diritti per immagini e testi?
4. Come identificare emittente e record LIDO in modo stabile **senza** associare ogni opera all'identità configurata dell'utente?
5. Si accettano override **solo temporanei** non salvati nel catalogo o occorre una specifica nuova dei dati, da discutere nel filone Gestione e UX?
6. Quale prova con un destinatario indipendente giustificherebbe la priorità P2, e quale risultato porterebbe a **non sviluppare**?

### Fonti esaminate (23 novembre 2026)

- Repertica Studio `main`: `README.md`, `docs/DATA-PORTABILITY.md`, `docs/MVP.md`, `src-tauri/src/models.rs`, `src-tauri/src/db.rs`, `src-tauri/src/export.rs`.
- Proposte tecniche REPS-TECH-001, -024, -030, -041, -051, -055, -057 e indice complessivo REPS-TECH-001…060; indice Gestione e UX REPS-UX-001…065 della PR #14, **non modificato**.
- [ICOM-CIDOC — LIDO v1.1 Schema](https://www.lido-schema.org/schema/v1.1/lido-v1.1.html), [XSD LIDO v1.1](https://www.lido-schema.org/schema/v1.1/lido-v1.1.xsd), [LIDO Primer](https://lido-schema.org/documents/primer/latest/lido-primer.html), [ICOM — panoramica del formato](https://icom-documentation.mini.icom.museum/working-groups/lido/lido-overview/lido-schema/).

**Esito: documento ideativo; nessuna modifica a software, database, test, dipendenze, workflow o configurazioni. Nessuna idea approvata o assegnata. PR documentale in bozza, non effettuare merge.**
