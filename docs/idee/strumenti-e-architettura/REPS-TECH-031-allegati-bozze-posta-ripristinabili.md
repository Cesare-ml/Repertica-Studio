# REPS-TECH-031 — Allegati delle bozze email: preflight, copie gestite e ripristino indipendente dai percorsi originali

- **ID stabile:** REPS-TECH-031
- **Data ricognizione:** 2026-11-22
- **Stato:** **Proposta — da valutare**
- **Filone:** Strumenti e architettura
- **Priorità proposta:** **P2 condizionata**; P1–P2 solo dopo evidenza di incidenti frequenti e validazione con utenti
- **Tipo:** analisi e documentazione, **nessuna implementazione o decisione approvata**

## 1. Problema, destinatari ed evidenze su `main`

**Problema concreto:** una bozza email può conservare il percorso assoluto di un allegato sul computer dell'autore, ma non i relativi byte. Dopo uno spostamento del file, la disconnessione di un'unità esterna o il ripristino su un altro computer, il testo della bozza sopravvive e l'allegato no. Più sottilmente, un file sullo stesso percorso può essere cambiato nel frattempo: il nome resta uguale, il contenuto da spedire potrebbe non esserlo. Questo è un **limite documentato**, non una prova di invio errato o di perdita di dati già verificatasi.

**Riscontro del 22/11/2026:**
- [Posta professionale](../../BUSINESS-MAIL-DESKTOP.md): client personale IMAP/SMTP **già implementato**, bozze SQLite offline, massimo **5 allegati**, **5 MiB ciascuno** e **15 MiB totali**; i riferimenti delle bozze sono percorsi, non copie; il documento segnala esplicitamente che non sopravvivono allo spostamento dei file.
- [Backend Rust](../../../src-tauri/src/business_mail.rs): `MailDraft.attachments: Vec<String>`; `mail_save_draft` serializza i percorsi nel `payload_json`. `mail_send` rilegge i file soltanto al momento dell'invio, ne controlla tipo regolare, dimensione e quantità, ma non conserva un hash di riferimento dal salvataggio della bozza. Il backend prepara gli allegati **prima** di aprire la sessione SMTP: è una tutela già presente, da non indebolire.
- [Componente desktop](../../../src/app/business-mail.component.ts): scelta esplicita dei file, rimozione dal compositore, salvataggio locale e conferma manuale prima dell'invio.
- [Backup](../../../src-tauri/src/backup.rs): snapshot SQLite + contenuto di `media/` con manifest SHA-256; non acquisisce automaticamente file arbitrari nominati nelle bozze e collocati fuori dall'archivio.
- [Schema](../../../src-tauri/src/db.rs): i draft sono in `mail_drafts`; le password restano nel portachiavi OS e non vengono incluse nel backup. Non è corretto confondere il ripristino della bozza testuale con il ripristino completo degli allegati.

**Destinatari:** artisti che allegano preventivi e fotografie alle email, gallerie che preparano schede per collezionisti, archivi che usano spesso file su dischi esterni o trasferiscono il lavoro tra computer. Chi non usa Posta professionale, o invia sempre subito, può non aver bisogno della funzione.

**Requisiti certi:** offline-first e nessun account Repertica, uso di caselle IMAP/SMTP facoltativo, invio esclusivamente su azione e conferma esplicite, backup completo in cartella liberamente scelta, dati esportabili, lingua di sistema con fallback inglese, compatibilità delle bozze esistenti.

**Ipotesi non dimostrate:** numero significativo di utenti che conserva bozze con allegati; disponibilità a occupare spazio locale per copie gestite; necessità di conservarle anche dopo la spedizione. Nessun problema runtime con invii reali è stato riprodotto.

## 2. Confini, coordinamento e anti-doppione

