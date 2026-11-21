# REPS-UX-004 — Rapporti di condizione e conservazione delle opere

- **Data di proposta:** 2026-11-21
- **Stato:** Proposta — da valutare
- **Filone:** Gestione e UX
- **Priorità ipotetica:** P2 (da discutere; P1 soltanto per archivi che movimentano spesso opere)
- **Perimetro:** esclusivamente ideativo/documentale. Non autorizza sviluppo, modifiche al database, nuovi workflow o procedure di restauro.

## Perché serve e a chi

Un artista, una galleria, un archivista o un conservatore può avere fotografie e note sparse, ma non un **rapporto datato e confrontabile** sulle condizioni di una specifica opera. Prima e dopo trasporti, mostre o prestiti è utile attestare *cosa è stato osservato, da chi e quando*, senza confondere una constatazione con una certificazione di autenticità o una perizia specialistica. Lo storico può aiutare a documentare danni, interventi e responsabilità, ma **non costituisce automaticamente prova legale**.

**Differenza dall'esistente:** il [MVP](../../MVP.md) gestisce descrizione, tecnica, ubicazione, foto principale e certificati immutabili, non un registro di rilievi conservativi. [REPS-UX-001](REPS-UX-001-provenienza-movimentazioni.md) propone cronologia di movimenti e un'osservazione sintetica delle condizioni: qui si dettagliano **rilievi, rilievi comparativi e conservazione**, eventualmente collegabili a un movimento, senza gestirne proprietà, posizione o provenienza. [REPS-TECH-003](https://github.com/Cesare-ml/Repertica-Studio/blob/docs/idee-strumenti-architettura-20261121/docs/idee/strumenti-e-architettura/REPS-TECH-003-diagnostica-integrita-media.md) verifica l'**integrità dei file digitali**, non la condizione fisica dell'opera. La [PR #17](https://github.com/Cesare-ml/Repertica-Studio/pull/17) propone una galleria fotografica aggiuntiva **non ancora unita a main**: un rapporto potrebbe riutilizzare immagini esistenti, ma non deve dipendere dalla PR o considerarla implementata.

## Scenari professionali

1. **Prima di una mostra:** un dipinto lascia lo studio. Si redige un rapporto preliminare con data, autore del rilievo, fotografie di recto/verso/dettagli, annotazione «piccola abrasione sul bordo», stato «osservato», eventuale documento cartaceo.
2. **Al rientro:** si apre il rapporto precedente in modalità confronto e si compila un nuovo rilievo. Le differenze sono segnalate come *osservazioni*, non come danni automaticamente attribuiti al trasportatore.
3. **Intervento di conservazione:** si registra un intervento eseguito da un professionista esterno, allegando documentazione e dichiarando chi l'ha riferito. Il software non suggerisce trattamenti.
4. **Archivio storico:** una fotografia non datata e una nota di vecchio proprietario vengono associate come fonti con data incerta e attendibilità **non verificata**, senza farle apparire come rilievi attuali.

## Esperienza utente e flusso

- **Opera → Conservazione:** riepilogo «ultimo rilievo disponibile» con **data e autore**, cronologia dei rapporti, filtro per fase (prima/dopo trasporto, inventario, manutenzione), e azione **Nuovo rapporto**.
- **Compilazione guidata:** identificazione opera (sola lettura), data/precisione, luogo e osservatore, ambito del controllo, stato generale opzionale con significato esplicito; sezioni libere per supporto, superficie, cornice, imballaggio e altri elementi personalizzabili. Ogni difetto può avere descrizione, posizione testuale o annotazione su **copia dell'immagine** (mai alterare l'originale), foto di riferimento e livello di certezza.
- **Bozza → Conferma rapporto:** prima della conferma mostra riepilogo e avvertenza che l'attestazione descrive osservazioni personali. Rapporto confermato leggibile e stampabile; una correzione sostanziale genera **rettifica/versione collegata**, non sovrascrittura invisibile. Nessun invio automatico.
- **Confronta due rapporti:** due pannelli affiancati con date/fonti, elenco delle differenze dichiarate manualmente e confronto fotografico; non affidare a algoritmi visivi la diagnosi del degrado. Esportazione di un **rapporto leggibile** per consegna volontaria a terzi con anteprima di dati inclusi.
- **Accessibilità:** navigazione da tastiera e screen reader, campi descrittivi oltre a marcatori grafici, testo ad alto contrasto; nessuna gravità del difetto codificata soltanto dal colore.

## Dati minimi (ipotesi, NON migrazione approvata)

- `condition_report`: UUID, `artwork_id`, data e precisione, luogo facoltativo, osservatore come testo o contatto locale opzionale, tipo rilievo, sintesi, stato bozza/confermato/rettificato, autore e timestamp locali, `supersedes_report_id` opzionale; collegamento opzionale `movement_id` **solo se REPS-UX-001 sarà approvata**.
- `condition_observation`: UUID, report, categoria codificata stabile, descrizione, posizione descrittiva, intensità dichiarata opzionale, grado di certezza, testo libero. Le categorie sono **etichette di lavoro**, non codici di valutazione professionale universali.
- `condition_media`: UUID, report/osservazione, riferimento a copia locale nel media store, vista (fronte/retro/dettaglio), didascalia e, se presente, checksum **come integrità di byte**, non autenticazione. Non imporre lo schema immagini della PR #17.
- `conservation_event` opzionale: data, descrizione intervento, operatore dichiarato, fonti/allegati, relazione a uno o più rapporti; **non** registrazione di costo (quella appartiene al registro spese / [UX-003](REPS-UX-003-costo-e-margine-per-opera.md)).
- Foto preesistenti e certificati restano invariati. Evitare inferenze retroattive: senza rapporti registrati l'app indica «non documentato», **mai** «condizioni perfette».

