# REPS-UX-011 — Cicli creativi e relazioni fra opere uniche

- **ID stabile:** REPS-UX-011
- **Data:** 2026-11-21
- **Stato:** Proposta — da valutare
- **Filone:** Gestione e UX
- **Priorità ipotetica:** P2 (da confermare con artisti, studiosi e piccoli archivi)
- **Perimetro:** solo studio documentale, nessuna implementazione né approvazione

## Motivazione e distinzione dalle funzioni presenti

In `main` ogni opera è una scheda autonoma (UUID e codice univoco) con titolo, tecnica, supporto, anno, dimensioni, descrizione, stato, ubicazione e fotografie. Le opere possono essere ricercate ma non sono **collegate esplicitamente in un ciclo creativo, dittico, polittico, serie di dipinti unici o sequenza di studi preparatori**. La possibilità di usare parole uguali nel titolo o nella descrizione non fornisce legami stabili e leggibili.

Questa proposta è **diversa da [UX-005](REPS-UX-005-edizioni-multipli-serie-numerate.md)**: una *edizione* raggruppa **esemplari fisici** di una composizione/tiratura con numerazione e disponibilità individuale; un **ciclo creativo** raggruppa **opere autonome, spesso uniche**, senza generare copie, creare numerazioni, stock o vendite. È distinta anche da [UX-009](REPS-UX-009-progetti-espositivi-checklist.md), che raccoglie opere **per una mostra temporanea**, e da [UX-007](REPS-UX-007-dossier-presentazione-opere.md), che prepara documenti da condividere. La ricerca federata [TECH-004](https://github.com/Cesare-ml/Repertica-Studio/pull/16) riguarda più archivi, non relazioni creative all'interno di un archivio.

## Destinatari e casi d'uso

1. **Artista con un ciclo di dieci dipinti:** assegna manualmente dieci schede a un gruppo «Paesaggi interiori», imposta l'ordine di presentazione e vede quali opere siano disponibili o vendute; la semplice associazione non cambia disponibilità e prezzi.
2. **Studio preparatorio:** collega un bozzetto a un dipinto concluso mediante relazione orientata «studio preparatorio di»; il bozzetto mantiene anno, dimensioni, foto, eventuale vendita e certificato indipendenti.
3. **Dittico di opere autonome:** due opere fisiche, ognuna con codice e fotografia, hanno relazione reciproca «opera accoppiata a»; un collezionista può comprare una sola opera senza che l'altra venga segnata automaticamente come venduta.
4. **Archivio retrospettivo:** un curatore cataloga il rapporto *dichiarato* fra opere di epoche diverse, con nota e fonte, senza costruire per somiglianza o titolo una genealogia presentata come certa.

## Esperienza utente proposta

- **Scheda opera → Cicli e collegamenti:** mostra i gruppi cui appartiene e le relazioni dirette («studio di», «derivata da», «abbinata a»); clic sui collegamenti per aprire altre schede, con indicazione chiara di stati e dati mancanti.
- **Cicli → Nuovo ciclo:** nome, testo introduttivo, intervallo cronologico **facoltativo**, immagine rappresentativa opzionale derivata da una delle opere, ordine manuale, note e fonte/affidabilità se l'attribuzione del ciclo è ricostruita.
- **Aggiungi opere:** ricerca locale, selezione esplicita delle schede esistenti, eventuale più di un ciclo per opera, ordinamento accessibile con pulsanti «sposta su/giù» anziché drag-and-drop obbligatorio. Anteprima segnala duplicati nello stesso ciclo.
- **Crea relazione:** scegli opera di partenza, opera correlata, tipo di relazione, nota/source; per un legame direzionale l'interfaccia mostra anche la relazione inversa con formulazione coerente («studio preparatorio di» / «ha studio preparatorio»); relazioni simmetriche sono mostrate su entrambi i lati.
- **Vista ciclo:** copertina facoltativa, elenco opere, numerazione **di presentazione** distinta dall'eventuale numero di edizione, filtri per anno/tecnica/stato; scheda stampabile a uso interno, senza esportare automaticamente recapiti o valori economici.
- **Sicurezza UX e accessibilità:** conferma per rimozione di una relazione e possibilità di correzione tracciata; nessuna eliminazione dell'opera collegata. Viste leggibili da tastiera/screen reader e non dipendenti dai soli colori.

## Modello informativo ipotetico (non schema da implementare)

- `creative_cycle`: UUID stabile, nome, descrizione, cronologia descrittiva o intervallo facoltativo, origine della definizione (artista/dichiarazione archivistica/attribuzione incerta), note, date creazione/modifica; **non** è un'opera vendibile e non possiede stato commerciale.
- `cycle_membership`: UUID, `cycle_id`, `artwork_id`, posizione/ordine, ruolo testuale facoltativo (es. dipinto, bozzetto), fonte/certezza della relazione, note. Unicità `(cycle_id, artwork_id)` per evitare doppio conteggio; un'opera può partecipare a più gruppi.
- `artwork_relation`: UUID, `source_artwork_id`, `target_artwork_id`, tipo codificato (`preparatory_for`, `derived_from`, `paired_with`, `related`), direzione/simmetria, fonte, note e traccia di rettifica. Non permettere collegamenti a sé; evitare cicli contraddittori nelle relazioni dichiarate come genealogiche **solo dopo aver concordato semantica dei tipi**.
- **Confini:** `artworks.id/code`, vendita unica per opera, certificati storici e fotografie restano invariati; il gruppo non eredita stato `sold` da un membro, non somma prezzi come ricavi e non crea certificato collettivo.
- **Origine delle associazioni:** manuale, mai dedotta automaticamente da somiglianza visiva, titolo, numero d'inventario o IA. Dati incerti devono essere mostrati come **attribuzioni**, non fatti verificati.
- **Storia:** modifiche all'appartenenza e alle relazioni possono essere annullate o rettificate con registrazione minima; distinguere «gruppo corrente» da eventuali fotografie o dossier **storici** già condivisi.

## Offline, impostazioni e localizzazione

- Operazioni esclusivamente su SQLite locale; nessun account, cloud, analisi IA, invio a piattaforme o accesso remoto. Se un'immagine manca, la vista mostra un segnaposto di indisponibilità senza scaricarla da Internet.
- Impostazioni opzionali: visualizzazione di più gruppi per opera, etichette dei tipi di relazione e ordinamento predefinito; non forzare mai un gruppo per le opere già presenti. Evitare gerarchie annidate nel primo esperimento per non introdurre percorsi ambigui.
- UI nella lingua del sistema, override IT/EN/ES/FR/DE, fallback inglese secondo [ECONOMY-I18N](../../ECONOMY-I18N.md). Etichette/tipi tradotti, **nomi dei cicli e note libere no**; date e ordinamento presentati secondo locale. Identificativi/tipi di relazione salvati con codici stabili non tradotti.

## Compatibilità, esportazione, importazione e privacy

- **Retrocompatibilità:** tutti i record opera storici devono rimanere singoli e visibili senza obbligo di appartenenza; eventuale schema nuovo deve essere additivo e versionato. Versioni vecchie dell'app non devono promettere la lettura di schemi più nuovi.
- **Backup:** i nuovi link e gruppi, se un giorno implementati, devono entrare nell'archivio locale verificato con salvataggio/ripristino su cartella liberamente scelta; l'immagine di copertina del ciclo non deve creare dipendenze irreversibili da file esterni. Nessun cambio al backup in questa proposta.
- **Export locale:** CSV dei gruppi e delle appartenenze con UUID e ordine per analisi; JSON versionato per relazione `cycle → member → artwork` e link tipizzati (con fonte e incertezza), coerente concettualmente con [TECH-001](https://github.com/Cesare-ml/Repertica-Studio/pull/16). Non sostituisce i cinque CSV e il backup esistenti.
- **Import:** file scelto esplicitamente, anteprima conflitti per UUID/codice, deduplica idempotente, rifiuto di relazioni senza opere esistenti salvo mappatura guidata, rollback transazionale. Nessuna creazione silenziosa di opere o vendite.
- **Privacy:** descrizioni/sorgenti potrebbero includere nomi di terzi, collezionisti e ubicazioni; esportazioni pubbliche **minime**, con anteprima e scelta dei campi. Escludere per default note private, contatti, importi, ubicazioni e link a file locali. La condivisione di un dossier UX-007, se mai sviluppato, resta un'azione esplicita separata.

## Complessità, rischi e alternative

- **Complessità:** bassa-media per elenco di gruppi e associazioni; media per relazioni tipizzate bidirezionali e navigazione; medio-alta per storia, importazioni e gerarchie di ciclo complesse.
- **Dipendenze:** ID stabili delle opere, ricerca già presente, i18n e backup SQLite; nessuna dipendenza dall'approvazione di UX-005/007/009 o dalle proposte tecniche.
- **Rischi:** confondere ciclo artistico con tiratura di multipli, raggruppare in modo storicamente inesatto, introdurre cicli genealogici incoerenti, trattare un insieme come un prodotto vendibile, duplicare le «opere esposte» di UX-009.
- **Alternative:** A) semplice tag libero «ciclo» su ogni opera (minima interazione, rischio di duplicati e grafie diverse); B) gruppi manuali senza relazioni orientate (MVP leggero); C) gruppi e relazioni tipizzate, solo dopo validazione del bisogno.
- **Contromisure:** nomi di sezioni distinti «Cicli creativi» vs «Edizioni numerate», riferimenti a fonti e incertezza, avvisi nei flussi, nessuna propagazione automatica di stati o ricavi, possibilità di iniziare da gruppi semplici.

