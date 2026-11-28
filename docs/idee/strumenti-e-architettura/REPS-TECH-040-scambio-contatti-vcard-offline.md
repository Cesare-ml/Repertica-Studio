# REPS-TECH-040 — Rubrica portabile: esportazione selettiva vCard dei collezionisti

- **ID stabile:** REPS-TECH-040
- **Data della ricognizione:** 2026-11-22
- **Stato:** **Proposta — da valutare**
- **Filone:** Strumenti e architettura — soltanto analisi e documentazione, nessuna implementazione o approvazione.
- **Priorità ipotetica:** P2–P3, condizionata alla frequenza reale degli scambi di contatti.
- **Complessità:** bassa–media per un export vCard conservativo; media–alta per importazione con corrispondenze e conflitti.

## 1. Problema, destinatari ed evidenze

Un'artista, una galleria o un archivista può voler trasferire nella rubrica personale **uno o più recapiti già registrati** in Repertica, senza inviare l'intero CSV dei collezionisti, aprire un account cloud o ricopiare a mano telefono e indirizzo email. La rubrica del sistema operativo e molti programmi di contatti usano il formato aperto `.vcf` (vCard); è distinto da una lista clienti per marketing.

**Fatti verificati nel branch `main` il 22 novembre 2026:**

- [`collectors.rs`](../../../src-tauri/src/collectors.rs) memorizza in SQLite UUID, nome e cognome, email, telefono, indirizzo **testuale non strutturato**, note, data/ora/luogo di nascita, profili social e riferimenti a fotografie. La scheda di un collezionista può avere anche relazioni con vendite: **non** sono dati da trasferire a una rubrica senza motivo.
- [`export.rs`](../../../src-tauri/src/export.rs) e [`DATA-PORTABILITY.md`](../../DATA-PORTABILITY.md) offrono già **12 registri CSV completi**, incluso `collectors` con dati privati e campi finanziari di altri registri; i CSV sono per ispezione/analisi, non un formato di reimportazione. L'esportatore rifiuta sovrascritture e destinazioni nell'archivio attivo.
- Nelle funzioni e nella documentazione consultate non emerge un **esportatore vCard**; la ricerca del termine nel codice `main` non ha restituito corrispondenze. Questo è un possibile spazio di interoperabilità, **non** un difetto segnalato da utenti e non la prova che nessuna soluzione esterna possa già soddisfare il bisogno.
- [`MCP.md`](../../MCP.md) descrive un server locale di sola lettura che volutamente **non** espone i dati di contatto: questa separazione va preservata.

**Requisiti certi di progetto:** Repertica Studio è Tauri/open source e gratuita, offline-first, senza account o abbonamento Repertica, con SQLite e fotografie locali; l'utente sceglie liberamente i percorsi di backup/export, i dati devono restare integralmente esportabili, la lingua UI segue il sistema operativo con fallback inglese.

**Ipotesi da verificare:** gli utenti trasferiscono recapiti da Repertica a rubriche di sistema o a clienti email con sufficiente frequenza; un file `.vcf` è effettivamente più pratico del CSV integrale e delle funzioni già disponibili negli altri programmi; i client target importano correttamente vCard 4.0. Nessun test con client o intervista è stato svolto in questa ricognizione.

## 2. Confini, alternative già esistenti e anti-doppioni

