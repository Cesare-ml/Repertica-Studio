# Brochure offline — Publication Engine per Repertica Studio

## Origine del modulo

Adattamento **locale e indipendente** del codice e dei contratti editoriali del repository privato Cesare-ml/Gestionale-Opere, in particolare:

- docs/publication-engine/BROCHURE_EDITOR_ARCHITECTURE.md
- backend/services/publishing/publicationCompiler.js
- backend/services/publicationPdfRenderer.js
- backend/services/publishing/templates/artworkGalleryClassic.js
- backend/services/publishing/templates/artworkFullBleed.js
- frontend/src/app/components/publication-workspace/

Non si importa il backend Express/PostgreSQL, Puppeteer, la gestione utenti, il cloud, né la Media Library remota. Le scelte **persistenza strutturata → composizione → PDF** restano separate.

## Prima versione implementata

1. Voce **Brochure** nel menu desktop, con elenco, creazione, riapertura, duplicazione ed eliminazione.
2. Copertina con artista, titolo e sottotitolo; introduzione facoltativa; pagina contatti.
3. Aggiunta di opere dall'archivio con ordinamento; una sola occorrenza per opera.
4. Layout per opera: **Classica**, **A tutta pagina** e **Immagine e testo**.
5. Fotografia principale o foto aggiuntiva dell'opera; titolo e descrizione modificabili solo nella brochure.
6. Scelta della lingua del documento, separata da quella dell'interfaccia (IT/EN/ES/FR/DE).
7. Esportazione PDF A4 nativa mediante Rust e immagini JPEG convertite localmente. Nessun browser headless, processo esterno, upload o accesso a Internet.
8. Migrazione additiva SQLite schema 15, tabella brochures, con documenti JSON persistenti inclusi nel backup dell'archivio.

L'editor salva il documento strutturato nel database locale; il PDF è un derivato esportato in un percorso scelto dall'utente. L'eliminazione della brochure non elimina opere o immagini.

## Limitazioni dichiarate

- L'anteprima Angular è **illustrativa**: il PDF finale usa un renderer nativo Rust, non un renderer HTML condiviso.
- Tre template iniziali, non l'intero registro del Publication Engine cloud; niente selezione multi-immagine nella stessa pagina.
- I prezzi non sono inclusi: l'archivio desktop non ha ancora il contratto equivalente del sistema web.
- Font PDF standard Helvetica/Helvetica-Bold in WinAnsi: caratteri non latini possono essere sostituiti.
- I testi introduttivi molto lunghi, titoli tecnici estesi e cartigli lunghi richiedono migliore gestione dell'overflow. Le descrizioni delle opere proseguono in pagine supplementari.
- I PDF esportati non sono ancora archiviati come snapshot immutabili nell'app: si conservano nella cartella di esportazione.
- Una fotografia cancellata o un'opera eliminata dopo il salvataggio della bozza rende il relativo documento non esportabile finché non viene corretto.

## Sicurezza e privacy

- Tutti i dati restano locali, senza server o account.
- Export PDF nella cartella scelta, fuori dalla directory interna del database. Rifiuto di file symlink e di destinazioni non PDF.
- Percorsi immagini limitati alla sottocartella media; rifiuto di symlink e input eccessivamente grandi.
- Testi serializzati come caratteri PDF in esadecimale, mai interpretati come HTML o script.
- Validazione di template, duplicati, numero massimo di opere, identificativi e lunghezza dei contenuti.

## File coinvolti

| File | Responsabilità |
|---|---|
| src-tauri/src/brochures.rs | Bozze, dati locali, PDF |
| src-tauri/src/db.rs | Schema 15 |
| src-tauri/src/lib.rs | Comandi Tauri |
| src/app/brochures.component.* | Editor, fotografie, anteprima |
| src/app/models.ts | Modello documento |
| src/app/api.ts | Ponte nativo |
| src/app/i18n.ts | Localizzazioni |
| src/assets/icons/brochures.svg | Navigazione |

## Verifica manuale

1. Avviare npm run tauri:dev e avere due opere con fotografie in archivio.
2. Creare una brochure, aggiungere due opere e ordinarle; usare un layout diverso per ciascuna.
3. Modificare titolo e descrizione di una singola opera senza cambiare i dati archivistici.
4. Selezionare una fotografia aggiuntiva e verificare l'anteprima.
5. Salvare e riaprire la bozza. Esportare PDF; verificare pagina di copertina, eventuale introduzione, opere e contatti.
6. Provare descrizioni lunghe, accenti, riavvio dell'app e backup.
7. Provare a esportare con immagine mancante: l'app deve mostrare errore e non scrivere PDF parziali.

## Evoluzioni successive

1. Conservare snapshot storici PDF con hash e inclusione nel backup.
2. Portare tutti i template del Publication Engine, media multipli e doppie pagine.
3. Renderer comune per preview e PDF, font incorporati Unicode, controllo DPI/overflow.
4. Prezzi dell'opera e campi editoriali IT/EN per singolo elemento; export/import delle bozze.