## Offline, preferenze, multilingua e portabilità

- **Completamente locale:** SQLite e copie media dell'archivio; accesso ai file tramite selettore nativo, senza webcam attiva di default, cloud, telemetria o riconoscimento IA. L'utente deve poter usare il flusso senza account.
- **Impostazioni:** modello di checklist scelto esplicitamente (breve/dettagliato), lingua del rapporto esportato, visualizzazione dati osservatore, avvisi locali facoltativi per ispezione periodica **solo dentro l'app aperta**; nessun promemoria di sistema non richiesto.
- **Lingue:** interfaccia in IT/EN/ES/FR/DE secondo lingua di sistema, fallback inglese (vedi [ECONOMY-I18N](../../ECONOMY-I18N.md)); stringhe libere rimangono nella lingua scritta, non vengono tradotte automaticamente; codici di categorie e stati indipendenti dalla lingua UI.
- **Export:** PDF stampabile e CSV riepilogativo per data, stato e opera; JSON versionato con UUID, rapporti, osservazioni, riferimenti relativi ai file e manifest media per portabilità. Modalità **con dati interni / per terzi** con anteprima dei nomi privati, recapiti e documenti prima dell'esportazione. Import guidato con anteprima, relazione a opera esistente o nuova e rollback; mai alterare rapporti confermati in modo implicito.
- **Compatibilità:** backup e ripristino dovrebbero includere tutti i dati e allegati senza modificare i backup precedenti; migrazione additiva, archivi vecchi apribili senza rapporti; una futura interoperabilità con [REPS-TECH-001](https://github.com/Cesare-ml/Repertica-Studio/blob/docs/idee-strumenti-architettura-20261121/docs/idee/strumenti-e-architettura/REPS-TECH-001-pacchetto-dati-aperto.md) va coordinata se entrambi approvati.

## Dipendenze, rischi, alternative e complessità

- **Dipendenze:** ID stabili delle opere, archivio SQLite, gestione media e backup, stampa locale, localizzazione. Le schede UX-001/TECH-001 e la PR #17 sono **integrazioni opzionali**, non prerequisiti certi.
- **Rischi:** scambiare una scheda di ispezione per perizia o assicurazione; attribuire automaticamente responsabilità; perdere risoluzione delle foto; esportare osservazioni con dati personali; confondere hash del file con attestazione di genuinità; esplosione di media in archivio. Mitigare con etichette, campi fonte/certezza, copie non distruttive, anteprima export e backup collaudati.
- **Complessità:** media per checklist e rapporto datato; medio-alta per confronto fotografico, rettifiche storiche e allegati portabili; alta se si tentasse diagnosi computerizzata o strumenti professionali normati, **fuori ambito**.
- **Alternativa minima:** rapporti testuali datati con 1–3 foto e PDF, senza editor di annotazioni né strumenti di confronto.

## Criteri di accettazione ipotetici (per un'eventuale futura decisione)

1. Due rapporti sulla stessa opera con date differenti rimangono consultabili; una rettifica non cancella la versione confermata e mantiene collegamento e autore dichiarato.
2. Rientro da prestito e vendita non modificano automaticamente il rapporto; se UX-001 è presente, il legame a un movimento è esplicito e opzionale.
3. Foto e annotazioni non alterano il media originale o lo snapshot di un certificato immutabile.
4. Un'opera senza rapporto mostra «non documentato» e non inventa un punteggio conservativo; ogni dato mancante o incerto resta distinguibile da zero/assenza di difetti.
5. Export verso terzi può omettere nome osservatore e contatti; un round trip locale preserva UUID, cronologia, date, media e rettifiche.
6. Rapporto stampabile, compilazione completa da tastiera, stringhe localizzate e test senza connettività; backup/ripristino conservano tutti gli allegati.

## Fasi orientative, NON roadmap approvata

- **A:** modelli di rilievo brevi, cronologia, testo/foto e stampa locale.
- **B:** confronti, versioni/rettifiche, esportazione strutturata e filtri per rapporti mancanti.
- **C:** collegamento facoltativo a movimenti e interventi conservativi, sempre senza prescrivere trattamenti.

## Requisiti certi, ipotesi e domande al proprietario

- **Certi dal progetto:** offline-first, immagini locali e certificati immutabili; backup con media; lingue di sistema con fallback inglese; archivi basati su opere identificabili.
- **Ipotesi:** i fruitori necessitano una scheda conservativa separata; utile confrontare più rilievi nel tempo; un registro interventi ha valore senza contabilità dedicata.
- **Da decidere:** rapporti destinati anche a restauratori professionisti o solo artista/galleria? Quante fotografie per rapporto e quale limite dimensionale? Sono utili annotazioni puntuali o bastano descrizioni? È preferibile una checklist completamente libera o guidata? Come descrivere la responsabilità della firma senza fornire validità giuridica impropria?

**Nessuna idea approvata, nessuna implementazione autorizzata.**
