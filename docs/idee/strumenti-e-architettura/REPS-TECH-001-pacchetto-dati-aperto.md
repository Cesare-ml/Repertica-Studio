# REPS-TECH-001 — Pacchetto aperto per portabilità completa e reimportazione verificabile

- **Data:** 2026-11-21
- **Stato:** **Proposta — da valutare**
- **Priorità proposta:** **P1–P2**, da validare; libertà di uscita dai dati importante, non urgenza implementativa.
- **Ambito:** interoperabilità *offline* fra archivi Repertica e programmi terzi. **Non** backup, **non** import da uno specifico gestionale, **non** sincronizzazione.

## Problema osservato e destinatari

Gli artisti e i professionisti devono poter lasciare un software o scambiarsi un archivio senza dipendere da una versione dell'app. Nel [README](../../../README.md), [MVP](../../MVP.md) e in [ECONOMY-I18N](../../ECONOMY-I18N.md) sono già presenti un **backup completo** (SQLite + media + manifest SHA-256) e due **CSV** per vendite e spese. Il primo è adatto al *ripristino fedele* di Repertica; i secondi sono registri tabellari **non reimportabili** e non includono tutti i domini. Non è documentato un contratto semantico aperto, leggibile da altri software, per opere, collezionisti, vendite, certificati, spese, impostazioni economiche e media.

**Destinatari:** artisti singoli, gallerie e archivi in migrazione, consulenti che gestiscono dati per conto di un artista, conservatori che necessitano di accesso futuro anche senza il programma originale.

## Scenari, specifica e flusso UX ipotizzati

1. **Esportazione portabile:** Impostazioni → Dati e portabilità → «Esporta archivio aperto». Informativa sulle informazioni personali incluse, selezione di una cartella libera (USB/NAS montato inclusi), riepilogo entità e spazio stimato, esecuzione con avanzamento e verifica. Risultato in **nuova cartella**, mai sovrascrivere.
2. **Ispezione senza Repertica:** chi riceve il pacchetto consulta `README.md` o il manifest e file `JSONL` UTF-8 leggibili da strumenti generici; foto originali conservate senza ricompressione. Tutti i riferimenti fra entità sono UUID stabili.
3. **Prova di reimportazione:** da menu «Verifica/Importa pacchetto» si apre una **simulazione**: versione, conteggi, checksum, relazioni, codici e seriali in conflitto, dimensioni. **Prima fase** soltanto in un **nuovo archivio vuoto** o in un'area temporanea isolata; nessuna fusione silenziosa con l'archivio corrente.
4. **Eventuale migrazione/merge, fase futura:** mappa esplicita degli ID, proposta di conflitti e approvazione record per record o per categoria. Nessuna riscrittura dei certificati già emessi; non trattare due opere con stesso titolo come identiche.
5. **Esito verificabile:** report locale che confronta esportazione e reimportazione, contatori per dominio, identificatori persistiti, riferimenti e hash media; errori non rendono utilizzabile un archivio parzialmente importato.

