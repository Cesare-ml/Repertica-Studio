# REPS-UX-054 — Vista temporanea di presentazione delle opere, senza dati gestionali sullo schermo

- **ID stabile:** REPS-UX-054
- **Data:** 2026-11-22
- **Stato:** Proposta — da valutare
- **Filone:** Gestione e UX
- **Priorità ipotetica:** P2 solo se la necessità emerge da prove con artisti/gallerie; altrimenti P3 o nessuno sviluppo
- **Decisione:** nessuna approvazione, assegnazione o implementazione autorizzata; questa è una scheda di studio

## 1. Problema e riscontri effettivi

**Situazione professionale:** un artista mostra le proprie opere sul portatile in studio, proietta il computer durante un incontro con un curatore, o condivide la finestra di Repertica in una videochiamata. Deve poter sfogliare immagini e dati essenziali senza mostrare accidentalmente chi ha acquistato altre opere, prezzi riservati, note, ubicazioni, incassi o margini.

**Constatazioni verificate sul repository `main` il 22/11/2026:**

- `README.md` e `src/app/app.component.html` descrivono/mostrano un archivio offline con tabella delle opere che include **prezzo** e **collezionista**, oltre a sezioni gestionali. Il prezzo calcolato o fisso non coincide necessariamente con il prezzo effettivamente concordato.
- `src/app/artwork-viewer.component.ts` contiene già un visualizzatore a schermo intero con navigazione fra opere e fotografie. La sua colonna informativa può mostrare **prezzo**, **«Venduta a»** con nome del collezionista, **stato**, **descrizione integrale** e pulsanti **«Modifica opera»** / **«Gestisci le fotografie»**. È utile all'operatore, ma non costituisce una vista prudente per interlocutori esterni.
- `docs/DATA-PORTABILITY.md` documenta esportazioni complete dei registri per portabilità, non una protezione della visualizzazione in presenza di ospiti.
- Nessuna prova di divulgazione reale o bug è stata raccolta. È una **lacuna d'esperienza potenziale**: da verificare con utenti reali, non una vulnerabilità attestata.

**Destinatari:** artisti che ricevono visitatori; gallerie che presentano un insieme di opere a clienti; archivi e curatori che esaminano selezioni davanti a terzi; professionisti che fanno condivisione schermo.

