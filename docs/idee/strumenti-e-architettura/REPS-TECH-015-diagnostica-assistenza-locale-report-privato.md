# REPS-TECH-015 — Diagnostica locale per assistenza con report minimizzato e condivisibile soltanto su scelta

- **ID stabile:** REPS-TECH-015
- **Data:** 2026-11-21
- **Stato:** **Proposta — da valutare**
- **Ambito:** studio tecnico e documentale, **nessuna implementazione né approvazione**
- **Priorità ipotetica:** **P2**, elevabile solo dopo evidenze di problemi ricorrenti nell'uso desktop e prove di efficacia con utenti.
- **Destinatari:** artista che usa il software senza competenze tecniche; studio/galleria con archivio su USB o NAS; manutentori volontari open source che ricevono segnalazioni senza poter accedere ai dati reali.

## 1. Problema, evidenze verificabili e confini

In un'app desktop che funziona offline con database e media locali, un problema come «backup non riuscito» non basta a distinguere un volume scollegato, permessi insufficienti, spazio esaurito, errore di integrità o incompatibilità dello schema. Chiedere all'utente di inviare SQLite, fotografie o screenshot contenenti clienti e incassi sarebbe eccessivo e rischioso. Un piccolo report locale, **senza dati commerciali o personali**, potrebbe rendere riproducibile il problema e ridurre le richieste di accesso all'archivio.