- **TECH-016** studia allegati documentali collegabili a opere e altre entità; **non** progetta la semantica di un *allegato in bozza email*, la decisione link/copia, il controllo prima dell'SMTP o la sua eliminazione sicura. Eventuale motore di custodia condiviso va solo valutato, non assunto.
- **TECH-001** riguarda la portabilità generale; **TECH-006/009/022** rispettivamente backup cifrati, conservazione/verifica e preflight dello spazio; questa proposta riguarda il **confine fra un percorso esterno al backup e una bozza ripristinata**.
- **TECH-014** riguarda permessi MCP: questa proposta non dà accesso agli allegati a MCP, host IA o plugin.
- **UX-035** della [PR Gestione e UX #14](https://github.com/Cesare-ml/Repertica-Studio/pull/14) studia preferenze di contatto dei collezionisti; **non** la conservazione dei byte dei file. Nessuna gestione newsletter, automazioni di invio, caching di posta ricevuta o modifica dei flussi commerciali.
- Il [documento Posta professionale](../../BUSINESS-MAIL-DESKTOP.md) **già cita** l'archiviazione degli allegati delle bozze come sviluppo possibile; questa scheda è un approfondimento di quel limite, non una nuova richiesta scollegata.

## 3. Scenari da verificare

1. **Bozza con file su USB:** l'utente salva una bozza mentre il disco è collegato, poi apre la bozza senza disco. Deve capire chiaramente quali file non saranno disponibili, senza credere che siano stati salvati nel backup.
2. **File sostituito sullo stesso percorso:** una fotografia viene rielaborata dopo il salvataggio. Prima dell'invio l'utente deve sapere se sta allegando la versione selezionata inizialmente o il file modificato, e deve scegliere consapevolmente.
3. **Cambio computer:** ripristino da backup in una cartella scelta. Bozza e testi sono presenti; una **copia gestita** dovrebbe essere recuperabile, mentre un **collegamento esterno** resterà eventualmente da ricollegare.
4. **Privacy di una foto privata:** l'utente non deve scoprire che una copia del documento è stata creata nel backup senza averne avuto avviso; né che una bozza cancellata abbia lasciato copie nascosta senza possibilità di controllo.
5. **Spazio esaurito o crash:** la copia di un allegato, se prevista, non deve produrre una bozza apparentemente valida che punti a file non persistito; non deve cancellare l'originale.
6. **Invio SMTP ambiguo:** se il server non conferma il risultato, Repertica deve mantenere la condotta attuale **non ritentare automaticamente**. I meccanismi di allegati non devono introdurre doppio invio.

## 4. Flussi UX possibili (solo ipotetici)

### Fase minima — protezione dei percorsi attuali, senza copie aggiuntive

- Nell'editor della bozza: elenco con nome, dimensione/stato dell'ultimo controllo e indicazione chiara **«Collegato al file originale; non incluso nel backup»**. Un file mancante appare come **«Da ricollegare»**, senza bloccare il salvataggio del testo.
- Al salvataggio, facoltativo **controllo locale** di esistenza, dimensione e impronta SHA-256 del file ordinario (nei limiti correnti). Se il controllo non riesce, avviso contestuale; l'utente può mantenere la bozza con riferimento non verificato.
- Alla riapertura e **prima della conferma di invio**, ricalcolo prudente e confronto con l'ultima impronta accettata (se nota). Distinguere: *disponibile e invariato*, *modificato*, *mancante*, *impossibile verificare*. Se è cambiato, azione esplicita **«Accetta questa versione»** o **«Scegli altro file»**; non riacquisizione silenziosa.
- La conferma **non** autorizza comunque invii automatici, ricerche ricorsive nelle cartelle o lettura di file non selezionati. I fallimenti interrompono l'invio **prima** di aprire SMTP.

### Fase opzionale — copia gestita e ripristinabile

- Alla selezione o al salvataggio: scelta comprensibile **«Collega il file nella sua posizione»** (nessuna copia, percorso fragile) o **«Conserva una copia privata nell'archivio per questa bozza»** (spazio e presenza nel backup). Proporre la modalità più semplice solo dopo prove con utenti; non cambiare retroattivamente le bozze esistenti.
- Anteprima: nome, peso, eventuale duplicato, consumo totale e avviso che i backup sono attualmente **in chiaro**. La copia è creata **solo dopo consenso**, con progresso e possibilità di annullamento.
- Dopo il ripristino: stato «Copia gestita verificata»; se la copia manca o l'hash diverge, **bloccare invio di quella bozza** e offrire scelta di un nuovo file, non cercare automaticamente il contenuto altrove.
- Eliminazione bozza: distinguere **rimuovere riferimento** e **rimuovere copia conservata non più usata**. Mai eliminare l'originale esterno o una copia referenziata da un'altra bozza senza verifica esplicita. Se si vuole conservare un documento fuori dalla posta, usare una funzione separata di archivio documentale (TECH-016), non farlo diventare un allegato «eterno» tacito.
- Accessibilità: controlli completi da tastiera, messaggi di stato annunciati da screen reader, progressi e dimensioni anche in testo, conferme focalizzate, zoom 200%, nessuna sola codifica cromatica.

## 5. Strutture dati e API **solo descrittive**

Due modalità coesistenti, nessuna migrazione obbligatoria degli allegati legacy.

```text
DraftAttachmentRef {
  id: UUID locale,
  draft_id: UUID,
  mode: "EXTERNAL_LINK" | "MANAGED_COPY",
  display_filename: string,
  mime_hint?: string,
  expected_size_bytes?: integer,
  expected_sha256?: lowercase_hex,
  original_path_local?: string,  // solo mode EXTERNAL_LINK; dato privato
  managed_media_id?: UUID,       // solo mode MANAGED_COPY
  verified_at?: local_datetime,
  consented_copy_at?: local_datetime
}
ManagedMailBlob {
  id: UUID, relative_media_path: string, size_bytes: integer,
  sha256: lowercase_hex, created_at: datetime,
  references_count: derived_integer // mai affidarsi solo a conteggio denormalizzato
}
```

**Vincoli:** id della bozza e revisione ottimistica attuali rimangono autorevoli; massimo 5 file da 5 MiB e 15 MiB totali resta il limite iniziale. Conservare MIME come *indicazione*, non come sicurezza o autenticità del documento. Non serializzare percorsi assoluti esterni nei pacchetti condivisibili salvo scelta deliberata; `expected_sha256` indica integrità relativa a un'osservazione locale, non originalità.

API ipotetiche: `draft_attachment_preflight(draft_id)`, `preview_managed_copy(draft_id, selected_file)`, `attach_managed_copy(draft_id, consent_token)`, `relink_external_attachment(draft_id, attachment_id, file)`, `release_draft_attachment(draft_id, attachment_id)`, `prepare_send_verified(draft_id, expected_revision)`. La preparazione di byte verificati e l'invio devono usare **gli stessi byte già aperti**, riducendo il rischio di cambio del file fra preflight e SMTP (TOCTOU). Ogni comando è sottoposto ad autorizzazione del processo locale, limiti e verifica delle revisioni. Nessuna di queste API è attuale o proposta come promessa di lavoro.

**Compatibilità legacy:** una `attachments: Vec<String>` esistente va interpretata come insieme di **link esterni non verificati**, con etichetta onesta. Non sostituire silenziosamente con blob gestiti o cancellare percorsi senza scelta. Le esportazioni CSV complete e i certificati esistenti non vanno modificati; per l'eventuale scambio portabile futuro, definire un manifest aperto versionato di `bozze + riferimenti + byte gestiti` e report per i link irrisolvibili, senza credenziali. Un backup ripristinato non ripristina password IMAP/SMTP del portachiavi OS: la UI deve ricordarlo separatamente.

## 6. Offline, sicurezza, privacy, impostazioni e lingue

- **Offline:** salvataggio, preflight, copie, verifica e ripristino completamente locali. L'invio SMTP richiede rete e scelta dell'utente; non introdurre una coda di invio automatica. Nessuna API cloud, login Repertica, abbonamento, telemetria o servizio IA.
- **Backup/ripristino:** se si adotta una copia gestita, deve trovarsi in un'area gestita e verificata dal backup, **non** in una cartella temporanea o in un percorso assoluto. Il backup deve rifiutare riferimenti gestiti mancanti, incoerenze di hash, file fuori archivio o link simbolici; nessun successo parziale. L'utente continua a scegliere liberamente la cartella di destinazione.
- **Sicurezza filesystem:** controllo no symlink/junction e file regolari, canonizzazione contenuta nella root, protezione da path traversal, aperture sicure contro race/sostituzioni, scrittura atomica in staging con `fsync` ove pertinente, rifiuto di file eseguibili come preview, limiti globali di quantità/spazio. Non eseguire mai file allegati. Considerare differenze tra ACL e semantiche Windows/macOS/Linux.
- **Privacy:** le bozze già contengono indirizzi, corpi e percorsi potenzialmente personali; una copia gestita amplifica la quantità di dati nei backup in chiaro. Per default **nessuna esposizione ad MCP**; nessun indicizzatore o sincronizzatore remoto. La cancellazione non promette cancellazione sicura da SSD, snapshot OS o backup storici: descrivere limiti reali e fornire un'azione verificabile per la copia attiva.
- **Impostazioni candidate:** mostra sempre stato di portabilità dell'allegato; preferenza per modalità link/copia solo **dopo validazione**, visualizzazione spazio consumato, verifica delle copie, pulizia conservativa con anteprima. I percorsi sensibili non devono entrare in log diagnostici. UI IT/EN/ES/FR/DE, selezione automatica lingua di sistema con fallback **inglese**, override manuale; nomi originali dei file mai tradotti. Errori tradotti senza occultare differenze tra `mancante`, `modificato` e `non verificato`.
- **Controllo dell'invio:** la conferma è sull'insieme effettivo di destinatari, oggetto, corpo e **nomi/impronte delle versioni**; nessuna schedulazione, nessun retry automatico dopo risposta SMTP ambigua, nessuna promessa di consegna o di archiviazione in «Inviati».

## 7. Dipendenze, alternative, rischi e complessità

**Dipendenze concettuali:** definizione della semantica di link/copia, delle revisioni di bozza e del ciclo di vita dei blob; schema e versione del backup; spazio su unità rimovibili, test per il modello di sicurezza dei percorsi; eventuale riuso prudente delle primitive di allegati-incasso senza estendere autorizzazioni. **Nessuna nuova libreria o dipendenza viene selezionata in questa scheda.**

**Alternative:**
1. **Solo avviso + preflight** in invio: minimo costo, non rende trasportabili gli allegati.
2. **Solo copia gestita**, senza link esterni: più semplice da spiegare ma aumenta spazio, rischio privacy e tempi di backup per tutti.
3. **Link o copia con scelta esplicita**: risponde a bisogni diversi ma aggiunge stati e manutenzione.
4. **Nessun cambiamento**: documentare bene il limite e richiedere di riselezionare i file prima dell'invio; valida se le bozze con allegati sono rare.
5. **Salvare file in un archivio documentale generico** (TECH-016): possibile componente futuro, ma troppo ampio se il problema riguarda solo bozze.

**Rischi:** copia di documenti sensibili nei backup non cifrati, crescita rapida dell'archivio, accesso a percorsi non autorizzati, TOCTOU durante hashing/lettura, perdita di file per garbage collection aggressiva, false conferme di portabilità, migrazioni non retrocompatibili, invio di versione diversa da quella mostrata, prestazioni su NAS/USB, errori di semantica della copia «inviati» IMAP. **Complessità** bassa–media per i soli avvisi con verifica; **media–alta** per copie gestite, lifecycle, formato portabile e ripristino robusto.

## 8. Criteri di accettazione e prove future **non eseguite**

1. Bozza legacy con percorso USB: apribile offline dopo disconnessione, contenuto testuale intatto, allegato «Mancante», **nessun invio** con tale riferimento.
2. File esterno sostituito con contenuto diverso ma **stessa dimensione e nome**: rilevare la divergenza rispetto all'impronta accettata e non inviare automaticamente il sostituto.
3. Invio dopo verifica: destinatari e byte effettivamente inviati devono coincidere con l'anteprima confermata; nessun cambio invisibile fra controllo e costruzione MIME; in mancanza di prova, bloccare con messaggio.
4. Copia gestita: file esterno eliminato, backup su cartella scelta, ripristino su altro computer: byte e hash dell'allegato corrispondono a quelli salvati; in caso di hash alterato il ripristino è rifiutato prima di sostituire l'archivio attivo.
5. Crash fra copia del file e commit di SQLite, disco pieno, annullamento, bozza modificata da un'altra finestra: nessun riferimento permanente a una copia incompleta; nessuna cancellazione di originali o file condivisi.
6. Due bozze referenziano lo stesso blob: cancellarne una non cancella il file necessario all'altra. Nessuna pulizia automatica di blob con referenze non verificate.
7. File speciale, symlink, path traversal, Unicode, nome con caratteri speciali, permessi negati, cartella su NAS sconnesso: errore sicuro e leggibile, nessuna lettura fuori scope né trasmissione remota.
8. Verificare limiti 5 × 5 MiB / totale 15 MiB sia sul salvataggio, sia prima dell'invio, su Windows/macOS/Linux; backup resta completo e i CSV/certificati invariati.
9. Password non incluse nel backup, nessun allegato esposto via MCP o diagnostica; avvisi su copie in chiaro e rimozione distinguono archivio attivo e backup storici.
10. IT/EN/ES/FR/DE, fallback inglese, navigazione tastiera e screen reader, zoom e flussi da chiavetta USB; nessuna mail viene inviata come effetto di apertura, salvataggio o ripristino bozza.
11. Incertezza SMTP o errore IMAP Inviati: nessun retry automatico, messaggio conservativo e integrità della bozza preservata secondo il comportamento esistente.

## 9. Priorità, fasi ed esiti alternativi

- **Fase 0 — validazione:** verificare con alcuni artisti/gallerie se le bozze con allegati vengono usate, se sono trasferite tra dispositivi e quali file vengono più spesso persi o modificati. Esaminare esempi **sintetici**, non accedere alle caselle personali.
- **Fase 1 — minima:** documentazione in UI e preflight locale per file collegati; usare le garanzie di invio manuale esistenti, senza modificare il backup.
- **Fase 2 — solo su domanda concreta:** copia gestita con consenso, backup verificato, gestione reversibile e ripristino multipiattaforma.
- **Fase 3 — eventuale:** export/import aperto delle bozze con allegati gestiti, specifica versionata e dimostrazione di round-trip su archivio indipendente; prima valutare TECH-001 e TECH-016.

**P2 condizionata**, non approvata. Non implementare può essere un esito corretto se la validazione dimostra bisogno marginale.

## 10. Questioni aperte per la revisione del proprietario

1. La Posta professionale è usata per bozze con allegati a lunga durata o solo per invii immediati?
2. È preferibile conservare soltanto collegamenti con avviso, oppure offrire copie gestite; quale dovrebbe essere il default **esplicito**?
3. Le copie gestite vanno eliminate quando la mail risulta inviata, quando si elimina la bozza, oppure soltanto su comando dedicato? Come spiegare che la consegna SMTP non è dimostrata?
4. Sono accettabili gli attuali limiti 5 file / 5 MiB ciascuno / 15 MiB in totale?
5. La portabilità delle bozze è una priorità rispetto alla privacy e al peso dei backup? È preferibile aspettare TECH-016/001?
6. Come gestire una bozza legacy dopo ripristino su un nuovo dispositivo, senza inventare file mancanti o inviare contenuto diverso?
7. Quali verifiche native Windows/macOS/Linux e su NAS/USB sono realisticamente sostenibili?

**Conclusione:** **Proposta — da valutare**. Questo file è solo un documento Markdown di analisi, non autorizza sviluppo, migrazioni, test, issue operative o modifica di codice. Nessun file dell'altro filone `docs/idee/gestione-e-ux/` viene modificato. [PR tecnica #16](https://github.com/Cesare-ml/Repertica-Studio/pull/16) da mantenere aperta, in bozza e senza merge.