**Riferimento orientativo esterno:** la [Information and Data Policy del Rijksmuseum](https://data.rijksmuseum.nl/policy/information-and-data-policy) distingue divulgazione di informazioni sulle collezioni e limitazioni per privacy/sicurezza/obblighi contrattuali (politica in vigore dal 17/07/2024). Non è una specifica per Repertica né un obbligo legale automatico: sostiene soltanto la necessità di decidere **quali dati rendere visibili**.

## 2. Confine con idee esistenti — anti-doppioni

- **UX-007:** dossier PDF/schede *esportati* verso destinatari. Qui si studia la **presentazione interattiva temporanea dentro l'app**, senza creare né inviare documenti.
- **UX-041:** titoli/descrizioni per lingua. L'eventuale scelta di un testo destinato al pubblico potrebbe riusare quella futura capacità, **senza richiederla**: nella prima fase la descrizione libera resta nascosta.
- **UX-053:** cartellini di sala stampabili. Non è una presentazione a schermo.
- **UX-035:** preferenze di contatto. Non concede permessi di esporre dati personali.
- **TECH-025:** proiezioni **CSV** con colonne condivisibili; non riguarda la UI temporanea.
- **TECH-013:** copie fotografiche condivisibili e rimozione metadati; non serve se si guardano le immagini **solo in locale**, ma sarebbe pertinente se la vista prevedesse un successivo export.
- **TECH-020:** accessibilità dei flussi critici; criterio comune da rispettare, non da replicare qui.
- **TECH-014/049:** confini MCP: questa vista **non modifica** privilegi, strumenti, risultati o disponibilità del server MCP.

**Esclusioni:** non è un account «ospite», non è un blocco del computer, non è DRM, non protegge contro chi usa fisicamente il computer o accede al database; non si pubblica su Repertica Sites, non si sincronizza, non si crea una modalità cloud, non si registrano visualizzazioni dei visitatori.

## 3. Scenari d'uso e flussi UX

1. **Visita in studio:** l'artista apre la scheda di una tela, sceglie «Presenta opere» e vede un'anteprima in cui sono già nascosti acquirente, prezzo, note, ubicazione e controlli di modifica. Sfoglia con frecce e chiude quando la visita termina.
2. **Condivisione schermo:** un curatore vede soltanto fotografie, titolo, anno, tecnica e dimensioni. La finestra non mostra sezioni economiche durante la navigazione; l'operatore è avvertito che notifiche di sistema, altre finestre o condivisione dell'intero desktop restano fuori dal controllo di Repertica.
3. **Galleria con opere vendute:** l'operatore può eventualmente includere opere vendute **senza** nome del compratore o importo. L'eventuale indicazione della disponibilità è una scelta esplicita, non un'inferenza commerciale.
4. **Dati incompleti/sensibili:** se una didascalia o una descrizione contiene dati riservati, **non viene mostrata** di default; non esiste un algoritmo che ne certifichi automaticamente la sicurezza.

**Flusso minimo proposto, senza un nuovo editor visuale:**

- Dalla scheda opera o dal visualizzatore esistente: azione secondaria **«Presenta senza dati gestionali»**. Se la selezione multipla non è ancora disponibile, la fase 1 può limitarsi a sfogliare le opere secondo l'ordine attuale, con scelta esplicita di mostrare solo quelle disponibili; nessuna dipendenza obbligatoria da future liste/viste salvate.
- **Anteprima prudente** prima di mostrare lo schermo: immagine e **soltanto** codice facoltativo, titolo, anno, tecnica/supporto, dimensioni; nessun testo libero, prezzo, nome del proprietario/acquirente, stato di vendita, ubicazione, note, dati di incasso, pulsante di modifica, collegamento alla rubrica o statistiche.
- Il contenuto della presentazione usa uno sfondo **opaco**: nessuna tabella gestionale leggibile dietro pannelli traslucidi, nessuna anteprima di dati nel titolo della finestra, tooltip, focus nascosto o menu contestuale. Una grafica semplice con immagini grandi e pulsanti precedente/successiva, chiusura sempre identificabile.
- Un'indicazione discreta **«Vista di presentazione — non è una protezione dell'archivio»** ricorda il limite senza interferire con le immagini. Uscita con un controllo ben visibile e `Esc`; il ritorno all'archivio gestionale è sempre **volontario**, non provocato dai tasti freccia di navigazione.
- Nel caso l'operatore voglia mostrare anche descrizione o prezzo richiesto, fase **successiva** con campi selezionati individualmente e nuova anteprima; mai mostrare il prezzo effettivo di vendita o i recapiti del collezionista per default. Il prezzo richiesto (UX-023) rimane distinto dall'importo venduto.

**Aspetti UX/accessibilità:** navigazione completa da tastiera, focus visibile, tasti con etichette accessibili, annuncio dell'opera corrente, supporto allo zoom sistema, contrasto e layout reattivo su portatili/proiettori, niente swipe obbligatori, indicazioni non affidate al solo colore. Se l'utente preme `Alt+Tab` o esce dalla finestra, non promettere schermatura del sistema operativo.

## 4. Dati necessari e ipotesi descrittive

**Requisiti certi del contesto:** esistono schede opera con titolo/codice/anno/tecnica/supporto/dimensioni e foto; esiste un viewer; esistono dati economici/personali da non mostrare. Non occorre creare un nuovo archivio.

**Ipotesi funzionale di fase 1:** semplice stato **temporaneo in memoria**, senza migrazione SQLite, senza cronologia ospiti e senza log di visite. Esempio puramente concettuale:

```text
PresentationViewV1 {
  artwork_ids: [ArtUUID],          // eventualmente intero catalogo/solo disponibili
  initial_artwork_id: ArtUUID,
  allowlisted_fields: ["title", "year", "technique", "support", "dimensions"],
  include_code: false,
  show_status: false,
  show_free_text: false,
  show_price: false,
  session_persistent: false
}
```

**Non è una API da sviluppare.** Un'eventuale preferenza locale futura per il formato della presentazione deve essere separata da impostazioni di privacy e permessi: il default rimane prudente a ogni apertura. Non creare duplicati delle fotografie o dei record. Nessuna modifica a stato, certificati, vendite, quote o backup. Una vista con dati in memoria non diventa evidenza storica dell'opera.

**Domanda di architettura da rivedere:** per evitare che dati riservati siano caricati inutilmente nel componente, sarebbe più affidabile una proiezione di soli campi consentiti. Ma la semplice limitazione della UI **non equivale a protezione del database**, né permette di qualificare il prodotto come multiutente sicuro. Eventuali progettazioni backend competono al futuro filone tecnico soltanto se il proprietario approva la funzione.

## 5. Offline, import/export, backup e compatibilità

- **100% offline**, su Tauri e SQLite locale, senza registrazione, abbonamento, API esterne o account. Le fotografie già archiviate vengono lette localmente.
- **Import:** nessuno nuovo; utilizza opere e immagini esistenti. Titoli mancanti restano esplicitamente mancanti; non inventare dati. Un eventuale elenco di ID selezionati si riferisce solo all'archivio attivo.
- **Export:** **nessuna esportazione dalla vista minima**. La portabilità integrale dell'archivio e i 12 CSV rimangono immutati. UX-007 e UX-053 restano i percorsi separati per documenti destinati all'esterno.
- **Backup/ripristino:** nessuna sessione temporanea da salvare. Se in futuro esistessero preset esplicitamente salvati, risiederebbero nell'archivio locale e seguirebbero backup completo e ripristino in cartella liberamente scelta, senza percorsi assoluti fragili; ripristinare un archivio non riapre una presentazione.
- **Compatibilità:** prima fase senza nuovi campi obbligatori e senza migrazioni di schede storiche, certificati e vendite; foto mancanti = stato vuoto comprensibile. Nessuna dipendenza dalla connessione o dalla lingua installata.
- **Multilingua:** interfaccia nella lingua di sistema, scelta manuale già prevista, fallback **inglese** per locale o singola traduzione non disponibile. Titoli/tecniche degli utenti **non vengono tradotti automaticamente**. Testare EN/IT/ES/FR/DE e lingue OS non supportate.

## 6. Sicurezza, privacy e limiti espliciti

- La vista è una **riduzione del rischio di divulgazione involontaria sullo schermo**, non un meccanismo di controllo degli accessi. Chi può chiuderla può accedere all'interfaccia completa; chi possiede file e backup può leggere i dati secondo le protezioni del proprio dispositivo. Nessun PIN cosmetico, nessun falso «accesso pubblico sicuro».
- Prezzi, nomi degli acquirenti, spese, percentuali e note libere sono esclusi di default. Le didascalie delle immagini e i nomi dei file possono anch'essi contenere informazioni personali: mostrarli soltanto dopo revisione, se mai implementata.
- Le immagini possono essere soggette a diritti di riproduzione o accordi contrattuali: la presenza nell'archivio **non prova** il diritto di proiettarle pubblicamente (relazione con UX-010).
- Screen sharing del desktop, screenshot, fotocamere, notifiche e memorie cache di altre applicazioni sono fuori dall'ambito: testo esplicito prima dell'uso.
- Mai inviare automaticamente screenshot, telemetria, cataloghi o nomi dei visitatori. Nessun profilo comportamento o statistiche di visione.
- Se la proiezione è attiva, l'utente può deliberatamente uscirne; non si promette che il sistema impedisca ogni gesto che riporti ai dati riservati.

## 7. Complessità, dipendenze, alternative e rischi

**Stima concettuale:** **bassa–media** per un'unica vista locale di sola consultazione riusando il viewer; **media–alta** se si introducono selezione multipla, profili persistenti, testi pubblici, secondo schermo e forti garanzie anti-divulgazione. Non è una stima di tempi/costi.

**Dipendenze già presenti:** modello Artwork, visualizzatore e galleria fotografica, localizzazione dell'interfaccia. **Non richieste**: login, AI, server, MCP, Sites, pubblicazione, motore PDF, nuove API di sincronizzazione, traduzioni automatiche.

**Alternative da provare prima di sviluppare:** usare il viewer già esistente in una sessione offline con **opere dimostrative senza dati riservati**; usare un dossier/brochure locale già disponibile oppure un PDF realizzato in LibreOffice. Per semplici incontri potrebbe essere più sicuro e conveniente un PDF statico rispetto a un'interfaccia che si può chiudere accidentalmente. Se funziona, **nessun nuovo sviluppo**.

**Rischi:** falsa sensazione di sicurezza; testi liberi o didascalie con contenuto riservato; prezzo calcolato scambiato per prezzo concordato; nomi collezionisti in hover/aria-label; esposizione di altri dati chiudendo la vista; immagini molto grandi che rallentano una presentazione; rottura dell'accessibilità; duplicazione di UX-007 se la vista degenera in un generatore di brochure.

## 8. Criteri di accettazione da usare **solo se** la proposta verrà approvata

1. Aprendo una presentazione da un archivio di prova con **prezzi, contatti, note e ubicazioni sensibili**, nessuno di tali dati appare nel contenuto visivo o nei controlli accessibili, nei tooltip o nel pannello dietro la vista.
2. Frecce precedenti/successive e scelta fotografie **non espongono** interfaccia di vendita, rubrica, modifica o export. `Esc` chiude chiaramente la presentazione; il passaggio alla normale app richiede azione deliberata.
3. Descrizioni libere, note e didascalie non sono visualizzate per default, incluse quelle che incorporano numeri di telefono; non si promette detection semantica.
4. Opere con zero immagini, immagini grandi, titoli lunghi e tecnica/anno mancanti mostrano un layout stabile, comprensibile e accessibile.
5. Prove native Windows/macOS con tastiera, screen reader, zoom e secondo schermo; UI in IT/EN/ES/FR/DE e fallback EN su OS non supportato.
6. Nessuna modifica di record, certificati, stato, vendite o percentuali; backup completo, ripristino e CSV continuano a funzionare come prima.
7. Documentazione e avviso spiegano esplicitamente che **non è una barriera di accesso**, non disattiva notifiche OS e non autorizza la riproduzione delle opere.
8. Test con artisti/gallerie: mostrare 10–15 opere a un interlocutore senza dover creare un secondo archivio e senza rivelare dati non selezionati; confronto cronometrato con brochure/PDF già esistente.

## 9. Fasi ipotetiche e priorità

- **Fase 0, ricerca (senza codice):** chiedere a 3–5 artisti/gallerie come mostrano oggi il catalogo. Provare viewer attuale e dossier/PDF con dati sintetici; osservare dove i partecipanti incontrano dati indesiderati e se preferiscono un documento statico.
- **Fase 1, eventuale MVP:** vista in memoria a campi pubblici fissi, immagini locali, background opaco, navigazione e uscita volontaria; nessun preset, export o nuova tabella.
- **Fase 2, opzionale:** selezione più precisa delle opere, ordine, campi aggiuntivi mediante anteprima, testi destinati alla presentazione e profili locali conservativi; solo se la fase 1 viene usata davvero.
- **Fase 3, non necessaria di default:** secondo schermo o strumenti di conferenza; rischi e complessità da valutare separatamente.

**Priorità proposta:** **P2 condizionata** a uso frequente durante incontri reali; **P3 o nessuno sviluppo** se il PDF/dossier copre l'esigenza. Non approvata.

## 10. Domande per il proprietario (non richiedono risposta per questa documentazione)

1. L'utente tipico mostra davvero Repertica sullo stesso schermo di un cliente, oppure prepara sempre PDF/immagini prima degli incontri?
2. Nella prima fase bastano tutte le opere disponibili, oppure è imprescindibile una **selezione manuale** di 5–15 pezzi? Il viewer mostra già opere vendute: vanno escluse di default?
3. Titolo, tecnica e dimensioni sono sempre pubblicabili? Il codice archivio potrebbe essere una referenza riservata?
4. Vale la pena offrire un prezzo di listino opzionale o è meglio evitare qualunque cifra durante la presentazione?
5. Preferire una vista interattiva con uscita esplicita oppure un export temporaneo PDF separato dal gestionale?
6. Ci sono accordi di riservatezza o diritti fotografici che rendono alcune immagini **non presentabili** a terzi?

**Esito:** proposta ideativa, senza prova definitiva di bisogno, senza autorizzazione di implementazione, senza modifica ad altro filone. Stato immutato: **Proposta — da valutare**.
