-- Explicit local copy only; no writes, no guessed historical finance splits.
BEGIN READ ONLY;
SET LOCAL statement_timeout = '5s';
WITH requested(id) AS (
  SELECT unnest(:'artwork_ids'::integer[])
), eligible AS (
  SELECT o.id, o.codice_archivio, o.titolo, o.anno_realizzazione, o.tecnica, o.supporto,
    CAST(o.larghezza_cm AS text) AS larghezza_cm,
    CAST(o.altezza_cm AS text) AS altezza_cm,
    CAST(o.profondita_cm AS text) AS profondita_cm,
    o.descrizione, CAST(o.stato AS text) AS stato, o.firma_presente, o.cornice_presente,
    CAST(o.vernice_finale AS text) AS vernice_finale,
    CAST(o.coefficiente_artista_override AS text) AS coefficiente_artista_override,
    CAST(o.prezzo_personalizzato AS text) AS prezzo_personalizzato
  FROM requested r JOIN public.opere o ON o.id=r.id
  WHERE o.id_utente_proprietario=CAST(:'owner_id' AS integer)
    AND o.stato IN ('DISPONIBILE','RISERVATA','VENDUTA')
    AND o.firma_presente IS NOT NULL AND o.cornice_presente IS NOT NULL
    AND o.vernice_finale IN ('OPACA','LUCIDA','SATINATA')
), projected AS (
  SELECT e.codice_archivio,
    (to_jsonb(e)-'id') || jsonb_build_object('id_vendita',NULL) AS artwork,
    CASE WHEN v.id IS NULL THEN NULL ELSE jsonb_build_object(
      'collector_key',CAST(c.id AS text),'first_name',c.nome,'last_name',c.cognome,
      'email',c.email,'phone',c.telefono,'address',c.indirizzo,'notes',c.note,
      'birth_date',to_char(c.data_nascita,'YYYY-MM-DD'),'birth_time',CAST(c.ora_nascita AS text),
      'birth_place',c.luogo_nascita,'instagram',c.instagram,'facebook',c.facebook,
      'tiktok',c.tiktok,'website',c.sito_web,'linkedin',c.linkedin) END AS collector,
    CASE WHEN v.id IS NULL THEN NULL ELSE jsonb_build_object(
      'amount',CAST(v.prezzo_vendita AS text),'tax',CAST(v.tasse AS text),
      'shipping',CAST(v.costi_spedizione AS text),'sold_at',to_char(v.data_ora_vendita,'YYYY-MM-DD'),
      'notes',v.note,'receipts',COALESCE((
        SELECT jsonb_agg(jsonb_build_object('amount',CAST(i.importo AS text),
          'received_at',to_char(i.data_incasso,'YYYY-MM-DD'),'method',i.metodo_pagamento,
          'notes',i.note) ORDER BY i.data_incasso,i.id)
        FROM public.vendite_incassi i WHERE i.id_vendita=v.id
      ),'[]'::jsonb),
      'installments',COALESCE((
        SELECT jsonb_agg(jsonb_build_object('position',sc.numero,'amount',CAST(sc.importo_previsto AS text),
          'due_date',to_char(sc.data_scadenza,'YYYY-MM-DD'),'notes',sc.note) ORDER BY sc.numero,sc.id)
        FROM public.vendite_scadenze sc WHERE sc.id_vendita=v.id
      ),'[]'::jsonb)) END AS sale
  FROM eligible e
  LEFT JOIN public.vendite v ON v.id_opera=e.id
  LEFT JOIN public.clienti c ON c.id=v.id_cliente
  WHERE (e.stato IN ('DISPONIBILE','RISERVATA') AND v.id IS NULL)
     OR (e.stato='VENDUTA' AND v.id IS NOT NULL
        AND (SELECT count(*) FROM public.vendite all_v WHERE all_v.id_opera=e.id)=1
        AND v.id_utente_proprietario=CAST(:'owner_id' AS integer)
        AND c.id_utente_proprietario=CAST(:'owner_id' AS integer)
        AND v.tipo_movimento='VENDITA'
        AND v.orario_vendita_conosciuto IS FALSE
        AND NULLIF(trim(v.metodo_pagamento),'') IS NULL
        AND NULLIF(trim(v.canale_vendita),'') IS NULL
        AND NULLIF(trim(v.campagna),'') IS NULL
        AND c.percorso_foto IS NULL
        AND NOT EXISTS(SELECT 1 FROM public.immagini_cliente ic WHERE ic.id_cliente=c.id)
        AND (NOT EXISTS(SELECT 1 FROM public.vendite_scadenze sc WHERE sc.id_vendita=v.id)
             OR NOT EXISTS(SELECT 1 FROM public.vendite_incassi i WHERE i.id_vendita=v.id))
        AND NOT EXISTS(SELECT 1 FROM public.vendite_incassi i WHERE i.id_vendita=v.id AND i.id_scadenza IS NOT NULL))
), guarded AS (
  SELECT artwork,collector,sale FROM projected
  WHERE (SELECT count(*) FROM requested) BETWEEN 1 AND 1000
    AND (SELECT count(DISTINCT id) FROM requested)=(SELECT count(*) FROM requested)
    AND (SELECT count(*) FROM projected)=(SELECT count(*) FROM requested)
)
SELECT COALESCE(json_agg(row_to_json(guarded)),'[]'::json) FROM guarded;
COMMIT;
