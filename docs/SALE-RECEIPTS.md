# Registrazione degli incassi — SQLite v4

Una vendita **non** implica che l'importo sia stato ricevuto: le registrazioni
`sale_receipts` sono manuali e distinte dal fatturato `sales`. Nessuna
vendita storica viene convertita automaticamente in un incasso.

Ogni incasso ha UUID, vendita collegata, data ISO, importo in centesimi EUR,
metodo (bonifico, contanti, carta, altro), riferimento e note facoltativi.
Una vendita può avere incassi parziali multipli. Il totale non può superare
l'importo lordo della vendita; l'invariante è verificata sia nel comando
Rust sia nei trigger SQLite. Non si può eliminare una vendita con incassi
registrati; eliminare prima gli incassi se la vendita è realmente annullata.

La migrazione SQLite v3→v4 è additiva e transazionale. I backup completi
(SQLite + media + manifest SHA-256) accettano schemi 1–4 e comprendono
automaticamente gli incassi. Le vecchie versioni dell'app non aprono v4.
Il CSV delle vendite **non** è un estratto degli incassi; rimane un
registro del fatturato. L'esportazione CSV `receipts` è ora disponibile dalla sezione Vendite, con
identificativi di vendita, opera e collezionista, data, importo esatto in
centesimi, metodo e riferimenti; include tutti gli incassi senza filtri.
Non equivale a estratto conto bancario, documento fiscale o backup
ripristinabile.

La pagina **Economia** espone ora un riepilogo aggiuntivo degli **incassi dichiarati
per mese**, con dodici mesi, importi in centesimi e conteggio delle operazioni,
raggruppati per data `received_at` e **non** per data della vendita. Il riepilogo
include anche i mesi senza incassi (zero), non interpreta vendite come pagamenti e
non include spese come se fossero pagate. È un rapporto delle sole registrazioni
manuali, non un estratto conto né un saldo di cassa verificato.

Comandi Tauri: `list_sale_receipts`, `record_sale_receipt`,
`delete_sale_receipt`, `list_sale_balances`, `receipt_year_overview`.
Le cifre indicano incassi **dichiarati dall'utente**, non verificati con
banche né documenti fiscali. Nessun collegamento online.

Verifiche previste: `cargo test --manifest-path src-tauri/Cargo.toml`
e `npm run build`. Collaudo desktop reale ancora necessario.