- [TECH-001](REPS-TECH-001-pacchetto-dati-aperto.md) studia un pacchetto **completo** e reimportabile dell'archivio: `.vcf` non è un backup, non riproduce vendite, opere, fotografie o legami.
- [TECH-019](REPS-TECH-019-importazione-guidata-cataloghi-tabellari.md) riguarda importazione generica di CSV/TSV esterni con mappatura; qui il valore differenziale è un **formato di rubriche interoperabile** e l'eventuale import `.vcf` non va confuso con un import tabellare.
- [TECH-025](REPS-TECH-025-esportazioni-tabellari-minimizzate-condivisione.md) studia esportazioni CSV con proiezione di colonne/righe per condivisione; questa proposta **non** ripete il sistema di profili CSV: si concentra su semantica, codifica, convalida e comportamento dei client vCard.
- [TECH-039](REPS-TECH-039-salvataggio-selettivo-email-eml-portabile.md) conserva un **messaggio ricevuto** in EML, non una scheda contatto; [TECH-014](REPS-TECH-014-bridge-mcp-locale-lettura-permessi.md) non deve diventare un canale per esportare contatti privati.
- Nell'altro filone, [UX-014](https://github.com/Cesare-ml/Repertica-Studio/blob/docs/idee-gestione-ux-20261121/docs/idee/gestione-e-ux/REPS-UX-014-rubrica-relazioni-professionali.md) propone una possibile **rubrica professionale e relazioni**; [UX-035](https://github.com/Cesare-ml/Repertica-Studio/blob/docs/idee-gestione-ux-20261121/docs/idee/gestione-e-ux/REPS-UX-035-preferenze-contatto-comunicazioni-individuali.md) studia preferenze dichiarate di comunicazione. **Né le relazioni né le preferenze equivalgono a esportare/importare vCard.** Quest'idea non amplia il modello CRM, non attribuisce consensi e non autorizza campagne. La [PR #14](https://github.com/Cesare-ml/Repertica-Studio/pull/14) resta intatta.

## 3. Scenari di utilizzo da validare

1. **Passaggio a una rubrica indipendente:** scegliere una persona e salvare un vCard con nome, email e telefono; aprirlo in un'app esterna senza Repertica, anche offline.
2. **Contatti per una fiera:** scegliere esplicitamente un piccolo gruppo di collezionisti, visualizzare il numero di schede e i campi inclusi, creare `contatti-selezionati.vcf`. Nessun invio a terzi o mailing list automatica.
3. **Contatto con indirizzo e URL ambigui:** Repertica conserva indirizzo in testo libero e profili social non necessariamente URL completi. Non inventare città/CAP, non trasformare arbitrariamente `@nickname` in link pubblico; segnalare i campi non mappabili e lasciare l'originale invariato.
4. **Persona con note riservate e fotografie:** il file minimo include solo recapiti scelti; esclude note, luogo/ora/data di nascita, foto, vendite, prezzi, cronologia delle comunicazioni e preferenze di contatto.
5. **Import eventuale da una rubrica:** prima anteprima locale di un `.vcf` e decisione su crea/ignora/abbina; mai modificare una scheda con vendite collegate sulla base del solo nome o di un telefono condiviso.

## 4. Specifica funzionale e flussi UX — proposta, non implementazione

**Fase minima: esportazione senza modifica del database.**

1. Dalla scheda Collezionista, azione secondaria **«Esporta contatto (.vcf)»**; in una fase successiva, selezione multipla con conteggio, filtro attivo chiarito e selezione sempre visibile. Non esportare automaticamente tutti i contatti per il solo fatto di aver aperto la pagina.
2. Anteprima **leggibile e navigabile** con nome/i del contatto, numero di schede, campi inclusi ed esclusi. Selezione iniziale prudente: nome, email e telefono presenti; indirizzo, sito e profili social solo con scelta esplicita. I dati speciali di nascita, note, foto, vendite e preferenze di comunicazione rimangono **sempre esclusi dalla prima fase**.
3. Evidenziare che `.vcf` è un file **personale in chiaro**, che l'esportazione non equivale a consenso per comunicazioni commerciali e che un programma esterno può sincronizzare il contatto secondo proprie impostazioni. Nessuna condivisione/invio automatico.
4. Dialogo nativo per file e cartella liberamente scelti (compreso disco esterno/NAS montato), nome neutro e sanificato, **nessuna sovrascrittura**, nessun salvataggio nella root privata dell'archivio. Fallimento di scrittura, unità rimossa o disco pieno non producono un file dichiarato valido.
5. Scrittura a cura del backend, con file temporaneo e pubblicazione prudente se il filesystem lo permette. Esito esplicito con percorso locale e avviso: **«Questo file esterno non è incluso automaticamente nei backup Repertica»**.
6. Un'eventuale funzione **«Importa vCard»** sarebbe separata e non attivata dall'export. Anteprima di ogni scheda, warning sui campi non gestiti e scelta umana per ogni conflitto prima di scrivere su SQLite; nessuna fusione/sovrascrittura silenziosa.

**Esclusioni:** nessuna rubrica online, CardDAV, Google/Apple/Microsoft API, sincronizzazione automatica, invio massivo, import continuo, scansione della rubrica di sistema, pubblicazione di elenchi, telemetria, permessi per agenti MCP o inferenza di preferenze e consensi.

## 5. Standard aperto, mappatura e dati/API solo descrittivi

Il riferimento primario da verificare è **IETF RFC 6350, vCard 4.0** (https://www.rfc-editor.org/rfc/rfc6350); vCard 3.0 è un'**alternativa di compatibilità**, non equivalente carattere per carattere, descritta da RFC 2426 (https://www.rfc-editor.org/rfc/rfc2426). Validare con i client target prima di decidere se supportare entrambe. In vCard 4.0: codifica UTF-8, linee CRLF, `BEGIN:VCARD`, `VERSION:4.0`, `FN` obbligatorio ed `END:VCARD`; escape dei caratteri di testo e *line folding* a larghezza consigliata di 75 **ottetti**, senza spezzare sequenze UTF-8. Non basta concatenare campi e separatori.

| Campo locale esistente | Ipotesi di output vCard | Regola di cautela |
| --- | --- | --- |
| `name`, `first_name`, `last_name` | `FN`; `N` se componenti affidabili | Non cercare di ricostruire cognomi da nomi storici indeterminati; il campo formattato deve esistere |
| `email` | `EMAIL` | Nessuna verifica del possesso dell'indirizzo, né permesso a inviare; esportare solo valore esplicitamente incluso |
| `phone` | `TEL` come testo (se conforme) | Non inventare prefissi internazionali o normalizzare distruttivamente |
| `address` | Eventuale `ADR` con `LABEL` letterale, dopo prove di compatibilità | È un indirizzo **libero**, non ricavare automaticamente via/città/CAP/stato/country; alternativa: omettere e segnalare |
| `website`, profili social | Eventuale `URL` solo se URL completo e validato | Nessuna costruzione automatica di URL da identificativi social; nessuna apertura remota dei link |
| `id`, dati di nascita, note, foto, riferimenti commerciali | Omessi nella fase minima | Non incorporare UUID interni o identificativi correlabili se non strettamente richiesti; niente `PHOTO`/allegati |

Più contatti possono essere serializzati come più oggetti `BEGIN:VCARD` nello stesso file; **l'accettazione di file multipli va provata su client reali**, con opzione di esportazione singola sempre disponibile. Il titolo del documento e il nome file non dipendono da valori non sanificati; nessuna firma/autenticità è implicata dal formato.

**Contratti concettuali non eseguibili:**

```text
ContactExportSelectionV1 { collector_ids: [UUID], fields: [name,email,phone,address,website,social_urls] }
ContactExportPreview { count, included_fields, excluded_categories, unmappable_fields, warnings }
exportSelectedVCard(selection, version: "4.0", destination, explicit_confirmation)
    -> { file_path, cards_written, bytes_written, warnings }

# Soltanto fase successiva, se approvata:
previewVCardImport(local_file) -> { candidates, field_mapping, duplicates?, conflicts, warnings }
confirmVCardImport(preview_token, user_decisions) -> { created, skipped, updated, errors }
```

Per export: nessuna nuova tabella o migrazione; selezione per UUID reali da SQLite, snapshot coerente, proiezione dei **soli campi autorizzati** costruita in Rust, non fiducia in campi dati da Angular; nessuna informazione privata nei nomi di file o nei log. Non conservare un registro esterno degli esportati per default. Per eventuale import: confronto su UUID solo se la loro provenienza è attendibile; nome/email/telefono sono indizi, **non identificativi certi**, e una scheda esistente legata a vendite richiede revisione particolarmente prudente. Nessun `UID` vCard standard da confondere automaticamente con l'UUID interno.

## 6. Offline, privacy, sicurezza e compatibilità

- **Offline:** selezione, anteprima, serializzazione e salvataggio funzionano senza rete; il file può essere aperto da un client compatibile anche se l'archivio Repertica è indisponibile. Il software importatore esterno **potrebbe** sincronizzare via Internet, ma Repertica non lo apre o invoca automaticamente.
- **Privacy:** il CSV integrale attuale comprende molti dati in più; un vCard selettivo riduce il perimetro ma **non rende anonimo** il contatto. File non cifrato, potenzialmente leggibile su dispositivi condivisi. Non registrare un'esportazione come consenso o come prova di corretto trattamento dei dati; valutare la finalità con la persona interessata. Nessuna pubblicazione, cloud o assistente IA per default.
- **Filesystem e input:** validare i percorsi di destinazione, impedire di scrivere nell'archivio attivo, non seguire link pericolosi, usare create-new, gestire permessi e cancellazioni prudenti dei soli file temporanei di questa operazione. Per un eventuale import: limite di dimensioni e numero di vCard, limiti di nesting/lunghezza, decoding robusto, no fetch di `URL`/`PHOTO` remoti, non aprire allegati, non interpretare markup o contenuti come comandi.
- **Compatibilità:** vCard è aperto ma il supporto fra rubriche può differire per versioni, `ADR`, caratteri Unicode e vCard multipli. Prove manuali o automatiche con parser indipendenti e client target Windows/macOS/Linux, senza affermare compatibilità universale. CSV integrali, backup e certificati rimangono invariati.
- **Backup:** l'export esterno `.vcf` non è nel backup SQLite/media. Eventuali importazioni **confermate** in SQLite entrerebbero invece nel normale backup integrale a cartella scelta, rispettando le regole di ripristino e la portabilità. Nessuna modifica ai formati di backup.

## 7. Impostazioni, lingue e accessibilità

Nessun account, nuova impostazione obbligatoria, dipendenza remota o autorizzazione a leggere la rubrica di sistema. Eventuali preferenze innocue (`versione vCard`, ultima cartella) restano locali; **non** salvare come default permanente «includi note personali/foto/dati di nascita». La selezione di campi riservati, qualora ammessa in una fase futura, richiede una decisione specifica.

**Lingua UI** da sistema operativo con fallback inglese; pulsanti, avvisi e date nell'interfaccia tradotti senza tradurre proprietà standard `FN`/`EMAIL`/`TEL` né valori originali del contatto. Nomi Unicode (accenti, apostrofi, caratteri non latini) devono sopravvivere esattamente al round-trip del formato quando supportati. Flusso completo con tastiera, annunci di numero di contatti e campi esclusi, focus stabile dopo annullamento/errori, anteprima screen-reader e indicatori non basati solo sul colore.

## 8. Dipendenze, rischi, complessità e alternative

**Dipendenze concettuali esistenti:** lettura `collectors` in SQLite, selezione UI, API Tauri, dialoghi nativi, convenzioni di `export.rs`, localizzazione e backup. **Nessun pacchetto viene aggiunto ora**; valutare l'uso di una libreria locale vCard affidabile rispetto a serializzazione minimale, solo dopo aver deciso di implementare.

**Alternative da confrontare prima dello sviluppo:**
1. **Nessuna nuova funzione:** esportare `collectors.csv` e usare un programma rubriche che importa CSV, selezionando manualmente le colonne. Costo tecnico nullo; maggiore esposizione di dati personali e mappature manuali.
2. **Esportazione del singolo contatto** soltanto, senza multi-selezione né import: prima verifica rapida dell'utilità, meno complessità e rischio.
3. **Formato vCard 3.0**, se i client usati dagli utenti mostrano problemi con 4.0; scegliere in base a prove, non per supposizione.
4. **Nessun import vCard:** se il bisogno è solo trasferire dati in uscita, il parser, i conflitti e le migrazioni non sono giustificati.

**Rischi:** diffusione di contatti privati; pretesa di anonimizzazione; esportazione accidentale di tutta la rubrica; campi social malformati e URL ricostruiti; perdita silenziosa dell'indirizzo non strutturato; escape/folding errati; vCard corretta secondo standard ma ignorata da un client; duplicati/merge impropri; sovrascrittura di documenti esistenti; false deduzioni di consenso. **Complessità stimata** bassa–media per export singolo sicuro, media per selezione multipla, media–alta per import con anteprima, matching e rollback; nessuna stima di prestazioni senza prove.

## 9. Criteri di accettazione ipotetici e test futuri (non eseguiti)

1. Su `main`, nessun campo di vendite, prezzi, quote, incassi, note, nascita, fotografia o identificativo interno appare in un vCard generato con opzioni minime; l'export completo CSV e i backup rimangono invariati.
2. La selezione di una scheda produce esattamente **un** contatto, quella di N schede esattamente N vCard, nell'ordine documentato; nessuna esportazione implicita dei contatti non selezionati.
3. `FN`, `VERSION`, CRLF, caratteri speciali (virgole, punti e virgola, backslash, line break), Unicode e fold a 75 ottetti sono convalidati contro parser indipendente RFC 6350. Il nome formattato storico non perde parti se il cognome non è strutturato.
4. Un indirizzo libero è omesso con avviso oppure esportato come etichetta fedele dopo test, **mai** suddiviso in città/CAP presunti. I social non vengono trasformati in URL inventati.
5. Il file si apre almeno nei client di rubrica concordati su Windows/macOS/Linux; i limiti di interoperabilità sono dichiarati. Un eventuale output v3 viene testato separatamente.
6. Destinazione già occupata, link/rinomina sospetta, cartella scollegata, permessi negati e disco pieno: nessun file preesistente sovrascritto e nessun file incompleto riportato come esportazione riuscita.
7. Durante l'export non partono richieste HTTP, SMTP, CardDAV né letture del portachiavi o della rubrica di sistema; nessun nuovo tool MCP fornisce i recapiti.
8. UI leggibile da tastiera e screen reader; lingua sistema/fallback inglese; valori Unicode del contatto preservati, avvisi privacy e numero di schede sempre accessibili.
9. Se si autorizzerà mai l'import: casi con due omonimi, recapito condiviso, vCard multiple, campi sconosciuti, file enorme e contenuti malformati portano ad anteprima e rifiuto o decisione esplicita; mai merge automatico né cambi a vendite/certificati.
10. Nessun `.vcf` esterno è presentato come parte dei backup Repertica; un eventuale contatto importato e confermato compare normalmente nel backup SQLite e nel ripristino verificato.

## 10. Fasi e questioni aperte per revisione del proprietario

- **Fase 0 — ricerca senza sviluppo:** raccogliere 3–5 casi reali di migrazione rubrica; identificare i client target e verificare se bastano CSV/contatti singoli.
- **Fase 1, solo se approvata:** esportazione `.vcf` **singola**, anteprima dei pochi campi sicuri, salvataggio locale atomico/conservativo, validazione RFC e prova con client reali.
- **Fase 2, soltanto dopo valutazione:** selezione multipla esplicita, avvisi di privacy e campi opzionali con compatibilità verificata.
- **Fase 3, separata e non presunta:** import `.vcf` con preview, identificazione ambiguità e conferme manuali, nessuna sincronizzazione.

**Decisioni da riservare al proprietario:** (a) il passaggio alle rubriche esterne è davvero frequente? (b) quali client OS/versioni giustificano v4 e/o v3? (c) l'export dovrebbe includere per default solo nome, oppure anche email/telefono dopo anteprima? (d) la rubrica UX-014, se mai approvata, dovrebbe estendere i destinatari oltre ai collezionisti? (e) si desidera soltanto esportare o anche importare, accettando maggiore rischio di conflitti? (f) gli indirizzi testuali devono essere omessi o esposti come `LABEL` dopo interop testing?

**Conclusione:** proposta tecnica **non approvata**, non assegnata e non implementata. Se i casi reali sono rari o i client esterni gestiscono già bene i CSV, la decisione migliore può essere **non aggiungere alcuna funzione**.
