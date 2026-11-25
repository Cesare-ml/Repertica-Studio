# Registro delle idee Repertica — 24 novembre 2026

Questo documento consolida **idee e requisiti emersi nella pianificazione**, non certifica che siano implementati. Le vecchie proposte delle automazioni in `docs/idee/gestione-e-ux/` e `docs/idee/strumenti-e-architettura/`, se presenti su branch/PR, vanno censite e deduplicate prima di qualsiasi implementazione; non eliminare quelle esistenti.

## Decisioni di prodotto confermate
- Desktop gratuito, open source, completamente locale e offline senza account obbligatorio; backup in cartella scelta ed export completo.
- Studio Online utilizzabile anche solo da smartphone, con archivio cloud, piano gratuito con quote media e possibile piano a pagamento; superamento quota senza cancellazione automatica, lettura/export garantiti.
- Sincronizzazione Desktop/Online **opt-in**, non prerequisito per Auctions; identificatori stabili, delta, versioni, conflitti e coda offline.
- Un unico account Repertica per Studio Online, Auctions e Sites con OAuth Authorization Code + PKCE; nessun riuso di account o chiavi VirgoStars.
- Auctions: una singola opera caricabile via web anche senza archivio preesistente; form e API condivisi con Studio Online, nessun archivio duplicato.
- Auctions gestibile completamente da web o Desktop con un solo runtime server-side; pubblicazione selettiva della sola opera, senza sincronizzare il resto dell'archivio.
- Offerte atomiche, anti-sniping, email verificata per offerenti senza account obbligatorio, notifiche, audit, separazione tra aggiudicazione e vendita effettivamente conclusa.
- Protezione assoluta dei tre account e delle opere/immagini del pilot Studio Online: nessun cutover senza parità funzionale, backup, test e rollback.

## Idee evolutive da valutare dopo l'MVP
1. Import/export interoperabile e formati aperti per archivi di grandi dimensioni.
2. Backup versionati e verifica periodica del ripristino, con NAS/dischi esterni.
3. Ricerca avanzata, filtri, provenienza e cronologia modifiche delle opere.
4. Media center per fotografie, RAW, duplicati, varianti non distruttive e quote storage trasparenti.
5. Statistiche mensili, coefficienti artista, materiali, spese e ripartizioni dei guadagni.
6. MCP locale opzionale, con autorizzazioni granulari e nessuna esposizione di dati senza consenso.
7. Strumenti per gallerie, collaborazioni e ruoli multiutente, senza compromettere la privacy degli archivi locali.
8. Migliorie di accessibilità, localizzazione e UX mobile.

Queste idee sono **proposte, non incarichi approvati automaticamente**. Prima di promuoverle a sviluppo verificare README, docs e PR per evitare duplicazioni o funzioni già realizzate.

## Priorità attuale
**P0:** MVP Auctions completo e sicuro, API opere condivise, SSO, parità web/Desktop, test E2E. **P1:** parità funzionale e sicurezza Studio Online. **P2:** sincronizzazione opzionale e piano cloud. **P3:** evoluzioni sopra, dopo revisione proprietario.

## Fonti operative
- [Auctions MVP brief](https://github.com/Cesare-ml/Repertica-auctions/blob/main/docs/AGENT_MVP_BRIEF.md)
- [Studio Online — architettura](https://github.com/Cesare-ml/Repertica-Studio-Online/issues/4)
- [Auctions — opera singola](https://github.com/Cesare-ml/Repertica-auctions/issues/3)

## Protocollo aggiornamento
Le future automazioni di sviluppo **non devono generare continuamente nuove idee**. Se scoprono un miglioramento pertinente, aggiornano questo registro con fonte, stato, dipendenze e priorità proposta in PR documentale, senza interrompere lo sviluppo MVP. Le proposte esistenti su altri branch vanno riportate senza perdita di dettaglio.