**Riscontro su `main`, 21/11/2026:**
- [`src-tauri/src/commands.rs`](../../../src-tauri/src/commands.rs) espone diverse operazioni come `Result<..., String>`, talora con `err(e.to_string())` e talora con frasi italiane. [`src/app/app.component.ts`](../../../src/app/app.component.ts) raccoglie `errorMessage` e converte un errore in stringa nell'helper `error()`. Sono presenti anche alcuni errori gestiti più specificamente.
- [`src/app/api.ts`](../../../src/app/api.ts) definisce gli `invoke` di Tauri ma non un comando per costruire un **report generale di assistenza**. [`src-tauri/src/lib.rs`](../../../src-tauri/src/lib.rs) non registra tale comando, né un servizio di telemetria.
- [`docs/MVP.md`](../../MVP.md) richiede ancora collaudi desktop reali di backup/ripristino su Windows/macOS e supporti esterni. Il backup verificato con SHA-256 e la [diagnostica locale delle fotografie](../../../src-tauri/src/media_audit.rs) **esistono già**: questa proposta **non** li reinventa.
- [`docs/ECONOMY-I18N.md`](../../ECONOMY-I18N.md) segnala l'attuale limite di errori Rust ancora in italiano. L'[issue #12 — Localizzazione errori backend](https://github.com/Cesare-ml/Repertica-Studio/issues/12) propone già codici tipizzati e mappe di traduzione: **non attribuire qui una seconda roadmap per i codici**, ma descriverli come possibile prerequisito/interfaccia.
- [`docs/DATA-PORTABILITY.md`](../../DATA-PORTABILITY.md) descrive sette CSV completi con dati personali; non sono report di supporto e **non** vanno allegati automaticamente.

**Valore da validare:** diagnosi di errori concreti senza dati dell'archivio. Non è stata osservata una perdita di dati né dimostrato che vi siano molti ticket; la domanda professionale va prima misurata.

**Distinzioni:** [TECH-003](REPS-TECH-003-diagnostica-integrita-media.md) verifica file media dell'archivio; [TECH-009](REPS-TECH-009-catalogo-verifiche-conservazione-backup.md) cataloga copie di backup; [TECH-012](REPS-TECH-012-migrazioni-archivio-preflight-copia-sicurezza.md) previene danni durante migrazioni. **TECH-015** riguarda esclusivamente *come descrivere e condividere volontariamente un problema tecnico* senza esportare contenuti. [TECH-014](REPS-TECH-014-bridge-mcp-locale-lettura-permessi.md) espone dati selezionati a un client MCP, cosa qui **esclusa**. Le schede [Gestione e UX, PR #14](https://github.com/Cesare-ml/Repertica-Studio/pull/14) trattano processi professionali, non diagnostica per assistenza. Nessun loro file viene modificato.

## 2. Scenari d'uso

1. **Backup su disco rimovibile:** un artista sceglie la cartella di destinazione, ma l'unità viene scollegata. La UI distingue un guasto di scrittura da un backup dichiarato integro; permette di annotare «destinazione non disponibile» e di salvare soltanto un resoconto tecnico, senza percorsi reali.
2. **Ripristino rifiutato:** prima di sostituire l'archivio, un controllo segnala hash non corrispondente o file mancante. Il report registra il tipo di verifica e che l'archivio originale non è stato sostituito, **solo se lo stato è dimostrabile**. Nessun tentativo di correzione automatica.
3. **Versione SQLite troppo nuova:** un'app vecchia non può aprire un archivio creato da una versione più recente. Il rapporto segnala versione dell'app, schema rilevato e schema supportato, senza includere le righe SQLite.
4. **Esportazione CSV non salvata:** cartella non scrivibile o file già presente. Il report espone operazione e codice, senza esportare il CSV o il nome della cartella.
5. **Avvio fallito prima che si apra Angular:** l'utente non può raggiungere Impostazioni. È un caso importante **non risolto dalla sola schermata interna**: valutare soltanto in fase successiva un messaggio di avvio minimale o un recupero tramite helper locale, senza promesse premature.

## 3. Flusso UX descrittivo

**A. Messaggio comprensibile e recuperabile:** l'errore nella finestra che ha fallito mostra una frase localizzata, un'azione coerente («Riprova», «Seleziona altra cartella» oppure «Non modificare la copia; consulta la guida») e, in forma discreta, **«Dettagli tecnici»** con un codice stabile copiabile. Se non c'è codice tipizzato, conservare temporaneamente un messaggio locale **senza trasformarlo automaticamente in report esportabile**.

**B. Diagnostica facoltativa:** Impostazioni → «Assistenza e diagnostica locale» → «Prepara report». Il rapporto è generato **su richiesta** (preferibilmente dal singolo errore selezionato) ed è visualizzato integralmente prima del salvataggio. Un'etichetta esplicita dice: «Nessun invio automatico. Controlla il contenuto prima di condividerlo».

**C. Anteprima minimizzata:** mostra sezioni «Versione», «Sistema», «Operazione», «Codice problema», «Esito delle verifiche», «Informazioni omesse». Le omissioni sono reali, non campi vuoti che tradiscano l'esistenza o il numero di clienti. Un comando «Copia soltanto codice e versione» è alternativa preferibile quando sufficiente.

**D. Salvataggio manuale:** dialogo nativo verso **qualsiasi cartella accessibile scelta dall'utente**, inclusa USB o NAS già montato; nessuna sovrascrittura silenziosa. File JSON UTF-8 con schema pubblico e opzionalmente testo Markdown sintetico, senza payload del database. Nessuna apertura di email, browser o issue GitHub automatica. La condivisione successiva è un atto esterno dell'utente.

**E. Nessun errore acquisito:** messaggio «Non ci sono eventi tecnici disponibili» e possibilità di consultare una piccola scheda di informazioni generali. Non inventare cronologie, registri permanenti o statistiche di crash che non esistono.

**F. Errore prima dell'avvio UI (fase eventuale):** schermata di avvio nativa *se tecnicamente sostenibile*, con codice privo di percorsi personali e indicazioni manuali. Un helper esterno introdurrebbe superficie d'attacco, packaging e manutenzione cross-platform: valutarlo separatamente, non considerarlo incluso nella prima fase.

**Accessibilità:** tastiera completa, pulsanti con etichette, ordine focus prevedibile e ritorno al controllo che ha fallito, messaggio letto da screen reader senza ripetizioni, stato visibile non affidato solo al colore, anteprima selezionabile senza scroll orizzontale al 200%, nessuna scorciatoia obbligatoria.

## 4. Contratto dati e API puramente concettuali

Non è richiesta una nuova tabella SQLite. Prima ipotesi: **report effimero, costruito in memoria e scartato dopo la chiusura**, con schema documentabile e versionato indipendentemente da `PRAGMA user_version`. Esempio illustrativo, **non codice da implementare**:

```json
{
  "report_schema": "repertica-support/1",
  "created_at_utc": "2026-11-21T11:00:00Z",
  "application_version": "0.1.0",
  "os_family": "windows",
  "architecture": "x86_64",
  "operation": "backup.create",
  "error_code": "BACKUP_DESTINATION_UNAVAILABLE",
  "outcome": "not_completed",
  "schema_version_supported": 6,
  "privacy_profile": "minimal",
  "data_included": ["technical_versions", "operation", "error_code", "outcome"]
}
```

Date, versioni e valori dell'esempio sono **sintetici**; non rappresentano una rilevazione di problemi reali. Operazioni ed errori sono **enum chiusi**: nomi illustrativi `BACKUP_DESTINATION_UNAVAILABLE`, `BACKUP_INTEGRITY_FAILED`, `ARCHIVE_SCHEMA_TOO_NEW`, `CSV_DESTINATION_EXISTS`, `MEDIA_REFERENCE_MISSING`. La tassonomia definitiva dipende dall'[issue #12](https://github.com/Cesare-ml/Repertica-Studio/issues/12) e non va duplicata qui.

**Eventuale API interna descrittiva:** `get_safe_diagnostic_summary(error_reference?)` produce un oggetto allowlist dal *risultato strutturato dell'operazione*, non da parsing di eccezioni arbitrarie; `save_support_report(destination, summary)` serializza **solo la preview approvata** e verifica che la destinazione non sia dentro la directory dati dell'app. Non introdurre un endpoint HTTP, servizio in background, accesso da client MCP, lettura generica di file o esportazione automatica del database.

Per evitare report che suggeriscono falsa affidabilità, un esito di rollback o «archivio invariato» è incluso **solo se derivato da una prova/garanzia verificabile**. Se sconosciuto, `outcome="unknown"`; nessun report sostituisce `integrity_check` o una prova di ripristino.

**Inclusioni possibili, da validare:** versione pubblica app, piattaforma OS generica e architettura, codice errore, categoria operazione, tempo UTC, versione schema se pertinente, indicatore di rollback *verificato*, categoria generica «unità esterna/locale» **solo con consenso separato**, indicazione che la verifica backup è stata avviata/completata. Per il bug reporter possono bastare codice e versione.

**Esclusioni predefinite inderogabili nella prima fase:** SQLite, righe o conteggi delle tabelle, email, nomi clienti/artisti, titoli, prezzi, note, contenuti CSV/PDF/foto, hash delle fotografie, ID UUID, file path completi o parziali, nomi dei volumi, hostname, username di sistema, indirizzi IP, seriali hardware, file di configurazione, token, variabili d'ambiente, stack trace grezzi e messaggi `to_string()` di errori OS. Gli hash non sono sempre anonimi: possono permettere correlazioni con dati esterni.

**Privacy by construction:** costruire il report da **allowlist di campi tipizzati**, non da un dump con ricerca-e-sostituzione o regex sui percorsi. I messaggi di errore grezzi possono contenere valori immessi e percorsi: non copiarli. Un report JSON deve avere lunghezza massima e schema con valori enumerati, e non accettare nomi di campi arbitrari dal frontend.

## 5. Offline, sicurezza, impostazioni e lingue

- **Offline-first radicale:** nessuna telemetria, invio automatico, account, abbonamento o accesso al web per creare/consultare/salvare il report. Il percorso di destinazione è scelto di volta in volta, senza preferenze cloud obbligatorie.
- **Integrità:** la generazione è in **sola lettura** rispetto al database e a `media/`; non esegue riparazioni, migrazioni, backup o scansioni invasive. Un report non dimostra autenticità o assenza di corruzione; non è sostitutivo del backup verificato.
- **Minimizzazione:** niente identificativo utente persistente né tracciamento dei cicli d'uso. Anche il timestamp esatto e la versione OS dettagliata possono agevolare correlazioni: consentire profilo ancora più scarno («solo codice e versione»).
- **Rischi residui:** un utente può condividere per errore il documento che ha salvato; Windows/macOS/Linux possono registrare file recenti e backup automatici della cartella scelta; un malware locale con i privilegi dell'utente resta fuori dal modello di minaccia. Vietato promettere anonimato assoluto.
- **Impostazioni:** diagnostica non persistente per default; nessuna acquisizione continua. Se una raccolta temporanea di eventi viene richiesta in futuro, richiedere consenso separato, periodo breve, limite di dimensione, comando «Cancella» e chiarire dove sono conservati i dati. Non includere come scelta approvata.
- **Multilingua:** usare la lingua OS o la scelta utente fra IT/EN/ES/FR/DE, con fallback inglese, come [ECONOMY-I18N](../../ECONOMY-I18N.md). Codici/chiavi macchina stabili in inglese; tradurre messaggi utente, azioni, errori, etichette e testo esplicativo. Non tradurre nomi di chiavi JSON né interpretare impropriamente errori OS sconosciuti.
- **Interoperabilità:** UTF-8, schema JSON pubblico e versionato, documentazione sufficiente a leggerlo anche senza Repertica; mai serializzare strutture Rust interne instabili o usare un formato binario proprietario.
- **Compatibilità:** non modifica lo schema SQLite, i certificati, i backup, gli export CSV o i documenti. Eventuali codici di errore nelle API devono preservare la corretta gestione degli errori nelle UI già esistenti; nessun errore «tecnico» deve interrompere rollback o trasformare una failure in successo.

## 6. Alternative, complessità, dipendenze, rischi

| Opzione | Vantaggio | Limite |
| --- | --- | --- |
| **Solo codice copiabile**, senza file | Minima superficie privacy; semplice supporto | Insufficiente per problemi di versione/ambiente |
| **Report minimale on-demand** (proposta centrale) | Informazioni riproducibili, condivisibili su decisione | Richiede modello errori stabile e validazione privacy |
| Log testuali completi su disco | Semplici da raccogliere in debug | Rischio elevato di percorsi e dati personali; sconsigliato come impostazione predefinita |
| Invio telemetrico/servizio cloud | Diagnosi centralizzata | Contrario ai principi di indipendenza e assenza di invio automatico; **fuori perimetro** |

**Complessità indicativa:** *bassa-media* per copia codice e report a mano di errori ben tipizzati; *media-alta* se si volesse coprire l'intero backend e i fallimenti prima della UI. Non assegnare giorni/uomini senza progettazione e test.

**Dipendenze reali:** codici errori tipizzati e localizzazione ([issue #12](https://github.com/Cesare-ml/Repertica-Studio/issues/12)); disponibilità di esiti affidabili per operazioni backup/ripristino; dialogo nativo di salvataggio già utilizzato dall'export CSV; test nativi su Windows, macOS e Linux. Nessun SDK esterno di analytics, logging remoto o servizio a pagamento dovrebbe essere necessario.

**Rischi da controllare:** suggerire «archivio salvo» senza prova; esportare stack trace con percorsi; report incompatibili dopo aggiornamenti; nomi file o stringhe di causa reinseriti attraverso campi liberi; messaggi tradotti non coerenti con il codice; report utile solo quando l'interfaccia parte; supporto che chieda al cliente il database completo quando non necessario.

## 7. Test di progettazione e criteri di accettazione eventuali

1. Da ogni errore previsto per backup, ripristino, CSV e compatibilità schema l'utente può ottenere **almeno un codice stabile** e un messaggio pertinente, senza perdita del contesto o dichiarazione di successo.
2. Il report minimo è valido rispetto a uno **schema JSON pubblico**, è offline, si salva nella cartella scelta, **non sovrascrive** file e non scrive nell'archivio privato.
3. **Fixture sintetiche avversarie** in titolo opera, nome cliente, path USB/NAS, stringa eccezione OS, nome del computer, nome volume, UUID, token e note non compaiono *né in anteprima né nei byte finali*; i test analizzano anche errori di serializzazione.
4. Il report non include database, fotografie, allegati, certificati, csv, hash file e nessun path per default; i controlli si basano su allowlist, non redazione euristica.
5. Su Windows/macOS/Linux si collaudano disco pieno, permessi, supporto espulso, path Unicode, errori SQLite, versione schema più nuova e destinazione già esistente. Mancata possibilità di completare un test deve risultare **non verificata**, non verde.
6. Report leggibile e flussi da tastiera/screen reader al 200%, con messaggi e istruzioni completi nelle lingue supportate o fallback inglese; test del codice invariabile al cambio lingua.
7. Riproduzione di un errore non altera SQLite/media, non attiva rete o log persistenti nascosti, non avvia ripristini e non pubblica automaticamente documenti.
8. Se l'app non arriva a inizializzare Tauri, documentare onestamente il **limite di fase 1**; eventuale diagnostica pre-avvio richiede un criterio di accettazione distinto.

## 8. Fasi proposte (nessuna autorizzata)

- **Fase 0 — Validazione del bisogno:** raccogliere 5–10 casi di assistenza reali/anonymizzati (supporti rimovibili, backup, migrazioni, errori import/export) e verificare che bastino codice, versione e stato; coordinare tassonomia con issue #12.
- **Fase 1 — Minimo utile:** messaggi localizzati, «Copia codice e versione», anteprima del report minimo **effimero**, eventuale salvataggio JSON solo su richiesta. Nessun log persistente.
- **Fase 2 — Analisi di copertura:** espandere soltanto categorie con casistiche dimostrate e controlli di sicurezza; collaudo cross-platform e revisione indipendente delle esclusioni privacy.
- **Fase 3 — Solo se richiesta:** studiare fallimenti prima dell'avvio e raccolta *opt-in* a scadenza di pochi eventi minimizzati, senza helper/daemon automatici in assenza di una decisione esplicita.

## 9. Domande per la revisione del proprietario

1. Gli utenti hanno già avuto errori di backup/ripristino difficili da descrivere, oppure la prima risposta dovrebbe essere soltanto una guida e un codice copiabile?
2. È più utile un file JSON completo e documentato, o basta un riepilogo testuale «codice+versione» condiviso manualmente?
3. Il supporto open source riceverebbe report tramite GitHub Issues, email o altro canale scelto dall'utente? **Nessuno** dovrebbe essere integrato come invio predefinito.
4. Concedere anche la versione dello schema SQLite e la tipologia generica di storage, o limitarle in principio a casi specifici?
5. Ci sono ragioni reali per una raccolta temporanea di eventi, nonostante il rischio di privacy e la complessità?
6. Quanto è importante diagnosticare errori **prima dell'avvio**? Vale la complessità di un fallback nativo?
7. Chi revisionerebbe regolarmente ogni nuovo campo del report per impedire regressioni di privacy?

**Decisione richiesta:** solo eventuale valutazione da parte del proprietario. **Stato invariato: Proposta — da valutare.** Nessun intervento al software, nessuna issue, PR implementativa, incarico o merge.