## Test e criteri di accettazione ipotetici

1. Un ciclo con otto opere uniche mostra otto membri; vendere un'opera non modifica le altre sette né il gruppo in un'«edizione venduta».
2. La stessa opera può appartenere a due cicli senza duplicare record, foto, vendite o certificati.
3. Un bozzetto collegato a un dipinto mostra relazione inversa corretta, senza autorelazioni o link duplicati; un dittico usa relazione simmetrica.
4. Eliminare una membership non elimina l'opera; cancellazioni di opere con certificati/vendite rimangono soggette ai vincoli attuali.
5. Un archivio esistente si apre invariato, con nessuna associazione presunta; backup/ripristino preservano tutti gli UUID e gli ordini dei gruppi.
6. Export/import locale ripetuto non duplica cicli o relazioni; opere mancanti sono segnalate prima di qualsiasi scrittura.
7. Una vista pubblica non rivela note private, contatti, prezzi storici, percorsi dei media o ubicazioni senza autorizzazione esplicita.
8. Ricerca, aggiunta, navigazione e ordinamento utilizzabili da tastiera/screen reader; funzionamento senza rete, lingua del sistema/fallback inglese e test Windows/macOS/Linux.

## Fasi di esplorazione, NON piano autorizzato

- **A:** parlare con artisti che producono cicli/dittici e verificare se basta un tag o servono gruppi manuali.
- **B:** gruppi con appartenenza e ordinamento, viste e filtri; nessuna relazione genealogica nella versione minima.
- **C:** legami tipizzati con fonte/rettifiche, export/import semantico e verifica di compatibilità con dossier e mostre, se eventualmente approvati.

## Distinzione fra dati certi, ipotesi e decisioni del proprietario

- **Certi dal repository:** oggi le opere hanno identità, codice e stato indipendenti, una vendita massima per record, certificati immutabili; non esiste un'entità ciclo creativo nel modello SQLite attuale. Lingue e backup locali sono già previsti.
- **Ipotesi:** molti artisti lavorano in serie di originali unici; gestirli come insieme può migliorare consultazione e ricerca senza mutare le singole schede.
- **Domande:** Repertica vuole catalogare prima **cicli** o **relazioni preparatorie**? Un'opera può stare in più cicli? Che differenza di etichetta serve fra «serie creativa», «edizione di multipli» e «mostra»? Vanno supportate relazioni con opere **non presenti nell'archivio**, come riferimenti puramente testuali? Serve un numero d'ordine persistente o soltanto un ordinamento UI?

**Stato: Proposta — da valutare. Nessun lavoro di sviluppo avviato o autorizzato.**