**Relazione con [issue #5](https://github.com/Cesare-ml/Repertica-Studio/issues/5):** l'issue descrive un esportatore specifico dal vecchio Gestionale Opere. Questa idea definisce un formato indipendente dalla sorgente che *potrebbe* essere utilizzato anche da quell'esportatore, previa approvazione; non sostituisce né implementa la migrazione.

## Ipotesi di modello dati / contratto — solo descrittivo

Struttura *ipotetica*, **non** schema approvato né file creati:

```text
Repertica-open-export-YYYYMMDD-<id>/
  manifest.json             # format_id, format_version, producer, exported_at, inclusions
  README.md                 # istruzioni, licenza del formato e avvertenze privacy
  data/artworks.jsonl
  data/collectors.jsonl
  data/sales.jsonl
  data/certificates.jsonl
  data/expenses.jsonl
  data/studio-settings.json # valutare omissione dei parametri locali non trasferibili
  media/<nome-sicuro>
```

- `manifest`: versione **dell'interscambio** separata da `PRAGMA user_version`, charset, checksum SHA-256 e byte dei file, identificativi dei domini, schema delle unità; `created_at` in UTC. Eventuali ampliamenti del formato vanno documentati con compatibilità e migrazioni.
- Valori denaro in **centesimi interi** con `currency`; quote in basis points; misure in **centimetri**; date evento `YYYY-MM-DD` quando non sono istanti, timestamp UTC quando lo sono; null distinto da stringa vuota. Nessuna conversione locale irreversibile.
- Esportare anche `snapshot_json` **immutabile** dei certificati, numero seriale e vincoli documentali; contatori/seriali vanno convalidati per impedire emissione di duplicati al successivo utilizzo.
- `media`: riferimento relativo e SHA-256, mai path assoluti, URL esterni obbligatori, symlink o credenziali. Specificare se si esportano i file inutilizzati (decisione aperta).
- Conservare gli identificatori originali e dichiarare esplicitamente le informazioni *non esportabili* (ad esempio preferenze macchina); nessuna traduzione automatica dei contenuti immessi dall'utente.

**API concettuale**, non interfaccia implementata: `inspectOpenPackage(path) -> Report`; `exportOpenPackage(folder, scope) -> Manifest`; `dryRunImport(path, target=empty) -> ImportPlan`; `commitImport(plan) -> Result`. Un `plan` va invalidato se i file o il DB cambiano prima dell'applicazione.

## Offline, sicurezza, impostazioni, lingua

- Nessuna API remota, login, SaaS o dipendenza da un fornitore. UI nella lingua di sistema, fallback inglese; i **nomi delle chiavi del formato** restano stabili e inglesi; documentazione di scambio accessibile anche a chi non usa l'app.
- Privacy: dati di clienti, prezzi e note possono essere sensibili, il pacchetto è **in chiaro** se non diversamente progettato. Mostrare dimensioni, categorie e rischi prima del salvataggio; evitare esportazioni automatiche in cartelle condivise. La cifratura può essere opzionale **solo** se il formato aperto rimane esportabile in chiaro consapevolmente.
- Importare esclusivamente da una cartella scelta dall'utente; validare dimensioni, conteggi, profondità dei file, tipi, checksum, symlink/path traversal, duplicati, collisioni Unicode e contenuti ostili prima di leggere o scrivere. Non eseguire macro, script, link, HTML o formule contenute nei dati.
- Crash: staging in cartella privata, verifica integrale prima del commit atomico; in caso di errore conservare archivio esistente, cancellare soltanto staging app-owned dopo consenso.
- Preferenze: percorso dell'ultima cartella opzionale e memorizzato in locale; opzioni di esclusione delle informazioni personali **solo se** il pacchetto risultante dichiara che non è fedele/riimportabile, senza spacciarlo per completo.

## Dipendenze, alternative, rischi, complessità

- **Dipendenze progettuali:** schema dati SQLite v1/v2, modelli per opere/collezionisti/vendite/certificati/spese e media, migrazioni schema, logica backup già presente. Richiede una specifica del formato versionata e separata dal database fisico.
- **Alternative:** (A) usare SQLite + media del backup: più semplice e già disponibile ma meno interscambiabile; (B) CSV per ogni tabella: facile per fogli elettronici, fragile per relazioni e valori annidati; (C) ZIP proprietario: pratico ma non risolve la semantica. Per una prima proposta, JSONL + cartella è più ispezionabile.
- **Rischi:** divergenza fra versione software e formato, perdita di seriali, duplicazione vendite, perdita di media, gigantismo dei pacchetti, esportazione involontaria di contatti, import di file manomessi; *mitigazioni*: versioning, hash, dry-run, staging, compatibilità esplicita e suite di round-trip.
- **Complessità stimata:** **alta** (contratto semantico, compatibilità, conflitti, sicurezza), non da avviare senza decisioni su confini e import.

## Criteri di accettazione verificabili (per un'eventuale implementazione futura)

1. Esportazione completa di almeno due archivi di prova (vuoto e con valori Unicode, note multilinea, foto, vendite e certificati) e lettura da un programma generico **senza Repertica**.
2. Reimportazione in **archivio vuoto** con uguaglianza degli ID, valori monetari, date, note, seriali, snapshot certificati, riferimenti e SHA-256 media. La verifica segnala ogni differenza; non inventa dati mancanti.
3. File mancante, versione incompatibile, ID/seriale duplicato, checksum alterato, percorso `../` e pacchetto incompleto bloccano la modifica dei dati reali, con messaggio localizzato.
4. Il pacchetto non contiene password, token, percorsi assoluti locali o preferenze OS; salvataggio su USB/NAS e dischi pieni gestiti con report e senza archivio corrotto.
5. Import ripetuto non duplica entità; nel primo rilascio è vietato importare nell'archivio già popolato.

## Distinguere certezze, ipotesi e decisioni

**Certo dal repository:** CSV vendite/spese unidirezionali, backup SQLite+media completo, UUID stabili, certificati immutabili, denaro in centesimi; l'import specifico da Gestionale Opere è già nell'issue #5.

**Ipotesi di progetto:** JSONL e struttura esatta, pulsante di verifica e staging, nomi dei file, versioning indipendente, primo import solo in archivio vuoto.

**Questioni per il proprietario:** serve prima sola **esportazione semantica**, o subito round-trip? L'export include i dati personali per impostazione predefinita o richiede una seconda conferma? Quale compatibilità con futuri domini e archivi molto grandi? È utile uno schema JSON Schema pubblico versionato? Come gestire archivi con lo stesso UUID originati da copie divergenti?

## Fasi ipotetiche, non autorizzate

1. **Specifica pubblica dei campi** e matrice di copertura, esempi sintetici e piano dei test.
2. **Esportazione completa** e validazione indipendente, separata dal backup.
3. **Import in archivio nuovo** con verifica preflight e transazione.
4. Solo dopo approvazione: merge assistito e profili per fonti terze.

> Questa scheda è una proposta da discutere, non una richiesta di PR di codice.
