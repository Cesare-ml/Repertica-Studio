# REPS-UX-002 — Trattative, proposte e prenotazioni prima della vendita

- **Stato:** Proposta — da valutare
- **Data:** 2026-11-21
- **Filone:** Gestione e UX
- **Priorità proposta:** P1–P2, da confermare
- **Perimetro:** esclusivamente documentale; non autorizza implementazioni

## Contesto e novità

Il [MVP](../../MVP.md) gestisce collezionisti, opere e una vendita registrata per opera; le opere hanno già lo stato `reserved`. La ricerca collega le opere vendute ai collezionisti, ma non registra **più interessamenti nel tempo, offerte, decisioni e motivi di perdita**. Non è un duplicato della normale anagrafica o della registrazione vendita. Il tema *incassi effettivi/documenti fiscali* è già rappresentato dalla [issue #13](https://github.com/Cesare-ml/Repertica-Studio/issues/13) e resta escluso.

## Problema e destinatari

Per un artista o studio che vende direttamente, la vendita è l'ultimo passaggio di una trattativa. Senza un registro, conversazioni, prezzo proposto e impegni finiscono in note sparse, con rischio di doppie promesse sull'opera riservata o mancati ricontatti. Serve un sistema leggero, **non un CRM connesso** o una piattaforma marketing.

## Scenari concreti

1. Una collezionista chiede informazioni su un dipinto, riceve una proposta e desidera essere ricontattata la settimana seguente; l'artista annota interazione e promemoria locale.
2. Due persone si interessano alla stessa opera: una fa una proposta, l'altra ne chiede la prenotazione. L'interfaccia distingue trattativa aperta da prenotazione esclusiva.
3. Una proposta viene accettata e si registra una vendita nell'MVP: il riepilogo mantiene il prezzo e le condizioni concordate, ma non genera incassi, ricevute o fatture.

## Flussi UX candidati

- Da **Opera** o **Collezionista** → **Nuova trattativa**, collegando un'opera e un contatto esistenti; una modalità *contatto ancora sconosciuto* è opzionale, non obbligatoria in v1.
- Lista **Trattative** con colonne opera, contatto, fase (nuova, in discussione, proposta, riservata, conclusa con vendita, persa, annullata), prezzo proposto e prossima azione. Filtri per scadenza e stato.
- Scheda trattativa: timeline di interazioni inserite manualmente (telefonata, incontro, messaggio, visita), campo note, offerta e controfferta con data, termini, scadenza, eventuale acconto **solo come nota testuale finché gli incassi non sono supportati**.
- **Prenota opera**: richiesta di conferma, verifica di altre prenotazioni attive e confronto con stato `reserved` già presente. La prenotazione non deve convertire una semplice offerta in vendita.
- **Concludi come vendita**: precompilazione della schermata vendita esistente con esplicita conferma e validazioni native; annullare lascia la trattativa intatta. **Persa** permette motivo facoltativo e non altera la scheda opera.
- Dashboard minimale con attività scadute o da fare; nessuna notifica desktop o comunicazione automatica senza scelta esplicita futura.
- Accessibilità: stati etichettati con testo, navigazione tastiera fra cronologia e campi, conferme reversibili dove possibile.

## Dati richiesti (ipotesi, non schema definitivo)

- `deal`: UUID, opera UUID, collezionista UUID, fase, creato il, aggiornato il, prossima azione, termine di riserva eventuale, commenti, riferimento vendita UUID opzionale.
- `deal_offer`: UUID, trattativa, tipo (proposta/controproposta), importo in **centesimi interi EUR**, data, validità, termini non fiscali, esito (aperta/accettata/rifiutata/scaduta). Non usare float.
- `deal_activity`: UUID, tipo di interazione, data, descrizione, prossima azione facoltativa; niente contenuto email o sincronizzazioni raccolte automaticamente.
- Storico dei passaggi di fase e delle prenotazioni; una sola **prenotazione esclusiva attiva** per opera in un dato momento. Più trattative informative possono coesistere.
- Non trasformare lo stato commerciale `reserved` preesistente senza una regola di migrazione/riconciliazione chiara; prevedere il caso di opere già riservate senza trattative.
- La vendita rimane l'unica fonte attendibile di ricavi registrati; una trattativa non entra nelle statistiche economiche.

## Impostazioni, offline, lingue e portabilità

- **Impostazioni:** fasi predefinite, durata suggerita delle prenotazioni e motivi di perdita precompilati **solo opzionali**; nessun profilo utente obbligatorio.
- **Offline:** dati e promemoria consultabili da locale, senza calendario cloud, rete, telemetria, mail, SMS o push; quando l'app è chiusa non promettere avvisi.
- **Lingue:** usare locale OS e fallback inglese; traduzioni coerenti in IT/EN/ES/FR/DE, inclusi motivi/etichette predefinite. Etichette libere restano nella lingua scritta dall'utente.
- **Import/export:** CSV di trattative e attività per consultazione, JSON versionato per portabilità con relazioni UUID; import in anteprima che rispetti vincoli dei contatti e non crei vendite. Backup locali già esistenti devono comprendere i nuovi dati.
- **Privacy:** storico negoziazioni e recapiti sono sensibili; nessun profilo commerciale automatico, deduzione di preferenze, email tracking o contatti non autorizzati. Export selettivo senza note private; dichiarare sempre quando i dati vengono condivisi manualmente.

## Complessità, dipendenze e alternative

- **Complessità:** media, ma alta nell'allineamento delle prenotazioni con `artworks.status` e con le vendite.
- **Dipendenze:** UUID opere/contatti, vendite uniche per opera, i18n locale e migrazioni SQLite; nessuna dipendenza da server.
- **Rischi:** conflitti su riserve, falsa rappresentazione di un'offerta come pagamento, conversioni duplicate, cronologia privacy, scadenze interpretate erroneamente.
- **Mitigazioni:** terminologia **trattativa ≠ prenotazione ≠ vendita ≠ incasso**; transizioni e controlli espliciti; niente somme di offerte nei ricavi; record non cancellati silenziosamente.
- **Alternativa più semplice:** solo attività e promemoria locali su opera/collezionista, rimandando prezzi, offerte e prenotazioni; utile se il flusso commerciale reale è raro.

## Criteri di accettazione ipotetici

1. Due trattative per un'opera possono coesistere, ma non due prenotazioni esclusive contemporanee.
2. Una proposta non modifica i totali mensili, la vendita o il certificato; la conversione richiede conferma esplicita.
3. Una vendita esistente blocca nuova conversione duplicata, secondo le regole del [MVP](../../MVP.md).
4. Chiudere una trattativa come persa conserva cronologia ma libera solo l'eventuale **propria** prenotazione, non quella di terzi.
5. Con app interamente offline sono accessibili attività scadute e promemoria; nessuna promessa di notifiche quando è chiusa.
6. Export/import di prova conserva collegamenti opera/contatto e impedisce offerte orfane.
7. Tutti i passaggi sono accessibili da tastiera e i messaggi di conflitto sono localizzati.

## Sviluppo per fasi (solo progettazione)

- **Fase A:** attività, promemoria locali e stati trattativa senza prenotazione esclusiva.
- **Fase B:** proposte economiche e gestione prenotazioni con migrazione degli stati legacy.
- **Fase C:** passaggio guidato a vendita, esportazione strutturata e report conversione (senza dati fiscali).

## Certezze, ipotesi, domande

- **Certo:** collezionisti, opere, vendite e `reserved` esistono; offline e niente account; valuta delle vendite attualmente EUR; ricavi non sono incassi.
- **Ipotesi:** l'artista segue trattative parallele e ha bisogno di promemoria; riserve con scadenza sono frequenti.
- **Da decidere:** una prenotazione deve sempre avere un collezionista? Il campo `reserved` va sincronizzato o mantenuto compatibile con prenotazioni manuali? Quanto dettaglio commerciale è utile senza trasformare l'app in CRM pesante?

**Decisione attuale:** nessuna approvazione o assegnazione allo sviluppo.
